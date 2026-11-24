# HANDOFF: `mark-deprecated`

`mark-deprecated` is a GitHub Actions workflow that each repository owns. The org CLI
`org/bin/acme` runs it locally with act and supplies what act cannot: listing across
repositories, the approval gate, the run history and an independent check of `main`. Nothing
needs to be running: there is no server, every command runs and exits, and there are no
long-running processes.

All three runs requested by ann and approved by bob are reported `succeeded`, and each origin's
`main` now has the expected `DEPRECATED.md`:

| Repository | Run id | Commit on `main` |
|---|---|---|
| alpha | `alpha-20261124t131010-e4ad1b` | `9abb6d1` Mark repository deprecated: use acme/dependabot-proxy-v2 |
| beta | `beta-20261124t131011-942f7f` | `5e83c01` Mark repository deprecated: use acme/valtio-store |
| gamma | `gamma-20261124t131012-899a18` | `c9228d7` Mark repository deprecated: use acme/graphify-core |

Paths below are relative to `/tmp/actions-ab/r5/gha-1`. `<repo>` is `alpha`, `beta` or `gamma`.

## Layout

| Path | What it is |
|---|---|
| `<repo>/.github/workflows/mark-deprecated.yml` | The action. It has `workflow_dispatch` inputs `successor` and `reason`, plus an optional `run_id` used for correlation. Job `preview` validates the inputs and shows the diff. Job `apply` (`needs: preview`, `environment: deprecation`) validates again, commits, pushes and verifies |
| `<repo>/.github/mark-deprecated/mark_deprecated.py` | The action's logic (Python standard library only): `validate`, `preview`, `apply`, `verify`. It prints one machine-readable line per step, `MARK_DEPRECATED_<KIND>=<json>`, and writes `$GITHUB_STEP_SUMMARY` |
| `<repo>/.github/mark-deprecated/settings.json` | The approval policy: environment `deprecation`, reviewers `ann` and `bob`, `prevent_self_review: true`, deployments from `main` only. It also holds the ruleset on `main` |
| `<repo>/.github/mark-deprecated/apply-github-settings.sh` | Applies `settings.json` through the GitHub API (`gh api`). `--dry-run` prints the requests it would make |
| `alpha/CODEOWNERS`, `beta/.github/CODEOWNERS`, `gamma/.github/CODEOWNERS` | Make `@ann @bob` the code owners of the action, its policy and CODEOWNERS itself |
| `org/bin/acme` | The org CLI: `list`, `preview`, `request`, `approve`, `reject`, `status`, `record`, `runs` |
| `org/repos.json` | The org's repositories, with their GitHub names (`acme/<repo>`) and origins |
| `org/people.json` | Known people (ann, bob, carol). Who may approve is not set here; it comes from each repository's `settings.json` |
| `org/act/stubs/create-github-app-token/` | A local stand-in for a `uses:` step that the workflow skips under act, so act never fetches it from github.com |
| `org/.state/runs/<id>.json` | Run records (the audit trail). Not committed |
| `org/.state/runs/<id>/` | Each run's act logs (`preview.log`, `apply.log`), event payloads and the exact workflow file used |

## How each requirement is met

| # | How |
|---|---|
| R1 Owned by the repository | Each repository's definition is committed on its origin's `main`. It consists of `.github/workflows/mark-deprecated.yml`, `.github/mark-deprecated/{mark_deprecated.py,settings.json,apply-github-settings.sh}` and the CODEOWNERS lines. `acme` reads these from `origins/<repo>.git` `main` on every call and keeps no copy. A change a maintainer pushes, to the inputs, the validation or the reviewers, applies to the next `list`, `preview`, `request` or `approve`. Each run records the `definition_sha` it used, and `apply` runs the workflow from that same commit, as GitHub does for a dispatch. On GitHub, CODEOWNERS plus the ruleset's code-owner review mean only ann or bob can change these files |
| R2 Validated | `mark_deprecated.py validate` runs as the first step of both jobs. It checks that `successor` matches `^[a-z0-9-]+/[a-z0-9-]+$` and is not the repository itself, and that `reason` is 20 to 200 characters (Unicode code points) on one line, with no control characters. `acme preview` and `acme request` run the `preview` job. If validation fails, `request` records the run as `refused`, with the `validation_errors`, and exits 2. No run can be approved from that state. Inputs reach the script through `env:`, never through `${{ }}` interpolated into a shell, so they cannot inject commands |
| R3 Previewed | The `preview` job (`mark_deprecated.py preview`) fetches `origin/main` and reports the change: the base commit, the operation (`create`, `update` or `none`), the new content and a unified diff. `acme preview` returns it without creating a run. `acme request` stores it in the record (`.preview`) before anyone approves |
| R4 Approved | On GitHub, the `apply` job uses `environment: deprecation`. `apply-github-settings.sh` configures that environment with required reviewers ann and bob and `prevent_self_review: true`, so the job waits until a reviewer other than the actor who dispatched it approves. Locally, `acme request` runs only `-j preview`. `acme approve` checks the approver against the reviewers in the repository's `settings.json` on `main`, refuses the requester (always) and anyone not listed (carol), records each refused attempt as an `approval_denied` event, and exits 3. Only then does it run `-j apply`. `acme reject` lets a reviewer close a run without applying it |
| R5 Done means observed | The workflow's `verify` step fetches `origin/main` and fails unless `DEPRECATED.md` is byte for byte `# Deprecated: use <successor>\n\n<reason>\n`. After act exits, `acme` reads `DEPRECATED.md` itself from `origins/<repo>.git` `main` and records it in `.observed` (time, `main_sha`, content, `matches`). A run is `succeeded` only if the apply job exited 0, the workflow's verify step matched and `acme`'s own observation matched. Otherwise it is `failed`, with the reason in `.failure`. A rejected push fails the `apply` step, and the git output is kept in `.apply.push` |
| R6 One place for the org | From `org/`, `bin/acme list` shows `mark-deprecated` for every repository in `repos.json`, with its inputs, definition commit and approvers. `request`, `preview`, `approve`, `status`, `record` and `runs` take `--repo <name>` or a run id. Nobody needs to enter a repository checkout |
| R7 Audited | `org/.state/runs/<id>.json` has `requested_by`/`requested_at`, `approved_by`/`approved_at` (or `rejected_by`/`rejected_at`), `inputs`, `outcome`/`outcome_at`, the preview, the push result, the observation and an append-only `events` list with a timestamp for each step, including denied approvals. The commit on `main` carries `Mark-Deprecated-Run:`, `Requested-by:` and `Approved-by:` trailers. On GitHub, the run history, the environment's deployment reviews and the commit hold the same facts |
| R8 Usable by agents | Every `acme` command is non-interactive and prints one JSON document on stdout, usage errors included. Exit codes: 0 ok; 1 the run failed; 2 invalid inputs or usage; 3 not allowed; 4 unknown repository, person or run, or a run in the wrong state (for example, approving a run twice). Identity comes from `--as NAME` or `ACME_AS=NAME`. `approve` is synchronous and returns the final record |

## Commands

Requirements: `git`, `python3` with PyYAML (both installed here), act at `bin/act`. Run the
commands from `org/`:

```bash
cd /tmp/actions-ab/r5/gha-1/org
```

Nothing has to be started first. To list the actions:

```bash
bin/acme list
```

To preview a run (this changes nothing):

```bash
bin/acme preview --repo alpha --successor acme/dependabot-proxy-v2 \
  --reason "Replaced by the v2 proxy, which supports OIDC registries." --as ann
```

To request a run as ann. It prints the record, which has status `awaiting_approval` and includes
`.preview`:

```bash
ID=$(bin/acme request --repo alpha --successor acme/dependabot-proxy-v2 \
  --reason "Replaced by the v2 proxy, which supports OIDC registries." --as ann | jq -r .id)
echo "$ID"
```

To approve the run as bob. This applies the change, observes `main` and prints the final record.
It exits 0 if the run succeeded and 1 if it failed:

```bash
bin/acme approve "$ID" --as bob
```

To check the run's status:

```bash
bin/acme status "$ID"
```

To read the full record:

```bash
bin/acme record "$ID"
```

To list runs, or close a run without applying it:

```bash
bin/acme runs [--repo alpha] [--status succeeded]
bin/acme reject "$ID" --as bob --comment "not now"
```

### Negative tests

```bash
# Invalid inputs: exit 2, record status "refused", nothing pushed
bin/acme request --repo beta --successor Acme/Store --reason "too short" --as ann
bin/acme request --repo beta --successor acme/beta --reason "Pointing at itself is not allowed." --as ann

# Self-approval and carol: exit 3, the run stays awaiting_approval, the attempt is recorded
ID=$(bin/acme request --repo beta --successor acme/new-store \
  --reason "Testing the approval rules end to end." --as ann | jq -r .id)
bin/acme approve "$ID" --as ann
bin/acme approve "$ID" --as carol
bin/acme record "$ID" | jq '.status, [.events[] | select(.event == "approval_denied")]'

# Rejected push: exit 1, status "failed", main unchanged
printf '#!/bin/sh\necho "push rejected by policy" >&2\nexit 1\n' > ../origins/gamma.git/hooks/pre-receive
chmod +x ../origins/gamma.git/hooks/pre-receive
ID=$(bin/acme request --repo gamma --successor acme/graphify-next \
  --reason "Testing a push that the code host rejects." --as ann | jq -r .id)
bin/acme approve "$ID" --as bob | jq '{status, failure, push: .apply.push.push_output, observed: .observed.matches}'
rm ../origins/gamma.git/hooks/pre-receive
```

For the rejected-push test, use inputs that differ from what `main` already has. If
`DEPRECATED.md` on `main` already has exactly the requested content, nothing is pushed
(`.apply.push.pushed` is `false`), and the run is `succeeded` because the requested state is
observed on `main`. For that reason, the preview reports `operation: "none"` before approval.

If `main` moves between checkout and push, the push is rejected as non-fast-forward. `apply`
then rebuilds the commit on the new tip and pushes once more. Any other rejection fails the run.

## What GitHub would enforce, and what could not run here

| Feature | As code | How GitHub enforces it | Here |
|---|---|---|---|
| Environment `deprecation` with required reviewers ann and bob | `settings.json`, applied by `apply-github-settings.sh` (`PUT /repos/{repo}/environments/deprecation`) | The `apply` job waits for a reviewer to approve the deployment | act ignores environment protection. `acme approve` enforces the same list from the same file |
| Prevent self-review | `prevent_self_review: true` in the same call | The user who triggered the run cannot approve it | Enforced by `acme approve` |
| Deployments only from `main` | `deployment_branches` → `POST .../deployment-branch-policies` | Runs from other branches cannot use the environment | Locally, runs are always dispatched on `main` |
| Ruleset on `main`: no deletion, no force push, PRs with code-owner review, bypass only for the `acme-deprecation-bot` GitHub App | `settings.json` `ruleset`, applied by `POST/PUT /repos/{repo}/rulesets` | Only the App token from `actions/create-github-app-token` (used only in the approved `apply` job) can push to `main` directly. Changes to the action need ann's or bob's review | Not enforced: the local origins accept any push. A `pre-receive` hook stands in for a rejection in the test above |
| CODEOWNERS for the action and its policy | `CODEOWNERS` / `.github/CODEOWNERS` | Through the ruleset's `require_code_owner_review` | Not enforced locally |
| Run history | Built in | The Actions run, its logs, step summary and deployment reviews | `org/.state/runs/` holds the records and act logs |
| Dispatch | `gh workflow run mark-deprecated.yml -R acme/<repo> -f successor=... -f reason=...` | | `acme request` runs `act workflow_dispatch -j preview` |

Not run here:
- `apply-github-settings.sh` was only run with `--dry-run`, because there is no GitHub.
- `actions/create-github-app-token` is skipped under act (`if: ${{ !env.ACT }}`); `acme` maps it to `org/act/stubs/`.
- `$GITHUB_STEP_SUMMARY` is written under act, but act discards it.
- The `concurrency:` group is not enforced by act. `acme` takes a lock per repository around `apply`.

How act is run: `acme` clones the origin's `main` into `org/.state/work/`, adds a `github` remote
`https://github.com/acme/<repo>.git` (used only so that `github.repository` is `acme/<repo>`, via
`--remote-name github`), writes the workflow from the run's `definition_sha` and a
`workflow_dispatch` event, and runs:

```bash
bin/act workflow_dispatch -C <clone> -W <workflow.yml> -e <event.json> -j preview|apply \
  -P ubuntu-latest=-self-hosted -a <requester> --remote-name github \
  --local-repository actions/create-github-app-token@v1=org/act/stubs/create-github-app-token
```

`actions/checkout` becomes act's local copy, whose `origin` is `origins/<repo>.git`, so the push
in `apply` goes to the local code host. act's caches go to `org/.state/`.
