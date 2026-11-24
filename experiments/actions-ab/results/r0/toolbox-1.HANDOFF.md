# Handoff: `mark-deprecated` on Toolbox

`mark-deprecated` is live on `alpha`, `beta` and `gamma`. Each repository owns its definition
in `<repo>/.toolbox/`, committed to its origin's `main`. The org portal (`org/toolbox.yaml`) lists
the action for all three repositories and dispatches runs to them. Toolbox does the validating,
previewing, approval enforcement, dispatch and postcondition check. `org/bin/acme-actions` is the
glue Toolbox doesn't ship: it starts and stops the server, approves runs, reads them, and keeps
their records on disk. It prints JSON for every command.

All three runs were requested by ann, approved by bob, and reported `succeeded`. Each
`DEPRECATED.md` on the origins' `main` is byte-identical to the required content:

| Repository | Org run | Repository run | Commit on origin `main` |
|---|---|---|---|
| alpha | `run_2fp22olkluj6` | `run_2fp24q9oqli8` | `f7d4ee4` |
| beta | `run_2fp22tc0m7jx` | `run_2fp257s5yk2i` | `8e41c13` |
| gamma | `run_2fp22y9neb1e` | `run_2fp25ozg81yi` | `5de19b5` |

Paths below are relative to `/tmp/actions-ab/r0/toolbox-1`.

## How it fits together

```
org/bin/acme-actions ──REST──▶ toolbox serve org/ (127.0.0.1:7110, "org" portal)
                                 │  workspace connector finds */.toolbox/toolbox.yaml
                                 │  previews, approval rule, run records (in memory)
                                 ▼  on approval: dispatch (federation runner: local)
                        toolbox run mark-deprecated  in <repo>/ (the repo's own portal)
                                 │  executor: .toolbox/mark_deprecated.py → git push origin main
                                 ▼  postcondition: re-read origin's main (.toolbox/origin_state.py)
                        succeeded only if DEPRECATED.md on origin main == expected content
```

| File | What it is |
|---|---|
| `<repo>/.toolbox/toolbox.yaml` | The repository's portal: the `mark-deprecated` action (input schema, precondition, approvers, executor, postcondition) and the `origin` connector |
| `<repo>/.toolbox/mark_deprecated.py` | The executor. In preview it prints the change. On a run it checks the approval with the org portal, then pushes one commit to origin `main` without force |
| `<repo>/.toolbox/origin_state.py` | Connector job: fetches origin's `main` into `refs/toolbox/origin/main` and reports its `DEPRECATED.md` |
| `org/toolbox.yaml` | The org portal: federates every `*/.toolbox/toolbox.yaml` next to `org/` |
| `org/bin/acme-actions` | CLI glue (Python 3 standard library only) over the org portal's REST API |
| `org/.state/` (git-ignored) | `serve.log`, `serve.pid`, `org-runs/<org run>.json` and `toolbox/runs/<repository run>.json` |

## R1 to R8

| # | How it is met | Where |
|---|---|---|
| R1 Owned by the repository | Each repository's action is defined in its own `.toolbox/toolbox.yaml`, with its executor and connector scripts beside it. All of it is committed to that repository's origin `main`. The approver list is part of that file, so the team changes who approves, what the action writes and how it is verified by committing there. The org portal has no copy: it reads each checkout's definition, and `server start` first fast-forwards each clean checkout of `main` to origin `main`. `list` shows the definition's last commit on origin `main` and whether the checkout matches it | `alpha/.toolbox/`, `beta/.toolbox/`, `gamma/.toolbox/` |
| R2 Validated | Toolbox checks the input schema before any preview or run: a missing input or a `reason` outside 20 to 200 characters gets HTTP 422 `invalid-inputs` (exit 2). Toolbox 0.1.1 does **not** enforce JSON Schema `pattern`, so the `successor` pattern is a Toolbox `preconditions` entry (CEL `matches`). A bad `successor` is a preview blocker: no confirm token is issued, so no run can start (exit 2). The executor checks both inputs again before it changes anything | `<repo>/.toolbox/toolbox.yaml` (`inputs`, `preconditions`); `<repo>/.toolbox/mark_deprecated.py` (`main`) |
| R3 Previewed | The executor declares `preview: true`. With `TOOLBOX_PHASE=preview` it fetches origin `main` and prints the change as effects: create or replace, the origin and the commit it builds on, and the file's lines as a `-`/`+` diff. `preview` shows it, and `request` shows the same preview and uses its confirm token. A token expires after 10 minutes, or as soon as the preview would differ | `mark_deprecated.py` (preview branch); `acme-actions preview` / `request` |
| R4 Approved | `approval: "[['user:ann', 'user:bob']]"` means one of ann or bob, never the requester. The org portal enforces it: the requester approving gets 403 "a requester cannot approve their own run", and carol gets 403 "not one of the pending approvers" (exit 2). Only an approved run is dispatched; the repository portal accepts approvals from the org (`acceptApprovalsFrom: [org]`) and re-checks them against its own rule. Toolbox's dispatch flags (`--approved-by`, `--parent-run`) can be typed by hand in a checkout, so the executor also asks the org portal for the dispatching run before it pushes. It refuses unless that run is live, matches this repository, action, inputs and requester, and has an approval by a maintainer other than the requester | `<repo>/.toolbox/toolbox.yaml` (`approval`, `federation`); `mark_deprecated.py` (`approved_by`) |
| R5 Done means observed | The postcondition compares `entity.deprecated` with the exact expected content. That field comes from the `origin` connector, which reads origin's `main`, not the checkout. `expect.refresh` re-reads it after the executor runs. A rejected push makes the executor exit 1, so the run is `failed`. A push that appears to succeed but never reaches `main` leaves the postcondition false, so the run is `failed` after `within: 30s`. Both were tested on a scratch copy (a `pre-receive` hook that rejects, and a `post-receive` hook that resets `main`) | `<repo>/.toolbox/toolbox.yaml` (`expect`, `connectors`); `<repo>/.toolbox/origin_state.py` |
| R6 One place for the org | From `org/`: `org/bin/acme-actions list` lists `mark-deprecated` for all three repositories, and `preview`/`request <repo>` act on any of them. Without a server, `toolbox run -o json` in `org/` also lists and previews them (federation) | `org/toolbox.yaml`; `org/bin/acme-actions` |
| R7 Audited | `record <run>` gives who requested it and when, each approval (who, when, which rule), the inputs, the effects, the outcome and when it happened, the postcondition's verification, and the full timeline of the org run and the repository run. Toolbox writes the repository run to `org/.state/toolbox/runs/<id>.json`, including `via: parent:org/<org run>`. The org server keeps runs in memory only (Toolbox 0.1 stores no runs in Postgres), so the CLI saves each org run it touches to `org/.state/org-runs/<id>.json` and reads it from there after a restart. The commit on origin `main` carries `Requested-by:` and `Approved-by:` trailers and both run ids | `org/bin/acme-actions` (`record`, `runs`); `org/.state/` |
| R8 Usable by agents | Every step is a non-interactive command that prints JSON: `list`, `preview`, `request`, `approve [--wait]`, `status [--wait]`, `record`, `runs`, and `server start/stop/status`. Identity is `--as <user>` (or `ACME_USER`). Exit codes are 0 done or succeeded, 1 failed or error, 2 refused (invalid inputs, not an approver, unknown run), 3 waiting (approval or verification). The same steps are available as REST (below) | `org/bin/acme-actions` |

## Commands

Run these from `/tmp/actions-ab/r0/toolbox-1`. The CLI finds Toolbox in `bin/` itself and starts
the server from `org/` (the org connector's `root: ..` is relative to where Toolbox starts).

```bash
cd /tmp/actions-ab/r0/toolbox-1
export PATH="$PWD/bin:$PATH"
A=org/bin/acme-actions

# Start the org portal on 127.0.0.1:7110, in the background (log: org/.state/serve.log).
# For a second instance: ACME_PORTAL_LISTEN=127.0.0.1:7111 on every command.
$A server start

# List the actions of all three repositories
$A list

# Preview (changes nothing)
$A preview alpha --as ann --successor acme/dependabot-proxy-v2 \
  --reason 'Replaced by the v2 proxy, which supports OIDC registries.'

# Request a run as ann (previews, then requests; prints the run id under .run.id)
RUN=$($A request alpha --as ann --successor acme/dependabot-proxy-v2 \
  --reason 'Replaced by the v2 proxy, which supports OIDC registries.' | jq -r .run.id)

# Approve as bob (--wait: return once it has succeeded or failed)
$A approve "$RUN" --as bob --wait

# Status: .status is awaiting_approval, running, succeeded or failed; .succeeded is a boolean
$A status "$RUN"          # add --wait to block while it runs

# The audit record
$A record "$RUN"

# Every run (server and saved records)
$A runs

# Stop the server
$A server stop
```

The same steps over REST, with the identity in a header:

```bash
U=http://127.0.0.1:7110; H='content-type: application/json'
curl -s $U/v1/actions -H 'x-toolbox-user: ann'
curl -s -X POST $U/v1/actions/mark-deprecated/previews -H 'x-toolbox-user: ann' -H "$H" \
  -d '{"entity":"portal:beta","inputs":{"successor":"acme/x","reason":"twenty characters or more"}}'
curl -s -X POST $U/v1/actions/mark-deprecated/runs -H 'x-toolbox-user: ann' -H "$H" \
  -d '{"entity":"portal:beta","inputs":{...same...},"confirm_token":"ct_..."}'
curl -s -X POST $U/v1/runs/<run>/approvals -H 'x-toolbox-user: bob' -H "$H" -d '{}'
curl -s $U/v1/runs/<run> -H 'x-toolbox-user: bob'
```

## Testing it

The three repositories already have `DEPRECATED.md` with the table's content. Re-running with the
same inputs succeeds without pushing: the executor sees the content already on `main`, and the
postcondition holds. To see a push, including a rejected one, use **different inputs**.

```bash
# Invalid inputs: each exits 2 and creates no run
$A preview beta --as ann --successor 'Acme/X' --reason 'A reason that is long enough.'   # pattern: blockers, no confirm token
$A request beta --as ann --successor 'acme/x' --reason 'too short'                        # 422 invalid-inputs
$A request beta --as ann --successor 'acme/x/y' --reason 'A reason that is long enough.'  # blocked
$A request beta --as ann --reason 'A reason that is long enough.'                         # 422: successor is required

# Self-approval and carol: both exit 2 (403 not-an-approver), and the run stays awaiting_approval
RUN=$($A request beta --as ann --successor acme/beta-next --reason 'Testing the approval rules here.' | jq -r .run.id)
$A approve "$RUN" --as ann
$A approve "$RUN" --as carol
$A status "$RUN"                     # exit 3, "status": "awaiting_approval"
# carol may request; then only ann or bob can approve:
$A request gamma --as carol --successor acme/g2 --reason 'carol asks, a maintainer approves.'

# A rejected push: the run fails (exit 1), with the hook's message in .run.error
printf '#!/bin/sh\necho "main is protected" >&2\nexit 1\n' > origins/beta.git/hooks/pre-receive
chmod +x origins/beta.git/hooks/pre-receive
$A approve "$RUN" --as bob --wait    # "status": "failed", "succeeded": false
$A record "$RUN"                     # outcome.status failed, outcome.error = the rejection
rm origins/beta.git/hooks/pre-receive
git -C origins/beta.git show main:DEPRECATED.md   # unchanged

# A push that is accepted but doesn't stay on main: failed after the 30s postcondition window
printf '#!/bin/sh\nwhile read old new ref; do git update-ref "$ref" "$old"; done\n' > origins/beta.git/hooks/post-receive
chmod +x origins/beta.git/hooks/post-receive
RUN=$($A request beta --as ann --successor acme/beta-reverted --reason 'This push will be reverted by a hook.' | jq -r .run.id)
$A approve "$RUN" --as bob --wait    # "status": "failed": postcondition not met within 30s
rm origins/beta.git/hooks/post-receive
```

To get past approval without the org portal, someone could run the action in a checkout with
Toolbox's dispatch flags (`toolbox run mark-deprecated -p alpha/.toolbox ... --approved-by
'user:ann|user:bob=user:bob' --parent-run org/<id> --yes`). The executor refuses it unless
`<id>` is a live org run for the same change, approved by another maintainer.

## Limits and choices

- **Identity is claimed, not proven.** The server runs Toolbox's dev auth: `x-toolbox-user`
  (`--as`) is taken at its word, as the sandbox intends. There is no OpenFGA (`authorization:
  none`), so anyone may *request*. The approval rule is still enforced. With real sign-in,
  `--as` would become a token.
- **The origins themselves accept any push.** Anyone with filesystem access can `git push` to
  `origins/*.git` without Toolbox. Branch protection on the code host is outside this system.
- **The org server keeps run state in memory.** Restarting it loses runs that are still pending
  approval. `org/.state/` keeps every record the CLI has touched and every repository run, and
  `status`/`record` fall back to those.
- **While a repository run executes, the org run still says `awaiting_approval`, with no pending
  approvers.** The CLI reports that as `running`. `toolboxState` keeps Toolbox's raw value.
- **The approvers are listed in each repository's file.** CODEOWNERS names
  `@dependabot/maintainers` and similar teams that aren't ann and bob, and a group approver
  would need an OpenFGA model.
- **The executor's approval check needs the org portal reachable.** It uses `ACME_PORTAL_URL`,
  which `server start` sets; the default is `http://127.0.0.1:7110`.
- **Toolbox gaps found:** `pattern` is not enforced on action inputs, and a `change` executor
  can't target `main` (it proposes a new branch), so this action uses a `command` executor.
  Approving and reading runs have no CLI command, hence the glue.
