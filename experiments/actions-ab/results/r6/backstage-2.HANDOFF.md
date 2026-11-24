# Handoff: `mark-deprecated` on Backstage

`mark-deprecated` is a Backstage scaffolder template that lives in each repository. It runs behind
an approval gate: a run takes effect only after a second maintainer approves it, and it is reported
as succeeded only after `DEPRECATED.md` is observed on the origin's `main`. All three runs below were
requested by ann, approved by bob and reported `succeeded`:

| Repository | Run id | Observed commit on `main` |
|---|---|---|
| alpha | `8a707608-7db0-43fd-b4c8-5e96b6c1af08` | `5499e8e` |
| beta | `597aadc6-d418-4373-bee5-4d66efcdf4f0` | `d4c44c8` |
| gamma | `099e6c18-7395-4b82-b04f-6e63e1c7ca5e` | `21e9ab7` |

The audit log also holds three test runs: two alpha runs that failed on purpose (a rejected push,
and a push that `main` then lost) and one gamma run that bob rejected. They stay in the log by
design.

## How it fits together

| Piece | Where | What it does |
|---|---|---|
| Action definition (one per repo) | `<repo>/.backstage/mark-deprecated.yaml`, listed by `<repo>/catalog-info.yaml` | A scaffolder `Template`: input schema, the step that writes `DEPRECATED.md`, and the owner group. Lives on the repo's origin `main` |
| People and maintainer groups | `org/catalog/people.yaml` (listed by `org/catalog-info.yaml`) | Users ann, bob, carol. Groups `alpha-maintainers`, `beta-maintainers`, `gamma-maintainers` (ann, bob) and `developers` (carol) |
| Catalog module | `backstage/packages/backend/src/acme/catalogModule.ts` | Entity provider. Every 3 s, reads `catalog-info.yaml` and its Location targets from each bare origin's `main` (never a checkout). It stamps the repository and source commit on each entity, puts a repo's entities in that repo's namespace (`template:alpha/mark-deprecated`), and drops `User`/`Group` entities from anywhere but the org origin |
| Scaffolder action | `backstage/packages/backend/src/acme/scaffolderModule.ts` (`acme:git:commit-file`) | Dry run: reports the change and writes nothing. Real run: claims the approved run, clones the origin, commits, pushes `main`, re-reads `main` from the origin, and fails unless the content matches. It writes only to the repository its template came from |
| Approval gate and audit | `backstage/packages/backend/src/acme/plugin.ts` (plugin `acme-actions`, `/api/acme-actions`) | List, preview, request, approve, reject, status, record. Stores runs and an append-only event log in SQLite. Creates the scaffolder task only after approval, then reads the origin's `main` again before it reports success |
| Shared helpers | `backstage/packages/backend/src/acme/shared.ts` | Config, git helpers (run with no global or system git config) |
| Wiring | `backstage/packages/backend/src/index.ts` (last three `backend.add`) | Registers the three features |
| Config | `backstage/app-config.local.yaml` (git-ignored by create-app) | Origins under `acme.actions`, token-to-user mapping, `accessRestrictions` on the three static tokens, catalog rules, SQLite at `backstage/.acme-data/` |
| Org CLI | `org/bin/acme-actions` (Node, no dependencies) | The one place for the org. Every command prints JSON |

The Backstage changes are committed in the app's local git repository (`backstage`, commit
`b6b15a8`). `app-config.local.yaml` is git-ignored there, so it is not in that commit.

## R1 to R8

| # | How it is met | Files |
|---|---|---|
| R1 Owned by the repository | Each repo's definition is committed on its origin's `main`: `.backstage/mark-deprecated.yaml`, listed by `catalog-info.yaml`. Backstage reads it from the bare origin's `main` (`git cat-file`), not from a checkout, and refreshes every 3 s. `list` shows the source commit (`definition.commit`). `spec.owner` names the group that may approve. The team changes the action by pushing to `main`. A repo can only define entities in its own namespace, and the action can only write to the repo its template came from. If a definition changes while a run awaits approval, the approval is refused (409) and the run must be requested again | `alpha/.backstage/mark-deprecated.yaml`, `alpha/catalog-info.yaml` (same in `beta/`, `gamma/`), `backstage/packages/backend/src/acme/catalogModule.ts` |
| R2 Validated | The template's JSON Schema has `successor` matching `^[a-z0-9-]+/[a-z0-9-]+$` and not the repo itself (`not: {const: acme/<repo>}`), and `reason` of 20 to 200 characters, both required. `acme-actions` validates against it on preview, request and approval, and rejects unknown inputs. The scaffolder's dry run validates again. Invalid input returns HTTP 400 (CLI exit code 3) with per-input `errors`, before any run is stored or anything changes | `<repo>/.backstage/mark-deprecated.yaml`, `plugin.ts` (`validateInputs`) |
| R3 Previewed | `preview` runs Backstage's scaffolder dry run (`POST /api/scaffolder/v2/dry-run`) on the repo's template. The action's dry-run branch reads the origin's `main` and returns `operation` (create, update or none), `before`, `after` and a unified `diff`. `request` stores the same preview on the run, so the approver sees it in `record` | `scaffolderModule.ts` (`isDryRun`), `plugin.ts` (`preview`) |
| R4 Approved | A request is stored as `awaiting_approval`; nothing runs. `approve` succeeds only if the approver is a member of the template's `spec.owner` group (catalog relations from `org/catalog/people.yaml`) and is not the requester. Otherwise HTTP 403, and an `approval_denied` event is logged. Identity comes from the bearer token and is never taken from the request body. People cannot bypass the gate: their tokens' `accessRestrictions` deny scaffolder task creation and dry runs (403). Also, the action refuses to push unless it can claim an approved run whose template and parameters match its task | `plugin.ts` (`/runs/:id/approve`, `/internal/runs/:id/claim`), `backstage/app-config.local.yaml`, `org/catalog/people.yaml` |
| R5 Done means observed | The action fails the task if the push is rejected, or if `main` on the origin does not then hold the exact content. On task completion, `acme-actions` reads the origin's `main` again itself, and only then sets `succeeded`, recording the observed commit, SHA-256 and time in `outcome.observed`. A failed or cancelled task, or a mismatch, gives `failed` with the git error. Both cases were tested on alpha: a rejecting `pre-receive` hook, and a `post-receive` hook that resets `main` | `scaffolderModule.ts`, `plugin.ts` (`sync`) |
| R6 One place for the org | `org/bin/acme-actions` lists `mark-deprecated` for all repos, and previews, requests, approves and inspects runs on any of them through one API (`/api/acme-actions`). People, teams and the CLI live in `org` | `org/bin/acme-actions`, `org/catalog/` |
| R7 Audited | Each run stores the requester and request time, inputs, the definition commit and spec hash, the preview, the approver and approval time (or rejecter), the scaffolder task id, the finish time and the outcome. It also has an append-only event log, with actor and time, covering `requested`, `approval_denied`, `approved`, `rejected`, `started`, `executing`, `pushed` or `already_present`, and `succeeded` or `failed`. Runs persist across restarts (SQLite in `backstage/.acme-data/`). Each pushed commit also carries `Acme-Run`, `Requested-by` and `Approved-by` trailers, with the requester as author | `plugin.ts` (tables `acme_action_runs`, `acme_action_run_events`) |
| R8 Usable by agents | Every step is one non-interactive CLI command or HTTP call that returns JSON. Exit codes: 0 ok; 1 run failed or rejected, or another error; 2 usage; 3 invalid inputs; 4 not allowed or not authenticated; 5 not found or conflict; 7 `wait` timed out. `wait` blocks until the run reaches a final state | `org/bin/acme-actions` |

## Commands

All commands assume this shell setup:

```bash
export PATH=/tmp/bs/node24/bin:$PATH
cd /tmp/actions-ab/r6/backstage-2
A=org/bin/acme-actions          # identity: --as ann|bob|carol, or --token <t>, or $ACME_TOKEN
```

### Start (and stop) Backstage

```bash
tmux new-session -d -s r6-backstage-2-backend -c /tmp/actions-ab/r6/backstage-2/backstage \
  'export PATH=/tmp/bs/node24/bin:$PATH; yarn workspace backend start 2>&1 | tee .acme-backend.log'
until $A --as ann whoami >/dev/null 2>&1; do sleep 2; done   # ready (about 15 s)

tmux kill-session -t r6-backstage-2-backend                  # stop
```

You can also run it in the foreground: `cd backstage && yarn workspace backend start`. It listens on
http://localhost:7670. To start with an empty audit log, stop it and delete `backstage/.acme-data/`.

### Each step

```bash
$A --as ann list                                   # mark-deprecated on alpha, beta, gamma
$A --as ann describe alpha                         # inputs schema, owner, maintainers, source commit

$A --as ann preview alpha --successor acme/dependabot-proxy-v2 \
   --reason "Replaced by the v2 proxy, which supports OIDC registries."

RUN=$($A --as ann request alpha --successor acme/dependabot-proxy-v2 \
   --reason "Replaced by the v2 proxy, which supports OIDC registries." | jq -r .id)

$A --as bob approve "$RUN" [--comment "ok"]
$A --as bob reject  "$RUN" [--comment "no"]        # alternative to approving

$A --as ann status "$RUN"                          # status, who and when, outcome
$A --as ann wait   "$RUN"                          # block until succeeded/failed/rejected
$A --as ann record "$RUN"                          # full record, preview and event log
$A --as ann runs [--repository alpha] [--status failed]
```

Run statuses: `awaiting_approval`, then `running`, then one of `succeeded`, `failed` or `rejected`.
Inputs can also be given as `--input reason=...` or `--values '{"successor":"…","reason":"…"}'`.

The same calls over HTTP:

```bash
H='Authorization: Bearer ann-token-0123456789'; B=http://localhost:7670/api/acme-actions
curl -s -H "$H" $B/actions
curl -s -H "$H" -H 'content-type: application/json' -X POST $B/actions/alpha/mark-deprecated/preview -d '{"values":{"successor":"acme/x","reason":"twenty characters at least"}}'
curl -s -H "$H" -H 'content-type: application/json' -X POST $B/actions/alpha/mark-deprecated/runs    -d '{"values":{…}}'
curl -s -H 'Authorization: Bearer bob-token-0123456789' -X POST $B/runs/$RUN/approve
curl -s -H "$H" $B/runs/$RUN            # status
curl -s -H "$H" $B/runs/$RUN/record     # record
```

## Testing the rules

Each case below was run, and its output matched what is described.

| Case | Command | Expected |
|---|---|---|
| Invalid inputs | `$A --as ann request beta --successor Acme/X --reason short` (also: `acme/beta` as successor, a 201-character reason, an extra `--foo 1`) | HTTP 400, exit 3, `error.errors` per input. No run is stored (`$A --as ann runs`) |
| Self-approval | `RUN=$($A --as ann request …)`, then `$A --as ann approve $RUN` | HTTP 403, exit 4: "requested this run and cannot approve it". `approval_denied` appears in the record |
| carol approves | `$A --as carol approve $RUN` | HTTP 403, exit 4: "is not a maintainer of …". Logged |
| Bypassing the gate | `curl -X POST localhost:7670/api/scaffolder/v2/tasks -H 'Authorization: Bearer ann-token-0123456789' …` | 403 (the same for `/v2/dry-run` and catalog writes) |
| Rejected push | `printf '#!/bin/sh\necho frozen >&2\nexit 1\n' > origins/alpha.git/hooks/pre-receive; chmod +x origins/alpha.git/hooks/pre-receive`, then request with **new** inputs, `$A --as bob approve $RUN`, `$A --as bob wait $RUN` | `failed`, exit 1, `outcome.error` quotes git's `pre-receive hook declined`, `main` unchanged. Remove the hook afterwards |

Requesting inputs that `main` already holds gives a preview of `operation: none`. Once approved, that
run succeeds without pushing (the event is `already_present`), because the file is observed with the
right content. Use different inputs when you test a rejected push.

## Limits

- Identity is the sandbox's static bearer tokens. `acme.actions.identities` in
  `app-config.local.yaml` maps each token's subject to a catalog user, and the CLI's `--as` flag
  only selects that sandbox token. With a real sign-in provider, user tokens work unchanged.
- The catalog polls the origins every 3 s. `list` also triggers a refresh, but the catalog can take
  a few seconds to process an edited definition.
- Approval rules are code in `plugin.ts`: any member of the owner group, other than the requester.
  Each repository chooses which group owns its action, but group membership is maintained in `org`.
