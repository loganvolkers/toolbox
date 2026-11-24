# Handoff: `mark-deprecated` on GitHub Actions

`mark-deprecated` is a GitHub Actions workflow that each repository owns. `org/bin/acme-actions` is
the org's single command line. It finds the action on each origin's `main`, runs the workflow
locally with act in host mode, holds each run until a maintainer other than the requester approves,
checks `main` itself before reporting success and records every step. On GitHub, the
environment's required reviewers and run history would take that runner's place.

All three requested runs succeeded. ann requested each run, bob approved it, and `DEPRECATED.md`
was observed on the origin's `main`:

| Run | Repository | Commit on `main` |
|---|---|---|
| `alpha-20261124T121433Z-fabb` | alpha | `6b58478b8421` |
| `beta-20261124T121435Z-51f1` | beta | `6175b3d83918` |
| `gamma-20261124T121436Z-614e` | gamma | `c78cb20d480c` |

Read any of them with `org/bin/acme-actions record <run id>`.

## What is where

Paths are relative to `/tmp/actions-ab/r3/gha-2`.

| Path | What it is |
|---|---|
| `<repo>/.github/workflows/mark-deprecated.yml` | The action. Job `plan` validates the inputs and previews the change. Job `apply` runs only with `mode=apply`, in environment `mark-deprecated`: it checks the approval when run under act, validates again, commits and pushes `DEPRECATED.md`, then fetches `main` to verify it |
| `<repo>/.github/mark-deprecated/settings.json` | Owned by the repository: the maintainers who may approve, `prevent_self_review`, the environment, the target file and branch, and the input rules (successor pattern, reason length) |
| `<repo>/.github/mark-deprecated/mark_deprecated.py` | The workflow's steps: `validate`, `preview`, `guard`, `apply` and `verify`. Each prints JSON, writes `<step>.json` and appends to the step summary |
| `<repo>/.github/mark-deprecated/apply-github-settings.sh` | GitHub settings as code. It creates environment `mark-deprecated` with the maintainers as required reviewers, `prevent_self_review: true` and `main` as the only deployment branch. It also creates ruleset `main`: no deletion, no force push, and changes need a pull request with a code owner's review, which a GitHub App may bypass to push `DEPRECATED.md` |
| `alpha/CODEOWNERS`, `beta/.github/CODEOWNERS`, `gamma/.github/CODEOWNERS` | Make ann and bob code owners of the workflow and of `.github/mark-deprecated/` |
| `org/bin/acme-actions` | The org command line. It is the local stand-in for `workflow_dispatch`, the environment approval gate and the run history |
| `org/acme-actions.json` | The org's repositories (name, slug, origin), its people, and the paths to act and its platform |
| `org/tests/smoke.sh` | 20 end-to-end checks against throwaway bare copies of the origins (the real origins are untouched): invalid inputs, self-approval, carol, an unknown person, success, a rejected push and rejection |
| `org/state/runs/<id>/record.json` | One record per run. Next to it are `plan.log` and `apply.log` (act's output), the event payloads, and `plan/` and `apply/` holding each step's JSON. Not committed (`org/.gitignore`) |
| `org/state/audit.jsonl` | An append-only log of every event across all runs, including denied approvals |

## R1 to R8

| # | How it is met | Files |
|---|---|---|
| R1 Owned by the repository | Each repository's definition is committed to its origin's `main`: the workflow, the settings (approvers and input rules) and the step script. `acme-actions` reads the action from the origin's `main` (`git show main:…`), and every run clones `main` fresh, so a change a maintainer pushes takes effect on the next run. `list` shows the definition's commit and who last changed it. CODEOWNERS plus the ruleset in `apply-github-settings.sh` make a change to the definition need ann's or bob's review on GitHub | `<repo>/.github/workflows/mark-deprecated.yml`, `<repo>/.github/mark-deprecated/settings.json`, `<repo>/.github/mark-deprecated/mark_deprecated.py`, CODEOWNERS |
| R2 Validated | The `validate` step checks the rules in the repository's `settings.json`. `successor` must fully match `^[a-z0-9-]+/[a-z0-9-]+$` and may not name this repository itself. `reason` must be 20 to 200 characters, one line (no control characters) and not blank. `acme-actions` refuses unknown or missing inputs before running anything. An invalid request is recorded as `refused`, exits 2 and never reaches approval. The `apply` job validates again before it writes | `mark_deprecated.py` (`check_inputs`), `settings.json` (`inputs`), `org/bin/acme-actions` (`parse_inputs`, `plan_result`) |
| R3 Previewed | `acme-actions preview` runs the `plan` job and returns the change (`create`, `update` or `none`), the base commit, the content before and after, and a unified diff; nothing is recorded or pushed. `request` runs the same job and stores the preview in the record, so the approver sees it with `status` and `record`. On GitHub, the preview is the `plan` job's step summary, which shows while the run waits for approval | `mark_deprecated.py` (`cmd_preview`), `acme-actions` (`cmd_preview`, `cmd_request`) |
| R4 Approved | A run waits in `awaiting_approval` and only `approve` starts the `apply` job. `approve` refuses the requester (`prevent_self_review`) and anyone not in the repository's current `settings.json` maintainers, such as carol. Both refusals exit 3 and are recorded as `approval_denied`. Under act, the workflow's `guard` step checks the same rule again. On GitHub, `environment: mark-deprecated` on the `apply` job, with the required reviewers and `prevent_self_review` from `apply-github-settings.sh`, enforces it | `acme-actions` (`cmd_approve`), `mark_deprecated.py` (`cmd_guard`), `mark-deprecated.yml` (`environment:`), `apply-github-settings.sh` |
| R5 Done means observed | The `verify` step fetches `main` and checks that the run's commit is on it and that `DEPRECATED.md` is byte-for-byte the expected content. Afterwards, `acme-actions` reads the bare origin's `main` itself and compares it with the content that was previewed and approved. The run is `succeeded` only if act exited 0, every step passed and both checks matched. Anything else is `failed`, with the reasons in `outcome.errors`. Every approved run pushes a commit (an empty audit commit if the content is already right), so a rejected push always fails the run | `mark_deprecated.py` (`cmd_apply`, `cmd_verify`), `acme-actions` (`observe`, `cmd_approve`) |
| R6 One place for the org | From `org/`, `bin/acme-actions list` lists every action on every repository's `main` (here, `mark-deprecated` × 3). `preview`, `request`, `approve`, `reject`, `status`, `record` and `runs` work on any repository without entering its checkout. The repositories are listed in `org/acme-actions.json` | `org/bin/acme-actions`, `org/acme-actions.json` |
| R7 Audited | `record.json` holds `requested_by`/`requested_at`, `approved_by`/`approved_at` (or `rejected_by`/`rejected_at`), `inputs`, the preview, `outcome` (status, time, commit, errors), `observed` (what was read from `main`, and when), the definition commit and a timestamped `events` list. That list includes denied approvals and refusals. Every event is also appended to `org/state/audit.jsonl`. The commit on `main` carries `Requested-by:`, `Approved-by:` and the run id. On GitHub, the run history and the environment's deployment reviews record the same | `org/state/runs/<id>/record.json`, `org/state/audit.jsonl`, `acme-actions` (`event`) |
| R8 Usable by agents | Every command is non-interactive, takes identity from `--as` (or `$ACME_USER`) and prints JSON on stdout. Inputs come from `-f key=value` or `--inputs-file file.json` (`-` reads stdin). Exit codes: 0 ok, 1 the run or a tool failed, 2 inputs refused, 3 not permitted, 4 not found, 5 wrong state (for example, approving a finished run). `status` includes a `next` list of the exact approve commands | `org/bin/acme-actions` |

## Commands

Nothing has to be running: there is no server. `acme-actions` runs act for the duration of each
command. It needs `python3` with PyYAML, `git`, `jq` (for the smoke test and the settings script)
and act at `bin/act`. act fetches `actions/checkout` from github.com once, even though the step is
skipped under act, and then reuses its cache (`--action-offline-mode`).

```bash
cd /tmp/actions-ab/r3/gha-2/org
A=bin/acme-actions

# List the actions on every repository's main
$A list
$A list --repo beta --action mark-deprecated

# Preview: validate and show the change, without recording or pushing anything
$A preview alpha mark-deprecated --as ann \
  -f successor=acme/dependabot-proxy-v2 \
  -f reason="Replaced by the v2 proxy, which supports OIDC registries."

# Request, as ann: validates, previews, records, and waits for approval. Prints the run id (.id)
ID=$($A request alpha mark-deprecated --as ann \
  -f successor=acme/dependabot-proxy-v2 \
  -f reason="Replaced by the v2 proxy, which supports OIDC registries." | jq -r .id)
# Or pass the inputs as JSON:
#   echo '{"successor":"acme/x","reason":"..."}' | $A request alpha mark-deprecated --as ann --inputs-file -

# Approve, as bob: runs the apply job, then verifies main. Returns the final status
$A approve "$ID" --as bob

# Reject instead (a maintainer or the requester)
$A reject "$ID" --as bob --comment "not yet"

# A run's status, its full record, and all runs
$A status "$ID"
$A record "$ID"
$A runs --repo alpha --status succeeded
```

How to test the refusals and failures:

```bash
$A preview alpha mark-deprecated --as ann -f successor=Acme/X -f reason=short; echo $?   # 2, with both errors
ID=$($A request beta mark-deprecated --as ann -f successor=acme/x -f reason="Twenty characters at least." | jq -r .id)
$A approve "$ID" --as ann;   echo $?    # 3: ann requested it
$A approve "$ID" --as carol; echo $?    # 3: carol is not a maintainer
$A record "$ID" | jq '.events'          # both denials are recorded

# A rejected push: make the origin refuse pushes, then approve a run
printf '#!/bin/sh\necho rejected >&2\nexit 1\n' > ../origins/beta.git/hooks/pre-receive
chmod +x ../origins/beta.git/hooks/pre-receive
$A approve "$ID" --as bob; echo $?      # 1, status "failed", outcome "apply: push to main was rejected"
rm ../origins/beta.git/hooks/pre-receive

# Everything above, against throwaway copies of the origins
tests/smoke.sh
```

## How GitHub would enforce what act cannot

| Feature | Configured in | On GitHub |
|---|---|---|
| Required reviewers, preventing self-review | `apply-github-settings.sh` (environment `mark-deprecated`), `mark-deprecated.yml` (`environment:` on `apply`) | The `apply` job pauses with "Waiting for review". Only ann or bob can approve, and `prevent_self_review` refuses the person who triggered the run, so carol and the requester cannot approve. Approval goes through the UI or `gh api -X POST repos/acme/<repo>/actions/runs/<run>/pending_deployments -F 'environment_ids[]=<id>' -f state=approved -f comment=…` |
| Only `main` deploys | `apply-github-settings.sh` (deployment branch policy) | A dispatch from another branch cannot use the environment |
| Ownership of the definition | CODEOWNERS and the ruleset in `apply-github-settings.sh` | A change to the workflow or to `.github/mark-deprecated/` needs a pull request approved by a code owner (ann or bob) |
| Pushing past the ruleset | `vars.MARK_DEPRECATED_APP_ID` and `secrets.MARK_DEPRECATED_APP_KEY` in the workflow, and the bypass in the ruleset | The `apply` job pushes with a GitHub App token, and only that App may bypass the pull request rule |
| Run history | GitHub itself | Each run shows who triggered it, the inputs, the reviewer and the logs. Request with `gh workflow run mark-deprecated.yml -R acme/<repo> -f successor=… -f reason=… -f mode=apply` |
| One run at a time per repository | `concurrency: mark-deprecated` in the workflow | Locally, `acme-actions` holds a per-repository lock instead |

## What I could not run here

- Anything that needs github.com as the code host: environments and their reviewers, `prevent_self_review`, rulesets, CODEOWNERS enforcement, the GitHub App token, `actions/checkout` (skipped under act, where the job clones `MD_ORIGIN` instead) and GitHub's run history and step summary. I ran `apply-github-settings.sh` only with `DRY_RUN=1`, which prints the API calls it would make.
- On GitHub, `requested_by`, `approved_by` and `run_id` are left empty: the workflow uses `github.triggering_actor` and `github.run_id`, and the environment decides who approves.

## Limits worth knowing

- Identity is whatever `--as` says, as the sandbox allows. Records are local JSON files, not tamper-proof storage.
- If `acme-actions` is killed during `approve`, the run stays `running`.
- `reason` must be one line, and `successor` may not be the repository itself. Both rules come from my reading of "another repository" and of a one-paragraph reason; change them in each repository's `settings.json` and `mark_deprecated.py`.
- Repeating a run whose content is already on `main` pushes an empty audit commit, so the push itself is observed.
