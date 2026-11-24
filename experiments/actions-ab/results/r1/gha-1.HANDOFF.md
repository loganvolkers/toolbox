# Handoff: `mark-deprecated` on GitHub Actions

`mark-deprecated` is a GitHub Actions workflow that each repository owns. An org-level gateway,
`org/bin/acme-actions`, lists it across the three repositories, dispatches it with act, enforces
the approval rule and keeps an audit record per run. The three requested runs, all requested by
ann and approved by bob, are reported `succeeded`:

| Repository | Run id | Commit on origin `main` |
|---|---|---|
| alpha | `mark-deprecated-alpha-20261124T101929Z-f1d5` | `7695a34` |
| beta | `mark-deprecated-beta-20261124T101932Z-5bfa` | `4155c61` |
| gamma | `mark-deprecated-gamma-20261124T101934Z-43c5` | `9d436f0` |

Paths below are relative to `/tmp/actions-ab/r1/gha-1`.

## What lives where

| Path | Role |
|---|---|
| `<repo>/.github/workflows/mark-deprecated.yml` | The action: `workflow_dispatch` with inputs `successor`, `reason`, `mode` (`preview`/`apply`), `request_id`. Jobs: `preview` (validate + diff), `apply` (environment `mark-deprecated`: authorize, commit, push), `verify` (read `DEPRECATED.md` back from `main`) |
| `<repo>/.github/mark-deprecated/config.json` | The repository's rules: input patterns and lengths, `maintainers` (approvers), environment, branch, path |
| `<repo>/.github/mark-deprecated/mark_deprecated.py` | The step logic: `validate`, `preview`, `authorize`, `apply`, `verify`; each prints one JSON result line |
| `<repo>/.github/mark-deprecated/apply-github-settings.sh` | GitHub settings as code: the environment with required reviewers and `prevent_self_review`, the branch policy, the ruleset on `main`, the deploy key |
| `<repo>/.github/mark-deprecated/README.md` | What the files are, for the owning team |
| `alpha/CODEOWNERS`, `beta/.github/CODEOWNERS`, `gamma/.github/CODEOWNERS` | The action's files are owned by `@ann @bob` |
| `org/catalog.json` | The org: repositories and their origins, people, the act binary |
| `org/bin/acme-actions` | The gateway CLI (Python 3, standard library only) |
| `org/state/runs/<id>.json` | One audit record per run (git-ignored runtime state) |
| `org/state/runs/<id>/{preview,apply}.log` | The act logs of each workflow run |
| `org/state/audit.jsonl` | Append-only log of every event, including refused requests and refused approvals |

## Start

Nothing runs in the background: there is no server. Each command is a one-shot process. You need
`python3`, `git` and the act binary at `bin/act`; `jq` is only used by `apply-github-settings.sh`
and in the examples below. Set `ACME_USER=<name>` instead of passing `--as <name>` if you prefer.

```bash
cd /tmp/actions-ab/r1/gha-1/org
A=bin/acme-actions          # or the absolute path, from any directory
```

## Commands

Every command prints JSON on stdout, including errors (`{"error": ..., "message": ...}`). Exit codes:
`0` ok, `2` invalid input, `3` not allowed, `4` not found, `5` wrong state (e.g. already approved),
`6` the run failed.

```bash
# List mark-deprecated (and any other action) on all three repositories, read from each origin's main
bin/acme-actions list
bin/acme-actions list --repo beta --action mark-deprecated

# Preview: validates and shows the diff; records nothing, changes nothing
bin/acme-actions preview --as ann --repo alpha \
  --successor acme/dependabot-proxy-v2 \
  --reason "Replaced by the v2 proxy, which supports OIDC registries."

# Request as ann: validates and previews again, then records a run awaiting approval.
# The output is the run record; its .preview.diff is what the run would change.
ID=$(bin/acme-actions request --as ann --repo alpha \
  --successor acme/dependabot-proxy-v2 \
  --reason "Replaced by the v2 proxy, which supports OIDC registries." | jq -r .id)

# Approve as bob: runs the workflow in apply mode, then observes origin main. Blocks until done.
bin/acme-actions approve --as bob "$ID" --comment "ok"

# Status and full record
bin/acme-actions status "$ID"
bin/acme-actions record "$ID"

# Other
bin/acme-actions reject --as bob "$ID" --comment "not now"   # requester (cancel) or a maintainer
bin/acme-actions runs [--repo alpha] [--status succeeded]
```

Inputs can also be given generically: `--input successor=acme/x --input reason="..."` or
`--inputs-json '{"successor": "acme/x", "reason": "..."}'`. Unknown or missing inputs are refused.

## How each requirement is met

| # | How | Files |
|---|---|---|
| R1 Owned by the repository | The workflow, its step script and its rules (inputs, approvers) are committed to each origin's `main`. The gateway reads them from origin `main` (`git ls-tree`/`show` on the bare repo) and runs the workflow from a fresh clone of origin `main`, so a change the team merges takes effect on the next run; the record keeps the definition commit at request and at apply. CODEOWNERS makes `@ann @bob` the owners, and the ruleset requires a code-owner review on `main` | `<repo>/.github/workflows/mark-deprecated.yml`, `<repo>/.github/mark-deprecated/config.json`, `<repo>/.github/mark-deprecated/mark_deprecated.py`, CODEOWNERS |
| R2 Validated | The workflow's first step, `mark_deprecated.py validate`, checks `successor` against `^[a-z0-9-]+/[a-z0-9-]+$` (full match, so a trailing newline fails), that it is not this repository itself, and `reason` is 20–200 characters on one line. `preview` and `request` run it before anything else and refuse with exit 2 and a list of errors; no run is created and the refusal is appended to `org/state/audit.jsonl`. `apply` validates again before it writes | `mark_deprecated.py` (`validation_errors`), `config.json` (`inputs`) |
| R3 Previewed | `preview` mode clones origin `main`, writes the proposed file and returns the unified diff, the current and proposed content, the base commit and `change` (`create`/`update`/`none`). `acme-actions preview` shows it without recording anything; `request` stores it in the record before any approval. On GitHub the same diff goes to the job summary | `mark_deprecated.py` (`cmd_preview`), workflow job `preview` |
| R4 Approved | `request` only previews. The `apply` job runs only from `approve`, which refuses (exit 3, recorded as `approval_refused`) the requester and anyone not in the repository's `maintainers` (so carol and unknown names). The workflow re-checks the same rule in its `authorize` step before it writes. On GitHub, the `apply` job's environment `mark-deprecated` has ann and bob as required reviewers with `prevent_self_review: true`, and the push credential is an environment secret that the job only receives after approval; `authorize` then reads the approver from the run's approvals API | `org/bin/acme-actions` (`cmd_approve`), `mark_deprecated.py` (`cmd_authorize`), `apply-github-settings.sh` |
| R5 Done means observed | The `verify` job clones origin `main` again and compares `DEPRECATED.md` byte for byte. The gateway then reads `main:DEPRECATED.md` from the origin itself. A run is `succeeded` only if the workflow exited 0, `verify` passed and the gateway's own read matches; otherwise it is `failed` with the failing step and git's output (e.g. `[remote rejected] (pre-receive hook declined)`) | `mark_deprecated.py` (`cmd_apply`, `cmd_verify`), `org/bin/acme-actions` (`observe`) |
| R6 One place for the org | `org/catalog.json` names the three repositories; `acme-actions list/preview/request/approve/status/record` work on any of them from `org/` | `org/catalog.json`, `org/bin/acme-actions` |
| R7 Audited | Each record has `requested_by`/`requested_at`, `approved_by`/`approved_at`, `inputs`, `status`/`outcome`, `started_at`/`finished_at`, `error`, the preview, the observed commit and content, the definition commits, the act logs, and a timestamped `events` list (requested, validated, previewed, approval_refused, approved, started, observed, succeeded/failed). Every event is also appended to `audit.jsonl`. The commit on `main` carries `Requested-by`, `Approved-by` and `Request-id` trailers | `org/state/runs/<id>.json`, `org/state/audit.jsonl` |
| R8 Usable by agents | Every step is one non-interactive command with JSON output and distinct exit codes; usage errors are JSON too | `org/bin/acme-actions` |

## Testing it

The three repositories already contain `DEPRECATED.md` from the runs above. Requesting the same
inputs again previews `change: "none"` and succeeds without pushing (the file is already right).
Use a different `reason` to exercise a real change.

```bash
cd /tmp/actions-ab/r1/gha-1/org

# Invalid inputs: exit 2, nothing changes, nothing is recorded as a run
bin/acme-actions request --as ann --repo alpha --successor "Acme/X" --reason "too short"
bin/acme-actions request --as ann --repo alpha --successor acme/alpha --reason "Twenty characters or more here."
bin/acme-actions request --as ann --repo alpha --successor acme/x --reason "$(printf 'x%.0s' {1..201})"

# Self-approval and carol: exit 3, recorded as approval_refused, the run stays awaiting_approval
ID=$(bin/acme-actions request --as ann --repo beta --successor acme/valtio-store \
  --reason "A second reason to test the approval rules." | jq -r .id)
bin/acme-actions approve --as ann "$ID"
bin/acme-actions approve --as carol "$ID"
bin/acme-actions status "$ID"

# Rejected push: make the origin refuse pushes, approve, expect exit 6 and status "failed"
printf '#!/bin/sh\necho "push rejected for the test" >&2\nexit 1\n' > ../origins/beta.git/hooks/pre-receive
chmod +x ../origins/beta.git/hooks/pre-receive
bin/acme-actions approve --as bob "$ID"; echo "exit $?"
bin/acme-actions record "$ID" | jq '{status, error, observed}'
rm ../origins/beta.git/hooks/pre-receive
```

To run the workflow without the gateway, as act would on the repository itself:

```bash
cd "$(mktemp -d)" && git clone -q /tmp/actions-ab/r1/gha-1/origins/alpha.git repo && cd repo
echo '{"inputs":{"successor":"acme/x","reason":"A reason long enough to pass.","mode":"preview"}}' > ../event.json
/tmp/actions-ab/r1/gha-1/bin/act workflow_dispatch -W .github/workflows/mark-deprecated.yml \
  -P ubuntu-latest=-self-hosted -e ../event.json --actor ann
```

In `apply` mode without the gateway, the `authorize` step refuses unless
`--env ACME_APPROVED_BY=<maintainer other than the actor>` is passed.

## How it maps to GitHub, and what could not run here

There is no GitHub, so act plays `workflow_dispatch` and the gateway plays the parts GitHub's
server provides. On GitHub:

| Concern | Here | On GitHub |
|---|---|---|
| Dispatch | gateway runs `act workflow_dispatch` from a clone of origin `main` | `gh workflow run mark-deprecated.yml -R acme/<repo> -f successor=... -f reason=... -f mode=apply` |
| Approval | gateway's `approve` checks maintainers and self-approval; `authorize` re-checks `ACME_APPROVED_BY` | the `apply` job waits on environment `mark-deprecated`: required reviewers ann and bob, `prevent_self_review: true` (`gh api -X POST repos/acme/<repo>/actions/runs/<run>/pending_deployments`); `authorize` reads `GET /repos/.../actions/runs/<run>/approvals` |
| Push credential | the local bare origin, no credential | deploy key stored only as the environment secret `MARK_DEPRECATED_DEPLOY_KEY`, released after approval; the ruleset on `main` lets deploy keys bypass the pull-request rule and requires code-owner review for everything else, including changes to the action |
| Deploy branch | the gateway always runs origin `main` | the environment's branch policy allows only `main` |
| Run history and audit | `org/state/runs/*.json`, `audit.jsonl` | the Actions run history, the environment's deployment history and approvals, the commit trailers |
| Org-level list | gateway reads `.github/*/config.json` from each origin | the same files via the contents API, or `gh workflow list -R acme/<repo>` |

Not run here: `apply-github-settings.sh` against the GitHub API (only checked with `DRY_RUN=1`,
which prints the request bodies), the environment's required reviewers and `prevent_self_review`,
the ruleset and deploy-key bypass (the `DeployKey` bypass actor type is from GitHub's ruleset
documentation and is unverified here), the approvals-API branch of `authorize`, the deploy-key
push, and the job summary. Under act, `actions/checkout` is skipped (act copies the local clone),
`ACT=true` selects the local code paths, and `github.repository` is derived from the local path,
so the repository's name comes from `config.json`.

## Limits

- Identity is self-declared (`--as`), as the sandbox allows. The gateway records it and enforces the
  approval rule on it; GitHub would authenticate it.
- Anyone with filesystem access can push to the bare origins or edit `org/state/` directly; on
  GitHub the ruleset and the run history close that.
- Runs execute one at a time (a lock in `org/state/.lock`). If the gateway is killed during
  `approve`, the record stays `running`; check origin `main` and the `apply.log`.
- `DEPRECATED.md` ends with a newline after the reason.
