# HANDOFF: `mark-deprecated` self-service action

`mark-deprecated` is built and has run on all three repositories. Each run was requested by ann, approved by bob, and reported `succeeded` only after `DEPRECATED.md` with the expected content was observed on the origin's `main`:

| Repository | Run id | Status |
|---|---|---|
| alpha | `run-20261124T121001Z-10682b` | succeeded |
| beta | `run-20261124T121001Z-ca9510` | succeeded |
| gamma | `run-20261124T121001Z-215729` | succeeded |

**Nothing needs to be running.** The system is one dependency-free Python 3 CLI, `/tmp/actions-ab/r3/vanilla-2/org/acme-actions`. It reads definitions straight from the bare origins and keeps run records on disk under a lock. No server or tmux session is used.

## Files

| Path | What it is |
|---|---|
| `alpha/.acme/actions/mark-deprecated.json` (and the same path in `beta` and `gamma`), committed to each `origins/<repo>.git` `main` | The action's definition, owned by that repository: maintainers, input rules, the file it writes and its template |
| `org/acme-actions` | The CLI: `list`, `preview`, `request`, `approve`, `reject`, `status`, `record`, `runs` |
| `org/repos.json` | The org registry: repositories, their origins and the people (ann, bob, carol) |
| `org/state/runs/<run-id>.json` | One record per run: inputs, the pinned definition, the preview, approvals, the push result, the observation and a timestamped event list. Not committed (`org/.gitignore`) |
| `org/state/audit.jsonl` | An append-only log of every event of every run, including refused approvals |

All paths are relative to `/tmp/actions-ab/r3/vanilla-2`. `org/acme-actions`, `org/repos.json`, `org/.gitignore` and this file are committed to `origins/org.git` `main`.

## How each requirement is met

| # | How |
|---|---|
| R1 Owned by the repository | The definition is `.acme/actions/mark-deprecated.json` in each repository, committed to its origin's `main`. The CLI holds no action logic of its own: it reads `.acme/actions/*.json` from `refs/heads/main` of `origins/<repo>.git` on every command, so a commit by the repository's team changes its action. That includes its `maintainers` list, its input rules and its template. A request pins the definition's commit and blob (`definition` in the run record), so the approved run executes exactly what was previewed. |
| R2 Validated | `validate()` in `org/acme-actions` applies the definition's rules: `successor` must fully match `^[a-z0-9-]+/[a-z0-9-]+$` and must not be the repository itself (`acme/<repo>`); `reason` must be 20 to 200 characters and a single line without control characters; unknown or missing inputs are refused. `preview` and `request` both validate before doing anything else. On failure they exit with code 2 and print `{"error":"invalid_inputs","errors":[...]}`. No run is recorded and nothing is pushed. |
| R3 Previewed | `preview` prints the target branch, the base commit, `change` (`create`, `update` or `none`), the file's content before and after, and a unified diff, all against the origin's current `main`. `request` stores the same preview in the run record (`preview`), so the approver sees it as well. |
| R4 Approved | `request` only records a run as `awaiting_approval`. Only `approve` makes it take effect, and `decide()` refuses (exit code 3) when the approver is the requester (`self_approval`) or is not in the pinned definition's `maintainers` (`not_a_maintainer`, which is what happens to carol). Unknown people are refused too (`unknown_person`). A run that is not `awaiting_approval` cannot be approved again. Refused attempts are written to the run's events and to `audit.jsonl`. |
| R5 Done means observed | After approval, `execute()` clones the origin into a temporary directory, writes the file, commits and pushes to `main`. It then reads `main:DEPRECATED.md` back from the bare origin itself. The run is `succeeded` only if that blob equals the rendered content byte for byte. Otherwise it is `failed`, with a cause such as `push rejected` or `push accepted but content differs`, and `approve` exits with code 5. `observation` records the commit seen on `main`. The push's exit code and output are kept in `push`. |
| R6 One place for the org | Everything runs from `org/`: `org/acme-actions list` lists the actions of every repository in `org/repos.json`, and `request --repo <name>` works on any of them. The CLI never touches the checkouts in `alpha/`, `beta/` or `gamma/`. |
| R7 Audited | `org/state/runs/<id>.json` holds `requestedBy` and `requestedAt`, `approvedBy` and `approvedAt` (or `rejectedBy`), `inputs`, `status`, `outcome` and `finishedAt`. `events` adds a timestamped entry for each step: `requested`, `previewed`, `approved`, `started`, `planned`, `push_accepted` or `push_rejected`, `observed` and `succeeded` or `failed`. Each event is also appended to `org/state/audit.jsonl`. The pushed commit is authored by the requester, committed by `acme-actions`, and carries `Run:`, `Requested-by:` and `Approved-by:` trailers. |
| R8 Usable by agents | Every command is non-interactive, takes identity from `--as <person>` or `ACME_USER`, and prints JSON on stdout. Errors are JSON too (`error`, `message`, details). Exit codes: 0 ok, 1 error, 2 invalid input, 3 not permitted, 4 not found, 5 run did not succeed. `org/acme-actions --help` and `<command> --help` describe the options. |

## Commands

```bash
cd /tmp/actions-ab/r3/vanilla-2/org

# Start: nothing to start.

# List the actions of all repositories (optionally --repo alpha)
./acme-actions list

# Preview: validate inputs and show what a run would change
./acme-actions preview --repo alpha --as ann \
  --input successor=acme/dependabot-proxy-v2 \
  --input "reason=Replaced by the v2 proxy, which supports OIDC registries."

# Request a run as ann; prints the run record, including "id"
./acme-actions request --repo alpha --as ann \
  --input successor=acme/dependabot-proxy-v2 \
  --input "reason=Replaced by the v2 proxy, which supports OIDC registries."
# Inputs can also be given as JSON: --inputs-json '{"successor":"acme/x","reason":"..."}'
# For scripting: RUN=$(./acme-actions request ... | python3 -c 'import json,sys;print(json.load(sys.stdin)["id"])')

# Approve as bob; this applies the change and prints the final record (exit 0 only if succeeded)
./acme-actions approve "$RUN" --as bob
# Or turn it down: ./acme-actions reject "$RUN" --as bob --note "why"

# Check a run's status
./acme-actions status "$RUN"

# Read a run's full record, including the event log
./acme-actions record "$RUN"

# List runs (filters: --repo, --status)
./acme-actions runs
cat state/audit.jsonl
```

## Testing the refusals

- **Invalid inputs:** for example `--input successor=Acme/X`, `--input successor=acme/alpha` on alpha, a reason shorter than 20 or longer than 200 characters, a missing input, or an unknown `--input foo=1`. `preview` and `request` exit 2, and no run appears in `./acme-actions runs`.
- **Self-approval:** `./acme-actions approve "$RUN" --as ann` on a run ann requested exits 3 with `self_approval`. The run stays `awaiting_approval`.
- **carol:** `./acme-actions approve "$RUN" --as carol` exits 3 with `not_a_maintainer`.
- **Rejected push:** install a hook that refuses pushes, for example:

  ```bash
  printf '#!/bin/sh\necho "main is protected" >&2\nexit 1\n' > ../origins/beta.git/hooks/pre-receive
  chmod +x ../origins/beta.git/hooks/pre-receive
  ```

  Then request and approve a run whose content differs from what is already on `main`. **The three repositories already carry the expected `DEPRECATED.md`, so a repeat of the same inputs needs no push.** It is correctly reported `succeeded` from the observation alone (`change: none`, event `no_change_needed`). Use a different successor or reason to force a push. `approve` then exits 5, and the run is `failed` with outcome `... (push rejected)` and `push.output` showing git's refusal. Remove the hook afterwards with `rm ../origins/beta.git/hooks/pre-receive`.

## Notes

- The file content is `# Deprecated: use <successor>`, an empty line, then `<reason>`, ending with a single newline.
- To try the system without touching the real origins, copy an origin and point the CLI at a different registry and state: `ACME_ACTIONS_REGISTRY=/path/repos.json ACME_ACTIONS_STATE=/path/state ./acme-actions ...`. Origin paths in a registry are relative to that registry file.
- A failed run is final. To retry, request a new run.
- There are no passwords in this sandbox, so `--as` is trusted. The approval rules are enforced on that stated identity.
