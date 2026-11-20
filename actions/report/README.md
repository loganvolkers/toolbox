# Toolbox report action

Score a repository against its Toolbox standards on every push and pull request, and fail when
anything gets worse than the committed baseline: a ratchet, in the manner of Betterer
([docs/design/ratchets.md](../../docs/design/ratchets.md)). It needs no server, no database and no
secrets ([docs/design/repo-portals.md](../../docs/design/repo-portals.md)).

## Adopt it in a repository

1. Copy [`examples/repo-portal-starter/toolbox.yaml`](../../examples/repo-portal-starter/toolbox.yaml)
   to `.toolbox/toolbox.yaml`. It scores README, CODEOWNERS (a catch-all rule, teams not people) and
   LICENSE, and checks the docs' relative links. Edit or add scorecards as the team likes.
2. Add the workflow:

   ```yaml
   # .github/workflows/toolbox.yml
   name: toolbox
   on:
     push: {branches: [main]}
     pull_request:
     workflow_dispatch:
       inputs:
         mode: {description: "ci, check, update or report", default: update}
   permissions:
     contents: read
   jobs:
     report:
       runs-on: ubuntu-latest
       steps:
         - uses: actions/checkout@v4
         - uses: loganvolkers/toolbox/actions/report@<commit sha>   # pin a commit
           with:
             mode: ${{ inputs.mode || 'ci' }}
   ```

3. **First run:** start the workflow by hand with `mode: update`. It uploads `results.json` in the
   `toolbox-report` artifact; commit it as `.toolbox/results.json`. Or, with Toolbox built locally,
   run `toolbox report .toolbox --baseline .toolbox/results.json --update`.

From then on:
- every pull request fails if a rule starts failing, or a failing rule's evidence grows (e.g. a new
  individual in CODEOWNERS);
- it also fails if an improvement isn't recorded in `results.json`, so the committed file always
  matches the code.

The job summary says which, and how to update the baseline.

## Inputs

| Input | Default | Meaning |
|---|---|---|
| `working-directory` | `.` | The checkout to score; `portal` and `baseline` are relative to it |
| `portal` | `.toolbox` | Directory with `toolbox.yaml` |
| `baseline` | `<portal>/results.json` | The ratchet's results file |
| `mode` | `ci` | `ci`: fail on regressions and on a stale or missing baseline. `check`: fail on regressions only. `update`: write improvements (or create the file) and upload it. `report`: no baseline |
| `skip` | | Connectors not to run, comma-separated (e.g. a slow test runner) |
| `html` | `true` | Also produce the one-page HTML report |
| `artifact-name` | `toolbox-report` | Name of the uploaded artifact (Markdown and HTML reports; the baseline in `update` mode) |
| `toolbox-dir` | | A directory with prebuilt `toolbox` and `toolbox-connector-*` binaries. When empty, the action builds them from its own source at the pinned commit and caches them, so the first run per commit compiles and later runs restore |
| `toolbox-version` | — | A released Toolbox version (e.g. `0.1.0`): download its binaries, SHA-256-verified, instead of building from source (D90) |

**Outputs:**
- `result`: `pass`, `regressed`, `stale`, `missing-baseline` or `error`;
- `exit-code`;
- `report`;
- `html`;
- `baseline`.

## Notes

- **Linux runners only** for now; it builds with the toolchain pinned in the toolbox repository.
  Prebuilt release binaries will replace the build once Toolbox publishes releases (that waits on the
  licence decision, Q1).
- **Security:**
  - Connectors run on the runner with the job's permissions. The `workspace` connector only reads
    files.
  - A `runner` connector executes the commands in `toolbox.yaml`, so treat the manifest like any
    other CI script.
  - Use `pull_request`, not `pull_request_target`, so code from forks never runs with write tokens.
- **Monorepos:** run several steps with different `portal` directories, or one portal whose
  connectors cover the whole tree.
- **This repository** uses the action on itself (`.github/workflows/ci.yml`): the
  Toolbox-for-Toolbox ratchet with prebuilt binaries, and a `report-action` job that builds from
  source and scores a pilot set up from the starter.
