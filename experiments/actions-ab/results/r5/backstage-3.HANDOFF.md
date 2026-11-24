# HANDOFF: `mark-deprecated`, a repository-owned self-service action on Backstage

**Status.** `mark-deprecated` is defined in each of `alpha`, `beta` and `gamma` (committed to each
origin's `main`) and runs through Backstage. ann requested it on all three repositories, bob
approved each run, and all three are reported `succeeded` after the system observed
`DEPRECATED.md` with the right content on each origin's `main`:

| Repository | Run id | Commit on `main` (observed) |
|---|---|---|
| alpha | `04c679fa-20e5-44f0-9688-f935fb0404e2` | `40613624f82da29b9bcbad6016ccd4afd0bce5bb` |
| beta | `edc019db-141d-4d4b-88ef-2a75b6d0b429` | `cf1ef4e67392b9d0c24c79adc2c874aa82dd1f93` |
| gamma | `ff1c9c26-32cd-445c-807a-ef296ff30cad` | `63b8d2dd6f7ada222c1e8a4b40464070951a977b` |

The backend is **stopped**. Start it with the command in [Start](#start) before you test.

## How it works

```
org/bin/acme (CLI, JSON out)  ──HTTP + static token──▶  Backstage backend :7510
                                                        ├─ acme-actions plugin   (/api/acme-actions)
                                                        │    validate · preview · request · approve
                                                        │    · run · observe main · audit (sqlite)
                                                        ├─ catalog module acme-git-origins
                                                        │    reads org.git main: repos.yaml, people
                                                        │    reads <repo>.git main: catalog-info.yaml
                                                        │    → Template <repo>/mark-deprecated
                                                        └─ scaffolder module acme-repo-actions
                                                             acme:repo:write-file  (dry-run plan; approval gate; push)
                                                             acme:repo:verify-file (reads main on the origin)
```

1. **Definition.** Each repository's `catalog-info.yaml` lists `.backstage/actions/mark-deprecated.yaml`,
   a Backstage scaffolder Template. The `acme-git-origins` catalog provider reads it from `main` on
   the repository's origin every 3 seconds. It forces the entity into a namespace named after the
   repository (`template:alpha/mark-deprecated`), so one repository cannot define another's
   actions. It also records the definition's commit and blob.
2. **Preview.** `acme-actions` validates the inputs against the template's own parameter schema
   and runs it through the scaffolder's dry-run. `acme:repo:write-file` dry-runs by reading `main` on
   the origin and returns the plan: path, content before and after, and a diff.
3. **Request.** This stores the run with the preview, the definition commit and blob, the
   requester and a timestamp. Status: `awaiting-approval`.
4. **Approve.** The approver must belong to the template's owner group, here
   `group:default/<repo>-maintainers` from `org/catalog/people.yaml`, and must not be the requester.
   If the definition or the target file changed on `main` since the preview, the run is `expired`
   and must be requested again: you approve what you saw.
5. **Execute.** `acme-actions` creates a scaffolder task. The real `acme:repo:write-file` first
   calls the approval gate (`/internal/runs/:id/gate`, scaffolder-only, with a one-time nonce
   passed as a task secret). It refuses unless the run is approved and its path and content are
   exactly the approved preview. Then it clones, commits (author = requester, with `Acme-Run`,
   `Requested-by` and `Approved-by` trailers) and pushes to `main`.
6. **Observe.** When the task ends, `acme-actions` itself reads `main` on the origin. The run is
   `succeeded` only if the task completed **and** every planned file is on `main` with the approved
   content. Otherwise it is `failed`, with the error and the observation.

Run statuses: `awaiting-approval` → `executing` → `succeeded` | `failed`, or `awaiting-approval` →
`rejected` | `cancelled` | `expired`.

## R1–R8

| # | How it is met | Files |
|---|---|---|
| R1 Owned by the repository | The action is a scaffolder Template committed to each repository's origin `main`, listed from the repository's `catalog-info.yaml`, owned by `group:default/<repo>-maintainers`. The catalog reads it **from the origin's `main`** (not a checkout). A change pushed to `main` (inputs, content, steps) becomes the action within seconds. Runs record the definition commit and blob they used. Entities from a repository are confined to its namespace. | `alpha/catalog-info.yaml`, `alpha/.backstage/actions/mark-deprecated.yaml` (same in `beta/`, `gamma/`); `backstage/packages/backend/src/acme/catalogModule.ts`; registry `org/repos.yaml` |
| R2 Validated | Inputs are checked against the template's JSON Schema: `successor` matches `^[a-z0-9-]+/[a-z0-9-]+$` and is not the repository itself; `reason` is 20–200 characters; both are required; unknown inputs are refused. This happens on preview **and** request, before any run is stored or anything changes (HTTP 400 with per-field `details`). The scaffolder re-validates on task creation, and the action validates its own inputs. | `.backstage/actions/mark-deprecated.yaml` (`spec.parameters`); `actionsPlugin.ts` (`validate`) |
| R3 Previewed | `acme preview` (no side effects) and every `acme request` run the repository's template in scaffolder dry-run mode. `acme:repo:write-file` returns the plan: `operation` (create/update/none), `before`, `after`, unified `diff` and `baseCommit`. The preview is stored on the run and shown by `status` and `record`, so the approver sees it too. | `scaffolderModule.ts` (dry-run branch), `actionsPlugin.ts` (`preview`) |
| R4 Approved | Nothing is pushed until a member of the owner group who is not the requester approves. The requester's own approval is refused (403), and so is carol's (403, not a maintainer). Every denied attempt is written to the run's timeline as `approval-denied`. People's tokens are restricted to `acme-actions` and catalog reads (`accessRestrictions`), so nobody can create a scaffolder task directly. The push action also refuses to run without the gate's approval for that exact content. | `actionsPlugin.ts` (`approve`, `gate`), `scaffolderModule.ts` (`fetchGate`), `backstage/app-config.local.yaml`, `org/catalog/people.yaml` |
| R5 Done means observed | After the scaffolder task ends, `acme-actions` reads `refs/heads/main:DEPRECATED.md` on the origin and compares it with the approved content. It reports `succeeded` only if the task completed and the content matches. A rejected push fails the task (`failed`, with git's error and an observation showing the file absent). The template's own `acme:repo:verify-file` step also fails the task if `main` lacks the content. | `actionsPlugin.ts` (`observe`), `scaffolderModule.ts` |
| R6 One place for the org | `org/bin/acme` lists `mark-deprecated` for all repositories in `org/repos.yaml` and requests, approves and inspects runs on any of them, without entering a repository. | `org/bin/acme`, `org/acme.json`, `org/repos.yaml` |
| R7 Audited | Each run stores `requestedBy/At`, `approvedBy/At`, `closedBy/At` (reject or cancel), `parameters`, the preview, the definition commit, `startedAt`, `finishedAt` and `outcome` (task status, error, observed commit). It also keeps an append-only timeline: `requested`, `previewed`, `approval-denied`, `approved`, `execution-started`, `gate-passed`, `succeeded`/`failed`/`rejected`/`cancelled`/`expired`, each with an actor and an ISO timestamp. Records are stored in sqlite and survive restarts. Commits on `main` also carry `Acme-Run`, `Requested-by` and `Approved-by` trailers. | `actionsPlugin.ts` (`acme_runs`, `acme_run_events`); `backstage/.data/acme-actions.sqlite` |
| R8 Usable by agents | Every step is a non-interactive CLI command or plain HTTP call with JSON on stdout. Errors are JSON too (`{status, error: {name, message, details}}`). Exit codes: `0` ok, `1` refused or failed request, `2` usage, `3` run finished without succeeding, `4` wait timed out. `approve --wait` and `wait` block until the run finishes. | `org/bin/acme` |

## Start

```bash
export PATH=/tmp/bs/node24/bin:$PATH
tmux new-session -d -s r5-backstage-3-backend -c /tmp/actions-ab/r5/backstage-3/backstage \
  'PATH=/tmp/bs/node24/bin:$PATH yarn workspace backend start > /tmp/actions-ab/r5/backstage-3/backstage/.backend.log 2>&1'
# ready when this returns (the catalog loads people and actions from git a few seconds after boot)
until curl -sf -o /dev/null -H 'Authorization: Bearer ann-token-0123456789' \
  http://localhost:7510/api/acme-actions/whoami; do sleep 2; done; echo ready
```

Stop it with `tmux kill-session -t r5-backstage-3-backend`. The log is in `backstage/.backend.log`.
Runs persist in `backstage/.data/`. Delete that directory while the backend is stopped to start from
an empty history.

## Use it (from the org place)

```bash
cd /tmp/actions-ab/r5/backstage-3/org
export PATH=/tmp/bs/node24/bin:$PATH

# list mark-deprecated on all three repositories (owner, maintainers, inputs, definition commit)
bin/acme --as ann actions --action mark-deprecated
bin/acme --as ann action beta mark-deprecated          # one action

# preview (no side effects)
bin/acme --as ann preview alpha mark-deprecated \
  --successor acme/dependabot-proxy-v3 --reason "Replaced by the v3 proxy, which supports OIDC registries."

# request as ann (validated and previewed); prints the run, including "id"
RUN=$(bin/acme --as ann request alpha mark-deprecated \
  --successor acme/dependabot-proxy-v3 --reason "Replaced by the v3 proxy, which supports OIDC registries." | jq -r .id)

# approve as bob; --wait blocks until the run finishes (exit 0 only if it succeeded)
bin/acme --as bob approve "$RUN" --wait

bin/acme --as ann status "$RUN"        # status, preview, outcome
bin/acme --as ann record "$RUN"        # full audit record with timeline
bin/acme --as ann runs --repo alpha    # list runs (also --status, --action)
bin/acme --as bob reject "$RUN" --reason "..."   # maintainer rejects; the requester's own reject cancels
```

Inputs can also be given as `--param successor=acme/x --param reason=...` or
`--params '{"successor":"acme/x","reason":"..."}'`. Identity is `--as <person>` (or `ACME_AS`), which
picks that person's static token from `org/acme.json`; `ACME_TOKEN` overrides it.

The same steps over HTTP (base `http://localhost:7510/api/acme-actions`, header
`Authorization: Bearer <person>-token-0123456789`):

| Step | Request |
|---|---|
| list | `GET /actions?action=mark-deprecated` (optional `repository=`) |
| show | `GET /actions/{repo}/{action}` |
| preview | `POST /actions/{repo}/{action}/preview` with body `{"parameters":{...}}` |
| request | `POST /actions/{repo}/{action}/runs` with body `{"parameters":{...}}` (201) |
| approve | `POST /runs/{id}/approve` with body `{"comment":"..."}` (optional) |
| reject / cancel | `POST /runs/{id}/reject` with body `{"reason":"..."}` |
| status | `GET /runs/{id}` |
| record | `GET /runs/{id}/record` |
| list runs | `GET /runs?repository=&status=&action=` |

Example: `curl -s -X POST -H 'Authorization: Bearer ann-token-0123456789' -H 'Content-Type: application/json' -d '{"parameters":{"successor":"acme/x","reason":"long enough reason text"}}' http://localhost:7510/api/acme-actions/actions/gamma/mark-deprecated/preview`

## Test the rules

All three repositories already contain `DEPRECATED.md` from the runs above. A run with the **same**
inputs previews `operation: none` and pushes nothing. Use a different `successor` or `reason` when
you want a push.

| Test | Command | Expected |
|---|---|---|
| bad successor | `bin/acme --as ann request alpha mark-deprecated --successor Acme/Foo --reason "twenty characters or more"` | exit 1, HTTP 400 `successor does not match pattern`; no run stored |
| self as successor | `... request alpha mark-deprecated --successor acme/alpha --reason "twenty characters or more"` | 400 `successor must not be "acme/alpha"` |
| reason too short / too long | `--reason short` or `--reason "$(printf 'a%.0s' {1..201})"` | 400 `minimum length of 20` / `maximum length of 200` |
| missing or unknown input | omit `--successor`, or add `--colour red` | 400 `successor is required` / `colour is not an input of this action` |
| self-approval | `bin/acme --as ann approve "$RUN"` (run requested by ann) | exit 1, 403; `approval-denied` in the record; run still `awaiting-approval` |
| carol approves | `bin/acme --as carol approve "$RUN"` | exit 1, 403 `not a maintainer`; `approval-denied` in the record |
| carol requests | `bin/acme --as carol request ...` then `bin/acme --as bob approve <id> --wait` | carol may request; ann or bob may approve; carol may not approve her own run |
| bypass the approval flow | `curl -s -X POST -H 'Authorization: Bearer ann-token-0123456789' http://localhost:7510/api/scaffolder/v2/tasks` | 403 `This token's access is restricted to plugin(s) 'acme-actions', 'catalog'` |
| rejected push | see below | run `failed`, `outcome.error` contains git's rejection, `observations[0].matches: false`, exit 3 |

Rejected push:

```bash
cd /tmp/actions-ab/r5/backstage-3/org
printf '#!/bin/sh\necho "main is protected" >&2\nexit 1\n' > ../origins/beta.git/hooks/pre-receive
chmod +x ../origins/beta.git/hooks/pre-receive
RUN=$(bin/acme --as ann request beta mark-deprecated --successor acme/valtio-next \
  --reason "Testing that a rejected push is not reported as success." | jq -r .id)
bin/acme --as bob approve "$RUN" --wait; echo "exit=$?"      # status "failed", exit=3
bin/acme --as ann record "$RUN" | jq '{status, outcome: {error: .outcome.error, observations: .outcome.observations}}'
rm ../origins/beta.git/hooks/pre-receive
```

Changing the definition: edit `.backstage/actions/mark-deprecated.yaml` in a repository checkout
(for example, `maxLength`), commit and push to `main`. Within a few seconds `acme action <repo>
mark-deprecated` shows the new commit and parameters. A run requested before the change, but not
yet approved, becomes `expired` when someone tries to approve it.

## Notes and limits

- **Identity.** There are no passwords in this sandbox. A static token identifies the caller (the
  token's subject is the person), and `--as` just picks that token. The request body never
  carries identity. Callers must be a `User` in `org/catalog/people.yaml`.
- **Maintainers** are the `<repo>-maintainers` groups in `org/catalog/people.yaml` on `org.git`
  `main`. Each template's `spec.owner` names its group. One approval from a maintainer other than
  the requester is required.
- **Catalog freshness.** The catalog re-reads the origins every 3 seconds. `preview` and `request`
  wait up to about 10 seconds for the catalog to match the definition on `main`, then return 409.
- The glue lives in `backstage/packages/backend/src/acme/` (registered in
  `backstage/packages/backend/src/index.ts`) and in `backstage/app-config.local.yaml`, committed
  in the `backstage` checkout. It uses packages that the create-app install already hoists (`yaml`,
  `jsonschema`, `express-promise-router`, `@backstage/plugin-scaffolder-node`,
  `@backstage/plugin-catalog-node`). They are not added to `packages/backend/package.json`, so no
  `yarn install` is needed.
- Before the three runs above, the rules were exercised on alpha: self-approval, carol, rejected
  push, definition drift, file drift, reject and cancel. Drift test commits were rolled back on
  the origin, and that test database was deleted. The history now holds only the three real runs.
