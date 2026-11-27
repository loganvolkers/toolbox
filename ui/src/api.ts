// Same-origin API client. In dev mode the identity comes from `?as=<user>`
// (remembered per browser), behind an identity-aware proxy it comes from the proxy.

export type Json = any; // responses are shaped by the server's OpenAPI; typed clients arrive with codegen

const AS_KEY = "seikika.as";

function devUser(): string | null {
  const q = new URLSearchParams(location.search).get("as");
  try {
    if (q) localStorage.setItem(AS_KEY, q);
    return q ?? localStorage.getItem(AS_KEY);
  } catch {
    return q;
  }
}

export interface ApiResult {
  status: number;
  body: Json;
  snapshot: string | null;
}

export type PortalApi = (path: string, init?: RequestInit) => Promise<ApiResult>;

export function portalClient(): PortalApi {
  const user = devUser();
  return async (path, init = {}) => {
    const headers = new Headers(init.headers);
    if (user) headers.set("x-seikika-user", user);
    if (init.body) headers.set("content-type", "application/json");
    const response = await fetch(path, { ...init, headers });
    const body = await response.json().catch(() => null);
    return { status: response.status, body, snapshot: response.headers.get("x-seikika-snapshot") ?? body?.snapshot ?? null };
  };
}
