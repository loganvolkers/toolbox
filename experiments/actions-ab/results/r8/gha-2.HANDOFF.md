# HANDOFF: `add-notice` on GitHub Actions (run locally with act)

**Status.** `add-notice` is defined in `alpha`, `beta` and `gamma`, committed to each origin's
`main`. It ran once on each repository. ann requested all three, dave approved alpha's run and
erin approved beta's and gamma's. All three are reported `succeeded`, and only after the README
was observed on the origin's `main`:

| Run | level | Approved by | Commit on `main` (observed) | Requested → approved → succeeded (UTC, 2026-11-24) |
|---|---|---|---|---|
| `alpha-1` | warning | dave | `71d7796307d3` | 16:06:34.405 → 16:06:40.280 → 16:06:41.534 |
| `beta-1` | info | erin | `01121d6149a7` | 16:06:39.518 → 16:06:41.592 → 16:06:42.121 |
| `gamma-1` | info | erin | `adad6b6aabf2` | 16:06:39.914 → 16:06:42.175 → 16:06:42.655 |

Each commit changes only the README (2 lines added). It carries `Requested-by:`, `Approved-by:` and
`Run:` trailers. beta's README is `readme.md`, and the action found it.

**How it fits together.** The action is an ordinary GitHub Actions workflow in each repository.
GitHub would gate it with a `release` environment. act cannot hold a job for approval, so the org
CLI `org/bin/acme` plays GitHub's part here. It runs the jobs before the gated job, which validate
and preview. It records the preview and enforces the environment's reviewers and
`prevent_self_review` from `org/org.yml`. Then it runs the whole workflow and observes `main`
before it reports success. Nothing is long-running: there is no server, and every command is a
one-shot CLI call that prints JSON.

## Commands

You need git, Python 3 with PyYAML (installed), `jq` for the examples (installed), and act at
`/tmp/actions-ab/r8/gha-2/bin/act` (`ACME_ACT` overrides that path). **Nothing needs to be
started.** Run everything from the org directory:

```bash
cd /tmp/actions-ab/r8/gha-2/org
```

| Step | Command |
|---|---|
| List actions (all repositories) | `bin/acme actions` (filters: `--repo alpha`, `--action add-notice`) |
| Preview only (no approvable run) | `bin/acme preview alpha add-notice --as carol -f level=info -f "message=Docs moved to the wiki this week."` |
| Request (as ann) | `bin/acme request alpha add-notice --as ann -f level=warning -f "message=This proxy is frozen while the v2 rewrite lands."` |
| Approve (as dave or erin) | `bin/acme approve alpha-2 --as dave --comment "ok"` |
| Reject (release only) | `bin/acme reject alpha-2 --as erin --comment "not now"` |
| Cancel (requester or maintainer) | `bin/acme cancel alpha-2 --as ann` |
| Status of a run | `bin/acme status alpha-2` |
| Full record and audit trail | `bin/acme record alpha-2` |
| List runs | `bin/acme runs` (filters: `--repo beta`, `--status waiting`) |
| Show the release group | `bin/acme group show release` |
| Change the release group | `bin/acme group set release --members dave,carol --as ann` (or `group add release carol --as ann`, `group remove release erin --as ann`) |
| Restore it | `bin/acme group set release --members dave,erin --as ann` |
| Whole test suite in a sandbox | `tests/acceptance.sh` (60 checks on bare copies of the origins; real repositories untouched) |

How the commands behave:
- Run ids are `<repository>-<n>`; `request` prints the new id in `.run.id`. The runs that already
  exist are `alpha-1`, `beta-1` and `gamma-1`, so your first new run on alpha is `alpha-2`.
- `--as` can be replaced by `export ACME_AS=ann`.
- Exit codes:

  | Code | Meaning |
  |---|---|
  | 0 | ok |
  | 1 | the run failed |
  | 2 | invalid input or usage |
  | 3 | not permitted |
  | 4 | not found |
  | 5 | wrong state, for example approving a run that is not waiting |

- Errors are JSON as well: `{"ok": false, "error": "<code>", "message": ...}`.
- `request` and `preview` return after the preview; `approve` returns when the run has finished.
  Both take about a second.
- Manual alternative for changing the group: edit `teams.release.members` in `org/org.yml`, then
  `git commit -am "release: ..." && git push origin main`. Only what is on `main` of
  `origins/org.git` counts, so an edit that is not pushed has no effect.

### Testing notes (what to expect)

| Test | How | Expected |
|---|---|---|
| Invalid level | `-f level=error` | exit 2, `invalid_inputs`, **no run created** (GitHub's dispatch API refuses choices the same way) |
| Message length or lines | `message=` of 9 or 161 characters, a two-line message, or a blank one | exit 2. The run is recorded as `refused` and can never be approved. 10 and 160 characters are accepted (lengths count Unicode characters) |
| Unknown or missing input | `-f colour=red`, or no `message` | exit 2, no run |
| Self-approval | a release member requests and then approves their own run, e.g. `request ... --as dave` then `approve <id> --as dave` | exit 3, `self_review`; recorded as `approve_denied` |
| ann approving her own run | `approve <id> --as ann` | exit 3, `not_a_reviewer`. ann is not in the release group, which is checked first, so a self-approval by ann reports that reason instead of `self_review` |
| Outsider approves | `--as carol`, `--as bob`, or a non-member such as `--as zed` | exit 3 (`not_a_reviewer` / `unknown_person`), run still `waiting`, `main` unchanged |
| Group change | `group set release --members dave,carol --as ann`, then `approve <id> --as erin` and `--as carol` | erin refused, carol allowed. Applies to pending runs too, because reviewers are checked at approval time. Only org owners (ann, bob) may change it |
| Rejected push | `printf '#!/bin/sh\nexit 1\n' > ../origins/gamma.git/hooks/pre-receive && chmod +x ../origins/gamma.git/hooks/pre-receive`, then request + approve, then `rm ../origins/gamma.git/hooks/pre-receive` | exit 1; status `failed`; reason `push to main was rejected: ... (pre-receive hook declined)`; `main` unchanged |
| README changed after preview | push another README change between request and approve | exit 1; `failed`: "changed since the approved preview; request a new run" |
| Same notice again | request the exact notice already on `main` | succeeds without a commit (`pushed: "false"`), because the README observed on `main` is already right |

## How each requirement is met

| # | How | Files |
|---|---|---|
| R1 Owned by the repository | Each repository defines the action on its own `main`: the workflow, its script and its unit tests. CODEOWNERS gives both files to `@acme/maintainers`. `acme` discovers actions from each origin's `main` and runs the workflow at the commit it was requested on, as GitHub does. Changing the action in the repository (for example the length limits in `notice.py`) changes what runs | `<repo>/.github/workflows/add-notice.yml`, `<repo>/.github/add-notice/notice.py`, `<repo>/.github/add-notice/test_notice.py`, `alpha/CODEOWNERS`, `beta/.github/CODEOWNERS`, `gamma/.github/CODEOWNERS` |
| R2 Validated | Two layers check inputs before anything can change. `acme` first applies GitHub's dispatch checks (known inputs, required, choice options, booleans) and refuses without creating a run. Then the workflow's first step (`notice.py validate`) checks one line, no control characters, not blank, and 10 to 160 characters; a failure there ends the run as `refused` before any job that writes | `bin/acme` (`resolve_inputs`), `notice.py` (`validate_inputs`) |
| R3 Previewed | The `preview` job computes the exact change against current `main` and writes a unified diff to the step summary. `request` and `preview` return it as `preview_diff`, and `record` shows the full summary and outputs. The preview pins the README blob, and the apply step refuses to write if the README changed since the approved preview | `add-notice.yml` (job `preview`, input `approved_base_blob`), `notice.py preview` |
| R4 Approved by the release group | The job that writes, `apply`, uses `environment: release`. `org.yml` declares `release` with `reviewers: [team:release]` and `prevent_self_review: true`. `acme approve` enforces exactly that, with reviewers expanded from the team at approval time. Requesters and non-members are refused, and every refusal is recorded. On GitHub, `github/apply-settings.py` creates that environment on every repository, and GitHub enforces it (see the GitHub section) | `org/org.yml` (`environments.release`), `bin/acme` (`cmd_review`), `github/apply-settings.py` |
| R5 Done means observed | The `verify` job fetches `main` and checks the README's SHA-256 against the preview's expected bytes. Then `acme` itself reads the README from the origin's `main` and compares it again. A run is `succeeded` only if every job succeeded **and** that observation matches; the observed commit and hash are in the record. A rejected push fails `apply`, so `verify` is skipped and the run is `failed` | `add-notice.yml` (job `verify`), `notice.py verify` and `apply`, `bin/acme` (`observe`) |
| R6 One place for the org | `bin/acme` in the org repository lists every repository's actions and requests, approves and reads runs on any of them. It reads repositories from `org.yml` and actions from each origin's `main` | `org/bin/acme`, `org/org.yml` (`repositories`) |
| R7 Audited | Each run record stores the requester and time, the inputs, the preview with its time, and the approval: who, when, comment, eligible reviewers, and the `org.yml` commit used. It also stores the outcome with its time and reason, the pushed and observed commits, and an event list that includes denied approvals and cancellations. Every event is also appended to the org-wide `audit.jsonl`, team changes included. Raw act logs are kept per phase | `org/state/runs/<id>.json`, `org/state/audit.jsonl`, `org/state/logs/<id>/{preview,apply}.log` (gitignored local stand-in for GitHub's run history) |
| R8 Usable by agents | Every command is non-interactive, identity is a flag (`--as`), inputs are flags (`-f k=v`), and output is JSON on stdout, usage errors included. Exit codes are distinct | `org/bin/acme` |
| R9 The group is defined once | `teams.release.members` in `org/org.yml` is the only list of approvers. Each repository's workflow names only the environment, and the environment names only the team. `acme` reads the list from `main` of `origins/org.git` on every approval, and `github/apply-settings.py` (run by the org's `apply-settings` workflow on push) syncs it to the GitHub team | `org/org.yml`, `org/.github/workflows/apply-settings.yml`, `org/github/apply-settings.py`, `org/.github/CODEOWNERS` |

## How GitHub would enforce what act cannot

What act does not run is configured as code, and `github/apply-settings.py` applies it through
the REST API. Without `--apply` it prints the 37 planned requests and calls nothing; I ran that
plan here, including as the org workflow's `plan` job under act.

- **The release group.** The GitHub team `acme/release` is synced from `org.yml`
  (`PUT`/`DELETE /orgs/acme/teams/release/memberships/{user}`). The `apply-settings` workflow
  runs on every push to `main` that touches `org.yml`, using the `ORG_ADMIN_TOKEN` secret of an
  `org-admin` environment. CODEOWNERS makes changes to `org.yml` need an org owner.
- **Required reviewers and self-review.** The script creates
  `PUT /repos/acme/{repo}/environments/release` with
  `reviewers: [{type: Team, id: <release team id>}]`, `prevent_self_review: true`, and a deployment
  branch policy of `main` only. GitHub then holds `apply` until one reviewer approves, rejects
  approval by the run's initiator (`github.actor`, here ann), and rejects anyone outside the team.
  Note: on Free, Pro and Team plans, required reviewers only work on public repositories.
- **Only an approved run can write `main`.** The ruleset `main-protection` covers the default
  branch: no deletion, no force-push, and changes only through pull requests with code-owner
  review. Its only bypass actor is the GitHub App `acme-release-bot`. That App's private key is a
  secret of the `release` environment, and GitHub releases environment secrets to a job only after
  approval. A workflow, or a maintainer, that drops `environment: release` therefore loses the
  push credential, and the ruleset refuses the push. The `bot` step mints the App token. It is
  skipped under act (`if: !env.ACT`), and `acme` maps that action to a local stub
  (`org/act/stubs/create-github-app-token`) so act never fetches it.
- **Run history.** GitHub keeps the run, its jobs, logs, step summary, and the review history
  (`GET /repos/acme/{repo}/actions/runs/{id}/approvals`). That history replaces `org/state/`.
- **Requester and approver.** The requester is `github.actor`. The approver comes from the
  approvals API in the `Who approved` step, and both go into the commit trailers.

One-time GitHub setup, which I did not run:
1. Create and install the `acme-release-bot` App with Contents read & write.
2. Store `RELEASE_BOT_APP_ID` as a variable of the `release` environment.
3. Run `python3 github/apply-settings.py --apply --release-bot-key key.pem` with an org-owner
   `GH_TOKEN`, or let the org workflow run it with `ORG_ADMIN_TOKEN`.

GitHub commands for each step (`<id>` is the GitHub run id):

| Step | GitHub |
|---|---|
| List | `gh workflow list -R acme/alpha` / `gh workflow view add-notice.yml -R acme/alpha` |
| Preview only | `gh workflow run add-notice.yml -R acme/alpha -f level=info -f message="..." -f preview_only=true`, then `gh run view <id> -R acme/alpha` (the step summary has the diff) |
| Request (ann's credentials) | `gh workflow run add-notice.yml -R acme/alpha -f level=warning -f message="..."` (or `POST /repos/acme/alpha/actions/workflows/add-notice.yml/dispatches` with `return_run_details: true`) |
| Pending approval | `gh api repos/acme/alpha/actions/runs/<id>/pending_deployments` |
| Approve (dave's credentials) | `gh api -X POST repos/acme/alpha/actions/runs/<id>/pending_deployments -F 'environment_ids[]=<release env id>' -f state=approved -f comment=ok` |
| Self-approval / outsider | the same call with ann's or carol's credentials is refused by GitHub |
| Reject | the same call with `-f state=rejected` |
| Status | `gh run view <id> -R acme/alpha --json status,conclusion,jobs` |
| Record | `gh api repos/acme/alpha/actions/runs/<id>`, `.../approvals`, `gh run view <id> --log` |
| Change the group | edit `org.yml` via PR → merge → `apply-settings` workflow; or `gh api -X PUT orgs/acme/teams/release/memberships/carol` |

Each run record also lists the equivalent `gh` commands under `.record.github`.

**For the three runs, GitHub would have performed these steps:**
- the dispatch, run as ann;
- holding `apply` at the `release` environment;
- checking that dave (alpha) and erin (beta, gamma) are in `acme/release` and are not the
  initiator, and recording their reviews;
- releasing the App key to `apply`;
- letting the push through the ruleset by App bypass;
- keeping the run history.

Here `bin/acme` did the holding, the reviewer checks and the recording. act ran the `preview`,
`apply` and `verify` jobs, and the push went straight to the local bare origins.

## What I could not run here

- **GitHub enforcement.** Nothing GitHub-side was run: environments, required reviewers and
  `prevent_self_review` as enforced by GitHub, teams, rulesets, the App token, environment
  secrets, the approvals API, run history, `gh workflow run`, and `apply-settings.py --apply`.
  Their configuration is committed, and the API plan was generated and checked against the
  bundled REST reference. The teams endpoints are not in that bundle; I used them from GitHub's
  teams API as I know it, so treat them as unverified here.
- **Identity.** It is only claimed with `--as`, because the sandbox has no authentication.
  Anyone with shell access could also edit `org/state/` or push to an origin directly. On GitHub,
  identities are authenticated, the ruleset blocks direct pushes, and the run history is not
  editable.
- **act's limits.** act ran in host mode with `actions/checkout` skipped: act copies the clone
  instead, and `github.repository` is the origin path. `acme` makes a fresh clone of the origin
  at the requested commit for each phase, so act runs exactly what is on `main`.

## Files

```
org/org.yml                               people, teams (release group), repositories, environments, ruleset
org/bin/acme                              the org CLI (Python 3 + PyYAML)
org/act/stubs/create-github-app-token/    offline stub for the GitHub-only token step
org/github/apply-settings.py              applies org.yml to GitHub (plan by default)
org/.github/workflows/apply-settings.yml  runs it on push to main
org/.github/CODEOWNERS                    org owners review org.yml
org/tests/acceptance.sh                   60 end-to-end checks in a sandbox
org/state/                                local run history, audit log, act logs (gitignored)
<repo>/.github/workflows/add-notice.yml   the action (alpha, beta, gamma)
<repo>/.github/add-notice/notice.py       its logic; test_notice.py its unit tests
```
