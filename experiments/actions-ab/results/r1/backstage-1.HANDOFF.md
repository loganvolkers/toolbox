# Handoff: `mark-deprecated` self-service action on Backstage

`mark-deprecated` runs on Backstage 1.57. It is a scaffolder Template that each repository
commits to its own origin's `main`. A small backend plugin, `self-service`, adds what Backstage
lacks:
- a request record;
- approval by a maintainer other than the requester;
- an audit trail;
- success only after the change has been observed on the origin's `main`.

Two custom scaffolder actions do the push and the check. One CLI, `org/bin/acme`, drives all
of it from the org repository with JSON output.

All three runs requested by ann and approved by bob were reported **succeeded**:

| Repository | Run id | Commit on origin `main` |
|---|---|---|
| alpha | `61b2ea13-9d4f-45f3-b54c-d477b85b191f` | `6db55b6` |
| beta | `2296e4d1-3fff-4d15-b536-da021a9f2414` | `f73a9fa` |
| gamma | `6cae03f9-142c-4f40-b3fa-2cc206693e10` | `19ab853` |

The database also keeps the test runs I made while building, all on alpha, plus one on beta:
- two failed runs: a rejected push, and a push accepted but reverted on `main`;
- one rejected run, requested by carol on beta;
- two scaffolder tasks that tried to bypass approval and were refused.

`acme runs` lists them.

## Pieces

All paths are relative to `/tmp/actions-ab/r1/backstage-1`.

| Piece | Path | Box or tool |
|---|---|---|
| Org registry: which repositories exist and where their origins are | `org/repositories.yaml` | config |
| People (catalog Users ann, bob, carol) | `org/catalog/people.yaml` | config |
| Each repository's action definition | `<repo>/.backstage/templates/mark-deprecated.yaml` | repository-owned |
| Each repository's component and its `maintainers` group (the approvers) | `<repo>/catalog-info.yaml` | repository-owned |
| Catalog module: reads the registry from `origins/org.git` `main`, then each repo's files from its origin `main`, every 5 s | `backstage/packages/backend/src/acme/catalogOrgOrigins.ts` | Backstage glue |
| Scaffolder actions `acme:git:commit-file` (dry-run = diff; real = guarded commit + push) and `acme:git:verify-file` | `backstage/packages/backend/src/acme/scaffolderGitActions.ts` | Backstage glue |
| `self-service` backend plugin: `/api/self-service` (list, preview, request, approve, reject, status, record) | `backstage/packages/backend/src/acme/selfServicePlugin.ts` | Backstage glue |
| git helpers (read/observe a bare origin) | `backstage/packages/backend/src/acme/git.ts` | Backstage glue |
| Registration of the three modules | `backstage/packages/backend/src/index.ts` (end of file) | Backstage glue |
| Config: port 7130, static tokens, persistent SQLite in `backstage/.data/`, org origin, catalog rules | `backstage/app-config.local.yaml` | config |
| CLI | `org/bin/acme` (Node, no dependencies) | org front door |
| Sandbox tokens for `--as` | `org/.acme/tokens.json` (the same tokens as `app-config.local.yaml`) | config |

## Requirements

| # | How it is met | Files |
|---|---|---|
| R1 Owned by the repository | The Template is committed to each repository's origin `main` in `.backstage/templates/mark-deprecated.yaml`. Its owner is `group:maintainers`, defined in the same repository's `catalog-info.yaml` (namespace = repository, e.g. `group:alpha/maintainers`). The catalog module reads these files **from the origin's `main`**, not from a checkout, and re-reads every 5 s. A commit to `main` that changes the inputs, steps or maintainers takes effect on the next refresh, with no Backstage change. `acme list` shows each definition's origin, path and commit. Repositories may only contribute Component, Group and Template entities, and only into their own namespace. | `<repo>/.backstage/templates/mark-deprecated.yaml`, `<repo>/catalog-info.yaml`, `backstage/packages/backend/src/acme/catalogOrgOrigins.ts` |
| R2 Validated | The Template's JSON Schema has these rules: `successor` must match `^[a-z0-9-]+/[a-z0-9-]+$` and must not name the repository itself; `reason` must be 20 to 200 characters; both are required; no other inputs are allowed. Preview and request both go through the scaffolder's `/v2/dry-run`, which validates against that schema and answers 400. The plugin then returns `400 {"error":"invalid inputs","errors":[…]}` and creates no run. Nothing is pushed. | `<repo>/.backstage/templates/mark-deprecated.yaml`, `selfServicePlugin.ts` (`preview`) |
| R3 Previewed | `acme preview` runs the repository's own Template as a scaffolder dry-run. `acme:git:commit-file` clones the origin's `main`, writes the file and returns the exact content and the unified `git diff`, without pushing. `acme request` stores that preview in the run, so the requester and the approver see it in `acme record` before approval. At approval the plugin re-runs the preview. If the definition now produces a different file, the approval is refused with 409 and the run must be requested again. | `scaffolderGitActions.ts`, `selfServicePlugin.ts` |
| R4 Approved | `POST /runs/:id/approve` identifies the caller from the token. It refuses with 403 when the caller is the requester. It also refuses with 403 when the caller is not in the members of the Template owner's group: carol is refused this way. A run that is not `awaiting_approval` gets 409. Every refused attempt is recorded as an `approval-refused` event. The scaffolder task is created only after approval, by the plugin. `acme:git:commit-file` refuses to push in four cases: the task was not dispatched by `plugin:self-service`; the run is not approved; the approver is the requester; the file differs from the previewed one. A direct `POST /api/scaffolder/v2/tasks`, with a user token or a guest login, therefore changes nothing. | `selfServicePlugin.ts` (`approve`), `scaffolderGitActions.ts` (`authorizeRun`) |
| R5 Done means observed | The Template's last step, `acme:git:verify-file`, fails the task unless `main` on the origin has `DEPRECATED.md` with the exact content. The plugin does not trust the task alone. After the task ends, it reads `refs/heads/main:DEPRECATED.md` from the bare origin itself and compares it with the previewed content. It reports `succeeded` only if the task completed **and** that observation matched. Otherwise it reports `failed` with the error and the observation. Tested both ways: a `pre-receive` hook that rejects the push, and a `post-receive` hook that resets `main` after accepting the push. Both were reported `failed`. | `scaffolderGitActions.ts`, `selfServicePlugin.ts` (`monitor`), `git.ts` (`observeOriginFile`) |
| R6 One place for the org | From `org/`, `bin/acme list` lists `mark-deprecated` for alpha, beta and gamma, and `bin/acme request <repo>` requests a run on any of them. The set of repositories is `org/repositories.yaml` on the org origin's `main`. | `org/bin/acme`, `org/repositories.yaml` |
| R7 Audited | Each run is stored in SQLite (`backstage/.data/self-service.sqlite`, tables `self_service_runs` and `self_service_events`) and survives restarts. `acme record <id>` returns the requester and `requestedAt`, the approver and `approvedAt` (or rejecter and `rejectedAt`), the inputs, the definition's origin/path/commit, the preview, and the outcome with `at`, the observed commit and any error. It also returns a timestamped event list with actors (`requested`, `previewed`, `approval-refused`, `approved`, `dispatched`, `observed`, `succeeded` or `failed`, `rejected`) and the scaffolder task log. The commit on `main` is authored by the requester and carries `Requested-by`, `Approved-by` and `Self-service-run` trailers. | `selfServicePlugin.ts`, `scaffolderGitActions.ts` |
| R8 Usable by agents | Every step is a non-interactive CLI command that prints one JSON document on stdout. Exit codes: 0 ok, 1 refused or failed (JSON error with `httpStatus`), 2 usage, 3 wait timed out. `acme wait <id>` exits 0 only if the run succeeded. The same operations are plain HTTP+JSON under `http://localhost:7130/api/self-service` (see the table below). | `org/bin/acme`, `selfServicePlugin.ts` |

## Commands

### Start (must be running)

```sh
export PATH=/tmp/bs/node24/bin:$PATH
cd /tmp/actions-ab/r1/backstage-1/backstage
yarn workspace backend start          # listens on http://localhost:7130; Ctrl-C stops it
```

The backend takes about 20 to 40 s to come up. The catalog fills within about 5 s after that.
This loop waits until all three actions are listed:

```sh
cd /tmp/actions-ab/r1/backstage-1/org
until bin/acme --as ann list 2>/dev/null | grep -q '"repository": "gamma"'; do sleep 2; done
```

### Use

Run these from `/tmp/actions-ab/r1/backstage-1/org`, with `PATH` set as above. Identity is
`--as ann|bob|carol`, which looks up that person's static token, or `--token <token>`, or the
`ACME_TOKEN` environment variable.

```sh
# list the actions in every repository (inputs schema, approvers, definition origin/commit)
bin/acme --as ann list

# preview: validates and shows the exact file and diff; changes nothing
bin/acme --as ann preview alpha --successor acme/dependabot-proxy-v2 \
  --reason "Replaced by the v2 proxy, which supports OIDC registries."

# request as ann (validated + previewed; prints the run, including its "id")
RUN=$(bin/acme --as ann request alpha --successor acme/dependabot-proxy-v2 \
  --reason "Replaced by the v2 proxy, which supports OIDC registries." \
  | node -pe 'JSON.parse(require("fs").readFileSync(0)).id')

# approve as bob
bin/acme --as bob approve "$RUN"

# check status (awaiting_approval | running | succeeded | failed | rejected)
bin/acme --as bob status "$RUN"
bin/acme --as bob wait "$RUN"            # blocks until done; exit 0 only if succeeded

# read the full audit record
bin/acme --as bob record "$RUN"

# other
bin/acme --as ann runs [--repository alpha] [--status failed]
bin/acme --as bob reject "$RUN" --comment "why"
bin/acme --as ann show beta              # one action
bin/acme --as ann request gamma --inputs '{"successor":"acme/x","reason":"twenty or more characters"}'
```

### Negative tests

```sh
# invalid inputs -> exit 1, httpStatus 400, no run created
bin/acme --as ann request alpha --successor Acme/X --reason short
bin/acme --as ann request alpha --successor acme/alpha --reason "points at itself, which is refused"
bin/acme --as ann request alpha --successor acme/x \
  --reason "$(printf 'x%.0s' $(seq 201))"     # 201 characters

# self-approval and carol -> exit 1, httpStatus 403; the run stays awaiting_approval
RUN=$(bin/acme --as ann request beta --successor acme/valtio-store \
  --reason "State management moves to the shared store package." \
  | node -pe 'JSON.parse(require("fs").readFileSync(0)).id')
bin/acme --as ann approve "$RUN"
bin/acme --as carol approve "$RUN"

# rejected push -> run reported failed, with the hook's message
printf '#!/bin/sh\necho "main is protected" >&2\nexit 1\n' > ../origins/beta.git/hooks/pre-receive
chmod +x ../origins/beta.git/hooks/pre-receive
bin/acme --as bob approve "$RUN"; bin/acme --as bob wait "$RUN"; echo "exit $?"   # failed, exit 1
rm ../origins/beta.git/hooks/pre-receive

# bypassing the plugin -> the scaffolder task fails at the push step; main unchanged
curl -s -X POST localhost:7130/api/scaffolder/v2/tasks -H 'Authorization: Bearer ann-token-0123456789' \
  -H 'Content-Type: application/json' \
  -d '{"templateRef":"template:alpha/mark-deprecated","values":{"successor":"acme/x","reason":"no approval at all here"}}'
```

The repositories are already deprecated. A new run whose content matches the current file
pushes nothing (`changed: false`) and still succeeds after observation. A run with a different
successor or reason replaces the file.

### HTTP API

Send `Authorization: Bearer <token>` and `Content-Type: application/json`. Base URL:
`http://localhost:7130/api/self-service`.

| Step | Request |
|---|---|
| list | `GET /actions[?repository=alpha]` |
| show | `GET /actions/:repository/:action` |
| preview | `POST /actions/:repository/:action/preview` `{"inputs":{…}}` |
| request | `POST /actions/:repository/:action/runs` `{"inputs":{…}}` (or `POST /runs` `{"repository","action","inputs"}`) |
| approve / reject | `POST /runs/:id/approve` / `POST /runs/:id/reject` `{"comment":…}` |
| status | `GET /runs/:id/status` |
| record | `GET /runs/:id` |
| runs | `GET /runs[?repository=&status=&action=]` |
| whoami | `GET /whoami` |

## Notes and limits

- **Content.** `DEPRECATED.md` is `# Deprecated: use <successor>\n\n<reason>\n`: the specified
  three lines with a final newline.
- **Identity.** There are no passwords in this sandbox, so identity is the static token. Each
  token maps to a service principal whose subject is the person's name. The plugin requires that
  name to be a catalog User. Backstage's own `createdBy` stays empty for these principals, so the
  plugin's record is the audit trail of who requested and who approved.
- **Persistence.** State lives in `backstage/.data/` (one SQLite file per plugin, plus the
  scaffolder working directory). A run left `running` by a restart is picked up again when the
  backend starts.
- **Process.** The backend was stopped at the end of this session; nothing is left running.
