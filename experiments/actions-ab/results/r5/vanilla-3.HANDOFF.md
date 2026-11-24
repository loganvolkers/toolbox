# Handoff: `mark-deprecated` self-service action

The system is `acme`, a single-file Python 3.12 command-line tool (standard library only) in this
directory. **No server and no long-running process are involved.** Every command runs
non-interactively and prints one JSON document on stdout.

The three requested runs succeeded: ann requested each one and bob approved it.

| Repository | Run id | Commit on origin `main` |
|---|---|---|
| alpha | `run-20261124130512-alpha-3eb377` | `e3906b3` |
| beta | `run-20261124130512-beta-ee2fc0` | `8bbb0c7` |
| gamma | `run-20261124130513-gamma-3ac876` | `ceaa2db` |

## Files

| Path | What it is |
|---|---|
| `<repo>/.acme/actions/mark-deprecated.json` (in `alpha`, `beta`, `gamma`, on each origin's `main`) | The repository's own action definition: inputs and their validation, approvers, and the effect (the file, branch and template) |
| `org/acme/acme.py` | The engine and CLI |
| `org/bin/acme` | Wrapper that runs it: `./bin/acme …` |
| `org/acme.json` | Org config: org name (`acme`), people (`ann`, `bob`, `carol`), repositories and their origins (relative paths), and the state directory |
| `org/.acme-state/runs/<run-id>.json` | One record per run (gitignored runtime state) |
| `org/.acme-state/audit.jsonl` | Append-only audit log of every event, including refused requests and refused approvals |
| `org/tests/test_acme.sh` | 39 end-to-end checks run against throwaway copies of the origins (needs `jq`) |

Exit codes: `0` ok, `2` invalid input, `3` forbidden (wrong or missing identity, self-approval,
non-maintainer), `4` not found, `5` the run did not succeed, `6` the run is not awaiting approval.
On errors the JSON is `{"ok": false, "error": {"kind", "message", "details"}}`.

## How each requirement is met

| # | How |
|---|---|
| R1 Owned by the repository | Each repository's definition is `.acme/actions/mark-deprecated.json`, committed to its origin's `main` (alpha `7a83f11`, beta `04a4358`, gamma `610d519`). The engine has no built-in actions. It reads definitions only from the origin's `main` (`git cat-file` on `origins/<repo>.git`, see `load_definition` in `org/acme/acme.py`) and discovers them by listing `.acme/actions/*.json`. The repository team changes the inputs, approvers or template by pushing to `main`. A run is pinned to the definition commit current when it was requested (`run.definition.commit`), and approval rules are read from that pinned commit. |
| R2 Validated | `validate_inputs` in `org/acme/acme.py` applies the definition's rules: `successor` must match `^[a-z0-9-]+/[a-z0-9-]+$` and must not be `acme/<this repo>` (`notSelf`). `reason` must be 20 to 200 characters, inclusive. Unknown inputs and missing inputs are refused. `preview` and `request` both refuse invalid inputs with exit 2 and a per-input error list. No run is created and nothing is written to any origin. A refused request is still logged in `audit.jsonl` as `request-refused`. Inputs are validated again at approval time. |
| R3 Previewed | `acme preview` renders the file from the definition's template against the origin's current `main` and returns the path, the change kind (`create`, `modify` or `unchanged`), the full new content and a unified diff. It writes nothing. `acme request` stores the same preview in the run record (`run.preview`) for the approver, together with the base commit it was computed against. |
| R4 Approved | `acme approve` is the only path that changes anything (see `decide` in `org/acme/acme.py`). It refuses with exit 3 when the approver is the requester, or is not listed in the pinned definition's `approval.approvers` (carol is not listed, so she is refused). Self-approval is refused by the engine, whatever a definition says. Each refused approval is recorded on the run (`approval-refused`). Only a run in `awaiting-approval` can be approved or rejected; otherwise the exit code is 6. Identity comes from `--as <person>` or `ACME_USER` and must be a person in `org/acme.json`. |
| R5 Done means observed | `execute` in `org/acme/acme.py` clones the origin, writes the file and commits it. The author is the requester, the committer is `acme-actions`, and the message carries the run id, requester and approver. The commit is made even if the content is already present, so every run has its own commit. It pushes to `main` without force. Afterwards it reads the origin directly and sets `succeeded` only if all three of these hold: the push was accepted, the run's commit is on `main` (`merge-base --is-ancestor`), and `main:DEPRECATED.md` is byte-for-byte the expected content. Otherwise the run is `failed`, `outcome.reasons` gives the cause (for example `push rejected: … pre-receive hook declined`), and `approve` exits 5. The observation is stored in `run.observation`. Non-fast-forward rejections are retried up to 3 times on a fresh clone. Other rejections are not retried. |
| R6 One place for the org | Everything runs from `org/`. `./bin/acme actions` lists every repository's actions straight from their origins. `preview`, `request`, `approve`, `status`, `record`, `runs` and `audit` all take the repository by name. You never need to enter a repository checkout. |
| R7 Audited | Each run's record, `org/.acme-state/runs/<id>.json` (`acme record <id>`), holds `requestedBy`/`requestedAt`, `approvedBy`/`approvedAt` (or `rejectedBy`/`rejectedAt`), `inputs`, the pinned definition commit, the preview, the observation, and `outcome` with its status and time. It also holds an `events` list of timestamped, actor-attributed events: `requested`, `approval-refused`, `approved`, `started`, `committed`, `pushed`/`push-rejected`, `observed`, then `succeeded`/`failed`. The same events, plus refused requests, are appended to `org/.acme-state/audit.jsonl` (`acme audit`). The commit on `main` also names the run, the requester and the approver. |
| R8 Usable by agents | Every step is a subcommand with flags only, no prompts, with JSON on stdout and meaningful exit codes. `./bin/acme --help` and `./bin/acme <cmd> --help` describe them. Inputs can be passed as `-i key=value` or `--inputs-json '{"successor": "…", "reason": "…"}'`. |

## Commands

Run these from the org directory. Nothing has to be started first.

```sh
cd /tmp/actions-ab/r5/vanilla-3/org
```

List the actions on all three repositories, and show one full definition:

```sh
./bin/acme actions
./bin/acme actions --repo alpha --show mark-deprecated
```

Preview. Needs no identity and changes nothing:

```sh
./bin/acme preview alpha mark-deprecated -i successor=acme/dependabot-proxy-v2 -i "reason=Replaced by the v2 proxy, which supports OIDC registries."
```

Request as ann. Prints the run, including `run.id` and `run.preview`:

```sh
./bin/acme request alpha mark-deprecated --as ann -i successor=acme/dependabot-proxy-v2 -i "reason=Replaced by the v2 proxy, which supports OIDC registries."
RUN=$(./bin/acme request alpha mark-deprecated --as ann -i successor=acme/dependabot-proxy-v2 -i "reason=Replaced by the v2 proxy, which supports OIDC registries." | jq -r .run.id)
```

Approve as bob. This executes the run, observes `main` and prints the final run (exit 0 if succeeded, 5 if not):

```sh
./bin/acme approve "$RUN" --as bob
./bin/acme reject "$RUN" --as bob --comment "why"   # alternatively
```

Check status, read the record, list runs and read the audit log:

```sh
./bin/acme status "$RUN"
./bin/acme record "$RUN"
./bin/acme runs                     # optional: --repo alpha --status succeeded
./bin/acme audit                    # optional: --run "$RUN"
```

The records of the three completed runs:

```sh
./bin/acme record run-20261124130512-alpha-3eb377
./bin/acme record run-20261124130512-beta-ee2fc0
./bin/acme record run-20261124130513-gamma-3ac876
```

### Negative tests to try

```sh
# R2: each is refused with exit 2; no run is created
./bin/acme request alpha mark-deprecated --as ann -i successor=Acme/X -i "reason=Long enough reason for the test."
./bin/acme request alpha mark-deprecated --as ann -i successor=acme/alpha -i "reason=Long enough reason for the test."
./bin/acme request alpha mark-deprecated --as ann -i successor=acme/x -i "reason=too short"

# R4: exit 3, and the run stays awaiting-approval
RUN=$(./bin/acme request beta mark-deprecated --as ann -i successor=acme/x -i "reason=Long enough reason for the test." | jq -r .run.id)
./bin/acme approve "$RUN" --as ann
./bin/acme approve "$RUN" --as carol
./bin/acme status "$RUN"

# R5: make the origin reject pushes, then approve. The run ends as failed (exit 5) and main is unchanged.
printf '#!/bin/sh\necho "main is protected" >&2\nexit 1\n' > ../origins/beta.git/hooks/pre-receive
chmod +x ../origins/beta.git/hooks/pre-receive
./bin/acme approve "$RUN" --as bob
./bin/acme record "$RUN" | jq '.run.status, .run.outcome, .run.observation'
rm ../origins/beta.git/hooks/pre-receive
```

The automated suite (`bash tests/test_acme.sh`) covers all of the above and more. It runs against
copies of the origins in `/tmp/r5-vanilla-3-acme-test.*`, which it deletes afterwards.

`DEPRECATED.md` already exists on all three origins. A new run whose inputs match the current
content still makes its own commit and pushes it, so a rejected push still fails that run.
