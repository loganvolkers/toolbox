<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from "vue";
import PageFrame from "@seikika/ui/components/PageFrame.vue";
import PageHeader from "@seikika/ui/components/PageHeader.vue";
import "@seikika/ui/theme.css";
import "./report.css";
import type { ReportData } from "./types";
import { boxLogo } from "./lib/bars";
import { aliasLevels } from "./lib/levels";
import { parseHash, trackByPath } from "./lib/routes";
import { registerIds } from "./lib/rings";
import { ruleTitles } from "./lib/rules";
import { campaignNamesSorted, summarize } from "./lib/summary";
import { tracksSorted } from "./lib/tracks";
import { fmt, plural } from "./lib/words";
import ReportContent from "./ReportContent.vue";
import ReportView from "./ReportView.vue";
import ReportActions from "./ReportActions.vue";
import { aggregateReport, sourceLabel } from "./lib/federation";
import type { ActionRequest } from "./action-request";

const props = defineProps<{ data: ReportData; initialRoute?: string; embedded?: boolean; requestAction?: ActionRequest }>();
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
  const value = parseHash(hash.value);
  if (value.mode === "tracks" && value.name) value.name = trackByPath(data.value, value.name);
  return value;
});
const active = computed(() => {
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
const tabs = computed(() => {
  const tracks = tracksSorted(data.value, summary.value).length;
  const campaigns = campaignNamesSorted(data.value).length;
  return [
    { id: "overview", label: "Overview" },
    ...(tracks ? [{ id: "tracks", label: "Tracks", count: tracks }] : []),
    { id: "catalog", label: "Catalog", count: summary.value.entityCount },
    ...(campaigns || active.value === "campaigns" ? [{ id: "campaigns", label: "Campaigns", count: campaigns }] : []),
    { id: "scorecards", label: "Scorecards", count: levelCards.value },
    ...(data.value.actions?.length || active.value === "actions" ? [{ id: "actions", label: "Actions", count: data.value.actions?.length ?? 0 }] : []),
    ...(comparisons.value ? [{ id: "compare", label: "Compare", count: comparisons.value }] : []),
  ];
});
const columns = computed(() => tabs.value.length <= 3 ? tabs.value.length : tabs.value.length === 4 ? 2 : 3);
const portal = computed(() => data.value.portal || "Seikika report");
const showSummary = computed(() => !data.value.sources || active.value === "overview");

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
  hash.value = location.hash;
}
function go(path: string) {
  if (props.initialRoute === undefined) location.hash = path;
  else {
    history.replaceState(null, "", "#" + path);
    syncHash();
  }
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
    if (tabFocus) {
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

watch(() => route.value.mode, mode => { if (mode === "actions") void ready(); });

onMounted(() => {
  if (props.initialRoute !== undefined) history.replaceState(null, "", "#" + props.initialRoute);
  window.addEventListener("hashchange", syncHash);
  window.addEventListener("click", rememberScroll, true);
  window.addEventListener("scroll", onScroll, { passive: true, capture: true });
  window.addEventListener("keydown", escapeTip);
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
        <div v-if="showSummary" class="meta">
          {{ plural(summary.entityCount, "entity", "entities") }} tracked ({{ fmt(Object.keys(summary.worst).length) }} scored) ·
          {{ plural(levelCards, "scorecard") }}
          <template v-if="comparisons"> · {{ plural(comparisons, "comparison factor") }}</template>
          {{ data.snapshot ? " · data from snapshot #" + data.snapshot : " · no data collected yet" }}
        </div>
      </template>
    </PageHeader>
    <details v-if="data.sources && showSummary" class="report-sources">
      <summary>{{ Object.keys(data.sources).length }} portal sources</summary>
      <ul><li v-for="(source, key) in data.sources" :key="key">
        {{ sourceLabel(data, String(key)) }}
        <code v-if="source.commit">{{ source.commit.slice(0, 12) }}</code>
        <span v-if="source.error" class="sub"> · {{ source.error }}</span>
      </li></ul>
    </details>
    <div ref="navigation" class="modes" role="tablist" aria-label="Report views" :style="{ '--tabcols': columns }" @keydown="arrow" @scroll="scrollTabs">
      <button v-for="tab in tabs" :id="'tab-' + tab.id" :key="tab.id" type="button" role="tab"
        :aria-selected="active === tab.id" aria-controls="report-body"
        :tabindex="active === tab.id || (!active && tab.id === 'overview') ? 0 : -1" @click="selectTab($event, tab.id)">
        {{ tab.label }}<span v-if="tab.count != null" class="count">{{ fmt(tab.count) }}</span>
      </button>
    </div>
    <div id="report-body" ref="content" tabindex="-1" :role="active ? 'tabpanel' : undefined" :aria-labelledby="active ? 'tab-' + active : undefined">
      <ReportActions v-if="route.mode === 'actions'" :data="data" :entity="route.query?.entity" :request="requestAction" />
      <ReportView v-else :data="data" :summary="summary" :route="route" @ready="ready" />
    </div>
    <div ref="tooltip" class="ftip" :hidden="!tip" aria-hidden="true" :style="tipPosition" @mouseenter="keepTip">{{ tip }}</div>
  </component>
</template>
