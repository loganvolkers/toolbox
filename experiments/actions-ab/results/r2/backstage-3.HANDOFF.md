# Handoff: `mark-deprecated`, a repository-owned self-service action on Backstage

**State.** `mark-deprecated` is defined in each of `alpha`, `beta` and `gamma` and ran once on each: ann
requested, bob approved, and each run is reported `succeeded`. `DEPRECATED.md` is on each origin's `main`:

| Repository | Run id | Commit on `main` |
|---|---|---|
| alpha | `a039df5b-8845-434e-b9c0-71fa2e3f5572` | `9880a15` |
| beta | `f8be920a-8154-4255-bd51-201edfc440ca` | `ac8dec2` |
| gamma | `81ad75d3-4c90-4957-96e7-c638ad05200f` | `ed9d000` |

There is one other run on record: `ad26d038-…` on alpha, requested by carol and approved by ann. I made
it `failed` on purpose, with a `pre-receive` hook that rejected the push (the hook is gone). A
scaffolder task started directly as ann, bypassing approval, also failed (`a8e50be1-…`). It is not a
run.

## How it works

Backstage provides the catalog, the scaffolder (templates, server-side input validation, dry-run
and execution) and authentication with the static tokens. The glue is three pieces in
`backstage/packages/backend/src/acme/`, registered in `backstage/packages/backend/src/index.ts`:

| Piece | File | What it does |
|---|---|---|
| Catalog module `acme-git-origins` | `catalogModule.ts` | Every 10 s, reads entities from each origin's **`main`** (not the checkouts): `catalog-info.yaml` and `.backstage/actions/*.yaml`. The org's people come from `origins/org.git`, and the list of repositories from `repositories.yaml` there. Each repository's entities go into a namespace named after it. An entity that claims another namespace is dropped, so `alpha` can't define `beta`'s action. |
| Scaffolder module | `scaffolderModule.ts` | Three platform actions that templates compose. `acme:repo:plan-file` computes the diff against `main` and works in dry-run. `acme:repo:commit-file` asks acme-actions for permission, then commits and pushes to `main` as the requester. `acme:repo:verify-file` fails unless `main` has the file with that exact content. |
| Backend plugin `acme-actions` | `actionsPlugin.ts` | The org API at `/api/acme-actions`: list, preview, request, approve, reject, status and record. It holds the run records in SQLite, enforces the approval rule, starts the scaffolder task only after approval, and decides the outcome by reading the origin's `main` itself. |

`git.ts` holds the git helpers, and `registry.ts` reads `repositories.yaml`. The config is in
`backstage/app-config.local.yaml` (gitignored, like the tokens already in it). I added `acme.org.origin`
and a file-backed SQLite (`backend.database.connection.directory: backstage/.data`), so records
survive restarts. The CLI for people and agents is `org/bin/acme-actions` (Node, no dependencies).

The lifecycle of a run:

```
request ── validate (template JSON Schema) ── preview (scaffolder dry-run) ──▶ awaiting-approval
approve (maintainer ≠ requester) ──▶ running: scaffolder task plan → commit+push → verify
task finished ── acme-actions reads origin main ──▶ succeeded (content matches) | failed
reject ──▶ rejected
```

## Requirements

| # | How it is met | Files |
|---|---|---|
| R1 Owned by the repository | The action is a Backstage Template committed to each repository's `main`, owned (`spec.owner`) by a `maintainers` Group that the repository also defines. Backstage reads both from the origin's `main`, so the team changes the action, or who maintains it, by pushing to `main`. The catalog picks it up within about 20 s. A run records the commit of the definition it used. If the definition changes between request and approval, approval is refused (`409 definition-changed`). | `alpha/.backstage/actions/mark-deprecated.yaml`, `alpha/catalog-info.yaml` (and the same two files in `beta/` and `gamma/`); `catalogModule.ts` |
| R2 Validated | The inputs schema is in the template: `successor` must match `^[a-z0-9-]+/[a-z0-9-]+$` and must not be the repository itself (`not: {const: acme/<repo>}`). `reason` is 20 to 200 characters. Both are required, and other keys are refused (`additionalProperties: false`). acme-actions validates against this schema before it does anything (`400 invalid-input` with one entry per input). The scaffolder validates again on dry-run and on task creation. No run record is created for invalid inputs. | template `spec.parameters`; `actionsPlugin.ts` `validateInputs` |
| R3 Previewed | `preview` runs the repository's own template as a scaffolder **dry-run**. `acme:repo:plan-file` reports the change: path, before, after, unified diff, and the base commit on `main`. `request` runs the same preview and stores it in the run, so the requester and the approver see exactly what will land. At execution, the push is refused unless its path and content equal the stored preview. | `actionsPlugin.ts` `preview`, `/internal/runs/:id/claim`; `scaffolderModule.ts` `planFileAction` |
| R4 Approved | `approve` is refused for the requester (`403 self-approval`) and for anyone who isn't a member of the action's owner group (`403 not-a-maintainer`, which covers carol). Refusals are recorded in the run's events. The scaffolder task is created only after approval, by acme-actions, with a per-run secret. `acme:repo:commit-file` will only push if the task was started by acme-actions **and** acme-actions confirms the run is approved by someone other than the requester, with the previewed content. Starting the template directly through `/api/scaffolder/v2/tasks` therefore fails at the commit step. | `actionsPlugin.ts` `/runs/:id/approve`, `/internal/runs/:id/claim`; `scaffolderModule.ts` `commitFileAction` |
| R5 Done means observed | When the scaffolder task ends, acme-actions reads `DEPRECATED.md` from the origin's `main` itself. The run is `succeeded` only if the task completed **and** the content equals the approved preview. Otherwise the run is `failed`, with a reason and what was observed. The template's `verify` step checks the same thing inside the task. A rejected push fails the `commit` step, so the run fails. Each run pushes a commit, even if the content is already there, so a rejected push always fails the run. | `actionsPlugin.ts` `refresh`, `observe`; `scaffolderModule.ts` `verifyFileAction` |
| R6 One place for the org | `org/bin/acme-actions` lists the actions of every repository in `org/repositories.yaml`, and requests, approves and inspects runs on any of them, without entering a repository. Adding a repository means adding it to `repositories.yaml` on the org's `main`. | `org/bin/acme-actions`, `org/repositories.yaml`, `org/catalog-info.yaml` |
| R7 Audited | `record` returns the full run: requester and `requestedAt`, inputs, the preview, approval (`by`, `at`), the scaffolder task with each step's status, and the outcome (`status`, `at`, `reason`, the observed commit). It also returns a timestamped event list: requested, previewed, approval-refused, approved, task-started, task-finished, observed, succeeded/failed/rejected. Records are stored in SQLite (`backstage/.data/acme-actions.sqlite`). On the code host, each commit's author is the requester, with `Acme-Run`, `Requested-by` and `Approved-by` trailers. | `actionsPlugin.ts` (`Run` type, `acme_runs` table) |
| R8 Usable by agents | Every step is one non-interactive command that prints JSON on stdout. Inputs are passed as flags (`--successor …`) or `--inputs '<json>'`. Exit codes: 0 ok; 1 refused or failed (the JSON has `httpStatus` and `error.code`); 2 usage; 3 when `status --wait` ends `failed` or `rejected`; 4 on timeout. The same API can be called with `curl` (below). | `org/bin/acme-actions` |

## Commands

### Start (must be running)

```bash
cd /tmp/actions-ab/r2/backstage-3/backstage
export PATH=/tmp/bs/node24/bin:$PATH
yarn workspace backend start            # listens on http://localhost:7510; leave it running
```

In another shell, wait until it is ready. This prints 3 when the catalog has read the three actions:

```bash
export PATH=/tmp/bs/node24/bin:$PATH
cd /tmp/actions-ab/r2/backstage-3/org
export ACME_TOKEN_ANN=ann-token-0123456789 ACME_TOKEN_BOB=bob-token-0123456789 ACME_TOKEN_CAROL=carol-token-0123456789
until [ "$(bin/acme-actions --as ann list 2>/dev/null | jq '.items | length')" = 3 ]; do sleep 2; done
```

To stop it, press Ctrl-C in the backend's shell, or run `pkill -f 'backstage-cli package start'`.

### Use (from `org/`, with the exports above)

```bash
# List actions (all repositories; filter with --repository alpha / --action mark-deprecated)
bin/acme-actions --as ann list
bin/acme-actions --as ann show alpha mark-deprecated

# Preview: nothing changes
bin/acme-actions --as ann preview alpha mark-deprecated \
  --successor acme/dependabot-proxy-v2 --reason 'Replaced by the v2 proxy, which supports OIDC registries.'

# Request as ann: validated, previewed, awaiting approval. Prints the run; keep its id
RUN=$(bin/acme-actions --as ann request alpha mark-deprecated \
  --successor acme/dependabot-proxy-v2 --reason 'Replaced by the v2 proxy, which supports OIDC registries.' | jq -r .id)

# Approve as bob
bin/acme-actions --as bob approve "$RUN"            # optional: --comment 'looks right'

# Status (add --wait to block until succeeded/failed/rejected; exit code 0 only for succeeded)
bin/acme-actions --as ann status "$RUN"
bin/acme-actions --as ann status "$RUN" --wait --timeout 120

# Full audit record
bin/acme-actions --as ann record "$RUN"

# Other commands
bin/acme-actions --as ann runs --repository alpha    # list runs (also --status succeeded|failed|...)
bin/acme-actions --as bob reject "$RUN" --comment 'not yet'   # only while awaiting approval
```

Equivalent raw API calls (`$B` is the Backstage URL, and the token selects the person):

```bash
B=http://localhost:7510/api/acme-actions
curl -s $B/actions -H 'Authorization: Bearer ann-token-0123456789'
curl -s -X POST $B/actions/alpha/mark-deprecated/preview -H 'Authorization: Bearer ann-token-0123456789' \
  -H 'content-type: application/json' -d '{"inputs":{"successor":"acme/x","reason":"Twenty or more characters here."}}'
curl -s -X POST $B/runs -H 'Authorization: Bearer ann-token-0123456789' -H 'content-type: application/json' \
  -d '{"repository":"alpha","action":"mark-deprecated","inputs":{"successor":"acme/x","reason":"Twenty or more characters here."}}'
curl -s -X POST $B/runs/$RUN/approve -H 'Authorization: Bearer bob-token-0123456789'
curl -s $B/runs/$RUN/status -H 'Authorization: Bearer ann-token-0123456789'
curl -s $B/runs/$RUN -H 'Authorization: Bearer ann-token-0123456789'
```

## Testing the refusals

`DEPRECATED.md` is already on the three mains, but a new run still commits (a new reason changes
the file, and the same reason produces a commit that only records the run). So each test below can
run against any repository.

```bash
# Invalid inputs: 400 invalid-input, exit 1, no run created
bin/acme-actions --as ann request beta mark-deprecated --successor 'Acme/Not Valid' --reason 'too short'
bin/acme-actions --as ann request beta mark-deprecated --successor acme/beta --reason 'A repository cannot succeed itself.'
bin/acme-actions --as ann request beta mark-deprecated --successor acme/x --reason 'Fine reason, long enough.' --extra 1

# Self-approval and carol: 403 self-approval / 403 not-a-maintainer; the run stays awaiting-approval
RUN=$(bin/acme-actions --as ann request beta mark-deprecated --successor acme/valtio-store-v2 --reason 'Testing the approval rules end to end.' | jq -r .id)
bin/acme-actions --as ann approve "$RUN"
bin/acme-actions --as carol approve "$RUN"
bin/acme-actions --as ann status "$RUN"           # awaiting-approval

# Rejected push: the run ends failed and main is unchanged
H=/tmp/actions-ab/r2/backstage-3/origins/beta.git/hooks/pre-receive
printf '#!/bin/sh\necho "main is protected" >&2\nexit 1\n' > $H && chmod +x $H
bin/acme-actions --as bob approve "$RUN"
bin/acme-actions --as ann status "$RUN" --wait; echo "exit=$?"   # status failed, exit 3, reason names the rejected push
git --git-dir /tmp/actions-ab/r2/backstage-3/origins/beta.git show main:DEPRECATED.md   # still the earlier content
rm $H

# Bypassing approval through the scaffolder API directly: the task fails at the commit step
curl -s -X POST http://localhost:7510/api/scaffolder/v2/tasks -H 'Authorization: Bearer ann-token-0123456789' \
  -H 'content-type: application/json' \
  -d '{"templateRef":"template:beta/mark-deprecated","values":{"successor":"acme/x","reason":"Skipping the approval step."}}'
```

## Limits and notes

- **Identity.** A static token's subject is the person (`ann` → `user:default/ann`). The caller must
  be a User in `org/catalog-info.yaml`. Anyone known can request a run; only the owner group's
  members can approve.
- **Statuses.** `awaiting-approval`, `running`, `succeeded`, `failed` and `rejected`. A run cannot
  be approved twice: the update only applies if the run is still awaiting approval.
- **Restarts.** Records survive restarts. A scaffolder task interrupted by a restart is failed by
  the scaffolder, and the run then reports `failed`. It is never reported `succeeded` without
  observing `main`.
- **Timing.** A definition change on `main` reaches the catalog within about 20 s. Until then, `list`
  shows the previous commit in `definition.commit`.
- Backstage's own example entities, such as `template:default/example-nodejs-template`, are still
  in the catalog. `list` shows only templates labelled `acme.io/action` that come from a registered
  repository.
- The Backstage changes are committed in `backstage/` (its own git repository, commit `061bb67`),
  except `app-config.local.yaml`, which the app gitignores.
