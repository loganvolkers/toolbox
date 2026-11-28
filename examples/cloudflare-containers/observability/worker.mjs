import { DurableObject } from "cloudflare:workers";

const DASHBOARD = "/d/seikika-runtime/seikika-runtime";
const OTLP_PATHS = new Set(["/v1/logs", "/v1/metrics", "/v1/traces"]);
const ENCODER = new TextEncoder();

function ingestAuthorized(request, token) {
  const authorization = request.headers.get("Authorization") ?? "";
  if (!/^[a-f0-9]{64}$/.test(token ?? "") || !/^Bearer [a-f0-9]{64}$/.test(authorization)) {
    return false;
  }
  return crypto.subtle.timingSafeEqual(ENCODER.encode(authorization.slice(7)), ENCODER.encode(token));
}

function upstreamRequest(request, ingest) {
  const url = new URL(request.url);
  const headers = new Headers();
  for (const name of ingest
    ? ["content-type", "content-encoding"]
    : ["accept", "content-type", "if-modified-since", "if-none-match", "accept-encoding"]) {
    const value = request.headers.get(name);
    if (value !== null) headers.set(name, value);
  }
  const target = new URL("http://observability");
  target.pathname = url.pathname;
  if (!ingest) target.search = url.search;
  return new Request(target, { method: request.method, headers, body: request.body, redirect: "manual" });
}

export class ObservabilityContainer extends DurableObject {
  #starting;
  #ready = false;

  async #start(container) {
    if (!container.running) container.start({ enableInternet: false });
    await container.setInactivityTimeout(30 * 60 * 1000);
    const collector = container.getTcpPort(13133);
    const grafana = container.getTcpPort(3000);
    const deadline = Date.now() + 20_000;
    while (Date.now() < deadline) {
      const checks = await Promise.all([
        collector.fetch("http://observability/ready", { signal: AbortSignal.timeout(1000) }).catch(() => null),
        grafana.fetch("http://observability/api/health", { signal: AbortSignal.timeout(1000) }).catch(() => null),
      ]);
      const healthy = checks.every((result) => result?.ok);
      await Promise.all(checks.map((result) => result?.body?.cancel()));
      if (healthy) {
        this.#ready = true;
        return;
      }
      await scheduler.wait(250);
    }
    throw new Error("Observability startup timed out");
  }

  async fetch(request) {
    const container = this.ctx.container;
    try {
      if (!container.running || !this.#ready) {
        this.#ready = false;
        this.#starting ??= this.#start(container).finally(() => { this.#starting = undefined; });
        await this.#starting;
      }
      const ingest = OTLP_PATHS.has(new URL(request.url).pathname);
      return await container.getTcpPort(ingest ? 4318 : 3000).fetch(upstreamRequest(request, ingest));
    } catch {
      this.#ready = false;
      return new Response("Observability is starting or unavailable", {
        status: 503,
        headers: { "Retry-After": "5" },
      });
    }
  }
}

export default {
  fetch(request, env) {
    const url = new URL(request.url);
    const { pathname } = url;
    if (OTLP_PATHS.has(pathname)) {
      if (request.method !== "POST") return new Response("Method not allowed", { status: 405 });
      if (!ingestAuthorized(request, env.OTLP_INGEST_TOKEN)) {
        return new Response("Unauthorized", { status: 401 });
      }
      if (request.headers.get("Content-Type") !== "application/x-protobuf") {
        return new Response("Expected OTLP protobuf", { status: 415 });
      }
    } else {
      const read = request.method === "GET" || request.method === "HEAD";
      const query = request.method === "POST" && pathname === "/api/ds/query";
      if (!read && !query) return new Response("Method not allowed", { status: 405 });
      if (pathname === "/") return Response.redirect(new URL(DASHBOARD, url), 302);
      if (/^\/(?:login|logout|signup|api\/admin)(?:\/|$)/.test(pathname)) {
        return new Response("Not found", { status: 404 });
      }
    }
    return env.OBSERVABILITY.getByName("primary").fetch(request);
  },
};
