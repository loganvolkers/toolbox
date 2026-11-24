# Handoff: Acme self-service actions (`mark-deprecated`)

`mark-deprecated` is defined in each of `alpha`, `beta` and `gamma`, committed to their origins'
`main`, and has been run on all three. In each case ann requested and bob approved, and each run is
reported `succeeded` after `DEPRECATED.md` was read back from the origin's `main`:

| Repository | Run id | Commit on origin `main` |
|---|---|---|
| alpha | `run-20261124-104135-23340e` | `d20708e` |
| beta | `run-20261124-104135-3cec38` | `08eef73` |
| gamma | `run-20261124-104135-fe4c0e` | `3b27c06` |

The system is a single Python 3 command-line tool, `org/bin/acme`, which uses only the standard
library and Git. **Nothing has to be running: there is no server and no daemon.** Every command
prints one JSON document on stdout, shaped `{"ok": true, "result": …}` or
`{"ok": false, "error": {"code", "message", "details"}}`. The exit codes are:

| Exit code | Meaning |
|---|---|
| 0 | OK |
| 2 | Refused: invalid inputs, an approval that isn't allowed, or a missing or unknown identity |
| 3 | The run failed: the change was not observed on `main` |
| 4 | Not found: unknown repository, action or run |
| 1 | Internal error |

## Files

| Path | What it is |
|---|---|
| `<repo>/.acme/actions/mark-deprecated.json` (in `alpha`, `beta` and `gamma`, on each origin's `main`) | The repository's own definition of the action: approvers, input schema, the file to write, and the commit message |
| `<repo>/.acme/README.md` | How the repository's team changes its actions |
| `org/bin/acme` | The engine and CLI: discovery, validation, preview, request, approve/reject, execute, observe, audit |
| `org/org.json` | The org registry: people (ann, bob, carol) and repositories (slug and bare origin path) |
| `org/state/runs/<run-id>.json` | One record per run, with its full audit trail. Not committed (`org/.gitignore`) |
| `org/state/audit.jsonl` | An append-only, org-wide audit log of every event, including refused requests and denied approvals |

## How each requirement is met

| # | How |
|---|---|
| R1 Owned by the repository | The definition is `.acme/actions/mark-deprecated.json` in each repository, committed to its origin's `main` (alpha `b8aedf4`, beta `d0053a8`, gamma `9b28c6c`). `acme` never reads the checkouts or a central copy. It reads the definition from the origin's `main` (`git --git-dir origins/<repo>.git show main:.acme/actions/…`), so a commit by the repository's team changes the action, including who may approve it (`approval.approvers`). Each run pins the definition commit and blob it was requested under (`definition` in the record), and executes that version. |
| R2 Validated | `validate_inputs` in `org/bin/acme` applies the definition's input schema: `successor` must match `^[a-z0-9-]+/[a-z0-9-]+$` and must not be the repository itself; `reason` must be 20–200 characters on one line. Unknown and missing inputs are refused. Refusal (exit 2, `invalid_inputs`, with one error per input) happens before a run record is created or anything is pushed. A refused `request` is logged in `state/audit.jsonl` as `request_refused`. |
| R3 Previewed | `acme preview` validates the inputs and returns the planned change against the current `main` commit: the operation (`add`, `modify` or `unchanged`), the before and after content, and a unified diff. It creates nothing. `acme request` returns the same preview and stores it in the run record (`preview`). |
| R4 Approved | `acme request` only creates a run in `awaiting_approval`. Only `acme approve` executes it, and approve refuses (exit 2, `approval_denied`, logged as an `approval_denied` event) if the approver is the requester, if the approver is not in `approval.approvers` of both the pinned and the current definition (carol is in neither), or if the person is unknown. The engine rejects any definition with `approval.required` < 1. A run that isn't awaiting approval cannot be approved again. |
| R5 Done means observed | `execute` in `org/bin/acme` clones the origin into a temporary directory, commits the file, and pushes to `main`. Whatever the push reported, it then reads `main:DEPRECATED.md` from the origin and compares it byte for byte with the rendered content. The run is `succeeded` only if they match; otherwise it is `failed` (exit 3), and the record keeps the push output and the observation. A rejected push (pre-receive hook) and an accepted push that `main` doesn't keep (a post-receive hook that resets the ref) were both tested and both reported `failed`. |
| R6 One place for the org | From `org/`, `bin/acme list` discovers the actions on every repository in `org.json` by reading `.acme/actions/` on each origin's `main`. `request`, `approve`, `status`, `show` and `runs` take a repository name or run id, so nobody needs to enter a repository's checkout. |
| R7 Audited | `org/state/runs/<id>.json` holds `requestedBy`/`requestedAt`, `approvals[]` and `approvedBy`/`approvedAt`, `inputs`, `status`, `outcome` (with push and observation), and `finishedAt`. Its `events[]` gives the timestamp and actor of every step: requested, previewed, approval_denied, approved, started, planned, pushed or push_failed, observed, and succeeded or failed. Every event is also appended to `org/state/audit.jsonl`. The commit on `main` carries `Acme-Run`, `Requested-by` and `Approved-by` trailers, with the requester as author and `acme-actions` as committer. |
| R8 Usable by agents | Every step is a non-interactive subcommand with JSON output and meaningful exit codes. Identity comes from `--as <person>` or `$ACME_USER`. Inputs come from `-i key=value` or `--inputs-json '{…}'` (or `-` for stdin). |

## Commands

Run these from the org checkout. Nothing needs to be started first.

```bash
cd /tmp/actions-ab/r1/vanilla-2/org

# List the actions on every repository (or one: --repo alpha)
bin/acme list
bin/acme describe alpha mark-deprecated

# Preview (validates; changes nothing)
bin/acme preview alpha mark-deprecated \
  -i successor=acme/dependabot-proxy-v2 \
  -i reason="Replaced by the v2 proxy, which supports OIDC registries."

# Request as ann (validated and previewed; prints the run id at .result.run.id)
bin/acme request alpha mark-deprecated --as ann \
  -i successor=acme/dependabot-proxy-v2 \
  -i reason="Replaced by the v2 proxy, which supports OIDC registries."
RUN=$(bin/acme request beta mark-deprecated --as ann \
  --inputs-json '{"successor":"acme/valtio-store","reason":"State management moves to the shared store package."}' \
  | python3 -c 'import json,sys; print(json.load(sys.stdin)["result"]["run"]["id"])')

# Approve as bob (applies the change, observes main, then reports succeeded or failed)
bin/acme approve "$RUN" --as bob

# Status, the full record, all runs, the org audit log
bin/acme status "$RUN"
bin/acme show "$RUN"
bin/acme runs                     # filters: --repo alpha --status succeeded
cat state/audit.jsonl

# Reject (a maintainer) or cancel (the requester) a run awaiting approval
bin/acme reject "$RUN" --as bob --reason "not yet"
```

## Testing the refusals

```bash
cd /tmp/actions-ab/r1/vanilla-2/org

# Invalid inputs: exit 2, invalid_inputs, and no run is created
bin/acme request alpha mark-deprecated --as ann -i successor=Not/Valid -i reason="too short"
bin/acme request alpha mark-deprecated --as ann -i successor=acme/x -i reason="$(printf 'x%.0s' {1..201})"
bin/acme request alpha mark-deprecated --as ann -i successor=acme/alpha -i reason="Pointing at itself is refused."

# Self-approval and carol: exit 2, approval_denied; the run stays awaiting_approval
RUN=$(bin/acme request alpha mark-deprecated --as ann -i successor=acme/new-thing \
  -i reason="A different reason, so the file changes." \
  | python3 -c 'import json,sys; print(json.load(sys.stdin)["result"]["run"]["id"])')
bin/acme approve "$RUN" --as ann
bin/acme approve "$RUN" --as carol
bin/acme status "$RUN"

# Rejected push: exit 3, the run is failed, and main is unchanged
printf '#!/bin/sh\necho "main is protected" >&2\nexit 1\n' > ../origins/alpha.git/hooks/pre-receive
chmod +x ../origins/alpha.git/hooks/pre-receive
bin/acme approve "$RUN" --as bob
bin/acme show "$RUN"              # .result.outcome.push.output shows the rejection
rm ../origins/alpha.git/hooks/pre-receive
```

Note: `alpha`, `beta` and `gamma` already have `DEPRECATED.md`. Re-running with the same inputs is a
no-op that still reports `succeeded`, because the intended content is observed on `main`. To see a
push rejected, use inputs that change the file, as above. A failed run is final; request a new run
to try again.
