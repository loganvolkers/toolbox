# Hand-off: `mark-deprecated` on Backstage

`mark-deprecated` is a Backstage scaffolder template that each repository defines on its own
origin's `main`. You list, preview, request, approve and audit runs of it from this directory
with `bin/acme-actions`, or with plain HTTP. A run changes `main` only after a maintainer other
than the requester approves it. It is reported `succeeded` only after Backstage has read
`DEPRECATED.md` back from the origin's `main` with exactly the requested content.

All three requested runs (requested by ann, approved by bob) have **succeeded**:

| Repository | Run id | Commit on origin `main` |
|---|---|---|
| alpha | `afdb7263-a782-4e23-ab6b-54b08a117499` | `648df1f` |
| beta | `9a41d5b5-ee75-4638-a5cb-98643faaa118` | `051bad6` |
| gamma | `523f2844-e740-44a7-8cd1-ace80cb7457f` | `d1cb61b` |

Four earlier test runs are also in the record. Each one demonstrates a refusal:

| Run id | Repository | Final status | What it shows |
|---|---|---|---|
| `85f1d729-97eb-43c7-a88c-526bfb9b5d46` | alpha | `rejected` | carol's and ann's approvals refused; bob rejected the run |
| `48a241e3-7659-4d0f-8b64-46904db44c15` | alpha | `failed` | a `pre-receive` hook rejected the push |
| `2a732ef4-9b48-46cc-bf96-405f2d8d449f` | beta | `failed` | a static token sent straight to the scaffolder gives a run with no requester, which is refused |
| `0538ddfa-f2db-41a4-9dee-2fb4de0b6544` | beta | `cancelled` | the run waited through a backend restart, then ann cancelled it |

`DEPRECATED.md` ends with a newline. Its bytes are `# Deprecated: use <successor>\n\n<reason>\n`.

## How it is built

The box is Backstage 1.57 with its scaffolder. Everything Acme-specific is a small amount of
glue in the backend package, plus YAML in each repository and in this org repository.

| Piece | File | What it does |
|---|---|---|
| Definitions per repository | `<repo>/.backstage/mark-deprecated.yaml`, `<repo>/catalog-info.yaml` (on each origin's `main`) | Scaffolder `Template` (inputs, JSON Schema, steps) and the repository's `Component`. `spec.owner` names the group whose members approve runs |
| Org registry and people | `org/repositories.yaml`, `org/catalog/people.yaml` (on `origins/org.git` `main`) | Which repositories exist and where their origins are; users ann, bob, carol; groups `alpha-maintainers`, `beta-maintainers`, `gamma-maintainers` (ann, bob) and `developers` (carol) |
| Catalog provider | `backstage/packages/backend/src/acme/catalogModule.ts` | Every 5 s, reads those files from the origins' `main` (never from checkouts) into the catalog. A repository's entities are forced into a namespace named after it (`template:alpha/mark-deprecated`). They are stamped with `acme.io/repository`, `acme.io/origin` and `acme.io/source-commit`, overwriting any `acme.io/*` the file sets. Repositories cannot define users or groups |
| Person sign-in | `backstage/packages/backend/src/acme/authModule.ts` | Auth provider `acme-token`: `GET /api/auth/acme-token/refresh` with a person's static token returns a Backstage user token for `user:default/<subject>`. With that token, the scaffolder records the person as the run's `createdBy` |
| Scaffolder actions | `backstage/packages/backend/src/acme/scaffolderModule.ts` | `acme:git:plan-file` (read-only diff against origin `main`); `acme:approval:await`; `acme:git:push-file` (clone, commit, push, no force); `acme:git:verify-file` (reads the file back from the origin). The target repository comes from the catalog annotation, never from inputs |
| Self-service API and approval gate | `backstage/packages/backend/src/acme/selfServicePlugin.ts` | Backend plugin mounted at `/api/self-service`: lists actions, previews, requests, approves, rejects, cancels, and serves status and records. Approvals and audit events are stored in its own tables (`approvals`, `approval_events`) |
| Shared git helpers | `backstage/packages/backend/src/acme/git.ts` | Origin reads, the registry loader, and the change digest |
| Wiring and config | `backstage/packages/backend/src/index.ts`, `backstage/app-config.yaml` (`acme.*`, `auth.providers.acme-token`, `scaffolder.taskRecovery`, an on-disk SQLite directory) | |
| Org tools | `org/bin/acme-actions` (CLI, Python 3 standard library), `org/bin/backstage-server` (start, stop, status) | |

Persistent state is in `backstage/data/sqlite/` (SQLite, one file per plugin). The backend log is
`backstage/data/backend.log`.

## Requirements

**R1 — Owned by the repository.** Each repository's action is
`.backstage/mark-deprecated.yaml` (with `catalog-info.yaml`), committed to
`origins/<repo>.git` `main`: alpha `8f2d52c`, beta `02459cf`, gamma `65fe46c`. Changes to these
files reach Backstage within about 5 s of landing on `main`, because the catalog provider reads
the origin, not a checkout (`catalogModule.ts`). Each repository has already changed its own
definition once: commits `5b9f2d0`, `f113896` and `646d3fa` bind the approval to the planned
previous state of the file. `acme-actions actions` shows the commit each definition was loaded
from. The template's `spec.owner` (for example `group:default/alpha-maintainers`) chooses who
approves. A template can act only on the repository that defines it:
`scaffolderModule.ts` `resolveRepository` takes the repository from the provider's annotation
and requires the template's namespace to match.

**R2 — Validated.** Inputs are checked against the template's JSON Schema (`successor` matches
`^[a-z0-9-]+/[a-z0-9-]+$` and is not `acme/<itself>`; `reason` is 20 to 200 characters; both are
required). The scaffolder runs that check before it creates a run or a dry run, so invalid
inputs get HTTP 400, CLI exit 1, and a list of `{field, message}` errors. No run is recorded and
nothing touches the origin. An unknown repository or action gets 404. File:
`<repo>/.backstage/mark-deprecated.yaml` (`spec.parameters`); the error formatting is in
`selfServicePlugin.ts`.

**R3 — Previewed.** `acme-actions preview` runs the repository's template as a scaffolder dry
run (`POST /api/scaffolder/v2/dry-run`, via `POST /api/self-service/preview`). Only the
read-only plan step does any work. The output includes `change.diff` (a unified diff against
the origin's current `main`), `change.content`, `change.changeType` (`create`, `update` or
`none`) and `change.baseCommit`. Every real run also starts with the same plan step. Its diff
is stored with the approval and returned by `status` while the run is `awaiting-approval`, so
the requester and the approver see the exact change before it can take effect. Files: the
`plan` step in the template, and `acme:git:plan-file` in `scaffolderModule.ts`.

**R4 — Approved.** The run's second step, `acme:approval:await`, records an approval request
and waits. `POST /api/self-service/runs/:id/approve` (`selfServicePlugin.ts`) accepts only a
signed-in person who:

- is a member of the template owner's group (catalog `hasMember` relation, from
  `org/catalog/people.yaml`), and
- is not the run's `createdBy`.

Anyone else gets HTTP 403, and the refused attempt is written to the audit trail. carol is not
in any maintainers group. The gate does not rely on the template keeping the approval step:
`acme:git:push-file` pushes only after it has consumed an approval that is `approved`, was
decided by someone other than the requester, has not been used before, and covers exactly the
content it is about to push (a SHA-256 digest of repository, path and content). A run with no
signed-in requester, such as a static token sent straight to the scaffolder, fails before
anyone can approve it.

**R5 — Done means observed.** A run is `succeeded` only if three things hold: the scaffolder
task completed, the push step consumed the approval, and `acme:git:verify-file` read
`DEPRECATED.md` from the origin's `main` with exactly the approved content and recorded that
observation (`deriveStatus` in `selfServicePlugin.ts`). A completed task without that
observation is reported as `unverified`. A rejected push throws, so the run is `failed`, the
verify step is skipped, and the record has a `push-rejected` event with git's error. If
`DEPRECATED.md` changed on `main` between the preview and the push, the push refuses and the
run fails.

**R6 — One place for the org.** From `/tmp/actions-ab/r5/backstage-1/org`:

- `bin/acme-actions actions` lists `mark-deprecated` for alpha, beta and gamma;
- `bin/acme-actions request <repo> …` requests a run on any of them;
- `runs`, `status`, `approve` and `record` work across all three.

The CLI talks to one API, `http://localhost:7430/api/self-service`. Which repositories appear is
decided by `org/repositories.yaml` on the org origin's `main`.

**R7 — Audited.** `bin/acme-actions record <run-id>` (`GET /api/self-service/runs/:id/record`)
returns:

- `requestedBy` and `requestedAt`;
- `inputs`;
- `approval.decidedBy`, `approval.decidedAt` and `approval.approverGroup`;
- `outcome.status`, `outcome.at`, `outcome.observed` (commit and time) and `outcome.error`;
- `definition` (the template and the commit it was loaded from);
- per-step start and finish times;
- the step log;
- a `timeline` of timestamped, attributed events: `requested`, `approval-requested`,
  `approval-refused` (who, and why), `approved`, `rejected`, `approval-consumed`, `pushed` or
  `push-rejected` or `no-change`, `observed-on-main` or `not-observed-on-main`, and
  `run-completed` or `run-failed`.

The record is built from the scaffolder's task and events and the plugin's `approvals` and
`approval_events` tables (`selfServicePlugin.ts`), all in `backstage/data/sqlite/`, so it
survives restarts. Each commit pushed to `main` also carries `Requested-by:`, `Approved-by:`
and `Backstage-run:` trailers, with the requester as the commit author.

**R8 — Usable by agents.** Every step is a non-interactive `bin/acme-actions` subcommand that
prints JSON on stdout. Exit codes: `0` ok; `1` refused or failed (the JSON says why); `2` usage;
`3` Backstage unreachable; `124` `wait` timed out. `wait` blocks until a run finishes (or, with
`--for awaiting-approval`, until it waits for approval). The same operations are plain HTTP
with JSON bodies (see the curl examples below). `bin/backstage-server` also prints JSON.

## Commands

All commands run from the org directory. Identity is `--as <person>`. The CLI sends that
person's static token from `backstage/app-config.local.yaml` (`<person>-token-0123456789`;
override with `ACME_TOKEN_<PERSON>` or `--token`).

```bash
cd /tmp/actions-ab/r5/backstage-1/org

# Start the backend (tmux session r5-backstage-1-backend, http://localhost:7430). It waits until
# the API answers and the actions are loaded, then prints one JSON line.
bin/backstage-server start
# Equivalent by hand:
#   cd /tmp/actions-ab/r5/backstage-1/backstage && PATH=/tmp/bs/node24/bin:$PATH yarn workspace backend start

# List the actions
bin/acme-actions --as ann actions --action mark-deprecated

# Preview (dry run; changes nothing)
bin/acme-actions --as ann preview alpha --successor acme/some-repo --reason "Twenty or more characters of reason."

# Request a run as ann. Prints the run, including runId, status "awaiting-approval" and preview.diff
bin/acme-actions --as ann request alpha --successor acme/some-repo --reason "Twenty or more characters of reason."
RUN=<runId from the output>
# or: RUN=$(bin/acme-actions --as ann request alpha --successor acme/some-repo --reason "Twenty or more characters of reason." | jq -r .runId)

# Approve as bob
bin/acme-actions --as bob approve "$RUN" --comment "LGTM"

# Check the status (queued | planning | awaiting-approval | approved | applying | succeeded |
# failed | rejected | cancelled | expired | unverified)
bin/acme-actions --as ann status "$RUN"
bin/acme-actions --as ann wait "$RUN" --timeout 120      # exit 0 only if succeeded

# Read the audit record
bin/acme-actions --as ann record "$RUN"

# Also available
bin/acme-actions --as ann runs [--repository alpha] [--status succeeded]
bin/acme-actions --as bob reject "$RUN" --comment "why"    # requester or an approver-group member
bin/acme-actions --as ann cancel "$RUN"                    # requester or approver; not once applying
bin/acme-actions --as carol whoami

# Stop the backend
bin/backstage-server stop
```

The same API over HTTP, with any person's static token:

```bash
H='Authorization: Bearer ann-token-0123456789'; B=http://localhost:7430/api/self-service
curl -s "$B/actions?action=mark-deprecated" -H "$H"
curl -s -X POST "$B/preview" -H "$H" -H 'Content-Type: application/json' \
  -d '{"repository":"alpha","action":"mark-deprecated","inputs":{"successor":"acme/some-repo","reason":"Twenty or more characters of reason."}}'
curl -s -X POST "$B/runs" -H "$H" -H 'Content-Type: application/json' \
  -d '{"repository":"alpha","action":"mark-deprecated","inputs":{"successor":"acme/some-repo","reason":"Twenty or more characters of reason."}}'
curl -s -X POST "$B/runs/$RUN/approve" -H 'Authorization: Bearer bob-token-0123456789' -H 'Content-Type: application/json' -d '{}'
curl -s "$B/runs/$RUN" -H "$H"
curl -s "$B/runs/$RUN/record" -H "$H"
```

You can also use the scaffolder API directly (`/api/scaffolder/v2/tasks`), but only with a user
token from `curl -s http://localhost:7430/api/auth/acme-token/refresh -H "$H" | jq -r
.backstageIdentity.token`. A static token sent straight to the scaffolder gives a run with no
requester, and that run fails before approval.

## Testing the refusals

Each repository already has its `DEPRECATED.md`. A run with the same successor and reason is
therefore a `none` change: it pushes nothing and succeeds, because `main` already has the
content. To exercise approval and pushes, use a reason or successor that differs from the
current file.

```bash
cd /tmp/actions-ab/r5/backstage-1/org
# Invalid inputs: each prints HTTP 400 with field errors and exits 1; no run is created
bin/acme-actions --as ann request alpha --successor "Not/Valid" --reason "Twenty or more characters of reason."
bin/acme-actions --as ann request alpha --successor acme/x --reason "too short"
bin/acme-actions --as ann request alpha --successor acme/alpha --reason "Twenty or more characters of reason."
bin/acme-actions --as ann preview alpha --successor acme/x --reason "$(printf 'x%.0s' $(seq 201))"

# Self-approval and carol: both HTTP 403, exit 1, and both are recorded in the timeline
RUN=$(bin/acme-actions --as ann request beta --successor acme/other --reason "A different reason to test approvals." | jq -r .runId)
bin/acme-actions --as ann approve "$RUN"      # 403: the requester cannot approve their own run
bin/acme-actions --as carol approve "$RUN"    # 403: only members of group:default/beta-maintainers may approve
bin/acme-actions --as bob reject "$RUN"       # or approve it, which would replace beta's DEPRECATED.md

# A rejected push: the run fails, and main is unchanged
printf '#!/bin/sh\necho "main is frozen" >&2\nexit 1\n' > /tmp/actions-ab/r5/backstage-1/origins/gamma.git/hooks/pre-receive
chmod +x /tmp/actions-ab/r5/backstage-1/origins/gamma.git/hooks/pre-receive
RUN=$(bin/acme-actions --as ann request gamma --successor acme/other --reason "Testing a rejected push to main." | jq -r .runId)
bin/acme-actions --as bob approve "$RUN"
bin/acme-actions --as ann wait "$RUN"         # status "failed", outcome.error has the hook's message, exit 1
bin/acme-actions --as ann record "$RUN"       # timeline has push-rejected; there is no observed-on-main
rm /tmp/actions-ab/r5/backstage-1/origins/gamma.git/hooks/pre-receive
```

An approval is spent by the first push attempt. After a failed push, request a new run rather
than retrying the old one.

## Limits and notes

- **Identity.** People are identified by bearer tokens; in this sandbox, knowing a token is
  being that person. Approver groups live in `org/catalog/people.yaml` on the org origin's
  `main`. A repository picks its approver group with `spec.owner` but cannot add people to it.
- **Timeouts.** A run waits for approval for at most 24 h (`acme.approvalTimeoutSeconds`), then
  becomes `expired`. Waiting runs hold scaffolder worker slots (10 by default).
- **Restarts.** If the backend stops while a run is waiting, it resumes about 30 s after the
  next start (`scaffolder.taskRecovery`). Its approval stays valid only for the same planned
  change. A run interrupted during its push step is reported `failed`, because its approval is
  already spent. A run interrupted after the push step finished resumes at the verify step and
  succeeds only if it observes the file on `main`.
- **Writes to origins.** No installed scaffolder action other than `acme:git:push-file` writes to
  the origins (the GitHub actions need GitHub, which is not here).
- **Repository checkouts.** `/tmp/actions-ab/r5/backstage-1/{alpha,beta,gamma}` track their
  origins and were fast-forwarded to `main` after the runs. Backstage never reads them.
- **Backend process.** It was stopped at hand-off; start it with `bin/backstage-server start`.
