---
id: D01M833YAH8MT4VY2W4K1PYAEHB
decided: 2026-11-23T02:30:00Z
---

# DX Benchmark the cloud/SaaS connector authoring SDLC against Port Ocean and Backstage

**A DX Benchmark (D01M82X2K0DYWTCA23MQJ4CKR0Z) measures the package-author connector
lifecycle: author → build-connector → test → change → reuse → read. Fresh agents author a
cloud/SaaS connector against a mock Jira-shaped issue-tracker API ("Forge"), once with Seikika's
connector protocol, once with Port Ocean, and once with a Backstage entity provider. The bar is
set before the run in [experiments/connector-ab/README.md](../../experiments/connector-ab/README.md).
The rivals are the strongest alternatives a team would use for SaaS catalog connectors, not local
tool fittings (L4).**

1. **Why this question:** earlier DX Benchmarks measured standards, the box and security scorecards
   on repository tools (ESLint-shaped work). No benchmark has measured *cloud connector authoring*
   on its own ([dx-benchmarks.md](../design/dx-benchmarks.md) §7 B4). Package-author
   `build-connector` is still partial ([package-author-sdlc.md](../design/package-author-sdlc.md)).
2. **Why Forge, not live Jira/GitHub/GCP:** a local mock with two tenants gives an independent
   truth, equal network conditions, and no credential wall. The API shape (Bearer auth, paginated
   projects and issues, then components) matches the SaaS connectors Ocean and Backstage already
   ship for Jira-like systems.
3. **Hypotheses C1–C3** live in the experiment README and are not edited after the run starts,
   except for the status line.
4. **Catalog:** [portal/benchmarks/connector-ab.yaml](../../portal/benchmarks/connector-ab.yaml).

**Status:** ✅

**Where:** [experiments/connector-ab/README.md](../../experiments/connector-ab/README.md),
[docs/design/dx-benchmarks.md](../design/dx-benchmarks.md),
[docs/design/package-author-sdlc.md](../design/package-author-sdlc.md)
