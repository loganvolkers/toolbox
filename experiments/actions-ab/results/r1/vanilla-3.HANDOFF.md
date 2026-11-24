# HANDOFF: `mark-deprecated` self-service action

**Status:** built and run. `mark-deprecated` succeeded on all three repositories, requested by ann
and approved by bob:

| Repository | Run id | Commit on origin `main` |
|---|---|---|
| acme/alpha | `run-20261124-105634-ae9bf6` | `15bb8e7` |
| acme/beta | `run-20261124-105634-232052` | `577bf57` |
| acme/gamma | `run-20261124-105634-e48094` | `3a9c733` |

**Nothing has to be running.** The system is one command-line program,
`/tmp/actions-ab/r1/vanilla-3/org/bin/acme`, written in Python 3.12 with only the standard library.
It reads action definitions straight from each origin's `main` and keeps run records in
`org/state/`. There is no server, and none was left running.

## Layout

| Path | What it is |
|---|---|
| `<repo>/.acme/actions/mark-deprecated.json` (on each origin's `main`) | The repository's own definition of the action: inputs and validation rules, approval policy, the file it writes, and the commit message |
| `<repo>/.acme/maintainers.json` (on each origin's `main`) | Who can approve runs in that repository (`ann`, `bob`) |
| `org/bin/acme` | The org-wide CLI: list, show, preview, request, approve, reject, status, record, runs |
| `org/repos.json` | The org's repositories and the path of each origin |
| `org/people.json` | Known identities (`ann`, `bob`, `carol`). Unknown names are refused |
| `org/state/runs/<run-id>.json` | One record per run, with timestamped events (not committed; `.gitignore`) |
| `org/state/audit.jsonl` | Append-only log of every event across all runs |
| `org/state/work/` | Scratch clones the executor uses during a run; each is deleted afterwards |
| `org/tests/test_acme.sh` | End-to-end tests on throwaway copies of the origins (35 checks) |

## How each requirement is met

| # | How | Files |
|---|---|---|
| R1 Owned by the repository | Each repository commits its own definition and approver list to its origin's `main`, in commits by ann. `acme` reads them from `refs/heads/main` of the bare origin, not from a checkout or from `org/`, so editing and pushing them in the repository changes the action. Each run pins the definition's commit and sha256 in its record | `alpha/.acme/actions/mark-deprecated.json`, `alpha/.acme/maintainers.json` (and the same in `beta/`, `gamma/`), `load_definition()` in `org/bin/acme` |
| R2 Validated | The rules live in the definition. `successor` must fully match `^[a-z0-9-]+/[a-z0-9-]+$` and must not be the repository itself (`acme/<name>`). `reason` must be 20 to 200 characters on a single line. Missing and unknown inputs are refused. `preview` and `request` both validate before anything is written: no run record is created and nothing is pushed. Invalid input exits 2 with `error.code = "invalid_input"` and one entry per failing field in `error.details` | `inputs` in each `mark-deprecated.json`, `validate()` in `org/bin/acme` |
| R3 Previewed | `acme preview` changes nothing and shows the plan against the current `main`: base commit, each file's operation (`create`/`modify`/`unchanged`), its content before and after, a unified diff, and the commit message. `request` stores that same preview in the run. When the run is approved, the executor recomputes the plan and fails the run (`preview is stale`) without pushing if `main` changed a target file after the preview. A request that would change nothing is refused (`no_change`, exit 5) | `plan_changes()`, `cmd_preview()`, and the stale check in `execute()` |
| R4 Approved | Runs start as `awaiting_approval`. `approve` is accepted only from someone listed in the repository's `.acme/maintainers.json` on `main` at approval time, and never from the requester (`requesterCanApprove: false`). It needs `required: 1` approval. If the requester or carol tries, the command exits 3 with `error.code = "forbidden"`, and the attempt is written to the run's record as an `approve_denied` event. Nothing touches the origin until approval. `reject` follows the same rules | `approval` in each definition, `check_can_decide()` and `cmd_approve()` in `org/bin/acme` |
| R5 Done means observed | After approval, the executor clones `main`, commits, and pushes. If the push fails, the run is `failed` and the record keeps git's stderr. If the push succeeds, the executor reads `DEPRECATED.md` from the origin's `refs/heads/main` and compares it byte for byte with the rendered content. The run is `succeeded` only when they match. Otherwise it is `failed` once a timeout passes (`ACME_OBSERVE_TIMEOUT`, default 10s). The observation (commit, path, sha256 of the expected and actual content) is stored in the record | `execute()`, `observe()` and `finish()` in `org/bin/acme` |
| R6 One place for the org | From `org/`, `bin/acme list` shows every action in every repository listed in `org/repos.json`, and `request <repo> <action>` works on any of them. There is no need to enter a repository | `org/bin/acme`, `org/repos.json` |
| R7 Audited | Each run record holds `requestedBy`/`requestedAt`, `approvals[]` (each with `by` and `at`), `inputs`, `outcome` (`status`, `at`, `message`), `execution` (start, push result, finish), `observation`, and `events[]` (`requested`, `approve_denied`, `approved`, `execution_started`, `push_attempted`, `observed`, `succeeded`/`failed`/`rejected`, each with `at` and `actor`). The same events go to `state/audit.jsonl`. Each commit the action pushes carries `Acme-Run`, `Requested-by` and `Approved-by` trailers | `org/state/runs/*.json`, `org/state/audit.jsonl`, `event()` in `org/bin/acme` |
| R8 Usable by agents | Every command is non-interactive and prints exactly one JSON document on stdout, usage errors included. Identity comes from `--as <person>` or `$ACME_USER`. Inputs come from `-i key=value` or `--inputs-json '{...}'`. Exit codes: `0` ok, `1` run failed, `2` invalid input or usage, `3` forbidden, `4` not found, `5` wrong state or no change | `org/bin/acme` (module docstring, `main()`) |

Notes:
- `DEPRECATED.md` ends with a single newline after `<reason>`: `# Deprecated: use <successor>\n\n<reason>\n`.
- There is no retry. A failed or rejected run is final; request a new one.
- Identity is asserted, not authenticated, because the sandbox has no passwords. The approval rules
  and the audit trail are enforced on the asserted identity.

## Commands

Run everything from the org directory:

```bash
cd /tmp/actions-ab/r1/vanilla-3/org
```

**Start:** nothing to start. To check the system end to end without touching the real origins,
run `bash tests/test_acme.sh`.

**List the actions** in all three repositories (or add `--repo alpha` for one):

```bash
bin/acme list
bin/acme show alpha mark-deprecated
```

**Preview** (validates and shows the diff; changes nothing):

```bash
bin/acme preview alpha mark-deprecated --as ann \
  -i successor=acme/dependabot-proxy-v2 \
  -i "reason=Replaced by the v2 proxy, which supports OIDC registries."
```

**Request as ann** (validates, stores the preview, prints `run.id`):

```bash
RUN=$(bin/acme request alpha mark-deprecated --as ann \
  --inputs-json '{"successor":"acme/new-thing","reason":"A different reason, at least twenty chars."}' \
  | python3 -c 'import json,sys; print(json.load(sys.stdin)["run"]["id"])')
echo "$RUN"
```

All three repositories are already deprecated. A request with the same inputs as a finished run is
refused as `no_change`, so use a different `successor` or `reason` (as above) to make a new run. It
will show `operation: "modify"`.

**Approve as bob** (once approved, the run executes, pushes and observes `main` in the same command):

```bash
bin/acme approve "$RUN" --as bob --comment "ok"
```

**Reject** (as a maintainer other than the requester):

```bash
bin/acme reject "$RUN" --as bob --comment "not now"
```

**Check status** and **read the record**:

```bash
bin/acme status "$RUN"          # {"status": "awaiting_approval" | "running" | "succeeded" | "failed" | "rejected", ...}
bin/acme record "$RUN"          # full audit record
bin/acme runs --status succeeded
bin/acme record run-20261124-105634-ae9bf6
```

### Negative tests

```bash
# Invalid inputs: exit 2, nothing changes
bin/acme preview alpha mark-deprecated --as ann -i successor=NotValid -i "reason=Replaced by the v2 proxy, which supports OIDC registries."
bin/acme request alpha mark-deprecated --as ann -i successor=acme/x -i "reason=too short"
bin/acme request alpha mark-deprecated --as ann -i successor=acme/alpha -i "reason=Points at itself, which is refused."

# Self-approval and carol: exit 3, recorded as approve_denied, run stays awaiting_approval
RUN=$(bin/acme request beta mark-deprecated --as ann -i successor=acme/other -i "reason=Testing the approval rules here." | python3 -c 'import json,sys; print(json.load(sys.stdin)["run"]["id"])')
bin/acme approve "$RUN" --as ann
bin/acme approve "$RUN" --as carol
bin/acme status "$RUN"

# Rejected push: run reported failed (exit 1), main unchanged
printf '#!/bin/sh\necho "main is protected" >&2\nexit 1\n' > ../origins/beta.git/hooks/pre-receive
chmod +x ../origins/beta.git/hooks/pre-receive
bin/acme approve "$RUN" --as bob          # status: failed; record.execution.push.stderr shows the rejection
rm ../origins/beta.git/hooks/pre-receive
```

Configuration (optional, used by the tests): `ACME_REPOS_FILE`, `ACME_PEOPLE_FILE`, `ACME_STATE_DIR`,
`ACME_OBSERVE_TIMEOUT`, `ACME_USER`.
