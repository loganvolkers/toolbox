// The reference portal (examples/reference-portal/toolbox.yaml) written with the builder.
// `tsx portal/acme.ts` prints the canonical manifest JSON; scripts/roundtrip.ts compares it to the YAML.
import { action, canonical, cel, connector, field, kind, scorecard, toManifest, uiBundle } from "../src/index.js";

// Policies are ordinary TS values, so they can live in a security-owned module (CODEOWNERS on the file).
export const approvedApps = {
  renovate: {
    steward: "group:platform", reviewed: "2026-09-02",
    permissions: { contents: "write", pull_requests: "write", issues: "write", checks: "read", metadata: "read" },
    eligible: "entity.lifecycle != 'deprecated'",
    approval: "entity.tier in ['0', '1'] ? 'security' : 'none'",
  },
  "snyk-security": {
    steward: "group:security", reviewed: "2026-08-14",
    permissions: { contents: "read", metadata: "read", security_events: "write" },
    eligible: "true",
    approval: "'none'",
  },
};

const github = connector("github", {
  package: "builtin:github", runtime: "in-image",
  config: { org: "acme", apiVersion: "2026-03-10", appId: "${env:GITHUB_APP_ID}", privateKey: "${secret:github-app-private-key}" },
  records: ["repository", "codeowners", "app", "installation"],
  refresh: { installation: ["external_ids", "selector"], repository: ["external_ids", "since"] },
  freshness: { repository: "15m", installation: "15m", app: "24h" },
});
const gitFiles = connector("git-files", {
  package: "builtin:git-files", runtime: "in-image",
  config: { org: "acme", paths: ["catalog-info.yaml"] },
  records: ["catalog-info"], freshness: { "catalog-info": "15m" },
});
const idp = connector("idp", {
  package: "builtin:scim", runtime: "remote",
  records: ["user", "group"], freshness: { user: "1h", group: "1h" },
});

const Group = kind("Group", {
  identityFrom: [idp.source],
  fields: { title: field.string(), parent: field.ref({ name: "Group" }), githubTeam: field.string(), members: field.list("string") },
  sources: { default: [idp.source] },
  map: [{
    from: idp.record("group"),
    id: cel`'group:' + record.slug`,
    set: cel`{'title': record.displayName, 'parent': has(record.parent) ? 'group:' + record.parent : null,
              'githubTeam': record.githubTeam, 'members': record.members}`,
  }],
});

const User = kind("User", {
  identityFrom: [idp.source],
  fields: { displayName: field.string(), email: field.string({ facet: "contact" }) },
  sources: { default: [idp.source] },
  map: [{ from: idp.record("user"), id: cel`'user:' + record.login`, set: cel`{'displayName': record.name, 'email': record.email}` }],
});

const catalogInfo = "git:catalog-info" as const;
const Repository = kind("Repository", {
  identityFrom: [github.source],
  fields: {
    title: field.string(), archived: field.boolean(), defaultBranchProtected: field.boolean(),
    lifecycle: field.enum(["experimental", "production", "deprecated"]),
    tier: field.enum(["0", "1", "2", "3"]),
    ownerClaim: field.ref(Group),
    codeowners: field.list("object"),
    admins: field.list("string"),
    ownership: field.codeowners({ claim: "ownerClaim", rules: "codeowners", path: "catalog-info.yaml", teamField: "Group.githubTeam" }),
    installedApps: field.list("string", { computed: cel`inbound.installedOn.map(i, i.app)` }),
  },
  sources: {
    default: [github.source],
    fields: {
      title: [catalogInfo, github.source],
      lifecycle: [github.source, catalogInfo],
      tier: [catalogInfo, "native"],
      ownerClaim: [catalogInfo],
    },
  },
  writeBack: { "ownership.owner": "github:custom-property/owner", tier: "github:custom-property/tier" },
  map: [
    {
      from: github.record("repository"), id: cel`'repository:' + record.name`,
      set: cel`{'title': record.name, 'archived': record.archived, 'defaultBranchProtected': record.default_branch_protected,
                'lifecycle': record.archived ? 'deprecated' : null, 'admins': record.admins}`,
    },
    { from: github.record("codeowners"), id: cel`'repository:' + record.repo`, set: cel`{'codeowners': record.rules}` },
    {
      from: gitFiles.record("catalog-info"), source: catalogInfo, id: cel`'repository:' + record.repo`,
      set: cel`{'title': has(record.content.metadata.title) ? record.content.metadata.title : null,
                'lifecycle': has(record.content.spec.lifecycle) ? record.content.spec.lifecycle : null,
                'tier': has(record.content.spec.tier) ? record.content.spec.tier : null,
                'ownerClaim': has(record.content.spec.owner) ? record.content.spec.owner : null}`,
    },
  ],
});

const GitHubApp = kind("GitHubApp", {
  identityFrom: [github.source],
  fields: { slug: field.string(), vendor: field.string(), permissions: field.object() },
  sources: { default: [github.source] },
  map: [{ from: github.record("app"), id: cel`'githubapp:' + record.slug`,
          set: cel`{'slug': record.slug, 'vendor': record.owner, 'permissions': record.permissions}` }],
});

const GitHubAppInstallation = kind("GitHubAppInstallation", {
  identityFrom: [github.source],
  fields: { app: field.string(), selection: field.enum(["all", "selected"]) },
  relations: { installedOn: { to: Repository.name, many: true } },
  sources: { default: [github.source] },
  map: [{
    from: github.record("installation"), id: cel`'installation:' + string(record.id)`,
    set: cel`{'app': record.app_slug, 'selection': record.repository_selection}`,
    relations: { installedOn: cel`record.repositories.map(r, 'repository:' + r)` },
  }],
});

const R = Repository.f; // typed CEL field paths: R.archived === "entity.archived"
const ownership = scorecard("ownership", {
  kind: Repository,
  rationale: `The owner is who answers for a repository and, through the derived owner tuple, who may see its
    owner-only details. Bronze accepts an inferred owner; silver requires a claim backed by CODEOWNERS, a tier that
    approvals depend on, and a protected branch, since a claim is only as safe as the branch that holds it.`,
  links: [
    { title: "CODEOWNERS-backed claims",
      url: "https://github.com/loganvolkers/toolbox/blob/main/docs/authorization.md#48-authority-of-permission-relevant-claims-codeowners" },
    { title: "D17 CODEOWNERS authority",
      url: "https://github.com/loganvolkers/toolbox/blob/main/docs/decisions/D01M7TCJ3280000000000000000.md" },
  ],
  owner: "group:platform", levels: ["bronze", "silver"],
  appliesTo: cel`!${R.archived}`,
  rules: [
    { name: "has-owner", level: "bronze", check: cel`${R.ownership}.status in ['verified', 'inferred']`,
      fix: "Add catalog-info.yaml with spec.owner, backed by CODEOWNERS for that file.",
      rationale: "A repository nobody owns has nobody to fix its failing standards, approve changes to it or answer for an incident." },
    { name: "claim-verified", level: "silver", check: cel`${R.ownership}.status == 'verified'`,
      fix: "Declare spec.owner in catalog-info.yaml and make that team a code owner of the file.",
      rationale: `Ownership grants permissions, so anyone who can edit catalog-info.yaml could otherwise name any team as
        owner. Requiring that team to own the file in CODEOWNERS means the claimed team reviews the claim.` },
    { name: "has-tier", level: "silver", check: cel`has(${R.tier})`,
      fix: "Set spec.tier in catalog-info.yaml, or ask the platform team to set it in Toolbox.",
      rationale: `The tier decides whether installing an approved GitHub App on the repository needs security's approval;
        without one, the policy cannot tell a critical repository from a sandbox.` },
    { name: "branch-protected", level: "silver", check: cel`${R.defaultBranchProtected}`,
      fix: "Enable a ruleset protecting the default branch.",
      rationale: `Without a ruleset, anyone with write access can push to the default branch unreviewed, including the
        CODEOWNERS and catalog-info.yaml that ownership rests on.`,
      links: [{ title: "GitHub rulesets",
        url: "https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-rulesets/about-rulesets" }] },
  ],
  facets: { summary: "public", details: "owner" },
});
const appsExample = { title: "Approved GitHub Apps example",
  url: "https://github.com/loganvolkers/toolbox/blob/main/docs/examples/approved-github-apps.md" };
const hygiene = scorecard("github-app-hygiene", {
  kind: Repository,
  rationale: `Every GitHub App installed on a repository acts on it with its own token, so each installation widens who
    can read or change the code. Security keeps the approved list in policy, and a single bronze rule is enough because
    any install outside it is a finding.`,
  links: [appsExample],
  owner: "group:security", levels: ["bronze"], appliesTo: cel`!${R.archived}`,
  rules: [{ name: "only-approved-apps", level: "bronze", check: cel`${R.installedApps}.all(a, a in policy.approvedApps)`,
            fix: "Remove unapproved apps, or request approval with the request-new-app action.",
            rationale: `An app installed outside the approved list has had no security review of its permissions or
              vendor. Toolbox's own app gets no webhook for other apps' installs, so this check is how such drift is caught.` }],
  facets: { summary: "public", details: "owner" },
});
const appApproval = scorecard("app-approval", {
  kind: GitHubApp,
  rationale: `Security approves an app's permissions as they were when it was reviewed. If the app later asks for more,
    the approval no longer describes what the app can do.`,
  links: [appsExample],
  owner: "group:security", levels: ["bronze"], appliesTo: cel`${GitHubApp.f.slug} in policy.approvedApps`,
  rules: [{ name: "permissions-match-snapshot", level: "bronze",
            check: cel`${GitHubApp.f.permissions} == policy.approvedApps[${GitHubApp.f.slug}].permissions`,
            fix: "The app requests permissions beyond its approved snapshot; security must re-approve it.",
            rationale: `A vendor can widen an app's permissions after approval. Until security re-reviews them,
              self-service installs would grant access nobody approved.` }],
  facets: { summary: "public", details: "public" },
});

const installUi = uiBundle({ name: "install-approved-app", path: "ui/install-approved-app", entry: "dist/index.js",
                             framework: "react", mounts: [{ slot: "action:install-approved-app" }] });
const executor = { type: "in-image", command: ["/portal/executors/github-apps/bin/github-apps"], identity: "user",
                   secrets: ["github-app-private-key"] } as const;
const installationOf = { installation: { kind: GitHubAppInstallation, where: cel`candidate.app == inputs.app` } };

const install = action("install-approved-app", {
  kind: Repository, scope: "day2",
  description: "Add this repository to a security-approved GitHub App already installed in the org.",
  inputs: { type: "object", required: ["app", "justification"], properties: {
    app: { type: "string", enumFrom: "policy.approvedApps.filter(a, !(a in entity.installedApps))" },
    justification: { type: "string", minLength: 10 } } },
  permission: "can_install_apps",
  approval: cel`policy.approvedApps[inputs.app].approval == 'security' ? ['group:security'] : []`,
  context: installationOf,
  preconditions: [cel`inputs.app in policy.approvedApps`, cel`!${R.archived}`],
  executor,
  expect: { postcondition: cel`inputs.app in ${R.installedApps}`, within: "2m",
            refresh: [{ connector: github, kind: "installation", selector: cel`app == inputs.app` }] },
  annotations: { destructive: false, idempotent: true, openWorld: true },
  mcp: { promote: "install_approved_github_app" },
  ui: { bundle: installUi },
});
const remove = action("remove-app", {
  kind: Repository, scope: "day2", description: "Remove this repository from a GitHub App installation.",
  inputs: { type: "object", required: ["app"], properties: { app: { type: "string", enumFrom: "entity.installedApps" } } },
  permission: "can_install_apps",
  context: installationOf,
  executor,
  expect: { postcondition: cel`!(inputs.app in ${R.installedApps})`, within: "2m",
            refresh: [{ connector: github, kind: "installation", selector: cel`app == inputs.app` }] },
  annotations: { destructive: true, idempotent: true, openWorld: true },
});

export const manifest = toManifest({
  portal: { name: "acme", title: "Acme developer portal", owner: "group:platform", toolbox: "0.1" },
  policies: { approvedApps },
  connectors: [github, gitFiles, idp],
  kinds: [Group, User, Repository, GitHubApp, GitHubAppInstallation],
  scorecards: [ownership, hygiene, appApproval],
  authz: {
    model: "authz/model.fga", tests: ["authz/model.fga.yaml"],
    tuples: [
      { static: [{ user: "user:*", relation: "viewer", object: "portal:acme" },
                 { user: "group:platform#member", relation: "admin", object: "portal:acme" }] },
      { kind: Group, emit: cel`entity.members.map(m, {'user': 'user:' + m, 'relation': 'member', 'object': entity.id})` },
      { kind: Group, emit: cel`has(entity.parent) ? [{'user': entity.id + '#member', 'relation': 'member', 'object': entity.parent}] : []` },
      { kind: Repository, emit: cel`[{'user': 'portal:acme', 'relation': 'portal', 'object': entity.id}]
          + (${R.ownership}.status in ['verified', 'inferred'] ? [{'user': ${R.ownership}.owner, 'relation': 'owner', 'object': entity.id}] : [])
          + ${R.admins}.map(a, {'user': 'user:' + a, 'relation': 'gh_admin', 'object': entity.id})` },
    ],
  },
  actions: [install, remove],
  ui: { bundles: [installUi] },
  mcp: { instructions: "Acme's developer portal. Use it to find owners, check standards and run approved self-service actions. Always preview an action and get the user's confirmation before running it." },
  tests: ["tests/ownership.test.yaml", "tests/apps.test.yaml"],
});

if (import.meta.url === `file://${process.argv[1]}`) process.stdout.write(canonical(manifest));
