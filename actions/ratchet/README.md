# Seikika ratchet

Enforce a committed scorecard baseline. Run [setup](../setup/README.md) first or supply
`seikika` on `PATH`. The action runs on Linux GitHub Actions runners.

| Input | Default | Meaning |
|---|---|---|
| `working-directory` | `.` | Checkout to score. Relative paths below resolve from here. |
| `portal` | `.seikika` | Directory containing `seikika.yaml`. |
| `baseline` | `<portal>/results.json` | Committed scorecard results. |
| `mode` | `ci` | `ci` also fails on a missing or stale baseline and on unexpected unknown results. `check` blocks regressions. |
| `skip` | Empty | Comma-separated connectors to skip. Use the same set when creating the baseline. |

The action prepares any committed lockfile with `seikika install --frozen`, writes a job summary
and annotations for regressions, and fails the step on a nonzero CLI exit. It reads the baseline.
Create or update it separately with `seikika report --baseline … --update`, then review and commit
the diff. [Set up a baseline](../../site/src/content/docs/operators/ci.md)

The `result` and `exit-code` outputs carry the scorecard verdict and CLI exit code. The `verdict`
and `report` outputs are absolute paths to the JSON verdict and Markdown report when the CLI
produces them. The files are available in the current job. Add an artifact upload step with an
`always()` condition and a check for a nonempty path to retain them even on failure.

[Verdict format](../report/README.md#other-ci-systems)
