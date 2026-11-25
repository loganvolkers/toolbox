# Federated actions

Self-service actions declared in each repository and offered, approved and run from one org portal
([federated-actions.md](../../docs/design/federated-actions.md),
[D01M86BY3YQX3SA1W99QXF1QA1H](../../docs/decisions/D01M86BY3YQX3SA1W99QXF1QA1H.md),
[D01M86J769VGE6K5FJJQ76EP4AA](../../docs/decisions/D01M86J769VGE6K5FJJQ76EP4AA.md); on GitLab and
mixed hosts, [forges.md](../../docs/design/forges.md),
[D01M87AXG095Q0Z0DGGTKYYAM7Y](../../docs/decisions/D01M87AXG095Q0Z0DGGTKYYAM7Y.md)). This is about
the box: Toolbox carries the action, validates its inputs, previews it, holds the approval, runs it
in the repository's own clone and records what it observed. The effect itself is a shell command
you write.

| Directory | What it shows |
|---|---|
| [`minimal-repo/`](minimal-repo/.toolbox/toolbox.yaml) | Start here. One repository, one approved action that commits to main |
| [`git-org/`](git-org/toolbox.yaml) | Start here. An org portal over repositories listed by git URL |
| [`repo/`](repo/.toolbox/toolbox.yaml) | Five actions: pull requests, a GitHub workflow, conditions, a vendored package |
| [`org/`](org/toolbox.yaml) | An org portal over sibling checkouts on disk, with its own fleet actions |
| [`gitlab-org/`](gitlab-org/toolbox.yaml) | An org portal over projects on a self-managed GitLab, subgroups included |
| [`gitlab-repo/`](gitlab-repo/.toolbox/toolbox.yaml) | A GitLab project whose federated runs go to its own pipeline ([`.gitlab-ci.yml`](gitlab-repo/.gitlab-ci.yml)) |
| [`hybrid-org/`](hybrid-org/toolbox.yaml) | One org portal over github.com, an Enterprise Server, gitlab.com, a self-managed GitLab and a bare repository; the Enterprise Server's repositories build on Buildkite through an executor-protocol bridge declared under `ci:` (D01M87T16JRH53YV6VVWM8PH169) |

## The shortest path

1. **Each repository** commits a `.toolbox/toolbox.yaml` like
   [`minimal-repo`](minimal-repo/.toolbox/toolbox.yaml): its name and owner, and an `include` of
   [`actions/add-code-of-conduct.yaml`](minimal-repo/.toolbox/actions/add-code-of-conduct.yaml),
   a standard that is the same file in every repository that offers it (`entity.name` and a
   test's `{portal}` stand for whichever repository holds it). An action on `kind: Portal` targets the
   repository itself. Its `inputs` are a JSON Schema, checked before any preview. `approval` names
   who may approve, and the requester never counts. A `change` executor with `land: push` runs
   your command in a fresh clone, commits and pushes to `base`, and succeeds only when that commit
   is on the origin. Its [test file](minimal-repo/.toolbox/tests/add-code-of-conduct.test.yaml)
   checks the approvals, an invalid input and a rejected push: `toolbox test .toolbox` runs each
   case in a scratch copy of the repository and its origin's `main`, so nothing real changes and
   no server is needed.
2. **The org portal** lists the repositories, like [`git-org`](git-org/toolbox.yaml). Nothing
   else is needed: each child is offered as `portal:<name>`, where the name is the last part of
   its URL.
3. **Read the key the org portal signs approvals with.** `toolbox run` in its directory lists
   every child's actions and ends with `org approves its children's runs with key ed25519:…`
   (`approval_key` with `-o json`). Toolbox makes the key once and keeps it, the children's clones
   and the runs under `$TOOLBOX_STATE_DIR` (default `~/.local/state/toolbox`). Paste it into each
   repository's `acceptApprovalsFrom`, then commit and push: until then the org portal offers the
   action, but the run is refused.
4. **Request, approve, check**, in the org portal's directory. Every command takes `-o json`.
   Exit codes: 0 succeeded, 1 failed, 2 refused, blocked or invalid, 3 waiting for an approval, 4
   unverified. `TOOLBOX_AS` stands for `--as`.

   ```sh
   cd git-org
   toolbox run --as ann add-code-of-conduct --entity portal:alpha -i contact=conduct@acme.dev          # preview: the diff, nothing changed
   toolbox run --as ann add-code-of-conduct --entity portal:alpha -i contact=conduct@acme.dev --yes    # request: exit 3, prints the run id
   toolbox run --as bob --approve <run-id> --note "checked" --wait                                     # runs it in alpha's clone
   toolbox run --as ann --verify <run-id>                                                              # the record: requester, approver, commit
   toolbox run --as ann --runs                                                                         # every run, one place
   ```

   The requester can't approve their own run, and someone not in `approval` is refused. The commit
   carries `Toolbox-Run`, `Requested-By` and `Approved-By` trailers. A push the origin rejects
   fails the run with the origin's message, and nothing is retried behind your back.

   The checkout takes `--as` on trust: anyone who can use `$TOOLBOX_STATE_DIR` can approve as
   anyone, and can read the signing key. That suits one operator, a trial, or a CI job that guards
   the state directory. For a team, serve it.
5. **Serve it** when approvers sign in from elsewhere, or for the UI, REST and MCP: `toolbox serve
   git-org --listen 127.0.0.1:8080 --detach` returns once ready and prints its URL, pid, log and
   the same key; `--stop` stops it. The same `toolbox run` commands then take `--url` (or
   `TOOLBOX_URL`). On a loopback address the server signs people in with `--as`; anywhere else,
   set `TOOLBOX_AUTH_MODE` (see `toolbox serve --help`). A served portal keeps its runs apart from
   the checkout's, so approve a run where it was requested.

In a repository's own checkout, `toolbox run` lists and previews its actions too, but a run that
needs an approval stops at exit 3 there: a repository holds no approvals, its parent does.

## Fields

Everything an action or a federation needs; `toolbox validate` checks the rest against
`schema/manifest.schema.json`.

| Field | What it says |
|---|---|
| `actions[].name`, `kind`, `description` | Required. `kind: Portal` targets the repository itself |
| `actions[].inputs` | Required JSON Schema; `required`, `pattern`, `minLength`, `maxLength`, `enum`, `additionalProperties: false` and `default` are checked before a preview. Each input reaches the command as `TOOLBOX_INPUT_<NAME>` |
| `actions[].approval` | CEL list of approvers: `"['user:ann']"` needs ann, `"[['user:ann', 'user:bob']]"` needs either, `"['group:security']"` needs a member. The requester never counts |
| `authz` | Parent: `{model: groups.fga, tuples: [{static: [{user: user:sam, relation: member, object: group:security}]}]}` defines the groups approvals name, once for every child; nested groups (`group:leads#member`) count. A checkout and a server read the same model |
| `actions[].preconditions` | A list of CEL over `entity` and `inputs`; one that is false blocks the run |
| `actions[].executor.type` | `change` (a commit), `command` (a shell command in the checkout), `github-workflow`, `gitlab-pipeline` (`{forge: <url>, project: group/name, ref}`: a pipeline with the run's inputs as typed inputs; `ci: <name>` instead of `forge` takes a `gitlab-ci` host's endpoint and token), `http`, `webhook`, `in-image` |
| `executor.run` | `command` and `change`: the shell command. A `change` runs it in a fresh worktree of `base` |
| `executor.land` | `change`: `push` commits to `base` once approved (never forced); `propose` (default) pushes `branch` for review |
| `executor.base`, `branch`, `message`, `remote` | `change`: the branch to start from and verify on, the proposal branch (`toolbox/{action}/{run}`), the commit message, the remote (`origin`). `message` takes `{action}`, `{run}`, `{entity}` and `{inputs.<name>}`; `branch` takes only `{action}` and `{run}` |
| `executor.secrets` | Environment variables the executor gets; it starts with a cleared environment |
| `actions[].expect` | `{postcondition: <CEL>, within: 30s}`: what must hold after the run for `succeeded`. A `land: push` change needs none: its commit on `base` is the check |
| `tests` | Test files; a file's `runs:` cases request, approve and run an action in a scratch copy (`as`, `inputs`, `approve`, `members`, `rejectPush`, `expect: {state, files, refused, error}`; `members: {'group:security': [user:sam]}` stands in for the org's groups), each starting from the file's `defaults`; `{portal}` in a value is the portal under test |
| `federation.children` | Parent: `[{git: <url or path>, branch: main, key: <name>}]`; each is offered as `portal:<key>` |
| `federation.acceptApprovalsFrom` | Child: `[{portal: <parent name>, key: ed25519:…}]`, the key the parent prints |
| `federation.signingKey` | Parent: the variable holding the signing seed, for servers that share runs. Without it Toolbox keeps one |
| `federation.runner` | Child: `{type: local}` lets the org portal run its actions in the org's clone of it; `{type: ci}` (GitHub: `workflow`, default `toolbox.yml`; `ref`) runs them in the CI the parent's entry for its forge picks (default the forge's own); without it they run only from the repository's checkout. A `ci` child never names its project, host or token: the parent uses where it cloned it from |
| `forges` | Any portal: `[{url, type: github\|gitlab, api, token: <env var>, projects: [globs]}]`. github.com and gitlab.com need no entry (`GITHUB_TOKEN`, `GITLAB_TOKEN`); a self-hosted forge needs its `url` and `type`. `projects` narrows which children a parent dispatches CI to (`*` stays within one path segment). `ci: <name>` (or `none`) sends its children's CI to a declared CI host |
| `ci` | Any portal: `[{name, type: github-actions\|gitlab-ci\|http, url, api, token: <env var>, project: <template with {project}>, projects: [globs]}]`. CI hosts apart from forges; an `http` host is a service speaking the executor protocol that bridges to any CI ([forges.md](../../docs/design/forges.md) §4.7) |
| `federation.runners` | Parent: a portal with `children` admits each child's `local` runner itself; declare `runners.local` or `runners.github-workflow` only to narrow it or to dispatch workflows |

## Check

`scripts/m5-acceptance.sh` runs every directory here: it serves `git-org` over local bare
repositories, pins the key, and requests, approves and observes runs of both repository examples,
then does it again from the `git-org` checkout with no server, including an approval by a member of a group the org portal declares.
