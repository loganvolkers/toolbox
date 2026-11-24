# Handoff: `mark-deprecated` on Backstage

`mark-deprecated` is a Backstage software template that each repository keeps on its own `main`.
A custom `self-service` backend plugin puts four things in front of it: input validation, a
dry-run preview, an approval gate, and a check that reads the result back from the origin. It
also writes an append-only audit log. People and agents use one CLI, `org/bin/acme`, which
prints JSON. They can also call the same HTTP API with curl.

All three runs requested by ann and approved by bob ended `succeeded`. Each was marked
succeeded only after `DEPRECATED.md` with the expected content was read from the origin's `main`:

| Repository | Run id | Commit on origin `main` |
|---|---|---|
| alpha | `ea0abad6-9e6b-4fc0-8a7f-19c5b5b6b04f` | `20c01ee` |
| beta | `ef1719f2-7cbe-489e-aa5e-2b0295e94d06` | `f71968b` |
| gamma | `9a89c3b1-05d2-4df4-a5d4-b91c346fbbd9` | `72307e8` |

The record also holds one earlier alpha run, `0e80d99e-e98e-4b63-8051-82c47a50347f`. It was a
test, and it is `failed`. A temporary pre-receive hook rejected its push. Before that, ann tried
to approve her own run and carol tried to approve it, and both attempts were refused. The record
shows all of this.

Each `DEPRECATED.md` is `# Deprecated: use <successor>`, a blank line, then `<reason>`, and it
ends with one newline.

## How it is built

| Part | File | What it does |
|---|---|---|
| Catalog provider | `backstage/packages/backend/src/acme/catalogModule.ts` | Every 5 s, reads the org registry `catalog/repositories.yaml` and the people file `catalog/people.yaml` from `origins/org.git` `main`. For each repository it reads `catalog-info.yaml` and `.backstage/**.yaml` from `origins/<repo>.git` `main`. Checkouts and uncommitted files are never read. A repository's entities are put in the catalog namespace named after that repository, so alpha cannot define an action for beta. Each entity is annotated with its source commit, path and blob. |
| Scaffolder actions | `backstage/packages/backend/src/acme/scaffolderModule.ts` | `acme:git:put-file` makes a shallow clone of the origin and builds one commit. In dry-run it returns the diff. In a real run it pushes with `git push`, without force, so the origin's hooks apply. `acme:git:verify-file` reads the file from the origin's `main` and fails unless the content matches exactly. Neither action takes the target repository as an input. They take it from the template's own `acme.io/repository` annotation. |
| Approval gate and audit | `backstage/packages/backend/src/acme/selfServicePlugin.ts` | Mounted at `/api/self-service`. It lists actions, validates inputs, previews with the scaffolder's `/v2/dry-run`, records runs, enforces approval, starts the scaffolder task, and decides the outcome by reading the origin itself. |
| Shared git helpers | `backstage/packages/backend/src/acme/git.ts` | Reads the registry and reads files from a branch. |
| Wiring | `backstage/packages/backend/src/index.ts` (last three `backend.add` lines) | Registers the three modules above. |
| Config | `backstage/app-config.local.yaml` | Restricts each person's token to `self-service` and catalog reads. Stores data in SQLite files under `backstage/data/`. Allows `Template`, `User` and `Group` entities. Holds the `acme.org` settings. |
| Action definition | `<repo>/.backstage/actions/mark-deprecated.yaml` and `<repo>/catalog-info.yaml`, committed on each `origins/<repo>.git` `main` | The template, owned by `group:default/<repo>-maintainers`, and the repository's Component. |
| Org place | `org/catalog/repositories.yaml`, `org/catalog/people.yaml`, `org/bin/acme`, `org/identities.json`, committed on `origins/org.git` `main` | The registry, the people and groups (ann and bob maintain all three repositories; carol is in `developers`), the CLI, and the mapping from person to token. |

The scaffolder only runs a task when the `self-service` plugin asks it to, after an approval.
People's tokens get HTTP 403 from `/api/scaffolder/*`, from catalog writes, and from MCP.

## Requirements

| # | How it is met | Where |
|---|---|---|
| R1 Owned by the repository | Each repository's `mark-deprecated` is a Backstage Template in `.backstage/actions/mark-deprecated.yaml`, committed to its origin's `main`, with `spec.owner: group:default/<repo>-maintainers`. The catalog reads it only from that `main`, every 5 s. A change merged there, such as a new input or a different owner, is what the system lists and runs. `acme show <repo> mark-deprecated` prints `definedIn` (origin, path, commit, blob). If the definition changes between preview and approval, the approval is refused with 409 and the run must be requested again. That check is in the code but was not exercised in this session. | `alpha/.backstage/actions/mark-deprecated.yaml` (and beta, gamma); `catalogModule.ts` |
| R2 Validated | The template's JSON Schema is the source of the rules: `successor` must match `^[a-z0-9-]+/[a-z0-9-]+$` and must not be the repository itself (`not: {const: acme/<repo>}`); `reason` must be 20 to 200 characters; both are required; no other keys are allowed. The plugin checks this before any preview, record or task, and answers HTTP 400 with a per-parameter `errors` list (CLI exit code 1). The scaffolder checks the same schema again when the task is created. | `selfServicePlugin.ts` `validateParameters`; the template's `spec.parameters` |
| R3 Previewed | `acme preview` (`POST /api/self-service/preview`) runs the template through the scaffolder's dry-run. `put-file` builds the real commit in a throwaway clone and returns, per change, the origin, branch, path, current content (`before`), new content (`after`), base commit and unified `diff`. Nothing is pushed and nothing is recorded. `acme request` runs the same preview and stores it in the run, so the approver sees exactly what they are approving. | `scaffolderModule.ts` (dry-run branch); `selfServicePlugin.ts` `preview` |
| R4 Approved | A run is created as `awaiting-approval`, and no task exists yet. `approve` is refused with 403 when the approver is the requester, and when the approver is not a member of the template's owner group. carol is in neither maintainer group. Each refusal is recorded as an `approval-refused` event. Only the first valid approval moves the run to `running` and starts the task, because a conditional update means a run cannot be approved twice. People's tokens cannot reach the scaffolder, so there is no way around the gate. | `selfServicePlugin.ts` `/runs/:id/approve`; `app-config.local.yaml` `accessRestrictions`; `org/catalog/people.yaml` |
| R5 Done means observed | The task's last step, `acme:git:verify-file`, reads `DEPRECATED.md` from the origin's `main` and fails on a mismatch. The plugin does not trust the task alone. When the task reports `completed`, the plugin reads every previewed path from the origin's branch itself and marks the run `succeeded` only if the content is byte-for-byte the previewed `after`. Otherwise the run is `failed`. If the push is rejected, the task fails and the run is `failed`, with git's error in `outcome.error`. | `scaffolderModule.ts` `verify-file`; `selfServicePlugin.ts` `reconcile` |
| R6 One place for the org | From `org/`, `bin/acme list` lists `mark-deprecated` for every repository in `catalog/repositories.yaml`. `bin/acme request <repo> mark-deprecated ...` requests a run on any of them. Nobody has to enter a repository. | `org/bin/acme`, `org/catalog/repositories.yaml` |
| R7 Audited | Each run row stores `requestedBy`/`requestedAt`, `approvedBy`/`approvedAt`, `inputs`, `startedAt`, `finishedAt` and `outcome` (task status, error, and the observed commit and blob). An append-only `events` list records each step with a timestamp and actor: `requested`, `previewed`, `approval-refused`, `approved`, `started`, `succeeded`, `failed` and `rejected`. The data is SQLite in `backstage/data/self-service.sqlite`, so it survives restarts (tested). The pushed commit is authored by the requester and carries `Run:`, `Requested-by:` and `Approved-by:` trailers. | `selfServicePlugin.ts` tables `runs` and `run_events`; `acme record <id>` |
| R8 Usable by agents | Every step is one non-interactive command that prints JSON on stdout: `list`, `show`, `preview`, `request`, `approve [--wait]`, `reject`, `status [--wait]`, `record`, `runs`. Exit codes are 0 ok; 1 refused (400, 403, 404, 409; the JSON body says why); 2 usage; 3 Backstage unreachable or 5xx; 4 a waited-for run did not succeed. The same API works with curl and a bearer token. | `org/bin/acme` |

## Commands

Run these from any shell. Node is needed for the backend and the CLI.

```sh
export PATH=/tmp/bs/node24/bin:$PATH
cd /tmp/actions-ab/r7/backstage-2/org
```

### Start Backstage (port 7870) and wait until it is ready

```sh
tmux new-session -d -s r7-backstage-2-backend -c /tmp/actions-ab/r7/backstage-2/backstage \
  'export PATH=/tmp/bs/node24/bin:$PATH; yarn workspace backend start > /tmp/actions-ab/r7/backstage-2/backstage/backend.log 2>&1'
until bin/acme list --as ann 2>/dev/null | grep -q '"action": "mark-deprecated"'; do sleep 2; done
```

Startup takes about 30–40 s. To stop it:

```sh
tmux kill-session -t r7-backstage-2-backend
```

### Identity

`--as <person>` takes the token from `org/identities.json`. You can instead set `ACME_TOKEN`.
`ACME_BACKSTAGE_URL` overrides `http://localhost:7870`.

### List the actions

```sh
bin/acme list --as ann
bin/acme list --as ann --action mark-deprecated --repo beta
bin/acme show alpha mark-deprecated --as carol
```

### Preview (records nothing)

```sh
bin/acme preview alpha mark-deprecated --as ann \
  --param successor=acme/dependabot-proxy-v2 \
  --param 'reason=Replaced by the v2 proxy, which supports OIDC registries.'
```

### Request as ann

This prints the run, including `id` and `preview`.

```sh
RUN=$(bin/acme request alpha mark-deprecated --as ann \
  --param successor=acme/dependabot-proxy-v3 \
  --param 'reason=Replaced again, by the v3 proxy with a new cache.' | jq -r .id)
```

### Approve as bob

`--wait` blocks until the outcome and exits with 0 only on `succeeded`.

```sh
bin/acme approve "$RUN" --as bob --wait
```

### Check the status and read the record

```sh
bin/acme status "$RUN" --as ann          # add --wait to block until it finishes
bin/acme record "$RUN" --as ann          # inputs, people, times, preview, outcome, events
bin/acme runs --as ann --repo alpha      # every run, newest last
```

### Reject a pending run

The requester or a maintainer can do this.

```sh
bin/acme reject "$RUN" --as bob --comment 'not yet'
```

### The same API with curl

```sh
curl -s -H 'Authorization: Bearer ann-token-0123456789' localhost:7870/api/self-service/actions
curl -s -X POST -H 'Authorization: Bearer ann-token-0123456789' -H 'content-type: application/json' \
  localhost:7870/api/self-service/runs \
  -d '{"repository":"beta","action":"mark-deprecated","parameters":{"successor":"acme/x","reason":"twenty characters at least"}}'
curl -s -X POST -H 'Authorization: Bearer bob-token-0123456789' localhost:7870/api/self-service/runs/<id>/approve
curl -s -H 'Authorization: Bearer ann-token-0123456789' localhost:7870/api/self-service/runs/<id>/status
curl -s -H 'Authorization: Bearer ann-token-0123456789' localhost:7870/api/self-service/runs/<id>
```

## Testing the refusals

Each command below should be refused with exit code 1 and a JSON `error`. None of them changes
any origin.

```sh
# Invalid inputs: HTTP 400 with an errors[] entry per parameter; no run is created.
bin/acme request alpha mark-deprecated --as ann --param successor=NotValid --param reason=short
bin/acme request alpha mark-deprecated --as ann --param successor=acme/alpha --param 'reason=Successor may not be itself.'
bin/acme request alpha mark-deprecated --as ann --param successor=acme/x --param "reason=$(printf 'x%.0s' $(seq 201))"
bin/acme request alpha mark-deprecated --as ann --param successor=acme/x --param 'reason=Long enough reason here.' --param extra=1

# Self-approval and carol's approval: HTTP 403, recorded as approval-refused; the run stays awaiting-approval.
RUN=$(bin/acme request gamma mark-deprecated --as ann --param successor=acme/other --param 'reason=Testing the approval rules today.' | jq -r .id)
bin/acme approve "$RUN" --as ann
bin/acme approve "$RUN" --as carol
bin/acme status "$RUN" --as ann

# Going around the gate: HTTP 403, because people's tokens cannot reach the scaffolder.
curl -s -X POST -H 'Authorization: Bearer ann-token-0123456789' -H 'content-type: application/json' \
  localhost:7870/api/scaffolder/v2/tasks -d '{"templateRef":"template:gamma/mark-deprecated","values":{}}'
```

### A rejected push

The run must end `failed`, never `succeeded`. Use inputs that differ from what is already on
`main`. If the content is identical, there is nothing to push, and the run succeeds because the
file is observed.

```sh
RUN=$(bin/acme request gamma mark-deprecated --as ann --param successor=acme/other --param 'reason=Testing a push that the origin rejects.' | jq -r .id)
printf '#!/bin/sh\necho "main is frozen" >&2\nexit 1\n' > ../origins/gamma.git/hooks/pre-receive
chmod +x ../origins/gamma.git/hooks/pre-receive
bin/acme approve "$RUN" --as bob --wait; echo "exit=$?"   # status failed, exit=4, git's error in outcome.error
git -C ../origins/gamma.git show main:DEPRECATED.md      # unchanged
rm ../origins/gamma.git/hooks/pre-receive
```

## Limits

- **Identity:** a person is who their static token says, through `org/identities.json` or `ACME_TOKEN`, because the sandbox has no passwords. A Backstage user session (`user:default/<name>`) works the same way.
- **Group membership:** maintainer membership is direct only. Nested groups are not followed.
- **Scaffolder audit:** the scaffolder records its own tasks under the `self-service` plugin's identity, so the scaffolder's `createdBy` is empty. The people appear in the self-service record and in the commit.
- **Dependencies:** the backend code uses packages that are already installed (`@backstage/plugin-scaffolder-node`, `@backstage/plugin-catalog-node`, `jsonschema`, `yaml`, `express-promise-router`, `knex`). They are not added to `packages/backend/package.json`, so that `yarn.lock` does not change.
- **Concurrent runs:** if two approved runs on the same repository push at the same moment, the second push is rejected as non-fast-forward and that run is reported `failed`.
- **Backstage repository:** `backstage/` is not committed. Its working tree already had unrelated `yarn.lock` and `packages/app/package.json` changes from setup. My files are listed under *How it is built*.
