# Handoff: `mark-deprecated` on Backstage

`mark-deprecated` is a Backstage scaffolder template that each repository commits to its own origin's
`main`. Around the scaffolder sits a small Backstage plugin, `acme-actions`, which adds four things
it lacks: preview, approval by another maintainer, success decided by observing `main`, and a run
record. One CLI in this repository, `bin/acme`, drives every step with JSON output.

All three runs requested by ann and approved by bob **succeeded**:

| Repository | Run id | Commit on `main` |
|---|---|---|
| alpha | `1eb3a61c-f780-47bf-9935-a843a12f3b65` | `a049b70` |
| beta | `e45ca9ee-16ff-426c-a22d-ad11b52ecbc2` | `8cc1669` |
| gamma | `4eeb9fc6-7518-438b-a554-d95daab9d073` | `a1cc65a` |

There is also one earlier test run on alpha, `aaeed306-aa03-4cff-a6e6-ff7ed86e1873`. Its push was
rejected by a temporary `pre-receive` hook, so it is reported as `failed`. It also records the two
refused approvals: ann's self-approval and carol's.

## How it fits together

| Piece | Path | What it does |
|---|---|---|
| The action's definition, one per repository | `<repo>/.backstage/actions/mark-deprecated.yaml` on `origins/<repo>.git` `main` | Scaffolder `Template`: the input schema (validation), the file it writes, owner = `group:default/<repo>-maintainers` |
| Repository catalog entry | `<repo>/catalog-info.yaml` | `Component` owned by the same group |
| Repository registry | `org/repos.yaml` on `origins/org.git` `main` | Which repositories exist and where their origins are |
| People and maintainer groups | `org/catalog/people.yaml` on `origins/org.git` `main` | Users ann, bob, carol; groups `alpha-`, `beta-` and `gamma-maintainers` (ann, bob); `developers` (carol) |
| Catalog provider | `backstage/plugins/acme-actions-backend/src/catalogModule.ts` | Every 5 s, reads the files above **from each origin's `main`** (never a checkout) into the catalog. Entities from a repository go in a namespace named after it, so the action's ref is `template:alpha/mark-deprecated`. Each entity is annotated with its origin, path, commit and blob id |
| Scaffolder action `acme:repo:commit-file` | `backstage/plugins/acme-actions-backend/src/scaffolderModule.ts` | Dry run: computes the diff against `main`. Real run: asks `acme-actions` to authorize the run (one-time token), clones, commits, pushes to `main`, then reads `main` back and fails unless the file is there |
| Run service `acme-actions` | `backstage/plugins/acme-actions-backend/src/plugin.ts`, `src/store.ts`, `src/git.ts` | HTTP API at `/api/acme-actions`: list, preview, request, approve, status, record. Stores runs and an append-only event log |
| Backend wiring | `backstage/packages/backend/src/index.ts`, `backstage/packages/backend/package.json` | Adds the plugin and its two modules |
| Config | `backstage/app-config.local.yaml` | Port 7730, the three static tokens, `acme.orgOrigin`, catalog rules allowing `Template`/`User`/`Group`, and SQLite persisted in `backstage/.acme-data/` |
| CLI | `org/bin/acme`, `org/acme.json` | Python 3, standard library only. `--as <user>` picks that person's token from `app-config.local.yaml` |

## R1 to R8

| # | How it is met | Files |
|---|---|---|
| R1 Owned by the repository | Each repository's definition is a scaffolder `Template` committed to its origin's `main` and owned by `group:default/<repo>-maintainers`. Backstage reads it from the origin's `main` by git (`git cat-file` on the bare repo), not from a checkout. A commit to `main` is in the catalog within about 5 s. Until it is, a request returns 409 (the catalog's blob id is checked against `main`). Approving a run whose definition changed since it was requested is refused with 409. Entities are namespaced by repository, so alpha cannot define beta's action | `alpha/.backstage/actions/mark-deprecated.yaml` (also in `beta/`, `gamma/`); `backstage/plugins/acme-actions-backend/src/catalogModule.ts`; `org/repos.yaml` |
| R2 Validated | The repository's template holds the schema: `successor` matches `^[a-z0-9-]+/[a-z0-9-]+$` and is not `acme/<itself>`; `reason` is 20 to 200 characters; both required; no extra fields. Preview and request run the scaffolder's own validation through `/api/scaffolder/v2/dry-run` **before** any run is created, and return 400 with per-field `errors`. The scaffolder validates again when the approved task is created | template files above; `plugin.ts` (`preview()`) |
| R3 Previewed | `acme preview` does a scaffolder dry run of the repository's own definition. It returns the exact file content, `change` (`create`/`modify`/`none`), the base commit and a unified diff, and creates no run. `acme request` returns the same preview in the run, and `status`/`record` show it to the approver. At start, the action refuses to push anything other than the previewed content | `scaffolderModule.ts` (dry-run branch); `plugin.ts` (`/preview`, `/runs`, `/internal/runs/:id/start`) |
| R4 Approved | A request creates a run in `awaiting-approval`; nothing is dispatched. `approve` is refused (403, and logged as `approval-denied`) when the approver is the requester, or is not a member of the action's owner group (carol). Only then does `acme-actions` create the scaffolder task, passing a one-time token. The action refuses to change anything without a valid token for an approved run. Creating a scaffolder task directly (`POST /api/scaffolder/v2/tasks`) therefore fails with "Refusing to run" | `plugin.ts` (`/runs/:id/approve`, `/internal/runs/:id/start`); `scaffolderModule.ts`; `org/catalog/people.yaml` |
| R5 Done means observed | Two checks. (1) After pushing, the action reads `main` back from the origin and fails the task unless `DEPRECATED.md` has the expected content. (2) The scaffolder task finishing is not enough for `acme-actions`: its sync loop (every 2 s, and on every `status`/`record` read) reads the origin's `main` itself. Only if the file matches the previewed content byte for byte is the run marked `succeeded`, with the observed commit and time. A rejected push fails the task, and the run is `failed` with git's error | `scaffolderModule.ts`; `plugin.ts` (`sync()`, `observe()`) |
| R6 One place for the org | From `org/`, `bin/acme actions` lists `mark-deprecated` for alpha, beta and gamma (with owners, approvers, definition commit and input schema). `bin/acme request … --repo <name>` requests a run on any of them. The registry of repositories is `org/repos.yaml` | `org/bin/acme`, `org/acme.json`, `org/repos.yaml` |
| R7 Audited | Each run stores requester, `requestedAt`, inputs, the definition (origin, path, commit, blob), the preview, approver, `approvedAt`, scaffolder task id, `startedAt`, `finishedAt` and outcome (observed commit, content hashes or error). An append-only event log holds `requested`, `approval-denied`, `approved`, `dispatched`, `started`, `succeeded`/`failed`, each with actor and timestamp. It is stored in SQLite under `backstage/.acme-data/` and survives restarts. The pushed commit is authored by the requester and carries `Acme-Run`, `Requested-by` and `Approved-by` trailers | `store.ts`; `plugin.ts`; `scaffolderModule.ts` |
| R8 Usable by agents | Every step is a non-interactive `bin/acme` command that prints JSON on stdout. Exit codes: 0 ok; 1 refused or failed (the JSON says why, with `status` = HTTP code); 2 usage or connection error; for `status --wait`, 1 = run failed and 4 = timed out. The same operations are plain HTTP+JSON under `http://localhost:7730/api/acme-actions` with `Authorization: Bearer <token>` | `org/bin/acme`; `plugin.ts` |

Identity: there are no passwords. Each person's static token (`app-config.local.yaml`) reaches
Backstage as a service principal named after them. `acme-actions` maps it to `user:default/<name>`
and rejects names that are not users in the catalog.

## Commands

### Start (must be running)

```bash
cd /tmp/actions-ab/r3/backstage-1/backstage
tmux new-session -d -s r3-backstage-1-backend -c /tmp/actions-ab/r3/backstage-1/backstage \
  'export PATH=/tmp/bs/node24/bin:$PATH; yarn workspace backend start 2>&1 | tee /tmp/actions-ab/r3/backstage-1/backstage/backend.log'
# ready when this prints 401 (unauthenticated) instead of 000:
until [ "$(curl -s -o /dev/null -w '%{http_code}' localhost:7730/api/acme-actions/actions)" = 401 ]; do sleep 2; done
```

Without tmux, run `export PATH=/tmp/bs/node24/bin:$PATH && yarn workspace backend start` in
`/tmp/actions-ab/r3/backstage-1/backstage` in its own terminal. If you delete `backstage/.acme-data/`
to start from an empty database, wait about 10 s after startup until `bin/acme --as ann actions`
lists three actions.

Stop: `tmux kill-session -t r3-backstage-1-backend`.

### Use (from the org repository)

```bash
cd /tmp/actions-ab/r3/backstage-1/org

# list the actions in every repository
bin/acme --as ann actions
bin/acme --as ann actions --action mark-deprecated --repo beta

# preview: validates and shows the change; creates nothing
bin/acme --as ann preview mark-deprecated --repo alpha \
  --set successor=acme/dependabot-proxy-v2 \
  --set "reason=Replaced by the v2 proxy, which supports OIDC registries."

# request as ann (prints the run, status awaiting-approval, with its preview)
ID=$(bin/acme --as ann request mark-deprecated --repo alpha \
  --set successor=acme/dependabot-proxy-v2 \
  --set "reason=Replaced by the v2 proxy, which supports OIDC registries." | jq -r .id)

# approve as bob (dispatches the run)
bin/acme --as bob approve "$ID"

# status, once or waiting until succeeded/failed
bin/acme --as ann status "$ID"
bin/acme --as ann status "$ID" --wait --timeout 120

# full record with the event log
bin/acme --as ann record "$ID"

# all runs (filters: --repo, --action, --status)
bin/acme --as ann runs
```

Inputs can also be given as one JSON object: `--inputs '{"successor":"acme/x","reason":"..."}'`.

HTTP equivalents, for example as bob:

```bash
H='Authorization: Bearer bob-token-0123456789'
curl -s -H "$H" localhost:7730/api/acme-actions/actions
curl -s -H "$H" -H 'content-type: application/json' localhost:7730/api/acme-actions/preview -d '{"action":"mark-deprecated","repo":"alpha","parameters":{"successor":"acme/x","reason":"twenty characters or more"}}'
curl -s -H 'Authorization: Bearer ann-token-0123456789' -H 'content-type: application/json' localhost:7730/api/acme-actions/runs -d '{"action":"mark-deprecated","repo":"alpha","parameters":{"successor":"acme/x","reason":"twenty characters or more"}}'
curl -s -H "$H" -X POST localhost:7730/api/acme-actions/runs/$ID/approve
curl -s -H "$H" localhost:7730/api/acme-actions/runs/$ID
curl -s -H "$H" localhost:7730/api/acme-actions/runs/$ID/record
```

## Testing the rules

```bash
cd /tmp/actions-ab/r3/backstage-1/org

# invalid inputs: HTTP 400 with per-field errors, no run created (exit 1)
bin/acme --as ann request mark-deprecated --repo alpha --set successor=Acme/Bad_Name --set reason=short
bin/acme --as ann request mark-deprecated --repo alpha --set successor=acme/alpha --set "reason=$(printf 'x%.0s' {1..201})"
bin/acme --as ann request mark-deprecated --repo alpha --set successor=acme/x          # reason missing

# a run to test approvals on (uses a new reason so that it is a real change)
ID=$(bin/acme --as ann request mark-deprecated --repo beta --set successor=acme/valtio-store \
  --set "reason=A different reason, long enough to pass." | jq -r .id)
bin/acme --as ann approve "$ID"     # 403: the requester cannot approve their own run
bin/acme --as carol approve "$ID"   # 403: carol is not a maintainer of beta

# rejected push: install a hook that rejects every push, then approve as bob
printf '#!/bin/sh\necho "main is protected" >&2\nexit 1\n' > /tmp/actions-ab/r3/backstage-1/origins/beta.git/hooks/pre-receive
chmod +x /tmp/actions-ab/r3/backstage-1/origins/beta.git/hooks/pre-receive
bin/acme --as bob approve "$ID"
bin/acme --as bob status "$ID" --wait   # status "failed", outcome.error has git's rejection; exit 1
rm /tmp/actions-ab/r3/backstage-1/origins/beta.git/hooks/pre-receive
bin/acme --as bob record "$ID"          # both denied approvals, the approval and the failure, timestamped

# bypassing the approval through the scaffolder API directly: the task fails with "Refusing to run"
curl -s -H 'Authorization: Bearer carol-token-0123456789' -H 'content-type: application/json' \
  localhost:7730/api/scaffolder/v2/tasks \
  -d '{"templateRef":"template:alpha/mark-deprecated","values":{"successor":"acme/evil","reason":"carol bypassing the approval gate"}}'
```

Things to know when testing:

- All three repositories already have `DEPRECATED.md`. A request with the same inputs as before
  previews `change: "none"`. Nothing is pushed, so a `pre-receive` hook is never consulted, and the
  run succeeds because the file is observed on `main`. To test a rejected push, use a different
  `reason` or `successor`, as above.
- A run can be approved once. Approving a run that is not `awaiting-approval` returns 409.
- Run states: `awaiting-approval` → `running` (approved and dispatched) → `succeeded` or `failed`.
  There is no reject or cancel command; an unapproved run stays `awaiting-approval`.
- Backstage's permission policy is still the create-app allow-all policy. The approval rule is
  enforced by `acme-actions` and by the action itself, not by Backstage permissions.
- The Backstage code (`backstage/plugins/acme-actions-backend/`, the backend wiring and
  `app-config.local.yaml`) is not committed: `backstage/` has no origin.
