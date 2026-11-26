# Handoff: `mark-deprecated`, a self-service action on Seikika

`mark-deprecated` is declared in each repository and offered, approved, run and audited from one
org portal. All of it is stock Seikika 0.1.1 (`/tmp/actions-ab/r2/seikika-3/bin/seikika`). The
only glue is `org/serve.sh`, a start script for the server.

The three requested runs were requested by ann, approved by bob, and **succeeded**:

| Repository | Org run id | Repository run id (`Seikika-Run` trailer) | Commit on `main` |
|---|---|---|---|
| alpha | `run_2frhaymfokv4` | `run_2frhb8sflhoe` | `docs: mark alpha deprecated` |
| beta | `run_2frhc8k3fnqq` | `run_2frhcdz3x9da` | `docs: mark beta deprecated` |
| gamma | `run_2frhdfbg0glg` | `run_2frhdkf234n3` | `docs: mark gamma deprecated` |

## How it fits together

| Piece | Path | What it does |
|---|---|---|
| Action definition, per repository | `<repo>/.seikika/seikika.yaml` (committed to `origins/<repo>.git` `main`) | Inputs schema, approval rule (`ann` or `bob`, never the requester), and a `change` executor with `land: push` that writes `DEPRECATED.md`, commits and pushes to `main` |
| Org portal | `org/seikika.yaml` | Lists the three origins under `federation.children`. Each one is offered as `portal:<repo>` |
| Server start script | `org/serve.sh` | `seikika serve` on `127.0.0.1:7490` with `SEIKIKA_STATE_DIR=/tmp/actions-ab/r2/seikika-3/state` |
| Server state | `/tmp/actions-ab/r2/seikika-3/state/` | `served/org/runs/*.json` org run records, `runs/*.json` repository run records, `federation/` clones of the origins, `served/org/signing-key` the approval key |

When the org server runs an action, it syncs its clone of the repository's origin first, so only
the definition committed on `main` counts. It records the approval, signs it, and runs the
repository's own action in that clone. The repository accepts the approval only if the signature
verifies against the key it pins (`federation.acceptApprovalsFrom`:
`ed25519:be66a6e8ebadb9e369ce07aef548027b1b9a1c10cbabbdab0baa618e507484cf`), and it checks its own
approval rule again.

## Requirements

| # | How it is met | Files |
|---|---|---|
| R1 Owned by the repository | The action lives in each repository's `.seikika/seikika.yaml`, committed to its origin's `main`. The org portal holds no copy: it reads each origin's `main` again before every list, preview and run. A maintainer changes the action by committing to their repository | `alpha/.seikika/seikika.yaml`, `beta/.seikika/seikika.yaml`, `gamma/.seikika/seikika.yaml` |
| R2 Validated | `inputs` is a JSON Schema: `successor` has `pattern: ^[a-z0-9-]+/[a-z0-9-]+$`, `reason` has `minLength: 20` and `maxLength: 200`, both are required, and `additionalProperties: false`. Seikika checks it before any preview. Bad input gets `422 invalid-inputs` and exit 2, with no run recorded and nothing changed | same files, `actions[0].inputs` |
| R3 Previewed | A request without `--yes`/`--confirm` is a preview. It runs the command in a throwaway worktree and prints the exact diff of `DEPRECATED.md`, the push to `main`, the approvers and any blockers, and changes nothing. A request with `--yes` records that preview on the run (`effects`) | executor in the same files |
| R4 Approved | `approval: "[['user:ann', 'user:bob']]"` means any one of ann or bob, and the requester never counts. Self-approval gets `403 not-an-approver: a requester cannot approve their own run`. carol gets `403 not-an-approver: user:carol is not one of the pending approvers`. Both exit 2. The repository re-checks the org's signed approval against its pinned key, and refuses a run whose approval isn't signed with it (`child-refuses-approvals`) | `approval` and `federation.acceptApprovalsFrom` in each `<repo>/.seikika/seikika.yaml` |
| R5 Done means observed | `land: push` pushes once, never forced. After the push, Seikika fetches `origin/main` and checks that the commit is on it and that every changed file's blob matches the preview (`verification.files[].matches`). Only then is the run `succeeded`. A rejected push is `failed` with the origin's message, and so is a push the origin accepted but that isn't on `main` ("the commit is not on origin/main: the push did not land"). A re-run that changes nothing is `unverified`, not `succeeded` | executor in each `<repo>/.seikika/seikika.yaml` |
| R6 One place for the org | The org portal serves every repository's actions. `seikika run --url http://127.0.0.1:7490` lists `mark-deprecated` on `portal:alpha`, `portal:beta` and `portal:gamma`, and every request, approval and record goes through it | `org/seikika.yaml`, `org/serve.sh` |
| R7 Audited | Each run record has `requested_by`, `approval.approved_by`, `inputs`, `effects`, `state`, `outputs.commit`, `verification`, and `events[]` with `at` (UTC), `by` and `state` for previewed, confirmed, awaiting_approval, approved (with the note), dispatched, running, succeeded or failed. The repository's own run record is nested under `child`. Records are kept in `state/served/org/runs/` and `state/runs/`, and survive a restart. Each commit also carries `Seikika-Run`, `Requested-By` and `Approved-By` trailers | `/tmp/actions-ab/r2/seikika-3/state/served/org/runs/<run>.json` |
| R8 Usable by agents | Every step is a non-interactive `seikika run` call that takes `-o json`. Exit codes: 0 succeeded, 1 failed (or stale or timed out), 2 invalid or refused, 3 awaiting approval. The skills in `seikika-docs/skills/run/SKILL.md` and `actions/SKILL.md` describe the loop | n/a |

## Commands

Set up the shell once:

```sh
cd /tmp/actions-ab/r2/seikika-3
export PATH=/tmp/actions-ab/r2/seikika-3/bin:$PATH
U=http://127.0.0.1:7490
```

**Start the org server.** This is the only process that must be running:

```sh
nohup /tmp/actions-ab/r2/seikika-3/org/serve.sh > /tmp/actions-ab/r2/seikika-3/state/serve.log 2>&1 &
echo $! > /tmp/actions-ab/r2/seikika-3/state/serve.pid
```

Keep `SEIKIKA_STATE_DIR` at `/tmp/actions-ab/r2/seikika-3/state` (`serve.sh` sets it), because the
repositories pin the signing key stored there. If you delete that directory, the server mints a
new key and prints it at start. Each repository then refuses approvals (`child-refuses-approvals`)
until a maintainer commits the new key to its `federation.acceptApprovalsFrom`.

Stop it with `kill $(cat /tmp/actions-ab/r2/seikika-3/state/serve.pid)`.

On a loopback address the server signs callers in with `--as` (there are no passwords in this
sandbox). `--as ann` and `--as user:ann` are the same person.

**List the actions** (all three repositories):

```sh
seikika run --url $U --as ann -o json
```

**Preview** (shows the diff, changes nothing, exit 0):

```sh
seikika run --url $U --as ann mark-deprecated --entity portal:alpha \
  -i successor=acme/dependabot-proxy-v2 \
  -i "reason=Replaced by the v2 proxy, which supports OIDC registries." -o json
```

**Request, as ann.** `--yes` records the preview and requests the run. It exits 3 and prints the
run's `id`:

```sh
seikika run --url $U --as ann mark-deprecated --entity portal:alpha \
  -i successor=acme/dependabot-proxy-v2 \
  -i "reason=Replaced by the v2 proxy, which supports OIDC registries." --yes -o json
```

Alternatively, run with `--confirm <token>` using the token the preview printed. The token is
valid for 10 minutes and can be used once.

**Approve, as bob.** The run starts at once. `--wait` follows it until it ends, then exits 0 on
success or 1 on failure:

```sh
seikika run --url $U --as bob --approve <run-id> --note "checked" --wait -o json
```

To reject instead, per `seikika run --help` (I did not exercise this):
`seikika run --url $U --as bob --reject <run-id> --note "why" -o json`.

**Check a run's status** (`.state`, for example `awaiting_approval`, `running`, `succeeded`,
`failed` or `unverified`):

```sh
seikika run --url $U --as ann --verify <run-id> -o json | python3 -c 'import json,sys; print(json.load(sys.stdin)["state"])'
```

**Read a run's record**, with who requested and approved it, the inputs, effects, events with
timestamps, the commit and verification:

```sh
seikika run --url $U --as ann --verify <run-id> -o json
```

**List every run:**

```sh
seikika run --url $U --as ann --runs -o json
```

Without the server, `seikika run -o json` in a repository checkout (for example
`/tmp/actions-ab/r2/seikika-3/alpha`) lists and previews that repository's action. A request there
stops at exit 3, because only the served org portal holds approvals.

## Testing the refusals

I ran each of these against a scratch copy of alpha before the real runs, then deleted the copy:

| Test | Command | Result |
|---|---|---|
| Bad successor | `... mark-deprecated --entity portal:beta -i successor=Acme/X -i "reason=A long enough reason for testing." -o json` | `422 invalid-inputs: successor: "Acme/X" does not match ...`, exit 2 |
| Short or long reason | `-i reason=short`, or 201 characters | `422 invalid-inputs: reason: ... shorter than 20 / longer than 200 characters`, exit 2 |
| Unknown input | `-i foo=bar` | `422 invalid-inputs: foo is not an input of mark-deprecated`, exit 2 |
| Self-approval | ann requests, then `seikika run --url $U --as ann --approve <id> -o json` | `403 not-an-approver: a requester cannot approve their own run`, exit 2. The run stays `awaiting_approval` |
| carol approves | `seikika run --url $U --as carol --approve <id> -o json` | `403 not-an-approver: user:carol is not one of the pending approvers`, exit 2 |
| Rejected push | Install a rejecting hook (below), request as ann, approve as bob with `--wait` | `state: failed`, with the last event's note "the push to origin main was refused, so nothing landed: ... (pre-receive hook declined)", exit 1. `main` is unchanged |

To make an origin reject pushes:

```sh
printf '#!/bin/sh\necho "main is frozen" >&2\nexit 1\n' > /tmp/actions-ab/r2/seikika-3/origins/beta.git/hooks/pre-receive
chmod +x /tmp/actions-ab/r2/seikika-3/origins/beta.git/hooks/pre-receive
# afterwards: rm /tmp/actions-ab/r2/seikika-3/origins/beta.git/hooks/pre-receive
```

The three repositories already have `DEPRECATED.md`. To get a real change on a rejected-push test,
use a different `successor` or `reason`. With the same inputs the preview warns "the change command
changed no files", and an approved run ends `unverified` rather than `succeeded`.

## Notes and limits

- The file content is exactly `# Deprecated: use <successor>`, a blank line, then `<reason>`, and
  it ends with one newline (`printf '# Deprecated: use %s\n\n%s\n'`). Inputs reach the command as
  environment variables (`SEIKIKA_INPUT_SUCCESSOR`, `SEIKIKA_INPUT_REASON`), never through the
  shell's parser.
- Approvers are named in each repository's rule (`user:ann`, `user:bob`). Adding a maintainer
  means committing a change to that line in the repository.
- The server runs without OpenFGA (it logs "running WITHOUT authorization"). Approval rules are
  enforced as shown above, but anyone signed in may *request* or preview a run, carol included.
  Only ann or bob can approve one.
- Sign-in is Seikika's dev mode on loopback, which fits this sandbox's "saying who you are is
  enough". Elsewhere, set `SEIKIKA_AUTH_MODE` (`seikika serve --help`).
- The `Seikika-Run` trailer on each commit is the repository's run id. The org run's record shows
  it under `verification.child_run`.
