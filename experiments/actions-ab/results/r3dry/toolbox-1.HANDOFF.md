# Handoff: `mark-deprecated`

`mark-deprecated` is a Toolbox self-service action. Each of `alpha`, `beta` and `gamma` declares it
in its own `.toolbox/toolbox.yaml` on its origin's `main`. The org portal in this directory offers
all three from one place, holds the approvals, runs each approved action in its clone of the
repository, and records what it observed. The glue I wrote is limited to `org/env.sh` (environment)
and `org/start.sh` (start and wait until ready). Validation, preview, approval, landing, verification
and the audit record are all Toolbox (`/tmp/actions-ab/r3dry/toolbox-1/bin/toolbox`, 0.1.1).

All three runs succeeded, requested by ann and approved by bob:

| Repository | Org run (record) | Child run (`Toolbox-Run` trailer) | Commit on `origins/<repo>.git` `main` |
|---|---|---|---|
| alpha | `run_2frvlmppl41l` | `run_2frvmbmng0py` | `abc9c4d` Mark alpha deprecated |
| beta | `run_2frvlts7g9ca` | `run_2frvn4hpv6du` | `d8d13ca` Mark beta deprecated |
| gamma | `run_2frvm0ietdqw` | `run_2frvo0w1l1by` | `24d5e46` Mark gamma deprecated |

## Files

| Path | What it is |
|---|---|
| `/tmp/actions-ab/r3dry/toolbox-1/{alpha,beta,gamma}/.toolbox/toolbox.yaml` | Each repository's definition of `mark-deprecated` (committed and pushed to `origins/<repo>.git` `main`) |
| `/tmp/actions-ab/r3dry/toolbox-1/org/toolbox.yaml` | The org portal: `federation.children` lists the three origins |
| `/tmp/actions-ab/r3dry/toolbox-1/org/env.sh` | Sets `PATH`, `TOOLBOX_STATE_DIR`, `XDG_STATE_HOME` and `TOOLBOX_URL=http://127.0.0.1:7900` |
| `/tmp/actions-ab/r3dry/toolbox-1/org/start.sh` | Starts the org portal on 127.0.0.1:7900, detached, and waits until all three actions are offered |
| `/tmp/actions-ab/r3dry/toolbox-1/.toolbox-state/served/org/signing-key` | The org portal's approval signing key. Each repository pins its public half (`ed25519:614a1f…62c9`) |
| `/tmp/actions-ab/r3dry/toolbox-1/.toolbox-state/served/org/runs/<run id>.json` | Org run records: the audit trail |
| `/tmp/actions-ab/r3dry/toolbox-1/.toolbox-state/runs/<child run id>.json` | The repository side of each run (what ran in the clone and what was verified) |
| `/tmp/actions-ab/r3dry/toolbox-1/.toolbox-state/served/org/serve.log` | Server log |

Always source `env.sh` first. The repositories accept approvals only when they are signed with the
key under `TOOLBOX_STATE_DIR`. A server started with a different state directory would create a new
key, and every run it approved would be refused.

## How each requirement is met

| # | How |
|---|---|
| R1 Owned by the repository | The action is defined in `alpha/.toolbox/toolbox.yaml`, `beta/.toolbox/toolbox.yaml` and `gamma/.toolbox/toolbox.yaml`, each committed to its origin's `main`. That file holds the inputs, the approvers (`approval: "[['user:ann', 'user:bob']]"`), the precondition and the command. The org portal (`org/toolbox.yaml`) only lists where the repositories are. It re-reads each repository's `main` before it lists, previews or runs, so a change the team pushes takes effect without touching the org. This was exercised: after the first manifests landed I pushed a commit-message change to each repository, and the real runs used the new message with no org change or restart. |
| R2 Validated | The `inputs` JSON Schema in each manifest is checked before any preview, on the server and in a checkout: `successor` must match `^[a-z0-9-]+/[a-z0-9-]+$`, `reason` is `minLength: 20` and `maxLength: 200`, both are required, and `additionalProperties: false`. Invalid inputs return `422 invalid-inputs` with each error, exit 2, and no run is created. A precondition, `!inputs.successor.endsWith('/<repo>')`, blocks naming the repository as its own successor: the preview has a blocker and no confirm token. |
| R3 Previewed | `toolbox run … mark-deprecated …` without `--yes` or `--confirm` previews. Nothing changes. The output has the exact `DEPRECATED.md` diff (`child.effects[0].diff`), the push to `main`, the approvers, any blockers and a `confirm_token`, valid for 10 minutes and usable once. `--confirm <token>` requests exactly what was previewed. `--yes` previews and requests in one step, and the preview is still recorded on the run (`events[].state == "previewed"`). |
| R4 Approved | `approval: "[['user:ann', 'user:bob']]"` in each repository's manifest means one of ann or bob, and Toolbox never counts the requester. A request waits in `awaiting_approval` (exit 3). Tested: ann approving her own run gets `403 not-an-approver: a requester cannot approve their own run`, and carol gets `403 not-an-approver: user:carol is not one of the pending approvers`, both exit 2. The org portal signs each approval with its key, and the repository's manifest accepts it only with that key pinned (`federation.acceptApprovalsFrom`). |
| R5 Done means observed | The executor is `type: change`, `land: push`, `base: main`. It runs the `printf` in a fresh worktree of `main`, commits, and pushes without force. It reports `succeeded` only after fetching `origin/main` and seeing the commit there, with `DEPRECATED.md`'s blob equal to the one the preview showed (`verification.child.files[].matches: true`, `checked: "fetched origin/main: the commit is on it, and every file it changed has the content the preview showed"`). Tested on a scratch copy of the origins: a `pre-receive` hook that rejects the push gives `failed` with the origin's message (exit 1). A `post-receive` hook that accepts the push and then resets `main` gives `failed`, "the commit is not on origin/main: the push did not land" (exit 1). A run whose inputs match the file already on `main` changes nothing and ends `unverified` (exit 4), not `succeeded`. |
| R6 One place for the org | `org/toolbox.yaml` lists the three origins under `federation.children`, and each is offered as `portal:<repo>`. With the org portal served (`org/start.sh`), `toolbox run --as <user> -o json` from `org/` lists `mark-deprecated` for `portal:alpha`, `portal:beta` and `portal:gamma`. Preview, request, approve, status and record all go through the same server, with `--entity portal:<repo>`. Nobody has to enter a repository. |
| R7 Audited | Each run is a JSON record in `.toolbox-state/served/org/runs/<run id>.json`, also printed by `toolbox run --verify <run id> -o json`. It has `requested_by`, `inputs`, `approval.approved_by`, `state` and `events[]`. Each event has a `state` (previewed, confirmed, awaiting_approval, approved / rejected, dispatched, running, succeeded / failed / unverified), an `at` timestamp, a `by` where a person acted, and the approver's `--note`. `child` and `verification` hold the repository-side run, the commit and what was observed. The commit on `main` carries `Toolbox-Run`, `Requested-By` and `Approved-By` trailers. Records survive a server restart. |
| R8 Usable by agents | Every step is a non-interactive `toolbox run` invocation with `-o json`. Exit codes: 0 succeeded, 1 failed or rejected, 2 invalid or refused, 3 awaiting approval, 4 unverified. `TOOLBOX_URL` and `TOOLBOX_AS` can stand for `--url` and `--as`. The agent skill for this command is `toolbox-docs/skills/run/SKILL.md`. |

## Commands

Run everything from the org directory with the environment sourced:

```sh
cd /tmp/actions-ab/r3dry/toolbox-1/org
. ./env.sh                       # PATH, TOOLBOX_STATE_DIR, XDG_STATE_HOME, TOOLBOX_URL=http://127.0.0.1:7900
```

**Start** (it must be running for everything below; it returns once ready):

```sh
./start.sh
```

**Stop:**

```sh
toolbox serve /tmp/actions-ab/r3dry/toolbox-1/org --stop
```

**List** the actions on all three repositories:

```sh
toolbox run --as ann -o json
```

**Preview** (changes nothing; prints the diff and a `confirm_token`):

```sh
toolbox run --as ann mark-deprecated --entity portal:alpha \
  -i successor=acme/dependabot-proxy-v3 -i "reason=Replaced again, by the v3 proxy this time." -o json
```

**Request** as ann. Either confirm the preview you read (same inputs, plus its token):

```sh
toolbox run --as ann mark-deprecated --entity portal:alpha \
  -i successor=acme/dependabot-proxy-v3 -i "reason=Replaced again, by the v3 proxy this time." \
  --confirm <confirm_token> -o json      # exit 3, prints "id": "run_…" and "state": "awaiting_approval"
```

or preview and request in one step:

```sh
toolbox run --as ann mark-deprecated --entity portal:alpha \
  -i successor=acme/dependabot-proxy-v3 -i "reason=Replaced again, by the v3 proxy this time." --yes -o json
```

**Approve** as bob (runs it, then waits for the outcome; or reject with `--reject <run id>`):

```sh
toolbox run --as bob --approve <run id> --note "checked" --wait -o json   # exit 0 when succeeded
```

**Check a run's status** (`.state`, and the exit code), and list every run:

```sh
toolbox run --as ann --verify <run id> -o json | python3 -c 'import json,sys; print(json.load(sys.stdin)["state"])'
toolbox run --as ann --runs -o json
```

**Read a run's record:**

```sh
toolbox run --as ann --verify <run id> -o json
cat /tmp/actions-ab/r3dry/toolbox-1/.toolbox-state/served/org/runs/<run id>.json
```

Use these run ids for the three completed runs: `run_2frvlmppl41l` (alpha), `run_2frvlts7g9ca`
(beta) and `run_2frvm0ietdqw` (gamma).

## Testing the refusals

Each repository already has a `DEPRECATED.md`, so use inputs that differ from it. A run with
identical inputs changes nothing and ends `unverified`.

```sh
# Invalid inputs: 422 invalid-inputs, exit 2, no run created
toolbox run --as ann mark-deprecated --entity portal:beta -i successor=Acme/Store -i reason=too-short -o json
toolbox run --as ann mark-deprecated --entity portal:beta -i successor=acme/x -i "reason=$(printf 'x%.0s' $(seq 201))" -o json
# Own repository as successor: a blocker, no confirm token
toolbox run --as ann mark-deprecated --entity portal:beta -i successor=acme/beta -i "reason=Pointing at itself is refused." -o json

# Self-approval and carol: 403 not-an-approver, exit 2; the run stays awaiting_approval
ID=$(toolbox run --as ann mark-deprecated --entity portal:beta -i successor=acme/other -i "reason=Testing approvals on beta." --yes -o json | python3 -c 'import json,sys; print(json.load(sys.stdin)["id"])')
toolbox run --as ann   --approve "$ID" -o json
toolbox run --as carol --approve "$ID" -o json
toolbox run --as bob   --reject  "$ID" --note "test only" -o json      # clean up: state rejected, exit 1

# Rejected push: the run fails with the origin's message (exit 1) and is never reported succeeded
printf '#!/bin/sh\necho "main is frozen" >&2\nexit 1\n' > /tmp/actions-ab/r3dry/toolbox-1/origins/gamma.git/hooks/pre-receive
chmod +x /tmp/actions-ab/r3dry/toolbox-1/origins/gamma.git/hooks/pre-receive
ID=$(toolbox run --as ann mark-deprecated --entity portal:gamma -i successor=acme/other -i "reason=Testing a rejected push on gamma." --yes -o json | python3 -c 'import json,sys; print(json.load(sys.stdin)["id"])')
toolbox run --as bob --approve "$ID" --wait -o json                     # state failed, exit 1
rm /tmp/actions-ab/r3dry/toolbox-1/origins/gamma.git/hooks/pre-receive
```

## Limits

- There are no passwords in this sandbox. The server is bound to loopback and uses Toolbox's dev
  sign-in, so `--as <name>` is who you are. Off loopback, set `TOOLBOX_AUTH_MODE` (`toolbox serve --help`).
- The server runs without OpenFGA, so reads and requests are open to anyone signed in. carol can
  request a run, but she cannot approve one. Approvals are enforced by each repository's `approval`
  list, not by OpenFGA.
- Maintainers are named in each repository's `approval` line. To change who may approve,
  that repository's team edits its own manifest.
- Right after `toolbox serve --detach` returns, the clones may not have been read yet, and a preview
  gets `404 not-offered`. `start.sh` waits until all three are offered.
