# Handoff: `mark-deprecated`, a self-service action owned by each repository

**Status:** built and used. `mark-deprecated` ran on `alpha`, `beta` and `gamma`. ann requested each run and bob approved it. Each run was reported `succeeded` only after `DEPRECATED.md` with the expected bytes was seen on its origin's `main`:

| Repository | Run | Commit on origin `main` |
|---|---|---|
| alpha | `run-20261124T130613Z-358c21` | `b685732` |
| beta | `run-20261124T130613Z-f24334` | `adf7c4a` |
| gamma | `run-20261124T130613Z-a96524` | `f70bc6a` |

**Design:** there is no platform and no server. `org/acme` is one Python 3.12 script that uses only the standard library. It reads each action's definition straight from the repository's origin `main`, keeps run records under `org/state/`, and does the git work itself. **Nothing has to be running.**

## Files

| Path | What it is |
|---|---|
| `<repo>/.acme/actions/mark-deprecated.json` (in `alpha`, `beta` and `gamma`, committed to each origin's `main`) | The action definition: input schema, approval rule, file template, commit message and branch |
| `<repo>/.acme/maintainers.json` (same three repositories) | The repository's maintainers (`ann`, `bob`), the only people who can approve |
| `org/acme` | The CLI: `list`, `describe`, `preview`, `request`, `approve`, `reject`, `status`, `show`, `runs`, `audit` |
| `org/acme.json` | The org registry: org name (`acme`), known people (`ann`, `bob`, `carol`), each repository's origin, and the state directory |
| `org/state/runs/<run-id>.json` | One record per run (git-ignored, because it is runtime state) |
| `org/state/audit.jsonl` | An append-only org audit log. It also records refused requests and refused approvals |
| `org/tests/test_acme.py` | Scenario tests that run against throwaway copies of the origins |

## How each requirement is met

| # | How | Where |
|---|---|---|
| R1 Owned by the repository | The definition and the maintainer list are committed to each repository's origin `main`. `acme` never keeps its own copy: every `list`, `preview` and `request` reads them from the origin's current `main` with `git cat-file`. A request pins the definition's commit and sha256 into the run record. If a team changes its file and pushes, the change applies to the next request. `test_repository_owns_its_definition` covers this. | `alpha/.acme/actions/mark-deprecated.json`, `alpha/.acme/maintainers.json` (and the same in `beta` and `gamma`); `load_repo()` in `org/acme` |
| R2 Validated | The inputs are checked against the definition's schema before any run is created or any git command writes. `successor` must fully match `^[a-z0-9-]+/[a-z0-9-]+$` (so a trailing newline fails) and must not be the repository itself (`notSelf`). `reason` must be 20 to 200 characters on one line (`singleLine`). Unknown or missing inputs are refused. A refusal exits with code 2 and returns the per-input errors, no run record is created, and the refusal goes to `state/audit.jsonl` as `request_refused`. The inputs are checked again just before execution. | `validate_inputs()` and `require_valid()` in `org/acme` |
| R3 Previewed | `acme preview` validates the inputs and returns the exact new file content, a unified diff against the origin's current `main`, the base commit, the commit message and the eligible approvers. It changes nothing. `request` stores the same preview in the run, so the approver sees it too (`acme show`). At execution, if the target file on `main` differs from what the preview saw, the run fails with `stale_preview` and pushes nothing, so what takes effect is always what was previewed. | `compute_preview()`, plus the staleness check in `execute()` |
| R4 Approved | `request` creates a run in the `pending_approval` state and changes nothing in the repository. `approve` refuses a requester who tries to approve their own run (`self_approval`), anyone outside the pinned definition's maintainer list (`not_maintainer`, which is how carol is refused) and unknown names, all with exit code 3. Refused approvals are written to the run's `refusals` and `events`. One approval from another maintainer (`approval.required: 1`) makes the run execute. Org policy rejects any definition that sets `allowSelfApproval: true`. | `cmd_approve()` and `refuse()`; `check_definition()` |
| R5 Done means observed | After approval, `acme` makes a fresh clone of the origin's `main` and commits the file. The commit's author is the requester, its committer is `acme-actions`, and trailers record the run id, requester and approvers. `acme` then pushes without force. If the push fails, the run is `failed` with `reason: push_rejected` and the remote's output. If the push succeeds, `acme` runs `ls-remote` and `fetch` against the origin and requires two things: the origin's `main` contains the run's commit, and `DEPRECATED.md` on `main` has the expected sha256. It checks up to 5 times, one second apart. Only then is the run `succeeded`. Otherwise it is `failed` with `reason: not_observed`. A run whose content already matches `main` still makes an (empty) commit, so its success is always this run's push being observed. | `execute()` and `observe()` in `org/acme`; tests `test_rejected_push_is_not_success` and `test_accepted_push_that_does_not_land_is_not_success` |
| R6 One place for the org | `org/acme` and the registry `org/acme.json` live in the org repository (pushed to `origins/org.git`). From `org/`, `./acme list` shows `mark-deprecated` for all three repositories, and `./acme request <repo> …` works on any of them without entering the repository. | `org/acme`, `org/acme.json` |
| R7 Audited | Each run record holds `requestedBy` and `requestedAt`, `inputs`, `approvals[]` (`by`, `at`), `refusals[]` (`by`, `at`, `reason`), and `outcome` (`status`, `at`, `message`, `reason`). It also holds `execution` (base commit, run commit, push exit code and output, observed `main` commit and hashes, start and finish times), the pinned definition, the preview, and a timestamped `events[]` log with the actor of each step. `state/audit.jsonl` mirrors every event across runs. | `org/state/runs/<id>.json`, `org/state/audit.jsonl`; `acme show`, `acme audit` |
| R8 Usable by agents | Every command is non-interactive. Identity comes from `--as NAME` (or `ACME_ACTOR`), inputs from flags, JSON or stdin, and every command prints one JSON document. Exit codes: `0` ok, `1` run failed, `2` invalid input, `3` forbidden, `4` not found, `5` wrong state (for example approving twice), `6` broken definition or internal error. | `org/acme` (module docstring, `emit()`) |

## Commands

Nothing needs starting: there is no server or daemon. Run everything from the org directory:

```sh
cd /tmp/actions-ab/r5/vanilla-2/org
```

**List the actions** (all repositories, or one with `--repo alpha`), and show one definition:

```sh
./acme list
./acme describe alpha mark-deprecated
```

**Preview** (changes nothing):

```sh
./acme preview alpha mark-deprecated --as ann \
  --input successor=acme/dependabot-proxy-v2 \
  --input "reason=Replaced by the v2 proxy, which supports OIDC registries."
```

**Request as ann.** The output includes the preview and `run.id`:

```sh
RUN=$(./acme request alpha mark-deprecated --as ann \
  --input successor=acme/dependabot-proxy-v2 \
  --input "reason=Replaced by the v2 proxy, which supports OIDC registries." \
  | python3 -c 'import json,sys; print(json.load(sys.stdin)["run"]["id"])')
echo "$RUN"
```

You can also pass the inputs as JSON: `--inputs-json '{"successor":"acme/x","reason":"..."}'`, or `--inputs-file -` to read them from stdin.

**Approve as bob.** This executes the run and prints the final status. It exits 0 if the run succeeded and 1 if it failed:

```sh
./acme approve "$RUN" --as bob
```

**Check the status and read the record:**

```sh
./acme status "$RUN"            # id, status, requester, approvals, outcome
./acme show "$RUN"              # the full record: inputs, preview, events, execution, observation
./acme runs --repo alpha        # all runs, filterable by --repo and --status
./acme audit --run "$RUN"       # audit events (omit --run for the whole org log, including refused requests)
```

**Reject or cancel** a pending run. Another maintainer rejects it; the requester cancels it:

```sh
./acme reject "$RUN" --as bob --comment "not yet"
```

## Testing the guardrails

All three repositories are now deprecated, so a new successful run *modifies* `DEPRECATED.md` on the real origins. To leave them alone, either run the scenario suite, which works on throwaway copies:

```sh
cd /tmp/actions-ab/r5/vanilla-2/org && python3 -m unittest discover -s tests -v
```

or point `acme` at copies with `ACME_CONFIG=/path/to/copy/acme.json`, a config shaped like `org/acme.json`.

Manual checks on the real system, each with its expected result:

```sh
cd /tmp/actions-ab/r5/vanilla-2/org

# Invalid inputs: exit 2, error.code "invalid_input" with per-input details, no run created
./acme request alpha mark-deprecated --as ann --input successor=Acme/X --input "reason=long enough reason text"
./acme request alpha mark-deprecated --as ann --input successor=acme/x --input "reason=too short"
./acme request alpha mark-deprecated --as ann --input successor=acme/x --input "reason=$(printf 'x%.0s' {1..201})"
./acme request alpha mark-deprecated --as ann --input successor=acme/alpha --input "reason=long enough reason text"

# A valid pending run to test approvals on
RUN=$(./acme request alpha mark-deprecated --as ann --input successor=acme/test-target \
  --input "reason=Testing the approval guardrails here." | python3 -c 'import json,sys; print(json.load(sys.stdin)["run"]["id"])')
./acme approve "$RUN" --as ann     # exit 3, self_approval refused
./acme approve "$RUN" --as carol   # exit 3, not_maintainer refused
./acme status "$RUN"               # still pending_approval; refusals are listed by: ./acme show "$RUN"

# Rejected push: the code host refuses every update to main
printf '#!/bin/sh\necho "main is protected" >&2\nexit 1\n' > ../origins/alpha.git/hooks/pre-receive
chmod +x ../origins/alpha.git/hooks/pre-receive
./acme approve "$RUN" --as bob     # exit 1, status "failed", outcome.reason "push_rejected", pushOutput holds the hook's message
git --git-dir ../origins/alpha.git show main:DEPRECATED.md   # unchanged
rm ../origins/alpha.git/hooks/pre-receive
```

A failed run is final: once the cause is fixed, request a new run.

## Limits

- **Identity is self-declared.** The sandbox has no passwords, so `--as` is trusted. The system enforces *who may do what* for a declared identity and records it, but it cannot stop someone from claiming to be bob, editing `org/state/` by hand, or pushing to the bare origins directly. A real deployment would put `acme` behind authentication and protect `main` so that only `acme-actions` can push.
- **Execution is synchronous.** The approving command performs the run. If that process is killed mid-run, the record stays `running`. Inspect the origin and request again.
- **One approval.** The approval count is read from each definition (`approval.required`). Every approver must be a maintainer other than the requester, and nobody can approve twice.
