# Handoff: `mark-deprecated`, a self-service action on Toolbox

**Status.** `mark-deprecated` is defined in each of `alpha`, `beta` and `gamma`, and committed to its
origin's `main`. It is listed and run from the org portal in this directory. All three requested runs
were requested by ann, approved by bob, and reported `succeeded` after `DEPRECATED.md` was observed
byte for byte on each origin's `main`:

| Repository | Run | Commit on origin `main` | Record |
|---|---|---|---|
| alpha | `run_2fr2ch41gqgm` | `17bc9f2` | [runs/alpha-run_2fr2ch41gqgm.json](runs/alpha-run_2fr2ch41gqgm.json) |
| beta | `run_2fr2e4m0yz3n` | `c9780b1` | [runs/beta-run_2fr2e4m0yz3n.json](runs/beta-run_2fr2e4m0yz3n.json) |
| gamma | `run_2fr2fp9cl8i8` | `58e6642` | [runs/gamma-run_2fr2fp9cl8i8.json](runs/gamma-run_2fr2fp9cl8i8.json) |

There is one more run on the server: `run_2fr2n01qld47` (alpha). It was a live check of the approval
rules: ann's own approval and carol's were refused, then bob rejected it, so nothing landed.

No server is running now. Start one with the commands in [How to use it](#how-to-use-it).

## How it fits together

```
/tmp/actions-ab/r1/toolbox-3/
├── alpha/.toolbox/toolbox.yaml     alpha's portal: mark-deprecated, owned by alpha (on origin main)
├── alpha/.toolbox/origin-main.py   reads DEPRECATED.md from origin's main for verification
├── beta/.toolbox/…, gamma/.toolbox/…   the same, one copy per repository
└── org/
    ├── toolbox.yaml                the org portal: finds the three repo portals, offers and dispatches their actions
    ├── bin/serve, bin/stop         start/stop `toolbox serve` on 127.0.0.1:7190
    ├── bin/actions                 one non-interactive CLI for every step (JSON out)
    ├── bin/install-action          bootstraps a repository's copy from templates/ (commit + push)
    ├── bin/approval-pubkey         prints the org's public approval key (what repositories pin)
    ├── templates/                  the starting definition install-action renders
    ├── runs/                       exported records of the three runs above
    ├── .secrets/approval-key       the org's private signing key (git-ignored, mode 600)
    └── .state/                     the server's run store, log and pid (git-ignored)
```

- **Repositories.** Each repository is a Toolbox *repo portal*. `mark-deprecated` is a `change`
  executor with `land: push`. Once the approval rule is met it commits `DEPRECATED.md` in a worktree
  of `origin/main` and pushes it to the origin's `main`, never forced. The action's
  `expect.postcondition` then compares the file's exact content on the origin's `main` with the
  inputs.
- **Org portal.** The org portal (`org/toolbox.yaml`) is a Toolbox *parent portal*. Its workspace
  connector reads `alpha/.toolbox/toolbox.yaml`, `beta/…` and `gamma/…`, so it offers each
  repository's actions on `portal:<repo>`. It runs them with `toolbox run` in that repository's
  checkout (`federation.runners.local`).
- **Approvals.** The org portal holds the approvals and signs them with `org/.secrets/approval-key`.
  Each repository accepts only approvals signed by the key it pins in
  `federation.acceptApprovalsFrom`.
- **Auth.** The server runs in Toolbox dev auth mode (`TOOLBOX_AUTH_MODE=dev`): you say who you are
  with `--as user:<name>`, and Toolbox records it on every event. There are no passwords in this
  sandbox.

## R1 to R8

| # | How it is met | Where |
|---|---|---|
| R1 Owned by the repository | Each repository's definition is its own file, committed to its origin's `main` (commits `24000c5` alpha, `782fdd5` beta, `e3b501b` gamma). CODEOWNERS gives `/.toolbox/` to `@ann @bob`. The team changes the action by committing to that file. The org portal reads each repository's checkout, and `bin/actions` fast-forwards that checkout from origin `main` and refreshes the server before every preview and request, so a pushed change applies to the next run. I tested this: lowering `maxLength` on a scratch origin made the next preview refuse a 150-character reason, and reverting it accepted the reason again. `org/templates/` is only the bootstrap copy. | `alpha/.toolbox/toolbox.yaml`, `beta/.toolbox/toolbox.yaml`, `gamma/.toolbox/toolbox.yaml`; `alpha/CODEOWNERS`, `beta/.github/CODEOWNERS`, `gamma/.github/CODEOWNERS`; `org/bin/actions` (`sync_repo`) |
| R2 Validated | The inputs are a JSON Schema: `successor` uses `pattern: ^[a-z0-9-]+/[a-z0-9-]+$`, and `reason` uses `minLength: 20` and `maxLength: 200`. A precondition `!inputs.successor.endsWith('/<repo>')` makes the successor *another* repository. Toolbox checks these at preview and again in the child at run time. Invalid inputs get `422 invalid-inputs` (exit 2) with no confirm token. A failed precondition is a blocker, and `bin/actions` refuses it (exit 2). Nothing is committed or pushed in either case. | `actions[0].inputs` and `preconditions` in each `<repo>/.toolbox/toolbox.yaml` |
| R3 Previewed | `preview` runs the change command in a scratch worktree and returns the effects: the `DEPRECATED.md` diff, the push to `main`, the approvers, the postcondition and a confirm token. It changes nothing. `request` previews first and returns the preview with the run. | `toolbox run mark-deprecated --url … --entity portal:<repo> -i …`; `org/bin/actions preview` / `request` |
| R4 Approved | `approval: "[['user:ann', 'user:bob']]"` means any one of ann or bob, and Toolbox never counts the requester. A run waits in `awaiting_approval` until then. ann approving her own run gets `403 not-an-approver: a requester cannot approve their own run`. carol gets `403 not-an-approver: user:carol is not one of the pending approvers`. Approvals are recorded by the org server and passed to the repository signed, and the repository checks the signature against its pinned key. | `approval` in each `<repo>/.toolbox/toolbox.yaml`; `federation` in `org/toolbox.yaml` and in each repo portal |
| R5 Done means observed | A run reports `succeeded` only when both checks hold. (1) The `land: push` commit is seen on `origin/main`. A rejected push ends the run `failed` ("the push to origin main was refused, so nothing landed", `commit: null`). (2) The postcondition holds: `entity.deprecatedMd == '# Deprecated: use ' + successor + '\n\n' + reason + '\n'`. `deprecatedMd` comes from the `origin-main` connector, which runs `git fetch origin main` and then `git show origin/main:DEPRECATED.md`. It never reads the checkout. Verification retries until `within: 2m`, then fails. I tested both on scratch origins: a pre-receive hook gave `failed`, and an executor that wrote extra bytes landed the commit but the run ended `failed` ("postcondition not met"). Rerunning with inputs whose file is already on `main` gives a preview warning, and the run ends `unverified`, never `succeeded`. | `executor` and `expect` in each `<repo>/.toolbox/toolbox.yaml`; `<repo>/.toolbox/origin-main.py` |
| R6 One place for the org | `org/toolbox.yaml` discovers the three repository portals and offers their actions as `portal:alpha`, `portal:beta` and `portal:gamma`. From `org/`, `bin/actions list` lists all three, and `bin/actions request <repo>` runs on any of them. The raw equivalent is `toolbox run --url http://127.0.0.1:7190 …`. `toolbox run -o json` in `org/` lists them even without a server. | `org/toolbox.yaml`, `org/bin/actions` |
| R7 Audited | Every run record has `requested_by`, `inputs`, `state`, `error`, `outputs.commit`, the verification (`held`, `held_before`, the commit observed on base), and timestamped `events` with `by`: previewed, confirmed, awaiting_approval, approved or rejected, dispatched, running, succeeded or failed. `bin/actions record <run>` prints that record plus an `audit` summary: who requested and when, who approved and when, the inputs, the outcome and when. Records live in `org/.state/served/org/runs/` and survive a restart. The repository side keeps its own record in `org/.state/runs/`. Landed commits carry `Toolbox-Run:` and `Requested-By:` trailers, and the author is "Acme org portal". | `org/.state/served/org/runs/*.json`, `org/runs/*.json`, `org/bin/actions` (`record`) |
| R8 Usable by agents | Every step is one non-interactive command with JSON on stdout and meaningful exit codes: 0 ok or succeeded, 1 failed, 2 invalid or refused, 3 awaiting approval. That holds for both `org/bin/actions` and raw `toolbox … -o json`. `status --wait` blocks until the run settles or waits for a person. | `org/bin/actions`; `toolbox run --help` |

## How to use it

Run everything from a shell. Paths are absolute, so the working directory doesn't matter unless a
step says so.

```bash
export PATH=/tmp/actions-ab/r1/toolbox-3/bin:$PATH
cd /tmp/actions-ab/r1/toolbox-3/org
```

### Start (the only thing that must be running)

```bash
bin/serve --background      # toolbox serve on http://127.0.0.1:7190; prints {"url","pid","log"}
# or, in a terminal of its own:  bin/serve
# second instance:               TOOLBOX_PORT=7191 bin/serve --background  (then TOOLBOX_URL=http://127.0.0.1:7191)
```

`bin/serve` does four things before it starts the server:

- fast-forwards the three checkouts from origin `main`;
- sets `TOOLBOX_AUTH_MODE=dev` and `TOOLBOX_STATE_DIR=org/.state`;
- loads `TOOLBOX_APPROVAL_KEY` from `org/.secrets/approval-key`;
- sets the git author for landed commits.

Keep that key file: the repositories pin its public half. If it is lost, create a new one with
`openssl rand -hex 32 > .secrets/approval-key`, then rerun `bin/install-action alpha beta gamma` to
re-pin it.

### List the actions

```bash
bin/actions list
# raw Toolbox: toolbox run --url http://127.0.0.1:7190 --as user:ann -o json
#              toolbox actions portal:alpha --url http://127.0.0.1:7190 --as user:ann -o json
```

### Preview (changes nothing)

```bash
bin/actions preview alpha --successor acme/dependabot-proxy-v2 \
  --reason "Replaced by the v2 proxy, which supports OIDC registries." --as user:ann
# raw: toolbox run mark-deprecated --url http://127.0.0.1:7190 --as user:ann --entity portal:alpha \
#        -i successor=acme/dependabot-proxy-v2 -i "reason=Replaced by the v2 proxy, which supports OIDC registries." -o json
```

### Request, as ann

```bash
bin/actions request alpha --successor acme/dependabot-proxy-v2 \
  --reason "Replaced by the v2 proxy, which supports OIDC registries." --as user:ann
# -> {"run": {"id": "run_…", "state": "awaiting_approval", …}, "preview": {…}}   exit 3
RUN=$(bin/actions request beta --successor acme/valtio-store \
  --reason "State management moves to the shared store package." --as user:ann | jq -r .run.id)
# raw: preview as above, then add  --confirm <confirm_token from the preview>  (valid 10 minutes, used once)
```

### Approve, as bob

```bash
bin/actions approve "$RUN" --as user:bob
# raw: toolbox run --url http://127.0.0.1:7190 --as user:bob --approve "$RUN" -o json
# reject instead: bin/actions reject "$RUN" --as user:bob --note "why"
```

### Check a run's status

```bash
bin/actions status "$RUN"           # {"id","state","error","pending_approvers","commit"}; exit 0 succeeded, 1 failed, 3 waiting
bin/actions status "$RUN" --wait    # until it succeeds, fails or waits for a person
bin/actions runs                    # every run
# raw: toolbox run --url http://127.0.0.1:7190 --as user:bob --verify "$RUN" --wait -o json
#      toolbox run --url http://127.0.0.1:7190 --as user:bob --runs -o json
```

### Read its record

```bash
bin/actions record "$RUN" | jq .audit    # who requested/approved and when, inputs, outcome and when, commit, observed_on_main
bin/actions record "$RUN" | jq .record   # the full Toolbox run record
# raw: curl -s -H 'x-toolbox-user: user:bob' http://127.0.0.1:7190/v1/runs/$RUN
```

### Stop

```bash
bin/stop
```

## Testing the guarantees

All three repositories already have `DEPRECATED.md` on `main`. Use a different successor or reason
so that a run has something to change, or the run will end `unverified` (R5).

```bash
# R2: invalid inputs: 422 / exit 2, no run created
bin/actions request alpha --successor "Not A Repo" --reason "too short" --as user:ann
bin/actions request alpha --successor acme/x --reason "$(printf 'x%.0s' {1..201})" --as user:ann
bin/actions request alpha --successor acme/alpha --reason "A successor that is this repository itself." --as user:ann   # blocker / exit 2

# R4: self-approval and carol: 403 / exit 2, the run stays awaiting_approval
RUN=$(bin/actions request gamma --successor acme/graphify-next --reason "Testing the approval rules for this action." --as user:ann | jq -r .run.id)
bin/actions approve "$RUN" --as user:ann
bin/actions approve "$RUN" --as user:carol
bin/actions status "$RUN"

# R5: a rejected push: the run ends "failed", never "succeeded"
printf '#!/bin/sh\necho "main is protected" >&2\nexit 1\n' > ../origins/gamma.git/hooks/pre-receive
chmod +x ../origins/gamma.git/hooks/pre-receive
bin/actions approve "$RUN" --as user:bob
bin/actions status "$RUN" --wait          # state: failed, error: "the push to origin main was refused …", exit 1
rm ../origins/gamma.git/hooks/pre-receive
```

## Notes and limits

- **Identity is self-declared.** Dev auth trusts `--as`, as the brief allows. With real sign-in
  (OIDC or IAP, plus `TOOLBOX_OPENFGA_BIN` for authorization) the same approval rules apply to
  verified identities. Toolbox's run CLI says approvals then need an interactive sign-in.
- **Server state.** The entity store is in memory and is rebuilt from the checkouts at start. Runs,
  spent confirm tokens and child run records are files under `org/.state/`. Set `DATABASE_URL` (Postgres)
  for a shared store.
- **Bypassing the wrapper.** Raw `toolbox run --url` does not fast-forward the checkouts. If a team has
  just pushed a change to its definition, run `bin/actions sync` first, or the server reads the
  checkout as it was. Raw `toolbox` can also hand out a confirm token when the org portal's cached
  schema is stale but the child refuses the inputs. Such a run fails in the child before any change,
  and `bin/actions` refuses it up front.
- **Repeat runs.** A second run with inputs whose file is already on `main` is previewed with a
  warning and ends `unverified`.
