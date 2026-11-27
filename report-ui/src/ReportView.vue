<script setup lang="ts">
import { computed, shallowRef, watchPostEffect } from "vue";
import DataValue from "@seikika/ui/components/DataValue.vue";
import SectionCard from "@seikika/ui/components/SectionCard.vue";
import { entitySource } from "./lib/federation";
import EntityPage from "@seikika/ui/components/EntityPage.vue";
import ScorecardPage from "@seikika/ui/components/ScorecardPage.vue";
import { entityOverview } from "@seikika/ui/entity";
import type { ReportData } from "./types";
import { kindTitle } from "./lib/kinds";
import { plural } from "./lib/words";
import { shortId, title } from "./lib/rings";
import { trackPath, type Route } from "./lib/routes";
import type { Summary } from "./lib/summary";
import { buildView } from "./report-view";
import ReportContent from "./ReportContent.vue";

const props = defineProps<{ data: ReportData; summary: Summary; route: Route }>();
const emit = defineEmits<{ ready: [] }>();

const entity = computed(() => {
  const id = props.route.id;
  if (props.route.mode !== "entity" || !id || !props.summary.kindOf[id]) return null;
  const overview = entityOverview(id, props.summary.kindOf[id], props.data.entityOverview?.[id]);
  return overview.title === id ? { ...overview, title: shortId(id) } : overview;
});
const cardName = computed(() => {
  const name = props.route.name;
  return props.route.mode === "scorecard" && name && props.data.scorecards[name] ? name : null;
});

const body = shallowRef<Node | Node[]>();
const headerActions = shallowRef<Node>();
const breadcrumbs = shallowRef<Node>();
const sidebar = shallowRef<Node[]>();
const beforeScorecard = shallowRef<Node[]>();
const detailClass = shallowRef("");

watchPostEffect(() => {
  const content = buildView(props.data, props.summary, props.route);
  sidebar.value = undefined;
  if (entity.value) {
    const heading = content.querySelector(":scope > .ehead");
    headerActions.value = heading?.querySelector(":scope > .elevel") ?? undefined;
    breadcrumbs.value = content.querySelector(":scope > .crumb") ?? undefined;
    breadcrumbs.value?.parentNode?.removeChild(breadcrumbs.value);
    heading?.remove();
  } else if (cardName.value && content.matches(".split")) {
    const [nav, detail] = content.children;
    sidebar.value = [...nav.childNodes];
    beforeScorecard.value = [...detail.querySelectorAll(":scope > .crumb, :scope > .picker")];
    for (const item of beforeScorecard.value) item.parentNode?.removeChild(item);
    detail.querySelector(":scope > .head")?.remove();
    detailClass.value = detail.className;
    body.value = [...detail.childNodes];
    emit("ready");
    return;
  }
  body.value = content;
  emit("ready");
});

const detail = computed(() => entity.value && props.data.entities?.[entity.value.id]);
const displayId = computed(() => entity.value && (props.data.entitySources?.[entity.value.id]?.original ?? entity.value.id));
const offer = computed(() => entity.value && props.data.actions?.some(item => item.entities.includes(entity.value!.id)));
const entityLink = (id: string) => props.summary.kindOf[id] ? "#entity/" + encodeURIComponent(id) : "";
const scoredBy = computed(() => {
  const id = entity.value?.id;
  return id ? Object.keys(props.data.scores).filter(name => props.data.scores[name][id]) : [];
});
const campaigns = computed(() => Object.entries(props.data.campaigns ?? {}).filter(([, campaign]) =>
  entity.value && campaign.entities?.[entity.value.id]));
const tracks = computed(() => [...new Set([
  ...scoredBy.value.map(name => props.data.scorecards[name]?.track),
  ...campaigns.value.map(([, campaign]) => campaign.track),
])].filter((name): name is string => Boolean(name && props.data.tracks?.[name])));
</script>

<template>
  <EntityPage v-if="entity" :entity="entity" heading="h2" class="view report-entity"
    :owner-href="entity.owner && summary.kindOf[entity.owner] ? '#entity/' + encodeURIComponent(entity.owner) : undefined">
    <template #breadcrumbs><ReportContent v-if="breadcrumbs" :build="() => breadcrumbs ?? null" /></template>
    <template #meta>
      <code v-if="displayId !== entity.title">{{ displayId }}</code>
      <div v-if="entitySource(data, entity.id)" class="sub">{{ entitySource(data, entity.id) }}</div>
      <div class="sub">{{ kindTitle(data, entity.kind) }} · {{ plural(scoredBy.length, "scorecard") }}</div>
    </template>
    <template #header-actions><ReportContent v-if="headerActions" :build="() => headerActions ?? null" /></template>
    <SectionCard v-if="campaigns.length || tracks.length" title="Campaigns and tracks">
      <ul>
        <li v-for="[name, campaign] in campaigns" :key="'campaign:' + name">
          <a :href="'#campaign/' + encodeURIComponent(name)">{{ campaign.title || name }}</a>
          <small class="sub"> · {{ campaign.entities?.[entity.id]?.meeting ? 'Meets goal' : 'Below goal' }}</small>
        </li>
        <li v-for="name in tracks" :key="'track:' + name">
          <a :href="'#' + trackPath(data, name)">{{ data.tracks?.[name]?.title || name }}</a>
        </li>
      </ul>
    </SectionCard>
    <ReportContent :build="() => body ?? null" />
    <div v-if="detail" class="report-entity-details">
      <SectionCard v-if="Object.keys(detail.fields).length" title="Fields">
        <dl>
          <template v-for="(value, name) in detail.fields" :key="name">
            <dt>{{ name }}</dt><dd><DataValue :value="value" :entity-href="entityLink" />
              <small v-if="detail.provenance?.[name]" class="sub"> · {{ detail.provenance[name] }}</small></dd>
          </template>
        </dl>
      </SectionCard>
      <SectionCard v-if="detail.relations?.in.length || detail.relations?.out.length" title="Related entities">
        <ul>
          <li v-for="edge in detail.relations.in" :key="'in:' + edge.relation + ':' + edge.from">
            {{ edge.relation }} ← <a :href="entityLink(edge.from)">{{ data.entitySources?.[edge.from]?.original ?? edge.from }}</a>
          </li>
          <li v-for="edge in detail.relations.out" :key="'out:' + edge.relation + ':' + edge.to">
            {{ edge.relation }} → <a :href="entityLink(edge.to)">{{ data.entitySources?.[edge.to]?.original ?? edge.to }}</a>
          </li>
        </ul>
      </SectionCard>
    </div>
    <SectionCard v-if="offer" title="Actions">
      <a :href="'#actions?entity=' + encodeURIComponent(entity.id)">Preview actions for {{ entity.title }}</a>
    </SectionCard>
  </EntityPage>
  <div v-else-if="cardName && sidebar" class="view split">
    <ReportContent as="section" :build="() => sidebar ?? null" />
    <section :class="detailClass">
      <ReportContent :build="() => beforeScorecard ?? null" />
      <ScorecardPage :title="title(data, cardName)" heading="h2">
        <template #header-actions><code class="sub">{{ cardName }}</code></template>
        <ReportContent :build="() => body ?? null" />
      </ScorecardPage>
    </section>
  </div>
  <ReportContent v-else class="view" :build="() => body ?? null" />
</template>
