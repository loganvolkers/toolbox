# Handoff: `mark-deprecated` self-service action

`mark-deprecated` is defined in each of `alpha`, `beta` and `gamma`, and has run on all three. ann requested each run, bob approved it, and each was reported `succeeded` only after `DEPRECATED.md` was read back from the origin's `main`:

| Run | Repository | Observed commit on origin `main` |
|---|---|---|
| `run-0001` | alpha | `7b4112e` |
| `run-0002` | beta | `249b641` |
| `run-0003` | gamma | `3d2b615` |

The system is one stdlib-only Python 3 CLI, `org/bin/acme`. **Nothing has to be running**: there is no server or daemon. Each command reads the origins directly, writes run records under `org/runs/`, and exits.

## Files

| Path | What it is |
|---|---|
| `org/bin/acme` | The CLI: discovery, validation, preview, request, approve/reject, execution, observation, records |
| `org/acme.json` | The org registry: the three repositories (origin path, branch) and the known people (ann, bob, carol) |
| `<repo>/.acme/actions/mark-deprecated.json` on `origins/<repo>.git` `main` | Each repository's own definition: maintainers, approval rule, input rules, file template |
| `org/runs/run-NNNN.json` | One record per run (state, inputs, preview, approvals, outcome, timestamped events) |
| `org/runs/audit.jsonl` | Append-only log of every event, including refused requests and denied approvals |

`org/runs/` is local state and is git-ignored. The tool and the registry are committed to `origins/org.git` `main`.

## How each requirement is met

| # | How |
|---|---|
| R1 Owned by the repository | Each repository commits its definition to its own origin's `main`, at `.acme/actions/mark-deprecated.json` (commits `80faaba` in alpha, `5f7db2f` in beta and `a6955e4` in gamma). `acme` reads definitions only from the origin's `main` (`git cat-file` on `origins/<repo>.git`), never from a checkout or the org. The definition carries the `maintainers` list, the input rules, the target path and the content template, so a team changes the action by committing to its repository. A request records the definition's blob id. If the definition changes before approval, the approval is refused and the run is marked `stale`. |
| R2 Validated | `validate_inputs` in `org/bin/acme` applies the definition's rules. `successor` must match `^[a-z0-9-]+/[a-z0-9-]+$` and must not be the repository itself (`acme/<repo>`). `reason` must be 20 to 200 characters on a single line, with no control characters. Unknown or missing inputs are refused. `preview` and `request` refuse invalid inputs with exit code 2 and a list of errors. No run is created and the origin is untouched; a refused request is still logged to `org/runs/audit.jsonl` as `request_refused`. |
| R3 Previewed | `acme preview` shows the operation (`create`, `update` or `unchanged`), the before and after content and a unified diff, computed against the origin's current `main` commit. `request` stores that preview in the run, and `record` shows it. At approval the diff is recomputed; if `main` changed it so the diff differs, the run is marked `stale` rather than applied, so what is approved is what was previewed. |
| R4 Approved | A run starts in `pending_approval`. Nothing is pushed until `acme approve` passes these checks (`cmd_approve`): the approver must be a known person, must not be the requester, and must be in `maintainers` both in the definition pinned at request time and on current `main`. carol is in neither list. A denied approval exits with code 3, the run stays pending, and an `approval_denied` event is recorded. `approval.required` (1 here) sets how many distinct approvers are needed. `acme reject` lets a maintainer reject a run, or the requester cancel it. |
| R5 Done means observed | `execute` makes the commit in a temporary clone and pushes it to `origins/<repo>.git` `main`. It then reads `main:DEPRECATED.md` from the origin and compares it byte for byte with the expected content. The run becomes `succeeded` only if they match. A rejected push becomes `failed` with reason `push_rejected` (the hook's stderr is kept). A push that is accepted but doesn't land becomes `failed` with reason `not_observed`, and any unexpected error becomes `failed` with reason `error`. In each failure case `approve` exits with code 1. |
| R6 One place for the org | Every command runs from `org/`. `org/acme.json` lists all three repositories. `acme list` discovers every `.acme/actions/*.json` on each origin's `main`, and `request` takes the repository as an argument, so nobody has to enter a repository. |
| R7 Audited | `org/runs/run-NNNN.json` holds `requested_by` and `requested_at`, `approvals[]` (`by`, `at`, `comment`), `inputs`, `preview`, the pinned `definition` (commit, blob id, maintainers), `outcome` (result, reason or message, observed commit), `finished_at`, and `events[]`. Each event has an `at` timestamp: requested, approval_denied, approved, started, push_started, pushed or push_rejected, observed, then succeeded, failed, stale, rejected or cancelled. Each commit names the requester as author and `acme-actions` as committer, with the trailers `Acme-Run:`, `Requested-by:` and `Approved-by:`. `org/runs/audit.jsonl` is the append-only log across all runs. |
| R8 Usable by agents | Every subcommand takes flags only (no prompts), prints JSON on stdout (`{"ok": true, ...}` or `{"ok": false, "error": {"code", "message", ...}}`) and uses stable exit codes. Identity is `--as NAME` or `$ACME_USER`. Inputs can be `--successor/--reason`, `-i KEY=VALUE` or `--inputs '<json>'`. A file lock (`org/runs/.lock`) serializes state changes, so concurrent agents can't double-apply a run. |

Exit codes: 0 ok, 1 run failed, 2 invalid input, 3 forbidden, 4 not found, 5 conflict (wrong state, or stale).

Run states: `pending_approval` → `running` → `succeeded` or `failed`. A pending run can instead end as `stale`, `rejected` or `cancelled`.

## Commands

Run these from the org directory. Nothing needs starting.

```bash
cd /tmp/actions-ab/r6/vanilla-1/org
```

List the actions in all three repositories, or one:

```bash
bin/acme list
bin/acme list --repo beta
bin/acme describe alpha mark-deprecated
```

Preview (no identity needed, changes nothing):

```bash
bin/acme preview alpha mark-deprecated --successor acme/dependabot-proxy-v2 \
  --reason 'Replaced by the v2 proxy, which supports OIDC registries.'
```

Request as ann. The response's `run.id` is the run id, e.g. `run-0004`:

```bash
bin/acme request alpha mark-deprecated --as ann --successor acme/dependabot-proxy-v2 \
  --reason 'Replaced by the v2 proxy, which supports OIDC registries.'
# for scripts:
RUN=$(bin/acme request alpha mark-deprecated --as ann --successor acme/new-thing \
  --reason 'A reason that is at least twenty characters.' | python3 -c 'import json,sys;print(json.load(sys.stdin)["run"]["id"])')
```

Approve as bob. This applies the run and reports the observed outcome:

```bash
bin/acme approve "$RUN" --as bob --comment 'LGTM'
```

Reject (a maintainer) or cancel (the requester):

```bash
bin/acme reject "$RUN" --as bob --comment 'not yet'
```

Check status, read the full record, or list runs:

```bash
bin/acme status "$RUN"
bin/acme record "$RUN"
bin/acme runs --repo alpha --status succeeded
cat runs/audit.jsonl
```

## Negative tests to try

All of these were run against a scratch copy of `org/` and `origins/`, which was then deleted.

```bash
# invalid inputs: exit 2, nothing created
bin/acme request alpha mark-deprecated --as ann --successor 'Acme/X' --reason 'too short'
bin/acme request alpha mark-deprecated --as ann --successor acme/alpha --reason 'A reason that is at least twenty characters.'
bin/acme request alpha mark-deprecated --as ann --successor acme/x --reason "$(python3 -c 'print("x"*201)')"

# self-approval and carol: exit 3, run stays pending_approval, attempt recorded
RUN=$(bin/acme request beta mark-deprecated --as ann --successor acme/x --reason 'A reason that is at least twenty characters.' | python3 -c 'import json,sys;print(json.load(sys.stdin)["run"]["id"])')
bin/acme approve "$RUN" --as ann
bin/acme approve "$RUN" --as carol

# rejected push: exit 1, status failed / push_rejected, main unchanged
printf '#!/bin/sh\necho "main is locked" >&2\nexit 1\n' > ../origins/beta.git/hooks/pre-receive
chmod +x ../origins/beta.git/hooks/pre-receive
bin/acme approve "$RUN" --as bob
bin/acme status "$RUN"
rm ../origins/beta.git/hooks/pre-receive
```

The last block runs against the real beta origin. Because the push is rejected, beta's `main` is not changed, but the failed run stays in `org/runs/`. A failed run is final, so to try again, request a new run.

## Notes

- `DEPRECATED.md` is written as `# Deprecated: use <successor>`, a blank line, `<reason>`, then a final newline.
- Identity is asserted with `--as`, as the sandbox requires; there are no passwords. Only the names in `org/acme.json` are accepted.
- If a run is requested when `DEPRECATED.md` already has the exact content, the preview says `unchanged`. Approving it pushes nothing; the run succeeds once the content is observed on `main`.
