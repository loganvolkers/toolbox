# Handoff: `mark-deprecated`, a self-service action owned by each repository

All three runs succeeded. ann requested each one and bob approved it, and Seikika saw the commit with
the right `DEPRECATED.md` on each origin's `main`:

| Repository | Org run | Commit on `main` | State |
|---|---|---|---|
| alpha | `run_2fsstldmtmtq` | `1ccb31c` | succeeded |
| beta | `run_2fst04vhrxmq` | `9d83038` | succeeded |
| gamma | `run_2fst19th4b1u` | `25465ea` | succeeded |

There is a fourth run, `run_2fst5wu29bhd` on alpha. It is `failed` on purpose: I added a
pre-receive hook that rejects pushes to `origins/alpha.git`, then removed it. The run shows how a
rejected push is reported (see "Test a rejected push" below).

The system is Seikika 0.1.1 (`/tmp/actions-ab/r4/seikika-2/bin`). There is no glue code: Seikika
validates the inputs, previews the change, holds and signs approvals, makes the commit, pushes it,
checks it on `main` and records the run. The only things written for this task are the YAML
manifests and test files below.

## Files

| Path | What it is |
|---|---|
| `<repo>/.seikika/seikika.yaml` (in `alpha`, `beta` and `gamma`, on each origin's `main`) | The repository's own definition of `mark-deprecated`: input schema, approvers, precondition, the `change` executor (`land: push`), and the org key it accepts approvals from |
| `<repo>/.seikika/tests/mark-deprecated.test.yaml` | 9 cases run by `seikika test`: success, 4 kinds of invalid input, self-successor, self-approval plus carol, carol as requester, rejected push |
| `org/seikika.yaml` | The org portal: lists the three origins as federated children (`portal:alpha`, `portal:beta`, `portal:gamma`) |
| `/tmp/actions-ab/r4/seikika-2/state/` (`SEIKIKA_STATE_DIR`, not in git) | Server state: `served/org/runs/<run id>.json` (org run records), `served/org/signing-key` (the approval key), `served/org/serve.log`, `runs/` (child run records), `federation/` (the org's clones of the repositories) |

## R1 to R8

| # | How it is met | Where |
|---|---|---|
| R1 Owned by the repository | Each repository declares the action in its own `.seikika/seikika.yaml`, committed to its origin's `main` (commits `273b7f7`, `4251fe8`, `9466ee8`). The org portal only lists the origins. It re-reads each repository's `main` before it lists, previews or runs, so a change the team lands is used on the next request. The commands are owned by `group:<repo>-maintainers` and approved by ann or bob | `alpha/.seikika/seikika.yaml`, `beta/.seikika/seikika.yaml`, `gamma/.seikika/seikika.yaml`; `org/seikika.yaml` |
| R2 Validated | `inputs` is a JSON Schema: both inputs are `required`, `additionalProperties: false`, `successor` must match `^[a-z0-9-]+/[a-z0-9-]+$`, and `reason` must be 20 to 200 characters long (`minLength`/`maxLength`). Seikika checks this before any preview: `422 invalid-inputs`, exit 2, nothing is recorded and nothing changes. There is also a precondition, `!inputs.successor.endsWith('/<repo>')`, so a repository can't name itself as its successor. That run is `Blocked` and can't be confirmed | `inputs:` and `preconditions:` in each `.seikika/seikika.yaml`; the test cases |
| R3 Previewed | `seikika run mark-deprecated …` without `--yes` or `--confirm` is a dry run. It renders the change in a scratch worktree of `main` and prints the exact diff of `DEPRECATED.md`, the push to `main`, the approvers and any blockers. It also prints a confirm token that is good for 10 minutes and can be spent once. At run time, a change that differs from the confirmed preview fails before it commits | `executor:` (`type: change`) in each manifest |
| R4 Approved | `approval: "[['user:ann', 'user:bob']]"` means one of ann or bob. Seikika never counts the requester: ann approving her own run gets `403 not-an-approver: a requester cannot approve their own run`, and carol gets `403 … user:carol is not one of the pending approvers`, both exit 2. The org server signs each approval with its ed25519 key. Each repository runs only on approvals signed by the key pinned in its `federation.acceptApprovalsFrom` | `approval:` and `federation.acceptApprovalsFrom` in each manifest; `state/served/org/signing-key` |
| R5 Done means observed | With `land: push`, Seikika commits in a fresh clone and pushes to `main` without force. It then fetches `origin/main` and checks that the commit is on it and that every file has the previewed blob. Only then is the run `succeeded`, and the record holds the commit and blob in `verification`. If the push is rejected, the run is `failed` with the origin's message and nothing is retried | `executor.land: push` in each manifest; a run record's `verification` and `outputs` |
| R6 One place for the org | `org/seikika.yaml` federates the three origins. Served on `127.0.0.1:7950`, `seikika run` lists all three actions, and `--entity portal:<repo>` previews, requests, approves and checks any of them. You don't need to enter any repository | `org/seikika.yaml` |
| R7 Audited | Each run record has `requested_by`, `approval.approved_by` / `approval.satisfied`, `inputs`, `state`, `outputs.commit`, `verification`, and `events`. `events` is timestamped and records who did each step: `previewed`, `confirmed`, `awaiting_approval`, `approved` (with bob's note), `dispatched`, `running`, `succeeded` or `failed` (with the error). The commit carries `Seikika-Run`, `Requested-By` and `Approved-By` trailers | `state/served/org/runs/<run id>.json` (org), `state/runs/<child run id>.json` (repository); `seikika run --verify <id> -o json` |
| R8 Usable by agents | Every step is one non-interactive `seikika run` command, with `-o json` for JSON output and meaningful exit codes: 0 succeeded, 1 failed, 2 invalid or refused, 3 awaiting approval, 4 unverified. `SEIKIKA_URL` and `SEIKIKA_AS` can replace `--url` and `--as` | the commands below |

## Commands

### Setup (every shell)

```sh
export PATH=/tmp/actions-ab/r4/seikika-2/bin:$PATH
export SEIKIKA_STATE_DIR=/tmp/actions-ab/r4/seikika-2/state   # must be this directory: it holds the signing key the repositories pin
export SEIKIKA_URL=http://127.0.0.1:7950
cd /tmp/actions-ab/r4/seikika-2/org
```

### Start and stop the server

```sh
seikika serve /tmp/actions-ab/r4/seikika-2/org --listen 127.0.0.1:7950 --detach   # returns once ready; prints the key
seikika serve /tmp/actions-ab/r4/seikika-2/org --stop
```

The server must print `approval key: ed25519:ad0cd4e2a626cc148107caebf63b8d45c9bf8c4cd2036c5ed0b59c027d9ea651`.
If it prints a different key, `SEIKIKA_STATE_DIR` is wrong. With the wrong key, previews show the
blocker `child-refuses-approvals` and the repositories refuse approvals.

### List

```sh
seikika run --as ann -o json
```

### Preview (changes nothing)

```sh
seikika run --as ann mark-deprecated --entity portal:alpha \
  -i successor=acme/dependabot-proxy-v2 -i 'reason=Replaced by the v2 proxy, which supports OIDC registries.' -o json
# .effects[0].diff is the change; .blockers lists anything that stops it; .confirm_token is for the request
```

### Request, as ann

Either confirm the preview you read:

```sh
seikika run --as ann mark-deprecated --entity portal:alpha \
  -i successor=acme/dependabot-proxy-v2 -i 'reason=Replaced by the v2 proxy, which supports OIDC registries.' \
  --confirm <confirm_token> -o json        # exit 3, .id is the run id, .state is awaiting_approval
```

or preview and request in one step (the preview is still recorded):

```sh
seikika run --as ann mark-deprecated --entity portal:alpha \
  -i successor=acme/dependabot-proxy-v2 -i 'reason=Replaced by the v2 proxy, which supports OIDC registries.' --yes -o json
```

### Approve, as bob

```sh
seikika run --as bob --approve <run id> --note "checked the diff" --wait -o json   # exit 0 once main has it
```

`--reject <run id>` rejects the run instead.

### Check a run's status, and read its record

```sh
seikika run --as ann --runs -o json                        # every run: id, action, entity, requester, state, started
seikika run --as ann --verify <run id> -o json             # the full record (state, requested_by, approval, inputs, events, verification)
seikika run --as ann --verify <run id> --wait -o json      # follow it until it settles
cat /tmp/actions-ab/r4/seikika-2/state/served/org/runs/<run id>.json   # the stored record
```

To read `state`, `requested_by`, `approved_by` and the event times in one line:

```sh
seikika run --as ann --verify run_2fst04vhrxmq -o json | python3 -c 'import json,sys; d=json.load(sys.stdin); print(d["state"], d["requested_by"], d["approval"]["approved_by"], d["inputs"], [(e["state"], e.get("by"), e["at"]) for e in d["events"]])'
```

## Testing the rules

### Offline, with no server

Each repository's test cases run in a scratch copy of the repository and its origin. Nothing real
changes:

```sh
for r in alpha beta gamma; do seikika validate /tmp/actions-ab/r4/seikika-2/$r/.seikika && seikika test /tmp/actions-ab/r4/seikika-2/$r/.seikika; done   # 9 passed each
```

### Live, against the server

**Invalid inputs** exit 2 with `422 invalid-inputs`, and no run is created:

```sh
seikika run --as ann mark-deprecated --entity portal:beta -i successor=Acme/Not_Valid -i 'reason=State management moves to the shared store package.' -o json
seikika run --as ann mark-deprecated --entity portal:beta -i successor=acme/valtio-store -i 'reason=too short' -o json
seikika run --as ann mark-deprecated --entity portal:beta -i successor=acme/valtio-store -i "reason=$(printf 'x%.0s' $(seq 201))" -o json
seikika run --as ann mark-deprecated --entity portal:beta -i successor=acme/valtio-store -o json   # reason missing
seikika run --as ann mark-deprecated --entity portal:beta -i successor=acme/beta -i 'reason=State management moves to the shared store package.' -o json   # .blockers names the precondition, no confirm_token; with --yes it exits 2 and no run is created
```

**Self-approval and carol.** Both exit 2 with `403 not-an-approver`, and the run stays
`awaiting_approval`:

```sh
ID=$(seikika run --as ann mark-deprecated --entity portal:beta -i successor=acme/other-store -i 'reason=Testing the approval rules end to end.' --yes -o json | python3 -c 'import json,sys; print(json.load(sys.stdin)["id"])')
seikika run --as ann   --approve $ID -o json    # 403: a requester cannot approve their own run
seikika run --as carol --approve $ID -o json    # 403: user:carol is not one of the pending approvers
seikika run --as bob   --reject  $ID -o json    # clean up without changing main (or approve it as bob to land it)
```

### Test a rejected push

The run must use inputs that differ from what is already on `main`. A change that `main` already has
succeeds without a commit.

```sh
H=/tmp/actions-ab/r4/seikika-2/origins/gamma.git/hooks/pre-receive
printf '#!/bin/sh\necho "main is protected"\nexit 1\n' > $H && chmod +x $H
ID=$(seikika run --as ann mark-deprecated --entity portal:gamma -i successor=acme/other-core -i 'reason=Testing that a rejected push fails the run.' --yes -o json | python3 -c 'import json,sys; print(json.load(sys.stdin)["id"])')
seikika run --as bob --approve $ID --wait -o json    # exit 1, .state is failed, .error has "remote: main is protected … pre-receive hook declined"
rm $H
git -C /tmp/actions-ab/r4/seikika-2/origins/gamma.git show main:DEPRECATED.md   # unchanged
```

## Notes

- `--as` takes `ann` or `user:ann`. People sign in with `--as` only because the server listens on
  a loopback address (Seikika's dev sign-in). A server on any other address needs
  `SEIKIKA_AUTH_MODE` set (see `seikika serve --help`).
- The file content is `# Deprecated: use <successor>`, a blank line, `<reason>`, and a trailing
  newline.
- To change the action for one repository, edit that repository's `.seikika/seikika.yaml`, run
  `seikika validate .seikika && seikika test .seikika`, and push to `main`. The org portal picks
  the change up on its next request.
- The server was stopped after the three runs. Start it with the command above before you test.
