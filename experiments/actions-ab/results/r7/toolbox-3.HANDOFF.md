# Handoff: `mark-deprecated`

`mark-deprecated` is a self-service action that each repository declares in its own `.toolbox/`.
The org portal in this directory offers it for all three repositories. All three real runs were
requested by ann, approved by bob, and reported as `succeeded` only after Toolbox fetched each
origin's `main` and found `DEPRECATED.md` with the previewed content.

| Repository | Run id (org) | State | Commit on origin `main` |
|---|---|---|---|
| alpha | `run_2fucpdyfxxzp` | succeeded | `66b4134` |
| beta | `run_2fuctzyb77dh` | succeeded | `69146eb` |
| gamma | `run_2fucu4hb9r7e` | succeeded | `3d86ed7` |

Each of these runs also has a child run record, which is the repository's own view of the run
(`run_2fucrbxr2c9p`, `run_2fucuanvg9gq` and `run_2fucui8rnqnz`). The run history also holds
`run_2fud4um9rkt8`, which was a live rehearsal of a rejected push and ended `failed` (see R5).

Platform: Toolbox 0.1.1 (`/tmp/actions-ab/r7/toolbox-3/bin`). No glue code was needed. Nothing
runs in the background: everything works from the org checkout without a server.

## Setup (every shell)

```sh
export PATH=/tmp/actions-ab/r7/toolbox-3/bin:$PATH
export TOOLBOX_STATE_DIR=/tmp/actions-ab/r7/toolbox-3/state
cd /tmp/actions-ab/r7/toolbox-3/org
```

`TOOLBOX_STATE_DIR` matters. It holds the org portal's approval signing key
(`state/served/org/signing-key`), the org's clones of the three origins
(`state/federation/org-*/`) and every run record (`state/runs/<run id>.json`). The repositories
accept approvals only when they are signed with that key, which is pinned in their action files.
If you use another state directory, a new key is created, and every run is refused with
`child-refuses-approvals`.

## Files

| Path | What it is |
|---|---|
| `alpha/.toolbox/toolbox.yaml` (and the same file in `beta/` and `gamma/`) | The repository's own portal: its name, owner group, and `include: [actions/*.yaml]` |
| `<repo>/.toolbox/actions/mark-deprecated.yaml` | The action: inputs schema, approvers, precondition, the `change` executor, the runner, and the pinned org key. Byte-identical in all three repositories |
| `<repo>/.toolbox/tests/mark-deprecated.test.yaml` | 14 test cases, run by `toolbox test .toolbox` in a scratch copy of the repository and its origin |
| `org/toolbox.yaml` | The org portal: `federation.children` lists `../origins/{alpha,beta,gamma}.git` |
| `/tmp/actions-ab/r7/toolbox-3/state/runs/*.json` | The run records |

All of these files are committed to and pushed to their origin's `main`:
`origins/{alpha,beta,gamma}.git` and `origins/org.git`.

## Requirements

**R1: Owned by the repository.** The action is defined in `<repo>/.toolbox/actions/mark-deprecated.yaml`,
committed to `origins/<repo>.git` `main`, with the repository's own `.toolbox/toolbox.yaml`. The org
portal does not copy the action. It clones each origin's `main` and reads the action from there
again before every list, preview or run. If a team changes its file on `main`, the org portal
offers the changed action on the next command. Each repository's `federation.acceptApprovalsFrom`
also decides whose approvals it accepts.

**R2: Validated.** The `inputs` JSON Schema sets these rules:

- `successor` must match `pattern: ^[a-z0-9-]+/[a-z0-9-]+$`;
- `reason` must have `minLength: 20` and `maxLength: 200`;
- both inputs are `required`;
- `additionalProperties: false` refuses any other input.

A precondition, `!inputs.successor.endsWith('/' + entity.name)`, refuses the repository as its own
successor. Toolbox checks all of this before the preview. Invalid inputs exit 2 with the errors,
and no run is created. If the precondition fails, the preview lists it under `blockers` and gives no
confirm token. A `--yes` request then exits 1 with `mark-deprecated is blocked` and creates no run.
The test cases that cover this: a successor that is not owner/name, a successor without an owner,
a reason of 19 characters, a reason under 20 characters, a reason over 200, a missing reason, an
unknown input, and the repository as its own successor. Boundary cases of exactly 20 and exactly
200 characters are accepted.

**R3: Previewed.** `toolbox run mark-deprecated --entity portal:<repo> -i …` without `--yes` or
`--confirm` changes nothing. It prints the effects, approvers and a confirm token:

- the exact diff that creates `DEPRECATED.md`;
- the push of one commit to `main`.

The run record's `effects` keeps that preview. At run time, Toolbox fails a change that would differ
from the confirmed preview before it commits.

**R4: Approved.** The action sets `approval: "[['user:ann', 'user:bob']]"`: one of ann or bob must
approve, and the requester never counts. Approvals are held and signed by the org portal. These
attempts were refused live on `run_2fucpdyfxxzp`, each with exit 2:

- ann approving her own run: `error: a requester cannot approve their own run`;
- carol approving: `error: user:carol is not one of the pending approvers ["user:ann|user:bob"]`.

The repository checks the org's signature against the key in its action file. A run requested in
a repository checkout, rather than through the org portal, stops at exit 3, because a repository
holds no approvals. The test case "ann cannot approve her own run, and carol is not an approver"
covers this, and "bob requests and ann approves" covers the reverse.

**R5: Done means observed.** The executor is `type: change`, `land: push`, `base: main`. After the
push, Toolbox fetches `origin/main`, checks that the commit is on it, and compares the blob of
every changed file with the preview. That check is the `verification` block in the record, for
example `"checked": "fetched origin/main: the commit is on it, and every file it changed has the
content the preview showed"`, with `files[].blob` and `matches: true`. Only then is the run
`succeeded`.

A rejected push gives `failed` (exit 1) and includes the origin's message. This was rehearsed live:
I added a `pre-receive` hook that exited 1 to `origins/alpha.git`, then ran `run_2fud4um9rkt8`. It
ended `failed`, `main` was unchanged, and I removed the hook afterwards. The test case "a push the
origin rejects fails the run" covers this too. If you rerun with the same inputs on a repository
whose `main` already has that exact `DEPRECATED.md`, the run succeeds without a commit
(`outputs.changed: false`), because the observed state is already correct. To test a rejected push,
use a different `reason`.

**R6: One place for the org.** `org/toolbox.yaml` (pushed to `origins/org.git`) federates the
three origins. From `org/`, `toolbox run` lists `mark-deprecated` on `portal:alpha`,
`portal:beta` and `portal:gamma`. Preview, request, approve, status and record all work from there
with `--entity portal:<repo>`. Toolbox runs the child's action in its clone of that repository
(`federation.runner: {type: local}` in the action file).

**R7: Audited.** `toolbox run --verify <run id> -o json` prints the record, and `toolbox run --runs`
lists all runs. The records are stored in `state/runs/<id>.json`. Each record has these fields:

- `requested_by`;
- `approval.approved_by` and `approval.satisfied[].by`;
- `inputs`;
- `state`, and `error` on failure;
- `effects`, which is the preview;
- `outputs.commit`;
- `verification`;
- `events[]`, each with `state`, `by`, `note` and an `at` timestamp (`previewed`, `confirmed`,
  `awaiting_approval`, `approved`, `dispatched`, `running`, `executor_succeeded`,
  `succeeded`/`failed`).

The `child` record shows the repository's side of the run. The commit on `main` also carries
`Toolbox-Run`, `Requested-By` and `Approved-By` trailers, so `git log` on the origin is a second
audit trail.

**R8: Usable by agents.** Every command is non-interactive and takes `-o json`. The exit codes are
stable:

| Exit code | Meaning |
|---|---|
| 0 | succeeded |
| 1 | failed or stale |
| 2 | invalid input or refused (including refused approvals) |
| 3 | awaiting approval |
| 4 | unverified |

`--as` can be replaced with `TOOLBOX_AS`. `--wait` blocks until the run settles. The operator
skills are in `toolbox-docs/skills/run` and `toolbox-docs/skills/add-action`.

## Commands

Run these from `org/`, with the setup above. Nothing needs to be started.

```sh
# List every repository's actions, with their inputs and approvers
toolbox run --as ann -o json

# Preview. This changes nothing and prints the diff and a confirm token
toolbox run --as ann mark-deprecated --entity portal:alpha \
  -i successor=acme/dependabot-proxy-v2 \
  -i 'reason=Replaced by the v2 proxy, which supports OIDC registries.' -o json

# Request as ann. Exits 3 (awaiting approval) and prints the record; its "id" is the run id
toolbox run --as ann mark-deprecated --entity portal:alpha \
  -i successor=acme/dependabot-proxy-v2 \
  -i 'reason=Replaced by the v2 proxy, which supports OIDC registries.' --yes -o json
# (Alternatively, confirm the preview's token: ... --confirm ct_… -o json)

# Approve as bob. Pushes, verifies on main, and exits 0 when succeeded
toolbox run --as bob --approve <run-id> --note "checked the diff" --wait -o json

# Check the status and read the record (who, when, inputs, outcome, verification)
toolbox run --as ann --verify <run-id> -o json        # exit code = state; .state in the JSON
toolbox run --as ann --runs -o json                   # every run

# Optional: reject instead of approving
toolbox run --as bob --reject <run-id> --note "why" -o json
```

Negative tests that you can run yourself:

```sh
toolbox run --as ann mark-deprecated --entity portal:beta -i successor=Bad_Name -i reason=short -o json      # exit 2, errors listed
toolbox run --as ann mark-deprecated --entity portal:beta -i successor=acme/beta -i 'reason=Its own successor, which is blocked.' -o json   # preview lists "blockers", no confirm_token
toolbox run --as ann mark-deprecated --entity portal:beta -i successor=acme/beta -i 'reason=Its own successor, which is blocked.' --yes -o json   # exit 1: "mark-deprecated is blocked", no run created
toolbox run --as ann --approve <run-id-requested-by-ann>      # exit 2: a requester cannot approve their own run
toolbox run --as carol --approve <run-id>                     # exit 2: not an approver
# Rejected push: make the origin refuse, request and approve with a NEW reason, then remove the hook
printf '#!/bin/sh\necho "main is protected" >&2\nexit 1\n' > ../origins/alpha.git/hooks/pre-receive
chmod +x ../origins/alpha.git/hooks/pre-receive
#   ... request as ann, approve as bob: state "failed", exit 1, main unchanged ...
rm ../origins/alpha.git/hooks/pre-receive
```

To rerun every case offline, in a scratch copy that leaves the real origin untouched:

```sh
cd ../alpha && toolbox validate .toolbox && toolbox test .toolbox
```

The same works in `beta` and `gamma`.

## Optional: serve it

You don't need a server. If you want the UI, REST or MCP for approvers elsewhere, run
`toolbox serve /tmp/actions-ab/r7/toolbox-3/org --listen 127.0.0.1:7890 --detach` with the same
`TOOLBOX_STATE_DIR`. Then add `--url http://127.0.0.1:7890` to the same commands, and stop the
server with `toolbox serve /tmp/actions-ab/r7/toolbox-3/org --stop`. A served portal keeps its runs
separate from the checkout's, so approve a run where it was requested. I did not use a server for
the runs above, and none is running.

## Note on content

`DEPRECATED.md` is exactly `# Deprecated: use <successor>`, a blank line, `<reason>`, and a final
newline.
