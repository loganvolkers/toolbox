# Handoff: `mark-deprecated`, a repository-owned self-service action

**Status:** `mark-deprecated` is defined in `alpha`, `beta` and `gamma`. Each was requested by ann
and approved by bob, and all three runs are reported `succeeded` (`run-0001` alpha, `run-0002` beta,
`run-0003` gamma). Each was reported only after `DEPRECATED.md` was read back from the origin's
`main` with the expected bytes.

**Nothing needs to be running.** The system is a single CLI, `org/bin/acme` (Python 3.12, stdlib
only). Each command reads the origins directly, does its work, and exits. It never starts a server
or a daemon. Every command prints one JSON document on stdout.

All commands below run from the org directory:

```sh
cd /tmp/actions-ab/r4/vanilla-3/org
```

## Layout

| Path | What it is |
|---|---|
| `<repo>/.acme/actions/mark-deprecated.json` (on `origins/<repo>.git` `main`) | The action's definition, owned by that repository: maintainers, input rules, effect (file, template, commit message) |
| `<repo>/.acme/README.md` | Tells the repository team that they own `.acme/actions/` |
| `org/acme.json` | Org registry: org name (`acme`), people (ann, bob, carol), and the three repositories with their origins |
| `org/bin/acme` | The engine and CLI: discovery, validation, preview, request, approval, execution, observation, audit |
| `org/state/runs/run-NNNN.json` | One record per run (gitignored runtime state) |
| `org/state/audit.jsonl` | Append-only audit log of every event, including refused requests and refused approvals |
| `org/tests/test_acme.py` | End-to-end tests against throwaway origins (`python3 -m unittest discover -s tests -v`, 10 tests, all pass) |

## How each requirement is met

| # | How | Where |
|---|---|---|
| R1 Owned by the repository | The definition is a file in each repository, committed to its origin's `main`. The CLI reads it with `git --git-dir origins/<repo>.git` at `main` on every command, never from a checkout or a copy. A change the team lands on `main` takes effect on the next command: for example, editing `maintainers` changes who can approve. Each run pins the definition's commit and blob. If the definition on `main` changes before approval, the run becomes `stale` and is not executed. | `alpha/.acme/actions/mark-deprecated.json` (also in `beta/` and `gamma/`); `load_definition_at`, `list_definitions` and `decide` in `org/bin/acme` |
| R2 Validated | The definition declares the rules: `successor` must match `^[a-z0-9-]+/[a-z0-9-]+$` (full match, so a trailing newline fails), and it may not be the repository itself (`notSelf`). `reason` must be 20 to 200 characters. Unknown and missing inputs are refused. `preview` and `request` validate before anything is written. A refused request creates no run and changes nothing, but it is logged as `request_refused` in `audit.jsonl`. It exits with code 2 and `error.code: "inputs_invalid"`, listing an error per input. | `validate_inputs`, `cmd_request` in `org/bin/acme` |
| R3 Previewed | `preview` returns the base commit, the file, the operation (`create`, `update` or `none`), the exact new content, a unified diff, and the commit message. Nothing is written. `request` stores the same preview in the run, so the approver sees what was previewed (`record`). At execution the file's state on `main` must still match the preview; otherwise the run fails with `stale_preview`. | `plan`, `cmd_preview`, `execute` in `org/bin/acme` |
| R4 Approved | A run starts as `pending_approval`, and only `approve` executes it. The approver must be a known person. They must be in `maintainers` of the definition currently on `main` (ann and bob), and they may not be the requester. carol gets `not_a_maintainer`, self-approval gets `self_approval`, and both exit 3 and are logged as `approve_refused` on the run. Anyone in the org may request, carol included. `reject` (a maintainer) and `cancel` (the requester) end a pending run. A file lock serializes the state changes. | `decide`, `cmd_cancel` in `org/bin/acme` |
| R5 Done means observed | After approval, the engine works in a scratch bare clone: it commits the rendered file on top of `main` and pushes it, non-force, to the origin. It then reads the origin's `main` itself. It reports `succeeded` only if `DEPRECATED.md` there has exactly the expected bytes and `main` contains the run's commit. A rejected push gives `failed` with `reason: "push_rejected"` and the hook's stderr. A push that doesn't stick gives `not_observed`. If `main` keeps moving, it retries three times and then reports `push_failed`. Exceptions give `failed`, never `succeeded`. The observation (commit, blob, `contentMatches`, time) is stored in `outcome.observed`. | `execute` in `org/bin/acme` |
| R6 One place for the org | `org/acme.json` lists the repositories and their origins. From `org/`, `bin/acme list` discovers every `.acme/actions/*.json` on every origin's `main`, and `bin/acme request <repo> ...` works on any of them, without entering the repositories. | `org/acme.json`, `cmd_list` in `org/bin/acme` |
| R7 Audited | The run record has `requestedBy`/`requestedAt`, `approvedBy`/`approvedAt`, `inputs`, `status`, `outcome` and `finishedAt`. It also has the pinned definition, the preview, and timestamped `events` (requested, refused attempts with who tried, approved, started, succeeded/failed/rejected/cancelled/stale). Every event is also appended to `state/audit.jsonl`. The commit on `main` is authored by the requester and carries `Acme-Run`, `Requested-by` and `Approved-by` trailers. | `org/state/runs/*.json`, `org/state/audit.jsonl`, `add_event` in `org/bin/acme` |
| R8 Usable by agents | Every step is one non-interactive command that prints JSON, including usage errors. Identity comes from `--as <person>` or `ACME_ACTOR`. Exit codes: 0 ok, 2 invalid input or definition, 3 forbidden, 4 not found, 5 conflict (not pending, definition changed), 6 the run executed and failed. `bin/acme --help` and `bin/acme <command> --help` describe each command. | `build_parser`, `emit` in `org/bin/acme` |

## Commands

Start: nothing to start.

List the actions for all three repositories, or describe one:

```sh
bin/acme list
bin/acme list --repo beta
bin/acme describe alpha mark-deprecated
```

Preview, without changing anything. Inputs are `-i key=value`, or `--inputs-json '{"successor":"...","reason":"..."}'`:

```sh
bin/acme preview alpha mark-deprecated \
  -i successor=acme/dependabot-proxy-v2 \
  -i "reason=Replaced by the v2 proxy, which supports OIDC registries."
```

Request a run as ann. This validates, previews, and prints the run, including `id` and `preview`:

```sh
bin/acme request alpha mark-deprecated --as ann \
  -i successor=acme/dependabot-proxy-v2 \
  -i "reason=Replaced by the v2 proxy, which supports OIDC registries."
```

Approve as bob. Approval executes the run and prints the final record. It exits 0 when the run succeeded and 6 when it failed:

```sh
bin/acme approve run-0001 --as bob
```

Check status, read the record, list runs, and read the audit log:

```sh
bin/acme status run-0001          # add --wait-terminal to exit 5 while pending or running
bin/acme record run-0001
bin/acme runs --repo alpha --status succeeded
bin/acme audit --run run-0001
```

Reject a run (a maintainer) or cancel it (the requester):

```sh
bin/acme reject run-0004 --as bob --comment "not yet"
bin/acme cancel run-0004 --as ann
```

## Testing the negative cases

`DEPRECATED.md` is already on each `main`. A request with the same inputs previews as
`"operation": "none"`. Approving it pushes nothing and succeeds by observation. To exercise a
real push, use a different `reason` or `successor`, which previews as `"operation": "update"`.

Invalid inputs (each exits 2; no run is created and nothing changes):

```sh
bin/acme request beta mark-deprecated --as ann -i successor=Acme/Store -i "reason=Long enough reason text."
bin/acme request beta mark-deprecated --as ann -i successor=acme/beta -i "reason=Long enough reason text."
bin/acme request beta mark-deprecated --as ann -i successor=acme/store -i "reason=too short"
bin/acme request beta mark-deprecated --as ann -i successor=acme/store -i "reason=$(printf 'x%.0s' $(seq 201))"
```

Self-approval and carol's approval (each exits 3; the run stays `pending_approval` and `main` is untouched):

```sh
RUN=$(bin/acme request beta mark-deprecated --as ann -i successor=acme/store-v2 \
  -i "reason=Moving again, to the v2 shared store." | python3 -c 'import json,sys; print(json.load(sys.stdin)["id"])')
bin/acme approve "$RUN" --as ann     # error.code: self_approval
bin/acme approve "$RUN" --as carol   # error.code: not_a_maintainer
bin/acme status "$RUN"               # still pending_approval
```

Rejected push (exits 6; the run is `failed` with `outcome.reason: "push_rejected"` and never `succeeded`):

```sh
printf '#!/bin/sh\necho "main is protected" >&2\nexit 1\n' > ../origins/beta.git/hooks/pre-receive
chmod +x ../origins/beta.git/hooks/pre-receive
bin/acme approve "$RUN" --as bob
bin/acme status "$RUN"
rm ../origins/beta.git/hooks/pre-receive
```

The automated suite covers all of the above plus a push that the origin silently undoes
(`not_observed`), a stale preview, and a maintainer change landed on `main`:

```sh
python3 -m unittest discover -s tests -v
```

## Notes and limits

- **Identity:** identity is asserted with `--as`, as the sandbox allows. Every assertion is recorded. In production, `--as` would be replaced by an authenticated principal.
- **File format:** the file content is `# Deprecated: use <successor>`, a blank line, then `<reason>` and a final newline, as rendered from the definition's `content` template.
- **Pinning:** a run executes the definition and preview it was requested with. If either changed on `main` before execution, it is refused (`stale`, `stale_preview`), not reinterpreted.
- **State:** run state is local to `org/state/` and is not committed. Back it up or move it with `ACME_STATE_DIR`. `ACME_CONFIG` points the CLI at another registry; the tests use that.
