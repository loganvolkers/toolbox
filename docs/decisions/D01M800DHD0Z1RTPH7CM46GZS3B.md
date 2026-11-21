---
id: D01M800DHD0Z1RTPH7CM46GZS3B
decided: 2026-11-21T21:30:00Z
---

# Toolbox owns the tool pin; any installer installs it

**Refines D90 (packages declare their tools; mise and aqua install them). Toolbox still downloads
no tool itself. Each tool has one pin, `version` in `tools:`, and an id per installer
(`install: {mise, aqua, pypi}`). Toolbox derives each installer's input from the pin
(`toolbox tools config --for …`) and runs the installer (`toolbox tools install --via …`): mise,
uv, pipx or pip, with aqua through its own config. mise is the recommendation, not a requirement.**

1. **Why not one installer:** a tool that a team already installs with aqua, uv or pipx must not
   need mise to enter the box. Installers are adapters; the pin is the contract.
2. **Why not Toolbox downloading:** mise and aqua verify checksums, GitHub artifact attestations,
   Cosign signatures and SLSA provenance, and keep registries current. A downloader in Toolbox would
   be a worse copy, and sprawl (D45). The research is in
   [tool-installation.md](../design/tool-installation.md) §3.
3. **This repository:** CI and the site-report script install zizmor and ruff from `tools:`.
   actionlint stays SHA-256-pinned in `scripts/ci-tools.sh`, and `toolbox doctor` fails CI if it
   drifts from the pin.

**Status:** ✅

**Where:** [docs/design/tool-installation.md](../design/tool-installation.md),
[crates/toolbox-cli/src/tools.rs](../../crates/toolbox-cli/src/tools.rs)
