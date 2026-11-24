# Handoff: `mark-deprecated`, a self-service action on Toolbox

`mark-deprecated` is a Toolbox self-service action. Each of `alpha`, `beta` and `gamma` declares it in
its own `.toolbox/toolbox.yaml` on its origin's `main`. The org portal in this directory
(`org/toolbox.yaml`) lists the three repositories as federation children. Served on
`127.0.0.1:7790`, it offers, previews, holds approvals for, runs and records every run. I wrote no
glue code: validation, preview, approval, the push, observing the result on `main` and the audit
record are all Toolbox's own (`toolbox` 0.1.1, in `/tmp/actions-ab/r3/toolbox-3/bin`).

The three requested runs succeeded: ann requested each one and bob approved it.

| Repository | Org run (the record to read) | Commit on origin `main` |
|---|---|---|
| alpha | `run_2fs8qeqxykn4` | `0beb31f` docs: mark alpha deprecated in favour of acme/dependabot-proxy-v2 |
| beta | `run_2fs8rmzf8s6g` | `ba915a4` docs: mark beta deprecated in favour of acme/valtio-store |
| gamma | `run_2fs8sx49n01o` | `b3db8dd` docs: mark gamma deprecated in favour of acme/graphify-core |

Two earlier alpha runs, `run_2fs8h3r68z56` and `run_2fs8mp80qigb`, are `failed` on purpose. They
were my R5 tests. In the first, a `pre-receive` hook rejected the push. In the second, a
`post-receive` hook moved `main` back after accepting the push. I removed both hooks afterwards.
`run_2fs9jcuvc1d7` (beta) is `rejected`: bob rejected it to test `--reject`, so nothing changed.

## Files

| Path | What it is |
|---|---|
| `/tmp/actions-ab/r3/toolbox-3/{alpha,beta,gamma}/.toolbox/toolbox.yaml` | The action, owned by each repository: inputs schema, precondition, approvers, executor. It also pins the org portal's approval key |
| `/tmp/actions-ab/r3/toolbox-3/org/toolbox.yaml` | The org portal: the three origins as `federation.children` (`portal:alpha`, `portal:beta`, `portal:gamma`) |
| `/tmp/actions-ab/r3/toolbox-3/org/env.sh` | Sets `PATH`, `TOOLBOX_STATE_DIR` and `TOOLBOX_URL`. Source it first |
| `/tmp/actions-ab/r3/toolbox-3/state/served/org/runs/<run id>.json` | One audit record per run (not committed) |
| `/tmp/actions-ab/r3/toolbox-3/state/served/org/signing-key` | The key the server signs approvals with. Its public half is pinned in each repository |
| `/tmp/actions-ab/r3/toolbox-3/state/served/org/serve.log` | The server's log |
| `/tmp/actions-ab/r3/toolbox-3/state/federation/` | The server's clones of the three origins, re-read before every list, preview and run |

## How each requirement is met

| # | How |
|---|---|
| R1 Owned by the repository | The definition is `<repo>/.toolbox/toolbox.yaml`, committed to `origins/<repo>.git` `main` (alpha `a50f39b`, beta `8610884`, gamma `5e3d496`). The org portal holds no copy. It fetches each origin's `main` before listing, previewing or running, so a maintainer's edit on `main` (for example, to inputs, approvers or the command) takes effect on the next request. Each record stores the `action_version` hash of the definition it ran |
| R2 Validated | `inputs` in each `.toolbox/toolbox.yaml` is a JSON Schema: `successor` must match `^[a-z0-9-]+/[a-z0-9-]+$`, `reason` must be 20 to 200 characters, both are required, and `additionalProperties: false` refuses any other input. Toolbox checks these before the preview, so a bad input gets `422 invalid-inputs`, exit 2, and no run is created. A precondition (`!inputs.successor.endsWith('/<repo>')`) refuses a successor that is the repository itself: the preview lists it under `blockers` with no confirm token, and `--yes` exits 2 |
| R3 Previewed | A request without `--confirm`/`--yes` is a preview. It changes nothing and prints the exact diff of `DEPRECATED.md` (computed by running the command in a fresh worktree of `main`), the push to `main`, the approvers, any blockers and a `confirm_token`. Running with `--confirm <token>` only works while that preview still holds, and a token can be used once only (valid 10 minutes). If the change would differ from the confirmed preview, the run fails before it commits |
| R4 Approved | `approval: "[['user:ann', 'user:bob']]"` in each `.toolbox/toolbox.yaml` means one of ann or bob, and the requester never counts. The run waits in `awaiting_approval` (exit 3) until then. Self-approval gets `403 not-an-approver: a requester cannot approve their own run`. carol gets `403 not-an-approver: user:carol is not one of the pending approvers`. The org server signs the approval with its key, and the repository accepts only approvals signed with the key it pins in `federation.acceptApprovalsFrom` |
| R5 Done means observed | `executor: {type: change, land: push, base: main}`. Toolbox commits in a fresh clone, pushes (never forced), then fetches `origin/main`. It reports `succeeded` only if the commit is on `main` and every changed file's blob matches the preview (`verification.files[].matches`). A rejected push gives `failed` with the origin's message (`run_2fs8h3r68z56`). A push that was accepted but did not stay on `main` gives `failed: the commit is not on origin/main` (`run_2fs8mp80qigb`). Nothing is retried |
| R6 One place for the org | `org/toolbox.yaml` lists the three origins. From `org/`, `toolbox run` lists `mark-deprecated` on `portal:alpha`, `portal:beta` and `portal:gamma`. You request a run on any of them with `--entity portal:<repo>`, without entering the repository checkouts |
| R7 Audited | `toolbox run --verify <id> -o json` (also stored in `state/served/org/runs/<id>.json`) shows `requested_by`, `approval.approved_by`, `inputs`, `state`, `outputs.commit`, `verification`, and `events[]`. Each event has `state`, `by` and `at` (UTC) for previewed, confirmed, awaiting_approval, approved (with the note), running, succeeded or failed. The repository's own record is under `child`. The commit on `main` carries `Toolbox-Run`, `Requested-By` and `Approved-By` trailers |
| R8 Usable by agents | Every step is one non-interactive `toolbox run` command with `-o json`. Exit codes: 0 succeeded (or preview), 1 failed, 2 invalid or refused, 3 awaiting approval, 4 unverified. You say who you are with `--as` (or `TOOLBOX_AS`), which the server accepts because it listens on loopback |

## Commands

Run these from any shell. Every command below goes through the server via `TOOLBOX_URL`, which
`env.sh` sets.

```sh
cd /tmp/actions-ab/r3/toolbox-3/org
. ./env.sh                                  # PATH, TOOLBOX_STATE_DIR, TOOLBOX_URL=http://127.0.0.1:7790
```

**Start the server** (in the background. It returns once ready and prints the URL, pid, log and approval key):

```sh
toolbox serve /tmp/actions-ab/r3/toolbox-3/org --listen 127.0.0.1:7790 --detach
```

The printed `approval key` must be
`ed25519:1825b557cd5726b715203e5e569d036d37020ef31727ada06fbac4a1a879e3f0`, the key pinned in each
repository. The key is kept in `state/served/org/signing-key`. If that file is deleted, the server
makes a new key and the repositories refuse its approvals (`child-refuses-approvals`) until a
maintainer pins the new key in each `.toolbox/toolbox.yaml` and pushes it.

**Stop the server** (run records and the key stay on disk):

```sh
toolbox serve /tmp/actions-ab/r3/toolbox-3/org --stop
```

**List the actions** for all three repositories:

```sh
toolbox run --as ann -o json
```

**Preview** (changes nothing. Prints the diff, approvers, blockers and a `confirm_token`):

```sh
toolbox run --as ann mark-deprecated --entity portal:alpha \
  -i successor=acme/dependabot-proxy-v2 \
  -i 'reason=Replaced by the v2 proxy, which supports OIDC registries.' -o json
```

**Request as ann.** Confirm the token from the preview. This exits 3 with `"state": "awaiting_approval"`
and prints the run's `id`:

```sh
toolbox run --as ann mark-deprecated --entity portal:alpha \
  -i successor=acme/dependabot-proxy-v2 \
  -i 'reason=Replaced by the v2 proxy, which supports OIDC registries.' \
  --confirm <confirm_token> -o json
```

To preview and request in one step (the preview is still recorded), use `--yes` instead of `--confirm <token>`.
Inputs can also be given as one JSON object: `--inputs '{"successor":"…","reason":"…"}'`.

**Approve as bob.** This runs it, and `--wait` follows it to `succeeded` (exit 0) or `failed` (exit 1):

```sh
toolbox run --as bob --approve <run id> --note "checked" --wait -o json
```

To reject instead: `toolbox run --as bob --reject <run id> --note "why" -o json` (state `rejected`, exit 1).

**Check a run's status and read its record:**

```sh
toolbox run --as ann --verify <run id> -o json          # full record; .state is the status
toolbox run --as ann --verify <run id> --wait -o json   # follow it until it settles
toolbox run --as ann --runs -o json                     # every run, all repositories
```

### Negative tests and what to expect

| Test | Command (after `. ./env.sh` in `org/`) | Result |
|---|---|---|
| Bad successor | `toolbox run --as ann mark-deprecated --entity portal:beta -i successor=Acme/Store -i 'reason=State management moves to the shared store package.' --yes -o json` | `422 invalid-inputs … does not match`, exit 2, no run |
| Short or long reason | `… -i successor=acme/x -i 'reason=too short' --yes -o json` | `422 invalid-inputs … shorter than 20 characters`, exit 2 |
| Successor is itself | `… --entity portal:beta -i successor=acme/beta …` | preview `blockers: precondition not met`, no token. `--yes` exits 2 |
| Self-approval | request as ann, then `toolbox run --as ann --approve <id> -o json` | `403 not-an-approver: a requester cannot approve their own run`, exit 2 |
| carol approves | `toolbox run --as carol --approve <id> -o json` | `403 not-an-approver: user:carol is not one of the pending approvers`, exit 2 |
| Rejected push | `H=/tmp/actions-ab/r3/toolbox-3/origins/beta.git/hooks/pre-receive; printf '#!/bin/sh\necho rejected >&2\nexit 1\n' > $H && chmod +x $H`, then request as ann and approve as bob with `--wait` | `"state": "failed"`, the last event quotes the origin's refusal, exit 1. `main` is unchanged. Remove the hook afterwards |

All three repositories are already deprecated. A new request with the same inputs previews no
change (`warnings: the change command changed no files`). A request with a different successor or
reason previews a `modify` diff of `DEPRECATED.md`, so use different inputs for a run that changes
something.

### Without a server

`toolbox` without `TOOLBOX_URL` works on the checkouts. Running `env -u TOOLBOX_URL toolbox run -o json` in
`org/` or in a repository lists and previews the same action. A run that needs approval stops at
exit 3 there, because only the served org portal keeps approvals. Validate the manifests with
`toolbox validate /tmp/actions-ab/r3/toolbox-3/org` and `toolbox validate /tmp/actions-ab/r3/toolbox-3/<repo>/.toolbox`.
