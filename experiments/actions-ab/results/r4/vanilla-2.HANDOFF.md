# HANDOFF: `mark-deprecated` self-service action

`mark-deprecated` is defined in each repository and run through one org-level CLI,
`/tmp/actions-ab/r4/vanilla-2/org/acme-actions` (Python 3, standard library only). Nothing needs to
be running: there is no server, and every command is a one-shot process.

All three runs were requested by ann, approved by bob, and reported `succeeded` after
`DEPRECATED.md` was observed on each origin's `main`:

| Repository | Run id | Commit on origin `main` |
|---|---|---|
| alpha | `run-20261124T124006Z-alpha-7f78bc` | `fdd0217` |
| beta | `run-20261124T124007Z-beta-66cbbb` | `497a621` |
| gamma | `run-20261124T124007Z-gamma-255faa` | `9263447` |

## Files

| Path | What it is |
|---|---|
| `<repo>/.acme/actions/mark-deprecated.json` (alpha, beta, gamma) | The repository's own action definition: maintainers, input rules, effect (file path, template, commit message). Committed to each origin's `main`. |
| `org/acme-actions` | The CLI: discovery, validation, preview, requests, approval, execution, verification, audit. |
| `org/repos.json` | Org registry: the three repositories (origin paths), the known people (ann, bob, carol). |
| `org/state/runs/<run-id>.json` | One record per run, with its full event history. Not committed (`org/.gitignore`). |
| `org/state/events.jsonl` | Append-only log of every event across all runs. |

Set `ACME_ACTIONS_STATE=<dir>` to keep run state somewhere else (for example, a scratch directory
for tests).

## How each requirement is met

| # | How |
|---|---|
| R1 Owned by the repository | The definition is `.acme/actions/mark-deprecated.json` in each repository, committed to `origins/<repo>.git` `main` (alpha `edcef4e`, beta `b893c35`, gamma `4abdaa5`). The CLI reads definitions **only from origin `main`** (`git cat-file` on the bare origin), never from the checkouts or from the org. A team changes its action (who maintains it, input rules, template) by committing to its own `main`; the next `list`/`request` picks it up. Each run pins the definition commit it was requested against (`record.definition.commit`), and approval uses the maintainers from that pinned definition. |
| R2 Validated | `validate()` in `org/acme-actions` checks inputs against the definition: `successor` must match `^[a-z0-9-]+/[a-z0-9-]+$` and must not name the repository itself (`acme/<repo>`); `reason` must be 20 to 200 characters; unknown and missing inputs are refused. `preview` and `request` both validate; invalid inputs return `{"ok": false, "error": {"code": "invalid_inputs", "details": [...]}}`, exit code 2, and no run is created and nothing is pushed. |
| R3 Previewed | `acme-actions preview` shows, without creating anything, the target origin and base commit, each changed file with operation (`create`/`modify`/`unchanged`), full new content, a unified diff and the commit message. `request` returns the same preview and stores it in the run record, so the approver sees it too (`record`). |
| R4 Approved | A requested run is `awaiting_approval`; nothing is pushed until `approve`. `approve` refuses (exit 2) when the approver is the requester (`self_approval`), is not in the definition's `maintainers` (`not_maintainer`, so carol is refused), is unknown (`unknown_person`), gives no identity (`no_actor`), or the run is not awaiting approval (`not_awaiting_approval`). Refused approvals are recorded as `approval_refused` events. State changes are serialised with a file lock (`org/state/.lock`), so a run cannot be approved twice. |
| R5 Done means observed | After `approve`, `execute()` builds a commit on top of origin `main` and pushes it to the origin (so server-side hooks apply). A rejected push sets the run `failed` (`reason: push_rejected`, with the push's stderr). Whether or not the push was accepted, the run is set `succeeded` only after the CLI reads `main:DEPRECATED.md` from the origin and its bytes equal the expected content (polling for up to 5 s); otherwise it is `failed` (`reason: not_observed`). Tested: a `pre-receive` hook that rejects gives `push_rejected`; a `post-receive` hook that reverts `main` gives `not_observed`. If `main` already has the exact content, no push is made and the run succeeds on observation (`change_already_present` event). |
| R6 One place for the org | From `org/`, `./acme-actions list` lists every action in every repository in `org/repos.json` by reading each origin; `request <repo> mark-deprecated ...` works for any of them. Nobody needs to enter a repository checkout. |
| R7 Audited | `org/state/runs/<run-id>.json` holds `requestedBy`/`requestedAt`, `approvedBy`/`approvedAt`, `inputs`, `status`, `outcome` (observed commit or failure reason and push stderr), `finishedAt`, the pinned definition, the preview, and an `events` list with a UTC timestamp and actor for each step (`requested`, `approval_refused`, `approved`, `push_started`, `push_accepted`/`push_rejected`, `change_observed`/`change_not_observed`, `succeeded`/`failed`). Every event is also appended to `org/state/events.jsonl`. The commit on `main` carries `Action-Run`, `Requested-by` and `Approved-by` trailers; its author is the requester and its committer is `acme-actions`. |
| R8 Usable by agents | Every command is non-interactive, takes identity from `--as <person>` (or `ACME_ACTOR`), prints exactly one JSON document on stdout, and exits 0 on success, 2 when refused or invalid, and 1 when a run fails or an internal error occurs. Inputs can be passed as repeated `-i key=value` or as `--inputs-json '{...}'`. |

## Commands

Nothing has to be started. Run everything from the org directory:

```bash
cd /tmp/actions-ab/r4/vanilla-2/org
```

List the actions across all three repositories (or one: `--repo alpha`):

```bash
./acme-actions list
./acme-actions show alpha mark-deprecated
```

Preview (validates; changes nothing):

```bash
./acme-actions preview alpha mark-deprecated \
  -i successor=acme/dependabot-proxy-v2 \
  -i "reason=Replaced by the v2 proxy, which supports OIDC registries."
```

Request a run as ann (prints the run id at `.run.id`, plus the preview):

```bash
./acme-actions request alpha mark-deprecated --as ann \
  -i successor=acme/dependabot-proxy-v2 \
  -i "reason=Replaced by the v2 proxy, which supports OIDC registries."
RUN=$(./acme-actions request beta mark-deprecated --as ann \
  --inputs-json '{"successor":"acme/valtio-store","reason":"State management moves to the shared store package."}' \
  | python3 -c 'import json,sys; print(json.load(sys.stdin)["run"]["id"])')
```

Approve as bob (executes, pushes, verifies; returns the final status):

```bash
./acme-actions approve "$RUN" --as bob
```

A maintainer can instead decline: `./acme-actions reject "$RUN" --as bob --comment "..."` (the
requester can withdraw the run the same way; it is recorded as `cancelled`).

Check a run's status, read its full record, list runs:

```bash
./acme-actions status "$RUN"
./acme-actions record "$RUN"
./acme-actions runs                      # filters: --repo alpha --status succeeded
```

## Testing the negative cases

To keep test runs out of the real audit trail, export a scratch state directory first:
`export ACME_ACTIONS_STATE=/tmp/actions-ab/r4/vanilla-2/r4-vanilla-2-test-state`.

```bash
# Invalid inputs: refused with exit 2, no run created
./acme-actions request alpha mark-deprecated --as ann -i successor=Acme/X -i reason=short
./acme-actions request alpha mark-deprecated --as ann -i successor=acme/alpha -i "reason=Pointing at itself is not allowed."

# Self-approval and carol: refused with exit 2, recorded as approval_refused
RUN=$(./acme-actions request alpha mark-deprecated --as ann -i successor=acme/new-alpha \
  -i "reason=Testing approval rules for this action." | python3 -c 'import json,sys; print(json.load(sys.stdin)["run"]["id"])')
./acme-actions approve "$RUN" --as ann
./acme-actions approve "$RUN" --as carol

# Rejected push: the run ends failed (push_rejected), never succeeded
printf '#!/bin/sh\necho "rejected by policy" >&2\nexit 1\n' > ../origins/alpha.git/hooks/pre-receive
chmod +x ../origins/alpha.git/hooks/pre-receive
./acme-actions approve "$RUN" --as bob        # exit 1, .run.status == "failed"
rm ../origins/alpha.git/hooks/pre-receive
```

Notes for testers:

- A rejected-push test must use inputs whose content differs from what `main` already has. If
  `DEPRECATED.md` on `main` already equals the requested content, no push is needed and the run
  succeeds on observation.
- A successful test run with different inputs replaces `DEPRECATED.md` on that origin's `main`.
- A failed or rejected run is final; request a new run to try again.
- Local checkouts (`alpha`, `beta`, `gamma`) lag the origins until you `git pull`; the system always
  reads and writes the origins.
