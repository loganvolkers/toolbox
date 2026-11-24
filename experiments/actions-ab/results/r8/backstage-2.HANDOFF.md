# Hand-off: `add-notice` on Backstage

`add-notice` is a Backstage scaffolder template that each repository defines and owns. People and agents use it through one org-level command, `org/bin/acme`, or its HTTP API. Each run is validated, previewed, approved by a member of the release group other than the requester, executed by the scaffolder, and marked `succeeded` only after Backstage has read the expected README back from the origin's `main`.

The three requested runs have succeeded (requested by ann):

| Repository | Run id | Approved by | Commit on `main` |
|---|---|---|---|
| alpha | `9b14d65c-8a2d-4dc4-b997-61f7e7b8f740` | dave | `f36ad54` |
| beta | `5fb57d5f-4c5b-4bb0-99d2-0ea7f930ceaa` | erin | `521ab26` |
| gamma | `314e29c9-9173-48d9-86e0-cecd1ab98cde` | erin | `3ca6f08` |

## How it fits together

Paths are relative to `/tmp/actions-ab/r8/backstage-2`.

| Piece | Where | What it does |
|---|---|---|
| Each repository's action | `<repo>/.backstage/add-notice.yaml`, `<repo>/catalog-info.yaml` | A scaffolder `Template` (`template:<repo>/add-notice`), owned by `group:default/<repo>-maintainers`. It declares the inputs and the steps `acme:notice:apply` and `acme:notice:verify` |
| Org config | `org/acme.yaml` | Lists the three repositories and their origins. Sets the approver group for `add-notice`: `group:default/release` |
| People and groups | `org/catalog/people.yaml` | Users ann, bob, carol, dave, erin; the `release` group (dave, erin); one maintainers group per repository |
| Catalog provider | `backstage/packages/backend/src/acme/catalogModule.ts` | Every 5 s, reads `org.git` and each repository's `main` straight from the bare origins (never from the checkouts) into the catalog. It skips Users and Groups defined in a repository, and any entity a repository puts outside its own namespace |
| Scaffolder actions | `backstage/packages/backend/src/acme/scaffolderModule.ts`, `notice.ts`, `git.ts` | `acme:notice:apply`: in a dry run it writes the planned diff; in a real run it pushes only for a run that the gate approved and dispatched. `acme:notice:verify`: reads `main` back and fails unless the README is exactly what was approved |
| Approval gate and run record | `backstage/packages/backend/src/acme/runsPlugin.ts` (Backstage plugin `acme-runs`, `/api/acme-runs`) | Validate, preview, request, approve/reject/cancel, dispatch the scaffolder task, observe `main`, keep the audit record |
| Org policy reader | `backstage/packages/backend/src/acme/org.ts` | Reads `acme.yaml` and `catalog/*.yaml` from `origins/org.git` `main` |
| Wiring and config | `backstage/packages/backend/src/index.ts`, `backstage/app-config.yaml` (`acme.orgOrigin`, on-disk SQLite in `backstage/.data/`, scaffolder working directory `backstage/.work/`) | |
| CLI | `org/bin/acme` (Node, no dependencies) | JSON on stdout for every command |
| Unit tests | `backstage/packages/backend/src/acme/notice.test.ts` | README transform and input rules: `yarn workspace backend test src/acme` |

A run moves through these states: `awaiting_approval`, then `approved`, then `running`, and ends as `succeeded` or `failed`. A pending run can instead end as `rejected` (by a release member) or `cancelled` (by its requester).

## Requirements

| # | How it is met |
|---|---|
| R1 Owned by the repository | The definition is `<repo>/.backstage/add-notice.yaml`, committed to `origins/<repo>.git` `main` by ann, a maintainer. Its `spec.owner` is `group:default/<repo>-maintainers`. Backstage reads it only from the origin's `main` (`catalogModule.ts`), so the team changes it with a normal commit and push, and the catalog picks the change up within a few seconds. Previews and runs use the catalog copy, so for example lowering `maxLength` in beta's file makes beta refuse longer messages. Who approves is org policy and cannot be changed from a repository's file. |
| R2 Validated | `POST /runs` and `POST /preview` check the inputs before anything is recorded or run (`notice.ts` `validateInputs`): `level` must be `info` or `warning`; `message` must be a string of 10 to 160 characters (counted as Unicode code points) with no line breaks or control characters; unknown fields are refused. The repository's own template schema is then enforced by the scaffolder dry run. Refusals are HTTP 400 with `errors: [{field, message}]` (CLI exit code 1), and no run is created. `acme:notice:apply` checks the inputs again, so a looser template cannot let bad input through. |
| R3 Previewed | `acme preview` (`POST /api/acme-runs/preview`) runs the repository's template through the scaffolder's `POST /api/scaffolder/v2/dry-run`. In a dry run, `acme:notice:apply` writes the unified diff, the README path and the base and expected git blob ids. `acme request` records that preview in the run, so the approver reads the same diff in `acme record`. At execution time, apply refuses to push if the README on `main` has changed since the preview (the run fails with "request a new run"), so what lands is what was previewed. |
| R4 Approved by the release group | `POST /runs/:id/approve` (`runsPlugin.ts`) reads `org/acme.yaml` and `org/catalog/people.yaml` fresh from `origins/org.git` `main` on every approval. It refuses with 403 if the approver is the requester, or is not a member of the group named in `actions.add-notice.approvers`. Each refusal is written to the run's record as an `approval_refused` event. Only an approved run is dispatched to the scaffolder, with a one-time secret, and `acme:notice:apply` asks the gate to claim the run with that secret before it pushes. A task started directly through the scaffolder API therefore fails before it changes anything (tested both without a run and with another run's id). |
| R5 Done means observed | The template's `acme:notice:verify` step reads the README's blob on the origin's `main` and fails if it differs from the approved blob. Separately, the gate polls the scaffolder task. When the task completes, the gate itself runs `git rev-parse main:<README>` on the origin and sets `succeeded` only if it matches, recording an `observed` event with the commit and blob. A rejected push fails the apply step. A push that is accepted but does not stay on `main` fails the verify step. In both cases the run is `failed`, with the reason and the commit that was observed on `main`. |
| R6 One place for the org | `org/bin/acme list` lists `add-notice` for every repository in `org/acme.yaml` (`GET /api/acme-runs/actions`). `acme request <repo> ...` requests a run on any of them. You never need to `cd` into a repository. |
| R7 Audited | Each run's record (`acme record <id>`, `GET /api/acme-runs/runs/:id`) contains `requestedBy`/`requestedAt`, `inputs`, `preview`, `approvedBy`/`approvedAt`, and `approval` (the group, its members and the org commit used to decide). It also has `task` (the scaffolder task id), `push` (commit and time), `outcome` (status, time, observed commit and blob, reason), and a timestamped `events` list: `requested`, `previewed`, `approval_refused`, `approved`, `dispatched`, `applying`, `pushed`, `observed`, then `succeeded` or `failed`. Records are kept in `backstage/.data/acme-runs.sqlite` and survive restarts. The commit on `main` is authored as the requester and carries `Requested-by:`, `Approved-by:` and `Acme-Run:` trailers. Backstage auditor events (`run-request`, `run-approve`) also go to the backend log. |
| R8 Usable by agents | Every step is a non-interactive CLI command that prints JSON (`org/bin/acme`), or an HTTP call with `Authorization: Bearer <token>`. Exit codes: `0` ok; `1` refused (the HTTP error body is printed, with `httpStatus`); `2` usage or connection error; `3` a run that `--wait` waited on did not succeed. |
| R9 The group is defined once | The release group is the `release` Group in `org/catalog/people.yaml` (`spec.members`; a User's `spec.memberOf: [release]` also counts). `org/acme.yaml` names it as the approver for `add-notice` in all repositories, and no repository file mentions it. Because the gate reads it from `origins/org.git` `main` at each approval, a pushed change applies to the next approval in all three repositories, with no restart. The catalog shows the same group as `group:default/release`. |

## Commands

### Start

Ports 7170/7171 must be free. Start in a tmux session (or any terminal) and wait for `Listening on :7170`. The first start takes about 40 s.

```bash
tmux new-session -d -s r8-backstage-2-backend -c /tmp/actions-ab/r8/backstage-2/backstage \
  'export PATH=/tmp/bs/node24/bin:$PATH; yarn workspace backend start 2>&1 | tee /tmp/actions-ab/r8/backstage-2/backstage/.backend.log'
# ready once the catalog lists add-notice for all three repositories
until curl -sf -H 'Authorization: Bearer ann-token-0123456789' http://localhost:7170/api/acme-runs/actions | jq -e '.items | length == 3' >/dev/null; do sleep 2; done
```

To stop it, run `tmux kill-session -t r8-backstage-2-backend` (or press Ctrl-C twice in its terminal).

### Use

```bash
cd /tmp/actions-ab/r8/backstage-2/org
export PATH=/tmp/bs/node24/bin:$PATH

bin/acme --as ann list                                  # add-notice in alpha, beta, gamma
bin/acme --as ann policy                                # approver group and its current members
bin/acme --as ann preview beta --level info --message "Releases now ship on the shared release train every second Tuesday."

ID=$(bin/acme --as ann request alpha --level warning --message "This proxy is frozen while the v2 rewrite lands; please file new issues against v2." | jq -r .id)
bin/acme --as dave approve "$ID" --wait                 # or: --as erin; --wait blocks until the run ends
bin/acme --as ann status "$ID"                          # short status and outcome
bin/acme --as ann record "$ID"                          # full audit record
bin/acme --as ann runs                                  # every run; filters: --repo R --status S
bin/acme --as erin reject "$ID" --reason "wording"      # release member, while awaiting approval
bin/acme --as ann cancel "$ID"                          # requester, while awaiting approval
```

`--as NAME` maps a person to their static token through `org/.tokens.json`, which is untracked; `ACME_TOKENS` points at another file. Setting `ACME_TOKEN=<token>` uses that token directly. `--inputs '<json>'` sends inputs verbatim, for example to test unknown fields. `ACME_BACKSTAGE_URL` defaults to `http://localhost:7170`.

The same calls over HTTP, where `T` is a person's token, for example `T=ann-token-0123456789`:

```bash
B=http://localhost:7170/api/acme-runs
curl -s $B/actions -H "Authorization: Bearer $T"
curl -s -X POST $B/preview -H "Authorization: Bearer $T" -H 'Content-Type: application/json' \
  -d '{"repo":"beta","action":"add-notice","inputs":{"level":"info","message":"Ten or more characters"}}'
curl -s -X POST $B/runs -H "Authorization: Bearer $T" -H 'Content-Type: application/json' \
  -d '{"repo":"beta","action":"add-notice","inputs":{"level":"info","message":"Ten or more characters"}}'
curl -s -X POST $B/runs/$ID/approve -H "Authorization: Bearer dave-token-0123456789" -H 'Content-Type: application/json' -d '{}'
curl -s $B/runs/$ID -H "Authorization: Bearer $T"        # status and record
curl -s $B/runs -H "Authorization: Bearer $T"
```

### Change who is in the release group

Edit `spec.members` of the `release` Group in `org/catalog/people.yaml`, commit, and push to the org's origin. The next approval uses the new list. A change that is not pushed has no effect.

```bash
cd /tmp/actions-ab/r8/backstage-2/org
sed -i 's/members: \[dave, erin\]/members: [dave, carol]/' catalog/people.yaml   # carol in, erin out
git commit -am "release group: carol replaces erin" && git push origin main
bin/acme --as ann policy                                # shows members [carol, dave]
```

## Testing notes

- **Invalid inputs:** for example `--level urgent`, a 9- or 161-character message, `--inputs '{"level":"info","message":"two\nlines here"}'`, or an extra field. Each returns HTTP 400 with per-field errors and creates no run.
- **Self-approval and outsiders:** `--as ann approve` on ann's own run, and `--as bob` or `--as carol` while they are not in the group, return 403. Each attempt is recorded as an `approval_refused` event. A token whose subject is not a User in `org/catalog/people.yaml` gets 403 for every call.
- **Rejected push:** install a hook on the origin, then request and approve a run. It ends `failed` with git's rejection message, and `main` is unchanged. Remove the hook afterwards.

  ```bash
  printf '#!/bin/sh\necho "main is frozen" >&2\nexit 1\n' > /tmp/actions-ab/r8/backstage-2/origins/beta.git/hooks/pre-receive
  chmod +x /tmp/actions-ab/r8/backstage-2/origins/beta.git/hooks/pre-receive
  ```

  A `post-receive` hook that moves `main` back to its old commit is also caught, by the verify step.
- **The README changed after the preview:** the run fails without pushing. Request a new run to get a fresh preview.
- **Repeating a notice:** a run whose notice is already there succeeds without a new commit (`push.changed: false`). A different notice replaces the existing notice line and the blank line after it.

## Limits

- The bare origins have no access control, so anyone with filesystem access can push to `main` without Backstage. The gate controls only changes made through Backstage.
- A repository's maintainers can change their template, for example to remove the verify step. They cannot change who approves, and the gate's own observation of `main` still decides `succeeded`.
- Backstage also shows its stock example entities (`examples/`); they are not part of the org.
