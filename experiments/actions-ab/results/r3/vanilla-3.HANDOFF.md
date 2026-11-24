# HANDOFF: `mark-deprecated`, a repository-owned self-service action

The system is one stdlib-only Python 3 CLI, `org/acme`. **Nothing has to be running**: there is no
server or daemon, so there is nothing to start or stop. Every command prints JSON on stdout,
including errors, and exits with a meaningful code.

The three requested runs succeeded. Each was requested by ann, approved by bob and observed on the
origin's `main`:

| Repository | Run id | Observed commit on `origins/<repo>.git` `main` |
|---|---|---|
| alpha | `run-20261124T121105Z-0c1c63` | `6430efe49b44a5cdec7d2be4b7baa6f6803198af` |
| beta | `run-20261124T121105Z-e2a283` | `d11d823833a7342b002514791e44cde543d9870b` |
| gamma | `run-20261124T121105Z-60bdf6` | `09419e920c55e10f3a81b90d44c8d62d4753e113` |

## Layout

| Path | What it is |
|---|---|
| `<repo>/.acme/actions/mark-deprecated.json` (on `origins/<repo>.git` `main`) | The repository's own definition: maintainers, approval rule, input validation, the file template, the target branch |
| `org/acme` | The org entry point (list, describe, preview, request, approve, reject, status, record, runs, audit) |
| `org/repos.json` | The org's repositories: name, `owner/name` slug, origin path |
| `org/people.json` | Known people (ann, bob, carol). An unknown identity is refused |
| `org/.acme/state/runs/<run-id>.json` | One record per run (gitignored state) |
| `org/.acme/state/audit.jsonl` | Append-only audit log of every event, including refused requests and refused approvals |
| `org/.acme/state/work/` | Temporary clones used during a run, deleted afterwards |

Set `ACME_STATE_DIR=/some/dir` to keep state somewhere else, for example a throwaway test.

## How each requirement is met

| # | How |
|---|---|
| R1 Owned by the repository | The definition is `.acme/actions/mark-deprecated.json` in each repo, committed to `origins/{alpha,beta,gamma}.git` `main`. `acme` reads it **from the origin's `main`** (`git --git-dir origins/<repo>.git cat-file blob main:.acme/actions/…`), not from a checkout or from the org repo. A team changes its action by committing to its repo: maintainers, `minLength`, the template and so on. `acme` refuses a definition whose `repository` field doesn't match the repository it lives in, or one that is malformed (it shows under `invalidDefinitions` in `acme list`). Each run pins the definition's commit and blob and stores a copy in its record (`definition`), so a later edit cannot change the rules for a run that is already waiting |
| R2 Validated | `validate_inputs` in `org/acme` applies the definition's rules: `successor` must fully match `^[a-z0-9-]+/[a-z0-9-]+$` and must not be the repo itself (`notEqual: acme/<repo>`); `reason` must be 20 to 200 characters on a single line (no newlines or control characters); required inputs must be present and unknown inputs are refused. `preview` and `request` both validate before anything is recorded or pushed. A refusal exits 2 with per-field errors, and a refused request is logged as `request_refused` in `audit.jsonl` |
| R3 Previewed | `acme preview` changes nothing. It shows the base commit, the operation (`create`, `modify` or `none`), the exact new content, a unified diff and the commit message. `acme request` stores the same preview in the run record, so the requester and the approver can read it with `acme record <id>` (`.preview`). If `DEPRECATED.md` on `main` changes between the request and the approval, the run fails as `stale_preview` instead of applying something nobody previewed |
| R4 Approved | A run starts as `awaiting_approval`, and only `acme approve` executes it. The approver must be in the pinned definition's `maintainers` and must not be the requester; both rules live in `cmd_approve` in `org/acme`. A self-approval or carol's approval exits 3 (`forbidden`), leaves the run waiting, and is recorded as an `approval_refused` event. `acme reject` closes a waiting run (a maintainer or the requester may reject) |
| R5 Done means observed | After pushing, `execute` re-reads the origin (`refs/heads/main` and then `main:DEPRECATED.md`) and compares the bytes with the rendered template. The run is `succeeded` only if they match (`observed` event, `matches: true`). A non-zero push is `failed` with reason `push_rejected` and the push output, and no observation can override that. A push that "succeeds" without the content reaching `main` is `failed` / `not_observed`. Any other error is `failed` / `error`. If `main` already holds the exact content, no commit is made and the run succeeds on observation alone (`no_change_needed`) |
| R6 One place for the org | Run everything from `org/` with `./acme`. `acme list` reads every repository in `org/repos.json` from its origin, and `request`, `approve` and the other commands take the repository by name (`alpha`) or slug (`acme/alpha`). You never `cd` into a repository |
| R7 Audited | Each run record has `requestedBy`/`requestedAt`, `approvedBy` (a list)/`approvedAt`, `rejectedBy`/`rejectedAt`, `inputs`, `outcome` (`result`, `reason`, `at`, observed commit) and a timestamped `events` list: `requested`, `approval_refused`, `approved`, `execution_started`, `push_attempted` (with exit code and output), `observed`, `succeeded`/`failed`/`rejected`. Every event is also appended to `org/.acme/state/audit.jsonl` (`acme audit`). The pushed commit is authored by the requester and carries `Acme-Run`, `Requested-by` and `Approved-by` trailers |
| R8 Usable by agents | Each step is one non-interactive command with JSON output on stdout. Inputs are given as `-i key=value` or `--inputs-json '{…}'`, and identity as `--as <person>` or `ACME_USER`. Errors come back as `{"error":{"code","message","details"}}`, argument errors included. Exit codes: 0 ok, 1 run failed (stdout is the run record) or internal error, 2 invalid input or usage, 3 forbidden or unknown person, 4 not found, 5 wrong state (for example approving a finished run), 6 invalid definition |

## Commands

There is nothing to start. All commands run from the org directory:

```sh
cd /tmp/actions-ab/r3/vanilla-3/org
```

List the actions across all three repositories, or describe one:

```sh
./acme list
./acme list --action mark-deprecated
./acme describe beta mark-deprecated
```

Preview (changes nothing):

```sh
./acme preview alpha mark-deprecated \
  -i successor=acme/dependabot-proxy-v2 \
  -i reason="Replaced by the v2 proxy, which supports OIDC registries."
```

Request as ann. This prints the run record with its `id` and `preview`, and `status` is `awaiting_approval`:

```sh
RUN=$(./acme request alpha mark-deprecated --as ann \
  -i successor=acme/dependabot-proxy-v2 \
  -i reason="Replaced by the v2 proxy, which supports OIDC registries." \
  | python3 -c 'import json,sys; print(json.load(sys.stdin)["id"])')
echo "$RUN"
```

Approve as bob. The run executes, is observed, and the final record is printed. Exit 0 means succeeded and exit 1 means failed:

```sh
./acme approve "$RUN" --as bob
```

Check its status, read its full record, list runs and read the audit log:

```sh
./acme status "$RUN"
./acme record "$RUN"
./acme runs
./acme runs --repo alpha --status succeeded
./acme audit --run "$RUN"
```

Reject a waiting run:

```sh
./acme reject "$RUN" --as bob --comment "not yet"
```

## Testing the refusals

```sh
# Invalid inputs: exit 2, nothing recorded or pushed
./acme request alpha mark-deprecated --as ann -i successor=Acme/X -i reason="too short"
./acme request alpha mark-deprecated --as ann -i successor=acme/alpha -i reason="Long enough reason for this test."

# Self-approval and carol: exit 3, the run stays awaiting_approval
RUN=$(./acme request beta mark-deprecated --as ann -i successor=acme/new-thing -i reason="A different reason for a test run." | python3 -c 'import json,sys; print(json.load(sys.stdin)["id"])')
./acme approve "$RUN" --as ann
./acme approve "$RUN" --as carol
./acme status "$RUN"

# Rejected push: make the origin refuse pushes, then approve. Exit 1, status failed, reason push_rejected
printf '#!/bin/sh\necho "main is frozen" >&2\nexit 1\n' > ../origins/beta.git/hooks/pre-receive
chmod +x ../origins/beta.git/hooks/pre-receive
./acme approve "$RUN" --as bob
./acme status "$RUN"
git --git-dir ../origins/beta.git show main:DEPRECATED.md   # unchanged
rm ../origins/beta.git/hooks/pre-receive
```

The three repositories are already deprecated. A rejected-push test must therefore use inputs that
differ from the current `DEPRECATED.md`, as above, so the run has something to push. With identical
inputs, the run needs no push and succeeds on observation alone (`no_change_needed`). A run left
waiting that you don't want to execute can be closed with `./acme reject`.

## Notes

- `DEPRECATED.md` is written exactly as `# Deprecated: use <successor>\n\n<reason>\n`, which is the specified content with a final newline.
- A run executes synchronously inside `acme approve`, and a global file lock (`org/.acme/state/lock`) serialises writes.
- The org tool is committed to `origins/org.git` `main`. Run state is local and gitignored.
