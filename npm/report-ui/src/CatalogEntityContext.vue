<script setup lang="ts">
import { computed } from "vue";
import SectionCard from "@seikika/ui/components/SectionCard.vue";
import { trackPath } from "./lib/routes";
import type { ReportData } from "./types";

const props = defineProps<{ data: ReportData; id: string }>();
const campaigns = computed(() => Object.entries(props.data.campaigns ?? {}).filter(([, campaign]) =>
  campaign.entities?.[props.id]));
const tracks = computed(() => {
  const scorecardTracks = Object.entries(props.data.scorecards).flatMap(([name, card]) =>
    props.data.scores[name]?.[props.id] && card.track ? [card.track] : []);
  const names = new Set([...scorecardTracks, ...campaigns.value.map(([, campaign]) => campaign.track)]);
  return [...names].filter((name): name is string => Boolean(name && props.data.tracks?.[name]));
});
</script>

<template>
  <SectionCard v-if="campaigns.length || tracks.length" title="Campaigns and tracks" class="entity-context-links">
    <ul>
      <li v-for="[name, campaign] in campaigns" :key="'campaign:' + name">
        <a :href="'#campaign/' + encodeURIComponent(name)">{{ campaign.title || name }}</a>
        <small class="sub"> · {{ campaign.entities?.[id]?.meeting ? 'Meets goal' : 'Below goal' }}</small>
      </li>
      <li v-for="name in tracks" :key="'track:' + name">
        <a :href="'#' + trackPath(data, name)">{{ data.tracks?.[name]?.title || name }}</a>
      </li>
    </ul>
  </SectionCard>
</template>
