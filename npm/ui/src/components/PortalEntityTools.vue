<script setup lang="ts">
import { computed, onUnmounted, ref, watch } from "vue";
import DataValue from "@seikika/ui/components/DataValue.vue";
import SectionCard from "@seikika/ui/components/SectionCard.vue";
import { usePortalApi, usePortalRequest } from "../requests";

const props = defineProps<{ id: string; snapshot?: string | number | null }>();
const emit = defineEmits<{ refreshed: [] }>();
const api = usePortalApi();
const entityPath = computed(() => "/v1/entities/" + encodeURIComponent(props.id));
const { result: detail, reload } = usePortalRequest(() => "/v1/views/entity/" + encodeURIComponent(props.id));
const { result: explanation, reload: reloadExplanation } = usePortalRequest(() => entityPath.value + "/explain?field=ownership");
const current = computed(() => detail.value?.status === 200 && detail.value.snapshot === props.snapshot ? detail.value.body : null);
const restricted = computed(() => Object.entries(current.value?.scorecards ?? {})
  .map(([name, score]) => ({ name, level: (score as { level?: { redacted?: boolean } }).level }))
  .filter(score => score.level?.redacted));
const ownership = computed(() => explanation.value?.status === 200 && explanation.value.snapshot === props.snapshot
  ? explanation.value.body?.authority : null);

watch(() => props.snapshot, () => { reload(); reloadExplanation(); });

const running = ref(false);
const failed = ref(false);
const message = ref("");
let refreshController: AbortController | undefined;
async function refresh() {
  if (running.value) return;
  refreshController = new AbortController();
  const { signal } = refreshController;
  running.value = true;
  failed.value = false;
  message.value = "Asking the connectors to re-read this entity…";
  try {
    const request = await api(entityPath.value + "/refresh", { method: "POST", signal });
    if (request.status !== 202) throw new Error(request.body?.detail ?? "HTTP " + request.status);
    let snapshot: string | null = null;
    for (const id of request.body.commands as string[]) {
      const command = await api("/v1/commands/" + encodeURIComponent(id) + "?wait=30", { signal });
      if (command.body?.status !== "succeeded") throw new Error("Refresh " + (command.body?.status ?? "failed"));
      snapshot = command.body.snapshot ?? snapshot;
    }
    if (signal.aborted) return;
    emit("refreshed");
    message.value = snapshot ? "Refreshed at snapshot " + snapshot : "Refreshed";
  } catch (error) {
    if (!signal.aborted) {
      failed.value = true;
      message.value = (error as Error).message;
    }
  } finally {
    running.value = false;
  }
}
onUnmounted(() => refreshController?.abort());
</script>

<template>
  <div class="portal-entity-tools">
    <div class="portal-tools">
      <button type="button" :disabled="running" :aria-busy="running" @click="refresh">Refresh</button>
      <span class="sub" :class="{ 'portal-failure': failed }" role="status">{{ message }}</span>
    </div>
    <p v-if="ownership" class="sub">
      <strong>Ownership · {{ ownership.status }}</strong> {{ ownership.explanation }}
      <em v-if="ownership.fix">{{ ownership.fix }}</em>
    </p>
    <ul v-if="restricted.length">
      <li v-for="{ name, level } in restricted" :key="name">
        <a :href="'#scorecard/' + encodeURIComponent(name)">{{ name }}</a> · <DataValue :value="level" />
      </li>
    </ul>
    <details v-if="current?.actions?.length || current?.freshness?.length">
      <summary>Connector freshness and action access</summary>
      <SectionCard v-if="current.freshness.length" title="Freshness" heading="h3">
        <ul>
          <li v-for="source in current.freshness" :key="source.connector + '/' + source.kind">{{ source.connector }}/{{ source.kind }} · {{ source.last_read ?? 'never read by a connector' }}</li>
        </ul>
      </SectionCard>
      <SectionCard v-if="current.actions.length" title="Action access" heading="h3">
        <ul>
          <li v-for="action in current.actions" :key="action.name">
            <strong>{{ action.name }}</strong> {{ action.description }}
            <span class="sub"> · {{ action.allowed ? 'Available' : 'Needs ' + action.permission }}</span>
          </li>
        </ul>
      </SectionCard>
    </details>
  </div>
</template>
