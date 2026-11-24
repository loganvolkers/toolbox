# Forges: One Federation over GitHub, GitLab and Whatever Hosts Code Next

*Status: design and v1 implementation, 2026-11-24 (D01M87AXG095Q0Z0DGGTKYYAM7Y). This is about the
box: carrying actions and runs between portals whose repositories live on different code hosts,
and remembering what each host did. The tools are the hosts themselves (GitHub, GitLab, Origin,
Gitea, a bare repository) and `git`. It extends [federated-actions.md](./federated-actions.md)
(D01M86BY3YQX3SA1W99QXF1QA1H), which assumed GitHub wherever it needed more than git. It was tested
against a real GitLab CE 19.6.1 and GitLab Runner 19.6.1 on one machine (§6).*

---

## 1. Conclusion

**A forge is where a repository's git, reviews, CI and people live. Toolbox needs six things
from one, and plain git gives two of them. A parent declares the forges it reaches by URL, and a
child's forge is the one its git URL is on, so a child never says where the parent's credentials
go. GitHub and GitLab are drivers behind one contract. A forge without a capability is still a
forge: what it lacks falls back to plain git or to the parent's checkout, and the run record
says which.**

| Capability | What Toolbox uses it for | Plain git | GitHub (github.com, Enterprise Server) | GitLab (gitlab.com, self-managed CE and EE) |
|---|---|---|---|---|
| **Read** | Clone and fetch a child; verify a change by ancestry or trailer | Yes | git | git |
| **Write** | A `change` pushes its branch, or the base with `land: push` | Yes | git | git |
| **Propose** | A `change` opens a review for its branch | No: the branch is the proposal | A pull request (REST, the forge's token) | A merge request: push options (`merge_request.create`) when the pusher knows the host, otherwise the parent opens it with its token |
| **Dispatch** | Start a run of the child's CI with the run's inputs, and get its id | No | `workflow_dispatch`, `return_run_details` | `POST /projects/:id/pipeline` with typed `inputs` |
| **Track** | Follow that run by id to its end | No | `GET /actions/runs/:id` | `GET /projects/:id/pipelines/:id` |
| **Collect** | Read the child's own run record from the CI run | No | Deferred: run artifacts are zip archives | `GET /projects/:id/jobs/:id/artifacts/<path>` |

- **GitLab-only organisations work on the free tier.** Every capability above is in GitLab CE.
  The governance GitLab sells in Premium (merge request approval rules, protected environments
  with approvers, CODEOWNERS approval) is what Toolbox's own approvals and verification provide
  (T6, T8), so a CE organisation gets "someone in release approves, never the requester, done
  when seen on main" without it (§3).
- **Hybrid organisations are one list.** A parent lists children on github.com, an Enterprise
  Server, gitlab.com, a self-managed GitLab and a bare repository side by side. Each child's run
  goes to its own forge, with that forge's token, to the project it was cloned from (§4).
- **GitHub keeps the zero-configuration default.** A child on `https://github.com` or
  `https://gitlab.com` needs no forge declaration: the host is known, the token is `GITHUB_TOKEN`
  or `GITLAB_TOKEN`, and a parent may dispatch to the children it lists. A self-hosted forge is
  one entry: its URL and its type.
- **A new forge is one driver.** It implements the capabilities it has. Nothing in the run
  lifecycle, the trust rules or the manifest changes. Origin, Cursor's forge, is mapped in §5
  from its CLI's documented commands; it would arrive with *read*, *write* and *propose* and
  without *dispatch*, and its children would run through the parent's checkout until it has one.

## 2. What GitLab changes

A GitLab project is a GitHub repository with different names and three differences that matter.
Each row was checked against the running GitLab (§6) or its documentation at
`gitlab-org/gitlab@master`.

| Concept | GitHub | GitLab | What Toolbox does |
|---|---|---|---|
| Where code lives | `owner/repo` | `group/subgroup/…/project`, nested to any depth | A project is its full path. Globs match it whole (`acme/*` matches `acme/pay`, not `acme/platform/pay`; `acme/**` would, and is spelled `acme/*/*` today) |
| Review | Pull request | Merge request | *Propose* is a capability, not a URL shape |
| Start CI with data | `workflow_dispatch` with string inputs | A pipeline with typed `inputs` (`spec:inputs`) or pipeline variables | **Inputs, not variables.** New projects on gitlab.com refuse pipeline variables (`ci_pipeline_variables_minimum_override_role: no_one_allowed` since 17.7); inputs are not restricted. Toolbox sends inputs |
| Data into a job | `${{ inputs.x }}` in `run:` is script injection; `env:` is the safe form | `$[[ inputs.x ]]` in `script:` is script injection; job `variables:` is the safe form | The shipped job maps every input to a job variable and the script reads the variable (§4.4) |
| A run's id | Returned since 2026-02 (`return_run_details`); before, found by name | Returned by the create call | Tracked by id on both |
| The job's own push | `GITHUB_TOKEN` with `contents: write` | `CI_JOB_TOKEN`, once the project allows job-token pushes (`ci_push_repository_for_job_token_allowed`) | A child's pipeline lands its own change with no extra secret |
| Who started it | `github.actor` | `GITLAB_USER_LOGIN` | Recorded as the forge's actor; the requester is still the parent's principal |
| Gates in the host | Environments with required reviewers, self-review prevention (all plans for public repositories) | Protected environments and approval rules (Premium); protected branches by role (CE) | On CE the gates are Toolbox's. On Premium, a pipeline's protected environment still applies after Toolbox's approval |
| Groups of people | Teams | Groups and subgroups, with members by role | The `gitlab` connector reports each group's members for the org's model to turn into approvers (§4.6) |

## 3. Stories

These extend [federated-actions.md](./federated-actions.md) §4. Each names what decides "done".
*Built* stories are checked by `scripts/gitlab-acceptance.sh` against a real GitLab (§6); its step
numbers are in brackets.

| # | Story | Done when | Status |
|---|---|---|---|
| **F1** | As a **platform engineer in a GitLab-only organisation**, I list our projects by their GitLab URLs and the org portal offers every project's actions | `toolbox run` in the org lists each child's actions, read from what is committed on GitLab; a private project is read with git's own credentials [1] | Built |
| **F2** | As a **repo maintainer on GitLab**, I want a change to arrive as a merge request, so review is its approval | The run is `proposed` with the merge request's URL; it stays `proposed` while the request is open and is `succeeded` once GitLab merges it and the base shows it, by merge commit or squash [2] | Built |
| **F3** | As a **release manager on GitLab**, I want a federated run to execute in the project's own pipeline, with the project's runner and credentials, never the org's | The parent dispatches a pipeline to the project it cloned, with its own token; the child's job runs `toolbox run` with the parent's signed grant; the parent tracks the pipeline by id and takes the child's verification from the job's artifact; inputs with quotes, `$VARS`, `$(…)`, newlines and `$[[ ]]` reach the job as data [3, 4] | Built |
| **F4** | As a **security owner**, I want a child to be unable to point the parent's dispatch at another project or host | A child that names `project`, `repo`, `api`, `url` or `token` in its runner is refused: the schema refuses all but `repo` (which `github-workflow` uses) and the parent says the child's manifest is invalid when the action isn't offered, and the parent refuses all six before it dispatches; a project outside the forge's `projects` is refused; a push the host's branch protection refuses fails the run with the host's words [5] | Built |
| **F5** | As a **platform engineer in a hybrid organisation**, I run one fleet change across GitHub, Enterprise Server, GitLab and bare repositories | Each child's run goes to its own forge with that forge's token; one parent lists them all | Built. GitLab and a bare repository side by side against real hosts [6]; routing by unit tests: `cargo test -p toolbox-model forge` resolves every child of [`hybrid-org`](../../examples/federated-actions/hybrid-org/toolbox.yaml) (github.com, Enterprise Server, gitlab.com, self-managed by https and ssh, a bare repository) to its driver, API, token and project, and `-p toolbox-exec ci_runner` builds each dispatch, and GitHub dispatch as in the M5 script |
| **F6** | As an **approver**, I want "someone in release" to mean the GitLab group, so people are managed in one place | The `gitlab` connector reports `acme/release`'s members; the requester and a Guest are refused, someone removed from the GitLab group is refused on the next read, and a member approves [8] | Built (§4.6) |
| **F7** | As a **repo owner moving from GitHub to GitLab**, I want my `.toolbox/` to move unchanged | The child's runner is `{type: ci}` on both; only the parent's `git:` URL changes | Built (the runner type is forge-neutral) |
| **F8** | As a **platform engineer**, I start the org's own pipeline (a deploy, a release) from the portal, with typed inputs | A `gitlab-pipeline` executor sends the run's inputs as pipeline inputs; an input GitLab refuses fails the run with GitLab's reason [7] | Built |
| **F10** | As an **approver in a GitLab-only organisation with no server**, I want the org's checkout to know it's me, not whoever I say I am | With `identity: true` on the forge, a request and an approval need the person's own GitLab token; `--as` alone, `--as` naming someone else and a token GitLab refuses are refused, and no record holds a token [9] | Built (§4.6) |
| **F9** | As a **forge author**, I add a forge Toolbox has never seen | One driver implementing the capabilities it has; no change to the lifecycle, the trust rules or the manifest | Designed (§5) |

## 4. The abstraction

### 4.1 Declaring forges

```yaml
# An org portal over four forges (examples/federated-actions/hybrid-org/toolbox.yaml)
forges:
  - url: https://gitlab.acme.dev          # a self-managed GitLab: its type can't be guessed
    type: gitlab
    token: ACME_GITLAB_TOKEN               # the variable holding the token, never the token
  - url: https://ghe.acme.dev
    type: github                           # api defaults to <url>/api/v3
    token: ACME_GHES_TOKEN
    projects: ["platform/*"]               # which children a parent may dispatch CI to
federation:
  children:
    - git: https://github.com/acme/web.git          # known host: no forge entry needed
    - git: https://gitlab.com/acme/mobile.git       # known host: GITLAB_TOKEN
    - git: https://gitlab.acme.dev/acme/platform/pay.git
    - git: https://ghe.acme.dev/platform/ledger.git
    - git: /srv/git/tools.git                        # plain git: local runner only
```

| Field | Meaning | Default |
|---|---|---|
| `url` | The forge's web URL. A git URL is on this forge when it starts with it (`https://`), or names its host (`git@host:path`, `ssh://`) | Required |
| `type` | `github` or `gitlab` | `github` for `https://github.com`, `gitlab` for `https://gitlab.com`; required otherwise |
| `api` | The API base | GitHub: `https://api.github.com` for github.com, else `<url>/api/v3`. GitLab: `<url>/api/v4` |
| `token` | The environment variable holding the API token | `GITHUB_TOKEN` or `GITLAB_TOKEN` |
| `projects` | Project paths (globs) a parent may dispatch CI to | The children the parent lists on this forge |

`forges` is top-level because any portal uses it: a repo portal's `gitlab-pipeline` executor and
its `change` reviews, as well as a parent's dispatches. Two entries with one URL fail validation.
A URL matched by no entry and no known host is plain git.

### 4.2 The child's side

```yaml
# .toolbox/toolbox.yaml in any child, on any forge
federation:
  runner: {type: ci}                 # GitHub also names its workflow: {type: ci, workflow: toolbox.yml}
  acceptApprovalsFrom: [{portal: org, key: "ed25519:…"}]
```

`ci` means *run me in my own CI*. The child names at most a `workflow` (GitHub: the file name,
default `toolbox.yml`) and a `ref` (default the branch the parent follows). It doesn't name its
project, host, API or token. Those come from where the parent cloned it, so the child can't
choose them. The child's `.toolbox/` is the same on GitHub and GitLab (F7). `github-workflow`
runners from before this design keep working unchanged.

### 4.3 Trust, revised

T1 of [federated-actions.md](./federated-actions.md) §5.3 becomes:

> **T1. The parent owns how it reaches a child.** A `ci` runner dispatches only to the project the
> child was cloned from, on a forge the parent declares or a known host, with that forge's API
> and token, and only if the project matches the forge's `projects`. A child whose runner names a
> `project`, `repo`, `api`, `url`, `token` or `forge` is refused, as is a `ci` child on plain git
> or on a forge without *dispatch*; the error says which. Such a child runs through the
> parent's checkout with `runner: {type: local}`, when the parent's `runners.local` allows it.

T2 to T8 are unchanged, and each applies the same way on every forge. The forge's token is read
by Toolbox's own HTTP client and never put in a process environment. git's transport
credentials are git's: a credential helper, `GIT_CONFIG_*` or SSH, as for any git client.
Toolbox doesn't put a forge token into git's configuration, because `GIT_CONFIG_*` reaches an
action's command (T2's base set) and the token would go with it.

### 4.4 A federated run in the child's CI

The parent sends six values. Their names are the same on both forges:

| Input | Value |
|---|---|
| `action` | The child action's name (`^[a-z][a-z0-9-]*$`, checked by the parent and by the pipeline's `regex`) |
| `inputs` | The run's inputs, as JSON |
| `requested_by` | The requester, as the parent recorded them |
| `toolbox_run` | The parent's run id |
| `parent_run` | `<parent portal>/<run id>` |
| `grant` | The parent's signed approvals (T5), empty when there are none |

The child's job runs `toolbox run "$TOOLBOX_ACTION" --portal .toolbox --inputs "$TOOLBOX_INPUTS"
--as "$TOOLBOX_REQUESTED_BY" --parent-run "$TOOLBOX_PARENT_RUN" --grant "$TOOLBOX_GRANT" --yes
--record toolbox-run.json -o json` and keeps `toolbox-run.json` as an artifact, also when it
fails. On GitLab the shipped job is
[`examples/federated-actions/gitlab-repo/.gitlab-ci.yml`](../../examples/federated-actions/gitlab-repo/.gitlab-ci.yml).
Its `spec:inputs` must be in `.gitlab-ci.yml` itself: GitLab reads a pipeline's inputs only from
that file, and an included file's `spec:` takes include-time inputs instead. Every input arrives
as a job variable with `expand: false`, never as text in the script, and the job runs only when
`action` is set. Exit 4 (`unverified`) is allowed as a warning, so a run with nothing to check
leaves the pipeline green. The parent then:

1. tracks the pipeline by the id GitLab returned: `created`, `pending` and `running` are
   `dispatched`; `success` ends the CI part; `failed`, `canceled` and `skipped` fail the run.
   `manual` is `dispatched` with a note that a person must start a job;
2. reads `toolbox-run.json` from the job named `toolbox`, whatever the pipeline concluded, and
   takes the child's state and verification as the run's (T8), with the child's run id. The
   artifact is the child's word, as a GitHub child's conclusion is: the parent trusts the child's
   pipeline as far as it trusts what the child commits;
3. leaves a child's `proposed` change `dispatched`, and checks it again in the parent's clone of
   the child, where the branch and its trailer are visible.

Before it dispatches, the parent previews the child in its own clone, as it would a `local`
child. That gives the run the child's approvers, blockers and effects, and the grant binds those
effects, so the child's job refuses a grant for a different change. The preview runs the
child's `change` command in the parent's clone, which is what a `local` child does too; the
change itself is made only in the child's pipeline.

The grant still expires 600 seconds after the last approval. A pipeline that waits longer for a
runner fails the child's grant check, and the run fails with that said. Approving again starts a
fresh grant.

### 4.5 Reviews and merges

A change's review is opened by whoever pushed its branch, if it can:

| Who pushed | How the review opens |
|---|---|
| The child's own pipeline, or a child whose manifest names its forge (a known host) | Push options on GitLab (`merge_request.create`, `.target`, `.title`, `.remove_source_branch`); the REST call on GitHub |
| The parent, for a `local` child on a forge the parent declares | The parent opens it with that forge's token: `POST /projects/:id/merge_requests` or `POST /repos/:repo/pulls`. GitLab can answer "source branch does not exist" for a moment after the push, so the call retries for up to ten seconds |

The review's description carries `Toolbox-Run: <child run id>`. A run is `succeeded` once the base
contains the change, seen in one of three ways: the commit is an ancestor (merge commit,
fast-forward), a commit carries the trailer (a squash that keeps the message), or a commit's subject
ends `(toolbox run <id>)`. GitLab's squash keeps only the merge request's title by default, and
the title Toolbox sets ends that way, so the third covers it.

### 4.6 People

Approvers are the portal's principals (T6), whatever forge a child is on. A GitLab-only
organisation usually signs in with GitLab itself (GitLab is an OIDC provider), so a principal
`user:dave` is GitLab's `dave`. An approver is named in the action (`user:ann`) or is a member
of one of the portal's groups, and on GitLab those groups can be GitLab's own. The `gitlab`
connector (`builtin:gitlab`, `crates/toolbox-connector-gitlab`) reports a group and its
subgroups as `group` records: the full path, the parent, and the usernames of direct members at
Developer or above (`minAccess`), with each member's access level. The org's model maps them to
`group:` tuples:

```yaml
connectors:
  - {name: gitlab, package: builtin:gitlab, runtime: in-image,
     config: {group: acme, url: "https://gitlab.acme.dev"}, records: [group, project]}
kinds:
  - name: Group
    identityFrom: [connector:gitlab]
    fields: {members: {type: list, items: string}}
    sources: {default: [connector:gitlab]}
    map: [{from: gitlab/group, id: "'group:' + record.slug", set: "{'members': record.members}"}]
authz:
  model: groups.fga                    # type group, define member: [user, group#member]
  tuples:
    - kind: Group
      emit: "entity.members.map(m, {'user': 'user:' + m, 'relation': 'member', 'object': entity.id})"
```

An action then asks for `['group:acme/release']`. In a checkout, `toolbox run` and `--approve`
run the connector each time, so an approval follows GitLab's membership at the moment it is
given. On a server, the connector's `freshness` sets how stale it may be. GitLab's inheritance
runs down (a parent group's members belong to its subgroups), the opposite of the reference
portal's `parent` tuple, so the connector reports direct members only and a model that wants
inheritance maps `parent` itself. No existing tool covers this fact: SCIM covers groups an
identity provider owns, and an organisation that syncs GitLab from one should use `builtin:scim`
instead; `glab api` returns raw pages, not records. The connector also reports `project` records
(path, default branch, archived) for a catalog; it doesn't choose children (§7).
The forge's own actor (`GITLAB_USER_LOGIN`, the dispatch token's owner) is recorded on the run.
It never replaces the requester.

A checkout has no sign-in, so it takes `--as` on trust (T6), unless a forge says who people are
(D01M87FMCVSQCQJMGDSN2MCTH5W):

```yaml
forges:
  - {url: "https://gitlab.acme.dev", type: gitlab, identity: true}
```

Then `toolbox run` needs each person's own token (`--identity-token-file`, or
`TOOLBOX_IDENTITY_TOKEN`), asks the forge who owns it, and acts as that user; a `--as` naming
someone else, a token the forge refuses, or no token at all is refused (exit 2). A `read_user`
token is enough on GitLab. The token goes to the forge's API only, never to an action's process or
a record. A child run by its parent is exempt, since the parent's signed grant names the
requester. Acceptance step [9] covers it.

## 5. Adding a forge

A driver is a `Kind` in `toolbox_model::forge` and one module in `toolbox-exec` (`github.rs`,
`gitlab.rs`). The trust rules, the manifest's shape and the run lifecycle don't change. Adding a
variant makes the compiler list every place a driver plugs in, because each is an exhaustive
`match` on `Kind`:

| Seam | Where | What the driver supplies |
|---|---|---|
| Name and defaults | `toolbox_model::forge` (`Kind::parse`, `default_api`, `default_token`, `KNOWN`) | Its `type:` name, API base, token variable, and its public host if it has one |
| Propose | `toolbox_exec::executor::open_review`, and the push in `change` | Open a review for a branch; or push options, if the host takes them |
| Dispatch | `toolbox_exec::federation::Child::ci` | Start the child's CI with the six inputs (§4.4) and return its id |
| Track, collect | `toolbox_exec::run` (recheck) | Map the host's run states to Toolbox's; read `toolbox-run.json` |
| A repo portal's own executor | `gitlab-pipeline`, `github-workflow` | Optional: start one named pipeline with typed inputs |

A driver without *dispatch* answers the `ci` arm with a refusal naming `local` (T1), so a
read-and-propose driver (Origin, below) fills in the first two rows only.

| Forge | Read, write | Propose | Dispatch, track | Collect | Notes |
|---|---|---|---|---|---|
| GitHub Enterprise Server | Yes | Yes | Yes; a server without `return_run_details` is tracked by run name (existing fallback) | Deferred, as github.com | The `github` driver with `api: <url>/api/v3` |
| **Origin** (Cursor's forge; projected from `origin` 2026.11.03's command reference) | Yes | Pull requests (`pr create`, `pr merge`, `pr requirements`) | Not in the CLI: checks are read (`pr checks`), not started | — | Arrives without *dispatch*: its children run `local`. Its rulesets (merge-time requirements, owner rules) are gates in the host, as GitLab's protected branches are. A repository mirrored from GitHub has a mirror state (`repo mirror status`, `detach`), so *propose* must go to whichever side currently owns it |
| Gitea, Forgejo | Yes | Pull requests | Forgejo Actions has `workflow_dispatch` (unverified here) | — | A third driver near GitHub's |
| Bitbucket, Azure Repos | Yes | Pull requests | Pipelines APIs (unverified here) | — | Not designed |
| A forge not yet invented | Yes, if it speaks git | Whatever it has | Whatever it has | Whatever it has | Plain git is the floor: it still offers actions, `change` and the local runner |

A driver that lacks a capability gets these fallbacks:

- without *propose*, a change is a pushed branch (as on a bare repository);
- without *dispatch*, a child runs through the parent's checkout (`local`);
- without *collect*, a federated CI run is verified by the parent re-reading the child's facts
  after the CI conclusion, and the record says `by: parent`.

## 6. Testing against a real GitLab

`scripts/gitlab/harness.sh up` starts GitLab CE and a GitLab Runner (shell executor, host
network, Toolbox's binaries mounted) in Docker and prints a root token. First boot takes
several minutes (the script waits up to ten); after that the containers are reused.
`scripts/gitlab-acceptance.sh` builds a fresh organisation in it on every run: five users and a
bot, a group with a subgroup, private projects with repo portals (one with `runner: {type: ci}`
and the shipped `.gitlab-ci.yml`), a bare repository beside them and an org portal that lists
them all by URL; step 8 adds a `release` group and the `gitlab` connector, and runs the
connector conformance kit against the real API. Its steps are the bracketed numbers in §3. It
needs Docker, not Postgres, and took 52 seconds against a warm harness on a 4-CPU machine. CI runs it in its own job
(`gitlab-acceptance` in `.github/workflows/ci.yml`).

Real runs found seven things a test double had not: a password GitLab refuses, a `local` child
on a self-managed GitLab opening no merge request, mergeability computed after the API returns,
a squash dropping the trailer, a `ci` child dispatched without its approvers (the parent had no
preview of it), the merge request API not yet seeing a just-pushed branch, and an error that
didn't say a child's manifest was invalid. Each is fixed and covered by a step. Building the
`gitlab` connector found an eighth, in Toolbox rather than GitLab: `toolbox connector test`
passed `${env:…}` references to the connector unresolved, so a connector whose token is a
reference failed the kit while `serve` and `run` worked, and the kit's report didn't say why.
The kit now resolves references as `serve` does (falling back to the text as written when they
can't resolve, as against a mock) and prints the connector's error.
The benchmark's dry run found a ninth, also in Toolbox: a checkout could not tell who was asking,
so the Toolbox arm wrapped `toolbox run` to ask GitLab who owned each person's token.
`forges[].identity` (§4.6, step 9) does that in the product.

The benchmark [experiments/actions-gitlab](../../experiments/actions-gitlab/README.md) runs
fresh agents against the same harness.

## 7. Deferred

| Item | Why not now |
|---|---|
| *Collect* on GitHub | Run artifacts are zip archives; reading one needs an unzip in the binary. A GitHub child's federated run is verified by the parent (§5 fallback) |
| Discovering children (all projects in a group) | Children are listed by URL, as on GitHub. A group listing is a connector query, and a parent should still choose what it dispatches to |
| Mapping one person across forges (`ann` on GitHub, `ann.smith` on GitLab) | Principals come from the portal's sign-in. A mapping belongs with identity resolution ([identity-resolution.md](./identity-resolution.md)) |
| Who merged a merge request (F21 on GitLab) | As on GitHub: T7 trusts the host's branch protection |
| Origin, Gitea, Bitbucket drivers | §5 |
| GitLab Premium's gates (approval rules, protected environments) in the harness | CE has none of them, so the harness can't show Toolbox's approvals composing with them. A Premium pipeline's protected environment applies after Toolbox's approval, by GitLab's own rules |

## 8. Open questions

| # | Question |
|---|---|
| FG1 | Should `projects` allow `**` (any depth) for GitLab's nested groups, or is listing each subgroup clearer for a trust rule? |
| FG2 | When a repository is mirrored on two forges (Origin's GitHub mirrors, GitLab pull mirrors), should the parent read the mirror state to choose where *propose* goes, or should the parent name the owner? |
| FG3 | Should a GitLab parent dispatch with a pipeline trigger token (scoped to one project) instead of a personal or group access token? Trigger tokens can't read the pipeline back, so *track* would need a second token |

## Sources

- GitLab `gitlab-org/gitlab@master`: `doc/ci/variables/_index.md` (restricting pipeline
  variables, `no_one_allowed` since 17.7), `doc/api/pipelines.md` (create with `inputs`),
  `doc/ci/inputs/_index.md`, `doc/topics/git/merge_requests` push options
  (`merge_request.create`), `doc/ci/jobs/ci_job_token.md` (job token pushes); read 2026-11-24.
- Hands-on: GitLab CE 19.6.1 and GitLab Runner 19.6.1 in Docker on this machine. A pipeline was
  created with inputs and variables, a job read them, an input containing `$(whoami)` arrived
  literally through job `variables:`, a job pushed with `CI_JOB_TOKEN`, push options opened a
  merge request, and a job artifact was read by path (research log, 2026-11-24).
- GitHub: [workflow dispatch run ids](https://github.blog/changelog/2026-02-19-workflow-dispatch-api-now-returns-run-ids/).
- Origin: `origin agent prompt` and `origin agent prompt reference` from origin
  2026.11.03-22-10-56-fbcd8ba (stable), read 2026-11-24. Not run against a repository.
- Toolbox: [federated-actions.md](./federated-actions.md), [federation.md](./federation.md),
  D01M86BY3YQX3SA1W99QXF1QA1H, D01M86J769VGE6K5FJJQ76EP4AA, D01M8727W7FEED6VVRY6MB3JFR3.
