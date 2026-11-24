# Handoff: the `mark-deprecated` self-service action

`mark-deprecated` is live on `alpha`, `beta` and `gamma`. Each repository owns a GitHub Actions
workflow that validates and previews its inputs, then waits for a second maintainer to approve. Only
then does it push `DEPRECATED.md` to `main` and read the file back. From this directory,
`bin/acme-actions` is the org's one place to list the action, preview, request, approve, check status
and read records. It enforces each repository's approval rules, runs the repository's own workflow
with act, and reports `succeeded` only after it has read the file from the origin's `main` itself.

The three requested runs were made by ann and approved by bob, and all three `succeeded`:

| Run | Repository | Commit on origin `main` | Record |
|---|---|---|---|
| `run-0001` | alpha | `ab8b290` | `runs/run-0001.json` |
| `run-0002` | beta | `256c376` | `runs/run-0002.json` |
| `run-0003` | gamma | `f94ccc2` | `runs/run-0003.json` |

`DEPRECATED.md` is `# Deprecated: use <successor>`, a blank line and `<reason>`, followed by a
final newline.

## Commands

Nothing needs to be running: there is no server or daemon. Every command is a single invocation that
prints JSON on stdout. You need `python3` with PyYAML, `git`, `jq` and `bash`, which are already
present here, and act at `../bin/act`.

```bash
cd /tmp/actions-ab/r2/gha-2/org

# List mark-deprecated for all three repositories, read live from each origin's main:
# inputs, definition commit, environment and who may approve.
bin/acme-actions list

# Preview: validate and show what a run would change (diff and the exact content). Nothing is recorded.
bin/acme-actions preview alpha -i successor=acme/dependabot-proxy-v2 \
  -i "reason=Replaced by the v2 proxy, which supports OIDC registries." --as ann

# Request as ann. Prints the run id, the preview and who may approve. The run waits in awaiting_approval.
bin/acme-actions request alpha -i successor=acme/dependabot-proxy-v2 \
  -i "reason=Replaced by the v2 proxy, which supports OIDC registries." --as ann
#   (inputs can also be given as JSON: --inputs-json '{"successor":"acme/x","reason":"..."}')

# Approve as bob. This runs the apply job, then observes main. It prints the outcome and exits 0 only on success.
bin/acme-actions approve run-0004 --as bob

# Check a run's status, read its full record, list runs.
bin/acme-actions status run-0004
bin/acme-actions record run-0004
bin/acme-actions runs --repo alpha

# Also: a reviewer rejects, or the requester cancels.
bin/acme-actions reject run-0004 --as bob --comment "not yet"
bin/acme-actions cancel run-0004 --as ann
```

You can give identity as `--as USER` or `ACME_USER=USER`. A command that acts on someone's behalf
refuses to run without one.

| Exit code | Meaning |
|---|---|
| 0 | OK, or the run succeeded |
| 1 | The run failed |
| 2 | Invalid inputs |
| 3 | Not allowed (unknown person, not a reviewer, self-review, no identity) |
| 4 | Not found, or the run is in the wrong state |
| 5 | Internal error |

Errors print `{"error": {"kind", "message", ...}}`.

### Testing the refusals

```bash
# Invalid inputs are refused before anything changes (exit 2). The workflow's refusals are recorded
# as runs with status "refused". A missing or unknown input is refused earlier, with no record.
bin/acme-actions request beta -i successor=Acme/Bad_Name -i "reason=too short" --as ann
bin/acme-actions request beta -i successor=acme/beta -i "reason=Twenty characters or more." --as ann   # its own repo
bin/acme-actions request beta -i successor=acme/x --as ann                                                # reason missing

# Self-approval and carol's approval are refused (exit 3). Each attempt is recorded as approve_refused.
bin/acme-actions request gamma -i successor=acme/graph-next -i "reason=Testing approval rules end to end." --as ann
bin/acme-actions approve run-NNNN --as ann
bin/acme-actions approve run-NNNN --as carol
bin/acme-actions approve run-NNNN --as bob      # allowed

# Rejected push: make the origin refuse pushes, then approve a run that needs a push.
printf '#!/bin/sh\necho "main is protected" >&2\nexit 1\n' > ../origins/alpha.git/hooks/pre-receive
chmod +x ../origins/alpha.git/hooks/pre-receive
bin/acme-actions request alpha -i successor=acme/proxy-v3 -i "reason=A new reason, so a push is needed." --as ann
bin/acme-actions approve run-NNNN --as bob      # exit 1, status "failed", outcome.observed.content_matches false
rm ../origins/alpha.git/hooks/pre-receive
```

A rejected push only fails a run that needs a push. If `main` already has exactly the requested
`DEPRECATED.md` (for example, the inputs of the three runs above), the apply job has nothing to push.
The controller still observes the right file on `main`, so it reports `succeeded`. The push step
retries twice, about 6 seconds in total, before it fails, to absorb races with other pushes.

## How each requirement is met

| # | How | Files |
|---|---|---|
| R1 Owned by the repository | Each repository commits the workflow, its approval rules and a settings script to its origin's `main`. CODEOWNERS gives them to ann and bob. The controller reads them from the origin's `main` and hardcodes neither inputs nor approvers. A run executes the workflow at the commit it was requested on, recorded as `definition.commit`. | `<repo>/.github/workflows/mark-deprecated.yml`, `<repo>/.github/environments/deprecation.yml`, `<repo>/.github/scripts/apply-github-settings.py`, `alpha/CODEOWNERS`, `beta/.github/CODEOWNERS`, `gamma/.github/CODEOWNERS` |
| R2 Validated | The workflow's `plan` job validates first. `successor` must match `^[a-z0-9-]+/[a-z0-9-]+$` and must not be the repository itself. `reason` must be 20 to 200 characters on one line and not blank. If validation fails, the job emits the errors and stops before the clone and preview steps; `apply` needs `plan`, so it never runs. The controller also refuses unknown or missing inputs, using the workflow's declared `workflow_dispatch` inputs, before it runs anything. Inputs reach the scripts only through `env`, never through `${{ }}` in `run`. | `mark-deprecated.yml` (job `plan`, step "Validate inputs"), `org/bin/acme-actions` (`parse_inputs`, `plan`) |
| R3 Previewed | The `plan` job clones `main`, writes the file and emits the change kind (`create`/`update`/`none`), the exact content and the unified diff. It writes them to the step summary and as JSON. `preview` shows this without recording a run. `request` stores it in the record, so the approver sees it, and `apply` waits behind it. | `mark-deprecated.yml` (step "Preview the change"), `org/bin/acme-actions` (`cmd_preview`, `cmd_request`), `record.preview` |
| R4 Approved | The `apply` job uses `environment: deprecation`. On GitHub, the environment's required reviewers (ann, bob) and `prevent_self_review: true` hold the job until another maintainer approves. Locally the controller enforces the same file, read live from `main` as GitHub does. The approver must be a listed reviewer and not the requester; carol is not listed. Refused attempts are recorded. A run can be approved once. | `<repo>/.github/environments/deprecation.yml`, `<repo>/.github/scripts/apply-github-settings.py`, `org/bin/acme-actions` (`review`) |
| R5 Done means observed | The workflow's last step clones `main` again and fails unless `DEPRECATED.md` is byte-identical to the requested content. Independently, after the job, the controller reads `refs/heads/main:DEPRECATED.md` from the origin. It reports `succeeded` only if the job passed and that file equals the previewed content. Otherwise the run is `failed`, with the reason, the errors and what it saw (`outcome.observed`). | `mark-deprecated.yml` (step "Verify DEPRECATED.md on main"), `org/bin/acme-actions` (end of `review`) |
| R6 One place for the org | The catalog names the repositories, their origins, the people and the actions. `bin/acme-actions list` reads every repository's definition from its origin. `preview` and `request` take a repository name, and the controller runs that repository's workflow in a temporary clone under `.work/`. Nobody has to enter a repository. | `org/org.yaml`, `org/bin/acme-actions` |
| R7 Audited | One JSON record per run holds `requested_by`/`requested_at`, `approved_by`/`approved_at`, `inputs`, `preview`, `definition.commit`, `outcome` (with the observed commit) and `finished_at`. It also has an `events` list with a timestamp and actor for every step, including refused approvals. Every event is appended to `runs/audit.jsonl`, and act's logs are kept per run. The pushed commit carries `Requested-by`, `Approved-by` and `Run` trailers. | `org/runs/run-*.json`, `org/runs/audit.jsonl`, `org/runs/logs/`, commit trailers on each origin |
| R8 Usable by agents | Every step is one non-interactive command with JSON on stdout and a meaningful exit code: `list`, `preview`, `request`, `approve`/`reject`/`cancel`, `status`, `record` and `runs`. Identity is a flag or an environment variable, inputs are flags or JSON, and act runs with stdin closed. | `org/bin/acme-actions` (see `--help`) |

## How GitHub would enforce what act cannot

act does not implement environments, reviews, rulesets, CODEOWNERS or the run history. They are
configured as code in each repository, and GitHub would enforce them like this:

- **Approval (R4).** Run `.github/scripts/apply-github-settings.py --repo acme/<name>` once with
  `gh` authenticated as a repository admin. It creates the `deprecation` environment with ann and bob
  as required reviewers, `prevent_self_review: true`, `can_admins_bypass: false`, and a deployment
  branch policy of `main` only. GitHub then pauses the `apply` job as "Waiting". Only ann or bob can
  approve it, in the UI or with
  `gh api -X POST repos/acme/<name>/actions/runs/<run>/pending_deployments -F 'environment_ids[]=<id>' -f state=approved -f comment=...`.
  The user who triggered the run is refused.
- **Ownership (R1).** The same script adds a `main` ruleset that blocks deletion and force pushes.
  It requires a pull request with code-owner review, so changes to the workflow, the environment file
  or the script need ann's or bob's approval. GitHub Actions is the ruleset's bypass actor
  (`Integration` 15368), so an approved run can push `DEPRECATED.md` with `GITHUB_TOKEN`. The workflow
  asks for `contents: write` only in `apply`.
- **Request, status and record (R6–R8).** The commands map to `gh workflow run mark-deprecated.yml
  -R acme/<name> -f successor=... -f reason=...`, `gh run view <run> -R acme/<name> --json
  status,conclusion,jobs`, and `gh api repos/acme/<name>/actions/runs/<run>/approvals` with the run
  logs. GitHub keeps that run history. On GitHub the `apply` job looks up the approver through the
  approvals API for its commit trailer. Locally the controller passes it in.

## What I could not run here

- GitHub's own enforcement: the environment wait and reviewer checks, `prevent_self_review`, the
  ruleset, CODEOWNERS and the run history. Locally the controller enforces the same environment file.
  `apply-github-settings.py` was run only with `--dry-run`, which prints the API calls. Its payloads
  follow GitHub's REST documentation and are not verified against the API. The ruleset bypass actor id
  for GitHub Actions (15368) is from secondary sources.
- The workflow's GitHub-only paths: authentication with `GITHUB_TOKEN`, used when `ORIGIN_URL` is
  `https://`; the `gh api` approver lookup; and step summaries rendered in the UI. act ran both jobs on
  the host with `-P ubuntu-latest=-self-hosted` against the local origins. The controller points the
  workflow at them with the `MARK_DEPRECATED_ORIGIN_URL` and `MARK_DEPRECATED_REPOSITORY` variables;
  on GitHub those variables are unset and the defaults apply.
- An org backend that calls GitHub. `bin/acme-actions` drives act and the local origins only. The
  `gh` mapping above is how it would talk to GitHub; it is not implemented.

## Notes and limits

- **Run state.** It lives in `org/runs/`. Approval and state changes take a file lock, so a run cannot
  be approved twice. Temporary clones, preview logs and act's cache live in `org/.work/`, which is
  ignored by git.
- **Who can act.** People come from `org/org.yaml`, and identity is self-declared, as the sandbox
  allows. Anyone listed may request a run. Only a repository's listed reviewers, other than the
  requester, may approve or reject it.
- **What a run executes.** A run executes the workflow at the commit it was requested on. Reviewers
  are read when someone approves, as with GitHub environments. If `main` moved after the preview, the
  `apply` job re-plans against the new `main`. The controller still requires the previewed content.
- **Tampering.** The record files are plain JSON in a git checkout. Tamper evidence comes from
  committing them to `origins/org.git`. On GitHub, the run history is the authoritative audit trail.
