// API client for the minimal UI. Same-origin; in dev mode the identity comes from `?as=<user>`
// (remembered per browser), behind an identity-aware proxy it comes from the proxy.

export type Json = any; // responses are shaped by the server's OpenAPI; typed clients arrive with codegen

const AS_KEY = "seikika.as";

export function devUser(): string | null {
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

export async function api(path: string, init: RequestInit = {}): Promise<ApiResult> {
  const headers = new Headers(init.headers);
  const user = devUser();
  if (user) headers.set("x-seikika-user", user);
  if (init.body) headers.set("content-type", "application/json");
  const r = await fetch(path, { ...init, headers });
  let body: Json = null;
  try {
    body = await r.json();
  } catch {
    body = null;
  }
  return { status: r.status, body, snapshot: r.headers.get("x-seikika-snapshot") ?? body?.snapshot ?? null };
}
