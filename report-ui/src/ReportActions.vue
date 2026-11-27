<script setup lang="ts">
import { computed, ref, watch } from "vue";
import SectionCard from "@seikika/ui/components/SectionCard.vue";
import type { ActionRequest } from "./action-request";
import type { ActionInput, ReportAction, ReportData } from "./types";
import { sourceLabel } from "./lib/federation";

const props = defineProps<{ data: ReportData; id?: string; entity?: string; request?: ActionRequest }>();
const emit = defineEmits<{ select: [id: string] }>();
const chosen = ref<number>(0);
const target = ref("");
const inputs = ref<Record<string, string | boolean>>(Object.create(null));
const preview = ref<{
  effects?: unknown; warnings?: string[]; blockers?: string[];
  approval?: unknown; confirm_token?: string; executor_preview?: boolean;
}>();
const result = ref("");
const error = ref("");
const busy = ref(false);
let revision = 0;
const actions = computed(() => (props.data.actions ?? []).filter(action => !props.entity || action.entities.includes(props.entity)));
const action = computed(() => actions.value[chosen.value]);
const live = computed(() => Boolean(props.request && action.value && !action.value.previewOnly));
const properties = computed(() => Object.entries(action.value?.inputs?.properties ?? {}));

watch([actions, () => props.id], () => {
  chosen.value = props.id ? actions.value.findIndex(action => action.catalogId === props.id) : 0;
}, { immediate: true });
watch([action, () => props.entity], () => {
  target.value = props.entity ?? action.value?.entities[0] ?? "";
  inputs.value = Object.assign(Object.create(null), Object.fromEntries(properties.value.map(([name, item]) => {
    const value = item.default;
    return [name, item.type === "boolean" && !item.enum?.length ? value === true : value == null
      ? "" : typeof value === "object" ? JSON.stringify(value) : String(value)];
  })));
}, { immediate: true });
watch([action, target, inputs], () => {
  revision++;
  preview.value = undefined;
  result.value = "";
  error.value = "";
  busy.value = false;
}, { deep: true });

function title(item: ReportAction) {
  const source = sourceLabel(props.data, item.source) || item.portal?.name;
  return item.name + (source ? " · " + source : "");
}

function selectAction(event: Event) {
  const { value } = event.target as HTMLSelectElement;
  const id = actions.value[Number(value)]?.catalogId;
  if (id) emit("select", id);
}

function label(id: string) {
  return props.data.entitySources?.[id]?.original ?? id;
}

function options(name: string, input: ActionInput) {
  return action.value?.choices?.[name]?.[target.value] ?? input.enum;
}

function value(input: ActionInput, raw: string | boolean) {
  if (input.type === "boolean") return raw === true || raw === "true";
  if (input.type === "integer" || input.type === "number") {
    const n = Number(raw);
    if (!Number.isFinite(n) || input.type === "integer" && !Number.isInteger(n)) throw new Error("Enter a valid number.");
    return n;
  }
  if (input.type === "array" || input.type === "object") {
    const parsed = JSON.parse(String(raw));
    if (input.type === "array" ? !Array.isArray(parsed) : !parsed || Array.isArray(parsed) || typeof parsed !== "object") {
      throw new Error("Enter a JSON " + input.type + ".");
    }
    return parsed;
  }
  return raw;
}

function inputsFor(item: ReportAction) {
  const values: Record<string, unknown> = Object.create(null);
  for (const [name, input] of properties.value) {
    const raw = inputs.value[name];
    if (raw === "" || raw === undefined) {
      if (item.inputs?.required?.includes(name)) throw new Error((input.title ?? name) + " is required.");
    } else {
      const parsed = value(input, raw);
      const allowed = options(name, input);
      if (allowed && !allowed.some(option => option === parsed)) {
        throw new Error("Choose an allowed value for " + (input.title ?? name) + ".");
      }
      values[name] = parsed;
    }
  }
  return values;
}

function show(v: unknown) {
  return typeof v === "string" ? v : JSON.stringify(v, null, 2);
}

async function submit(dispatch: boolean) {
  const item = action.value;
  if (!item || busy.value || (dispatch && (!live.value || !preview.value?.confirm_token))) return;
  const id = ++revision;
  error.value = "";
  result.value = "";
  busy.value = true;
  try {
    const parameters = inputsFor(item);
    if (!live.value || !props.request) {
      preview.value = { effects: { action: item.name, target: target.value && label(target.value), inputs: parameters, executor: item.executor },
        approval: item.approval, executor_preview: false };
      return;
    }
    const uri = "/v1/actions/" + encodeURIComponent(item.name) + (dispatch ? "/runs" : "/previews");
    const response = await props.request(uri, { method: "POST", body: JSON.stringify({
      entity: item.target || target.value || null, inputs: parameters,
      ...(dispatch ? { confirm_token: preview.value?.confirm_token } : {}),
    }) });
    if (id !== revision) return;
    const body = response.body as { detail?: string; id?: string; state?: string } & NonNullable<typeof preview.value>;
    if (response.status >= 400) throw new Error(body?.detail || "Action request failed.");
    if (dispatch) {
      result.value = [body?.id, body?.state].filter(Boolean).join(" · ") || "Action submitted.";
      preview.value = undefined;
    } else {
      preview.value = body;
    }
  } catch (reason) {
    if (id === revision) error.value = reason instanceof Error ? reason.message : String(reason);
  } finally {
    if (id === revision) busy.value = false;
  }
}
</script>

<template>
  <section class="report-actions">
    <h2>Actions</h2>
    <p v-if="!actions.length" class="sub">No actions are available for this selection.</p>
    <template v-else>
      <label class="report-field">Action
        <select v-model.number="chosen" @change="selectAction">
          <option v-if="!action" :value="-1" disabled>Choose an action</option>
          <option v-for="(item, index) in actions" :key="index" :value="index">{{ title(item) }}</option>
        </select>
      </label>
      <p v-if="!action" class="empty">This action is unavailable in this snapshot.</p>
      <SectionCard v-else :title="action.name" :description="action.description">
        <p class="sub">{{ sourceLabel(data, action.source) || action.portal?.name || "Local portal" }} · {{ action.executor }}</p>
        <form @submit.prevent="submit(false)">
          <label v-if="action.entities.length" class="report-field">Target
            <select v-model="target">
              <option v-for="id in action.entities" :key="id" :value="id">{{ label(id) }}</option>
            </select>
          </label>
          <label v-for="[name, input] in properties" :key="name" class="report-field">
            {{ input.title ?? name }}<template v-if="action.inputs?.required?.includes(name)"> (required)</template>
            <select v-if="options(name, input)" v-model="inputs[name]">
              <option value="">Select a value</option>
              <option v-for="option in options(name, input)" :key="String(option)" :value="String(option)">{{ option }}</option>
            </select>
            <input v-else-if="input.type === 'boolean'" v-model="inputs[name]" type="checkbox" />
            <textarea v-else-if="input.type === 'array' || input.type === 'object'" v-model="inputs[name] as string"
              :placeholder="input.type === 'array' ? '[]' : '{}'" rows="3"></textarea>
            <input v-else v-model="inputs[name]" :type="input.type === 'integer' || input.type === 'number' ? 'number' : 'text'"
              :step="input.type === 'integer' ? '1' : 'any'" />
            <small v-if="input.description" class="sub">{{ input.description }}</small>
          </label>
          <div class="report-filters">
            <button type="submit" :disabled="busy">Preview action</button>
            <button type="button" :disabled="!live || busy || !preview?.confirm_token" @click="submit(true)">Dispatch action</button>
          </div>
          <p v-if="!live" class="sub">Preview only. Action dispatch requires a running portal.</p>
        </form>
        <p v-if="error" role="alert">{{ error }}</p>
        <p v-if="result" role="status">{{ result }}</p>
        <div v-if="preview" aria-live="polite">
          <h3>Dispatch preview</h3>
          <p v-if="!live" class="sub">This preview shows the submitted inputs and executor.</p>
          <ul v-if="preview.blockers?.length || preview.warnings?.length">
            <li v-for="(line, index) in [...(preview.blockers ?? []), ...(preview.warnings ?? [])]" :key="index">{{ line }}</li>
          </ul>
          <pre v-if="preview.effects != null">{{ show(preview.effects) }}</pre>
          <p v-if="preview.approval">Approval <code>{{ show(preview.approval) }}</code></p>
        </div>
      </SectionCard>
      <slot v-if="action?.catalogId" :id="action.catalogId" />
    </template>
  </section>
</template>
