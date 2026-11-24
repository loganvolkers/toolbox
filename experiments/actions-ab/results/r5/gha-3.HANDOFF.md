# Handoff: `mark-deprecated`, a repository-owned self-service action

**Status.** `mark-deprecated` is defined in `alpha`, `beta` and `gamma`, each on its origin's `main`. All three
runs ann requested have been approved by bob and are reported as `succeeded`: runs 1 (alpha), 2 (beta) and 3 (gamma)
in `org/.state/runs/`. Each origin's `main` has `DEPRECATED.md` with the exact content, committed by
`github-actions[bot]` with `Requested-by: ann` and a `Workflow-run: acme run <id>` trailer.

**Shape.** This is about the box: a GitHub Actions workflow in each repository does the work, and one
small piece of org-level glue, `org/acme`, provides what act cannot run locally: the
environment approval gate, run history and the org-wide listing.

| Piece | Where | What it is |
|---|---|---|
| The action | `<repo>/.github/workflows/mark-deprecated.yml` | `workflow_dispatch` workflow with three jobs: `validate`, `preview`, and `apply` gated by the `deprecation` environment |
| Its declaration and approvers | `<repo>/.github/self-service.yml` | Lists the repository's self-service actions, the gated job, and the environment's reviewers (`ann`, `bob`) with `prevent_self_review: true` |
| GitHub settings as code | `<repo>/.github/scripts/apply-github-settings.py` | Applies `self-service.yml` through the GitHub API: environment reviewers, prevent self-review, a main-only deployment policy, a `main` ruleset and a read-only default token |
| Ownership | `alpha/CODEOWNERS`, `beta/.github/CODEOWNERS`, `gamma/.github/CODEOWNERS` | The three files above are owned by `@ann @bob` |
| Org glue | `org/acme` (Python 3, PyYAML), `org/org.yml` | Lists, previews, requests, approves and rejects runs, and keeps their records. It runs the workflows with `bin/act` and observes `main` itself |
| Run history | `org/.state/runs/<id>.json`, plus `org/.state/runs/<id>/{request,apply}.jsonl` act logs | The local stand-in for GitHub's run history. Not in git (`org/.gitignore`) |
| Tests | `org/tests/e2e.sh` | 16 end-to-end checks against copies of the origins in `org/.state/e2e` |

## How each requirement is met

| # | How | Files |
|---|---|---|
| R1 Owned by the repository | The action's whole definition (inputs, validation, preview, the write to `main`, the observation) is one workflow in each repository, committed to its origin's `main` by ann. The approvers are in the same repository. CODEOWNERS gives both files to the maintainers. `acme` reads the definition from the origin's `main` at request time and pins the run to that commit (`definition_sha` in the record), so a change the team merges applies to the next run. `acme` contains nothing specific to `mark-deprecated` | `<repo>/.github/workflows/mark-deprecated.yml`, `<repo>/.github/self-service.yml`, `<repo>/CODEOWNERS` or `<repo>/.github/CODEOWNERS` |
| R2 Validated | The `validate` job checks `successor` with `re.fullmatch(r"[a-z0-9-]+/[a-z0-9-]+")`, refuses the repository itself as its own successor, and checks that `reason` is 20 to 200 characters (Unicode characters, not bytes). `preview` and `apply` both `need` it, so nothing is read or written after a failure. Before that, `acme` refuses unknown or missing input names, as GitHub's dispatch API does. A refused request is recorded with state `invalid` and exits 2 | `mark-deprecated.yml` job `validate`; `org/acme` `parse_inputs`, `cmd_request` |
| R3 Previewed | The `preview` job fetches `main` and reports the change (`create`, `update` or `none`), the full new content and a unified diff, as the job output `preview` and the step summary. `acme preview` runs it without creating a run. `acme request` runs it, and the run waits for approval with the preview in `status` and `record` | `mark-deprecated.yml` job `preview`; `org/acme` `cmd_preview`, `phase_one` |
| R4 Approved | `apply` declares `environment: deprecation`. On GitHub the environment has required reviewers `ann` and `bob` with `prevent_self_review: true`, so GitHub holds the job until a reviewer other than the person who triggered it approves. act ignores environments, so `acme` enforces the same rules from the same `self-service.yml`, read from the origin's `main` when someone approves. Approvers must be listed reviewers (carol is not), and the requester cannot approve. A refused attempt exits 3 and is recorded as `approval_denied` in the run's record. Only a run in `waiting_for_approval` can be approved, and the state change happens under a file lock, so a run cannot be approved twice | `<repo>/.github/self-service.yml`, `<repo>/.github/scripts/apply-github-settings.py`; `org/acme` `authorize_review`, `cmd_approve` |
| R5 Done means observed | Two independent observations. In the workflow, the `apply` job's last step re-fetches `main` after the push and fails unless `DEPRECATED.md` is byte-for-byte the expected content. A rejected push already fails the push step. After act returns, `acme` fetches the origin's `main` into its own mirror and compares the file with the content the preview declared. The run is `succeeded` only if the job succeeded **and** `acme` observed the content. Otherwise it is `failed`, with the job's errors and the `main` SHA it saw. A run whose content is already on `main` makes no commit and succeeds, because the content is observed | `mark-deprecated.yml` steps `Commit and push to main` and `Observe main`; `org/acme` `execute` |
| R6 One place for the org | `org/org.yml` names the three repositories and their origins. From `org/`, `./acme list` discovers each repository's actions from its `.github/self-service.yml` and workflow on the origin's `main`, and `./acme request --repo <name>` runs on any of them without entering a checkout | `org/acme`, `org/org.yml` |
| R7 Audited | Each run is one JSON record. It holds `requested_by` and `requested_at`, `approved_by` and `approved_at`, `inputs`, `outcome` (`state`, `at`, `detail`, `main_sha`), the pinned `definition_sha`, the preview, the environment rules that applied, paths to act's logs, and an append-only `events` list. Every event has an `at` timestamp and, where a person acted, an `actor`. Denied approval attempts are events too. Records are written atomically under a lock. On `main`, each commit carries `Requested-by:` and `Workflow-run: acme run <id>` | `org/.state/runs/<id>.json`; `org/acme` `add_event`, `cmd_record` |
| R8 Usable by agents | Every command is non-interactive, takes identity from `--as` or `ACME_USER`, and prints one JSON document on stdout, including usage errors. Inputs can be given as `-f name=value` or `--inputs-json '{...}'`. Exit codes: `0` ok, `1` internal error, `2` invalid input or usage, `3` not permitted, `4` run failed, `5` not found or wrong state | `org/acme` |

Run states: `requested` (being validated and previewed), `invalid`, `waiting_for_approval`, `running`, `succeeded`,
`failed`, `rejected`, `cancelled`.

## Commands

**Nothing needs to be running.** `acme` is a CLI: each command runs act synchronously and returns, so there is no server.
Requirements: `python3` with PyYAML, `git`, `jq` (only for the examples), and act at `/tmp/actions-ab/r5/gha-3/bin/act`
(path set in `org/org.yml`). Every command below is run from the org directory:

```bash
cd /tmp/actions-ab/r5/gha-3/org
```

List the actions of all three repositories (inputs, reviewers, the commit they come from):

```bash
./acme list
./acme list --repo beta --action mark-deprecated
```

Preview a run (validates and shows the diff; changes nothing and records nothing):

```bash
./acme preview --repo alpha --action mark-deprecated \
  -f successor=acme/dependabot-proxy-v2 \
  -f "reason=Replaced by the v2 proxy, which supports OIDC registries." --as ann
```

Request a run as ann (validates and previews, then waits for approval; prints the run, including `.run.id`):

```bash
./acme request --repo alpha --action mark-deprecated \
  -f successor=acme/dependabot-proxy-v2 \
  -f "reason=Replaced by the v2 proxy, which supports OIDC registries." --as ann
# or: --inputs-json '{"successor": "acme/x", "reason": "..."}'
RUN=$(./acme runs --state waiting_for_approval | jq -r '.runs[-1].id')
```

Approve as bob (runs the `apply` job, observes `main`, and prints the final state; exits 0 only if `succeeded`):

```bash
./acme approve "$RUN" --as bob --comment "ok"
```

Check a run's status, read its full record, and list runs:

```bash
./acme status "$RUN"            # .run.state, people, inputs, preview, outcome
./acme record "$RUN"            # the full record, with timestamped events
./acme runs --repo alpha        # filters: --repo, --action, --state
```

Reject or cancel a waiting run:

```bash
./acme reject "$RUN" --as bob --comment "not yet"   # a listed reviewer
./acme cancel "$RUN" --as ann                       # the requester or a reviewer
```

The three runs made here:

```bash
./acme runs | jq -c '.runs[] | {id, repository, state, requested_by, approved_by}'
```

## Testing it

**Run every check.** `tests/e2e.sh` runs 16 checks against copies of the origins, with its own run history in
`org/.state/e2e`, so it changes neither the real origins nor runs 1 to 3:

```bash
tests/e2e.sh
```

**Invalid inputs.** Each of these exits 2 with `"valid": false` and the `validate` job's errors:

```bash
./acme preview --repo alpha --action mark-deprecated -f successor=Acme/X -f "reason=long enough reason text here"
./acme preview --repo alpha --action mark-deprecated -f successor=acme/next -f "reason=too short"
./acme request --repo alpha --action mark-deprecated -f successor=acme/alpha -f "reason=its own successor is refused" --as ann
./acme request --repo alpha --action mark-deprecated -f successor=acme/next --as ann
```

The first two are previews. The third creates a run in state `invalid`. The fourth is missing `reason`, so it is
refused before any run is created.

**Self-approval and carol.** Request as ann, then try to approve as ann and as carol. Both exit 3, the run stays
`waiting_for_approval`, and both attempts appear as `approval_denied` events in `./acme record`:

```bash
RUN=$(./acme request --repo beta --action mark-deprecated -f successor=acme/next \
  -f "reason=Testing the approval rules here." --as ann | jq -r .run.id)
./acme approve "$RUN" --as ann;   echo "exit $?"   # 3: requester cannot approve
./acme approve "$RUN" --as carol; echo "exit $?"   # 3: not a reviewer
./acme status "$RUN" | jq .run.state             # "waiting_for_approval"
./acme reject "$RUN" --as bob                     # clean up without changing main
```

**Rejected push.** Make the origin refuse pushes, then request and approve a run whose content **differs** from what
is already on `main`. A run whose content is already there makes no push and correctly succeeds. The approval exits 4,
the run is `failed`, and `.run.errors` names the failed push step:

```bash
HOOK=/tmp/actions-ab/r5/gha-3/origins/alpha.git/hooks/pre-receive
printf '#!/bin/sh\necho "pushes to main are frozen" >&2\nexit 1\n' > "$HOOK"; chmod +x "$HOOK"
RUN=$(./acme request --repo alpha --action mark-deprecated -f successor=acme/next \
  -f "reason=This push will be rejected by the hook." --as ann | jq -r .run.id)
./acme approve "$RUN" --as bob; echo "exit $?"   # 4, state "failed"
rm -f "$HOOK"
```

`tests/e2e.sh` also covers a push that is accepted but undone by a `post-receive` hook. That run is `failed` because the
change is not observed on `main`.

## How GitHub would enforce what act cannot

| Feature | Configured as code in | How GitHub enforces it |
|---|---|---|
| Required reviewers | `self-service.yml` `environment.reviewers`, applied by `apply-github-settings.py` (`PUT /repos/{o}/{r}/environments/deprecation`) | A job with `environment: deprecation` waits until a listed reviewer approves it (`POST /repos/{o}/{r}/actions/runs/{id}/pending_deployments`) |
| Prevent self-review | `prevent_self_review: true`, same call | GitHub refuses approval from the user who triggered the run |
| Only from `main` | `deployment_branches: [main]`, applied as a deployment branch policy | The environment, and so the write job, cannot run from another branch |
| Least-privilege token | The workflow sets `permissions: {}`, and only `apply` gets `contents: write`. The script sets the repository's default token to read-only | Only the approved job can push |
| `main` cannot be deleted or force-pushed | Ruleset `protect-main` (deletion, non_fast_forward) | Push rejected |
| Maintainers own the definition | CODEOWNERS | Changes to these files request review from `@ann @bob`. This is blocking only with a pull-request rule (see below) |
| Run history and audit | — | The Actions run log, `run-name` (shows successor and requester), the step summary (preview) and the deployment review history (`GET /repos/{o}/{r}/actions/runs/{id}/approvals`) |
| One serial apply per repository | `concurrency: mark-deprecated` on `apply` | GitHub queues a second apply |

On GitHub, the workflow authenticates with `github.token`: `ORIGIN_URL` falls back to `https://github.com/<repo>.git`
with an `http.extraheader`. The equivalent of `acme request` is
`gh workflow run mark-deprecated.yml -R acme/alpha -f successor=... -f reason=...`. Status is `gh run view`.
To apply the settings, run `.github/scripts/apply-github-settings.py --repo acme/<name>` as a repository admin, and
add `--dry-run` to print the API calls first.

## What I could not run here

- **No GitHub API.** `apply-github-settings.py` ran only with `--dry-run`, which prints the calls. GitHub's environments,
  required reviewers, prevent self-review, deployment branch policies, rulesets, CODEOWNERS enforcement, run history and
  `concurrency` did not run. `acme` reproduces the approval rules and run history locally, and the rejected-push test uses
  a git hook in place of a ruleset.
- **GitHub-only paths of the workflow did not run.** The `https` and token branch of the fetch, and the
  `github.server_url` run URL, were not exercised. Locally, act gets `ORIGIN_URL` and `RUN_URL` through `--var`.
- **No blocking pull-request rule on `main`.** A pull-request rule would make CODEOWNERS review blocking, but it would
  also block the `apply` job's direct push with `GITHUB_TOKEN`. Doing both needs a GitHub App token as a ruleset bypass
  actor. That is not configured here.
- **Identity is claimed, not proven.** `--as` is trusted, as the task allows. Anyone with shell access could also run
  act directly or push to `origins/*.git`. That is the local equivalent of a repository admin bypassing GitHub.
  `acme` controls only the runs that go through it.
- **Interrupted runs.** If `acme` is killed during `approve`, the run stays `running`. There is no automatic recovery;
  its act log is in `org/.state/runs/<id>/apply.jsonl`.
