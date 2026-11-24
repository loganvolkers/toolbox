# HANDOFF: `mark-deprecated` self-service action

`mark-deprecated` has been run on alpha, beta and gamma. In each case ann requested it, bob
approved it, and the run was reported `succeeded` only after `DEPRECATED.md` was read back from
the origin's `main` (runs `run-0001`, `run-0002` and `run-0003`).

The system is one Python 3 script, `org/bin/acme-actions`, which uses only the standard library
and `git`. **Nothing needs to be running**: there is no server or daemon. Every command prints
one JSON document on stdout and exits with a meaningful code.

| Exit | Meaning |
|---|---|
| 0 | ok |
| 1 | the run executed and **failed** (push rejected, not observed, preview stale) |
| 2 | invalid inputs or usage; nothing changed |
| 3 | forbidden (self-approval, not a maintainer, unknown person) |
| 4 | not found (repository, action, run) |
| 5 | conflict (the run is not awaiting approval) |
| 6 | broken action definition or config |

## Files

| Path | What it is |
|---|---|
| `<repo>/.acme/actions/mark-deprecated.json` (alpha, beta, gamma), on each origin's `main` | The action definition, owned by the repository: maintainers, input rules and the file template |
| `org/bin/acme-actions` | The engine and CLI: list, show, preview, request, approve, reject, cancel, status, record, runs, verify |
| `org/acme.json` | The org registry: org slug `acme`, the three repositories (origin paths relative to `org/`), and the people (ann, bob, carol) |
| `org/state/runs/run-NNNN.json` | One record per run, including its events. Not committed (`org/.gitignore`) |
| `org/state/audit.jsonl` | An append-only log of every event across all runs |

## How each requirement is met

| # | How |
|---|---|
| R1 Owned by the repository | Each repository's definition is `.acme/actions/mark-deprecated.json`, committed to its origin's `main`. The engine reads definitions **from the origin's `main`** (`git cat-file` on `origins/<repo>.git`), never from the org or a checkout. The definition declares `maintainers` (who may approve), the inputs and their rules, the commit message and the `DEPRECATED.md` template. A team changes its action by committing to that file. Each run pins the definition commit and blob it was requested under (`definition` in the record), and approval and execution use that pinned version. The engine refuses a malformed definition (exit 6), and refuses one that tries to enable self-approval. |
| R2 Validated | `preview` and `request` validate inputs against the definition before anything is written (`validate_inputs` in `org/bin/acme-actions`). `successor` must fully match `^[a-z0-9-]+/[a-z0-9-]+$` (`re.fullmatch`, so a trailing newline fails) and must not be this repository (`acme/<repo>`). `reason` must be 20 to 200 characters. Unknown or missing inputs are refused. A refused input gives exit 2 with every error listed, no run is created and no git write happens. Inputs are validated again at execution. |
| R3 Previewed | `preview` shows the exact file content, a unified diff against the origin's current `main`, the operation (`create`, `modify` or `unchanged`), the commit message and the approvers. It changes nothing. `request` returns the same preview and stores it in the run. At execution the engine re-plans against the current `main`. If the target file or its rendered content differs from what was previewed, the run fails with `preview_stale` and nothing is pushed, so an approval only ever applies the change that was previewed. |
| R4 Approved | A run starts as `awaiting_approval`, and nothing touches git until `approve`. `approve` refuses (exit 3, and logs an `approval_denied` event) if the approver is the requester (`self_approval`), is not in the pinned definition's `maintainers` (`not_a_maintainer`, which is carol's case), or is not in `org/acme.json` (`unknown_person`). A run can be approved only once: approving again gives exit 5. State changes take a file lock (`org/state/.lock`), so two concurrent approvals cannot both execute. Identity comes from `--as NAME` or `ACME_USER`. |
| R5 Done means observed | After approval, the engine makes a temporary bare clone, builds the commit with git plumbing and pushes it to the origin with a plain, non-force `git push origin <commit>:refs/heads/main`, so the origin's hooks apply. It then **reads the origin's `main` directly** (`observe()`) and compares the sha256 of `DEPRECATED.md` with the expected content. The run is `succeeded` only if they match. A rejected push gives `failed` with `push_rejected`. A push that was accepted but whose result is absent from `main` (for example, reverted by a hook) gives `failed` with `not_observed`. An internal error gives `failed`, never `succeeded`. The observation (head commit, sha256 values, time) is stored in `outcome.observed`. `verify <run>` re-observes later, and recovers a run left `running` by a crash. |
| R6 One place for the org | From `org/`, `bin/acme-actions list` discovers the actions on every repository registered in `org/acme.json` by reading `.acme/actions/*.json` from each origin's `main`. `request`, `approve` and the other commands take the repository as an argument, so nobody needs to enter a repository. |
| R7 Audited | `record <run>` returns the full record: `requestedBy` and `requestedAt`, `approvedBy` and `approvedAt` (or `rejectedBy` and `rejectedAt`), `inputs`, the pinned `definition`, the `preview`, and `outcome` (`status`, `at`, `reason`, the pushed `commit`, the observation). It also returns an `events` list (`requested`, `approval_denied`, `approved`, `executing`, `push` with git's output and exit code, `succeeded` or `failed`), each event with an `at` timestamp (UTC, in milliseconds) and an `actor`. Every event is also appended to `org/state/audit.jsonl`. The commit on `main` carries the trailers `Acme-Run`, `Requested-by` and `Approved-by`, and its author is the requester. |
| R8 Usable by agents | Every step is one non-interactive command with JSON on stdout and the exit codes above. Inputs are given as `-i NAME=VALUE` or `--inputs-json '{...}'`, and identity as `--as` or `ACME_USER`. `bin/acme-actions --help` and `bin/acme-actions <cmd> --help` describe the commands. |

## Commands

Run these from the org directory. Nothing needs to be started first.

```sh
cd /tmp/actions-ab/r3/vanilla-1/org
A=bin/acme-actions
```

List the actions on all repositories, or show one definition:

```sh
$A list
$A list --repo beta
$A show alpha mark-deprecated
```

Preview a run. This validates the inputs and shows the change without writing anything:

```sh
$A preview alpha mark-deprecated -i successor=acme/dependabot-proxy-v3 -i "reason=Replaced by the v3 proxy with better caching."
```

Request a run as ann. The run id is in `.run.id`, for example `run-0004`:

```sh
$A request alpha mark-deprecated --as ann -i successor=acme/dependabot-proxy-v3 -i "reason=Replaced by the v3 proxy with better caching."
# equivalently:
ACME_USER=ann $A request alpha mark-deprecated --inputs-json '{"successor":"acme/dependabot-proxy-v3","reason":"Replaced by the v3 proxy with better caching."}'
RUN=$($A request alpha mark-deprecated --as ann -i successor=acme/x-y -i "reason=Twenty or more characters here." | python3 -c 'import json,sys;print(json.load(sys.stdin)["run"]["id"])')
```

Approve as bob. The run executes and is verified in the same command. Exit 0 means `succeeded`
and exit 1 means `failed`:

```sh
$A approve run-0004 --as bob --comment "ok"
```

Check a run's status, read its full record, or list runs:

```sh
$A status run-0004          # .run.status and .terminal
$A record run-0004          # the full audit record with .run.events
$A runs                     # all runs; filter with --repo alpha or --status failed
```

Other commands:

```sh
$A reject run-0004 --as bob --comment "not yet"   # a maintainer declines
$A cancel run-0004 --as ann                        # the requester withdraws
$A verify run-0004 --as bob                        # re-read main for this run
```

## Testing the guarantees

| Test | Command | Expected |
|---|---|---|
| Invalid successor | `$A preview beta mark-deprecated -i successor=Acme/Store -i "reason=Long enough reason text here."` | exit 2, `invalid_inputs` |
| Successor is the repository itself | `... -i successor=acme/beta ...` | exit 2 |
| Reason too short or too long | `-i reason=short`, or a reason of 201 or more characters | exit 2; `request` creates no run |
| Self-approval | `$A request ... --as ann`, then `$A approve <run> --as ann` | exit 3, `self_approval`; the run stays `awaiting_approval` |
| carol approves | `$A approve <run> --as carol` | exit 3, `not_a_maintainer` |
| Rejected push | `printf '#!/bin/sh\necho denied >&2\nexit 1\n' > ../origins/gamma.git/hooks/pre-receive; chmod +x ../origins/gamma.git/hooks/pre-receive`, then request and approve a run that changes the file | exit 1, `status: failed`, `outcome.reason` starts with `push_rejected`, and `main` is unchanged. Remove the hook afterwards. |

**For the rejected-push test, use inputs that change the file.** All three repositories already
hold `DEPRECATED.md`. A run whose inputs render exactly the current content previews as
`unchanged`. It pushes nothing (a `no_change_needed` event), observes that the right content is
already on `main`, and reports `succeeded`, because the desired state is genuinely observed. Use
a different reason or successor, and the run must push, so a rejected push makes it fail.

## Limits

- Identity is asserted rather than authenticated, as the sandbox requires. The rules about who
  may approve are enforced; who you are is taken from `--as` or `ACME_USER`.
- Run state lives in `org/state/`, on this machine, and is not committed.
- Approval executes synchronously. A failed run is final: fix the cause, then request a new run.
