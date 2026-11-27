import { readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";

const {
  SEIKIKA_PREVIEW_CF_TEAM_DOMAIN: team,
  SEIKIKA_PREVIEW_CF_AUDIENCE: audience,
  SEIKIKA_PREVIEW_CF_ADMIN: admin,
  SEIKIKA_PREVIEW_CF_VIEWER: viewer,
} = process.env;

if (team || audience || admin || viewer) {
  const login = admin || viewer;
  const role = admin ? "admin" : "viewer";
  if (!/^https:\/\/[a-z0-9-]+\.cloudflareaccess\.com$/.test(team ?? "") ||
      !/^[a-f0-9]{64}$/.test(audience ?? "") ||
      !/^[A-Za-z0-9._+-]{1,128}$/.test(login ?? "") ||
      Boolean(admin) === Boolean(viewer)) {
    throw new Error("Set the Preview Access team domain, audience, and exactly one admin or viewer login");
  }

  const stage = process.argv[2];
  const configPath = join(stage, "wrangler.toml");
  const portalPath = join(stage, "portal", "seikika.yaml");
  const config = readFileSync(configPath, "utf8");
  const portal = readFileSync(portalPath, "utf8");
  const previewAuth = /^\[previews\.vars\]\nSEIKIKA_AUTH_MODE = "token"$/m;
  const staticTuples = /^    - static:$/m;
  if (!previewAuth.test(config) ||
      !/^  name: seikika-federation$/m.test(portal) ||
      !staticTuples.test(portal)) {
    throw new Error("Preview Access variables require the staged seven-fork portal and Wrangler config");
  }

  writeFileSync(configPath, config.replace(previewAuth, [
    "[previews.vars]",
    'SEIKIKA_AUTH_MODE = "cloudflare-access"',
    `SEIKIKA_CF_TEAM_DOMAIN = "${team}"`,
    `SEIKIKA_AUDIENCE = "${audience}"`,
  ].join("\n")));
  writeFileSync(portalPath, portal.replace(staticTuples, [
    "    - static:",
    `      - user: 'user:${login}'`,
    `        relation: ${role}`,
    "        object: portal:seikika-federation",
  ].join("\n")));
}
