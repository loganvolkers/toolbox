# Handoff: `mark-deprecated`

`mark-deprecated` is a Seikika self-service action. Each repository (`alpha`, `beta`, `gamma`)
defines it in its own `.seikika/seikika.yaml` on its origin's `main`. The org portal in this
directory lists every repository's actions, records requests and approvals, and runs an approved
run in that repository's checkout. The run pushes one commit to the origin's `main`, never forced.

The three runs requested by ann and approved by bob have **succeeded**:

| Repository | Run id (org) | Commit on origin `main` |
|---|---|---|
| alpha | `run_2fqnqqdjm0pm` | `9c9a09d` |
| beta | `run_2fqnqv728xfj` | `d3d79fe` |
| gamma | `run_2fqnqzfvy8fi` | `9ad79e6` |

Paths below are relative to `/tmp/actions-ab/r1/seikika-2` unless they start with `/`.

## Files

| File | What it is |
|---|---|
| `alpha/.seikika/seikika.yaml` (same in `beta/`, `gamma/`) | The repository's portal and its `mark-deprecated` action: inputs, precondition, approval rule, executor. Committed to `origins/<repo>.git` `main` |
| `org/seikika.yaml` | The org portal: finds the three repository portals and runs their actions (federation, `runners.local`) |
| `org/scripts/serve.sh` | Starts the org portal on 127.0.0.1:7150 (dev sign-in, approval signing key, persistent run records) |
| `org/.secrets/approval.key` | The org's approval signing key (gitignored). Each repository pins its public key, `ed25519:a5ccf776…ef7aa` |
| `org/.state/served/org/runs/<run id>.json` | The org run records (gitignored local state; they survive restarts) |
| `org/.state/runs/<child run id>.json` | The record the repository side keeps of the same run |

## How each requirement is met

| # | How | Where |
|---|---|---|
| R1 Owned by the repository | The action's whole definition (inputs, precondition, approvers, what it writes) is in the repository, committed to its origin's `main` (`b718187` alpha, `edbb782` beta, `a63378f` gamma). The org portal only discovers it from the checkout. `serve.sh` fast-forwards each checkout to `origin/main` at start and every 30 s, so a maintainer's change on `main` is what gets served | `<repo>/.seikika/seikika.yaml`, `org/seikika.yaml` (`connectors[0].config.portals`), `org/scripts/serve.sh` |
| R2 Validated | `inputs` is a JSON Schema: `successor` must match `^[a-z0-9-]+/[a-z0-9-]+$`, `reason` must be 20 to 200 characters, and both are required. Invalid inputs get `422 invalid-inputs`, exit 2, and no run is created. A precondition also refuses the repository itself as its successor (`acme/alpha` on alpha): the preview lists it under `blockers` and returns no confirm token | `<repo>/.seikika/seikika.yaml` `actions[0].inputs`, `preconditions` |
| R3 Previewed | A preview, a request without `--confirm`, changes nothing. It returns `effects` (the exact `DEPRECATED.md` diff and the push to `origin/main`), `blockers`, the approvers and a `confirm_token`. A run starts only with that token, which is valid for 10 minutes and spent once (`409 token-used`). If the inputs or facts changed since the preview, the run is refused (`409 preview-stale`) | Seikika `change` executor, `<repo>/.seikika/seikika.yaml` `executor` |
| R4 Approved | `approval: "[['user:ann', 'user:bob']]"` means one of ann or bob, never the requester. Self-approval gets `403 not-an-approver: a requester cannot approve their own run`. carol gets `403 not-an-approver: user:carol is not one of the pending approvers`. Both exit 2 and the run stays `awaiting_approval`. The org signs the approval, and the repository accepts it only if the signature verifies against the key pinned in `federation.acceptApprovalsFrom` and it names this run's action, version, inputs, effects and requester. Running the action directly in a checkout (`seikika run` without the server) stops at `awaiting_approval`, because a repository portal holds no approvals | `<repo>/.seikika/seikika.yaml` `approval`, `federation`; `org/seikika.yaml` `federation.signingKey`; `org/.secrets/approval.key` |
| R5 Done means observed | `executor: {type: change, land: push, base: main}`. The run commits `DEPRECATED.md` in a worktree of `origin/main` and pushes it, never forced. It is `succeeded` only after the commit is seen on `origin/main` (`verification.by: base`, with the commit id). The commit's tree fixes `DEPRECATED.md`'s content, which is the previewed diff. A rejected push makes the run `failed` (exit 1), with git's rejection message in its last event (tested with a `pre-receive` hook). A rerun that changes nothing is `unverified`, not `succeeded` | `<repo>/.seikika/seikika.yaml` `executor` |
| R6 One place for the org | From `org/`, `seikika run --url http://127.0.0.1:7150 …` lists all three repositories' `mark-deprecated` (entities `portal:alpha`, `portal:beta`, `portal:gamma`), and previews, requests, approves and checks a run on any of them. Without a server, `seikika run -p . -o json` in `org/` lists them from the checkouts | `org/seikika.yaml`, `org/scripts/serve.sh` |
| R7 Audited | Each run record has `requested_by`, `inputs`, `approval.approved_by`, `state` and `events`. `events` holds every step with its time (`at`) and person (`by`): previewed, confirmed, awaiting_approval, approved or rejected (with `--note`), dispatched, running, then succeeded, failed or unverified with the reason. It also has `outputs.commit` and `verification`. The records are kept as files and survive a restart. The repository side keeps its own record (`child_run`), which notes the org run it came from (`via: parent:org/<id>`). The commit on `main` is authored by the requester | `org/.state/served/org/runs/*.json`, `org/.state/runs/*.json` |
| R8 Usable by agents | Every step is one non-interactive `seikika run` command with `-o json`. Exit codes: 0 succeeded (or unverified), 1 failed, stale or timed out, 2 invalid or refused, 3 awaiting approval. Errors are JSON too (`{"error": "<status> <code>: …"}`). Each person is named with `--as user:<name>` (dev sign-in: the sandbox has no passwords, and a call without `--as` gets `401`). The agent skills are `seikika-docs/skills/run/SKILL.md` and `actions/SKILL.md` | all commands below |

## Commands

Set this up in every shell:

```sh
export PATH=/tmp/actions-ab/r1/seikika-2/bin:$PATH
cd /tmp/actions-ab/r1/seikika-2/org
U=http://127.0.0.1:7150
```

**Start** the org portal. It runs in the foreground, so use its own terminal (Ctrl-C stops it),
or run it in the background as below:

```sh
mkdir -p .state && nohup scripts/serve.sh > .state/serve.log 2>&1 &   # optional port: scripts/serve.sh 7151
sleep 3; grep -o 'serving .*' .state/serve.log                       # serving seikika.yaml on http://127.0.0.1:7150 …
# stop it (server and checkout sync):
kill "$(cat .state/serve-7150.pid)"
```

**List** the actions on all three repositories:

```sh
seikika run --url $U --as user:ann -o json
```

**Preview.** This changes nothing. Read `effects`, `blockers` and `approval`, and keep
`confirm_token`:

```sh
seikika run --url $U --as user:ann mark-deprecated --entity portal:alpha \
  -i successor=acme/dependabot-proxy-v2 \
  -i 'reason=Replaced by the v2 proxy, which supports OIDC registries.' -o json
```

**Request** as ann: the same command plus `--confirm <confirm_token>`. It prints the run (`id`,
`state: awaiting_approval`) and exits 3:

```sh
seikika run --url $U --as user:ann mark-deprecated --entity portal:alpha \
  -i successor=acme/dependabot-proxy-v2 \
  -i 'reason=Replaced by the v2 proxy, which supports OIDC registries.' \
  --confirm ct_… -o json
```

You can also preview and request in one step with `--yes` in place of `--confirm`. The preview is
still recorded.

**Approve** as bob. The run then executes, and `--wait` returns when it ends (0 succeeded, 1
failed). To reject instead, use `--reject <run id> --note '…'`:

```sh
seikika run --url $U --as user:bob --approve run_… --note 'ok' --wait -o json
```

**Check the status** of one run, or of all runs:

```sh
seikika run --url $U --as user:ann --verify run_… -o json   # .state
seikika run --url $U --as user:ann --runs -o json
```

**Read the record**: the `--verify` output above is the full record. To save it to a file, or to
read the stored copy:

```sh
seikika run --url $U --as user:ann --verify run_… --record /tmp/record.json -o json
cat .state/served/org/runs/run_….json
```

The same commands work for `--entity portal:beta` and `--entity portal:gamma`.

## Testing notes

- **Invalid inputs.** For example `-i successor=Acme/X`, a reason under 20 or over 200
  characters, a missing input, or `successor=acme/alpha` on alpha. The first three get `422
  invalid-inputs` (exit 2). The self-successor case shows up in the preview's `blockers` with no
  token, and `--yes` exits 1. No run is created and nothing changes.
- **Self-approval and carol.** Request as ann, then
  `seikika run --url $U --as user:ann --approve <id> -o json`, or `--as user:carol`. Both get 403
  and exit 2, and `--verify` still shows `awaiting_approval`.
- **Rejected push.** Make the origin refuse pushes:
  `printf '#!/bin/sh\nexit 1\n' > ../origins/beta.git/hooks/pre-receive && chmod +x ../origins/beta.git/hooks/pre-receive`.
  Then request and approve a run with **new** inputs, so there is a change to push. The run is
  `failed` (exit 1), its last event quotes git's `[remote rejected]`, and origin's `main` is
  unchanged. Remove the hook afterwards.
- **Reruns on these repositories.** `DEPRECATED.md` is already on `main` everywhere. New inputs
  preview an `update` diff and land like the first run. The same inputs preview no effects (a
  warning says nothing changes) and end `unverified`, because nothing new was observed.
- **Trust model.** Dev sign-in trusts `--as`, as this sandbox has no passwords. In production, set
  `SEIKIKA_AUTH_MODE` to `oidc`, `iap-google` or `token`, and give the server `SEIKIKA_OPENFGA_BIN`
  so `permission: can_run_actions` is enforced too.
