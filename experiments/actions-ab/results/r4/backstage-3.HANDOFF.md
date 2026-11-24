# Handoff: `mark-deprecated` on Backstage

`mark-deprecated` is a self-service action defined in each repository, run through Backstage
1.57's scaffolder, and gated by an approval service, `acme-actions`, which is a Backstage backend
plugin. ann requested it on alpha, beta and gamma, bob approved each run, and each one was
reported `succeeded` only after `DEPRECATED.md` was seen on the origin's `main`:

| Repository | Run id | Observed commit on `origins/<repo>.git` `main` |
|---|---|---|
| alpha | `2464c2ff-2d28-4b0e-a1a5-5e183635344e` | `eef20c1b495420d63e4dd0d7e830444a16df893f` |
| beta | `c92bc9ad-2fe1-460f-aa81-e2730d06f495` | `43664a5f05ecf1ba2dc81edb2e0afce618868114` |
| gamma | `3e2d50b7-948f-450a-8f10-6a5ee25300e3` | `709bfc8700dedfe8e6410554332d3d33d2977e6d` |

The run store also holds two earlier test runs on alpha. `3c72cd2f-…` **failed** because a
temporary `pre-receive` hook rejected its push. `4e81fe35-…` was refused once because its
definition had changed. bob then approved it, and it **succeeded**. That run left the commit
`3285ef0` ("Mark deprecated: use acme/test-target") on alpha's `main`. The real alpha run then
replaced the file with the correct content in `eef20c1`. I did not rewrite `main`'s history.
The store also holds one gamma run, from checking the commands in *Testing the rules*. ann
withdrew it, so it is `rejected`, and it changed nothing.

All paths below are relative to `/tmp/actions-ab/r4/backstage-3`.

## How it fits together

```
person or agent ──► org/bin/acme-actions (CLI, JSON out) ──► /api/acme-actions (approval service)
                                                                │  definitions: catalog
                                                                │  preview: scaffolder dry-run
                                                                │  run: scaffolder task, after approval
                                                                ▼
 origins/org.git main ──┐                                 scaffolder actions
 origins/<repo>.git main ┴─► catalog provider ──► catalog  acme:git:publish-file → push to origin main
                              (acme-origins)               acme:git:observe-file → check origin main
```

- **Definitions come from origins, not checkouts.** The catalog module
  `backstage/packages/backend/src/acme/catalogModule.ts` reads `repositories.yaml` and
  `catalog/*.yaml` from `origins/org.git` `main`. It then reads `catalog-info.yaml` and
  `.backstage/actions/*.yaml` from each listed repository's origin `main`. It reloads every
  10 seconds, or at once with `acme-actions refresh`. A repository's entities are forced into a
  catalog namespace named after it, so the template is `template:alpha/mark-deprecated` and the
  approvers are `group:alpha/maintainers`. One repository cannot define or override another's.
- **People cannot reach the scaffolder directly.** In `backstage/app-config.local.yaml`, each
  static token is restricted to the `catalog` plugin (read only) and the `acme-actions` plugin. A
  direct call from ann to `/api/scaffolder/v2/tasks` returns 403. Signed-in users, such as the
  guest provider's, are denied `scaffolder.task.create`, `scaffolder.template.dry-run` and
  `scaffolder.action.execute` by `backstage/packages/backend/src/acme/permissionModule.ts`. Only
  `acme-actions` starts scaffolder work, using its own service credentials.

## R1 to R8

| # | How it is met | Files |
|---|---|---|
| R1 Owned by the repository | Each repository commits its own scaffolder template to its origin's `main`. The template sets the inputs, their validation, the file content and the steps. The same repository's `catalog-info.yaml` declares `group:<repo>/maintainers` (ann, bob), which owns the template and approves its runs. Only a group from that same repository can approve. To change the action, commit and push to that repository. Backstage reloads it within 10 s. A run requested against an older definition is refused at approval with 409, and the requester must request it again. | `alpha/.backstage/actions/mark-deprecated.yaml`, `alpha/catalog-info.yaml` (same in `beta/`, `gamma/`); loader `backstage/packages/backend/src/acme/catalogModule.ts` |
| R2 Validated | Inputs are checked against the template's own JSON Schema: `successor` must match `^[a-z0-9-]+/[a-z0-9-]+$` and must not be the repository itself, `reason` must be 20 to 200 characters, both are required, and unknown keys are rejected. This happens on `preview` and `request`, before any record is written or anything changes. The response is HTTP 400 with a `validation` list, and the CLI exits 2. The scaffolder validates again when the task starts. | `ActionService.validate` in `backstage/packages/backend/src/acme/actionsPlugin.ts`; schemas in each `.backstage/actions/mark-deprecated.yaml` |
| R3 Previewed | `preview` runs the repository's own template through the scaffolder's dry-run. `acme:git:publish-file` then reports the change without pushing: path, operation (`create`, `update` or `none`), before, after, a diff, and the base commit. `request` runs the same preview and stores it in the run. The approver approves that change set, and observation checks it. | `ActionService.preview` in `actionsPlugin.ts`; dry-run branch in `backstage/packages/backend/src/acme/scaffolderModule.ts` |
| R4 Approved | A run starts as `awaiting_approval` and nothing is pushed until it is approved. On approval, the requester is refused with 403 ("requested this run and cannot approve it"). Anyone not in `group:<repo>/maintainers`, such as carol, is refused with 403. Each refusal is recorded as an `approval_denied` event. Approval is an atomic status change, and only then does `acme-actions` start the scaffolder task. People cannot start one themselves (see above). | `ActionService.approve` in `actionsPlugin.ts`; `permissionModule.ts`; `accessRestrictions` in `backstage/app-config.local.yaml` |
| R5 Done means observed | Success requires two checks. First, the scaffolder task must complete; its last step, `acme:git:observe-file`, fails unless the origin's `main` has the exact content. Second, `acme-actions` independently reads `git show main:DEPRECATED.md` from the origin and compares it with the previewed content. A rejected push fails the publish step, so the run becomes `failed`, with git's error as the reason and an `observed` event showing the file absent or different. | `ActionService.checkRunning` and `observe` in `actionsPlugin.ts`; `acme:git:observe-file` in `scaffolderModule.ts` |
| R6 One place for the org | `org/` holds the repository list (`org/repositories.yaml`), the people (`org/catalog/people.yaml`), and the CLI (`org/bin/acme-actions`). The CLI lists `mark-deprecated` on all three repositories and requests, previews, approves and inspects runs on any of them, without entering the repositories. | `org/bin/acme-actions`, `org/repositories.yaml`, `org/catalog/people.yaml` |
| R7 Audited | Each run record stores `requestedBy`/`requestedAt`, `approvedBy`/`approvedAt`, `startedAt`, `finishedAt`, the inputs, the definition used (origin, path, git blob, spec hash), the preview, the scaffolder task id and the outcome, including what was observed and at which commit. It also has an append-only `events` list with a timestamp and actor for each event: `requested`, `approval_denied`, `approved`, `started`, `observed`, then `succeeded`, `failed` or `rejected`. The store is SQLite on disk (`backstage/.data/acme-actions.sqlite`), so records survive restarts. Commits on `main` carry `Acme-Run`, `Requested-by` and `Approved-by` trailers, with the requester as author. | `backstage/packages/backend/src/acme/store.ts`; `backend.database` in `backstage/app-config.local.yaml` |
| R8 Usable by agents | Every step is a non-interactive CLI command or HTTP call, and every response is JSON. CLI exit codes: 0 ok, 2 invalid input (400), 3 not allowed (401/403), 4 not found, 5 conflict (409), 6 the run did not succeed (`wait`), 7 timeout (`wait`), 1 other. The HTTP API lives under `http://localhost:8010/api/acme-actions` (see the end of this file). | `org/bin/acme-actions`; router in `actionsPlugin.ts` |

## Start

Nothing is running now. Start the Backstage backend, which includes `acme-actions`, and wait for
it to come up:

```bash
cd /tmp/actions-ab/r4/backstage-3/backstage
tmux new-session -d -s r4-backstage-3-backend \
  'export PATH=/tmp/bs/node24/bin:$PATH; yarn workspace backend start > /tmp/actions-ab/r4/backstage-3/backstage/.work/backend.log 2>&1'
until curl -sf http://localhost:8010/api/acme-actions/health; do sleep 2; done; echo
```

In the foreground, the same thing is
`export PATH=/tmp/bs/node24/bin:$PATH; cd /tmp/actions-ab/r4/backstage-3/backstage && yarn workspace backend start`.
Stop it with `tmux kill-session -t r4-backstage-3-backend`. The CLI needs only `python3`.

## Use

```bash
cd /tmp/actions-ab/r4/backstage-3/org

# List the actions on every repository (or --repo alpha), with inputs and approvers
bin/acme-actions --as ann list

# Preview: what a run would change. Nothing is written.
bin/acme-actions --as ann preview alpha --successor acme/dependabot-proxy-v2 \
  --reason "Replaced by the v2 proxy, which supports OIDC registries."

# Request as ann. Prints the run, including its id and preview. Status: awaiting_approval.
RUN=$(bin/acme-actions --as ann request beta --successor acme/valtio-store \
  --reason "State management moves to the shared store package." \
  | python3 -c 'import json,sys; print(json.load(sys.stdin)["id"])')

# Approve as bob. Starts the run.
bin/acme-actions --as bob approve "$RUN" --comment "LGTM"

# Status (summary), wait for the outcome (exit 0 only if it succeeded), full record
bin/acme-actions --as bob status "$RUN"
bin/acme-actions --as bob wait "$RUN" --timeout 120
bin/acme-actions --as bob record "$RUN"

# All runs, filterable by --repo and --status
bin/acme-actions --as bob runs
```

Other commands: `show <repo> [action]`, `reject <run> [--comment]` (a maintainer rejects, or the
requester withdraws), `whoami`, `refresh`. Inputs can also be given generically, as
`--input key=value` (repeatable) or `--inputs-json '{"successor":"…","reason":"…"}'`.
`DEPRECATED.md` is already on all three `main` branches. A new run with the same inputs previews
`operation: none` and succeeds without a new commit, because the content is still observed. Use
different inputs to see a real change.

## Testing the rules

```bash
cd /tmp/actions-ab/r4/backstage-3/org

# Invalid inputs: HTTP 400, exit 2, and no run is created
bin/acme-actions --as ann request alpha --successor "Not/Valid!" --reason "too short"
bin/acme-actions --as ann request alpha --successor acme/alpha --reason "Points at itself, which is refused."
bin/acme-actions --as ann request alpha --inputs-json '{"successor":"acme/x","reason":"Long enough reason text here.","extra":"no"}'
bin/acme-actions --as ann request alpha --successor acme/x --reason "$(printf 'x%.0s' {1..201})"

# Self-approval and carol: HTTP 403, exit 3; the run stays awaiting_approval and both refusals are in its record
RUN=$(bin/acme-actions --as ann request gamma --successor acme/graphify-next \
  --reason "Testing approval rules for the handoff." | python3 -c 'import json,sys; print(json.load(sys.stdin)["id"])')
bin/acme-actions --as ann approve "$RUN"
bin/acme-actions --as carol approve "$RUN"
bin/acme-actions --as bob record "$RUN"
bin/acme-actions --as ann reject "$RUN" --comment "test only"   # withdraw it

# Rejected push: the run must end failed (exit 6 from wait), not succeeded
HOOK=/tmp/actions-ab/r4/backstage-3/origins/beta.git/hooks/pre-receive
printf '#!/bin/sh\necho "push rejected for test" >&2\nexit 1\n' > "$HOOK" && chmod +x "$HOOK"
RUN=$(bin/acme-actions --as ann request beta --successor acme/other-store \
  --reason "Testing that a rejected push is not success." | python3 -c 'import json,sys; print(json.load(sys.stdin)["id"])')
bin/acme-actions --as bob approve "$RUN"
bin/acme-actions --as bob wait "$RUN"; echo "exit=$?"   # status failed, reason has git's error
rm "$HOOK"
```

## HTTP API

Base `http://localhost:8010/api/acme-actions`, header `Authorization: Bearer <name>-token-0123456789`
(the static tokens in `backstage/app-config.local.yaml`). JSON in and out.

| Method and path | Purpose |
|---|---|
| `GET /health` | No auth. `{"status":"ok"}` |
| `GET /whoami` | The person the token acts as |
| `GET /actions?repository=&action=` | List action definitions |
| `GET /actions/{repo}/{action}` | One definition |
| `POST /actions/{repo}/{action}/preview` `{"inputs":{…}}` | Validate and dry-run. 400 with `validation` on bad inputs |
| `POST /runs` `{"repository","action","inputs"}` | Request a run (201). Validates and previews first |
| `POST /runs/{id}/approve` `{"comment"?}` | 403 for the requester or a non-maintainer; 409 if not awaiting approval or the definition changed |
| `POST /runs/{id}/reject` `{"comment"?}` | Reject (maintainer) or withdraw (requester) |
| `GET /runs?repository=&action=&status=` | List runs |
| `GET /runs/{id}` | Full record with `events` |
| `POST /refresh` | Reload definitions from the origins now |

Example: `curl -s -H 'Authorization: Bearer ann-token-0123456789' http://localhost:8010/api/acme-actions/actions`.

## Code

| Path | What it is |
|---|---|
| `backstage/packages/backend/src/index.ts` | Adds the four Acme modules. The allow-all permission policy is replaced |
| `backstage/packages/backend/src/acme/actionsPlugin.ts` | The `acme-actions` backend plugin: API, validation, preview, approval, execution, observation |
| `backstage/packages/backend/src/acme/store.ts` | Runs and their event log (knex, SQLite) |
| `backstage/packages/backend/src/acme/catalogModule.ts` | Catalog provider reading the origins' `main` |
| `backstage/packages/backend/src/acme/scaffolderModule.ts` | Scaffolder actions `acme:git:publish-file` and `acme:git:observe-file` |
| `backstage/packages/backend/src/acme/permissionModule.ts` | Permission policy that keeps users away from the scaffolder |
| `backstage/packages/backend/src/acme/origins.ts` | Git helpers and the org's repository list |
| `backstage/app-config.local.yaml` | Port, token restrictions, on-disk database, `acme.org.origin` |
| `org/bin/acme-actions` | The CLI |
