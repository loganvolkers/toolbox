<script setup lang="ts">
import DataValue from "./DataValue.vue";
/** Displays a scorecard level or a catalog redaction marker. */
interface Props {
  /** Level such as `bronze` or `gold`. Nullish values display `none`. Objects use `DataValue`. */
  level: unknown;
}
defineProps<Props>();
</script>

<template>
  <DataValue v-if="level !== null && typeof level === 'object'" :value="level" />
  <span v-else class="seikika-level" :data-level="String(level ?? 'none')">{{ level ?? "none" }}</span>
</template>
