# Handoff: `mark-deprecated` on alpha, beta and gamma

`mark-deprecated` is a Toolbox self-service action (Toolbox 0.1.1). Each repository declares it in
its own `.toolbox/`. The org portal in this directory offers all three from one served place,
where runs are requested, approved and checked. All three requested runs have succeeded:

| Repository | Run (org portal) | Commit on origin `main` | Requested by | Approved by |
|---|---|---|---|---|
| alpha | `run_2fu0gl4iwkgn` | `abc3601` | user:ann | user:bob |
| beta | `run_2fu0gsy9rwxf` | `db7a13a` | user:ann | user:bob |
| gamma | `run_2fu0h09oyvit` | `58a8594` | user:ann | user:bob |

The run history also holds `run_2fu0thzrc0s2`, state `failed`. It was a live rehearsal of a
rejected push on alpha, using a temporary `pre-receive` hook that has since been removed. Nothing
landed from it.

## Files

| Path | What it is |
|---|---|
| `<repo>/.toolbox/toolbox.yaml` (alpha, beta, gamma) | The repository's portal: its name and owner (`group:<repo>-maintainers`), and `include: [actions/*.yaml]` |
| `<repo>/.toolbox/actions/mark-deprecated.yaml` | The action: inputs schema, approvers, precondition, the `change` executor, and the org portal's approval key it accepts. It is the same file in all three repositories |
| `<repo>/.toolbox/tests/mark-deprecated.test.yaml` | 14 cases that `toolbox test .toolbox` runs in a scratch copy of the repository and its origin. It is the same file in all three |
| `org/toolbox.yaml` | The org portal. `federation.children` lists `../origins/{alpha,beta,gamma}.git` |
| `org/state/` (gitignored) | `TOOLBOX_STATE_DIR` for the org server: its signing key, run records, pid, log and its clones of the children |
| `org/state/served/org/runs/<run id>.json` | One audit record per run |
| `org/state/served/org/signing-key` | The key whose public half each repository pins. **Keep this state directory:** a server started with a different `TOOLBOX_STATE_DIR` creates a new key, and the repositories refuse its approvals (`child-refuses-approvals`) |

`<repo>` is `/tmp/actions-ab/r6/toolbox-3/<repo>`. The `.toolbox/` files are committed on each
origin's `main`, and `org/toolbox.yaml` is committed on `origins/org.git`'s `main`.

## Requirements

| # | How it is met |
|---|---|
| R1 Owned by the repository | The action is declared in `<repo>/.toolbox/actions/mark-deprecated.yaml` and committed to `origins/<repo>.git` `main` (alpha `44edcbd`, beta `b1d8c08`, gamma `c48d577`). The org portal doesn't copy it: the server keeps a clone of each origin and re-reads `main` before it lists, previews or runs an action, so a change the team pushes is what gets offered. Approvals count only when signed with the key the repository pins in `federation.acceptApprovalsFrom`, so the repository also decides whose approvals it trusts |
| R2 Validated | `inputs` in the action file is a JSON Schema: `successor` must match `^[a-z0-9-]+/[a-z0-9-]+$`, `reason` must be 20 to 200 characters, both are required, and `additionalProperties: false` rejects anything else. Toolbox checks the schema before it previews, so invalid inputs get `422 invalid-inputs` with exit 2 and nothing changes. A precondition (`!inputs.successor.endsWith('/' + entity.name)`) blocks a repository from naming itself as its successor. Covered by the test cases `a successor that is not owner/name…`, `…upper case…`, `…third path part…`, `a reason under 20…`, `…over 200…`, `a missing reason…`, `an unknown input…` and `the repository cannot be its own successor` |
| R3 Previewed | `toolbox run … mark-deprecated --entity portal:<repo> -i …` without `--yes` prints the effects and changes nothing. The effects are the exact `DEPRECATED.md` diff and "push branch:main, once approved", plus the required approvers. Every run's record keeps that preview under `effects`. Before it commits, Toolbox also refuses to land a change that differs from the confirmed preview |
| R4 Approved | `approval: "[['user:ann', 'user:bob']]"` in the action file means one of ann or bob is needed, and the requester never counts. On the server, a requester approving their own run gets `403 not-an-approver: a requester cannot approve their own run` and carol gets `403 not-an-approver: user:carol is not one of the pending approvers`. Both exit 2, and the run stays `awaiting_approval`. The org portal signs each approval, and the repository checks the signature against its pinned key. Covered by the test cases `ann cannot approve her own run, and carol is not an approver`, `carol may request, but only a maintainer approves` and `bob requests, ann approves`, and checked live on `run_2fu0gl4iwkgn` and `run_2fu0thzrc0s2` |
| R5 Done means observed | `executor: {type: change, land: push, base: main}`. After pushing, Toolbox fetches `origin/main` and checks that the commit is on it and that every file has the previewed blob. Only then does it set `succeeded`; see `verification.child` in a record, for example `"checked": "fetched origin/main: the commit is on it, and every file it changed has the content the preview showed"`. A rejected push sets `failed`, with the origin's message, and exits 1. Covered by the test case `a push the origin rejects fails the run with the origin's message`, and rehearsed live as `run_2fu0thzrc0s2` |
| R6 One place for the org | `org/toolbox.yaml`, served from `/tmp/actions-ab/r6/toolbox-3/org` on `http://127.0.0.1:7690`. `toolbox run` lists `mark-deprecated` on `portal:alpha`, `portal:beta` and `portal:gamma`, and the same command with `--entity portal:<repo>` previews or requests a run on any of them. `toolbox run --runs` lists every run |
| R7 Audited | Each run's record (`toolbox run --verify <id> -o json`, stored at `org/state/served/org/runs/<id>.json`) has `requested_by`, `inputs`, `approval.approved_by` / `approval.satisfied`, `state`, `verification` and `events`. `events` lists each step (`previewed`, `confirmed`, `awaiting_approval`, `approved` with the approver and note, `running`, `succeeded` or `failed` with the error), each with `at` (UTC) and `by`. The landed commit carries the `Toolbox-Run`, `Requested-By` and `Approved-By` trailers too |
| R8 Usable by agents | Every step is a non-interactive `toolbox run` command, and `-o json` gives JSON. Exit codes: 0 succeeded, 1 failed, 2 invalid or refused, 3 awaiting approval, 4 unverified. `TOOLBOX_URL` and `TOOLBOX_AS` can replace `--url` and `--as` |

## Commands

Set up the shell once:

```sh
export PATH=/tmp/actions-ab/r6/toolbox-3/bin:$PATH
export TOOLBOX_STATE_DIR=/tmp/actions-ab/r6/toolbox-3/org/state   # must be this directory: it holds the pinned key
export TOOLBOX_URL=http://127.0.0.1:7690
cd /tmp/actions-ab/r6/toolbox-3/org
```

Start the org portal; stop it when you are done:

```sh
toolbox serve . --listen 127.0.0.1:7690 --detach
# prints: approval key: ed25519:ce2ca47940ce9ac4f712ee2fc7b8c9ea1cc33bf08dfcdde98997bd5d978cdf5e  (must match the repositories' pin)
toolbox serve . --stop
```

On this loopback address, `--as <user>` is the sign-in (dev mode, no passwords), and the server
records it.

List, preview, request, approve, check, read:

```sh
toolbox run --as ann -o json                                         # list: mark-deprecated on portal:alpha, portal:beta, portal:gamma

toolbox run --as ann mark-deprecated --entity portal:alpha \
  -i successor=acme/dependabot-proxy-v2 \
  -i "reason=Replaced by the v2 proxy, which supports OIDC registries." -o json      # preview: diff + approvers, changes nothing (exit 0)

toolbox run --as ann mark-deprecated --entity portal:alpha \
  -i successor=acme/dependabot-proxy-v2 \
  -i "reason=Replaced by the v2 proxy, which supports OIDC registries." --yes -o json  # request: exit 3, prints the record; .id is the run id

toolbox run --as bob --approve <run-id> --note "reviewed" --wait -o json            # approve and wait: exit 0 when succeeded, 1 when failed

toolbox run --as ann --verify <run-id> -o json                                      # status and full record (.state, .requested_by, .approval, .inputs, .events, .verification)
toolbox run --as ann --runs -o json                                                 # every run
```

To pass inputs as JSON, use `--inputs '{"successor":"acme/x","reason":"..."}'` instead of `-i`.

## Testing refusals

| Try | Command | Expected |
|---|---|---|
| Invalid inputs | `toolbox run --as ann mark-deprecated --entity portal:beta -i successor=Bad -i reason=short -o json` | `422 invalid-inputs`, exit 2, no run created |
| Self as successor | `toolbox run --as ann mark-deprecated --entity portal:beta -i successor=acme/beta -i "reason=Twenty characters or more." --yes -o json` | the preview lists `blockers: ["precondition not met: …"]`; with `--yes` the request is refused with exit 2 and no run is created |
| Self-approval | request as ann (`--yes`), then `toolbox run --as ann --approve <id> -o json` | `403 not-an-approver: a requester cannot approve their own run`, exit 2, still `awaiting_approval` |
| carol approves | `toolbox run --as carol --approve <id> -o json` | `403 not-an-approver`, exit 2 |
| Rejected push | see below | `failed` with the origin's message, exit 1, `main` unchanged |

To make the origin reject a push, add a hook, run the test, then remove the hook:

```sh
printf '#!/bin/sh\necho "main is protected" >&2\nexit 1\n' > /tmp/actions-ab/r6/toolbox-3/origins/beta.git/hooks/pre-receive
chmod +x /tmp/actions-ab/r6/toolbox-3/origins/beta.git/hooks/pre-receive
toolbox run --as ann mark-deprecated --entity portal:beta -i successor=acme/other-store \
  -i "reason=Testing a push that the origin refuses." --yes -o json   # exit 3
toolbox run --as bob --approve <run-id> --wait -o json                 # exit 1, state failed, events[-1].note has "main is protected"
rm /tmp/actions-ab/r6/toolbox-3/origins/beta.git/hooks/pre-receive
```

Use inputs that differ from the content already on `main`. A run whose content `main` already has
succeeds without a commit (`outputs.changed: false`), because the desired state is already
observed, so it never pushes.

The same cases run offline, without a server and without touching the real origins:

```sh
cd /tmp/actions-ab/r6/toolbox-3/alpha && toolbox validate .toolbox && toolbox test .toolbox   # 14 passed, 0 failed (same for beta and gamma)
```
