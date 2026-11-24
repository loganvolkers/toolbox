# Handoff: `mark-deprecated`, a repository-owned self-service action on Backstage

**Status.** `mark-deprecated` is defined in, and owned by, each of `alpha`, `beta` and `gamma`. It is
listed and run from this org repository through one CLI (`org/bin/acme-actions`) or one HTTP API
(`/api/acme-actions`). ann requested a run on each repository and bob approved each one. All three are
reported `succeeded`, and `DEPRECATED.md` is on each origin's `main`:

| Repository | Run id | Status | Commit on `main` |
|---|---|---|---|
| alpha | `15ba509a-1ea9-4ff1-9032-b1bd94406e18` | succeeded | `6c4948e` |
| beta | `c9759f87-8534-497e-85ea-f40ba10d660a` | succeeded | `220bd56` |
| gamma | `f82b7586-2e90-4ed8-a264-7b9c67ea9e12` | succeeded | `3ec59de` |

Three earlier test runs are also in the record, and that is intended:

| Run id | What it tested | Outcome |
|---|---|---|
| `154f77cf-1b22-4cce-b959-540db5fe06b0` (alpha) | carol's approval and ann's self-approval were both refused, then bob approved while a pre-receive hook rejected the push | `failed` |
| `7c57e18b-62b8-4d3f-8ebf-8d9b292d1211` (beta) | the push was accepted, but a post-receive hook rewound `main` | `failed`, because the change was not observed |
| `7d9fbb0a-c8b0-4c86-a9db-1418e4a1830d` (gamma) | the requester withdrew the run | `rejected`, with `outcome.result` = `withdrawn` |

In all paths below, `$ROOT` is `/tmp/actions-ab/r5/backstage-2`.

## How it works

Backstage provides the catalog, the scaffolder (template execution and dry runs), auth, and the
permission plumbing. The glue is one backend directory, `$ROOT/backstage/packages/backend/src/acme/`,
wired into `$ROOT/backstage/packages/backend/src/index.ts`:

| File | Role |
|---|---|
| `catalogProvider.ts` | A catalog `EntityProvider` (`acme-origins`). It reads `repos.yaml` and `catalog-info.yaml` from `main` on `origins/org.git`. Then, for each listed repository, it reads `catalog-info.yaml` from `main` on that repository's bare origin, follows relative `Location` targets inside the same commit, and emits the entities in a namespace named after the repository. Each entity is stamped with `acme.io/repository`, `acme.io/origin`, `acme.io/source-commit`, `acme.io/source-path` and `acme.io/source-blob`. It refreshes every 15 s, and synchronously before every list, preview, request and approval |
| `scaffolderActions.ts` | The scaffolder actions `acme:repo:commit-file` and `acme:repo:observe-file`, described under R4 and R5 |
| `plugin.ts` | The `acme-actions` backend plugin: the HTTP API, identity, validation, preview, approval rules, dispatch to the scaffolder, the watcher, and the final independent observation of `main` |
| `store.ts` | The run and audit-event tables, through Backstage's database service (SQLite files in `$ROOT/backstage/data/db/`, so the record survives restarts) |
| `git.ts`, `registry.ts` | Git helpers, and the in-process state shared by the three features above |

Configuration is in `$ROOT/backstage/app-config.local.yaml`:
- `acme.orgOrigin` points at `origins/org.git`;
- the database is persistent;
- `catalog.locations` is empty, because everything comes from the origins;
- each person's static token has `accessRestrictions`.

The life of a run:

1. **Request.** The inputs are validated against the repository's template, and the template is run as a scaffolder dry run to produce a preview. The run is stored as `awaiting_approval`.
2. **Approve.** The approver must be a maintainer and must not be the requester. The run moves to `dispatching`, and a single-use nonce is minted.
3. **Execute.** A scaffolder task runs the repository's template, carrying the nonce as a task secret. `commit-file` clones the origin, commits and pushes to `main`. `observe-file` reads `main` back.
4. **Verify.** The plugin waits for the task, then re-reads the origin's `main` itself. The run becomes `succeeded` only if that check passes; otherwise it becomes `failed`.

## Requirements

### R1: owned by the repository

Each repository's definition is committed to its origin's `main`:
- `$ROOT/<repo>/.backstage/actions/mark-deprecated.yaml` is a Backstage scaffolder `Template`. It carries the label `acme.io/action: mark-deprecated` and `spec.owner: group:maintainers`, and it holds the input schema, the file content and the steps;
- `$ROOT/<repo>/catalog-info.yaml` defines the repository's `Component`, its `maintainers` `Group` (ann and bob) and a `Location` that points at the template.

The catalog reads these files from the origin's `main`, not from the checkout, so a change takes effect once it lands on `main`. That includes inputs, content, steps and who the maintainers are. `list` shows the commit and blob each definition was read from. Runs pin the blob they were previewed against: if the definition changes between request and approval, approval is refused with `DefinitionChanged` (in `plugin.ts`, `approve`), and the scaffolder action refuses too (`scaffolderActions.ts`, `approvedRun`). The actions only ever write to the repository that owns the template, as stamped by the provider and never taken from inputs, so one repository's template cannot change another repository.

### R2: validated

`successor` must match `^[a-z0-9-]+/[a-z0-9-]+$` and must not be the repository itself (`acme/<repo>`). `reason` must be 20 to 200 characters long. No other inputs are allowed. These rules are the template's JSON Schema in `.backstage/actions/mark-deprecated.yaml`.

`plugin.ts` (`validateInputs`) checks inputs on preview, on request (before a run is even stored) and again before execution. The scaffolder re-validates the values against the same schema when it creates the task. Invalid inputs get HTTP 400 `InvalidInputs` with a per-field `errors` list, and the CLI exits 1. Nothing is stored or pushed.

### R3: previewed

`preview` (`POST /api/acme-actions/actions/<repo>/<action>/preview`) runs the repository's own template as a Backstage scaffolder dry run (`/api/scaffolder/v2/dry-run`). In dry-run mode, `acme:repo:commit-file` pushes nothing. It reads the current file on the origin's `main` and returns, for each file:
- the repository, origin, branch and path;
- the base commit;
- the `before` and `after` contents;
- whether the content changes;
- a unified `diff`.

`request` runs the same dry run and stores the result in the run (`preview`), so the requester and the approver can see exactly what an approval would change (`record`).

### R4: approved

In `plugin.ts`, `approve` loads the template's owner group from the catalog (the `maintainers` group in the repository's own `catalog-info.yaml`) and enforces two rules:
- the approver must be a member: carol gets 403 `NotAMaintainer`;
- the approver must not be the requester: 403 `SelfApproval`.

Run state must be `awaiting_approval`; anything else gets 409. Every refused attempt is written to the record as an `approval_refused` event.

Approval cannot be bypassed:
- Every person's token has `accessRestrictions` in `app-config.local.yaml`. It may call `acme-actions`, and only read the catalog and scaffolder, so creating scaffolder tasks or dry runs directly, or registering catalog locations, returns 403.
- Even a task created some other way changes nothing. `acme:repo:commit-file` and `acme:repo:observe-file` refuse to run unless the task carries the run id and the single-use nonce minted at approval, which the store keeps only as a hash (`store.ts`, `claimForTask`). The nonce also binds the run to exactly one scaffolder task.

Rejection: `reject` is allowed for a maintainer, or for the requester, who withdraws the run.

### R5: done means observed

There are three gates, and all must pass before a run is reported `succeeded`:
1. `acme:repo:commit-file` pushes to `main`, and a failed or rejected push fails the step (`push_rejected` event). Every run makes its own commit, even when the content is already there, so a rejected push always fails the run.
2. `acme:repo:observe-file` reads the origin's `main` after the push. It fails unless `DEPRECATED.md` has exactly the expected content and `main` contains this run's commit (`observed` or `observation_failed` event).
3. After the scaffolder task completes, the plugin reads the origin's `main` itself (`plugin.ts`, `observe`). It checks every previewed file against its previewed content, and checks that the pushed commit is an ancestor of `main` (`verified_on_main` or `verification_failed` event). Only then does the run become `succeeded`.

Any failure gives `failed`, with `outcome.error`. Two cases were tested: a pre-receive hook rejection, and a post-receive hook that rewound `main` after accepting the push. Both were reported `failed`.

### R6: one place for the org

From `$ROOT/org`:
- `bin/acme-actions list` lists `mark-deprecated` for every repository in `org/repos.yaml`, with owner, maintainers, definition location and input schema;
- `bin/acme-actions preview|request <repo> ...` works on any of them without entering the repository.

`org/repos.yaml` is the org's list of repositories, and `org/catalog-info.yaml` lists the people. Backstage reads both from `main` on `origins/org.git`.

### R7: audited

`record <run-id>` (`GET /api/acme-actions/runs/<id>`) returns:
- `requestedBy` and `requestedAt`;
- `approvedBy` and `approvedAt` (or `rejectedBy` and `rejectedAt`);
- `inputs`, the pinned `definition` and the `preview`;
- `status`, `outcome` (the observed `main` commit, per-file matches, or the error) and `finishedAt`;
- the time-ordered `events`: `requested`, `approval_refused`, `approved`, `dispatched`, `pushed` or `push_rejected`, `observed` or `observation_failed`, `verified_on_main` or `verification_failed`, and `succeeded`, `failed`, `rejected` or `withdrawn`, each with its actor and timestamp;
- the scaffolder task's log, as `scaffolderLog`.

The record is stored in `acme_runs` and `acme_run_events` in `$ROOT/backstage/data/db/acme-actions.sqlite`. Each commit on `main` is also authored by the requester and carries `Requested-by:`, `Approved-by:`, `Backstage-Run:` and `Backstage-Task:` trailers.

### R8: usable by agents

Every step is a non-interactive command that prints JSON on stdout: `org/bin/acme-actions`, which uses only the Python 3 standard library. Its exit codes:

| Exit code | Meaning |
|---|---|
| 0 | ok |
| 1 | refused or failed request; the JSON body says why, and includes `httpStatus` |
| 2 | usage error |
| 4 | `approve` (which waits by default) or `wait` finished, but the run is not `succeeded` |

The same operations are plain HTTP with JSON bodies; the curl equivalents are below. `--compact` prints one line.

## Commands

### Start (must be running)

```bash
export PATH=/tmp/bs/node24/bin:$PATH
cd /tmp/actions-ab/r5/backstage-2/backstage
mkdir -p data/work data/db
tmux new-session -d -s r5-backstage-2-backend \
  "export PATH=/tmp/bs/node24/bin:\$PATH; cd /tmp/actions-ab/r5/backstage-2/backstage && yarn workspace backend start 2>&1 | tee data/backend.log"
until curl -sf -H 'Authorization: Bearer ann-token-0123456789' http://localhost:7470/api/acme-actions/whoami >/dev/null; do sleep 2; done; echo ready
```

Alternatively, run `yarn workspace backend start` in the foreground of a terminal after the `export`, `cd` and `mkdir`. Startup takes about 10–30 s.

To stop: `tmux kill-session -t r5-backstage-2-backend`.

### Use (from the org)

```bash
cd /tmp/actions-ab/r5/backstage-2/org
A=bin/acme-actions            # --as NAME picks that person's sandbox token; --url overrides http://localhost:7470

$A list --as carol                                   # mark-deprecated for alpha, beta, gamma
$A show alpha --as carol                             # one definition, including its input schema

$A preview alpha --as ann \
  --successor acme/dependabot-proxy-v2 \
  --reason "Replaced by the v2 proxy, which supports OIDC registries."

RUN=$($A request alpha --as ann \
  --successor acme/dependabot-proxy-v2 \
  --reason "Replaced by the v2 proxy, which supports OIDC registries." | jq -r .id)

$A approve "$RUN" --as bob          # approves, waits, prints the final status; exit 0 only if succeeded
                                    # (--no-wait returns at once; then: $A wait "$RUN" --as bob)
$A status "$RUN" --as ann
$A record "$RUN" --as ann
$A runs --as ann                    # all runs; filter with --repository alpha --status failed
$A reject "$RUN" --as bob --comment "why"
$A whoami --as bob
```

The same operations over HTTP (`T` is the caller's token):

```bash
B=http://localhost:7470/api/acme-actions; T=ann-token-0123456789
curl -s -H "Authorization: Bearer $T" $B/actions
curl -s -H "Authorization: Bearer $T" -H 'content-type: application/json' -X POST $B/actions/alpha/mark-deprecated/preview \
  -d '{"inputs":{"successor":"acme/x-y","reason":"twenty or more characters here"}}'
curl -s -H "Authorization: Bearer $T" -H 'content-type: application/json' -X POST $B/runs \
  -d '{"repository":"alpha","action":"mark-deprecated","inputs":{"successor":"acme/x-y","reason":"twenty or more characters here"}}'
curl -s -H "Authorization: Bearer bob-token-0123456789" -X POST "$B/runs/$RUN/approve?wait=true"
curl -s -H "Authorization: Bearer $T" $B/runs/$RUN/status
curl -s -H "Authorization: Bearer $T" $B/runs/$RUN
```

## Testing the rules

Each case below was run and behaved as described.

| Case | Command | Expected |
|---|---|---|
| Bad successor or short reason | `$A preview alpha --as ann --successor Acme/X --reason short` (and the same with `request`) | 400 `InvalidInputs`; exit 1; no run stored |
| Successor is the repository itself | `... --successor acme/alpha ...` | 400 `successor must not be "acme/alpha"` |
| Reason over 200 characters | `--reason "$(printf 'x%.0s' {1..201})"` | 400 `InvalidInputs` |
| Unknown input | `--input foo=bar` | 400 `foo is not an input of this action` |
| Self-approval | `$A approve "$RUN" --as ann` (run requested by ann) | 403 `SelfApproval`; recorded as `approval_refused` |
| carol approves | `$A approve "$RUN" --as carol` | 403 `NotAMaintainer`; recorded |
| Rejected push | `printf '#!/bin/sh\nexit 1\n' > ../origins/alpha.git/hooks/pre-receive; chmod +x ../origins/alpha.git/hooks/pre-receive`, then request as ann and `$A approve "$RUN" --as bob`, then `rm ../origins/alpha.git/hooks/pre-receive` | status `failed`, `error` quotes git's rejection; exit 4; `main` unchanged |
| Push accepted, `main` rewound | a `post-receive` hook containing `while read o n r; do git update-ref "$r" "$o"; done` | `failed` (`observation_failed`) |
| Bypass approval | `curl -X POST -H 'Authorization: Bearer ann-token-0123456789' -H 'content-type: application/json' localhost:7470/api/scaffolder/v2/tasks -d '{"templateRef":"template:alpha/mark-deprecated","values":{}}'` | 403 |

The rejected-push test still fails correctly on a repository that is already deprecated, because every run pushes its own commit.

## Limits and notes

- **Identity.** Identity is the sandbox's static token. Backstage treats it as a service principal whose subject is the person's name, and `acme-actions` accepts only subjects that are `User` entities in `org/catalog-info.yaml`.
- **Scaffolder's own record.** The scaffolder's `createdBy` is empty, because tasks are dispatched by `acme-actions`. The audit source is the `acme-actions` record, plus the commit trailers.
- **Definition changes.** Changing a template or maintainers group on a repository's `main` takes effect within 15 s, or immediately on the next CLI call, since every call re-syncs the catalog.
- **Resetting.** Deleting `$ROOT/backstage/data/` wipes the audit record and the catalog cache.
- **Where the code is committed.** The Backstage glue is committed to the app's own git repository (`$ROOT/backstage`, commit `Acme self-service actions…`). `app-config.local.yaml` is gitignored there by create-app, so it is edited in place.
