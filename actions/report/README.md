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
         - uses: loganvolkers/toolbox/actions/report@v0.1.0
           with:
             toolbox-version: "0.1.0"   # download attested binaries (D90)
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
- `result`: `pass`, `regressed`, `stale`, `missing-baseline`, `unknown` or `error`;
- `exit-code`;
- `report`;
- `html`;
- `baseline`;
- `verdict`: the verdict JSON (below), also uploaded in the artifact.

Each regression is also an `::error` annotation on the run and the pull request, naming the
scorecard, the entity and what got worse.

## Other CI systems

The action is a thin wrapper. Any pipeline gets the same gate from the CLI and its verdict file
(D96):

```bash
toolbox report .toolbox --baseline .toolbox/results.json --ci --verdict verdict.json
# exit 0: pass. 1: the ratchet failed (see .result). 2: toolbox couldn't run (see .message)
jq -r '.regressions[] | "\(.scorecard) · \(.entity): \(.message)"' verdict.json
```

`verdict.json` (`toolbox-verdict/v1`) holds:

| Field | Meaning |
|---|---|
| `result` | `pass`, `regressed`, `stale` (better, but the file doesn't record it), `missing-baseline`, `unknown` (a rule couldn't be evaluated, under `--ci` or `--update`) or `error` |
| `exitCode` | The process's exit code: 0, 1 or 2 |
| `mode` | `ci`, `check` (`--baseline` alone), `update` or `report` (no baseline) |
| `baseline`, `written` | The results file, and whether this run wrote it |
| `regressions`, `improvements`, `changes` | Each `{scorecard, entity, message}`; `changes` are neither, such as a new entity at or above the floor |
| `unknownRules`, `message` | How many rule outcomes are unknown; the error, when `result` is `error` |

`stale` exits 1 only under `--ci`. A verdict is written on every run, including errors, so a
pipeline step can always read it. To post the regressions in a merge request, turn them into your
system's format (for example a GitLab Code Quality report, or a comment).

## Notes

- Prefer `toolbox-version: "0.1.0"` (or later) so the action downloads attested binaries (D90).
  When that input is empty it still builds from the action's source (Linux runners; cold start is
  slow). If the Toolbox repository is private, the job token needs `contents: read` on it.
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
  source and scores a pilot set up from the starter: it creates the baseline, passes, then adds an
  individual to CODEOWNERS and checks that the job fails with that regression in the verdict.
