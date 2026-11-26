# Handoff: `mark-deprecated`

`mark-deprecated` is a Seikika self-service action. Each repository defines it in its own
`.seikika/seikika.yaml` on its origin's `main`. A Seikika server serves the org portal in this
directory, which lists all three repositories' actions and runs them. ann and bob are the only
approvers, and nobody approves their own run. A run is reported `succeeded` only after Seikika
has fetched the origin's `main` and seen `DEPRECATED.md` there with the content the preview
showed.

All three runs requested by ann and approved by bob have succeeded:

| Repository | Run | Commit on origin `main` |
|---|---|---|
| alpha | `run_2fr9m0o280by` | `21d465c` |
| beta | `run_2fr9n6novrxs` | `c7ea0a1` |
| gamma | `run_2fr9o9z86ahj` | `f75bb75` |

An earlier alpha run, `run_2fr9gnv6tkiz`, is recorded as `failed`. It was a deliberate test of a
rejected push: a temporary `pre-receive` hook on `origins/alpha.git` refused the push, so nothing
landed. The hook has since been removed. A beta run, `run_2fral7das1ap`, is recorded as
`rejected`: it checked the handoff's approval tests (ann and carol were refused, then bob
rejected it).

No glue code was needed. The pieces are Seikika (`/tmp/actions-ab/r2/seikika-1/bin`), three repo
manifests, the org manifest and a start script.

| File | What it is |
|---|---|
| `alpha/.seikika/seikika.yaml`, `beta/.seikika/seikika.yaml`, `gamma/.seikika/seikika.yaml` | Each repository's definition of `mark-deprecated`: the input schema, the approvers, the executor, and the org key it accepts approvals from. Committed to each origin's `main` |
| `org/seikika.yaml` | The org portal: lists the three origins as `federation.children` |
| `org/bin/serve.sh` | Starts the org portal server on `127.0.0.1:7410`, with state in `org/.seikika-state` |
| `org/.seikika-state/` (git-ignored) | Server state: run records (`served/org/runs/*.json`), the child runs (`runs/*.json`), clones of the children (`federation/`), and the approval signing key (`served/org/signing-key`) |

## How each requirement is met

| # | How | Where |
|---|---|---|
| R1 Owned by the repository | The action is declared in each repository's `.seikika/seikika.yaml`, committed to `origins/<name>.git` `main`. The org portal only lists the repositories. It clones each one and syncs it before every list, preview and run, so only what is on `main` counts. A repository's team changes its action by committing to its own `main` | `alpha/.seikika/seikika.yaml` (same for beta and gamma); `org/seikika.yaml` |
| R2 Validated | `inputs` is a JSON Schema: `successor` must match `^[a-z0-9-]+/[a-z0-9-]+$`, `reason` must be 20 to 200 characters, both are required, and other fields are refused. The server checks it before any preview or run. Invalid inputs return `422 invalid-inputs` with exit code 2, and no run is created | `inputs:` in each `.seikika/seikika.yaml` |
| R3 Previewed | Running a request without `--yes`/`--confirm` previews it. It shows the exact `DEPRECATED.md` diff, the push to `main`, the approvers and a confirm token, and changes nothing. A request with `--yes` records the same preview as the run's first event (`previewed`). If, at run time, the change would differ from the confirmed preview, it fails before it commits | `executor: {type: change, land: push}` |
| R4 Approved | `approval: "[['user:ann', 'user:bob']]"` means one of ann or bob, and never the requester. A run waits in `awaiting_approval` (exit 3) until someone approves it. If the requester tries, they get `403 not-an-approver: a requester cannot approve their own run`. carol gets `403 not-an-approver: user:carol is not one of the pending approvers`. The org server signs each approval, and each repository accepts only approvals signed with the key it pins in `federation.acceptApprovalsFrom` | `approval:` and `federation.acceptApprovalsFrom` in each `.seikika/seikika.yaml`; key in `org/.seikika-state/served/org/signing-key` |
| R5 Done means observed | `change` with `land: push` writes `DEPRECATED.md` in a fresh worktree of `main`, commits, and pushes without forcing. Seikika then fetches `origin/main` and checks two things: the commit is on it, and each changed file's blob matches the preview. Only then is the run `succeeded` (see `verification.child.files[].matches` in the record). If the push is rejected, the run is `failed`, the error contains the origin's message, and exit code is 1. Nothing is retried | `executor:` in each `.seikika/seikika.yaml`; record field `verification` |
| R6 One place for the org | Serve `org/` and every command takes `--url http://127.0.0.1:7410`. `seikika run` lists `mark-deprecated` on `portal:alpha`, `portal:beta` and `portal:gamma`. `--entity portal:<name>` previews, requests or approves on any of them, and `--runs` lists every run. You don't need to go into a repository's checkout | `org/seikika.yaml`, `org/bin/serve.sh` |
| R7 Audited | Each run record has: `requested_by`; `approval.approved_by`; `inputs`; `state` and `outputs` (commit, `landed`); `error` on failure; and `events` with a timestamp and actor for each step (`previewed`, `confirmed`, `awaiting_approval`, `approved` by whom with their note, `dispatched`, `running`, `succeeded`/`failed`). Records are on disk and survive a restart. The landed commit also carries `Seikika-Run`, `Requested-By` and `Approved-By` trailers | `org/.seikika-state/served/org/runs/<run id>.json`; `seikika run --url … --verify <run id>` |
| R8 Usable by agents | Each step is a non-interactive `seikika run` command that takes `-o json`. Exit codes: 0 succeeded, 1 failed, 2 invalid or refused, 3 awaiting approval. The agent skills are in `seikika-docs/skills/run/SKILL.md` and `seikika-docs/skills/actions/SKILL.md` | the commands below |

## Commands

Run everything from the org directory. Use the same shell for every command.

```sh
cd /tmp/actions-ab/r2/seikika-1/org
export PATH=/tmp/actions-ab/r2/seikika-1/bin:$PATH
U=http://127.0.0.1:7410
```

**Start the server.** It must be running for every command below. Run it in its own terminal, or
in the background:

```sh
nohup ./bin/serve.sh > serve.log 2>&1 &
sleep 3; grep approval_key serve.log | tail -1   # must be ed25519:18015d83…07f1, the key the repositories pin
```

Keep `SEIKIKA_STATE_DIR` at its default (`org/.seikika-state`), which `serve.sh` sets. That
directory holds the signing key the repositories pin, plus the run records. With a different
state directory the server generates a new key, and the repositories refuse its approvals
(`child-refuses-approvals`). To stop the server, run `pkill -f 'seikika serve --listen 127.0.0.1:7410'`.

On this loopback address, `--as <user>` is how a caller says who they are (`ann`, `bob`,
`carol`, or `user:ann`, and so on).

**List the actions** for all three repositories:

```sh
seikika run --url $U --as ann -o json
```

**Preview.** This changes nothing. It prints the diff, the approvers and a confirm token:

```sh
seikika run --url $U --as ann mark-deprecated --entity portal:alpha \
  -i successor=acme/dependabot-proxy-v2 \
  -i "reason=Replaced by the v2 proxy, which supports OIDC registries." -o json
```

**Request as ann.** Use either `--yes`, which previews and requests in one step, or
`--confirm <confirm_token>` from the preview, within 10 minutes. It exits 3 with
`state: awaiting_approval`. The run id is `.id`:

```sh
RID=$(seikika run --url $U --as ann mark-deprecated --entity portal:alpha \
  -i successor=acme/dependabot-proxy-v2 \
  -i "reason=Replaced by the v2 proxy, which supports OIDC registries." \
  --yes -o json | python3 -c 'import json,sys; print(json.load(sys.stdin)["id"])')
echo $RID
```

**Approve as bob.** The run starts at once. `--wait` follows it until it ends:

```sh
seikika run --url $U --as bob --approve $RID --note "checked the diff" --wait -o json
```

**Check a run's status, or read its full record.** Both use the same command. Exit 0 means
`succeeded`, 1 means `failed`, 3 means still waiting. Read `.state` for the status, or the
whole object for the record:

```sh
seikika run --url $U --as ann --verify $RID -o json
seikika run --url $U --as ann --verify $RID -o json | python3 -c 'import json,sys; d=json.load(sys.stdin); print(d["state"])'
seikika run --url $U --as ann --runs -o json    # every run, with state and requester
cat .seikika-state/served/org/runs/$RID.json     # the record on disk
```

To reject a run instead: `seikika run --url $U --as bob --reject $RID --note "why"`.

## Negative tests

All three repositories already have `DEPRECATED.md` with the inputs above. Requesting the same
inputs again changes no files: the preview has no effects and warns `the change command changed
no files`. So for new runs, use a different `reason` or `successor`. Any run that succeeds
overwrites `DEPRECATED.md` on that repository's `main` with the new content.

```sh
# Invalid inputs: 422 invalid-inputs, exit 2, no run created
seikika run --url $U --as ann mark-deprecated --entity portal:beta -i successor=Acme/X -i "reason=too short" --yes -o json
seikika run --url $U --as ann mark-deprecated --entity portal:beta -i successor=acme/x --yes -o json   # reason missing

# Self-approval and carol: 403 not-an-approver, exit 2; the run stays awaiting_approval
RID=$(seikika run --url $U --as ann mark-deprecated --entity portal:beta -i successor=acme/x \
  -i "reason=Test of the approval rules only." --yes -o json | python3 -c 'import json,sys; print(json.load(sys.stdin)["id"])')
seikika run --url $U --as ann   --approve $RID -o json
seikika run --url $U --as carol --approve $RID -o json
seikika run --url $U --as bob   --reject  $RID --note "test only" -o json   # clean up without landing

# Rejected push: the run ends failed (exit 1) with the origin's message, and main is unchanged
H=/tmp/actions-ab/r2/seikika-1/origins/gamma.git/hooks/pre-receive
printf '#!/bin/sh\necho "main is protected" >&2\nexit 1\n' > $H && chmod +x $H
RID=$(seikika run --url $U --as ann mark-deprecated --entity portal:gamma -i successor=acme/x \
  -i "reason=Test of a rejected push to main." --yes -o json | python3 -c 'import json,sys; print(json.load(sys.stdin)["id"])')
seikika run --url $U --as bob --approve $RID --wait -o json; echo "exit=$?"
seikika run --url $U --as ann --verify $RID -o json | python3 -c 'import json,sys; d=json.load(sys.stdin); print(d["state"], d["error"])'
rm $H
```

## Notes

- The server logs `running WITHOUT authorization (dev mode only)` because no OpenFGA binary is
  installed. This means anyone, carol included, may *request* a run. Approval is still enforced
  by each action's `approval` rule and the requester exclusion, as shown above. To restrict who
  may request, too, install OpenFGA (`SEIKIKA_OPENFGA_BIN`) and add an `authz:` model.
- Identity on loopback is whatever `--as` says: the sandbox has no passwords. Off loopback, set
  `SEIKIKA_AUTH_MODE` (`seikika serve --help`).
- The repository definitions were committed with the org key pinned. If the key ever changes,
  update `federation.acceptApprovalsFrom` in each repository's `.seikika/seikika.yaml` and push.
  `seikika run --url $U` prints the current key.
