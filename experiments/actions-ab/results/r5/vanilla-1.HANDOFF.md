# HANDOFF: `mark-deprecated` self-service action

**Status:** built, and run on all three repositories. ann requested each run and bob approved it.
All three are reported `succeeded` after `DEPRECATED.md` was observed on each origin's `main`:

| Run | Repository | successor | Commit on origin `main` |
|---|---|---|---|
| `run-0001` | alpha | `acme/dependabot-proxy-v2` | `d82b91a` |
| `run-0002` | beta | `acme/valtio-store` | `5a0080b` |
| `run-0003` | gamma | `acme/graphify-core` | `ceda1ac` |

**Nothing has to be running.** The system is one stdlib-only Python 3 CLI, `org/acme`, with no
server and no packages to install. Each command does its work and exits.

## Layout

| Path | What it is |
|---|---|
| `org/acme` | The CLI: list, describe, preview, request, approve, reject, status, record, runs |
| `org/org.json` | The org: people (ann, bob, carol) and each repository's origin and checkout |
| `<repo>/.acme/actions/mark-deprecated.json` (committed to `origins/<repo>.git` `main`) | The repository's own definition: inputs and their rules, approvers, self-approval policy, effect (file path, content template, commit message) |
| `org/state/runs/run-NNNN.json` | One record per run (gitignored runtime state) |
| `org/state/audit.ndjson` | Append-only log of every event across runs, including denied approvals |

## How each requirement is met

| # | How | Where |
|---|---|---|
| R1 Owned by the repository | Each repository's definition is committed to its origin's `main` by ann (`alpha` 41182e9, `beta` 62d75bd, `gamma` e87af06). `acme` reads definitions only from `refs/heads/main` of the origin (`git cat-file`), never from a local copy or from `org`. The approver list and the self-approval policy come from the definition on `main` when the approval happens, so the team changes who may approve, the input rules or the content template by committing to its own repository. Every run records the definition commit it was requested and executed under. | `alpha/.acme/actions/mark-deprecated.json` (same path in `beta` and `gamma`); `load_definition` in `org/acme` |
| R2 Validated | `preview`, `request` and `approve` validate inputs against the definition: `successor` must match `^[a-z0-9-]+/[a-z0-9-]+$` and must not be the repository itself (`notIn: ["acme/<repo>"]`); `reason` must be 20 to 200 characters on a single line (no control characters); unknown or missing inputs are refused. An invalid request exits 2 with `error: invalid_input` and a list of errors. No run is recorded and nothing is pushed. Inputs are validated again at approval, against the definition then on `main`. | `validate_inputs` in `org/acme`; `inputs` in each definition |
| R3 Previewed | `acme preview` shows, without changing anything: the base commit, the commit message, and for each file its operation (`create`, `update` or `none`), its current and new content, and a unified diff. `request` returns the same preview and stores it in the run record, so approvers see it too. | `compute_changes` in `org/acme` |
| R4 Approved | A request creates a run in `pending_approval`; nothing touches the origin until `approve`. `approve` refuses (exit 3) anyone not in the definition's `approval.approvers` (carol: `not_an_approver`), the requester (`self_approval`, since `allowSelfApproval` is false), people not in `org/org.json` (`unknown_person`) and calls with no identity (`identity_required`). Every refusal is recorded in the run's events. A run can be approved only once (otherwise exit 5, `wrong_state`). A file lock serializes approvals. | `check_approver`, `cmd_approve` in `org/acme` |
| R5 Done means observed | After approval, `acme` clones the origin's `main`, commits `DEPRECATED.md` and pushes it to `main`. If the push is rejected, the run is `failed` with `reason: push_rejected` and git's stderr. If the push is accepted, `acme` reads `main:DEPRECATED.md` back from the origin. Only an exact byte match makes the run `succeeded`; otherwise it is `failed`, `reason: not_observed_on_main` (tested with a hook that accepts the push and then resets `main`). `approve` exits 0 only when the run succeeds and 6 when it fails. The observation (main commit, match, whether the pushed commit is on `main`) is stored in `outcome`. | `execute` in `org/acme` |
| R6 One place for the org | Everything runs from `org/`: `./acme list` reads every repository's origin listed in `org/org.json`, and `request`, `approve` and the rest take a repository name. Run records for all repositories live in `org/state/`. | `org/acme`, `org/org.json` |
| R7 Audited | Each run record has `requestedBy`/`requestedAt`, `approvedBy`/`approvedAt` (or `rejectedBy`/`rejectedAt`), `inputs`, `status`, `outcome` and `finishedAt`. It also holds the stored preview, the definition commits, and a timestamped `events` list with an actor on each event (requested, approval_denied, approved, executing, observed, succeeded or failed). The same events are appended to `org/state/audit.ndjson`. The commit on `main` is authored by the requester, committed by `acme-actions`, and carries `Acme-Run`, `Requested-by` and `Approved-by` trailers. | `event`, `save_run` in `org/acme`; `org/state/` |
| R8 Usable by agents | Every command is non-interactive, takes identity as `--as <person>` or `ACME_USER`, and prints one JSON document on stdout, errors included (`{"ok": false, "error": ..., "message": ...}`). Exit codes: 0 ok, 1 usage or internal error, 2 invalid input, 3 not permitted, 4 not found, 5 wrong run state, 6 run failed. `./acme --help` and `./acme <command> --help` describe each command. | `org/acme` |

## Commands

All commands run from the org checkout:

```bash
cd /tmp/actions-ab/r5/vanilla-1/org
```

Start: nothing to start.

List the actions on all three repositories, or describe one:

```bash
./acme list
./acme describe alpha
```

Preview (no identity needed; changes nothing):

```bash
./acme preview alpha --successor acme/dependabot-proxy-v2 --reason 'Replaced by the v2 proxy, which supports OIDC registries.'
```

Request as ann. The output's `run.id` is the run id, e.g. `run-0004`:

```bash
./acme request alpha --as ann --successor acme/dependabot-proxy-v2 --reason 'Replaced by the v2 proxy, which supports OIDC registries.'
RUN=$(./acme request beta --as ann --successor acme/valtio-store --reason 'State management moves to the shared store package.' | python3 -c 'import json,sys; print(json.load(sys.stdin)["run"]["id"])')
```

Inputs can also be passed as `--input successor=... --input reason=...` or `--inputs-json '{"successor": "...", "reason": "..."}'`.

Approve as bob. This executes the run and verifies it before returning, so the output is the final status:

```bash
./acme approve "$RUN" --as bob
```

Reject as an approver, or cancel as the requester:

```bash
./acme reject "$RUN" --as bob --comment 'not yet'
```

Check a run's status, read its full record, or list runs:

```bash
./acme status "$RUN"
./acme record "$RUN"
./acme runs                      # also: --repo alpha, --status pending_approval
```

## Testing the rules

The three repositories already contain `DEPRECATED.md`. A new run with the **same** inputs has
nothing to change: its preview shows `operation: none`, and approving it pushes nothing and reports
`succeeded`, because the content is observed on `main`. To exercise a real change (and a rejected
push), use a different reason, for example `'Testing a rejected push for this repository.'`.

```bash
# Invalid inputs: exit 2, no run recorded
./acme request alpha --as ann --successor 'Acme/Bad_Name' --reason 'too short'
./acme request alpha --as ann --successor acme/alpha --reason 'Pointing at itself should be refused.'

# Self-approval and carol: exit 3, recorded as approval_denied
RUN=$(./acme request alpha --as ann --successor acme/new-proxy --reason 'Testing the approval rules here.' | python3 -c 'import json,sys; print(json.load(sys.stdin)["run"]["id"])')
./acme approve "$RUN" --as ann
./acme approve "$RUN" --as carol

# Rejected push: the run ends failed (exit 6), and status never says succeeded
printf '#!/bin/sh\necho "push rejected by policy" >&2\nexit 1\n' > ../origins/alpha.git/hooks/pre-receive
chmod +x ../origins/alpha.git/hooks/pre-receive
./acme approve "$RUN" --as bob
./acme status "$RUN"            # status: failed, outcome.reason: push_rejected
rm ../origins/alpha.git/hooks/pre-receive
```

A failed run stays failed; request a new run to try again.

## Limits

- Identity is asserted (`--as`), as the sandbox has no passwords. In a real deployment, the `--as`
  check would be replaced by the caller's authenticated identity.
- One approval is required; the definition schema has no multi-approver quorum.
- The only effect type is `write-file` to `main`.
- Run records are local files in `org/state/` (gitignored). They are not committed to `org.git`.
