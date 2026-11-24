# HANDOFF: `mark-deprecated` on GitHub Actions (run locally with act)

**Status.** `mark-deprecated` is defined in, and owned by, each of `alpha`, `beta` and `gamma`. ann
requested it on all three, bob approved each run, and each run is reported `succeeded` after the
system saw `DEPRECATED.md` with the approved content on the origin's `main`:

| Run | Repository | Inputs | `main` after the run |
|---|---|---|---|
| `alpha-mark-deprecated-1` | acme/alpha | `acme/dependabot-proxy-v2`, "Replaced by the v2 proxy, which supports OIDC registries." | `1008cba` |
| `beta-mark-deprecated-1` | acme/beta | `acme/valtio-store`, "State management moves to the shared store package." | `af33c81` |
| `gamma-mark-deprecated-1` | acme/gamma | `acme/graphify-core`, "Graph extraction now lives in graphify-core." | `f30b1b4` |

`org/tests/acceptance.sh` checks the whole flow (42 checks, all passing) against throwaway copies
of the origins. It covers invalid inputs, self-approval, carol, a rejected push and rejection.

## How it fits together

All paths are relative to `/tmp/actions-ab/r3/gha-3`.

| Piece | Where | Role |
|---|---|---|
| Workflow | `<repo>/.github/workflows/mark-deprecated.yml` | The action. It has three jobs: `validate`, `preview`, and `apply`. `apply` is gated by the environment `mark-deprecated` |
| Script | `<repo>/.github/mark-deprecated/mark_deprecated.py` | Has `validate`, `preview` and `apply` subcommands. Also works outside Actions, with the same command and JSON output |
| Approval rules | `<repo>/.github/environments/mark-deprecated.json` | Required reviewers (`ann`, `bob`), `prevent_self_review: true`, and deployment from `main` only |
| GitHub settings as code | `<repo>/.github/mark-deprecated/configure-github.sh`, `ruleset-main.json` | Apply the environment and a `main` ruleset through the GitHub REST API |
| Ownership | `alpha/CODEOWNERS` (appended), `beta/.github/CODEOWNERS`, `gamma/.github/CODEOWNERS` | `@ann @bob` own the workflow, the script and the rules |
| Org CLI | `org/bin/acme` | The single place for the org. It lists, previews, requests, approves, rejects, shows status and reads records. It is the glue GitHub would otherwise provide: dispatch, environment approval, run history |
| Org config | `org/acme.json` | Org name, the people (ann, bob, carol), each repository's origin, and the act path |
| Run records | `org/runs/<run-id>.json` | One JSON file per run. Each state change is committed and pushed to `origins/org.git` |
| Local state | `org/.acme/` (gitignored) | act cache, origin mirrors, act logs per run, the lock |
| Test | `org/tests/acceptance.sh` | End-to-end checks on sandbox copies |

**How a run works.**
1. **Request.** `acme request` reads the workflow from the origin's `main` and pins its commit as
   `definition_sha`. It clones that commit and runs `act -j preview`, which also runs `validate`.
   It then records the validation result and the preview. The run waits for approval.
2. **Approve.** `acme approve` checks the approver against the environment rules on the current
   `main`, then runs `act -j apply` from the pinned definition.
3. **Apply.** The `apply` job commits `DEPRECATED.md` on top of `origin/main` and pushes it without
   force. It then fetches `main` again and fails unless the file is there.
4. **Observe.** `acme` then fetches the origin itself. It marks the run `succeeded` only if the job
   succeeded and `main` holds exactly the previewed content.

## R1–R8

| # | How it is met | Files |
|---|---|---|
| R1 Owned by the repository | **Definition in the repo:** the workflow, script and approval rules are committed to each origin's `main` (alpha `4ca0c02`, beta `e17f464`, gamma `cf59911`). `CODEOWNERS` assigns them to the maintainers.<br>**Read from `main`:** `acme` reads the definition from the origin's `main` on every call, so a change merged there is what runs next. Nothing about the action is defined in `org/` | `<repo>/.github/workflows/mark-deprecated.yml`, `<repo>/.github/mark-deprecated/`, `<repo>/.github/environments/mark-deprecated.json`, CODEOWNERS |
| R2 Validated | **Rules:** `successor` must fully match `^[a-z0-9-]+/[a-z0-9-]+$` and must not be the repository itself. `reason` must be 20–200 characters, on one line, and not blank. The rules are in `mark_deprecated.py`.<br>**Where they run:** the `validate` job runs before `preview`, and `apply` runs them again. Before calling act, `acme` also rejects unknown or missing inputs, as GitHub does when dispatching.<br>**On failure:** the run is recorded as `refused`, its errors are listed, the exit code is 2, and nothing is pushed | `mark_deprecated.py` (`validate()`), `mark-deprecated.yml` (`validate` job), `org/bin/acme` (`validate_shape`, `dry_run`) |
| R3 Previewed | **Before requesting:** `acme preview` runs the `validate` and `preview` jobs and records nothing.<br>**At request time:** `acme request` stores the preview in the run record. The preview gives `change` (create, update or none), the current and new content, and a unified diff against the current `main`.<br>**What is checked later:** after approval, the exact previewed content is what must appear on `main`. On GitHub, the preview is written to the run's step summary, where it is visible while `apply` waits for approval | `mark_deprecated.py` (`preview`), `mark-deprecated.yml` (`preview` job), `org/bin/acme` (`cmd_preview`, `cmd_request`) |
| R4 Approved | **Gate:** `apply` runs only from `acme approve`.<br>**Who can approve:** someone listed in the repository's `.github/environments/mark-deprecated.json` `reviewers`, read from the current `main`.<br>**Self-review:** with `prevent_self_review`, the approver must not be the requester.<br>**Refusals:** carol, unknown people and the requester are refused with exit 3. Each refusal is recorded as an `approval_denied` event, and the run keeps waiting.<br>**On GitHub:** `configure-github.sh` sets the same rules as environment protection rules | `<repo>/.github/environments/mark-deprecated.json`, `org/bin/acme` (`cmd_approve`), `configure-github.sh` |
| R5 Done means observed | **In the job:** `apply` pushes without force, retrying up to 3 times only on non-fast-forward. It then fetches `main` and exits non-zero unless `DEPRECATED.md` equals the expected bytes.<br>**In `acme`:** `acme` then fetches the origin itself and compares `main:DEPRECATED.md` to the previewed content. It reports `succeeded` only if both checks pass; otherwise it reports `failed` with the reason (for example, the push rejection text) and exit 5.<br>**Errors:** any error after approval also ends as `failed`, never as `running` or `succeeded` | `mark_deprecated.py` (`apply`), `org/bin/acme` (`execute`) |
| R6 One place for the org | **Commands:** from `org/`, `bin/acme list` finds the actions on every repository in `acme.json`, and `bin/acme request --repo <name>` works on any of them. No repository checkout is used.<br>**Discovery:** an action is a `workflow_dispatch` workflow with an environment-gated job. Adding a repository means adding one line to `acme.json` | `org/bin/acme`, `org/acme.json` |
| R7 Audited | **Record fields:** each record has `requested_by` and `requested_at`, `approved_by` and `approved_at`, `inputs`, `state` with `completed_at`, `result` (exit code, push attempts, the commit seen on `main` and its content), and a timestamped `events` list. The events include refused requests, denied approvals and rejections.<br>**History:** records are committed and pushed to `origins/org.git` (`git log` in `org/` shows each step). The commit on the repository's `main` carries `Requested-by`, `Approved-by` and `Action-run` trailers, with the requester as author | `org/runs/*.json`, `org/bin/acme` (`save`, `publish`, `event`) |
| R8 Usable by agents | **Non-interactive:** every command is a flag-driven shell command. Identity comes from `--as` or `ACME_AS`. There are no prompts.<br>**Output:** one JSON document on stdout per command, including errors (`{"error": ...}`). `status` includes a `next` hint.<br>**Exit codes:** 0 ok, 2 inputs refused, 3 not allowed, 4 no such run or wrong state, 5 run failed, 1 unexpected | `org/bin/acme` |

## Commands

**Nothing needs to be running.** `acme` is a CLI and act runs each job on the host. You need
`python3` with PyYAML, `git`, `jq` (for the test and the examples), and `bin/act`. Run everything
from the org checkout:

```bash
cd /tmp/actions-ab/r3/gha-3/org
```

List the actions on all three repositories (or one, with `--repo alpha`):

```bash
bin/acme list
bin/acme list | jq '.actions[] | {repository, action, reviewers, prevent_self_review, inputs}'
```

Preview, without recording anything:

```bash
bin/acme preview --repo alpha --as ann \
  -i successor=acme/dependabot-proxy-v2 \
  -i "reason=Replaced by the v2 proxy, which supports OIDC registries."
```

Request a run as ann. The output gives the run id (for example `alpha-mark-deprecated-2`), state
`waiting_for_approval`, and the preview:

```bash
bin/acme request --repo alpha --as ann \
  -i successor=acme/some-successor \
  -i "reason=A reason that is between 20 and 200 characters."
RUN=$(bin/acme runs --repo alpha --state waiting_for_approval | jq -r '.runs[-1].id')
```

Approve as bob. This runs `apply` and observes `main`. It exits 0 with `"state": "succeeded"`,
or 5 with `"state": "failed"`:

```bash
bin/acme approve "$RUN" --as bob
```

Check status, read the full record, or list all runs:

```bash
bin/acme status "$RUN"
bin/acme record "$RUN"
bin/acme runs
```

A reviewer or the requester can reject a waiting run:

```bash
bin/acme reject "$RUN" --as bob --comment "not now"
```

Run the full acceptance test. It uses sandbox copies and leaves the real origins alone:

```bash
tests/acceptance.sh                # KEEP=1 keeps the sandbox for inspection
```

### Negative tests to try

These need different inputs from the content already on `main`. If `DEPRECATED.md` already has
exactly the requested content, `apply` has nothing to push. The run then succeeds as a no-op:
`job_result.attempts[0].result` is `already-present`, and the content is observed on `main`.

```bash
# Invalid inputs: exit 2, state "refused", main unchanged
bin/acme preview --repo beta -i successor=Acme/Bad -i "reason=A perfectly valid reason here."
bin/acme request --repo beta --as ann -i successor=acme/beta -i "reason=A perfectly valid reason here."  # itself
bin/acme request --repo beta --as ann -i successor=acme/next -i "reason=too short"
bin/acme request --repo beta --as ann -i successor=acme/next -i "reason=$(printf 'x%.0s' {1..201})"

# Self-approval and carol: exit 3, the run stays waiting_for_approval, and the denial is in the record
RUN=$(bin/acme request --repo beta --as ann -i successor=acme/other-store -i "reason=Testing the approval rules here." | jq -r .id)
bin/acme approve "$RUN" --as ann
bin/acme approve "$RUN" --as carol
bin/acme record "$RUN" | jq '.events[] | select(.event == "approval_denied")'

# Rejected push: exit 5, state "failed", the failure quotes the rejection, main unchanged
printf '#!/bin/sh\necho "main is frozen" >&2\nexit 1\n' > ../origins/beta.git/hooks/pre-receive
chmod +x ../origins/beta.git/hooks/pre-receive
bin/acme approve "$RUN" --as bob
bin/acme status "$RUN"
rm ../origins/beta.git/hooks/pre-receive
```

## How GitHub would enforce what act cannot

The workflow is a normal GitHub Actions workflow. On GitHub, `org/bin/acme` is replaced by GitHub
itself:

| Step | On GitHub |
|---|---|
| List | `gh workflow list -R acme/<repo>` |
| Preview | The `preview` job's step summary on the run page |
| Request | `gh workflow run mark-deprecated.yml -R acme/<repo> --ref main -f successor=… -f reason=…` |
| Approve | `gh api -X POST repos/acme/<repo>/actions/runs/<id>/pending_deployments -f state=approved -F 'environment_ids[]=<id>' -f comment=ok`, or the run page |
| Status | `gh run view <id> -R acme/<repo> --json status,conclusion` |
| Record | `gh run view --json`, plus `GET …/runs/<id>/approvals` and the environment's deployment history |

The rules are configured as code. A repository admin runs, once per repository:

```bash
.github/mark-deprecated/configure-github.sh acme/<repo>
```

It sets the following:

- **Environment `mark-deprecated`.** The required reviewers are ann and bob, as users resolved to
  ids. `prevent_self_review: true`, so GitHub refuses an approval by whoever triggered the run.
  Deployment branch policy is `main` only, so a workflow edited on another branch cannot reach the
  gated job. GitHub pauses `apply` until a listed reviewer other than the requester approves.
  carol is not a reviewer, so she cannot approve. GitHub enforces this on the server.
- **Ruleset `main`, from `ruleset-main.json`.** It blocks deletion and force pushes, and requires
  pull requests with code-owner review. Changes to the action's files therefore need ann's or bob's
  review (CODEOWNERS). The GitHub Actions app (integration id 15368) can bypass the ruleset, so
  `apply` can push `DEPRECATED.md` with `GITHUB_TOKEN` (`contents: write` on that job only).
- **Run history.** GitHub keeps each run, its actor, its approvals and its logs.
- **Approved-by.** `mark_deprecated.py` reads `Approved-by` from the run's approvals API when
  `APPROVED_BY` is not set locally.

## What I could not run here

- **GitHub itself.** Not run: environment protection, required reviewers, `prevent_self_review`,
  deployment branch policies, rulesets, CODEOWNERS enforcement, the approvals and
  pending-deployments APIs, `GITHUB_TOKEN`, and run history.
  - `configure-github.sh` and `ruleset-main.json` are untested against the API.
  - The bypass actor id 15368 for GitHub Actions is from memory and unverified.
  - Locally, `org/bin/acme` enforces the same reviewer and self-review rules, read from the same
    JSON file.
- **act limitations.** act skips `actions/checkout`; it copies the local clone instead. act does
  not pause on `environment:`. That is why `acme` runs `-j preview` at request time and
  `-j apply` only after approval. On GitHub it is one run that waits.
- **No server-side enforcement.** The bare origins accept any push and identity is just `--as`.
  Someone could bypass `acme` by running act directly or pushing to an origin. On GitHub,
  authentication, environment protection and the ruleset close that. Here, `acme` is the only
  supported path, and it records who did what.
- **Not tested:** concurrent `acme` invocations. A file lock (`org/.acme/lock`) serializes them.
- **Failed record pushes are not surfaced.** If pushing a record to `origins/org.git` fails, the
  local record under `org/runs/` stays authoritative, and the failure is not reported.

No long-running processes were started; nothing is left running.
