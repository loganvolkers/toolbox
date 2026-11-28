<script setup lang="ts">
import { computed } from "vue";
/** Displays a catalog value, including redaction markers and entity references. */
interface Props {
  /** Value to display. Nullish values show a dash, and objects should be JSON-serializable. */
  value: unknown;
  /** Builds a URL for entity references. Omit it to display those references as text. */
  entityHref?: (id: string) => string;
}
const props = defineProps<Props>();
const redaction = computed(() => {
  const value = props.value as { redacted?: boolean; reason?: string; requires?: string; visible_to?: string } | null;
  return value?.redacted ? value : null;
});
const reference = computed(() => typeof props.value === "string" && /^[a-z][a-z0-9-]*:(?!\/\/)[^\s]+$/i.test(props.value)
  ? props.entityHref?.(props.value)
  : undefined);
</script>

<template>
  <span v-if="redaction" class="redacted" :title="redaction.requires && 'Needs ' + redaction.requires">
    🔒 {{ redaction.visible_to ? 'Visible to ' + redaction.visible_to : redaction.reason ?? 'Permission required' }}
  </span>
  <span v-else-if="value == null" class="muted">—</span>
  <a v-else-if="reference" :href="reference">{{ value }}</a>
  <template v-else-if="Array.isArray(value)">
    <span v-if="!value.length" class="muted">none</span>
    <template v-for="(item, index) in value" :key="index"><template v-if="index">, </template><DataValue :value="item" :entity-href="entityHref" /></template>
  </template>
  <code v-else-if="typeof value === 'object'">{{ JSON.stringify(value) }}</code>
  <span v-else-if="typeof value === 'boolean'">{{ value ? "yes" : "no" }}</span>
  <span v-else>{{ String(value) }}</span>
</template>
