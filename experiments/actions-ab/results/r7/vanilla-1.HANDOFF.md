# HANDOFF: `mark-deprecated` self-service action

**Status:** built and used. `mark-deprecated` is defined in `alpha`, `beta` and `gamma` and runs from
`org/` through one CLI, `org/bin/acme`. All three runs were requested by ann, approved by bob, and
reported `succeeded` only after `DEPRECATED.md` was read back from each origin's `main`:

| Run | Repository | successor | Requested by | Approved by | Status | Commit on origin `main` |
|---|---|---|---|---|---|---|
| `run-0001` | alpha | `acme/dependabot-proxy-v2` | ann | bob | succeeded | `1afcf73` |
| `run-0002` | beta | `acme/valtio-store` | ann | bob | succeeded | `5b85e48` |
| `run-0003` | gamma | `acme/graphify-core` | ann | bob | succeeded | `07bc5af` |

**Nothing has to be running.** No server or daemon is involved: each command reads the origins,
does its work, prints JSON and exits. You need `python3` (3.12, standard library only) and `git`.

## Layout

| Path | What it is |
|---|---|
| `<repo>/.acme/actions/mark-deprecated.json` (on each origin's `main`) | The repository's own definition of the action: approvers, input rules, and the effect (file, branch, template, commit message) |
| `org/bin/acme` | The CLI (a wrapper around `org/acme/acme.py`) |
| `org/acme/acme.py` | All of the logic: discovery, validation, preview, the run store, approval, execution and observation |
| `org/acme.json` | Org config: the people (ann, bob, carol) and where each repository's origin is |
| `org/state/runs/run-NNNN.json` | One record per run, with every event (not committed: `org/.gitignore`) |
| `org/state/audit.jsonl` | An append-only log of every event across runs, including refused approvals |
| `org/tests/test_acme.py`, `org/tests/fixtures/mark-deprecated.json` | End-to-end tests against throwaway origins under `org/.test-tmp/` |

## How each requirement is met

| # | How | Where |
|---|---|---|
| R1 Owned by the repository | Each repository's definition is committed to its own origin's `main` at `.acme/actions/mark-deprecated.json` (alpha `4db6e85`, beta `008db78`, gamma `6242846`). `acme` reads definitions **only** from `refs/heads/main` of the origin (`git ls-tree`/`cat-file` on the bare repo), never from the org or a checkout. A team changes its action, including who may approve (`approvers`), by changing that file on `main`. Approval checks the approvers on the **current** `main`, so removing a maintainer takes effect immediately. The effect a run executes is pinned to the definition the run was requested and previewed against (its commit and blob are in the record under `definition`). | `origins/<repo>.git` → `main:.acme/actions/mark-deprecated.json`; `definitions()`, `get_definition()` in `org/acme/acme.py` |
| R2 Validated | Inputs are checked against the definition's rules before any preview, record or git write. `successor` must fully match `^[a-z0-9-]+/[a-z0-9-]+$` (so a trailing newline fails) and must not be the repository itself (`notEqual: "{org}/{repo}"`). `reason` must be 20 to 200 characters with no control characters or newlines. Unknown inputs, missing inputs and non-string values are refused. A refusal exits with code 2, prints `error.code = "invalid_input"` with one entry per problem, and creates no run. | `validate_inputs()`, `build_preview()` in `org/acme/acme.py`; the `inputs` block of each definition |
| R3 Previewed | `acme preview` shows the base commit, the change (`create`, `modify` or `none`), the file before and after, a unified diff and the commit message. It writes nothing. `acme request` stores the same preview in the run, so the approver sees exactly what will happen (`acme record`). At execution, if the file on `main` no longer matches what the preview was based on, the run fails at stage `plan` and does not overwrite it. | `build_preview()`, `cmd_preview()`, the `plan` check in `execute()` |
| R4 Approved | A run starts as `awaiting_approval` and changes nothing. `acme approve` executes it only if the approver is a known person, is in the definition's `approvers` on the current `main` (ann and bob), and is not the requester. Self-approval and carol's approval are refused with exit code 3 (`forbidden`). The run stays `awaiting_approval`, and the refusal is recorded as an `approval_refused` event. The pushed commit is authored by the requester and committed by the approver, with `Acme-Run`, `Requested-by` and `Approved-by` trailers. Maintainers can `reject`, and the requester can cancel with `reject`. | `cmd_approve()`, `cmd_reject()`, `refuse()` |
| R5 Done means observed | On approval the run is `running`. `acme` clones the origin, commits the previewed file on top of `main`, and pushes `<commit>:refs/heads/main`. It then reads `main:DEPRECATED.md` from the **origin** and compares it byte for byte with the preview. Only a match gives `succeeded`. A rejected push gives `failed` (stage `push`, with the origin's stderr). If the push is accepted but the content is not on `main`, the run also fails (stage `observe`). So does a `main` that moved since the preview (stage `plan`). `approve` exits with 6 when the run failed. The observation (commit, blob, `matches`) is stored in the record, and `acme status <run> --observe` re-reads the origin on demand. | `execute()`, `observe()` |
| R6 One place for the org | From `org/`, `acme actions` lists every action in every repository in `org/acme.json`. It shows each one's approvers, input schema and source commit. `preview`, `request`, `approve`, `status`, `record` and `runs` take a repository name, so nobody has to enter a repository. | `org/bin/acme`, `org/acme.json`, `cmd_actions()` |
| R7 Audited | Each run record keeps `requestedBy`/`requestedAt`, `inputs`, `approval.by`/`approval.at` and `outcome` (status, `at`, pushed commit, observed commit/blob, error). It also keeps `events`, a timestamped list (`requested`, `approval_refused`, `approved`, `started`, `succeeded`/`failed`, `rejected`/`cancelled`) with who did each. Every event is also appended to `org/state/audit.jsonl`. Times are UTC ISO 8601 with milliseconds. | `org/state/runs/run-NNNN.json`, `org/state/audit.jsonl`; `add_event()` |
| R8 Usable by agents | Every command is non-interactive, takes identity from `--as <person>` (or `$ACME_USER`) and prints one JSON document on stdout (`"ok": true/false`; errors as `error.code`, `error.message` and details). Exit codes: 0 ok, 2 invalid input or usage, 3 forbidden, 4 not found, 5 wrong state (for example, approving a finished run), 6 run failed, 7 invalid definition. Inputs can be given as `-i key=value` or `--inputs-json '{...}'`. A file lock (`org/state/lock`) serialises state changes. | `main()` in `org/acme/acme.py` |

## Commands

Run everything from the org directory:

```sh
cd /tmp/actions-ab/r7/vanilla-1/org
export PATH="$PWD/bin:$PATH"       # or call ./bin/acme
```

**Start:** nothing to start.

**List the actions** (all repositories, or one):

```sh
acme actions
acme actions --repo beta
acme describe gamma mark-deprecated
```

**Preview** (changes nothing):

```sh
acme preview alpha mark-deprecated --as ann \
  -i successor=acme/dependabot-proxy-v2 \
  -i "reason=Replaced by the v2 proxy, which supports OIDC registries."
```

**Request as ann** (prints the run id in `.run.id` and the preview):

```sh
acme request alpha mark-deprecated --as ann \
  --inputs-json '{"successor": "acme/dependabot-proxy-v2", "reason": "Replaced by the v2 proxy, which supports OIDC registries."}'
RUN=$(acme request beta mark-deprecated --as ann -i successor=acme/valtio-store \
  -i "reason=State management moves to the shared store package." | python3 -c 'import json,sys; print(json.load(sys.stdin)["run"]["id"])')
```

**Approve as bob** (executes, then observes the origin; exit 0 means succeeded, 6 means failed):

```sh
acme approve "$RUN" --as bob
```

**Reject or cancel:**

```sh
acme reject "$RUN" --as bob --comment "not yet"   # a maintainer rejects
acme reject "$RUN" --as ann                       # the requester cancels
```

**Check status, read the record, list runs:**

```sh
acme status run-0001
acme status run-0001 --observe     # also re-reads DEPRECATED.md on the origin's main now
acme record run-0001               # full record with every event, the preview and the outcome
acme runs --repo alpha --status succeeded
```

**Run the tests** (throwaway origins under `org/.test-tmp/`; the real origins are not touched):

```sh
python3 -m unittest discover -s tests -v
```

## Testing the refusals by hand

All of these work on the real repositories. The successful runs have already put `DEPRECATED.md` on
each `main`. If you request the **same** inputs again, the run succeeds without pushing
(`outcome.alreadyInPlace: true`), because the content is already observed on `main`. To exercise a
push, use a different `reason` or `successor`; the preview then shows `change: "modify"`.

```sh
# Invalid inputs: exit 2, error.code "invalid_input", no run created
acme request alpha mark-deprecated --as ann -i successor=Acme/Proxy -i "reason=Long enough reason text here."
acme request alpha mark-deprecated --as ann -i successor=acme/alpha -i "reason=Long enough reason text here."
acme request alpha mark-deprecated --as ann -i successor=acme/x -i "reason=too short"
acme request alpha mark-deprecated --as ann -i successor=acme/x -i "reason=$(printf 'x%.0s' $(seq 201))"
acme runs                                   # unchanged

# Self-approval and carol: exit 3, error.code "forbidden"; the run stays awaiting_approval
RUN=$(acme request alpha mark-deprecated --as ann -i successor=acme/proxy-v3 \
  -i "reason=Testing approval rules on a fresh request." | python3 -c 'import json,sys; print(json.load(sys.stdin)["run"]["id"])')
acme approve "$RUN" --as ann
acme approve "$RUN" --as carol
acme record "$RUN"                          # shows both approval_refused events
acme reject "$RUN" --as bob                 # tidy up without changing main

# Rejected push: the run is reported failed (stage "push"), never succeeded
printf '#!/bin/sh\necho "main is protected" >&2\nexit 1\n' > ../origins/alpha.git/hooks/pre-receive
chmod +x ../origins/alpha.git/hooks/pre-receive
RUN=$(acme request alpha mark-deprecated --as ann -i successor=acme/proxy-v3 \
  -i "reason=Testing what happens when the push is rejected." | python3 -c 'import json,sys; print(json.load(sys.stdin)["run"]["id"])')
acme approve "$RUN" --as bob; echo "exit $?"   # exit 6
acme status "$RUN"                             # status "failed", observed.matches false
rm ../origins/alpha.git/hooks/pre-receive
```

## Notes and limits

- **Identity** is asserted with `--as`, as the sandbox allows; there is no authentication. People
  must be listed in `org/acme.json`, and only a definition's `approvers` can approve.
- **Content** is exactly `# Deprecated: use <successor>`, a blank line, `<reason>`, then a final
  newline.
- **Run records** live in `org/state/`, which is git-ignored so that runs do not race on the org
  repository. Back the directory up if the records must outlive this machine.
- **A crash** between `started` and the outcome leaves a run `running`. `acme status <run> --observe`
  shows whether the change reached `main`.
