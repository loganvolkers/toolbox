# HANDOFF: the `mark-deprecated` self-service action

`mark-deprecated` is built and has run on all three repositories. ann requested each run, bob
approved it, and each run was reported `succeeded` only after `DEPRECATED.md` with the exact
content was observed on the origin's `main`:

| Repository | Run | Status | Commit on `origins/<repo>.git` `main` |
|---|---|---|---|
| alpha | `run-20261124T112636Z-50ce91` | succeeded | `d6a7f1a` |
| beta | `run-20261124T112636Z-96a2b2` | succeeded | `6d37d53` |
| gamma | `run-20261124T112636Z-484ea4` | succeeded | `385f9df` |

The system is one Python 3 script, `org/acme`, with no dependencies beyond `git`. **Nothing has
to be running:** there is no server. Every command reads the origins directly and keeps run
records under `org/state/`.

## Layout

| Path | What it is | Owner |
|---|---|---|
| `<repo>/.acme/actions/mark-deprecated.json` (on each origin's `main`) | The action's definition: maintainers, input rules, the effect and the file template | The repository's team |
| `org/acme` | The CLI: list, describe, preview, request, approve, reject, status, record, runs | Org |
| `org/repos.json` | Which repositories exist and where their origins are | Org |
| `org/people.json` | Who exists (ann, bob, carol). Unknown names are refused | Org |
| `org/state/runs/<run-id>.json` | One record per run, with its full event history (not committed, `.gitignore`d) | Written by `acme` |
| `org/state/audit.ndjson` | Append-only log of every event of every run | Written by `acme` |
| `org/tests/test_acme.py` | End-to-end tests on throwaway origins (invalid inputs, self-approval, carol, rejected push, tampered `main`) | Org |

## How each requirement is met

| # | How |
|---|---|
| R1 Owned by the repository | The definition is `.acme/actions/mark-deprecated.json` in each repository, committed to its origin's `main` (alpha `1ff70c9`, beta `2a1c408`, gamma `2c37db0`). `acme` reads it from the origin's `main` at request time (`git cat-file blob main:.acme/actions/<name>.json`), never from a copy in `org`. The team changes inputs, maintainers, template or target path by committing to that file. Each run pins the definition commit it was requested against (`definition.commit` in the record), so approval and execution use exactly what was previewed. `org/repos.json` only says where the repositories are |
| R2 Validated | `validate_inputs` in `org/acme` applies the definition's rules (`pattern` as a full match of `^[a-z0-9-]+/[a-z0-9-]+$`, `minLength` 20, `maxLength` 200, `required`, `notThisRepository` so the successor must be another repository) and refuses unknown inputs. Both `preview` and `request` run it, before any run is recorded or anything is pushed. Failure: exit code 2, `"error": "invalid_inputs"`, one entry per problem in `errors`. A request that would change nothing is refused with exit 5, `nothing_to_change` |
| R3 Previewed | `acme preview` (no identity needed) returns the base commit of `main`, and for each file the operation (`create`/`update`/`unchanged`), `before`, `after` and a unified `diff`. `acme request` returns the same preview and stores it in the run record, so the requester and the approver see what will change |
| R4 Approved | `acme request` only records the run as `pending_approval`. `acme approve` executes it only if the approver is in the pinned definition's `maintainers` (ann, bob) **and** is not the requester. Self-approval gives exit 3 `self_approval`, carol gives exit 3 `not_maintainer`, an unknown person gives exit 3 `unknown_person`, and no identity gives exit 2. Refused attempts are written to the record as `approval_refused` events. A run that is not pending cannot be approved again (exit 5). Maintainers can `reject`; the requester can cancel with `reject` |
| R5 Done means observed | After approval, `execute` clones the origin, writes the file, commits (author = requester, committer `acme-actions`, trailers `Acme-Run`, `Requested-by`, `Approved-by`) and pushes to `main`. Then `observe` reads `DEPRECATED.md` back from the origin's `main` and compares bytes with the rendered content. The run is `succeeded` only if the push was accepted (or nothing needed pushing) **and** the observed bytes match. A rejected push, or a `main` that does not have the content afterwards, gives `failed` (exit 6) with the reason in `outcome.message`; the `observed` event records what was actually seen |
| R6 One place for the org | Every command runs from `org/`: `./acme list` shows the action for all three repositories, and `request`/`approve` take the repository name. Nobody has to enter `alpha`, `beta` or `gamma` |
| R7 Audited | `./acme record <id>` returns the full record: `requestedBy`, `approvedBy`, `inputs`, `outcome`, `definition`, `preview` and the timestamped events `requested` → (`approval_refused`) → `approved` → `executed` (push result and commit) → `observed` → `succeeded`/`failed`, each with `at` (UTC) and `actor`. `./acme status <id>` summarises it with `requestedAt`, `approvedAt` and `finishedAt`. Every event also goes to `org/state/audit.ndjson`. The commit on `main` carries the run id, requester and approver |
| R8 Usable by agents | Every command is non-interactive and prints one JSON document on stdout, with `"ok": true/false`. Exit codes: 0 ok, 1 error, 2 invalid input, 3 forbidden, 4 not found, 5 wrong state (or nothing to change), 6 run executed and failed. Identity is `--as <person>` or `ACME_ACTOR`. Inputs are `-i name=value` (repeatable) or `--inputs-json '{...}'`. `./acme --help` and `./acme <command> --help` describe each command |

## Commands

All commands run from the org directory. Nothing needs to be started.

```bash
cd /tmp/actions-ab/r2/vanilla-2/org

# List the action on every repository (and one definition in full)
./acme list
./acme describe alpha mark-deprecated

# Preview
./acme preview alpha mark-deprecated \
  -i successor=acme/dependabot-proxy-v2 \
  -i "reason=Replaced by the v2 proxy, which supports OIDC registries."

# Request as ann (prints the run id in "id"; the run is pending_approval)
./acme request alpha mark-deprecated --as ann \
  -i successor=acme/dependabot-proxy-v2 \
  -i "reason=Replaced by the v2 proxy, which supports OIDC registries."
RUN=$(./acme request beta mark-deprecated --as ann \
  --inputs-json '{"successor": "acme/new-home", "reason": "A different reason, long enough."}' \
  | python3 -c 'import json,sys; print(json.load(sys.stdin)["id"])')

# Approve as bob (executes, pushes, observes; prints the final status)
./acme approve "$RUN" --as bob

# Reject (maintainer) or cancel (requester) instead
./acme reject "$RUN" --as bob --comment "not yet"

# Status, record, and all runs
./acme status "$RUN"
./acme record "$RUN"
./acme runs
./acme runs --repo alpha --status succeeded

# Tests on throwaway origins (does not touch the real ones)
python3 -m unittest discover -s tests -v
```

## Testing the rules by hand

`DEPRECATED.md` is already on all three `main` branches, so a request with the **same** inputs as
the original runs is refused as `nothing_to_change` (exit 5). Use a different `reason` or
`successor` to get a run that changes something.

```bash
cd /tmp/actions-ab/r2/vanilla-2/org

# Invalid inputs: each exits 2 with "invalid_inputs"; no run is recorded, nothing is pushed
./acme preview alpha mark-deprecated -i successor=Acme/Proxy -i "reason=Long enough reason text here."
./acme request alpha mark-deprecated --as ann -i successor=acme/proxy -i "reason=too short"
./acme request alpha mark-deprecated --as ann -i successor=acme/alpha -i "reason=Not another repository, so refused."
./acme request alpha mark-deprecated --as ann -i successor=acme/proxy -i "reason=$(printf 'x%.0s' {1..201})"

# Self-approval and carol: each exits 3; the run stays pending_approval
RUN=$(./acme request alpha mark-deprecated --as ann -i successor=acme/other-proxy \
  -i "reason=Testing the approval rules by hand." | python3 -c 'import json,sys; print(json.load(sys.stdin)["id"])')
./acme approve "$RUN" --as ann
./acme approve "$RUN" --as carol
./acme status "$RUN"

# Rejected push: the run is reported failed (exit 6) and main is unchanged
printf '#!/bin/sh\necho "main is protected" >&2\nexit 1\n' > ../origins/alpha.git/hooks/pre-receive
chmod +x ../origins/alpha.git/hooks/pre-receive
./acme approve "$RUN" --as bob
git --git-dir ../origins/alpha.git show main:DEPRECATED.md
rm ../origins/alpha.git/hooks/pre-receive
```

## Limits

- Identity is asserted (`--as`), as the sandbox has no passwords. In a real deployment `acme`
  would take identity from the code host or an SSO token, and only the `acme-actions` identity
  would be allowed to push.
- Who may change a definition is the code host's concern: anyone who can push to a repository's
  `main` can change its `maintainers`. A run uses the definition pinned at request time.
- Run records are files under `org/state/`, guarded by `flock`. They are not committed to the org
  origin.
- Approval executes the run in the same command. A failed run is not retried; request a new one.
