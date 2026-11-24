# Handoff: `mark-deprecated` on GitHub Actions

`mark-deprecated` is a GitHub Actions workflow that each repository owns. The org-level CLI
`org/bin/acme-actions` lists it on every repository, previews it, requests it and approves it.
It also observes the result and keeps the audit record.

On 2026-11-24 I ran it on all three repositories. In each run ann requested, bob approved, and
the run was reported `succeeded` only after `DEPRECATED.md` with the exact content was read back
from the origin's `main`:

| Run | Repository | Commit on `main` | Status |
|---|---|---|---|
| `run-0001` | acme/alpha | `ef69197` | succeeded |
| `run-0002` | acme/beta | `49b007b` | succeeded |
| `run-0003` | acme/gamma | `742c298` | succeeded |

All paths below are under `/tmp/actions-ab/r6/gha-1/`.

## What must be running

Nothing needs to be running. There is no server and no daemon. Every step is one command that
exits when it's done. Each run's record lives in a file, `org/runs/<id>.json`, so a waiting
run survives between commands.

The CLI needs `python3` with PyYAML (already installed here), `git`, and act at `bin/act`. It
finds act through `org/repos.json`. The test script also uses `jq`.

## Commands

Run these from `/tmp/actions-ab/r6/gha-1/org`. Every command prints JSON on stdout. Say who you
are with `--as <login>` or with `ACME_ACTOR=<login>`. The people are listed in `org/people.json`.

```bash
cd /tmp/actions-ab/r6/gha-1/org

# List the self-service actions on every repository's main, with their inputs and reviewers
bin/acme-actions list
bin/acme-actions list --repo beta --action mark-deprecated

# Preview: validate the inputs and show the diff and the exact file. No record, no change.
bin/acme-actions preview alpha mark-deprecated --as ann \
  -f successor=acme/dependabot-proxy-v2 \
  -f "reason=Replaced by the v2 proxy, which supports OIDC registries."

# Request as ann: validates, previews, records the run and stops at the approval gate.
# Prints the record; .id is the run id and .status is waiting_for_approval.
bin/acme-actions request alpha mark-deprecated --as ann \
  -f successor=acme/dependabot-proxy-v2 \
  -f "reason=Replaced by the v2 proxy, which supports OIDC registries."
# The same request, with the inputs as one JSON object:
bin/acme-actions request alpha mark-deprecated --as ann \
  --inputs-json '{"successor":"acme/dependabot-proxy-v2","reason":"Replaced by the v2 proxy, which supports OIDC registries."}'

# Approve as bob: runs the gated jobs, then observes main. Exit 0 means succeeded.
bin/acme-actions approve run-0004 --as bob --comment "ok"

# Status, the full record, all runs
bin/acme-actions status run-0004
bin/acme-actions show run-0004
bin/acme-actions runs --repo alpha --status succeeded

# Also available: reject (by a reviewer) and cancel (by the requester or a reviewer)
bin/acme-actions reject run-0004 --as bob --comment "not yet"
bin/acme-actions cancel run-0004 --as ann
```

These are the exit codes:

| Exit code | Meaning |
|---|---|
| 0 | ok, or the run succeeded |
| 1 | the run failed |
| 2 | invalid inputs or usage |
| 3 | not permitted |
| 4 | not found |
| 5 | wrong state, for example approving a run that has already finished |

A refusal prints `{"error": {"kind", "message", ...}}`.

These are the run statuses:

| Status | Meaning |
|---|---|
| `validating` | the request is being validated and previewed |
| `invalid` | the inputs were refused; this is final |
| `waiting_for_approval` | waiting for a reviewer |
| `running` | approved; the gated jobs are running |
| `succeeded` | the result was observed on `main` |
| `failed` | the run did not complete, or `main` does not have the expected content |
| `rejected` | a reviewer rejected the run |
| `cancelled` | the requester or a reviewer cancelled the run |

The whole regression suite is `org/tests/e2e.sh`. It copies the three origins into a scratch
sandbox under `org/.state/e2e`, so the real repositories are not touched. It covers 36 checks:
invalid inputs at both length bounds, self-approval, carol, an unknown person, a successful run,
a rejected push, reject, cancel and the record's contents. It ends with `all checks passed`
(exit 0).

## How each requirement is met

| # | How | Files |
|---|---|---|
| R1 Owned by the repository | The action is a `workflow_dispatch` workflow on each origin's `main`. Its approval settings are code in the same repository. CODEOWNERS assigns the workflow, the settings and CODEOWNERS itself to the maintainers, @ann and @bob. The CLI reads the definition from `main` at request time and pins that commit for the run (`definition.commit` in the record), so a commit to `main` changes the action. | `<repo>/.github/workflows/mark-deprecated.yml`, `<repo>/.github/settings/mark-deprecated-environment.json`, `alpha/CODEOWNERS`, `beta/.github/CODEOWNERS`, `gamma/.github/CODEOWNERS` |
| R2 Validated | The first job, `validate`, refuses bad inputs before any job that writes runs. The successor must fully match `^[a-z0-9-]+/[a-z0-9-]+$`, so a trailing newline fails, and must not be the repository itself. The reason must be 20 to 200 characters and one line. Before that, the CLI checks inputs the way GitHub's dispatch API does: unknown inputs, missing required inputs and wrong types are refused. An invalid request is recorded as `invalid` and can't be approved. | `mark-deprecated.yml` job `validate`; `org/bin/acme-actions` `dispatch_inputs` |
| R3 Previewed | The `preview` job clones `main`, renders the file and prints the diff, both in the step summary and in a machine-readable `ACME_RESULT preview` line. `acme-actions preview` runs only the jobs before the gate. `request` stores the preview in the record (`.preview.diff`, `.preview.content`), where the requester and the approver can see it before approving. | `mark-deprecated.yml` job `preview`; `org/bin/acme-actions` `cmd_preview`, `cmd_request` |
| R4 Approved | The `apply` job runs in the `mark-deprecated` environment. The environment's reviewers are ann and bob, and `prevent_self_review` is true. Locally, the CLI is the gate: it runs the jobs up to the gate, stops, and runs the gated jobs only after `approve`. It reads the reviewers from the repository's settings file on `main` at approval time. Self-approval, carol (not a reviewer) and unknown names are refused with exit 3, and every refused attempt is written to the record as an `approve_denied` event. A per-repository lock and the state check make a second approval of the same run fail with exit 5. | `.github/settings/mark-deprecated-environment.json`; `org/bin/acme-actions` `check_reviewer`, `decide` |
| R5 Done means observed | Two checks, both required. First, the workflow's `verify` job clones `main` again and compares `DEPRECATED.md` byte for byte. Second, the CLI reads `DEPRECATED.md` from the origin's `main` itself (`git cat-file`) and compares it with the content that was previewed and approved. The run is `succeeded` only if act exited 0, every job succeeded, `verify` observed the file, and the CLI's own read matches. A rejected push fails `apply` with the server's message, so `verify` never runs and the run is `failed`. `outcome.observed` records which `main` commit was read. | `mark-deprecated.yml` jobs `apply` and `verify`; `org/bin/acme-actions` `observe`, `execute` |
| R6 One place for the org | `org/repos.json` registers the three origins. `acme-actions list` discovers every gated `workflow_dispatch` workflow on each origin's `main`, and `request`, `approve` and the reads all work from `org/` by repository name. | `org/repos.json`, `org/people.json`, `org/bin/acme-actions` |
| R7 Audited | `org/runs/<id>.json` holds the requester and `requested_at`, the inputs, the preview, the approver and `approved_at` (or the rejection), the outcome and `finished_at`. It also has a timestamped `events` list: requested, previewed, waiting, each denied attempt, approved, started, observed, and the final status. Every change to a record is committed to `org` and pushed to `origins/org.git`, with the person who acted as the commit author, so the record has its own history. The commit the action pushes to the repository carries `Requested-by:`, `Approved-by:` and `Request-Id:` trailers. Raw act logs are in `org/.state/logs/<id>/`, which is local and not committed. | `org/runs/`, `git -C org log`, `org/bin/acme-actions` `save`, `audit_commit` |
| R8 Usable by agents | Every step is a non-interactive command with JSON output and distinct exit codes. Inputs can be passed as `-f key=value` or as `--inputs-json`. Identity is `--as` or `ACME_ACTOR`. | `org/bin/acme-actions` |

## How the local run maps to GitHub

On GitHub, the CLI isn't needed for enforcement. Each part maps like this:

| Locally | On GitHub |
|---|---|
| `acme-actions list` | `gh workflow list -R acme/<repo>` and `gh workflow view mark-deprecated -R acme/<repo>` |
| `preview` | `gh workflow run mark-deprecated -R acme/<repo> -f successor=… -f reason=… -f dry_run=true`. `apply` and `verify` are skipped, and the diff is in the run summary. |
| `request` | `gh workflow run mark-deprecated -R acme/<repo> -f successor=… -f reason=…`. The run executes `validate` and `preview`, then waits at the environment. |
| The approval gate | The `mark-deprecated` environment with required reviewers ann and bob and `prevent_self_review: true`. GitHub won't let the person who triggered the run approve it, and won't let anyone who isn't a reviewer approve it. Approval is done in the UI or with `POST /repos/acme/<repo>/actions/runs/<id>/pending_deployments`. |
| `status` and `show` | `gh run view <id> -R acme/<repo> --json status,conclusion,jobs`, and `GET /repos/acme/<repo>/actions/runs/<id>/approvals` for who approved and when. Run history is GitHub's run list. |
| Branch protection | `.github/settings/main-ruleset.json`: pull requests with code-owner review, no force pushes and no deletion on the default branch. Only the org's deprecation GitHub App (`vars.MARK_DEPRECATED_APP_ID`) bypasses it. The `apply` job mints that App's token with `openssl` and `curl`, so the workflow has no `uses:` steps. Everyone else, including carol, can only change `main` through a reviewed pull request, and the workflow file itself needs a code-owner review. |
| Applying the settings | `.github/settings/apply.sh owner/name` sets the environment, its branch policy (`main` only) and the ruleset through the REST API. `.github/workflows/apply-settings.yml` runs it on every push to `main` that touches `.github/settings/`, using a `REPO_SETTINGS_TOKEN` secret with administration rights. Without `MARK_DEPRECATED_APP_ID`, the script refuses to apply the ruleset, because the ruleset would then block the action's own push. |

## What I could not run here

- **Environments, required reviewers, `prevent_self_review`, rulesets, CODEOWNERS enforcement and
  run history.** There is no GitHub here, and act ignores `environment:`. The CLI enforces the same
  rules from the same settings file. `apply.sh --dry-run acme/alpha` (with
  `MARK_DEPRECATED_APP_ID` set) prints the API requests it would send. I checked those
  requests against the REST API shape but never sent them.
- **The GitHub App token step and the `gh api …/approvals` lookup.** Both are skipped under act,
  because `vars.MARK_DEPRECATED_APP_ID` is empty and `ACT` is set. Locally, the push uses act's
  stand-in token, and the CLI passes the approver it enforced as `ACME_LOCAL_APPROVED_BY`. It is
  not verified that GitHub accepts an App as a ruleset bypass actor in exactly this JSON form.
- **`apply-settings.yml`.** It needs GitHub and an admin token.
- **The local gate is not a security boundary.** Anyone with a shell here can push to
  `origins/*.git` directly, or edit `org/runs/*.json`. Edits to the records still show in the
  org repository's git history. On GitHub, the environment and the ruleset are enforced by the
  server.

Under act, the workflow's `https://github.com/acme/<repo>.git` URLs are rewritten to the local
bare origins with a git `insteadOf` file, `org/.state/gitconfig`, passed in as
`GIT_CONFIG_GLOBAL`. The workflow file is the same one GitHub would run.

## Testing it yourself

All three repositories are already deprecated. A request with the same inputs as before finds
nothing to change. It succeeds without a push, because `main` already has the expected content.
To see a change, use a different reason.

**Invalid inputs.** Each of these exits 2. The record is `invalid`, and `main` does not move.

```bash
bin/acme-actions request alpha mark-deprecated --as ann -f successor=Acme/X -f "reason=too short"
bin/acme-actions request alpha mark-deprecated --as ann -f successor=acme/alpha -f "reason=Twenty-plus characters here."
bin/acme-actions preview alpha mark-deprecated --as ann -f successor=acme/x -f "reason=$(printf 'y%.0s' $(seq 201))"
```

**Self-approval and carol.** Both approvals exit 3, and both appear as `approve_denied` events
in the record.

```bash
id=$(bin/acme-actions request beta mark-deprecated --as ann -f successor=acme/next \
  -f "reason=Testing the approval rules here." | jq -r .id)
bin/acme-actions approve "$id" --as ann
bin/acme-actions approve "$id" --as carol
bin/acme-actions show "$id" | jq '.events'
```

**A rejected push.** The approval exits 1. The status is `failed`, `outcome.reason` includes
"push to main is closed", `outcome.observed.matches` is false, and `main` is unchanged.

```bash
printf '#!/bin/sh\necho "push to main is closed" >&2\nexit 1\n' > ../origins/gamma.git/hooks/pre-receive
chmod +x ../origins/gamma.git/hooks/pre-receive
id=$(bin/acme-actions request gamma mark-deprecated --as ann -f successor=acme/next \
  -f "reason=This push should be rejected by the hook." | jq -r .id)
bin/acme-actions approve "$id" --as bob
bin/acme-actions status "$id"
rm ../origins/gamma.git/hooks/pre-receive
```

Note that the run records are pushed to `origins/org.git`, so a hook on that origin affects only
the audit push. The CLI reports that as `"audit": {"pushed": false}`.

## Notes

- **Exact content.** The file is `# Deprecated: use <successor>`, a blank line, `<reason>`, and
  a final newline.
- **One-line reasons.** Reasons with control characters, including newlines, are refused. This
  keeps the file to exactly that shape.
- **Reviewer changes.** A run uses the workflow from the `main` commit it was requested at. The
  reviewers are read from `main` at approval time, as GitHub reads environment settings live. To
  change who can approve, commit to `.github/settings/mark-deprecated-environment.json`.
- **Interrupted runs.** If `approve` is killed while a run is `running`, the record stays
  `running`. Check `main` and the log in `org/.state/logs/<id>/execute.jsonl`.
