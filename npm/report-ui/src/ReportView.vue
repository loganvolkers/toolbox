<script setup lang="ts">
import { computed, ref, shallowRef, watch, watchPostEffect } from "vue";
import SectionCard from "@seikika/ui/components/SectionCard.vue";
import { entitySourceKeys, sourceLabel } from "./lib/federation";
import EntityPage from "@seikika/ui/components/EntityPage.vue";
import ScorecardPage from "@seikika/ui/components/ScorecardPage.vue";
import { entityOverview } from "@seikika/ui/entity";
import type { CatalogFilter, CatalogReport } from "./types";
import { kindTitle } from "./lib/kinds";
import { el } from "./lib/dom";
import { plural } from "./lib/words";
import { shortId, title } from "./lib/rings";
import { catalogEntityPath, type Route } from "./lib/routes";
import type { Summary } from "./lib/summary";
import { buildView } from "./report-view";
import ReportContent from "./ReportContent.vue";
import CatalogEntityFacts from "./CatalogEntityFacts.vue";
import ServiceRuntime from "./ServiceRuntime.vue";
import CatalogEntityContext from "./CatalogEntityContext.vue";
import MetaCatalogDetails from "./MetaCatalogDetails.vue";

const props = defineProps<{ data: CatalogReport; summary: Summary; route: Route; filterCatalog?: CatalogFilter }>();
const emit = defineEmits<{ ready: [] }>();

const entity = computed(() => {
  const id = props.route.id;
  if (props.route.mode !== "entity" || !id || !props.summary.kindOf[id]) return null;
  const fields = {
    ...props.data.entities?.[id]?.fields, ...props.data.entityOverview?.[id],
  };
  const overview = entityOverview(id, props.summary.kindOf[id], fields);
  if (props.data.entityObservations?.[id]?.length) {
    overview.links = overview.links.map(link => ({ ...link, label: link.label !== "Source" ? link.label
      : link.href === fields.repositoryUrl ? "Repository" : link.href === fields.sourceUrl ? "Scanned revision" : link.label }));
  }
  return overview.title === id ? { ...overview, title: shortId(id) } : overview;
});
const cardName = computed(() => {
  const name = props.route.name;
  return props.route.mode === "scorecard" && name && props.data.scorecards[name] ? name : null;
});

const catalogKind = computed(() => props.route.mode === "catalog" && props.route.kind && props.filterCatalog
  && Object.hasOwn(props.data.catalog, props.route.kind) ? props.route.kind : undefined);
const expression = computed(() => catalogKind.value ? props.route.query?.filter ?? "" : "");
const draft = ref("");
const catalogIds = shallowRef<ReadonlySet<string>>();
const filtering = ref(false);
const filterError = ref("");
watch([catalogKind, expression], ([, filter]) => { draft.value = filter; }, { immediate: true });
watch([catalogKind, expression, () => props.data.snapshot], async ([kind, filter, snapshot], _, onCleanup) => {
  const controller = new AbortController();
  onCleanup(() => controller.abort());
  catalogIds.value = undefined;
  filterError.value = "";
  filtering.value = Boolean(kind && filter);
  if (!kind || !filter || !props.filterCatalog) return;
  try {
    const ids = await props.filterCatalog(kind, filter, snapshot, controller.signal);
    if (!controller.signal.aborted) catalogIds.value = ids;
  } catch (error) {
    if (!controller.signal.aborted) filterError.value = (error as Error).message;
  } finally {
    if (!controller.signal.aborted) filtering.value = false;
  }
}, { immediate: true });
function submitFilter(filter: string) {
  const [path, query] = location.hash.split("?", 2);
  const params = new URLSearchParams(query);
  if (filter.trim()) params.set("filter", filter.trim());
  else params.delete("filter");
  location.hash = path + (params.size ? "?" + params : "");
}

const body = shallowRef<Node | Node[]>();
const headerActions = shallowRef<Node>();
const breadcrumbs = shallowRef<Node>();
const sidebar = shallowRef<Node[]>();
const beforeScorecard = shallowRef<Node[]>();
const detailClass = shallowRef("");

watchPostEffect(() => {
  if (filtering.value || filterError.value) return;
  const content = buildView(props.data, props.summary, props.route, catalogIds.value);
  sidebar.value = undefined;
  if (entity.value) {
    const heading = content.querySelector(":scope > .ehead");
    headerActions.value = heading?.querySelector(":scope > .elevel") ?? undefined;
    breadcrumbs.value = content.querySelector(":scope > .crumb") ?? undefined;
    breadcrumbs.value?.parentNode?.removeChild(breadcrumbs.value);
    heading?.remove();
  } else if (cardName.value && content.matches(".split")) {
    const [nav, detail] = content.children;
    const id = props.route.catalogId;
    if (id) {
      const evaluated = Object.keys(props.data.scorecards).some(name => props.data.scores[name]?.[id]);
      const target = evaluated ? ".report-catalog-scores h2" : ".report-catalog-details h2";
      detail.querySelector(".jumpbar")?.append(el("button", {
        type: "button",
        className: "link",
        textContent: evaluated ? "How this scorecard is evaluated" : "Catalog details",
        onclick: (event: MouseEvent) => {
          const button = event.currentTarget as HTMLButtonElement;
          const heading = button.closest(".view")?.querySelector<HTMLElement>(target);
          if (!heading) return;
          heading.tabIndex = -1;
          heading.scrollIntoView({ block: "start", behavior: matchMedia("(prefers-reduced-motion: reduce)").matches ? "auto" : "smooth" });
          heading.focus({ preventScroll: true });
        },
      }));
    }
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

const displayId = computed(() => entity.value && (props.data.entitySources?.[entity.value.id]?.original ?? entity.value.id));
const sources = computed(() => entity.value ? entitySourceKeys(props.data, entity.value.id) : []);
const facts = ref<HTMLElement>();
function showSourceFields() {
  const heading = facts.value?.querySelector<HTMLElement>("h2, h3");
  if (!heading) return;
  heading.tabIndex = -1;
  heading.focus({ preventScroll: true });
  heading.scrollIntoView({ block: "start" });
}
const offer = computed(() => entity.value && props.data.actions?.some(item => item.entities.includes(entity.value!.id)));
const service = computed(() => {
  const item = entity.value;
  if (item?.kind !== "Service") return;
  const detail = props.data.entities?.[item.id];
  if (typeof detail?.fields.code === "string") return detail;
});
const scoredBy = computed(() => {
  const id = entity.value?.id;
  return id ? Object.keys(props.data.scores).filter(name => props.data.scores[name][id]) : [];
});
</script>

<template>
  <EntityPage v-if="entity" :entity="entity" heading="h2" class="view report-entity"
    :owner-href="entity.owner && summary.kindOf[entity.owner] ? '#' + catalogEntityPath(data, entity.owner) : undefined">
    <template #breadcrumbs><ReportContent v-if="breadcrumbs" :build="() => breadcrumbs ?? null" /></template>
    <template #meta>
      <code v-if="displayId !== entity.title">{{ displayId }}</code>
      <div v-for="source in sources" :key="source" class="sub">{{ sourceLabel(data, source) }}</div>
      <button v-if="sources.length > 1" type="button" class="link source-fields-jump" @click="showSourceFields">View source fields</button>
      <div class="sub">{{ kindTitle(data, entity.kind) }}
        <template v-if="!data.publicCatalog"> · {{ plural(scoredBy.length, "scorecard") }}</template>
        <span v-if="data.snapshot != null" class="asof"> · as of snapshot <code>{{ data.snapshot }}</code></span>
      </div>
    </template>
    <template #header-actions><ReportContent v-if="headerActions" :build="() => headerActions ?? null" /></template>
    <slot name="entity-tools" :id="entity.id" :source="data.entitySources?.[entity.id]?.source" :snapshot="data.snapshot" />
    <CatalogEntityContext :id="entity.id" :data="data" />
    <ReportContent v-if="!data.publicCatalog" :build="() => body ?? null" />
    <div ref="facts">
      <ServiceRuntime v-if="service" :data="data" :id="entity.id" :detail="service" />
      <CatalogEntityFacts v-else :data="data" :summary="summary" :id="entity.id" />
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
      <MetaCatalogDetails v-if="route.catalogId" :data="data" :summary="summary" :id="route.catalogId" show-scores>
        <template v-if="$slots['entity-tools']" #entity-tools="context">
          <slot name="entity-tools" v-bind="context" />
        </template>
      </MetaCatalogDetails>
    </section>
  </div>
  <div v-else-if="catalogKind" class="view">
    <form class="report-filters report-catalog-filter" @submit.prevent="submitFilter(draft)">
      <label>
        <span>CEL filter over <code>entity</code></span>
        <input v-model="draft" placeholder="entity.ownership.status != 'verified'" />
      </label>
      <button type="submit">Apply</button>
      <button v-if="expression" type="button" @click="submitFilter('')">Clear CEL filter</button>
    </form>
    <p v-if="filtering" role="status">Applying filter…</p>
    <p v-else-if="filterError" role="alert">{{ filterError }}</p>
    <ReportContent v-else :build="() => body ?? null" />
  </div>
  <div v-else-if="route.catalogId" class="view">
    <ReportContent :build="() => body ?? null" />
    <MetaCatalogDetails :data="data" :summary="summary" :id="route.catalogId" show-scores>
      <template v-if="$slots['entity-tools']" #entity-tools="context">
        <slot name="entity-tools" v-bind="context" />
      </template>
    </MetaCatalogDetails>
  </div>
  <ReportContent v-else class="view" :build="() => body ?? null" />
</template>
