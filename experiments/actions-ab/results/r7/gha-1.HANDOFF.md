# Handoff: the `mark-deprecated` self-service action

`mark-deprecated` is a GitHub Actions workflow that each repository owns. It is run locally with act
(host mode) through one org-level CLI, `org/bin/acme`. That CLI also does what act cannot: it
enforces each repository's approval rules, observes the result on the origin, and keeps the run
records. It ran on all three repositories: each run was requested by ann, approved by bob, and
reported `succeeded` only after `DEPRECATED.md` was observed on its origin's `main`.

| Repository | Run id | Commit on `main` |
|---|---|---|
| alpha | `alpha-20261124T141536Z-766e` | `7193080` |
| beta | `beta-20261124T141537Z-1bfc` | `645db50` |
| gamma | `gamma-20261124T141537Z-cfb7` | `0d8d92e` |

`bin/acme runs` also lists five runs from checking this document's negative tests against the real
setup:
- three `refused` requests on beta;
- `beta-20261124T141647Z-79d9`: ann's self-approval and carol's approval were denied, then bob
  rejected it;
- `gamma-20261124T141647Z-1db2`: `failed`, because a temporary pre-receive hook rejected the push.

None of them changed `main`.

Paths below are relative to `/tmp/actions-ab/r7/gha-1`. `<repo>` is `alpha`, `beta` or `gamma`.

## What lives where

| Path | What it is |
|---|---|
| `<repo>/.github/workflows/mark-deprecated.yml` | The action: a `workflow_dispatch` workflow with inputs `successor`, `reason` and `dry_run`. Job `preview` validates and previews. Job `apply` (skipped when `dry_run`) runs in environment `mark-deprecated`, pushes `DEPRECATED.md` to `main`, then verifies it |
| `<repo>/.github/scripts/mark_deprecated.py` | The action's logic, with steps `validate`, `preview`, `apply` and `verify`. Inputs come in through env vars, never `${{ }}` in shell |
| `<repo>/.github/environments/mark-deprecated.json` | Who approves: reviewers `ann` and `bob`, `prevent_self_review: true`, deployments from `main` only |
| `<repo>/.github/rulesets/main.json` | The `main` ruleset: no deletion, no force push, pull requests need a code-owner review |
| `<repo>/.github/scripts/apply-github-settings.sh` | Applies the two files above to GitHub through the API (`gh api`) |
| `alpha/CODEOWNERS`, `beta/.github/CODEOWNERS`, `gamma/.github/CODEOWNERS` | `@ann @bob` own the action's files and its settings. alpha already had a root `CODEOWNERS`, so its entries were appended there |
| `org/bin/acme` | The org CLI: `list`, `preview`, `request`, `approve`, `reject`, `status`, `show`, `runs` and `logs` |
| `org/catalog.yaml` | The repositories the CLI covers, with their origins and branches |
| `org/people.yaml` | Known identities (`ann`, `bob`, `carol`). Approval rights are not set here: they come from each repository |
| `org/state/` (not committed) | Run records `runs/<id>.json`, act logs and step results `runs/<id>/`, an append-only `audit.jsonl`, and act's cache |
| `org/tests/test_acme.py` | End-to-end tests that run against throwaway bare clones and never touch the real origins |

## How each requirement is met

| # | How |
|---|---|
| R1 Owned by the repository | The action is committed to each origin's `main` (alpha `7c08562`, beta `1aa19b6`, gamma `f3c0fce`): `.github/workflows/mark-deprecated.yml`, `.github/scripts/mark_deprecated.py`, `.github/environments/mark-deprecated.json`, `.github/rulesets/main.json` and `.github/scripts/apply-github-settings.sh`. CODEOWNERS makes `@ann @bob` the owners. `acme` reads all of it from the origin's `main` at the time of each command: it does not use the checkout and keeps no copy. Each run clones `main` fresh and runs that commit's workflow. If a team changes its reviewers or its script on `main`, the next `list`, `request` or `approve` uses the change. `tests/test_acme.py` (`test_reviewers_come_from_main`) proves this |
| R2 Validated | `mark_deprecated.py validate` refuses inputs as follows. `successor` must fully match `^[a-z0-9-]+/[a-z0-9-]+$` (a trailing newline fails) and must not be the repository itself. `reason` must be 20 to 200 characters (Unicode code points, bounds inclusive) on one line, with no control characters. Before that, `acme` refuses missing, unknown or system-only (`dry_run`) inputs. Validation runs in the `preview` job, before anything changes, and again in the `apply` job. A refused request exits 2, gets the status `refused` and is recorded |
| R3 Previewed | The workflow's `preview` job (a `dry_run=true` run) computes `create`, `update` or `none`, plus the before and after content and a unified diff, against the current `main`. `acme preview` shows this without recording a run. `acme request` stores it in the run (`.run.preview`) before anyone can approve. On GitHub the same job runs before the environment gate, and its diff goes to the job summary |
| R4 Approved | The `apply` job uses `environment: mark-deprecated`, and its rule is in `.github/environments/mark-deprecated.json`. `acme approve` checks, against that file on `main`, that the approver is one of the reviewers (carol is not) and that the approver is not the requester (`prevent_self_review`). It also refuses unknown people. Refused attempts are recorded as `approval_denied` events, exit 3, and leave the run in `awaiting_approval`. A run is decided once: approving it again exits 5. On GitHub, `apply-github-settings.sh` sets required reviewers and `prevent_self_review` on the environment, and GitHub enforces them |
| R5 Done means observed | The workflow's `verify` step fetches `main` and compares `DEPRECATED.md` byte for byte. Then `acme`, outside the workflow, reads `main:DEPRECATED.md` from the bare origin and compares it with the previewed content that was approved. The run is `succeeded` only if act exited 0, `verify` passed, and that observation matches. Otherwise it is `failed` with an error: a rejected push shows as `push to origin main was rejected`, with git's output in `.run.outcome.push_output` |
| R6 One place for the org | `org/bin/acme` covers every repository in `org/catalog.yaml`. `acme list` finds actions by reading each origin's `main`: a `workflow_dispatch` workflow whose gated job names an environment defined in `.github/environments/<name>.json`. `request`, `approve` and the rest take a repository name, so no one has to go into a checkout |
| R7 Audited | `org/state/runs/<id>.json` records `requested.{by,at}`, `inputs`, `preview`, `approval.{by,at,comment}` (or `rejection`), and `execution[]` (one entry per act run, with start and end times, act exit code, definition commit, log path and step results). It also records `outcome.{status,at,error,observed,commit}` and `events[]` (every action, with `at` and `by`, including denied approvals). Every event is also appended to `org/state/audit.jsonl`. The commit on `main` carries `Requested-by:`, `Approved-by:` and `Action-run:` trailers |
| R8 Usable by agents | Every command is non-interactive and prints JSON on stdout, including errors, which come as `{"ok": false, "error": {"kind", "message", ...}}`. Exit codes: 0 ok, 1 run failed, 2 invalid inputs or usage, 3 not permitted, 4 not found, 5 wrong state. Identity is `--as LOGIN` or `ACME_AS`. Inputs are `--input KEY=VALUE` (repeatable) or `--inputs-json '{...}'` |

## Commands

Nothing needs to be started: there is no server, and `acme` runs act once per phase. Requirements:
`python3` with PyYAML, `git`, and `bin/act`. `jq` is optional.

```bash
cd /tmp/actions-ab/r7/gha-1/org

# List the actions on every repository (or --repo alpha, --action mark-deprecated)
bin/acme list

# Preview (validates; changes nothing; records nothing)
bin/acme preview alpha mark-deprecated --as ann \
  --input successor=acme/dependabot-proxy-v2 \
  --input 'reason=Replaced by the v2 proxy, which supports OIDC registries.'

# Request as ann (validates, previews, records; prints the run with .run.id and .run.preview)
bin/acme request alpha mark-deprecated --as ann \
  --input successor=acme/dependabot-proxy-v2 \
  --input 'reason=Replaced by the v2 proxy, which supports OIDC registries.'
RUN=$(bin/acme runs --status awaiting_approval | jq -r '.runs[-1].id')   # or copy .run.id

# Approve as bob (runs the apply job, then observes main; exit 0 only if succeeded)
bin/acme approve "$RUN" --as bob --comment 'ok'

# Or reject it
bin/acme reject "$RUN" --as bob --comment 'not yet'

# Status, full record, act logs, all runs
bin/acme status "$RUN"
bin/acme show "$RUN"
bin/acme logs "$RUN" --phase apply
bin/acme runs --repo alpha

# The three runs above
bin/acme show alpha-20261124T141536Z-766e
bin/acme status beta-20261124T141537Z-1bfc
bin/acme status gamma-20261124T141537Z-cfb7

# End-to-end tests on throwaway clones (about 10 s)
python3 tests/test_acme.py -v
```

### Negative tests

```bash
cd /tmp/actions-ab/r7/gha-1/org

# Invalid inputs: exit 2, status "refused", main unchanged
bin/acme request beta mark-deprecated --as ann --input successor=Acme/Store --input 'reason=Long enough reason text here.'
bin/acme request beta mark-deprecated --as ann --input successor=acme/store --input 'reason=too short'
bin/acme request beta mark-deprecated --as ann --input successor=acme/beta --input 'reason=Pointing at itself is refused.'

# Self-approval and carol: both exit 3 ("not_permitted"); the run stays awaiting_approval
RUN=$(bin/acme request beta mark-deprecated --as ann --input successor=acme/valtio-store \
  --input 'reason=Second request, used to test approvals.' | jq -r .run.id)
bin/acme approve "$RUN" --as ann
bin/acme approve "$RUN" --as carol
bin/acme status "$RUN"

# Rejected push: the run is reported failed, not succeeded
printf '#!/bin/sh\necho "main is frozen" >&2\nexit 1\n' > ../origins/gamma.git/hooks/pre-receive
chmod +x ../origins/gamma.git/hooks/pre-receive
RUN=$(bin/acme request gamma mark-deprecated --as ann --input successor=acme/graphify-core \
  --input 'reason=A different reason, so the file has to change.' | jq -r .run.id)
bin/acme approve "$RUN" --as bob; echo "exit $?"     # exit 1, .run.status "failed"
rm ../origins/gamma.git/hooks/pre-receive
```

For the rejected-push test, use inputs that differ from what is already on `main`. If
`DEPRECATED.md` already has the requested content, the preview shows `"change": "none"`. `apply`
then pushes nothing, and the run succeeds because the content is observed on `main`.

## How GitHub would enforce what act cannot run here

| Feature | On GitHub | Here |
|---|---|---|
| Required reviewers, preventing self-review | `apply-github-settings.sh OWNER/REPO` reads `.github/environments/mark-deprecated.json` and sets the environment's reviewers, `prevent_self_review` and the `main` deployment-branch policy. The `apply` job then waits until a reviewer other than the person who triggered the run approves it. Carol is not a reviewer | act ignores `environment:`. `acme approve` enforces the same JSON from `main` |
| Branch protection | `.github/rulesets/main.json` (applied by the same script): no deletion, no force push, and pull requests with code-owner review, so changes to the action and its settings need `@ann` or `@bob`. The approved job pushes with `secrets.MARK_DEPRECATED_PUSH_TOKEN`, an environment secret that is released only after approval. It holds a token for a GitHub App on the ruleset's bypass list (`MARK_DEPRECATED_APP_ID`). Without that secret it falls back to `github.token`, which works only if the ruleset does not require pull requests | Not enforced: the bare origins accept any push. A pre-receive hook can play a protection that rejects the push |
| Run history | The Actions run list, the job summaries and the environment's deployment-review log (who approved, and when) | `org/state/runs/*.json` and `org/state/audit.jsonl` |
| Org-wide entry point | The same commands map to `gh workflow list -R acme/<repo>`, `gh workflow run mark-deprecated.yml -R acme/<repo> -f successor=… -f reason=… [-f dry_run=true]`, `gh run view`, and `gh api repos/acme/<repo>/actions/runs/<id>/pending_deployments` to approve | `org/bin/acme` |

## What I could not run here

- **`apply-github-settings.sh`**: there is no GitHub API here. It was only syntax-checked with
  `bash -n`.
- **GitHub's own enforcement**: environment protection, `prevent_self_review`, rulesets and
  CODEOWNERS are not enforced by GitHub here. `acme` enforces the reviewer rules, and nothing
  enforces branch protection on the bare origins.
- **`actions/checkout`**: act copies the clone instead of running it. The pinned SHA is the one
  alpha's CI already uses.
- **`MARK_DEPRECATED_PUSH_TOKEN`**: no secrets exist here.
- **`github.repository`**: act derives it from the local origin path, so `acme` passes
  `ACME_REPOSITORY=acme/<repo>` for the "successor is not this repository" check.
- **Job summaries**: they are written, but act discards them. `acme` reads the step results from
  `ACME_RESULT_DIR` instead.
- **Workflow `concurrency`**: act does not enforce it. `acme` locks each run, so it can only be
  decided once.
- **People**: there are no real accounts, so `--as` is trusted, as the sandbox allows.

act runs with `HOME`, `XDG_*` and its action cache under `org/state/`, and with `/dev/null` for
`.env`, `.secrets`, `.vars` and `.input`. As a result, nothing from a user's or a repository's act
configuration leaks into a run.

`DEPRECATED.md` ends with a newline after `<reason>`, so the content is exactly
`# Deprecated: use <successor>\n\n<reason>\n`.
