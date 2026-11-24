# Handoff: `add-notice`, a self-service action owned by each repository

`add-notice` is defined in each repository: `alpha`, `beta` and `gamma`. It runs from the org
checkout through one CLI, `org/bin/acme`. The CLI is Python 3.12 and uses only the standard library.

- **No server.** Nothing has to be running, and nothing was left running.
- **JSON output.** Every command prints JSON on stdout.
- **Exit codes.**
  - `0`: ok.
  - `1`: the run failed, or an internal error happened.
  - `2`: refused. Refusals are invalid inputs, an approver who is not allowed, an unknown run, repo
    or person, or a run that is not pending.
- **Status.** The three requested runs are `run-0001` (alpha), `run-0002` (beta) and `run-0003`
  (gamma). All three are `succeeded`.

## Where things live

| Path | What it is |
|---|---|
| `<repo>/.acme/actions/add-notice/action.json` (on each origin's `main`) | The action's definition: owners (ann, bob), input schema, the approval group (`release`) and the command to run |
| `<repo>/.acme/actions/add-notice/run.py` (on each origin's `main`) | The repository's implementation. It edits the root README (`README.md` in any case) |
| `org/org.json` (on `origins/org.git` `main`) | The people (ann, bob, carol, dave, erin) and the three repositories with their origins |
| `org/groups.json` (on `origins/org.git` `main`) | **The only definition of the release group** |
| `org/bin/acme` | The runner and CLI |
| `org/runs/run-NNNN.json` | One record per run, as JSON |
| `org/runs/audit.log` | An append-only JSON-lines log of every event of every run |

`org/runs/` is runner state. It is gitignored and is not configuration.

## How each requirement is met

| # | How |
|---|---|
| R1 | Each repository's definition lives in `<repo>/.acme/actions/add-notice/action.json`, and its implementation in `run.py` beside it. Both are committed to each origin's `main`. The runner reads them only from the origin's `main`, at the commit it runs. If a maintainer changes them on `main`, the next run uses the change. `owners` names ann and bob. |
| R2 | `acme` checks the inputs against the `inputs` schema in `action.json` before it clones or runs anything: `level` must be `info` or `warning`; `message` must be 10 to 160 characters, one line, with no control or line-separator characters, and not blank. Unknown or missing inputs are refused too. `run.py` checks the same rules again, and refuses unless the repository has exactly one root README (any case) that is a regular file. A refusal exits with code 2 and `"error": "invalid_inputs"` or `"error": "action_refused"`. No run is created and nothing is pushed. |
| R3 | `acme preview` runs the repository's action on a throwaway clone of `main` and prints the unified diff and the changed files with their expected blob ids. It changes nothing. `acme request` stores the same preview in the run record (`preview.diff`), and the approver reads it with `acme show`. On approval the runner re-runs the action on the current `main`. If the result differs from what was previewed (because `main` changed), the run fails with `reason: stale` and nothing is pushed. |
| R4 | `acme approve` checks four things, in `org/bin/acme` `decide()`: the approver is a known person; the run is `pending_approval`; the approver is not the requester (`self_approval`); and the approver is a member of the group named in the definition's `approval.group`. That membership is read from `groups.json` on the org origin's `main` at approval time (`not_in_group`). Only after all four does it push. Refused attempts are recorded in the run's events as `approve_refused`. A run is approved and executed once; a second approval returns `not_pending`. |
| R5 | After the push, the runner reads the origin's `main` again. It compares the blob of every changed path with the blob it previewed and was approved for (`observed` event). Only an exact match sets `succeeded`. A rejected push fails with `reason: push_rejected`, including the remote's message. If the push is accepted but `main` does not then hold the content (for example, a hook rewinds it), the run fails with `reason: not_observed`. Any other error fails with `reason: error`. A run is never left `running`. |
| R6 | From `org/`, `./bin/acme list` lists the actions every repository in `org.json` defines on its `main`, with each one's inputs, owners and current approvers. `preview`, `request`, `approve`, `status` and `show` all take the repository name, so nobody has to enter a repository. |
| R7 | Each `org/runs/run-NNNN.json` records `requestedBy`/`requestedAt`, `approvedBy`/`approvedAt`, `inputs`, `outcome` (`result`, `at`, commit, observed `main`) and `preview`. It also holds a timestamped `events` list: `requested`, `approve_refused`, `approved`, `started`, `push_attempted`, `observed`, and `succeeded` or `failed`. The same events are appended to `org/runs/audit.log`. The commit on `main` has the requester as author, `Acme Actions` as committer, and `Run:`, `Requested-by:` and `Approved-by:` trailers. |
| R8 | Every step is a single non-interactive command that prints JSON and sets a meaningful exit code. Identity comes from `--as <name>` or `$ACME_USER`. Inputs come from repeated `-i key=value` or from `--inputs-json '{...}'`. Even usage errors print JSON. |
| R9 | The group is defined only in `groups.json` on `main` of `origins/org.git`. The definitions only name the group (`"approval": {"group": "release"}`). Every approval reads the current membership from the org origin's `main`. `acme group-add` and `acme group-remove` commit and push a change there; only members of the group's `managedBy` group, here `release` itself, may run them. The change is then in the org's git history with its author. Editing `org/groups.json` by hand and pushing it works too. An edit that is not pushed has no effect, and `acme groups` warns about it. |

## Commands

Everything below runs from the org checkout:

```sh
cd /tmp/actions-ab/r8/vanilla-3/org
```

**Start:** nothing needs to be started. The only requirements are `python3` (3.12) and `git`.

**List the actions** for all three repositories:

```sh
./bin/acme list
./bin/acme list --repo beta --action add-notice
```

**Preview** a run. This changes nothing, and `--as` is optional:

```sh
./bin/acme preview alpha add-notice -i level=info -i "message=Docs moved to the new site this week."
# just the diff:
./bin/acme preview alpha add-notice -i level=info -i "message=Docs moved to the new site this week." \
  | python3 -c 'import json,sys; print(json.load(sys.stdin)["diff"])'
```

**Request a run as ann.** This prints the new record, including its `id` (for example
`run-0004`) and `preview.diff`:

```sh
./bin/acme request --as ann alpha add-notice -i level=warning -i "message=Docs moved to the new site this week."
# or: ACME_USER=ann ./bin/acme request alpha add-notice --inputs-json '{"level":"info","message":"Docs moved to the new site this week."}'
```

**Approve as dave or erin.** This pushes, observes `main`, and prints the record. It exits 0 if the
run succeeded and 1 if it failed:

```sh
./bin/acme approve --as dave run-0004
./bin/acme reject --as erin run-0004 --reason "wording"   # release group, not the requester
./bin/acme cancel --as ann run-0004                        # the requester only
```

**Check a run's status:**

```sh
./bin/acme status run-0004
./bin/acme runs                           # all runs
./bin/acme runs --state pending_approval
```

A run's states are `pending_approval`, `running`, `succeeded`, `failed`, `rejected` and `cancelled`.

**Read a run's record**, including its preview, approvals, outcome and events:

```sh
./bin/acme show run-0004
cat runs/audit.log
```

**Change who is in the release group.** The actor must be a current member of `release`:

```sh
./bin/acme groups                                  # current members, from the org origin's main
./bin/acme group-add --as dave release carol
./bin/acme group-remove --as dave release erin
# or by hand: edit groups.json, then
git commit -am "release: ..." && git push origin main
```

A person must exist in `org.json` before they can be added to a group. To add a new person, edit
`org.json` and push it.

## Test recipes

```sh
cd /tmp/actions-ab/r8/vanilla-3/org
```

**Invalid inputs.** Each of these exits 2 and creates no run:

```sh
./bin/acme request --as ann alpha add-notice -i level=error -i "message=Valid length message."
./bin/acme request --as ann alpha add-notice -i level=info -i "message=too short"
./bin/acme request --as ann alpha add-notice -i level=info -i "message=$(printf 'x%.0s' {1..161})"
./bin/acme request --as ann alpha add-notice -i level=info -i "$(printf 'message=two\nlines here')"
```

**Self-approval and outsiders.** Each of these exits 2 and is recorded as `approve_refused`:

```sh
ID=$(./bin/acme request --as dave beta add-notice -i level=info -i "message=Self approval test here." \
  | python3 -c 'import json,sys; print(json.load(sys.stdin)["id"])')
./bin/acme approve --as dave  $ID   # self_approval
./bin/acme approve --as ann   $ID   # not_in_group (maintainer)
./bin/acme approve --as carol $ID   # not_in_group
./bin/acme approve --as erin  $ID   # succeeds
```

**A group change.** After this, erin is refused and carol may approve:

```sh
./bin/acme group-remove --as dave release erin
./bin/acme group-add --as dave release carol
```

**A rejected push.** The change never reaches `main`, and the run is reported as `failed` with
`reason: push_rejected`:

```sh
printf '#!/bin/sh\necho "main is frozen" >&2\nexit 1\n' > ../origins/gamma.git/hooks/pre-receive
chmod +x ../origins/gamma.git/hooks/pre-receive
ID=$(./bin/acme request --as ann gamma add-notice -i level=info -i "message=This push will be rejected." \
  | python3 -c 'import json,sys; print(json.load(sys.stdin)["id"])')
./bin/acme approve --as erin $ID; echo "exit=$?"     # exit=1, state failed
./bin/acme show $ID                                  # push_attempted event with the remote's message
rm ../origins/gamma.git/hooks/pre-receive
```

A failed run is final. To try again, request a new run.

**Stale preview.** Request a run, push any change to that README on the origin, then approve. The run
fails with `reason: stale` and nothing is pushed.

## Notes

- The notice format comes from `run.py`.
  - The new first line is `> **Info:** <message>` or `> **Warning:** <message>`, then one blank
    line, then the README as it was.
  - An existing leading notice line (`> **Info:** …` or `> **Warning:** …`) followed by a blank line
    is replaced, not stacked.
  - The README's line ending (LF or CRLF) and any UTF-8 BOM are kept.
- If the requested notice is already on `main`, an approved run pushes nothing. It still checks
  `main` and reports `succeeded`.
- Runs are serialized by a lock on `org/runs/.lock`.
- Temporary clones are made under `$TMPDIR` with the prefix `r8-vanilla-3-acme-` and removed
  afterwards.
