// A private service binding probes readiness without exposing the portal through Access.
const metricsCron = "*/5 * * * *";
const readyCron = "7 * * * *";
const connection = "planetscale_postgres_connection_state";
const maximum = "planetscale_postgres_settings_max_connections";

async function databaseHealth(env) {
  const root = "https://api.planetscale.com/v1/organizations";
  const query = new URLSearchParams([["metrics[]", connection], ["metrics[]", maximum]]);
  const response = await fetch(`${root}/${env.PLANETSCALE_ORG}/databases/${env.DATABASE}/branches/main/metrics/instant?${query}`, {
    headers: { Authorization: env.PLANETSCALE_SERVICE_TOKEN_ID + ":" + env.PLANETSCALE_SERVICE_TOKEN },
    redirect: "error",
    signal: AbortSignal.timeout(15000),
  });
  if (!response.ok) throw new Error("Seikika database metrics are unavailable");
  const { metrics } = await response.json();
  const active = metrics.find((item) => item.metric === connection)?.values ?? [];
  const limits = metrics.find((item) => item.metric === maximum)?.values ?? [];
  const primary = limits.filter((item) => item.role === "primary");
  if (primary.length !== 1) throw new Error("Seikika primary connection limit is unavailable");
  const [limit] = primary;
  const sessions = active.filter((item) => item.pod === limit.pod && item.role === "primary");
  if (!sessions.length || !Number.isFinite(limit.value) || limit.value < 1) {
    throw new Error("Seikika primary connection metrics are incomplete");
  }
  const count = sessions.reduce((total, item) => total + item.value, 0);
  if (!Number.isFinite(count)) throw new Error("Seikika database connection count is invalid");
  if (count >= limit.value * 0.7) {
    console.error("Seikika database connection saturation", { count, limit: limit.value });
  } else {
    console.log("Seikika database connection headroom", { count, limit: limit.value });
  }
}

async function readiness(env) {
  const until = Date.now() + 240000;
  do {
    try {
      const response = await env.PORTAL.fetch("https://portal.internal/readyz", { signal: AbortSignal.timeout(30000) });
      if (response.ok && (await response.json()).ready === true) {
        console.log("Seikika production is ready");
        return;
      }
      if (response.body && !response.bodyUsed) await response.body.cancel();
    } catch {
      // A cold container may require several probes before it can serve HTTP.
    }
    await new Promise((resolve) => setTimeout(resolve, 10000));
  } while (Date.now() < until);
  throw new Error("Seikika production readiness did not recover after warm-up");
}

export default {
  async scheduled(event, env) {
    if (event.cron === metricsCron) await databaseHealth(env);
    if (event.cron === readyCron) await readiness(env);
  },
};
