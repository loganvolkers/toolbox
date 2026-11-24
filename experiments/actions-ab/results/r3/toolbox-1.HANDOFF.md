# Handoff: `mark-deprecated`, a self-service action on Toolbox

`mark-deprecated` is defined in each of `alpha`, `beta` and `gamma`, and is offered, approved, run
and audited from one org portal in this directory. All three runs below were requested by ann,
approved by bob and reported `succeeded` after Toolbox had seen `DEPRECATED.md` on each origin's
`main`.

| Repository | Run id (org) | State | Commit on origin `main` |
|---|---|---|---|
| alpha | `run_2fs8km8vza9q` | succeeded | `abb9499` |
| beta | `run_2fs8lu7j4b8d` | succeeded | `2dbbacf` |
| gamma | `run_2fs8myzd8w1j` | succeeded | `141430f` |

Run `run_2fs8ekkz5u2f` (alpha, requested by carol, approved by ann) was a rejected-push test: a
`pre-receive` hook on `origins/alpha.git` refused the push, and the run is recorded as `failed`
with the origin's message. The hook was removed afterwards.

## How it fits together

| Piece | Path |
|---|---|
| Each repository's action definition (on its origin's `main`) | `/tmp/actions-ab/r3/toolbox-1/{alpha,beta,gamma}/.toolbox/toolbox.yaml` |
| Org portal: lists the three repositories by their origins | `/tmp/actions-ab/r3/toolbox-1/org/toolbox.yaml` |
| Environment for every command below | `/tmp/actions-ab/r3/toolbox-1/org/env.sh` |
| Server state: run records, clones of the repositories, approval signing key, log | `/tmp/actions-ab/r3/toolbox-1/state/served/org/` and `state/federation/` |

There is no glue code: everything is Toolbox 0.1.1 (`/tmp/actions-ab/r3/toolbox-1/bin/toolbox`).
The org server (`toolbox serve`) keeps a clone of each repository's `main`, reads its
`.toolbox/toolbox.yaml` again before it lists, previews or runs, holds the requests and approvals,
signs each approval, and runs the action in its clone. Each repository accepts approvals only when
signed with the key it pins (`federation.acceptApprovalsFrom`), which is the key in
`state/served/org/signing-key`. So always start the server with `TOOLBOX_STATE_DIR` set as
`env.sh` sets it; a different state directory makes a new key, and the repositories then refuse
its approvals.

## Requirements

| # | How it is met |
|---|---|
| R1 Owned by the repository | The action is declared in each repository's `.toolbox/toolbox.yaml` (for example `alpha/.toolbox/toolbox.yaml`), committed to its origin's `main` (alpha `af99914`, beta `f9bfb22`, gamma `edb18a2`). The org portal (`org/toolbox.yaml`) only lists the origins under `federation.children`; it holds no definition. The server re-reads each repository's `main` before every list, preview and run, so a commit by the repository's team changes what is offered. The record carries the definition's `action_version` hash |
| R2 Validated | `inputs` in each `.toolbox/toolbox.yaml` is a JSON Schema: `successor` with `pattern: ^[a-z0-9-]+/[a-z0-9-]+$`, `reason` with `minLength: 20` and `maxLength: 200`, both required, `additionalProperties: false`. Toolbox checks it before a preview, and the server refuses with `422 invalid-inputs` (exit 2) without creating a run. A precondition, `!inputs.successor.endsWith('/<repo>')`, also blocks naming the repository as its own successor (the preview lists it under `blockers`, and the run is refused) |
| R3 Previewed | A request without `--yes` or `--confirm` is a preview: it prints the diff of `DEPRECATED.md`, the push to `main`, the approvers and a confirm token, and changes nothing. A request with `--yes` records that preview on the run (`effects`), and what lands must match it |
| R4 Approved | `approval: "[['user:ann', 'user:bob']]"` in each `.toolbox/toolbox.yaml`: one of ann or bob, and the requester never counts. The server refuses `403 not-an-approver` (exit 2) for the requester approving their own run, and for carol. The approval is signed with the org key and checked by the repository against its pinned key before anything runs |
| R5 Done means observed | `executor: {type: change, land: push, base: main}`: Toolbox runs the `printf` in a fresh worktree of `main`, commits, pushes (never forced), then fetches `origin/main` and checks that the commit is on it and that `DEPRECATED.md` has the blob the preview showed (`verification.child.files[].matches` in the record). Only then is the run `succeeded`. A rejected push makes the run `failed` (exit 1) with the origin's message, as `run_2fs8ekkz5u2f` shows |
| R6 One place for the org | `org/toolbox.yaml` federates the three origins. From `/tmp/actions-ab/r3/toolbox-1/org`, `toolbox run` (with `TOOLBOX_URL` set by `env.sh`) lists `mark-deprecated` on `portal:alpha`, `portal:beta` and `portal:gamma`, and requests, approvals, status and records all go through that one server |
| R7 Audited | Each run's record (`toolbox run --verify <id> -o json`, stored as `state/served/org/runs/<id>.json`) has `requested_by`, `approval.approved_by`, `inputs`, `state`, `error` on failure, `verification`, and `events`: every step (previewed, confirmed, awaiting_approval, approved, dispatched, running, succeeded or failed) with `at` (UTC) and `by`. The commit on `main` also carries `Toolbox-Run`, `Requested-By` and `Approved-By` trailers. Records survive a server restart |
| R8 Usable by agents | Every step is one `toolbox` command with `-o json` and meaningful exit codes: 0 succeeded, 1 failed, 2 invalid or refused, 3 awaiting approval, 4 unverified. `--as` names the person (dev sign-in on the loopback address); `TOOLBOX_URL` and `TOOLBOX_AS` can replace `--url` and `--as` |

## Commands

Every block assumes this first:

```sh
cd /tmp/actions-ab/r3/toolbox-1/org
. ./env.sh          # PATH, TOOLBOX_STATE_DIR, XDG_STATE_HOME, TOOLBOX_URL=http://127.0.0.1:7710
```

**Start and stop the server** (nothing else needs to run):

```sh
toolbox serve . --listen 127.0.0.1:7710 --detach   # returns once it answers; prints the approval key
toolbox serve . --stop
```

The printed key must be `ed25519:c7c4689c3de4dd4a50f1989a4c7839ceff3afb2e5ab8ebfd65574bde05f7555b`,
the one the repositories pin.

**List the actions** (all three repositories, or one):

```sh
toolbox run --as ann -o json
toolbox actions portal:alpha --as ann -o json
```

**Preview** (changes nothing):

```sh
toolbox run --as ann mark-deprecated --entity portal:alpha \
  -i successor=acme/dependabot-proxy-v2 \
  -i "reason=Replaced by the v2 proxy, which supports OIDC registries." -o json
```

**Request as ann** (records the preview and the request; exit 3, waiting for an approval; the run
id is `.id`):

```sh
RUN=$(toolbox run --as ann mark-deprecated --entity portal:alpha \
  -i successor=acme/some-successor \
  -i "reason=A reason of twenty to two hundred characters." --yes -o json \
  | python3 -c 'import json,sys; print(json.load(sys.stdin)["id"])')
echo "$RUN"
```

**Approve as bob** (runs it; `--wait` follows it until it settles; exit 0 when succeeded):

```sh
toolbox run --as bob --approve "$RUN" --note "looks right" --wait -o json
```

To refuse a run instead: `toolbox run --as bob --reject "$RUN" --note "why" -o json`.

**Check a run's status and read its record**:

```sh
toolbox run --as ann --verify "$RUN" -o json     # .state, .requested_by, .approval.approved_by, .inputs, .events[].at/.by, .verification
toolbox run --as ann --verify "$RUN" --wait -o json   # keep checking until it settles
toolbox run --as ann --runs -o json              # every run, one place
```

### Negative tests

```sh
# Invalid inputs: 422 invalid-inputs, exit 2, no run created
toolbox run --as ann mark-deprecated --entity portal:beta -i successor=Acme/Bad -i reason=short --yes -o json
toolbox run --as ann mark-deprecated --entity portal:beta -i successor=acme/x -i "reason=$(printf 'a%.0s' $(seq 201))" --yes -o json
toolbox run --as ann mark-deprecated --entity portal:beta -i reason="Missing the successor input entirely." --yes -o json

# Self-approval and carol: 403 not-an-approver, exit 2, the run keeps waiting
toolbox run --as ann --approve "$RUN" -o json
toolbox run --as carol --approve "$RUN" -o json

# Rejected push: refuse every push to an origin, request, approve, expect state "failed" and exit 1
printf '#!/bin/sh\necho "push rejected by policy" >&2\nexit 1\n' > ../origins/beta.git/hooks/pre-receive
chmod +x ../origins/beta.git/hooks/pre-receive
RUN=$(toolbox run --as ann mark-deprecated --entity portal:beta -i successor=acme/other-store \
  -i "reason=Testing that a rejected push fails the run." --yes -o json \
  | python3 -c 'import json,sys; print(json.load(sys.stdin)["id"])')
toolbox run --as bob --approve "$RUN" --wait -o json; echo "exit=$?"   # exit=1, .state == "failed", .error has the origin's message
git -C ../origins/beta.git show main:DEPRECATED.md                     # unchanged
rm ../origins/beta.git/hooks/pre-receive
```

The three repositories already have `DEPRECATED.md`. Use inputs that differ from the current file
when you test a new run, otherwise the preview warns that the change command changed no files.

### Check the result on the origins directly

```sh
for r in alpha beta gamma; do git -C ../origins/$r.git show main:DEPRECATED.md; git -C ../origins/$r.git log -1 --format=%B main; done
```
