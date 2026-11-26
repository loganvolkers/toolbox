# mark-deprecated: handoff

`mark-deprecated` is a Seikika self-service action. Each repository declares it in its own
`.seikika/`, and the org portal in this directory offers all three, holds approvals, runs them
and records them. No glue code was needed: inputs, preview, approval, push, verification and the
audit record are all Seikika (0.1.1, `/tmp/actions-ab/r6/seikika-2/bin`).

All three runs requested by ann and approved by bob were reported `succeeded`, and each origin's
`main` now has the exact `DEPRECATED.md`:

| Repository | Run | State | Commit on origin `main` |
|---|---|---|---|
| alpha | `run_2ftrrglc3vlx` | succeeded | `c1a0276` (`Requested-By: user:ann`, `Approved-By: user:bob`) |
| beta | `run_2ftrw9w7le7q` | succeeded | see `--verify` |
| gamma | `run_2ftrxd6l202v` | succeeded | see `--verify` |

`run_2fts2j47nl0j` (alpha, `failed`) is a rehearsal of a rejected push (see "Rejected push" below).
It changed nothing on `main`.

## Files

| Path | What it is |
|---|---|
| `<repo>/.seikika/seikika.yaml` | The repository's portal: its name, owner and `include: [actions/*.yaml]` |
| `<repo>/.seikika/actions/mark-deprecated.yaml` | The action: inputs schema, approvers, precondition, executor, and the org key it accepts approvals from. Byte for byte the same in alpha, beta and gamma |
| `<repo>/.seikika/tests/mark-deprecated.test.yaml` | 15 test cases run by `seikika test .seikika`. Byte for byte the same in all three |
| `org/seikika.yaml` | The org portal: `federation.children` lists `../origins/{alpha,beta,gamma}.git` |
| `/tmp/actions-ab/r6/seikika-2/.seikika-state/` | Server state (`SEIKIKA_STATE_DIR`): the approval signing key, the clones of each repository and the run records |
| `.seikika-state/served/org/runs/<run id>.json` | One JSON record per run |
| `.seikika-state/served/org/signing-key` | The key the org signs approvals with. The repositories pin its public half, so **always start the server with this `SEIKIKA_STATE_DIR`**; another state directory makes a new key, which the repositories refuse (`child-refuses-approvals`) |

`<repo>` is `/tmp/actions-ab/r6/seikika-2/{alpha,beta,gamma}`; each is committed and pushed to
`origins/<repo>.git` `main`.

## How each requirement is met

| # | How | Where |
|---|---|---|
| R1 Owned by the repository | The action is defined in each repository's `.seikika/` and committed to its origin's `main` (commit "Add the mark-deprecated self-service action (.seikika/)"). The org portal holds no action definitions: it clones each origin's `main` and reads it again before every list, preview or run. A team that changes its file and pushes changes what the org offers | `<repo>/.seikika/actions/mark-deprecated.yaml`, `org/seikika.yaml` |
| R2 Validated | `inputs` is a JSON Schema: both inputs required, `additionalProperties: false`, `successor` matches `^[a-z0-9-]+/[a-z0-9-]+$`, `reason` has `minLength: 20` and `maxLength: 200`. A precondition also refuses the repository as its own successor (`acme/alpha` on alpha). Invalid inputs are refused before a preview exists: `422 invalid-inputs`, exit 2 | `mark-deprecated.yaml` `inputs` and `preconditions` |
| R3 Previewed | `seikika run … mark-deprecated …` without `--yes` prints the effects (the `DEPRECATED.md` diff and the push) and changes nothing. `--yes` records the preview with the request, and the record's `effects` keep it. At run time, a commit that would differ from the preview fails before it is pushed | `executor` in `mark-deprecated.yaml` |
| R4 Approved | `approval: "[['user:ann', 'user:bob']]"`: one of ann or bob, and the requester never counts. ann approving her own run gets `403 not-an-approver: a requester cannot approve their own run`. carol gets `403 not-an-approver: user:carol is not one of the pending approvers`. Both exit 2 and the run stays `awaiting_approval`. The org signs approvals, and each repository accepts only the key pinned in `federation.acceptApprovalsFrom` | `mark-deprecated.yaml` `approval`, `federation` |
| R5 Done means observed | `executor: {type: change, land: push, base: main}`. After pushing, Seikika fetches `origin/main` and checks that the commit is on it and that every file it changed has the blob the preview showed. Only then is the run `succeeded`; the record's `verification.child.files[].matches` shows it. A rejected push fails the run with the origin's message (exit 1), and nothing is retried | `mark-deprecated.yaml` `executor`; run records |
| R6 One place for the org | `org/seikika.yaml` federates the three origins. Served, it lists `mark-deprecated` on `portal:alpha`, `portal:beta` and `portal:gamma`, and previews, requests, approves and runs any of them from here | `org/seikika.yaml` |
| R7 Audited | Each run record has `requested_by`, `inputs`, `approval.approved_by`, `state`, `outputs`, `verification`, `error` on failure, and `events`, each with `at` and `by` (previewed, confirmed, awaiting_approval, approved, running, succeeded or failed). The commit on `main` also carries `Seikika-Run`, `Requested-By` and `Approved-By` trailers | `.seikika-state/served/org/runs/<id>.json`; `seikika run --verify <id> -o json` |
| R8 Usable by agents | Every step is one `seikika run` command, with no prompts. `-o json` gives JSON, and the exit codes are 0 succeeded, 1 failed, 2 invalid or refused, 3 awaiting approval, 4 unverified. `SEIKIKA_URL` and `SEIKIKA_AS` can replace `--url` and `--as` | commands below |

## Commands

Set up the shell. Every command below assumes this:

```sh
cd /tmp/actions-ab/r6/seikika-2
export PATH=/tmp/actions-ab/r6/seikika-2/bin:$PATH
export SEIKIKA_STATE_DIR=/tmp/actions-ab/r6/seikika-2/.seikika-state
export SEIKIKA_URL=http://127.0.0.1:7650
```

**Start the org portal.** It returns once the server answers. On this loopback address, `--as`
says who you are.

```sh
seikika serve org --listen 127.0.0.1:7650 --detach
# check it prints: approval key: ed25519:f1035f7a2f04bb11dd97ac21c9f9a0bece005e3c5b75a8ab9eb5861a8826cac4
```

**List** the actions on all three repositories:

```sh
seikika run --as ann -o json
```

**Preview** (changes nothing; prints the diff):

```sh
seikika run --as ann mark-deprecated --entity portal:alpha \
  -i successor=acme/dependabot-proxy-v2 \
  -i 'reason=Replaced by the v2 proxy, which supports OIDC registries.' -o json
```

**Request as ann.** It exits 3 (`awaiting_approval`) and prints the run's record; its `id` is the
run id:

```sh
RUN=$(seikika run --as ann mark-deprecated --entity portal:alpha \
  -i successor=acme/dependabot-proxy-v2 \
  -i 'reason=Replaced by the v2 proxy, which supports OIDC registries.' --yes -o json \
  | python3 -c 'import json,sys; print(json.load(sys.stdin)["id"])')
echo "$RUN"
```

Because alpha's `main` already has this content, approving this request again succeeds without
a new commit (`outputs.changed: false`). Use other inputs to see a new commit.

**Approve as bob.** It runs the action, waits, and exits 0 only once `main` is seen to have the file:

```sh
seikika run --as bob --approve "$RUN" --note "looks right" --wait -o json
```

**Check a run's status, read its record, list every run:**

```sh
seikika run --as ann --verify "$RUN" -o json | python3 -c 'import json,sys; print(json.load(sys.stdin)["state"])'
seikika run --as ann --verify "$RUN" -o json     # the full record
seikika run --as ann --runs -o json              # every run: id, repository, requester, state
git -C origins/alpha.git show main:DEPRECATED.md # what main has
```

**Stop the server:**

```sh
seikika serve org --stop
```

## Testing the refusals

Run these against the served portal.

| Test | Command | Expected |
|---|---|---|
| Bad successor | `seikika run --as ann mark-deprecated --entity portal:beta -i successor=Acme/Bad -i 'reason=Twenty characters or more here.' -o json` | `422 invalid-inputs`, exit 2, no run |
| Short reason | `… -i successor=acme/x -i reason=short` | `422 invalid-inputs`, exit 2 |
| Reason over 200 | `… -i reason=$(printf 'r%.0s' $(seq 201))` | `422 invalid-inputs`, exit 2 |
| Self as successor | `… --entity portal:beta -i successor=acme/beta …` | preview: exit 0 with `blockers: ["precondition not met: …"]` and no `confirm_token`; with `--yes`: exit 2, no run |
| Self-approval | request as ann with `--yes`, then `seikika run --as ann --approve <id> -o json` | `403 not-an-approver`, exit 2, still `awaiting_approval` |
| carol approves | `seikika run --as carol --approve <id> -o json` | `403 not-an-approver`, exit 2, still `awaiting_approval` |

### Rejected push

This was rehearsed live as `run_2fts2j47nl0j`. Make the origin refuse pushes, then request with
**new inputs**: a run whose content `main` already has succeeds without a commit
(`outputs.changed: false`).

```sh
printf '#!/bin/sh\necho "main is protected" >&2\nexit 1\n' > origins/alpha.git/hooks/pre-receive
chmod +x origins/alpha.git/hooks/pre-receive
seikika run --as ann mark-deprecated --entity portal:alpha -i successor=acme/push-test \
  -i 'reason=Testing that a rejected push fails.' --yes -o json        # exit 3, note the id
seikika run --as bob --approve <id> --wait -o json                      # exit 1, state failed, error: "…main is protected…"
git -C origins/alpha.git show main:DEPRECATED.md                        # unchanged
rm origins/alpha.git/hooks/pre-receive
```

## Tests without a server

Each repository's cases run the real lifecycle in a scratch copy of the repository and its
origin's `main`. Nothing reaches the real origin and no run is recorded.

```sh
cd /tmp/actions-ab/r6/seikika-2/alpha && seikika validate .seikika && seikika test .seikika   # 15 passed
```

The cases in `.seikika/tests/mark-deprecated.test.yaml` cover:

- **Valid runs:** ann requests and bob approves; bob requests and ann approves; the reason
  boundaries at 20 and 200 characters are accepted; shell and `printf` characters in the reason
  are written literally.
- **Invalid inputs:** a successor that is not `owner/name`; an upper-case successor; reasons of 19
  and 201 characters; a missing input; an unknown input.
- **Precondition:** the repository as its own successor is blocked.
- **Approvals:** ann's own approval and carol's are refused, and carol's own request needs a
  maintainer.
- **Rejected push:** the run fails with the origin's message and `main` is unchanged.
