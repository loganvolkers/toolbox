# Set up Seikika

Put the CLI and the workspace and runner connectors on `PATH` for later steps in this job.
Check out the consumer repository separately. Pass exactly one input.

| Input | Meaning |
|---|---|
| `seikika-version` | Published CLI version, such as `0.2.1`. Downloads the three release archives and checks their SHA-256 digests. |
| `seikika-dir` | Directory holding an existing executable named `seikika` and its connectors. Relative to the job's checkout. |

The `bin-dir` output is the absolute binary directory. The action runs on Linux GitHub Actions runners.
Install other connector binaries and tools separately when your manifest requires them. If your
portal has a lockfile, run `seikika install .seikika --frozen` before standalone CLI commands
that need its packages.

[Choose the steps for your CI job](../../npm/site/src/content/docs/operators/ci.md)
