# Handoff: `mark-deprecated`, a repository-owned self-service action on Backstage

`mark-deprecated` is a Backstage scaffolder template that lives in each repository. A small
Backstage backend plugin, `acme-actions`, adds what Backstage lacks: a stored preview, approval
by another maintainer, an audit record, and a success state that is set only after the file is
seen on the origin's `main`. One CLI in this directory, `bin/acme`, drives every step for all
repositories and prints JSON.

All three runs succeeded (requested by ann, approved by bob):

| Repository | Run id | `main` after the run |
|---|---|---|
| alpha | `e0fd5628-c4ab-4ac4-a97b-10e2180bb0c9` | `ddaff7e` Mark alpha deprecated: use acme/dependabot-proxy-v2 |
| beta | `12f45b0c-7abd-4e34-a8b2-5062e60164e8` | `540f204` Mark beta deprecated: use acme/valtio-store |
| gamma | `a7981c90-fc9e-43e2-9fda-22a0218cf0ca` | `a773252` Mark gamma deprecated: use acme/graphify-core |

Three earlier alpha runs are kept in the audit trail on purpose. They were my tests of the
failure paths and changed nothing on `main`:

- `d2b0d426-…`: self-approval and carol were refused, then bob rejected the run.
- `703bfd9f-…`: the push was rejected by a `pre-receive` hook, and the run is `failed`.
- `bf687c9f-…`: the push was accepted, then a `post-receive` hook reset `main`, and the run is `failed`.

## Where things are

All paths are under `/tmp/actions-ab/r2/backstage-2`.

| Path | What it is |
|---|---|
| `<repo>/catalog-info.yaml` (repo = alpha, beta, gamma) | The repository's Component and its `maintainers` Group (ann, bob). The group owns the action and approves its runs |
| `<repo>/.backstage/actions/mark-deprecated.yaml` | The action: a scaffolder Template with the input schema, the file content template and the steps |
| `org/repositories.yaml` | The repositories Backstage reads, with their origins |
| `org/catalog/people.yaml` | Users ann, bob and carol |
| `org/bin/acme` | The CLI: one place for list, show, preview, request, approve, reject, status, record and runs |
| `org/.acme/tokens.json` | Local token map for `--as` (gitignored; recreate it as shown below) |
| `backstage/plugins/acme-actions-backend/src/catalogModule.ts` | Catalog module: an entity provider that reads the org and each repository from its origin's `main` every 3 s |
| `backstage/plugins/acme-actions-backend/src/scaffolderModule.ts` | Scaffolder actions `acme:fs:write` and `acme:git:push-to-main` |
| `backstage/plugins/acme-actions-backend/src/service.ts` | The `acme-actions` plugin: preview, request, approve, reject, claim, monitor and observe |
| `backstage/plugins/acme-actions-backend/src/router.ts` | HTTP API at `/api/acme-actions` |
| `backstage/plugins/acme-actions-backend/src/store.ts` | Run and event tables (`acme_runs`, `acme_run_events`) |
| `backstage/packages/backend/src/index.ts` | Adds the plugin and both modules to the backend |
| `backstage/app-config.local.yaml` | Port 7470, the static tokens, file-backed SQLite in `backstage/.data/`, `acme.orgOrigin`, catalog rules |

## R1 to R8

| # | How it is met | Files |
|---|---|---|
| R1 Owned by the repository | Each repository's action is a scaffolder Template committed to that repository's origin `main` at `.backstage/actions/mark-deprecated.yaml`. Its owner is the repository's own `maintainers` group in `catalog-info.yaml`. The catalog module reads only from origin `main`, so a change lands when the team pushes it (picked up within 3 s; `list` shows, in `definedIn.commit`, the `main` commit the definition was read from, and each run records it). The provider forces every entity from a repository into that repository's namespace and stamps its origin, so one repository cannot define or redirect another's action | `alpha/.backstage/actions/mark-deprecated.yaml` (and beta, gamma), `alpha/catalog-info.yaml`, `backstage/plugins/acme-actions-backend/src/catalogModule.ts` |
| R2 Validated | The template declares the schema: `successor` matches `^[a-z0-9-]+/[a-z0-9-]+$`, `reason` has `minLength: 20` and `maxLength: 200`, both are required, and `additionalProperties: false`. Preview and request first run Backstage's own scaffolder validation (`POST /api/scaffolder/v2/dry-run`). Invalid inputs return HTTP 400 with the schema errors, no run is stored, and nothing is pushed | template files above; `RunService.preview` in `service.ts` |
| R3 Previewed | `acme preview` (and `acme request`, which stores the same preview in the run) runs the template as a scaffolder dry run. It returns each file the run would write, whether it is `create`, `modify` or `unchanged` against the current origin `main`, the exact content and a unified diff. The approved preview is pinned: before pushing, `acme:git:push-to-main` sends the sha256 of every file it is about to commit, and the plugin refuses if they differ from the preview (for example, if the template changed after the request) | `RunService.preview` and `RunService.claim` in `service.ts`; `scaffolderModule.ts` |
| R4 Approved | A request is stored as `awaiting-approval`. `approve` is refused with 403 if the approver is the requester, or is not a member of the template's owner group (read fresh from the catalog at approval time; carol is in no group). Only an approval starts the scaffolder task, with a one-time nonce. The push action works only when the plugin releases the run for that nonce. A task started directly through the scaffolder API, or a second execution of the same run, is refused before git is touched. Refused attempts are recorded as `approval-refused` events | `RunService.approve`, `RunService.claim` in `service.ts`; `createPushToMainAction` in `scaffolderModule.ts` |
| R5 Done means observed | The push action clones `main`, commits, pushes, and fails if the push is rejected or if `main` does not then hold the expected bytes. Independently, the plugin marks a run `succeeded` only when the scaffolder task is `completed` **and** its own read of `refs/heads/main` in the origin shows every previewed file with the approved sha256. Anything else is `failed`, with the reason (for example `Push to main was rejected: remote: …`). The observation (commit and per-file sha256) is stored in the record | `RunService.check` and `RunService.observe` in `service.ts`; `scaffolderModule.ts` |
| R6 One place for the org | `org/bin/acme` lists `mark-deprecated` for every repository in `org/repositories.yaml` and requests, approves and inspects runs on any of them through Backstage. No step needs a repository checkout | `org/bin/acme`, `org/repositories.yaml` |
| R7 Audited | Each run stores who requested it and when, who approved it and when, its inputs, the template ref and the commit that defined it, the preview, the scaffolder task id, and the outcome with its time and observation. It also keeps an event log with a timestamp and actor for every step, including refused approvals. Records are in SQLite under `backstage/.data/` and survive restarts. The commit on `main` carries `Acme-Run`, `Requested-by` and `Approved-by` trailers and is authored by the requester | `store.ts`, `RunService.record` in `service.ts` |
| R8 Usable by agents | Every step is a non-interactive `bin/acme` command or a plain HTTP call that prints JSON. Exit codes: 0 ok, 1 refused or failed (HTTP 4xx/5xx), 2 usage error, 3 `status --wait` ended without success | `org/bin/acme`, `router.ts` |

## Commands

### Start Backstage (required)

```bash
export PATH=/tmp/bs/node24/bin:$PATH
cd /tmp/actions-ab/r2/backstage-2/backstage
yarn workspace backend start                # foreground; Ctrl-C stops it
# or in the background:
nohup yarn workspace backend start > .data/backend.log 2>&1 &
# wait until it is ready (seconds with a warm build cache; allow a few minutes on a cold one):
until curl -sf -o /dev/null http://localhost:7470/.backstage/health/v1/readiness; do sleep 3; done; echo ready
```

Stop the background backend with
`pkill -f '/tmp/actions-ab/r2/backstage-2/backstage/node_modules/@backstage/cli/bin/backstage-cli package start'`
(the full path matters: it leaves other Backstage instances on the machine alone). Dependencies
are already installed. After a fresh clone, run `yarn install` first.

### Use it

Run these from the org directory. `--as` picks the person. The token map is gitignored; this
recreates it:

```bash
export PATH=/tmp/bs/node24/bin:$PATH
cd /tmp/actions-ab/r2/backstage-2/org
mkdir -p .acme && echo '{"ann":"ann-token-0123456789","bob":"bob-token-0123456789","carol":"carol-token-0123456789"}' > .acme/tokens.json
# alternatives: ACME_TOKEN_ANN=ann-token-0123456789 ..., or --token <token>

bin/acme list --as carol                                   # mark-deprecated for alpha, beta, gamma
bin/acme show alpha --as carol                             # inputs schema, owner, approvers
bin/acme preview alpha --as ann --successor acme/dependabot-proxy-v2 \
  --reason "Replaced by the v2 proxy, which supports OIDC registries."
ID=$(bin/acme request alpha --as ann --successor acme/dependabot-proxy-v2 \
  --reason "Replaced by the v2 proxy, which supports OIDC registries." | jq -r .id)
bin/acme approve "$ID" --as bob
bin/acme status "$ID" --as ann                             # status, terminal, succeeded, message
bin/acme status "$ID" --as ann --wait --timeout 120        # blocks; exit 0 only if succeeded
bin/acme record "$ID" --as ann                             # full audit record with events
bin/acme runs --as ann --repo alpha                        # runs, newest first
bin/acme reject "$ID" --as bob --comment "not now"         # only while awaiting approval
```

`<action>` defaults to `mark-deprecated`, so `bin/acme preview alpha mark-deprecated …` is
equivalent. Inputs can also be passed as `--inputs '{"successor":"…","reason":"…"}'`.

The same calls over HTTP (`Authorization: Bearer <token>`, JSON bodies):

| Step | Call |
|---|---|
| list | `GET /api/acme-actions/actions` |
| show | `GET /api/acme-actions/actions/<repo>/<action>` |
| preview | `POST /api/acme-actions/preview` `{"repo":"alpha","action":"mark-deprecated","inputs":{…}}` |
| request | `POST /api/acme-actions/runs` (same body) |
| approve / reject | `POST /api/acme-actions/runs/<id>/approve` or `/reject` `{"comment":"…"}` |
| status | `GET /api/acme-actions/runs/<id>/status` |
| record | `GET /api/acme-actions/runs/<id>` |
| runs | `GET /api/acme-actions/runs?repo=&status=` |

Run states are `awaiting-approval`, then `approved` and `running`, ending in `succeeded`,
`failed` or `rejected`.

## Testing the failure paths

```bash
# Invalid inputs: HTTP 400, exit 1, no run stored
bin/acme preview alpha --as ann --successor Acme/Bad --reason "long enough reason text here"
bin/acme request alpha --as ann --successor acme/ok --reason "too short"
bin/acme request alpha --as ann --successor acme/ok --reason "$(printf 'a%.0s' $(seq 201))"

# Self-approval and carol: 403, the run stays awaiting-approval, attempts are in the record
ID=$(bin/acme request beta --as ann --successor acme/other-store --reason "A different reason so the file changes." | jq -r .id)
bin/acme approve "$ID" --as ann      # 403: requester cannot approve
bin/acme approve "$ID" --as carol    # 403: not a member of group:beta/maintainers
bin/acme record "$ID" --as ann | jq .events

# Rejected push: the run is failed, never succeeded
printf '#!/bin/sh\necho "main is protected" >&2\nexit 1\n' > ../origins/beta.git/hooks/pre-receive
chmod +x ../origins/beta.git/hooks/pre-receive
bin/acme approve "$ID" --as bob
bin/acme status "$ID" --as ann --wait     # exit 3, status failed, message "Push to main was rejected: …"
rm ../origins/beta.git/hooks/pre-receive

# Bypassing approval through the scaffolder API directly: the task fails before any git operation
curl -s -H 'Authorization: Bearer carol-token-0123456789' -H 'content-type: application/json' \
  -X POST http://localhost:7470/api/scaffolder/v2/tasks \
  -d '{"templateRef":"template:alpha/mark-deprecated","values":{"successor":"acme/x","reason":"Skipping the approval step."}}'
```

For the rejected-push test, use inputs whose file differs from what is already on `main`. If
`DEPRECATED.md` already has exactly the requested content, the preview says `unchanged`, no push
is needed, and the run succeeds once the content is observed on `main`. That is the intended
reading of "done means observed".

## Notes and limits

- `DEPRECATED.md` is `# Deprecated: use <successor>`, a blank line, `<reason>`, and a final
  newline.
- Identity: the static tokens map to principals with subject `ann`, `bob` or `carol`, which
  `acme-actions` treats as `user:default/<subject>`. Backstage's permission framework is still
  allow-all and grants service principals everything, so approval is enforced in `acme-actions`
  and in the push action, not by permission policies.
- Anyone with a token can start a scaffolder task from these templates through the scaffolder
  API, but `acme:git:push-to-main` refuses to run without an approved, unexecuted run whose
  preview matches. Nothing reaches an origin that way.
- The changes to the Backstage app (`backstage/plugins/acme-actions-backend`,
  `packages/backend`, `app-config.local.yaml`) are in its working tree and not committed. That
  repository had no origin and already had uncommitted setup changes.
