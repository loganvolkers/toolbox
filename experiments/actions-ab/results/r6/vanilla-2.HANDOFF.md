# HANDOFF: `mark-deprecated`, a repository-owned self-service action

## Status

The three runs requested by ann and approved by bob all **succeeded**. Each was reported as
succeeded only after the tool saw `DEPRECATED.md` with the expected content on the origin's `main`.

| Run | Repository | Commit on origin `main` |
|---|---|---|
| `run-0001` | alpha | `d174a9e` |
| `run-0002` | beta | `8951cea` |
| `run-0003` | gamma | `ce9c5be` |

## What it is

A dependency-free Python 3 CLI, `acme`, in the org repository. **Nothing needs to be running**:
there is no server, and every command is a one-shot process.

| Piece | Path |
|---|---|
| Action definition, one per repository, on its origin's `main` | `<repo>/.acme/actions/mark-deprecated.json` (alpha, beta, gamma) |
| CLI engine | `org/acme_actions.py` |
| CLI entry point | `org/bin/acme` |
| Org registry of repositories (name, slug, origin) | `org/repos.json` |
| People (identities) | `org/people.json` |
| Run records, one JSON file per run | `org/state/runs/run-NNNN.json` |
| Append-only org audit log, including refused requests | `org/state/audit.jsonl` |
| Fetch cache of each origin's `main` | `org/state/mirrors/<repo>.git` |
| End-to-end scenarios, run against scratch copies of the origins | `org/tests/scenarios.sh` |

`org/state/` is runtime data and is git-ignored. The CLI is committed to `origins/org.git` `main`.

## How each requirement is met

| # | How | Files |
|---|---|---|
| R1 Owned by the repository | Each repository's definition is `.acme/actions/mark-deprecated.json`, committed to its origin's `main` by ann. The definition holds the inputs and their validation rules, the approvers, the file template and the commit message. The CLI reads the definition from the origin's `main` every time and has no copy of its own, so the repository's team changes the action by changing that file. Each run pins the definition commit it was requested against, so editing the definition cannot change a pending run's approvers. | `alpha/.acme/actions/mark-deprecated.json`, `beta/...`, `gamma/...`; `load_definitions` in `org/acme_actions.py` |
| R2 Validated | `preview` and `request` check the inputs against the definition before anything is written. `successor` must fully match `^[a-z0-9-]+/[a-z0-9-]+$` and must not be the repository itself (`notSelf`). `reason` must be 20 to 200 characters. Required, unknown and non-string inputs are also checked. If validation fails, the command exits 2 with `error.code` set to `invalid_inputs` and per-input errors; no run is created and the origin is not touched. | `validate_inputs` in `org/acme_actions.py` |
| R3 Previewed | `acme preview` prints the base commit, each file's operation (create, modify or unchanged), its before and after content, a unified diff and the commit message. `request` returns the same preview and stores it in the run. At approval time the change is recomputed against the current `main`. If it differs from what was previewed, the run fails with `stale_preview` and nothing is pushed. | `compute_plan` and `execute` in `org/acme_actions.py` |
| R4 Approved | A run is created as `awaiting_approval`. Only `approve` executes it. The requester is always refused (`self_approval`, exit 3). Anyone not in the pinned definition's `approval.approvers` (`["ann","bob"]`), such as carol, is also refused (`not_an_approver`, exit 3). Unknown names are refused (`unknown_person`). Every refused approval is recorded in the run's events. | `cmd_approve` in `org/acme_actions.py` |
| R5 Done means observed | After approval, the tool builds a commit on the origin's current `main` and pushes it without force. It then fetches `main` from the origin again and compares `DEPRECATED.md` byte for byte, retrying up to 5 times at 1-second intervals. Only a match sets the run to `succeeded`. A rejected push gives `failed` with `push_rejected` and the push output. If the push is accepted but `main` doesn't have the content, the run is `failed` with `not_observed`. Any crash during execution is recorded as `failed`. `approve` exits 6 for a failed run. | `execute`, `verify` and `fail` in `org/acme_actions.py` |
| R6 One place for the org | From `org/`, run `./bin/acme list` to see `mark-deprecated` for all repositories in `repos.json`, read from each origin. `request`, `approve`, `status`, `record` and `runs` take a repository name or run id, so nobody has to enter a repository. | `org/bin/acme`, `org/repos.json` |
| R7 Audited | Each run record holds `requestedBy` and `requestedAt`, `inputs`, the pinned definition (path, commit and content), the preview, `approval` (`by`, `at`, `comment`), `execution` (commit, base, push exit code and output), `verification` (when observed, the `main` commit, per-file match, whether `main` contains the run's commit) and `outcome` (status, `at`, detail). `events[]` lists every step with a timestamp and actor: requested, approval_refused, approved, committed, pushed or push_rejected, observed, then succeeded or failed. The same events are appended to `state/audit.jsonl`, along with refused requests. The pushed commit's author is the requester, and its trailers are `Acme-Run`, `Requested-by` and `Approved-by`. | `org/state/runs/*.json`, `org/state/audit.jsonl` |
| R8 Usable by agents | Every command is non-interactive and prints one JSON document on stdout, including errors (`{"ok": false, "error": {"code", "message", "details"}}`) and usage errors. Identity comes from `--as <user>` or `ACME_USER`. Exit codes: 0 ok, 2 invalid input or usage, 3 forbidden, 4 not found, 5 wrong state (for example, already approved, or no change), 6 run failed. | `org/acme_actions.py` |

## Commands

Run everything from the org directory. Requires `python3` and `git`; `jq` is optional.

```sh
cd /tmp/actions-ab/r6/vanilla-2/org

# Start: nothing to start (no server). Optional self-test on scratch copies of the origins (needs jq):
tests/scenarios.sh

# List the actions in all three repositories (or: --repo alpha)
./bin/acme list
./bin/acme show alpha mark-deprecated

# Preview (validates; changes nothing)
./bin/acme preview alpha mark-deprecated \
  --input successor=acme/dependabot-proxy-v2 \
  --input "reason=Replaced by the v2 proxy, which supports OIDC registries."
# equivalently: --inputs-json '{"successor":"acme/x","reason":"..."}'

# Request as ann (prints the run id as .run.id, plus the preview)
./bin/acme request alpha mark-deprecated --as ann \
  --input successor=acme/dependabot-proxy-v2 \
  --input "reason=Replaced by the v2 proxy, which supports OIDC registries."

# Approve as bob (executes, pushes and observes main; exit 0 only if succeeded)
./bin/acme approve run-0001 --as bob

# Status and full record
./bin/acme status run-0001
./bin/acme record run-0001
./bin/acme runs                 # all runs; filter with --repo alpha or --status failed

# Other: reject (approver), cancel (requester), reconcile (re-observe a run stuck after a crash)
./bin/acme reject run-0004 --as bob --comment "not yet"
./bin/acme cancel run-0004 --as ann
./bin/acme reconcile run-0004
```

## Testing the negative cases

- **Invalid inputs:** for example `--input successor=Acme/X`, `successor=acme/alpha` on alpha, a
  reason under 20 or over 200 characters, or a missing input. Each exits 2 with `invalid_inputs`;
  `./bin/acme runs` shows no new run.
- **Self-approval:** request as ann, then `approve <id> --as ann`. It exits 3 with `self_approval`,
  and the run stays `awaiting_approval`.
- **carol's approval:** `approve <id> --as carol` exits 3 with `not_an_approver`.
- **Rejected push.** The three repositories already have `DEPRECATED.md`, and requesting identical
  content is refused (`no_change`, exit 5) because there would be nothing to push. Use a different
  reason, for example:

  ```sh
  printf '#!/bin/sh\necho "rejected by policy" >&2\nexit 1\n' > ../origins/alpha.git/hooks/pre-receive
  chmod +x ../origins/alpha.git/hooks/pre-receive
  ./bin/acme request alpha mark-deprecated --as ann --input successor=acme/other-repo \
    --input "reason=Testing that a rejected push is reported as failed."
  ./bin/acme approve <id> --as bob    # exit 6; .run.status == "failed"; .error.code == "push_rejected"
  ./bin/acme record <id>              # outcome and execution.pushOutput show the rejection
  rm ../origins/alpha.git/hooks/pre-receive
  ```

  The run's previewed change must still be what `main` would receive at approval time. If someone
  else changes `DEPRECATED.md` on `main` in between, the run fails with `stale_preview` instead of
  overwriting their change.

## Design notes

- The engine supports only the `write-file` step type with `${input}`, `${repo.name}` and
  `${repo.slug}` substitution. A repository can change what the action writes and who approves it,
  but it cannot run arbitrary code through the org tool.
- Any person in `people.json` can request a run. Approval is the gate.
- Mutating commands are serialized with a file lock on `state/lock`, and records are written
  atomically, so concurrent agents are safe on this machine.
- There is no authentication in this sandbox: `--as` is trusted, as the brief allows. Every action is
  attributed and recorded.
