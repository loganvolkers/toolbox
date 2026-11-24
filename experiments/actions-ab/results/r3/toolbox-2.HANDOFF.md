# mark-deprecated: handoff

`mark-deprecated` is a Toolbox self-service action. Each repository declares and owns its copy in
`.toolbox/toolbox.yaml` on its origin's `main`. The org portal in this directory
(`org/toolbox.yaml`) offers the three copies from one place, records who requested and approved
each run, and runs approved ones in its own clone of the repository. No glue code was needed. The
whole system is three repository manifests, one org manifest and the Toolbox binaries in
`/tmp/actions-ab/r3/toolbox-2/bin`.

All three runs, requested by ann and approved by bob, are `succeeded`:

| Repository | Org run (use with `--verify`) | Repository run | Commit on `origins/<repo>.git` `main` |
|---|---|---|---|
| alpha | `run_2fs8noobxudf` | `run_2fs8o03y8a9w` | `bb0a224` |
| beta | `run_2fs8p0dppttx` | `run_2fs8p68zuj4x` | `b72c488` |
| gamma | `run_2fs8q91hzpsp` | `run_2fs8qetej1uw` | `f0cf17a` |

The org server also holds three test runs: `run_2fs8cfmubqnf` (requested by ann, then rejected by
bob after ann's and carol's approvals were refused), `run_2fs8fbkldwv1` (requested by carol and
approved by bob; it **failed** because the origin rejected the push) and `run_2fs913tgbak0` (a
repeat with identical inputs; it ended `unverified` because it changed nothing). Since that last
test, the action refuses such a no-op at preview. `run_2fsa5mnm7r98` (rejected) and
`run_2fsa5vjvd0fc` (failed, push rejected on gamma) came from running the refusal commands below,
exactly as written.

## Files

| Path | What it is |
|---|---|
| `/tmp/actions-ab/r3/toolbox-2/{alpha,beta,gamma}/.toolbox/toolbox.yaml` | Each repository's definition of `mark-deprecated`: inputs, approvers, the command, and the org key it trusts. Committed to `origins/<repo>.git` `main` |
| `/tmp/actions-ab/r3/toolbox-2/org/toolbox.yaml` | The org portal: lists the three origins as `federation.children` (`portal:alpha`, `portal:beta`, `portal:gamma`). Committed to `origins/org.git` `main` |
| `/tmp/actions-ab/r3/toolbox-2/.state/served/org/runs/<run id>.json` | The org server's run records (the audit trail) |
| `/tmp/actions-ab/r3/toolbox-2/.state/served/org/signing-key` | The key the org server signs approvals with. The repositories pin its public half |
| `/tmp/actions-ab/r3/toolbox-2/.state/served/org/serve.log`, `serve.pid` | The server's log and pid |
| `/tmp/actions-ab/r3/toolbox-2/.state/federation/` | The org server's clones of the three repositories |
| `/tmp/actions-ab/r3/toolbox-2/.state/runs/` | The repository-level run records (the child half of each run) |

## How each requirement is met

| # | How | Where |
|---|---|---|
| R1 Owned by the repository | The action, its inputs schema, approvers and command live in each repository's `.toolbox/toolbox.yaml`, committed to its origin's `main`. The org portal declares no actions. It re-reads each child's `main` before it lists, previews or runs. A repository changing its definition takes effect on the next request, with no org change; this was exercised once, by commit "mark-deprecated: refuse a run that would change nothing" in each repository | `{alpha,beta,gamma}/.toolbox/toolbox.yaml`, `org/toolbox.yaml` |
| R2 Validated | `inputs` is a JSON Schema: both inputs `required`, `successor` with `pattern: ^[a-z0-9-]+/[a-z0-9-]+$`, `reason` with `minLength: 20` and `maxLength: 200`, and `additionalProperties: false`. Toolbox checks it before the preview, so nothing runs: `422 invalid-inputs`, exit 2. Two preconditions, also checked before anything changes: the successor must not be the repository itself (`!inputs.successor.endsWith('/<repo>')`), and the command refuses a run that would leave `DEPRECATED.md` unchanged. Both produce a `Blocked:` preview, and `--yes` exits 2 without creating a run | `inputs`, `preconditions`, `executor.run` in each `.toolbox/toolbox.yaml` |
| R3 Previewed | Without `--yes` or `--confirm`, `toolbox run` previews: it runs the command in a throwaway worktree of `main` and prints the exact diff of `DEPRECATED.md`, the push to `main`, the approvers, any blockers and a confirm token. Nothing is pushed. With `--yes`, the preview is recorded as the run's first event. The run later fails rather than commit anything that differs from the confirmed preview | `toolbox run … mark-deprecated --entity portal:<repo> -i …` |
| R4 Approved | `approval: "[['user:ann', 'user:bob']]"` means one of ann or bob, and the requester never counts. The run waits (`awaiting_approval`, exit 3) until a maintainer other than the requester approves on the org server. Self-approval gets `403 not-an-approver: a requester cannot approve their own run`. carol gets `403 not-an-approver: user:carol is not one of the pending approvers`. The org server signs the approval, and the repository accepts it only with the key pinned in its `federation.acceptApprovalsFrom`. Running from a repository checkout can't bypass this: it stops at `awaiting_approval`, exit 3, because a repository portal holds no approvals | `approval`, `federation.acceptApprovalsFrom` in each `.toolbox/toolbox.yaml` |
| R5 Done means observed | `executor: {type: change, land: push, base: main}` commits in a fresh clone and pushes to `main` (never forced). The run is `succeeded` only after Toolbox fetches the origin's `main`, finds the commit on it, and finds `DEPRECATED.md` with the blob the preview showed (`verification.files[].matches: true` in the record). A rejected push fails the run (`failed`, exit 1) with the origin's message. A run that changed nothing is `unverified` (exit 4), never `succeeded` | `executor` in each `.toolbox/toolbox.yaml`, `verification` in the run record |
| R6 One place for the org | `org/toolbox.yaml` lists `../origins/{alpha,beta,gamma}.git` as federation children. One server (`toolbox serve org`) lists all three `mark-deprecated` actions and takes requests, approvals and status checks for any of them, with `--entity portal:<repo>`. `toolbox run` in `org/` without a server lists and previews them too | `org/toolbox.yaml` |
| R7 Audited | Each run's record (`--verify <id>`, or the file under `.state/served/org/runs/`) has `requested_by`, `approval.approved_by` (or `rejected_by`), `inputs`, `state`, `error` and `verification`, and a timestamped `events` list: previewed, confirmed, awaiting_approval, approved (by whom, with the note), dispatched, running, succeeded or failed. The repository's half of the run is nested under `child`. Each commit also carries `Toolbox-Run`, `Requested-By` and `Approved-By` trailers | `.state/served/org/runs/`, `git -C origins/<repo>.git log -1 main` |
| R8 Usable by agents | Every step is one non-interactive `toolbox run` command. All of them take `-o json`, and exit codes are stable: 0 succeeded, 1 failed or stale, 2 invalid or refused, 3 awaiting approval, 4 unverified. On this loopback server, `--as <person>` (or `TOOLBOX_AS`) says who is acting | the commands below |

## Commands

Set up the shell once. The state directory must be this one, because it holds the approval key the
repositories pin and the run records. A server started with another state directory gets a new key,
and its runs are refused with `child-refuses-approvals`.

```sh
cd /tmp/actions-ab/r3/toolbox-2/org
export PATH=/tmp/actions-ab/r3/toolbox-2/bin:$PATH
export TOOLBOX_STATE_DIR=/tmp/actions-ab/r3/toolbox-2/.state
export XDG_STATE_HOME=/tmp/actions-ab/r3/toolbox-2/.state/xdg
U=http://127.0.0.1:7750
```

Start the server. It returns once the server answers and prints
`approval key: ed25519:100228bea8140d55dba0c09b8c9c9b77439642b6b7c72a91f27be7e607178de9`. Stop it
when you are done.

```sh
toolbox serve /tmp/actions-ab/r3/toolbox-2/org --listen 127.0.0.1:7750 --detach
toolbox serve /tmp/actions-ab/r3/toolbox-2/org --stop
```

List the actions for all three repositories:

```sh
toolbox run --url $U --as ann -o json
```

Preview. This changes nothing and prints the diff and a confirm token:

```sh
toolbox run --url $U --as ann mark-deprecated --entity portal:alpha \
  -i successor=acme/dependabot-proxy-v3 -i reason="Replaced by the v3 proxy, which supports OIDC registries." -o json
```

Request as ann. This exits 3 (`awaiting_approval`) and prints the run's `id`. `--confirm <token>`
from the preview works in place of `--yes`.

```sh
toolbox run --url $U --as ann mark-deprecated --entity portal:alpha \
  -i successor=acme/dependabot-proxy-v3 -i reason="Replaced by the v3 proxy, which supports OIDC registries." --yes -o json
```

Approve as bob. This runs the action and waits until it settles: exit 0 and `"state": "succeeded"`,
or exit 1 and `"state": "failed"`. To reject instead, use `--reject <run id>`.

```sh
toolbox run --url $U --as bob --approve <run id> --note "agreed" --wait -o json
```

Check a run's status and read its record (`state`, `requested_by`, `approval`, `inputs`, `events`,
`verification`). The second command lists every run.

```sh
toolbox run --url $U --as ann --verify <run id> -o json
toolbox run --url $U --as ann --runs -o json
```

Confirm what landed:

```sh
git -C /tmp/actions-ab/r3/toolbox-2/origins/alpha.git show main:DEPRECATED.md
git -C /tmp/actions-ab/r3/toolbox-2/origins/alpha.git log -1 main
```

All three repositories are already deprecated with the inputs from the task. Repeating those exact
inputs is refused at preview ("nothing to change"), so use a different successor or reason to test
a new run. That run updates `DEPRECATED.md`.

## Testing the refusals

The results below were observed with these commands:

```sh
# Invalid inputs: 422 invalid-inputs, exit 2, no run
toolbox run --url $U --as ann mark-deprecated --entity portal:beta -i successor='Acme/x y' -i reason=short --yes -o json
toolbox run --url $U --as ann mark-deprecated --entity portal:beta -i successor=acme/x -i reason="$(printf 'x%.0s' {1..201})" --yes -o json
toolbox run --url $U --as ann mark-deprecated --entity portal:beta -i successor=acme/x --yes -o json   # missing reason
# The successor is the repository itself: Blocked, exit 2, no run
toolbox run --url $U --as ann mark-deprecated --entity portal:beta -i successor=acme/beta -i reason="A reason that is long enough." --yes -o json

# Self-approval and carol: 403 not-an-approver, exit 2; the run stays awaiting_approval
RUN=$(toolbox run --url $U --as ann mark-deprecated --entity portal:beta -i successor=acme/valtio-store-v2 \
  -i reason="State management moves to the v2 store package." --yes -o json | python3 -c 'import json,sys; print(json.load(sys.stdin)["id"])')
toolbox run --url $U --as ann   --approve $RUN -o json
toolbox run --url $U --as carol --approve $RUN -o json
toolbox run --url $U --as bob   --reject  $RUN --note "test" -o json   # "state": "rejected", exit 1

# Rejected push: the run is failed (exit 1) with the origin's message, and main is unchanged
H=/tmp/actions-ab/r3/toolbox-2/origins/gamma.git/hooks/pre-receive
printf '#!/bin/sh\necho "main is protected" >&2\nexit 1\n' > $H && chmod +x $H
RUN=$(toolbox run --url $U --as ann mark-deprecated --entity portal:gamma -i successor=acme/graphify-next \
  -i reason="Graph extraction now lives in graphify-next." --yes -o json | python3 -c 'import json,sys; print(json.load(sys.stdin)["id"])')
toolbox run --url $U --as bob --approve $RUN --wait -o json; echo "exit=$?"
toolbox run --url $U --as ann --verify $RUN -o json
rm $H
```

## Limits

- People sign in with `--as` because the server listens on loopback, which Toolbox treats as dev
  sign-in. That matches this sandbox, where saying who you are is enough. On any other address,
  set `TOOLBOX_AUTH_MODE` (`toolbox serve --help`).
- Anyone with push access to an origin can still change `main` directly. Toolbox governs runs of
  the action, not the code host. In a real deployment, branch protection would leave the org
  runner as the only path to `main`.
- Without `--database-url`, run records are JSON files under `TOOLBOX_STATE_DIR`. They survive a
  restart: this was checked by stopping and restarting the server, after which the key and the
  records were unchanged.
