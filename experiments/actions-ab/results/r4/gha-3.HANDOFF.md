# HANDOFF: `mark-deprecated` on GitHub Actions

`mark-deprecated` is a self-service action that each repository defines and owns in its own
`.github/`. It runs as a GitHub Actions workflow, which act executes locally in host mode. One CLI in
this directory, `org/bin/acme`, is the org's single place to list the action, preview a run, request
one, approve it, and read its status and audit record. Requests, approvals, refusals and outcomes are
recorded, and a run is reported `succeeded` only after `acme` has itself read `DEPRECATED.md` with the
expected content on the origin's `main`.

The three requested runs (requested by ann, approved by bob) all succeeded:

| Repository | Run id | Commit on `origins/<repo>.git` `main` |
|---|---|---|
| alpha | `alpha-mark-deprecated-20261124t124602z-5991` | `f055f49` Mark deprecated: use acme/dependabot-proxy-v2 |
| beta | `beta-mark-deprecated-20261124t124604z-17fd` | `cca470b` Mark deprecated: use acme/valtio-store |
| gamma | `gamma-mark-deprecated-20261124t124605z-9305` | `04cb699` Mark deprecated: use acme/graphify-core |

## What is where

Paths are relative to `/tmp/actions-ab/r4/gha-3`.

| Path | Owner | What it is |
|---|---|---|
| `<repo>/.github/self-service/mark-deprecated.json` | the repository | The action's definition: inputs and their rules, `approvers`, the workflow, the GitHub environment and the effect (branch, path, template) |
| `<repo>/.github/self-service/mark_deprecated.py` | the repository | The action's logic: `validate`, `preview`, `apply`, `verify`. Standard-library Python |
| `<repo>/.github/workflows/mark-deprecated.yml` | the repository | The workflow. `mode=preview` runs the *validate and preview* job; `mode=apply` also runs *apply and verify* in environment `mark-deprecated` |
| `<repo>/.github/self-service/apply-github-settings.sh` | the repository | Applies the GitHub enforcement through the API: environment reviewers, no self-review, the `main` ruleset and the deploy key |
| `<repo>/.github/workflows/self-service-settings.yml` | the repository | Reruns that script whenever `.github/self-service/` changes on `main` |
| `alpha/CODEOWNERS`, `beta/.github/CODEOWNERS`, `gamma/.github/CODEOWNERS` | the repository | Make `@ann @bob` the code owners of the files above |
| `org/bin/acme` | the org | The org CLI: the glue GitHub Actions does not provide |
| `org/acme.json` | the org | Repositories (origin path, GitHub slug), people (ann, bob, carol), path to act |
| `org/bin/install-mark-deprecated`, `org/template/` | the org | Starter used to add the action to a repository. Once copied, the copy belongs to that repository |
| `org/state/` (not committed) | runtime | `runs/<id>.json` run records, `audit.jsonl` (org-wide, append-only), `logs/<id>-{preview,apply}.log` act logs |

## Requirements

**R1 Owned by the repository.** Each repository's definition, logic and workflow are committed to its
origin's `main`: `origins/<repo>.git`, in `.github/self-service/` and
`.github/workflows/mark-deprecated.yml`. `acme` reads them from `main` on the origin (not from a checkout,
and not from the org) every time: `list` reads the definition, the workflow runs from a fresh clone of
`main`, and `approve` re-reads `approvers` from `main`. So a change the team merges to `main`, such as a
different approver list or reason length, takes effect at once. The CODEOWNERS entries make `@ann @bob`
own those paths. On GitHub, the ruleset from `apply-github-settings.sh` requires a code-owner-approved
pull request to change them.

**R2 Validated.** Invalid inputs are refused at three layers, all driven by the repository's
`mark-deprecated.json`, and nothing is written in any of them:
1. `acme preview` and `acme request` check the rules before any workflow runs (`validate()` in
 `org/bin/acme`). They exit 2 with `error.code: invalid_inputs` and per-input errors, record no run,
 and log the refusal in `org/state/audit.jsonl` (`request_refused`).
2. The workflow's *validate and preview* job (`mark_deprecated.py preview`) re-checks the inputs. The
 *apply and verify* job depends on it (`needs: preview`).
3. `mark_deprecated.py apply` validates again before it writes anything.

The rules are: `successor` must fully match `^[a-z0-9-]+/[a-z0-9-]+$` and must not be the repository
itself; `reason` must be 20 to 200 characters, one line, and not blank. Unknown and missing inputs are
refused.

**R3 Previewed.** `acme preview` (no record) and `acme request` run the repository's workflow in
`mode=preview` with act. The result shows the base commit on `main`, `change`
(`create`/`update`/`none`), the exact file content and a unified diff. `request` stores this preview
in the run record (`.record.preview`), so the approver sees it with `acme record` before approving.
On GitHub, the same job writes the preview to the run's step summary, which reviewers see before
they approve the environment.

**R4 Approved.** A run takes effect only through `acme approve`. In `decide()` in `org/bin/acme`,
under a lock, approval is refused (exit 3, `error.code: not_allowed`) when the approver is the
requester, or is not in the repository's `approvers` as read from `main` (ann and bob; carol is not
one). Every refused attempt is recorded in the run (`approval_denied`) and in the audit log. A run
that is not `awaiting_approval` cannot be approved again (exit 5). As a second check that does not
depend on `acme`, `mark_deprecated.py apply` refuses to push unless `approved_by` is an approver
and differs from the requester. On GitHub, enforcement comes from the `mark-deprecated` environment,
which `apply-github-settings.sh` configures with required reviewers = `approvers`,
`prevent_self_review: true` and `can_admins_bypass: false`. The push credential, the deploy key in
secret `MARK_DEPRECATED_DEPLOY_KEY`, is a secret of that environment, so only an approved job can
push. On GitHub the apply step reads the approver from the run's approvals API (`github_approver()`).

**R5 Done means observed.** After the apply workflow, `acme` reads the origin itself (`observe()` in
`org/bin/acme`: `git --git-dir origins/<repo>.git show main:DEPRECATED.md`) and compares it
byte-for-byte with the definition's template rendered with the run's inputs. The run becomes
`succeeded` only if the workflow exited 0, its own `verify` step (which fetches `main` after the
push) passed, and that observation matches. Otherwise it is `failed`, with `outcome.error` (for
example `push rejected`) and the observation in `outcome.observed`. I tested this with a
`pre-receive` hook that rejects pushes: the run was reported `failed`, and `main` was unchanged.

**R6 One place for the org.** All commands run from `/tmp/actions-ab/r4/gha-3/org` through
`bin/acme`. `acme list` reads every repository in `acme.json` from its origin. `--repo <name>`
targets any repository, without a checkout of it.

**R7 Audited.** `org/state/runs/<id>.json` holds `requested_by`/`requested_at`,
`approved_by`/`approved_at`, `inputs`, the preview, both executions (workflow commit, act exit code,
start and finish times, log path, apply and verify results), `status` and `outcome` (result, time,
error, pushed commit, observation). It also holds an `events` list of
`{at, event, actor, …}`: `requested`, `previewed`, `approval_denied`, `approved`,
`execution_started`, `execution_finished`, `observed`, then `succeeded`, `failed` or `rejected`. Every
event, plus refused requests, is also appended to `org/state/audit.jsonl`. The commit on `main`
carries `Run-Id`, `Requested-by` and `Approved-by` trailers.

**R8 Usable by agents.** Every `acme` command is non-interactive and prints one JSON object on stdout
(`ok`, plus `run`, `record`, `actions`, `runs` or `error`). Exit codes: 0 ok, 1 run failed, 2 invalid
input or usage, 3 not allowed, 4 not found, 5 wrong run state. Identity is `--as NAME` or
`ACME_AS=NAME`. Inputs are `--successor`/`--reason`, `--input NAME=VALUE` or `--inputs-json '{...}'`.

## Commands

Requirements: `python3`, `git`, and `/tmp/actions-ab/r4/gha-3/bin/act`. `jq` is optional; it is used
below only to pick fields. **Nothing needs to be running**: there is no server, and `acme` invokes
act per step. I started no long-running process.

```bash
cd /tmp/actions-ab/r4/gha-3/org

# List mark-deprecated for all three repositories
bin/acme list
bin/acme list mark-deprecated --repo beta

# Preview (validates, shows the change, records nothing)
bin/acme preview mark-deprecated --repo alpha --successor acme/dependabot-proxy-v2 \
  --reason "Replaced by the v2 proxy, which supports OIDC registries."

# Request as ann (validates and previews; prints the run id)
RUN=$(bin/acme request mark-deprecated --as ann --repo alpha --successor acme/some-successor \
  --reason "A different reason, long enough to be valid." | jq -r .run.id)

# Approve as bob (executes, then verifies on the origin's main)
bin/acme approve "$RUN" --as bob

# Status, and the full record
bin/acme status "$RUN"
bin/acme record "$RUN"

# Other commands
bin/acme runs [--repo alpha] [--status succeeded]
bin/acme reject "$RUN" --as bob --comment "why"   # an approver rejects, or the requester withdraws
```

### Testing the refusals

`DEPRECATED.md` is already on all three `main`s. A request with the *same* inputs previews
`change: none`; approving it pushes nothing and succeeds, because the content is already on `main`.
To exercise a real push, use a different `reason` or `successor`.

```bash
cd /tmp/actions-ab/r4/gha-3/org

# Invalid inputs: exit 2, no run created
bin/acme request mark-deprecated --as ann --repo alpha --successor Acme/X --reason "too short"
bin/acme request mark-deprecated --as ann --repo alpha --successor acme/alpha \
  --reason "Pointing at itself is refused too."
bin/acme request mark-deprecated --as ann --repo alpha --successor acme/x \
  --reason "$(printf 'x%.0s' {1..201})"

# Self-approval and carol: exit 3; the run stays awaiting_approval and records the denials
RUN=$(bin/acme request mark-deprecated --as ann --repo beta --successor acme/other-store \
  --reason "Testing the approval rules for this action." | jq -r .run.id)
bin/acme approve "$RUN" --as ann
bin/acme approve "$RUN" --as carol
bin/acme record "$RUN" | jq '.record.status, [.record.events[] | {event, actor}]'

# Rejected push: the run is reported failed, and main is unchanged
printf '#!/bin/sh\necho "push rejected for test" >&2\nexit 1\n' > ../origins/beta.git/hooks/pre-receive
chmod +x ../origins/beta.git/hooks/pre-receive
bin/acme approve "$RUN" --as bob      # exit 1, status failed, outcome.error "push rejected"
rm ../origins/beta.git/hooks/pre-receive
git --git-dir ../origins/beta.git show main:DEPRECATED.md   # still the original content
```

## How GitHub would enforce what act cannot

| Feature | Configured as code | How GitHub enforces it |
|---|---|---|
| Approval by another maintainer | `.github/self-service/apply-github-settings.sh` → `PUT /repos/{repo}/environments/mark-deprecated` with `reviewers` = `approvers`, `prevent_self_review: true`, `can_admins_bypass: false`; `environment: mark-deprecated` on the apply job | The apply job waits until a listed reviewer, other than the user who triggered the run, approves it |
| Only approved runs can push | Deploy key created by `--rotate-deploy-key`, stored as environment secret `MARK_DEPRECATED_DEPLOY_KEY`; ruleset bypass actor `DeployKey` | Environment secrets are released only after approval; everyone else must go through a pull request |
| Repository owns the action | Ruleset `main-owned-by-maintainers` (pull request, code-owner review, last-push approval, no force push or deletion) plus CODEOWNERS | Changes to `.github/self-service/` and the workflow need ann's or bob's review |
| Settings follow the definition | `.github/workflows/self-service-settings.yml` (secret `REPO_ADMIN_TOKEN`) | Reapplies the settings when the definition changes on `main` |
| Run history | Every run is a `workflow_dispatch` run with `run-name: mark-deprecated <mode> <run_id>` | The Actions tab keeps history, logs and the approval for each run |

On GitHub, `acme request` would map to `gh workflow run mark-deprecated.yml -R acme/<repo> -f
mode=apply -f successor=… -f reason=… -f run_id=…`, run as the requester. The preview job runs
first, and the apply job waits at the environment. `acme approve` would map to
`POST /repos/acme/<repo>/actions/runs/{id}/pending_deployments` with `state=approved`, run as the
approver. Locally, `acme` plays that environment gate: it runs the preview job at request time and
the whole workflow after approval.

## What I could not run here

- **GitHub itself.** Environments, required reviewers, `prevent_self_review`, environment secrets,
 rulesets, CODEOWNERS enforcement, the Actions run history and the approvals API used by
 `github_approver()` all need github.com. I ran `apply-github-settings.sh --dry-run acme/alpha`,
 which prints the exact API requests, but not against the API. The ruleset's `DeployKey` bypass
 actor and the deploy-key push path are unverified.
- **The settings workflow.** `self-service-settings.yml` refuses to run under act, or without
 `REPO_ADMIN_TOKEN`, so it cannot reach the GitHub API from this machine.
- **Identity.** It is self-asserted (`--as`), as the sandbox specifies. Locally, R4 is enforced by
 `acme` and by the workflow's own approver check. Anyone with shell access could still push to
 `origins/*.git` or edit `org/state/` directly, so the local audit log is not tamper-proof.
- **Interrupted runs.** A run whose `acme approve` process is killed mid-run stays `running`. There
 is no reaper.

act 0.2.98 ran with `-P ubuntu-latest=-self-hosted`, offline, with its host workspaces under
`org/state/act/`. act skips `actions/checkout` for the local repository, so nothing is downloaded.
`act --validate --strict` passes on both workflows.
