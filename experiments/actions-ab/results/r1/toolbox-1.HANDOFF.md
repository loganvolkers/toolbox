# Handoff: the `mark-deprecated` self-service action

This is about the box: Toolbox carries, runs, approves, verifies and remembers the action. The tools
are git and `printf`.

**Status.** `mark-deprecated` is defined in each of `alpha`, `beta` and `gamma` and offered from this
org portal. ann requested it on all three and bob approved each run. All three runs are reported
`succeeded`, and `DEPRECATED.md` on each origin's `main` matches the required content byte for byte:

| Repository | Org run | Repository run | Commit on `origins/<repo>.git` `main` |
|---|---|---|---|
| alpha | `run_2fq6surjgp8c` | `run_2fq6v4gjta7g` | `653591e` |
| beta | `run_2fq6t07ge58u` | `run_2fq6v4wbik2y` | `efa094f` |
| gamma | `run_2fq6t59s2fse` | `run_2fq6v77acr4f` | `b70dae8` |

## How it fits together

| Piece | Path | Role |
|---|---|---|
| Repository portal | `<repo>/.toolbox/toolbox.yaml` (on each origin's `main`) | Defines `mark-deprecated`: inputs schema, precondition, approvers, executor. It pins the org portal's approval key. |
| Org portal | `org/toolbox.yaml` | Finds every `*/.toolbox/toolbox.yaml` under `org/var/repos`, offers those actions on `portal:<repo>` entities, and dispatches runs to them (`federation`). It defines no action itself. |
| Repository list | `org/repos.txt` | The repositories the org syncs. Keep it in step with `federation.runners.local.portals` in `org/toolbox.yaml`. |
| Sync | `org/bin/sync` | Keeps `org/var/repos/<repo>` at `origins/<repo>.git` `main`. Every 5 seconds it checks, and when a `main` moved it calls `toolbox refresh portal:<repo>`. |
| Start / stop | `org/bin/start`, `org/bin/stop` | Start `toolbox serve` on `127.0.0.1:7110` plus the sync loop. Stop both. |
| Runtime state (not committed) | `org/var/` | `approval-key` (the signing key: keep it, the repositories pin its public half), `repos/` (clones), `state/` (run records), `server.log`, `sync.log`. |

The executor runs in a worktree of a fresh `origin/main`, commits as `Acme org portal
<toolbox@acme.example>`, and pushes to `main` without forcing. The person who asked and the person
who approved are in the run record, not in the commit.

## R1 to R8

| # | How it is met | Files |
|---|---|---|
| R1 Owned by the repository | The whole action (inputs, precondition, approvers, executor) is in each repository's `.toolbox/toolbox.yaml`, committed to its origin's `main`. The org portal reads only its clones of `main` (`org/var/repos/<repo>`), which `bin/sync` keeps current, so a definition counts once it is pushed to `main`. Uncommitted edits in a checkout don't count. A team changes the action by committing to its own file. I tested this: a `maxLength` change pushed to `main` showed in the org's listing within one sync interval. Each repository also decides whose approvals it accepts (`federation.acceptApprovalsFrom`). | `alpha/.toolbox/toolbox.yaml`, `beta/.toolbox/toolbox.yaml`, `gamma/.toolbox/toolbox.yaml`, `org/bin/sync` |
| R2 Validated | The inputs are a JSON Schema: `successor` has `pattern: ^[a-z0-9-]+/[a-z0-9-]+$`, and `reason` has `minLength: 20` and `maxLength: 200`. Both are required, and unknown inputs are refused. A precondition refuses the repository itself as its successor (`!inputs.successor.endsWith('/<repo>')`). Invalid inputs fail the preview with `422 invalid-inputs` (exit 2). A failed precondition shows as a blocker, the preview gives no confirm token, and `--yes` is refused. Neither creates a run. | `<repo>/.toolbox/toolbox.yaml` (`inputs`, `preconditions`) |
| R3 Previewed | Without `--confirm`, the request is a preview. It runs the executor in a throwaway worktree and returns the exact diff of `DEPRECATED.md`, the push to `main`, the approvers, any blockers and a confirm token (valid 10 minutes, spent once). It changes nothing. A run starts only with `--confirm <token>`, which is bound to the previewed inputs and effects. | `<repo>/.toolbox/toolbox.yaml` (`executor`) |
| R4 Approved | `approval: "[['user:ann', 'user:bob']]"` means one of ann or bob, never the requester. A confirmed run waits in `awaiting_approval` (exit 3). Self-approval gives `403 not-an-approver: a requester cannot approve their own run`. carol gets `403 not-an-approver: user:carol is not one of the pending approvers`. An approval without a signed-in identity gets `401`. The org signs each approval with `org/var/approval-key`, and a repository executes only a run whose approval verifies against the key it pins. | `<repo>/.toolbox/toolbox.yaml` (`approval`, `federation.acceptApprovalsFrom`), `org/toolbox.yaml` (`federation.signingKey`) |
| R5 Done means observed | `land: push` pushes once the approval rule is met. After the push, Toolbox fetches `origin`, checks that the commit is on `origin/main`, and checks that `DEPRECATED.md` there is what the run wrote. Only then is the state `succeeded`. I tested three failures in a scratch copy, and each one gave `failed`: a pre-receive hook that rejects the push ("the push to origin main was refused, so nothing landed"), a post-receive hook that rewinds `main` ("the commit is not on origin/main"), and a hook that overwrites the file ("DEPRECATED.md on origin/main differ from what it wrote"). | `<repo>/.toolbox/toolbox.yaml` (`executor.land: push`) |
| R6 One place for the org | The org portal at `http://127.0.0.1:7110`, served from `org/`, lists `mark-deprecated` on `portal:alpha`, `portal:beta` and `portal:gamma`. You can preview, request, approve and check any of them with `--url`, without entering a repository. | `org/toolbox.yaml`, `org/bin/start` |
| R7 Audited | Each run record holds `requested_by`, `inputs`, `approval.approved_by`, `state`, `outputs` (commit, files), `verification`, and `events` with a timestamp and actor for each event (`previewed`, `confirmed`, `awaiting_approval`, `approved` by bob, `dispatched`, `running`, `executor_succeeded`, then `succeeded` or `failed` with the reason). The record also embeds the repository's own run as `child`. Records are files and survive a restart. | `org/var/state/served/org/runs/<run id>.json` (org run), `org/var/state/runs/<child run id>.json` (repository run) |
| R8 Usable by agents | Every step is a non-interactive `toolbox` command with `-o json` and documented exit codes: 0 succeeded, 1 failed or stale, 2 invalid or refused, 3 awaiting approval. Identity is a flag (`--as`). | the commands below |

## Commands

Run everything from `/tmp/actions-ab/r1/toolbox-1`. Sign-in is dev mode, so `--as user:<name>` says
who you are. The server records that name and enforces the approval rules against it.

```bash
cd /tmp/actions-ab/r1/toolbox-1
export PATH="$PWD/bin:$PATH"
U=http://127.0.0.1:7110

# Start: the org portal on 127.0.0.1:7110 and the sync loop
org/bin/start

# List the actions, every repository
toolbox run --url $U --as user:ann -o json
# or for one repository
toolbox actions portal:alpha --url $U --as user:ann -o json

# Preview (changes nothing). Read effects[].diff, blockers and approval, and keep confirm_token.
toolbox run --url $U --as user:ann mark-deprecated --entity portal:alpha \
  -i successor=acme/dependabot-proxy-v2 \
  -i "reason=Replaced by the v2 proxy, which supports OIDC registries." -o json

# Request as ann: the same inputs plus the token from the preview. Exit 3, state awaiting_approval.
# Keep "id".
toolbox run --url $U --as user:ann mark-deprecated --entity portal:alpha \
  -i successor=acme/dependabot-proxy-v2 \
  -i "reason=Replaced by the v2 proxy, which supports OIDC registries." \
  --confirm <confirm_token> -o json

# Approve as bob. The run then executes, pushes and verifies.
toolbox run --url $U --as user:bob --approve <run id> -o json

# Check the status and read the full record. Add --wait to follow it until it settles.
toolbox run --url $U --as user:ann --verify <run id> -o json
toolbox run --url $U --as user:ann --verify <run id> --wait -o json
# List the runs
toolbox run --url $U --as user:ann --runs -o json
# The same record on disk
cat org/var/state/served/org/runs/<run id>.json

# Stop
org/bin/stop
```

A preview and its request can be scripted in one step:

```bash
P=$(toolbox run --url $U --as user:ann mark-deprecated --entity portal:beta \
  -i successor=acme/valtio-store -i "reason=State management moves to the shared store package." -o json)
toolbox run --url $U --as user:ann mark-deprecated --entity portal:beta \
  -i successor=acme/valtio-store -i "reason=State management moves to the shared store package." \
  --confirm "$(echo "$P" | python3 -c 'import json,sys; print(json.load(sys.stdin)["confirm_token"])')" -o json
```

## Testing the failure cases

All three repositories are already deprecated. A new run with different inputs is therefore an
`update` of `DEPRECATED.md`. A run with the same inputs previews with the warning "the change command
changed no files".

| Case | Command | Expected |
|---|---|---|
| Bad successor | `toolbox run --url $U --as user:ann mark-deprecated --entity portal:gamma -i successor=Acme/X -i "reason=Long enough reason text here." -o json` | `422 invalid-inputs: successor ... does not match`, exit 2, no run |
| Short or long reason | `... -i successor=acme/x -i reason=tiny ...`, or a reason over 200 characters | `422 invalid-inputs: reason ... shorter than 20` or `longer than 200`, exit 2, no run |
| Itself as successor | `... --entity portal:gamma -i successor=acme/gamma ...` | `blockers: ["precondition not met: ..."]`, no `confirm_token` |
| Self-approval | request as ann, then `toolbox run --url $U --as user:ann --approve <id> -o json` | `403 not-an-approver: a requester cannot approve their own run`, exit 2, still `awaiting_approval` |
| carol approves | `toolbox run --url $U --as user:carol --approve <id> -o json` | `403 not-an-approver: user:carol is not one of the pending approvers`, exit 2 |
| Rejected push | `printf '#!/bin/sh\necho rejected >&2\nexit 1\n' > origins/beta.git/hooks/pre-receive && chmod +x origins/beta.git/hooks/pre-receive`, then request on `portal:beta` with new inputs as ann and approve as bob | `--verify <id> --wait` exits 1 with `state: failed` and "the push to origin main was refused, so nothing landed". Remove the hook afterwards: `rm origins/beta.git/hooks/pre-receive` |

## Notes and limits

- **Identity.** Identity is asserted with `--as`, because the sandbox has no passwords
  (`TOOLBOX_AUTH_MODE=dev` in `org/bin/start`). In production, the same server takes OIDC or
  personal access tokens, and approvals then need an interactive sign-in. The approval rule and the
  self-approval ban are enforced either way.
- **Approval key.** `org/var/approval-key` is the org's approval signing key. If it is lost,
  `bin/start` makes a new one and warns that the repositories don't pin it. Pin the new public key,
  which `bin/start` prints, in each `<repo>/.toolbox/toolbox.yaml` and push it.
- **Adding a repository.** Add it to `org/repos.txt` and to `federation.runners.local.portals` in
  `org/toolbox.yaml`, commit its `.toolbox/toolbox.yaml` to its origin, then run `org/bin/stop` and
  `org/bin/start`.
- **Approval notes.** `--note` on `--approve` is not kept. The `approved` event's note shows the
  approver group instead.
