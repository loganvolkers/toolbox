# Handoff: `mark-deprecated`, a repository-owned self-service action on Backstage

**Status.** `mark-deprecated` is defined in, and owned by, each of `alpha`, `beta` and `gamma`.
It runs through Backstage 1.57's scaffolder with validation, a dry-run preview, approval by a
second maintainer, observation of the result on origin `main`, and an audit record. ann requested
it on all three repositories, bob approved each run, and all three runs are reported `succeeded`:

| Repository | Run id | Commit on origin `main` |
|---|---|---|
| alpha | `31330919-eb34-43f5-937d-c6c6cc805148` | `5c600d03cd83a8db5621330512010f9c6afca866` |
| beta | `d1519bc9-e9c0-4cc2-9f38-e0aae55794fa` | `cc30146a1fe9e07dbb2450a4d7f411831bd87ca3` |
| gamma | `a4e47659-1f7a-4949-ba34-2df6e8578b49` | `d57bd280c839488d43c1e669be381500a1c58165` |

The store also holds three test runs I made with throwaway inputs: one rejected and two failed
pushes. They are in `acme-actions runs`, and none of them changed `main`.

Nothing is running now. Start the portal with the first command below.

## How it fits together

| Part | What it is | Path |
|---|---|---|
| Action definition (per repository) | A scaffolder `Template` (`spec.type: repo-action`) with the inputs' JSON Schema and one step | `<repo>/.backstage/actions/mark-deprecated.yaml` on each origin's `main` |
| Repository's catalog entry | `Component`, owner `group:default/<repo>-maintainers` | `<repo>/catalog-info.yaml` |
| People and maintainer groups | `User` ann, bob, carol; `Group` `alpha-`/`beta-`/`gamma-maintainers` = ann, bob; `developers` = carol | `org/catalog/people.yaml` |
| Org repository list | Which origins the portal reads | `org/repos.yaml` |
| Entity provider (catalog module) | Every 10 s, reads the files above **from each origin's `main`** into the catalog | `backstage/packages/backend/src/acme/catalogModule.ts` |
| Scaffolder action `acme:repo:commit-file` | Dry run: writes the file into the workspace (preview). Real run: claims the approved run, commits, pushes to origin `main`, and re-reads it | `backstage/packages/backend/src/acme/scaffolderModule.ts` |
| Backend plugin `acme-actions` | The API: list, preview, request, approve/reject/cancel, status, record; approval rules; observation; audit store | `backstage/packages/backend/src/acme/plugin.ts`, `store.ts`, `git.ts` |
| Wiring | `backend.add(...)` for the three features above | `backstage/packages/backend/src/index.ts` |
| Config | Port 7930, static tokens, persistent SQLite in `backstage/data/`, `acme.orgOrigin`, catalog rules | `backstage/app-config.local.yaml` (gitignored by create-app, so it is not in the app's git history) |
| Org CLI (one place for the org) | `acme-actions`, Python 3 stdlib, JSON on stdout | `org/bin/acme-actions`, settings in `org/acme-actions.json` |

The Backstage code is committed in the app's local repository (`backstage`, commit `5a77098`).
The repository and org files are committed and pushed to their origins' `main`.

## Requirements

| # | How it is met | Files |
|---|---|---|
| R1 Owned by the repository | Each repository's action is a scaffolder Template committed to **its own origin's `main`**, with `spec.owner: group:default/<repo>-maintainers`. The portal reads it only from there: the entity provider runs `git cat-file` against `origins/<repo>.git` `main` every 10 s (or at once with `acme-actions refresh`). Each Template is forced into the catalog namespace named after its repository (`template:alpha/mark-deprecated`) and annotated `acme.io/repository`, so a repository can only define actions for itself. Product repositories cannot define `User`/`Group` entities, so a repository cannot grant itself approvers. When the team edits the file and pushes to `main`, the action changes. `acme-actions show <repo> mark-deprecated` prints `definedIn.commit` | `<repo>/.backstage/actions/mark-deprecated.yaml`, `<repo>/catalog-info.yaml`, `catalogModule.ts` |
| R2 Validated | Before anything is stored or changed, `POST /runs` and `/preview` validate the inputs against the Template's own `parameters` JSON Schema: `successor` must match `^[a-z0-9-]+/[a-z0-9-]+$` and must not be the repository itself (`acme/<repo>`), `reason` must be 20 to 200 characters, both are required, and unknown keys are refused. The response is HTTP 400 `{"error":"invalid-input","errors":[{field,message}]}`, CLI exit 1, and no run is created. The scaffolder validates the same schema again when the task is created, and the action validates its own inputs (path, content size) | `plugin.ts` (`validateInputs`), Template `parameters` |
| R3 Previewed | `acme-actions preview` (and every `request`) runs the Template through the **scaffolder's dry run** (`POST /api/scaffolder/v2/dry-run`). In a dry run `acme:repo:commit-file` writes the file into the dry-run workspace, and the plugin compares it with origin `main`. It returns per file the `operation` (create/update/unchanged), the exact `content`, its `blobSha`, a unified `diff`, the commit message and the `baseCommit`. The preview is stored in the run, so the approver sees it too (`acme-actions record`). A request that would change nothing is refused (409 `no-change`) | `plugin.ts` (`preview`), `scaffolderModule.ts` (dry-run branch) |
| R4 Approved | A run is created `awaiting-approval` and does nothing until `approve` succeeds. Approve checks: the approver is not the requester (403 `self-approval`), and the approver is a member of the Template's `spec.owner` group in the catalog (403 `not-a-maintainer`, so carol is refused). Refused attempts are recorded as `approval-refused` events. Only then does the plugin create the scaffolder task, passing a one-time execution token as a task secret. **The gate is in the action, not only the API.** `acme:repo:commit-file` refuses to touch a repository unless the acme-actions plugin confirms (`POST /internal/runs/:id/claim`, scaffolder service principal only) an approved run with a matching token, this task, this repository, and the **same file content as the approved preview**. So creating a scaffolder task directly (`POST /api/scaffolder/v2/tasks`), forging a run id, or editing the Template between request and approval cannot change `main` | `plugin.ts` (`/runs/:id/approve`, `/internal/runs/:id/claim`), `scaffolderModule.ts` |
| R5 Done means observed | The action pushes without force to `origins/<repo>.git` `refs/heads/main` (retries only a non-fast-forward race), then re-reads the file there and fails if it differs. Independently, the plugin's sync loop (every 2 s, and on every `status`/`record` call) marks a run `succeeded` **only if** the scaffolder task completed **and** the plugin itself read every expected file at origin `main`'s current commit with the exact previewed content (blob sha match). Otherwise it marks the run `failed`, with the error (for example `push … was rejected: … pre-receive hook declined`) and the observation (`observedBlobSha`, `matches:false`). Tested with a rejecting `pre-receive` hook on alpha and on beta: both runs `failed`, `main` unchanged | `plugin.ts` (`doSync`, `observe`), `scaffolderModule.ts` (`commitAndPush`) |
| R6 One place for the org | From `org/`, `bin/acme-actions list` lists `mark-deprecated` for all three repositories (owner, maintainers, input schema, defining commit). `request <repo> …` targets any of them, and status, approval and records work from there too. Nothing requires entering a repository checkout. The org also owns `repos.yaml` (which repositories exist) and `catalog/people.yaml` (who is in which group) | `org/bin/acme-actions`, `org/repos.yaml`, `org/catalog/people.yaml` |
| R7 Audited | Each run is a row in `acme_runs` with `requestedBy`/`requestedAt`, `approvedBy`/`approvedAt`, `inputs`, the stored preview, `templateCommit`, `scaffolderTaskId`, `status`, `finishedAt` and `outcome` (error, observation with commit and blob shas, and the scaffolder task log). It also has an append-only event list (`acme_run_events`) with a timestamp and actor for each event: `requested`, `approval-refused`, `approved`, `rejected`, `cancelled`, `task-created`, `execution-started`, `execution-refused`, `observed`, `succeeded`/`failed`. Records persist across restarts (SQLite in `backstage/data/acme-actions.sqlite`). The pushed commit is authored by the requester, committed by "Acme Portal", and carries `Acme-Run`, `Requested-by` and `Approved-by` trailers | `store.ts`, `plugin.ts` (`view`) |
| R8 Usable by agents | Every step is one non-interactive command with JSON on stdout and meaningful exit codes: 0 ok, 1 refused by the API (the JSON error is printed), 2 usage, 3 `wait` ended without success, 4 portal unreachable. Identity comes from `--as <user>` or `--token`. The same steps are plain HTTP under `http://localhost:7930/api/acme-actions` with `Authorization: Bearer <token>` (see the API section below) | `org/bin/acme-actions`, `plugin.ts` |

## Commands

All commands run from the org directory with Node on `PATH`:

```bash
export PATH=/tmp/bs/node24/bin:$PATH
cd /tmp/actions-ab/r4/backstage-1/org
```

**Start the portal** (Backstage backend on port 7930). It is ready when `list` answers, about
5 to 30 seconds after start:

```bash
tmux new-session -d -s r4-backstage-1-backend -c /tmp/actions-ab/r4/backstage-1/backstage \
  "export PATH=/tmp/bs/node24/bin:\$PATH; yarn workspace backend start 2>&1 | tee /tmp/actions-ab/r4/backstage-1/backstage/backend.log"
until bin/acme-actions --as ann list >/dev/null 2>&1; do sleep 2; done
```

Or run `cd /tmp/actions-ab/r4/backstage-1/backstage && yarn workspace backend start` in the
foreground. **Stop it** with `tmux kill-session -t r4-backstage-1-backend`.

**List the actions** (all repositories, or one):

```bash
bin/acme-actions --as ann list
bin/acme-actions --as ann list --repo beta
bin/acme-actions --as ann show alpha mark-deprecated
```

**Preview** (validates and dry-runs; creates nothing):

```bash
bin/acme-actions --as ann preview alpha mark-deprecated \
  --set successor=acme/new-proxy --set reason="Replaced by the new proxy, see the migration guide."
```

**Request as ann** (validated, previewed, stored `awaiting-approval`; prints the run with its
`id`):

```bash
RUN=$(bin/acme-actions --as ann request alpha mark-deprecated \
  --set successor=acme/new-proxy --set reason="Replaced by the new proxy, see the migration guide." | jq -r .id)
# equivalently: --inputs '{"successor":"acme/new-proxy","reason":"..."}'
```

**Approve as bob**, or reject. The requester can also cancel their own run while it awaits
approval:

```bash
bin/acme-actions --as bob approve "$RUN" --comment "ok"
bin/acme-actions --as bob reject "$RUN" --comment "not yet"
bin/acme-actions --as ann cancel "$RUN"
```

**Check a run's status** (`awaiting-approval`, `running`, `succeeded`, `failed`, `rejected`,
`cancelled`):

```bash
bin/acme-actions --as ann status "$RUN"
bin/acme-actions --as ann wait "$RUN" --timeout 120   # exit 0 only if succeeded
bin/acme-actions --as ann runs [--repo alpha] [--status failed]
```

**Read a run's record** (requester, approver, inputs, preview, outcome, timestamped events):

```bash
bin/acme-actions --as ann record "$RUN"
bin/acme-actions --as ann record 31330919-eb34-43f5-937d-c6c6cc805148   # alpha's real run
```

**Re-read the repositories' actions now** (after pushing a change to a Template), instead of
waiting up to 10 s:

```bash
bin/acme-actions --as ann refresh
```

## Test scenarios and what to expect

`alpha`, `beta` and `gamma` already have `DEPRECATED.md` from the real runs. A request with exactly
those inputs is refused with 409 `no-change`, so **use a different `reason` or `successor`** to
create a run that changes `main`.

| Scenario | Command | Expected |
|---|---|---|
| Invalid successor | `bin/acme-actions --as ann request alpha mark-deprecated --set successor="Acme/Bad Name" --set reason="Twenty characters or more here."` | exit 1, HTTP 400 `invalid-input`, `errors[0].field = successor`; no run created |
| Successor is the repository itself | `... --set successor=acme/alpha ...` | 400, `must not be this repository itself` |
| Reason too short or too long | `--set reason=short`, or a 201-character reason | 400, `does not meet minimum length of 20` / `does not meet maximum length of 200` |
| Missing or unknown input | omit `--set successor`, or add `--set colour=red` | 400, `requires property "successor"` / `is not an input of this action` |
| Self-approval | ann requests, then `bin/acme-actions --as ann approve "$RUN"` | exit 1, 403 `self-approval`; run stays `awaiting-approval`; `approval-refused` event recorded |
| carol approves | `bin/acme-actions --as carol approve "$RUN"` | exit 1, 403 `not-a-maintainer`; `approval-refused` recorded. carol may *request* runs, which ann or bob can then approve |
| Rejected push | `H=/tmp/actions-ab/r4/backstage-1/origins/alpha.git/hooks/pre-receive; printf '#!/bin/sh\necho rejected >&2\nexit 1\n' > $H; chmod +x $H`, then request with new inputs, approve as bob, and `wait`. Remove it with `rm $H` | `wait` exits 3; status `failed`, `error` contains `pre-receive hook declined`, `observed.files[0].matches=false`; `main` unchanged. Remove the hook afterwards |
| Bypass the approval | `curl -s -H 'Authorization: Bearer carol-token-0123456789' -H 'content-type: application/json' -X POST http://localhost:7930/api/scaffolder/v2/tasks -d '{"templateRef":"template:alpha/mark-deprecated","values":{"successor":"acme/x","reason":"Trying to skip the approval step."}}'` | The task fails with `refusing to change the repository: this task was not started from an approved run`; `main` unchanged |
| Approve twice / approve a finished run | `approve` again | 409 `not-awaiting-approval` |
| Unknown repository or action | `request delta mark-deprecated ...` | 404 `unknown-action` |
| Team changes its action (R1) | Edit `<repo>/.backstage/actions/mark-deprecated.yaml` (for example the `reason` `maxLength`), commit, `git push origin main`, `bin/acme-actions --as ann refresh` | `show` reports the new schema and `definedIn.commit`. Runs requested before the change but approved after it fail at the claim (`content … differs from the approved preview`) whenever the content would differ |

## HTTP API (for agents that prefer it)

Base URL `http://localhost:7930/api/acme-actions`. Send `Authorization: Bearer
<ann|bob|carol>-token-0123456789`. Requests and responses are JSON, and errors are
`{"error": code, "message", ...}`.

| Method and path | Purpose |
|---|---|
| `GET /whoami` | caller and the actions they maintain |
| `GET /actions[?repository=&action=]` | list actions |
| `GET /actions/:repo/:action` | one action |
| `POST /actions/:repo/:action/preview` `{"inputs":{...}}` | validate and dry-run |
| `POST /runs` `{"repository","action","inputs"}` | request (201, the run) |
| `POST /runs/:id/approve` / `reject` `{"comment"?}`, `POST /runs/:id/cancel` | decide |
| `GET /runs/:id/status`, `GET /runs/:id`, `GET /runs[?repository=&status=]` | status, full record, list |
| `POST /refresh` | re-read origins now |

## Notes and limits

- **Identity.** Identity is the static token's subject (`ann`, `bob`, `carol`), mapped to
  `user:default/<subject>`, which must exist in the org catalog. A signed-in Backstage user
  principal also works. `org/acme-actions.json` holds the sandbox tokens from
  `app-config.local.yaml` for `--as`. Outside a sandbox, use `--token` or `ACME_TOKEN_<USER>`
  instead.
- **Who may change approvers.** Approvers are the members of the Template's `spec.owner` group,
  and groups are defined only in the org repository. A repository team chooses which group owns
  its action, and the org decides who is in a group.
- **Unchanged content.** If `main` already has the approved content when the task executes (for
  example, someone made the same change by hand after the request), the action pushes nothing, the
  plugin still observes the content, and the run succeeds. A request whose preview changes nothing
  is refused up front.
- **Dependencies.** The backend `package.json` declares the packages the glue imports. They were
  already installed, so `yarn install` was not rerun and `yarn.lock` was not updated.
  `yarn tsc` passes.
