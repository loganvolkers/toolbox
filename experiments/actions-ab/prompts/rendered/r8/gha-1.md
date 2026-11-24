You are a platform engineer at Acme. Work only inside /tmp/actions-ab/r8/gha-1: do not read or search files outside it, except the system's own tools and the paths "Your platform" names below.

## The org

- Three repositories: `alpha`, `beta` and `gamma`. Each has a bare origin at `/tmp/actions-ab/r8/gha-1/origins/<name>.git`, which plays the code host (its `main` is where changes land), and a checkout at `/tmp/actions-ab/r8/gha-1/<name>` that tracks it.
- `/tmp/actions-ab/r8/gha-1/org` is the org-level place. Its origin is `/tmp/actions-ab/r8/gha-1/origins/org.git`.
- People: **ann** and **bob** maintain all three repositories. **carol** is a developer. **dave** and **erin** are the **release** group, which signs off on what readers of the repositories are told. Anyone in the org may request a run. There are no passwords in this sandbox, so saying who you are is enough to act as someone, but your system must record who did what and enforce the approval rules below.
- If you start a server, listen on port 7120 (or 7121 for a second one). Other workspaces share this machine: start the name of any tmux session or other named resource with `r8-gha-1-`.

## The task

Build a self-service action, `add-notice`, that each repository's maintainers own, and run it on all three repositories.

`add-notice` puts a notice at the top of the repository's root README (`README.md`, in any letter case) on its origin's `main`. The README's new first line is

```
> **Info:** <message>
```

or `> **Warning:** <message>` for level `warning`, then one blank line, then the README as it was. If the README already starts with such a notice line followed by a blank line, the new notice replaces them, so a README never has more than one notice. Nothing else in the README changes.

Inputs:
- `level`: `info` or `warning`;
- `message`: one line of 10 to 160 characters.

Requirements:

| # | Requirement |
|---|---|
| R1 | **Owned by the repository.** Each repository's definition of the action lives in that repository, committed to its origin's `main`, so the repository's team owns it and can change it |
| R2 | **Validated.** Invalid inputs are refused before anything changes |
| R3 | **Previewed.** Before a run takes effect, the requester can see what it would change |
| R4 | **Approved by the release group.** A run takes effect only after a member of the release group other than the requester approves it. The requester cannot approve their own run, and nobody outside the release group can approve |
| R5 | **Done means observed.** The system reports a run as succeeded only after it has seen the README with the right content on the origin's `main`. If the change does not reach `main` (for example, the push is rejected), the run must not be reported as succeeded |
| R6 | **One place for the org.** From `/tmp/actions-ab/r8/gha-1/org`, a person or an agent can list `add-notice` for all three repositories and request a run on any of them, without going into each repository |
| R7 | **Audited.** Each run's record shows who requested it, who approved it, its inputs, its outcome, and when each of those happened |
| R8 | **Usable by agents.** Every step (list, preview, request, approve, check the status, read the record) works non-interactively from a shell, with machine-readable output |
| R9 | **The group is defined once.** Who is in the release group is defined in one place at the org level, not in each repository. Changing it there changes who may approve, for all three repositories |

Then, with what you built, request `add-notice` on each repository as ann, have dave approve alpha's run and erin approve beta's and gamma's, and confirm that each run is reported as succeeded:

| Repository | level | message |
|---|---|---|
| alpha | `warning` | This proxy is frozen while the v2 rewrite lands; please file new issues against v2. |
| beta | `info` | Releases now ship on the shared release train every second Tuesday. |
| gamma | `info` | Graph extraction now lives in graphify-core; this package re-exports it. |

Finally, write `/tmp/actions-ab/r8/gha-1/org/HANDOFF.md`. For each of R1 to R9, say how your system meets it, with file paths. Give the exact shell commands someone else would use to start anything that must be running, list the actions, preview, request (as ann), approve (as dave or erin), check a run's status, read its record, and change who is in the release group. Someone will follow it to test your system, including with invalid inputs, self-approval, approvals by people outside the release group, a change to the group, and a rejected push of a change that is not yet on `main`. Stop any long-running process you started before you finish.

## Your platform

Use **GitHub Actions**. There is no GitHub here, so run workflows locally with act, at `/tmp/actions-ab/r8/gha-1/bin/act`. There is no Docker either: use act's host mode, `-P ubuntu-latest=-self-hosted`. Its documentation is in `/tmp/actions-ab/r8/gha-1/gha-docs`: GitHub's own docs pages for Actions, branch rules and code owners, with the REST reference for actions, deployments and repositories (`github-docs/`, from github/docs, unrendered), act's README at v0.2.98 (`act/`), and the act user guide (`act-docs/`, last changed 2025-05-01, so older than this act). `act --help` also works. The origins here are local paths, so under act `github.repository` is an origin's path, not `acme/<name>` as on GitHub. GitHub features that act cannot run, such as environments with required reviewers, preventing self-review, teams, branch protection and the run history, count if you configure them as code in the repository or the org (a workflow, or a script that applies the settings through the GitHub API) and explain in `HANDOFF.md` how GitHub would enforce them. You need not rebuild such a feature here. Where GitHub would perform a step (a reviewer approving, refusing a self-approval, keeping the run history), `HANDOFF.md` may give the GitHub command for it (`gh` or a REST call); a test of that step is credited when its configuration is committed and explained. For the three runs, say which steps GitHub would have performed. Say in `HANDOFF.md` what you could not run here. Write whatever glue GitHub Actions doesn't provide yourself, in any language. You may install packages.

Stop when the three runs have succeeded and `HANDOFF.md` is written. Reply with a short summary.
