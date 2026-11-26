# Handoff: `mark-deprecated`

`mark-deprecated` is a Seikika self-service action. Each of alpha, beta and gamma declares it in
its own repository, on its origin's `main`. One org portal, `org/seikika.yaml`, offers all three
from one place and is served on `127.0.0.1:7610`. That portal validates inputs, previews the
diff, holds approvals, runs the change in its clone of the repository, pushes to `main` and checks
the result there. All of this is built into Seikika. The only glue I wrote is the shell command
that writes `DEPRECATED.md`, plus `org/env.sh`, which sets three environment variables.

The three requested runs succeeded. ann requested each run and bob approved it.

| Repository | Org run id | Commit on origin `main` | State |
|---|---|---|---|
| alpha | `run_2ftn9wrqarhq` | `7b4ed7f` | succeeded |
| beta | `run_2ftnh3t9x2po` | `0d382ad` | succeeded |
| gamma | `run_2ftnhbcwbxhh` | `49fbba1` | succeeded |

The server also holds a fourth run, `run_2ftno5san8jn` on alpha. It is my live check of a
rejected push: the run failed and `main` did not change (see R5).

## Files

| Path | What it is |
|---|---|
| `alpha/.seikika/seikika.yaml` (same in `beta/`, `gamma/`) | The repository's portal: its name, its owner (`group:<repo>-maintainers`) and `include: [actions/*.yaml]` |
| `alpha/.seikika/actions/mark-deprecated.yaml` (same in `beta/`, `gamma/`) | The action: inputs schema, approvers, precondition, the `change` executor and the pinned org approval key. Byte-identical in all three repositories |
| `alpha/.seikika/tests/mark-deprecated.test.yaml` (same in `beta/`, `gamma/`) | 15 test cases, run by `seikika test .seikika` |
| `org/seikika.yaml` | The org portal: `federation.children` lists `../origins/{alpha,beta,gamma}.git` |
| `org/env.sh` | Sets `PATH`, `SEIKIKA_STATE_DIR` and `SEIKIKA_URL` |
| `state/` (`/tmp/actions-ab/r6/seikika-1/state`) | Server state (not in git): `served/org/signing-key`, `served/org/runs/<run id>.json` (org run records), `runs/<run id>.json` (the repositories' child run records), `served/org/serve.log`, and `federation/` (the org's clones of each origin) |

All of these, except `state/`, are committed and pushed to their origins' `main`.

## Requirements

| # | How it is met |
|---|---|
| R1 Owned by the repository | The definition is in `<repo>/.seikika/actions/mark-deprecated.yaml` and `<repo>/.seikika/seikika.yaml`, committed to `origins/<repo>.git` `main`. The org portal holds no copy. Before every list, preview and run, it fetches each origin's `main` into `state/federation/`. A maintainer changes the action by committing to their repository. The child also checks that the approval was signed with the org key pinned in its own file (`federation.acceptApprovalsFrom`), so the repository decides which portal may approve its runs. |
| R2 Validated | `inputs` is a JSON Schema: both inputs `required`, `additionalProperties: false`, `successor` with `pattern: ^[a-z0-9-]+/[a-z0-9-]+$`, and `reason` with `minLength: 20, maxLength: 200`. Seikika checks it before it previews and refuses with exit 2 and `422 invalid-inputs`. A precondition, `!inputs.successor.endsWith('/' + entity.name)`, blocks a repository from naming itself as successor ("another repository"). |
| R3 Previewed | `seikika run mark-deprecated …` without `--yes` previews the run and changes nothing. It shows the exact diff of `DEPRECATED.md`, the push to `main`, the approvers and any blockers, and returns a confirm token. The same `effects` diff is stored in the run record. Before it commits, the change is checked against the confirmed preview, and a mismatch fails the run. |
| R4 Approved | `approval: "[['user:ann', 'user:bob']]"` means either ann or bob must approve, and the requester never counts. In the live checks, ann approving her own run returned `403 not-an-approver: a requester cannot approve their own run`. carol got `403 not-an-approver: user:carol is not one of the pending approvers`. Both exit 2, and the run stays `awaiting_approval`. Nothing is committed before an approval. |
| R5 Done means observed | `executor: {type: change, land: push, base: main}`. After pushing, Seikika fetches `origin/main` and reports `succeeded` only if its commit is on `main` and every changed file's blob matches the preview. The record shows this in `verification.checked` and `verification.files[].matches`. A rejected push fails the run with the origin's message, exit 1. Live check: with a `pre-receive` hook on `origins/alpha.git`, `run_2ftno5san8jn` went to `failed` with "the push to origin main was refused, so nothing landed … main is protected", and `main` did not change. |
| R6 One place for the org | `org/seikika.yaml` lists the three origins. From `org/`, `seikika run --as ann` lists `mark-deprecated` on `portal:alpha`, `portal:beta` and `portal:gamma`. `--entity portal:<repo>` previews or requests a run on any of them, and `seikika run --runs` lists every run. Nobody needs to enter a repository. |
| R7 Audited | Each record (`seikika run --verify <id> -o json`, kept in `state/served/org/runs/<id>.json`) has `requested_by`, `inputs`, `approval.approved_by`, `state`, `outputs.commit`, `verification`, and `events[]`. Each event has an `at` timestamp: previewed and confirmed (by the requester), awaiting_approval, approved (by the approver, with the note), dispatched, running, then succeeded or failed (with the error). The commit on `main` carries the `Seikika-Run:`, `Requested-By:` and `Approved-By:` trailers. |
| R8 Usable by agents | Every step is a non-interactive `seikika run` command. `-o json` prints machine-readable output and exit codes report the state: 0 succeeded, 1 failed, 2 invalid or refused, 3 awaiting approval, 4 unverified. `SEIKIKA_URL` and `SEIKIKA_AS` can replace `--url` and `--as`. |

## Commands

Run these from any directory; the examples use `org/`. Identity is `--as <name>`, which the server
accepts because it listens on loopback (dev sign-in). It records the identity as `user:<name>`.

```sh
cd /tmp/actions-ab/r6/seikika-1/org
. ./env.sh          # PATH, SEIKIKA_STATE_DIR=/tmp/actions-ab/r6/seikika-1/state, SEIKIKA_URL=http://127.0.0.1:7610
```

**Start the server** (it must be running for request, approve and status checks):

```sh
seikika serve /tmp/actions-ab/r6/seikika-1/org --listen 127.0.0.1:7610 --detach
# prints: approval key: ed25519:97921ac0a0f8f3fb0739689404fbd6b1b7e95eb21e44cb9de527c5607520a6d4
# That key must match the one pinned in each repository. It will, as long as SEIKIKA_STATE_DIR is
# the one above.
```

**Stop it when done:** `seikika serve /tmp/actions-ab/r6/seikika-1/org --stop`

**List the actions** (all three repositories):

```sh
seikika run --as ann -o json
seikika actions portal:beta --as ann -o json     # one repository's actions
```

**Preview** (changes nothing; exit 0, or 2 if the inputs are invalid):

```sh
seikika run --as ann mark-deprecated --entity portal:alpha \
  -i successor=acme/new-thing -i "reason=Explain in 20 to 200 characters why." -o json
```

**Request as ann** (exit 3, `state: awaiting_approval`; the run id is `.id`):

```sh
RUN=$(seikika run --as ann mark-deprecated --entity portal:alpha \
  -i successor=acme/new-thing -i "reason=Explain in 20 to 200 characters why." --yes -o json \
  | python3 -c 'import json,sys; print(json.load(sys.stdin)["id"])')
echo $RUN
```

**Approve as bob** (exit 0 once `DEPRECATED.md` is observed on `main`, exit 1 if it fails):

```sh
seikika run --as bob --approve $RUN --note "checked the preview" --wait -o json
```

**Check the status, read the record, list all runs:**

```sh
seikika run --as ann --verify $RUN -o json | python3 -c 'import json,sys; print(json.load(sys.stdin)["state"])'
seikika run --as ann --verify $RUN -o json      # full record: requested_by, approval.approved_by, inputs, events[].at, verification
seikika run --as ann --runs -o json
```

To reject instead of approving: `seikika run --as bob --reject $RUN --note "why"`.

## Testing the refusals

| Test | Command | Expected |
|---|---|---|
| Invalid successor | `seikika run --as ann mark-deprecated --entity portal:alpha -i successor=Acme/X -i "reason=Long enough reason text here." -o json` | exit 2, `422 invalid-inputs … does not match` |
| Reason too short or too long | `-i reason=short`, or 201 characters | exit 2, `422 invalid-inputs … shorter than 20` / `longer than 200` |
| Missing or unknown input | omit `-i reason=…`, or add `-i extra=x` | exit 2, `422 invalid-inputs` |
| Self as successor | `-i successor=acme/alpha` on `portal:alpha` | preview lists a precondition blocker; `--yes` exits 2 and creates no run |
| Self-approval | request as ann, then `seikika run --as ann --approve $RUN` | exit 2, `403 not-an-approver: a requester cannot approve their own run` |
| carol approves | `seikika run --as carol --approve $RUN` | exit 2, `403 not-an-approver: user:carol is not one of the pending approvers` |
| Rejected push | see below | approve exits 1, `state: failed`, origin's message in the last event; `main` unchanged |

Rejected push: make the origin refuse pushes, then request with **new** inputs. A run whose content
`main` already has succeeds without a commit.

```sh
printf '#!/bin/sh\necho "main is protected" >&2\nexit 1\n' > /tmp/actions-ab/r6/seikika-1/origins/alpha.git/hooks/pre-receive
chmod +x /tmp/actions-ab/r6/seikika-1/origins/alpha.git/hooks/pre-receive
RUN=$(seikika run --as ann mark-deprecated --entity portal:alpha -i successor=acme/other-thing \
  -i "reason=Testing that a rejected push fails the run." --yes -o json | python3 -c 'import json,sys; print(json.load(sys.stdin)["id"])')
seikika run --as bob --approve $RUN --wait -o json; echo "exit $?"    # exit 1, state failed
git -C /tmp/actions-ab/r6/seikika-1/origins/alpha.git show main:DEPRECATED.md   # unchanged
rm /tmp/actions-ab/r6/seikika-1/origins/alpha.git/hooks/pre-receive
```

The same cases also run without a server, in a scratch copy of each repository and its origin, so
the real origin is untouched. Run `cd /tmp/actions-ab/r6/seikika-1/alpha && seikika validate
.seikika && seikika test .seikika`, and the same in `beta` and `gamma`; each passes 15 of 15. The
cases cover:

- ann requests and bob approves, with the exact content;
- bob requests and ann approves;
- invalid successors (not `owner/name`, upper case);
- reasons of 9, 19 and 201 characters refused, and of 20 and 200 accepted;
- a missing reason, and an unknown input;
- the repository as its own successor;
- approvals by ann (her own run) and carol refused;
- carol may request, but only bob's approval counts;
- a rejected push fails with the origin's message.

## Notes

- `DEPRECATED.md` ends with a newline after `<reason>`: `# Deprecated: use <successor>\n\n<reason>\n`.
- Commits on `main` are authored by `Seikika <seikika@localhost>`. Who did what is in the
  trailers and the run record.
- Anyone signed in may request a run, carol included. Only ann or bob, and never the requester,
  may approve one.
