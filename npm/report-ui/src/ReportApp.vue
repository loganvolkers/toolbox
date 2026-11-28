<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from "vue";
import PageFrame from "@seikika/ui/components/PageFrame.vue";
import PageHeader from "@seikika/ui/components/PageHeader.vue";
import "@seikika/ui/theme.css";
import "./report.css";
import type { CatalogFilter, ReportData } from "./types";
import { boxLogo } from "./lib/bars";
import { aliasLevels } from "./lib/levels";
import { canonicalDetailHash, parseHash, reportRoute } from "./lib/routes";
import { registerIds } from "./lib/rings";
import { ruleTitles } from "./lib/rules";
import { campaignNamesSorted, summarize } from "./lib/summary";
import { tracksSorted } from "./lib/tracks";
import { fmt, plural } from "./lib/words";
import ReportContent from "./ReportContent.vue";
import ReportView from "./ReportView.vue";
import ReportActions from "./ReportActions.vue";
import AssessmentView from "./AssessmentView.vue";
import MetaCatalogDetails from "./MetaCatalogDetails.vue";
import { aggregateReport, sourceLabel } from "./lib/federation";
import type { ActionRequest } from "./action-request";

/** Inputs for mounting a static report or a host-backed live view. */
interface Props {
  /** Report snapshot and its configuration. */
  data: ReportData;
  /** Initial report route without the hash, defaulting to the browser location. */
  initialRoute?: string;
  /** Suppress the outer report header when the host renders its own. */
  embedded?: boolean;
  /** Host callback when a report action can be executed. */
  requestAction?: ActionRequest;
  /** Host-provided route names rendered through the extra-view slot. */
  extraViews?: readonly string[];
  /** Show the pending slot while the host is loading a snapshot. */
  pending?: boolean;
  /** Host callback for evaluating filters against the live catalog. */
  filterCatalog?: CatalogFilter;
}

const props = defineProps<Props>();
const hash = ref(props.initialRoute === undefined ? location.hash : "#" + props.initialRoute);
const content = ref<HTMLElement>();
const navigation = ref<HTMLElement>();
const tooltip = ref<HTMLElement>();
const tip = ref("");
const tipPosition = ref({ left: "", top: "" });
let tipTrigger: Element | undefined;
let tipTimer: ReturnType<typeof setTimeout> | undefined;
let scrollTimer: ReturnType<typeof setTimeout> | undefined;
let tabFocus = false;
let lastHash: string | undefined;

const data = computed(() => aggregateReport(props.data));
watch(data, data => {
  aliasLevels(data);
  ruleTitles(data);
  registerIds(Object.values(data.catalog ?? {}).flat());
}, { immediate: true, flush: "sync" });
const summary = computed(() => summarize(data.value));
const route = computed(() => {
  const value = parseHash(hash.value, props.extraViews);
  if (!data.value.publicCatalog) return reportRoute(data.value, value);
  return ["catalog", "entity"].includes(value.mode) || props.extraViews?.includes(value.mode)
    ? value : { mode: "catalog" };
});
const actionsMode = computed(() => route.value.mode === "actions" || route.value.mode === "action");
const catalogFilter = computed(() => route.value.kind && Object.hasOwn(props.data.catalog, route.value.kind)
  ? props.filterCatalog : undefined);
const assessmentMode = computed(() => {
  const mode = route.value.mode;
  if (mode === "assessments" || mode === "assessment" || mode === "rubric" || mode === "rubrics") return mode;
});
const extra = computed(() => props.extraViews?.includes(route.value.mode));
const active = computed(() => {
  if (extra.value || props.pending) return null;
  if (assessmentMode.value) return "assessments";
  if (actionsMode.value) return "actions";
  const current = route.value;
  if (current.mode === "missing") return null;
  if (current.bare && !data.value.scorecards[current.name ?? ""]) return null;
  if (current.mode === "entity") return "catalog";
  if (current.mode === "campaign") return "campaigns";
  if (current.mode === "rule" || current.mode === "scorecard") return "scorecards";
  return current.mode;
});
const comparisons = computed(() => Object.keys(data.value.comparisons ?? {}).length);
const levelCards = computed(() => summary.value.rows.filter(row => !row.compare).length);
const tabs = computed<{ id: string; label: string; count?: number }[]>(() => {
  const catalog = { id: "catalog", label: "Catalog", count: summary.value.entityCount };
  if (data.value.publicCatalog) return [catalog];
  const tracks = tracksSorted(data.value, summary.value).length;
  const campaigns = campaignNamesSorted(data.value).length;
  return [
    { id: "overview", label: "Overview" },
    ...(tracks ? [{ id: "tracks", label: "Tracks", count: tracks }] : []),
    catalog,
    ...(campaigns || active.value === "campaigns" ? [{ id: "campaigns", label: "Campaigns", count: campaigns }] : []),
    { id: "scorecards", label: "Scorecards", count: levelCards.value },
    { id: "assessments", label: "Assessments" },
    ...(data.value.actions?.length || active.value === "actions" ? [{ id: "actions", label: "Actions", count: data.value.actions?.length ?? 0 }] : []),
    ...(comparisons.value ? [{ id: "compare", label: "Compare", count: comparisons.value }] : []),
  ];
});
const columns = computed(() => tabs.value.length <= 3 ? tabs.value.length : tabs.value.length === 4 ? 2 : 3);
const portal = computed(() => data.value.portal || "Seikika report");
const showSummary = computed(() => !data.value.publicCatalog && (!data.value.sources || active.value === "overview"));
const reportSources = computed(() => Object.entries(data.value.sources ?? {}).filter(([, source]) => !source.repository));
const publication = computed(() => {
  const item = route.value.name || route.value.id || route.value.scorecard;
  const source = item && data.value.itemSources?.[item];
  return source ? { title: data.value.sources?.[source]?.title, href: "#overview?" + new URLSearchParams({ source }) } : undefined;
});
const selectedSource = computed(() => route.value.mode === "overview" ? route.value.query?.source : undefined);
const sourceList = ref<HTMLDetailsElement>();

function rememberScroll() {
  clearTimeout(scrollTimer);
  if (props.initialRoute === undefined) history.replaceState({ ...history.state, y: scrollY }, "");
}
function hideTip() {
  clearTimeout(tipTimer);
  tipTrigger = undefined;
  tip.value = "";
}
function onScroll() {
  hideTip();
  clearTimeout(scrollTimer);
  scrollTimer = setTimeout(rememberScroll, 200);
}
function syncHash() {
  clearTimeout(scrollTimer);
  hideTip();
  const current = location.hash;
  const canonical = canonicalDetailHash(data.value, current);
  if (canonical !== current) history.replaceState(history.state, "", canonical);
  hash.value = canonical;
}
function go(path: string) {
  if (props.initialRoute === undefined) location.hash = path;
  else {
    history.replaceState(null, "", "#" + path);
    syncHash();
  }
}
function selectAction(id: string) {
  const entity = route.value.query?.entity;
  go("action/" + encodeURIComponent(id) + (entity ? "?entity=" + encodeURIComponent(entity) : ""));
}
function selectTab(event: MouseEvent, path: string) {
  tabFocus = event.currentTarget === document.activeElement;
  go(path);
}
function arrow(event: KeyboardEvent) {
  const keys = ["ArrowLeft", "ArrowRight", "Home", "End"];
  if (!keys.includes(event.key)) return;
  const buttons = [...(navigation.value?.querySelectorAll<HTMLButtonElement>('[role="tab"]') ?? [])];
  if (!buttons.length) return;
  event.preventDefault();
  const index = buttons.findIndex(button => button === document.activeElement);
  const change = index + (event.key === "ArrowLeft" ? -1 : 1);
  const to = event.key === "Home" ? 0 : event.key === "End" ? buttons.length - 1 : (change + buttons.length) % buttons.length;
  tabFocus = true;
  go(tabs.value[to].id);
}
function scrollTabs() {
  const nav = navigation.value;
  if (!nav) return;
  nav.classList.toggle("more", nav.scrollLeft + nav.clientWidth < nav.scrollWidth - 4);
  nav.classList.toggle("less", nav.scrollLeft > 4);
}
async function ready() {
  await nextTick();
  const view = content.value;
  if (!view) return;
  const selected = navigation.value?.querySelector<HTMLElement>('[aria-selected="true"]');
  const headings = view.querySelectorAll<HTMLElement>("h2");
  const target = view.querySelector<HTMLElement>("[data-focus]");
  const heading = target?.querySelector<HTMLElement>("h2") ?? ((route.value.name || route.value.kind) ? headings[1] : null) ?? headings[0];
  if (props.initialRoute === undefined) document.title = heading && route.value.mode !== "overview" ? heading.textContent + " · " + portal.value : portal.value;
  if (selected && navigation.value && navigation.value.scrollWidth > navigation.value.clientWidth) {
    const nav = navigation.value;
    nav.scrollLeft += selected.getBoundingClientRect().left - nav.getBoundingClientRect().left - (nav.clientWidth - selected.offsetWidth) / 2;
  }
  scrollTabs();

  if (props.initialRoute === undefined && hash.value !== lastHash) {
    const frame = view.closest(".seikika-report");
    const y = frame?.getBoundingClientRect().top ?? 0;
    const inset = frame ? Number.parseFloat(getComputedStyle(frame).scrollMarginTop) || 0 : 0;
    const top = Math.max(0, y + scrollY - inset);
    const publicationRow = sourceList.value?.querySelector<HTMLElement>('[data-publication][aria-current="true"]');
    if (publicationRow) {
      publicationRow.focus();
    } else if (tabFocus) {
      selected?.focus({ preventScroll: true });
      scrollTo(0, top);
    } else if (heading && (lastHash !== undefined || props.embedded && hash.value)) {
      const targetTop = target ? target.getBoundingClientRect().top + scrollY - inset : top;
      scrollTo(0, targetTop - top < innerHeight * .6 ? top : targetTop - 8);
      if (lastHash !== undefined) {
        heading.tabIndex = -1;
        heading.focus({ preventScroll: true });
      }
    }
    if (lastHash !== undefined && typeof history.state?.y === "number") scrollTo(0, history.state.y);
  }
  tabFocus = false;
  lastHash = hash.value;
}
async function showTip(event: Event) {
  const trigger = (event.target as Element | null)?.closest<HTMLElement>(".basis[data-tip],abbr.gloss,a.rl[data-tip]");
  if (!trigger?.dataset.tip) return;
  clearTimeout(tipTimer);
  tipTrigger = trigger;
  tip.value = trigger.dataset.tip;
  await nextTick();
  if (!tooltip.value || tipTrigger !== trigger) return;
  const box = trigger.getBoundingClientRect();
  const { offsetWidth, offsetHeight } = tooltip.value;
  tipPosition.value = {
    left: Math.max(12, Math.min(box.left, document.documentElement.clientWidth - 12 - offsetWidth)) + "px",
    top: Math.max(8, box.bottom + offsetHeight + 4 > innerHeight ? box.top - 4 - offsetHeight : box.bottom + 4) + "px",
  };
}
function deferHide(event: Event) {
  if (tipTrigger?.contains(event.target as Node) || event.target === tooltip.value) {
    if (event.type === "focusout") hideTip();
    else if (tipTrigger !== document.activeElement) tipTimer = setTimeout(hideTip, 200);
  }
}
function keepTip() {
  clearTimeout(tipTimer);
}
function escapeTip(event: KeyboardEvent) {
  if (event.key === "Escape") hideTip();
}

watch(() => props.data, () => { if (props.initialRoute === undefined) syncHash(); }, { flush: "sync" });
watch(route, () => {
  if (actionsMode.value || extra.value) void ready();
}, { flush: "post" });
watch(() => [assessmentMode.value, route.value.id, route.value.query?.rubric], ([mode]) => {
  if (mode) void ready();
}, { flush: "post" });

onMounted(() => {
  if (props.initialRoute !== undefined) history.replaceState(null, "", "#" + props.initialRoute);
  syncHash();
  window.addEventListener("hashchange", syncHash);
  window.addEventListener("click", rememberScroll, true);
  window.addEventListener("scroll", onScroll, { passive: true, capture: true });
  window.addEventListener("keydown", escapeTip);
  if (assessmentMode.value || actionsMode.value || extra.value) void ready();
});
onUnmounted(() => {
  clearTimeout(scrollTimer);
  clearTimeout(tipTimer);
  window.removeEventListener("hashchange", syncHash);
  window.removeEventListener("click", rememberScroll, true);
  window.removeEventListener("scroll", onScroll, true);
  window.removeEventListener("keydown", escapeTip);
});
</script>

<template>
  <component :is="embedded ? 'div' : PageFrame" :class="embedded ? 'seikika-report seikika-ui' : 'seikika-report'" @mouseover="showTip" @mouseout="deferHide" @focusin="showTip" @focusout="deferHide">
    <button type="button" class="skip" @click="content?.focus()">Skip to content</button>
    <PageHeader :title="portal" :heading="embedded ? 'h2' : 'h1'">
      <template #title><span class="top"><ReportContent as="span" :build="boxLogo" />{{ portal }}</span></template>
      <template #meta>
        <div v-if="showSummary && !pending" class="meta">
          {{ plural(summary.entityCount, "entity", "entities") }} tracked ({{ fmt(Object.keys(summary.worst).length) }} scored) ·
          {{ plural(levelCards, "scorecard") }}
          <template v-if="comparisons"> · {{ plural(comparisons, "comparison factor") }}</template>
          {{ data.snapshot ? " · data from snapshot #" + data.snapshot : " · no data collected yet" }}
        </div>
      </template>
    </PageHeader>
    <a v-if="publication" class="report-publication-link" :href="publication.href">Publication details<span class="sr"> for {{ publication.title }}</span></a>
    <slot name="tools" :route="route" />
    <details v-if="reportSources.length && showSummary && !pending" ref="sourceList" class="report-sources" :open="Boolean(selectedSource) || reportSources.length <= 3">
      <summary>Report sources ({{ reportSources.length }})</summary>
      <ul><li v-for="[key, source] in reportSources" :key="key" :data-publication="key" :aria-current="selectedSource === key ? 'true' : undefined" tabindex="-1">
        {{ sourceLabel(data, key) }}
        <code v-if="source.commit" :title="source.commit">Commit {{ source.commit.slice(0, 12) }}</code>
        <code v-else-if="source.digest">Artifact SHA-256 {{ source.digest }}</code>
        <span v-else-if="source.local && data.snapshot" class="sub"> · Report snapshot {{ data.snapshot }}</span>
        <span v-if="source.error" class="sub"> · {{ source.error }}</span>
      </li></ul>
    </details>
    <div v-if="!pending" ref="navigation" class="modes" role="tablist" aria-label="Report views" :style="{ '--tabcols': columns }" @keydown="arrow" @scroll="scrollTabs">
      <button v-for="tab in tabs" :id="'tab-' + tab.id" :key="tab.id" type="button" role="tab"
        :aria-selected="active === tab.id" aria-controls="report-body"
        :tabindex="active === tab.id || (!active && tab.id === tabs[0]?.id) ? 0 : -1" @click="selectTab($event, tab.id)">
        {{ tab.label }}<span v-if="tab.count != null" class="count">{{ fmt(tab.count) }}</span>
      </button>
    </div>
    <div id="report-body" ref="content" tabindex="-1" :role="active ? 'tabpanel' : undefined" :aria-labelledby="active ? 'tab-' + active : undefined">
      <slot v-if="extra" name="extra-view" :route="route" :data="data" />
      <div v-else-if="pending" class="view"><slot name="pending" /></div>
      <ReportActions v-else-if="actionsMode" :data="data" :id="route.id" :entity="route.query?.entity" :request="requestAction" @select="selectAction">
        <template #default="{ id }">
          <MetaCatalogDetails :data="data" :summary="summary" :id="id" show-scores>
            <template v-if="$slots['entity-tools']" #entity-tools="context">
              <slot name="entity-tools" :id="context.id" :source="context.source" :snapshot="context.snapshot" />
            </template>
          </MetaCatalogDetails>
        </template>
      </ReportActions>
      <div v-else-if="assessmentMode" class="view">
        <AssessmentView :data="data" :mode="assessmentMode" :id="route.id" :filters="route.query" />
        <MetaCatalogDetails v-if="route.catalogId" :data="data" :summary="summary" :id="route.catalogId">
          <template v-if="$slots['entity-tools']" #entity-tools="context">
            <slot name="entity-tools" :id="context.id" :source="context.source" :snapshot="context.snapshot" />
          </template>
        </MetaCatalogDetails>
      </div>
      <template v-else>
        <ReportView :data="data" :summary="summary" :route="route" :filter-catalog="catalogFilter" @ready="ready">
          <template v-if="$slots['entity-tools']" #entity-tools="context">
            <slot name="entity-tools" :id="context.id" :source="context.source" :snapshot="context.snapshot" />
          </template>
        </ReportView>
        <slot v-if="route.mode === 'overview'" name="overview" :data="data" :summary="summary" />
      </template>
    </div>
    <div ref="tooltip" class="ftip" :hidden="!tip" aria-hidden="true" :style="tipPosition" @mouseenter="keepTip">{{ tip }}</div>
  </component>
</template>
