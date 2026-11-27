import { DurableObject } from "cloudflare:workers";

const PORT = 8080;
const IDLE_TIMEOUT_MS = 30 * 60 * 1000;
const STARTUP_TIMEOUT_MS = 25_000;

function containerEnvironment(env) {
  const { DATABASE_URL, SEIKIKA_SIGNING_KEY, SEIKIKA_AUTH_MODE } = env;
  if (!DATABASE_URL || !/^[0-9a-f]{64}$/i.test(SEIKIKA_SIGNING_KEY ?? "")) {
    throw new Error("Set DATABASE_URL and a 32-byte hex SEIKIKA_SIGNING_KEY");
  }
  const values = { DATABASE_URL, SEIKIKA_SIGNING_KEY, SEIKIKA_AUTH_MODE };
  if (SEIKIKA_AUTH_MODE === "cloudflare-access") {
    const { SEIKIKA_CF_TEAM_DOMAIN, SEIKIKA_AUDIENCE } = env;
    if (!SEIKIKA_CF_TEAM_DOMAIN || !SEIKIKA_AUDIENCE) {
      throw new Error("Set SEIKIKA_CF_TEAM_DOMAIN and SEIKIKA_AUDIENCE for Cloudflare Access");
    }
    return { ...values, SEIKIKA_CF_TEAM_DOMAIN, SEIKIKA_AUDIENCE };
  }
  if (SEIKIKA_AUTH_MODE !== "token") {
    throw new Error("SEIKIKA_AUTH_MODE must be token or cloudflare-access");
  }
  return values;
}

export class SeikikaContainer extends DurableObject {
  #starting;
  #listening = false;

  async #start(container) {
    if (!container.running) {
      container.start({ env: containerEnvironment(this.env), enableInternet: true });
    }
    await container.setInactivityTimeout(IDLE_TIMEOUT_MS);
    const port = container.getTcpPort(PORT);
    const deadline = Date.now() + STARTUP_TIMEOUT_MS;
    while (Date.now() < deadline) {
      // Wait for the HTTP listener. The portal's /readyz still reports whether federation is ready.
      const response = await port.fetch("http://seikika/healthz", {
        signal: AbortSignal.timeout(1_000),
      }).catch(() => null);
      if (response) {
        await response.body?.cancel();
        if (response.ok) {
          this.#listening = true;
          return;
        }
      }
      await scheduler.wait(250);
    }
    throw new Error("Seikika HTTP startup timed out");
  }

  async fetch(request) {
    const container = this.ctx.container;
    if (!container.running || !this.#listening) {
      this.#listening = false;
      this.#starting ??= this.#start(container).finally(() => {
        this.#starting = undefined;
      });
      try {
        await this.#starting;
      } catch (error) {
        console.error(
          "Seikika container did not start serving HTTP",
          error instanceof Error ? error.name : "Unknown error",
        );
        return new Response("Seikika is unavailable", {
          status: 503,
          headers: { "Retry-After": "5" },
        });
      }
    }

    const url = new URL(request.url);
    url.protocol = "http:";
    url.host = "seikika";
    url.port = "";
    const forwarded = new Request(url, request);
    forwarded.headers.delete("Host");
    return container.getTcpPort(PORT).fetch(forwarded);
  }
}

export default {
  fetch(request, env) {
    const { pathname } = new URL(request.url);
    if (pathname === "/metrics" || pathname === "/metrics/") {
      return new Response("Not found", { status: 404 });
    }
    return env.SEIKIKA.getByName("primary").fetch(request);
  },
};
