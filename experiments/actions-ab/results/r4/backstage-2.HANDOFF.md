# Handoff: `mark-deprecated`, a repository-owned self-service action on Backstage

**Status:** built and run. `mark-deprecated` is defined in each of `alpha`, `beta` and `gamma`
(committed to its origin's `main`). ann requested it on all three, bob approved, and each run is
reported `succeeded` only after Backstage read `DEPRECATED.md` with the exact content on the
origin's `main`:

| Repository | Run | Observed on `main` at |
|---|---|---|
| alpha | `alpha-20261124125122-fced16` | `5952fc12e4efbbd9ed6230d714edbf9e78e45a2a` |
| beta | `beta-20261124125123-f19a12` | `9d21614bdbe9d207d9df12c00aff09b61f6e42e3` |
| gamma | `gamma-20261124125125-277215` | `29cec30e79fd14c4f3ac54010227805154e9e617` |

Earlier test runs are kept in the audit trail on purpose. `gamma-20261124125003-6dfaa1` failed
because a `pre-receive` hook rejected the push. `gamma-20261124125107-6a6c46` failed because a
`post-receive` hook accepted the push and then moved `main` back, so the change was never observed.
`beta-20261124125114-5c200f` was requested by carol and rejected by bob.
`alpha-20261124125321-0ce302` is the rejected-push recipe below, run after a restart. It shows self-
and carol-approval refused, then `failed` with the hook's rejection. All test hooks have been removed.

## How it fits together

Backstage provides the catalog, the scaffolder (templates, dry-run, task execution and task logs)
and static-token auth. The glue I wrote is in `backstage/packages/backend/src/acme/` and is added in
`backstage/packages/backend/src/index.ts`:

| Piece | File | What it does |
|---|---|---|
| Catalog module `acme-origins` | `catalogModule.ts` | Reads `org/repositories.yaml` and `org/catalog-info.yaml` (people) from `origins/org.git` `main`, then each repository's `catalog-info.yaml` (and its Location targets) **from that repository's origin `main`**, every 10 s. A repository may only define entities named after itself and owned by its own groups. Each entity is stamped with `acme.io/origin`, `acme.io/commit` and `acme.io/source-path`. |
| Scaffolder actions | `scaffolderModule.ts` | `acme:git:commit-files` supports dry-run, which shows the diff against `main`. A real run asks acme-actions for a *grant*, refuses unless its files equal the approved preview and `main` has not changed under them, then commits (author = requester, `Requested-by`/`Approved-by` trailers) and pushes. `acme:git:verify-files` re-reads the origin's `main` and fails unless the content matches. |
| Backend plugin `acme-actions` | `plugin.ts`, `contracts.ts` | The API for list, preview, request, approve/reject, status, record and audit. It stores runs and audit events in its own database, starts the scaffolder task only after a valid approval, and decides `succeeded` only by observing the origin's `main` itself. |
| Org CLI | `org/bin/acme-actions` | Python 3 (standard library only), JSON on stdout, never prompts. |
| Config | `backstage/app-config.local.yaml` | Static tokens (as given), `acme.org.origin`, a catalog rule for `acme-origin` locations, and a file-backed SQLite directory `backstage/.acme-data/` so runs survive restarts. This file is gitignored by the Backstage app. |

Run lifecycle: `awaiting_approval` → (`rejected`) or `dispatching` → `running` → `succeeded` | `failed`.

## R1–R8

| # | How it is met | Files |
|---|---|---|
| R1 Owned by the repository | The action is a scaffolder Template in each repository, committed to its origin's `main`: its inputs schema, wording and steps. The approvers are the repository's own `<repo>-maintainers` group in its `catalog-info.yaml`. Backstage reads both **from the origin's `main`** (not from checkouts), so a team changes its action or its maintainers with a commit to `main`; the change is live within 10 s. A repository cannot define another repository's entities or maintainers. | `alpha/.backstage/mark-deprecated.yaml`, `alpha/catalog-info.yaml` (same in `beta/`, `gamma/`); `backstage/packages/backend/src/acme/catalogModule.ts` |
| R2 Validated | Inputs are checked twice before anything is recorded or changed: by the platform contract (`successor` matches `^[a-z0-9-]+/[a-z0-9-]+$` and is not `acme/<repo>` itself; `reason` is 20–200 characters; no unknown inputs; both required), then by the repository template's own JSON Schema through the scaffolder dry-run. Refusals return HTTP 400 with `errors[]` (CLI exit 1), create no run, and are written to the audit trail as `request_refused`. The scaffolder re-validates when the task is created. | `backstage/packages/backend/src/acme/contracts.ts`; `plugin.ts` (`preview`, `request`) |
| R3 Previewed | `preview` runs the repository's template through Backstage's scaffolder **dry-run**. `acme:git:commit-files` clones `main` and returns the git diff, whether the file is created, updated or unchanged, and the base commit. It changes nothing. Every `request` stores this preview in the run, and the requester sees it in the response before anyone approves. Approval approves exactly that change: the task refuses to push other files, or to push if `DEPRECATED.md` on `main` changed since the preview. Approval is refused if the template changed since the preview. | `plugin.ts` (`preview`), `scaffolderModule.ts` |
| R4 Approved | No scaffolder task exists until an approval is accepted. `approve` refuses the requester (`self_approval`, 403), anyone not in the template owner's group in the catalog (`not_a_maintainer`, 403, which covers carol), and anything not awaiting approval (409). Every refusal is audited as `approval_refused`. Creating a scaffolder task directly, bypassing acme-actions, fails before any change: the actions ask acme-actions for a grant, which is only given to the task it dispatched for an approved run, and only the `plugin:scaffolder` principal may ask. | `plugin.ts` (`approve`, `grant`), `scaffolderModule.ts` (`fetchGrant`) |
| R5 Done means observed | When the task ends, acme-actions reads `refs/heads/main:DEPRECATED.md` from the bare origin itself and compares it byte for byte with the contract's content. The run is `succeeded` only if the task completed **and** the content matches, and the observed commit and time are recorded. A rejected push fails the publish step. An accepted push that does not stay on `main` fails the verify step and the observation. Either way the run is `failed` and the error is in `outcome.message`. | `plugin.ts` (`sync`, `observe`), `scaffolderModule.ts` (`acme:git:verify-files`) |
| R6 One place for the org | From `org/`, `bin/acme-actions list` lists `mark-deprecated` for all three repositories (owner, maintainers, input schema, and the origin, commit and path that define it), and `bin/acme-actions request <repo> …` requests a run on any of them. The org repository also holds the list of repositories and the people. | `org/bin/acme-actions`, `org/repositories.yaml`, `org/catalog-info.yaml` |
| R7 Audited | Each run stores requester and `requestedAt`, approver (or rejecter) with their time and comment, inputs, the preview, `dispatchedAt`, the scaffolder task id, `finishedAt`, and the outcome with the observed commit and time. `record` adds the append-only event trail (`requested`, `previewed`, `approval_refused`, `approved`, `dispatched`, `observed`, `succeeded`/`failed`, each with actor and timestamp) and the scaffolder task log. The commit on `main` carries `Acme-Run`, `Requested-by` and `Approved-by` trailers. Records persist in `backstage/.acme-data/` across restarts. | `plugin.ts` (`runs`/`events` tables, `record`), `backstage/app-config.local.yaml` |
| R8 Usable by agents | Every step is one non-interactive CLI command that prints JSON. Exit codes: 0 ok; 1 refused (4xx); 2 usage error or server unreachable; 3 `wait` saw `failed`/`rejected`, timed out, or the run still needs approval. The same steps are plain HTTP+JSON under `/api/acme-actions` (see below). | `org/bin/acme-actions`, `plugin.ts` |

The content written is exactly `# Deprecated: use <successor>`, a blank line, then `<reason>`, ending
in one newline, which is `printf '# Deprecated: use %s\n\n%s\n'`.

## Start it

```bash
export PATH=/tmp/bs/node24/bin:$PATH
cd /tmp/actions-ab/r4/backstage-2/backstage
tmux new-session -d -s r4-backstage-2-backend \
  'export PATH=/tmp/bs/node24/bin:$PATH; yarn workspace backend start 2>&1 | tee .acme-backend.log'
# ready when this prints JSON (about 30 s):
until curl -sf -H 'Authorization: Bearer ann-token-0123456789' \
  http://localhost:7970/api/acme-actions/whoami; do sleep 2; done
```

Stop it with `tmux kill-session -t r4-backstage-2-backend`. Nothing else needs to run: no frontend
and no external services. Git hooks in `origins/*.git/hooks` are the only push policy.

## Use it (from the org)

```bash
cd /tmp/actions-ab/r4/backstage-2/org
A=bin/acme-actions            # or: --as can be replaced by ACME_AS=ann, or --token / ACME_TOKEN

$A --as ann list                                  # mark-deprecated for alpha, beta, gamma
$A --as ann preview alpha --successor acme/dependabot-proxy-v2 \
   --reason 'Replaced by the v2 proxy, which supports OIDC registries.'      # diff only, no run
$A --as ann request alpha --successor acme/dependabot-proxy-v2 \
   --reason 'Replaced by the v2 proxy, which supports OIDC registries.'      # prints the run; note "id"
RUN=$($A --as ann request beta --successor acme/valtio-store \
   --reason 'State management moves to the shared store package.' \
   | python3 -c 'import json,sys; print(json.load(sys.stdin)["id"])')
$A --as bob approve "$RUN" --comment 'ok'          # starts the scaffolder task
$A --as ann status "$RUN"                          # status + outcome
$A --as ann wait "$RUN" --timeout 120              # exit 0 only if succeeded
$A --as ann record "$RUN"                          # full audit record + task log
$A --as ann runs [--repository alpha] [--status failed]
$A --as ann audit [--run "$RUN"] [--unattached]    # --unattached: refused requests
$A --as bob reject "$RUN" --comment 'why'          # maintainer, or the requester withdrawing
```

`--input key=value` and `--inputs-json '{…}'` work for any input, and `--action` defaults to
`mark-deprecated`. People and tokens for `--as` are in `org/sandbox-tokens.json`, a copy of the
sandbox tokens in `app-config.local.yaml`. The server identifies the person from the token; the CLI
only picks which token to send.

The same steps over HTTP (send `Authorization: Bearer <token>`; bodies are JSON):

| Step | Request |
|---|---|
| list | `GET /api/acme-actions/actions[?repository=alpha]` |
| preview | `POST /api/acme-actions/actions/<repo>/mark-deprecated/preview` `{"inputs":{"successor":…,"reason":…}}` |
| request | `POST /api/acme-actions/actions/<repo>/mark-deprecated/runs` (same body) → 201 run |
| approve / reject | `POST /api/acme-actions/runs/<id>/approve` / `…/reject` `{"comment":…}` |
| status | `GET /api/acme-actions/runs/<id>` |
| record | `GET /api/acme-actions/runs/<id>/record` |
| runs, audit | `GET /api/acme-actions/runs`, `GET /api/acme-actions/audit[?run=<id>]` |

## Test recipes

All three repositories are already deprecated, so a new run is an *update* of `DEPRECATED.md`. The
preview shows `"change": "update"`; a run with identical inputs is a no-op that still succeeds once
the content is observed.

```bash
cd /tmp/actions-ab/r4/backstage-2/org; A=bin/acme-actions
# Invalid inputs: each exits 1 with "error":"invalid_inputs", creates no run, and changes no origin.
$A --as ann request alpha --successor 'Acme/X' --reason 'twenty characters ok!'
$A --as ann request alpha --successor acme/ok --reason 'too short'
$A --as ann request alpha --successor acme/ok --reason "$(printf 'x%.0s' $(seq 201))"
$A --as ann request alpha --successor acme/alpha --reason 'pointing at itself is refused'
$A --as ann request alpha --reason 'successor is missing here'

# Self-approval and carol: 403 self_approval / not_a_maintainer; the run stays awaiting_approval.
RUN=$($A --as ann request alpha --successor acme/proxy-v3 --reason 'Testing the approval rules here.' \
  | python3 -c 'import json,sys; print(json.load(sys.stdin)["id"])')
$A --as ann approve "$RUN"; $A --as carol approve "$RUN"; $A --as ann status "$RUN"

# Rejected push: the run must end failed, never succeeded.
printf '#!/bin/sh\necho "main is protected" >&2\nexit 1\n' > ../origins/alpha.git/hooks/pre-receive
chmod +x ../origins/alpha.git/hooks/pre-receive
$A --as bob approve "$RUN"; $A --as ann wait "$RUN"      # exit 3, "status": "failed", outcome names the rejection
rm ../origins/alpha.git/hooks/pre-receive
git -C ../origins/alpha.git show main:DEPRECATED.md       # unchanged

# Bypass: a scaffolder task created directly is refused before it changes anything.
curl -s -X POST -H 'Authorization: Bearer ann-token-0123456789' -H 'Content-Type: application/json' \
  http://localhost:7970/api/scaffolder/v2/tasks \
  -d '{"templateRef":"template:default/alpha-mark-deprecated","values":{"successor":"acme/x","reason":"skipping the approval step"}}'
# then GET /api/scaffolder/v2/tasks/<id> → "failed" (acme-actions did not grant task …)
```

## Limits and choices

- **Who may request:** anyone in the catalog (carol included). Who may approve: members of the
  template owner's group, read from the catalog at approval time, excluding the requester.
- **Sandbox identity:** static tokens give a service principal whose subject is the person's name.
  acme-actions maps it to `user:default/<name>` and refuses names that are not Users in the catalog,
  and refuses backend plugins acting as people.
- **Catalog lag:** a change to a template or to maintainers on `main` is picked up within 10 s
  (`acme.catalogRefreshSeconds`). `POST /api/catalog/acme-origins/refresh` forces a refresh.
- **The contract is the platform's:** a repository may change its template's wording, steps and
  stricter input rules. If its template would write anything other than the `mark-deprecated`
  content, preview refuses (`contract_mismatch`), so the spec stays enforceable.
- **The Backstage app** is a separate local git repository at `backstage/` with no remote. My glue is
  committed there. The `packages/app/package.json` and `yarn.lock` changes in its working tree
  predate this work and are not mine.
