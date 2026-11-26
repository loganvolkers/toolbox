# Handoff: `mark-deprecated`

`mark-deprecated` is a Seikika self-service action. Each repository declares and owns its copy in
`.seikika/seikika.yaml` on its origin's `main`. The org portal in this directory offers all three
from one server on `127.0.0.1:7450`, where people and agents preview, request, approve, check and
audit runs. It has run on all three repositories: ann requested each run, bob approved it, and
each was reported `succeeded` only after Seikika fetched the origin's `main` and found
`DEPRECATED.md` with the previewed content.

| Repository | Org run id | Commit on origin `main` | State |
|---|---|---|---|
| alpha | `run_2ft9dukkmzph` | `d11a2fb` | succeeded |
| beta | `run_2ft9e1ruumyp` | `bd84fb9` | succeeded |
| gamma | `run_2ft9e8udqvey` | `0efd4c2` | succeeded |

Each commit carries `Seikika-Run`, `Requested-By: user:ann` and `Approved-By: user:bob` trailers.
The file is `# Deprecated: use <successor>`, a blank line, then `<reason>`, ending with a newline.

All of this is Seikika 0.1.1 (`bin/`). The only glue I wrote is `org/env.sh`, which sets four
environment variables.

## How each requirement is met

| # | How | Files |
|---|---|---|
| R1 Owned by the repository | Each repository's action, its approvers, its input schema, its executor and the org key whose approvals it accepts are in that repository, committed to its origin's `main` (alpha `c52d61d`, beta `e3d5f53`, gamma `b1bfe6f`). The org portal holds no action definitions. Before it lists, previews or runs an action, the org server fetches each repository's `main` again, so a team's change takes effect once it lands on `main`. | `alpha/.seikika/seikika.yaml`, `beta/.seikika/seikika.yaml`, `gamma/.seikika/seikika.yaml` |
| R2 Validated | `inputs` is a JSON Schema: `successor` must match `^[a-z0-9-]+/[a-z0-9-]+$`, `reason` must be 20 to 200 characters, both are required, and other inputs are refused (`additionalProperties: false`). A precondition also refuses the repository itself as its own successor (`acme/<repo>`). Seikika checks these before any preview, so nothing is recorded or changed: the response is `422 invalid-inputs` (exit 2), and a failed precondition returns `blocked`. | `<repo>/.seikika/seikika.yaml` (`inputs`, `preconditions`) |
| R3 Previewed | Without `--yes` or `--confirm`, `seikika run <action> …` only previews: it shows the file diff, the push to `main`, the approvers and a confirm token, and changes nothing. A run then executes only if its change matches the confirmed preview; the verification compares every changed file's blob with it. | Seikika built-in; executor `type: change` in `<repo>/.seikika/seikika.yaml` |
| R4 Approved | `approval: "[['user:ann', 'user:bob']]"` means one of ann or bob must approve, and the requester never counts. The server refuses a requester's own approval (`403 not-an-approver: a requester cannot approve their own run`) and carol's (`403 not-an-approver: user:carol is not one of the pending approvers`), both with exit 2. I checked both live on `run_2ft9dukkmzph` before bob approved it. The org server signs each approval with its key, and the repository runs the change only if that signature verifies against the key pinned in `federation.acceptApprovalsFrom`. | `<repo>/.seikika/seikika.yaml` (`approval`, `federation.acceptApprovalsFrom`); key: `.seikika-state/served/org/signing-key` |
| R5 Done means observed | `executor: {type: change, land: push, base: main}` commits in a fresh clone and pushes without forcing. Then it fetches `origin/main` and reports `succeeded` only when the commit is there and every file it changed has the previewed content (`verification.files[].matches`). A rejected push fails the run (exit 1) with the origin's message, and nothing is retried. I rehearsed this live against a scratch copy of alpha's origin that had a `pre-receive` hook, and `main` stayed unchanged. If `main` already has the exact content, the run succeeds with `changed: false` and makes no commit, because the content was observed. | `<repo>/.seikika/seikika.yaml` (`executor`) |
| R6 One place for the org | `org/seikika.yaml` lists the three origins as federation children (`portal:alpha`, `portal:beta`, `portal:gamma`). From `org/`, `seikika run` lists all three actions and requests a run on any of them with `--entity portal:<repo>`. `seikika run --runs` lists every run. | `org/seikika.yaml`, `org/env.sh` |
| R7 Audited | Each run's record contains `requested_by`, `inputs`, `approval.approved_by` (with the approver's note), `state`, `outputs` (the commit), `verification`, and `events`. Each event records its state, timestamp (`at`) and actor (`by`): previewed, confirmed, awaiting_approval, approved, dispatched, running, succeeded or failed. Records are saved as JSON and survive a server restart. The commit trailers repeat the requester and approver on `main`. | `.seikika-state/served/org/runs/<run id>.json` (read with `--verify`) |
| R8 Usable by agents | Every step is a non-interactive `seikika run` command with `-o json`. Exit codes are 0 succeeded, 1 failed, 2 invalid or refused, 3 waiting for approval, and 4 unverified. `SEIKIKA_URL` and `SEIKIKA_AS` can replace `--url` and `--as`. The skills in `seikika-docs/skills/run` and `seikika-docs/skills/add-action` describe the same steps. | `org/env.sh` |

The lifecycle is also covered by test cases that each repository owns. They run the server's
code in a scratch copy of the repository and its origin, so the real origin is never touched.
Each repository has 14 cases, and all pass in all three: success for ann→bob and bob→ann, the
19/20/200/201-character boundaries, a bad successor, an upper-case successor, a missing input,
an unknown input, the repository as its own successor, ann's self-approval and carol's refused
approval, carol requesting while a maintainer approves, and a rejected push. Run them with
`cd /tmp/actions-ab/r5/seikika-2/<repo> && seikika test .seikika`. The cases are in
`<repo>/.seikika/tests/mark-deprecated.test.yaml`.

## Commands

Run everything from the org directory with the environment sourced. **Use this state directory.**
It holds the signing key that the repositories pin. A server started with the default state
directory would create a new key, and the repositories would refuse its approvals
(`child-refuses-approvals`).

```sh
cd /tmp/actions-ab/r5/seikika-2/org
. ./env.sh                      # PATH, SEIKIKA_STATE_DIR, XDG_STATE_HOME, SEIKIKA_URL=http://127.0.0.1:7450
```

**Start and stop the server.** The only process that must be running is the org server. It
starts in the background and returns once it is ready:

```sh
seikika validate .
seikika serve . --listen 127.0.0.1:7450 --detach     # prints the URL, pid, log and approval key ed25519:e16bf558…0702
seikika serve . --stop                               # stop it when done
```

**List** every repository's action:

```sh
seikika run --as ann -o json
```

**Preview** a run. This changes nothing; it prints the diff, the approvers and a confirm token:

```sh
seikika run --as ann mark-deprecated --entity portal:alpha \
  -i successor=acme/dependabot-proxy-v2 \
  -i 'reason=Replaced by the v2 proxy, which supports OIDC registries.' -o json
```

**Request** a run as ann. Either confirm the token from the preview, or preview and request in one
step with `--yes`; the preview is still recorded. Both exit 3 and print the run id (`.id`):

```sh
seikika run --as ann mark-deprecated --entity portal:alpha \
  -i successor=acme/dependabot-proxy-v2 \
  -i 'reason=Replaced by the v2 proxy, which supports OIDC registries.' --confirm ct_… -o json
seikika run --as ann mark-deprecated --entity portal:beta \
  -i successor=acme/valtio-store \
  -i 'reason=State management moves to the shared store package.' --yes -o json
```

**Approve** a run as bob. `--wait` follows the run until it settles, exiting 0 only once `main` has
the file:

```sh
seikika run --as bob --approve <run-id> --note "checked" --wait -o json
```

**Check the status, read the record, and list every run:**

```sh
seikika run --as ann --verify <run-id> -o json        # .state, .requested_by, .inputs, .approval.approved_by, .events[].at, .verification
seikika run --as ann --runs -o json
cat /tmp/actions-ab/r5/seikika-2/.seikika-state/served/org/runs/<run-id>.json   # the same record on disk
```

## Testing the refusals against the live server

- **Invalid inputs** are refused with exit 2 and `422 invalid-inputs`, before any preview or run
  exists:

  ```sh
  seikika run --as ann mark-deprecated --entity portal:gamma -i successor=Acme/Core -i reason=short --yes -o json
  ```

- **Self-approval and carol.** Request a run as ann, then try both approvals below. Each exits 2
  with `403 not-an-approver`, and the run stays `awaiting_approval`:

  ```sh
  seikika run --as ann --approve <run-id>
  seikika run --as carol --approve <run-id>
  ```

- **Rejected push.** Make the origin refuse pushes, then request with a reason different from the
  one already on `main`. If the content is identical, there is nothing to push: the run makes no
  commit and succeeds with `changed: false`, because the file was observed on `main`. Bob's
  approval then exits 1 with the run `failed`, the record holds the origin's message, and `main`
  is unchanged:

  ```sh
  printf '#!/bin/sh\necho "main is protected" >&2\nexit 1\n' > ../origins/alpha.git/hooks/pre-receive
  chmod +x ../origins/alpha.git/hooks/pre-receive
  seikika run --as ann mark-deprecated --entity portal:alpha -i successor=acme/other \
    -i 'reason=Testing that a rejected push fails the run.' --yes -o json   # note .id
  seikika run --as bob --approve <run-id> --wait -o json                    # exit 1, state failed
  rm ../origins/alpha.git/hooks/pre-receive
  ```

- **A team changes its action.** Commit a change to `<repo>/.seikika/seikika.yaml` and push it to
  `main`. The next list, preview or run on the org server uses it, with no restart.

## Notes

- Sign-in is by `--as`, because the server listens on loopback. Off loopback, set
  `SEIKIKA_AUTH_MODE` (see `seikika serve --help`). The pinned key is
  `ed25519:e16bf55865f24ad88db193fd44a31653dbc2fd0ceb971abdc2efc24477ab0702`.
- Carol may request a run but cannot approve one. The task says nothing against requesting, and
  the approval rule is unchanged.
- Refused approval attempts are returned to the caller as 403 errors. They are not added to the
  run's `events`.
- Commits are authored by `Seikika <seikika@localhost>`. The people involved are recorded in the
  `Requested-By` and `Approved-By` trailers and in the run record.
