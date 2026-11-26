# add-notice: handoff

`add-notice` is a Seikika self-service action that alpha, beta and gamma each own in their own
`.seikika/`. It is offered, approved and run from one org portal in this directory
(`/tmp/actions-ab/r8/seikika-1/org`). All three requested runs succeeded and were observed on the
origins' `main`:

| Repository | Org run id | Requested by | Approved by | State | Commit on `main` |
|---|---|---|---|---|---|
| alpha | `run_2fw2x7kzrzzj` | user:ann | user:dave | succeeded | `3c114ad` (`README.md`) |
| beta | `run_2fw2xcquz4wb` | user:ann | user:erin | succeeded | `1c385bd` (`readme.md`) |
| gamma | `run_2fw2xh6n1bbo` | user:ann | user:erin | succeeded | `7ed2463` (`README.md`) |

Each org run dispatches one child run in the repository's portal (`run_2fw30ngboear`,
`run_2fw30y26tvwh`, `run_2fw3163cyknj`), which is why `--runs` lists six records. Use the org run
ids above. Each child run id is in its org run's `verification.child_run` and in the commit's
`Seikika-Run` trailer.

## Nothing needs to be running

The org portal is used from its checkout: `seikika run` in this directory does everything, with no
server or database. Seikika keeps the org's approval signing key, its clones of the three
repositories and every run record under one state directory. Every command must use the same one:

```sh
cd /tmp/actions-ab/r8/seikika-1/org
. ./env.sh      # PATH=/tmp/actions-ab/r8/seikika-1/bin:$PATH, SEIKIKA_STATE_DIR=/tmp/actions-ab/r8/seikika-1/state
```

| Path | What it is |
|---|---|
| `/tmp/actions-ab/r8/seikika-1/state/served/org/signing-key` | The key the org signs approvals with. Its public half is pinned in each repository |
| `/tmp/actions-ab/r8/seikika-1/state/runs/<run id>.json` | One record per run |
| `/tmp/actions-ab/r8/seikika-1/state/federation/org-*/<repo>` | The org's clones of the repositories, fetched again from the origin before each list, preview and run |

## Commands

Run every command from `/tmp/actions-ab/r8/seikika-1/org` after `. ./env.sh`. Every command takes
`-o json`. Exit codes: 0 succeeded, 1 failed, 2 invalid inputs, blocked or refused, 3 awaiting
approval, 4 unverified.

```sh
# List add-notice for all three repositories (inputs, approver, entity ids portal:alpha|beta|gamma)
seikika run --as ann -o json

# Preview: validates the inputs and shows the README diff. Changes nothing and records no run
seikika run --as ann add-notice --entity portal:beta -i level=info -i 'message=Releases now ship on the shared release train every second Tuesday.' -o json

# Request as ann: exit 3, the JSON record's "id" is the run id, "state" is awaiting_approval
seikika run --as ann add-notice --entity portal:beta -i level=info -i 'message=Releases now ship on the shared release train every second Tuesday.' --yes -o json
#   all inputs at once also works: --inputs '{"level":"warning","message":"..."}'

# Approve as dave or erin: pushes to main, then verifies main; exit 0 only once main has it
seikika run --as erin --approve <run-id> --note "release sign-off" --wait -o json
#   reject instead:  seikika run --as erin --reject <run-id> --note "why"

# Status of one run (exit code + "state")
seikika run --as ann --verify <run-id> -o json | jq -r .state

# Its record: requester, approvers, inputs, outcome, and when each happened
seikika run --as ann --verify <run-id> -o json | jq '{id, state, requested_by, inputs, approval, error, events, verification}'

# Every run
seikika run --as ann --runs -o json
```

The record is also the file `/tmp/actions-ab/r8/seikika-1/state/runs/<run-id>.json`.

### Change who is in the release group

Edit the `group:release` tuples in `/tmp/actions-ab/r8/seikika-1/org/seikika.yaml`, then
validate. The change takes effect on the next approval for all three repositories, including runs
already waiting. Commit and push it so the org's history records it:

```sh
cd /tmp/actions-ab/r8/seikika-1/org && . ./env.sh
# e.g. replace erin with carol:
sed -i 's/{user: user:erin, relation: member, object: group:release}/{user: user:carol, relation: member, object: group:release}/' seikika.yaml
seikika validate . && git commit -qam "release group: erin out, carol in" && git push -q origin main
```

To add someone, add a line `- {user: user:<name>, relation: member, object: group:release}`.
Nested groups work too: `{user: group:<other>#member, relation: member, object: group:release}`.

### Make a push fail (for testing R5)

The origins are bare repositories, so a `pre-receive` hook plays a protected branch:

```sh
printf '#!/bin/sh\necho "main is protected" >&2\nexit 1\n' > /tmp/actions-ab/r8/seikika-1/origins/gamma.git/hooks/pre-receive
chmod +x /tmp/actions-ab/r8/seikika-1/origins/gamma.git/hooks/pre-receive
# request with a new message, approve: the approval exits 1 and the run is "failed" with the origin's message
rm /tmp/actions-ab/r8/seikika-1/origins/gamma.git/hooks/pre-receive   # undo
```

## How each requirement is met

| # | How | Files |
|---|---|---|
| R1 Owned by the repository | Each repository's action is defined in its own `.seikika/`, committed to its origin's `main` (commits `3fa35b9`, `769e43e`, `0af2477`). The org portal reads only what is on each origin's `main`: it fetches its clone before every list, preview and run, so a change that is not pushed to `main` has no effect. To change the action, a maintainer edits these files, runs `seikika validate .seikika && seikika test .seikika`, and pushes. | `<repo>/.seikika/seikika.yaml` (portal name, `owner: group:maintainers`, include), `<repo>/.seikika/actions/add-notice.yaml` (inputs, approver, executor, pinned org key), `<repo>/.seikika/scripts/add-notice.sh` (the README edit), `<repo>/.seikika/tests/add-notice.test.yaml`, `<repo>/.seikika/tests/add-notice-script.sh` |
| R2 Validated | The action's `inputs` JSON Schema requires `level` ∈ {info, warning} and a `message` of 10–160 characters (counted as characters, not bytes) matching `^[^\r\n]*$` (one line), with `additionalProperties: false`. Seikika checks the schema before any preview or request and exits 2 with each error. Nothing is recorded or changed. The script checks the level and the single line again. | `<repo>/.seikika/actions/add-notice.yaml` (`inputs`), `<repo>/.seikika/scripts/add-notice.sh` |
| R3 Previewed | `seikika run add-notice ...` without `--yes` runs the script in a scratch worktree of `main` and prints the exact README diff, the push it would make and the approver. A request (`--yes`) records the same preview as the run's `effects`. If the action or `main` changes after the preview so the result would differ, the approved run fails as `preview_stale` before it commits (tested). | `<repo>/.seikika/actions/add-notice.yaml` (`executor: change`) |
| R4 Approved by the release group | `approval: "['group:release']"`: one member of `group:release` must approve, and the requester never counts. The org portal holds approvals and signs them with its key. Each repository accepts only approvals signed with the key pinned in `federation.acceptApprovalsFrom`, so a run requested from a repository's own checkout stops at `awaiting_approval` (tested). Refusals exit 2 and are recorded as `approval_refused` events: ann approving her own run, bob and carol (not in the group), and dave approving a run dave requested. | `<repo>/.seikika/actions/add-notice.yaml` (`approval`, `acceptApprovalsFrom`), `org/seikika.yaml` (`authz`), `org/groups.fga` |
| R5 Done means observed | `executor: change` with `land: push`. Once approved, Seikika commits in a fresh worktree of `main`, pushes (never forced), fetches the origin's `main`, and reports `succeeded` only if the commit is on it and every changed file's blob matches the preview (`verification.child.files[].matches`). A rejected push fails the run with the origin's message (exit 1, `main` unchanged; tested with a `pre-receive` hook). The three READMEs were also checked byte for byte: the notice line, a blank line, then the previous README. | `<repo>/.seikika/actions/add-notice.yaml` (`executor`) |
| R6 One place for the org | `org/seikika.yaml` lists the three origins as `federation.children` (`portal:alpha`, `portal:beta`, `portal:gamma`). `seikika run` in `org/` lists all three repositories' `add-notice` and requests, approves and checks runs on any of them. | `org/seikika.yaml`, `org/env.sh` |
| R7 Audited | Each run record has `requested_by`, `inputs`, `approval.approved_by` and `approval.satisfied`, `state`, `error`, `effects` (the diff), `verification` (commit and blobs seen on `main`) and `events`. Every event has an `at` timestamp and a `by` person: previewed, confirmed, awaiting_approval, each approval_refused, approved (with the note), dispatched, running, succeeded or failed. The commit on `main` carries `Seikika-Run`, `Requested-By` and `Approved-By` trailers. | `state/runs/<run id>.json`, `git -C origins/<repo>.git log main` |
| R8 Usable by agents | Each step is one non-interactive `seikika run` command with `-o json` and the exit codes above. `--as` (or `SEIKIKA_AS`) says who is acting, and `--wait` blocks until the run settles. There are no prompts. | Commands above |
| R9 The group is defined once | `group:release` is defined only by the `authz` tuples in `org/seikika.yaml`, with the model `org/groups.fga`. The repositories name `group:release` but never its members. Their test files' `members:` is a test-only stand-in, used by `seikika test` and ignored by real runs. Editing the org tuples changed who may approve on the next approval (tested: with erin replaced by carol, erin was refused and carol's approval landed). | `org/seikika.yaml`, `org/groups.fga` |

## How the README is changed

`.seikika/scripts/add-notice.sh` finds the one root file named `README.md` in any letter case. It
fails if there is none or more than one. It writes `> **Info:** <message>` (or `> **Warning:**`),
one blank line, then the README unchanged, byte for byte. If line 1 matches
`^> \*\*(Info|Warning):\*\* .+$` and line 2 is empty, those two lines are replaced instead, so a
README never has more than one notice. Repeating a notice that is already there changes nothing,
and that run succeeds without a commit.

## Tests

| Command | What it checks |
|---|---|
| `cd <repo> && seikika validate .seikika && seikika test .seikika` | 13 lifecycle cases in a scratch copy of the repository and its origin's `main`: success for each approver, invalid level, length bounds (10 and 160 are accepted, 9 and 161 refused), two lines, a missing input, an unknown input, self-approval and non-members refused, and a rejected push failing the run |
| `cd <repo> && sh .seikika/tests/add-notice-script.sh` | 13 byte-exact README cases: new notice, lowercase `readme.md`, no final newline, replace rather than stack, repeat is a no-op, look-alike lines kept, shell metacharacters kept literal, no README fails, two READMEs fail |
| `cd org && seikika validate .` | The org manifest and its groups |

## Limits

- `--as` is taken on trust, as this sandbox requires. Anyone who can read
  `/tmp/actions-ab/r8/seikika-1/state` holds the signing key and could approve as anyone. For a
  real team, serve the org portal behind sign-in (`seikika serve org` with `SEIKIKA_AUTH_MODE`)
  and use `seikika run --url`. That path was not set up or tested here.
- `executor.run` in each repository runs on the machine that approves the run. A maintainer who
  changes it on `main` changes what runs, which is the point of R1. The release group approves
  each run's previewed effect, not the script.
