# HANDOFF: `mark-deprecated`, a repository-owned self-service action

The system is one stdlib-only Python 3.12 CLI, `org/bin/acme` (code in `org/acme.py`). Nothing has to be
running: there is no server, and every command runs to completion and prints one JSON document to stdout.
Each repository defines its own action on its origin's `main`; the org-level CLI finds and runs those
definitions from `/tmp/actions-ab/r2/vanilla-3/org`.

All three requested runs have succeeded:

| Repository | Run | Requested by | Approved by | Observed commit on origin `main` |
|---|---|---|---|---|
| alpha | `run-20261124T112823Z-c4ba1d` | ann | bob | `8cd0c51` |
| beta | `run-20261124T112824Z-8db89c` | ann | bob | `626c6cb` |
| gamma | `run-20261124T112824Z-36ede6` | ann | bob | `37fa321` |

## Layout

| Path | What it is |
|---|---|
| `<repo>/.acme/actions/mark-deprecated.json` (alpha, beta, gamma) | The action: inputs and their validation, the approval rule, the file it writes, the commit message |
| `<repo>/.acme/maintainers.json` | The repository's maintainers (ann, bob). They are the only people who can approve its runs |
| `org/acme.py`, `org/bin/acme` | The org runner: list, describe, preview, request, approve, reject, cancel, status, show, runs, reconcile |
| `org/repos.json` | The org's repositories and their origins |
| `org/people.json` | Known people (ann, bob, carol). Anyone else is refused |
| `org/state/runs/<run-id>.json` | One record per run (not committed; `org/.gitignore`) |
| `org/state/audit.jsonl` | Append-only log of every event across all runs, including refused approvals |

## How each requirement is met

| # | How |
|---|---|
| R1 Owned by the repository | Each repository's definition is `.acme/actions/mark-deprecated.json` and its approvers are `.acme/maintainers.json`, committed to that repository's origin `main` (alpha `072c02d`, beta `1afa99e`, gamma `311d524`). The runner reads them from the origin's `refs/heads/main` on every command (`load_definition` in `org/acme.py`), never from the org, so a change by the repository's team takes effect when it lands on `main`. A run pins the definition blob it was requested against; if the definition changes before approval, approval fails the run with `definition_changed` and it must be requested again. |
| R2 Validated | `validate_inputs` in `org/acme.py` applies the definition's rules before anything is recorded or changed: `successor` must match `^[a-z0-9-]+/[a-z0-9-]+$` and must not be the repository itself; `reason` must be 20 to 200 characters and a single line; unknown and missing inputs are refused. `preview` and `request` both validate, exit 2 and return every error under `error.details`. No run is created for invalid inputs. |
| R3 Previewed | `acme preview` returns the planned change (path, `create`/`modify`/`unchanged`, full new content, a unified diff, the base commit) without creating a run. `acme request` returns the same preview and stores it in the run record. At execution the runner rechecks that the files are still as previewed; if `DEPRECATED.md` changed on `main` in between, the run fails with `stale_preview` instead of applying something the requester never saw. |
| R4 Approved | A run starts in `pending_approval` and changes nothing until `acme approve`. Approval requires that the approver is in the repository's current `.acme/maintainers.json` and is not the requester (`allowSelfApproval: false` in the definition). Self-approval fails with exit 3 and `self_approval`; carol's approval fails with exit 3 and `not_a_maintainer`; both refusals are recorded as `approve_denied` events. A run can be approved once only (`not_pending`, exit 5). Maintainers can `reject`; the requester or a maintainer can `cancel`. |
| R5 Done means observed | `execute` in `org/acme.py` clones the origin, commits, and pushes to `refs/heads/main`. A rejected push fails the run (`push_rejected`, with git's output); a non-fast-forward race is retried twice from a fresh clone. After the push, the runner reads `DEPRECATED.md` straight from the origin's `refs/heads/main` and compares it byte for byte with the previewed content, and checks that the pushed commit is on `main`. Only then is the run `succeeded`, with the observed commit and the file's SHA-256 in `outcome.observed`. Otherwise it is `failed` with `not_observed`. Tested before the real runs on a scratch copy of the origins: a `pre-receive` hook that rejects gave `failed/push_rejected`; a `post-receive` hook that reset `main` after an accepted push gave `failed/not_observed`. |
| R6 One place for the org | From `/tmp/actions-ab/r2/vanilla-3/org`, `bin/acme list` lists the actions of every repository in `org/repos.json` (read from each origin), and `bin/acme request <repo> mark-deprecated ...` requests a run on any of them. Nobody has to enter a repository checkout. |
| R7 Audited | Each run record (`org/state/runs/<run-id>.json`, read with `acme show`) holds `requested_by`/`requested_at`, `approved_by`/`approved_at`, `inputs`, `started_at`, `finished_at`, `state` and `outcome`, the pinned definition and the preview, and an `events` list with a UTC timestamp and actor for each step (`requested`, `approve_denied`, `approved`, `started`, `pushed`/`push_rejected`, `observed`, `succeeded`/`failed`, `rejected`, `cancelled`). The same events are appended to `org/state/audit.jsonl`. The commit on `main` is authored by the requester, committed by `acme-actions`, and carries `Acme-Run`, `Requested-by` and `Approved-by` trailers. |
| R8 Usable by agents | Every command is non-interactive, takes identity from `--as <person>` or `ACME_ACTOR`, takes inputs as `-i key=value` or `--inputs-json '{...}'`, and prints JSON to stdout: `{"ok": true, ...}` or `{"ok": false, "error": {"code", "message", "details"}}`. Exit codes: 0 ok, 1 run failed, 2 invalid input or usage, 3 forbidden, 4 not found, 5 conflict. Approval runs the change synchronously, so the `approve` reply already holds the final state. |

## Commands

Nothing needs to be started. Run these from the org directory:

```sh
cd /tmp/actions-ab/r2/vanilla-3/org
```

List the actions on all three repositories, or describe one:

```sh
bin/acme list
bin/acme list --action mark-deprecated
bin/acme describe alpha mark-deprecated
```

Preview without creating a run:

```sh
bin/acme preview alpha mark-deprecated -i successor=acme/dependabot-proxy-v2 \
  -i "reason=Replaced by the v2 proxy, which supports OIDC registries."
```

Request a run as ann. The reply holds `run.id` and the preview:

```sh
bin/acme request alpha mark-deprecated --as ann -i successor=acme/dependabot-proxy-v2 \
  -i "reason=Replaced by the v2 proxy, which supports OIDC registries."
RUN=$(bin/acme request beta mark-deprecated --as ann \
  --inputs-json '{"successor":"acme/valtio-store","reason":"State management moves to the shared store package."}' \
  | python3 -c 'import json,sys; print(json.load(sys.stdin)["run"]["id"])')
```

Approve as bob. This executes the run and returns its final state:

```sh
bin/acme approve "$RUN" --as bob
```

Check a run's status, read its full record, and list runs:

```sh
bin/acme status "$RUN"
bin/acme show "$RUN"
bin/acme runs
bin/acme runs --repo alpha --state succeeded
```

Reject, cancel, and finish an interrupted run:

```sh
bin/acme reject "$RUN" --as bob --comment "not yet"
bin/acme cancel "$RUN" --as ann
bin/acme reconcile "$RUN" --as bob   # only for a run left in approved/running by a killed process
```

## Testing the refusals

The three repositories are already deprecated with the inputs above. Requesting them again with the
same inputs previews `"op": "unchanged"`, and approval succeeds without pushing (it still verifies
`main`). To exercise a real push, use a different reason, which previews `"op": "modify"`.

```sh
cd /tmp/actions-ab/r2/vanilla-3/org

# Invalid inputs: exit 2, every error listed, no run created
bin/acme preview alpha mark-deprecated -i successor=Acme/Proxy -i reason=short
bin/acme request alpha mark-deprecated --as ann -i successor=acme/alpha -i "reason=Points at itself, which is refused."
bin/acme request alpha mark-deprecated --as ann -i successor=acme/x

# Self-approval and carol's approval: exit 3, recorded as approve_denied
RUN=$(bin/acme request alpha mark-deprecated --as ann -i successor=acme/proxy-v3 \
  -i "reason=Testing the approval rules for this action." | python3 -c 'import json,sys; print(json.load(sys.stdin)["run"]["id"])')
bin/acme approve "$RUN" --as ann
bin/acme approve "$RUN" --as carol
bin/acme show "$RUN"

# Rejected push: the run is reported failed (exit 1, outcome.error = push_rejected) and main is unchanged
printf '#!/bin/sh\necho "main is protected" >&2\nexit 1\n' > ../origins/alpha.git/hooks/pre-receive
chmod +x ../origins/alpha.git/hooks/pre-receive
bin/acme approve "$RUN" --as bob
git --git-dir ../origins/alpha.git show main:DEPRECATED.md
rm ../origins/alpha.git/hooks/pre-receive
```

Set `ACME_STATE`, `ACME_CONFIG` or `ACME_PEOPLE` to point the runner at another state directory,
repository list or people list, for example a scratch copy of the origins.

## Limits

- Identity is asserted, as the sandbox specifies: `--as` is trusted, and only names in `org/people.json` are accepted.
- Run records live on the org machine in `org/state/`, not in git. They are written atomically under a file lock, which also serialises approvals and executions across processes.
