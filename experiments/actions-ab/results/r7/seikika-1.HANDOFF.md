# Handoff: `mark-deprecated` on alpha, beta and gamma

`mark-deprecated` is a Seikika self-service action. Each repository declares it in its own
`.seikika/`, on its origin's `main`. The org portal in this directory lists the three repositories
and is where runs are requested, approved and checked. **Nothing needs to be running.** The org
portal works from its checkout with no server, so there is nothing to start or stop.

Status: ann requested a run on each repository and bob approved each one. All three runs were
reported `succeeded`, and `DEPRECATED.md` is on each origin's `main`, byte for byte:

| Repository | Org run (read this one) | Commit on `origins/<repo>.git` `main` |
|---|---|---|
| alpha | `run_2fu4y0jzhury` | `e1e70d3` |
| beta | `run_2fu54ekvubnu` | `19155bd` |
| gamma | `run_2fu54j04mtz4` | `3fe7d40` |

`--runs` also lists a second run per repository (for example `run_2fu50tl9itr4` for alpha). That
is the repository's own run, which the org run dispatched; the org run's record names it in
`verification.child_run`. It also lists `run_2fu5a83e8rf9`, a live check of a rejected push
(see R5): it `failed`, and alpha's `main` was not changed.

## Files

| Path | What it is |
|---|---|
| `<repo>/.seikika/seikika.yaml` | The repository's portal: its name, its owner (`group:<repo>-maintainers`) and `include: [actions/*.yaml]` |
| `<repo>/.seikika/actions/mark-deprecated.yaml` | The action: inputs, approval rule, precondition, executor, and the org key the repository accepts approvals from |
| `<repo>/.seikika/tests/mark-deprecated.test.yaml` | Eleven test cases: success, invalid inputs, self-approval and carol, a rejected push |
| `org/seikika.yaml` | The org portal: `federation.children` lists `../origins/{alpha,beta,gamma}.git` |
| `org/env.sh` | Sets `PATH` and `SEIKIKA_STATE_DIR` |
| `/tmp/actions-ab/r7/seikika-1/.seikika-state/` | `SEIKIKA_STATE_DIR`: the org signing key (`served/org/signing-key`), the org's clones of the repositories (`federation/`) and the run records (`runs/<run id>.json`) |

`<repo>` is `/tmp/actions-ab/r7/seikika-1/{alpha,beta,gamma}`. The action file and the test file
are the same, byte for byte, in all three repositories. `entity.name` and the tests' `{portal}`
stand for the repository that holds them.

## How each requirement is met

| # | How |
|---|---|
| R1 Owned by the repository | The definition is `<repo>/.seikika/actions/mark-deprecated.yaml`, committed to each origin's `main` ("Add the mark-deprecated action…"; alpha `cbf189f`, beta `fd518a0`, gamma `3a0d026`). The org portal holds no action definitions. It reads each child's `.seikika/` from the origin's `main` again before every list, preview and run, so a change a repository's team pushes takes effect on the next request. The approval rule `[['user:ann', 'user:bob']]` is in the same file, so the team owns who approves |
| R2 Validated | `inputs` in the action file is a JSON Schema with `required: [successor, reason]`, `additionalProperties: false`, `successor.pattern: ^[a-z0-9-]+/[a-z0-9-]+$` and `reason` `minLength: 20` and `maxLength: 200`. A precondition, `!inputs.successor.endsWith('/' + entity.name)`, blocks a repository being named as its own successor. Seikika checks all of this before the preview, and records no run: an invalid input exits 2 with the errors, and a failed precondition exits 1 with `mark-deprecated is blocked: precondition not met`. Covered by test cases "a successor that is not owner/name…", "…upper case…", "a reason under/over…", "a missing reason…", "an input the action does not declare…" and "the repository cannot be its own successor" |
| R3 Previewed | `seikika run mark-deprecated …` without `--yes` or `--confirm` prints the effects: the exact diff of `DEPRECATED.md` and "push branch:main, one commit, pushed once approved (never forced)". It also prints the approvers and a confirm token, and changes nothing. `--yes` records that preview as the run's `previewed` event, and its `effects` stay on the record. When the run executes, Seikika refuses to commit a change that differs from the confirmed preview |
| R4 Approved | `approval: "[['user:ann', 'user:bob']]"` in the action file: either one, and the requester never counts. The org portal holds the approval and signs it with its key, and each repository accepts only that key (`federation.acceptApprovalsFrom`). ann approving her own run is refused with `a requester cannot approve their own run`. carol is refused with `user:carol is not one of the pending approvers`. Both exit 2, and the run stays `awaiting_approval`. Checked live on alpha and beta, and by the test case "ann cannot approve her own run, and carol is not an approver" |
| R5 Done means observed | `executor: {type: change, land: push, base: main}`. After pushing, Seikika fetches `origin/main` and reports `succeeded` only if its commit is there and every file it changed has the blob the preview showed (`verification.child.files[].matches`). A rejected push fails the run (exit 1) with the origin's message. Covered by test case "a push the origin rejects fails the run with the origin's message", and checked live with a `pre-receive` hook on `origins/alpha.git` (run `run_2fu5a83e8rf9`) |
| R6 One place for the org | `org/seikika.yaml` federates the three origins. From `org/`, `seikika run` lists `mark-deprecated` on `portal:alpha`, `portal:beta` and `portal:gamma`, and `--entity portal:<repo>` previews, requests, approves and checks a run on any of them. The org portal runs each action in its own clone (`federation.runner: {type: local}` in the action file) |
| R7 Audited | `seikika run --verify <run id> -o json` (stored at `$SEIKIKA_STATE_DIR/runs/<run id>.json`) shows `requested_by`, `inputs`, `approval.approved_by` and `approval.satisfied`, `effects`, `state`, `outputs.commit` and `verification`. It also has `events`, each with `at` (UTC) and `by`: `previewed`, `confirmed`, `awaiting_approval`, `approved` (with the note), `dispatched`, `running`, `executor_succeeded`, then `succeeded` or `failed`. The commit on `main` carries `Seikika-Run`, `Requested-By` and `Approved-By` trailers |
| R8 Usable by agents | Every step is one non-interactive `seikika run` command that takes `-o json`. Exit codes: 0 succeeded, 1 failed, 2 invalid or refused, 3 awaiting approval, 4 unverified. `SEIKIKA_AS` can stand in for `--as` |

## Commands

Run everything from the org portal's directory, with the environment set:

```sh
cd /tmp/actions-ab/r7/seikika-1/org
. ./env.sh          # PATH and SEIKIKA_STATE_DIR; nothing to start
```

**List** every repository's actions, with their inputs and approvers:

```sh
seikika run --as ann -o json
```

**Preview.** This prints the diff and the approvers, and changes nothing (exit 0):

```sh
seikika run --as ann mark-deprecated --entity portal:alpha \
  -i successor=acme/dependabot-proxy-v2 \
  -i 'reason=Replaced by the v2 proxy, which supports OIDC registries.' -o json
```

**Request as ann.** This exits 3 (awaiting approval) and prints the record, including `id`:

```sh
RUN=$(seikika run --as ann mark-deprecated --entity portal:alpha \
  -i successor=acme/dependabot-proxy-v2 \
  -i 'reason=Replaced by the v2 proxy, which supports OIDC registries.' \
  --yes -o json | python3 -c 'import json,sys; print(json.load(sys.stdin)["id"])'); echo $RUN
```

**Approve as bob.** The run executes, and the command exits 0 only once `main` has the file:

```sh
seikika run --as bob --approve "$RUN" --note "checked" --wait -o json
```

`--reject "$RUN"` rejects it instead.

**Check a run's status, and read its record:**

```sh
seikika run --as ann --verify "$RUN" -o json     # .state, .requested_by, .approval.approved_by, .inputs, .events[].at
seikika run --as ann --runs -o json              # every run
```

### Negative checks

```sh
# Invalid inputs: exit 2, errors listed, no run recorded, nothing changes
seikika run --as ann mark-deprecated --entity portal:beta -i successor=Not_Valid -i reason=short --yes
# Its own successor: blocked by the precondition, exit 1, no run recorded
seikika run --as ann mark-deprecated --entity portal:beta -i successor=acme/beta -i 'reason=Its own successor is blocked here.' --yes

# Self-approval and carol: each exits 2, and the run stays awaiting_approval
RUN=$(seikika run --as ann mark-deprecated --entity portal:beta -i successor=acme/store-two \
  -i 'reason=A second deprecation notice for testing.' --yes -o json | python3 -c 'import json,sys; print(json.load(sys.stdin)["id"])')
seikika run --as ann   --approve "$RUN"; echo "exit $?"     # a requester cannot approve their own run
seikika run --as carol --approve "$RUN"; echo "exit $?"     # user:carol is not one of the pending approvers
seikika run --as ann --verify "$RUN" -o json | python3 -c 'import json,sys; print(json.load(sys.stdin)["state"])'   # awaiting_approval

# Rejected push: the run fails (exit 1) with the origin's message, and main does not move
printf '#!/bin/sh\necho "main is protected" >&2\nexit 1\n' > ../origins/beta.git/hooks/pre-receive
chmod +x ../origins/beta.git/hooks/pre-receive
seikika run --as bob --approve "$RUN" --wait; echo "exit $?"   # failed
rm ../origins/beta.git/hooks/pre-receive
git -C ../origins/beta.git show main:DEPRECATED.md              # unchanged
```

Without the hook, that last approval would replace beta's `DEPRECATED.md` with the new inputs.
Reject the run (`seikika run --as bob --reject "$RUN"`) if you don't want that. A request whose
content `main` already has succeeds without a commit (`outputs.changed: false`).

The same cases run without touching the real origins, in a scratch copy of each repository:

```sh
for r in alpha beta gamma; do (cd ../$r && seikika validate .seikika && seikika test .seikika); done   # 11 passed in each
```

## Known limits

- **A blocked precondition exits 1, not 2.** Seikika's help lists 2 for "invalid or refused", but
  a request blocked by a precondition exits 1, the code for "failed". It is still refused before
  anything changes, and no run is recorded.

- **Refused approvals exit 0 with `-o json`.** In plain-text mode, a refused `--approve` (by the
  requester or by carol) exits 2. With `-o json`, it prints `{"error": …}` but exits 0. A script
  that approves with `-o json` should check for an `error` key, or approve without `-o json`
  and then read the state with `--verify -o json`. Either way, the run is not approved.
- **Refused approval attempts are not written into the run's `events`.** The record shows who did
  approve, not who tried and was refused.
- Identity is whatever `--as` says, as this sandbox allows. The rules on who may approve are
  enforced, but nothing authenticates the person.
- `seikika serve /tmp/actions-ab/r7/seikika-1/org --listen 127.0.0.1:7810 --detach` would serve
  the same portal (UI, REST, MCP), with the same commands plus `--url http://127.0.0.1:7810`. A
  served portal keeps its own runs, so approve a run where it was requested. Stop it with
  `seikika serve /tmp/actions-ab/r7/seikika-1/org --stop`. Nothing here needs it, and none is running.
