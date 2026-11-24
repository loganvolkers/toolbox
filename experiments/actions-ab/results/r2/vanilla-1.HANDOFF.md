# Handoff: `mark-deprecated` self-service action

Status: `mark-deprecated` is defined in `alpha`, `beta` and `gamma`. ann requested one run on each
repository, bob approved each, and all three are reported `succeeded`: runs `alpha-0001`,
`beta-0001` and `gamma-0001`. Each origin's `main` has `DEPRECATED.md` with the exact content.

The system is one stdlib-only Python 3.12 CLI, `org/bin/acme`. **Nothing has to be running:** there
is no server and no daemon. Every command runs, prints one JSON document to stdout and exits.

## Layout

| Path | What it is |
|---|---|
| `<repo>/.acme/actions/mark-deprecated.json` | The repository's definition of the action: inputs and validation, approval policy, effect (on each origin's `main`) |
| `<repo>/.acme/maintainers.json` | Who may approve runs of that repository's actions (`ann`, `bob`) |
| `<repo>/.acme/README.md` | A short note for the repository's team on how to change the two files |
| `org/bin/acme` | The runner: list, describe, preview, request, approve, reject, cancel, status, show, runs, audit |
| `org/acme.json` | Org registry: known people (`ann`, `bob`, `carol`) and the three repositories' origins |
| `org/state/runs/<run-id>.json` | One record per run (not committed, see `org/.gitignore`) |
| `org/state/audit.jsonl` | An append-only log of every event across all runs, including refused requests and approvals |
| `org/tests/test_acme.py` | End-to-end tests against throwaway origins (they never touch the real ones) |

`<repo>` is `/tmp/actions-ab/r2/vanilla-1/{alpha,beta,gamma}`. The runner reads definitions from
`origins/<repo>.git` `main`, not from the checkouts.

## How each requirement is met

| # | How | Where |
|---|---|---|
| R1 Owned by the repository | Each repository's definition, and its list of maintainers, are committed to its origin's `main`. The runner reads both from `main` on the origin (`git show main:.acme/...`), so a change to them takes effect when it is merged to `main`, and not before. A run pins the definition commit it was requested against (`record.definition.commit`). The org only holds the engine and the list of people. The test `test_definition_owned_by_repo` shows a team tightening `reason` and changing its maintainers by committing to `main` | `<repo>/.acme/actions/mark-deprecated.json`, `<repo>/.acme/maintainers.json`, `load_definition` in `org/bin/acme` |
| R2 Validated | `preview` and `request` check every input against the definition before any record or git change is made. `successor` must fully match `^[a-z0-9-]+/[a-z0-9-]+$` and must not be the repository itself (`acme/<repo>`). `reason` must be 20 to 200 characters, on one line. Required inputs must be present and unknown inputs are refused. A refusal exits with code 2 and `error.code: "invalid_input"`, lists each problem, and appends `request_refused` to the audit log | `validate_inputs` in `org/bin/acme` and `inputs` in each definition |
| R3 Previewed | `acme preview` is a dry run: it shows the file, `create`/`modify`/`unchanged`, the exact new content, a unified diff against the current `main`, the base commit, the commit message and who can approve. It writes nothing. `acme request` stores the same preview in the run, and `acme show` displays it before approval. If the file on `main` changes between the preview and the approval, the run fails as `stale_preview` instead of applying a change nobody saw | `compute_preview` in `org/bin/acme`, `record.preview` |
| R4 Approved | A run starts as `awaiting_approval`, and nothing changes until it is approved. `approve` checks that the approver is a known person who is listed in the repository's `.acme/maintainers.json` on `main` now, and is not the requester. The rule that a requester cannot approve their own run is enforced by the engine, so a definition cannot turn it off. Self-approval, carol (not a maintainer) and unknown people get exit code 3 (`forbidden`). Each refusal is recorded as an `approve_refused` event, and the run stays `awaiting_approval`. carol may request runs | `cmd_decide` in `org/bin/acme` |
| R5 Done means observed | After approval the runner clones the origin, commits the change (author = the requester, committer = `acme-actions`, with `Run-Id`, `Requested-by` and `Approved-by` trailers) and pushes to `main`. Then it reads the origin again on its own. The run is `succeeded` only if (a) the push was accepted, (b) `main:DEPRECATED.md` equals the expected bytes and (c) the run's commit is on `main`. Otherwise it is `failed` with `outcome.code` set to `push_rejected`, `not_observed`, `stale_preview` or `clone_failed`, `approve` exits 6, and the record keeps the push output and the observation. The runner always makes a commit, an empty one if the content is already there, so a rejected push can never pass as success | `execute` in `org/bin/acme`, `record.observation`, `record.execution.push` |
| R6 One place for the org | From `org/`, `bin/acme list` lists every action on every registered repository's `main`, with inputs, approvers and whether the definition is valid. `bin/acme request <repo> <action>` requests a run on any repository without entering it. `runs` and `audit` cover the whole org | `org/acme.json`, `list_actions` in `org/bin/acme` |
| R7 Audited | Each run record has `requestedBy`/`requestedAt`, `approvals[]` and `approvedBy`/`approvedAt`, `inputs`, `outcome` (`status`, `at`, `code`, `message`, `commit`), `execution` (timestamps, commit, push result), `observation` (`at`, observed `main` commit, matches), and an ordered `events[]` with an `at` and an `actor` on each. Every event is also appended to `state/audit.jsonl`, which also holds refused requests. The commit on `main` carries the run id, the requester and the approver | `org/state/runs/<id>.json`, `org/state/audit.jsonl` |
| R8 Usable by agents | Every step is a non-interactive subcommand with JSON on stdout, and none of them prompt. Identity comes from `--as <person>` or `ACME_USER`. Inputs come from `-i key=value`, `--inputs-json '{...}'` or `--inputs-file f.json`. Exit codes: `0` ok, `2` invalid input or usage, `3` forbidden, `4` not found, `5` wrong state (for example, approving a finished run), `6` the run executed but failed, `1` internal error. Errors are `{"ok": false, "error": {"code", "message", "details"}}` | `org/bin/acme` |

## Commands

Run these from the org directory:

```bash
cd /tmp/actions-ab/r2/vanilla-1/org
```

**Start:** nothing. There is no server to start or stop.

**List** the actions on all repositories, or one action's full definition:

```bash
bin/acme list
bin/acme list --action mark-deprecated
bin/acme describe alpha mark-deprecated
```

**Preview** (a dry run that records nothing):

```bash
bin/acme preview alpha mark-deprecated -i successor=acme/dependabot-proxy-v2 -i "reason=Replaced by the v2 proxy, which supports OIDC registries."
```

**Request as ann.** This prints the run id (`.run.id`, for example `alpha-0002`) and the preview:

```bash
bin/acme request alpha mark-deprecated --as ann -i successor=acme/dependabot-proxy-v2 -i "reason=Replaced by the v2 proxy, which supports OIDC registries."
RUN=$(bin/acme request beta mark-deprecated --as ann --inputs-json '{"successor":"acme/valtio-store","reason":"State management moves to the shared store package."}' | python3 -c 'import json,sys; print(json.load(sys.stdin)["run"]["id"])')
```

**Approve as bob.** This executes the run and observes `main`. It exits 0 if the run succeeded and 6 if it failed:

```bash
bin/acme approve "$RUN" --as bob
```

**Status, record, run list and audit log:**

```bash
bin/acme status "$RUN"          # {"run": {"status": ..., "requestedBy", "approvedBy", "outcome", ...}}
bin/acme show "$RUN"            # the full record (alias: bin/acme record "$RUN")
bin/acme runs --repo alpha      # filter with --status succeeded|failed|awaiting_approval|rejected|cancelled
bin/acme audit --run "$RUN"
```

**Other steps:** a maintainer can refuse a run with `bin/acme reject "$RUN" --as bob --comment "..."`,
and the requester can withdraw it with `bin/acme cancel "$RUN" --as ann`.

The three runs already made can be checked with:

```bash
for id in alpha-0001 beta-0001 gamma-0001; do bin/acme status $id; done
for r in alpha beta gamma; do git --git-dir ../origins/$r.git show main:DEPRECATED.md; done
```

## Negative tests

Each of these leaves `main` unchanged:

```bash
# Invalid inputs: exit 2, error.code invalid_input, no run created
bin/acme request alpha mark-deprecated --as ann -i successor=Acme/Bad -i "reason=Long enough reason for the test."
bin/acme request alpha mark-deprecated --as ann -i successor=acme/x -i "reason=too short"
bin/acme request alpha mark-deprecated --as ann -i successor=acme/x -i "reason=$(printf 'x%.0s' {1..201})"
bin/acme request alpha mark-deprecated --as ann -i successor=acme/alpha -i "reason=Long enough reason for the test."
bin/acme request alpha mark-deprecated --as ann -i successor=acme/x          # missing reason

# Self-approval and carol: exit 3, run stays awaiting_approval
RUN=$(bin/acme request alpha mark-deprecated --as ann -i successor=acme/x -i "reason=Testing the approval rules here." | python3 -c 'import json,sys; print(json.load(sys.stdin)["run"]["id"])')
bin/acme approve "$RUN" --as ann
bin/acme approve "$RUN" --as carol
bin/acme status "$RUN"
bin/acme reject "$RUN" --as bob --comment "test only"   # clean up without changing main

# Rejected push: approve exits 6, status failed, outcome.code push_rejected
printf '#!/bin/sh\necho "main is protected" >&2\nexit 1\n' > ../origins/alpha.git/hooks/pre-receive
chmod +x ../origins/alpha.git/hooks/pre-receive
RUN=$(bin/acme request alpha mark-deprecated --as ann -i successor=acme/x -i "reason=This push should be rejected." | python3 -c 'import json,sys; print(json.load(sys.stdin)["run"]["id"])')
bin/acme approve "$RUN" --as bob; echo "exit=$?"
bin/acme show "$RUN"     # outcome, execution.push.output, observation
rm ../origins/alpha.git/hooks/pre-receive
```

The automated version of these checks runs against throwaway origins:

```bash
python3 -m unittest discover -s tests -v
```

## Notes and limits

- Identity is asserted, as the sandbox requires (`--as`). The runner checks every identity against
  `org/acme.json` and records who did what. A real deployment would take identity from SSO or the
  code host.
- Run state is local files under `org/state/` and is not committed. Mutations are serialised with
  an `flock` on `org/state/.lock`, and records are written atomically. To use another location,
  set `ACME_STATE`. `ACME_ORG` points the runner at another org directory, which is how the tests
  work.
- Approval runs the action synchronously in the same command. If that process is killed partway,
  the run stays `running`. There is no automatic reconcile; check `main` and the record by hand.
- A definition's effect is declarative (`write-file`). It cannot target `.git/` or `.acme/`, so an
  action cannot rewrite its own definition or the maintainers list.
