# Handoff: the `mark-deprecated` self-service action

**Status:** built and run. `mark-deprecated` is defined in `alpha`, `beta` and `gamma`. It was
requested by ann and approved by bob on each, and all three runs are `succeeded`. Each was
reported only after `DEPRECATED.md` with the exact content had been observed on the origin's
`main`:

| Run | Repository | successor | Commit on `main` |
|---|---|---|---|
| `run-0001` | alpha | `acme/dependabot-proxy-v2` | `def9db5` |
| `run-0002` | beta | `acme/valtio-store` | `8cffb4e` |
| `run-0003` | gamma | `acme/graphify-core` | `f75e292` |

**Nothing needs to be running.** The system is a single Python 3.12 CLI with no dependencies
(`org/bin/acme`). It reads definitions straight from the bare origins and keeps run state in
`org/state/`, which is local and gitignored. No server or daemon is involved, so there is nothing
to start or stop.

## Layout

| Path | What it is |
|---|---|
| `<repo>/.acme/actions/mark-deprecated.json` (on each origin's `main`) | The action definition the repository owns: inputs and their rules, approvers, the file template, the commit message and how success is verified |
| `<repo>/.acme/README.md` | A note to the repository's team on how they own and change it |
| `org/bin/acme` | The org CLI and engine: discovery, validation, preview, request, approval, execution, observation, audit |
| `org/acme.json` | The org registry: the org slug, the people (ann, bob, carol) and the three repositories with their origins |
| `org/state/runs/run-NNNN.json` | One record per run: inputs, preview, requester, approver, events with timestamps, observation, outcome |
| `org/state/audit.jsonl` | An append-only, hash-chained log of every event across all runs, including denied approvals |
| `org/tests/test_acme.py` | End-to-end tests against throwaway copies of the origins (they never touch the real ones) |

`<repo>` is `/tmp/actions-ab/r6/vanilla-3/{alpha,beta,gamma}`, and its origin is
`/tmp/actions-ab/r6/vanilla-3/origins/<repo>.git`.

## How each requirement is met

| # | How | Where |
|---|---|---|
| R1 Owned by the repository | Each repository's definition is committed to its origin's `main` (alpha `d12f684`, beta `7d9b9ca`, gamma `45db566`). The CLI reads it from the origin's `main` with `git cat-file`, never from a checkout or from org config, so a commit to `main` changes the action (the approvers, input rules, template or commit message) for that repository only. Each run pins the definition commit it was requested against (`definitionCommit` in the record), and approval and execution use that same commit. `acme lint` checks every definition | `<repo>/.acme/actions/mark-deprecated.json`; `load_definition` and `lint_definition` in `org/bin/acme` |
| R2 Validated | `preview` and `request` validate inputs against the definition before anything is written: `successor` must match `^[a-z0-9-]+/[a-z0-9-]+$` and must not be the repository itself (`notSelf`); `reason` must be 20 to 200 characters; any input may not contain newlines or control characters; unknown and missing inputs are refused. A refusal exits 2 with `error.kind = "invalid_input"` and lists the errors per input. No run is recorded and the origin is untouched | `validate_inputs` in `org/bin/acme` |
| R3 Previewed | `acme preview` validates the inputs and shows the plan without changing anything: for each file, the operation (`create`, `modify` or `unchanged`), the before and after content and a unified diff, plus the base commit and the definition commit. `request` stores the same preview in the run, so the approver sees it with `acme record`. The approval records the plan's digest (`approvedBy.planDigest`) | `plan` in `org/bin/acme`; the `preview` field of each run record |
| R4 Approved | A request leaves the run in `awaiting_approval` and changes nothing. `approve` refuses (exit 3, `permission_denied`) when the approver is the requester, is not in the pinned definition's `approval.approvers` (carol is not), or is unknown. Every refusal is recorded as an `approval_denied` event. Only an allowed approval moves the run to `approved` and then executes it. A run is approved at most once, and a maintainer can `reject` it instead; the requester can `cancel` it | `cmd_approve` and `cmd_decline` in `org/bin/acme` |
| R5 Done means observed | After approval, the engine clones the origin, writes the rendered file and pushes to `main`. It then re-reads `main` on the origin itself and compares `DEPRECATED.md` byte for byte with the previewed content. The run becomes `succeeded` only if the push did not fail and the content matches. A rejected push (for example by a `pre-receive` hook) or an error makes it `failed`, with the git output in `outcome.detail`. So does a push that git accepted but that did not leave the content on `main`. A non-fast-forward because `main` moved is retried up to three times. `approve` exits 5 when the run fails | `execute` and `observe` in `org/bin/acme`; the `observation` field of each run record |
| R6 One place for the org | Everything runs from `/tmp/actions-ab/r6/vanilla-3/org`: `acme list` discovers the actions on every repository registered in `acme.json`, and `preview`, `request`, `approve` and so on take the repository name as an argument. You never go into a repository | `org/acme.json`; `org/bin/acme` |
| R7 Audited | Each run record has `requestedBy {person, at, comment}`, `approvedBy {person, at, comment, planDigest}`, `inputs`, `outcome {result, at, detail, commit, observedHead}`, `startedAt` and `finishedAt`. It also has an `events` list with a UTC timestamp and an actor on every step: `requested`, `approval_denied`, `approved`, `started`, `pushed` or `push_rejected`, `observed`, then `succeeded`, `failed`, `rejected` or `cancelled`. Every event is also appended to `org/state/audit.jsonl` in a hash chain, which `acme audit` verifies (`chainIntact`). On the origin, the commit is authored by the requester and committed by `acme-actions`, with `Acme-Run`, `Requested-by` and `Approved-by` trailers | `org/state/runs/*.json`, `org/state/audit.jsonl`; `Store` in `org/bin/acme` |
| R8 Usable by agents | Every command is non-interactive and prints exactly one JSON document on stdout. The JSON has `"ok": true` on success, or `"ok": false` with `error.kind` and `error.message` on failure. Identity comes from `--as <person>` or `ACME_ACTOR`. Inputs come from repeated `-i key=value` flags or from `--inputs-json '{...}'`. Exit codes: 0 ok, 2 invalid input or usage, 3 permission denied, 4 not found or wrong state, 5 run failed, 6 internal error | `main` in `org/bin/acme`; `bin/acme --help` |

## Commands

Start nothing. Run everything from the org directory:

```sh
cd /tmp/actions-ab/r6/vanilla-3/org
```

List the actions on all three repositories (`--repo alpha` narrows it):

```sh
bin/acme list --action mark-deprecated
bin/acme show alpha mark-deprecated        # one definition in full
bin/acme lint                              # check every definition
```

Preview. This validates the inputs and shows the diff without changing anything:

```sh
bin/acme preview alpha mark-deprecated \
  -i successor=acme/dependabot-proxy-v2 \
  -i "reason=Replaced by the v2 proxy, which supports OIDC registries."
```

Request as ann. This prints the run, including its `id` (`run-NNNN`) and the preview:

```sh
bin/acme request alpha mark-deprecated --as ann \
  -i successor=acme/dependabot-proxy-v2 \
  -i "reason=Replaced by the v2 proxy, which supports OIDC registries."
# or: --inputs-json '{"successor":"acme/dependabot-proxy-v2","reason":"..."}'
RUN=$(bin/acme request beta mark-deprecated --as ann -i successor=acme/x -i "reason=Some reason that is long enough." \
      | python3 -c 'import json,sys; print(json.load(sys.stdin)["id"])')
```

Approve as bob. This executes the run, observes the result and prints the final run:

```sh
bin/acme approve "$RUN" --as bob
```

Check the status, read the record and see the audit log:

```sh
bin/acme status "$RUN"         # state, terminal, succeeded, requester, approver, outcome
bin/acme record "$RUN"         # the full record: inputs, preview, approvals, events, observation, outcome
bin/acme runs                  # every run (--repo, --state filter)
bin/acme audit --run "$RUN"    # hash-chained events; chainIntact must be true
```

Other actions: `bin/acme reject "$RUN" --as bob --comment "..."` (a maintainer) and
`bin/acme cancel "$RUN" --as ann` (the requester).

## Testing the refusals

**Invalid inputs exit 2.** No run is created and the origin is unchanged:

```sh
bin/acme preview alpha mark-deprecated -i successor=Acme/Thing -i "reason=Long enough reason here."
bin/acme request alpha mark-deprecated --as ann -i successor=acme/x -i "reason=too short"
bin/acme request alpha mark-deprecated --as ann -i successor=acme/x -i "reason=$(printf 'x%.0s' $(seq 201))"
bin/acme request alpha mark-deprecated --as ann -i successor=acme/alpha -i "reason=Long enough reason here."
```

**Self-approval and carol exit 3.** The run stays `awaiting_approval` and the refusals appear in
the record as `approval_denied`:

```sh
RUN=$(bin/acme request gamma mark-deprecated --as ann -i successor=acme/x -i "reason=Testing self-approval refusal." \
      | python3 -c 'import json,sys; print(json.load(sys.stdin)["id"])')
bin/acme approve "$RUN" --as ann     # requester: refused
bin/acme approve "$RUN" --as carol   # not a maintainer: refused
bin/acme status "$RUN"               # still awaiting_approval
bin/acme reject "$RUN" --as bob      # clean up
```

**A rejected push exits 5, and the run is `failed`, never `succeeded`:**

```sh
printf '#!/bin/sh\necho "main is protected" >&2\nexit 1\n' > ../origins/alpha.git/hooks/pre-receive
chmod +x ../origins/alpha.git/hooks/pre-receive
RUN=$(bin/acme request alpha mark-deprecated --as ann -i successor=acme/other -i "reason=Testing a rejected push here." \
      | python3 -c 'import json,sys; print(json.load(sys.stdin)["id"])')
bin/acme approve "$RUN" --as bob     # exit 5, state "failed", outcome.detail has the hook's message
bin/acme status "$RUN"
rm ../origins/alpha.git/hooks/pre-receive
```

Use inputs that differ from what is already on `main`, as above. If `DEPRECATED.md` already has
exactly the requested content, the preview says `unchanged`, the run pushes nothing (its record
shows `no_change_needed`) and it succeeds, because the expected content is observed on `main`.

**The automated suite** covers all of the above, plus a hook that accepts the push but leaves
`main` unchanged, approver changes made by committing to a repository's definition, and the
reject and cancel paths. It runs on copies of the origins:

```sh
python3 tests/test_acme.py
```

## Limits

- Identity is asserted (`--as`), as the sandbox specifies. Approval rules are enforced against
  the asserted identity, and every action is recorded under it.
- Who may approve is whatever the repository's definition on `main` says. Changing it is a commit
  to `main`, so it is governed by the repository's own review rules, not by this system.
- The state in `org/state/` is local files guarded by a file lock. That is enough for one machine;
  a shared deployment would put it behind a service and a database.
