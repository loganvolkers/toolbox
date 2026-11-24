# Handoff: `mark-deprecated` on Backstage

`mark-deprecated` is a Backstage scaffolder template that each repository defines and owns on its
origin's `main`. A run is one scaffolder task. It validates its inputs, previews the change, waits
for approval by a maintainer other than the requester, pushes `DEPRECATED.md` to the origin's `main`,
and is reported as succeeded only after the system has read the file back from `main`. Runs are
requested and approved from this directory with `bin/acme-actions`, which prints JSON.

All three runs have succeeded: requested by ann, approved by bob, with the file observed on `main`.

| Repository | Run id | Commit on `main` | State |
|---|---|---|---|
| alpha | `38bfd501-6baa-43d5-8696-5aa9cf314500` | `50affcb` | succeeded |
| beta | `d240d22a-9a11-4886-9429-edc8c54c179a` | `989834f` | succeeded |
| gamma | `2c5d1e81-7140-4efe-86d8-3c8e07e5f45b` | `3b48209` | succeeded |

The run history (`bin/acme-actions --as ann runs`) also keeps the test runs:
- rejected pushes: beta `7fb818d2…` and alpha `c772e24d…`, both failed with `main` unchanged;
- a run bob rejected: beta `aefd8644…`;
- cancelled runs: gamma `0f881672…` and alpha `4ebe53b2…`, after self-approval and carol's
  approval were refused. Gamma `c56ad673…`, cancelled before cancellations were labelled, shows as
  failed.

## Moving parts

| Part | Path | Role |
|---|---|---|
| Action definition (one per repository) | `<repo>/.backstage/mark-deprecated.yaml` on each origin's `main` | Scaffolder Template `template:<repo>/mark-deprecated`: input schema, steps, owner |
| Repository catalog entry | `<repo>/catalog-info.yaml` | Group `group:<repo>/maintainers` (ann, bob), Component `<repo>`, and a Location pointing at the template |
| Org catalog | `org/catalog/org.yaml` | Users ann, bob, carol, and the list of repositories |
| Org CLI | `org/bin/acme-actions` | list / show / preview / request / approve / reject / status / record / runs / cancel (Python 3, stdlib only) |
| Catalog module | `backstage/packages/backend/src/acme/catalogGitOrigin.ts` | Location type `git-origin`: reads entity files from `origins/<repo>.git` at `main` |
| Scaffolder actions | `backstage/packages/backend/src/acme/scaffolderActions.ts` | `acme:deprecation:plan`, `acme:approval:await`, `acme:repo:commit-file`, `acme:repo:verify-file` |
| Runs plugin | `backstage/packages/backend/src/acme/runsPlugin.ts` | `/api/acme-runs`: approvals, audit events, run state |
| Shared helpers | `backstage/packages/backend/src/acme/git.ts` | git calls, input rules, diff |
| Wiring and config | `backstage/packages/backend/src/index.ts`, `backstage/app-config.local.yaml` | registers the three features; sets the origins root, sqlite persistence in `backstage/.data/`, and the org catalog location |

## Requirements

| # | How it is met |
|---|---|
| R1 Owned by the repository | Each repository's definition is `.backstage/mark-deprecated.yaml`, committed on its origin's `main` together with `catalog-info.yaml`, which names `group:<repo>/maintainers` as owner. Backstage reads both straight from `origins/<repo>.git` at `main` (`catalogGitOrigin.ts`), so only what has landed on `main` counts. An edit is live within about 10 s (`catalog.processingInterval: 5s`). When a run is registered, `runsPlugin.ts` (`POST /internal/runs`) checks that the template's `backstage.io/managed-by-location` is that repository's own origin and that its namespace matches. `commit-file` refuses to write to any repository other than the template's namespace, so alpha's definition cannot change beta. |
| R2 Validated | The template's JSON Schema has `successor` matching `^[a-z0-9-]+/[a-z0-9-]+$` and not `acme/<self>`, `reason` of 20–200 characters, both required, and no other properties. The scaffolder checks it on `POST /v2/tasks` and `POST /v2/dry-run`, and refuses with HTTP 400 before any task exists. `acme:deprecation:plan` checks the same rules again (`git.ts`, `validateDeprecationInputs`) before registering the run, and so does the plugin. A refused input never reaches git. |
| R3 Previewed | `acme-actions preview` runs the template through the scaffolder's dry-run (`POST /api/scaffolder/v2/dry-run`). `plan` reads the current `main` and returns the unified diff, the new content and the base commit, and nothing is written. Every real run also stores this preview when it is requested (`preview` in the record), so the requester and the approver see exactly what will be pushed before approving. The content pushed is the stored, approved content. |
| R4 Approved | The `acme:approval:await` step blocks until the run is approved (`runsPlugin.ts`, `POST /runs/:id/approve`). The approver is identified from their token: a static token's subject maps to `user:default/<subject>`, which must exist in the catalog. Approval is refused (HTTP 403) when the approver is the requester, or is not a member of the template's owner group as resolved from the catalog (carol is not). A decided, failed or cancelled run returns HTTP 409. `commit-file` re-checks the approval server-side (`POST /internal/runs/:id/apply`) before pushing. Refused attempts are recorded as `approve-refused` events. |
| R5 Done means observed | After the push, `acme:repo:verify-file` asks the plugin to verify (`POST /internal/runs/:id/verify`). The plugin reads `DEPRECATED.md` from `refs/heads/main` of the origin and compares it to the approved content. A mismatch or a missing file fails the step, and so the task. The run is reported `succeeded` only if the scaffolder task completed and the plugin recorded the observation (`deriveState` in `runsPlugin.ts`). If the task completed without one, the run is reported `unverified`. A rejected push fails the `apply` step, and the run is reported `failed` with git's error message. |
| R6 One place for the org | `org/catalog/org.yaml` lists the repositories, and `org/bin/acme-actions` lists every repository's actions (`list` queries the catalog for templates tagged `acme-action`), previews them, requests them, and decides and audits runs by id, without entering any repository. Onboarding a repository means adding one target to `org/catalog/org.yaml`. |
| R7 Audited | `acme-actions record <id>` returns `requested` (by, at), `inputs`, `preview`, `approval` or `rejection` (by, at, comment), `push` (commit, at), `verification` (observed commit, at), and `outcome` (state, at, error). It also returns `events`, an append-only timeline that includes refused approvals, and `log`, the scaffolder's step log. Records live in sqlite under `backstage/.data/` and survive restarts. The commit on `main` carries `Requested-by`, `Approved-by` and `Backstage-Run` trailers, with ann as author and bob as committer. |
| R8 Usable by agents | Every command is non-interactive and prints one JSON document. Exit codes: `0` ok; `1` unexpected error or server down; `2` refused (validation 400, not allowed 403, conflict 409, not found 404, bad token 401); `3` a `--wait` ended in a state other than `succeeded`; `4` a `--wait` timed out. Identity comes from `--as NAME` or `ACME_AS`, or a raw token in `BACKSTAGE_TOKEN`. The same operations are plain HTTP (listed below). |

Run states: `awaiting-approval`, `approved`, `applying`, `verifying`, `verified`, `succeeded`,
`failed`, `rejected`, `cancelled`, `unverified`. Only `succeeded` means `DEPRECATED.md` was observed
on `main`.

## Start

Backstage must be running. Start it in its own terminal; the API is ready when the second command
prints `200`:

```bash
export PATH=/tmp/bs/node24/bin:$PATH
cd /tmp/actions-ab/r2/backstage-1/backstage && yarn workspace backend start
```

```bash
until curl -s -o /dev/null -w '%{http_code}\n' -H 'Authorization: Bearer ann-token-0123456789' \
  http://localhost:7430/api/catalog/entities?limit=1 | grep -q 200; do sleep 2; done; echo 200
```

It listens on http://localhost:7430. After a fresh start, give the catalog about 15 s to load
before `list`, which should return three items. Stop it with Ctrl-C in its terminal.

## Use

Run these from `/tmp/actions-ab/r2/backstage-1/org`:

```bash
cd /tmp/actions-ab/r2/backstage-1/org
A=bin/acme-actions

# List the action for all three repositories (anyone)
$A --as ann list
$A --as ann show beta                       # one repository's full definition

# Preview: validate the inputs and show the diff; changes nothing
$A --as ann preview alpha --successor acme/dependabot-proxy-v2 \
  --reason "Replaced by the v2 proxy, which supports OIDC registries."

# Request a run as ann; prints the run id and the stored preview, state awaiting-approval
$A --as ann request alpha --successor acme/dependabot-proxy-v2 \
  --reason "Replaced by the v2 proxy, which supports OIDC registries."
RUN=<id printed above>     # or: RUN=$($A --as ann request ... | python3 -c 'import json,sys; print(json.load(sys.stdin)["id"])')

# Approve as bob (add --wait to block until the run ends; exit 0 only if it succeeded)
$A --as bob approve "$RUN" --wait
$A --as bob reject "$RUN" --comment "why"   # the alternative to approving

# Status, full record, history
$A --as ann status "$RUN"                   # add --wait to block until it ends
$A --as ann record "$RUN"
$A --as ann runs --repo alpha               # optional --state succeeded
$A --as ann cancel "$RUN"                   # cancels a run that is still awaiting approval
```

The equivalent HTTP calls, each with `-H 'Authorization: Bearer <name>-token-0123456789'`:

| Step | Request |
|---|---|
| list | `GET /api/catalog/entities/by-query?filter=kind=template,metadata.tags=acme-action` |
| preview | `POST /api/scaffolder/v2/dry-run` with `{template, values, secrets: {}, directoryContents: []}`, where `template` is the catalog entity's `apiVersion`, `kind`, `metadata` and `spec` |
| request | `POST /api/scaffolder/v2/tasks` with `{"templateRef":"template:alpha/mark-deprecated","values":{"successor":…,"reason":…}}` |
| approve / reject | `POST /api/acme-runs/runs/<id>/approve` (or `/reject`) with `{"comment": "…"}` (optional) |
| status / record | `GET /api/acme-runs/runs/<id>` |
| history | `GET /api/acme-runs/runs?repository=alpha` |

## Testing the rules

All three repositories already have `DEPRECATED.md`. A request with the same inputs is a no-op: the
preview shows `changeType: none`, nothing is pushed, and the run succeeds because the content is
observed on `main`. To exercise a push, use a reason that differs from the current file.

```bash
cd /tmp/actions-ab/r2/backstage-1/org; A=bin/acme-actions
runid() { python3 -c 'import json,sys; print(json.load(sys.stdin)["id"])'; }

# Invalid inputs: each exits 2 with a 400 ValidationError; no run is created
$A --as ann preview alpha --successor Acme/Proxy --reason "Replaced by the v2 proxy, which supports OIDC registries."
$A --as ann request alpha --successor acme/proxy --reason "too short"
$A --as ann request alpha --successor acme/alpha --reason "Points at itself, which is not another repo."
$A --as ann request alpha --successor acme/proxy --reason "$(printf 'x%.0s' $(seq 201))"

# Self-approval and carol: both exit 2 with 403; the run stays awaiting-approval
RUN=$($A --as ann request alpha --successor acme/proxy-v3 --reason "Testing the approval rules for this run." | runid)
$A --as ann approve "$RUN"
$A --as carol approve "$RUN"
$A --as ann status "$RUN"
$A --as ann cancel "$RUN"     # or let bob approve it, which pushes the new content

# Rejected push: the run is reported failed, not succeeded (approve --wait exits 3)
printf '#!/bin/sh\necho "push rejected (test)" >&2\nexit 1\n' > ../origins/alpha.git/hooks/pre-receive
chmod +x ../origins/alpha.git/hooks/pre-receive
RUN=$($A --as ann request alpha --successor acme/proxy-v4 --reason "Testing a rejected push to main here." | runid)
$A --as bob approve "$RUN" --wait; echo "exit=$?"
$A --as ann record "$RUN"     # state failed, outcome.error is git's rejection, no verification
git --git-dir ../origins/alpha.git show main:DEPRECATED.md   # unchanged
rm ../origins/alpha.git/hooks/pre-receive
```

## Limits

- Identity is whatever token the caller presents. In this sandbox, `--as NAME` derives
  `NAME-token-0123456789`. A real deployment would use a sign-in provider; the plugin already accepts
  Backstage user principals.
- A run waiting for approval is a live scaffolder task. If the backend stops, the scaffolder marks
  such tasks failed after it restarts, and they must be requested again. Completed records persist
  in `backstage/.data/`. To start from a clean history, stop the backend and delete that directory.
- `yarn workspace backend start` is the development server; it restarts itself when backend source
  changes.
- Maintainers are read from the catalog. A change to `group:<repo>/maintainers` in a repository's
  `catalog-info.yaml` applies once it is on `main` and the catalog has refreshed.
- Alpha's outcome timestamp was written before a fix that orders the outcome after the last event,
  so it reads 13 ms earlier than its verification. The audit rows were left as written.
