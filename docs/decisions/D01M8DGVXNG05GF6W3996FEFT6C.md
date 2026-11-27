---
id: D01M8DGVXNG05GF6W3996FEFT6C
decided: 2026-11-27
---

# Compose consumer GitHub Actions by capability

Ship `actions/setup` to register a released or prebuilt CLI, `actions/ratchet` to enforce a
committed scorecard baseline, and `actions/build-portal` to generate static site files. Each
is a step operators can put in their own workflow. Use the CLI directly after setup for
validation, fixture tests, and other operator commands.

The ratchet reads the baseline and emits the CLI verdict. Baseline updates are a separate
decision. The build emits files suitable for an artifact or static host. The consumer's
workflow grants upload and deployment permissions where it publishes the files.

Keep `actions/report` as the one-step integration introduced by
[D69](D01M7VXK0480000000000000000.md). All action paths ship together on the Seikika release tag.
The action revision and the version of the CLI that it installs are separate pins.

[Comparison and gap analysis](../audits/consumer-github-actions.md) ·
[Repo portals](../design/repo-portals.md)
