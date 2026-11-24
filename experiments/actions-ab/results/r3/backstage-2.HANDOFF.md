# Handoff: `mark-deprecated` on Backstage

`mark-deprecated` runs on all three repositories. Each run was requested by ann, approved by bob,
and reported as `succeeded` only after Backstage read `DEPRECATED.md` back from the origin's
`main`:

| Repository | Run id | Commit on origin `main` |
|---|---|---|
| alpha | `a6b4b88d-e9ba-40c5-9725-746af5a1e4e4` | `be5f0782a574dcf9bc73d04b2460810ef87e2ff7` |
| beta | `e17490fb-b001-4f4c-ac8c-7d79949fcd3d` | `bf68d79768383919e66cdf5e0335d4acde32b17f` |
| gamma | `e646003e-0aaa-4cd2-9f0a-f5fa4961df4b` | `4a3fbde75adcb550d08619b7a19c7f6763d49266` |

The run list also has verification runs, from before and after the real ones. Two were rejected
(one requested by ann, one by carol), after attempts at self-approval and carol's approval were
refused and recorded. Two alpha runs failed because a `pre-receive` hook rejected the push. Nothing
was written by any of them.

## How it fits together

Everything goes through the Backstage backend at `http://localhost:7770`. The glue is three
backend features in `backstage/packages/backend/src/acme/`, registered in
`backstage/packages/backend/src/index.ts` and configured in `backstage/app-config.local.yaml`:

| Piece | File | What it does |
|---|---|---|
| Catalog provider `acme-git-origins` | `catalogModule.ts` | Every 15 s (or on `acme sync`), reads `org` origin `main`: `catalog-info.yaml` (people) and `repositories.yaml` (repositories). For each repository it reads `catalog-info.yaml` from its origin's `main`, plus the files its `Location` entities point to. Each repository's entities go into a namespace named after it (`template:alpha/mark-deprecated`), so one repository cannot define another's |
| Scaffolder action `acme:repo:commit-file` | `scaffolderModule.ts` | In dry-run, computes the change against the origin's `main` and returns a preview with a diff. In a real task, asks `acme-runs` to authorize it with the run's one-time token. Then it clones the origin, commits (author = requester, trailers `Acme-Run`, `Requested-by`, `Approved-by`), pushes to `main`, and reads the file back |
| Backend plugin `acme-runs` | `runsPlugin.ts` | The API under `/api/acme-runs`: list actions, validate, preview (scaffolder dry-run of the repository's own template), request, approve, reject, status and record, plus the audit store |
| Shared git helpers | `git.ts` | `git` calls and the repository list |

The org-level CLI is `org/bin/acme` (Python 3, standard library only). It calls `/api/acme-runs`
with the static token of the person named by `--as`. `org/bin/backstage-start` and
`org/bin/backstage-stop` start and stop the backend.

A run moves through `awaiting-approval` → `approved` → `running` → `succeeded` | `failed`. From
`awaiting-approval` it can also become `rejected`, or `expired` if the definition changed.

## Requirements

| # | How it is met | Files |
|---|---|---|
| R1 Owned by the repository | Each repository holds its own definition on its origin's `main`. `.backstage/mark-deprecated.yaml` is the scaffolder Template: inputs, validation, the steps and the exact file content. `catalog-info.yaml` holds the component, the `maintainers` group (ann, bob) that owns the template, and a `Location` pointing at the template. Backstage reads these from the origin's `main` and from nowhere else, so a commit to `main` changes the action. The list output shows the source commit and blob. If the definition changes while a run awaits approval, approving it fails with `definition-changed` and the run expires | `alpha/.backstage/mark-deprecated.yaml`, `alpha/catalog-info.yaml` (likewise for beta and gamma); `backstage/packages/backend/src/acme/catalogModule.ts` |
| R2 Validated | `preview` and `request` check the inputs against the template's own `parameters` JSON Schema (`successor` pattern `^[a-z0-9-]+/[a-z0-9-]+$` and not the repository itself; `reason` 20–200 characters; both required; no other inputs). Then the scaffolder dry-run validates them again. Invalid inputs return HTTP 400 `invalid-input` with one entry per problem, and nothing is recorded or changed. A request that would change nothing returns 409 `no-change` | `RunsService.validate` and `RunsService.preview` in `runsPlugin.ts`; `parameters` in each `.backstage/mark-deprecated.yaml` |
| R3 Previewed | `acme preview` runs the repository's template as a scaffolder dry-run. The action returns `{repository, origin, branch, baseCommit, changes: [{path, operation: add\|modify\|none, before, after, diff}]}` without writing anything. `acme request` stores the same preview in the run, so the approver sees what they approve. At execution the action must present exactly the previewed content, or `acme-runs` refuses (`change-differs`) | `planChange` and dry-run branch in `scaffolderModule.ts`; `RunsService.preview`, `RunsService.authorize` |
| R4 Approved | `approve` refuses the requester (403 `self-approval`) and anyone outside the template owner's group in the catalog (403 `not-a-maintainer`; carol is not a member). Both refusals are recorded as `approval-refused` events. Only an approval mints a random one-time token, passed to the scaffolder task as a secret. The action refuses to run without it, so calling the scaffolder API directly (`POST /api/scaffolder/v2/tasks`) fails with `refused: this action only runs for an approved acme-runs run` before any change. A token works once and only for its template | `RunsService.approve`, `RunsService.authorize`, `/internal/authorize` (scaffolder only) in `runsPlugin.ts`; token check in `scaffolderModule.ts`; group `maintainers` in each `catalog-info.yaml` |
| R5 Done means observed | The action reads `DEPRECATED.md` back from the origin's `main` after pushing and fails if it differs. Separately, `acme-runs` polls the scaffolder task. When the task completes, it reads the file from the origin's `main` itself and compares it with the approved content. Only then does it set `succeeded` (`outcome.observed: true`, `observedAt`, `commit`). A rejected push fails the task, so the run is `failed` with git's error in `outcome.error`. A task that completes without the file on `main` is also `failed` | `RunsService.check` in `runsPlugin.ts`; end of the handler in `scaffolderModule.ts` |
| R6 One place for the org | From `org/`, `bin/acme actions` lists every repository's actions; `bin/acme request <repo> mark-deprecated …` works on any of them. The org repository says which repositories exist (`org/repositories.yaml`) and who the people are (`org/catalog-info.yaml`) | `org/bin/acme`, `org/repositories.yaml`, `org/catalog-info.yaml` |
| R7 Audited | `acme record <run>` returns `requestedBy`, `requestedAt`, `inputs`, `approvedBy`, `approvedAt`, `startedAt`, `finishedAt` and `outcome` (with `at`, `observedAt`, `commit` or `error`), plus the stored preview and the scaffolder task id. It also returns an ordered `events` list (`requested`, `approval-refused`, `approved`, `started`, `execution-authorized`, `succeeded`/`failed`/`rejected`/`expired`), each with `at` and `actor`. Records are stored in SQLite (`backstage/.acme-data/acme-runs.sqlite`) and survive restarts. The commit on `main` repeats the run id, requester and approver | tables `acme_runs` and `acme_run_events` in `runsPlugin.ts`; `backend.database` in `backstage/app-config.local.yaml` |
| R8 Usable by agents | Every step is a non-interactive command that prints JSON. Exit codes: 0 ok; 1 refused or error, with `{error, message, details, httpStatus}`; 2 usage; 3 a waited-for run did not succeed. `approve --wait` and `wait` block until the run ends. The same steps are plain HTTP+JSON under `/api/acme-runs` | `org/bin/acme`; routes in `runsPlugin.ts` |

## Commands

Run from `/tmp/actions-ab/r3/backstage-2/org`. Identity is `--as <person>` or `ACME_AS`. The CLI
reads that person's token from `backstage/app-config.local.yaml`; you can override it with
`ACME_TOKEN`.

```bash
cd /tmp/actions-ab/r3/backstage-2/org

# Start Backstage (tmux session r3-backstage-2-backend, port 7770). It prints JSON once the API
# answers and the catalog has the actions. Log: backstage/.acme-data/backend.log
bin/backstage-start
# Equivalent by hand:
#   cd /tmp/actions-ab/r3/backstage-2/backstage && PATH=/tmp/bs/node24/bin:$PATH yarn workspace backend start

# List the actions of all three repositories (or one: --repository beta)
bin/acme --as ann actions
bin/acme --as ann action alpha mark-deprecated

# Preview: validates and shows the change; records nothing
bin/acme --as ann preview alpha mark-deprecated --successor acme/new-thing --reason "A reason that is at least twenty characters."

# Request as ann: prints the run record (status awaiting-approval); keep its id
RUN=$(bin/acme --as ann request alpha mark-deprecated --successor acme/new-thing \
  --reason "A reason that is at least twenty characters." | jq -r .id)

# Approve as bob; --wait blocks until the run ends; exit 0 only if it succeeded
bin/acme --as bob approve "$RUN" --wait

# Status (short) and the full record
bin/acme --as ann status "$RUN"
bin/acme --as ann record "$RUN"

# Others
bin/acme --as ann runs [--repository alpha] [--status succeeded]
bin/acme --as bob reject "$RUN" --reason "not now"   # a maintainer or the requester
bin/acme --as ann wait "$RUN" --timeout 120
bin/acme --as ann sync    # re-read definitions from the origins now, instead of within 15 s

# Stop Backstage
bin/backstage-stop
```

The same API over HTTP:

```bash
H='-H content-type:application/json'
curl -s localhost:7770/api/acme-runs/actions -H 'authorization: Bearer ann-token-0123456789'
curl -s -X POST localhost:7770/api/acme-runs/preview $H -H 'authorization: Bearer ann-token-0123456789' \
  -d '{"repository":"alpha","action":"mark-deprecated","inputs":{"successor":"acme/x","reason":"twenty characters or more"}}'
curl -s -X POST localhost:7770/api/acme-runs/runs $H -H 'authorization: Bearer ann-token-0123456789' -d '<same body>'
curl -s -X POST "localhost:7770/api/acme-runs/runs/$RUN/approve?wait=60" $H -H 'authorization: Bearer bob-token-0123456789' -d '{}'
curl -s localhost:7770/api/acme-runs/runs/$RUN/status -H 'authorization: Bearer ann-token-0123456789'
curl -s localhost:7770/api/acme-runs/runs/$RUN -H 'authorization: Bearer ann-token-0123456789'
```

## Testing the rules

All three repositories already have `DEPRECATED.md` with the table's content. A request with
those exact inputs is refused as `no-change`, so use a different `reason` (the preview then shows
`operation: modify`).

```bash
cd /tmp/actions-ab/r3/backstage-2/org
OK_REASON="Testing the approval rules with a new reason."

# Invalid inputs: HTTP 400 invalid-input, exit 1, nothing recorded
bin/acme --as ann request alpha mark-deprecated --successor Acme/X --reason short
bin/acme --as ann request alpha mark-deprecated --successor acme/x --reason "$OK_REASON" --extra 1
bin/acme --as ann request alpha mark-deprecated --successor acme/alpha --reason "$OK_REASON"

# Self-approval and carol: 403 self-approval / not-a-maintainer, recorded as approval-refused
RUN=$(bin/acme --as ann request alpha mark-deprecated --successor acme/x --reason "$OK_REASON" | jq -r .id)
bin/acme --as ann approve "$RUN"
bin/acme --as carol approve "$RUN"
bin/acme --as ann record "$RUN" | jq .events

# Rejected push: the run ends failed with git's error, and main keeps the old DEPRECATED.md
printf '#!/bin/sh\necho "main is frozen" >&2\nexit 1\n' > ../origins/alpha.git/hooks/pre-receive
chmod +x ../origins/alpha.git/hooks/pre-receive
bin/acme --as bob approve "$RUN" --wait; echo "exit=$?"          # exit 3, status failed
git --git-dir ../origins/alpha.git show main:DEPRECATED.md       # unchanged
rm ../origins/alpha.git/hooks/pre-receive

# Bypassing approval through the scaffolder API: the task fails, main is unchanged
curl -s -X POST localhost:7770/api/scaffolder/v2/tasks -H 'content-type: application/json' \
  -H 'authorization: Bearer carol-token-0123456789' \
  -d '{"templateRef":"template:alpha/mark-deprecated","values":{"successor":"acme/x","reason":"no approval needed, right?"}}'
```

## Limits

- **Identity is the static token.** Backstage maps each token to a subject (`ann`, `bob`,
  `carol`), and `acme-runs` accepts only subjects that are `User` entities from `org/catalog-info.yaml`.
- **The permission policy is still create-app's allow-all.** Anyone can call the scaffolder API
  directly. The approval gate lives in `acme:repo:commit-file` and `acme-runs`, not in Backstage
  permissions. Any template that uses the action needs an approved run token.
- **Maintainership is data in each repository** (`catalog-info.yaml`, group `maintainers`). Who can
  change it depends on who can push to the origin's `main`. There is no branch protection in this
  sandbox.
- **The glue is not in an org repository.** It lives in `backstage/packages/backend/src/acme/` and
  `backstage/app-config.local.yaml`, next to the Backstage app. Only the org files and the
  repositories' definitions are committed to origins.
