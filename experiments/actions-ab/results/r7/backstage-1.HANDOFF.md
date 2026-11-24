# Handoff: `mark-deprecated`, a repository-owned self-service action on Backstage

`mark-deprecated` ran on all three repositories. ann requested each run and bob approved it. Each was
reported `succeeded` only after Backstage read `DEPRECATED.md` with the approved content on the
origin's `main`:

| Repository | Run | Commit on `origins/<repo>.git` `main` |
|---|---|---|
| alpha | `run-967e908f9294` | `58a7c25` Mark deprecated: use acme/dependabot-proxy-v2 |
| beta | `run-1775d0e9ee48` | `239f4b0` Mark deprecated: use acme/valtio-store |
| gamma | `run-46c62477f56f` | `c89c820` Mark deprecated: use acme/graphify-core |

The record also holds the negative checks, kept as written:

- `run-154f3ac5500f` and `run-dadc6ad043e5` (alpha) and `run-4deb8c21904a` (beta) failed because a
  `pre-receive` hook rejected the push.
- `run-bfe2a186f733` failed because the push was accepted, then reverted by a `post-receive` hook.
- `run-644de97a5ebb` was rejected by bob.
- `run-ea499def6a04` was superseded because gamma's definition changed before approval.

## How it works

The glue is a Backstage app with four additions. They are in
`/tmp/actions-ab/r7/backstage-1/backstage/packages/backend/src/acme/` and wired in
`packages/backend/src/index.ts`:

| File | What it adds |
|---|---|
| `catalogModule.ts` | Catalog entity provider `acme-git-origins`. Every 15 s, and before each run, it reads `catalog-info.yaml` and `.backstage/*.yaml` from each origin's `main`. It forces each repository's entities into a namespace named after the repository (`template:alpha/mark-deprecated`). |
| `scaffolderModule.ts` | Scaffolder actions `acme:git:commit-file` and `acme:git:verify-file`. Both support dry runs. A real `commit-file` asks the run store for permission first (see R4). |
| `actionsPlugin.ts` | Backend plugin `acme-actions` at `http://localhost:7830/api/acme-actions`. It handles list, preview (a scaffolder dry run), request, approve and reject, status, and the record. After approval it creates the scaffolder task, tracks it, and then observes `main` itself. |
| `runStore.ts` | The `acme_runs` and `acme_run_events` tables in the plugin's database. |
| `permissionModule.ts` | Replaces the allow-all permission policy. |
| `repoSource.ts`, `git.ts`, `config.ts` | Read definitions from the origins, run git, and read the `acme:` config. |

Configuration is in `/tmp/actions-ab/r7/backstage-1/backstage/app-config.local.yaml`. It holds the
`acme.repositories` origins, a persistent SQLite directory (`backstage/.data/sqlite`), and
`accessRestrictions` on the three static tokens. The file is git-ignored by the app, so it is not
in the app's commit.

The org's entry point is `/tmp/actions-ab/r7/backstage-1/org/bin/acme`, a Python 3 CLI with only
standard-library dependencies. Its settings are in `org/acme.json` (URL and the sandbox tokens). The
people (`User` ann, bob, carol) are in `org/catalog-info.yaml`.

## Requirements

| # | How it is met |
|---|---|
| R1 Owned by the repository | Each repository's definition is on its origin's `main`. `<repo>/.backstage/mark-deprecated.yaml` is the Backstage `Template`: inputs and their rules, steps, and what to observe. `<repo>/catalog-info.yaml` defines `Group maintainers` (members `user:default/ann`, `user:default/bob`) and the `Component`. The template's `spec.owner: group:maintainers` makes those members its approvers. Backstage reads only from the origin's `main` (`repoSource.ts`, `catalogModule.ts`), so a change the team pushes there is the new definition. `acme actions` shows the commit and blob in use and whether the catalog serves it. A repository can define actions only in its own namespace and cannot define users. |
| R2 Validated | `acme preview` and `acme request` validate inputs against the template's `parameters` JSON Schema, read from `main`, before anything else happens (`validateInputs` in `actionsPlugin.ts`). Bad input returns HTTP 400 `InvalidInputs` with one entry per input, and the CLI exits 2. No run is created and nothing is pushed. The rules: `successor` must match `^[a-z0-9-]+/[a-z0-9-]+$` and must not be the repository itself (`acme/<repo>`); `reason` must be 20 to 200 characters; unknown inputs are refused. The scaffolder validates again when it creates the task, and the action's own schema checks `path`. |
| R3 Previewed | `acme preview` runs the repository's template as a scaffolder dry run (`/v2/dry-run`), which pushes nothing. `acme:git:commit-file` clones `main` and reports the change: `changeType` (`create`/`modify`/`none`), `baseCommit`, the full new `content` and a unified `diff`. `acme request` runs the same preview and stores it in the run, so `acme status` shows the requester and approver the `change` before approval. |
| R4 Approved | `POST /runs/:id/approve` (`Engine.approve` in `actionsPlugin.ts`) refuses the requester, with HTTP 403 and CLI exit 3. It also refuses anyone who is not a member of the template owner's group on `main` today, which includes carol. Every refusal is recorded as an `approval-denied` event. If the definition changed on `main` since the request, the run is `superseded` (HTTP 409). Approval is enforced where the change happens: `acme:git:commit-file` pushes only after `RunStore.claimPublish` confirms all of these: (a) the task carries the nonce of a run that is approved and `running`; (b) the run targets this repository; (c) the path and content are exactly what was previewed; (d) the run has not published before. Backstage's own routes around the gate are closed. The people's tokens carry `accessRestrictions`: they can call `acme-actions` and only read from `catalog` and `scaffolder`, and restricted tokens cannot be delegated, so `scaffolder:execute-template` is refused too. `permissionModule.ts` denies task creation, dry runs and catalog location changes to signed-in users. The guest sign-in provider is removed. |
| R5 Done means observed | The template's `verify` step (`acme:git:verify-file`) reads `DEPRECATED.md` from the origin's `main` and fails the task on a mismatch. Then `acme-actions` reads it again itself (`Engine.observe`, `observeFile` in `git.ts`). The run becomes `succeeded` only if `main` has exactly the previewed content. The `observed` field records the commit, path and match. A rejected push fails the task, so the run is `failed` with git's message. A push that is accepted but does not stay on `main` fails verification, and the run is `failed` too. |
| R6 One place for the org | From `/tmp/actions-ab/r7/backstage-1/org`, `bin/acme actions` lists every self-service action of every configured repository (all three `mark-deprecated`). `bin/acme request <repo> mark-deprecated …` requests a run on any of them. Nobody needs to enter the repositories. |
| R7 Audited | `acme record <run>` (`GET /runs/:id/record`) returns `requestedBy`/`requestedAt`, `approvedBy`/`approvedAt`, `inputs`, `status`/`outcome` (with reason and time), `startedAt` and `observed`. It also has a `timeline`, an `events` list (requested, previewed, approval-denied, approved, rejected, started, publish-authorized, observed, succeeded/failed) with actor and ISO-8601 time, and the scaffolder task's log. Records are stored in `backstage/.data/sqlite/acme-actions.sqlite` and survive restarts. The commit on `main` is authored by the requester and carries `Acme-Run`, `Requested-by` and `Approved-by` trailers. |
| R8 Usable by agents | Every command is non-interactive and prints one JSON document on stdout, errors included (`{"error": {"code", "message", "details"}, "httpStatus"}`). Exit codes: 0 ok, 1 server or connection error, 2 invalid input, 3 not allowed, 4 not found, 5 wrong state, 6 run finished without succeeding, 7 wait timed out. Identity comes from `--as <person>` or `ACME_AS`, and the token from `org/acme.json` or `ACME_TOKEN`. The same JSON API can be called with curl; see below. |

## Commands

### Start Backstage

Run these from a shell; tmux keeps the backend running in the background:

```sh
export PATH=/tmp/bs/node24/bin:$PATH
cd /tmp/actions-ab/r7/backstage-1/backstage
tmux new-session -d -s r7-backstage-1-backend -c "$PWD" "export PATH=/tmp/bs/node24/bin:\$PATH; yarn workspace backend start > .backend.log 2>&1"
until curl -sf -H 'Authorization: Bearer ann-token-0123456789' http://localhost:7830/api/acme-actions/whoami; do sleep 2; done
```

The backend listens on port 7830 and takes about 10 seconds to start. To run it in the foreground
instead, use `yarn workspace backend start`. To stop it, run
`tmux kill-session -t r7-backstage-1-backend`.

### Use the action

Run these from `/tmp/actions-ab/r7/backstage-1/org`:

```sh
cd /tmp/actions-ab/r7/backstage-1/org

# List the actions in every repository
bin/acme --as carol actions
bin/acme --as carol actions --action mark-deprecated
bin/acme --as carol show alpha mark-deprecated

# Preview (changes nothing)
bin/acme --as ann preview alpha mark-deprecated --successor acme/dependabot-proxy-v2 \
  --reason 'Replaced by the v2 proxy, which supports OIDC registries.'
# Generic form for any action: -i key=value (repeatable) or --inputs-json '{"successor": "...", "reason": "..."}'

# Request as ann; the response includes the run id and the preview
RUN=$(bin/acme --as ann request alpha mark-deprecated --successor acme/dependabot-proxy-v2 \
  --reason 'Replaced by the v2 proxy, which supports OIDC registries.' | python3 -c 'import json,sys; print(json.load(sys.stdin)["id"])')

# Approve as bob
bin/acme --as bob approve "$RUN" --comment 'ok'
bin/acme --as bob reject "$RUN" --comment 'why'   # instead of approving

# Status, waiting, the record, and listing runs
bin/acme --as ann status "$RUN"
bin/acme --as ann wait "$RUN" --timeout 120       # exits 0 only if the run succeeded
bin/acme --as ann record "$RUN"
bin/acme --as ann runs --repository alpha --status succeeded
```

The same calls with curl (JSON in, JSON out):

```sh
B=http://localhost:7830/api/acme-actions
curl -s -H 'Authorization: Bearer carol-token-0123456789' $B/actions
curl -s -X POST -H 'Authorization: Bearer ann-token-0123456789' -H 'content-type: application/json' \
  $B/actions/alpha/mark-deprecated/preview -d '{"inputs":{"successor":"acme/x","reason":"A reason of twenty or more characters."}}'
curl -s -X POST -H 'Authorization: Bearer ann-token-0123456789' -H 'content-type: application/json' \
  $B/actions/alpha/mark-deprecated/runs -d '{"inputs":{"successor":"acme/x","reason":"A reason of twenty or more characters."}}'
curl -s -X POST -H 'Authorization: Bearer bob-token-0123456789' -H 'content-type: application/json' $B/runs/$RUN/approve -d '{}'
curl -s -H 'Authorization: Bearer ann-token-0123456789' $B/runs/$RUN          # status
curl -s -H 'Authorization: Bearer ann-token-0123456789' $B/runs/$RUN/record   # audit record
```

## Testing it

```sh
cd /tmp/actions-ab/r7/backstage-1/org
# Invalid inputs: HTTP 400 and exit 2; no run is created
bin/acme --as ann request alpha mark-deprecated --successor 'Acme/X_y' --reason 'too short'; echo $?
bin/acme --as ann request alpha mark-deprecated --successor acme/alpha --reason 'A reason of twenty or more characters.'; echo $?

# Self-approval and carol: HTTP 403 and exit 3; both are recorded as approval-denied
RUN=$(bin/acme --as ann request beta mark-deprecated --successor acme/new-beta \
  --reason 'A different reason, so there is something to push.' | python3 -c 'import json,sys; print(json.load(sys.stdin)["id"])')
bin/acme --as ann approve "$RUN"; echo $?
bin/acme --as carol approve "$RUN"; echo $?

# Rejected push: the run is failed, wait exits 6, main is unchanged
printf '#!/bin/sh\necho "main is frozen" >&2\nexit 1\n' > ../origins/beta.git/hooks/pre-receive
chmod +x ../origins/beta.git/hooks/pre-receive
bin/acme --as bob approve "$RUN"; bin/acme --as ann wait "$RUN"; echo $?
rm ../origins/beta.git/hooks/pre-receive
```

Behaviour to know when testing:

- **Exact content.** The file is `# Deprecated: use <successor>`, an empty line, then `<reason>`, and ends
  with one trailing newline.
- **Same content, nothing to push.** All three repositories already have their `DEPRECATED.md`. A run
  whose content already matches `main` has `changeType: "none"` in its preview. It pushes nothing, so
  a `pre-receive` hook never runs, and it succeeds because the right content is observed on `main`.
  To test a rejected push, use different inputs (as above) so there is a commit to push.
- **Definition changes.** A change to `.backstage/mark-deprecated.yaml` on `main` takes effect for the
  next preview or request immediately. A run requested under the old definition becomes `superseded`
  when someone tries to approve it.
- **Execution.** Approved runs execute in about a second. Before creating the task, `acme-actions`
  waits until the catalog serves the approved definition, at most 60 seconds.
- **Bypasses are closed.** With a person's token, the following are refused with 403:
  `POST /api/scaffolder/v2/tasks`, `/v2/dry-run`, `POST /api/catalog/locations`, `/api/mcp-actions`,
  and invoking `scaffolder:execute-template`.
