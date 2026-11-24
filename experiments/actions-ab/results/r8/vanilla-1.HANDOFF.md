# HANDOFF: the `add-notice` self-service action

`add-notice` is built and has run on all three repositories. The runs are `run-0001` (alpha, approved by
dave), `run-0002` (beta, approved by erin) and `run-0003` (gamma, approved by erin). All three
were requested by ann and all three are `succeeded`.

The system is one command-line tool, `org/bin/acme`. It is written in Python 3.12 using only the
standard library. **Nothing needs to be running:** there is no server and no daemon. Each command
reads the origins' `main` through git, does its work and exits. Every command prints a single JSON
document on stdout, either `{"ok": true, ...}` or `{"ok": false, "error": {"kind", "code", "message", "details"}}`.

| Exit code | Meaning |
|---|---|
| 0 | ok |
| 1 | the run was approved and carried out, but failed (for example, the push was rejected) |
| 2 | invalid inputs or usage; nothing changed |
| 3 | forbidden (self-approval, not in the release group, not in the org) |
| 4 | not found (repository, action, run) |
| 5 | conflict (the run is not in a state that allows this, or there is nothing to do) |
| 6 | system error (git failed, the org config is missing) |

## Layout

| Path | What it is |
|---|---|
| `<repo>/.acme/actions/add-notice/action.json` | The action's definition in each of `alpha`, `beta` and `gamma`: inputs schema, owners (ann, bob), approval group, the script to run and the files it may write |
| `<repo>/.acme/actions/add-notice/run.py` | The repository's own implementation: it puts the notice at the top of the root README, finding it in any letter case |
| `<repo>/.acme/actions/add-notice/test_run.py` | Its unit tests: `python3 -m unittest discover -s .acme/actions/add-notice` |
| `org/repos.json` | The repositories in the org and their origins |
| `org/groups.json` | **The only definition of the groups:** `org` (may request) and `release` (may approve) |
| `org/policy.json` | Org policy: `requesterGroup: org`, `approvalGroup: release` |
| `org/bin/acme`, `org/acme/*.py` | The engine (`engine.py`), the run store and audit log (`store.py`), git access (`gitops.py`) and the command-line interface (`cli.py`) |
| `org/templates/add-notice/` | The template the three repositories' copies were made from |
| `org/tests/test_acme.py` | End-to-end tests against throwaway origins: `cd org && python3 -m unittest discover -s tests` |
| `org/state/` (git-ignored) | Run records `state/runs/run-NNNN.json`, the hash-chained audit log `state/audit.jsonl`, git caches and temporary clones |

Repository definitions and org configuration are always read from the **origins' `main`**, never
from a checkout's working tree. An edit counts only once it is committed and pushed.

## How each requirement is met

| # | How | Where |
|---|---|---|
| R1 Owned by the repository | Each repository holds its own definition and script, committed to its origin's `main` (alpha `1d44c76`, beta `1a66b00`, gamma `8b0b02b`). The engine reads them from that `main` at request time, and again at execution time. The repository's team can change the inputs, the description, the script or the files it may write. Org policy fixes only one thing: the approval group. A definition that names another group, or allows self-approval, is refused (`bad_definition`) | `alpha/.acme/actions/add-notice/` (and the same in `beta`, `gamma`); `org/acme/engine.py` `check_definition` and `prepare` |
| R2 Validated | The engine checks the inputs against the repository's `action.json`. `level` must be one of `info` or `warning`. `message` must be 10 to 160 characters on one line: no newline or control characters, and not blank. Unknown or missing inputs are refused too. A refused request creates no run and makes no change, but it is logged in the audit log as `request_refused`. `run.py` checks the inputs again on its own | `engine.py` `validate_inputs`; `.acme/actions/add-notice/action.json` `inputs`; `run.py` `check_inputs` |
| R3 Previewed | `acme preview` runs the action in a throwaway clone of `main` and returns a unified diff and content hashes. It records and changes nothing. `acme request` stores the same preview in the run, and anyone can read it with `acme record`. On approval, the engine applies the change to the current `main` and requires the result to equal the previewed bytes. If the README or the definition changed after the request, the run fails as `stale_preview` or `definition_changed` instead of doing something nobody saw | `engine.py` `preview`, `prepare`, `_execute` |
| R4 Approved by the release group | `acme approve` reads `release` from the org origin's `main` at that moment. It refuses the requester (`self_approval`) and anyone not in the group (`not_an_approver`), with exit code 3. Each refused attempt is recorded in the run as `approval_refused`. A run changes `main` only from inside `approve`, after these checks. Names are compared lower-cased and trimmed, so `DAVE` is dave | `engine.py` `approve`; `org/groups.json`; `org/policy.json` |
| R5 Done means observed | After pushing, the engine fetches the origin's `main` again. It reports `succeeded` only if `main` contains the run's commit **and** the README there has the exact bytes (sha256) that were previewed and approved. A rejected push becomes `failed` with reason `push_rejected` and the origin's output, and is never retried silently. Any other mismatch becomes `failed` / `not_observed`. Unexpected errors become `failed` / `internal_error`. If the process dies mid-run, the run stays `executing` until `acme verify` observes `main` and finishes it either way | `engine.py` `_execute`, `_observe`, `verify` |
| R6 One place for the org | `org/bin/acme` and `org/repos.json`: `list` reads every repository's `.acme/actions/*/action.json` from its origin's `main`, and `request <repo> <action>` works on any of them, all from `org/` | `org/bin/acme`, `org/repos.json`, `engine.py` `definitions` |
| R7 Audited | Each run record has `requested_by`/`requested_at`, `approved_by`/`approved_at`, `inputs`, `outcome`/`outcome_at`, and an `events` list. Every event carries a UTC millisecond timestamp and, where relevant, an actor: `requested`, `approval_refused`, `approved` (with a snapshot of the release group and the org commit it came from), `execution_started`, `commit_created`, `pushed` or `push_rejected`, `observed` (with the `main` commit and hashes), then `succeeded` or `failed`. The same events, plus refused requests and group changes, go to the append-only, sha256-chained `state/audit.jsonl`; check it with `acme audit --verify`. The commit on `main` also carries `Acme-Run:`, `Requested-By:` and `Approved-By:` trailers | `org/state/runs/run-NNNN.json`, `org/state/audit.jsonl`, `store.py` |
| R8 Usable by agents | Every step is a non-interactive subcommand that prints JSON and uses the exit codes above. Identity is given with `--as <name>` or `ACME_USER`. Inputs are given with `-i key=value` (repeatable) or `--inputs-json '{...}'`. Usage errors are JSON too | `org/acme/cli.py` |
| R9 The group is defined once | `org/groups.json` on the org origin's `main` is the only list of the release group, and no repository lists approvers. `approve` re-reads it each time, so a change applies to all three repositories at once. `acme group show` warns if your checkout's copy differs from `main` | `org/groups.json`, `engine.py` `config`, `change_group`, `local_config_drift` |

## Commands

All commands run from the org checkout. Nothing has to be started first.

```bash
cd /tmp/actions-ab/r8/vanilla-1/org
A=bin/acme   # or: export PATH=/tmp/actions-ab/r8/vanilla-1/org/bin:$PATH and use `acme`
```

**List the actions** (all repositories, or one):

```bash
$A list
$A list --repo beta
$A describe gamma add-notice
```

**Preview** (records and changes nothing):

```bash
$A preview alpha add-notice -i level=warning -i "message=This proxy is frozen while the v2 rewrite lands; please file new issues against v2."
```

**Request a run, as ann.** The output includes the run id (`run-NNNN`) and the preview. The run then waits for approval:

```bash
$A request beta add-notice --as ann -i level=info -i "message=Releases now ship on the shared release train every second Tuesday."
$A request gamma add-notice --as ann --inputs-json '{"level":"info","message":"Graph extraction now lives in graphify-core; this package re-exports it."}'
```

**Approve, as dave or erin.** Approval executes the run, pushes, observes `main` and prints the final status. The exit code is 0 if the run succeeded and 1 if it failed:

```bash
$A approve run-0004 --as dave
$A approve run-0004 --as erin --comment "ok for readers"
```

Reject or cancel instead:

```bash
$A reject run-0004 --as erin --reason "wrong wording"   # release group only
$A cancel run-0004 --as ann                             # requester only
```

**Check a run's status, and read its record:**

```bash
$A status run-0001
$A record run-0001                  # inputs, preview diff, every event with time and actor
$A record run-0001 --with-content   # also the approved README bytes (base64)
$A runs                             # all runs; filter with --repo alpha or --status awaiting_approval
$A verify run-0001                  # re-observe main now (and finish an interrupted run)
$A audit --run run-0001             # this run's entries in the org audit log
$A audit --verify                   # check the audit hash chain
```

**Change who is in the release group.** Either of these works, and both apply to all three repositories from the next approval on:

```bash
$A group show release
$A group add release carol --as ann      # commits and pushes org/groups.json to the org origin's main
$A group remove release carol --as ann
git pull -q --rebase origin main         # bring your checkout up to date afterwards
```

Or edit `org/groups.json` by hand, then commit and push it. Edits that are not pushed do not count:

```bash
git pull -q --rebase origin main
python3 - <<'EOF'
import json; p = "groups.json"; d = json.load(open(p))
d["groups"]["release"]["members"] = ["dave"]        # e.g. remove erin
open(p, "w").write(json.dumps(d, indent=2) + "\n")
EOF
git commit -qam "groups: release is dave only" && git push -q origin main
```

## Testing the edge cases

The commands below assume a fresh run id; use the one that `request` prints.

```bash
# Invalid inputs: each returns exit 2 with per-input errors, and no run is created
$A request alpha add-notice --as ann -i level=error -i "message=Valid length message."
$A request alpha add-notice --as ann -i level=info  -i "message=too short"           # 9 characters
$A request alpha add-notice --as ann -i level=info  -i "message=$(printf 'x%.0s' {1..161})"
$A request alpha add-notice --as ann --inputs-json '{"level":"info","message":"two\nlines here"}'
$A request alpha add-notice --as ann -i level=info -i "message=Valid length message." -i extra=1

# Self-approval and outsiders: each returns exit 3, and the run stays awaiting_approval
id=$($A request alpha add-notice --as dave -i level=info -i "message=Self approval test message." | python3 -c 'import json,sys;print(json.load(sys.stdin)["run"]["id"])')
$A approve $id --as dave    # self_approval
$A approve $id --as carol   # not_an_approver
$A approve $id --as ann     # not_an_approver (maintainer, but not in release)
$A approve $id --as erin    # succeeds

# A push that is rejected before the change reaches main
printf '#!/bin/sh\necho "main is protected" >&2\nexit 1\n' > ../origins/beta.git/hooks/pre-receive
chmod +x ../origins/beta.git/hooks/pre-receive
id=$($A request beta add-notice --as ann -i level=warning -i "message=This push will be rejected by the origin." | python3 -c 'import json,sys;print(json.load(sys.stdin)["run"]["id"])')
$A approve $id --as dave    # exit 1: status failed, outcome.reason push_rejected, push_output contains the hook's message
$A status $id               # failed, never succeeded
git --git-dir ../origins/beta.git show main:readme.md | head -1   # unchanged
rm ../origins/beta.git/hooks/pre-receive
```

A failed run is final. To try again, request a new run.

## Notes and limits

- **Identity** is asserted with `--as`, as the sandbox allows. The system records it on every event
  and enforces the rules on it, but it does not authenticate anyone.
- **Who may change groups** is not restricted beyond the org origin's own write access. Every
  change is a commit on the org origin's `main`, authored by the `--as` person, and is logged as
  `group_changed` in the audit log.
- **Bypassing the system:** anyone who pushes to an origin directly bypasses it. In a real deployment,
  the code host's branch protection would allow only the actions service to push to the README.
  `run.py` changes the README only; the engine refuses a script that touches any file `writes` does
  not list.
- **Concurrency:** runs are locked one at a time per run, and record writes are atomic. If an
  approver's process is killed during execution, `acme verify <run>` finishes the run from what it
  observes on `main`.
- **Stopping:** no long-running processes or tmux sessions were started.
