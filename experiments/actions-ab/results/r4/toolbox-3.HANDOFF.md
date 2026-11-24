# Handoff: `mark-deprecated`

`mark-deprecated` is a Toolbox self-service action. Each of `alpha`, `beta` and `gamma` declares
it in its own `.toolbox/toolbox.yaml` on its origin's `main`. The org portal in this directory
(`org/toolbox.yaml`) lists the three repositories, and `toolbox serve` serves it on
`127.0.0.1:7990`. People and agents list, preview, request, approve, check and read every run there
with `toolbox run`. No custom code was needed: Toolbox validates the inputs, previews the diff,
holds and signs the approvals, commits and pushes, and verifies the result on the origin.
`org/env.sh` (three environment variables) is the only glue.

**Status:** all three runs were requested by ann, approved by bob, and are reported `succeeded`.
`DEPRECATED.md` is on `main` in `origins/alpha.git`, `origins/beta.git` and `origins/gamma.git`.

| Repository | Run id (org portal) | State | Commit on origin `main` |
|---|---|---|---|
| alpha | `run_2fstm5swm1u4` | succeeded | `8e7b1b5` |
| beta | `run_2fstr6fxo4v0` | succeeded | `fabe3fd` |
| gamma | `run_2fstsadflxkn` | succeeded | `3b80482` |

Paths below are relative to `/tmp/actions-ab/r4/toolbox-3`.

## How each requirement is met

| # | How | Files |
|---|---|---|
| R1 Owned by the repository | Each repository declares `mark-deprecated` (inputs, approvers, the command that writes `DEPRECATED.md`) and its test cases, committed to its origin's `main`. The org portal holds no copy: it clones each origin's `main` and syncs that clone before every list, preview and run, so a change the team pushes to `main` takes effect on the next request. The repository also pins the org portal's approval key, so it decides whose approvals it accepts | `alpha/.toolbox/toolbox.yaml`, `beta/.toolbox/toolbox.yaml`, `gamma/.toolbox/toolbox.yaml`, and `*/.toolbox/tests/mark-deprecated.test.yaml` |
| R2 Validated | `inputs` is a JSON Schema: both inputs required, no others allowed (`additionalProperties: false`), `successor` must match `^[a-z0-9-]+/[a-z0-9-]+$`, and `reason` must be 20 to 200 characters long. Toolbox checks it before the preview, so an invalid request exits 2 with `422 invalid-inputs` and no run is created. A precondition also blocks a repository naming itself as its own successor (`acme/alpha` on alpha) | `actions[0].inputs` and `preconditions` in each `<repo>/.toolbox/toolbox.yaml` |
| R3 Previewed | Without `--yes`, `toolbox run <action> …` only previews. It runs the command in a scratch worktree and prints the exact diff of `DEPRECATED.md`, the push to `main`, and who must approve, and it changes nothing. A request (`--yes`) records that preview on the run (`effects`), and the change that lands must match it: a change that would differ fails before anything is committed | the same manifests (`executor: {type: change, land: push}`) |
| R4 Approved | `approval: "[['user:ann', 'user:bob']]"` means either ann or bob, and the requester never counts. The server refuses ann approving her own run (`403 not-an-approver: a requester cannot approve their own run`) and carol (`403 not-an-approver: user:carol is not one of the pending approvers`), both exit 2. The run waits in `awaiting_approval` until another maintainer approves it. The org portal signs each approval, and the repository accepts it only with the key pinned in `federation.acceptApprovalsFrom` | `approval` and `federation.acceptApprovalsFrom` in each `<repo>/.toolbox/toolbox.yaml`; the signing key is `.toolbox-state/served/org/signing-key` |
| R5 Done means observed | `land: push` commits in a fresh worktree of `main` and pushes without forcing. Toolbox then fetches `origin/main` and reports `succeeded` only if the commit is on it and every changed file's blob matches the preview (`verification.child.files[].matches`). A rejected push makes the run `failed`, quoting the origin's message (exit 1). This was checked live on a scratch copy of alpha's origin with a rejecting `pre-receive` hook, and it is a test case in every repository | `executor` in each manifest; the `rejectPush` case in each `<repo>/.toolbox/tests/mark-deprecated.test.yaml` |
| R6 One place for the org | `org/toolbox.yaml` lists the three origins as `federation.children`, so the action is offered as `portal:alpha`, `portal:beta` and `portal:gamma`. From `org/`, one `toolbox run` lists all three, and `--entity portal:<name>` previews, requests or approves on any of them | `org/toolbox.yaml`, `org/env.sh` |
| R7 Audited | Each run is one JSON record holding `requested_by`, `inputs`, `approval.approved_by`, `state`, `outputs` (the commit), `verification` (what was seen on `main`), and timestamped `events` (`previewed`, `confirmed` and `awaiting_approval` by the requester; `approved` by the approver, with the note; `running`; `succeeded` or `failed`, with the error). The commit on `main` carries `Toolbox-Run`, `Requested-By` and `Approved-By` trailers. Read it with `toolbox run --verify <id> -o json` | `.toolbox-state/served/org/runs/<run id>.json` |
| R8 Usable by agents | Every step is one non-interactive `toolbox run` command, and `-o json` makes every output machine-readable. The exit code is the state: 0 succeeded, 1 failed, 2 invalid or refused, 3 waiting for approval, 4 unverified. `--as` names the person (allowed because the server listens on loopback). `--wait` follows a run until it settles | `toolbox run --help`; `toolbox-docs/skills/run/SKILL.md` |

## Commands

### Start the server (must be running for everything below)

```sh
cd /tmp/actions-ab/r4/toolbox-3/org
. ./env.sh                                          # PATH, TOOLBOX_STATE_DIR, TOOLBOX_URL=http://127.0.0.1:7990
toolbox validate .                                  # optional: check the org manifest
toolbox serve . --listen 127.0.0.1:7990 --detach    # returns once ready; prints the approval key
```

The printed key must be `ed25519:151ff661eda992fee33b37bbb13e71e4f93987e61c44aee4f8ff80cdd1f8faea`,
the key each repository pins. It is reused only if `TOOLBOX_STATE_DIR` is the one in `env.sh`.
Stop the server with `toolbox serve . --stop`, run from `org/` with `env.sh` sourced. Runs survive
a restart.

Every command below assumes `cd /tmp/actions-ab/r4/toolbox-3/org && . ./env.sh`. Drop `-o json`
to get human-readable output.

### List the actions (all three repositories)

```sh
toolbox run --as ann -o json
```

### Preview (changes nothing; prints the diff, the approvers and a confirm token)

```sh
toolbox run --as ann mark-deprecated --entity portal:alpha \
  -i successor=acme/dependabot-proxy-v2 \
  -i 'reason=Replaced by the v2 proxy, which supports OIDC registries.' -o json
```

### Request as ann (exit 3, `state: awaiting_approval`; the run id is `.id`)

```sh
toolbox run --as ann mark-deprecated --entity portal:alpha \
  -i successor=acme/dependabot-proxy-v2 \
  -i 'reason=Replaced by the v2 proxy, which supports OIDC registries.' --yes -o json
# or, after a preview: toolbox run --as ann mark-deprecated --entity portal:alpha -i … --confirm <ct_… token> -o json
```

### Approve as bob (runs it, waits until it is observed on main: exit 0 `succeeded`, 1 `failed`)

```sh
toolbox run --as bob --approve <run-id> --note "checked the preview" --wait -o json
```

`toolbox run --as bob --reject <run-id>` rejects a run instead.

### Check a run's status and read its record

```sh
toolbox run --as ann --verify <run-id> -o json    # the full record; .state is the status, the exit code matches it
toolbox run --as ann --runs -o json               # every run, all repositories
cat /tmp/actions-ab/r4/toolbox-3/.toolbox-state/served/org/runs/<run-id>.json   # the same record on disk
git --git-dir=/tmp/actions-ab/r4/toolbox-3/origins/alpha.git show main:DEPRECATED.md
```

## Testing the refusals

These were run live against the server, except two: the rejected push ran on a scratch copy of
alpha's origin, and carol's request is covered by the offline test cases.

| Case | Command | Expected |
|---|---|---|
| Invalid successor or reason | `toolbox run --as ann mark-deprecated --entity portal:beta -i successor=Acme/X -i reason=short --yes -o json` | exit 2, `422 invalid-inputs` naming both inputs; no run created |
| Reason over 200 characters | `-i "reason=$(printf 'x%.0s' $(seq 201))"` | exit 2, `422 invalid-inputs` |
| Unknown or missing input | `-i extra=1` or no `reason` | exit 2, `422 invalid-inputs` |
| Repository as its own successor | `-i successor=acme/alpha` on `portal:alpha` | exit 2, blocked: `precondition not met` |
| Self-approval | ann requests, then `toolbox run --as ann --approve <id> -o json` | exit 2, `403 not-an-approver: a requester cannot approve their own run`; still `awaiting_approval` |
| carol approves | `toolbox run --as carol --approve <id> -o json` | exit 2, `403 not-an-approver`; still `awaiting_approval` |
| carol requests | carol requests; ann or bob approves | allowed: carol may ask, and only a maintainer other than her approves |
| Rejected push | add a rejecting hook to an origin (below), request with **new** inputs, and approve as bob | exit 1, `state: failed`, with the origin's `remote rejected` message; `main` unchanged |

To reject pushes on an origin, and to undo that afterwards:

```sh
printf '#!/bin/sh\necho "main is protected" >&2\nexit 1\n' > /tmp/actions-ab/r4/toolbox-3/origins/alpha.git/hooks/pre-receive
chmod +x /tmp/actions-ab/r4/toolbox-3/origins/alpha.git/hooks/pre-receive
# … request and approve a run with a different successor or reason …
rm /tmp/actions-ab/r4/toolbox-3/origins/alpha.git/hooks/pre-receive
```

`DEPRECATED.md` is already on every `main`. A run whose inputs produce the same content therefore
pushes nothing and reports `succeeded` with `outputs.changed: false`, because the file it needed
was observed on `main`. Use different inputs to test a rejected push.

The same cases run offline, without the server or the real origins, in a scratch copy of each
repository: `cd /tmp/actions-ab/r4/toolbox-3/alpha && toolbox validate .toolbox && toolbox test .toolbox`
(nine cases per repository, all passing).

## Notes

- The commit's `Toolbox-Run` trailer is the repository-side (child) run id. The org record links
  it as `verification.child_run`.
- Refused approval attempts are answered with 403 but are not appended to the run's `events`.
- Sign-in by `--as` is accepted only because the server listens on a loopback address. On any other
  address, set `TOOLBOX_AUTH_MODE` (`toolbox serve --help`).
