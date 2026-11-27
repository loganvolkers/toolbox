<script setup lang="ts">
import { computed, onMounted, onUnmounted, provide, ref, watch } from "vue";
import ReportApp from "../../../report-ui/src/ReportApp.vue";
import type { CatalogFilter, ReportData } from "../../../report-ui/src/types";
import type { PortalApi } from "../api";
import { portalApi, usePortalApi, usePortalRequest } from "../requests";
import { restorePortalRoute } from "../routes";
import PortalStatus from "./PortalStatus.vue";
import PortalLiveView from "./PortalLiveView.vue";
import PortalEntityTools from "./PortalEntityTools.vue";
import PortalTeams from "./PortalTeams.vue";

const publicOrganizationReport = "https://seikika.com/demo/organization/";
const props = defineProps<{ initialRoute?: string }>();
const publicEntry = !/^\/(report|ops)(\/|$)/.test(location.pathname);
if (props.initialRoute === undefined) restorePortalRoute(new URL(location.href));
const api = usePortalApi();
provide(portalApi, api);
const empty: ReportData = { portal: "Seikika", catalog: {}, scorecards: {}, scores: {} };
const extraViews = ["search", "operations"];
const readReport: PortalApi = async (path, init) => {
  if (publicEntry) {
    const catalog = await api("/v1/views/home?public_catalog=true", init).catch(() => null);
    if (catalog?.status === 503 || (catalog?.status === 200
      && (catalog.body?.publicCatalog || catalog.body?.viewerMethod === "guest"))) return catalog;
  }
  return api(path, init);
};
const { result, reload } = usePortalRequest(() => "/v1/views/report?partial=true", {
  retryUnavailable: true, request: readReport,
});
const report = computed<ReportData | undefined>(() => result.value?.status === 200 ? result.value.body : undefined);
const guest = computed(() => report.value?.publicCatalog === true || report.value?.viewerMethod === "guest");
const guestReport = computed(() => report.value?.viewerMethod === "guest");
const federation = computed(() => report.value?.federationStatus);
watch(report, (current, _, onCleanup) => {
  if (current?.federationStatus?.pending) {
    const timer = setTimeout(reload, 2_000);
    onCleanup(() => clearTimeout(timer));
  }
});
const query = ref("");
const newer = ref(false);
const updating = ref(false);
watch(result, () => { newer.value = updating.value = false; });
function update() {
  updating.value = true;
  reload();
}
function search() {
  if (query.value.trim()) location.hash = "search?" + new URLSearchParams({ q: query.value.trim() });
}

const filterCatalog: CatalogFilter = async (kind, expression, snapshot, signal) => {
  const params = new URLSearchParams({ kind, filter: expression, fields: "id", limit: "1000" });
  if (snapshot != null) params.set("min_snapshot", String(snapshot));
  const ids = new Set<string>();
  do {
    const response = await api("/v1/entities?" + params, { signal });
    if (response.status !== 200) throw new Error(response.body?.detail ?? "Unable to filter the catalog.");
    if (response.snapshot !== String(snapshot)) throw new Error("Update to the newer snapshot to apply this filter.");
    for (const item of response.body.items) ids.add(item.id);
    if (!response.body.next_cursor) return ids;
    params.set("cursor", response.body.next_cursor);
  } while (!signal.aborted);
  return ids;
};

let poll: ReturnType<typeof setInterval>;
onMounted(() => {
  poll = setInterval(async () => {
    const displayed = report.value;
    if (guest.value || !displayed?.snapshot) return;
    const next = await api("/v1/snapshot").catch(() => null);
    if (displayed === report.value && next?.snapshot) newer.value = next.snapshot !== String(displayed.snapshot);
  }, 10_000);
});
onUnmounted(() => clearInterval(poll));
</script>

<template>
  <ReportApp :data="report ?? empty" :initial-route="initialRoute" :pending="!report" :extra-views="extraViews"
    :request-action="guest ? undefined : api" :filter-catalog="guest ? undefined : filterCatalog">
    <template #tools="{ route }">
      <nav class="portal-tools" aria-label="Live portal">
        <form role="search" @submit.prevent="search">
          <input v-model="query" type="search" aria-label="Search" placeholder="Search entities" />
        </form>
        <template v-if="guest">
          <a :href="guestReport ? '#scorecards' : publicOrganizationReport">Public scorecards</a>
          <a :href="(guestReport ? '' : publicOrganizationReport) + '#scorecard/meta-scorecard'">Meta scoring</a>
        </template>
        <a :href="guest ? '/ops' : '#operations'" :aria-current="route.mode === 'operations' ? 'page' : undefined">Operations</a>
        <button v-if="newer || updating" type="button" :disabled="updating" @click="update">
          {{ updating ? "Updating data…" : "Newer data — update" }}
        </button>
        <span v-if="report?.viewer" class="sub">Signed in as {{ report.viewer }}</span>
        <a v-else-if="report || result?.status === 401 || result?.status === 0" class="portal-sign-in" href="/report">Sign in</a>
      </nav>
      <p v-if="federation?.pending" class="sub" role="status">{{ federation.detail }}. Local results are available.</p>
      <div v-else-if="federation" class="portal-problem" role="alert">
        <p>{{ federation.detail }}</p>
        <button type="button" :disabled="updating" @click="update">Reload portal data</button>
      </div>
    </template>
    <template #pending>
      <PortalStatus :result="result" loading="Loading portal snapshot…" @retry="reload" />
    </template>
    <template #extra-view="{ route, data }">
      <PortalLiveView :mode="route.mode" :query="route.query?.q ?? ''" :guest-report="guestReport ? data : undefined" />
    </template>
    <template #entity-tools="{ id, source, snapshot }">
      <PortalEntityTools v-if="!guest && !source?.startsWith('child:')" :key="id" :id="id"
        :snapshot="snapshot" @refreshed="reload" />
    </template>
    <template #overview="{ data, summary }">
      <PortalTeams v-if="!guest" :visible-entities="summary.kindOf" :snapshot="data.snapshot" />
    </template>
  </ReportApp>
</template>
