# set-codeowners: handoff

`set-codeowners` is a Seikika self-service action. Each project (`acme-w1/alpha`, `beta`, `gamma`)
declares it in its own `.seikika/` on `main`. This project (`acme-w1/org`) is the Seikika **org
portal**: it lists the action for all three projects, holds and signs approvals, runs approved
changes, and keeps the run records. People and agents use it through `bin/acme-actions`, which
takes who you are from your GitLab token.

**Nothing has to be running.** There is no server and no CI job. Each command is a short-lived
`seikika run` in this checkout. The shared GitLab Runner is not used.

The three requested runs succeeded:

| Project | Run (org) | pattern | owners | Requested by | Approved by | Commit on `main` |
|---|---|---|---|---|---|---|
| alpha | `run_2fzn5e9byho5` | `*` | `@acme/proxy-maintainers` | ann | dave | `8ed07ae` |
| beta | `run_2fznpd359ul7` | `/docs/` | `@acme/docs @carol` | ann | erin | `3ab4af8` |
| gamma | `run_2fznpur9gx71` | `/graphify/security.py` | `@acme/security` | ann | erin | `a9d8e73` |

## How it fits together

| Piece | Where | What it does |
|---|---|---|
| The action | `<project>/.seikika/actions/set-codeowners.yaml` | Inputs schema, approval rule `['group:acme-w1/release']`, a `change` executor with `land: push` to `main`, and the pinned org approval key |
| The effect | `<project>/.seikika/scripts/set-codeowners.py` | Edits root `CODEOWNERS`: replaces the first line whose first field is the pattern, otherwise appends; creates the file; ends it with a newline; leaves every other line byte for byte |
| Its tests | `<project>/.seikika/tests/set-codeowners.test.yaml`, `set_codeowners_test.py` | Lifecycle cases (`seikika test .seikika`) and file-editing cases (`python3 .seikika/tests/set_codeowners_test.py`) |
| The project's portal | `<project>/.seikika/seikika.yaml` | Names the project and includes `actions/*.yaml` |
| The org portal | `org/seikika.yaml`, `org/groups.fga` | Lists the three projects (`federation.children`); the `gitlab` connector reads acme-w1's groups and their **direct** members from GitLab into the approval model |
| Identity | `org/bin/acme-actions` | Resolves your GitLab token to your username (`GET /api/v4/user`), requires membership of `acme-w1`, refuses `--as`/`--grant`/`--portal`/`--url`/`--skip`/`--fixtures`, then runs `seikika run --as user:<username>` on this portal |
| Service | `/tmp/actions-gitlab/r0/seikika-1/service/` (not in git) | `bot.token`: the group access token `r0-seikika-1-actions` (bot, Maintainer, `read_api` + `write_repository`), which pushes approved changes and reads groups. `home/`: its git identity. `state/`: the approval signing key, the clones of the projects and the run records. `org/bin/setup-service` recreates it |

`<project>` is `/tmp/actions-gitlab/r0/seikika-1/{alpha,beta,gamma}`. The three projects' files
are the same, byte for byte, except `seikika.yaml`.

## Requirements

| # | How it is met |
|---|---|
| R1 Owned by the project | The definition is in each project's `main`: `.seikika/actions/set-codeowners.yaml`, `.seikika/scripts/set-codeowners.py`, `.seikika/tests/`, `.seikika/seikika.yaml`. The org portal clones each project and re-reads `.seikika/` on `main` before every list, preview and run, so a change the project's maintainers commit there takes effect at once. Before committing, check it with `seikika validate .seikika && seikika test .seikika`. Example: the commits `set-codeowners: quote the pattern input's description` changed the action in all three projects |
| R2 Validated | The `inputs` JSON Schema in `set-codeowners.yaml`. `pattern` must match `^[/*][^\s#]*$`, with length 1–100. `owners` must match `^@[A-Za-z0-9_.-]+(/[A-Za-z0-9_.-]+)*( @…){0,2}$`. Both are required, and `additionalProperties: false` refuses extra inputs. Seikika checks the schema before any preview: an invalid input gives exit 2 and `errors` in the JSON, and nothing is recorded or changed. `set-codeowners.py` checks the same rules again and exits 2 without writing |
| R3 Previewed | Without `--yes`, a request is a preview: the unified diff of `CODEOWNERS` against `main` (`effects`), who must approve, and a confirm token. Nothing changes. The approver sees the same `effects` in `--verify <run>` before approving. If `main` changed so that the change would differ from the confirmed preview, the run fails before it commits |
| R4 Approved by the release group | `approval: "['group:acme-w1/release']"` in `set-codeowners.yaml`. Seikika never counts the requester, and refuses (exit 2) anyone who is not a member of that group, as `org/seikika.yaml`'s connector reads it. Identity comes from the GitLab token in `org/bin/acme-actions`, so nobody can claim to be someone else. Each project pins the org portal's key (`acceptApprovalsFrom`), so it accepts only approvals the org portal signed. Refused attempts are recorded as `approval_refused` events |
| R5 Done means observed | `executor: {type: change, land: push, base: main}`. After pushing, Seikika fetches `origin/main`, checks that the commit is on it and that `CODEOWNERS` has the previewed blob (`verification.child.files[].matches`), and only then records `succeeded`. A push GitLab rejects (for example, `main` protected with "No one" allowed) gives state `failed`, exit 1, with GitLab's message. Nothing is retried or forced. This was tested on gamma with "No one" allowed to push or merge, then the protection was restored |
| R6 One place for the org | `org/seikika.yaml` `federation.children` lists the three projects as `portal:alpha`, `portal:beta` and `portal:gamma`. `org/bin/acme-actions` lists, previews, requests, approves and reads runs for any of them from `/tmp/actions-gitlab/r0/seikika-1/org` |
| R7 Audited | Each run's record (`--verify <run> -o json`) has `requested_by`, `inputs`, `approval.approved_by` / `satisfied` / `rejected_by`, `state`, `error`, `outputs.commit`, `verification`, and `events[]`. Each event has `at` (UTC) and `by`: previewed, confirmed, awaiting_approval, approval_refused, approved (with the note), dispatched, running, succeeded or failed. Records are kept in `service/state/`. The commit on `main` carries `Seikika-Run`, `Requested-By` and `Approved-By` trailers, and GitLab shows the push by the `r0-seikika-1-actions` bot |
| R8 Usable by agents | Every step is a non-interactive shell command with `-o json`, and the token comes from `GITLAB_TOKEN` or `GITLAB_TOKEN_FILE`. Exit codes: 0 succeeded, 1 failed, 2 invalid/blocked/refused (including the wrapper's own refusals, a JSON `{error, message}` on stderr), 3 awaiting approval, 4 unverified |
| R9 The group is GitLab's | The `gitlab` connector in `org/seikika.yaml` (`group: acme-w1`, `minAccess: 5`) reads `acme-w1/release`'s direct members from GitLab's API on every command, and they become `member` tuples of `group:acme-w1/release` (`org/groups.fga`). Members inherited from `acme-w1`, such as ann and bob, are not counted. Add or remove a direct member in GitLab and the next `--approve` follows it. This was tested: with erin removed she was refused, and she was restored afterwards |

**Trust boundary.** `bin/acme-actions` is the only supported entry point. The service directory
holds the bot token and the approval signing key. Anyone who can read it on this machine could
bypass the wrapper, as an operator could. Keep it readable only by the operator (`chmod 700` is
set). In production, put it on a host that people reach only through the wrapper.

## Commands

Start every shell like this. There is nothing to start:

```sh
export HOME=/tmp/actions-gitlab/r0/seikika-1/home
export PATH=/tmp/actions-gitlab/r0/seikika-1/bin:$PATH
cd /tmp/actions-gitlab/r0/seikika-1/org && git pull -q
P=/tmp/actions-gitlab/r0/seikika-1/people
```

Act as someone by giving their token: `GITLAB_TOKEN_FILE=$P/<name>.token` (or
`GITLAB_TOKEN=<token>`). Ops' username is `ops-w1`. Check who you are:

```sh
GITLAB_TOKEN_FILE=$P/ann.token bin/acme-actions whoami
```

**List** set-codeowners for all three projects:

```sh
GITLAB_TOKEN_FILE=$P/ann.token bin/acme-actions -o json
```

**Preview** (changes nothing; shows the diff, the approvers and a confirm token):

```sh
GITLAB_TOKEN_FILE=$P/ann.token bin/acme-actions set-codeowners --entity portal:alpha \
  -i 'pattern=*' -i 'owners=@acme/proxy-maintainers' -o json
```

**Request** as ann. This previews and requests in one step, exits 3 (awaiting approval) and prints
the run's record. Keep `.id`:

```sh
RUN=$(GITLAB_TOKEN_FILE=$P/ann.token bin/acme-actions set-codeowners --entity portal:beta \
  -i 'pattern=/docs/' -i 'owners=@acme/docs @carol' --yes -o json | jq -r .id); echo "$RUN"
```

To request exactly the preview you read, run the same command with
`--confirm <confirm_token>` (from the preview's JSON) instead of `--yes`, within 10 minutes. The
confirm command the Markdown preview prints names `--portal .`; the wrapper accepts that for this
portal only.

**Approve** as dave or erin. On the last approval it pushes to `main` and verifies. With
`--wait`, it exits 0 once `main` has the content, or 1 if it failed:

```sh
GITLAB_TOKEN_FILE=$P/dave.token bin/acme-actions --approve "$RUN" --note "looks right" --wait -o json
GITLAB_TOKEN_FILE=$P/erin.token bin/acme-actions --reject "$RUN" --note "not now" -o json   # or reject
```

**Check a run's status**, then **read its record** (any member of acme-w1):

```sh
GITLAB_TOKEN_FILE=$P/ann.token bin/acme-actions --verify "$RUN" -o json | jq -r .state
GITLAB_TOKEN_FILE=$P/ann.token bin/acme-actions --verify "$RUN" -o json
GITLAB_TOKEN_FILE=$P/ann.token bin/acme-actions --runs -o json          # every run
```

`--runs` also lists each project's own run, which the org run points to as
`verification.child_run`. The run to approve and to check is the id the request printed.

### What a tester should see

| Try | Command (after a request as ann, `$RUN`) | Result |
|---|---|---|
| Invalid inputs | `GITLAB_TOKEN_FILE=$P/ann.token bin/acme-actions set-codeowners --entity portal:gamma -i pattern=docs/ -i owners=@x --yes -o json` (also `#`, spaces, more than 100 characters, 4 owners, an owner without `@`, a double space) | exit 2, `errors`, no run, nothing changed |
| Self-approval | `GITLAB_TOKEN_FILE=$P/ann.token bin/acme-actions --approve "$RUN" -o json` (or a run dave requested, approved by dave) | exit 2, `a requester cannot approve their own run` |
| Outside the release group | the same with `bob.token`, `carol.token` or `ops.token` | exit 2, `user:bob is not one of the pending approvers` |
| Pretending | `… bin/acme-actions --as user:dave --approve "$RUN"`, or a bad token | exit 2, `refused-argument` / `unauthenticated` |
| Membership change | remove erin from `acme-w1/release` in GitLab (`DELETE /groups/acme-w1%2Frelease/members/<erin's id>` with ops' token), then approve as erin | exit 2. Add a user and they can approve on the next command |
| Protected `main` | set `main`'s "Allowed to push and merge" to No one, request, then approve as dave with `--wait` | exit 1, state `failed`, GitLab's `You are not allowed to push code to protected branches`, and `CODEOWNERS` is unchanged |

## Changing it

- **The action** (any project's maintainers): edit `.seikika/` in the project, run
  `seikika validate .seikika && seikika test .seikika && python3 .seikika/tests/set_codeowners_test.py`,
  then commit to `main`. Keep the standard files the same in every project, unless a project wants
  its own.
- **The approvers**: change `acme-w1/release`'s membership in GitLab. Nothing here changes.
- **A new project**: add it to `federation.children` in `org/seikika.yaml`, and copy `.seikika/`
  into it with its own `seikika.yaml`.
- **The service**: `OWNER_TOKEN_FILE=$P/ops.token bin/setup-service` creates it if it is missing.
  A new service directory has a new signing key. Paste the key it prints into each project's
  `acceptApprovalsFrom`.
