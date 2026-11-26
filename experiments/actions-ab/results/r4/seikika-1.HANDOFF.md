# Handoff: `mark-deprecated`, a self-service action per repository

**Status.** `mark-deprecated` is declared in `alpha`, `beta` and `gamma`, each in its own
`.seikika/seikika.yaml` on its origin's `main`. It is offered from one org portal
(`org/seikika.yaml`), served by Seikika. ann requested a run on each repository, bob approved each
one, and all three are reported `succeeded`. I checked myself that `DEPRECATED.md` on each origin's
`main` matches the expected content byte for byte:

| Repository | Run id (org) | Commit on `origins/<repo>.git` `main` | State |
|---|---|---|---|
| alpha | `run_2fssefqli4ia` | `085bd49` | succeeded |
| beta | `run_2fssemupwey3` | `6931115` | succeeded |
| gamma | `run_2fssetokm8ak` | `0279ea7` | succeeded |

Seikika (`/tmp/actions-ab/r4/seikika-1/bin`) does all the work: input validation, preview,
approvals, the commit and push, verification on the fetched `main`, and run records. The only
glue I wrote is `org/env.sh`, which sets the environment, and the shell one-liner in each
manifest that writes the file. The org server is **stopped**; start it as shown below.

## Files

| Path | What it is |
|---|---|
| `alpha/.seikika/seikika.yaml` (and `beta/`, `gamma/`) | The repository's own action: inputs schema, precondition, approvers, `change` executor that pushes to `main`, and the org key it accepts approvals from |
| `alpha/.seikika/tests/mark-deprecated.test.yaml` (and `beta/`, `gamma/`) | 10 cases run by `seikika test`: the success case, invalid inputs, a repository naming itself as successor, self-approval, carol's approval, and a rejected push |
| `org/seikika.yaml` | The org portal: lists the three origins as children `portal:alpha`, `portal:beta` and `portal:gamma` |
| `org/env.sh` | Puts `bin/` on `PATH` and sets `SEIKIKA_STATE_DIR`, `XDG_STATE_HOME` and `SEIKIKA_URL` |
| `.seikika-state/served/org/signing-key` | The org's approval signing key. Its public half is pinned in each repository's manifest |
| `.seikika-state/served/org/runs/<run id>.json` | The org's run records, which survive a restart |
| `.seikika-state/served/org/serve.log` | The server log |
| `.seikika-state/federation/` | The server's clones of the three origins |

Keep `SEIKIKA_STATE_DIR=/tmp/actions-ab/r4/seikika-1/.seikika-state`, which `env.sh` sets. With
another directory, the server makes a new signing key and the repositories refuse its approvals
(`child-refuses-approvals`).

## How each requirement is met

| # | How | Where |
|---|---|---|
| R1 Owned by the repository | Each repository's definition is committed to its origin's `main` (commits `bad1290`, `acaa7db` and `e248bda`). The org portal holds no definition of the action. It clones each origin and reads `.seikika/seikika.yaml` again before every list, preview and run, so a change a team pushes to its own `main` (approvers, schema, command) takes effect with no change to the org. The team tests its change with `seikika test <repo>/.seikika` | `alpha/.seikika/seikika.yaml`, `beta/.seikika/seikika.yaml`, `gamma/.seikika/seikika.yaml`; `org/seikika.yaml` |
| R2 Validated | `inputs` is a JSON Schema: both inputs are required, `additionalProperties: false`, `successor` must match `^[a-z0-9-]+/[a-z0-9-]+$`, and `reason` must have `minLength: 20` and `maxLength: 200`. Seikika checks the inputs before any preview or run: the server returns `422 invalid-inputs` with exit 2 and creates no run. A precondition (`!inputs.successor.endsWith('/<repo>')`) also blocks a repository from naming itself as its successor, with exit 2 | `inputs` and `preconditions` in each `.seikika/seikika.yaml` |
| R3 Previewed | Without `--yes` or `--confirm`, `seikika run` only previews. It prints the exact diff of `DEPRECATED.md`, the push to `main`, the approvers and a confirm token valid for 10 minutes, and changes nothing. The run's record keeps the previewed effects. If the change would differ from the confirmed preview, the run fails before it commits | Seikika `change` executor (`executor:` in each manifest) |
| R4 Approved | `approval: "[['user:ann', 'user:bob']]"` requires one of ann or bob, and the requester never counts. So ann's run needs bob, and bob's needs ann. Self-approval returns `403 not-an-approver: a requester cannot approve their own run`. carol gets `403 not-an-approver: user:carol is not one of the pending approvers`. Both exit 2. The org signs the approval, and the repository accepts it only with the org's key pinned in `federation.acceptApprovalsFrom`. The commit carries `Requested-By` and `Approved-By` trailers. Refused attempts are answered (403) but are not written into the run record | `approval` and `federation.acceptApprovalsFrom` in each manifest |
| R5 Done means observed | `executor: {type: change, land: push, base: main}`. After pushing, Seikika fetches `origin/main`, checks that the commit is on it, and checks that `DEPRECATED.md`'s blob equals the previewed content. Only then is the run `succeeded`. The record's `verification` shows `"matches": true` and the blob. A rejected push fails the run (exit 1) with the origin's message, and nothing is retried. I checked this live against a scratch copy of alpha's origin with a `pre-receive` hook, and in the `rejectPush` test case | each manifest's `executor`; the record's `verification` field |
| R6 One place for the org | `org/seikika.yaml` lists the three origins under `federation.children`. One server (`seikika serve org`) lists, previews, requests, approves and reports runs for all three as `portal:alpha`, `portal:beta` and `portal:gamma`. It runs each action in its own clone of that repository, so nobody has to go into the repositories | `org/seikika.yaml` |
| R7 Audited | Each run is a JSON record in `.seikika-state/served/org/runs/<id>.json`, read with `--verify <id>`. It holds `requested_by`, `inputs` and `approval.approved_by`. Its `events` list has `previewed`, `confirmed`, `awaiting_approval`, `approved` (with `by` and the note), `dispatched`, `running` and `succeeded` or `failed`, each with an `at` timestamp. It also holds `outputs.commit`, `verification`, `error` on failure, and the child repository's own run (`child`). The commit on `main` repeats this in its `Seikika-Run`, `Requested-By` and `Approved-By` trailers | `.seikika-state/served/org/runs/` |
| R8 Usable by agents | Every step is one non-interactive `seikika run` command, and `-o json` gives JSON. Identity comes from `--as`, or from `SEIKIKA_AS`, because the server listens on loopback. Exit codes: 0 succeeded, 1 failed, 2 invalid or refused, 3 waiting for approval, 4 unverified | the commands below |

## Commands

Run these from any directory.

```sh
. /tmp/actions-ab/r4/seikika-1/org/env.sh

# Start the org portal. It returns once the server answers and prints the URL, pid, log and approval key.
seikika serve /tmp/actions-ab/r4/seikika-1/org --listen 127.0.0.1:7910 --detach
# The key printed must be ed25519:1c2f9c8c769c0d7050b180aae6d6758492e839952db31af3a3319e16bbe69ac1

# List mark-deprecated on all three repositories
seikika run --as ann -o json

# Preview: shows the diff and changes nothing. Prints a confirm token (ct_...).
seikika run --as ann mark-deprecated --entity portal:alpha \
  -i successor=acme/dependabot-proxy-v2 \
  -i reason='Replaced by the v2 proxy, which supports OIDC registries.' -o json

# Request as ann. Either confirm the token you previewed:
seikika run --as ann mark-deprecated --entity portal:alpha \
  -i successor=acme/dependabot-proxy-v2 \
  -i reason='Replaced by the v2 proxy, which supports OIDC registries.' --confirm <ct_token> -o json
# or preview and request in one step. Exit 3 means it is waiting for approval. The run id is the JSON field "id".
seikika run --as ann mark-deprecated --entity portal:alpha \
  -i successor=acme/dependabot-proxy-v2 \
  -i reason='Replaced by the v2 proxy, which supports OIDC registries.' --yes -o json

# Approve as bob. --wait follows the run to the end: exit 0 succeeded, 1 failed.
seikika run --as bob --approve <run-id> --note "checked the preview" --wait -o json

# Check a run's status and read its full record (state, requester, approver, inputs, events with times, verification)
seikika run --as ann --verify <run-id> -o json
# Wait for an unfinished run to settle
seikika run --as ann --verify <run-id> --wait -o json

# Every run, one place
seikika run --as ann --runs -o json

# Stop the server when you are done
seikika serve /tmp/actions-ab/r4/seikika-1/org --stop
```

Use `--entity portal:beta` or `portal:gamma` for the other repositories. Without the server,
`seikika validate <repo>/.seikika && seikika test <repo>/.seikika` runs all 10 cases in a scratch
copy, so the real origin is untouched.

## Test the refusals

All three repositories already have the `DEPRECATED.md` from the runs above. A new run with the
same inputs commits nothing and succeeds with `outputs.changed: false`. To see a change, use a
different reason.

```sh
. /tmp/actions-ab/r4/seikika-1/org/env.sh
# Invalid inputs: 422 invalid-inputs, exit 2, and no run is created
seikika run --as ann mark-deprecated --entity portal:beta -i successor=Acme/Store -i reason='too short' --yes -o json
seikika run --as ann mark-deprecated --entity portal:beta -i successor=acme/x -i reason="$(printf 'x%.0s' $(seq 201))" --yes -o json
seikika run --as ann mark-deprecated --entity portal:beta -i successor=acme/x --yes -o json   # missing reason

# Self-approval and carol: both get 403 not-an-approver, exit 2, and the run stays awaiting_approval
RID=$(seikika run --as ann mark-deprecated --entity portal:beta -i successor=acme/valtio-store \
  -i reason='State management moves to the shared store package, for real.' --yes -o json \
  | python3 -c 'import json,sys; print(json.load(sys.stdin)["id"])')
seikika run --as ann   --approve $RID -o json
seikika run --as carol --approve $RID -o json
seikika run --as ann --verify $RID -o json   # "state": "awaiting_approval"

# Rejected push: make the origin refuse pushes, then approve. The run fails (exit 1) with the
# origin's message, and DEPRECATED.md on main is unchanged.
printf '#!/bin/sh\necho "main is protected" >&2\nexit 1\n' > /tmp/actions-ab/r4/seikika-1/origins/beta.git/hooks/pre-receive
chmod +x /tmp/actions-ab/r4/seikika-1/origins/beta.git/hooks/pre-receive
seikika run --as bob --approve $RID --wait -o json   # "state": "failed", "error": "the push to origin main was refused ... main is protected"
git --git-dir=/tmp/actions-ab/r4/seikika-1/origins/beta.git show main:DEPRECATED.md   # still the earlier content
rm /tmp/actions-ab/r4/seikika-1/origins/beta.git/hooks/pre-receive
```

## Notes

- `DEPRECATED.md` is `# Deprecated: use <successor>`, a blank line and `<reason>`. Like any text
  file, it ends with a newline. It is written by
  `printf '# Deprecated: use %s\n\n%s\n' "$SEIKIKA_INPUT_SUCCESSOR" "$SEIKIKA_INPUT_REASON"`.
- The commits are authored by `Seikika`. Who requested and who approved each run is in the
  commit trailers and in the run record.
- People sign in with `--as` only because the server listens on loopback, which is right for this
  sandbox, where saying who you are is enough. Anywhere else, set `SEIKIKA_AUTH_MODE` (see
  `seikika serve --help`).
