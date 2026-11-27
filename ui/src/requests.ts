import { inject, ref, shallowRef, watch, type InjectionKey } from "vue";
import { portalClient, type ApiResult, type PortalApi } from "./api";

export const portalApi: InjectionKey<PortalApi> = Symbol("portal API");

export function usePortalApi() {
  return inject(portalApi, portalClient, true);
}

export function usePortalRequest(
  path: () => string,
  options: { retryUnavailable?: boolean } = {},
) {
  const request = usePortalApi();
  const revision = ref(0);
  const result = shallowRef<ApiResult | null>(null);
  watch([path, revision], ([url], old, onCleanup) => {
    const controller = new AbortController();
    let retry: ReturnType<typeof setTimeout>;
    onCleanup(() => {
      controller.abort();
      clearTimeout(retry);
    });
    if (url !== old?.[0]) result.value = null;
    async function load() {
      try {
        const response = await request(url, { signal: controller.signal });
        if (controller.signal.aborted) return;
        if (options.retryUnavailable && response.status === 503) {
          retry = setTimeout(load, 1_000);
        } else {
          result.value = response;
        }
      } catch {
        if (!controller.signal.aborted) {
          result.value = { status: 0, body: { title: "Unable to connect", detail: "Retry the request." }, snapshot: null };
        }
      }
    }
    void load();
  }, { immediate: true });
  return { result, reload: () => { revision.value++; } };
}
