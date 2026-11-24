# Handoff: `mark-deprecated`, a repository-owned self-service action on GitHub Actions

**Status.** `mark-deprecated` is defined in `alpha`, `beta` and `gamma`, each on its origin's
`main`. It ran once on each repository: ann requested every run, bob approved it, and each run was
reported `succeeded` only after `DEPRECATED.md` with the exact content was observed on the origin's
`main`.

| Repository | Run id | Commit on `main` | Status |
|---|---|---|---|
| alpha | `alpha-20261124T140343Z-0e6f68` | `9e4aa29` | succeeded |
| beta | `beta-20261124T140348Z-2563ed` | `a05d317` | succeeded |
| gamma | `gamma-20261124T140351Z-69fe3a` | `8afe0eb` | succeeded |

**Nothing has to be running.** There is no server. Every step is a single command that exits.
You need `python3` (3.8+), `git`, `jq` (only for the examples and the self-test) and act at
`/tmp/actions-ab/r6/gha-3/bin/act`. No Docker is needed: act runs in host mode
(`-P ubuntu-latest=-self-hosted`).

## How it fits together

Each repository owns the action: a GitHub Actions workflow, its manifest and its step script, all
under code-owner review. GitHub has no org-wide catalogue or approval queue for `workflow_dispatch`,
and act cannot pause at an environment gate. That glue is `org/bin/acme`, a single-file CLI:

1. It **discovers** actions by reading `.github/self-service/*/action.json` from each origin's `main`
   (repositories are listed in `org/acme.json`).
2. To **request** a run, it pins the definition commit, clones the origin at that commit and runs the
   repository's workflow with act up to the `preview` job (`validate` → `preview`). It then records
   the run as `awaiting_approval`, together with the preview.
3. To **approve** a run, it applies the rules of the workflow's `deprecation` environment, as GitHub
   would. The reviewers are the ones in the repository's `action.json` on `main` at approval time,
   and self-review is prevented. It then runs the same pinned workflow with act up to `verify`
   (`validate` → `preview` → `apply` → `verify`).
4. **Done means observed.** After act finishes, `acme` reads the origin's `main` itself. The run is
   `succeeded` only if all of these hold: the workflow exited 0; the `apply` job pushed; the
   `verify` job observed the file; and `acme`'s own read of the origin finds `DEPRECATED.md` with
   the expected content and this run's commit (trailer `Acme-Run-Id: <id>`) on `main`. Otherwise
   the run is `failed`.

```
alpha/ (same files in beta/ and gamma/)
  .github/workflows/mark-deprecated.yml              the action: validate -> preview -> apply (environment: deprecation) -> verify
  .github/self-service/mark-deprecated/action.json   manifest: inputs and their rules, target file, approval rules
  .github/self-service/mark-deprecated/mark_deprecated.py   the four steps; JSON out, step summary, job outputs
  .github/self-service/mark-deprecated/apply-settings.py    GitHub settings as code: environment reviewers, prevent self-review, main-only, ruleset
  .github/workflows/mark-deprecated-settings.yml     runs apply-settings.py on every change to the above on main
  CODEOWNERS (alpha, root) / .github/CODEOWNERS (beta, gamma)   @ann @bob own all of the above
org/
  acme.json                 repositories (origins), people, act settings
  bin/acme                  the org CLI (list, preview, request, approve, reject, status, record, runs)
  bin/acme-selftest         every rule exercised against throwaway copies of the origins
  state/                    (git-ignored) run records, logs, audit log
    runs/<run-id>/record.json      the run's record
    runs/<run-id>/preview.log, apply.log    act's full output
    runs/<run-id>/results/<phase>/{validate,preview,apply,verify}.json   what each job reported
    audit.jsonl             append-only: one line per event, across all runs
```

## Requirements

| # | How it is met | Files |
|---|---|---|
| R1 Owned by the repository | The workflow, manifest and step script are committed to each repository's origin `main` (alpha `700e934`, beta `a63de13`, gamma `51f79de`). CODEOWNERS makes ann and bob the owners, so any change goes through their review. `acme` reads the definition from the origin's `main` and holds no copy of it, so a change a team merges (to inputs, reviewers, template or steps) takes effect on the next request. Each run pins the definition commit it was requested at (`definition.sha` in the record), as GitHub runs a dispatched workflow at the dispatch commit. | `<repo>/.github/workflows/mark-deprecated.yml`, `<repo>/.github/self-service/mark-deprecated/*`, `alpha/CODEOWNERS`, `{beta,gamma}/.github/CODEOWNERS` |
| R2 Validated | The workflow's first job, `validate`, checks the inputs against `action.json`: `successor` must fully match `^[a-z0-9-]+/[a-z0-9-]+$` and must not be this repository; `reason` must be one line of 20 to 200 characters; both are required. `preview`, and later `apply`, depend on it and validate again. Unknown input names are refused by `acme` before act runs. A refused request is recorded with status `refused` and its errors, `acme` exits 2, and nothing has been cloned to or pushed to the origin. Inputs reach the shell only through `env:`, never through `${{ }}` in `run:`, so they cannot inject commands. | `mark_deprecated.py` (`validate_values`), `action.json` (`inputs`), `mark-deprecated.yml` (`env:`) |
| R3 Previewed | The `preview` job fetches the origin's `main` and reports, for `DEPRECATED.md`, whether it would be created or updated (or left unchanged), its content before and after, and a unified diff. `acme preview` shows this without creating a run. `acme request` shows it and stores it in the record before anyone can approve. On GitHub, the same preview is written to the run's job summary before the run waits at the environment. If `DEPRECATED.md` on `main` changes between the preview and the approval, `apply` refuses (the before-hash no longer matches), so what was approved is what lands. | `mark_deprecated.py` (`cmd_preview`, check in `cmd_apply`), `org/bin/acme` (`cmd_preview`, `cmd_request`) |
| R4 Approved | `apply` runs in the environment `deprecation`. `action.json` names its reviewers (`ann`, `bob`) and sets `preventSelfReview: true`. `acme approve` enforces exactly that: the approver must be a reviewer listed on the repository's `main`, and must not be the requester. Otherwise it exits 3 and records an `approval_denied` event; the run keeps waiting and nothing changes. carol is not a reviewer, so she can never approve. Only `awaiting_approval` runs can be approved, so a run cannot be approved twice. On GitHub, `apply-settings.py` (run by `mark-deprecated-settings.yml`) configures the same environment, plus a ruleset that blocks direct pushes to `main`; see *How GitHub enforces it*. | `action.json` (`approval`), `mark-deprecated.yml` (`environment: deprecation`), `org/bin/acme` (`cmd_approve`), `apply-settings.py`, `mark-deprecated-settings.yml` |
| R5 Done means observed | `apply` commits onto a fresh fetch of `main`, with the trailers `Acme-Run-Id`, `Requested-by` and `Approved-by`, and pushes. A rejected push fails the job and the error is recorded. Then `verify` fetches `main` again and requires both the exact content and this run's commit. Finally, `acme` reads the origin itself (`observe()`, recorded as `outcome.observed`). `succeeded` requires all of these; anything else is `failed` with `outcome.error`, and `acme` exits 4. Every run pushes its own commit, even when the file already has that content, so a rejected push can never pass as a no-op. | `mark_deprecated.py` (`cmd_apply`, `cmd_verify`), `org/bin/acme` (`observe`, `cmd_approve`) |
| R6 One place for the org | From `org/`, `bin/acme list` lists the actions of all three repositories (inputs, rules, reviewers, definition commit), read from their origins. `bin/acme request <repo> mark-deprecated ...` works on any of them, and so do approve, status and record. Adding a repository is one entry in `acme.json`. | `org/bin/acme`, `org/acme.json` |
| R7 Audited | `state/runs/<id>/record.json` holds `requested_by`/`requested_at`, `approved_by`/`approved_at`, `inputs`, `preview`, `outcome` (with `at`), `finished_at`, the pinned definition, the paths of the logs, and `events[]`. Each event has `at`, `event` and `actor`: `requested`, `previewed`, `approval_denied`, `approved`, `started`, `succeeded`/`failed`/`refused`/`rejected`/`cancelled`. The same events are appended to `state/audit.jsonl`. The commit on `main` carries the requester, the approver and the run id as trailers. | `org/bin/acme` (`event`, `save`), `org/state/` |
| R8 Usable by agents | Every command is non-interactive, takes everything as flags (`--as` or `ACME_USER`, `--input name=value` or `--inputs-json`), prints a single JSON document on stdout (progress goes to stderr) and uses distinct exit codes: 0 ok, 1 usage or system error, 2 inputs refused, 3 not allowed, 4 run failed. | `org/bin/acme` |

## Commands

Run these from `/tmp/actions-ab/r6/gha-3/org`. Nothing has to be started first.

```bash
cd /tmp/actions-ab/r6/gha-3/org

# List the action on all three repositories (or: --repo beta)
bin/acme list

# Preview: validate the inputs and show the change. Creates no run and changes nothing.
bin/acme preview alpha mark-deprecated \
  --input successor=acme/dependabot-proxy-v2 \
  --input "reason=Replaced by the v2 proxy, which supports OIDC registries."

# Request a run as ann: validates, previews, and prints the run id, the diff and who may approve
bin/acme request alpha mark-deprecated --as ann \
  --input successor=acme/dependabot-proxy-v2 \
  --input "reason=Replaced by the v2 proxy, which supports OIDC registries."
RUN=$(bin/acme runs --repo alpha --status awaiting_approval | jq -r '.runs[-1].id')

# Approve as bob: runs apply and verify, then observes main. Exit 0 = succeeded, 4 = failed.
bin/acme approve "$RUN" --as bob

# Status and full record
bin/acme status "$RUN"
bin/acme record "$RUN"
bin/acme runs                        # every run; filter with --repo / --status

# Turn a run down (a reviewer) or withdraw it (the requester)
bin/acme reject "$RUN" --as bob --comment "not yet"
```

`--inputs-json '{"successor":"acme/x","reason":"..."}'` can replace the `--input` flags, and
`ACME_USER=ann` can replace `--as ann`.

### Testing the rules

`bin/acme-selftest` runs every case below against throwaway copies of the origins under
`state/selftest-*`. It never writes the real origins, prints one PASS or FAIL line per check (24 of
them) and exits non-zero on any failure. Pass `--keep` to keep the sandbox. To try the cases by hand
on the real origins:

```bash
# Invalid inputs: each prints status "refused" with the errors and exits 2; main does not move
bin/acme request beta mark-deprecated --as ann --input successor=Acme/Store --input "reason=State management moves to the shared store package."
bin/acme request beta mark-deprecated --as ann --input successor=acme/valtio-store --input "reason=too short"
bin/acme request beta mark-deprecated --as ann --input successor=acme/beta --input "reason=Points at itself, which is refused."
bin/acme preview beta mark-deprecated --input successor=acme/x            # missing reason

# Self-approval and carol: both exit 3 with "kind": "forbidden"; the run stays awaiting_approval
RUN=$(bin/acme request gamma mark-deprecated --as ann --input successor=acme/graphify-core \
      --input "reason=Graph extraction now lives in graphify-core." 2>/dev/null | jq -r .id)
bin/acme approve "$RUN" --as ann
bin/acme approve "$RUN" --as carol
bin/acme record "$RUN" | jq '.events'    # shows the two approval_denied events

# Rejected push: make the origin refuse pushes, approve as bob -> exit 4, status "failed"
printf '#!/bin/sh\necho "push rejected for testing" >&2\nexit 1\n' > ../origins/gamma.git/hooks/pre-receive
chmod +x ../origins/gamma.git/hooks/pre-receive
bin/acme approve "$RUN" --as bob
bin/acme status "$RUN" | jq '{status, error: .outcome.error, push: .outcome.apply.push_output, observed: .outcome.observed.ok}'
rm ../origins/gamma.git/hooks/pre-receive
```

Any other way of rejecting the push, such as making the origin read-only, ends the same way. To run
the workflow itself with act, outside `acme`, run this from a clone of an origin:

```bash
git clone -q /tmp/actions-ab/r6/gha-3/origins/alpha.git /tmp/actions-ab/r6/gha-3/alpha-try && cd /tmp/actions-ab/r6/gha-3/alpha-try
echo '{"inputs":{"successor":"acme/x","reason":"Only a preview, nothing is pushed."}}' > ../try-event.json
../bin/act workflow_dispatch -W .github/workflows/mark-deprecated.yml -j preview \
  -e ../try-event.json -P ubuntu-latest=-self-hosted -a ann --env ACME_REPOSITORY=acme/alpha
```

Pass `-W` with this one file: act otherwise parses every workflow in the folder, and one of alpha's
existing workflows (`ghcr.yml`) fails act's schema check. Do not run `-j apply` or `-j verify` this
way: act ignores `environment:`, so it would push without an approval (see below).

## How GitHub enforces it

These parts are configured as code in each repository, but GitHub, not act, enforces them:

| Rule | GitHub mechanism | Configured by |
|---|---|---|
| A maintainer other than the requester approves | Environment `deprecation` with required reviewers ann and bob and `prevent_self_review: true`. The `apply` job waits until a reviewer who did not trigger the run approves it. carol is not a reviewer. | `apply-settings.py` → `PUT /repos/{repo}/environments/deprecation` |
| Only the reviewed definition can deploy | Deployment branch policy: only `main` may deploy to `deprecation`, so a branch carrying an edited workflow cannot. | `apply-settings.py` → `POST .../deployment-branch-policies` |
| Nothing reaches `main` without approval | Ruleset `protect-main`: pull requests with code-owner review, no force pushes, no deletion. Deploy keys may bypass it. The repository's only write deploy key is stored as the environment secret `DEPRECATION_DEPLOY_KEY`, which GitHub hands only to an approved `apply` job (`actions/checkout` with `ssh-key`). | `apply-settings.py` → `POST /repos/{repo}/rulesets` |
| The team owns the definition | CODEOWNERS on `.github/self-service/`, the two workflows and CODEOWNERS itself, enforced by the ruleset's `require_code_owner_review`. | `CODEOWNERS` |
| Settings follow the code | `mark-deprecated-settings.yml` reruns `apply-settings.py` whenever these files change on `main`. | `mark-deprecated-settings.yml` (secret `REPO_ADMIN_TOKEN`) |
| Run history and approvals | The Actions run list, the job summaries (validation result, preview diff, pushed commit, observation) and the environment's deployment review log. `apply` reads the approver from `GET /actions/runs/{id}/approvals` for the commit trailer. | built in |

One-time setup on GitHub, per repository:

- `ssh-keygen -t ed25519 -f key -N ''`
- `gh repo deploy-key add key.pub --allow-write -R acme/<repo>`
- `gh secret set DEPRECATION_DEPLOY_KEY --env deprecation -R acme/<repo> < key`
- `gh secret set REPO_ADMIN_TOKEN -R acme/<repo>`
- then run `mark-deprecated-settings.yml` once.

On GitHub, `acme`'s commands would map to:

| `acme` command | GitHub equivalent |
|---|---|
| `list` | read `action.json` through the contents API |
| `request` | `gh workflow run mark-deprecated.yml -R acme/<repo> -f successor=... -f reason=...` |
| `approve` / `reject` | `POST /repos/{repo}/actions/runs/{id}/pending_deployments` with `state: approved` or `rejected` |
| `status` / `record` | `gh run view`, plus `GET /actions/runs/{id}/approvals` |

That GitHub backend is not implemented: `acme` runs act only.

To preview what `apply-settings.py` would send, without sending it:

```bash
cd /tmp/actions-ab/r6/gha-3/alpha && ACME_REPOSITORY=acme/alpha python3 .github/self-service/mark-deprecated/apply-settings.py --dry-run
```

It also dry-runs automatically when `GH_TOKEN` is unset, for example under act.

## What I could not run here

- **GitHub's own enforcement:** the environment gate (required reviewers, prevent self-review,
  main-only deployment), the `protect-main` ruleset, CODEOWNERS review and the run history UI. act
  ignores `environment:`, so locally `acme approve` is the gate. It applies the same rules from the
  same `action.json`, but anyone who runs act directly, or pushes to a bare origin with git, bypasses
  it. The origins here have no branch protection, and on GitHub only the ruleset closes that gap.
- **`apply-settings.py` against the GitHub API.** It ran only as a dry run, both directly and
  through `mark-deprecated-settings.yml` under act. The API shapes follow GitHub's REST docs but have
  not been exercised.
- **The `approvals` API lookup** in `mark_deprecated.py`. Under act, `acme` passes the approver in
  `ACME_APPROVED_BY` instead.
- **Checking out with the deploy key.** act copies the local clone instead of running
  `actions/checkout`, so the `ssh-key` input is unused here. The push goes to the file-path origin.
- **The job summaries** that GitHub shows on the run page. act prints them in the log
  (`state/runs/<id>/apply.log`), and `acme` reads the same results from `results/<phase>/*.json`.
- **Identity.** `--as` is trusted, as this sandbox has no authentication. People must be listed in
  `org/acme.json`, and unknown names are refused. On GitHub, identity is the signed-in user.
