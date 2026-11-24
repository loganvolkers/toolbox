# Handoff: the `mark-deprecated` self-service action

`mark-deprecated` is a GitHub Actions workflow that each repository owns (`alpha`, `beta`, `gamma`).
It is run from one place, the org repository, with `org/bin/acme`. There is no GitHub and no Docker
here, so `bin/acme` runs each repository's own workflow with act in host mode. It also does the two
jobs GitHub would do that act cannot: it keeps the run history, and it enforces the
`mark-deprecated` environment's required reviewers.

**Status:** ann requested all three runs and bob approved each one. All three are reported
`succeeded`, and `DEPRECATED.md` on every origin's `main` matches the expected content byte for byte.

| Run | Repository | `main` after the run | Status |
|---|---|---|---|
| `alpha-1` | alpha | `0fdf724` Mark repository deprecated: use acme/dependabot-proxy-v2 | succeeded |
| `beta-1` | beta | `cca831d` Mark repository deprecated: use acme/valtio-store | succeeded |
| `gamma-1` | gamma | `c01305f` Mark repository deprecated: use acme/graphify-core | succeeded |

Paths below are relative to `/tmp/actions-ab/r7/gha-3` unless they start with `.github/`, which
means inside each repository (`alpha/`, `beta/`, `gamma/`, and on each origin's `main`).

## How it works

```
org/bin/acme request alpha ...            org/bin/acme approve alpha-1 --as bob
  │ reads the definition from               │ checks: a required reviewer, not the requester
  │ origins/alpha.git main                  │ (recorded in the run, then re-checked inside the job)
  ▼                                         ▼
act -j plan  (alpha's own workflow)       act (alpha's workflow, same commit): plan, then apply
  validate → preview, nothing written       gate → commit → push to main → observe main
  ▼                                         ▼
run alpha-1: awaiting_approval            bin/acme fetches main itself; succeeded only if
  (record + preview + audit events)         DEPRECATED.md is there with the previewed content
```

| File | Role |
|---|---|
| `.github/workflows/mark-deprecated.yml` | The action: `workflow_dispatch` inputs, a `plan` job (validate and preview) and an `apply` job behind `environment: mark-deprecated` |
| `.github/actions/mark-deprecated/mark_deprecated.py` | The steps: `plan`, `gate` (local runner only), `apply` (commit, push, observe). Uses only the Python standard library |
| `.github/settings/environment-mark-deprecated.json` | Who approves: required reviewers `ann` and `bob`, `prevent_self_review: true`, no admin bypass, deployments from `main` only |
| `.github/settings/ruleset-main.json`, `.github/settings/apply.sh`, `.github/workflows/apply-settings.yml` | GitHub settings as code. Applied through the GitHub API on every change to `.github/settings/` |
| `alpha/CODEOWNERS`, `beta/.github/CODEOWNERS`, `gamma/.github/CODEOWNERS` | The files above are owned by `@ann @bob` |
| `org/bin/acme` | The org front door: `list`, `preview`, `request`, `approve`, `reject`, `status`, `show`, `runs`, `logs`. Prints JSON |
| `org/acme.json` | The registry: repositories and their origins, the people (ann, bob, carol), where act is |
| `org/.acme/` (git-ignored) | The run history: `runs/<id>/record.json`, `runs/<id>/plan.log` and `apply.log` (act output), `audit.jsonl` (append-only, one line per event) |
| `org/templates/repo/`, `org/scripts/install-action.sh` | The template the three repositories started from, and the script that installed it. Each repository now owns and may change its own copy |
| `org/tests/e2e.sh`, `org/tests/settings.sh` | Tests (see the end of this file) |

## R1 to R8

| # | How it is met |
|---|---|
| R1 Owned by the repository | Each repository's definition lives on its origin's `main`: `.github/workflows/mark-deprecated.yml`, `.github/actions/mark-deprecated/mark_deprecated.py` and `.github/settings/environment-mark-deprecated.json`, committed in `cc9d263` (alpha), `39ccf61` (beta) and `e410f5c` (gamma). `bin/acme` reads them from the origin's `main` every time: `list` shows the commit, and `request` runs the workflow at that commit. It never uses the checkout or a central copy. CODEOWNERS assigns these files to `@ann @bob`. On GitHub, the `main` ruleset (`.github/settings/ruleset-main.json`) also requires a pull request approved by a code owner. To change the action, the reviewers or the rules, a team edits its own repository |
| R2 Validated | `validate()` in `.github/actions/mark-deprecated/mark_deprecated.py` checks the following. `successor` must fully match `^[a-z0-9-]+/[a-z0-9-]+$` (a trailing newline does not pass) and must not be this repository. `reason` must be 20 to 200 characters. `bin/acme preview` and `bin/acme request` run the workflow's `plan` job first. If it refuses, the command exits 2 with `error.code: "invalid_inputs"` and a per-input `errors` list. No run is created and nothing is written. The `apply` job validates again before writing |
| R3 Previewed | The `plan` job reads the origin's `main` and returns a preview: base commit, `DEPRECATED.md` as `add`/`modify`, the exact new content and a unified diff. On GitHub it also appears in the step summary. `bin/acme preview` shows it without creating a run. `request` returns it and stores it in the run, so the approver sees it too (`bin/acme show <id>` → `.record.preview`). The preview is binding: `apply` refuses with `stale_preview` if `DEPRECATED.md` on `main` changed since then. The success check compares `main` with the previewed content. A request that would change nothing is refused (`no_change`, exit 5) |
| R4 Approved | The `apply` job sits behind `environment: mark-deprecated`. Its reviewers and `prevent_self_review` come from `.github/settings/environment-mark-deprecated.json`. `bin/acme approve` (`cmd_approve`) enforces the following. The requester cannot approve (`self_review`). Anyone who is not a required reviewer, such as carol, is refused (`not_a_reviewer`). Refusals exit 3, are recorded as `approval_denied` events, and leave the run waiting. The run then executes inside the job; its `gate` step (`mark_deprecated.py gate`) checks the approval again against the repository's settings file. Running the workflow with act directly, without an approval, therefore fails at the gate. `bin/acme reject` lets a reviewer reject a run, or the requester withdraw it |
| R5 Done means observed | After pushing, `apply` fetches `main` fresh and requires `DEPRECATED.md` to match byte for byte; it retries a few times before giving up. A rejected push ends the run `failed` with `outcome.reason: "push_rejected"` and the git output. After act finishes, `bin/acme` fetches `main` itself and marks the run `succeeded` only if three things hold: act exited 0, the job reported success, and `main` holds the previewed content (`outcome.observed_commit`, `outcome.observed_matches_preview`). Otherwise the run is `failed` and `approve` exits 1 |
| R6 One place for the org | From `org/`, `bin/acme list` shows `mark-deprecated` on alpha, beta and gamma. It reads each origin's `main` and reports the definition commit, inputs and reviewers. `bin/acme request <repo> …` works for any repository in `org/acme.json`, and `bin/acme runs` lists runs across all of them |
| R7 Audited | `org/.acme/runs/<id>/record.json` (`bin/acme show <id>`) holds the following, each with a UTC timestamp to the millisecond: `requested_by`/`requested_at`, `inputs`, `approval.approved_by`/`approved_at`, and `outcome` (status, reason, observed commit, push result). It also holds an `events` list: requested, previewed, approval_denied, approved, apply_started, push_accepted or push_rejected, observed, succeeded or failed. Every event is also appended to `org/.acme/audit.jsonl`. The commit on `main` carries `Run:`, `Requested-by:` and `Approved-by:` trailers, with the requester as author and `acme-self-service[bot]` as committer |
| R8 Usable by agents | Every step is a non-interactive `bin/acme` subcommand that prints JSON, with `ok: true/false` and `error.code` on failure; usage errors are JSON too. Exit codes: 0 ok, 1 run failed or internal error, 2 invalid input or usage, 3 forbidden, 4 not found, 5 wrong state (already approved or rejected, or no change). Identity comes from `--as <login>` or `ACME_USER` |

## Commands

Nothing has to be running: there is no server. Each command runs act and exits. You need Python 3
(PyYAML is optional; `list` uses it for input descriptions), `git` and `bin/act`. A run's `approve`
takes about 3 seconds.

```bash
cd /tmp/actions-ab/r7/gha-3/org

# List the action on every repository (definition commit, inputs, reviewers)
bin/acme list
bin/acme list --repo beta

# Preview: validate and show what a run would change; creates nothing
bin/acme preview alpha --successor acme/some-successor --reason "Twenty characters or more, please."

# Request as ann (validated and previewed first; prints the run id, e.g. alpha-2)
bin/acme request alpha --successor acme/some-successor --reason "Twenty characters or more, please." --as ann

# Approve as bob: runs the workflow, pushes, observes main; exit 0 only if it succeeded
bin/acme approve alpha-2 --as bob

# Status, full record (inputs, preview, approvals, outcome, events), act logs
bin/acme status alpha-2
bin/acme show alpha-2
bin/acme logs alpha-2 --phase apply      # or --phase plan

# All runs, optionally filtered
bin/acme runs --repo alpha --status succeeded

# Reject (a reviewer) or withdraw (the requester)
bin/acme reject alpha-2 --as bob --comment "not yet"
```

The three runs above: `bin/acme show alpha-1`, `bin/acme show beta-1`, `bin/acme show gamma-1`.

### Negative cases

All three repositories are now deprecated, so a new request must ask for different content. A
request for what `main` already has is refused with `no_change`.

```bash
cd /tmp/actions-ab/r7/gha-3/org

# Invalid inputs: exit 2, error.code "invalid_inputs", no run created
bin/acme request alpha --successor Acme/Thing --reason "A perfectly long enough reason." --as ann
bin/acme request alpha --successor acme/alpha --reason "A perfectly long enough reason." --as ann
bin/acme request alpha --successor acme/thing --reason "too short" --as ann

# Self-approval and carol: exit 3 (self_review / not_a_reviewer); the run stays awaiting_approval
id=$(bin/acme request alpha --successor acme/thing --reason "A perfectly long enough reason." --as ann | jq -r .run.id)
bin/acme approve "$id" --as ann
bin/acme approve "$id" --as carol
bin/acme status "$id"

# A rejected push: make the origin refuse updates, then approve. Exit 1, status failed,
# outcome.reason push_rejected, and main is unchanged
printf '#!/bin/sh\necho "pushes frozen" >&2\nexit 1\n' > ../origins/alpha.git/hooks/pre-receive
chmod +x ../origins/alpha.git/hooks/pre-receive
bin/acme approve "$id" --as bob
bin/acme show "$id" | jq .record.outcome
rm ../origins/alpha.git/hooks/pre-receive
```

### Tests

- `org/tests/e2e.sh [alpha|beta|gamma]` builds a throwaway copy of one origin, without
  `DEPRECATED.md`, and installs the action into it. It then checks 29 cases: invalid inputs at
  every boundary (19, 20, 200 and 201 characters, the repository as its own successor, a trailing
  newline), that previews change nothing, self-approval, carol, success with the exact bytes,
  double approval, no-change, a rejected push, withdrawal, and act run directly without approval.
  The real origins are not touched. All 29 pass for alpha, beta and gamma.
- `org/tests/settings.sh` runs `.github/settings/apply.sh` against a stub `gh` and checks the API
  calls it makes. It passes.
- Lint: `zizmor --offline .github/workflows` (zizmor 1.32.0) finds nothing in either workflow.

## On GitHub, and what could not run here

On GitHub the same workflow runs unchanged. The `act`-only steps (`if: ${{ env.ACT }}`) are skipped
and the `!env.ACT` steps run. `bin/acme` is then not needed: GitHub provides the gate and the run
history.

| Concern | How GitHub enforces it | What runs here instead |
|---|---|---|
| Approval | `environment: mark-deprecated` holds the `apply` job until a required reviewer (ann or bob) approves. `prevent_self_review: true` stops the person who started the run from approving, carol is not a reviewer, and `can_admins_bypass: false` closes the admin path. Settings are applied by `.github/settings/apply.sh` (`PUT /repos/{repo}/environments/mark-deprecated`), run by `.github/workflows/apply-settings.yml` with an admin token (`secrets.REPO_ADMIN_TOKEN`) | act ignores environment protection rules. `bin/acme approve` enforces the same rules from the same JSON file, and the workflow's `gate` step checks them again |
| Edited workflows | The environment's deployment branch policy allows only `main` (`POST …/deployment-branch-policies`). A workflow edited on a feature branch can never reach the gate | `bin/acme` only runs the definition on the origin's `main` |
| Pushing to `main` | Ruleset `main` (`.github/settings/ruleset-main.json`, `POST /repos/{repo}/rulesets`) blocks force pushes and deletion, and requires a pull request with code owner review. Only the self-service GitHub App bypasses it. The `apply` job pushes with that app's token (`actions/create-github-app-token`, scoped to this repository and `contents: write`; `vars.SELF_SERVICE_APP_ID`, `secrets.SELF_SERVICE_APP_PRIVATE_KEY`) | Local bare origins have no rules; the push goes straight to the bare repository |
| Run history and audit | Actions run history: `triggering_actor`, inputs, conclusion and logs. `GET /repos/{repo}/actions/runs/{id}/approvals` gives who approved and when; the script reads it to fill `Approved-by:` | `org/.acme/runs/*/record.json` and `audit.jsonl` |
| Commands | list `gh workflow view mark-deprecated.yml -R acme/alpha --yaml`; request `gh workflow run mark-deprecated.yml -R acme/alpha -f successor=… -f reason=…`; preview in the `plan` job's summary (`gh run view <id> -R acme/alpha`); approve `gh api -X POST repos/acme/alpha/actions/runs/<id>/pending_deployments -F 'environment_ids[]=<env id>' -f state=approved -f comment=…`; status `gh run view <id> --json status,conclusion`; record `gh api repos/acme/alpha/actions/runs/<id>` and `…/approvals` | `bin/acme` |

Not run here, because they need GitHub:

- the environment gate, `prevent_self_review`, the deployment branch policy and the ruleset;
- `actions/checkout`, the app-token step and the approvals API lookup;
- `apply-settings.yml` itself. Its script was run against a stub `gh` (`org/tests/settings.sh`) and in `DRY_RUN=1` mode, but never against the real API;
- step summaries and the run history UI.

act also ignores the workflow's `concurrency` group.

Limits of the local stand-in:

- **Asserted identity.** `--as` is trusted, as the sandbox requires.
- **Writable state.** Anyone with write access to `org/.acme/` could edit records. GitHub's run history and approvals are not editable.
- **Interrupted approvals.** If `bin/acme approve` is killed mid-run, the run stays `running`.
- **Pinned definition.** A run uses the definition commit it was requested at, as GitHub does. A changed reviewer list applies to new requests.
