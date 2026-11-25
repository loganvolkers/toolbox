<script setup lang="ts">
import DataValue from "./DataValue.vue";
defineProps<{ level: unknown }>();
</script>

<template>
  <DataValue v-if="level !== null && typeof level === 'object'" :value="level" />
  <span v-else class="tb-level" :data-level="String(level ?? 'none')">{{ level ?? "none" }}</span>
</template>
