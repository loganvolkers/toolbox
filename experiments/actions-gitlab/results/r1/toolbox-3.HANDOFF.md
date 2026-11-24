# HANDOFF: `set-codeowners` on acme-w15

`set-codeowners` is a Toolbox self-service action. Each project (`acme-w15/alpha`, `beta`, `gamma`)
owns its definition in its own `.toolbox/`. This project, `acme-w15/org`, is the org portal that
lists them, holds approvals and runs them. People prove who they are with their own GitLab token.
The release group is read live from GitLab on every command. **Nothing has to be running:** every
step is a `toolbox run` command in this directory, and no server is involved.

The three requested runs succeeded and were verified on each project's `main`:

| Project | Run (org) | pattern | owners | Requested by | Approved by |
|---|---|---|---|---|---|
| alpha | `run_2g07830fjreq` | `*` | `@acme/proxy-maintainers` | user:ann | user:dave |
| beta | `run_2g07ex2eixef` | `/docs/` | `@acme/docs @carol` | user:ann | user:erin |
| gamma | `run_2g07fgsw5mva` | `/graphify/security.py` | `@acme/security` | user:ann | user:erin |

## Files

| Path | What it is |
|---|---|
| `org/toolbox.yaml` | The org portal. It has three parts. The `gitlab` connector reads `acme-w15` and its subgroups with their direct members, and `authz` turns them into `group:` memberships. `forges` makes GitLab the identity source (`identity: true`). `federation.children` lists the three projects |
| `org/groups.fga` | The authorization model: users and groups, with group membership |
| `<project>/.toolbox/toolbox.yaml` | The project's portal: its name and `include: [actions/*.yaml]` |
| `<project>/.toolbox/actions/set-codeowners.yaml` | The action: its inputs schema, approval rule, executor, and the org key the project accepts approvals from |
| `<project>/.toolbox/tests/set-codeowners.test.yaml` | 12 cases (`toolbox test .toolbox`): a success, 7 invalid inputs, a 100-character boundary case, self-approval and outsider refusals, and a rejected push |

The `<project>` is `alpha`, `beta` or `gamma`, on that project's `main`.

## How each requirement is met

| # | How | Where |
|---|---|---|
| R1 | Each project's action, tests and portal live in its own repository on `main`. The org portal clones each project and syncs it before every listing, preview and run, so only what is on the project's `main` counts. To change the action, a project's maintainers edit and push the file; nothing in `org` changes | `alpha/.toolbox/actions/set-codeowners.yaml` (the same in `beta/` and `gamma/`) |
| R2 | The inputs are a JSON Schema with `additionalProperties: false`. `pattern` must match `^[/*][^\s#]*$` with `maxLength: 100`. `owners` must match `^@seg(/seg)*( @seg(/seg)*){0,2}$`, where `seg` is `[A-Za-z0-9_.-]+`. Toolbox checks them before any preview. An invalid run has state `invalid` and exit 2, and nothing is created | `inputs:` in `.toolbox/actions/set-codeowners.yaml`; cases in `.toolbox/tests/set-codeowners.test.yaml` |
| R3 | A preview (no `--yes`) prints the exact `CODEOWNERS` diff, the push it would do, the approvers, and a confirm token, and changes nothing. `--yes` records the same preview in the run (`effects`) | `toolbox run set-codeowners --entity portal:<p> -i …` |
| R4 | `approval: "['group:acme-w15/release']"`. The requester never counts, and anyone not in the group is refused with exit 2, which is recorded as `approval_refused`. Identity is not taken on trust: `forges[].identity: true` means `--as` alone is refused. Toolbox asks GitLab who owns the token in `--identity-token-file`, and `--as` must agree with it. Toolbox signs the approval with the org key, and each project accepts only that key (`federation.acceptApprovalsFrom`) | `org/toolbox.yaml` (`forges`, `authz`); the `approval:` and `federation:` blocks of the action |
| R5 | The executor is `type: change` with `land: push` to `main`. After the push, Toolbox fetches `origin/main` and checks that the commit is on it and that every changed file has the previewed content (`verification.child.files[].matches`). Only then is the run `succeeded`. A rejected push makes the run `failed` (exit 1) with GitLab's message, and it is never retried or forced. I tested this on a temporary project whose `main` allowed "No one" to push or merge; the project has been deleted | `executor:` in the action; the `rejectPush` test case |
| R6 | From `org/`, `toolbox run` lists `set-codeowners` for `portal:alpha`, `portal:beta` and `portal:gamma`. A run on any of them is requested with `--entity portal:<p>` | `org/toolbox.yaml` `federation.children` |
| R7 | Each run's record holds `requested_by`, `inputs`, `approval.approved_by` and `satisfied`, `effects` (the diff), `state`, `error` and `verification`. Its `events` list every step with `at` (UTC) and `by`: previewed, confirmed, awaiting_approval, approval_refused (with the reason), approved (with the note), dispatched, running, succeeded or failed. The commit on `main` carries `Toolbox-Run`, `Requested-By` and `Approved-By` trailers | `toolbox run --verify <run>`; files in `$TOOLBOX_STATE_DIR/runs/<run>.json` |
| R8 | Every command is non-interactive and takes `-o json`. Exit codes: 0 succeeded, 1 failed, 2 invalid, refused or blocked, 3 awaiting approval, 4 unverified | the commands below |
| R9 | Approval membership is `group:acme-w15/release`. On every command, the `gitlab` connector reads that group's **direct** members (inherited members such as ann, bob and ops-w15 do not count) and maps them to group tuples. I tested this in GitLab: after dave was removed he was refused, and after bob was added directly to `acme-w15/release` he could approve. The original membership is restored | `org/toolbox.yaml` (`connectors`, `kinds`, `authz.tuples`), `org/groups.fga` |

## Setup for every shell (nothing to start)

```sh
export HOME=/tmp/actions-gitlab/r1/toolbox-3/home
export PATH=/tmp/actions-gitlab/r1/toolbox-3/bin:$PATH
export TOOLBOX_STATE_DIR=/tmp/actions-gitlab/r1/toolbox-3/home/.local/state/toolbox
export GITLAB_TOKEN=$(cat /tmp/actions-gitlab/r1/toolbox-3/people/ops.token)  # reads the groups; any member's token works
P=/tmp/actions-gitlab/r1/toolbox-3/people
cd /tmp/actions-gitlab/r1/toolbox-3/org
```

Use the same `TOOLBOX_STATE_DIR` for every command. It holds the org's signing key, the clones of
the projects, and the run records. Requests and approvals must share it.

## Commands

```sh
# List every project's set-codeowners (and the org key)
toolbox run -o json

# Preview: the CODEOWNERS diff and approvers; changes nothing
toolbox run set-codeowners --entity portal:beta -i pattern=/docs/ -i owners='@acme/docs @carol' \
  --identity-token-file $P/ann.token -o json

# Request as ann: exit 3 (awaiting approval); the run id is .id
RUN=$(toolbox run set-codeowners --entity portal:beta -i pattern=/docs/ -i owners='@acme/docs @carol' \
  --identity-token-file $P/ann.token --yes -o json | jq -r .id); echo "$RUN"

# Approve as dave (or erin); it pushes, then verifies on main. Exit 0 = succeeded, 1 = failed
toolbox run --approve "$RUN" --identity-token-file $P/dave.token --note "release sign-off" --wait -o json

# Check a run's status
toolbox run --verify "$RUN" --identity-token-file $P/ann.token -o json | jq '{id, state, error}'

# Read its record: who requested and approved, inputs, outcome, and when each happened
toolbox run --verify "$RUN" --identity-token-file $P/ann.token -o json \
  | jq '{id, state, requested_by, inputs, approval, error, verification, events}'

# Every run
toolbox run --runs -o json
```

To reject a run, use `toolbox run --reject "$RUN" --identity-token-file $P/erin.token`.

### Expected refusals

| Try | Result |
|---|---|
| `-i pattern=docs/`, a `#` or space in the pattern, a pattern over 100 characters, 0 or 4 owners, an owner without `@`, or two spaces between owners | `invalid`, exit 2, nothing created |
| ann approves her own run (`--identity-token-file $P/ann.token`) | `a requester cannot approve their own run`, exit 2 |
| bob, carol or ops-w15 approves | `user:<x> is not one of the pending approvers`, exit 2 |
| dave requests and dave approves | refused as the requester, exit 2 |
| `--as dave` with ann's token | `--as says user:dave, but … the identity token is user:ann`, exit 2 |
| `--as dave` without a token | refused, because this portal takes identity from GitLab tokens; exit 2 |
| A project's `main` protected to "No one" for push and merge | after approval: `failed`, exit 1, with GitLab's `pre-receive hook declined` message; never `succeeded` |
| A direct member added to or removed from `acme-w15/release` in GitLab | takes effect on the next command, including on a run already waiting |

## Notes and limits

- **Who pushes:** after approval, the org portal pushes from its clone with the git credentials in
  `$HOME/.git-credentials` (ops-w15), not as the approver. Branch protection on `main` therefore
  applies to ops-w15, an Owner. With "No one" allowed, the push is refused and the run fails, as R5
  requires.
- **Trust boundary:** identity comes from GitLab tokens. However, the signing key and run records are
  files in `TOOLBOX_STATE_DIR` on this host, so anyone who can write there can forge approvals. To
  move this to a shared host, serve the portal behind sign-in (`toolbox serve` with
  `TOOLBOX_AUTH_MODE`; see `toolbox serve --help`).
- **Role threshold:** Toolbox's GitLab connector counts direct members with the Developer role or
  above. A direct Guest or Reporter in `acme-w15/release` is not an approver. GitLab also refuses a
  direct role lower than an inherited one, so bob or ann must be added as Maintainer.
- **Misleading hint:** the `awaiting_approval` note in a record says "approve with toolbox run --as
  <approver>". On this portal, use `--identity-token-file` as shown above instead.
- **Run ids:** `--runs` lists both the org run (the id that `--yes` printed) and the run the project
  portal recorded for it (`verification.child_run`). Approve and verify the org id.
- **Changing the action:** edit `.toolbox/actions/set-codeowners.yaml` in the project, then run
  `toolbox validate .toolbox && toolbox test .toolbox` there, and push to `main`. If the org key
  ever changes (a new `TOOLBOX_STATE_DIR`), put the key that `toolbox run` prints into each project's
  `acceptApprovalsFrom`.
