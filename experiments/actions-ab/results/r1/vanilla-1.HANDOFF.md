# HANDOFF: `mark-deprecated` self-service action

The system is one stdlib-only Python 3 CLI, `org/bin/acme-actions`. It has **no server**: there is nothing to start. Every
command prints one JSON document on stdout and exits non-zero on refusal, so a person or an agent can drive every step
non-interactively.

All three runs succeeded: `run-0001` (alpha), `run-0002` (beta) and `run-0003` (gamma). Each was requested by ann,
approved by bob and verified on its origin's `main`.

All paths below are relative to `/tmp/actions-ab/r1/vanilla-1`.

## Layout

| Path | What it is |
|---|---|
| `org/bin/acme-actions` | The CLI: list, show, preview, request, approve, reject, cancel, status, record, runs |
| `org/repos.json` | The repositories the org knows: name, slug (`acme/<name>`), origin path |
| `org/people.json` | Known people: ann, bob, carol |
| `org/state/runs/run-NNNN.json` | One record per run (gitignored local state; override the location with `ACME_ACTIONS_STATE`) |
| `org/state/audit.jsonl` | An append-only log of every event across all runs |
| `<repo>/.acme/actions/mark-deprecated.json` | The repository's definition of the action: inputs, approval policy, steps, commit message |
| `<repo>/.acme/maintainers.json` | The repository's maintainers (`["ann", "bob"]`), who may approve |

## How each requirement is met

| # | How |
|---|---|
| R1 Owned by the repository | The definition lives in `alpha/.acme/actions/mark-deprecated.json` (and the same path in `beta` and `gamma`). It is committed to each `origins/<name>.git` `main`. The CLI reads it **from the origin's `main`** (`git cat-file` on the bare repo), never from the checkout or from `org/`. Whoever can change a repository's `main` can change its inputs, template, commit message or approvers (`.acme/maintainers.json`). A run pins the definition commit it was requested against (`record.definition.commit`), so the change that executes is the one that was previewed. |
| R2 Validated | `validate_inputs` in `org/bin/acme-actions` applies the definition's rules before any preview, request or write. `successor` must match `^[a-z0-9-]+/[a-z0-9-]+$` and must not be this repository's own slug. `reason` must be 20 to 200 characters with no control characters or newlines. Inputs that are unknown or missing are refused. A refusal exits with code 2 and `{"ok": false, "error": {"code": "invalid_inputs", "details": {"errors": [...]}}}`, and no run is created. |
| R3 Previewed | `preview` validates the inputs and shows a unified diff of `DEPRECATED.md` against the current origin `main` (`base_commit`, `before`, `after`, `change`: create, modify or unchanged). `request` records the same preview in the run, and `record` shows it later. If origin `main` changes that file to something else between the request and the approval, the run fails with `stale_preview` instead of applying a change nobody saw. |
| R4 Approved | A run starts as `awaiting_approval` and executes only inside `approve`. `check_approver` refuses anyone who is not listed in the repository's `.acme/maintainers.json` on origin `main`, so carol gets `not_a_maintainer`. It also refuses the requester (`self_approval`), whatever the definition says. Both refusals exit with code 3 and are logged as `approval_refused` events. Approving a run that is not awaiting approval exits with code 6. |
| R5 Done means observed | After pushing, `observe` re-reads `DEPRECATED.md` from `origins/<name>.git` `refs/heads/main` and compares it byte for byte with the rendered content. The state becomes `succeeded` only if the push succeeded **and** that content was observed. Otherwise the state is `failed`, with `error.code` set to `push_rejected` (with git's output), `not_observed`, `stale_preview` or `clone_failed`. The observation (main commit, expected and observed content, time) is stored in `record.execution.verification`. `approve` exits with code 5 when a run fails. |
| R6 One place for the org | Everything runs from `org/`. `org/repos.json` points at every origin, `list` shows every repository's `mark-deprecated` (read from each origin), and `request <repo>` works for any of them without entering the repository. |
| R7 Audited | Each `org/state/runs/run-NNNN.json` has `requested_by`/`requested_at`, `approved_by`/`approved_at`, `inputs`, `decisions` (who, when, comment, and the maintainers source used), `preview`, `execution` (commit, push result, verification), `state`/`outcome`, `finished_at` and a timestamped `events` list (requested, approval_refused, approved, execution_started, pushed or push_rejected, verified or verification_failed, succeeded or failed). The events are also appended to `org/state/audit.jsonl`. The commit on `main` is authored by the requester, committed by `acme-actions`, and carries `Run-Id`, `Requested-by` and `Approved-by` trailers. |
| R8 Usable by agents | Every subcommand is non-interactive and prints JSON. Identity comes from `--as <person>` or `ACME_USER`. Exit codes: 0 ok, 1 internal error, 2 invalid input, 3 forbidden, 4 not found, 5 run failed, 6 wrong state. `acme-actions <cmd> --help` documents the flags. |

## Commands

There is nothing to start (no server, no daemon). Run the commands from the org directory:

```sh
cd /tmp/actions-ab/r1/vanilla-1/org
A=./bin/acme-actions
```

List the actions on all repositories (optionally filtered with `--repo alpha` or `--action mark-deprecated`):

```sh
$A list
$A show alpha mark-deprecated
```

Preview (this validates the inputs and changes nothing):

```sh
$A preview alpha mark-deprecated \
  --input successor=acme/dependabot-proxy-v2 \
  --input "reason=Replaced by the v2 proxy, which supports OIDC registries."
# or: --inputs-json '{"successor":"acme/x","reason":"..."}'
```

Request as ann. The output includes `run.id` (for example `run-0004`) and the preview:

```sh
$A request alpha mark-deprecated --as ann \
  --input successor=acme/dependabot-proxy-v2 \
  --input "reason=Replaced by the v2 proxy, which supports OIDC registries."
```

Approve as bob. The run executes and is verified within this command, which prints the final state:

```sh
$A approve run-0004 --as bob --comment "ok"
```

Reject or cancel a run instead of approving it:

```sh
$A reject run-0004 --as bob --comment "not yet"
$A cancel run-0004 --as ann
```

Check a run's status, read its full record, or list runs:

```sh
$A status run-0004
$A record run-0004
$A runs --repo alpha --state succeeded
```

## Testing the refusals

| Test | Command | Expected result |
|---|---|---|
| Invalid successor | `$A request alpha mark-deprecated --as ann -i successor=Acme/X -i "reason=twenty or more characters"` | exit 2, `invalid_inputs`, no run created |
| Reason too short or too long | `-i reason=short` (or 201 characters) | exit 2, `invalid_inputs` |
| Self-approval | `$A approve <run> --as ann` on ann's run | exit 3, `self_approval`; the run stays `awaiting_approval` |
| carol approves | `$A approve <run> --as carol` | exit 3, `not_a_maintainer` |
| Rejected push | Install a rejecting hook, then approve (example below) | exit 5, state `failed`, `error.code` `push_rejected`; `main` is unchanged |

The rejected push test, step by step:

```sh
printf '#!/bin/sh\necho "main is protected" >&2\nexit 1\n' > ../origins/beta.git/hooks/pre-receive
chmod +x ../origins/beta.git/hooks/pre-receive
$A request beta mark-deprecated --as ann -i successor=acme/other-store -i "reason=Testing that a rejected push fails."
$A approve <run-id> --as bob        # exit 5, "state": "failed", "error": {"code": "push_rejected", ...}
$A record <run-id>                  # execution.push.output holds git's rejection; verification shows the old content
rm ../origins/beta.git/hooks/pre-receive
```

## Notes

- `DEPRECATED.md` is written as `# Deprecated: use <successor>\n\n<reason>\n`, which ends with a newline.
- Requesting the same inputs again on a repository that already has that exact content previews `"change": "unchanged"`. After approval, the run pushes nothing and succeeds because the content is observed on `main`. To exercise a push, use different inputs.
- The sandbox has no authentication, so `--as` is trusted, as the task allows. Approval rights come from each repository's `.acme/maintainers.json` on origin `main`. In a real deployment, branch protection on `.acme/` would stop a non-maintainer from adding themselves there.
- The checkouts `alpha`, `beta` and `gamma` were fast-forwarded after the runs. The CLI never depends on them.
