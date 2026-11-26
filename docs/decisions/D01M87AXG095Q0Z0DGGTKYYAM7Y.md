---
id: D01M87AXG095Q0Z0DGGTKYYAM7Y
decided: 2026-11-24T18:30:00Z
---

# Forges are drivers behind six capabilities; GitHub is a default, not an assumption

**Extends D01M86BY3YQX3SA1W99QXF1QA1H and D01M86J769VGE6K5FJJQ76EP4AA.** Federated actions assumed
GitHub wherever they needed more than git: pull requests, `workflow_dispatch`, `GITHUB_TOKEN`. An
organisation on GitLab, on a GitHub Enterprise Server, on both, or on a host not yet built,
needs the same lifecycle. Tested against a real GitLab CE 19.6.1 and runner
(`scripts/gitlab-acceptance.sh`).

1. **A forge offers up to six capabilities**: *read* and *write* (git), *propose* (a review),
   *dispatch* (start its CI with inputs), *track* (follow that run by id) and *collect* (read the
   child's run record from it). Plain git is the floor and has the first two. A missing
   capability falls back: no *propose*, the pushed branch is the proposal; no *dispatch*, the
   child runs `local`; no *collect*, the parent verifies after the CI conclusion.
2. **`forges: [{url, type?, api?, token?, projects?}]`, top level.** A git URL is on the forge
   whose `url` is its longest prefix, or whose host it names (ssh). `https://github.com` and
   `https://gitlab.com` are known: no entry, `GITHUB_TOKEN` or `GITLAB_TOKEN`. `type` is
   `github` (github.com and Enterprise Server, `<url>/api/v3`) or `gitlab` (gitlab.com and
   self-managed, `<url>/api/v4`). `token` names an environment variable, never a value.
   `projects` globs (`*` within one path segment) narrow where a parent may dispatch; without it,
   a parent may dispatch to the children it lists.
3. **`federation.runner: {type: ci, workflow?, ref?}` replaces naming a repository.** The child
   says *run me in my own CI*; the parent dispatches to the project it cloned, on that project's
   forge, with that forge's token. T1 becomes: the parent owns how it reaches a child. A runner
   naming `project`, `repo`, `api`, `url`, `token` or `forge` is refused: the schema refuses all
   but `repo` (which `github-workflow` uses), and the parent refuses all six before it dispatches. `github-workflow` runners keep working.
4. **The parent previews a `ci` child in its own clone before it dispatches**, as it does a
   `local` child, so the run has the child's approvers, blockers and effects and the grant binds
   them. Before this, a GitHub `ci` child was dispatched with no approvers and no grant.
5. **GitLab pipelines take inputs, not variables.** The parent sends `action`, `inputs`,
   `requested_by`, `seikika_run`, `parent_run` and `grant` as pipeline inputs; the shipped
   `.gitlab-ci.yml` maps each to a job variable with `expand: false`. New gitlab.com projects
   refuse pipeline variables; inputs are not restricted, and the job never interpolates them
   into script text. The parent reads `seikika-run.json` from the `seikika` job's artifacts
   whatever the pipeline concluded. Exit 4 (`unverified`) is a warning in the job.
6. **A review is opened by whoever pushed.** A pusher that knows its host uses push options on
   GitLab, or the REST call on GitHub; when the parent pushed a `local` child's branch to a forge
   it declares, the parent opens the review with that forge's token. The review's title ends
   `(seikika run <id>)` and its description carries `Seikika-Run:`, and a run is merged when its
   commit is an ancestor of the base, a commit carries the trailer, or a subject ends with the
   marker (GitLab's squash keeps only the title).
7. **A new forge is a `Kind` and one module.** Each seam is an exhaustive `match` on `Kind`, so
   the compiler lists them: name and defaults, propose, dispatch, track and collect, and an
   optional executor of its own (`gitlab-pipeline`, beside `github-workflow`). Origin, Cursor's
   forge, maps to *read*, *write* and *propose* from its CLI's documented commands; its children
   run `local` until it has a dispatch.
8. **The `gitlab-pipeline` executor** starts one named project's pipeline from any portal, with
   the run's inputs as typed pipeline inputs, tracked by id; GitLab's refusal is the run's error.

Rejected: a `forge:` field on the child (it would let a child choose where the parent's token
goes, T1); putting forge tokens into git's configuration for clones (`GIT_CONFIG_*` reaches
action commands, T2), so clones use git's own credential helpers; a trait object per forge (the
lifecycle has five seams, and an enum makes a missing one a compile error); pipeline trigger
tokens for GitLab dispatch (they can't read the pipeline back, so *track* would need a second
token; open as FG3); and GitHub artifact collection now (zip archives, so deferred).

Owning docs: [forges.md](../design/forges.md), [federated-actions.md](../design/federated-actions.md)
§5.3, [examples/federated-actions](../../examples/federated-actions/README.md).
