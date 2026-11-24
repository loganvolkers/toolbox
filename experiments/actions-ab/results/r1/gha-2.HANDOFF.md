# HANDOFF: the `mark-deprecated` self-service action (GitHub Actions + act)

`mark-deprecated` is a GitHub Actions workflow that each repository owns. It's run from one org-level
CLI, `org/bin/acme`. On 2026-11-24, ann requested it on alpha, beta and gamma and bob approved
each run. All three runs were reported `succeeded` only after `DEPRECATED.md` was read back from
each origin's `main`:

| Repository | Run id | Commit on `main` |
|---|---|---|
| alpha | `20261124T104701Z-alpha-2669` | `a868675` |
| beta | `20261124T104707Z-beta-3d58` | `18c5f9c` |
| gamma | `20261124T104709Z-gamma-0120` | `73642af` |

Nothing needs to be running: there is no server. Every step is one shell command that prints one JSON
document.

## Layout

| Path | What it is | Owner |
|---|---|---|
| `<repo>/.github/workflows/mark-deprecated.yml` | The action: `workflow_dispatch` inputs, then jobs `validate` → `preview` → `apply` (environment `deprecation`) → `verify` | The repository's maintainers |
| `<repo>/.github/environments/deprecation.json` | Approval policy as code: required reviewers `ann` and `bob`, `prevent_self_review: true`, deploy only from protected branches, plus the `main` ruleset | The repository's maintainers |
| `<repo>/.github/scripts/apply-github-settings.sh` | Applies that policy to GitHub through the REST API (environment, environment variable, ruleset) | The repository's maintainers |
| `alpha/CODEOWNERS`, `beta/.github/CODEOWNERS`, `gamma/.github/CODEOWNERS` | Assign the three files above to `@ann @bob` | The repository's maintainers |
| `org/bin/acme` | Org-level CLI (Python 3 + PyYAML): catalog, preview, request, approve, reject, status, record. It is the glue that act doesn't provide | Platform |
| `org/acme.json` | The org: the repositories (slug and origin path) and the people | Platform |
| `org/.acme/runs/<run-id>/` | Per run: `record.json` (the audit record) and act's JSON logs, `act-request.jsonl` and `act-apply.jsonl`. Not committed (`.gitignore`) | Written by `acme` |

`<repo>` is `/tmp/actions-ab/r1/gha-2/alpha`, `…/beta` or `…/gamma`. The same files are on each
origin's `main`, at `/tmp/actions-ab/r1/gha-2/origins/<repo>.git`.

## How each requirement is met

| # | How | Files |
|---|---|---|
| R1 Owned by the repository | The whole definition is a workflow committed to each origin's `main`: inputs, validation, preview, apply and verify. The approval policy sits beside it. CODEOWNERS gives these files to the maintainers. `acme` reads the definition from the origin's `main` at request time and runs that exact commit (recorded as `definition.commit`), so a maintainer's change to the workflow takes effect on the next request. The org side holds no per-action logic. It discovers any `workflow_dispatch` workflow that has jobs `validate`, `preview`, `apply` and `verify`, where `apply` has an `environment` | `<repo>/.github/workflows/mark-deprecated.yml`, `<repo>/.github/environments/deprecation.json`, CODEOWNERS |
| R2 Validated | The workflow's `validate` job checks the inputs. `successor` must match `^[a-z0-9-]+/[a-z0-9-]+$` and must not be the repository itself. `reason` must be 20 to 200 characters, on one line, not blank. `preview` and `apply` both `need` `validate`. `acme` first checks the names against the declared inputs (unknown or missing input), then runs `validate`. A refused request exits 2 with the errors, is recorded with status `refused`, and nothing is pushed | `mark-deprecated.yml` job `validate`; `org/bin/acme` (`declared_input_errors`, `cmd_request`) |
| R3 Previewed | The `preview` job clones `main`, writes the file and outputs the change (`create`, `update` or `none`), the unified diff, the base commit and the exact new content. It also writes them to the step summary. `acme preview` shows this without recording anything. `acme request` stores it in the run, which waits as `awaiting_approval`. At approval time, `apply` refuses to run if the diff differs from the approved one (`ACME_APPROVED_DIFF_SHA`) or if `DEPRECATED.md` on `main` changed since the preview | `mark-deprecated.yml` jobs `preview` and `apply`; `org/bin/acme` (`cmd_preview`, `cmd_request`) |
| R4 Approved | `apply` runs in environment `deprecation`, whose required reviewers are ann and bob, with `prevent_self_review: true`. act cannot pause for environments, so `acme approve` is the gate. It reads the reviewers from `.github/environments/deprecation.json` on the origin's `main` at approval time and denies (exit 3) anyone who isn't a reviewer, such as carol, and the requester. Only after an approval does it run `apply`. Denied attempts are recorded as `approval_denied` events. A second gate inside `apply`, which runs only under act, rechecks the reviewer, self-approval and the approved diff | `deprecation.json`; `org/bin/acme` (`cmd_approve`); `mark-deprecated.yml` step "Approval gate" |
| R5 Done means observed | `apply` pushes without force. If the push fails and `main` hasn't moved, which means a rejection, the job fails. If `main` moved because of a race, it rebases and retries, up to 3 attempts. `verify` clones `main` again and requires both the pushed commit on `main` and `DEPRECATED.md` to be byte-identical to the expected content. After the workflow, `acme` reads `main:DEPRECATED.md` directly from the bare origin and compares it with the previewed content. The run is `succeeded` only if all four jobs succeeded **and** that observation matches. Otherwise it is `failed`, with the reasons in `outcome.problems` and exit code 1. I tested a pre-receive hook that rejects pushes (reported `failed`) and a post-receive hook that reverts `main` after accepting the push (also `failed`) | `mark-deprecated.yml` jobs `apply` and `verify`; `org/bin/acme` (`observe`, `cmd_approve`) |
| R6 One place for the org | From `org/`, `bin/acme list` lists `mark-deprecated` for alpha, beta and gamma with its inputs and approvers. `bin/acme request <repo> mark-deprecated …` works for any repository listed in `org/acme.json` | `org/bin/acme`, `org/acme.json` |
| R7 Audited | `org/.acme/runs/<id>/record.json` holds the inputs, `requested.by/at`, `preview.at`, `approval.by/at` (with the policy commit and reviewers), `rejection`, `outcome.status/at` (commit, pushed, observed `main` sha and content sha256, problems), per-job results with start and finish times, and an `events` list (`requested`, `previewed`, `approval_denied`, `approved`, `rejected`, `refused`, `workflow_finished`, `succeeded`/`failed`), each with `at` and `actor`. The commit on `main` carries `Requested-by:`, `Approved-by:` and `Action-run:` trailers | `org/bin/acme` (`cmd_record`); `org/.acme/runs/` |
| R8 Usable by agents | Every command is non-interactive and prints one JSON document on stdout, including usage errors. Exit codes are 0 ok, 1 run failed, 2 invalid input or usage, 3 forbidden, 4 not found, 5 wrong state. Identity comes from `--as <login>` or `ACME_USER`. Inputs come from `--input name=value` (repeatable) or `--inputs-json '{…}'`. `approve` blocks until the outcome has been observed | `org/bin/acme` |

## Commands

Requirements: `python3` with PyYAML, `git`, and act at `/tmp/actions-ab/r1/gha-2/bin/act`. `acme` always
runs act with `-P ubuntu-latest=-self-hosted --json --no-cache-server` and a throwaway `HOME`.
Nothing has to be started.

```bash
cd /tmp/actions-ab/r1/gha-2/org

# Start: nothing to start (no server, no daemon).

# List the action on all three repositories (inputs, definition commit, approvers)
bin/acme list
bin/acme list --repo beta --action mark-deprecated

# Preview only: no record, no change
bin/acme preview alpha mark-deprecated --as ann \
  --input successor=acme/dependabot-proxy-v2 \
  --input "reason=Replaced by the v2 proxy, which supports OIDC registries."

# Request as ann: validates and previews, then waits for approval. Prints the run (.run.id) and .preview.
REQ=$(bin/acme request alpha mark-deprecated --as ann \
  --input successor=acme/dependabot-proxy-v2 \
  --input "reason=Replaced by the v2 proxy, which supports OIDC registries.")
echo "$REQ"
RUN=$(echo "$REQ" | jq -r .run.id)

# Approve as bob: runs apply and verify, observes main, prints the outcome (exit 0 succeeded, 1 failed)
bin/acme approve "$RUN" --as bob

# Status and full record
bin/acme status "$RUN"
bin/acme record "$RUN"

# Other commands
bin/acme reject "$RUN" --as bob --comment "not now"   # a reviewer or the requester; only while awaiting_approval
bin/acme runs [--repo alpha] [--status succeeded]
bin/acme --help
```

The same request as JSON:
`bin/acme request beta mark-deprecated --as ann --inputs-json '{"successor":"acme/valtio-store","reason":"State management moves to the shared store package."}'`

The file written is exactly `# Deprecated: use <successor>`, a blank line, then `<reason>`. Like any
text file, it ends with one newline.

## Testing the refusals

All three repositories already contain `DEPRECATED.md` from the runs above. A new request with the
**same** inputs previews `change: none`. Approving it pushes nothing, and it succeeds after the
content is observed on `main`. That's correct, since `main` already says it. To exercise a push, use
a different `successor` or `reason`.

```bash
cd /tmp/actions-ab/r1/gha-2/org

# Invalid inputs: exit 2, status "refused", nothing changes
bin/acme request alpha mark-deprecated --as ann --input successor=Acme/Bad_Name --input "reason=too short"
bin/acme request alpha mark-deprecated --as ann --input successor=acme/alpha --input "reason=Twenty or more characters here."
bin/acme request alpha mark-deprecated --as ann --input successor=acme/x            # missing reason
bin/acme request alpha mark-deprecated --as ann --input successor=acme/x --input reason="$(printf 'x%.0s' {1..201})"

# Self-approval and carol: exit 3 approval-denied, recorded as approval_denied events
RUN=$(bin/acme request alpha mark-deprecated --as ann --input successor=acme/proxy-v3 \
  --input "reason=Superseded by the v3 proxy for all registries." | jq -r .run.id)
bin/acme approve "$RUN" --as ann
bin/acme approve "$RUN" --as carol
bin/acme record "$RUN" | jq .record.events

# Rejected push: reported "failed", never "succeeded"
printf '#!/bin/sh\necho "push rejected by test hook" >&2\nexit 1\n' > ../origins/alpha.git/hooks/pre-receive
chmod +x ../origins/alpha.git/hooks/pre-receive
bin/acme approve "$RUN" --as bob ; echo "exit=$?"     # exit 1, status failed, outcome.problems explains why
git --git-dir ../origins/alpha.git show main:DEPRECATED.md   # unchanged
rm ../origins/alpha.git/hooks/pre-receive
```

## How GitHub would enforce what act can't, and what I couldn't run here

| Feature | Configured as code | How GitHub enforces it | Run here? |
|---|---|---|---|
| Required reviewers on `apply` | `environment: deprecation` in the workflow; `reviewers` in `deprecation.json`, applied by `apply-github-settings.sh` (`PUT /repos/{o}/{r}/environments/deprecation`) | The run pauses at `apply` until a listed reviewer approves under *Review deployments* | No. act ignores `environment`; `acme approve` enforces the same rule |
| Prevent self-review | `prevent_self_review: true` | The person who triggered the run cannot approve it | No. `acme approve` enforces it |
| Only `main` can deploy | `deployment_branch_policy.protected_branches: true` | A copy of the workflow on a feature branch cannot reach the environment, so it cannot use its secret | No |
| `main` protected | ruleset `protect-main`: no deletion or force-push, PRs with code-owner review. The only bypass actor is the deprecation GitHub App | Humans change the action through reviewed PRs. Only the App can push straight to `main`, and its private key is an **environment** secret (`DEPRECATION_APP_PRIVATE_KEY`), released only to an approved `apply` job | No. The bare origins have no protection; I only checked the API bodies with `apply-github-settings.sh … --dry-run` |
| CODEOWNERS | `@ann @bob` own the action files | Required code-owner review under the ruleset | No |
| Push token and approver identity | Steps "Mint a push token" (`actions/create-github-app-token@v1`) and "Read who approved this run" (`GET …/actions/runs/{id}/approvals`) | Run only on GitHub (`if: ${{ !env.ACT }}`) | No. Skipped under act; locally `acme` passes the approver in `ACME_APPROVED_BY` |
| Run history and audit | Not needed | Actions run history, the deployment approvals API and the audit log | No. `org/.acme/runs/*/record.json` stands in for them |
| Org-level entry point | `org/bin/acme` | Each command maps to an API call. `list` reads the workflow files through the contents API. `request` runs `gh workflow run mark-deprecated.yml -R acme/<repo> -f successor=… -f reason=…` as the requester. The preview is the `preview` job's output and summary. `approve` posts to `POST …/actions/runs/{id}/pending_deployments`. `status` and `record` read the run, its jobs and its approvals | Only the local backend is implemented; a GitHub backend is not |

What did run here: every workflow job (`validate`, `preview`, `apply`, `verify`) under act 0.2.98 in host
mode against the bare origins. That includes the refusals, the self-approval and carol denials, a
rejected push, a push that `main` reverted, a change to `main` between preview and approval, a
no-op re-run and a reject. All of these ran on scratch copies of the origins, which I then deleted. The
workflow passes actionlint 1.7.12 (without shellcheck).

**Local trust model.** There are no passwords, so `--as` is trusted, as the brief allows. The approval
gate is `acme`. Anyone with a shell here could bypass it by running act directly or pushing to a bare
origin. On GitHub, the environment, the environment secret and the ruleset close that gap.
