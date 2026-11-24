# mark-deprecated: handoff

`mark-deprecated` is a GitHub Actions workflow that each repository owns. A small org-level
CLI, `org/bin/acme-actions`, lists the action across the org and runs requests through
preview, approval, execution and observation, keeping an audit record for each run. GitHub
isn't available here, so the CLI runs each repository's committed workflow with act in host
mode and enforces locally the approval rules that GitHub's `deprecation` environment would
enforce.

**Status.** All three runs succeeded. Each was requested by ann and approved by bob, and each
`DEPRECATED.md` was observed on its origin's `main`:

| Repository | Run id | Commit on `main` |
|---|---|---|
| alpha | `md-alpha-20261124T113112Z-65c7` | `276957bc7498` |
| beta | `md-beta-20261124T113115Z-6f64` | `594fa864a716` |
| gamma | `md-gamma-20261124T113117Z-e8dd` | `cc05b39b54a8` |

Nothing needs to be running: there is no server, and every step is one shell command.
`acme-actions runs` also lists the runs from checking the failure paths below. Two on beta
were refused for invalid inputs, and one on beta failed on a rejected push. None of them
changed an origin.

## Layout

Each repository has the same files, committed to its origin's `main` (`/tmp/actions-ab/r2/gha-3/origins/<repo>.git`):

| Path (in alpha, beta and gamma) | What it is |
|---|---|
| `.github/workflows/mark-deprecated.yml` | The action: `workflow_dispatch` inputs `successor`, `reason`, `mode` (`preview`/`apply`) and `request_id`. It has three jobs: `validate`, `preview`, and `apply`, which runs only in apply mode and is gated by `environment: deprecation`. Inputs reach the scripts through `env`, never through `${{ }}` in `run:`, so they can't inject shell. |
| `.github/mark-deprecated/run.py` | The action's logic: `validate`, `preview` and `apply`, where apply also verifies the result. It works the same outside the workflow. |
| `.github/environments/deprecation.json` | Who can approve: `reviewers: [ann, bob]`, `prevent_self_review: true`, and only `main` may deploy. |
| `.github/mark-deprecated/apply-github-settings.sh` | Applies `deprecation.json` to GitHub through the REST API (`gh api`). `DRY_RUN=1` prints the requests instead. |
| `.github/workflows/mark-deprecated-settings.yml` | Reruns that script whenever `deprecation.json` changes on `main`. |

The org repository (`/tmp/actions-ab/r2/gha-3/org`, origin `origins/org.git`):

| Path | What it is |
|---|---|
| `org/catalog.json` | The repositories and origins, the people (ann, bob, carol), the path to act, and where each repository keeps the action. |
| `org/bin/acme-actions` | The CLI (Python 3 with PyYAML). Every command prints one JSON document. |
| `org/.acme-actions/runs/<run-id>.json` | The run records (audit trail). Not committed: `.gitignore` excludes them. |
| `org/.acme-actions/runs/<run-id>/{preview,apply}/` | Each run's act log (`act.log`), the event it sent, and the workflow's own JSON results. |

## Requirements

| # | How it is met |
|---|---|
| R1 Owned by the repository | The workflow, its logic (`run.py`) and its approvers (`deprecation.json`) are committed under `.github/` on each origin's `main`. The CLI holds no action logic: `list` reads the definition from each origin's `main`, and `request` pins the run to that commit (`definition.commit` in the record), which `approve` then executes. If a team changes its files on `main`, the next run uses them. The same goes for approvers: `approve` reads `deprecation.json` from current `main`. |
| R2 Validated | `run.py` `validate()` checks that `successor` fully matches `^[a-z0-9-]+/[a-z0-9-]+$` and isn't the repository itself. It also checks that `reason` is 20 to 200 characters with no control characters such as newlines or tabs. The `validate` job runs first and the other jobs `need` it, and `apply` validates again before cloning. An invalid request is refused with exit code 2 and the list of errors; it is recorded with status `refused` and nothing reaches the origin. |
| R3 Previewed | The `preview` job clones `main` and stages the file without committing. It reports `create`, `update` or `unchanged`, the content before and after, and the unified diff. `acme-actions preview` shows this without creating a run, and `request` stores it in the record (`preview`) before anyone can approve. |
| R4 Approved | **Here:** `acme-actions approve` refuses the requester ("the requester cannot approve their own run"), anyone not listed in the repository's `deprecation.json` (so carol is refused), and anyone not in `catalog.json`. Each refusal exits 3 and is recorded as an `approval_denied` event. Only an approval starts the apply. A per-run file lock stops two approvals racing. **On GitHub:** the `apply` job's `environment: deprecation` holds the job until a required reviewer approves. `apply-github-settings.sh` configures ann and bob as required reviewers with `prevent_self_review: true`, so the person who dispatched the run can't approve it and carol can't approve at all. |
| R5 Done means observed | `run.py apply` pushes, then fetches `main` from the origin again and compares `DEPRECATED.md` byte for byte. It exits 0 only if the push succeeded (or nothing needed changing) and the content matches. After act exits, the CLI reads `DEPRECATED.md` from the origin's `main` itself and compares it with the approved preview content. A run is `succeeded` only if both agree; otherwise it is `failed`, with the push output and the observation recorded. A rejected push therefore ends as `failed` (exit 1). A non-fast-forward is retried up to 3 times by recloning; any other rejection is not retried. If the CLI dies mid-run, the next `status` or `record` call finds the run `running` with no live process and settles it as `failed`, with what `main` showed at that moment. |
| R6 One place for the org | From `org/`, `acme-actions list` shows `mark-deprecated` for alpha, beta and gamma: inputs, approvers, and the definition commit read from each origin. `preview`, `request`, `approve`, `status`, `record` and `runs` all take `--repo` or a run id, so nobody needs to enter a repository. |
| R7 Audited | Each record in `org/.acme-actions/runs/<id>.json` has `requested_by`/`requested_at`, `approved_by`/`approved_at`, `inputs`, `started_at`, `finished_at`, `status` and `outcome` (summary, observed commit and content, the workflow's result). Its `events` list holds every request, preview, refusal, denied approval, approval, rejection and outcome, with timestamp and actor. The commit on the origin also carries `Mark-Deprecated-Run`, `Requested-by` and `Approved-by` trailers, with ann as author and `mark-deprecated[bot]` as committer. |
| R8 Usable by agents | Every command takes only flags and prints one JSON document to stdout; errors come back as `{"error": {"code", "message", ...}}`. Exit codes: 0 ok, 1 run failed, 2 invalid input, 3 forbidden, 4 not found, 5 wrong state (for example, approving twice), 6 internal error. There are no prompts. The caller's identity comes from `--as` or `$ACME_USER`. |

## Commands

You need `git`, `jq` (used by the settings script and below), and Python 3 with PyYAML (`pip install pyyaml` if it's missing). act comes from `/tmp/actions-ab/r2/gha-3/bin/act`, set in `catalog.json`; the CLI always passes `-P ubuntu-latest=-self-hosted`. Nothing needs to be started.

```bash
cd /tmp/actions-ab/r2/gha-3/org
A=bin/acme-actions

# List the action on every repository
$A list

# Preview (changes nothing; --as is optional)
$A preview --repo alpha --successor acme/dependabot-proxy-v2 \
  --reason "Replaced by the v2 proxy, which supports OIDC registries." --as ann

# Request as ann (validates, previews, records; prints .run.id)
ID=$($A request --repo alpha --successor acme/dependabot-proxy-v2 \
  --reason "Replaced by the v2 proxy, which supports OIDC registries." --as ann | jq -r .run.id)

# Approve as bob (runs the workflow's apply job, then observes main)
$A approve "$ID" --as bob

# Check status / read the full record / list runs
$A status "$ID"
$A record "$ID"
$A runs --repo alpha

# Reject (a reviewer) or withdraw (the requester) a run awaiting approval
$A reject "$ID" --as bob --comment "not yet"
```

To test the failure paths:

```bash
# Invalid inputs: exit 2, status "refused", origin unchanged
$A request --repo beta --successor Acme/Store --reason "too short" --as ann
$A request --repo beta --successor acme/beta --reason "Points at itself, which is not allowed." --as ann

# Self-approval and carol: exit 3, recorded as approval_denied, run stays awaiting_approval
ID=$($A request --repo beta --successor acme/new-store --reason "Testing the approval rules here." --as ann | jq -r .run.id)
$A approve "$ID" --as ann
$A approve "$ID" --as carol

# Rejected push: the run ends "failed" (exit 1), never "succeeded"
printf '#!/bin/sh\necho "main is protected" >&2\nexit 1\n' > ../origins/beta.git/hooks/pre-receive
chmod +x ../origins/beta.git/hooks/pre-receive
$A approve "$ID" --as bob; echo "exit=$?"
$A status "$ID"
rm ../origins/beta.git/hooks/pre-receive
```

A rejected push only fails a run that needs a change. If `DEPRECATED.md` on `main` already has exactly the content the inputs would write, apply has nothing to push and reports `succeeded`, because the desired state is observed. To see a rejection, use inputs that differ from what is already there, as in the example above.

To run a repository's workflow directly with act, without the CLI, run this from a checkout:

```bash
cd /tmp/actions-ab/r2/gha-3/alpha
EV=$(mktemp)
printf '{"inputs":{"successor":"acme/x","reason":"Twenty or more characters here.","mode":"preview"}}' > "$EV"
../bin/act workflow_dispatch -W .github/workflows/mark-deprecated.yml -P ubuntu-latest=-self-hosted \
  -e "$EV" --var MD_ORIGIN_URL=/tmp/actions-ab/r2/gha-3/origins/alpha.git \
  --var MD_REPO_SLUG=acme/alpha --no-cache-server
```

## What could not run here, and how GitHub enforces it

| Feature | Here | On GitHub |
|---|---|---|
| Required reviewers and prevent-self-review (`environment: deprecation`) | act ignores environment protection. The CLI enforces the same rules from the same file, `deprecation.json`. | `apply-github-settings.sh` (via the `mark-deprecated settings` workflow, with an admin token in `MD_SETTINGS_TOKEN`) creates the environment with ann and bob as required reviewers, `prevent_self_review: true`, and a `main`-only branch policy. GitHub then pauses the `apply` job until a reviewer other than the person who triggered the run approves it in the UI or through `POST /repos/{owner}/{repo}/actions/runs/{run_id}/pending_deployments`. |
| Run history and approval history | Records in `org/.acme-actions/runs/` and the act logs | The Actions run history (who dispatched, inputs, logs) and the deployment review history (who approved, when) |
| Org-wide entry point | `acme-actions`, running act against each origin | `gh workflow run mark-deprecated.yml -R acme/<repo> -f successor=... -f reason=... -f mode=apply` from anywhere, plus `gh run view --json` for status. The catalog would list the repositories. |
| Authentication | None: `--as` is trusted. | GitHub identity. The dispatcher is `github.actor`, and the token behind the clone and push is `secrets.GITHUB_TOKEN` with `contents: write` on the `apply` job only. |
| Branch protection | Not configured. A pre-receive hook stands in for a rejected push. | If `main` gets a ruleset, give the push a bypass (a GitHub App token) or the push is rejected, and per R5 the run fails. |

`apply-github-settings.sh` was only run with `DRY_RUN=1`, which printed the expected `PUT /environments/deprecation` and `POST .../deployment-branch-policies` requests. The `actions/checkout` steps are skipped by act, which copies the checked-out definition in instead. Locally, someone with shell access could run act in `apply` mode directly and bypass the CLI's approval check. On GitHub, the environment gate closes that path.
