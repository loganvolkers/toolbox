You are a platform engineer at Acme. Work only inside /tmp/actions-ab/r6/backstage-3: do not read or search files outside it, except the system's own tools and the paths "Your platform" names below.

## The org

- Three repositories: `alpha`, `beta` and `gamma`. Each has a bare origin at `/tmp/actions-ab/r6/backstage-3/origins/<name>.git`, which plays the code host (its `main` is where changes land), and a checkout at `/tmp/actions-ab/r6/backstage-3/<name>` that tracks it.
- `/tmp/actions-ab/r6/backstage-3/org` is the org-level place. Its origin is `/tmp/actions-ab/r6/backstage-3/origins/org.git`.
- People: **ann** and **bob** maintain all three repositories. **carol** is a developer, not a maintainer. There are no passwords in this sandbox, so saying who you are is enough to act as someone, but your system must record who did what and enforce the approval rules below.
- If you start a server, listen on port 7710 (or 7711 for a second one). Other workspaces share this machine: start the name of any tmux session or other named resource with `r6-backstage-3-`.

## The task

Build a self-service action, `mark-deprecated`, that each repository's maintainers own, and run it on all three repositories.

`mark-deprecated` marks one repository deprecated by adding `DEPRECATED.md` to its origin's `main`, with exactly this content:

```
# Deprecated: use <successor>

<reason>
```

Inputs:
- `successor`: another repository, `owner/name`, matching `^[a-z0-9-]+/[a-z0-9-]+$`;
- `reason`: 20 to 200 characters.

Requirements:

| # | Requirement |
|---|---|
| R1 | **Owned by the repository.** Each repository's definition of the action lives in that repository, committed to its origin's `main`, so the repository's team owns it and can change it |
| R2 | **Validated.** Invalid inputs are refused before anything changes |
| R3 | **Previewed.** Before a run takes effect, the requester can see what it would change |
| R4 | **Approved.** A run takes effect only after a maintainer other than the requester approves it. The requester cannot approve their own run, and carol cannot approve |
| R5 | **Done means observed.** The system reports a run as succeeded only after it has seen `DEPRECATED.md` with the right content on the origin's `main`. If the change does not reach `main` (for example, the push is rejected), the run must not be reported as succeeded |
| R6 | **One place for the org.** From `/tmp/actions-ab/r6/backstage-3/org`, a person or an agent can list `mark-deprecated` for all three repositories and request a run on any of them, without going into each repository |
| R7 | **Audited.** Each run's record shows who requested it, who approved it, its inputs, its outcome, and when each of those happened |
| R8 | **Usable by agents.** Every step (list, preview, request, approve, check the status, read the record) works non-interactively from a shell, with machine-readable output |

Then, with what you built, request `mark-deprecated` on each repository as ann, have bob approve each run, and confirm that each run is reported as succeeded:

| Repository | successor | reason |
|---|---|---|
| alpha | `acme/dependabot-proxy-v2` | Replaced by the v2 proxy, which supports OIDC registries. |
| beta | `acme/valtio-store` | State management moves to the shared store package. |
| gamma | `acme/graphify-core` | Graph extraction now lives in graphify-core. |

Finally, write `/tmp/actions-ab/r6/backstage-3/org/HANDOFF.md`. For each of R1 to R8, say how your system meets it, with file paths. Give the exact shell commands someone else would use to start anything that must be running, list the actions, preview, request (as ann), approve (as bob), check a run's status and read its record. Someone will follow it to test your system, including with invalid inputs, self-approval, carol's approval and a rejected push. Stop any long-running process you started before you finish.

## Your platform

Use **Backstage**. A Backstage 1.57 app, created with `@backstage/create-app` and with the scaffolder backend installed, is at `/tmp/actions-ab/r6/backstage-3/backstage`. Node 24 and yarn are in `/tmp/bs/node24/bin`; add that directory to `PATH`. Start the backend with `yarn workspace backend start` in `/tmp/actions-ab/r6/backstage-3/backstage`. It listens on http://localhost:7710 (set in `app-config.local.yaml`). The same file gives ann, bob and carol a static API token each (`ann-token-0123456789`, `bob-token-0123456789`, `carol-token-0123456789`); send `Authorization: Bearer <token>`. There is no GitHub here: the origins are local git repositories. You don't need the frontend; the API is enough. Write whatever glue Backstage doesn't provide yourself (custom scaffolder actions, backend modules, scripts), in any language. You may install packages.

Stop when the three runs have succeeded and `HANDOFF.md` is written. Reply with a short summary.
