# Handoff: `mark-deprecated` on Backstage

`mark-deprecated` is a self-service action that each repository (`alpha`, `beta`, `gamma`) defines
in its own repository as a Backstage scaffolder Template. Runs go through `acme-actions`, a small
Backstage backend plugin. It validates the inputs against the repository's own template, previews
the change as a scaffolder dry-run and waits for a maintainer other than the requester to approve.
It then runs the template through the scaffolder and reports success only after it has read
`DEPRECATED.md` with the approved content back from the origin's `main`. Every step is a JSON HTTP
call, wrapped by one CLI in this org repository: `bin/acme-actions`.

The three requested runs succeeded. ann requested each one, bob approved it, and the content was
observed on `main`:

| Repository | Run id | Commit on origin `main` |
|---|---|---|
| alpha | `run-87e3f902-f7b1-4df9-afae-29443de04f80` | `a8d5f71` |
| beta | `run-3476cd7b-b0e6-4bfc-ac3f-9c3e6d71108a` | `47d2439` |
| gamma | `run-adb12629-474c-4dfb-8304-8dbadcd54846` | `5edd3ac` |

The run history (`runs`) also holds the test runs made while checking the refusals below. They
are on purpose `failed` (rejected pushes, and a push that was accepted but then reverted on
`main`) or `rejected` (withdrawn by their requester). None of them changed `main`.

Paths below are relative to `/tmp/actions-ab/r6/backstage-3`.

## How it fits together

| Part | Where | What it does |
|---|---|---|
| Repository-owned definition | `<repo>/catalog-info.yaml`, `<repo>/.backstage/mark-deprecated.yaml` (on each `origins/<repo>.git` `main`) | A Component and a Location; the Location points at the Template that defines the inputs, the owner (`group:default/<repo>-maintainers`) and the step that writes `DEPRECATED.md` |
| Org data | `org/catalog-info.yaml` (on `origins/org.git` `main`) | Users ann, bob, carol; groups `<repo>-maintainers` (ann, bob) and `developers` (carol); the list of repositories (a `Location` with `spec.type: acme-git`) |
| Catalog provider | `backstage/packages/backend/src/acme/catalogModule.ts` | Every 5 s, reads those files from each origin's `main` (not from checkouts). Each repository's entities go into a namespace named after it, so its template is `template:<repo>/mark-deprecated`. Repositories may not declare Users or Groups |
| Scaffolder action | `backstage/packages/backend/src/acme/scaffolderModule.ts` (`acme:git:commit-file`) | Dry-run: clones the origin and reports the diff. Real run: claims the approved run from `acme-actions`, commits as the requester, pushes to `main`, then reads the file back from the origin |
| Run service | `backstage/packages/backend/src/acme/actionsPlugin.ts` (plugin `acme-actions`, `/api/acme-actions`) | Lists actions; handles validation, preview, request, approve and reject; dispatches the scaffolder task; checks `main` independently; keeps the run record and its events |
| Shared git helpers | `backstage/packages/backend/src/acme/git.ts` | |
| Wiring and config | `backstage/packages/backend/src/index.ts`, `backstage/packages/backend/package.json`, `backstage/app-config.local.yaml` (`acme.git.*`, SQLite files in `backstage/.data`, catalog rules) | |
| CLI | `org/bin/acme-actions` (Python 3 standard library) | One command per step, JSON on stdout |

## Requirements

| # | How it is met |
|---|---|
| R1 Owned by the repository | Each repository commits its definition to its origin's `main`: `alpha/.backstage/mark-deprecated.yaml` (and the same path in `beta` and `gamma`), referenced from `<repo>/catalog-info.yaml`. The template's `spec.owner` is `group:default/<repo>-maintainers`. Backstage reads it from `origins/<repo>.git` `main` (`catalogModule.ts`), so the team changes the action by pushing to `main`: inputs, schema, file content, commit message and owner. The change shows up in `list` within about 10 s, and `definedIn.commit` shows which commit is live. Edits in a checkout that are not pushed have no effect. Each repository's entities are placed in its own namespace, so it cannot define or impersonate another repository's action. The platform code only allows `acme:git:commit-file` and `debug:log` steps, and only changes to the template's own repository (`actionsPlugin.ts`, `preview`) |
| R2 Validated | `POST /preview` and `POST /runs` validate the inputs against the template's own `spec.parameters` JSON Schema (`successor` pattern `^[a-z0-9-]+/[a-z0-9-]+$` and not the repository itself; `reason` 20 to 200 characters; both required; no unknown inputs). Invalid inputs return HTTP 400, CLI exit code 1, with per-input `details`. No run is created and nothing is cloned or pushed (`actionsPlugin.ts`, `validateInputs`). The scaffolder validates the same schema again when the task is created, and the action checks its own inputs with zod |
| R3 Previewed | `preview` and `request` run the repository's template as a scaffolder dry-run (`POST /api/scaffolder/v2/dry-run`). In dry-run, `acme:git:commit-file` clones the origin and returns the path, `create`/`update`/`none`, the base commit, the exact content and a unified diff (`scaffolderModule.ts`). `request` stores this preview in the run, and the run executes only if the step would write exactly the previewed content (`actionsPlugin.ts`, `claim`). A request whose preview changes nothing is refused with 409 |
| R4 Approved | A run starts in `awaiting_approval`. `approve` is refused with 403 if the caller requested the run, or if the caller is not a member of the template owner's group (carol is not in any `<repo>-maintainers` group). Each refusal is recorded as an `approve_refused` event (`actionsPlugin.ts`, `decide`). The scaffolder task starts only after approval. Starting the task some other way does not help: `acme:git:commit-file` refuses to push unless `acme-actions` hands over an approved run for that exact task, template and content (`claim`), and a run can be claimed only once |
| R5 Done means observed | The action reads `main` back from the origin after pushing and fails the step on a mismatch. A rejected push fails the step (`push_rejected` event). After the scaffolder task completes, `acme-actions` independently reads `DEPRECATED.md` from `origins/<repo>.git` `main` and sets `succeeded` only if it matches the approved content; otherwise the run is `failed` (`actionsPlugin.ts`, `reconcile`; `git.ts`, `observeFile`). Both failure cases were tested (see below) |
| R6 One place for the org | From `org/`, `bin/acme-actions list` lists `mark-deprecated` for every repository named in `org/catalog-info.yaml`, with owner, maintainers, input schema and source commit. `request <repo> mark-deprecated ...` works for any of them. Nothing needs to be run inside a repository |
| R7 Audited | The `acme_runs` and `acme_run_events` tables (`backstage/.data/acme-actions.sqlite`, which survives restarts) store, for each run: requester and time, approver and time, inputs, preview, scaffolder task id, outcome (observed head and commit, or the error) and finish time. They also store the timed event list: `requested`, `approve_refused`, `approved`/`rejected`, `dispatched`, `started`, `pushed`/`push_rejected`, `observed`, `succeeded`/`failed`. `record <run>` returns all of it. The commit on `main` is authored by the requester and has `Requested-by`, `Approved-by`, `Acme-Run` and `Scaffolder-Task` trailers. Requests and decisions also go to Backstage's auditor log |
| R8 Usable by agents | Every step is a non-interactive CLI command that prints JSON and uses exit codes (0 ok, 1 refused or failed, 2 usage, 3 `wait` ended failed/rejected, 4 `wait` timed out). The same calls are plain JSON HTTP under `http://localhost:7710/api/acme-actions` with `Authorization: Bearer <token>` |

## Start

Backstage must be running. Nothing else is needed.

```bash
tmux new-session -d -s r6-backstage-3-backend -c /tmp/actions-ab/r6/backstage-3/backstage \
  'PATH=/tmp/bs/node24/bin:$PATH yarn workspace backend start'
until curl -sf -H 'Authorization: Bearer ann-token-0123456789' \
  http://localhost:7710/api/acme-actions/whoami; do sleep 2; done; echo
```

Stop it with `tmux kill-session -t r6-backstage-3-backend`. Runs persist in `backstage/.data/`.
Delete that directory, with Backstage stopped, to start from an empty history.

## Use

Run these from the org repository. `--as NAME` picks that person's sandbox token from
`org/.acme-tokens.json`, which is not committed. Recreate it if it is missing:

```bash
cd /tmp/actions-ab/r6/backstage-3/org
[ -f .acme-tokens.json ] || printf '{"ann":"ann-token-0123456789","bob":"bob-token-0123456789","carol":"carol-token-0123456789"}\n' > .acme-tokens.json
A="python3 bin/acme-actions"        # or ./bin/acme-actions; --token TOKEN also works
```

| Step | Command |
|---|---|
| Who am I | `$A --as ann whoami` |
| List the actions | `$A --as ann list` (filters: `--repository alpha`, `--action mark-deprecated`) |
| One action's schema, owner and steps | `$A --as ann show alpha mark-deprecated` |
| Preview (changes nothing) | `$A --as ann preview alpha mark-deprecated --input successor=acme/dependabot-proxy-v3 --input "reason=Replaced by the v3 proxy, which supports OIDC registries."` |
| Request as ann | `RUN=$($A --as ann request alpha mark-deprecated --input successor=acme/dependabot-proxy-v3 --input "reason=Replaced by the v3 proxy, which supports OIDC registries." \| jq -r .id)` |
| Approve as bob | `$A --as bob approve $RUN` (optional `--comment "..."`) |
| Reject, or withdraw as the requester | `$A --as bob reject $RUN --comment "..."` |
| Check status | `$A --as ann status $RUN` (`status` is `awaiting_approval`, `running`, `succeeded`, `failed` or `rejected`) |
| Wait until it finishes | `$A --as ann wait $RUN --timeout 120` (exit code 0 only if `succeeded`) |
| Read the record | `$A --as ann record $RUN` |
| List runs | `$A --as ann runs` (filters: `--repository`, `--action`, `--status`) |

Inputs can also be given as JSON: `--inputs-json '{"successor":"acme/x","reason":"..."}'`.

The same calls over HTTP, with token `T` (for example `bob-token-0123456789`):

```bash
B=http://localhost:7710/api/acme-actions; H="Authorization: Bearer $T"
curl -s -H "$H" "$B/actions?action=mark-deprecated"
curl -s -H "$H" -H 'content-type: application/json' -X POST "$B/preview" \
  -d '{"repository":"alpha","action":"mark-deprecated","inputs":{"successor":"acme/x","reason":"Twenty or more characters here."}}'
curl -s -H "$H" -H 'content-type: application/json' -X POST "$B/runs" -d '<same body>'   # request
curl -s -H "$H" -H 'content-type: application/json' -X POST "$B/runs/$RUN/approve" -d '{}'
curl -s -H "$H" "$B/runs/$RUN/status"     # status
curl -s -H "$H" "$B/runs/$RUN"            # full record
```

## Testing the refusals

The three repositories already contain the requested `DEPRECATED.md`. Requesting the same
successor and reason again is refused with 409 (*Nothing to change*), so tests need a new reason
or successor.

```bash
# Invalid inputs: 400, no run created, exit code 1
$A --as ann request alpha mark-deprecated --input successor=Not/Valid_ --input reason=short
$A --as ann request alpha mark-deprecated --input successor=acme/alpha --input "reason=Twenty or more characters here."
$A --as ann request alpha mark-deprecated --input successor=acme/x --input "reason=Twenty or more characters here." --input extra=1

# Self-approval and carol: both 403, both recorded as approve_refused
RUN=$($A --as ann request beta mark-deprecated --input successor=acme/test --input "reason=Testing the approval rules here." | jq -r .id)
$A --as ann approve $RUN
$A --as carol approve $RUN
$A --as ann reject $RUN --comment "test"          # withdraw it again

# Rejected push: the run ends failed and main does not change
printf '#!/bin/sh\necho "main is frozen" >&2\nexit 1\n' > ../origins/alpha.git/hooks/pre-receive
chmod +x ../origins/alpha.git/hooks/pre-receive
RUN=$($A --as ann request alpha mark-deprecated --input successor=acme/test --input "reason=This push must be rejected by origin." | jq -r .id)
$A --as bob approve $RUN; $A --as bob wait $RUN; echo "exit=$?"   # status failed, exit 3
$A --as bob record $RUN | jq '.outcome, [.events[].type]'
rm ../origins/alpha.git/hooks/pre-receive
```

Two more cases were tested. A `post-receive` hook that accepts the push and then resets `main` to
its previous commit gives a `failed` run, because the read-back of `main` does not match. Calling
`POST /api/scaffolder/v2/tasks` for `template:<repo>/mark-deprecated` directly, with no run id or
with the id of a run that is still awaiting approval, gives a failed scaffolder task and no push.

## Notes

- **File content:** `DEPRECATED.md` is `# Deprecated: use <successor>`, a blank line, `<reason>`,
  then a final newline, as rendered from the template's block scalar.
- **Identity:** callers are identified by Backstage's static external-access tokens in
  `backstage/app-config.local.yaml`. Token subject `ann` maps to `user:default/ann`, which must
  exist in the catalog; plugin principals are refused on the person-facing endpoints. The
  `/internal/...` endpoints accept only the scaffolder plugin's service token.
- **Approver rights:** they are taken from the template owner's group when the approval is made.
  A repository can therefore change who may approve its runs by changing `spec.owner`, but group
  membership is defined only in the org repository.
- **Timestamps:** every timestamp is UTC ISO 8601.
