# HANDOFF: the `add-notice` self-service action

`add-notice` is a Toolbox federated action. Each repository declares it in its own `.toolbox/`.
The org portal in this directory lists all three, previews them, holds the approvals and runs them.
Toolbox does the input validation, preview, approval rule, commit, push and verification on `main`
itself. The only glue I wrote is the README transformation (`add-notice.py`, about 60 lines of
Python). No server is needed: everything runs from this checkout with `toolbox run`.

State on handoff: the three requested runs **succeeded**, and the notices are on each origin's
`main`.

| Repository | Run (org) | Requested by | Approved by | Commit on `main` |
|---|---|---|---|---|
| alpha | `run_2fw7fe89d3uk` | user:ann | user:dave | `8f80ca8` (README.md) |
| beta | `run_2fw7fjehyhtx` | user:ann | user:erin | `a095eae` (readme.md) |
| gamma | `run_2fw7foi8033n` | user:ann | user:erin | `a6947e9` (README.md) |

Each org run has a child run, which ran the change in the org's clone of the repository. Its id
is `verification.child_run` in the record, and it is the `Toolbox-Run` trailer on the commit: for
example, `run_2fw7g14ur0d4` for alpha. Both kinds appear in `--runs`.

The run list also holds two probe runs I made while testing. `run_2fw75zvjoi2i` was on alpha: it
recorded refused approvals by dave (the requester), ann, carol, and erin while she was out of the
group, and then erin rejected it. `run_2fw79bkymssq` was on beta: it failed on a push the origin
rejected (a temporary `pre-receive` hook, since removed).

## Files

| Path | What it is |
|---|---|
| `org/toolbox.yaml` | The org portal: `federation.children` lists the three origins (offered as `portal:alpha`, `portal:beta`, `portal:gamma`), and `authz.tuples` defines `group:release` (dave, erin) and the maintainer groups |
| `org/groups.fga` | The OpenFGA model for groups (`member: [user, group#member]`) |
| `org/env.sh` | Sets `PATH` and `TOOLBOX_STATE_DIR`. **Source it before every command** |
| `<repo>/.toolbox/toolbox.yaml` | The repository's portal: its name, its owner (`group:<repo>-maintainers`), `include: [actions/*.yaml]` |
| `<repo>/.toolbox/actions/add-notice.yaml` | The action: inputs schema, `approval: "['group:release']"`, a `change` executor with `land: push` on `main`, and the org key it accepts approvals from. It is the same file in all three repositories |
| `<repo>/.toolbox/actions/add-notice.py` | The effect: finds the root README in any letter case, replaces an existing `> **Info|Warning:** …` line plus blank line at the top, and prepends the new notice. `--self-test` checks it |
| `<repo>/.toolbox/tests/add-notice.test.yaml` | 11 lifecycle cases run by `toolbox test .toolbox` in a scratch copy: success with dave and with erin, five invalid inputs, an unknown input, refused approvers, and a rejected push |
| `/tmp/actions-ab/r8/toolbox-2/state/` | `TOOLBOX_STATE_DIR`: the org's signing key, its clones of the repositories, and every run record (`runs/`). It is not in git |

## How each requirement is met

| # | How |
|---|---|
| R1 Owned by the repository | The action is `<repo>/.toolbox/actions/add-notice.yaml` plus `add-notice.py`, committed to each origin's `main` (alpha `8cb31a9`, beta `8c6bd14`, gamma `d4c86c6`). Before it lists, previews or runs anything, the org portal fetches each origin's `main` and reads its `.toolbox/` again. A maintainer changes the action by editing those files, running `toolbox validate .toolbox && toolbox test .toolbox` and pushing to `main` |
| R2 Validated | `inputs` in `add-notice.yaml` is a JSON Schema with `additionalProperties: false`, `required: [level, message]`, `level` as `enum: [info, warning]`, and `message` limited to `minLength: 10`, `maxLength: 160` and a pattern that forbids line breaks and control characters. Toolbox checks the schema before the preview and exits 2 (`invalid`) without recording a run or changing anything. `add-notice.py` checks the inputs again before it writes |
| R3 Previewed | `toolbox run add-notice --entity portal:<repo> -i …` without `--yes` prints the README diff and the push to `main`, and changes nothing. The same diff is stored in the run record as `effects`. When the run lands, Toolbox checks that the file on `main` matches the preview |
| R4 Approved by the release group | `approval: "['group:release']"` in `add-notice.yaml`. Toolbox refuses the requester (`a requester cannot approve their own run`) and anyone who is not a member (`… is not one of the pending approvers`) with exit 2, and records each refusal. The org signs the approval with its key, and each repository accepts only that key (`federation.acceptApprovalsFrom` in `add-notice.yaml`) |
| R5 Done means observed | The `change` executor uses `land: push`. After pushing, Toolbox fetches `origin/main` and reports `succeeded` only if the commit is on `main` and every changed file has the previewed blob (`verification.child.files[].matches`). A rejected push leaves the run `failed` (exit 1) with the origin's message. Nothing is retried or force-pushed |
| R6 One place for the org | `org/toolbox.yaml` `federation.children`. From `org/`, `toolbox run` lists all three repositories' actions, and `--entity portal:<repo>` previews and requests on any of them |
| R7 Audited | Each run record (`toolbox run --verify <id> -o json`, stored under `state/`) has `requested_by`, `inputs`, `approval.approved_by` and `approval.satisfied`, `state`, `error`, `outputs.commit`, `verification`, and `events`, each with `at` and `by` (previewed, confirmed, awaiting_approval, approval_refused, approved, succeeded or failed or rejected). The commit on `main` also carries `Toolbox-Run`, `Requested-By` and `Approved-By` trailers |
| R8 Usable by agents | Every step is a non-interactive `toolbox run` with `-o json`. Exit codes: 0 succeeded, 1 failed or rejected, 2 invalid, blocked or refused, 3 awaiting approval, 4 unverified. `TOOLBOX_AS` can replace `--as` |
| R9 The group is defined once | `group:release` is defined only in `org/toolbox.yaml` under `authz.tuples`, using the model `org/groups.fga`. The repositories name only `group:release`. Every approval for every repository is decided against the org's tuples, read from `org/toolbox.yaml` at the time of the approval. I tested this: with erin removed, her approval was refused |

## Commands

Run everything from the org directory, with the environment sourced. Everyone must use this state
directory: it holds the signing key the repositories pin, and the runs. A different state
directory makes a new key, which the repositories would refuse (`child-refuses-approvals`).
`--as` is taken on trust, as this sandbox intends, and is recorded on every event.

```sh
cd /tmp/actions-ab/r8/toolbox-2/org
. ./env.sh          # PATH=/tmp/actions-ab/r8/toolbox-2/bin:$PATH, TOOLBOX_STATE_DIR=/tmp/actions-ab/r8/toolbox-2/state
```

**Start:** nothing needs to be running. There is no server and no daemon.

**List** the actions of all three repositories:

```sh
toolbox run --as ann -o json
```

**Preview** (shows the README diff and changes nothing):

```sh
toolbox run --as ann add-notice --entity portal:alpha -i level=info -i message="Releases now ship every second Tuesday." -o json
```

**Request** as ann. This exits 3 (awaiting approval), and the run id is in `.id`:

```sh
RUN=$(toolbox run --as ann add-notice --entity portal:beta -i level=warning -i message="This package is frozen while v2 lands." --yes -o json | python3 -c 'import json,sys; print(json.load(sys.stdin)["id"])'); echo "$RUN"
```

**Approve** as dave or erin. This exits 0 once `main` is seen to have the change, or 1 if it failed:

```sh
toolbox run --as dave --approve "$RUN" --note "release sign-off" --wait -o json
toolbox run --as erin --reject  "$RUN" --note "not now"      # to reject instead
```

**Check status** and **read the record** (requester, approver, inputs, outcome, timestamped events,
commit and verification):

```sh
toolbox run --as ann --verify "$RUN" -o json
toolbox run --as ann --verify "$RUN" -o json | python3 -c 'import json,sys; d=json.load(sys.stdin); print(d["state"], d["requested_by"], d["approval"]["approved_by"], d["inputs"]); [print(e) for e in d["events"]]'
toolbox run --as ann --runs -o json                            # every run
```

**Change who is in the release group:** edit the `group:release` tuples in `org/toolbox.yaml`. The
next approval for any repository uses the new list. Commit the change so it is recorded:

```sh
# for example, add carol
sed -i 's|^\(\s*\)- {user: user:erin, relation: member, object: group:release}|&\n\1- {user: user:carol, relation: member, object: group:release}|' toolbox.yaml
toolbox validate . && git commit -qam "release group: add carol" && git push -q origin HEAD:main
```

### Things to try, and what you should see

| Try | Expected |
|---|---|
| `-i level=urgent`, a message under 10 or over 160 characters, a message with a newline (`-i message="$(printf 'line one ok\nline two')"`), a missing input, or an extra `-i colour=red` | Exit 2, `Invalid inputs …`. No run is recorded and nothing changes |
| The requester approves their own run (for example, dave requests and dave approves) | Exit 2, `a requester cannot approve their own run`. Recorded as `approval_refused` |
| ann, bob or carol approves | Exit 2, `user:<x> is not one of the pending approvers ["group:release"]`. Recorded |
| Remove erin from `group:release`, then she approves | Exit 2, refused. Add her back and her approval counts |
| Rejected push: `printf '#!/bin/sh\necho "main is protected" >&2\nexit 1\n' > ../origins/gamma.git/hooks/pre-receive && chmod +x ../origins/gamma.git/hooks/pre-receive`, then request and approve a notice that is **not** already on `main` | The approval exits 1, and the run is `failed` with the origin's message. `main` is unchanged. Remove the hook afterwards with `rm ../origins/gamma.git/hooks/pre-receive` |
| Request the exact notice that is already on `main` | `succeeded` with no new commit (`outputs.changed: false`), because the README already has that content |
| A second notice on a repository | The existing notice line and its blank line are replaced, so there is still one notice |

**Change the action itself**, as a maintainer of a repository:

```sh
cd /tmp/actions-ab/r8/toolbox-2/alpha && . ../org/env.sh
python3 .toolbox/actions/add-notice.py --self-test && toolbox validate .toolbox && toolbox test .toolbox
git commit -qam "add-notice: …" && git push -q origin HEAD:main   # the org portal picks it up on its next command
```
