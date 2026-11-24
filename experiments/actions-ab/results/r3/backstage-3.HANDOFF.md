# Handoff: `mark-deprecated`, a repository-owned self-service action on Backstage

**Status.** `mark-deprecated` is defined in each of `alpha`, `beta` and `gamma` and runs through
Backstage (1.57, scaffolder backend) with a validate → preview → approve → push → observe lifecycle.
ann requested it on all three repositories, bob approved each run, and each run is reported as
`succeeded` after Backstage observed `DEPRECATED.md` on the origin's `main`:

| Repository | Run | Status | `main` after the run |
|---|---|---|---|
| alpha | `run-c81b517002` | succeeded | `a1d948d` Mark alpha deprecated: use acme/dependabot-proxy-v2 |
| beta | `run-7678bb086c` | succeeded | `68d1ac1` Mark beta deprecated: use acme/valtio-store |
| gamma | `run-2d82e19133` | succeeded | `89211fc` Mark gamma deprecated: use acme/graphify-core |

The run list also holds my negative tests: `run-d8a1129aa3` (alpha, push rejected by a hook, so
`failed`, and `main` was unchanged), `run-ee57bbcc5e` (gamma, requested by carol, rejected by bob)
and `run-1639b5c974` (beta, its approval was refused because the definition changed after the
request, then rejected). beta's `main` also has a later commit, `7612711`, which changed only its
action's description. That commit was the change that tested the definition-changed refusal.

## How it fits together

| Piece | Path | What it is |
|---|---|---|
| Action definition (per repository) | `<repo>/.backstage/actions/mark-deprecated.yaml` on each origin's `main` | A Backstage scaffolder Template: input schema, owner team, steps, preview outputs |
| Repository descriptor | `<repo>/catalog-info.yaml` | Component `<repo>`, owned by `group:default/<repo>-maintainers` |
| Org repository list | `org/repositories.yaml` (read from `origins/org.git` `main`) | Which repositories exist and where their origins are |
| People and teams | `org/catalog/people.yaml` | Users ann, bob, carol; groups `<repo>-maintainers` = ann, bob; `developers` = carol |
| Catalog provider | `backstage/packages/backend/src/acme/catalogProvider.ts` | Reads the above straight from the origins every 10 s (`acme.refreshSeconds`) |
| Scaffolder actions | `backstage/packages/backend/src/acme/scaffolderActions.ts` | `acme:run:claim` (approval gate), `acme:git:commit-file` (preview diff, or commit and push), `acme:git:verify-file` (observe on `main`) |
| Runs API (backend plugin `acme-actions`) | `backstage/packages/backend/src/acme/runsPlugin.ts` | List, preview, request, approve, reject, status, record; approval rules; audit trail; independent post-run observation |
| Git helpers / registry | `backstage/packages/backend/src/acme/git.ts`, `registry.ts` | |
| Wiring | `backstage/packages/backend/src/index.ts` (three `backend.add` lines) | |
| Config | `backstage/app-config.local.yaml` | Port 7810, the three static tokens, on-disk SQLite (`backstage/.data/`), catalog fed only by the provider, `acme.org.origin` |
| CLI for people and agents | `org/bin/acme` (Node, no dependencies) | A thin JSON client over the runs API |

A run is: **request** (validated, then previewed with a scaffolder dry run of the repository's own
template) → **awaiting-approval** → **approve** by a maintainer other than the requester → the
plugin starts a scaffolder task from the template → `acme:run:claim` fetches the *approved* inputs
from the plugin (and fails unless this task belongs to an approved run) → `acme:git:commit-file`
commits as the requester and pushes to the origin's `main` → `acme:git:verify-file` reads `main` →
the plugin reads the origin's `main` once more and compares it with the approved preview →
**succeeded** or **failed**. A maintainer or the requester can **reject** a run that is awaiting approval.

## R1–R8

| # | How it is met | Files |
|---|---|---|
| R1 Owned by the repository | Each repository's definition is a scaffolder Template committed to its own origin's `main` (`alpha` `e4af91b`, `beta` `c47c811` then `7612711`, `gamma` `49c9683`). The catalog reads it from the origin, not from a central copy. A repository may only define templates in its own namespace (`template:<repo>/mark-deprecated`) and a Component with its own name. The template's `spec.owner` is the repository's maintainer team, whose members are the approvers. A change pushed to `main` shows up in `acme actions` within about 10 s, or at once after `acme refresh` plus a few seconds of catalog processing. Tested: changing beta's description on `main` showed up in the list. | `<repo>/.backstage/actions/mark-deprecated.yaml`, `<repo>/catalog-info.yaml`, `catalogProvider.ts` |
| R2 Validated | Inputs are checked against the template's own JSON Schema (`successor`: `^[a-z0-9-]+/[a-z0-9-]+$` and not `acme/<repo>` itself; `reason`: 20–200 characters; both required) and unknown inputs are refused. This happens on `preview` and on `request` before any run is created (HTTP 400 with a per-field `errors` list, exit 1). The scaffolder validates again when the task is created. | `mark-deprecated.yaml` `spec.parameters`, `runsPlugin.ts` `validateInputs` |
| R3 Previewed | `acme preview` (no run created) and `acme request` both run the repository's template as a scaffolder **dry run**. The `acme:git:commit-file` step reads the origin's `main` and returns the path, the current content (`before`), the new `content`, a unified `diff`, `changed` and `baseCommit`. The preview is stored in the run, so the approver sees it too (`acme record`). | `runsPlugin.ts` `preview`, `scaffolderActions.ts` `commitFileAction` |
| R4 Approved | Only `POST /runs/:id/approve` starts a run. It refuses: the requester (403); anyone who is not a member of the template's owner group, such as carol (403); a run not awaiting approval (409); and a run whose definition changed since the request (409). Every refused attempt is recorded as an `approval-denied` event. Starting the template directly through `/api/scaffolder/v2/tasks` does not bypass this: `acme:run:claim` and `acme:git:commit-file` ask the plugin for the approved run owning *this* scaffolder task, and fail if there is none. Tested: a direct task failed at `claim` and changed nothing. | `runsPlugin.ts` (`/runs/:id/approve`, `/internal/claim`, `/internal/runs/by-task`), `scaffolderActions.ts` |
| R5 Done means observed | `succeeded` is set only by the plugin. It needs the scaffolder task to complete (its `verify` step already read `main`) **and** its own read of the origin's `main`: `DEPRECATED.md` must equal the approved preview content, and the run's commit must be on `main`. Every run makes its own commit (empty if the content is already there), so a rejected push always ends in `failed`. That holds even when `main` already has the right file. A rejected push fails the `write` step, so the run is `failed` and its outcome carries git's message. Tested with a `pre-receive` hook. | `runsPlugin.ts` `poll`/`observe`, `scaffolderActions.ts` `verifyFileAction` |
| R6 One place for the org | From `/tmp/actions-ab/r3/backstage-3/org`, `bin/acme actions` lists every repository's actions, and `bin/acme request <repo> mark-deprecated …` requests a run on any of them. The repository list is `org/repositories.yaml` on `origins/org.git` `main`; adding a repository there adds it to the catalog. | `org/bin/acme`, `org/repositories.yaml`, `catalogProvider.ts` |
| R7 Audited | `acme record <run>` returns: `requestedBy`/`requestedAt`, `approvedBy`/`approvedAt` (or `rejectedBy`/`rejectedAt`), `inputs`, the `preview`, `startedAt`, `claimedAt`, `finishedAt`, `outcome` (result, message, what was observed, and the commit). It also returns an ordered `events` list with timestamp and actor (`requested`, `approval-denied`, `approved`, `started`, `claimed`, `succeeded`/`failed`/`rejected`) and a copy of the scaffolder task log. Records are in SQLite on disk (`backstage/.data/`), so they survive restarts. Each commit on `main` also carries `Acme-Run:`, `Requested-by:` and `Approved-by:` trailers, with the requester as author. | `runsPlugin.ts` (tables `acme_runs`, `acme_run_events`) |
| R8 Usable by agents | Every step is one non-interactive command that prints JSON. Exit codes: 0 ok, 1 refused/error (JSON has `error`, `httpStatus`, and `errors` for validation), 2 usage, 3 `wait` ended without success. The same operations are plain HTTP+JSON under `/api/acme-actions` with `Authorization: Bearer <token>`. | `org/bin/acme`, `runsPlugin.ts` |

## Commands

All commands assume this shell setup:

```bash
export PATH=/tmp/bs/node24/bin:$PATH
cd /tmp/actions-ab/r3/backstage-3/org
ANN=ann-token-0123456789 BOB=bob-token-0123456789 CAROL=carol-token-0123456789
```

### Start (and stop) Backstage

The backend uses a file watcher. This machine is shared and had nearly used up its inotify
instances (126 of 128), so the backend crashed with `EMFILE: too many open files, watch`. I raised
the limit with the command below. Run it again if that error comes back:

```bash
sudo sysctl -w fs.inotify.max_user_instances=1024
```

Start the backend in a tmux session on port 7810, then wait until it answers. That takes 5–60 s.
With an empty `backstage/.data/`, `acme actions` can be empty for the first ~15 s, until the
catalog has read the origins:

```bash
tmux new-session -d -s r3-backstage-3-backend -c /tmp/actions-ab/r3/backstage-3/backstage \
  'export PATH=/tmp/bs/node24/bin:$PATH; yarn workspace backend start 2>&1 | tee /tmp/actions-ab/r3/backstage-3/backend.log'
until curl -sf http://localhost:7810/api/acme-actions/health; do sleep 2; done; echo
```

Stop it with:

```bash
tmux send-keys -t r3-backstage-3-backend C-c; sleep 3; tmux kill-session -t r3-backstage-3-backend
```

State lives in `backstage/.data/` (SQLite, one file per plugin). To start from an empty run history,
stop the backend and delete that directory. That does not undo commits on the origins.

### Use it

| Step | Command |
|---|---|
| Who am I | `ACME_TOKEN=$ANN bin/acme whoami` |
| List actions (all repositories) | `ACME_TOKEN=$ANN bin/acme actions` (filters: `--repository alpha`, `--action mark-deprecated`) |
| One action and its input schema | `ACME_TOKEN=$ANN bin/acme describe alpha mark-deprecated` |
| Preview (creates nothing) | `ACME_TOKEN=$ANN bin/acme preview alpha mark-deprecated --successor acme/dependabot-proxy-v2 --reason "Replaced by the v2 proxy, which supports OIDC registries."` |
| Request as ann | `RUN=$(ACME_TOKEN=$ANN bin/acme request alpha mark-deprecated --successor acme/dependabot-proxy-v2 --reason "Replaced by the v2 proxy, which supports OIDC registries." \| jq -r .id); echo $RUN` |
| Approve as bob | `ACME_TOKEN=$BOB bin/acme approve $RUN` |
| Reject | `ACME_TOKEN=$BOB bin/acme reject $RUN --comment "why"` |
| Status | `ACME_TOKEN=$ANN bin/acme status $RUN` (`.status`: `awaiting-approval`, `running`, `succeeded`, `failed`, `rejected`) |
| Wait for the end | `ACME_TOKEN=$ANN bin/acme wait $RUN --timeout 120` (exit 0 only if `succeeded`) |
| Record (audit) | `ACME_TOKEN=$ANN bin/acme record $RUN` |
| All runs | `ACME_TOKEN=$ANN bin/acme runs` (filters: `--repository`, `--status`) |
| Re-read origins now | `ACME_TOKEN=$ANN bin/acme refresh` |

Inputs can also be passed as JSON: `--inputs '{"successor":"acme/x","reason":"..."}'`.

The same operations over HTTP, for example:

```bash
curl -s -H "Authorization: Bearer $ANN" http://localhost:7810/api/acme-actions/actions
curl -s -H "Authorization: Bearer $ANN" -H 'Content-Type: application/json' \
  -X POST http://localhost:7810/api/acme-actions/actions/alpha/mark-deprecated/preview \
  -d '{"inputs":{"successor":"acme/x","reason":"Twenty characters or more."}}'
curl -s -H "Authorization: Bearer $ANN" -H 'Content-Type: application/json' \
  -X POST http://localhost:7810/api/acme-actions/runs \
  -d '{"repository":"alpha","action":"mark-deprecated","inputs":{"successor":"acme/x","reason":"Twenty characters or more."}}'
curl -s -H "Authorization: Bearer $BOB" -X POST http://localhost:7810/api/acme-actions/runs/$RUN/approve
curl -s -H "Authorization: Bearer $ANN" http://localhost:7810/api/acme-actions/runs/$RUN
curl -s -H "Authorization: Bearer $ANN" http://localhost:7810/api/acme-actions/runs/$RUN/record
```

Check the result on the code host with
`git --git-dir /tmp/actions-ab/r3/backstage-3/origins/alpha.git show main:DEPRECATED.md`. The file is
exactly `# Deprecated: use <successor>`, a blank line, then `<reason>`, followed by a single trailing
newline.

### Tests to try

| Case | Command | Expected |
|---|---|---|
| Invalid successor / short reason | `ACME_TOKEN=$ANN bin/acme request alpha mark-deprecated --successor Acme/X --reason short` | exit 1, HTTP 400, `errors` for both fields, no run created (`bin/acme runs` unchanged) |
| Successor is the repository itself | `... --successor acme/alpha --reason "Twenty characters or more."` | 400 `must not be "acme/alpha"` |
| Reason > 200 chars | `... --reason "$(printf 'x%.0s' {1..201})"` | 400 |
| Unknown input | `... --colour red` | 400 `unknown input` |
| Self-approval | request as ann, then `ACME_TOKEN=$ANN bin/acme approve $RUN` | 403, `approval-denied` event in the record, status still `awaiting-approval` |
| carol approves | `ACME_TOKEN=$CAROL bin/acme approve $RUN` | 403 `carol is not a maintainer of alpha` |
| Rejected push | `H=/tmp/actions-ab/r3/backstage-3/origins/alpha.git/hooks/pre-receive; printf '#!/bin/sh\necho rejected >&2\nexit 1\n' > $H; chmod +x $H`, then request as ann with new inputs, approve as bob, `bin/acme wait $RUN` | `failed` (exit 3), outcome quotes `pre-receive hook declined`, `main` unchanged. Remove the hook afterwards: `rm $H`. |
| Bypass approval via the scaffolder | `curl -s -X POST -H "Authorization: Bearer $ANN" -H 'Content-Type: application/json' http://localhost:7810/api/scaffolder/v2/tasks -d '{"templateRef":"template:beta/mark-deprecated","values":{"successor":"acme/x","reason":"Twenty characters or more."}}'` | The task fails at step `claim` after about 30 s (`not started by an approved run`). Nothing is pushed. |
| Double approval | approve a finished run again | 409 |

## Notes and limits

- **Identity** is the token: the server maps a static token's subject (`ann`, `bob`, `carol`) to a
  person. Signed-in Backstage users also work, by user name. Approver rights come from the catalog:
  members of the group named in the template's `spec.owner`, from `org/catalog/people.yaml`.
- **Who can change what.** A repository's team can change its action, including its owner team. The
  people in each team are defined in the org repository. A repository cannot define entities for
  another repository: other namespaces and other Component names are skipped.
- **Definition drift.** The run stores the definition's blob sha at request time. Approval is
  refused if the catalog's definition has changed since then, so nobody approves a run whose
  preview came from a different definition. Request a new run in that case.
- **Concurrency.** If a push loses a race with another change to `main` (non-fast-forward), the
  action rebases and retries up to twice. A hook rejection is not retried.
- **Glue.** The Backstage glue imports packages already present in the app's `node_modules`
  (`@backstage/plugin-scaffolder-node`, `@backstage/plugin-catalog-node`, `yaml`, `jsonschema`,
  `express`). Nothing was installed. `yarn tsc` passes.
- **Example data.** `catalog.locations` is empty in `app-config.local.yaml`, so the create-app
  example entities are no longer loaded.
