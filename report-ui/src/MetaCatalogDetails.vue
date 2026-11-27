<script setup lang="ts">
import { computed } from "vue";
import { entityOverview } from "@seikika/ui/entity";
import EntityPage from "@seikika/ui/components/EntityPage.vue";
import SectionCard from "@seikika/ui/components/SectionCard.vue";
import CatalogEntityContext from "./CatalogEntityContext.vue";
import CatalogEntityFacts from "./CatalogEntityFacts.vue";
import ReportContent from "./ReportContent.vue";
import { entitySource } from "./lib/federation";
import { kindTitle } from "./lib/kind-title";
import { title } from "./lib/rings";
import { catalogEntityPath } from "./lib/routes";
import type { Summary } from "./lib/summary";
import { buildView } from "./report-view";
import type { CatalogReport } from "./types";

const props = defineProps<{ data: CatalogReport; summary: Summary; id: string; showScores?: boolean }>();
const kind = computed(() => props.summary.kindOf[props.id]);
const overview = computed(() => entityOverview(props.id, kind.value, {
  ...props.data.entities?.[props.id]?.fields, ...props.data.entityOverview?.[props.id],
}));
const owner = computed(() => overview.value.owner);
const original = computed(() => props.data.entitySources?.[props.id]?.original ?? props.id);
const scores = computed(() => Object.keys(props.data.scorecards).filter(name => props.data.scores[name]?.[props.id]));
const facts = computed(() => props.data.entities?.[props.id]);
const hasFields = computed(() => Boolean(facts.value && Object.keys(facts.value.fields).length));
const hasRelations = computed(() => Boolean(facts.value?.relations?.in.length || facts.value?.relations?.out.length));
const offers = computed(() => props.data.actions?.some(action => action.entities.includes(props.id)));
function scoringDetails() {
  const content = buildView(props.data, props.summary, { mode: "entity", id: props.id });
  content.querySelector(":scope > .crumb")?.remove();
  const heading = content.querySelector(":scope > .ehead");
  const level = heading?.querySelector(":scope > .elevel");
  if (heading) heading.replaceWith(...(level ? [level] : []));
  return content;
}
</script>

<template>
  <div class="report-entity-details report-catalog-details">
    <SectionCard title="Catalog details">
      <EntityPage :entity="overview" :owner-href="owner && summary.kindOf[owner] ? '#' + catalogEntityPath(data, owner) : undefined">
        <template #header>
          <a :href="'#catalog/' + encodeURIComponent(kind)">Browse {{ kindTitle(data, kind) }} in Catalog</a>
          <p><code>{{ original }}</code>
            <span v-if="facts && data.snapshot != null" class="sub asof"> · as of snapshot <code>{{ data.snapshot }}</code></span>
          </p>
          <p v-if="entitySource(data, id)" class="sub">{{ entitySource(data, id) }}</p>
          <p v-if="overview.description">{{ overview.description }}</p>
        </template>
      </EntityPage>
      <a v-if="offers" :href="'#actions?entity=' + encodeURIComponent(id)">Preview actions for this entity</a>
    </SectionCard>
    <slot v-if="facts" name="entity-tools" :id="id" :source="data.entitySources?.[id]?.source" :snapshot="data.snapshot" />
    <CatalogEntityContext :id="id" :data="data" />
    <SectionCard v-if="showScores && scores.length" class="report-catalog-scores"
      :title="kind === 'Scorecard' ? 'How this scorecard is evaluated' : 'Catalog scorecards'">
      <ul>
        <li v-for="name in scores" :key="name">
          <a :href="'#' + (data.comparisons?.[name] ? 'compare/' : 'scorecard/') + encodeURIComponent(name)">{{ title(data, name) }}</a>
        </li>
      </ul>
      <details :key="id" class="report-catalog-disclosure">
        <summary>Show rule results</summary>
        <ReportContent :build="scoringDetails" />
      </details>
    </SectionCard>
    <details v-if="hasFields || hasRelations" :key="id" class="seikika-section report-catalog-disclosure">
      <summary>{{ hasFields && hasRelations ? 'Catalog fields and relationships' : hasFields ? 'Catalog fields' : 'Catalog relationships' }}</summary>
      <CatalogEntityFacts :data="data" :summary="summary" :id="id" />
    </details>
  </div>
</template>
