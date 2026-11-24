# Handoff: `mark-deprecated` self-service action

**State:** `mark-deprecated` is defined in `alpha`, `beta` and `gamma`, each on its origin's `main`.
It ran once on each repository: requested by ann, approved by bob, and reported `succeeded` after
`DEPRECATED.md` was observed on the origin's `main`. Those are runs `1` (alpha), `2` (beta) and
`3` (gamma). Nothing needs to be running: there is no server, and every step is one shell command
that prints JSON.

## How it fits together

| Part | Where | Role |
|---|---|---|
| Action definition | `<repo>/.github/workflows/mark-deprecated.yml` | A `workflow_dispatch` workflow with three jobs: `validate`, `preview`, and `apply` (gated by the `deprecation` environment) |
| Approval rule and branch rules | `<repo>/.github/settings.yml` | The `deprecation` environment (reviewers ann and bob, `prevent_self_review: true`, deployable from `main` only) and a ruleset that requires code-owner review on `main` |
| Ownership | `alpha/CODEOWNERS`, `beta/.github/CODEOWNERS`, `gamma/.github/CODEOWNERS` | ann and bob own the workflow, the settings file and CODEOWNERS itself |
| Org CLI | `org/bin/acme-actions` (Python 3, PyYAML) | Lists actions, previews, requests, enforces the environment rule, runs the repository's workflow with act, observes `main`, and keeps the audit record |
| Catalog and people | `org/catalog.yml`, `org/people.yml` | The repositories and their origins, the actions on offer, and the known identities |
| Settings applier | `org/bin/apply-github-settings` | Turns each repository's `.github/settings.yml` into GitHub REST calls (dry run by default) |
| Tests | `org/tests/selftest.sh` | 15 end-to-end checks against throwaway copies of the origins |
| Run state | `org/.state/` (git-ignored) | `runs/<id>/record.json`, the act logs (`preview.log`, `apply.log`) and an append-only `audit.jsonl` |

`<repo>` is `/tmp/actions-ab/r1/gha-3/alpha`, `beta` or `gamma`. The CLI reads each definition
from the origin's `main` (`origins/<repo>.git`), not from the checkout. A run clones the origin's
`main` into `org/.state/runs/<id>/src` and pins that commit. Both the preview and the apply run
the workflow from that commit with
`act workflow_dispatch -P ubuntu-latest=-self-hosted`. act runs with `HOME` set to
`org/.state/act-home`, and no `.env`, `.secrets` or `.vars` files are passed in.

A run moves through these states:
`validating` → `refused` | `awaiting_approval` → `rejected` | `running` → `succeeded` | `failed`.
If the CLI itself breaks, the state is `error`. Exit codes are 0 ok, 1 run failed, 2 invalid input,
3 not allowed, 4 unknown run or wrong state, 5 executor or internal error. Errors are JSON
`{"error": {"code", "message", ...}}` on stdout.

## Requirements

| # | How it is met | Files |
|---|---|---|
| R1 Owned by the repository | The whole definition (inputs, validation rules, rendering, push, verification) is one workflow committed to each origin's `main`. The approval rule sits next to it in `.github/settings.yml`, and CODEOWNERS gives both to ann and bob. The CLI holds no copy: `list` and every run read the definition from the origin's `main`, and a run records the definition commit it used. Change the workflow on `main` and the next run uses the change | `<repo>/.github/workflows/mark-deprecated.yml`, `<repo>/.github/settings.yml`, CODEOWNERS; `org/bin/acme-actions` (`definition_for`, `checkout_definition`) |
| R2 Validated | The workflow's `validate` job checks that `successor` matches `^[a-z0-9-]+/[a-z0-9-]+$` and is not the repository itself, and that `reason` is 20 to 200 characters (Unicode characters, `LC_ALL=C.UTF-8`) on a single line. `preview` and `apply` both `need` it. The CLI also refuses unknown or missing inputs against the workflow's `inputs:` schema. A refused request stops before anything is cloned for writing or pushed; the run is recorded as `refused` and the CLI exits 2 | `validate` job in the workflow; `parse_inputs`, `cmd_request` in `org/bin/acme-actions` |
| R3 Previewed | The `preview` job clones `main`, writes the file, and emits the real `git diff` (to `preview.json` under act, and to the step summary on GitHub). `acme-actions preview` shows it without creating a run. `request` runs the same preview and stores it in the run, so `status` and `record` show the requester and the approver exactly what will change. The apply then confirms that `main` holds the previewed content | `preview` job; `cmd_preview`, `cmd_request` |
| R4 Approved | `apply` is the only job that writes, and it targets the `deprecation` environment. Locally, `acme-actions approve` enforces that environment's rule from the repository's `.github/settings.yml` on the origin's `main`: the approver must be a listed reviewer (so carol is refused) and must not be the requester (`prevent_self_review`). Refused approvals are recorded as `approval_refused` events and exit 3. Only an accepted approval starts `act -j apply`. As a second guard, the apply step itself refuses to run without a recorded approver. On GitHub the environment enforces this (see below) | `environment: deprecation` and the first lines of the `apply` step; `<repo>/.github/settings.yml`; `check_approver`, `cmd_review` |
| R5 Done means observed | Each approved run commits with an `Acme-Run: <uid>` trailer; the commit is empty if the content is already there, so every run must push. The workflow then re-clones the origin's `main` and checks that its commit is an ancestor and that `DEPRECATED.md` is byte-identical. Separately, the CLI reads the origin's `main` itself and finds the run's commit by trailer. It compares the file with the approved preview, and reports `succeeded` only if act exited 0 **and** both checks pass. Anything else is `failed`, with the push error in `outcome.errors`. Tested with a rejecting `pre-receive` hook | Last part of the `apply` step; `execute`, `observe` in `org/bin/acme-actions`; `org/tests/selftest.sh` |
| R6 One place for the org | From `org/`, `bin/acme-actions list` lists `mark-deprecated` for every repository in `org/catalog.yml`, with its definition commit, inputs and approvers. `preview`, `request`, `approve`, `reject`, `status`, `record` and `runs` work on any repository by name, without entering it | `org/bin/acme-actions`, `org/catalog.yml` |
| R7 Audited | `org/.state/runs/<id>/record.json` holds who requested (`requested.by/at`), the inputs, the preview (`preview.at`), the approval rule that applied, who approved (`approved.by/at`) or rejected, the definition commit, and the outcome (`outcome.status/at`, act exit code, observed `main` sha, commit and content). It also has a timestamped `events` list that includes refused approvals. Every event is also appended to `org/.state/audit.jsonl`. The landed commit carries `Requested-by`, `Approved-by` and `Acme-Run` trailers, so `main` itself records who did what | `save_run`, `add_event` in `org/bin/acme-actions`; `git log` on each origin |
| R8 Usable by agents | Every command takes flags only (`--as` or `ACME_ACTOR`, `-f key=value`, `--inputs-json`), never prompts, prints JSON on stdout, and uses distinct exit codes. `approve` blocks until the run reaches a final state | `org/bin/acme-actions` |

## Commands

Requirements: `python3` with PyYAML, `git`, `jq` (used by the workflow), and act at
`/tmp/actions-ab/r1/gha-3/bin/act` (configured in `org/catalog.yml`). **Nothing has to be started.**

```bash
cd /tmp/actions-ab/r1/gha-3/org

# List the action across the org (definition commit, inputs, approvers)
bin/acme-actions list

# Preview: validates and shows the diff; creates no run and changes nothing
bin/acme-actions preview alpha --as ann \
  -f successor=acme/dependabot-proxy-v2 \
  -f "reason=Replaced by the v2 proxy, which supports OIDC registries."

# Request as ann: validates, previews, records the run as awaiting_approval, prints .run.id
RUN=$(bin/acme-actions request alpha --as ann \
  -f successor=acme/dependabot-proxy-v2 \
  -f "reason=Replaced by the v2 proxy, which supports OIDC registries." | jq -r .run.id)

# Approve as bob: runs the apply job, observes main, prints the final status (exit 0 only if succeeded)
bin/acme-actions approve "$RUN" --as bob

# Reject instead (a reviewer other than the requester)
bin/acme-actions reject "$RUN" --as bob --comment "not yet"

# Status, full record, all runs
bin/acme-actions status "$RUN"
bin/acme-actions record "$RUN"
bin/acme-actions runs --repo alpha --status succeeded
```

`--inputs-json '{"successor": "...", "reason": "..."}'` can replace the `-f` flags. Instead of
`--as NAME` you can set `ACME_ACTOR=NAME`. Repositories can be named `alpha` or `acme/alpha`.

### Negative tests

```bash
cd /tmp/actions-ab/r1/gha-3/org

# Invalid inputs: exit 2, run recorded as "refused", origin main unchanged
bin/acme-actions request beta --as ann -f successor=Not/Valid! -f reason=too-short
bin/acme-actions request beta --as ann -f successor=acme/beta -f "reason=A repository cannot succeed itself."

# Self-approval and carol: exit 3, recorded as approval_refused events
RUN=$(bin/acme-actions request beta --as ann -f successor=acme/valtio-store \
  -f "reason=State management moves to the shared store package." | jq -r .run.id)
bin/acme-actions approve "$RUN" --as ann
bin/acme-actions approve "$RUN" --as carol

# Rejected push: make the origin refuse pushes, then approve. Exit 1, status "failed"
printf '#!/bin/sh\necho "main is frozen" >&2\nexit 1\n' > ../origins/beta.git/hooks/pre-receive
chmod +x ../origins/beta.git/hooks/pre-receive
bin/acme-actions approve "$RUN" --as bob
bin/acme-actions record "$RUN" | jq .outcome
rm ../origins/beta.git/hooks/pre-receive

# All of the above against throwaway copies of the origins
tests/selftest.sh
```

A run whose inputs match what is already on `main` still commits (an empty commit carrying its
trailers) and pushes. That way a rejected push fails even when the file content would not change.

## How GitHub would enforce what act cannot

| Feature | Configured as code | Enforcement on GitHub |
|---|---|---|
| Required reviewers | `environments[deprecation].reviewers` in `<repo>/.github/settings.yml`, applied by `org/bin/apply-github-settings --apply` (`PUT /repos/{owner}/{repo}/environments/deprecation`) | The `apply` job pauses as "Waiting for review" until ann or bob approves the deployment. Until then it gets no `GITHUB_TOKEN` and runs no steps |
| No self-review | `prevent_self_review: true` in the same file and call | GitHub does not let the person who triggered the run approve it |
| Only `main` can deploy | `deployment_branch_policy` plus `deployment_branches: [main]` (`POST .../deployment-branch-policies`) | An edited workflow on another branch cannot reach the gated job |
| Who may change the definition | CODEOWNERS plus the `main-reviewed` ruleset in `settings.yml` (`POST /repos/{owner}/{repo}/rulesets`) | A change to `main` needs a pull request approved by a code owner. The GitHub Actions app (id 15368) bypasses the ruleset, so the reviewed workflow can commit `DEPRECATED.md` |
| Approver in the commit | The `apply` step calls `GET /repos/{owner}/{repo}/actions/runs/{run_id}/approvals` when not under act | The `Approved-by` trailer names the approving reviewer |
| Run history and audit | Built in | The Actions run history, the deployments and their review history, and the org audit log |
| One run at a time | `concurrency: mark-deprecated-${{ github.repository }}` | Queues runs per repository. Locally the CLI takes a per-repository lock instead |

On GitHub the org-level CLI would map to `gh`. Its local backend is not wired to that, but the
calls would be:

| Step | Command |
|---|---|
| List | `gh workflow list -R acme/<repo>` |
| Preview | `gh workflow run mark-deprecated.yml -R acme/<repo> -f successor=… -f reason=… -f dry_run=true` |
| Request | `gh workflow run mark-deprecated.yml -R acme/<repo> -f successor=… -f reason=…` |
| Approve | `gh api -X POST repos/acme/<repo>/actions/runs/<id>/pending_deployments` with `environment_ids` and `state=approved` |
| Status | `gh run view <id> -R acme/<repo> --json status,conclusion` |
| Record | `gh api repos/acme/<repo>/actions/runs/<id>/approvals` plus the run's logs |

## What could not run here

- **GitHub itself.** Environments, required reviewers, `prevent_self_review`, deployment branch
  policies, rulesets, CODEOWNERS enforcement, the approvals API call in the `apply` step, the
  `GITHUB_TOKEN` clone and push path, `concurrency`, `run-name`, the step summary page and the run
  history were not exercised. `bin/apply-github-settings` was only run in its default dry-run mode.
  Using GitHub Actions (app id 15368) as a ruleset bypass actor is unverified. If GitHub does not
  allow it, a dedicated GitHub App token would be the bypass actor for the push instead.
- **Approval locally rests on the CLI.** act ignores `environment:`, so locally the gate is
  `bin/acme-actions approve`. Someone running `act -j apply` by hand with `ACME_APPROVED_BY` set
  would bypass it. Identity is asserted with `--as`, since this sandbox has no authentication. Run
  records in `org/.state/` are plain files that anyone with filesystem access could edit. On GitHub,
  authentication, the environment gate and the run history replace all three.

## Details a tester may care about

- `DEPRECATED.md` is exactly `# Deprecated: use <successor>`, an empty line, then `<reason>`, ending
  with a newline.
- `reason` must be a single line (no control characters). `workflow_dispatch` string inputs are
  single-line on GitHub too.
- Two people approving the same run at once: a per-run file lock admits one, and the other gets
  exit 4 (wrong state).
