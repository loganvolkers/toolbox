# HANDOFF: the `add-notice` self-service action

`add-notice` is a GitHub Actions workflow that lives in each repository (`alpha`, `beta`,
`gamma`) and is owned by its maintainers. Each run validates the inputs and previews the
README diff. A release-group member other than the requester must approve before the
change is pushed to `main`, and the run counts as succeeded only once `main` is observed
with the right README.

There is no GitHub here, so the org-level CLI `org/bin/acme` plays GitHub's part:
- it dispatches the workflow's jobs with act in host mode;
- it holds the gated job until an eligible reviewer approves;
- it keeps the run history.

The GitHub-side enforcement (environment required reviewers, prevent self-review, team
membership, rulesets) is committed as code in `org/github/apply-settings.py`, which
`org/.github/workflows/apply-settings.yml` runs on GitHub.

The three requested runs all **succeeded**. Times are UTC, 2026-11-24.

| Run | Requested by | Approved by | Commit on `main` | Requested, approved, finished |
|---|---|---|---|---|
| `alpha-add-notice-1` (warning) | ann | dave | `7f82083` | 16:21:53, 16:21:59, 16:22:00 |
| `beta-add-notice-1` (info, `readme.md`) | ann | erin | `37fb2b7` | 16:21:53, 16:22:01, 16:22:01 |
| `gamma-add-notice-1` (info) | ann | erin | `9b4f6de` | 16:21:54, 16:22:01, 16:22:02 |

Each record is in `org/runs/<id>/record.json`. Each commit carries the trailers
`Requested-by`, `Approved-by` and `Run`.

Nothing needs to be running. There is no server: every command is a one-shot CLI call, and
no process was left running.

## Files

| Path | What it is |
|---|---|
| `<repo>/.github/workflows/add-notice.yml` (alpha, beta, gamma) | The action's definition. It is a `workflow_dispatch` workflow with inputs `level` (choice) and `message`, plus an optional `previewed_base`. Job `preview` validates the inputs and shows the diff. Job `apply` (`environment: release`) commits, pushes to `main` and verifies `main` |
| `<repo>/.github/add-notice/notice.py` | The tool the workflow runs. It is standalone: `validate`, `preview`, `apply` and `verify` work against any git checkout. It prints JSON and also writes GitHub step outputs and a step summary |
| `alpha/CODEOWNERS`, `beta/.github/CODEOWNERS`, `gamma/.github/CODEOWNERS` | `@ann @bob` own `/.github/workflows/add-notice.yml` and `/.github/add-notice/`. alpha already had a root `CODEOWNERS`, so the lines were appended there: GitHub reads only the first CODEOWNERS file it finds |
| `org/org.yml` | Org members, the `release` approval environment (reviewers: team `release`, `prevent_self_review: true`) and the repositories with their origins |
| `org/teams/release.yml` | **The only definition of the release group** (dave, erin) |
| `org/bin/acme` | The org CLI: `list`, `preview`, `request`, `approve`, `reject`, `cancel`, `status`, `record`, `runs` and `group` |
| `org/runs/<id>/record.json` | Run records, the local stand-in for GitHub's run history. The act logs and step results are in `org/runs/<id>/logs/` (gitignored) |
| `org/github/apply-settings.py` | Applies `org.yml` and `teams/` to GitHub through the REST API: team membership, each repository's `release` environment, workflow token permissions, and a ruleset on `main`. `--dry-run` prints the calls |
| `org/.github/workflows/apply-settings.yml` | On GitHub, runs `apply-settings.py` on every push to the org's `main` that touches `org.yml` or `teams/` |

## Requirements

| # | How it is met | Paths |
|---|---|---|
| R1 Owned by the repository | Each repository's workflow and tool are committed to its origin's `main` (commits `426dd13` alpha, `22bbdd2` beta, `7037953` gamma, by ann). CODEOWNERS names ann and bob for them. `bin/acme` reads the definition from each origin's `main` and never keeps a copy. A run uses the workflow at the commit it was requested on, as GitHub does (`definition_commit` in the record) | `<repo>/.github/workflows/add-notice.yml`, `<repo>/.github/add-notice/notice.py`, the CODEOWNERS files |
| R2 Validated | Validation happens twice before any change. First, `bin/acme` applies GitHub's dispatch checks: known inputs, required inputs, and `choice` options, which act itself does not check. Second, the `preview` job runs `notice.py`, which checks that `level` is `info` or `warning` and that `message` is one line (no CR, LF, U+2028/2029 or control characters), 10 to 160 characters, and not blank. An invalid request gets status `invalid`, exits with code 2, and never reaches `apply`. `apply` re-validates | `notice.py` `validate()`, `bin/acme` `dispatch_errors()` |
| R3 Previewed | `bin/acme preview` (a dry run, with no record) and `bin/acme request` run the workflow's `preview` job and return the unified diff, the README path, the base blob and the expected new blob. Approval pins `previewed_base` to the README blob the requester saw. If `main`'s README changed in between, `apply` refuses with "changed since it was previewed" and the run fails without pushing. On GitHub the diff is in the `preview` job's step summary and log | `add-notice.yml` job `preview`, `notice.py preview` |
| R4 Approved by the release group | `apply` is gated by `environment: release`. **On GitHub:** `apply-settings.py` sets that environment's required reviewers to team `release` with `prevent_self_review: true`, and GitHub enforces it. **Here:** `bin/acme approve` allows only members of the teams that `org.yml` lists for the environment (read from the org origin's `main` at approval time), refuses the requester, and records refused attempts as `approval_refused` events. The workflow's first `apply` step also re-checks this under act and fails closed, so running act directly cannot push without an eligible approver | `org.yml` `environments.release`, `bin/acme` `review()`, `add-notice.yml` step "Approval gate", `apply-settings.py` |
| R5 Done means observed | There are three checks, and all must pass. (1) The workflow's last step fetches `main` again and requires the README blob to equal the expected blob and the first line to be the notice. (2) `bin/acme` then reads the origin's `main` itself and compares the blob with the one the preview predicted (the `observed` event). (3) The workflow must have exited 0. A rejected push fails the job at "Push to main". The run is then `failed`, and its errors include git's `[remote rejected]` lines | `add-notice.yml` step "Verify main has the notice", `notice.py verify`, `bin/acme` `execute()` |
| R6 One place for the org | From `org/`: `bin/acme list` reads each origin's `main` and lists every workflow with `workflow_dispatch`, a `preview` job and an approval environment. For each one it shows the inputs, eligible approvers and a ready-made request command. `bin/acme request <repo> <action>` works for any listed repository | `bin/acme`, `org.yml` `repositories` |
| R7 Audited | `runs/<id>/record.json` holds `requested_by`/`requested_at`, `approved_by`/`approved_at`, `inputs`, `preview`, `outcome` (status, time, commit, observed blob) and `finished_at`. It also has a timestamped `events` list with an actor on each entry: `requested`, `previewed`, `waiting_for_approval`, `approval_refused`, `approved`/`rejected`, `started`, `observed`, `succeeded`/`failed`. Records are written atomically under a per-run lock. Each commit on `main` carries trailers. Group changes are commits by `--as` on the org origin | `org/runs/`, `bin/acme` |
| R8 Usable by agents | Every command is non-interactive, prints one JSON document, and uses exit codes: 0 ok, 1 run failed or error, 2 invalid input, 3 not allowed (identity, approval rule or run state). act is run with stdin closed and with the user's `.actrc`, `.env`, `.secrets` and `.vars` ignored | `bin/acme` |
| R9 The group is defined once | `org/teams/release.yml` is the only list of members, and `org.yml` maps environment `release` to it. Both `bin/acme approve` and the workflow's act gate read it from the org origin's `main` at approval time, so a change there applies to all three repositories at once. On GitHub, `apply-settings.yml` syncs the `acme/release` team from the same file, and every repository's environment names that team | `org/teams/release.yml`, `org/org.yml`, `org/github/apply-settings.py` |

## Commands

These need Python 3 with PyYAML (present here), git, and act at
`/tmp/actions-ab/r8/gha-3/bin/act` (override it with `ACME_ACT`). Run everything from the
org checkout. `--as NAME` says who you are; you can set `ACME_AS` instead. `jq` is used
below only to pull out a run id.

```bash
cd /tmp/actions-ab/r8/gha-3/org

# Start: nothing. There is no server or daemon.

# List add-notice (and any other self-service action) for all three repositories
bin/acme list
bin/acme list --repo beta

# Preview without creating a run (anyone in the org)
bin/acme preview alpha add-notice --as carol --input level=info --input "message=A preview of a notice, not applied."

# Request a run as ann: validates, previews, and waits for approval. Prints the run id and the diff.
ID=$(bin/acme request alpha add-notice --as ann --input level=info --input "message=Our docs moved to the wiki." | jq -r .run.id); echo "$ID"

# Approve as a release member (dave or erin). This runs apply: commit, push, verify.
bin/acme approve "$ID" --as dave --comment "ok"
# Or turn it down (a release member), or withdraw it (the requester)
bin/acme reject "$ID" --as erin --comment "not now"
bin/acme cancel "$ID" --as ann

# Status and the full record
bin/acme status "$ID"
bin/acme record "$ID"
bin/acme runs                       # every run; filter with --repo R or --status S
bin/acme record alpha-add-notice-1  # one of the three runs above

# Change who is in the release group (commits teams/release.yml to the org origin's main)
bin/acme group show release
bin/acme group add release carol --as ann
bin/acme group remove release erin --as ann
bin/acme group set release dave erin --as ann   # back to the original
# The same change by hand: edit teams/release.yml, then
git commit -am "teams/release: ..." && git push origin main

# What would be applied on GitHub (prints the REST calls)
python3 github/apply-settings.py --dry-run
```

The group change takes effect when it reaches the org origin's `main`; an edit that is not
pushed changes nothing. `group` also fast-forwards this checkout when it has no local edits.

### Test recipes

Each block below says what to expect. Run ids come from the request's JSON (`.run.id`).

```bash
cd /tmp/actions-ab/r8/gha-3/org

# Invalid inputs: each exits 2 with status "invalid" and the reason in .errors; main is unchanged.
bin/acme request alpha add-notice --as ann --input level=error --input "message=Long enough message"
bin/acme request alpha add-notice --as ann --input level=info --input message=short
bin/acme request alpha add-notice --as ann --input level=info --input "message=$(printf 'x%.0s' $(seq 161))"
bin/acme request alpha add-notice --as ann --input level=info --input "message=$(printf 'two\nlines here')"
bin/acme request alpha add-notice --as ann --input level=info --input "message=Long enough message" --input foo=bar
bin/acme request alpha add-notice --as mallory --input level=info --input "message=Long enough message"   # exit 3: not an org member

# Self-approval: exit 3 "dave requested this run and cannot approve it"; the run stays
# "waiting" and an approval_refused event is recorded.
ID=$(bin/acme request beta add-notice --as dave --input level=info --input "message=Requested by a release member." | jq -r .run.id)
bin/acme approve "$ID" --as dave
# Approvals from outside the release group: exit 3 "bob is not a reviewer for environment release" (recorded).
bin/acme approve "$ID" --as bob
bin/acme approve "$ID" --as ann
bin/acme approve "$ID" --as carol
# An eligible approver: exit 0, status "succeeded".
bin/acme approve "$ID" --as erin

# A change to the group: erin is refused in every repository until she is added back.
bin/acme group remove release erin --as ann
ID=$(bin/acme request gamma add-notice --as ann --input level=info --input "message=Testing a release group change." | jq -r .run.id)
bin/acme approve "$ID" --as erin     # exit 3: not a reviewer
bin/acme approve "$ID" --as dave     # exit 0: succeeded
bin/acme group add release erin --as ann

# A rejected push: exit 1, status "failed", never "succeeded". The errors include
# "! [remote rejected] HEAD -> main (pre-receive hook declined)", "step failed: Push to main",
# and the README blob observed on main next to the expected one.
H=/tmp/actions-ab/r8/gha-3/origins/gamma.git/hooks/pre-receive
printf '#!/bin/sh\necho "main is frozen" >&2\nexit 1\n' > "$H"; chmod +x "$H"
ID=$(bin/acme request gamma add-notice --as ann --input level=warning --input "message=This change must not reach main." | jq -r .run.id)
bin/acme approve "$ID" --as erin
bin/acme status "$ID"
rm "$H"

# main's README changed after the preview: status "failed", "changed since it was previewed
# ... request the run again", and nothing is pushed.
ID=$(bin/acme request alpha add-notice --as ann --input level=info --input "message=Previewed against the old README." | jq -r .run.id)
(cd ../alpha && git pull -q && echo "extra line" >> README.md && git commit -qam "edit README" && git push -q origin main)
bin/acme approve "$ID" --as dave

# Bypassing the CLI: running the workflow with act directly fails at the "Approval gate" step
# ("no approval recorded"), and nothing is pushed.
D=$(mktemp -d); git clone -q ../origins/alpha.git "$D/alpha" && cd "$D/alpha"
/tmp/actions-ab/r8/gha-3/bin/act workflow_dispatch -W .github/workflows/add-notice.yml -P ubuntu-latest=-self-hosted \
  --no-cache-server --action-cache-path "$D/act" -a ann --input level=info --input "message=Trying to skip approval."
cd - && rm -rf "$D"
```

## On GitHub

| Step | GitHub (`gh`, REST) | Here |
|---|---|---|
| List | `for r in alpha beta gamma; do gh workflow view add-notice.yml -R acme/$r --yaml; done` | `bin/acme list` |
| Request and preview | `gh workflow run add-notice.yml -R acme/alpha --ref main -f level=warning -f message='...'`, then `RUN=$(gh run list -R acme/alpha -w add-notice.yml -L 1 --json databaseId -q '.[0].databaseId')` and `gh run view $RUN -R acme/alpha --log` (the diff is also in the `preview` job's summary) | `bin/acme request`, which runs job `preview` with act |
| Wait for approval | The `apply` job waits on environment `release`, and GitHub notifies team `acme/release` | Status `waiting` |
| Approve (as dave, with his token) | `ENV=$(gh api repos/acme/alpha/actions/runs/$RUN/pending_deployments -q '.[0].environment.id'); gh api -X POST repos/acme/alpha/actions/runs/$RUN/pending_deployments -F "environment_ids[]=$ENV" -f state=approved -f comment=ok` | `bin/acme approve` |
| Refuse self-approval and outsiders | GitHub enforces the environment's `reviewers` (team `release`) and `prevent_self_review: true`, set by `apply-settings.py` | `bin/acme` `review()` plus the act-only gate step |
| Apply and verify | The same `apply` job continues in the same run. It reads the approver from `GET /repos/{o}/{r}/actions/runs/{id}/approvals`, pushes with `GITHUB_TOKEN` (`contents: write`), then verifies | `bin/acme` runs the workflow again with act, with `previewed_base` pinned and `ACME_APPROVED_BY` set |
| Status and record | `gh run view $RUN -R acme/alpha --json status,conclusion,jobs,createdAt,updatedAt` and `gh api repos/acme/alpha/actions/runs/$RUN/approvals` | `bin/acme status`, `bin/acme record` |
| Change the group | Push an edit to `teams/release.yml` on the org's `main`; `apply-settings.yml` syncs it with `PUT`/`DELETE /orgs/acme/teams/release/memberships/{user}` | `bin/acme group ...` |

For the three runs, GitHub would have done the following itself:
- created the run when ann dispatched it, and run `preview` on a hosted runner;
- held `apply` at the `release` environment, and checked that dave (alpha) and erin (beta, gamma) are in team `release` and are not the requester;
- resumed `apply` in the same run, and kept the run history and approvals.

Here, `bin/acme` did each of those, and the commits, pushes and verification ran in the
workflow under act.

For R6 on GitHub, the org-wide listing would use the commands above. `bin/acme` has no
GitHub backend; it lists and dispatches through the local origins.

## What I could not run here

- **No GitHub.** I could not run environments with required reviewers, prevent self-review, team membership, the `release` team's access to repositories, rulesets, CODEOWNERS review requests, the run history or step summaries. `bin/acme` and the act-only gate step stand in for the approval part. `apply-settings.py` was only run with `--dry-run`.
- **The teams endpoints** (`/orgs/{org}/teams...`) are not in the REST reference provided here. I used them from GitHub's teams API without checking them against this copy.
- **The `actions/checkout` pin** in `apply-settings.yml` (`11bd719…`, v4.2.2) was not checked online.
- **Workflow paths that only run on GitHub:**
  - the checkout with `GITHUB_TOKEN` through `http.extraheader`;
  - the `gh api .../approvals` lookup for the approver;
  - the "Approved through the environment" branch of the gate step.

  act takes the local-path branches instead (`github.repository` is the origin's path and `ACT=true`).
- **Rulesets and the run's push.** If `main` gets a ruleset that requires pull requests, the `apply` job's push needs a bypass actor (for example, a GitHub App token in place of `GITHUB_TOKEN`). The ruleset applied here only blocks force-pushes and deletion of `main`.
- **Checks run.** `yamllint` (alpha's CI config) and `actionlint` 1.7.12 pass on both workflows; shellcheck was not installed, so actionlint skipped shell checks. `notice.py` was exercised on edge cases: a replaced notice, CRLF, a BOM, a missing trailing newline, an empty README, lowercase `readme.md`, and invalid inputs.
- **Identity is claimed.** `--as` is trusted, as the sandbox allows. Anyone with write access to an origin, `runs/` or the org origin could get around the CLI. On GitHub, authentication, rulesets and the server-side run history close that gap.
