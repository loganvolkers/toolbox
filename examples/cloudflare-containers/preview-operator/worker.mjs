// Reconcile opted-in same-repository pull requests from the trusted controller.
const cfOrigin = "https://api.cloudflare.com/client/v4";
const psOrigin = "https://api.planetscale.com/v1";
const githubIssuer = "https://token.actions.githubusercontent.com";
const workflowPath = ".github/workflows/cloudflare-previews.yml";
const appName = (number) => `Seikika managed PR ${number}`;
const databaseName = (number) => `seikika-preview-pr-${number}`;
const namePattern = /^Seikika managed PR ([1-9]\d*)$/;

async function request(origin, path, { auth, method = "GET", body, allowMissing = false, type } = {}) {
  const headers = { Accept: "application/json", "User-Agent": "Seikika-preview-operator" };
  if (auth) headers.Authorization = auth;
  if (body) headers["Content-Type"] = type ?? "application/json";
  const response = await fetch(origin + path, {
    method, headers, body: body ? JSON.stringify(body) : undefined, redirect: "manual",
  });
  if (response.status === 404 && allowMissing) return null;
  if (!response.ok) {
    await response.body?.cancel();
    throw new Error(`${new URL(origin).hostname} ${method} failed with HTTP ${response.status}`);
  }
  return response.status === 204 ? null : response.json();
}

function clients(env) {
  const account = "/accounts/" + env.CLOUDFLARE_ACCOUNT_ID;
  const worker = `/workers/workers/${env.CLOUDFLARE_WORKER}/previews`;
  const dbRoot = `/organizations/${env.PLANETSCALE_ORG}/databases`;
  const cf = async (path, options) => {
    const response = await request(cfOrigin, account + path, { ...options, auth: "Bearer " + env.CLOUDFLARE_API_TOKEN });
    if (response && response.success !== true) throw new Error("Cloudflare returned an unsuccessful API result");
    return response?.result ?? null;
  };
  const ps = (path, options) => request(psOrigin, dbRoot + path, {
    ...options, auth: env.PLANETSCALE_SERVICE_TOKEN_ID + ":" + env.PLANETSCALE_SERVICE_TOKEN,
  });
  return { cf, ps, worker };
}

function decode(segment) {
  const base64 = segment.replaceAll("-", "+").replaceAll("_", "/");
  return Uint8Array.from(atob(base64), (c) => c.charCodeAt(0));
}

async function authorized(incoming, env) {
  const header = incoming.headers.get("Authorization") ?? "";
  if (!header.startsWith("Bearer ") || header.length > 10000) return false;
  try {
    const parts = header.slice(7).split(".");
    if (parts.length !== 3 || !parts.every((part) => /^[A-Za-z0-9_-]+$/.test(part))) return false;
    const [head, payload, signature] = parts;
    const decodeText = (segment) => JSON.parse(new TextDecoder().decode(decode(segment)));
    const key = decodeText(head);
    const claims = decodeText(payload);
    const now = Math.floor(Date.now() / 1000);
    if (key.alg !== "RS256" || typeof key.kid !== "string" ||
        claims.iss !== githubIssuer || claims.aud !== env.GITHUB_OIDC_AUDIENCE ||
        claims.repository !== env.GITHUB_REPOSITORY || claims.repository_id !== env.GITHUB_REPOSITORY_ID ||
        claims.ref !== "refs/heads/main" ||
        claims.workflow_ref !== `${env.GITHUB_REPOSITORY}/${workflowPath}@refs/heads/main` ||
        !["schedule", "pull_request_target", "workflow_dispatch"].includes(claims.event_name) ||
        !Number.isInteger(claims.iat) || claims.iat < now - 600 || claims.iat > now + 30 ||
        !Number.isInteger(claims.exp) || claims.exp <= now) return false;
    const { keys } = await request(githubIssuer, "/.well-known/jwks");
    const jwk = keys.find((j) => j.kid === key.kid && j.kty === "RSA" && j.alg === "RS256");
    if (!jwk) return false;
    const publicKey = await crypto.subtle.importKey(
      "jwk", jwk, { name: "RSASSA-PKCS1-v1_5", hash: "SHA-256" }, false, ["verify"],
    );
    return crypto.subtle.verify(
      "RSASSA-PKCS1-v1_5", publicKey, decode(signature), new TextEncoder().encode(`${head}.${payload}`),
    );
  } catch {
    return false;
  }
}

function directDatabaseUrl(role) {
  const host = new URL(role.access_host_url.includes("://") ? role.access_host_url : "https://" + role.access_host_url);
  if (!/^[a-z0-9.-]+\.psdb\.cloud$/.test(host.hostname) || !role.username || !role.password) {
    throw new Error("Unexpected PlanetScale direct credentials");
  }
  return `postgresql://${encodeURIComponent(role.username)}:${encodeURIComponent(role.password)}@${host.hostname}:5432/postgres?sslmode=verify-full`;
}

async function ensureAccess(cf, env, preview, apps, number) {
  const host = new URL(preview.urls[0]).hostname;
  if (host !== `${preview.slug}-${env.CLOUDFLARE_WORKER}.${env.WORKER_SUBDOMAIN}.workers.dev`) {
    throw new Error("Unexpected preview hostname");
  }
  const found = apps.find((app) => app.name === appName(number));
  if (found && found.domain !== host) throw new Error("Managed Access application domain changed");
  if (!found && apps.some((app) => app.domain === host)) {
    throw new Error("Preview already has an independently managed Access application");
  }
  const email = env.PREVIEW_ADMIN_EMAIL;
  const include = [{ email: { email } }];
  const app = found ?? await cf("/access/apps", {
    method: "POST",
    body: {
      name: appName(number), domain: host, type: "self_hosted", session_duration: "12h",
      allowed_idps: [env.CLOUDFLARE_IDP_ID], auto_redirect_to_identity: true,
      policies: [{ name: "Preview owner", decision: "allow", include }],
    },
  });
  const [policy, ...others] = await cf(`/access/apps/${app.id}/policies`);
  if (!policy || others.length || policy.decision !== "allow" ||
      policy.include?.length !== 1 || policy.include[0].email?.email !== email) {
    throw new Error("Managed preview Access policy differs from its expected owner-only rule");
  }
  return app;
}

async function provisionDatabase(cf, ps, env, number) {
  const name = databaseName(number);
  const database = await ps("/" + name, { allowMissing: true });
  if (database) {
    if (database.name !== name || database.kind !== "postgresql") throw new Error("Unexpected preview database");
    const branch = await ps(`/${name}/branches/main`, { allowMissing: true });
    if (branch?.ready && (branch.cluster_name !== env.PLANETSCALE_PREVIEW_SKU || branch.replicas !== 0)) {
      throw new Error("Preview database size differs from the configured single-node SKU");
    }
    return Boolean(branch?.ready);
  }
  const proof = await cf("/hyperdrive/integrationsOperations/planetscale/createDatabaseSignature", { method: "POST" });
  if (proof.account_id !== env.CLOUDFLARE_ACCOUNT_ID) throw new Error("Unexpected Cloudflare billing account");
  await ps("", {
    method: "POST",
    body: {
      name, kind: "postgresql", major_version: "17", region: env.PLANETSCALE_PREVIEW_REGION,
      cluster_size: env.PLANETSCALE_PREVIEW_SKU, replicas: 0,
      cloudflare_account_id: proof.account_id, cloudflare_timestamp: proof.timestamp,
      cloudflare_signature: proof.signature,
    },
  });
  console.log("Preview database provisioning started", number);
  return false;
}

async function setRuntimeSecrets(cf, ps, key, deploy, number, access) {
  const env = {};
  let role;
  const roles = `/${databaseName(number)}/branches/main/roles`;
  if (deploy.env?.DATABASE_URL?.type !== "secret_text") {
    role = await ps(roles, {
      method: "POST", body: { name: "seikika-preview", inherited_roles: ["postgres"] },
    });
    env.DATABASE_URL = { type: "secret_text", text: directDatabaseUrl(role) };
  }
  if (deploy.env?.SEIKIKA_SIGNING_KEY?.type !== "secret_text") {
    const seed = crypto.getRandomValues(new Uint8Array(32));
    const text = Array.from(seed, (byte) => byte.toString(16).padStart(2, "0")).join("");
    env.SEIKIKA_SIGNING_KEY = { type: "secret_text", text };
  }
  for (const [name, text] of Object.entries(access)) {
    if (deploy.env?.[name]?.type !== "secret_text") env[name] = { type: "secret_text", text };
  }
  if (Object.keys(env).length === 0) return false;
  try {
    await cf(key, { method: "PATCH", type: "application/merge-patch+json", body: { env } });
  } catch (error) {
    // Read back the binding before revoking a new role after an ambiguous PATCH response.
    const latest = await cf(key);
    if (role && latest.env?.DATABASE_URL?.type !== "secret_text") {
      await ps(roles + "/" + role.id, { method: "DELETE", allowMissing: true });
    }
    throw error;
  }
  return true;
}

async function provision(clients, env, apps, previews, pr) {
  const { cf, ps, worker } = clients;
  const branch = pr.branch;
  const preview = previews.find((item) => item.name === branch);
  if (!preview) return;
  const runtime = worker + "/" + preview.id;
  const app = await ensureAccess(cf, env, preview, apps, pr.number);
  if (!await provisionDatabase(cf, ps, env, pr.number)) return;
  const latest = runtime + "/deployments/latest";
  const deployment = await cf(latest, { allowMissing: true });
  if (!deployment) return;
  await setRuntimeSecrets(cf, ps, latest, deployment, pr.number, {
    SEIKIKA_PREVIEW_ACCESS_ISSUER: env.CLOUDFLARE_ACCESS_ISSUER,
    SEIKIKA_PREVIEW_ACCESS_AUDIENCE: app.aud,
    SEIKIKA_PREVIEW_ACCESS_ADMIN: env.PREVIEW_ADMIN_EMAIL.split("@")[0],
  });
}

async function cleanup(clients, env, app, number, previews) {
  const { cf, ps, worker } = clients;
  const suffix = `-${env.CLOUDFLARE_WORKER}.${env.WORKER_SUBDOMAIN}.workers.dev`;
  if (!app.domain.endsWith(suffix)) throw new Error("Managed Access app has an unexpected domain");
  const slug = app.domain.slice(0, -suffix.length);
  if (!/^[a-z0-9-]+$/.test(slug)) throw new Error("Managed Access app has an invalid preview slug");
  const preview = previews.find((item) =>
    item.slug === slug && item.urls?.includes("https://" + app.domain));
  if (preview) {
    await cf(worker + "/" + preview.id, { method: "DELETE", allowMissing: true });
  }
  const appId = `${env.CLOUDFLARE_WORKER}_${slug}_SeikikaContainer`;
  for (const container of await cf("/containers/applications")) {
    if (container.name === appId) {
      await cf("/containers/applications/" + container.id, { method: "DELETE", allowMissing: true });
    }
  }
  await ps("/" + databaseName(number), { method: "DELETE", allowMissing: true });
  await cf("/access/apps/" + app.id, { method: "DELETE", allowMissing: true });
  console.log("Managed preview resources removed", number);
}

export async function reconcile(env, pulls) {
  const client = clients(env);
  const [apps, previews] = await Promise.all([
    client.cf("/access/apps?per_page=100"),
    client.cf(client.worker),
  ]);
  let failures = 0;
  const safely = async (number, callback) => {
    try {
      await callback();
    } catch {
      failures++;
      console.error("Managed preview reconciliation failed", number);
    }
  };
  for (const pr of pulls) {
    await safely(pr.number, () => provision(client, env, apps, previews, pr));
  }
  const wanted = new Set(pulls.map((pr) => pr.number));
  for (const app of apps) {
    const marker = namePattern.exec(app.name);
    if (!marker) continue;
    const number = Number(marker[1]);
    if (!wanted.has(number)) await safely(number, () => cleanup(client, env, app, number, previews));
  }
  if (failures) throw new Error(`Managed Preview reconciliation failed for ${failures} PRs`);
}

export default {
  async fetch(incoming, env) {
    if (incoming.method !== "POST" || new URL(incoming.url).pathname !== "/reconcile") {
      return new Response("Not found", { status: 404 });
    }
    if (!await authorized(incoming, env)) return new Response("Unauthorized", { status: 401 });
    const pulls = await incoming.json().catch(() => null);
    if (!Array.isArray(pulls) || !pulls.every((pr) =>
      Number.isSafeInteger(pr?.number) && pr.number > 0 &&
      typeof pr.branch === "string" && pr.branch.length > 0 && pr.branch !== "main"
    )) return new Response("Invalid pull request list", { status: 400 });
    await reconcile(env, pulls);
    return new Response(null, { status: 204 });
  },
};
