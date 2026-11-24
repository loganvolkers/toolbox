# add-notice: handoff

`add-notice` puts one `> **Info:** <message>` or `> **Warning:** <message>` notice, then a blank
line, at the top of a repository's root README on its origin's `main`, replacing an existing notice.
Each repository (`alpha`, `beta`, `gamma`) owns its definition in `.toolbox/`. This org portal
(`/tmp/actions-ab/r8/toolbox-3/org`) lists all three, holds the approvals, and defines the release
group once. It is built on Toolbox's federated actions: `toolbox run` in this directory lists,
previews, requests, approves, runs and verifies.

**Nothing has to be running.** Every step is a `toolbox run` command in this directory. There is no
server, no daemon and no port.

The three requested runs succeeded:

| Repository | Run (org portal) | Approved by | Commit on `main` |
|---|---|---|---|
| alpha | `run_2fwb8vs9bpo6` | dave | `76fd80c` |
| beta | `run_2fwb9020mfsn` | erin | `b3a5c30` |
| gamma | `run_2fwb944mrvn3` | erin | `2426881` |

## Setup (every shell)

```sh
cd /tmp/actions-ab/r8/toolbox-3/org
. ./env.sh      # puts bin/ on PATH and sets TOOLBOX_STATE_DIR=/tmp/actions-ab/r8/toolbox-3/state
```

`TOOLBOX_STATE_DIR` matters. It holds the org portal's approval-signing key (which the repositories
pin), its clones of the three origins, and every run record. Without it, Toolbox uses
`~/.local/state/toolbox`, makes a new key, and every repository then refuses the approvals
(`child-refuses-approvals`). There are no passwords: `--as <name>` (or `TOOLBOX_AS`) says who you
are, and it is recorded on the run.

## Commands

Every command takes `-o json` for machine-readable output. Exit codes: `0` succeeded, `1` failed
(for example, a rejected push), `2` invalid inputs or a refused approval, `3` waiting for approval,
`4` unverified.

```sh
# List the actions of all three repositories
toolbox run --as ann -o json

# Preview: the README diff and a confirm token. Changes nothing
toolbox run --as ann add-notice --entity portal:alpha -i level=warning \
  -i 'message=This proxy is frozen while the v2 rewrite lands; please file new issues against v2.' -o json

# Request as ann: records the preview, then waits for approval (exit 3). The run id is `.id`
toolbox run --as ann add-notice --entity portal:beta -i level=info \
  -i 'message=Releases now ship on the shared release train every second Tuesday.' --yes -o json
#   or, after a preview, confirm that exact preview (valid 10 minutes):
#   toolbox run --as ann add-notice --entity portal:beta -i level=info -i 'message=…' --confirm ct_… -o json

# Approve as dave or erin: pushes to main, then checks main (exit 0 once it has the README)
toolbox run --as dave --approve <run-id> --note "looks right" --wait -o json
# Reject instead
toolbox run --as erin --reject <run-id> --note "wrong wording" -o json

# Check a run's status: `.state` (exit code as above)
toolbox run --as ann --verify <run-id> -o json | python3 -c 'import json,sys; print(json.load(sys.stdin)["state"])'

# Read its record: requested_by, inputs, approval.approved_by, effects (the diff), events (each with `at`),
# outputs.commit and verification
toolbox run --as ann --verify <run-id> -o json

# Every run
toolbox run --as ann --runs -o json
```

`--runs` lists two records for each approved run: the org portal's run, which is the id to use and
holds the approvals, and the repository's run that executed it (`verification.child_run`, also in
the commit's `Toolbox-Run` trailer).

### Change who is in the release group

The group is the `authz` block of `/tmp/actions-ab/r8/toolbox-3/org/toolbox.yaml`, one line per
member. The next approval reads the file, so the change applies at once to all three repositories,
including runs that are already waiting. Commit and push it so the change is recorded:

```sh
cd /tmp/actions-ab/r8/toolbox-3/org
# add carol
sed -i '/- static:/a\        - {user: user:carol, relation: member, object: group:release}' toolbox.yaml
# remove erin
sed -i '/{user: user:erin, relation: member, object: group:release}/d' toolbox.yaml
toolbox validate . && grep 'group:release' toolbox.yaml
git commit -qam "release group: add carol, remove erin" && git push -q origin main
```

### Things to try

| Test | Command | Expected |
|---|---|---|
| Invalid level | `toolbox run --as ann add-notice --entity portal:alpha -i level=loud -i 'message=Ten or more chars' --yes -o json` | exit 2, `errors`, no run recorded, nothing changes |
| Message too short or too long | `-i 'message=too short'` (9 characters) or 161 characters | exit 2 |
| Two-line message | `-i "message=$(printf 'line one\nline two')"` | exit 2 |
| Unknown input | `--inputs '{"level":"info","message":"Ten or more chars","x":1}'` | exit 2 |
| Self-approval | dave requests (`--as dave … --yes`), then `toolbox run --as dave --approve <id>` | exit 2, `a requester cannot approve their own run`, recorded as `approval_refused` |
| Approval from outside the group | `toolbox run --as bob --approve <id>` (also ann or carol) | exit 2, `not one of the pending approvers`, recorded |
| Group change | remove erin as above, then `toolbox run --as erin --approve <id>` | exit 2. Add carol, and her approval is accepted |
| Rejected push | see below | exit 1, `state: failed` with the origin's message, `main` unchanged |

Rejected push: make the origin refuse pushes, request a change that is not on `main` yet, approve
it, then remove the hook.

```sh
H=/tmp/actions-ab/r8/toolbox-3/origins/gamma.git/hooks/pre-receive
printf '#!/bin/sh\necho "main is protected" >&2\nexit 1\n' > $H && chmod +x $H
ID=$(toolbox run --as ann add-notice --entity portal:gamma -i level=warning \
  -i 'message=This notice must not land on main.' --yes -o json | python3 -c 'import json,sys; print(json.load(sys.stdin)["id"])')
toolbox run --as dave --approve $ID --wait -o json; echo "exit $?"   # exit 1, state failed, error has the hook's message
toolbox run --as ann --verify $ID -o json                            # still failed
rm $H
```

## How each requirement is met

| # | How | Files |
|---|---|---|
| R1 Owned by the repository | Each repository commits its own definition to its origin's `main`: a manifest that includes the action, the action, the script that edits the README, and their tests. CODEOWNERS gives `.toolbox/` to ann and bob. The org portal reads each origin's `main` every time, so a change the team pushes takes effect on the next list or preview. The action files are the same in all three repositories, but each repository's copy is its own to change | `<repo>/.toolbox/toolbox.yaml`, `<repo>/.toolbox/actions/add-notice.yaml`, `<repo>/.toolbox/actions/add-notice.sh`, `<repo>/.toolbox/tests/`, `alpha/CODEOWNERS`, `beta/.github/CODEOWNERS`, `gamma/.github/CODEOWNERS` |
| R2 Validated | The inputs are a JSON Schema in the action: `level` is `info` or `warning`; `message` is required, 10 to 160 characters, and has no line break (`pattern: '^[^\r\n]+$'`); no other inputs (`additionalProperties: false`). Toolbox checks them before any preview, and an invalid request exits 2 and records nothing. The script checks them again before it writes | `<repo>/.toolbox/actions/add-notice.yaml` (`inputs`), `add-notice.sh` |
| R3 Previewed | A run without `--yes` or `--confirm` is a preview: the exact README diff (`effects`), who must approve, and a confirm token. A request records that preview, and the run fails before it commits if the change it would make differs from that preview | `toolbox run … add-notice …` (no `--yes`) |
| R4 Approved by the release group | `approval: "['group:release']"`. The org portal decides approvals: it refuses the requester and anyone who is not a member when they approve, and records each refusal. Only an approval it signed with its key counts in a repository (`acceptApprovalsFrom` pins that key), so nothing lands without it | `<repo>/.toolbox/actions/add-notice.yaml` (`approval`, `federation.acceptApprovalsFrom`), `org/toolbox.yaml` (`authz`) |
| R5 Done means observed | The executor is a `change` with `land: push`: after the approval it commits the README in a fresh clone of `main` and pushes (never forced). Toolbox then fetches the origin's `main` and reports `succeeded` only if the commit is on it and the README blob matches the preview (`verification.child.files[].matches`). A rejected push is reported as `failed` with the origin's message (exit 1). So is a push the origin accepted that did not stay on `main` (rehearsed with a hook that resets `main`: `the commit is not on origin/main`) | `<repo>/.toolbox/actions/add-notice.yaml` (`executor`) |
| R6 One place for the org | `org/toolbox.yaml` lists the three origins under `federation.children` (offered as `portal:alpha`, `portal:beta`, `portal:gamma`). Every command above runs in `org/` and never enters a repository | `org/toolbox.yaml` |
| R7 Audited | Each run record has `requested_by`, `inputs`, `approval.approved_by` (and `satisfied`), `state`, `effects`, `outputs.commit`, `verification`, and `events`: one per step (previewed, confirmed, awaiting_approval, each refused approval, approved with the note, dispatched, running, succeeded or failed), each with `at` and `by`. Each commit also carries `Toolbox-Run`, `Requested-By` and `Approved-By` trailers | `/tmp/actions-ab/r8/toolbox-3/state/runs/<run-id>.json`, read with `toolbox run --verify <run-id> -o json`; `git log` on each origin |
| R8 Usable by agents | Every step is a non-interactive `toolbox run` command with `-o json` and distinct exit codes (0, 1, 2, 3, 4). `--yes` skips the confirm prompt, and `--wait` blocks until the run settles | the commands above |
| R9 The group is defined once | `group:release` (dave, erin) is defined only in the org portal's `authz` tuples, over the model in `groups.fga`. The repositories name the group, never its members. Changing `org/toolbox.yaml` changes who may approve in all three repositories at the next approval (rehearsed: after removing erin her approval is refused, and after adding carol hers is accepted) | `org/toolbox.yaml` (`authz`), `org/groups.fga` |

## Testing a change to the action

In a repository, before pushing a change to `.toolbox/`:

```sh
cd /tmp/actions-ab/r8/toolbox-3/alpha && . ../org/env.sh
toolbox validate .toolbox            # the manifest and the action
sh .toolbox/tests/add-notice.sh      # the README edit on 18 fixture READMEs
toolbox test .toolbox                # 14 lifecycle cases in a scratch copy of the origin
```

`toolbox test` runs preview, request, approval, push and verification in a scratch clone, so
nothing reaches the real origin. Its cases cover both release members approving, the 10- and
160-character limits, invalid inputs, self-approval, approvals by ann, bob and carol, and a rejected
push. Its `members:` line stands in for the org portal's group.

## Limits

- `--as` is taken on trust, as this sandbox intends. Anyone who can write to `TOOLBOX_STATE_DIR` can
  also read the signing key. For real use, serve the org portal behind sign-in
  (`toolbox serve . --listen 127.0.0.1:7190`, with `TOOLBOX_AUTH_MODE` set) and pass `--url`.
- A repository with no root README, or with two (for example,   `README.md` and `readme.md`), is
  blocked at preview (exit 2, `the executor's preview failed`), and nothing changes.
