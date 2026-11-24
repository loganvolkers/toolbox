# Handoff: `mark-deprecated`, a repository-owned self-service action on Backstage

`mark-deprecated` runs on Backstage 1.57. Each repository defines the action as a scaffolder
template on its own origin's `main`. Every run is validated, previewed by a scaffolder dry run,
approved by a maintainer other than the requester, and executed by a custom scaffolder action.
A run is reported `succeeded` only after the system has read `DEPRECATED.md` back from the
origin's `main`. One CLI, `org/bin/acme`, lists, previews, requests, approves and audits runs for
all three repositories. It prints JSON.

The three requested runs (requested by ann, approved by bob) all **succeeded**:

| Repository | Run | Commit on `main` |
|---|---|---|
| alpha | `run-0ed8ff42e9c4` | `f416330` |
| beta | `run-3398dd30a0dd` | `cef4429` |
| gamma | `run-0e7f4e90e771` | `09eb876` |

`DEPRECATED.md` is exactly `# Deprecated: use <successor>`, a blank line, `<reason>`, and one
trailing newline.

## How it fits together

Backstage provides the catalog, the scaffolder (template execution, parameter validation, dry
run, task log) and static-token auth. The glue is three small pieces in
`/tmp/actions-ab/r7/backstage-3/backstage/packages/backend/src/acme/`:

| File | What it adds |
|---|---|
| `gitOriginUrlReader.ts` | A Backstage URL reader for `http://git.acme.local/<repo>/blob/<ref>/<path>`. It serves files from the bare origins with `git show`, so the catalog reads definitions from each origin's `main` |
| `scaffolderModule.ts` | Scaffolder action `acme:git:commit-file`. A dry run computes the diff. A real run needs a one-time approval grant: it clones the origin, commits (author = requester, trailers `Self-service-run`, `Requested-by`, `Approved-by`), pushes to `main`, then reads `main` back and fails unless the file is there with exactly the content |
| `selfServicePlugin.ts` | Backend plugin `self-service` at `/api/self-service`: action list, preview, requests, approvals, dispatch to the scaffolder, verification on `main`, and the audit record (SQLite `backstage/.data/self-service.sqlite`) |
| `git.ts` | Shared git helpers |

They are wired in `backstage/packages/backend/src/index.ts`, and configured in
`backstage/app-config.yaml` under `acme:`, the catalog location and the SQLite database directory.

Data flow: `org/catalog-info.yaml` (on `origins/org.git` `main`) lists people and the three
repositories. Each repository's `catalog-info.yaml` lists its `.backstage/actions/mark-deprecated.yaml`.

## Requirements

| # | How it is met | Files |
|---|---|---|
| R1 Owned by the repository | Each repository's definition is a scaffolder template committed to its origin's `main`, with `spec.owner: group:default/<repo>-maintainers`. Backstage reads it from `origins/<repo>.git` `main` (re-read every 5 s, or now with `acme refresh`). The plugin only accepts a template whose catalog namespace equals the repository it was read from, so one repository cannot define actions for another. The scaffolder action also refuses a step that targets a repository other than the run's. If the definition changes between request and approval, approval is refused (409) and the run must be requested again | `alpha/.backstage/actions/mark-deprecated.yaml`, `beta/...`, `gamma/...`; `<repo>/catalog-info.yaml`; `org/catalog-info.yaml`; `backstage/packages/backend/src/acme/gitOriginUrlReader.ts` |
| R2 Validated | The repository's template declares the input schema: `successor` matches `^[a-z0-9-]+/[a-z0-9-]+$`, `reason` has `minLength: 20` and `maxLength: 200`, both are required, and no other inputs are allowed. The scaffolder validates the inputs against it (in the dry run, and again when the task is created). Invalid inputs return HTTP 400 `ValidationError` with an `errors` list, and no run is created | template files above; `selfServicePlugin.ts` (`preview`, `request`) |
| R3 Previewed | `acme preview` runs a scaffolder dry run. `acme:git:commit-file` reads the current file from `main` and returns `before`, `after`, `changed` and a unified `diff`, and nothing is written. Every request stores its preview, and the approver sees it in the run record (`preview.changes`) | `scaffolderModule.ts` (dry-run branch); `selfServicePlugin.ts` (`preview`) |
| R4 Approved | A run starts as `awaiting-approval`. `approve` is refused (403) for the requester and for anyone who is not a member of the template's owner group in the catalog (carol is not; ann and bob are). Refusals are recorded as `approval-refused` events. Only an approval creates the scaffolder task, with a random one-time grant passed as a task secret. A real `acme:git:commit-file` run refuses to act unless the task was created by the `self-service` plugin and the grant matches an approved, unfinished run for the same repository. A task created directly through `POST /api/scaffolder/v2/tasks` therefore fails without changing anything | `org/catalog/people.yaml` (groups); `selfServicePlugin.ts` (`approve`, `consume`); `scaffolderModule.ts` |
| R5 Done means observed | The action pushes, then reads the origin itself and fails unless its commit is on `main` and the file content matches exactly. When the task completes, the plugin checks independently: every previewed file on `origins/<repo>.git` `main` must equal the approved `after` content, and the run's commit must be an ancestor of `main`. Only then is the run `succeeded`. A rejected push fails the task (status `failed`, `outcome.reason` is git's rejection message), and so does a verification mismatch (`outcome.stage: verify`) | `scaffolderModule.ts`; `selfServicePlugin.ts` (`verify`, `reconcile`) |
| R6 One place for the org | `org/bin/acme` lists every repository's actions (`acme actions --name mark-deprecated`) and requests, approves and reads runs on any repository by name, from `/tmp/actions-ab/r7/backstage-3/org`. The org's people and repository list are `org/catalog-info.yaml` and `org/catalog/people.yaml` | `org/bin/acme`, `org/catalog-info.yaml`, `org/catalog/people.yaml` |
| R7 Audited | Each run row stores `requestedBy/At`, `approvedBy/At` (or `rejectedBy/At`), `inputs`, `startedAt`, `finishedAt`, `outcome` (result, commit, the `main` commit, observed blob, reason). Plus an append-only event list (`requested`, `previewed`, `approval-refused`, `approved`, `started`, `executing`, `pushed`, `succeeded`/`failed`/`rejected`, each with `at` and `actor`; `actor: null` is the self-service plugin itself), the stored preview, the definition's source (URL, blob, origin commit) and the scaffolder task log. Records persist in `backstage/.data/self-service.sqlite`. The commit on `main` also names the run, requester and approver | `selfServicePlugin.ts` (`runs`, `run_events` tables, `view`) |
| R8 Usable by agents | Every step is one non-interactive command that prints JSON. Exit codes: 0 ok, 1 refused or failed, 2 usage, 3 Backstage down. `acme wait` exits 0 only on `succeeded`. The same operations are plain HTTP+JSON under `/api/self-service` with `Authorization: Bearer <token>` | `org/bin/acme`; HTTP API below |

## Commands

Everything below assumes:

```bash
export PATH=/tmp/bs/node24/bin:$PATH
cd /tmp/actions-ab/r7/backstage-3/org
```

### Start (and stop) Backstage

```bash
tmux new-session -d -s r7-backstage-3-backend -c /tmp/actions-ab/r7/backstage-3/backstage \
  'export PATH=/tmp/bs/node24/bin:$PATH; yarn workspace backend start 2>&1 | tee /tmp/actions-ab/r7/backstage-3/backstage/.backend.log'
# ready when this returns (about 15 s):
until curl -sf -H 'Authorization: Bearer ann-token-0123456789' http://localhost:7910/api/self-service/actions >/dev/null; do sleep 2; done

# stop:
tmux kill-session -t r7-backstage-3-backend
```

Without tmux: `cd /tmp/actions-ab/r7/backstage-3/backstage && yarn workspace backend start`.
State (catalog, scaffolder tasks, run records) is in `backstage/.data/`. To start from scratch,
stop the backend and delete that directory.

### Use it

Pick who you are with `--as ann|bob|carol`, which uses that person's static token from
`backstage/app-config.local.yaml`. Alternatively, set `ACME_TOKEN=<token>`.

```bash
# list mark-deprecated on all repositories (definition source, approvers, input schema)
bin/acme --as ann actions --name mark-deprecated
bin/acme --as ann show alpha mark-deprecated

# preview: validates and shows the diff; changes nothing
bin/acme --as ann preview alpha mark-deprecated \
  --successor acme/dependabot-proxy-v2 --reason "Replaced by the v2 proxy, which supports OIDC registries."

# request as ann (prints the run, including .id, status awaiting-approval and the preview)
RUN=$(bin/acme --as ann request alpha mark-deprecated \
  --successor acme/dependabot-proxy-v2 --reason "Replaced by the v2 proxy, which supports OIDC registries." | jq -r .id)

# approve as bob (dispatches the run)
bin/acme --as bob approve "$RUN" --comment "LGTM"

# status (any person); wait blocks until succeeded/failed/rejected, exit 0 only if succeeded
bin/acme --as ann status "$RUN"
bin/acme --as ann wait "$RUN" --timeout 120

# full audit record
bin/acme --as ann record "$RUN"

# other
bin/acme --as bob reject "$RUN" --comment "why"   # a maintainer, or the requester withdrawing
bin/acme --as ann runs --repo alpha --status succeeded
bin/acme --as ann refresh                            # re-read definitions from the origins now
```

Inputs can also be given as `--set key=value` (repeatable) or `--inputs '{"successor":"…","reason":"…"}'`.

### The same over HTTP

```bash
T='Authorization: Bearer ann-token-0123456789'; B=http://localhost:7910/api/self-service
curl -s -H "$T" "$B/actions?name=mark-deprecated"
curl -s -H "$T" -H 'content-type: application/json' -X POST $B/preview \
  -d '{"repository":"beta","action":"mark-deprecated","inputs":{"successor":"acme/x","reason":"twenty or more characters"}}'
curl -s -H "$T" -H 'content-type: application/json' -X POST $B/runs \
  -d '{"repository":"beta","action":"mark-deprecated","inputs":{"successor":"acme/x","reason":"twenty or more characters"}}'
curl -s -H 'Authorization: Bearer bob-token-0123456789' -H 'content-type: application/json' -X POST $B/runs/<id>/approve -d '{}'
curl -s -H "$T" $B/runs/<id>/status
curl -s -H "$T" $B/runs/<id>
```

## How to test the negative paths

Each of these leaves `main` unchanged. Check with
`git --git-dir=/tmp/actions-ab/r7/backstage-3/origins/<repo>.git log --oneline -1 main`.

```bash
# invalid inputs -> exit 1, HTTP 400 ValidationError, no run created
bin/acme --as ann request beta mark-deprecated --successor "Acme/Bad Name" --reason "too short"
bin/acme --as ann request beta mark-deprecated --successor acme/x --reason "$(printf 'x%.0s' $(seq 201))"
bin/acme --as ann request beta mark-deprecated --set successor=acme/x   # reason missing

# self-approval and carol -> exit 1, HTTP 403, recorded as approval-refused
RUN=$(bin/acme --as ann request gamma mark-deprecated --successor acme/x --reason "A test request, to be refused." | jq -r .id)
bin/acme --as ann approve "$RUN"
bin/acme --as carol approve "$RUN"
bin/acme --as carol request gamma mark-deprecated --successor acme/x --reason "carol may request; ann or bob approves"  # allowed

# bypassing approval through the scaffolder API -> the task fails, nothing is pushed
curl -s -X POST http://localhost:7910/api/scaffolder/v2/tasks -H 'Authorization: Bearer ann-token-0123456789' \
  -H 'content-type: application/json' \
  -d '{"templateRef":"template:gamma/mark-deprecated","values":{"successor":"acme/x","reason":"Skipping the approval step."}}'

# rejected push -> run reported failed with git's message, never succeeded
H=/tmp/actions-ab/r7/backstage-3/origins/alpha.git/hooks/pre-receive
printf '#!/bin/sh\necho "main is frozen" >&2\nexit 1\n' > $H && chmod +x $H
RUN=$(bin/acme --as ann request alpha mark-deprecated --successor acme/x --reason "Testing a rejected push to main." | jq -r .id)
bin/acme --as bob approve "$RUN"
bin/acme --as ann wait "$RUN"; echo "exit=$?"    # status failed, outcome.reason "Push to alpha@main was rejected: ..."; exit 1
rm $H
```

These were run before the three real runs. Their records are in `acme runs`:
`run-b923def8285d` was self-approval and carol (both refused), then withdrawn.
`run-fdbdddd35505` was a rejected push (failed).
`run-287a8adf4e66` was carol's own request, which she withdrew.

## Notes and limits

- Identity is the static token's subject (`ann`, `bob`, `carol`), mapped to `user:default/<subject>`.
  It must be a User in the catalog. Plugin and service tokens cannot request or approve.
- Every run makes a commit, even when `DEPRECATED.md` already has the requested content (that
  commit is empty), so a rerun always pushes and a rejected push is always reported as `failed`.
- Who may approve comes from the catalog. The template's `spec.owner` is chosen by the repository;
  that group's members are set in `org/catalog/people.yaml`.
- The backend is the development server (`backstage-cli package start`), so error bodies include
  stack traces. The CLI strips them.
- A run whose backend is restarted mid-execution stays `running`, because the scaffolder does not
  resume that task. Requests that are only awaiting approval survive restarts.
