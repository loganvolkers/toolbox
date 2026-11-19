# Worked Example: Self-Service Installation of Approved GitHub Apps

*Status: example design, 2026-11-19. Shows how a **Toolbox operator** builds a governed self-service
action on Toolbox, using GitHub's 13 Oct 2026 App changes, and how they direct a coding agent to
build, test and ship it to the API, CLI and MCP. Source: the GitHub changelog entry "GitHub App
improvements for least privilege and management", read via its GitHub-hosted mirror because
github.blog is not reachable from this environment.*

---

## Part 1. What GitHub changed, and why it enables this

From the changelog (13 Oct 2026, REST API version `2026-03-10`; GHES 3.24):

| Change | What it means here |
|---|---|
| New repository-level fine-grained permission **`installation_repositories:write`**, for GitHub Apps and fine-grained PATs | A caller holding it on a repository can **add or remove that repository from any app installation in the org**. A *user* caller must also be an org owner or an admin of that repository |
| The **Add/Remove a repository to/from an app installation** APIs now accept Apps and fine-grained PATs (previously only the classic `repo` scope) | Automation can manage which repos an app sees **without an admin's broad PAT** |
| **Repo admins can add their repo to an app already installed in the org**, even if the app has organization-level permissions | The org owner's first install is the approval. Adding more repos doesn't widen the app's org-level access |
| **Three sensitive permission sets still need owner approval:** org-level permissions, repository administration, repository creation | Toolbox can't (and shouldn't) bypass owner approval for *new* apps or broader permissions |
| Orgs can **restrict app installation to owners** | An org can make Toolbox the governed path instead of the GitHub UI (see §1.6) |
| **Get an app** now returns private and internal apps that share ownership | Toolbox can read the *exact permissions and events* of every approved app for audit and drift detection |
| **`offline_access`** for GitHub Apps (refresh tokens with expiring user tokens) | Toolbox can act **as the requesting user** across a long approval wait without long-lived tokens |

## Part 2. The feature plan

### 2.1 Goal

A repository admin, or an agent acting for one, can add their repository to a **security-approved**
GitHub App in one step from the API, CLI or MCP:
- with **eligibility rules and approvals** defined in code
- with a **dry-run** that shows exactly what the app will be able to access
- **audited**, **reversible**, and **continuously checked** for drift

### 2.2 Capabilities

1. **Catalog of apps and installations** (the GitHub connector):
   - `GitHubApp` entities: slug, owner, **permissions and events** (from Get an app).
   - `GitHubAppInstallation` entities: installation id, repository selection, suspended status.
   - Relations: `installedOn` (installation → repository) and `admin` (user → repository), the latter
     from collaborator permissions.
   - Kept fresh by webhooks (`installation`, `installation_repositories`) plus periodic full syncs.
2. **An approved-apps policy, in code, owned by security** (a module under CODEOWNERS
   `@acme/security`). Per app:
   - **Approved permission snapshot.** If the app later requests more permissions, self-service for
     that app suspends automatically until security re-approves.
   - **Eligibility** (CEL over the repository), e.g. `repo.spec.lifecycle != 'deprecated'`, or "not PCI
     scope unless the app is PCI-approved".
   - **Approval policy:** automatic for most repos; security approval for tier-0/1 or PCI repos.
   - **Grant expiry and review cadence** (optional), plus per-team quotas.
   - **Prerequisite check:** the app must already be installed in the org (owner-approved). `toolbox
     plan` fails if an approved app has no org installation.
3. **Self-service actions** scoped to `Repository` (day-2):
   - `install-approved-app` (inputs: `app`, chosen from apps *eligible for this repo*; `justification`)
   - `remove-app`
   - `request-new-app`, which opens a security review issue. New apps need an owner install and a
     policy PR, which is out of scope for self-service.
4. **Authorization (ReBAC):** `can_install_apps` on a repository = GitHub repo admin, or platform
   admin. Agents need a ceiling grant (`agent:claude-code` may run it, but only on behalf of a user).
5. **Execution as a durable workflow:**
   1. Re-check eligibility and permission (state may have changed during an approval wait).
   2. Call GitHub's "Add a repository to an app installation" API.
   3. Confirm through the webhook or a read-back.
   4. Record the action-sourced fragment.
   5. Notify.
6. **Two identity modes** for the GitHub call:
   - **`user` (default):** an on-behalf-of user token via the Toolbox GitHub App's user authorization
     with `offline_access`, so GitHub *also* enforces "must be repo admin" (defence in depth).
   - **`service`:** the Toolbox App's own token with `installation_repositories:write`, for orgs that
     restrict installs to owners and want Toolbox as the only path (§2.6).
7. **Continuous governance:**
   - A scorecard, `github-app-hygiene`, with rules for: only approved apps installed; installation
     permissions match the approved snapshot; no grants past expiry; no unused installations.
   - **Drift automation:** an unapproved app added outside Toolbox notifies the repo owner and security
     (optional auto-remove in `enforce` mode).
8. **Notifications:** the repo owner, the app's steward and security get the run result; the repo's
   living dashboard issue shows its installed apps and their status.

### 2.3 Non-goals

- Approving *new* apps or broader permissions (that remains an org-owner decision in GitHub).
- Installing apps with `repository_creation` or `administration` permissions via self-service: these
  are always sensitive and need an owner.

### 2.4 Interfaces (generated from one definition)

| Surface | Shape |
|---|---|
| **API** | `POST /v1/actions/install-approved-app/runs {entity: "repository:default/payments-api", inputs: {app: "renovate", justification: "..."}, dry_run: true}`; typed in the portal's generated OpenAPI |
| **CLI** | `toolbox run install-approved-app --entity repository:default/payments-api --app renovate --dry-run`, then without `--dry-run`; `toolbox runs watch <id>` |
| **MCP** | Discoverable via `list_actions(ref)`. **Promoted to a first-class tool** `install_approved_github_app` (annotations: `readOnlyHint: false`, `destructiveHint: false`, `idempotentHint: true`, `openWorldHint: true`). Elicitation asks for a missing justification; long approvals use MCP tasks |
| **UI** | A custom action UI (tier 3, [frontend.md](../design/frontend.md) §1, §3): repository and app comboboxes with rich, permission-shaped options and disabled reasons, a live dry-run panel, the approval path, and an inline run timeline with undo. Covered by page-level Storybook stories per persona |

### 2.5 Dry-run output (what humans and agents see before acting)

```
install-approved-app (dry run) — repository:default/payments-api ← app: renovate
  eligibility      ✓ lifecycle=production, tier=2, not PCI
  current state    renovate installed in org acme (installation 4412), selection=selected, 37 repos
  will grant       contents:write, pull_requests:write, issues:write, checks:read, metadata:read
  approved snapshot ✓ matches (reviewed 2026-09-02 by group:security)
  approval         none required (tier 2)
  identity         as user:alice (repo admin ✓ in GitHub)
  effect           PUT add repository 81723 to installation 4412  (idempotent)
```

### 2.6 Recommended org setup

- **Option A (lighter):** leave GitHub's default (repo admins may add approved apps). Toolbox adds
  eligibility, approvals, audit, MCP and drift detection on top. Drift detection catches installs
  made outside Toolbox.
- **Option B (strict):** restrict app installation to org owners in GitHub, and run the action in
  `service` mode, making Toolbox the only self-service path. *To verify in the spike:* that the owner
  restriction applies to users but not to an App holding `installation_repositories:write`. The
  changelog implies this but doesn't state it.

### 2.7 Platform gaps this example exposes (added to the plan)

| Gap | Resolution |
|---|---|
| Action inputs whose options depend on the target entity | `tb.schema.enumFrom(query)`: options computed by the circuit per entity (eligible apps for this repo) and shown in dry-run, MCP schema hints and the UI |
| Acting as the user across long approval waits | Executor identity `user` with refresh tokens (`offline_access`) stored in the secrets vault, refreshed inside durable workflow steps |
| Pinning and diffing an external permission set | `approvedSnapshot` values in code + a circuit rule comparing them to the connector's `GitHubApp.permissions` |
| GitHub API version pinning | Connector config `apiVersion: "2026-03-10"`; `plan` warns on version changes |

## Part 3. What the operator's code looks like

**Repository layout.** The new module is owned by security, and the executor lives in the GitHub
connector package:

```
modules/github-apps/
├── index.ts               # kinds/mappings (if not already shipped), policy, actions, scorecard, automations
├── approved-apps.ts       # the approved list (security-owned)
├── authz/github-apps.fga  # extend type repository; can_install_apps
├── authz/github-apps.fga.yaml
├── tests/
│   ├── fixtures/*.catalog.yaml        # repos, collaborators, installations
│   ├── cassettes/*.json               # recorded GitHub API responses
│   ├── eligibility.test.ts            # table-driven CEL policy tests
│   └── actions.test.yaml              # dry-run snapshots + simulated runs
└── README.md
CODEOWNERS: /modules/github-apps/ @acme/security
```

```ts
// modules/github-apps/approved-apps.ts
export const approvedApps = {
  renovate: {
    appSlug: "renovate",
    steward: "group:default/platform",
    approvedSnapshot: { permissions: { contents: "write", pull_requests: "write", issues: "write",
                                       checks: "read", metadata: "read" }, reviewed: "2026-09-02" },
    eligible: "repo.spec.lifecycle != 'deprecated'",
    approval: "repo.spec.tier in ['0','1'] ? 'security' : 'none'",
  },
  "snyk-security": {
    appSlug: "snyk-security",
    steward: "group:default/security",
    approvedSnapshot: { permissions: { contents: "read", metadata: "read", security_events: "write" }, reviewed: "2026-08-14" },
    eligible: "true",
    approval: "'none'",
  },
} as const;
```

```ts
// modules/github-apps/index.ts (abridged)
import * as tb from "@toolbox/sdk";
import { github } from "@toolbox/connector-github";
import { approvedApps } from "./approved-apps";

const mod = new tb.Module("github-apps", { owner: "group:default/security" });
const { repository, app, installation } = github.kinds;          // shipped by the connector package

const policy = tb.policyTable("approved-github-apps", approvedApps);  // typed; CEL fields type-checked

new tb.Action("install-approved-app", {
  scope: { day2: repository },
  description: "Add this repository to a security-approved GitHub App already installed in the org.",
  inputs: tb.schema.object({
    app: tb.schema.enumFrom(policy.eligibleFor("entity")),     // per-repo options from the circuit
    justification: tb.schema.string({ minLength: 10 }),
  }),
  policy: { permission: "can_install_apps" },
  approvals: { when: policy.approvalFor("entity", "inputs.app"), approvers: "security from portal" },
  preconditions: [ policy.snapshotMatches("inputs.app"), policy.installedInOrg("inputs.app") ],
  executor: github.act.addRepositoryToInstallation({ identity: "user", apiVersion: "2026-03-10" }),
  dryRun: "native",
  annotations: { destructive: false, idempotent: true, openWorld: true },
  mcp: { promote: "install_approved_github_app" },
});

new tb.Action("remove-app", { scope: { day2: repository }, /* … */
  executor: github.act.removeRepositoryFromInstallation({ identity: "user" }),
  annotations: { destructive: true, idempotent: true } });

new tb.Scorecard("github-app-hygiene", {
  appliesTo: repository, owner: "group:default/security",
  levels: ["bronze", "silver"],
  rules: [
    tb.rule("only-approved-apps", { level: "bronze",
      check: "entity.installations.all(i, i.app.slug in policy.approvedSlugs)" }),
    tb.rule("permissions-match-snapshot", { level: "silver",
      check: "entity.installations.all(i, policy.snapshotMatches(i.app))" }),
  ],
});

new tb.Automation("unapproved-app-drift", {
  on: tb.on.ruleFails("github-app-hygiene/only-approved-apps"),
  run: tb.notify({ to: ["owner", "group:default/security"] }),     // `enforce` mode would run remove-app
});
```

```
# modules/github-apps/authz/github-apps.fga
module github-apps

extend type repository
  relations
    define admin: [user]                                       # derived from GitHub collaborator permission=admin
    define can_install_apps: admin or admin from portal
```

The executor is a connector-protocol `act` handler in the GitHub connector package. It's TypeScript,
running out of process, with durable steps:
1. Call the "Add a repository to an app installation" endpoint (path per GitHub's REST docs for API
   version `2026-03-10`).
2. Treat "already added" as success (idempotent).
3. Map 403/404/422 to typed errors.
4. Respect rate limits through the connector's `rate_limit` hints.
5. Return an action-sourced fragment (`installedOn` edge with provenance).

## Part 4. How the operator directs the agentic coding harness

### 4.1 Preconditions in the portal repo

These are already present from `toolbox init`:
- `AGENTS.md`
- Toolbox Agent Skills (`add-action`, `add-permission`, `add-scorecard`, `extend-connector`)
- the authoring MCP server (`toolbox mcp --dev`: `validate`, `test`, `plan`, `explain_error`,
  `preview_url`, `docs_search`)
- a GitHub **sandbox org** with a test app, and credentials in the CI environment

### 4.2 The brief the operator gives the agent

> **Task:** Implement self-service installation of approved GitHub Apps, as specified in
> `docs/examples/approved-github-apps.md` Part 2. Put it in a new module `modules/github-apps`
> (owner `group:default/security`).
>
> **Work test-first, in this order:**
> 1. Write the property list for this feature in `spec/properties.md` (at least: only repo admins
>    or portal admins can run it; never executes without required approvals; permission and
>    eligibility are re-checked at execution; idempotent; drift is detected within one sync).
> 2. Write failing tests:
>    - `authz/github-apps.fga.yaml`: admin can, non-admin can't, agent without a user can't.
>    - Catalog fixtures + recorded cassettes for apps, installations and collaborators.
>    - Table-driven eligibility tests for every approved app.
>    - Dry-run snapshot tests.
>    - Simulated-run tests with the fake GitHub: 403, 422, rate limits, duplicate webhooks, crash
>      between steps, and admin rights removed during an approval wait.
> 3. Extend the GitHub connector package only if it lacks the `app`/`installation`/collaborator
>    record kinds or the `act` handlers; use the `extend-connector` skill.
> 4. Implement until `toolbox test` and `toolbox simulate --seeds 2000 --module github-apps` are green.
>    Use `explain_error` for any failure.
> 5. Run `toolbox plan --remote prod` and summarise the plan in the PR description. Required sections:
>    new actions, MCP surface (+`install_approved_github_app`), authz diff, declassifications (expect
>    none), notification volume, meta-catalog owners.
> 6. Open the PR. Don't merge. Request review from `@acme/security` and `@acme/platform`.
>
> **Constraints:**
> - Pin GitHub API version `2026-03-10`.
> - Default identity mode `user`.
> - Ship in `pilot` mode for `group:default/platform` and `group:default/payments` only.
> - No secrets in code; use `tb.secret("vault:…")`.

### 4.3 What the agent's loop looks like (and what Toolbox gives it)

| Step | Agent action | Toolbox support |
|---|---|---|
| Understand | Reads `AGENTS.md`, the skills and the example doc; calls `docs_search("enumFrom")` | Docs and skills ship in the binary |
| Red | Writes tests; `toolbox test` fails as expected | Typed SDK, fixtures format, `.fga.yaml` (the same OpenFGA engine as production) |
| Green | Implements the module and connector handler; iterates on `validate` and `explain_error` diagnostics | Typed CEL errors point at the exact expression and field, with fixes |
| Simulate | `toolbox simulate` with the fake GitHub and fault injection; replays failing seeds with `--seed` | Deterministic, reproducible failures |
| Integrate | Runs the sandbox-org integration test (real GitHub API) via the CI job | Catches API reality the simulation can't model |
| Plan | `toolbox plan --remote prod`; pastes the summary | Full plan on the PR (the portal repo is the access boundary) |
| PR | Opens the PR | The plan check, gates and ownership-aware review (security owns the module; platform owns `Kind:Repository` because the authz module extends it) |

### 4.4 CI, preview, deploy and verify (no human steps beyond review)

1. **PR workflow** (`toolbox-dev/workflows/portal.yml`):
   - test → remote plan with gates → build → **preview environment** (synthetic org + sandbox GitHub
     connection).
   - The agent runs MCP smoke tests against the preview: `list_actions`, a dry-run of
     `install_approved_github_app`, and a real run against the sandbox org.
2. **Review:** security approves the policy and module; platform approves the repository-kind
   extension; the `authz-loss`/`authz-gain` gate shows "+N users gain `can_install_apps` on repos
   they already administer in GitHub".
3. **Merge to main:**
   - build → attest + sign → the customer's deploy hook.
   - The new circuit epoch builds in the background and cuts over.
   - `toolbox verify` checks that the MCP tool list includes `install_approved_github_app` and that the
     `github-app-hygiene` scorecard is evaluating.
   - Rollback is automatic on failure.
4. **Pilot → GA:** the pilot teams use it. The agent watches the analytics events
   (`action.run.transitioned`, `mcp.tool.called`) and the scorecard. A follow-up PR flips `pilot` to
   `on`, and a separate PR switches drift handling to `enforce` once false-positive rates are known.

### 4.5 What a repo admin's agent does after GA

> *"Add Renovate to payments-api."*
> → MCP `install_approved_github_app(repo="payments-api", app="renovate", dry_run=true)`
> → shows the dry-run (§2.5)
> → user confirms in the client
> → run (as the user)
> → completes, and the repo's dashboard issue lists Renovate as installed and approved.

## Sources

- GitHub changelog, 13 Oct 2026: "GitHub App improvements for least privilege and management"
  ([github.blog](https://github.blog/changelog/2026-10-13-github-app-improvements-for-least-privilege-and-management/);
  text read from the [changelog mirror issue](https://github.com/forks-felickz/github-changelog-reader/issues/731))
- [GitHub roadmap #1376](https://github.com/github/roadmap/issues/1376)
- GitHub docs referenced by the changelog: "Add a repository to an app installation" and "Remove a
  repository from an app installation" (REST, API version 2026-03-10); "Restricting GitHub App
  installation to organization owners"; "Refreshing user access tokens"
