# HANDOFF: `mark-deprecated`, a self-service action on GitHub Actions

**State:** `mark-deprecated` is defined in `alpha`, `beta` and `gamma`, each on its origin's `main`.
It ran once per repository, requested by ann and approved by bob, and all three runs are reported
`succeeded`:

| Run | Repository | Successor | Commit on origin `main` |
|---|---|---|---|
| `alpha-1` | alpha | `acme/dependabot-proxy-v2` | `f83ae40` |
| `beta-1` | beta | `acme/valtio-store` | `bd214fa` |
| `gamma-1` | gamma | `acme/graphify-core` | `e908b3f` |

**Nothing needs to be running.** There is no server: the org tool `org/bin/acme` is a CLI. It reads
each repository's action from its origin's `main`, runs the workflow's jobs with act in host mode,
enforces the approval gate, checks origin `main` itself, and keeps run records on disk.

## How it fits together

```
<repo>/.github/workflows/mark-deprecated.yml       the action: workflow_dispatch, jobs validate -> preview -> apply -> verify
<repo>/.github/self-service/mark_deprecated.py     the logic each job runs (validate | preview | apply | verify)
<repo>/.github/self-service/mark-deprecated.json   catalog entry + approval gate (environment "deprecation", reviewers, prevent_self_review)
<repo>/.github/self-service/repository-settings.json  collaborators (ann, bob: maintain; carol: push) + ruleset on main
<repo>/.github/self-service/apply-github-settings.sh  applies both JSON files to GitHub through the REST API
alpha/CODEOWNERS, beta/.github/CODEOWNERS, gamma/.github/CODEOWNERS   @ann @bob own the files above

org/bin/acme          the org's one entry point (list, preview, request, approve, reject, status, record, runs, audit)
org/repos.json        the org's repositories and their origins; also where act is
org/people.json       who exists (ann, bob, carol)
org/.acme/            run state (gitignored): runs/<id>/record.json, request.log, execute.log, out/*.json; audit.jsonl
```

On GitHub, one `workflow_dispatch` run holds the whole lifecycle: the `apply` job waits at the
`deprecation` environment until a required reviewer approves. act cannot pause a run for approval,
so `acme` splits it at that gate:

- `request` runs the jobs up to `preview` (`act -j preview`, which also runs `validate`).
- `approve` runs the jobs up to `verify` (`act -j verify`: validate, preview, apply, verify), but
  only after it has checked the environment's rules.

Both phases use the workflow at the commit of `main` the run was requested on, which is recorded as
`definition.sha`. The job names for each phase come from the repository's
`mark-deprecated.json` (`phases`).

## Commands

Run everything from the org directory. Every command prints JSON on stdout (errors too, as
`{"ok": false, "error": {"kind", "message", ...}}`) and a one-line message on stderr when it fails.
Identity is `--as <login>`, or the `ACME_USER` environment variable.

Exit codes:

| Code | Meaning |
|---|---|
| 0 | ok |
| 1 | the run failed |
| 2 | invalid input or usage |
| 3 | forbidden |
| 4 | not found |
| 5 | conflict: the run is not awaiting approval |

Requirements: `python3` with PyYAML (installed here: 6.0.4), `git`, and act at
`/tmp/actions-ab/r2/gha-1/bin/act`, which `org/repos.json` points to. Override it with `ACME_ACT`.

```bash
cd /tmp/actions-ab/r2/gha-1/org

# Start: nothing to start.

# List mark-deprecated on all three repositories (inputs, definition commit, reviewers, owners)
bin/acme list --action mark-deprecated

# Preview: runs the repository's validate + preview jobs; changes nothing, creates no run
bin/acme preview alpha mark-deprecated \
  -f successor=acme/dependabot-proxy-v2 \
  -f "reason=Replaced by the v2 proxy, which supports OIDC registries." --as ann

# Request as ann: validates and previews again, then the run waits for approval. Prints the run, including .run.id
bin/acme request alpha mark-deprecated \
  -f successor=acme/dependabot-proxy-v2 \
  -f "reason=Replaced by the v2 proxy, which supports OIDC registries." --as ann
RUN=$(bin/acme runs --repo alpha --status awaiting_approval | jq -r '.runs[-1].id')

# Approve as bob: runs apply + verify, then observes origin main; returns when the run has finished
bin/acme approve "$RUN" --as bob

# Status (short) and the full record
bin/acme status "$RUN"
bin/acme record "$RUN"

# Also useful
bin/acme reject "$RUN" --as bob --comment "why"   # a reviewer, or the requester withdrawing
bin/acme runs [--repo alpha] [--status succeeded]
bin/acme audit [--run "$RUN"]                     # append-only event log, including refused approvals and requests
```

Run ids are `<repository>-<n>`, so the next alpha run is `alpha-2`. A run's `status` is one of:

| Status | Meaning |
|---|---|
| `validating` | the request is being validated and previewed |
| `refused` | the inputs were invalid; nothing ran past validation |
| `awaiting_approval` | waiting for a reviewer |
| `running` | approved; the workflow is executing |
| `succeeded` | the change was observed on origin `main` |
| `failed` | the change did not land |
| `rejected` | a reviewer rejected it, or the requester withdrew it |

## Requirements

| # | How it is met | Files |
|---|---|---|
| R1 Owned by the repository | Each repository's action is defined in that repository: the workflow, its logic, its catalog entry and approval gate, and its GitHub settings. It was committed to the origin's `main` (alpha `682902c`, beta `74283ee`, gamma `0bedd04`). CODEOWNERS makes ann and bob the owners of those files, and the ruleset requires a code-owner review on `main`. `acme` reads the definition from origin `main` on every call and pins each run to that commit, so whatever a team merges is what runs. To change validation, the template or the reviewers, edit the repository's files. | `<repo>/.github/workflows/mark-deprecated.yml`, `<repo>/.github/self-service/*`, `alpha/CODEOWNERS`, `{beta,gamma}/.github/CODEOWNERS` |
| R2 Validated | The `validate` job (`mark_deprecated.py validate`) checks the inputs. `successor` must fully match `^[a-z0-9-]+/[a-z0-9-]+$` and must not be the repository itself. `reason` must be 20 to 200 characters on one line, and not blank. `preview` and `apply` check again. `acme` also refuses unknown or missing inputs, using the workflow's declared `workflow_dispatch` inputs. An invalid request exits 2 with the errors and is recorded as `refused`. No run reaches `apply`, so nothing changes. | `.github/self-service/mark_deprecated.py` (`validation_errors`), `org/bin/acme` (`check_inputs`, `preview_in`) |
| R3 Previewed | The `preview` job renders `DEPRECATED.md`, diffs it against the current origin `main`, and writes the diff to the step summary and to `preview.json`. `acme preview` shows it without creating a run. `acme request` stores it in the run (`.run.preview`, plus a `previewed` event) before anyone can approve, so the requester and the approver both see it. | `mark_deprecated.py` (`preview`), `org/bin/acme` (`cmd_preview`, `cmd_request`) |
| R4 Approved | `apply` is the only job that writes, and it is gated by `environment: deprecation`. `mark-deprecated.json` sets that environment's required reviewers (ann, bob) with `prevent_self_review: true` and `can_admins_bypass: false`, and it deploys from `main` only. On GitHub, `apply-github-settings.sh` applies these and GitHub enforces them. Locally, `acme approve` enforces the same JSON, read at the run's pinned commit: the approver must be a reviewer, so carol is refused (exit 3), and must not be the requester, so ann cannot approve her own run (exit 3). Every refused attempt is recorded as an `approval_refused` event. A run executes at most once: the state changes under a lock, and a second approve exits 5. | `.github/workflows/mark-deprecated.yml` (`apply.environment`), `.github/self-service/mark-deprecated.json`, `apply-github-settings.sh`, `org/bin/acme` (`cmd_approve`) |
| R5 Done means observed | The `verify` job fetches origin `main` and compares `DEPRECATED.md` byte for byte with the expected content. If they differ it fails, and on GitHub the run fails. `acme` then reads origin `main` itself (`git show main:DEPRECATED.md` on the bare origin). It reports `succeeded` only if the workflow exited 0, `verify` passed, and its own observation matches the previewed content. Otherwise the run is `failed`, with the errors, for example `push to main was rejected: remote: ...`. The record keeps the observed `main` sha and the file content. | `mark_deprecated.py` (`apply`, `verify`), `org/bin/acme` (`cmd_approve`, `outcome.observed`) |
| R6 One place for the org | From `org/`, `bin/acme list` discovers every `.github/self-service/<action>.json` on the `main` of each repository in `org/repos.json`. `preview`, `request`, `approve`, `status` and `record` take a repository name or run id. Nobody needs to enter a repository checkout: `acme` works against the origins. | `org/bin/acme`, `org/repos.json`, `org/people.json` |
| R7 Audited | `org/.acme/runs/<id>/record.json` holds `requested_by`/`requested_at`, `approved_by`/`approved_at`, `inputs`, `definition.sha`, `preview`, `started_at`, `finished_at` and `outcome`. The outcome includes the conclusion, the commit, the observed `main` sha and content, and the errors. It also holds a timestamped `events` list: requested, previewed, awaiting_approval, approval_refused, approved, started, observed, then succeeded, failed, refused or rejected. `org/.acme/audit.jsonl` keeps every event as an append-only log, including previews. The act output is kept in `request.log` and `execute.log`. The commit on `main` carries `Requested-by`, `Approved-by` and `Run` trailers. On GitHub, the equivalents are the run history and the environment's approval log. | `org/bin/acme` (`add_event`, `summary`), `org/.acme/` |
| R8 Usable by agents | Every step is a non-interactive CLI call with JSON on stdout and distinct exit codes. Inputs are passed as `-f key=value` and identity as `--as` or `ACME_USER`. `approve` blocks until the run has finished, so its output is final. `status` is cheap to poll. | `org/bin/acme` |

## Testing it

All of these were run against copies of the origins before the real runs, and behaved as shown:

```bash
cd /tmp/actions-ab/r2/gha-1/org
# Invalid inputs: exit 2, errors listed, a "refused" run is recorded, origin unchanged
bin/acme request alpha mark-deprecated -f successor=Acme/X -f reason=short --as ann
bin/acme request alpha mark-deprecated -f successor=acme/alpha -f "reason=Points at itself, which is refused." --as ann
bin/acme preview alpha mark-deprecated -f successor=acme/x     # missing reason, refused before act runs

# Self-approval and carol: exit 3, recorded as approval_refused; the run stays awaiting_approval
bin/acme request beta mark-deprecated -f successor=acme/new-store -f "reason=Testing the approval rules again." --as ann
bin/acme approve beta-2 --as ann
bin/acme approve beta-2 --as carol

# Rejected push: the run is reported failed (exit 1); outcome.errors holds the hook's message
printf '#!/bin/sh\necho "main is frozen" >&2\nexit 1\n' > ../origins/beta.git/hooks/pre-receive
chmod +x ../origins/beta.git/hooks/pre-receive
bin/acme approve beta-2 --as bob; bin/acme status beta-2
rm ../origins/beta.git/hooks/pre-receive
```

To test a rejected push, use inputs that differ from the current `DEPRECATED.md`. If `main` already
has exactly the requested content, `apply` has nothing to push. In that case it records
`"changed": false`, and the run succeeds because the content is observed on `main`.

The file is written as `# Deprecated: use <successor>\n\n<reason>\n`, with a trailing newline.

`ACME_REGISTRY=<path to a repos.json>` and `ACME_STATE_DIR=<dir>` point `acme` at other origins and
another state directory. That is how the failure paths were tested without touching the real origins.

## What GitHub enforces, and what could not run here

| Feature | On GitHub | Here |
|---|---|---|
| Environment `deprecation` with required reviewers ann and bob, `prevent_self_review`, no admin bypass, deployable from `main` only | `apply-github-settings.sh acme/<repo>` creates it (`PUT /repos/{o}/{r}/environments/deprecation` plus a branch policy). GitHub pauses `apply` and shows "Review deployments" only to ann and bob. The requester cannot approve. | Not run: there is no GitHub. Its output was checked with `DRY_RUN=1`. `acme approve` enforces the same JSON. |
| Who is a maintainer | The collaborators in `repository-settings.json`, applied by the same script | `acme` uses the reviewer list and `org/people.json` |
| Ruleset on `main`: code-owner review, no force-push, no deletion | Applied by the same script. The GitHub Actions app (integration 15368) is a bypass actor so `apply` can push `DEPRECATED.md`. Narrow that if your org disallows it, for example with a GitHub App token used only by this workflow. | Not run. Locally the origins accept any push. A `pre-receive` hook stands in for a rejected push. |
| CODEOWNERS | Requests a review from ann or bob on changes to the action's files. The ruleset makes that review required. | The file is committed, but nothing enforces it locally. |
| Run history and approval log | `gh run list --workflow mark-deprecated.yml`, `gh run view <id>`, `GET /repos/{o}/{r}/actions/runs/{id}/approvals` | `org/.acme/runs/*/record.json` and `org/.acme/audit.jsonl` |
| `concurrency: mark-deprecated` | Serialises runs per repository | act ignores it. `acme` locks around state changes, and a non-fast-forward push fails the run. |
| `github.repository` | `acme/<repo>` | act derives it from the origin path, so `acme` passes `vars.ACME_REPOSITORY` |
| Approver in the commit trailer | `apply` reads it from the run's approvals API | `acme` passes `ACME_APPROVED_BY` |

The local CLI corresponds to these GitHub calls, which can't be tested here:

```bash
gh workflow run mark-deprecated.yml -R acme/alpha -f successor=... -f reason=...     # request
gh run view <id> -R acme/alpha --log                                                 # preview: the job log and step summary
gh api -X POST repos/acme/alpha/actions/runs/<id>/pending_deployments \
  -F "environment_ids[]=<deprecation env id>" -f state=approved -f comment=ok        # approve
gh run view <id> -R acme/alpha --json status,conclusion                              # status
```

The org-wide view would query `gh api repos/acme/<repo>/contents/.github/self-service` for each
repository.

Not run here: anything that needs GitHub itself (environments and their approval UI, rulesets,
CODEOWNERS enforcement, the REST calls in `apply-github-settings.sh`, the approvals API). Under act,
the `actions/checkout` step copies the local clone instead of fetching from GitHub, and jobs run
directly on this host (`-P ubuntu-latest=-self-hosted`). No Docker was used.
