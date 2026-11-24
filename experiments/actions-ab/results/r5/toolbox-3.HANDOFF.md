# Handoff: `mark-deprecated` on Toolbox

`mark-deprecated` is declared by each repository in its own `.toolbox/toolbox.yaml`, offered from
one org portal (`org/toolbox.yaml`) served by `toolbox serve` on port 7490, and run there with
Toolbox's own validation, preview, approvals, commit, push and check on `main`. No glue code was
needed: the only files outside the Toolbox binaries are the manifests, their test files and
`org/toolbox.env`.

The three requested runs succeeded (ann requested, bob approved):

| Repository | Run | Commit on origin `main` |
|---|---|---|
| alpha | `run_2ft8ezkfli8m` | `47f70c05b324a35aa3c18bdd83eb256c48233e4c` |
| beta | `run_2ft8f6me9e9g` | `fc12f1c3f543ca49b08fdc1441cc8c4dfc7d5482` |
| gamma | `run_2ft8fd7fs6m0` | `f9cb13f76b5cb6d49aac79b109097a755274387e` |

A fourth run, `run_2ft8qwu65rrh` on alpha, is a rehearsal of a rejected push (a temporary
`pre-receive` hook on `origins/alpha.git`, since removed). It is recorded as `failed` with the
origin's message, and alpha's `main` did not move.

## Files

| Path | What it is |
|---|---|
| `alpha/.toolbox/toolbox.yaml` (and `beta/`, `gamma/`) | The repository's action: inputs schema, approvers, precondition, executor, pinned org key |
| `alpha/.toolbox/tests/mark-deprecated.test.yaml` (and `beta/`, `gamma/`) | 12 cases each: success, invalid inputs, self-approval, carol, rejected push |
| `org/toolbox.yaml` | The org portal: lists the three origins as federation children `portal:alpha`, `portal:beta`, `portal:gamma` |
| `org/toolbox.env` | `PATH`, `TOOLBOX_STATE_DIR`, `TOOLBOX_URL` for every command below |
| `/tmp/actions-ab/r5/toolbox-3/state/served/org/` | The server's signing key, pid, log (`serve.log`) and run records (`runs/<run id>.json`) |
| `/tmp/actions-ab/r5/toolbox-3/state/federation/` | The server's clones of the three origins, where runs execute |

## How each requirement is met

| # | How | Where |
|---|---|---|
| R1 Owned by the repository | Each repository's action is in its own `.toolbox/toolbox.yaml`, committed and pushed to its origin's `main`. The org portal holds no copy: it re-reads each child's `main` before it lists, previews or runs, so a change the team pushes takes effect on the next request. Each repository also pins which org key it accepts approvals from (`federation.acceptApprovalsFrom`) and that the org may run it (`federation.runner: {type: local}`) | `alpha/.toolbox/toolbox.yaml`, `beta/.toolbox/toolbox.yaml`, `gamma/.toolbox/toolbox.yaml` (commits `f8800ab`, `7a98c1d`, `3830e5b`) |
| R2 Validated | `inputs` is a JSON Schema: both required, `additionalProperties: false`, `successor` matches `^[a-z0-9-]+/[a-z0-9-]+$`, `reason` has `minLength: 20`, `maxLength: 200`. Toolbox checks it before any preview: an invalid request exits 2 with `422 invalid-inputs` and creates no run. A precondition also blocks a repository naming itself (`acme/<self>`) as successor | `inputs` and `preconditions` in each `.toolbox/toolbox.yaml`; cases in `.toolbox/tests/mark-deprecated.test.yaml` |
| R3 Previewed | A request without `--yes`/`--confirm` is a preview: it prints the exact diff of `DEPRECATED.md`, the push to `main`, the approvers, and a confirm token. Nothing changes. Every run record keeps its `effects` (the diff) and a `previewed` event. At run time Toolbox refuses a change that differs from the confirmed preview | `executor` in each `.toolbox/toolbox.yaml`; `effects` in `state/served/org/runs/<id>.json` |
| R4 Approved | `approval: "[['user:ann', 'user:bob']]"`: either maintainer, and the requester never counts. ann approving her own run gets `403 not-an-approver: a requester cannot approve their own run`; carol gets `403 not-an-approver: user:carol is not one of the pending approvers` (both exit 2, run stays `awaiting_approval`). The org signs approvals with its key and the repository accepts only that pinned key | `approval` in each `.toolbox/toolbox.yaml`; key in `state/served/org/signing-key` |
| R5 Done means observed | `executor: {type: change, land: push, base: main}` runs the `printf` in a fresh worktree of `main`, commits, pushes (never forced), then fetches `origin/main` and checks the commit is on it and each file's blob matches the preview. Only then is the run `succeeded` (`verification.child.checked`, `files[].matches`). A rejected push fails the run with the origin's message (exit 1) | `executor` in each `.toolbox/toolbox.yaml`; `verification` in each run record; rejected-push case in each test file; live `run_2ft8qwu65rrh` |
| R6 One place for the org | `org/toolbox.yaml` lists the three origins as `federation.children`. Served from `org`, one `toolbox run --url http://127.0.0.1:7490` lists, previews, requests, approves and checks `mark-deprecated` on `portal:alpha`, `portal:beta` and `portal:gamma`; `--runs` lists every run | `org/toolbox.yaml`, `org/toolbox.env` |
| R7 Audited | Each record has `requested_by`, `inputs`, `approval.approved_by` and `approval.satisfied`, `state`, `outputs.commit`, `verification`, and `events` with `at` timestamps and `by` for previewed, confirmed, awaiting_approval, approved (with the note), dispatched, running, executor_succeeded and succeeded or failed. Records survive a server restart. The commit on `main` carries `Toolbox-Run`, `Requested-By` and `Approved-By` trailers | `state/served/org/runs/<run id>.json`; `toolbox run --verify <id> -o json`; `git log` on each origin |
| R8 Usable by agents | Every step is a `toolbox run` command with flags only (no prompts), `-o json` output, and exit codes: 0 succeeded, 1 failed, 2 invalid or refused, 3 awaiting approval, 4 unverified. `TOOLBOX_URL` and `TOOLBOX_AS` can replace `--url` and `--as` | commands below |

## Commands

Run everything from any directory. Identity is `--as <name>`: the server listens on loopback, where
callers say who they are, and it records that name on every step.

**Set up the shell, then start the server** (returns once it answers; prints the approval key,
which must be `ed25519:e98d4fddf53ea7e3d8a83ffeb8626bd4b7b05dd0003878653193d6ac762f459b`, the key
the repositories pin; it is if `TOOLBOX_STATE_DIR` is the one in `toolbox.env`):

```sh
. /tmp/actions-ab/r5/toolbox-3/org/toolbox.env
toolbox serve /tmp/actions-ab/r5/toolbox-3/org --listen 127.0.0.1:7490 --detach
```

**List the actions** (all three repositories):

```sh
toolbox run --as ann -o json
```

**Preview** (shows the diff and a confirm token; changes nothing):

```sh
toolbox run --as ann mark-deprecated --entity portal:alpha \
  -i successor=acme/dependabot-proxy-v2 \
  -i reason="Replaced by the v2 proxy, which supports OIDC registries." -o json
```

**Request as ann** (exit 3, `awaiting_approval`; the run id is `.id` in the JSON). Either confirm
the preview you read, with the command it printed (`--confirm ct_…`, valid 10 minutes, once), or
preview and request in one step with `--yes`:

```sh
toolbox run --as ann mark-deprecated --entity portal:alpha \
  -i successor=acme/dependabot-proxy-v2 \
  -i reason="Replaced by the v2 proxy, which supports OIDC registries." --yes -o json
```

**Approve as bob** (`--wait` follows the run until it settles: exit 0 only once `main` has it):

```sh
toolbox run --as bob --approve <run-id> --note "checked" --wait -o json
```

**Check a run's status and read its record** (`.state`, `.requested_by`,
`.approval.approved_by`, `.inputs`, `.events[].at`, `.verification`):

```sh
toolbox run --as ann --verify <run-id> -o json
toolbox run --as ann --runs -o json            # every run, with state
```

The raw record is also at `/tmp/actions-ab/r5/toolbox-3/state/served/org/runs/<run-id>.json`.

**Stop the server:**

```sh
toolbox serve /tmp/actions-ab/r5/toolbox-3/org --stop
```

## Testing the refusals

| Test | Command | Expected |
|---|---|---|
| Bad successor | `toolbox run --as ann mark-deprecated --entity portal:beta -i successor=Acme/Store -i reason="State management moves to the shared store package." -o json` | exit 2, `422 invalid-inputs`, no run |
| Short or long reason | `... -i successor=acme/x -i reason=short` (or 201 characters) | exit 2, `422 invalid-inputs`, no run |
| Missing or extra input | omit `reason`, or add `-i extra=x` | exit 2, `422 invalid-inputs` |
| Self-approval | request as ann, then `toolbox run --as ann --approve <id> -o json` | exit 2, `403 not-an-approver: a requester cannot approve their own run` |
| carol approves | `toolbox run --as carol --approve <id> -o json` | exit 2, `403 not-an-approver` |
| Rejected push | `H=/tmp/actions-ab/r5/toolbox-3/origins/alpha.git/hooks/pre-receive; printf '#!/bin/sh\necho "main is protected" >&2\nexit 1\n' > $H && chmod +x $H`, then request (with a different reason) and approve; remove the hook afterwards | approve exits 1, run `failed` with the origin's message, `main` unchanged |
| The same cases offline | `cd /tmp/actions-ab/r5/toolbox-3/alpha && toolbox validate .toolbox && toolbox test .toolbox` (also `beta`, `gamma`) | 12 passed per repository, in a scratch copy; the real origin is untouched |

Notes for testers:

- The three repositories already have `DEPRECATED.md`. A new run with the same inputs succeeds
  without a commit (`outputs.changed: false`); use a different `reason` to see a new commit.
- To change the action, edit the repository's `.toolbox/toolbox.yaml`, run `toolbox test .toolbox`,
  then commit and push to its origin's `main`; the org portal picks it up on the next request.
- If the server starts with another state directory it prints another key, and the repositories
  refuse its approvals (`child-refuses-approvals`) until that key is pinned in their
  `federation.acceptApprovalsFrom`.
