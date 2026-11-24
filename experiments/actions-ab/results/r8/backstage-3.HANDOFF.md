# add-notice on Backstage: handoff

`add-notice` is a self-service action in each of `alpha`, `beta` and `gamma`. It is a Backstage
scaffolder template that the repository commits to its own `main`. A run validates its inputs,
records a preview, and waits until a member of the org's release group other than the requester
approves it. It then pushes the README change to the origin's `main` and reports `succeeded` only
after it has read `main` back and found exactly the approved README.

The three requested runs succeeded:

| Repository | Run id | Requested by | Approved by | Commit observed on `main` |
|---|---|---|---|---|
| alpha | `58979567-9e89-48ce-b1e6-5840984185c4` | ann | dave | `48b62e7` |
| beta | `e18be33d-4325-4b9d-bb24-d00b7ad311f2` | ann | erin | `fc4f643` |
| gamma | `f11a4e95-c1f8-46ce-86d2-0558b7cf6d67` | ann | erin | `8f7aafb` |

## How it fits together

| Part | Where | What it does |
|---|---|---|
| Org registry | `org/repositories.yaml` | Lists the repositories and their origins |
| People and the release group | `org/catalog/people.yaml`, `org/catalog/release-group.yaml` | Users, and the only definition of who may approve |
| Each repository's action | `<repo>/.backstage/add-notice.yaml` (plus `catalog-info.yaml`: the component and its `maintainers` group) | The scaffolder template: parameters (validation) and steps |
| Catalog ingestion | `backstage/packages/backend/src/acme/orgEntityProvider.ts`, `catalogModule.ts` | An entity provider that polls the origins every 2 s and reads the org's and each repository's files from `main` (from the origins, not from checkouts). Repository entities are forced into a namespace named after the repository, so alpha's action is `template:alpha/add-notice` |
| Scaffolder actions | `backstage/packages/backend/src/acme/scaffolderModule.ts` | `acme:notice:prepare` (validate, preview, record), `acme:notice:await-approval`, `acme:notice:publish` (push, then observe) |
| Approvals and audit | `backstage/packages/backend/src/acme/noticeRunsPlugin.ts`, `runStore.ts` | The `notice-runs` backend plugin: `/api/notice-runs/*` for preview, approve, reject, status and records, backed by SQLite |
| README logic | `backstage/packages/backend/src/acme/notice.ts` (tests: `notice.test.ts`) | Finds the README in any letter case, writes the notice, makes the commit, pushes, observes |
| Config | `backstage/app-config.yaml` (sections marked `Acme`) | Persistent SQLite in `backstage/.data/`, scaffolder task recovery, `acme.*` settings. `app-config.local.yaml` (port 7210 and the people's tokens) is unchanged |
| CLI | `org/bin/acme` | One JSON document per command on stdout. Exit codes: 0 ok, 1 refused or not succeeded, 2 usage |

The Backstage changes are committed in the `backstage` git repository (branch `master`, commits
`Acme add-notice: …` and `app-config: …`).

## Requirements

| # | How it is met | Files |
|---|---|---|
| R1 Owned by the repository | Each repository commits its own template to its origin's `main`, owned by its `maintainers` group (ann, bob), which is defined in the same repository. Backstage reads it from the origin's `main` only, so a pushed change to the template is live within about 2 s. The template's entity ref is `template:<repo>/add-notice`. The run's target (origin and branch) comes from the catalog entity's provider-set annotations, never from step inputs, so one repository's template cannot write to another repository | `alpha/.backstage/add-notice.yaml`, `beta/.backstage/add-notice.yaml`, `gamma/.backstage/add-notice.yaml`, `*/.backstage/catalog-info.yaml`, `backstage/packages/backend/src/acme/orgEntityProvider.ts` |
| R2 Validated | Two layers. First, the template's JSON Schema: `level` must be `info` or `warning`, and `message` must be 10–160 characters with no line breaks or control characters. The scaffolder refuses a `POST /api/scaffolder/v2/tasks` that breaks it with HTTP 400, so no task is created. Second, the platform floor in `validateInputs`, which the preview endpoint and the `prepare` step both enforce, so a repository that loosens its schema still cannot get past it. Both checks run before anything is recorded or pushed | `*/.backstage/add-notice.yaml` (`parameters`), `backstage/packages/backend/src/acme/notice.ts` (`validateInputs`) |
| R3 Previewed | `acme preview` (`POST /api/notice-runs/preview`) runs a scaffolder dry-run of the repository's own template, from the catalog. It returns the README path, the base commit and a unified diff, and changes nothing. Every run also records its preview (base blob, expected SHA-256, diff) when it is requested, and `acme request`, `status` and `record` show it before approval. `publish` refuses to push if the README on `main` changed after the preview (`stale-preview`), so what lands is what was previewed and approved | `backstage/packages/backend/src/acme/noticeRunsPlugin.ts` (`preview`), `scaffolderModule.ts` (`acme:notice:prepare`), `notice.ts` (`computePreview`, `pushNotice`) |
| R4 Approved by the release group | `await-approval` blocks the scaffolder task until the run is approved. Only `POST /api/notice-runs/runs/<id>/approve` can approve it. That endpoint refuses (403) the requester, and anyone who is not in `spec.members` of `group:default/release` as ingested from the org origin's current `main` (it waits for the catalog to catch up with the latest org commit first). `publish` checks the run's recorded state again, so a template that drops the approval step still cannot push. The internal endpoints the actions use only accept the scaffolder plugin's own service token, and they cannot approve. Refused attempts are written to the audit log | `backstage/packages/backend/src/acme/noticeRunsPlugin.ts` (`decide`, `releaseGroup`), `scaffolderModule.ts`, `identity.ts` |
| R5 Done means observed | `publish` makes a commit in a scratch bare clone and pushes it without force, so the origin's hooks run. It then reads the origin's `main` and compares the README's SHA-256 with the approved content. Only then does it record `succeeded`. A rejected push records `failed` with reason `push-rejected`. A push that is accepted while `main` does not end up with the content records `failed` with reason `not-observed`. Either way the scaffolder task fails. The reported `status` is `succeeded` only when the run recorded an observation and the scaffolder task completed | `backstage/packages/backend/src/acme/notice.ts` (`pushNotice`, `observe`), `scaffolderModule.ts` (`acme:notice:publish`), `noticeRunsPlugin.ts` (`deriveStatus`) |
| R6 One place for the org | From `org/`, `bin/acme actions` lists add-notice for every registered repository, and `bin/acme request <repo> …` requests a run on any of them. Behind it are the Backstage catalog and scaffolder APIs, which anyone can also call directly (see "Plain HTTP" below). To add a repository, add it to `org/repositories.yaml` | `org/bin/acme`, `org/repositories.yaml` |
| R7 Audited | Each run's record includes who requested it and when, who approved or rejected it and when (with the group membership and org commit in force), its inputs, its preview, and its outcome and when (reason, pushed commit, observed commit). An append-only event list adds `requested`, `previewed`, `awaiting-approval`, `approval-refused` (who tried, and why they were refused), `approved`/`rejected`, `publishing`, `succeeded`/`failed`. `acme record <id> --log` adds the scaffolder's step log. Records survive restarts in `backstage/.data/notice-runs.sqlite`. Each commit on `main` also carries `Requested-by` and `Approved-by` trailers, with the run id | `backstage/packages/backend/src/acme/runStore.ts`, `noticeRunsPlugin.ts` |
| R8 Usable by agents | Every step is a non-interactive command that prints JSON and exits non-zero on refusal or failure: `actions`, `approvers`, `preview`, `request`, `approve`, `reject`, `cancel`, `status [--wait]`, `record`, `runs`. Identity comes from `--as`, `--token` or `$ACME_TOKEN`. The same operations are plain JSON over HTTP | `org/bin/acme` |
| R9 The group is defined once | `org/catalog/release-group.yaml` (`spec.members`) on the org origin's `main` is the only definition. Repositories may not define `User` entities, and only the org group's own `spec.members` counts, so a `memberOf` elsewhere adds nobody. Every approval reads the current membership, for all three repositories, after waiting until the catalog has ingested the org's latest commit | `org/catalog/release-group.yaml`, `backstage/packages/backend/src/acme/noticeRunsPlugin.ts` (`releaseGroup`), `orgEntityProvider.ts` |

## Commands

### Start Backstage (and later stop it)

```bash
export PATH=/tmp/bs/node24/bin:$PATH
cd /tmp/actions-ab/r8/backstage-3/backstage
tmux new-session -d -s r8-backstage-3-backend 'export PATH=/tmp/bs/node24/bin:$PATH; yarn workspace backend start 2>&1 | tee .backend.log'
# ready when this prints {"person": "ann"} (about 30 s)
until curl -sf -H 'Authorization: Bearer ann-token-0123456789' http://localhost:7210/api/notice-runs/whoami; do sleep 2; done

# stop
tmux kill-session -t r8-backstage-3-backend
```

Nothing else needs to run: the catalog provider, the scaffolder and the `notice-runs` plugin are all
in this one backend. State lives in `backstage/.data/`. Deleting it resets the run records and
tasks; the catalog is rebuilt from the origins.

### Use it (from the org)

```bash
export PATH=/tmp/bs/node24/bin:$PATH
cd /tmp/actions-ab/r8/backstage-3/org
```

`--as NAME` reads the person's token from `org/tokens.local.json`, which is gitignored and maps
ann, bob, carol, dave and erin to the tokens in `app-config.local.yaml`. `--token TOKEN` or
`ACME_TOKEN=…` also work.

```bash
# list the action in every repository (repository, ref, owner, origin/path/commit it was read from, parameters)
bin/acme --as ann actions

# preview: what a run would change; changes nothing
bin/acme --as ann preview alpha --level warning --message "This proxy is frozen while the v2 rewrite lands; please file new issues against v2."

# request, as ann: prints the run (id, status "awaiting-approval", preview)
bin/acme --as ann request alpha --level warning --message "This proxy is frozen while the v2 rewrite lands; please file new issues against v2."
RUN=$(bin/acme --as ann request beta --level info --message "Releases now ship on the shared release train every second Tuesday." | jq -r .id)

# approve, as dave or erin (optionally --comment "…"); or reject
bin/acme --as dave approve "$RUN"
bin/acme --as erin reject "$RUN" --comment "wrong wording"

# status (add --wait to block until it is final; exit 0 only for "succeeded")
bin/acme --as ann status "$RUN"
bin/acme --as ann status "$RUN" --wait --timeout 120

# the full record with audit events; --log adds the scaffolder's step log
bin/acme --as ann record "$RUN"
bin/acme --as ann record "$RUN" --log

# other commands
bin/acme --as ann runs --repo alpha        # list runs (filter with --status succeeded|failed|…)
bin/acme --as ann approvers                # the release group as approvals enforce it right now
bin/acme --as ann cancel "$RUN"            # cancel a run that has not been published
```

A run's `status` moves through `pending` (task created, not yet recorded), `awaiting-approval`,
`approved`, `publishing`, and then ends as `succeeded`, `failed` (see `outcome.reason`:
`push-rejected`, `not-observed`, `stale-preview`, `push-failed`), `rejected`, `cancelled` or
`expired` (no decision within 7 days, `acme.approvals.timeout`).

### Change who is in the release group

Edit the org repository and push it. The change applies to approvals in all three repositories as
soon as it is on the org origin's `main`: an approval waits, for up to 30 s, until the catalog has
ingested the latest org commit.

```bash
cd /tmp/actions-ab/r8/backstage-3/org
# e.g. add carol: append "    - carol" under spec.members
printf '    - carol\n' >> catalog/release-group.yaml
git commit -am "release group: add carol" && git push origin main
bin/acme --as ann approvers        # members now include carol; sourceCommit is the new org commit
# to remove her again, delete the line (or: git revert --no-edit HEAD), commit and push
```

### Plain HTTP (what the CLI calls)

All requests take `-H "Authorization: Bearer <name>-token-0123456789"`, and bodies are JSON.

| Step | Request |
|---|---|
| List | `GET /api/notice-runs/actions`, or the catalog directly: `GET /api/catalog/entities/by-query?filter=kind=template,spec.type=add-notice` |
| Preview | `POST /api/notice-runs/preview` `{"repository":"alpha","values":{"level":"info","message":"…"}}` |
| Request | `POST /api/scaffolder/v2/tasks` `{"templateRef":"template:alpha/add-notice","values":{"level":"info","message":"…"}}` → `{"id":…}` |
| Approve / reject | `POST /api/notice-runs/runs/<id>/approve` or `/reject` `{"comment":"…"}` |
| Status and record | `GET /api/notice-runs/runs/<id>` (record with `status` and `events`); `GET /api/notice-runs/runs?repository=alpha` |
| Scaffolder view | `GET /api/scaffolder/v2/tasks/<id>`, `GET /api/scaffolder/v2/tasks/<id>/events` |
| Approvers | `GET /api/notice-runs/approvers` |

## Testing it

All of these except the stale preview were run against this system before the final runs (the
stale preview is covered by the unit tests). An approved run lands its notice on `main`, so when
you only want to test the approval rules, keep a rejecting hook in place (see the rejected-push
test) or reject the run afterwards. Every refusal below exits 1 and prints `{"error": {"status": …, "name": …, "message": …}}`.

- **Invalid inputs.** `bin/acme --as ann preview beta --level error --message short` and
  `bin/acme --as ann request beta --level info --message "$(printf 'x%.0s' {1..161})"` return 400
  `InputError`. To bypass the CLI, send the scaffolder a two-line message; it answers 400 and
  creates no task:
  `curl -s -XPOST -H 'Authorization: Bearer ann-token-0123456789' -H 'Content-Type: application/json' localhost:7210/api/scaffolder/v2/tasks -d '{"templateRef":"template:alpha/add-notice","values":{"level":"info","message":"two\nlines of text"}}'`
- **Self-approval.** After ann requests a run, `bin/acme --as ann approve <id>` returns 403
  ("ann requested run … and cannot approve it"). The same check stops dave or erin approving a
  run they requested themselves.
- **Approval by someone outside the group.** `--as bob approve` or `--as carol approve` returns
  403 ("… is not a member of group:default/release (members: dave, erin)"). The run keeps
  waiting, and the attempt appears as an `approval-refused` event in `record`.
- **Group change.** Add carol as shown above, and her approval is accepted. Remove her, and it is
  refused again. Pending runs are checked against the group as it is when someone approves.
- **Rejected push.** Before approving, make the origin reject pushes:
  `printf '#!/bin/sh\necho "main is frozen" >&2\nexit 1\n' > /tmp/actions-ab/r8/backstage-3/origins/alpha.git/hooks/pre-receive && chmod +x /tmp/actions-ab/r8/backstage-3/origins/alpha.git/hooks/pre-receive`.
  Then `approve` and `status --wait`: the status is `failed`, `outcome.reason` is
  `push-rejected` with the hook's message, the exit code is 1, and `main` is unchanged. Remove the
  hook afterwards (`rm …/hooks/pre-receive`). Wait for `status --wait` to finish before removing
  it, because the push happens within about a second of the approval.
  A post-receive hook that resets `main`
  (`while read old new ref; do git update-ref "$ref" "$old"; done`) gives `failed` with reason
  `not-observed`.
- **Stale preview.** Push a change to the README after requesting and before approving. `publish`
  refuses with `stale-preview` rather than push something the approver didn't see.
- **Unit tests.** `cd backstage && CI=1 yarn workspace backend test src/acme` runs 19 tests:
  README transform, CRLF, BOM, letter case, validation, push, observe, a rejected push, a stale
  preview.

## Things to know

- **Earlier test runs are in the record.** `bin/acme --as ann runs` also lists six test runs from
  before the final three. Three are failed publishes on alpha: a rejected push, a `main` reset
  after the push, and a rejected push after carol approved during the group test. Two are
  rejections, one on alpha and one on gamma. The sixth succeeded on alpha by mistake: carol
  approved it because my test script failed to remove her from the group first. Its notice
  ("Test run: carol is no longer an approver.") landed in `ef0bdff`, and I reverted it by hand in
  `0267c20`, before the final alpha run, which was then applied to the original README. The org
  history shows the add-carol commit and its revert.
- **Restarts.** A run waiting for approval survives a backend restart. About 30 s after start-up,
  the scaffolder's task recovery resumes the task after its last completed step, and approvals made
  in the meantime are kept. Until then, `status` may show the run as `approved` while the task
  catches up.
- **Identity** is the static token. Backstage authenticates each person as a service principal
  whose subject is their name, and `notice-runs` treats that subject (or a user token's entity
  name) as the person. The permission framework still uses the stock allow-all policy, so the
  approval rules are enforced in `notice-runs` itself.
- **Catalog sources.** The catalog has no static locations; everything comes from the origins'
  `main` branches. Backstage's catalog API would still let anyone register an extra location,
  because of the allow-all policy. Such a location cannot take over `template:<repo>/add-notice`,
  which the provider already owns. A run is also only recorded when its target matches
  `org/repositories.yaml` and its template ref is `template:<repo>/add-notice`.
