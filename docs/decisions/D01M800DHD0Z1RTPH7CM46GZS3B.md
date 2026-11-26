---
id: D01M800DHD0Z1RTPH7CM46GZS3B
decided: 2026-11-21T21:30:00Z
---

# Seikika owns the tool pin; any installer installs it

**Refines D90 (packages declare their tools; mise and aqua install them). Seikika still downloads
no tool itself. Each tool has one pin, `version` in `tools:`, and an id per installer
(`install: {mise, aqua, pypi}`). Seikika derives each installer's input from the pin
(`seikika tools config --for …`) and runs the installer (`seikika tools install --via …`): mise,
uv, pipx or pip, with aqua through its own config. mise is the recommendation, not a requirement.**

1. **Why not one installer:** a tool that a team already installs with aqua, uv or pipx must not
   need mise to enter the box. Installers are adapters; the pin is the contract.
2. **Why not Seikika downloading:** mise and aqua verify checksums, GitHub artifact attestations,
   Cosign signatures and SLSA provenance, and keep registries current. A downloader in Seikika would
   be a worse copy, and sprawl (D45). The research is in
   [tool-installation.md](../design/tool-installation.md) §3.
3. **This repository:** CI and the site-report script install zizmor and ruff from `tools:`.
   actionlint stays SHA-256-pinned in `scripts/ci-tools.sh`, and `seikika doctor` fails CI if it
   drifts from the pin.

**Status:** ✅

**Where:** [docs/design/tool-installation.md](../design/tool-installation.md),
[crates/seikika-cli/src/tools.rs](../../crates/seikika-cli/src/tools.rs)
