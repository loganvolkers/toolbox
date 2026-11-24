# Federated actions

Self-service actions declared in each repository and offered, approved and run from one org portal
([federated-actions.md](../../docs/design/federated-actions.md),
[D01M86BY3YQX3SA1W99QXF1QA1H](../../docs/decisions/D01M86BY3YQX3SA1W99QXF1QA1H.md),
[D01M86J769VGE6K5FJJQ76EP4AA](../../docs/decisions/D01M86J769VGE6K5FJJQ76EP4AA.md)). This is about
the box: Toolbox carries the action, validates its inputs, previews it, holds the approval, runs it
in the repository's own clone and records what it observed. The effect itself is a shell command
you write.

| Directory | What it shows |
|---|---|
| [`minimal-repo/`](minimal-repo/.toolbox/toolbox.yaml) | Start here. One repository, one approved action that commits to main |
| [`git-org/`](git-org/toolbox.yaml) | Start here. An org portal over repositories listed by git URL |
| [`repo/`](repo/.toolbox/toolbox.yaml) | Five actions: pull requests, a GitHub workflow, conditions, a vendored package |
| [`org/`](org/toolbox.yaml) | An org portal over sibling checkouts on disk, with its own fleet actions |

## The shortest path

1. **Each repository** commits a `.toolbox/toolbox.yaml` like
   [`minimal-repo`](minimal-repo/.toolbox/toolbox.yaml). An action on `kind: Portal` targets the
   repository itself. Its `inputs` are a JSON Schema, checked before any preview. `approval` names
   who may approve, and the requester never counts. A `change` executor with `land: push` runs
   your command in a fresh clone, commits and pushes to `base`, and succeeds only when that commit
   is on the origin.
2. **The org portal** lists the repositories, like [`git-org`](git-org/toolbox.yaml). Nothing
   else is needed: each child is offered as `portal:<name>`, where the name is the last part of
   its URL.
3. **Serve the org portal** and read the key it signs approvals with:

   ```sh
   toolbox validate git-org
   toolbox serve --listen 127.0.0.1:7100 git-org &
   toolbox run --url http://127.0.0.1:7100 --as ann   # the actions, and the key to pin
   ```

   The server keeps the key under `$TOOLBOX_STATE_DIR/served/org/` and reuses it on restart.
   Paste it into each repository's `acceptApprovalsFrom`, then commit and push: until then the
   server offers the action, but the run is refused. On a loopback address the server signs
   people in with `--as`. Anywhere else, set `TOOLBOX_AUTH_MODE` (see `toolbox serve --help`).
4. **Request, approve, check.** Every command takes `-o json`. Exit codes: 0 succeeded, 1 failed,
   2 refused or invalid, 3 waiting for an approval.

   ```sh
   U=http://127.0.0.1:7100
   toolbox run --url $U --as ann add-code-of-conduct --entity portal:alpha -i contact=conduct@acme.dev          # preview: the diff, nothing changed
   toolbox run --url $U --as ann add-code-of-conduct --entity portal:alpha -i contact=conduct@acme.dev --yes    # request: exit 3, prints the run id
   toolbox run --url $U --as bob --approve <run-id> --note "checked" --wait                                     # runs it in alpha's clone
   toolbox run --url $U --as ann --verify <run-id>                                                              # the record: requester, approver, commit
   toolbox run --url $U --as ann --runs                                                                         # every run, one place
   ```

   The requester can't approve their own run, and someone not in `approval` is refused. The commit
   carries `Toolbox-Run` and `Requested-By` trailers. A push the origin rejects fails the run with
   the origin's message, and nothing is retried behind your back.

Without a server, `toolbox run` in the org or repository checkout lists and previews the same
actions. A run that needs an approval stops at exit 3 there, because only a served portal keeps
approvals.

## Check

`scripts/m5-acceptance.sh` runs every directory here: it serves `git-org` over local bare
repositories, pins the key, and requests, approves and observes runs of both repository examples.
