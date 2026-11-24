# Handoff: the `add-notice` self-service action

`add-notice` is live on `alpha`, `beta` and `gamma`. Three runs, all requested by ann, have succeeded:

| Run | Repository | Approved by | Commit on `main` |
|---|---|---|---|
| `run-0001` | alpha | dave | `e605cca` |
| `run-0002` | beta | erin | `d4a492b` |
| `run-0003` | gamma | erin | `5e3c926` |

The system is a single command-line tool, `org/bin/acme`, written in Python 3.12 with the standard library only. **Nothing has to be running**: there is no server or daemon, and every step is a shell command that prints one JSON document.

## Layout

| Path | What it is |
|---|---|
| `org/bin/acme` | The CLI and engine: list, describe, preview, request, approve, reject, cancel, status, show, runs, audit, group, member |
| `org/people.json` | Org members, **the release group**, and the org policy (`approvalGroup: release`) |
| `org/repos.json` | The repository registry: each repository's origin and target branch |
| `org/templates/actions/add-notice.{json,py}` | The starter that the repositories copied; it is not used at run time |
| `org/tests/test_acme.py` | End-to-end tests against a throwaway org that the tests build under `org/state/` |
| `<repo>/.acme/actions/add-notice.json` | The repository's action definition: inputs schema, owners, approval group, target branch, script |
| `<repo>/.acme/actions/add-notice.py` | The repository's implementation: it edits the root README |
| `org/state/runs/run-NNNN.json` | One record per run (gitignored, local to this machine) |
| `org/state/audit.jsonl` | Append-only audit log of every event, including refused requests and denied approvals |

`<repo>` means `/tmp/actions-ab/r8/vanilla-2/{alpha,beta,gamma}`. `acme` reads `people.json` and `repos.json` from the **org origin's `main`**, and action definitions from **each repository origin's `main`**. Edits to working copies have no effect until they are pushed; when the org checkout differs from its origin, `acme` adds a `warnings` entry saying so.

## How each requirement is met

| # | How |
|---|---|
| R1 Owned by the repository | Each repository has its own `.acme/actions/add-notice.json` and `add-notice.py`, committed to its origin's `main` (alpha `364d467`, beta `97d83b9`, gamma `3081e72`). `acme` reads them from the origin's `main` (`load_action` in `org/bin/acme`), so a pushed change by the team changes the action. A request pins the definition commit it was validated against (`definition.commit` in the record), and the approved run executes that version. The org policy in `org/people.json` stops a repository from weakening approval: a definition whose `approval.group` is not `release` is listed as `valid: false` and cannot run. |
| R2 Validated | `request` and `preview` check inputs against the definition's JSON Schema (`validate` in `org/bin/acme`): `level` must be `info` or `warning`. `message` must be 10 to 160 characters on one line with no control or line-separator characters and at least one visible character. Unknown and missing inputs are refused. A refusal exits with code 2, creates no run, changes nothing, and is logged as `request_refused` in `org/state/audit.jsonl`. The script checks the inputs again before it writes anything (`check_inputs` in `add-notice.py`). |
| R3 Previewed | `acme preview` clones the origin into a scratch directory, runs the repository's script, and returns the unified diff without pushing. `acme request` also stores that preview (`preview.diff`, `preview.baseCommit`) in the run record, so approvers see it through `acme show`. |
| R4 Approved by release | Nothing is pushed until `acme approve`. At approval time `approve` reads the release group from the org origin's `main`. It refuses (exit 3) the requester, people outside the group, and non-members. Each refusal is recorded as an `approval_denied` event, and the run stays `pending_approval`. |
| R5 Done means observed | After pushing, `observe` reads the origin's `main` and compares the blob id of every changed file with the blob the run produced. Only a match sets `succeeded`, and the record stores what was observed (`result.observed`). A rejected push, a failed script, or a mismatch sets `failed` with an `error` and exits with code 6. A push rejected because `main` moved is retried on the new `main` up to 3 times. Any other rejection fails the run straight away. |
| R6 One place for the org | From `org/`, `bin/acme list` lists the actions of every repository in `org/repos.json`, and `bin/acme request <repo> ...` requests a run on any of them. No repository checkout is needed: `acme` works on the origins through scratch clones in `org/state/work/`. |
| R7 Audited | `org/state/runs/run-NNNN.json` records `requestedBy`/`requestedAt`, `approvedBy`/`approvedAt`, `inputs`, `status`, `finishedAt`, `error`, and the `result` (commit, push log, observation). Its `events` list holds every step with actor and UTC timestamp, including denied approvals. `org/state/audit.jsonl` holds the same events plus refused requests and group changes. The commit on `main` is authored by the requester, committed by `acme-actions`, and carries `Acme-Run`, `Requested-by` and `Approved-by` trailers. |
| R8 Usable by agents | Every command is non-interactive and prints JSON on stdout. Exit codes are 0 ok, 2 invalid input, 3 not allowed, 4 not found, 5 wrong run state (for example, approving a run twice), 6 run failed, 1 internal error. Identity comes from `--as <person>` or `$ACME_USER`. `preview --diff` prints only the diff, for people. |
| R9 Group defined once | The release group exists only in `groups.release` in `org/people.json` on the org origin's `main`. Repository definitions name the group (`"approval": {"group": "release"}`) and never list its members. `acme group add/remove` commits the change to the org origin, and it applies at the next approval in every repository, including runs that are already pending. |

## Commands

Run everything from the org checkout:

```bash
cd /tmp/actions-ab/r8/vanilla-2/org
```

**Start:** nothing. There is no server. The tool needs only `python3` and `git`.

**List the actions** (all repositories, or one):

```bash
bin/acme list
bin/acme list --repo beta
bin/acme describe gamma add-notice
```

**Preview** (changes nothing):

```bash
bin/acme preview alpha add-notice -i level=info -i 'message=A preview of a notice for alpha.'
bin/acme preview alpha add-notice -i level=info -i 'message=A preview of a notice for alpha.' --diff   # just the diff
```

**Request as ann** (validates, stores the preview, prints the run id; the run waits for approval):

```bash
bin/acme request alpha add-notice --as ann -i level=warning -i 'message=This proxy is frozen while the v2 rewrite lands; please file new issues against v2.'
# equivalently: --inputs-json '{"level":"warning","message":"..."}'
```

**Approve as dave or erin** (pushes, observes `main`, and prints the final status):

```bash
bin/acme approve run-0004 --as dave
bin/acme approve run-0004 --as erin
```

Release group members can turn a run down with `bin/acme reject run-0004 --as erin --reason '...'`, and the requester can withdraw it with `bin/acme cancel run-0004 --as ann`.

**Check a run's status / read its record / list runs / read the audit log:**

```bash
bin/acme status run-0004
bin/acme show run-0004
bin/acme runs                      # also --repo alpha, --status pending_approval
bin/acme audit --run run-0004      # or bin/acme audit for everything
```

**Change who is in the release group** (commits `people.json` to the org origin's `main` as the named person, and fast-forwards this checkout):

```bash
bin/acme group show release
bin/acme group remove release erin --as ann
bin/acme group add release carol --as ann
bin/acme member add frank --as ann && bin/acme group add release frank --as ann   # people outside the org must join first
```

Another way is to edit `people.json` in this checkout, then `git commit -am '...' && git push origin main`. It takes effect only once pushed.

## Things a tester will try, and what to expect

| Try | Command | Expected |
|---|---|---|
| Invalid inputs | `bin/acme request alpha add-notice --as ann -i level=error -i 'message=short'` (also 161 characters, a message containing `$'\n'`, a missing or extra input) | exit 2, `invalid_inputs`, no run created, `main` unchanged |
| Self-approval | `bin/acme request beta add-notice --as dave -i level=info -i 'message=Requested by a release member.'` then `bin/acme approve <id> --as dave` | exit 3, `self_approval`; erin can approve it |
| Approval from outside the group | `bin/acme approve <id> --as carol` (also `ann`, `bob`, or an unknown name) | exit 3, `not_in_approval_group` / `not_a_member`; the run stays `pending_approval` |
| Group change | `bin/acme group remove release erin --as ann` then `bin/acme approve <id> --as erin` | exit 3; after `group add release carol --as ann`, carol can approve, in every repository |
| Rejected push | `printf '#!/bin/sh\necho protected >&2\nexit 1\n' > ../origins/alpha.git/hooks/pre-receive && chmod +x ../origins/alpha.git/hooks/pre-receive`, request, approve as dave, then `rm ../origins/alpha.git/hooks/pre-receive` | exit 6, status `failed` with the hook's message in `error`, `main` unchanged; it is never reported as succeeded |
| Approve twice | `bin/acme approve run-0001 --as erin` | exit 5, `not_pending` |

A failed run stays failed. To try again, request a new run. The tests run all of these cases against a throwaway org: `python3 -m unittest discover -s tests -v` (16 tests, all passing).

## Limits

- No passwords: `--as` is trusted, as the brief allows. `acme` records every actor, but anyone who can run it can claim any name.
- Any org member may change the release group through `acme group`. The change is a commit on the org origin, authored by that person and logged in `audit.jsonl`. In a real deployment, branch protection on `org` main would be the control.
- Run records live in `org/state/`, on this machine. They are not in git.
- The engine runs the repository's own script (pinned to the definition commit that was requested), the same trust model as CI. A script gets 60 seconds.
