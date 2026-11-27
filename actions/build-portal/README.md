# Build Seikika portal

Build a static snapshot of a repository portal. Run [setup](../setup/README.md) first or put
`seikika` on `PATH`. The action runs on Linux GitHub Actions runners.

| Input | Default | Meaning |
|---|---|---|
| `working-directory` | `.` | Checkout to score. The portal path is relative to it. |
| `portal` | `.seikika` | Directory containing `seikika.yaml`. |
| `format` | `html` | `html` produces an offline `index.html`. `web` also emits `report.json` for a host such as Astro and `seikika-report.js`. |
| `skip` | Empty | Comma-separated connectors to skip. |

The action prepares any committed lockfile with `seikika install --frozen`. It writes the snapshot
to a fresh runner-local directory and returns its absolute `directory` and `index` paths.
`report-json` is available in `web` mode. A snapshot can show failing scorecards without a
baseline. A CLI or connector error fails the build.

Review the snapshot's catalog data and rule evidence before uploading or deploying it.
The workflow controls its audience and deployment permissions.
[Host the snapshot on GitHub Pages or in Astro](../../site/src/content/docs/operators/publish.md)
