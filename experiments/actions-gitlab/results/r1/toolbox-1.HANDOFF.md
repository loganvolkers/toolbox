# HANDOFF: `set-codeowners`

`set-codeowners` is a Toolbox action that each of `acme-w11/alpha`, `beta` and `gamma` declares on
its own `main`. This project (`acme-w11/org`) is the org portal. From here you list it for all
three projects, preview, request, approve, check runs and read their records, all from a shell
and with `-o json`. Nothing has to be running: every command runs from this checkout, and there is
no server.

Identity comes from GitLab. The portal's GitLab forge is marked `identity: true`, so each command
that requests, approves or rejects needs the person's own GitLab token (`--identity-token-file`).
Toolbox asks GitLab who owns the token, and `--as` alone is refused. Approval needs a direct
member of the GitLab group `acme-w11/release` other than the requester. Toolbox reads that group's
direct members from GitLab on every request and approval.

The three requested runs succeeded:

| Project | Org run | pattern | owners | Approved by | Commit on `main` |
|---|---|---|---|---|---|
| alpha | `run_2g08q9fqe2nz` | `*` | `@acme/proxy-maintainers` | dave | `fea679c` |
| beta | `run_2g08qr1wgaid` | `/docs/` | `@acme/docs @carol` | erin | `f8625f9` |
| gamma | `run_2g08r90n0c1v` | `/graphify/security.py` | `@acme/security` | erin | `271973c` |

`run_2g09j0lregu3` is a failed test run on gamma, recorded on purpose. While it ran, gamma's
`main` was protected with "No one" allowed to push or merge, and erin was swapped out of
`acme-w11/release` for carol. Erin's approval was refused, carol's was accepted, the push was
rejected and the run `failed`. GitLab was then restored: release = dave and erin (Developer), and
gamma's `main` allows Maintainers to push and merge. `run_2g0alldo3cvr` (alpha) was requested
through preview and `--confirm`, then rejected by erin, to test those commands.

## Files

| Path | What it is |
|---|---|
| `alpha/.toolbox/toolbox.yaml` (and in `beta`, `gamma`) | The project's portal: its name, its owner, and `include: [actions/*.yaml]` |
| `<project>/.toolbox/actions/set-codeowners.yaml` | The action: inputs (JSON Schema), approval rule, executor, and the org portal's key the project accepts approvals from |
| `<project>/.toolbox/scripts/set-codeowners.py` | The edit: replace the first line whose first field is the pattern, or append the rule; the file ends with a newline |
| `<project>/.toolbox/tests/set-codeowners.test.yaml` | 17 lifecycle cases for `toolbox test .toolbox` (approvals, invalid inputs, a rejected push) |
| `<project>/.toolbox/scripts/test_set_codeowners.py` | 13 unit cases for the edit itself |
| `org/toolbox.yaml` | The org portal: the three projects (`federation.children`), the GitLab forge with `identity: true`, and the gitlab connector that reads `acme-w11`'s groups into `group:` tuples |
| `org/groups.fga` | The authorization model those tuples use (`group#member`) |
| `org/env.sh` | The environment every command needs (below) |
| `$HOME/.local/state/toolbox/runs/<run id>.json` | Run records (`HOME=/tmp/actions-gitlab/r1/toolbox-1/home`) |
| `$HOME/.local/state/toolbox/served/org/signing-key` | The key the org portal signs approvals with. The projects pin its public half, `ed25519:b4777ecc…2679` |
| `$HOME/.local/state/toolbox/federation/org-3830c719/<project>` | The org portal's clones of the projects, synced from `main` before every list, preview and run |

## How each requirement is met

| # | How |
|---|---|
| R1 | The definition is in each project, committed to its `main`: `.toolbox/actions/set-codeowners.yaml`, `.toolbox/scripts/set-codeowners.py` and `.toolbox/tests/set-codeowners.test.yaml`, included from `.toolbox/toolbox.yaml`. The org portal holds no copy. It clones each project's `main` and offers whatever is committed there, re-synced before every list, preview and run. A project's maintainers change the action by changing those files (then `toolbox validate .toolbox && toolbox test .toolbox`), and the change shows up in the org portal on its next command. `federation.runner: {type: local}` in that file lets the org portal run it, and `acceptApprovalsFrom` says whose approvals the project accepts |
| R2 | The `inputs` JSON Schema in `set-codeowners.yaml` is checked before any preview, request or run. `pattern`: `^[/*][^\s#]{0,99}$`, `maxLength: 100`. `owners`: `^@[A-Za-z0-9_.-]+(/[A-Za-z0-9_.-]+)*( @…){0,2}$`. Both are `required`, with `additionalProperties: false`. An invalid input exits 2 with the reason, and no run is created. `set-codeowners.py` checks the inputs again and exits 2 without writing. Cases: `tests/set-codeowners.test.yaml` and `scripts/test_set_codeowners.py` |
| R3 | `toolbox run set-codeowners --entity portal:<p> -i …` with no `--yes` or `--confirm` is a preview. It runs the edit in a scratch worktree of the project's `main` and prints the unified diff of `CODEOWNERS`, the push it would make, the approvers, and a confirm token. Nothing changes. A request (`--yes`, or `--confirm <token>`) records that same diff as the run's `effects`. If `main` changes so that the result would differ from the preview, the run fails before it commits (`preview_stale`) |
| R4 | `approval: "['group:acme-w11/release']"` in `set-codeowners.yaml`. The org portal (`org/toolbox.yaml`) decides approvals. Who is asking comes from GitLab (`forges[].identity: true`, `--identity-token-file`), never from a free-text `--as`. The requester is always refused (`a requester cannot approve their own run`, exit 2), and anyone not in the group is refused (`is not one of the pending approvers`, exit 2): Maintainers and the Owner too, since membership of the parent group does not count. The approval is signed with the org portal's key, and the project refuses a grant without that signature (`acceptApprovalsFrom`) |
| R5 | `executor: {type: change, land: push, base: main}`. After approval, Toolbox commits in a fresh worktree of `main` and pushes (never forced). Then it fetches `origin/main` and checks that the commit is on it and that `CODEOWNERS` has the blob the preview showed (`verification.child.files[].matches`). Only then is the run `succeeded`. A rejected push, for example to a `main` protected so that "No one" may push, makes the run `failed` with GitLab's message (exit 1), as `run_2g09j0lregu3` shows. A rule that is already in place commits nothing and succeeds (`outputs.changed: false`) |
| R6 | `org/toolbox.yaml` lists the three projects under `federation.children`. From `/tmp/actions-gitlab/r1/toolbox-1/org`, `toolbox run` lists `set-codeowners` on `portal:alpha`, `portal:beta` and `portal:gamma`. `--entity portal:<p>` previews, requests, approves and checks any of them, and `toolbox run --runs` lists every run |
| R7 | Each run record (`toolbox run --verify <id> -o json`, kept in `$HOME/.local/state/toolbox/runs/<id>.json`) has `requested_by`, `inputs`, `approval.approved_by` and `approval.satisfied`, `state`, `outputs` (commit), `verification`, and `events`. Each event has an `at` timestamp: `previewed` and `confirmed` (by the requester), `awaiting_approval`, `approval_refused`, `approved` (by whom, with the `--note`), `dispatched`, `running`, and then `succeeded` or `failed` (with the error). The commit on `main` also carries `Toolbox-Run`, `Requested-By` and `Approved-By` trailers |
| R8 | Every step is a `toolbox run` command with `-o json` and no prompt. Exit codes: 0 succeeded, 1 failed, 2 invalid, blocked or refused, 3 awaiting approval, 4 unverified |
| R9 | The gitlab connector in `org/toolbox.yaml` (`builtin:gitlab`, `group: acme-w11`, `minAccess: 5`) reads each group's direct members (`/groups/:id/members`, not `/members/all`). The `Group` kind and `authz.tuples` turn them into `user:<username> member group:acme-w11/release`. The connector runs again on every request and approval, so a membership change in GitLab applies to the next approval on all three projects, with no change to any file. Tested: removing erin made her approval refused, and adding carol made hers count |

## Commands

Run everything from the org checkout, with the environment from `env.sh`. That file sets `HOME`
(git's push credentials and Toolbox's state), `PATH`, and `GITLAB_TOKEN`, which the connector
uses only to read the groups.

```sh
cd /tmp/actions-gitlab/r1/toolbox-1/org && git pull -q
. ./env.sh
P=/tmp/actions-gitlab/r1/toolbox-1/people
```

**Start anything:** nothing needs to be running. Listing and reading records need no token.
Preview, request, approve and reject need `--identity-token-file`. Don't use `toolbox serve` for approvals: on a
loopback address it signs people in with `--as` on trust, which bypasses the GitLab identity
check.

**List** the actions of all three projects:

```sh
toolbox run -o json
```

**Preview**, as the requester. It shows the `CODEOWNERS` diff in `.effects` and changes nothing.
It prints `.confirm_token`, and `.effects` is empty when the rule is already in place:

```sh
toolbox run set-codeowners --entity portal:beta -i 'pattern=/docs/' -i 'owners=@acme/docs @carol' \
  --identity-token-file $P/ann.token -o json
```

**Request, as ann.** Either confirm the preview you read (within 10 minutes):

```sh
toolbox run set-codeowners --entity portal:beta -i 'pattern=/docs/' -i 'owners=@acme/docs @carol' \
  --confirm <ct_… token from the preview> --identity-token-file $P/ann.token -o json
```

or preview and request in one step:

```sh
toolbox run set-codeowners --entity portal:beta -i 'pattern=/docs/' -i 'owners=@acme/docs @carol' \
  --yes --identity-token-file $P/ann.token -o json        # exit 3, awaiting approval; .id is the run id
RUN=$(toolbox run set-codeowners --entity portal:gamma -i 'pattern=/x/' -i 'owners=@acme/security' \
  --yes --identity-token-file $P/ann.token -o json | jq -r .id)
```

**Approve, as dave or erin.** The approval runs the change and waits for the outcome: exit 0 when
`main` has it, exit 1 when it failed.

```sh
toolbox run --approve "$RUN" --identity-token-file $P/dave.token --note "checked" --wait -o json
toolbox run --approve "$RUN" --identity-token-file $P/erin.token --note "checked" --wait -o json
toolbox run --reject  "$RUN" --identity-token-file $P/erin.token --note "no" -o json   # to reject instead: state rejected, exit 1
```

**Check a run's status:**

```sh
toolbox run --verify "$RUN" -o json | jq -r .state      # exit code: 0, 1, 3 or 4 as above
toolbox run --runs -o json                               # every run, newest first
```

**Read its record** (requester, approver, inputs, outcome and when each happened):

```sh
toolbox run --verify "$RUN" -o json | jq '{id, state, requested_by, inputs, approved_by: .approval.approved_by, events, outputs, verification}'
```

`--runs` lists two ids per run. One is the org portal's run, which `--yes` prints and which holds
the approvals. The other is the project-side run that executed it, given in the org run's
`verification.child_run`. Use the org run's id.

### What the refusals look like

| Try | Result |
|---|---|
| `-i pattern=docs/`, `-i 'pattern=/a b'`, `-i 'pattern=/a#b'`, a pattern of 101 characters, `-i owners=carol`, four owners, two spaces between owners | exit 2, `errors: ["pattern: … does not match …"]`, no run created |
| ann approves her own run (`--identity-token-file $P/ann.token`) | exit 2, `a requester cannot approve their own run` |
| bob, carol or ops approves | exit 2, `user:<name> is not one of the pending approvers ["group:acme-w11/release"]` |
| `--as user:dave` with no token, or with someone else's token | exit 2, refused: identity comes from the token |
| Remove erin from `acme-w11/release` in GitLab, then erin approves | refused as above. Add someone as a direct member and their approval counts |
| gamma's `main` protected with "No one" for push and merge, then an approved run | exit 1, `state: failed`, `error: the push to origin main was refused … You are not allowed to push code to protected branches` |

## Changing the action

In a project's checkout, edit `.toolbox/actions/set-codeowners.yaml` or the script. Then:

```sh
export HOME=/tmp/actions-gitlab/r1/toolbox-1/home PATH=/tmp/actions-gitlab/r1/toolbox-1/bin:$PATH
toolbox validate .toolbox && toolbox test .toolbox && python3 .toolbox/scripts/test_set_codeowners.py
git commit -am "…" && git push origin HEAD:main
```

The org portal offers the new version on its next command. If the org portal's signing key is ever
replaced (a new `HOME` or state directory), update `acceptApprovalsFrom` in each project: until
then, runs are refused.
