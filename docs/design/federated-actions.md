# Federated Self-Service Actions: Defined Where the Knowledge Is, Run Where the Credentials Are

*Status: design and v1 implementation, 2026-11-24 (D01M86BY3YQX3SA1W99QXF1QA1H). This is about the box:
carrying actions between portals, running them, remembering their runs and reporting their
outcome. The tools are what actions call: CI workflows, `git`, scripts, a template engine. It
revises [federation.md](./federation.md) (D68) and implements [action-runs.md](./action-runs.md)
(D49), which M5 left paused (D64). The market research behind it is summarised in §2, with its
sources at the end.*

---

## 1. Conclusion

**An action is declared in the portal of whoever owns its effect, and a parent portal offers it
by reading the declaration, never by copying it. A run goes down to the portal that owns the
effect, which previews, runs and verifies it with its own executor, credentials and gates. A run
succeeds only when its effect is observed.**

- **The four flows of federation.** Two are revised, two are new.

  | Flow | What moves | How |
  |---|---|---|
  | Standards down | Kinds, scorecards, policies, and now actions | Packages a child `include`s (D89). A package action runs in the child that includes it |
  | Results up | Scorecard results, entities | Committed reports and connector records ([federation.md](./federation.md) §4) |
  | **Actions up** | Each child's declared actions, as data | The workspace connector's `portal` records. The parent offers them on the child's entity |
  | **Runs down** | A request to run one, with who asked and who approved | The child runs `toolbox run` in its checkout (`runner: local`) or its own CI workflow (`runner: github-workflow`) |

- **One action, one lifecycle, every level.** A repo portal runs its actions with no server
  (`toolbox run`). A served portal runs the same definition through REST and MCP. A parent runs a
  child's through the child. All three call one preparation step, so a preview means the same
  thing wherever it is asked.
- **Trust stays with the child.** The parent never runs a child's code with its own credentials.
  A served parent refuses a child that declared no runner. A repo portal holds no approvals of its
  own, and accepts a parent's only when it names that parent. A change goes to review, and review
  is its approval.
- **This fills the gap the market leaves.** Every IDP we surveyed defines actions centrally. The
  repo-level action already exists as the repo's CI workflow, but no portal discovers, governs and
  verifies repo-declared actions. Trust is the reason, and the trust rules in §5.3 are the answer.

## 2. What the market enables, and where each action belongs

The research brief read 13 product families first-hand: Backstage, Port, Cortex, OpsLevel, Roadie,
Spotify Portal, Compass, HCP Terraform, Humanitec/Score, Kratix/Crossplane, GitHub, GitLab and
ITSM. Its findings:

1. **Every IDP defines actions centrally** (Port, Cortex, OpsLevel, Compass). Repo-declared actions
   reach a portal only through gated doors: Backstage's catalog rules reject the `Template` kind
   by default, and Cortex accepts workflows from git only in a dedicated, allowlisted repository.
2. **The per-repo action already exists: it is the repo's CI workflow.** Port's GitHub backend,
   Backstage's `github:actions:dispatch` and OpsLevel's CI templates are front doors to it.
3. **"Dispatched" is the market's meaning of "done".** Port, OpsLevel and Backstage report a run
   finished when the backend says so. Only control planes (Crossplane `Ready`, Kratix) and Spotify's
   Fleetshift (a Soundcheck check flips to passing) observe the effect.
4. **Templates drift.** Backstage #14416 and #31361 ("update components generated from templates")
   were closed *not planned*. The tools that solve drift are repo-level: `copier update`,
   `cruft check`, projen, `nx migrate`.
5. **Fleet changes are the fastest-growing central action**: defined once, a pull request per
   repo, done when each repo's check passes (Fleetshift, OpsLevel Maintenance Agent, Port's
   self-healing scorecards).
6. **The strongest gates live in the repo or the target system**: GitHub environments with
   required reviewers and self-review prevention, GitLab protected environments.
7. **Credentials decide where an action can run.** Backstage's threat model warns that the
   scaffolder runs with elevated permissions. Running a repo-declared action with a portal-wide
   credential is the dangerous combination.
8. **One definition for UI, API and MCP is table stakes** (Backstage Actions Registry, Port
   Workflows as MCP tools, Roadie Actions).

**The rule we draw from it.** Put an action's *definition* at the level that owns the decision it
hides (Parnas), and its *run* where the credentials and gates already are.

| Action (families documenting it, of 13) | Definition home | Run place | Toolbox shape | Story |
|---|---|---|---|---|
| Create a service from a template (7) | Company or org | Central: no repo exists yet | A `create` action in the company portal | S8 |
| Deploy, promote, roll back (7) | Repo pipeline, under org policy | Repo CI, behind its environments | `github-workflow` executor in the repo portal | S4 |
| Trigger a pipeline (6) | Repo | Repo CI | `github-workflow` executor | S4 |
| Add a capability to a repo by PR (6) | Org or company package | The target repo | A package `change` action | S3, S9 |
| Provision a resource (6) | Platform defines; repo requests | Platform executor | A central action over a repo-declared request | S11 (deferred) |
| Fix a scorecard failure (5) | The standard's owner | Each repo, by PR | A package `change` action whose postcondition is the rule | S3 |
| Request access, with expiry (4) | Company | Central, as the user | A company action with approvers | S5 |
| Incident and on-call (4) | Company integration | Central | A company action on the owner's entity | S5 |
| Restart, scale, flags, day-2 runbooks (2 each) | Repo or team | Repo CI or a team package | Repo `command` or team package actions | S1, S10 |
| Decommission (2, weak evidence) | Repo initiates; platform tears down | Central and repo | Deferred until users ask | S12 (deferred) |
| Rotate a secret (0 found) | Unknown | — | Not designed | — |

## 3. Personas

| Persona | Level | Wants | Today |
|---|---|---|---|
| **App developer** | Repo | Run the repo's day-2 operations without a ticket | `gh workflow run`, a Makefile, or a ticket to the platform team |
| **Repo maintainer** | Repo | Own what runs against the repo; approve what matters | Branch protection, environments, CODEOWNERS |
| **Platform engineer** | Company or org | Publish golden paths and fleet fixes once | Backstage templates, Fleetshift-style scripts |
| **Team lead** | Team | Publish the team's runbooks without the platform team | A wiki page, a shared repo of scripts |
| **Approver** | Any | Approve someone else's run, see what it will do | Port approvals, GitHub required reviewers |
| **Agent** | Any | Find an action, preview it, run it, know when it's done | MCP tools that return at dispatch |

## 4. User stories

Each story names the level that defines the action, where it runs, and a check that decides
"done". *Built* stories are exercised by `scripts/m5-acceptance.sh` (§7); its step numbers are in
brackets.

| # | Story | Defined at | Runs at | Done when | Status |
|---|---|---|---|---|---|
| **S1** | As an **app developer**, I want my repo's day-2 operations declared in `.toolbox/` next to the code, so the team that builds it runs it, from a checkout or a portal | Repo | The repo's checkout or CI | `toolbox run write-readme -i text=…` previews the effect, runs it, and ends `succeeded` only after the repo's facts show it [1] | Built |
| **S2** | As a **repo maintainer**, I want an action that changes code to arrive as a pull request, so review is its approval and nothing lands on main unreviewed | Repo | A branch from the remote's base | The run is `proposed` with the branch and a `Toolbox-Run` trailer, stays `proposed` while unmerged, and is `succeeded` once merged and observed on the base [4] | Built |
| **S3** | As a **platform engineer**, I want to publish a fleet fix once, in a package, and run it on every repo that includes it, so a standard doesn't need N teams to act by hand | Org package | Each repo, as a pull request | One org action opens a pull request per repo, and each run succeeds when that repo verifies the merged change [6] | Built |
| **S4** | As a **release manager**, I want deploys to run in the repo's own workflow, behind its own environment reviewers, so a portal can never bypass a gate | Repo workflow | The repo's CI | The run is `dispatched`, tracked to the workflow's conclusion, and verified by the postcondition. The repo's environment rules decide approval | Built (executor and tracking; tested against a fake GitHub API) |
| **S5** | As a **security owner**, I want access-granting actions to need a maintainer's approval, never the requester's own | Company or repo | Where the credential is | A run is `awaiting_approval`; the requester's own approval is refused (`not-an-approver`); another maintainer's approval runs it [3, 7] | Built |
| **S6** | As an **agent**, I want to discover, preview and run actions over MCP or the CLI, and be told "done" only when the effect is observed | Anywhere | Wherever the executor is | `preview_action` returns effects and a confirm token; `run_action` runs it; `get_run` ends `succeeded` with the verification [7] | Built |
| **S7** | As an **approver at the org**, I want to approve a run once in the org portal and have the repo accept it, so a repo portal needs no approval service | Repo (approval rule) | The repo | The repo runs a parent's dispatch with the parent's recorded approval only when it lists the parent in `acceptApprovalsFrom` [7] | Built |
| **S8** | As a **platform engineer**, I want "create a service" as a versioned golden path that records its template version | Company | Central | A `create` action whose postcondition is the new repo's entity, with the template version as a field | Designed (the `create` scope exists; no reference template yet) |
| **S9** | As a **repo owner**, I want "update from template" to open a pull request with the template's newer version, and a rule that flags drift | Template's package | The repo | A package `change` action (copier/cruft semantics), verified by the drift rule passing | Designed (S3's mechanism; no reference package yet) |
| **S10** | As a **team lead**, I want my team's runbooks in a team package that only our repos include | Team package | Each including repo | The org portal offers the team's actions only on the repos that include the package [5] | Built (package actions flow up) |
| **S11** | As a **developer**, I want to request a database by declaring it next to my code, fulfilled by a platform-owned action | Request in repo; fulfilment at the platform | Platform executor | — | Deferred: needs an action triggered by a record, not a person |
| **S12** | As a **service owner**, I want a decommission action that archives, tears down and unregisters, verified when connectors no longer see the resources | Repo and platform | Central and repo | — | Deferred: thin market evidence; validate with users first |

**What the stories show.** S1 to S7 and S10 put the definition in the repo or a package and the
run in the repo. Only S5 (access), S8 (create) and S11 (provisioning) need a central executor,
because only they need credentials no repo holds. That is the thin central core Team Topologies and
DORA argue for: most actions belong to the teams that own the effect.

## 5. Federation, revised

### 5.1 What changed from federation.md

[federation.md](./federation.md) federated configuration down and results up, with **no runtime
coupling**. Actions need one runtime edge: a parent must ask a child to run something. The
revision keeps every other property.

| federation.md (D68) | Now |
|---|---|
| No portal calls another at runtime | No portal *queries* another. A run is the one call that goes down, and only to a child that declared how (`federation.runner`) |
| Packages carry kinds, scorecards and policies | Packages carry actions too. A package action runs in the child that includes it |
| Waivers' approvals wait on M5 (FQ3) | Approvals exist: recorded on the run, by principal, with self-approval refused |
| Nothing implemented | Actions up, runs down and package actions are built (§7) |

### 5.2 The parent's view

A parent names the kind whose entities are its children. The workspace connector finds each
child's manifest and reports it as a `portal` record, packages included:

```yaml
# examples/federated-actions/org/toolbox.yaml (abridged)
connectors:
  - name: workspace
    package: builtin:workspace
    config: {root: .., crates: false, portals: ["*/.toolbox/toolbox.yaml"]}
    records: [portal]
kinds:
  - name: Portal
    fields: {name: …, path: …, actions: {type: list, items: object}, runner: {type: object}}
    map:
    - from: workspace/portal
      id: "'portal:' + record.name"
federation: {portals: Portal}
```

Every child action is then offered on the child's entity (`portal:beta`), with the parent's
permission (`can_run_actions` by default) and the child's input schema. The parent drops
`enumFrom`, because only the child can compute those options. The child validates them when it
previews.

### 5.3 Trust rules

| # | Rule | Enforced by |
|---|---|---|
| T1 | **The parent never runs a child's code with its own credentials.** The child runs it, with its executor | The `toolbox` executor runs `toolbox run` in the child; `github-workflow` dispatches the child's workflow |
| T2 | **A child decides whether it can be reached.** A served parent refuses a child with no `federation.runner` (`409 no-runner`), and names the checkout command | `toolbox serve` |
| T3 | **The child previews.** A parent's preview is the child's preview, so blockers, effects and approvals are the child's | Both previews come from one `prepare` |
| T4 | **A repo portal holds no approvals.** An action with approvers ends `awaiting_approval` (exit 3) in a checkout | `toolbox run` |
| T5 | **A child trusts a parent's approvals only by name.** A run dispatched with `--parent-run org/<id> --approved-by …` counts the approvals only if `acceptApprovalsFrom` lists `org` | `toolbox run` |
| T6 | **Nobody approves their own run, and an approver is someone named.** Each approval entry is a user, a group, or a list meaning any one of them. A group needs an authorization model to know its members; without one it can't be met, rather than meaning anyone | The run lifecycle and `toolbox serve` (`not-an-approver`) |
| T7 | **Review is a change's approval.** A `change` action's approvers approve by reviewing and merging its pull request, behind the repo's branch protection; the run waits as `proposed`, not `awaiting_approval` | `by_review` in the preview |
| T8 | **The child verifies.** A federated run's verification is the child's, recorded as `{by: child portal, child_run, child}` | `recheck` asks the child |

### 5.4 Runners

| Runner | How a parent dispatches | Where it fits | Credentials |
|---|---|---|---|
| `local` | `toolbox run <action> --portal <child> --parent-run <parent>/<id> --approved-by …` in the child's checkout | A monorepo, a fleet checkout, a developer laptop, the acceptance script | The checkout's own: git push rights, local tools |
| `github-workflow` | `workflow_dispatch` of the child's workflow with `action`, `inputs`, `requested_by` and `toolbox_run`; the workflow runs `toolbox run` | A served org portal over repos in GitHub | The repo's `GITHUB_TOKEN`, environment secrets and OIDC, behind its environment reviewers |

## 6. One action at every level

### 6.1 The definition

```yaml
# examples/federated-actions/repo/.toolbox/toolbox.yaml
actions:
  - name: set-owners
    kind: Repository
    scope: day2
    description: >-
      Make a team the owner of every path (writes CODEOWNERS directly;
      maintainers approve).
    inputs:
      type: object
      required: [team]
      properties: {team: {type: string, pattern: "^@[a-z0-9-]+/[a-z0-9-]+$"}}
    permission: can_run_actions          # OpenFGA relation on the target entity
    approval: "[['user:ann', 'user:bob']]" # CEL: each entry must approve; a list is any one of it
    executor:
      type: command                       # or change, github-workflow, in-image
      run: >-
        mkdir -p .github && printf '* %s\n' "$TOOLBOX_INPUT_TEAM" >
        .github/CODEOWNERS
    expect:
      postcondition: has(entity.codeowners) && inputs.team in entity.owners
      within: 30s
```

### 6.2 Executors

| Type | Runs | Preview | Ends at | Verified by |
|---|---|---|---|---|
| `command` | A command in the checkout, with `TOOLBOX_INPUT_*` | Toolbox's, plus the command's own when `preview: true` (`TOOLBOX_PHASE=preview`) | Its exit | Re-reading the portal's facts |
| `change` | A command in a worktree of the remote's base; commits it to `toolbox/<action>/<run>` with `Toolbox-Run` and `Requested-By` trailers, and pushes | The files it would change | `proposed` | The facts at the base branch, once merged |
| `github-workflow` | `workflow_dispatch`, tracked by run id | Toolbox's | `dispatched`, then the workflow's conclusion | The postcondition after a refresh |
| `in-image` | The executor protocol over stdio (D40) | The executor's | Its terminal message | The postcondition, or the executor's `verify` |
| `toolbox` (federated) | The child's `toolbox run` | The child's | The child's state | The child |

### 6.3 Lifecycle, tokens and records

- **States** are [action-runs.md](./action-runs.md) §2's, plus `proposed` for a change under
  review. A run is `succeeded` only when its postcondition held at a snapshot after the run. An
  action with no postcondition ends `unverified`, never `succeeded`.
- **Preview and confirm.** A preview returns the effects, blockers, warnings, approvers and a
  confirm token, `ct_<issued>_<hash>`. The hash covers the action version, inputs, effects and
  snapshot, and the token lives for 600 seconds. A changed preview is `preview_stale`: CLI exit 1,
  HTTP `409 preview-stale`.
- **Exit codes** (CLI): 0 succeeded, proposed or dispatched; 1 failed or stale; 2 invalid inputs or
  refused; 3 awaiting approval.
- **Records.** Every run is a JSON record, carrying who requested it, through which surface, the
  approvals with notes, every state change with time and actor, the effects, the outputs and the
  verification. `toolbox run` writes it to the state directory, and `toolbox run --verify <id>`
  checks a waiting run again. A served portal keeps runs in memory for now (§7.2), and re-checks
  waiting ones every 10 seconds.

### 6.4 Interfaces

| Need | CLI (repo or parent, no server) | REST (served) | MCP (served) |
|---|---|---|---|
| List | `toolbox run` | `GET /v1/actions` | `list_actions` |
| Preview | `toolbox run <action> -i k=v` | `POST /v1/actions/{name}/previews` | `preview_action` |
| Run | `--confirm <token>`, or `--yes` | `POST /v1/actions/{name}/runs` | `run_action` |
| Approve | — (T4) | `POST /v1/runs/{id}/approvals` | `approve_run` |
| Check | `--verify <id>` | `GET /v1/runs/{id}`, `POST /v1/runs/{id}/verify` | `get_run`, `list_runs` |
| Cancel | — | `POST /v1/runs/{id}/cancel` | — |

## 7. What is built, and what is deferred

### 7.1 Acceptance

`scripts/m5-acceptance.sh` builds a local org with four repos (bare origins, checkouts and
reviewer clones), an org portal over them, and an org package vendored into each repo. It needs no
network and no Postgres.

| Step | Checks | Stories |
|---|---|---|
| 1 | A repo action: list, preview, confirm, observed | S1 |
| 2 | A stale token exits 1 (`preview_stale`); invalid inputs exit 2 | S1, S6 |
| 3 | A repo portal holds no approvals: exit 3, `awaiting_approval` | S5 (T4) |
| 4 | A change is a pull request with the trailer; stays `proposed` until merged; then `succeeded` | S2 (T7) |
| 5 | The org offers every child's actions, packages included; a federated run is verified by the child | S6, S10 (T1, T8) |
| 6 | A fleet change: one org action, three pull requests, three verified runs | S3 |
| 7 | The served org: a child without a runner is refused; approval by the org, refused for the requester, accepted for another maintainer, accepted by the child; MCP preview, run and check | S5, S6, S7 (T2, T5, T6) |

### 7.2 Deferred, with the reason

| Item | Why not now |
|---|---|
| Durable server runs (Postgres, D39) | Runs live in memory in `serve`; CLI runs are files. Durable runs are the durable-runtime track |
| A `toolbox run` GitHub Action for `runner: github-workflow` children | The dispatch is built and tested against a fake API; the composite action that receives it is next |
| A federated github-workflow run's verification | The parent sees the workflow's conclusion; the child's own verification needs the child's run record as a workflow artifact |
| Duplicate child portal names | Two children with one name shadow each other; validation should name both |
| Fan-out in one command | A fleet change is one run per child (a loop); `--all` waits for a use beyond scripts |
| S8, S9, S11, S12 | §4 |

## 8. Against the alternatives

The benchmark [experiments/actions-ab](../../experiments/actions-ab/README.md) measures this table
with agents. This is the designer's reading, not a result.

| Property | Toolbox | GitHub Actions (`workflow_dispatch`) | Backstage scaffolder | A vanilla app |
|---|---|---|---|---|
| Definition lives in the repo | Yes (`.toolbox/`) | Yes (`.github/workflows/`) | A `Template` entity in any registered location, if catalog rules allow it | Wherever it is written |
| Typed, validated inputs before any effect | JSON Schema, checked at preview | Typed inputs (string, choice, boolean, number, environment), no pattern | JSON Schema parameters | Hand-written |
| Preview of the effect | Every run, bound to a confirm token | None | Dry run for actions that support it | Hand-written |
| Approval, self-approval refused | Approvers in the action; requester refused | Environment reviewers, "prevent self-review" (repo settings) | Not built in | Hand-written |
| Done means observed | Postcondition over re-read facts | The job's exit status | The task's steps returning | Hand-written |
| One place for many repos | A parent portal reads children's actions | Per repo; reusable workflows share steps | One catalog of templates | Hand-written |
| Audit of who ran and approved | The run record | The run history | Task logs and audit events | Hand-written |
| Agents | MCP tools and CLI exit codes | `gh workflow run` | Actions Registry MCP (alpha) | Hand-written |

## 9. Open questions

| # | Question |
|---|---|
| FA1 | Should a parent be able to *restrict* a child's action (who may run it from the parent) beyond `federation.permission`? Today the parent's permission and the child's approval rule both apply |
| FA2 | Should a fleet change be one run with child runs, or N runs? N runs keeps each child's record whole; one run is easier to watch |
| FA3 | How does a repo portal take approvals from a person without a server? Today only a parent's dispatch carries them (T5) |
| FA4 | Is the workspace connector the right way to find children in a large org, or should children push their `portal` record (federation.md option B)? |

## Sources

- Research brief, 2026-11-24, by an independent research agent: 13 product families read
  first-hand, with every claim tagged primary, secondary or inference. Its primary sources include:
  - Backstage `backstage/backstage` at `b3ed9690` (scaffolder actions, threat model, catalog rules,
    Actions Registry) and issues
    [#14416](https://github.com/backstage/backstage/issues/14416) and
    [#31361](https://github.com/backstage/backstage/issues/31361);
  - Port, Cortex and OpsLevel documentation on actions, backends, approvals and run status;
  - GitHub's [workflow dispatch run ids changelog](https://github.blog/changelog/2026-02-19-workflow-dispatch-api-now-returns-run-ids/),
    [workflow syntax](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax)
    and [environments](https://docs.github.com/en/actions/reference/workflows-and-actions/deployments-and-environments);
  - Spotify Portal's Fleetshift documentation; copier, cruft, projen and Nx documentation;
  - DORA's [platform engineering capability](https://dora.dev/capabilities/platform-engineering/);
    Team Topologies key concepts; Parnas (1972); Conway (1968); Vogels (2006).
- Toolbox: [federation.md](./federation.md), [action-runs.md](./action-runs.md),
  [action-entities.md](./action-entities.md), [packages.md](./packages.md),
  [repo-portals.md](./repo-portals.md), D49, D64, D68, D89.
