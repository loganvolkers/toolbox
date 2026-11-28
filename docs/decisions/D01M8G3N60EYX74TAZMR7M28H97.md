---
id: D01M8G3N60EYX74TAZMR7M28H97
decided: 2026-11-28T03:37:20Z
---

# Enforce module boundaries in the maintainer SDLC

Keep shipped Rust crates under `crates/`, first-party npm workspace members under `npm/`, and
Seikika for Seikika under `.seikika/`. Independently installable examples and experiments retain
their own manifests. Core product code consumes portal data through configuration or runtime
contracts and imports other npm packages through their public exports.

Block new cycles, forbidden dependencies, private imports, and known deployment link patterns
with the standalone architecture check. Use its per-module findings in the scorecard and campaign.
Use browser probes with alternate portal data and evidence-based assessments to detect hard-coding
that the static scan cannot identify.

See [Module coupling](../design/module-coupling.md) for the tool comparison and enforcement scope.
