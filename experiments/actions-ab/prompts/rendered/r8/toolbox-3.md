You are a platform engineer at Acme. Work only inside /tmp/actions-ab/r8/toolbox-3: do not read or search files outside it, except the system's own tools and the paths "Your platform" names below.

## The org

- Three repositories: `alpha`, `beta` and `gamma`. Each has a bare origin at `/tmp/actions-ab/r8/toolbox-3/origins/<name>.git`, which plays the code host (its `main` is where changes land), and a checkout at `/tmp/actions-ab/r8/toolbox-3/<name>` that tracks it.
- `/tmp/actions-ab/r8/toolbox-3/org` is the org-level place. Its origin is `/tmp/actions-ab/r8/toolbox-3/origins/org.git`.
- People: **ann** and **bob** maintain all three repositories. **carol** is a developer. **dave** and **erin** are the **release** group, which signs off on what readers of the repositories are told. Anyone in the org may request a run. There are no passwords in this sandbox, so saying who you are is enough to act as someone, but your system must record who did what and enforce the approval rules below.
- If you start a server, listen on port 7190 (or 7191 for a second one). Other workspaces share this machine: start the name of any tmux session or other named resource with `r8-toolbox-3-`.

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
| R6 | **One place for the org.** From `/tmp/actions-ab/r8/toolbox-3/org`, a person or an agent can list `add-notice` for all three repositories and request a run on any of them, without going into each repository |
| R7 | **Audited.** Each run's record shows who requested it, who approved it, its inputs, its outcome, and when each of those happened |
| R8 | **Usable by agents.** Every step (list, preview, request, approve, check the status, read the record) works non-interactively from a shell, with machine-readable output |
| R9 | **The group is defined once.** Who is in the release group is defined in one place at the org level, not in each repository. Changing it there changes who may approve, for all three repositories |

Then, with what you built, request `add-notice` on each repository as ann, have dave approve alpha's run and erin approve beta's and gamma's, and confirm that each run is reported as succeeded:

| Repository | level | message |
|---|---|---|
| alpha | `warning` | This proxy is frozen while the v2 rewrite lands; please file new issues against v2. |
| beta | `info` | Releases now ship on the shared release train every second Tuesday. |
| gamma | `info` | Graph extraction now lives in graphify-core; this package re-exports it. |

Finally, write `/tmp/actions-ab/r8/toolbox-3/org/HANDOFF.md`. For each of R1 to R9, say how your system meets it, with file paths. Give the exact shell commands someone else would use to start anything that must be running, list the actions, preview, request (as ann), approve (as dave or erin), check a run's status, read its record, and change who is in the release group. Someone will follow it to test your system, including with invalid inputs, self-approval, approvals by people outside the release group, a change to the group, and a rejected push of a change that is not yet on `main`. Stop any long-running process you started before you finish.

## Your platform

Use **Toolbox**. Its binaries are in `/tmp/actions-ab/r8/toolbox-3/bin`; add that directory to `PATH`. Its documentation is in `/tmp/actions-ab/r8/toolbox-3/toolbox-docs`: the agent skills in `skills/` (one per command), two example portals in `examples/`, and the manifest's JSON Schema, `manifest.schema.json`. `toolbox --help` and `toolbox <command> --help` also work. Write whatever glue Toolbox doesn't provide yourself, in any language. You may install packages.

Stop when the three runs have succeeded and `HANDOFF.md` is written. Reply with a short summary.
