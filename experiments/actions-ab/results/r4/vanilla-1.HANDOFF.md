# Handoff: `mark-deprecated` self-service action

The system is one stdlib-only Python 3.12 CLI, `org/bin/acme`. It reads each repository's action
definition straight from that repository's origin `main`, keeps run records under `org/state/`, and
prints one JSON document for every command. **Nothing needs to be running**: there is no server or
daemon. A run executes inside the `approve` command, which pushes to the origin and then checks it.

All three runs are done and reported as `succeeded`:

| Run | Repository | Requested by | Approved by | Commit on origin `main` |
|---|---|---|---|---|
| `run-000001` | alpha | ann | bob | `e34048c4cd43` |
| `run-000002` | beta | ann | bob | `6963841c45bf` |
| `run-000003` | gamma | ann | bob | `3fb7ba401901` |

## Layout

| Path | What it is |
|---|---|
| `<repo>/.acme/actions/mark-deprecated.json` (on `origins/<repo>.git` `main`) | The action definition: input rules, approval policy, the file template, the commit message and the check |
| `<repo>/.acme/repo.json` (on `main`) | The repository's maintainers (`ann`, `bob`), who are the approvers |
| `org/bin/acme` | The CLI: list, describe, preview, request, approve, reject, cancel, status, show, runs, audit |
| `org/acme.json` | The org registry: the org name (`acme`), the people (ann, bob, carol), the three repositories and their origins, and the bot identity that commits (`acme-actions`) |
| `org/state/runs/run-NNNNNN.json` | One record per run (gitignored runtime state) |
| `org/state/audit.jsonl` | The org-wide append-only audit log, hash-chained |

## How each requirement is met

| # | How |
|---|---|
| R1 Owned by the repository | Each repository's definition is `.acme/actions/mark-deprecated.json` and its maintainers are in `.acme/repo.json`. Both were committed to each origin's `main`: alpha `27e3cec`, beta `c878fbf`, gamma `168fc98`. `acme` copies neither file. It reads them with `git cat-file` from `refs/heads/main` of `origins/<repo>.git`, so a commit to the repository's `main` changes the input rules, the template or the approvers. Each run pins the definition's commit and blob (`definition_ref` in the record). |
| R2 Validated | `validate_inputs` in `org/bin/acme` applies the definition's rules before it writes anything: `successor` must fully match `^[a-z0-9-]+/[a-z0-9-]+$` and must not be the repository itself (`notOneOf: ["{{repo.full_name}}"]`); `reason` must be 20 to 200 characters inclusive. Unknown or missing inputs, and control characters or newlines, are also refused. `preview` and `request` both validate, and a failure exits with code 2 and `error.kind = "invalid_inputs"`, listing every error. No run record is created and nothing is pushed. A request whose content is already on `main` is refused with `no_change` (exit 5). |
| R3 Previewed | `acme preview` returns the change against the current `main` commit: `change` (`create` or `modify`), the full new `content`, its sha256 and a unified `diff`. `request` stores the same preview in the run record, so the approver can see it with `acme show` before approving. If `main` moves before approval, the change is recomputed and the new diff is recorded in `execution.diff`. |
| R4 Approved | A run starts `pending_approval`, and only `approve` executes it. `cmd_approve` refuses the requester (`self_approval`, exit 3), and refuses anyone not in the pinned run's approvers or not in `.acme/repo.json` on current `main` (`not_maintainer`, exit 3, which is what carol gets). People not in `org/acme.json` are refused too (`unknown_person`). Every refused attempt is recorded as an `approval_denied` event. A run can be approved only once (`not_pending`, exit 5). |
| R5 Done means observed | `execute` clones the origin, commits `DEPRECATED.md` with git plumbing and pushes `<commit>:refs/heads/main`. Then it reads `DEPRECATED.md` from the origin's own `refs/heads/main` and compares the bytes to the expected content. `succeeded` requires both a successful push and a byte-for-byte match. A rejected push, a git error, or a push that is accepted but does not leave the right file on `main` gives `failed`, with the push output in `execution.push` and what was seen in `verification`. Both paths were tested on throwaway copies of the origins, with a rejecting `pre-receive` hook and with a `post-receive` hook that rewinds `main`. |
| R6 One place for the org | Everything runs from `org/`. `./bin/acme list` reads all three origins and lists their actions, and `request <repo> mark-deprecated ...` works for any repository in `org/acme.json` without entering its checkout. |
| R7 Audited | The run record (`org/state/runs/<id>.json`) has `requested_by`/`requested_at`, `approved_by`/`approved_at`, `inputs`, `outcome` (`status`, `at`, `message`), `finished_at` and timestamped `events` (`requested`, `approval_denied`, `approved`, `execution_started`, then `succeeded` or `failed`). Each event is also appended to `org/state/audit.jsonl`, where every entry hashes the one before it. `acme audit` checks that chain. The commit on `main` is authored by the requester, committed by `acme-actions`, and carries `Acme-Run:`, `Requested-by:` and `Approved-by:` trailers. |
| R8 Usable by agents | Every subcommand is non-interactive: identity comes from `--as <person>` or `$ACME_AS`, and inputs from `-i key=value` or `--inputs-json '{...}'`. Every command, including usage errors, prints one JSON document on stdout with `"ok": true` or `false`. Exit codes: 0 ok, 1 run failed, 2 invalid input or usage, 3 permission denied, 4 not found, 5 wrong state. |

## Commands

Run these from the org directory. Nothing needs starting.

```bash
cd /tmp/actions-ab/r4/vanilla-1/org

# List the actions for all three repositories (or --repo alpha)
./bin/acme list
./bin/acme describe alpha mark-deprecated

# Preview (validates; changes nothing)
./bin/acme preview alpha mark-deprecated \
  -i successor=acme/dependabot-proxy-v2 \
  -i "reason=Replaced by the v2 proxy, which supports OIDC registries."

# Request as ann: prints the run with its id ("run.id") and preview; status pending_approval
./bin/acme request alpha mark-deprecated --as ann \
  -i successor=acme/dependabot-proxy-v2 \
  -i "reason=Replaced by the v2 proxy, which supports OIDC registries."
# (alternative input form)
./bin/acme request beta mark-deprecated --as ann \
  --inputs-json '{"successor":"acme/valtio-store","reason":"State management moves to the shared store package."}'

# Approve as bob: executes the run, then checks origin main; exit 0 only if it succeeded
./bin/acme approve run-000001 --as bob

# Status, full record, all runs, audit log
./bin/acme status run-000001
./bin/acme show run-000001
./bin/acme runs                 # --repo alpha, --status pending_approval
./bin/acme audit                # --run run-000001; "chain_intact": true

# Reject (a maintainer) or cancel (the requester) a pending run
./bin/acme reject run-000004 --as bob --comment "not yet"
./bin/acme cancel run-000004 --as ann
```

To pull out a single field, pipe to `python3 -c 'import json,sys; print(json.load(sys.stdin)["status"])'`
(or `jq -r .status` if jq is installed).

## Testing the refusals

The three repositories are already deprecated with the inputs above. Requesting the same inputs again
is refused with `no_change`, so use a different `reason` or `successor` for new runs.

```bash
cd /tmp/actions-ab/r4/vanilla-1/org

# Invalid inputs: exit 2, nothing recorded or pushed
./bin/acme request alpha mark-deprecated --as ann -i successor=Acme/Proxy -i reason=short
./bin/acme request alpha mark-deprecated --as ann -i successor=acme/alpha -i "reason=Pointing at itself is refused."
./bin/acme preview alpha mark-deprecated -i successor=acme/x -i "reason=$(printf 'x%.0s' $(seq 201))"

# A new pending run (take the id from .run.id)
ID=$(./bin/acme request alpha mark-deprecated --as ann -i successor=acme/proxy-v3 \
  -i "reason=Testing approval rules and a rejected push." | python3 -c 'import json,sys; print(json.load(sys.stdin)["run"]["id"])')

./bin/acme approve $ID --as ann     # self-approval: exit 3, kind self_approval
./bin/acme approve $ID --as carol   # not a maintainer: exit 3, kind not_maintainer

# Rejected push: make the origin refuse pushes, then approve as bob
printf '#!/bin/sh\necho "main is protected" >&2\nexit 1\n' > ../origins/alpha.git/hooks/pre-receive
chmod +x ../origins/alpha.git/hooks/pre-receive
./bin/acme approve $ID --as bob     # exit 1; status failed; execution.push shows the rejection
./bin/acme status $ID
rm ../origins/alpha.git/hooks/pre-receive
```

Without the hook, that last approval would really replace alpha's `DEPRECATED.md`. To put it back,
request the original alpha inputs again and approve them.

## Notes and limits

- **Identity is asserted, not authenticated.** This sandbox has no passwords, so `--as` is trusted.
  The approval rules and the audit trail are enforced by the CLI. Anyone with write access to
  `org/state/` could edit a record by hand, but `acme audit` would then report the hash chain broken
  for edits to the log.
- **The file content** is exactly `# Deprecated: use <successor>\n\n<reason>\n`, ending in one newline.
  To change it, edit the repository's definition template on `main`.
- **Execution is synchronous** inside `approve`, under a file lock (`org/state/lock`), so two approvals
  cannot race. If `main` moves between the clone and the push, the push is rejected as
  non-fast-forward and the run is `failed`. Request it again.
- **Test overrides:** `ACME_CONFIG=<path>` points at another registry, with origins relative to it, and
  `ACME_STATE_DIR=<dir>` keeps records somewhere else. These were used to test the failure paths on
  throwaway copies of the origins.
