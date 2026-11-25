<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import PageFrame from "@toolbox-dev/ui/components/PageFrame.vue";
import { api, devUser, type ApiResult } from "../api";
import { portalRoute } from "../routes";
import PortalHeader from "./PortalHeader.vue";
import PortalView from "./PortalView.vue";

const route = ref(portalRoute(new URL(location.href)));
const user = ref(devUser());
const result = ref<ApiResult | null>(null);
const ownership = ref(null);
const newer = ref(false);
const revision = ref(0);
const request = computed(() => route.value.api);

function syncLocation() {
  user.value = devUser();
  route.value = portalRoute(new URL(location.href));
}
function navigate(url: string) {
  if (location.pathname + location.search !== url) history.pushState(null, "", url);
  syncLocation();
}
function follow(event: MouseEvent) {
  if (event.defaultPrevented || event.button || event.metaKey || event.ctrlKey || event.shiftKey || event.altKey) return;
  const link = (event.target as Element | null)?.closest("a");
  if (!link || link.target || link.download || link.origin !== location.origin) return;
  const target = new URL(link.href);
  if (target.hash) return;
  event.preventDefault();
  navigate(target.pathname + target.search);
}

watch([request, revision, user], async ([path], previous, onCleanup) => {
  const requestController = new AbortController();
  onCleanup(() => requestController.abort());
  if (path !== previous?.[0] || user.value !== previous?.[2]) {
    result.value = null;
    ownership.value = null;
  }
  if (!path) return;
  newer.value = false;
  const options = { signal: requestController.signal };
  const current = route.value;
  if (current.page === "entity") {
    void api("/v1/entities/" + encodeURIComponent(current.id) + "/explain?field=ownership", options)
      .then(value => {
        if (!options.signal.aborted) ownership.value = value.status === 200 ? value.body?.authority : null;
      })
      .catch(() => {});
  }
  try {
    const next = await api(path, options);
    if (!options.signal.aborted) result.value = next;
  } catch {
    if (!options.signal.aborted) result.value = { status: 0, body: { title: "Unable to connect", detail: "Retry the request." }, snapshot: null };
  }
}, { immediate: true });

const refresh = ref({ running: false, failed: false, message: "" });
let refreshId = 0;
async function refreshEntity() {
  const current = route.value;
  if (current.page !== "entity" || refresh.value.running) return;
  const operation = ++refreshId;
  refresh.value = { running: true, failed: false, message: "Asking the connectors to re-read this entity…" };
  try {
    const request = await api("/v1/entities/" + encodeURIComponent(current.id) + "/refresh", { method: "POST" });
    if (request.status !== 202) throw new Error(request.body?.detail ?? "HTTP " + request.status);
    let snapshot: string | null = null;
    for (const id of request.body.commands as string[]) {
      const command = await api("/v1/commands/" + encodeURIComponent(id) + "?wait=30");
      if (command.body?.status !== "succeeded") throw new Error("Refresh " + (command.body?.status ?? "failed"));
      snapshot = command.body.snapshot ?? snapshot;
    }
    if (refreshId === operation) {
      refresh.value = { running: false, failed: false, message: snapshot ? "Refreshed at snapshot " + snapshot : "Refreshed" };
      revision.value++;
    }
  } catch (error) {
    if (refreshId === operation) refresh.value = { running: false, failed: true, message: (error as Error).message };
  }
}
watch([request, user], () => {
  refreshId++;
  refresh.value = { running: false, failed: false, message: "" };
});

let poll: ReturnType<typeof setInterval>;
onMounted(() => {
  addEventListener("popstate", syncLocation);
  poll = setInterval(async () => {
    const displayed = result.value;
    if (!displayed?.snapshot) return;
    const next = await api("/v1/snapshot").catch(() => null);
    if (displayed === result.value && next?.snapshot) newer.value = next.snapshot !== displayed.snapshot;
  }, 10_000);
});
onUnmounted(() => {
  removeEventListener("popstate", syncLocation);
  clearInterval(poll);
});
</script>

<template>
  <PageFrame class="portal-ui" @click="follow">
    <template #navigation>
      <PortalHeader :user="user" :newer="newer" @navigate="navigate" @reload="revision++" />
    </template>
    <PortalView :route="route" :result="result" :ownership="ownership" :refresh="refresh"
      @navigate="navigate" @refresh="refreshEntity" @retry="revision++" />
  </PageFrame>
</template>
