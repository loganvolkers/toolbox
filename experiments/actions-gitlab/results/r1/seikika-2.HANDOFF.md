# HANDOFF: `set-codeowners` on Seikika + GitLab

`set-codeowners` is a Seikika self-service action that alpha, beta and gamma each declare in their
own `.seikika/`. This project (`acme-w13/org`) is the org portal that lists all three and holds the
approvals. **Nothing has to be running**: every step is a `seikika run` command in this
checkout. There is no server.

- Who you are comes from your own GitLab token (`--identity-token-file`). Seikika asks GitLab who
  owns it, and `--as` alone is refused.
- Who may approve is read live from GitLab: the direct members of `acme-w13/release`.
- An approved run is committed and pushed to the project's `main`. It is `succeeded` only after
  Seikika fetches `origin/main` and finds the commit there with the previewed `CODEOWNERS` content.

Runs done so far (requested by ann):

| Project | Run | Approved by | State | Commit on `main` |
|---|---|---|---|---|
| alpha | `run_2g07srxq0yn2` | dave | succeeded | `999fd1d` (`* @acme/proxy-maintainers`) |
| beta | `run_2g085ch7vywy` | erin | succeeded | `8cd000f` (`/docs/ @acme/docs @carol`) |
| gamma | `run_2g085ybz9fe7` | erin | succeeded | `ec0398f` (`/graphify/security.py @acme/security`) |

There were also three check runs. `run_2g08gkwnbfgv` was approved by carol while she was briefly a
direct member of the release group, and **failed** because alpha's `main` was set to
"No one" may push or merge. `run_2g08lqynr9ke` and `run_2g08p7l3y4at` were rejected after the
membership and token checks. `run_2g09r2g7o3vu` repeated gamma's rule and succeeded with
`outputs.changed: false` and no commit. Alpha's protection was restored to Maintainers and carol
was removed from the release group afterwards.

## Files

| Path | What it is |
|---|---|
| `acme-w13/{alpha,beta,gamma}`: `.seikika/seikika.yaml` | The project's portal: its name, and `include: [actions/*.yaml]` |
| `acme-w13/{alpha,beta,gamma}`: `.seikika/actions/set-codeowners.yaml` | The action: input schema, approval rule, executor, and the org key it accepts approvals from |
| `acme-w13/{alpha,beta,gamma}`: `.seikika/scripts/set-codeowners.py` | The edit itself: replace the first line whose first field is the pattern, or append; final newline; re-validates its inputs |
| `acme-w13/{alpha,beta,gamma}`: `.seikika/tests/set-codeowners.test.yaml` | `seikika test` cases: success, invalid inputs, self-approval and outsiders refused, a rejected push fails |
| `acme-w13/{alpha,beta,gamma}`: `.seikika/scripts/test_set_codeowners.py` | Unit tests of the exact `CODEOWNERS` edit |
| `acme-w13/org`: `seikika.yaml` | The org portal: `federation.children` (the three projects), the `gitlab` connector, the `Group` kind and `authz`, and `forges` with `identity: true` |
| `acme-w13/org`: `groups.fga` | The authorization model: `group#member` |
| `acme-w13/org`: `env.sh` | Sets `HOME`, `PATH`, `GITLAB_TOKEN` (the read-only token below) and `P` (the people's token directory) |
| `/tmp/actions-gitlab/r1/seikika-2/portal-secrets/gitlab-read.token` | Group access token `r1-seikika-2-org-portal-read` on `acme-w13`: `read_api`, Reporter. It is used only to read groups and members |
| `/tmp/actions-gitlab/r1/seikika-2/home/.local/state/seikika/` | Seikika's state: `served/org/signing-key` (the org's approval key), `federation/org-*/` (clones of the projects), `runs/run_*.json` (run records) |

The three projects' action files are byte-for-byte identical; only `.seikika/seikika.yaml` differs.
Each project pins the org portal's key in `federation.acceptApprovalsFrom`:
`ed25519:d5ed162db5904966351b82bc6896cffa4824488660fa26cb1b26281b10c73185`. The private half is in
the state directory above. If that directory is lost, `seikika run` here makes a new key. Paste the
new key into each project's action file, or the projects will refuse the org's approvals.

## How each requirement is met

| # | How | Where |
|---|---|---|
| R1 Owned by the project | The action's definition, script and tests are committed to each project's `main`, under protection that lets only its Maintainers push or merge. The org portal re-reads each project's `main` before every list, preview and run, so a change a project's maintainers merge takes effect on the next run. The org portal holds no copy of the action | `<project>/.seikika/actions/set-codeowners.yaml`, `<project>/.seikika/scripts/set-codeowners.py`, `<project>/.seikika/tests/set-codeowners.test.yaml` |
| R2 Validated | The inputs are a JSON Schema with `required`, `additionalProperties: false` and regexes. `pattern` is `^[/*][^\s#]*$` with `maxLength: 100`. `owners` is `^@[A-Za-z0-9_.-]+(/[A-Za-z0-9_.-]+)*( @…){0,2}$`. Seikika checks them before any preview and refuses with exit 2 and `errors`, so no run is created. The script checks them again and exits 2 before it writes anything | `set-codeowners.yaml` (`inputs`), `set-codeowners.py` (`PATTERN`, `OWNERS`) |
| R3 Previewed | `seikika run set-codeowners …` without `--yes` prints the effects: the `CODEOWNERS` diff against current `main`, and "push branch:main". The confirm token and the record of a run (`effects`) keep the same diff. A change that would differ from the preview fails before it commits | org `seikika.yaml`; output of the preview command below |
| R4 Approved by the release group | `approval: "['group:acme-w13/release']"`. The requester never counts (`a requester cannot approve their own run`, exit 2), and anyone not shown to be in the group is refused (`is not one of the pending approvers`, exit 2). Identity is the owner of the GitLab token in `--identity-token-file` (`forges[].identity: true`). `--as` without a token is refused, and an `--as` that disagrees with the token is refused. The project accepts only approvals signed with the org portal's key | `<project>/.seikika/actions/set-codeowners.yaml` (`approval`, `federation.acceptApprovalsFrom`), org `seikika.yaml` (`forges`) |
| R5 Done means observed | `executor: {type: change, land: push, base: main}`. After pushing, Seikika fetches `origin/main`. The run is `succeeded` only if the commit is on it and every changed file (`CODEOWNERS`) has the previewed blob (`verification.child.checked`, `files[].matches`). A push GitLab refuses, such as on a fully protected `main`, makes the run `failed` with GitLab's message (exit 1), and nothing is retried. A rule `main` already has commits nothing and succeeds (`outputs.changed: false`) | `set-codeowners.yaml` (`executor`); `run_2g08gkwnbfgv` is the protected-`main` failure |
| R6 One place | This directory's `seikika.yaml` lists the three projects as `federation.children`, offered as `portal:alpha`, `portal:beta` and `portal:gamma`. `seikika run` here lists all three actions and requests a run on any of them | `acme-w13/org`: `seikika.yaml` |
| R7 Audited | Each run's record has `requested_by`, `approval.approved_by`, `approval.rejected_by`, `inputs`, `state`, `effects`, `outputs.commit`, `verification`, and `events[]`. Each event has `at` (UTC), `by` and `state`: previewed, confirmed, awaiting_approval, approval_refused (with who and why), approved, dispatched, running, succeeded/failed/rejected. Records are kept as `runs/run_*.json` in the state directory. The commit on `main` also carries `Seikika-Run`, `Requested-By` and `Approved-By` trailers | `~/.local/state/seikika/runs/` (with this `HOME`), `git log` on each project |
| R8 Usable by agents | Every step is one non-interactive `seikika run` command with `-o json`. Exit codes: 0 succeeded, 1 failed, 2 invalid, refused or blocked, 3 awaiting approval, 4 unverified | commands below |
| R9 The group is GitLab's | The `gitlab` connector reads `acme-w13`'s groups and each group's **direct** members (Developer or above) on every command, and `authz` turns them into `group:acme-w13/release` membership. Adding or removing a direct member in GitLab changes who may approve, for all three projects, on the next command. Inherited members (ann, bob, carol, ops-w13 via `acme-w13`) are not members. Without `GITLAB_TOKEN` the connector fails, and approving or rejecting fails closed (exit 2) | org `seikika.yaml` (`connectors`, `kinds`, `authz`), `groups.fga` |

## Commands

There is nothing to start. Every command runs from a checkout of this project on `main`.

```sh
export HOME=/tmp/actions-gitlab/r1/seikika-2/home
cd /tmp/actions-gitlab/r1/seikika-2/org && git pull -q origin main
. ./env.sh          # HOME, PATH, GITLAB_TOKEN (read-only group token), P=<people token dir>
```

List the actions on all three projects. This needs no identity:

```sh
seikika run -o json
```

Preview, as ann. This prints the diff and changes nothing (exit 0):

```sh
seikika run --identity-token-file $P/ann.token set-codeowners --entity portal:alpha \
  -i 'pattern=/docs/' -i 'owners=@acme/docs @carol' -o json
```

Request, as ann. This records the preview, then waits for approval (exit 3). The run id is `.id`:

```sh
RUN=$(seikika run --identity-token-file $P/ann.token set-codeowners --entity portal:alpha \
  -i 'pattern=/docs/' -i 'owners=@acme/docs @carol' --yes -o json | jq -r .id); echo "$RUN"
```

Approve, as dave or erin. This pushes, then verifies on `main` (exit 0 succeeded, 1 failed):

```sh
seikika run --identity-token-file $P/dave.token --approve "$RUN" --note "ok" --wait -o json
# or reject:  seikika run --identity-token-file $P/erin.token --reject "$RUN" --note "why" -o json
```

Check a run's status:

```sh
seikika run --verify "$RUN" -o json | jq -r .state
```

Read a run's record. List every run:

```sh
seikika run --verify "$RUN" -o json
seikika run --runs -o json
```

Each request appears in `--runs` twice, as the org's run and as the project's child run
(`verification.child_run`). Read the org's run, the id that `--yes` printed.

### What the checks look like

Expected exit codes and messages:

- **Invalid inputs** (for example `-i 'pattern=docs'`, `'/a b'`, `'/a#b'`, 101 characters,
  `-i 'owners=@a @b @c @d'`, `'@a  @b'`, `'carol'`): exit 2, `errors` lists each failed field, and
  no run is created.
- **Self-approval**: approving with `$P/ann.token` on ann's run gives exit 2,
  `a requester cannot approve their own run`.
- **Approval from outside the release group**: approving with `$P/bob.token`, `$P/carol.token` or
  `$P/ops.token` gives exit 2, `user:<name> is not one of the pending approvers
  ["group:acme-w13/release"]`. `--as dave` without a token gives exit 2. `--as dave` with
  someone else's token gives exit 2, and `--as` disagrees with the token.
- Every refused approval is recorded as an `approval_refused` event, with who and when.
- **Membership change** (as ops-w13, an Owner): add carol directly to the release group, and her
  next approval counts. Remove her, and the next one is refused:

```sh
curl -s -H "PRIVATE-TOKEN: $(cat $P/ops.token)" -X POST "http://127.0.0.1:8929/api/v4/groups/acme-w13%2Frelease/members?user_id=4&access_level=30"
curl -s -H "PRIVATE-TOKEN: $(cat $P/ops.token)" -X DELETE "http://127.0.0.1:8929/api/v4/groups/acme-w13%2Frelease/members/4"
```

- **Protected `main`**: set the project's `main` to "No one" may push or merge. The approval is
  accepted, the push is refused, and the run is `failed` (exit 1). The error holds GitLab's
  `You are not allowed to push code to protected branches`, and `main` is unchanged.

## Changing the action

A project's maintainers edit the files under `.seikika/` in that project, then run:

```sh
seikika validate .seikika && seikika test .seikika
python3 -m unittest discover -s .seikika/scripts -p 'test_*.py'
```

Both run in a scratch copy, so nothing reaches GitLab. Then merge to `main`. The org portal picks
up the change on the next command. To offer another project, add its `git` URL under
`federation.children` in this project's `seikika.yaml`, then copy the action files into it.
