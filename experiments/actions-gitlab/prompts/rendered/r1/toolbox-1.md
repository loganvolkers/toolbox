You are a platform engineer at Acme. Work only inside /tmp/actions-gitlab/r1/toolbox-1: do not read or search files outside it, except the system's own tools and the paths "Your platform" names below. Start every shell with `export HOME=/tmp/actions-gitlab/r1/toolbox-1/home`, so git and glab act as you.

## The org

- Acme's code lives on a self-managed **GitLab** (Community Edition 19.6.1) at `http://127.0.0.1:8929`, in the group `acme-w11`. Its web UI and REST API (`/api/v4`) are there; there is no browser here, so use the API, `git`, or `glab`.
- Three projects: `acme-w11/alpha`, `acme-w11/beta` and `acme-w11/gamma`, with checkouts at `/tmp/actions-gitlab/r1/toolbox-1/alpha`, `/tmp/actions-gitlab/r1/toolbox-1/beta` and `/tmp/actions-gitlab/r1/toolbox-1/gamma`. A project's `main` is where changes land. `acme-w11/org` is the org-level project, empty, with its checkout at `/tmp/actions-gitlab/r1/toolbox-1/org`.
- You are the GitLab user `ops-w11`, an Owner of `acme-w11`. Your API token is in `/tmp/actions-gitlab/r1/toolbox-1/people/ops.token`, and `git` pushes as you (`/tmp/actions-gitlab/r1/toolbox-1/home/.gitconfig`).
- People, each a GitLab user with an API token in `/tmp/actions-gitlab/r1/toolbox-1/people/<name>.token` (acting as someone means using their token): **ann** and **bob** are Maintainers of `acme-w11`; **carol** is a Developer; **dave** and **erin** are Developers and the only direct members of the subgroup `acme-w11/release`, the **release group**, which signs off on who owns code. Anyone in the group may request a run. Your system must record who did what and enforce the approval rules below.
- `/tmp/actions-gitlab/r1/toolbox-1/bin/glab` is the GitLab CLI (1.132.0); put `/tmp/actions-gitlab/r1/toolbox-1/bin` on `PATH`. `/tmp/actions-gitlab/r1/toolbox-1/gitlab-docs` is this GitLab version's own documentation, as Markdown.
- One GitLab Runner serves the whole instance: a shell executor that runs untagged jobs, with bash, git, curl, python3 and jq. Jobs reach GitLab at the same address.
- Other workspaces share this GitLab and its runner. Change nothing outside `acme-w11`: no instance settings, no runners, no other groups or users. Start the name of anything you create outside `/tmp/actions-gitlab/r1/toolbox-1` (a tmux session, a token) with `r1-toolbox-1-`. If you start a server, listen on port 7310.

## The task

Build a self-service action, `set-codeowners`, that each project's maintainers own, and run it on all three projects.

`set-codeowners` sets one rule in the project's root `CODEOWNERS` file on `main`. A rule is a line `<pattern> <owners>`, the pattern and the owners separated by single spaces. If a line in the file already starts with that pattern as its first field, the first such line is replaced by the rule; otherwise the rule is appended as the file's last line, and the file is created if there is none. The file ends with a newline. Nothing else in it changes.

Inputs:
- `pattern`: starts with `/` or `*`, has no whitespace and no `#`, at most 100 characters;
- `owners`: one to three owners separated by single spaces, each `@` followed by a GitLab user or group path (letters, digits, `_`, `.`, `-`, with `/` between path segments).

Requirements:

| # | Requirement |
|---|---|
| R1 | **Owned by the project.** Each project's definition of the action lives in that project, committed to its `main`, so the project's team owns it and can change it |
| R2 | **Validated.** Invalid inputs are refused before anything changes |
| R3 | **Previewed.** Before a run takes effect, the requester can see what it would change |
| R4 | **Approved by the release group.** A run takes effect only after a member of the release group other than the requester approves it. The requester cannot approve their own run, and nobody outside the release group can approve |
| R5 | **Done means observed.** The system reports a run as succeeded only after it has seen `CODEOWNERS` with the right content on the project's `main`. If the change does not reach `main` (for example, `main` is protected against it), the run must not be reported as succeeded |
| R6 | **One place for the org.** From `/tmp/actions-gitlab/r1/toolbox-1/org`, a person or an agent can list `set-codeowners` for all three projects and request a run on any of them, without going into each project |
| R7 | **Audited.** Each run's record shows who requested it, who approved it, its inputs, its outcome, and when each of those happened |
| R8 | **Usable by agents.** Every step (list, preview, request, approve, check the status, read the record) works non-interactively from a shell, with machine-readable output |
| R9 | **The group is GitLab's.** The release group is the GitLab group `acme-w11/release` and its direct members. Changing its membership in GitLab changes who may approve, for all three projects, with no change to your system |

Then, with what you built, request `set-codeowners` on each project as ann, have dave approve alpha's run and erin approve beta's and gamma's, and confirm that each run is reported as succeeded:

| Project | pattern | owners |
|---|---|---|
| alpha | `*` | `@acme/proxy-maintainers` |
| beta | `/docs/` | `@acme/docs @carol` |
| gamma | `/graphify/security.py` | `@acme/security` |

Finally, write `HANDOFF.md` at the root of the `org` project's `main`. For each of R1 to R9, say how your system meets it, with file paths. Give the exact shell commands someone else would use to start anything that must be running, list the actions, preview, request (as ann), approve (as dave or erin), check a run's status, and read its record. Someone will follow it to test your system, including with invalid inputs, self-approval, approvals by people outside the release group, a change to the release group's membership in GitLab, and a run on a project whose `main` is protected so that nobody may push or merge to it. Stop any long-running process you started before you finish.

## Your platform

Use **Toolbox**, with GitLab as its code host. Its binaries are in `/tmp/actions-gitlab/r1/toolbox-1/bin`, and the runner's jobs have the same binaries on their `PATH`. Its documentation is in `/tmp/actions-gitlab/r1/toolbox-1/toolbox-docs`: the agent skills in `skills/` (one per command), two example portals in `examples/`, and the manifest's JSON Schema, `manifest.schema.json`. `toolbox --help` and `toolbox <command> --help` also work. Write whatever glue Toolbox doesn't provide yourself, in any language. You may install packages.

Stop when the three runs have succeeded and `HANDOFF.md` is on the `org` project's `main`. Reply with a short summary.
