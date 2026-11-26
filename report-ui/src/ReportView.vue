<script setup lang="ts">
import { computed, shallowRef, watchPostEffect } from "vue";
import EntityPage from "@seikika/ui/components/EntityPage.vue";
import ScorecardPage from "@seikika/ui/components/ScorecardPage.vue";
import { entityOverview } from "@seikika/ui/entity";
import type { ReportData } from "./types";
import { kindTitle } from "./lib/kinds";
import { plural } from "./lib/words";
import { shortId, title } from "./lib/rings";
import type { Route } from "./lib/routes";
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

const scoreCount = computed(() => {
  const id = entity.value?.id;
  return id ? Object.values(props.data.scores).filter(scores => scores[id]).length : 0;
});
</script>

<template>
  <EntityPage v-if="entity" :entity="entity" heading="h2" class="view report-entity"
    :owner-href="entity.owner && summary.kindOf[entity.owner] ? '#entity/' + encodeURIComponent(entity.owner) : undefined">
    <template #breadcrumbs><ReportContent v-if="breadcrumbs" :build="() => breadcrumbs ?? null" /></template>
    <template #meta>
      <code v-if="entity.id !== entity.title">{{ entity.id }}</code>
      <div class="sub">{{ kindTitle(data, entity.kind) }} · {{ plural(scoreCount, "scorecard") }}</div>
    </template>
    <template #header-actions><ReportContent v-if="headerActions" :build="() => headerActions ?? null" /></template>
    <ReportContent :build="() => body ?? null" />
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
