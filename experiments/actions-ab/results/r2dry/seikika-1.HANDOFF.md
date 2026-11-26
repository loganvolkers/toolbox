# Handoff: `mark-deprecated`

`mark-deprecated` is a Seikika self-service action. Each repository declares its own copy on its
origin's `main`. An org portal in this directory lists those copies and lets you preview, request,
approve, run and audit a run on any repository from here. All three runs succeeded. They were
requested by ann and approved by bob:

| Repository | Run | State | Commit on origin `main` |
|---|---|---|---|
| alpha | `run_2fqyrbpxnthf` | succeeded | `8b2be71` |
| beta | `run_2fqzdhj8u96g` | succeeded | `1fd882b` |
| gamma | `run_2fqzdp692liq` | succeeded | `3b7299c` |

The record also holds `run_2fqzivyvcg18`, a test run on beta. It **failed** because a temporary
`pre-receive` hook rejected the push, and beta's `main` was left untouched. The hook has since been
removed.

The alpha commit's author is `Cursor Agent`, because the host's global git identity was in effect
for that first run. Later commits are authored by `Seikika org portal`, which `org/bin/serve` now
sets. Every commit carries `Seikika-Run` and `Requested-By` trailers.

Root below is `/tmp/actions-ab/r2dry/seikika-1`, written `$ROOT`.

## Files

| Path | What it is |
|---|---|
| `$ROOT/alpha/.seikika/seikika.yaml` (same in `beta`, `gamma`) | The repository's own definition of `mark-deprecated`: inputs, approval rule, precondition, executor. Committed to each origin's `main` |
| `$ROOT/org/seikika.yaml` | The org portal: lists the three origins as federation children |
| `$ROOT/org/bin/serve` | Starts the org portal server on `127.0.0.1:7110`, with a fixed state directory and a git identity for the commits it lands |
| `$ROOT/org/bin/check-deprecated` | Optional independent check: compares a run's record with `DEPRECATED.md` on the origin's `main` |
| `$ROOT/state/served/org/runs/*.json` | The run records (the audit trail), written by the server |
| `$ROOT/state/served/org/signing-key` | The key the org portal signs approvals with. Its public half is pinned in each repository's `.seikika/seikika.yaml` |
| `$ROOT/state/federation/` | The server's clones of the three origins (re-fetched before each list, preview and run) |

Keep `$ROOT/state`. If it is deleted, the server makes a new signing key, and every repository will
refuse its approvals until the new key (printed by `seikika run --url … --as ann`) is pinned in each
`.seikika/seikika.yaml` and pushed.

## Requirements

| # | How it is met |
|---|---|
| R1 Owned by the repository | The action is defined in `<repo>/.seikika/seikika.yaml`, committed to `origins/<repo>.git` `main` in commits `d69091a` (alpha), `e5150cc` (beta) and `1fbb457` (gamma). The org portal only lists the origins (`org/seikika.yaml`). The server re-fetches each origin before it lists, previews or runs, so a change the team pushes to `main` takes effect with no change in `org/`. The approver list (`[['user:ann', 'user:bob']]`) also lives in that file |
| R2 Validated | `inputs` in each `.seikika/seikika.yaml` is a JSON Schema: `successor` must match `^[a-z0-9-]+/[a-z0-9-]+$`, `reason` must be 20 to 200 characters, both are required, and nothing else is accepted. Seikika checks it before any preview: `422 invalid-inputs`, exit 2, and no run is created. A precondition (`!inputs.successor.endsWith('/<repo>')`) refuses naming the repository as its own successor (exit 2, blocker listed, no run created) |
| R3 Previewed | A request without `--yes` or `--confirm` is a preview. It prints the exact diff of `DEPRECATED.md`, the push to `main`, the approvers, any blockers and a confirm token. The preview runs in a scratch worktree, so nothing changes |
| R4 Approved | `approval: "[['user:ann', 'user:bob']]"` means one of ann or bob, and never the requester. The server refuses the requester (`403 not-an-approver: a requester cannot approve their own run`) and carol (`403 not-an-approver: user:carol is not one of the pending approvers`). Both exit 2, and the run stays `awaiting_approval`. The org portal signs each approval with `state/served/org/signing-key`. Each repository accepts only approvals signed with the key pinned in its `federation.acceptApprovalsFrom`. Running the action in a repository checkout without the server stops at exit 3, because a checkout holds no approvals |
| R5 Done means observed | The executor is `type: change, land: push, base: main`. After approval it writes the file in a fresh worktree, commits, and pushes to the origin's `main` without forcing. The run is `succeeded` only after it sees that commit on `origin/main` (`verification.by: base`, with the commit id in the record). Git commits are addressed by content, so that commit fixes `DEPRECATED.md`'s exact bytes, which were shown in the preview. If the origin refuses the push, the run is `failed`, and the record quotes the origin's message (tested: `run_2fqzivyvcg18`). Nothing is retried. `org/bin/check-deprecated <run>` independently re-reads `DEPRECATED.md` on `main` and compares it with the inputs |
| R6 One place for the org | `org/seikika.yaml` federates the three origins. From `$ROOT/org`, `seikika run` lists all three `mark-deprecated` actions (`portal:alpha`, `portal:beta`, `portal:gamma`). With the server running, `seikika run --url http://127.0.0.1:7110 …` lists, previews, requests, approves and audits runs on any of them |
| R7 Audited | Each record (`seikika run --url … --verify <run>`, stored in `state/served/org/runs/<run>.json`) contains `requested_by`, `inputs`, `approval.approved_by`, `state`, `outputs.commit`, `verification`, `error` (when failed), and `events`. `events` is a timestamped list: `previewed` and `confirmed` by the requester, `awaiting_approval`, `approved` by the approver with their note, `dispatched`, `running`, `executor_succeeded`, then `succeeded` or `failed`. The child repository's own run is embedded under `child`. The landed commit carries `Seikika-Run` and `Requested-By` trailers |
| R8 Usable by agents | Every step is a non-interactive `seikika` command with `-o json`. Exit codes are 0 (succeeded), 1 (failed or timed out), 2 (invalid, refused or not allowed) and 3 (waiting for approval). Identity is `--as <user>` (dev sign-in on a loopback address). `check-deprecated` prints JSON too |

## Commands

```sh
cd /tmp/actions-ab/r2dry/seikika-1
export PATH=$PWD/bin:$PATH
U=http://127.0.0.1:7110
```

**Start the server** (it must be running for requests, approvals and records). Stop it with Ctrl-C
or `kill`:

```sh
nohup org/bin/serve > state/serve.log 2>&1 &
sleep 2
```

**List the actions** (and the approval key the repositories pin):

```sh
seikika run --url $U --as ann -o json
# without a server, from the org checkout (list and preview only):
(cd org && SEIKIKA_STATE_DIR=$PWD/../state seikika run -o json)
```

**Preview** (changes nothing; prints the diff and the approvers):

```sh
seikika run --url $U --as ann mark-deprecated --entity portal:alpha \
  -i successor=acme/dependabot-proxy-v2 \
  -i 'reason=Replaced by the v2 proxy, which supports OIDC registries.' -o json
```

**Request as ann.** This exits 3 (awaiting approval) and prints the run's `id`:

```sh
seikika run --url $U --as ann mark-deprecated --entity portal:alpha \
  -i successor=acme/dependabot-proxy-v2 \
  -i 'reason=Replaced by the v2 proxy, which supports OIDC registries.' --yes -o json
RUN=$(seikika run --url $U --as ann mark-deprecated --entity portal:beta \
  -i successor=acme/valtio-store -i 'reason=A different reason, for a retest run.' --yes -o json \
  | python3 -c 'import json,sys; print(json.load(sys.stdin)["id"])')
```

Instead of `--yes`, you can confirm the token from a preview with `--confirm <token>` (valid for 10
minutes, usable once).

**Approve as bob.** This runs the action and waits for the outcome: exit 0 if succeeded, 1 if
failed:

```sh
seikika run --url $U --as bob --approve $RUN --note "checked the preview" --wait -o json
```

Bob can turn a run down with `seikika run --url $U --as bob --reject $RUN --note "why" -o json`.

**Check the status and read the record:**

```sh
seikika run --url $U --as ann --verify $RUN -o json   # full record: state, requested_by, approved_by, inputs, events with times, commit
seikika run --url $U --as ann --runs -o json          # every run, all repositories
org/bin/check-deprecated $RUN                         # optional: re-read DEPRECATED.md on origin main and compare it with the inputs
```

## Negative tests

| Test | Command | Expected |
|---|---|---|
| Invalid inputs | `seikika run --url $U --as ann mark-deprecated --entity portal:alpha -i successor=Acme/x_y -i reason=short --yes -o json` | `422 invalid-inputs …`, exit 2, no run |
| Own successor | `… --entity portal:alpha -i successor=acme/alpha -i 'reason=twenty characters or more' --yes -o json` | blocker `precondition not met`, exit 2, no run |
| Self-approval | `seikika run --url $U --as ann --approve $RUN -o json` (where ann requested `$RUN`) | `403 not-an-approver: a requester cannot approve their own run`, exit 2 |
| Carol approves | `seikika run --url $U --as carol --approve $RUN -o json` | `403 not-an-approver: user:carol is not one of the pending approvers`, exit 2 |
| Rejected push | `printf '#!/bin/sh\necho rejected >&2\nexit 1\n' > origins/beta.git/hooks/pre-receive; chmod +x origins/beta.git/hooks/pre-receive`, then request as ann and approve as bob | the approve command exits 1. `--verify` shows `state: failed`, and its `error` and last event quote the origin's rejection. `origins/beta.git` `main` is unchanged. Remove the hook afterwards |

All three repositories already have `DEPRECATED.md`. A retest with the same inputs changes no files:
its preview warns `the change command changed no files`. Use a different `reason` or `successor`, so
the run has something to land.
