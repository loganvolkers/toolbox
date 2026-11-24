# Handoff: `mark-deprecated` on GitHub Actions

All three runs succeeded. ann requested each one and bob approved it, and the system read
`DEPRECATED.md` back from each origin's `main` before it reported success:

| Run | Repository | Commit on `main` | Status |
|---|---|---|---|
| `run-0001` | acme/alpha | `a9298a4` | succeeded |
| `run-0002` | acme/beta | `ed76d97` | succeeded |
| `run-0003` | acme/gamma | `fdadfb7` | succeeded |

The design has two parts:

- **In each repository, owned by its maintainers:** a `workflow_dispatch` workflow with four jobs,
  `validate → preview → apply → verify`. `apply` runs in the `deprecation` environment, whose
  protection rules (ann and bob as required reviewers, no self-review) are kept as code.
- **In the org repository:** `bin/acme-actions`, the glue that GitHub Actions does not provide
  locally. It finds the actions on each origin's `main`, runs the workflows with act in host mode,
  stands in for GitHub's environment approval gate and run history, and reads `main` back before
  it reports success.

## Files

| Path | What it is |
|---|---|
| `<repo>/.github/workflows/mark-deprecated.yml` | The action: a `workflow_dispatch` workflow with jobs `validate`, `preview`, `apply` (environment `deprecation`) and `verify` |
| `<repo>/.github/self-service/mark_deprecated.py` | The workflow's steps: `validate`, `preview`, `gate`, `apply`, `verify` (Python standard library only). Each prints JSON |
| `<repo>/.github/self-service/mark-deprecated.json` | The action manifest: description, input rules (pattern, 20–200 characters), and the postcondition (path, branch, exact content template). The script and the org CLI both read it |
| `<repo>/.github/environments/deprecation.json` | Environment protection as code: `reviewers: [ann, bob]`, `prevent_self_review: true`, deployments from `main` only |
| `<repo>/.github/rulesets/protect-main.json` | Ruleset for `main`: pull request with code-owner review, no force-push, no deletion |
| `<repo>/.github/self-service/apply-github-settings.sh` | Applies the environment and the ruleset to a real GitHub repository through the REST API (`gh api`) |
| `alpha/CODEOWNERS`, `beta/.github/CODEOWNERS`, `gamma/.github/CODEOWNERS` | `@ann @bob` own the workflow, `.github/self-service/`, `.github/environments/` and `.github/rulesets/` |
| `org/bin/acme-actions` | The org CLI: `list`, `preview`, `request`, `approve`, `reject`, `status`, `record`, `runs` |
| `org/catalog.json` | The org's repositories (with their origin paths), the people (ann, bob, carol), and the act binary |
| `org/runs/<run-id>.json` | One audit record per run. Each state change is also committed and pushed to `origins/org.git` |
| `org/runs/<run-id>/{preview,execute}.log` | The act logs for each phase of a run |

`<repo>` is `alpha`, `beta` or `gamma`. Each repository's copy was committed to its origin's `main`
by the commit "Add the mark-deprecated self-service action" (`3a9c224`, `de548bc`, `a8ad56b`).

## How to use it

Nothing needs to be started: there is no server and no daemon. The CLI needs `python3` with
PyYAML (6.0.4 is installed), `git`, and `/tmp/actions-ab/r6/gha-2/bin/act`. `jq` is only used
below to pick fields out of the output.

Every command prints a single JSON object on stdout. Exit codes:

- `0`: ok;
- `1`: refused, denied or failed (the JSON has `ok: false` and an `error` or `errors` field);
- `2`: usage error, also reported as JSON.

Say who you are with `--as NAME` or `ACME_ACTOR=NAME`; only ann, bob and carol are known.

```bash
cd /tmp/actions-ab/r6/gha-2/org

# List the action on every repository (inputs, rules, reviewers, definition commit)
bin/acme-actions list
bin/acme-actions list mark-deprecated --repo beta

# Preview: validate and show the diff to main; changes nothing and creates no record
bin/acme-actions preview mark-deprecated --repo alpha --as ann \
  --input successor=acme/dependabot-proxy-v2 \
  --input "reason=Replaced by the v2 proxy, which supports OIDC registries."

# Request a run as ann; it is validated and previewed, then waits for approval
RUN=$(bin/acme-actions request mark-deprecated --repo alpha --as ann \
  --input successor=acme/dependabot-proxy-v2 \
  --input "reason=Replaced by the v2 proxy, which supports OIDC registries." | jq -r .run.id)
#   (agents can pass --inputs-json '{"successor":"...","reason":"..."}' instead of --input)

# Approve as bob; this executes the run and returns its final status
bin/acme-actions approve "$RUN" --as bob

# Check a run's status, read its full record, list runs
bin/acme-actions status "$RUN"      # .run.status, .run.succeeded
bin/acme-actions record "$RUN"      # everything, including .record.events
bin/acme-actions runs --repo alpha --status succeeded

# Reject (a reviewer) or cancel (the requester) a run that is waiting
bin/acme-actions reject "$RUN" --as bob --comment "not yet"
```

A run's `status` is one of the following:

- `validating`;
- `refused`: invalid inputs; nothing changed;
- `awaiting_approval`;
- `rejected`;
- `running`;
- `succeeded`;
- `failed`.

`approve` blocks while the workflow runs, which takes a few seconds.

### Testing the rules

`alpha`, `beta` and `gamma` already carry `DEPRECATED.md`. A new run with the same inputs is a
no-op that still succeeds, because the content it observes is right. To test a real change, use
different inputs.

```bash
cd /tmp/actions-ab/r6/gha-2/org
OK_REASON="A reason that is long enough to pass."

# Invalid inputs: refused before anything changes (status "refused", exit 1)
bin/acme-actions request mark-deprecated --repo beta --as ann --input successor=Acme/Store --input "reason=$OK_REASON"
bin/acme-actions request mark-deprecated --repo beta --as ann --input successor=acme/store --input reason=too-short
bin/acme-actions request mark-deprecated --repo beta --as ann --input successor=acme/beta --input "reason=$OK_REASON"   # itself
bin/acme-actions preview mark-deprecated --repo beta --input successor=acme/store --input "reason=$(head -c 201 /dev/zero | tr '\0' x)"

# Self-approval and carol: denied (exit 1); the run stays awaiting_approval and the denial is recorded
RUN=$(bin/acme-actions request mark-deprecated --repo beta --as ann --input successor=acme/new-store --input "reason=$OK_REASON" | jq -r .run.id)
bin/acme-actions approve "$RUN" --as ann
bin/acme-actions approve "$RUN" --as carol
bin/acme-actions record "$RUN" | jq '.record.events'

# Rejected push: the run is reported "failed", never "succeeded"
printf '#!/bin/sh\necho "push rejected by test" >&2\nexit 1\n' > ../origins/beta.git/hooks/pre-receive
chmod +x ../origins/beta.git/hooks/pre-receive
bin/acme-actions approve "$RUN" --as bob          # exit 1, .run.status == "failed"
bin/acme-actions record "$RUN" | jq '.record.outcome, .record.execution.steps.apply'
rm ../origins/beta.git/hooks/pre-receive
```

## R1–R8

| # | How it is met | Where |
|---|---|---|
| R1 Owned by the repository | The whole definition lives in each repository and was pushed to its origin's `main`: the workflow, the steps, the input rules and template, and the approval rules. CODEOWNERS gives it to ann and bob, and the ruleset requires a code-owner review to change it. The org CLI has no copy: it reads the definition from each origin's `main` and runs that commit. A run is pinned to the commit it was requested at (`definition.commit` in the record), so a team's edit applies to later runs | `<repo>/.github/workflows/mark-deprecated.yml`, `<repo>/.github/self-service/`, `<repo>/.github/environments/deprecation.json`, CODEOWNERS |
| R2 Validated | The repository's `validate` job checks four things: `successor` against `^[a-z0-9-]+/[a-z0-9-]+$`, `successor` is not the repository itself, `reason` is 20–200 characters (Unicode code points) and not blank, and `reason` has no control characters. It runs at request time, before any approval or push; an invalid request is recorded as `refused` with its errors, and nothing changes. `apply` validates again before it commits. The CLI also refuses unknown or missing inputs, as GitHub does for `workflow_dispatch` | `mark_deprecated.py` (`validation_errors`), `mark-deprecated.json` (`inputs`) |
| R3 Previewed | The `preview` job fetches `main` and reports `change` (`create`, `update` or `noop`), the base commit, the exact new content and a unified diff. `acme-actions preview` shows this without creating a run; `request` stores it in the record (`.preview`) before anyone can approve. On GitHub it goes to the run's job summary | `mark_deprecated.py` (`cmd_preview`), `acme-actions` (`cmd_preview`, `cmd_request`) |
| R4 Approved | `apply` runs in the `deprecation` environment, whose required reviewers are ann and bob with `prevent_self_review: true`. Locally, `acme-actions approve` enforces those rules from the repository's own file at the run's definition commit: the approver must be a reviewer (so carol is denied), must not be the requester, and the run must be `awaiting_approval`. A lock makes concurrent approvals safe. Inside the workflow, the `gate` step checks again: under act it reads the run record and refuses unless a different reviewer approved these exact inputs, so running act directly fails at `apply`. On GitHub, it reads the run's approvals through the API | `deprecation.json`, `acme-actions` (`cmd_approve`), `mark_deprecated.py` (`cmd_gate`) |
| R5 Done means observed | Two independent checks must both pass. The workflow's `verify` job fetches `main` and compares `DEPRECATED.md` byte for byte. Then the CLI reads `refs/heads/main:DEPRECATED.md` from the bare origin itself and compares it with the manifest's template. If the push is rejected or the content differs, the run is `failed`, and the record holds the push output and what was observed | `mark_deprecated.py` (`cmd_apply`, `cmd_verify`), `acme-actions` (`observe`, `cmd_approve`) |
| R6 One place for the org | From `org/`, `acme-actions list` shows `mark-deprecated` for all three repositories, and `request`/`approve`/`status`/`record`/`runs` work on any of them by name (`--repo alpha` or `acme/alpha`). The repositories are listed in `catalog.json`, and their actions are discovered from `.github/self-service/*.json` on each origin's `main` | `org/bin/acme-actions`, `org/catalog.json` |
| R7 Audited | `runs/<id>.json` records `requested_by`/`requested_at`, `inputs`, `preview`, `approval.by`/`approval.at`, `outcome` (status, time, errors, observed commit and content), and an `events` list with a timestamp and actor for each step: requested, validated, previewed, approval_denied, approved, started, pushed or push_failed, observed, succeeded or failed. Each state change is committed and pushed to `origins/org.git`, so the history is in git too. The deprecation commit on `main` carries `Requested-by`, `Approved-by` and `Run` trailers | `org/runs/`, `org/bin/acme-actions` (`event`, `save`, `commit_record`) |
| R8 Usable by agents | Every command is non-interactive, takes flags (or `--inputs-json`), prints a single JSON object, and uses exit codes `0`/`1`/`2`. `list` also returns a ready-made `request` command line for each action | `org/bin/acme-actions` |

## How GitHub would enforce this, and what could not run here

On GitHub, each `acme-actions` command maps to a GitHub feature:

| `acme-actions` command | GitHub |
|---|---|
| `list` | `gh workflow list -R acme/<repo>` and the repository's `.github/self-service/*.json` |
| `request` | `gh workflow run mark-deprecated.yml -R acme/<repo> -f successor=… -f reason=…` |
| `approve` | Approve in the UI, or `POST /repos/acme/<repo>/actions/runs/<id>/pending_deployments` |
| `status` / `record` | `gh run view <id>`, the run's job summaries, and `GET …/runs/<id>/approvals` |

GitHub's rules would be applied with `.github/self-service/apply-github-settings.sh acme/<repo>`,
run once by an admin and again after a change to the settings files:

- **Environment with required reviewers.** GitHub pauses the `apply` job until a reviewer
  approves.
- **`prevent_self_review`.** GitHub stops the person who triggered the run from approving it.
- **Deployment branch policy `main`.** The environment, and with it the push, can only be used by
  runs from `main`.
- **Ruleset.** Changes to `main` need a pull request with code-owner review, which protects the
  action's definition. The GitHub Actions app (integration id `15368`) can bypass it so `apply` can
  push `DEPRECATED.md`. That id is from memory and not verified.
- **Run history.** GitHub keeps it, with the triggering actor, approvals and logs.

What I could not run here:

- **GitHub itself.** No GitHub, so neither `apply-github-settings.sh` nor the GitHub branch of the
  `gate` step (`gh api …/approvals`) has run. The environment pause, `prevent_self_review`, the
  deployment branch policy, the ruleset, CODEOWNERS enforcement, the `concurrency` group and the
  `GITHUB_TOKEN` push were not exercised either. Locally, `acme-actions` enforces the same
  reviewer rules from the same file, and the `gate` step re-checks them under act.
- **`actions/checkout`.** act skips it and copies the checkout instead. The CLI clones the origin
  at the definition commit, so the workflow pushes straight to the bare origin.
- **Identity.** Identity is self-declared (`--as`), as the brief allows. Records are plain files,
  so their tamper evidence is the git history in `origins/org.git`, not access control. A forged
  record could get past the in-workflow `gate` if someone ran act by hand. The approval rules
  themselves are enforced by `acme-actions`.
- **Interrupted runs.** If `acme-actions approve` is killed while the workflow is running, the run
  stays `running`; there is no reaper.

Checks run before handing off (on throwaway copies of the origins, then on the real ones):

- valid preview;
- invalid inputs: bad pattern, short, long, the repository itself, blank, multi-line, unknown or
  missing inputs;
- self-approval and carol denied;
- approving a refused run denied;
- a push rejected by a `pre-receive` hook, reported as `failed`;
- running act directly without approval, stopped at `gate`;
- two concurrent approvals: one ran, the other was denied;
- updating an existing `DEPRECATED.md`;
- bob requesting and ann approving;
- auto-commit of records to the org origin;
- `act --validate --strict` on the workflow.

No long-running processes were started.
