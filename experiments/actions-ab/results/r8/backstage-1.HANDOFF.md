# add-notice on Backstage: handoff

`add-notice` is a self-service action that each repository defines, and its maintainers own, as a
Backstage scaffolder Template on its origin's `main`. Requests come from this org directory with
`bin/acme` or over HTTP. Each run is previewed, approved by a member of the release group other
than the requester, executed by the Backstage scaffolder, and reported as succeeded only after
the README on `main` has been observed with the right content.

Ann requested all three runs below. Each succeeded, and each origin's `main` now starts with its notice:

| Repository | Run id | Approved by | Commit on `main` |
|---|---|---|---|
| alpha | `48f897b0-c8a8-451a-bedf-c118434a864e` | dave | `65a994d` (`README.md`) |
| beta | `d74287ad-25fc-47aa-a04b-4a5fe0f08067` | erin | `4cb48b8` (`readme.md`) |
| gamma | `a0973ded-0500-4749-84d0-98e52248822d` | erin | `a766974` (`README.md`) |

## How it fits together

| Piece | Where | Role |
|---|---|---|
| Org definition | this repo, read from `origins/org.git` `main`: `acme.yaml`, `people.yaml`, `groups/release.yaml`, `groups/maintainers.yaml` | repositories, people, groups, which group approves `add-notice` |
| Action definition | each repo, `.backstage/actions/add-notice.yaml` (+ `catalog-info.yaml`), read from its origin's `main` | the Template `template:<repo>/add-notice`: parameter schema, steps, owner |
| Catalog provider `acme-org` | `backstage/packages/backend/src/acme/catalogModule.ts` | every 3 s, ingests the org's Users and Groups and each repo's entities from the origins' `main` (never from checkouts); forces each repo's entities into the namespace named after it |
| Scaffolder actions | `backstage/packages/backend/src/acme/scaffolderModule.ts` | `acme:readme:notice` (dry run: diff; real run: claim the approved run, clone, edit, commit, push) and `acme:readme:observe` (wait for the expected README blob on `main`) |
| Plugin `notice-runs` | `backstage/packages/backend/src/acme/runsPlugin.ts` | HTTP API at `/api/notice-runs`: list, preview, request, approve and reject; a run state machine; independent observation of `main`; the audit record (SQLite table `notice_runs`) |
| Contract and git helpers | `backstage/packages/backend/src/acme/notice.ts`, `git.ts`, `org.ts` | input rules, the README transform, README lookup in any letter case, reading the org |
| Wiring and config | `backstage/packages/backend/src/index.ts`; `backstage/app-config.local.yaml` (gitignored, so not committed: adds `acme.orgOrigin`, on-disk SQLite in `backstage/data/`, `catalog.locations: []` and the catalog rules) | |
| Org CLI | `org/bin/acme` (`org/bin/acme.mjs`) | every step from a shell, JSON output |

The run lifecycle is `awaiting-approval` → `approved` → `starting` → `running` → `succeeded` or
`failed`. A run in `awaiting-approval` can also become `rejected`.

## R1 to R9

| # | How it is met | Files |
|---|---|---|
| R1 Owned by the repository | Each repo's definition is `.backstage/actions/add-notice.yaml`, committed to its origin's `main`, with `spec.owner: group:default/<repo>-maintainers`. The provider reads it only from the origin's `main`: a commit that is in a checkout but not on `main` (for example, one whose push was rejected) is never used. Its entities go into the namespace named after the repo, so one repo cannot define another's action. The team changes the action by pushing to `main`; the catalog picks it up within seconds. | `alpha/.backstage/actions/add-notice.yaml`, `beta/…`, `gamma/…`; `backstage/packages/backend/src/acme/catalogModule.ts` (`readRepositoryEntities`) |
| R2 Validated | Inputs are checked twice before a run record exists or anything changes. First the fixed contract: `level` is `info` or `warning`; `message` is a string of 10 to 160 characters (code points), with no line breaks or control characters; no other keys. Then the repo's own parameter schema, through a scaffolder dry run of the definition on `main`, so a repo can tighten the rules. The action validates again when it runs. A refusal returns HTTP 400 `InputError`, and the CLI exits 1. | `notice.ts` (`validateNoticeInputs`); `runsPlugin.ts` (`preview`); the `parameters` of each Template |
| R3 Previewed | `acme preview` (`POST /api/notice-runs/actions/<repo>/add-notice/preview`) returns the unified diff of the README against `main`, the README path, the base commit, the expected blob, and the output and log of the definition's scaffolder dry run. It changes nothing. Every request stores the same preview in its record (`.preview`), so approvers see exactly what they approve. | `runsPlugin.ts` (`preview`); `scaffolderModule.ts` (dry-run branch) |
| R4 Approved by the release group | `approve` authenticates the caller's token and reads the approver group (`acme.yaml` `approvals.add-notice: release`) and its members from the org origin's `main` at that moment. The requester is refused, and so is anyone outside the group (HTTP 403); every refused attempt is written to the record. The scaffolder task is created only after approval, and only by `notice-runs`. The push action refuses any task that `notice-runs` did not start for an approved run with the same task id, repository and inputs, so creating a task directly through `/api/scaffolder/v2/tasks` cannot change `main`. The definition is pinned at request time: if `.backstage/actions/add-notice.yaml` changes on `main` before the run starts, the run fails and must be requested again. | `runsPlugin.ts` (`/runs/:id/approve`, `start`, `/internal/runs/:id/claim`); `scaffolderModule.ts` (`approvedTarget`) |
| R5 Done means observed | A push error makes the task, and so the run, `failed`. The observe step must see the expected README blob on `main`. After the task completes, `notice-runs` reads `main` itself again and reports `succeeded` only if the README there equals the transform applied to the README at the push's base commit. Otherwise the run is `failed`, and the record keeps the observed and expected blobs. | `scaffolderModule.ts` (`acme:readme:notice` push, `acme:readme:observe`); `runsPlugin.ts` (`observe`, `follow`) |
| R6 One place for the org | From `org/`, `bin/acme actions` lists `add-notice` for every repository in `acme.yaml`, with its owner, schema, source blob and approvers. `bin/acme request <repo> …` requests a run on any of them. Adding a repository is one entry in `acme.yaml`. | `org/bin/acme`, `org/bin/acme.mjs`, `org/acme.yaml`; `runsPlugin.ts` (`/actions`) |
| R7 Audited | `acme record <id>` (`GET /api/notice-runs/runs/<id>/record`) returns: who requested the run and when; the inputs; the definition (path, commit, blob); the preview; who approved and when, with the group's members and the org commit used; refused approval attempts; the scaffolder task id; the push (base commit, new commit); the observation; the outcome and when; a timestamped `events` list; and the scaffolder task log. The commit on `main` also has the requester as author and `Run:`, `Requested-by:`, `Approved-by:` and `Backstage-task:` trailers. Records survive restarts (`backstage/data/`). | `runsPlugin.ts` (`Run`, `RunStore`, `/runs/:id/record`) |
| R8 Usable by agents | Every step is one non-interactive command with JSON on stdout. Errors go to stderr as JSON, for example `{"error":{"name":"NotAllowedError","message":…,"status":403}}`. Exit codes: 0 ok; 1 refused or error; 2 usage; 3 run failed or rejected (`status --wait`); 4 timeout. The same steps are plain HTTP with `Authorization: Bearer <token>`. | `org/bin/acme.mjs` |
| R9 The group is defined once | The release group is only `org/groups/release.yaml` on `origins/org.git` `main`, and `org/acme.yaml` names it as the approver group of `add-notice`. Approval checks read it live, so a pushed change applies to all three repositories at the next approval, with no restart. The catalog shows it as `group:default/release`. | `org/groups/release.yaml`, `org/acme.yaml`; `org.ts` (`groupMembers`) |

## Commands

These commands assume you are in the org directory. `bin/acme` puts `/tmp/bs/node24/bin` on
`PATH` itself. Who you are is `--as <person>`, which uses the token `<person>-token-0123456789`;
you can also set `ACME_AS` or `ACME_TOKEN`.

### Start Backstage (the only thing that must be running)

```bash
cd /tmp/actions-ab/r8/backstage-1/backstage
mkdir -p data
tmux new-session -d -s r8-backstage-1-backend \
  'export PATH=/tmp/bs/node24/bin:$PATH; yarn workspace backend start 2>&1 | tee data/backend.log'
# ready when this prints JSON (about 10 s):
until curl -sf -H 'Authorization: Bearer ann-token-0123456789' http://localhost:7130/api/notice-runs/approvers; do sleep 1; done
```

To run it in the foreground instead: `cd /tmp/actions-ab/r8/backstage-1/backstage && PATH=/tmp/bs/node24/bin:$PATH yarn workspace backend start`.
To stop it: `tmux kill-session -t r8-backstage-1-backend`. Runs persist in `backstage/data/`;
delete that directory for a clean slate.

### Use it

```bash
cd /tmp/actions-ab/r8/backstage-1/org

# list the actions on all repositories
bin/acme actions --as ann

# preview (changes nothing)
bin/acme preview alpha --level info --message "A notice of at least ten characters." --as ann

# request a run as ann; prints the run, including .id and .preview.diff
RUN=$(bin/acme request alpha --level info --message "A notice of at least ten characters." --as ann | jq -r .id)

# approve as dave (or erin); reject instead with: bin/acme reject "$RUN" --reason "…" --as erin
bin/acme approve "$RUN" --as dave

# status (add --wait to block until it ends; exit 0 succeeded, 3 failed or rejected)
bin/acme status "$RUN" --as ann
bin/acme status "$RUN" --wait --as ann

# the full record
bin/acme record "$RUN" --as ann

# list runs, optionally filtered
bin/acme runs --repo alpha --status succeeded --as ann

# who may approve right now
bin/acme approvers --as ann
```

The same steps over HTTP (`$T` is a person's token):

```bash
B=http://localhost:7130/api/notice-runs; T=ann-token-0123456789
curl -s -H "Authorization: Bearer $T" $B/actions
curl -s -H "Authorization: Bearer $T" -H 'Content-Type: application/json' -X POST $B/actions/alpha/add-notice/preview -d '{"level":"info","message":"A notice of at least ten characters."}'
curl -s -H "Authorization: Bearer $T" -H 'Content-Type: application/json' -X POST $B/runs -d '{"repository":"alpha","action":"add-notice","inputs":{"level":"info","message":"A notice of at least ten characters."}}'
curl -s -H "Authorization: Bearer dave-token-0123456789" -H 'Content-Type: application/json' -X POST $B/runs/<id>/approve -d '{}'
curl -s -H "Authorization: Bearer $T" $B/runs/<id>
curl -s -H "Authorization: Bearer $T" $B/runs/<id>/record
```

### Change who is in the release group

Edit the one `members:` line, then commit and push to the org origin. The change takes effect at
the next approval on every repository.

```bash
cd /tmp/actions-ab/r8/backstage-1/org
sed -i 's/^  members: .*/  members: [dave, carol]/' groups/release.yaml
git commit -am "Release group: dave, carol" && git push origin main
bin/acme approvers --as ann          # shows the new members
# restore:
sed -i 's/^  members: .*/  members: [dave, erin]/' groups/release.yaml
git commit -am "Release group: dave, erin" && git push origin main
```

A change in the checkout that is not pushed has no effect, by design.

## What to expect in the tests

These were run against a temporary sandbox repository, which has since been removed (see
`git log` in this repo). The exception is the identical-notice no-op in the last row, which
follows from the code but was not run.

| Test | Result |
|---|---|
| `--level error`, a message under 10 or over 160 characters, a message with a newline, a missing message, an extra input | HTTP 400 `InputError`, CLI exit 1; no run is created |
| A repo tightens its own schema (for example `maxLength: 40`) and pushes it | longer messages are refused with "Refused by template:<repo>/add-notice, the repository's own definition on main: message does not meet maximum length of 40" |
| The requester approves (even if they are in the release group) | 403 "`<name>` requested run … and cannot approve it"; recorded as `approval-refused` |
| ann, bob or carol approves | 403 "`<name>` is not in the release group (members: dave, erin, …)"; recorded |
| Approving a run that is not awaiting approval | 409 `ConflictError` |
| Group change pushed to `origins/org.git` | the next approval uses the new members, with no restart |
| Push rejected, for example by `printf '#!/bin/sh\necho rejected >&2\nexit 1\n' > /tmp/actions-ab/r8/backstage-1/origins/alpha.git/hooks/pre-receive && chmod +x …/pre-receive` | the run ends `failed`, with reason "Push to … main was rejected: remote: rejected …"; `status --wait` exits 3; `main` is unchanged. Remove the hook afterwards. |
| Push accepted but `main` put back by a hook | the observe step times out; the run is `failed` with the observed and expected blobs |
| Definition changed on `main` between request and approval | the run is `failed`: "… changed on main since the run was requested and previewed; request a new run" |
| A task created directly with `POST /api/scaffolder/v2/tasks` | the task fails with `NotAllowedError`; `main` is unchanged |
| A notice is added to a README that already starts with a notice and a blank line | the old notice is replaced. CRLF READMEs keep CRLF. The README is found in any letter case (beta's is `readme.md`). A README that already has the identical notice is a no-op that still ends `succeeded` (the push result is `unchanged`). |

Known limits:

- Identity is the static token, as the sandbox defines it.
- Two approved runs on one repository at the same moment can race, and the loser fails with a
  rejected (non-fast-forward) push rather than retrying.
- The org origin's `main` must parse. If it does not, every call fails closed with HTTP 500.
