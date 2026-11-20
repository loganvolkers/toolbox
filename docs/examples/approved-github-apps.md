# Worked Example: Self-Service Installation of Approved GitHub Apps

*Status: example design, 2026-11-19. Shows how a **Toolbox operator** builds a governed self-service
action on Toolbox, using GitHub's 13 Oct 2026 App changes, and how they direct a coding agent to
build, test and ship it to the API, CLI and MCP. Source: the GitHub changelog entry "GitHub App
improvements for least privilege and management", read via its GitHub-hosted mirror because
github.blog is not reachable from this environment.*

> **Updated 2026-11-20** for the v1 baseline (D33–D49) and checked against GitHub's REST OpenAPI
> description. The runnable version of this example, with its manifest, authorization model,
> fixtures, protocol transcripts and expected outputs, is the
> [reference portal](../../examples/reference-portal/README.md). Execution now follows
> [action-runs.md](../design/action-runs.md) (preview → confirm → dispatch → verify), and the executor
> is an in-image executor (D40). §2.6 records what the API description confirms and what it
> doesn't.

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
- with a **preview** that shows exactly what the app will be able to access, and a run that only
  counts as done once the change is observed
- **audited**, **reversible**, and **continuously checked** for drift

### 2.2 Capabilities

1. **Catalog of apps and installations** (the GitHub connector):
   - `GitHubApp` entities: slug, owner, **permissions and events** (from Get an app).
   - `GitHubAppInstallation` entities: installation id, repository selection, suspended status.
   - Relations: `installedOn` (installation → repository) and `admin` (user → repository), the latter
     from collaborator permissions.
   - **Kept fresh by API reads, not webhooks.** GitHub delivers `installation` and
     `installation_repositories` webhooks only to the app being installed (`supported-webhook-types:
     ["app"]` in the API description), so Toolbox's own app never sees other apps' installs. The
     connector reads installations on a freshness schedule and on scoped refresh commands
     ([connector-control.md](../design/connector-control.md)); which credential can list another app's
     installation repositories is open (Q14, §2.6).
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
5. **Execution: preview → confirm → dispatch → verify** ([action-runs.md](../design/action-runs.md)):
   1. **Preview** (Toolbox layer: eligibility, permission, approval path; executor layer: the grant
      with the app's current permissions, and a warning if they drifted from the approved snapshot).
   2. **Confirm** with the token bound to that preview; wait for approval if required.
   3. **Dispatch:** Toolbox re-checks permission and preconditions, then hands the run to the in-image
      executor, which must not exceed the confirmed effects.
   4. **Verify:** a scoped `refresh` of the app's installation, then the postcondition
      `inputs.app in entity.installedApps` at the resulting snapshot. Only then is the run
      `succeeded`; if nothing can observe the change, `unverified`.
   5. Record the action-sourced fragment and report the result.
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
| **API** | `POST /v1/actions/install-approved-app/previews {entity: "repository:checkout-web", inputs: {app: "snyk-security", justification: "..."}}` → effects + `confirm_token`; then `POST /v1/actions/install-approved-app/runs {…, confirm_token}`; `GET /v1/runs/{id}/events` |
| **CLI** | `toolbox run install-approved-app --entity repository:checkout-web --app snyk-security --preview`, then `--confirm <token>`; `toolbox runs watch <id>` |
| **MCP** | `list_actions(id)`, `action_options`, `preview_action`, `run_action`; **promoted** to `install_approved_github_app` (annotations: `destructiveHint: false`, `idempotentHint: true`, `openWorldHint: true`). The first call returns the preview and token; the client confirms with the user; long runs and approvals are MCP tasks |
| **UI** | A custom action UI bundle mounted through the UI extension contract ([reference spec](../../examples/reference-portal/ui/install-approved-app/README.md)): rich app combobox with disabled reasons, the live preview, the approval path, a confirm button bound to the token, and a run timeline that includes "checking the result" |

### 2.5 Preview output (what humans and agents see before acting)

```
install-approved-app (preview) — repository:default/payments-api ← app: renovate
  eligibility      ✓ lifecycle=production, tier=2, not PCI
  current state    renovate installed in org acme (installation 4412), selection=selected, 37 repos
  will grant       contents:write, pull_requests:write, issues:write, checks:read, metadata:read
  approved snapshot ✓ matches (reviewed 2026-09-02 by group:security)
  approval         none required (tier 2)
  identity         as user:alice (repo admin ✓ in GitHub)
  effect           PUT /user/installations/4412/repositories/81723  (idempotent)
  will verify      refresh installation 4412, then: 'renovate' in entity.installedApps  (within 2m)
  confirm token    valid 10 min for this exact preview
```

### 2.6 Recommended org setup

- **Option A (lighter):** leave GitHub's default (repo admins may add approved apps). Toolbox adds
  eligibility, approvals, audit, MCP and drift detection on top. Drift detection catches installs
  made outside Toolbox.
- **Option B (strict):** restrict app installation to org owners in GitHub, and run the action in
  `service` mode, making Toolbox the only self-service path. *To verify in the spike:* that the owner
  restriction applies to users but not to an App holding `installation_repositories:write`. The
  changelog implies this but doesn't state it.

**What GitHub's REST OpenAPI description confirms** (`github/rest-api-description` at `c9c95fe`,
read 2026-11-20) [P]:

| Question | Finding |
|---|---|
| Add endpoint (Q8) | `PUT /user/installations/{installation_id}/repositories/{repository_id}` (`apps/add-repo-to-installation-for-authenticated-user`), responses 204/304/403/404; `enabledForGitHubApps: true` |
| Remove endpoint | `DELETE` on the same path; requires the installation's `repository_selection` to be `selected` |
| New `installation_repositories` permission and `offline_access` | **Not in the description yet** (the strings appear only as webhook event names). Which token types the 2026-03-10 behaviour accepts must be verified in the sandbox |
| Webhooks for other apps' installs | `installation` and `installation_repositories` are app-only webhooks, so Toolbox can't rely on them for verification (Q14) |
| Reading an app | `GET /apps/{app_slug}` (usable by GitHub Apps) returns permissions and events |
| Listing installations | `GET /orgs/{org}/installations` needs an org owner (or `admin:read` for OAuth/classic PATs); returns `repository_selection` but not the repositories |
| Listing an installation's repositories | `GET /user/installations/{installation_id}/repositories` with a user token lists repositories the user can access; `GET /installation/repositories` covers only the calling app's own installation |

### 2.7 Platform gaps this example exposes (added to the plan)

| Gap | Resolution |
|---|---|
| Action inputs whose options depend on the target entity | `enumFrom` (CEL) on an input, computed per entity in the recompute pipeline and shown in the preview, MCP `action_options` and the UI |
| Executors need catalog data the target entity doesn't hold (the app's installation) | Action `context` lookups (`{kind, where}`), resolved at the preview snapshot and passed as `related` |
| A run must not be "done" until the change is visible | `expect` postcondition + scoped refresh ([action-runs.md](../design/action-runs.md), D49) |
| Acting as the user across long approval waits | Executor identity `user` with refresh tokens (`offline_access`) stored in the secrets vault, refreshed inside durable workflow steps |
| Pinning and diffing an external permission set | `policies.approvedApps.<app>.permissions` in the manifest + the `app-approval` scorecard comparing them to `GitHubApp.permissions` |
| GitHub API version pinning | Connector config `apiVersion: "2026-03-10"`; `plan` warns on version changes |

## Part 3. What the operator's code looks like

The complete, validated configuration is in the reference portal: the
[manifest](../../examples/reference-portal/toolbox.yaml) (`policies.approvedApps`, the `GitHubApp`
and `GitHubAppInstallation` kinds, the `github-app-hygiene` and `app-approval` scorecards, the
`install-approved-app` and `remove-app` actions), the [authorization model and
tests](../../examples/reference-portal/authz/), the [executor
specification](../../examples/reference-portal/executors/github-apps/README.md) and the [UI bundle
specification](../../examples/reference-portal/ui/install-approved-app/README.md). The action, abridged:

```yaml
- name: install-approved-app
  kind: Repository
  scope: day2
  inputs:
    type: object
    required: [app, justification]
    properties:
      app: {type: string, enumFrom: "policy.approvedApps.filter(a, !(a in entity.installedApps))"}
      justification: {type: string, minLength: 10}
  permission: can_install_apps            # gh_admin or portal admin (authz/model.fga)
  approval: "policy.approvedApps[inputs.app].approval == 'security' ? ['group:security'] : []"
  context:
    installation: {kind: GitHubAppInstallation, where: "candidate.app == inputs.app"}
  executor: {type: in-image, command: [/portal/executors/github-apps/bin/github-apps], identity: user}
  expect:
    postcondition: "inputs.app in entity.installedApps"
    refresh: [{connector: github, kind: installation, selector: "app == inputs.app"}]
    within: 2m
  mcp: {promote: install_approved_github_app}
  ui: {bundle: install-approved-app}
```

**Repository layout in a real portal repo.** The policy and module are owned by security; the
executor ships in the image:

```
policies/approved-apps.yaml          # CODEOWNERS: @acme/security
toolbox.yaml                         # kinds, scorecards, actions (or split files merged by the builder)
authz/model.fga, authz/model.fga.yaml
executors/github-apps/               # in-image executor (any language) + cassettes
ui/install-approved-app/             # UI bundle (React by default) + stories
tests/*.test.yaml, fixtures/         # expectations and connector-record fixtures
Dockerfile, .github/workflows/portal.yml
```

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
>    - Preview snapshot tests.
>    - Simulated-run tests with a simulated executor and connector: 403, 422, rate limits, crash
>      between steps, admin rights removed during an approval wait, a stale preview, and an executor
>      success whose effect never appears (must end `failed`, not `succeeded`).
> 3. Extend the GitHub connector package only if it lacks the `app`/`installation`/collaborator
>    record kinds or the `act` handlers; use the `extend-connector` skill.
> 4. Implement until `toolbox test` is green, including the failure-injection tests. Use
>    `explain_error` for any failure.
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
| Understand | Reads `AGENTS.md`, the skills, this doc and the reference portal; calls `docs_search("enumFrom")` | Docs and skills ship in the binary |
| Red | Writes tests; `toolbox test` fails as expected | Typed SDK, fixtures format, `.fga.yaml` (the same OpenFGA engine as production) |
| Green | Implements the module and connector handler; iterates on `validate` and `explain_error` diagnostics | Typed CEL errors point at the exact expression and field, with fixes |
| Failure injection | `toolbox test` with the simulated executor and connector | Reproducible run-lifecycle failures (deterministic simulation is deferred, D44) |
| Integrate | Runs the sandbox-org integration test (real GitHub API) via the CI job | Catches API reality the simulation can't model |
| Plan | `toolbox plan --remote prod`; pastes the summary | Full plan on the PR (the portal repo is the access boundary) |
| PR | Opens the PR | The plan check, gates and ownership-aware review (security owns the module; platform owns `Kind:Repository` because the authz module extends it) |

### 4.4 CI, preview, deploy and verify (no human steps beyond review)

1. **PR workflow** (`toolbox-dev/workflows/portal.yml`):
   - test → remote plan with gates → build → **preview environment** (synthetic org + sandbox GitHub
     connection).
   - The agent runs MCP smoke tests against the preview: `list_actions`, a preview of
     `install_approved_github_app`, and a real, verified run against the sandbox org.
2. **Review:** security approves the policy and module; platform approves the repository-kind
   extension; the `authz-loss`/`authz-gain` gate shows "+N users gain `can_install_apps` on repos
   they already administer in GitHub".
3. **Merge to main:**
   - build → attest + sign → the customer's deploy hook.
   - The new image recomputes derived state under the new manifest before taking traffic.
   - `toolbox verify` checks that the MCP tool list includes `install_approved_github_app` and that the
     `github-app-hygiene` scorecard is evaluating.
   - Rollback is automatic on failure.
4. **Pilot → GA:** the pilot teams use it. The agent watches the analytics events
   (`action.run.transitioned`, `mcp.tool.called`) and the scorecard. A follow-up PR flips `pilot` to
   `on`, and a separate PR switches drift handling to `enforce` once false-positive rates are known.

### 4.5 What a repo admin's agent does after GA

> *"Add Renovate to payments-api."*
> → MCP `install_approved_github_app(repo="payments-api", app="renovate")` returns the preview (§2.5)
>   and a confirm token
> → the user confirms in the client
> → `install_approved_github_app(…, confirm_token=…)` runs as the user (an MCP task)
> → Toolbox refreshes the installation and checks the postcondition
> → the task completes as **succeeded** with the snapshot in which payments-api lists Renovate.

## Sources

- GitHub changelog, 13 Oct 2026: "GitHub App improvements for least privilege and management"
  ([github.blog](https://github.blog/changelog/2026-10-13-github-app-improvements-for-least-privilege-and-management/);
  text read from the [changelog mirror issue](https://github.com/forks-felickz/github-changelog-reader/issues/731))
- [GitHub roadmap #1376](https://github.com/github/roadmap/issues/1376)
- GitHub docs referenced by the changelog: "Add a repository to an app installation" and "Remove a
  repository from an app installation" (REST, API version 2026-03-10); "Restricting GitHub App
  installation to organization owners"; "Refreshing user access tokens"
- [github/rest-api-description](https://github.com/github/rest-api-description) at `c9c95fe`
  (`descriptions/api.github.com/api.github.com.json`, read from raw.githubusercontent.com on
  2026-11-20)
