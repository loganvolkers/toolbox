import { readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";

const preview = {
  section: "previews.vars",
  team: process.env.SEIKIKA_PREVIEW_CF_TEAM_DOMAIN,
  audience: process.env.SEIKIKA_PREVIEW_CF_AUDIENCE,
  admin: process.env.SEIKIKA_PREVIEW_CF_ADMIN,
  viewer: process.env.SEIKIKA_PREVIEW_CF_VIEWER,
};
const production = {
  section: "vars",
  team: process.env.SEIKIKA_PRODUCTION_CF_TEAM_DOMAIN,
  audience: process.env.SEIKIKA_PRODUCTION_CF_AUDIENCE,
  admin: process.env.SEIKIKA_PRODUCTION_CF_ADMIN,
};
const configured = ({ team, audience, admin, viewer }) => Boolean(team || audience || admin || viewer);
if (configured(preview) && configured(production)) {
  throw new Error("Set Cloudflare Access build variables for one deployment environment");
}

const {
  section,
  team,
  audience,
  admin,
  viewer,
} = configured(production) ? production : preview;
const repositories = process.env.SEIKIKA_PRODUCTION_PUBLIC_REPOSITORIES;
if (repositories !== undefined && (!configured(production) ||
    !/^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+(?:,[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+)*$/.test(repositories))) {
  throw new Error("Public repositories require production Access and comma-separated GitHub owner/repo names");
}

if (team || audience || admin || viewer) {
  const login = admin || viewer;
  const role = admin ? "admin" : "viewer";
  if (!/^https:\/\/[a-z0-9-]+\.cloudflareaccess\.com$/.test(team ?? "") ||
      !/^[a-f0-9]{64}$/.test(audience ?? "") ||
      !/^[A-Za-z0-9._+-]{1,128}$/.test(login ?? "") ||
      Boolean(admin) === Boolean(viewer)) {
    throw new Error("Set the Access team domain, audience, and exactly one admin or viewer login");
  }

  const stage = process.argv[2];
  const configPath = join(stage, "wrangler.toml");
  const portalPath = join(stage, "portal", "seikika.yaml");
  const config = readFileSync(configPath, "utf8");
  const portal = readFileSync(portalPath, "utf8");
  const auth = `[${section}]\nSEIKIKA_AUTH_MODE = "token"\n`;
  const staticTuples = /^    - static:$/m;
  if (!config.split("\n\n").some((block) => `${block}\n` === auth) ||
      !/^  name: seikika-federation$/m.test(portal) ||
      !staticTuples.test(portal)) {
    throw new Error("Access variables require the staged seven-fork portal and Wrangler config");
  }

  writeFileSync(configPath, config.replace(auth, [
    `[${section}]`,
    'SEIKIKA_AUTH_MODE = "cloudflare-access"',
    `SEIKIKA_CF_TEAM_DOMAIN = "${team}"`,
    `SEIKIKA_AUDIENCE = "${audience}"`,
    ...(repositories === undefined ? [] : [`SEIKIKA_PUBLIC_REPOSITORIES = "${repositories}"`]),
    "",
  ].join("\n")));
  writeFileSync(portalPath, portal.replace(staticTuples, [
    "    - static:",
    `      - user: 'user:${login}'`,
    `        relation: ${role}`,
    "        object: portal:seikika-federation",
  ].join("\n")));
}
