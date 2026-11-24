# Handoff: `mark-deprecated`, a repository-owned self-service action on Backstage

`mark-deprecated` is defined by each repository (alpha, beta, gamma) on its origin's `main`, is
listed and requested from this org repository, takes effect only after a different maintainer
approves the previewed change, and is reported `succeeded` only once Backstage has read
`DEPRECATED.md` with the approved content back from the origin's `main`.

The three requested runs (ann requested, bob approved) all succeeded:

| Repository | Run id | Status | `main` after the run |
|---|---|---|---|
| alpha | `9083c6cd-5d11-4d18-a043-b315223f2621` | succeeded | `e0bb7b43` |
| beta | `edf87627-5e8a-45af-a9e5-50a07d923b6b` | succeeded | `ebe0a677` |
| gamma | `fff0445c-05f4-4e7b-949e-a909383ce69d` | succeeded | `11131651` |

Two test runs are also on record:

- `78850c31-b7a7-4039-8dc7-5b5a6c585be6`, a rejected-push test on alpha. carol requested it and
  ann approved it, then a temporary `pre-receive` hook on `origins/alpha.git` refused the push. It
  is `failed`, and alpha's `main` was not changed by it.
- A `cancelled` run on beta: a 200-character `reason` boundary check that ann withdrew before
  approval.

The beta record also shows ann's refused self-approval and carol's refused approval and rejection.

## How it fits together

| Part | Path | What it does |
|---|---|---|
| Action definition (per repository) | `<repo>/.backstage/actions/mark-deprecated.yaml` | A Backstage scaffolder `Template` (`spec.type: repo-action`): the input schema, the owner group and the step that writes `DEPRECATED.md` |
| Repository entry (per repository) | `<repo>/catalog-info.yaml` | The `Component` and its `maintainers` `Group` (ann, bob). That group approves the repository's actions |
| Org registry | `org/repositories.yaml` | Which repositories exist and where their origins are |
| People | `org/catalog/people.yaml` | `User` entities for ann, bob and carol |
| Catalog provider | `backstage/packages/backend/src/acme/catalogModule.ts` | Every 5 s, reads the org files and each repository's definition files **from the origins' `main`** (not from checkouts), and publishes them in a namespace named after the repository (`template:alpha/mark-deprecated`) |
| Scaffolder action | `backstage/packages/backend/src/acme/scaffolderModule.ts` | `acme:repo:write-file`. In a dry run it only proposes the file. In a real run it clones the origin, commits as the requester (with `Run`, `Requested-by` and `Approved-by` trailers) and pushes to `main`. It refuses to run unless the repo-actions plugin started it |
| Approval and runs plugin | `backstage/packages/backend/src/acme/repoActionsPlugin.ts` | The `repo-actions` backend plugin at `/api/repo-actions`: listing, validation, preview (scaffolder dry run), requests, approval rules, task dispatch, verification on the origin, and the audit trail |
| Git helpers | `backstage/packages/backend/src/acme/git.ts` | Reads origins' `main`, and the org registry |
| Wiring | `backstage/packages/backend/src/index.ts`, `backstage/app-config.local.yaml` | Registers the three features. The config adds per-person access restrictions, `acme.orgOrigin`, persistent SQLite in `backstage/data/`, and catalog rules |
| CLI | `org/bin/repo-actions` | The org-level client. Every command prints JSON |
| Start and stop | `org/bin/backstage-up`, `org/bin/backstage-down` | Start the backend in the background and wait until actions are listed; stop it |
| Identities (sandbox) | `org/identities.json` | Each person's static token. `--as <person>` picks one |

## R1 to R8

| # | How it is met | Where |
|---|---|---|
| R1 Owned by the repository | Each repository's action is the `Template` in `.backstage/actions/mark-deprecated.yaml`, committed to that origin's `main` (alpha `61f28f3`, beta `78ba6f0`, gamma `244cf97`). Backstage reads definitions only from the origin's `main`, so a change lands when the team pushes it. The provider forces each repository's entities into a namespace named after that repository, so one repository cannot define another's action. The template's owner is the repository's own `maintainers` group, declared in its `catalog-info.yaml`. `list` shows each definition's origin, path and commit | `<repo>/.backstage/actions/mark-deprecated.yaml`, `<repo>/catalog-info.yaml`, `catalogModule.ts` |
| R2 Validated | `successor` has `pattern: ^[a-z0-9-]+/[a-z0-9-]+$` and must not be the repository itself (`not: {const: acme/<repo>}`). `reason` has `minLength: 20` and `maxLength: 200`. Both are required, and unknown inputs are refused (`additionalProperties: false`). `preview` and `request` validate against the repository's schema before anything is recorded or run, and return HTTP 400 `invalid-inputs` with per-input messages. The scaffolder validates again when the approved task is created | template `spec.parameters`; `validateInputs` in `repoActionsPlugin.ts` |
| R3 Previewed | `preview` runs a scaffolder **dry run** of the repository's own template. `acme:repo:write-file` only proposes the file in the dry-run workspace. The plugin then diffs that against the origin's `main` and returns `changes[]` (`path`, `operation` create/update/unchanged, `before`, `after`, unified `diff`) and the base commit. `request` stores that preview in the run, so the approver sees exactly what will land | `preview` in `repoActionsPlugin.ts`; dry-run branch in `scaffolderModule.ts` |
| R4 Approved | A run starts as `awaiting-approval`. `approve` refuses a requester approving their own run (403 `self-approval`), anyone outside the template owner group's members (403 `not-a-maintainer`, which is how carol is refused), a run that is not awaiting approval (409), and a run whose definition changed on `main` since the request (409 `definition-changed`). Every refusal is written to the run's record. Only after approval does the plugin create the scaffolder task, with its own service identity. People cannot bypass it: their tokens are restricted to `read` on the scaffolder, so direct task creation and dry runs return 403. The action itself also refuses any initiator other than `plugin:repo-actions`, which stops a guest user who creates a task directly | `approve` in `repoActionsPlugin.ts`; `accessRestrictions` in `backstage/app-config.local.yaml`; initiator check in `scaffolderModule.ts` |
| R5 Done means observed | While a run is `running`, the plugin polls its scaffolder task. When the task ends, it reads every previewed path from the origin's `main` (`git cat-file` on `refs/heads/main`). The run is `succeeded` only if the task completed **and** every file equals the approved content; otherwise it is `failed`. A rejected push fails the task, and the error (including the hook's message) and the observed `main` go into `outcome`. If a run asks for content `main` already has, nothing is pushed, and the run succeeds because the content is observed | `poll` in `repoActionsPlugin.ts` |
| R6 One place for the org | From `org/`: `bin/repo-actions list` shows `mark-deprecated` for alpha, beta and gamma (inputs, owner, approvers, definition commit), and `bin/repo-actions request <repo> mark-deprecated ...` works for any of them. The repository registry is `org/repositories.yaml`, read from `origins/org.git` `main` | `org/bin/repo-actions`, `org/repositories.yaml` |
| R7 Audited | `record <run-id>` returns who requested it and when (`requestedBy`, `requestedAt`), who approved or rejected it and when, the inputs, the stored preview, the definition ref, path, commit and spec hash, the scaffolder task id, the outcome with the observed `main` commit and file contents (`finishedAt`), and an append-only `events[]` list (`requested`, `approval-refused`, `approved`, `started`, `observed`, `succeeded` or `failed`, `rejected`, `cancelled`), each with `at` and `actor`. Records are in SQLite at `backstage/data/repo-actions.sqlite` and survive restarts. The commit on `main` carries the requester as author and `Run`, `Requested-by` and `Approved-by` trailers | `record`, `event` in `repoActionsPlugin.ts` |
| R8 Usable by agents | Every step is a non-interactive CLI command or HTTP call that prints JSON: `list`, `show`, `preview`, `request`, `approve`, `reject`, `cancel`, `status` (`--wait` blocks until a terminal state), `record` and `runs`. Exit codes: 0 ok, 1 refused or failed (with a JSON `error.code`), 2 usage, 3 `status --wait` ended in a state other than `succeeded`, 4 timeout | `org/bin/repo-actions` |

## Commands

All commands run from the org checkout. They need only Node on `PATH`.

```bash
cd /tmp/actions-ab/r1/backstage-3/org
export PATH=/tmp/bs/node24/bin:$PATH
```

**Start Backstage.** This is the only thing that must be running. It listens on
http://localhost:7210, waits until the actions are listed, and logs to
`/tmp/actions-ab/r1/backstage-3/backstage/backend.log`:

```bash
bin/backstage-up
# equivalent by hand: cd /tmp/actions-ab/r1/backstage-3/backstage && yarn workspace backend start
```

**List the actions** across all repositories (add `--repository alpha` for one):

```bash
bin/repo-actions --as ann list
bin/repo-actions --as ann show alpha mark-deprecated
```

**Preview.** This changes nothing:

```bash
bin/repo-actions --as ann preview alpha mark-deprecated \
  --successor acme/dependabot-proxy-v2 \
  --reason 'Replaced by the v2 proxy, which supports OIDC registries.'
```

**Request as ann.** This prints the run, including its `id` and the stored preview:

```bash
RUN=$(bin/repo-actions --as ann request alpha mark-deprecated \
  --successor acme/dependabot-proxy-v2 \
  --reason 'Replaced by the v2 proxy, which supports OIDC registries.' \
  | node -pe 'JSON.parse(require("fs").readFileSync(0)).id')
echo "$RUN"
```

**Approve as bob:**

```bash
bin/repo-actions --as bob approve "$RUN" --comment 'Looks right'
```

**Check the status** (`--wait` blocks until `succeeded`, `failed`, `rejected` or `cancelled`):

```bash
bin/repo-actions --as ann status "$RUN"
bin/repo-actions --as ann status "$RUN" --wait --timeout 120
```

**Read the record:**

```bash
bin/repo-actions --as ann record "$RUN"
bin/repo-actions --as ann runs --repository alpha    # all runs, newest first
```

Inputs can also be given as `--input successor=acme/x` or `--inputs-json '{"successor":"acme/x","reason":"..."}'`.
Without the CLI, use `curl -H 'Authorization: Bearer ann-token-0123456789'` against
`http://localhost:7210/api/repo-actions`:

| Method and path | Step |
|---|---|
| `GET /actions` and `GET /actions/:repo/:action` | list, show |
| `POST /actions/:repo/:action/preview` with body `{"inputs":{...}}` | preview |
| `POST /actions/:repo/:action/runs` with body `{"inputs":{...}}` | request |
| `POST /runs/:id/approve`, `/reject` (body `{"comment":"..."}`) and `/cancel` | decide |
| `GET /runs/:id/status`, `GET /runs/:id` and `GET /runs` | status, record, list runs |

**Stop Backstage:**

```bash
bin/backstage-down
```

## Testing the refusals

The inputs below differ from what is already on `main`. A run whose content is already on `main`
needs no push, so it succeeds even when pushes are blocked.

```bash
# Invalid inputs: HTTP 400 invalid-inputs, exit 1, nothing recorded or changed
bin/repo-actions --as ann preview beta mark-deprecated --successor 'Acme/Not Valid' --reason 'too short'
bin/repo-actions --as ann request beta mark-deprecated --successor acme/beta --reason 'Pointing at itself is refused.'
bin/repo-actions --as ann request beta mark-deprecated --successor acme/x --reason "$(printf 'x%.0s' {1..201})"

# Self-approval and carol's approval: HTTP 403, the run stays awaiting-approval, refusals are recorded
RUN=$(bin/repo-actions --as ann request beta mark-deprecated --successor acme/valtio-store-v2 \
  --reason 'Second test: state moves to the v2 store.' | node -pe 'JSON.parse(require("fs").readFileSync(0)).id')
bin/repo-actions --as ann approve "$RUN"      # self-approval
bin/repo-actions --as carol approve "$RUN"    # not a maintainer
bin/repo-actions --as ann status "$RUN"       # awaiting-approval
bin/repo-actions --as bob reject "$RUN" --comment 'test only'

# A rejected push: the run is failed, not succeeded, and main is unchanged
printf '#!/bin/sh\necho "main is frozen" >&2\nexit 1\n' > ../origins/gamma.git/hooks/pre-receive
chmod +x ../origins/gamma.git/hooks/pre-receive
RUN=$(bin/repo-actions --as ann request gamma mark-deprecated --successor acme/graphify-next \
  --reason 'Push-rejection test: this must not succeed.' | node -pe 'JSON.parse(require("fs").readFileSync(0)).id')
bin/repo-actions --as bob approve "$RUN"
bin/repo-actions --as bob status "$RUN" --wait   # exit 3, status failed, outcome.message has the hook's error
rm ../origins/gamma.git/hooks/pre-receive

# Direct scaffolder access with a person's token is refused (403)
curl -s -o /dev/null -w '%{http_code}\n' -X POST http://localhost:7210/api/scaffolder/v2/tasks \
  -H 'Authorization: Bearer ann-token-0123456789' -H 'Content-Type: application/json' \
  -d '{"templateRef":"template:alpha/mark-deprecated","values":{}}'
```

## Limits

- Identity is the sandbox's static tokens. A person is whoever the token's subject names
  (`user:default/<subject>`). Signed-in Backstage users are mapped by their user entity ref.
- Approvers are whoever the repository's `maintainers` group lists on its `main`, the same trust
  model as `CODEOWNERS`.
- Two approved runs on one repository at the same moment: the second push is refused as
  non-fast-forward, and that run fails rather than retrying.
- The `definition-changed` guard is implemented but was not exercised in this session.
- The Backstage changes live in the local `backstage/` checkout (`packages/backend/src/acme/`,
  `packages/backend/src/index.ts` and the git-ignored `app-config.local.yaml`). That checkout has
  no code-host origin in this sandbox.
