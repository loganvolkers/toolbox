# Handoff: `mark-deprecated`

`mark-deprecated` is a Toolbox self-service action. Each repository declares and owns it, and
the org portal in this directory lists it, previews it, takes requests, holds approvals and runs
it. All three runs requested by ann and approved by bob succeeded, and `DEPRECATED.md` is on each
origin's `main`:

| Repository | Org run | Commit on origin `main` (trailers `Toolbox-Run`, `Requested-By: user:ann`, `Approved-By: user:bob`) |
|---|---|---|
| alpha | `run_2ft987uf1kao` | `mark-deprecated: use acme/dependabot-proxy-v2 (run run_2ft99njkwzdb)` |
| beta | `run_2ft9c88vhae6` | `mark-deprecated: use acme/valtio-store (run run_2ft9clji4hkq)` |
| gamma | `run_2ft9cf5hfvgb` | `mark-deprecated: use acme/graphify-core (run run_2ft9dgdubmc8)` |

No glue code was needed. Toolbox (`/tmp/actions-ab/r5/toolbox-1/bin/toolbox`, 0.1.1) validates
the inputs, previews the diff, holds the approval, commits and pushes, then fetches `main` to
verify the result. It also records the run. The only code we wrote is the one-line `printf` that
writes the file.

## Files

| Path | What it is |
|---|---|
| `alpha/.toolbox/toolbox.yaml`, `beta/.toolbox/toolbox.yaml`, `gamma/.toolbox/toolbox.yaml` | Each repository's definition of `mark-deprecated`, committed to its origin's `main` |
| `<repo>/.toolbox/tests/mark-deprecated.test.yaml` | 14 lifecycle cases per repository, run by `toolbox test .toolbox` |
| `org/toolbox.yaml` | The org portal. It lists the three origins as `federation.children` (`portal:alpha`, `portal:beta`, `portal:gamma`) |
| `/tmp/actions-ab/r5/toolbox-1/.state/` | `TOOLBOX_STATE_DIR`: the server's clones of the children (`federation/`), its approval signing key, pid and log (`served/org/`), the org run records (`served/org/runs/<run id>.json`) and the child run records (`runs/<child run id>.json`) |

## Requirements

| # | How it is met |
|---|---|
| R1 Owned by the repository | The action is declared in `<repo>/.toolbox/toolbox.yaml` and committed to `origins/<repo>.git` `main`, with `owner: group:<repo>-maintainers` and `approval: [['user:ann', 'user:bob']]`. The org portal holds no copy of it: before it lists, previews or runs an action, the server fetches each child's `main` again. A commit to a repository's `.toolbox/toolbox.yaml` therefore changes what the org offers, and the run record pins the version it ran (`action_version`) |
| R2 Validated | `inputs` in each manifest is a JSON Schema. `successor` must match `^[a-z0-9-]+/[a-z0-9-]+$`; `reason` must be 20 to 200 characters; both are required; `additionalProperties: false` refuses unknown inputs. A precondition refuses the repository itself as its own successor (`!inputs.successor.endsWith('/<repo>')`). Invalid inputs fail with `422 invalid-inputs` and exit 2 before any preview, so nothing changes. The test file covers each case, including the 19/20 and 200/201 character boundaries |
| R3 Previewed | A request without `--yes` or `--confirm` is a preview. It prints the exact diff of `DEPRECATED.md`, says that `main` gets one commit once approved (never forced), and names the approvers. It changes nothing. The `--yes` request records the same preview in the run (`effects`) |
| R4 Approved | The `approval` CEL in each manifest needs ann or bob, and the requester never counts. In the live check, ann approving her own run returned `403 not-an-approver: a requester cannot approve their own run`, and carol got `403 not-an-approver: user:carol is not one of the pending approvers`; both exit 2. The org server signs each approval with its key, and each child accepts only that key (`federation.acceptApprovalsFrom` in `<repo>/.toolbox/toolbox.yaml`). A run with no valid approval therefore cannot run, even inside the repository's own portal |
| R5 Done means observed | The executor is `type: change`, `land: push`, `base: main`. Once the run is approved, Toolbox commits in a fresh worktree of `main` and pushes. It then fetches `origin/main` and checks that the commit is on it and that `DEPRECATED.md` has the blob the preview showed (`verification.child.files[].matches`). Only then is the run `succeeded`. A rejected push fails the run (`state: failed`, exit 1) with the origin's message. This was rehearsed against a scratch copy of alpha's origin with a rejecting `pre-receive` hook, through a served org portal, and it is the `rejectPush` case in each test file |
| R6 One place for the org | From `org/`, `toolbox run --url http://127.0.0.1:7410 --as <who>` lists `mark-deprecated` for `portal:alpha`, `portal:beta` and `portal:gamma`. Requests, approvals, status and records all go through the same server; `--runs` lists every run in the org |
| R7 Audited | Each run record (`toolbox run --verify <id> -o json`, kept in `.state/served/org/runs/<id>.json`) has `requested_by`, `inputs`, `approval.approved_by`, `state`, `outputs.commit`, `verification`, and `events`. Each event has `at` (UTC), `state` (`previewed`, `confirmed`, `awaiting_approval`, `approved`, `dispatched`, `running`, `executor_succeeded`, then `succeeded` or `failed`), and, where a person acted, `by` and `note`. The commit on `main` also carries `Toolbox-Run`, `Requested-By` and `Approved-By` trailers |
| R8 Usable by agents | Every step is one non-interactive `toolbox run` command that takes `-o json`. Exit codes: 0 succeeded, 1 failed, 2 invalid or refused, 3 awaiting approval, 4 unverified. `TOOLBOX_URL` and `TOOLBOX_AS` can replace `--url` and `--as` |

## Commands

Run these from `/tmp/actions-ab/r5/toolbox-1/org`. `TOOLBOX_STATE_DIR` must point at `.state`,
because that is where the signing key the repositories pin is kept. A server started with a
different state directory makes a new key, and the children then refuse its approvals
(`child-refuses-approvals`).

```sh
cd /tmp/actions-ab/r5/toolbox-1/org
export PATH=/tmp/actions-ab/r5/toolbox-1/bin:$PATH
export TOOLBOX_STATE_DIR=/tmp/actions-ab/r5/toolbox-1/.state
export TOOLBOX_URL=http://127.0.0.1:7410

# Start the server (returns once ready; prints the approval key ed25519:5d6580ad…2512)
toolbox serve . --listen 127.0.0.1:7410 --detach

# List the actions of all three repositories
toolbox run --as ann -o json

# Preview (changes nothing; prints the diff, the approvers and a confirm token)
toolbox run --as ann mark-deprecated --entity portal:alpha \
  -i successor=acme/dependabot-proxy-v2 \
  -i "reason=Replaced by the v2 proxy, which supports OIDC registries." -o json

# Request as ann: exit 3 (awaiting approval); the JSON's "id" is the run id
toolbox run --as ann mark-deprecated --entity portal:alpha \
  -i successor=acme/dependabot-proxy-v2 \
  -i "reason=Replaced by the v2 proxy, which supports OIDC registries." --yes -o json
#   (or confirm a preview instead: append --confirm <ct_… token> to the preview command)

# Approve as bob and follow the run until it ends: exit 0 once DEPRECATED.md is seen on main
toolbox run --as bob --approve <run-id> --note "checked the preview" --wait -o json

# Check a run's status (exit code and "state") and read its full record
toolbox run --as ann --verify <run-id> -o json

# Every run in the org
toolbox run --as ann --runs -o json

# Stop the server
toolbox serve . --stop
```

Each repository's definition can be checked without a server, in a scratch copy that never
touches the real origin:

```sh
cd /tmp/actions-ab/r5/toolbox-1/alpha && toolbox validate .toolbox && toolbox test .toolbox
```

## Testing the refusals

| Case | Command | Expected |
|---|---|---|
| Invalid successor or reason | `toolbox run --as ann mark-deprecated --entity portal:beta -i successor=Acme/X -i reason=short` | `422 invalid-inputs`, naming both inputs; exit 2 |
| Own repository as successor | `... --entity portal:beta -i successor=acme/beta -i "reason=Twenty or more characters here."` | Preview reports the precondition as a blocker; a `--yes` request is refused with exit 2 |
| Self-approval | `toolbox run --as ann --approve <ann's run id>` | `403 not-an-approver: a requester cannot approve their own run`; exit 2 |
| carol approves | `toolbox run --as carol --approve <run id>` | `403 not-an-approver`; exit 2 |
| carol requests | `toolbox run --as carol mark-deprecated ... --yes` | carol may request, and the run waits for ann or bob, as for anyone else |
| Rejected push | Add a rejecting hook (`printf '#!/bin/sh\necho "main is protected" >&2\nexit 1\n' > /tmp/actions-ab/r5/toolbox-1/origins/alpha.git/hooks/pre-receive && chmod +x $_`), request with **different** inputs than the existing `DEPRECATED.md` (for example `successor=acme/other-proxy`), and approve as bob | `failed`, exit 1, with the origin's `main is protected` message in the last event; `main` is unchanged. Remove the hook afterwards |

Note on the rejected-push test: alpha, beta and gamma already have `DEPRECATED.md` on `main`. If
a run asks for exactly the content `main` already has, Toolbox has nothing to commit. It observes
the file on `main` and reports `succeeded` with `outputs.changed: false`, which meets R5 because
the content was observed on `main`. To exercise a rejected push, use inputs that change the file.
