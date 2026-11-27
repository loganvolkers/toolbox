<script setup lang="ts">
import { computed, watch } from "vue";
import LevelBadge from "@seikika/ui/components/LevelBadge.vue";
import SectionCard from "@seikika/ui/components/SectionCard.vue";
import { usePortalRequest } from "../requests";

const props = defineProps<{ visibleEntities: Record<string, string>; snapshot?: string | number | null }>();
const { result, reload } = usePortalRequest(() => "/v1/views/home");
type Owned = { id: string; levels: Record<string, unknown>; failing: string[] };
const owned = computed(() => {
  if (result.value?.status !== 200 || result.value.snapshot !== props.snapshot) return;
  return (result.value.body.owned as Owned[]).filter(item => Object.hasOwn(props.visibleEntities, item.id));
});
watch(() => props.snapshot, reload);
</script>

<template>
  <SectionCard v-if="owned" title="Your teams’ entities">
    <p v-if="!owned.length" class="sub">Your teams have no visible owned entities yet.</p>
    <table v-else class="portal-table">
      <thead><tr><th>Entity</th><th>Levels</th><th>Failing rules</th></tr></thead>
      <tbody>
        <tr v-for="entity in owned" :key="entity.id">
          <td><a :href="'#entity/' + encodeURIComponent(entity.id)">{{ entity.id }}</a></td>
          <td><div v-for="[name, level] in Object.entries(entity.levels)" :key="name">
            {{ name }} <LevelBadge v-if="typeof level === 'string'" :level="level" />
          </div></td>
          <td>{{ entity.failing.join(', ') || 'None' }}</td>
        </tr>
      </tbody>
    </table>
  </SectionCard>
</template>
