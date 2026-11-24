# HANDOFF: `mark-deprecated`, a repository-owned self-service action

**Status.** `mark-deprecated` is defined in `alpha`, `beta` and `gamma`, each on its origin's `main`.
It has run once on each repository: ann requested, bob approved, and each run is reported
`succeeded` after `DEPRECATED.md` was read back from the origin's `main`.

| Run | Repository | successor | Requested by | Approved by | Status | Commit on `main` |
|---|---|---|---|---|---|---|
| 1 | acme/alpha | `acme/dependabot-proxy-v2` | ann | bob | succeeded | `97a4c2e` |
| 2 | acme/beta | `acme/valtio-store` | ann | bob | succeeded | `b079295` |
| 3 | acme/gamma | `acme/graphify-core` | ann | bob | succeeded | `e6dc34b` |

Check them with `bin/acme runs` and `bin/acme record 1` (see [Commands](#commands)).

## How it fits together

The platform is GitHub Actions, run locally with act (`../bin/act`, host mode
`-P ubuntu-latest=-self-hosted`), because there is no GitHub and no Docker here.

- **In each repository (the action, owned by its maintainers):**
  - `.github/workflows/mark-deprecated.yml` is a `workflow_dispatch` workflow with inputs
    `successor` and `reason`. It has three jobs: `validate`, then `preview`, then `apply`. `apply`
    sits behind the environment `deprecation`, pushes `DEPRECATED.md` to `main`, then verifies it there.
  - `.github/self-service/mark_deprecated.py` holds the step logic (`validate|preview|apply|verify`).
  - `.github/self-service.yml` is the repository's catalogue. It declares the action, which job
    previews and which job applies, and the `deprecation` environment's reviewers (`ann`, `bob`)
    with `prevent_self_review: true`.
  - `.github/self-service/apply_github_settings.py` and `.github/workflows/self-service-settings.yml`
    apply that environment and a `main` ruleset to GitHub through the REST API.
  - `CODEOWNERS` (root in alpha, `.github/CODEOWNERS` in beta and gamma) makes ann and bob the
    owners of all of the above.
- **In `org/` (one place for the org):**
  - `bin/acme` is a CLI that prints JSON. It reads every repository's catalogue from its origin's
    `main`, runs the workflow's jobs with act, holds each run at the environment gate until a
    permitted reviewer approves, then observes `main` itself and records every step.
  - `acme.yml` lists org members (ann, bob, carol), the three repositories with their origins, and
    the act settings.
  - Run records and the audit log live in `org/.state/`, which is gitignored. That is the run
    history GitHub would keep.

A run goes through these states:

`request` → `previewing` → `awaiting_approval` (or `refused` when inputs are invalid) → `approve` →
`running` → `succeeded` | `failed`. A run awaiting approval can also be rejected (`rejected`).

## Requirements

| # | How it is met | Files |
|---|---|---|
| R1 Owned by the repository | The whole definition is committed to each repository's origin `main`: the workflow, its script, the catalogue entry and the approvers. CODEOWNERS gives ann and bob ownership, so the team changes it by pull request. `acme` reads the definition from the origin's `main` at request time, so a merged change takes effect on the next request; a request already made keeps the commit it previewed. The org config holds no per-repository action logic. | `<repo>/.github/workflows/mark-deprecated.yml`, `<repo>/.github/self-service/mark_deprecated.py`, `<repo>/.github/self-service.yml`, `alpha/CODEOWNERS`, `{beta,gamma}/.github/CODEOWNERS` |
| R2 Validated | The workflow's `validate` job runs first, and `preview` and `apply` `need` it. It refuses a `successor` that doesn't match `^[a-z0-9-]+/[a-z0-9-]+$` or that names the repository itself ("another repository"). It refuses a `reason` outside 20 to 200 characters (Unicode characters, not bytes), one containing control characters such as newlines (the file has one reason line), and a blank one. Before running anything, `acme` also refuses unknown or missing inputs. An invalid request is recorded as `refused`, with exit code 2, and nothing runs past validation. `apply` re-validates before it writes. | `mark_deprecated.py` (`validate_inputs`), `mark-deprecated.yml` (`validate` job), `org/bin/acme` (`parse_inputs`, `interpret_preview`) |
| R3 Previewed | The `preview` job clones the origin's current `main` and reports the change as JSON: path, `create`, `update` or `unchanged`, the exact new content, and a unified diff. On GitHub it also writes the step summary, which a reviewer sees on the run page before approving. `acme preview` shows it without creating a run. `acme request` stores it in the run record (`.preview`), so the requester and the approver can read it before anything is pushed. | `mark_deprecated.py` (`cmd_preview`), `mark-deprecated.yml` (`preview` job), `org/bin/acme` (`cmd_preview`, `cmd_request`) |
| R4 Approved | `apply` declares `environment: deprecation`. On GitHub, the environment's required reviewers (ann, bob) and `prevent_self_review: true` hold the job until another maintainer approves. act ignores environment protection, so `acme approve` enforces the same rule from the same declaration. It reads the environment's reviewers from the repository's `.github/self-service.yml` on `main` at approval time, refuses anyone who is not a reviewer (carol), refuses the requester (self-review), and refuses non-members. Only after that does it run the `apply` job. Refused attempts get exit code 3 and are recorded (`approval_denied`). A run can be approved once; later attempts get exit code 5. | `<repo>/.github/self-service.yml` (`environments.deprecation`), `mark-deprecated.yml` (`apply.environment`), `apply_github_settings.py`, `org/bin/acme` (`cmd_approve`) |
| R5 Done means observed | `apply` commits `DEPRECATED.md` with trailers (`Acme-Run-Id`, `Requested-by`, `Approved-by`) and pushes to `main`. It retries only when `main` moved underneath it, and fails on any other rejection. Its `verify` step then re-clones `main` and checks the content and that the run's commit is on `main`. Then `acme` does not trust the workflow: it reads the origin's `main` itself and reports `succeeded` only if the act job exited 0, every previewed file on `main` equals the previewed content, and this run's own commit is an ancestor of `main`. Because the run's commit must be on `main`, a rejected push fails even when an earlier run already left identical content. A rejected push gives `failed`, exit code 1, and the hook's message in `.outcome.reason`. | `mark_deprecated.py` (`cmd_apply`, `cmd_verify`), `org/bin/acme` (`observe`, `cmd_approve`) |
| R6 One place for the org | From `org/`, `bin/acme list` lists `mark-deprecated` on all three repositories: inputs, the definition commit, and the approvers. `bin/acme preview\|request <repo> mark-deprecated` acts on any of them without entering its checkout. Repositories are registered in `org/acme.yml`. | `org/bin/acme`, `org/acme.yml` |
| R7 Audited | Each run has a record, `org/.state/runs/<id>/record.json`, read with `acme record <id>`. It holds `requested_by` and `requested_at`, `approval.by` and `approval.at` with the reviewer list it was checked against, `inputs`, the definition commit, the preview, and `outcome` (`status`, `at`, the `main` commit observed, and the reason). Its `events` list timestamps every step, including denied approvals. Every event is also appended to `org/.state/audit.jsonl` (`acme audit`). Each run's act logs are kept in `runs/<id>/{preview,apply}.log`. On the code host, each run's commit carries `Acme-Run-Id`, `Requested-by` and `Approved-by` trailers. | `org/bin/acme` (`add_event`, `finish`, `cmd_record`, `cmd_audit`), `org/.state/` |
| R8 Usable by agents | Every step is a non-interactive `bin/acme` subcommand that prints one JSON object, errors included (`{"ok": false, "error": {"code", "message", ...}}`). Exit codes are 0 ok, 1 run failed, 2 invalid input or usage, 3 permission denied, 4 not found, and 5 wrong state. Identity is `--as <person>` or `ACME_AS`. Inputs are `--input NAME=VALUE`, repeatable, or `--inputs-json '{...}'`. | `org/bin/acme` |

## Commands

Nothing has to be running: `acme` is a CLI, and each command runs act in the foreground and
returns. You need `python3` with PyYAML (installed here), `git`, `jq` for the examples, and act at
`../bin/act`. Run everything from the org checkout:

```bash
cd /tmp/actions-ab/r7/gha-2/org

# List the actions of all three repositories, or one
bin/acme list
bin/acme list --repo beta

# Preview, without creating a run
bin/acme preview alpha mark-deprecated \
  --input successor=acme/dependabot-proxy-v2 \
  --input "reason=Replaced by the v2 proxy, which supports OIDC registries."

# Request a run as ann: validates and previews, then waits for approval.
# The output has the run id (.id) and the preview (.preview).
bin/acme request alpha mark-deprecated --as ann \
  --input successor=acme/dependabot-proxy-v2 \
  --input "reason=Replaced by the v2 proxy, which supports OIDC registries."
RUN=$(bin/acme runs --status awaiting_approval | jq -r '.runs[-1].id')

# Approve as bob: runs apply, then observes main. Exit 0 means succeeded, 1 means failed.
bin/acme approve "$RUN" --as bob --comment "LGTM"

# Status, the full record, every run, and the audit log
bin/acme status "$RUN"
bin/acme record "$RUN"
bin/acme runs
bin/acme audit --run "$RUN"

# Reject instead of approving (a reviewer or the requester)
bin/acme reject "$RUN" --as bob --comment "wrong successor"
```

To test the refusals:

```bash
# Invalid inputs: refused before anything changes (exit 2; recorded as "refused")
bin/acme request beta mark-deprecated --as ann --input successor=Acme/Store --input "reason=too short"
bin/acme request beta mark-deprecated --as ann --input successor=acme/beta --input "reason=Twenty characters or more here."
bin/acme request beta mark-deprecated --as ann --input successor=acme/x \
  --input "reason=$(python3 -c 'print("x"*201)')"
bin/acme request beta mark-deprecated --as ann --input successor=acme/x   # missing reason: refused, no run created

# Self-approval and carol: both refused (exit 3), recorded as approval_denied
RUN=$(bin/acme request beta mark-deprecated --as ann --input successor=acme/valtio-store \
  --input "reason=State management moves to the shared store package." | jq -r .id)
bin/acme approve "$RUN" --as ann
bin/acme approve "$RUN" --as carol
bin/acme status "$RUN"            # still awaiting_approval

# Rejected push: reported failed (exit 1), never succeeded
printf '#!/bin/sh\necho "main is frozen" >&2\nexit 1\n' > ../origins/beta.git/hooks/pre-receive
chmod +x ../origins/beta.git/hooks/pre-receive
bin/acme approve "$RUN" --as bob  # status "failed"; .outcome.reason quotes the hook
rm ../origins/beta.git/hooks/pre-receive
```

The rejected-push test also fails when the requested content is already on `main`, because success
requires this run's own commit on `main`. Runs that succeed really change the origin's `main`. To
check validation without changing anything, use `bin/acme preview`.

## What GitHub would enforce, and what could not run here

| Feature | Here | On GitHub |
|---|---|---|
| Required reviewers and preventing self-review | `acme approve` enforces both from `.github/self-service.yml`. | `apply_github_settings.py` sends `PUT /repos/{repo}/environments/deprecation` with reviewers ann and bob and `prevent_self_review: true`. GitHub then pauses `apply` until a reviewer other than the user who triggered the run approves, and records the approval (`GET /actions/runs/{id}/approvals`, which `mark_deprecated.py` reads for the `Approved-by` trailer). |
| Deployment branches | Not enforced; runs always use `main`. | Custom deployment branch policy `main` on the environment. |
| Branch protection | The origins accept direct pushes. | A ruleset on `main` requires a pull request with a code owner review, blocks force-push and deletion, and lets GitHub Actions (integration 15368) bypass, so only approved runs push directly. Together with CODEOWNERS, changes to the action need ann's or bob's review. |
| Settings as code | Dry run only: `python3 .github/self-service/apply_github_settings.py --repo acme/alpha --dry-run` prints the requests. | `self-service-settings.yml` reapplies them on every push to `main` that changes `self-service.yml`. It needs the secret `SELF_SERVICE_ADMIN_TOKEN` with Administration: write, because `GITHUB_TOKEN` cannot manage environments. |
| Run history and preview display | `org/.state/` records and the logs; previews in JSON. | The Actions run history; `preview` writes `$GITHUB_STEP_SUMMARY`; `run-name` shows the successor and requester. |
| Identity | `--as` (no passwords in this sandbox); members are checked against `acme.yml`. | The GitHub login of the dispatcher and the reviewer. |

**Not run here:**
- Anything against the real GitHub API: environments, rulesets, approvals and the settings workflow.
- The HTTPS push path with `GH_TOKEN` in `mark_deprecated.py`. Locally it pushes to the bare origin
  through `vars.ACME_ORIGIN_URL`.
- `actions/checkout`: act copies the workspace instead.

**Unverified:** whether the ruleset's `Integration` bypass for GitHub Actions (app id 15368) lets
`GITHUB_TOKEN` push to `main` on every plan is from GitHub's documentation, not tested. If it does
not, use a GitHub App token for the push.

**Checked here:**
- act `--validate --strict` passes on both workflows.
- yamllint passes with alpha's `.yamllint.yaml`.
- On scratch copies of the origins, I tested:
  - invalid inputs;
  - self-approval, carol and a non-member;
  - double approval;
  - reject;
  - a push rejected by a `pre-receive` hook, including when the content was already on `main`;
  - `main` moving between request and approval.

## Limits

- `org/.state/` is a local, writable store: the audit log is append-only by convention, not
  tamper-proof. On GitHub, the run history and approvals are kept by GitHub.
- If `acme approve` is killed mid-run, the record stays `running`. Read `runs/<id>/apply.log` and
  `main` to see what happened.
- Approvals use the reviewers on `main` at approval time, as GitHub does. The run executes the
  definition commit it previewed at request time.
