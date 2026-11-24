# Handoff: `mark-deprecated` self-service action

`mark-deprecated` has run on alpha, beta and gamma. In each case ann requested it, bob approved it,
and the run was reported as `succeeded` only after `DEPRECATED.md` was read back from the origin's
`main`:

| Run | Repository | Requested by | Approved by | Outcome | Commit on origin `main` |
|---|---|---|---|---|---|
| run-0001 | alpha | ann | bob | succeeded | `b0dd4b5` |
| run-0002 | beta | ann | bob | succeeded | `14f892f` |
| run-0003 | gamma | ann | bob | succeeded | `733499e` |

## What it is

The system is one command-line tool with no server and no daemon, so **nothing needs to be started**.

| Path | What it is |
|---|---|
| `org/acme` | Entry point (a shell wrapper around `org/acme.py`) |
| `org/acme.py` | The engine: Python 3.12, standard library only |
| `org/repos.json` | The org registry: the three repositories (origin path and slug) and the people (ann, bob, carol) |
| `<repo>/.acme/actions/mark-deprecated.json` | Each repository's own definition of the action, committed to `origins/<repo>.git` `main` |
| `org/state/runs/run-NNNN.json` | One record per run (git-ignored; it is runtime state) |
| `org/state/audit.jsonl` | Append-only org audit log, hash-chained |

All paths below are relative to `/tmp/actions-ab/r7/vanilla-3`.

The engine reads definitions from each **origin's `main`** through a fresh fetch, never from the local
checkouts. It pins the definition in the run record when the run is requested, so the approver approves exactly what was
previewed. It executes by committing to a throwaway clone and pushing to `main` without force. It
then fetches `main` again and compares `DEPRECATED.md` byte for byte.

## Requirements

| # | How it is met | Where |
|---|---|---|
| R1 Owned by the repository | The whole action is declared in each repository: its maintainers, approval policy, input rules, the file it writes and its template. It is committed to each origin's `main`, so the repository's team changes it with an ordinary commit. `org/acme` holds no per-action logic; it interprets whatever the definition on `main` says. | `alpha/.acme/actions/mark-deprecated.json` (and the same in `beta/` and `gamma/`), commits `01c1e54`, `feda71a`, `ee10349` |
| R2 Validated | `preview` and `request` check every input against the definition: unknown or missing inputs, `pattern` (full match), `minLength`/`maxLength` (characters), `singleLine`, and `notThisRepository` (the successor cannot be the repository itself). On failure no run is created and nothing is fetched for writing. The command exits 2 with every error listed. Inputs are checked again just before execution. | `validate_inputs` in `org/acme.py` |
| R3 Previewed | `acme preview` (no run is created) and `acme request` (stored in the run as `preview`) show the target origin, branch and base commit, whether the file is created, modified or unchanged, its full new content, and a unified diff. `acme record` shows the same preview to the approver. | `compute_preview` in `org/acme.py` |
| R4 Approved | A run is created as `pending_approval`. Nothing is pushed until `acme approve`. Approval is refused (exit 3) when the approver is the requester, or is not in the definition's `maintainers` (so carol is refused). It is also refused for unknown people, and with exit 5 when the run is not pending. Refused attempts are written to the record and the audit log. `acme reject` has the same rules. | `decide` in `org/acme.py`; `maintainers` in each definition |
| R5 Done means observed | After the push, or after a rejected push, the engine re-fetches the origin's `main` and compares `DEPRECATED.md` with the expected bytes. The outcome is `succeeded` only on a match (event `observed`, `matches: true`). A rejected push gives `failed` ("push to origin was rejected", exit 1). An accepted push that does not end up on `main` also gives `failed` ("does not have the requested content"). Both were tested with a `pre-receive` hook and with a `post-receive` hook that resets `main`. | `execute` in `org/acme.py` |
| R6 One place for the org | `org/acme list` lists `mark-deprecated` for all three repositories, read from their origins. `org/acme request <repo> mark-deprecated ...` works for any repository named in `org/repos.json`, from the org directory. | `org/acme`, `org/repos.json` |
| R7 Audited | Each record has `requestedBy`/`requestedAt`, `approvedBy`/`approvedAt` (or `rejectedBy`/`rejectedAt`), `inputs`, `outcome`, `finishedAt`, `result` (head and commit), and a timestamped `events` list. The list covers `requested`, `approval_refused`, `approved`, `pushed`/`push_rejected` (with git's output), `observed` and `succeeded`/`failed`. Each event is also appended to `org/state/audit.jsonl` with a SHA-256 chain that `acme audit` verifies. The commit on `main` carries `Acme-Run`, `Requested-by` and `Approved-by` trailers, with the requester as author. | `org/state/runs/*.json`, `org/state/audit.jsonl`, `audit` in `org/acme.py` |
| R8 Usable by agents | Every subcommand is non-interactive (identity comes from `--as` or `ACME_ACTOR`), prints exactly one JSON document on stdout, and uses distinct exit codes: 0 ok, 1 run failed, 2 invalid input, 3 forbidden, 4 not found, 5 wrong run state, 6 system error. Errors are `{"ok": false, "error": {"code", "message", "details"}}`. | `org/acme.py`; `org/acme --help` |

## Commands

```sh
cd /tmp/actions-ab/r7/vanilla-3/org

# Start: nothing to start. Requirements: python3 (3.12) and git.

# List the actions in all three repositories
./acme list
./acme list --repo beta                 # one repository

# Preview (validates; changes nothing, creates no run)
./acme preview alpha mark-deprecated \
  --input successor=acme/dependabot-proxy-v2 \
  --input "reason=Replaced by the v2 proxy, which supports OIDC registries."

# Request a run as ann (validates, previews, records; prints the run id, e.g. run-0004)
./acme request alpha mark-deprecated --as ann \
  --input successor=acme/dependabot-proxy-v2 \
  --input "reason=Replaced by the v2 proxy, which supports OIDC registries."
# Inputs can also be passed as JSON: --inputs-json '{"successor":"acme/x","reason":"..."}'

# Approve as bob (executes, then observes origin main; exits 0 only if it succeeded)
./acme approve run-0004 --as bob [--comment "lgtm"]
./acme reject  run-0004 --as bob [--comment "why"]

# Status, the full record, all runs, the audit log
./acme status run-0004
./acme record run-0004
./acme runs [--repo alpha] [--state pending_approval]
./acme audit [--run run-0004]

# Pull a field out with any JSON tool, e.g.
./acme status run-0004 | python3 -c 'import json,sys; print(json.load(sys.stdin)["run"]["outcome"])'
```

## Testing the failure paths

Each command below should exit non-zero with a JSON error, and nothing should change on any origin
(`git --git-dir=../origins/alpha.git log --oneline -1 main` stays the same):

```sh
# Invalid inputs (exit 2, no run created)
./acme preview alpha mark-deprecated -i successor=Acme/Proxy -i "reason=too short"
./acme request alpha mark-deprecated --as ann -i successor=acme/alpha -i "reason=points at itself, which is refused"
./acme request alpha mark-deprecated --as ann -i successor=acme/x -i "reason=$(printf 'x%.0s' $(seq 201))"
./acme request alpha mark-deprecated --as ann -i successor=acme/x            # reason missing

# Self-approval and carol (exit 3; the run stays pending_approval)
ID=$(./acme request alpha mark-deprecated --as ann -i successor=acme/proxy-v3 \
      -i "reason=Testing the approval rules for this action." | python3 -c 'import json,sys;print(json.load(sys.stdin)["run"]["id"])')
./acme approve "$ID" --as ann
./acme approve "$ID" --as carol
./acme status  "$ID"                     # still pending_approval
./acme reject  "$ID" --as bob            # clean up

# Rejected push (approve exits 1; outcome failed; main unchanged)
printf '#!/bin/sh\necho "main is protected" >&2\nexit 1\n' > ../origins/alpha.git/hooks/pre-receive
chmod +x ../origins/alpha.git/hooks/pre-receive
ID=$(./acme request alpha mark-deprecated --as ann -i successor=acme/proxy-v3 \
      -i "reason=This change must be rejected by the origin." | python3 -c 'import json,sys;print(json.load(sys.stdin)["run"]["id"])')
./acme approve "$ID" --as bob            # "ok": false, state failed, "push to origin was rejected"
./acme record "$ID"                      # push_rejected event carries git's output
rm ../origins/alpha.git/hooks/pre-receive
```

Use inputs that **differ** from the ones already applied when you test a rejected push. If
`DEPRECATED.md` on `main` already has the requested content, the run records `change_not_needed`.
It then observes the content and reports `succeeded` without pushing. That is the intended
idempotent behaviour: the outcome is the observed state.

Running this tool with `ACME_REGISTRY=<file>` and `ACME_STATE_DIR=<dir>` points it at another
registry and state directory, for example scratch copies of the origins.

## Limits

- Identity is asserted with `--as`, as the sandbox allows. There is no authentication. The rules
  are enforced by the engine, and the records live on the local filesystem.
- The definition's `effect.type` supports `commit-file` only, with exactly one approval.
- Runs execute synchronously inside `acme approve`; a file lock (`org/state/.lock`) serialises
  concurrent requests and approvals.
