# HANDOFF: `mark-deprecated` on Backstage

`mark-deprecated` is a self-service action owned by each repository and run through Backstage
1.57 (scaffolder plus custom glue). It has run on all three repositories: ann requested each run,
bob approved it, and each was reported `succeeded` only after the system read `DEPRECATED.md` on
the origin's `main`.

| Repository | Run | Status | Commit on origin `main` |
|---|---|---|---|
| alpha | `run-7718f7d8` | succeeded | `b7375bd` |
| beta | `run-3915f43f` | succeeded | `64b21f0` |
| gamma | `run-6145eb70` | succeeded | `9ae8048` |

The audit log also holds four failed runs from my own tests of the failure paths, and none of
them changed `main`. On alpha, `run-56c55212` was blocked from self-approval and from carol's
approval, then failed on a push rejected by a `pre-receive` hook. `run-7e7e062d` hit the same
rejected push with content that was already on `main`. `run-08c1250c` had its push accepted and
then reverted by a `post-receive` hook. On beta, `run-083fad6f` came from running the "Testing
the rules" commands below. All test hooks have been removed.

## How it fits together

```
org origin main ──► repos.yaml, catalog/people.yaml, actions/mark-deprecated.yaml (contract)
repo origin main ─► catalog-info.yaml, .backstage/actions/mark-deprecated.yaml (Template)
        │  read with git, straight from the bare origins (never from working copies)
        ▼
Backstage backend (:7170)
  catalog  + module acme-origins         (EntityProvider: origins' main -> catalog)
  scaffolder + module acme-actions       (action acme:repo:commit-file)
  plugin action-runs  /api/action-runs   (validate, preview, request, approve, run, observe, audit)
        ▲
org/bin/acme-actions  (CLI, JSON on stdout)   or   curl with Bearer tokens
```

| Piece | Path |
|---|---|
| Repo-owned action definition (Backstage Template) | `<repo>/.backstage/actions/mark-deprecated.yaml` on `origins/<repo>.git` `main` |
| Repo catalog entry (Component, owner group) | `<repo>/catalog-info.yaml` on `origins/<repo>.git` `main` |
| Org: repositories and origins | `org/repos.yaml` |
| Org: people and maintainer groups | `org/catalog/people.yaml` |
| Org: action contract (input schema and postcondition) | `org/actions/mark-deprecated.yaml` |
| Org CLI | `org/bin/acme-actions` (Python 3, stdlib only; config `org/acme.json`) |
| Start / stop | `org/bin/backstage-up`, `org/bin/backstage-down` |
| Catalog provider (origins to catalog) | `backstage/packages/backend/src/acme/catalogModule.ts` |
| Scaffolder action `acme:repo:commit-file` | `backstage/packages/backend/src/acme/scaffolderModule.ts` |
| `action-runs` plugin (API, approval, execution, observation) | `backstage/packages/backend/src/acme/actionRunsPlugin.ts` |
| Run and audit store (tables `runs`, `run_events`) | `backstage/packages/backend/src/acme/runStore.ts` |
| Org service (reads org origin, signs run grants) | `backstage/packages/backend/src/acme/org.ts` |
| Git helpers | `backstage/packages/backend/src/acme/git.ts` |
| Wiring | `backstage/packages/backend/src/index.ts` |
| Config (persistent SQLite, `acme.*`, catalog rules) | `backstage/app-config.local.yaml` (appended below the given tokens) |
| Runtime data (SQLite per plugin, grant key, backend log) | `backstage/.acme-data/` |

## R1 to R8

| # | How it is met |
|---|---|
| R1 Owned by the repository | Each repository's definition is a Backstage Template at `.backstage/actions/mark-deprecated.yaml`, committed (by ann) to `origins/<repo>.git` `main`, with `spec.owner: group:default/<repo>-maintainers`. The `acme-origins` provider (`catalogModule.ts`) reads only the origin's `main` with `git` and rescans every 15 s. Before every list, preview and request, `action-runs` checks that the catalog matches the origin's current `main` commit and waits for a refresh if it doesn't, so a change pushed to `main` takes effect on the next call. Entities from a repository are confined to its own namespace (`template:<repo>/mark-deprecated`) and may not be User or Group, so one repository cannot define actions or people for another. Each run records the definition's commit and blob, and it fails with `definition-changed` if the definition changes between request and execution. |
| R2 Validated | `action-runs` validates inputs against the org contract (`org/actions/mark-deprecated.yaml`: `successor` must match `^[a-z0-9-]+/[a-z0-9-]+$`, `reason` must be 20 to 200 characters, no other keys) and against the repository template's own parameter schema. It also refuses a `successor` equal to the target repository's slug (`distinctFromTarget`). The scaffolder validates again on dry-run and on task creation. Invalid inputs get HTTP 400 with `errors: [{source, field, message}]`, no run is created and nothing is written. The CLI exits 1. |
| R3 Previewed | `preview`, and every `request`, runs the repository's template through the Backstage scaffolder **dry-run** (`POST /api/scaffolder/v2/dry-run`). `acme:repo:commit-file` in dry-run mode clones the origin and returns `{change: create\|modify\|none, before, after, diff, baseCommit}` without pushing. `satisfiesContract` says whether the result matches the org contract. The preview is stored in the run record, so the approver can see it too (`record <id>` → `.preview`). |
| R4 Approved | A request creates a run in `awaiting-approval`; nothing is written. `approve` needs a member of the template's owner group (catalog `hasMember` relations, built from `org/catalog/people.yaml`) other than the requester. Self-approval and non-maintainers such as carol get HTTP 403, and each denied attempt is recorded as an `approval-denied` event. The write happens only in `acme:repo:commit-file`, and only with an HMAC-signed run grant that `action-runs` issues after approval and passes as a scaffolder task secret. Running the template straight through the scaffolder API (`POST /api/scaffolder/v2/tasks`) fails with "No valid run grant". |
| R5 Done means observed | Each run pushes its own commit (an empty commit if the content is already there) with an `Acme-Run: <id>` trailer. The scaffolder action fails if the push is rejected, if that commit is not an ancestor of the origin's `main`, or if `DEPRECATED.md` on the origin differs. Separately, `action-runs` (`verify()`) reads `DEPRECATED.md` from `origins/<repo>.git` `main` itself, compares it with the contract's postcondition rendered from the inputs, and looks for the run's commit on `main`. Only then does the status become `succeeded`, with `outcome.observed` holding the head, the run commit and the blob. Otherwise the status is `failed`, with `reason` set to `task-failed` (including git's message) or `postcondition-not-observed`. |
| R6 One place for the org | `org/bin/acme-actions` lists `mark-deprecated` for every repository in `org/repos.yaml` and previews, requests, approves and reads runs on any of them, all from `org/`. Missing or broken definitions show up under `missing` in `list`. |
| R7 Audited | The `runs` and `run_events` tables live in the `action-runs` SQLite database under `backstage/.acme-data/db/`, which persists across restarts. `record <id>` returns `requested {by, at}`, `approved {by, at, comment}`, `rejected`, `inputs`, `started {at, scaffolderTaskId}`, `outcome {status, at, observed \| reason, message}`, the definition and contract versions, the preview, and timestamped events (`requested`, `previewed`, `approval-denied`, `approved`, `started`, `task-completed`, `observed`, `succeeded`/`failed`). The commit on `main` carries `Requested-by`, `Approved-by` and `Acme-Run` trailers, with the requester as author. |
| R8 Usable by agents | Every step is a non-interactive CLI command that prints JSON on stdout. Exit codes: 0 ok, 1 refused (400/403/404/409), 2 usage, 3 backend unreachable or 5xx, 4 run ended but did not succeed (with `--wait`). The same operations are plain HTTP+JSON under `/api/action-runs` (listed below). |

## Commands

All commands run from the org directory. Identity is `--as ann|bob|carol` (or `ACME_AS`), which
maps to the static tokens in `org/acme.json` (the ones from `backstage/app-config.local.yaml`).

```bash
cd /tmp/actions-ab/r1/backstage-2/org
```

**Start** (required). This runs `yarn workspace backend start` in the background, with Node 24 on
`PATH`, logs to `backstage/.acme-data/backend.log`, and waits until three actions are listed:

```bash
bin/backstage-up
```

To start it in the foreground instead, run:

```bash
export PATH=/tmp/bs/node24/bin:$PATH
cd /tmp/actions-ab/r1/backstage-2/backstage && yarn workspace backend start
```

**List** the actions on all repositories, or show one:

```bash
bin/acme-actions --as ann list
bin/acme-actions --as ann show mark-deprecated beta
```

**Preview** (validates, then dry-runs; writes nothing):

```bash
bin/acme-actions --as ann preview mark-deprecated alpha \
  -i successor=acme/dependabot-proxy-v2 \
  -i "reason=Replaced by the v2 proxy, which supports OIDC registries."
```

**Request** as ann. Prints the run record; the id is `.id`, and the status is `awaiting-approval`:

```bash
RUN=$(bin/acme-actions --as ann request mark-deprecated alpha \
  -i successor=acme/dependabot-proxy-v2 \
  -i "reason=Replaced by the v2 proxy, which supports OIDC registries." | jq -r .id)
echo "$RUN"
```

`--inputs '{"successor":"…","reason":"…"}'` works instead of `-i`.

**Approve** as bob. `--wait` blocks until the run ends, prints its status, and exits 4 unless it
succeeded:

```bash
bin/acme-actions --as bob approve "$RUN" --wait
```

**Status** and **record**:

```bash
bin/acme-actions --as bob status "$RUN"          # add --wait to block until it ends
bin/acme-actions --as bob record "$RUN"          # full audit record with events and preview
bin/acme-actions --as ann runs --repo alpha      # all runs (filters: --repo --status --action)
```

**Reject or cancel** (rejected by a maintainer, cancelled by the requester), and **refresh** the
catalog from the origins now:

```bash
bin/acme-actions --as bob reject "$RUN" --comment "not yet"
bin/acme-actions --as ann refresh
```

**Stop**:

```bash
bin/backstage-down
```

### Same calls over HTTP

The base URL is `http://localhost:7170/api/action-runs`, with header
`Authorization: Bearer ann-token-0123456789` (or `bob-…`, `carol-…`):

| Step | Request |
|---|---|
| list | `GET /actions[?repository=&action=]` |
| show | `GET /actions/{repo}/{action}` |
| preview | `POST /actions/{repo}/{action}/preview` with body `{"inputs": {...}}` |
| request | `POST /actions/{repo}/{action}/runs` with body `{"inputs": {...}}` → 201 |
| approve | `POST /runs/{id}/approve` with body `{"comment": "..."}` (optional) |
| reject / cancel | `POST /runs/{id}/reject` |
| status | `GET /runs/{id}/status` |
| record | `GET /runs/{id}` |
| list runs | `GET /runs[?repository=&status=&action=]` |
| who am I | `GET /whoami` |

For example:

```bash
curl -s -X POST http://localhost:7170/api/action-runs/actions/beta/mark-deprecated/preview \
  -H 'Authorization: Bearer ann-token-0123456789' -H 'Content-Type: application/json' \
  -d '{"inputs":{"successor":"acme/valtio-store","reason":"State management moves to the shared store package."}}'
```

## Testing the rules

Each of these was run here, with the results shown.

```bash
# Invalid inputs: HTTP 400, exit 1, no run created
bin/acme-actions --as ann request mark-deprecated beta -i successor=Not/Valid -i reason=short
bin/acme-actions --as ann request mark-deprecated beta -i successor=acme/beta -i "reason=pointing at itself is refused"
bin/acme-actions --as ann request mark-deprecated beta -i successor=acme/x -i "reason=twenty chars or more" -i extra=1

# Self-approval and carol: HTTP 403, exit 1, recorded as approval-denied events
RUN=$(bin/acme-actions --as ann request mark-deprecated beta -i successor=acme/other -i "reason=Testing the approval rules here." | jq -r .id)
bin/acme-actions --as ann   approve "$RUN"
bin/acme-actions --as carol approve "$RUN"

# Rejected push: the run is reported failed (exit 4), and main is unchanged
printf '#!/bin/sh\necho "push rejected" >&2\nexit 1\n' > ../origins/beta.git/hooks/pre-receive
chmod +x ../origins/beta.git/hooks/pre-receive
bin/acme-actions --as bob approve "$RUN" --wait
bin/acme-actions --as bob record "$RUN" | jq '.status, .outcome'
rm ../origins/beta.git/hooks/pre-receive

# Bypassing approval through the scaffolder API: the task fails with "No valid run grant"
curl -s -X POST http://localhost:7170/api/scaffolder/v2/tasks -H 'Authorization: Bearer ann-token-0123456789' \
  -H 'Content-Type: application/json' \
  -d '{"templateRef":"template:beta/mark-deprecated","values":{"successor":"acme/x","reason":"no approval at all here"}}'
```

A rejected push fails even when `DEPRECATED.md` already has the requested content, because every
run must land its own commit on `main`.

To change a repository's action (R1), edit `.backstage/actions/mark-deprecated.yaml` in its
checkout, commit, and `git push origin HEAD:main`. The next `list`, `preview` or `request` uses the
new version. A run requested before the change fails with `definition-changed` when it is
approved.

## Notes and limits

- The approval rule comes from the template's `spec.owner` group, whose members are defined in
  `org/catalog/people.yaml`. A repository can change which group owns its action, but people and
  group membership change only in the org repo.
- There are no passwords in this sandbox, so a static token is the identity. In production that
  would be Backstage sign-in.
- The new code imports packages that are already installed and hoisted in `node_modules`
  (`yaml`, `jsonschema`, `express-promise-router`, `@backstage/plugin-scaffolder-node`,
  `@backstage/plugin-catalog-node`). They are not added to `packages/backend/package.json`.
  `yarn tsc` passes.
- The Backstage changes are committed in `backstage/`'s own git repository (commit `9cf788c`).
  The org files are committed to `origins/org.git` `main`.
