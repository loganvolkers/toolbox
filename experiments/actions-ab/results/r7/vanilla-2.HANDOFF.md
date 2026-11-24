# HANDOFF: `mark-deprecated` self-service action

`mark-deprecated` is defined in each repository and run from the org by one CLI, `org/acme`. The
CLI is Python 3.12 with only the standard library. Nothing needs to stay running: there is no
server, and each command is a single process that exits.
`run-0001` (alpha), `run-0002` (beta) and `run-0003` (gamma) were requested by ann and approved by
bob. All three are reported `succeeded`, and `DEPRECATED.md` is on each origin's `main`.

All paths below are relative to `/tmp/actions-ab/r7/vanilla-2`.

## Pieces

| File | What it is |
|---|---|
| `alpha/.acme/actions/mark-deprecated.json` (also in `beta/`, `gamma/`) | The action definition, committed to each origin's `main`. It holds the owners (maintainers), the approval policy, the input rules and the effect (the file and template to write) |
| `org/acme` | The org CLI: discovery, validation, preview, request, approve/reject, execution, observation, records |
| `org/acme.json` | The org registry: the repositories (origin paths) and the known people (`ann`, `bob`, `carol`) |
| `org/.acme-state/runs/run-NNNN.json` | One record per run (git-ignored, local state) |
| `org/.acme-state/audit.jsonl` | An append-only log of every event in every run, including refused approvals |
| `org/.acme-state/lock` | An exclusive `flock`. Requests, approvals and executions are serialized through it |

## How each requirement is met

| # | How |
|---|---|
| R1 Owned by the repository | The definition is `<repo>/.acme/actions/mark-deprecated.json`, committed to `origins/<repo>.git` `main`. `acme` reads definitions **only** from the origin's `main` (`git ls-tree`/`cat-file` on the bare repo; see `load_definitions` in `org/acme`), never from the org or a checkout. A team changes the action, including its `owners`, inputs and template, by committing to its own repo. A run records the definition's commit and blob. If the definition changes on `main` before approval, approval is refused (`definition_changed`) and the run must be requested again. |
| R2 Validated | `validate_inputs` in `org/acme` applies the definition's rules. `successor` must match `^[a-z0-9-]+/[a-z0-9-]+$` and must not be the repo itself (`acme/<repo>`). `reason` must be 20–200 characters. Unknown or missing inputs are refused. Both `preview` and `request` validate before anything is recorded or written. A refusal exits with code 2 and returns `{"error":{"kind":"invalid_input","errors":[...]}}`, and no run is created. |
| R3 Previewed | `acme preview` validates the inputs and returns the change computed against the origin's current `main`: `change` (`create`/`modify`/`none`), the unified `diff`, the full `after` content, `base_commit` and the commit message. It records nothing. `acme request` returns the same preview and stores it in the run. At execution, if `DEPRECATED.md` on `main` no longer matches what was previewed, the run fails (`changed since the preview`) instead of writing something nobody saw. |
| R4 Approved | A run starts in `pending_approval`. `acme approve` checks the approver against the `owners` in the repo's definition on `main` (`ann`, `bob`), refuses the requester (`self_approval`, exit 3) and refuses non-owners (`not_a_maintainer`, exit 3, which covers carol). Refused attempts are recorded as `approval_denied` events. Execution only happens inside `approve`, after a valid approval. Anyone known can *request*, including carol. |
| R5 Done means observed | `execute` in `org/acme` clones the origin, commits `DEPRECATED.md` and pushes to `main`. Then, **independently of the push result**, it reads `main:DEPRECATED.md` from the bare origin and compares the bytes with the previewed content. A run is `succeeded` only if the push succeeded (or no change was needed) **and** the observed content matches. A rejected push (for example from a `pre-receive` hook) gives `failed` with `push to origin main was rejected` and the hook output. A push that is accepted but doesn't land on `main` also gives `failed`. Both cases were tested in a scratch copy. `approve` exits 6 when the run fails. `acme verify <run>` re-observes the origin at any time, and `acme reconcile <run>` settles a run left `executing` by a crashed process, by observation. |
| R6 One place for the org | From `org/`, `./acme list` lists `mark-deprecated` for alpha, beta and gamma, read from their origins. `./acme preview` and `./acme request` take `--repo <name>`, so nobody has to go into a repository. `./acme runs` lists runs across repos. |
| R7 Audited | `acme record <run>` returns the full record: `requested_by`/`requested_at`, `inputs`, `approvals[{by,at}]`, the `preview`, the definition commit and blob, `push` attempts with git output, the `observation` (origin `main` sha, blob, sha256, `matches`) and `outcome{status,at,...}`. It also returns `events[]`, a timestamped list with an actor for each event: `requested`, `approval_denied`, `approved`, `execution_started`, `push_finished`/`push_failed`, `observed`, `succeeded`/`failed`/`rejected`/`cancelled`. Every event is also appended to `org/.acme-state/audit.jsonl`. The commit on `main` is authored by the requester, committed by `acme-actions`, and carries `Acme-Run:`, `Requested-by:` and `Approved-by:` trailers. |
| R8 Usable by agents | Every command is non-interactive and prints one JSON document on stdout. Exit codes: 0 ok, 2 invalid input, 3 forbidden, 4 not found, 5 wrong state (for example approving a run that is not pending), 6 the run executed and failed, 1 internal error. Identity is `--as <person>` or `ACME_USER=<person>`. `./acme <command> --help` documents each command. |

## Commands

Nothing to start. Requirements: `git` and `python3` (3.12). `jq` is optional, for reading output.

```bash
cd /tmp/actions-ab/r7/vanilla-2/org

# list the actions on all three repositories (or: --repo alpha)
./acme list
./acme list | jq -c '.actions[] | {repo, action, owners, valid}'

# preview (validates; changes and records nothing)
./acme preview mark-deprecated --repo alpha \
  --input successor=acme/dependabot-proxy-v2 \
  --input reason="Replaced by the v2 proxy, which supports OIDC registries."

# request as ann (returns run id + preview; status pending_approval)
./acme request mark-deprecated --repo alpha --as ann \
  --input successor=acme/dependabot-proxy-v2 \
  --input reason="Replaced by the v2 proxy, which supports OIDC registries."
# equivalently: ACME_USER=ann ./acme request mark-deprecated --repo alpha --inputs-json '{"successor":"...","reason":"..."}'

# approve as bob (executes, observes origin main, returns final status)
./acme approve run-0004 --as bob

# status, the full record, all runs
./acme status run-0004
./acme record run-0004
./acme runs                       # filters: --repo alpha --status succeeded

# other
./acme reject run-0004 --as bob --reason "not now"   # a maintainer rejects; the requester cancels
./acme verify run-0001                               # re-observe DEPRECATED.md on origin main now
./acme reconcile run-0004 --as bob                   # only for a run stuck in "executing"
```

### Negative tests to try

```bash
# invalid inputs -> exit 2, no run created
./acme request mark-deprecated --repo beta --as ann --input successor=Acme/Store --input reason="long enough reason here"
./acme request mark-deprecated --repo beta --as ann --input successor=acme/beta  --input reason="long enough reason here"
./acme request mark-deprecated --repo beta --as ann --input successor=acme/x     --input reason="too short"

# self-approval and carol -> exit 3, the run stays pending, attempts recorded as approval_denied
ID=$(./acme request mark-deprecated --repo beta --as ann --input successor=acme/other-store \
      --input reason="Testing approval rules for the handoff." | jq -r .run.id)
./acme approve "$ID" --as ann
./acme approve "$ID" --as carol

# rejected push -> exit 6, status failed, DEPRECATED.md on main unchanged
printf '#!/bin/sh\necho "main is protected" >&2\nexit 1\n' > ../origins/beta.git/hooks/pre-receive
chmod +x ../origins/beta.git/hooks/pre-receive
./acme approve "$ID" --as bob
./acme record "$ID" | jq '.run.status, .run.outcome, .run.observation'
rm ../origins/beta.git/hooks/pre-receive
```

## Notes

- `DEPRECATED.md` content is `# Deprecated: use <successor>\n\n<reason>\n`: exactly the
  specified lines, ending with a trailing newline. The template is `effect.content` in each
  repository's definition.
- All three repositories are already deprecated. A new request with the **same** inputs previews
  `change: "none"`. If approved, it succeeds without pushing, because the content is already observed
  on `main`. To test a rejected push, use a different `successor` or `reason`, as above, so that a
  push is actually needed.
- One approval from an owner other than the requester is required (`approval.required` in the
  definition). A repository can raise this in its own definition.
- Run state is local to this `org` checkout (`.acme-state/`, git-ignored). Run every command from
  `org/`, or call `org/acme` by its path: it locates its own config.
