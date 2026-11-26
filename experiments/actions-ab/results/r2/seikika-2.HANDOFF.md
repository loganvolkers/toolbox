# Handoff: `mark-deprecated` on Seikika

**Status:** `mark-deprecated` is declared in `alpha`, `beta` and `gamma`, offered from one org
portal in this directory, and has run on all three repositories. Each run was requested by ann,
approved by bob, and reported `succeeded` only after Seikika fetched the origin's `main` and found
`DEPRECATED.md` with the previewed content.

| Repository | Org run | Commit on `origins/<repo>.git` `main` | State |
|---|---|---|---|
| alpha | `run_2frfhubjihzi` | `52dde32` | succeeded |
| beta | `run_2frfi2pr6io4` | `72a3978` | succeeded |
| gamma | `run_2frfia5re0ua` | `570f1a3` | succeeded |

The server also holds earlier test runs, kept as evidence:

- `run_2frf6ayc6wms` (alpha): the origin's `pre-receive` hook rejected the push, so the run `failed`.
- `run_2frfdsnih5kr` (alpha): the push was accepted, then a `post-receive` hook reset `main`. The
  run `failed` with "the commit is not on origin/main".
- `run_2frfyixi0j8x` (beta): a re-run with identical inputs changed nothing, so it ended
  `unverified`, not `succeeded`.

In `run_2frf6ayc6wms`, carol's approval and ann's self-approval were both refused with `403
not-an-approver` before bob approved.

This is about the box: Seikika (in `/tmp/actions-ab/r2/seikika-2/bin`) carries, validates,
previews, approves, runs, verifies and records the action. The effect itself is one `printf` line
in each repository's manifest. The only glue written here is `scripts/serve.sh` (start the server
with a fixed state directory) and `scripts/run-status.sh` (exit 0 only on `succeeded`).

## Files

| Path | What it is |
|---|---|
| `/tmp/actions-ab/r2/seikika-2/{alpha,beta,gamma}/.seikika/seikika.yaml` | Each repository's own `mark-deprecated` definition, committed to its origin's `main` |
| `/tmp/actions-ab/r2/seikika-2/org/seikika.yaml` | The org portal: lists the three origins as federation children |
| `/tmp/actions-ab/r2/seikika-2/org/scripts/serve.sh` | Starts the org portal on `127.0.0.1:7450` with `SEIKIKA_STATE_DIR=/tmp/actions-ab/r2/seikika-2/state/seikika` |
| `/tmp/actions-ab/r2/seikika-2/org/scripts/run-status.sh` | Prints a run's record as JSON. Exit codes: 0 succeeded, 3 awaiting approval, 4 in progress, 1 anything else |
| `/tmp/actions-ab/r2/seikika-2/state/seikika/served/org/runs/<run id>.json` | The run records (written by the server, survive restarts) |
| `/tmp/actions-ab/r2/seikika-2/state/seikika/served/org/signing-key` | The key the org portal signs approvals with; its public half is pinned in each repository |
| `/tmp/actions-ab/r2/seikika-2/state/seikika/federation/` | The server's clones of the three origins |

## How each requirement is met

| # | How | Where |
|---|---|---|
| R1 Owned by the repository | The action (inputs, approvers, effect) is declared in each repository's `.seikika/seikika.yaml`, committed to `origins/<repo>.git` `main` (alpha `a86a1b4`, beta `624943d`, gamma `f27ef7f`). The org portal has no copy of it: it clones each origin's `main` and re-reads the manifest before every list, preview and run. A maintainer who edits and pushes that file changes the action. The org portal saw the actions appear after the push, without a restart. | `{alpha,beta,gamma}/.seikika/seikika.yaml` |
| R2 Validated | `inputs` is a JSON Schema: `successor` with `pattern: ^[a-z0-9-]+/[a-z0-9-]+$`, `reason` with `minLength: 20, maxLength: 200`, both required, no other inputs (`additionalProperties: false`). Seikika checks it before any preview, request or run, and answers `422 invalid-inputs`, exit 2. Nothing is recorded or changed. | same files, `inputs:` |
| R3 Previewed | A preview (no `--yes`/`--confirm`) runs the effect in a throwaway worktree and returns the exact diff of `DEPRECATED.md` and the push it would make. It changes nothing. A request (`--yes`) records the preview as the run's first event, and the run fails before committing if the change would differ from the preview. | `executor: {type: change, land: push}` in the repo manifests |
| R4 Approved | `approval: "[['user:ann', 'user:bob']]"` means one of ann or bob, and Seikika never counts the requester. The org server holds the approval. Carol gets `403 not-an-approver: user:carol is not one of the pending approvers`, and a self-approval gets `403 not-an-approver: a requester cannot approve their own run`, both with exit 2. The org portal signs bob's approval, and each repository accepts it only with the key pinned in its `federation.acceptApprovalsFrom`. | repo manifests `approval:` and `federation:` |
| R5 Done means observed | `land: push` commits in a fresh clone and pushes to `main` (never forced). Then Seikika fetches `origin/main` and checks that the commit is on it and that every changed file's blob matches the preview (`verification.files[].matches`). Only then is the run `succeeded`. A rejected push is `failed` with the origin's message. A push that does not stay on `main` is `failed` with "the commit is not on origin/main". A run that changes nothing is `unverified`. | run record `verification`, `outputs.landed` |
| R6 One place for the org | `org/seikika.yaml` lists the three origins under `federation.children`. Served, it offers `mark-deprecated` as `portal:alpha`, `portal:beta` and `portal:gamma`, and previews, requests, approves and runs them all through `http://127.0.0.1:7450`. It runs each in its own clone of that repository (`federation.runners.local`). | `org/seikika.yaml`, `org/scripts/serve.sh` |
| R7 Audited | Each run record has `requested_by`, `inputs`, `approval.approved_by`, `state`, `outputs.commit` and `verification`. `events[]` holds `previewed`, `confirmed`, `awaiting_approval`, `approved` (with `by` and the note), `running`, `succeeded`/`failed`, each with `at` (UTC). `child` holds the repository-side record, and `via` links the two. The commit on `main` carries `Seikika-Run`, `Requested-By` and `Approved-By` trailers. | `state/seikika/served/org/runs/*.json`, `seikika run --verify` |
| R8 Usable by agents | Every step is one non-interactive `seikika` command with `-o json`. Exit codes: 0 succeeded, 1 failed/stale, 2 invalid or refused, 3 awaiting approval. `--wait` follows a run until it settles, and `scripts/run-status.sh` turns "succeeded or not" into an exit code. The commands are in the next section. | `seikika run --help` |

## Commands

Run these from any directory. On a loopback server, `--as user:<id>` says who you are, and the
server records it (`--as ann` works too). Seikika prints a warning that it runs without OpenFGA:
anyone may list, preview and request, and only the action's `approval` rule limits approvals.

```sh
export PATH=/tmp/actions-ab/r2/seikika-2/bin:$PATH
U=http://127.0.0.1:7450
```

**Start** the org portal (leave it running in its own terminal, or in the background as shown;
stop it with Ctrl-C or `kill`):

```sh
nohup /tmp/actions-ab/r2/seikika-2/org/scripts/serve.sh > /tmp/actions-ab/r2/seikika-2/state/serve.log 2>&1 &
sleep 2; seikika run --url $U --as user:ann -o json | grep approval_key
```

The printed `approval_key` must be
`ed25519:301540d92f6ba98ffdcc5c9c94339327a65855137e5570497d7c2004b7bf52c5`, the key pinned in the
three repositories. It is, as long as `SEIKIKA_STATE_DIR` stays at the script's default. If you
start the server with a different state directory, it creates a new key, and the repositories
refuse its runs (`child-refuses-approvals`) until you pin the new key and push.

**List** `mark-deprecated` for all three repositories, with its input schema and approvers:

```sh
seikika run --url $U --as user:ann -o json
```

**Preview** (shows the diff, changes nothing):

```sh
seikika run --url $U --as user:ann mark-deprecated --entity portal:alpha \
  -i successor=acme/dependabot-proxy-v2 \
  -i "reason=Replaced by the v2 proxy, which supports OIDC registries." -o json
```

**Request as ann.** This prints the run record with its `id` and exits 3 (awaiting approval):

```sh
RUN=$(seikika run --url $U --as user:ann mark-deprecated --entity portal:alpha \
  -i successor=acme/dependabot-proxy-v2 \
  -i "reason=Replaced by the v2 proxy, which supports OIDC registries." \
  --yes -o json | python3 -c 'import json,sys; print(json.load(sys.stdin)["id"])')
echo $RUN
```

`--inputs '{"successor":"…","reason":"…"}'` passes both inputs as JSON instead.

**Approve as bob.** This runs the change and, with `--wait`, returns once it has settled. Exit 0
means succeeded, 1 means failed:

```sh
seikika run --url $U --as user:bob --approve $RUN --note "checked the diff" --wait -o json
```

To reject instead: `seikika run --url $U --as user:bob --reject $RUN --note "why" -o json`.

**Check a run's status** (exit 0 only for `succeeded`):

```sh
/tmp/actions-ab/r2/seikika-2/org/scripts/run-status.sh $RUN | python3 -c 'import json,sys; print(json.load(sys.stdin)["state"])'
```

**Read a run's record** (requester, approver, inputs, outcome, timestamps, observed commit):

```sh
seikika run --url $U --as user:ann --verify $RUN -o json
seikika run --url $U --as user:ann --runs -o json      # every run, all repositories
```

## Testing the refusals

All three repositories already have `DEPRECATED.md`. A request with the same inputs changes
nothing: its preview warns "the change command changed no files", and the run ends `unverified`,
not `succeeded`. To exercise a real change, use a different `reason` (the preview shows an
`update` diff).

| Test | Command | Expected |
|---|---|---|
| Invalid successor | `seikika run --url $U --as user:ann mark-deprecated --entity portal:beta -i successor=Acme/Store -i "reason=State management moves to the shared store package." -o json` | `422 invalid-inputs … does not match`, exit 2, no run recorded |
| Reason too short / too long | `-i reason=short` or a reason over 200 characters | `422 invalid-inputs … shorter than 20` / `longer than 200`, exit 2 |
| Missing or extra input | omit `-i successor=…`, or add `-i extra=1` | exit 2 |
| Self-approval | request as ann, then `seikika run --url $U --as user:ann --approve $RUN -o json` | `403 not-an-approver: a requester cannot approve their own run`, exit 2, run still `awaiting_approval` |
| Carol approves | `seikika run --url $U --as user:carol --approve $RUN -o json` | `403 not-an-approver: user:carol is not one of the pending approvers`, exit 2 |
| Rejected push | see below | run `failed` with the origin's message, exit 1, `main` unchanged |

Rejected push, on beta:

```sh
printf '#!/bin/sh\necho "pre-receive: main is frozen" >&2\nexit 1\n' > /tmp/actions-ab/r2/seikika-2/origins/beta.git/hooks/pre-receive
chmod +x /tmp/actions-ab/r2/seikika-2/origins/beta.git/hooks/pre-receive
RUN=$(seikika run --url $U --as user:ann mark-deprecated --entity portal:beta \
  -i successor=acme/valtio-store -i "reason=A different reason, so there is something to push." \
  --yes -o json | python3 -c 'import json,sys; print(json.load(sys.stdin)["id"])')
seikika run --url $U --as user:bob --approve $RUN --wait -o json; echo "exit $?"   # failed, exit 1
git -C /tmp/actions-ab/r2/seikika-2/origins/beta.git show main:DEPRECATED.md          # unchanged
rm /tmp/actions-ab/r2/seikika-2/origins/beta.git/hooks/pre-receive
```

## Changing the action (repository team)

Edit `<repo>/.seikika/seikika.yaml`, check it with
`seikika validate /tmp/actions-ab/r2/seikika-2/<repo>/.seikika`, then commit and push to the
origin's `main`. The org portal picks the change up on its next list, preview or run. Inside a
repository checkout, `seikika run` lists the action and `seikika run mark-deprecated -i … -o json`
previews it without the server. Approved runs need the server, because only it holds approvals.
