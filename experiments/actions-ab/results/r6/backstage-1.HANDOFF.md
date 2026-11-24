# Handoff: `mark-deprecated` on the Acme Backstage portal

`mark-deprecated` is a self-service action that each repository's maintainers own. It is defined
in each repository, listed and run from this org directory, executed by the Backstage
scaffolder, gated on approval by a maintainer other than the requester, and reported as succeeded
only once Backstage has read `DEPRECATED.md` back from the origin's `main`.

Ann requested and Bob approved all three runs, and all three are reported as **succeeded**:

| Repository | Run id | Commit on origin `main` |
|---|---|---|
| alpha | `run-57fc6dfb-054d-4947-a804-1218b39a153a` | `7343918` |
| beta | `run-9ccf9ade-09f9-4730-b9a2-62e9732bdb5f` | `a2d5a48` |
| gamma | `run-b13bb855-4775-4455-8c9b-7804763acb29` | `7e2b8ca` |

Each `DEPRECATED.md` is exactly `# Deprecated: use <successor>\n\n<reason>\n`, ending in a
newline. The run history also holds alpha runs from testing: `failed` runs whose push the origin
rejected, and a `cancelled` run whose definition changed before approval. None of them changed
`main`.

## How it fits together

```
org/bin/acme-actions ──HTTP──▶ Backstage /api/acme-actions   (plugin: runs, rules, audit)
                                 │  reads actions from the catalog
                                 │  preview  = scaffolder dry-run of the repo's own template
                                 │  approve  → scaffolder task (template:<repo>/mark-deprecated)
                                 │  monitor  → reads DEPRECATED.md back from origin main
catalog ◀── acme-git-origins provider: reads org.git main (repositories.yaml, catalog/)
                                       and <repo>.git main (catalog-info.yaml, .backstage/)
scaffolder actions: acme:git:commit-file (plans in dry run; for real, pushes only an approved,
                    previewed change), acme:git:verify-file
```

Platform code (Backstage glue, committed in the Backstage app's git repo as `b2c4d10`):

| File | What it does |
|---|---|
| `/tmp/actions-ab/r6/backstage-1/backstage/packages/backend/src/acme/catalogModule.ts` | Catalog entity provider: reads entities from each origin's `main` |
| `/tmp/actions-ab/r6/backstage-1/backstage/packages/backend/src/acme/scaffolderModule.ts` | Scaffolder actions `acme:git:commit-file` and `acme:git:verify-file` |
| `/tmp/actions-ab/r6/backstage-1/backstage/packages/backend/src/acme/plugin.ts` | `acme-actions` backend plugin: HTTP API, identity, validation, preview, approval rules, execution, observation, records |
| `/tmp/actions-ab/r6/backstage-1/backstage/packages/backend/src/acme/store.ts` | Run and audit-event tables, in the plugin's own database |
| `/tmp/actions-ab/r6/backstage-1/backstage/packages/backend/src/acme/org.ts`, `git.ts` | Org registry reader; git helpers |
| `/tmp/actions-ab/r6/backstage-1/backstage/packages/backend/src/index.ts` | Adds the two modules and the plugin to the backend |
| `/tmp/actions-ab/r6/backstage-1/backstage/app-config.local.yaml` | Port 7630, the three people's static tokens (restricted to `acme-actions` and read-only catalog), `acme.org.origin`, persistent SQLite in `backstage/.data/`, catalog rules. Gitignored, like every `*.local.yaml` |

Org and repository files (committed and pushed to each origin's `main`):

| File | What it does |
|---|---|
| `org/repositories.yaml` | The org registry: which repositories exist, their `owner/name` and origin |
| `org/catalog/people.yaml` | Users ann, bob and carol. Only the org can define people |
| `org/bin/acme-actions` | The org CLI (Node, no dependencies) |
| `org/acme-actions.json` | Portal URL, plus the sandbox token for each person, so `--as NAME` works |
| `<repo>/.backstage/actions/mark-deprecated.yaml` | The repository's action definition: a scaffolder Template with inputs schema and steps |
| `<repo>/catalog-info.yaml` | The repository's Component and its `maintainers` Group (members ann and bob) |

## Requirements

| # | How it is met |
|---|---|
| R1 Owned by the repository | Each repository defines its action in `alpha/.backstage/actions/mark-deprecated.yaml` (and the same path in `beta` and `gamma`), committed to its origin's `main`. Its approvers are its own `maintainers` Group in `<repo>/catalog-info.yaml`. `catalogModule.ts` reads these files straight from `origins/<repo>.git` `main`, not from a checkout, and forces each repository's entities into a namespace named after it (`template:alpha/mark-deprecated`), so one repository cannot define another's action. A repository also cannot define users. `plugin.ts` `isOwnedBy()` only treats a template as a repository's action if it was read from that repository's origin. A pushed change shows up within about 5 s; I tested this by changing alpha's description (commit `a88724f`). A run records the definition's path, blob and commit. If the definition changes between request and approval, approval is refused with 409 and the run must be requested again. |
| R2 Validated | The inputs schema lives in each template: `successor` must match `^[a-z0-9-]+/[a-z0-9-]+$` and must not be the repository itself, and `reason` must be 20–200 characters. `plugin.ts` `validateInputs()` checks inputs against that schema, and rejects unknown inputs, on both `preview` and `request`. Errors come back as HTTP 400 with `errors: [{input, message}]` before any run is recorded or anything changes. The scaffolder validates the same schema again on dry-run and task creation, and the scaffolder actions validate their own inputs with zod. |
| R3 Previewed | `preview` runs the repository's own template in **scaffolder dry-run** mode (`plugin.ts` `preview()`). In a dry run, `acme:git:commit-file` reads the current file from origin `main` and reports the planned change: path, add/update/none, full content, content hash and unified diff. Nothing is changed or recorded. `request` repeats the preview and stores it in the run (`preview` in the record), so the approver sees exactly what will be applied. |
| R4 Approved | `request` only records a run (`awaiting-approval`); nothing executes. `approve` (`plugin.ts` `approve()`) refuses with 403 if the caller is the requester, or if the caller is not a member of the template owner's Group (`carol is not a maintainer of acme/alpha`). Each refusal is recorded as an `approval-refused` event. Only after approval does the plugin start the scaffolder task, passing a one-time run token as a task secret. Before it pushes, `acme:git:commit-file` calls `/internal/authorize`, which checks the token, the approval, the task, and that the change's hash equals the approved preview. Without a valid run token, the action refuses to push. The people's tokens are restricted to the `acme-actions` plugin and read-only catalog access (`app-config.local.yaml`), so nobody can start scaffolder tasks or register catalog locations directly (403). |
| R5 Done means observed | The plugin's monitor (`plugin.ts` `check()` and `observe()`) waits for the scaffolder task to finish, then reads each planned file from the origin with `git show main:DEPRECATED.md`. It marks the run `succeeded` only if the task completed **and** the content on `main` equals the approved content. Otherwise the run is `failed`, with the reason and the observation. A rejected push fails the `deprecate` step, so the run fails with `push to acme/alpha@main was rejected: …` and `main` is untouched (tested with a `pre-receive` hook; see below). The template's own `verify` step (`acme:git:verify-file`) also checks `main`, but the platform's observation does not depend on it. |
| R6 One place for the org | From `org/`, `bin/acme-actions list` lists every repository's actions, using `org/repositories.yaml` and the catalog. `bin/acme-actions preview\|request <repo> …` works on any repository without going into it. The same operations are in the HTTP API `http://localhost:7630/api/acme-actions/*`. |
| R7 Audited | `record <run-id>` (`GET /api/acme-actions/runs/:id`) returns the inputs and the definition (ref, path, blob, commit, owner, approvers). It shows `requested {by, at}`, `approved {by, at}`, `decided`, the preview, the execution (scaffolder task id and status), and the `outcome {status, reason, observation, at}`. It ends with an append-only `events` list `{at, actor, type, detail}`, including refused approvals. Records are in `acme_runs` and `acme_run_events` (`store.ts`), in SQLite under `backstage/.data/`, so they survive restarts. The commit on `main` also carries `Acme-Run`, `Requested-by` and `Approved-by` trailers, with the requester as author. |
| R8 Usable by agents | Every step is a non-interactive CLI command or HTTP call that returns JSON. CLI exit codes: `0` ok, `1` the portal refused (the JSON body includes `status` and `error.message`), `2` usage, `3` the run ended without succeeding (`--wait`), `4` the wait timed out. `status --wait` and `approve --wait` block until the run is terminal. |

## Commands

All commands assume this setup:

```bash
export PATH=/tmp/bs/node24/bin:$PATH
cd /tmp/actions-ab/r6/backstage-1/org
```

### Start (the only long-running process)

```bash
tmux new-session -d -s r6-backstage-1-backend -c /tmp/actions-ab/r6/backstage-1/backstage \
  'export PATH=/tmp/bs/node24/bin:$PATH; yarn workspace backend start 2>&1 | tee .backend.log'
# ready when this prints ann:
until bin/acme-actions --as ann whoami 2>/dev/null | grep -q '"ann"'; do sleep 2; done; echo ready
```

In a foreground terminal, `yarn workspace backend start` in `/tmp/actions-ab/r6/backstage-1/backstage`
does the same. Allow a few seconds after start for the catalog to load the actions; `list` shows
all three when it has. To stop:

```bash
tmux kill-session -t r6-backstage-1-backend
```

### Use

```bash
# List the actions of every repository
bin/acme-actions --as carol list

# Preview: what a run would change (nothing is recorded or changed)
bin/acme-actions --as ann preview alpha --successor acme/dependabot-proxy-v2 \
  --reason "Replaced by the v2 proxy, which supports OIDC registries."

# Request, as ann (prints the run, including its preview; status awaiting-approval)
ID=$(bin/acme-actions --as ann request alpha --successor acme/dependabot-proxy-v2 \
  --reason "Replaced by the v2 proxy, which supports OIDC registries." | jq -r .id); echo $ID

# Approve, as bob (add --wait to block until the run succeeds (exit 0) or fails (exit 3))
bin/acme-actions --as bob approve "$ID"

# Status (add --wait to block until terminal)
bin/acme-actions --as bob status "$ID"

# Full record: who, when, inputs, preview, outcome, events
bin/acme-actions --as bob record "$ID"

# All runs (filters: --repository alpha, --status failed)
bin/acme-actions --as ann runs

# Reject a pending run (a maintainer), or cancel it (the requester)
bin/acme-actions --as bob reject "$ID" --comment "not yet"
```

`<repository>` can be `alpha` or `acme/alpha`. Inputs can also be given as `--input reason=...`
or `--inputs-json '{"successor":"...","reason":"..."}'`. `--as NAME` reads the token from
`org/acme-actions.json`. `--token TOKEN` or `$ACME_TOKEN` also work.

The same operations over HTTP, as Bearer-token calls:

```bash
U=http://localhost:7630/api/acme-actions
curl -s -H 'Authorization: Bearer carol-token-0123456789' $U/actions
curl -s -H 'Authorization: Bearer ann-token-0123456789' -H 'content-type: application/json' \
  $U/preview -d '{"repository":"alpha","action":"mark-deprecated","inputs":{"successor":"acme/x-y","reason":"twenty characters or more"}}'
curl -s -H 'Authorization: Bearer ann-token-0123456789' -H 'content-type: application/json' \
  $U/runs -d '{"repository":"alpha","action":"mark-deprecated","inputs":{"successor":"acme/x-y","reason":"twenty characters or more"}}'
curl -s -X POST -H 'Authorization: Bearer bob-token-0123456789' $U/runs/$ID/approve
curl -s -H 'Authorization: Bearer bob-token-0123456789' $U/runs/$ID/status
curl -s -H 'Authorization: Bearer bob-token-0123456789' $U/runs/$ID
```

## Testing the rules

All three repositories already have `DEPRECATED.md`. A run whose content is identical to what
is on `main` has nothing to push, and it succeeds because the right content is observed there.
**To exercise a change (and the rejected-push case), use a different reason.**

```bash
R="Testing a changed reason for the deprecation notice."

# Invalid inputs: HTTP 400 with per-input errors, exit 1; no run is recorded
bin/acme-actions --as ann request alpha --successor Not/Valid --reason short
bin/acme-actions --as ann request alpha --successor acme/alpha --reason "$R"         # self as successor
bin/acme-actions --as ann request alpha --successor acme/x --reason "$R" --colour red  # unknown input

# Self-approval and carol's approval: 403, recorded as approval-refused events
ID=$(bin/acme-actions --as ann request alpha --successor acme/x --reason "$R" | jq -r .id)
bin/acme-actions --as ann approve "$ID"      # 403: requester cannot approve
bin/acme-actions --as carol approve "$ID"    # 403: carol is not a maintainer

# Rejected push: the run fails and main is unchanged
H=/tmp/actions-ab/r6/backstage-1/origins/alpha.git/hooks/pre-receive
printf '#!/bin/sh\necho "main is frozen" >&2\nexit 1\n' > $H && chmod +x $H
bin/acme-actions --as bob approve "$ID" --wait; echo "exit=$?"    # status failed, exit 3
rm -f $H
git --git-dir=/tmp/actions-ab/r6/backstage-1/origins/alpha.git show main:DEPRECATED.md   # unchanged

# Bypassing the portal: 403 (people's tokens cannot reach the scaffolder)
curl -s -H 'Authorization: Bearer ann-token-0123456789' -H 'content-type: application/json' \
  http://localhost:7630/api/scaffolder/v2/tasks -d '{"templateRef":"template:alpha/mark-deprecated","values":{}}'
```

Other behaviour:

- Approving twice, or approving a run that is no longer pending, returns 409.
- Carol may request runs; ann or bob approves them.
- To start over with an empty run history, stop the backend and delete
  `/tmp/actions-ab/r6/backstage-1/backstage/.data/`.

## Limits

- Identity is the static Backstage token, mapped to the person named by its subject. There are no
  passwords in this sandbox; a real deployment would use a sign-in provider, with the same rules.
- One approval from the owner group is required; the rule is fixed in the plugin, not configurable
  per action.
- The provider polls the origins every 5 seconds. There are no push webhooks here.
- No frontend page; the API and CLI are the interface.
