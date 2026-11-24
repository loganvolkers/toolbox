# Handoff: `mark-deprecated`

`mark-deprecated` is a Toolbox self-service action. Each of `alpha`, `beta` and `gamma` declares it in
its own repository. The org portal in this directory offers it for all three. There, people and agents
list, preview, request, approve and audit runs. Nothing needs to be running: every step is a
`toolbox run` command in this directory, with `-o json` for machine-readable output. A server is
optional (see *Optional: serve it*).

The three requested runs succeeded. Each was requested by ann and approved by bob, and Toolbox observed
`DEPRECATED.md` on the origin's `main`:

| Repository | Org run id | Commit on `origins/<repo>.git` `main` |
|---|---|---|
| alpha | `run_2fu88ifjltwa` | `d3cb7cf` mark-deprecated for portal:alpha |
| beta | `run_2fu8ekfk7p7u` | `d7a29fe` mark-deprecated for portal:beta |
| gamma | `run_2fu8eoqroal9` | `6e3d3ab` mark-deprecated for portal:gamma |

Each commit carries `Toolbox-Run`, `Requested-By: user:ann` and `Approved-By: user:bob` trailers
(`git -C ../origins/alpha.git log -1 main`). Run `run_2fu8ooutdib7` is a live rejected-push test on
alpha. Its state is `failed`, and `main` was left unchanged.

## Files

| Path | What it is |
|---|---|
| `<repo>/.toolbox/toolbox.yaml` | The repository's portal: its name, its owner (`group:<repo>-maintainers`), `include: [actions/*.yaml]` |
| `<repo>/.toolbox/actions/mark-deprecated.yaml` | The action: inputs schema, approvers, precondition, the change it makes, and the pinned org key. Byte-identical in all three repositories |
| `<repo>/.toolbox/tests/mark-deprecated.test.yaml` | 11 test cases, run by `toolbox test .toolbox` in a scratch copy of the repository and its origin |
| `org/toolbox.yaml` | The org portal: `federation.children` lists `../origins/{alpha,beta,gamma}.git` |
| `org/env.sh` | Sets `PATH` and `TOOLBOX_STATE_DIR`, and unsets `TOOLBOX_URL` so commands use this checkout. Source it before every command |
| `/tmp/actions-ab/r7/toolbox-2/.toolbox-state/` | `TOOLBOX_STATE_DIR`: the org's signing key, its clones of the repositories, and run records (`federation/…/runs/`). Not in git |

`<repo>` is `/tmp/actions-ab/r7/toolbox-2/{alpha,beta,gamma}`. All of these files are committed to
the matching `origins/<repo>.git` `main`. The `org` files are committed to `origins/org.git` `main`.

## How each requirement is met

| # | How | Where |
|---|---|---|
| R1 Owned by the repository | Each repository commits its own copy of the action to its origin's `main`. The org portal re-reads every child from its origin's `main` before it lists, previews or runs. So a change a team pushes to its own `.toolbox/actions/mark-deprecated.yaml` takes effect from the org without any change to `org/`. The repository's portal owner is `group:<repo>-maintainers`, and the approvers (`user:ann`, `user:bob`) are declared in the repository's file | `<repo>/.toolbox/toolbox.yaml`, `<repo>/.toolbox/actions/mark-deprecated.yaml`, `org/toolbox.yaml` |
| R2 Validated | `inputs` is a JSON Schema: both inputs `required`, `additionalProperties: false`, `successor` with `pattern: ^[a-z0-9-]+/[a-z0-9-]+$`, `reason` with `minLength: 20, maxLength: 200`. Toolbox checks it before the preview, and invalid inputs exit 2 with `errors`. The precondition `!inputs.successor.endsWith('/' + entity.name)` blocks a repository naming itself as its successor. Requesting a blocked run exits 1 and records nothing | `actions/mark-deprecated.yaml` (`inputs`, `preconditions`). Test cases: *a successor that is not owner/name*, *a successor without an owner*, *a reason shorter than 20*, *longer than 200*, *exactly 200 is accepted*, *a missing reason*, *the repository cannot name itself* |
| R3 Previewed | A run without `--yes`/`--confirm` is only a preview. It prints the diff of `DEPRECATED.md`, the push to `main`, the approvers and a confirm token, and it changes nothing. `--yes` records the same preview as the run's `effects` | `executor: {type: change, land: push, base: main}` in `actions/mark-deprecated.yaml` |
| R4 Approved | `approval: "[['user:ann', 'user:bob']]"`: one of ann or bob, and the requester never counts. The org portal holds the approval and signs it with its key. Each repository accepts only approvals signed with the key it pins in `federation.acceptApprovalsFrom`. Self-approval fails with `a requester cannot approve their own run`, and carol fails with `user:carol is not one of the pending approvers`. Both exit 2, and the run stays `awaiting_approval` | `actions/mark-deprecated.yaml` (`approval`, `federation.acceptApprovalsFrom`), `.toolbox-state/` (key). Test case: *ann cannot approve her own run, and carol is not an approver*. Also checked live on `run_2fu88ifjltwa` before bob approved |
| R5 Done means observed | After the push, Toolbox fetches the origin's `main` and checks two things: its commit is there, and each changed file's blob matches the confirmed preview. Only then is the state `succeeded`. The record's `verification` shows `fetched origin/main: the commit is on it, and every file it changed has the content the preview showed`. A rejected push makes the run `failed` (exit 1), with the origin's message | Test case: *a push the origin rejects fails the run*. Live: `run_2fu8ooutdib7` |
| R6 One place for the org | `org/toolbox.yaml` federates the three origins. In `org/`, `toolbox run` lists `mark-deprecated` on `portal:alpha`, `portal:beta` and `portal:gamma`, and `--entity portal:<repo>` previews and requests a run on any of them | `org/toolbox.yaml` |
| R7 Audited | Each run's record (`--verify <id> -o json`) contains `requested_by`, `approval.approved_by`, `inputs`, `state`, `outputs.commit`, `verification`, and `events`. Each event has `state`, `by` and `at`: previewed, confirmed, awaiting_approval, approved (with bob's `--note`), dispatched, running, succeeded or failed. The child repository's run is nested under `child`. The commit on `main` also carries the trailers | `.toolbox-state/federation/…/runs/*.json` |
| R8 Usable by agents | Every step is one non-interactive `toolbox run` command. With `-o json` it prints JSON. Exit codes: 0 succeeded, 1 failed/blocked/stale, 2 invalid or refused, 3 awaiting approval, 4 unverified. `--as` (or `TOOLBOX_AS`) says who is acting | Commands below |

## Commands

Run these from the org directory, with the environment set:

```sh
cd /tmp/actions-ab/r7/toolbox-2/org && . ./env.sh
```

**Start:** nothing needs to be running.

**List** the actions for all three repositories:

```sh
toolbox run --as ann -o json
```

**Preview.** This changes nothing, and prints the diff and a confirm token:

```sh
toolbox run --as ann mark-deprecated --entity portal:alpha \
  -i successor=acme/dependabot-proxy-v2 \
  -i "reason=Replaced by the v2 proxy, which supports OIDC registries." -o json
```

**Request** as ann. It exits 3 (awaiting approval) and prints the record, whose `id` is the run id.
Either confirm the token from the preview with `--confirm ct_…` within 10 minutes and the same inputs,
or preview and request in one step with `--yes`:

```sh
toolbox run --as ann mark-deprecated --entity portal:alpha \
  -i successor=acme/dependabot-proxy-v2 \
  -i "reason=Replaced by the v2 proxy, which supports OIDC registries." --yes -o json
```

**Approve** as bob. This pushes, then verifies on the origin's `main`. It exits 0 when succeeded and 1
when failed:

```sh
toolbox run --as bob --approve <run-id> --note "checked the preview" --wait -o json
```

**Reject** as bob:

```sh
toolbox run --as bob --reject <run-id> --note "why" -o json
```

**Check a run's status and read its record** (`.state`, `.requested_by`, `.approval.approved_by`,
`.inputs`, `.events[].at`, `.verification`):

```sh
toolbox run --as ann --verify <run-id> -o json
```

**List every run:**

```sh
toolbox run --as ann --runs -o json
```

For the other repositories, use `--entity portal:beta` or `--entity portal:gamma`.

## Testing it

Each repository's test cases cover every refusal and a rejected push, without touching the real
origins:

```sh
cd /tmp/actions-ab/r7/toolbox-2/alpha && . ../org/env.sh && toolbox validate .toolbox && toolbox test .toolbox   # 11 passed
```

Live, from `org/`:

| Scenario | Command | Expected |
|---|---|---|
| Invalid inputs | `toolbox run --as ann mark-deprecated --entity portal:alpha -i successor=Bad_Name -i reason=short -o json` | exit 2, `errors` for both inputs. Also with `--yes` |
| Successor is the repository itself | `... --entity portal:beta -i successor=acme/beta -i "reason=Pointing at itself should be blocked." --yes -o json` | exit 1, `mark-deprecated is blocked: precondition not met`, no run recorded |
| Self-approval | request as ann (`--yes`), then `toolbox run --as ann --approve <id> -o json` | exit 2, `a requester cannot approve their own run`. Still `awaiting_approval` |
| carol approves | `toolbox run --as carol --approve <id> -o json` | exit 2, `user:carol is not one of the pending approvers`. Still `awaiting_approval` |
| Rejected push | See below | exit 1, state `failed`, the origin's message in `error`. `main` unchanged |

To test a rejected push, make the origin refuse pushes, request a run, have bob approve it, then
restore the origin:

```sh
H=../origins/alpha.git/hooks/pre-receive
printf '#!/bin/sh\necho "main is protected" >&2\nexit 1\n' > $H && chmod +x $H
ID=$(toolbox run --as ann mark-deprecated --entity portal:alpha -i successor=acme/other-repo \
  -i "reason=Testing a push that the origin rejects." --yes -o json | python3 -c 'import json,sys; print(json.load(sys.stdin)["id"])')
toolbox run --as bob --approve $ID --wait -o json; echo "exit $?"    # failed, exit 1
rm $H
```

Use inputs whose content differs from the `DEPRECATED.md` already on `main`. If a run's content is
already on `main`, it commits nothing, never pushes, and succeeds with `outputs.changed: false`.

Keep `TOOLBOX_STATE_DIR` as `env.sh` sets it. A different state directory makes a new signing key, and
the repositories would then refuse its approvals (`child-refuses-approvals`). A new org key must be
pinned in every repository's `actions/mark-deprecated.yaml`, then committed and pushed.

## Optional: serve it

The same portal can be served with a UI, REST and MCP, for approvers elsewhere. It uses the same key.
Tested on 127.0.0.1:7850 and then stopped. A served portal keeps its runs apart from the checkout's, so
approve a run where it was requested.

```sh
cd /tmp/actions-ab/r7/toolbox-2 && . org/env.sh
toolbox serve org --listen 127.0.0.1:7850 --detach
export TOOLBOX_URL=http://127.0.0.1:7850   # every toolbox run command above then goes through the server
toolbox serve org --stop
```
