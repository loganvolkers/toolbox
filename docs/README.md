# Seikika design and research

Use this index to find the implementation contract, design decisions, and research behind Seikika.
Start with the [architecture and roadmap](plan.md), the [decision log](decisions/README.md),
or the [open questions](questions.md). For build and review instructions, see [AGENTS.md](../AGENTS.md).

- [docs/research.md](research.md) — landscape (Backstage, Port, Cortex, DX, OpsLevel, Datadog, OSS alternatives) and validation
- [Connector directory research](audits/connector-directory.md) compares how five products help operators discover, configure, and verify connectors.
- [Standards package directory research](audits/package-directory.md) compares package discovery, installation, and verification across five ecosystems.
- [docs/v1-spec.md](v1-spec.md) — the v1 implementation contract (milestones, contracts, API, MCP, CLI, data model, properties)
- [examples/reference-portal/](../examples/reference-portal/README.md) — reference implementation anchor: manifest, schemas, fixtures, oracle and authz tests
- [docs/plan.md](plan.md) — architecture and delivery roadmap
- [docs/ocean.md](ocean.md) — lessons from Port Ocean for the connector protocol
- [docs/authorization.md](authorization.md) — relationship-based access control on the catalog graph (OpenFGA-compatible)
- [docs/red-team.md](red-team.md) — devil's-advocate review: gaps a competitor would exploit, ranked by deal risk
- [docs/principal-review.md](principal-review.md) — principal-engineer review: security, consistency, scope holes and proposed ADRs
- [docs/adversarial-critique.md](adversarial-critique.md) — adversarial critique: should Seikika be built at all? Hypotheses, alternatives, kill criteria
- [docs/graphify-red-team.md](graphify-red-team.md) — red team of the Graphify verdict by a session without the designer's context: CAFE(S), adopters, doc claims, methodology holes; triaged in [docs/graphify-red-team-response.md](graphify-red-team-response.md)
- [docs/technology-critique.md](technology-critique.md) — technology-only critique: sprawl, replaceability by generic components, the Backstage compatibility claim, a minimal architecture
- [docs/design/dx-benchmarks.md](design/dx-benchmarks.md) — DX Benchmarks: how we benchmark agents' developer experience with Seikika against the alternatives on open source, the prior art, and the catalog
- [experiments/](../experiments/) — small reproducible experiments cited by the docs, and the DX Benchmarks' harnesses
- Detailed designs:
  - [design/meta-catalog.md](design/meta-catalog.md): the portal catalogs, owns and scores itself
  - [design/incremental-computation.md](design/incremental-computation.md): DBSP incremental circuit + DBOS-model durable workflows over one change log
  - [design/extensibility.md](design/extensibility.md): code-first extensibility and the right language for each job
  - [design/repo-value.md](design/repo-value.md): **the current focus**, proving repo-level value: hypotheses, pilots, stop conditions
  - [design/repo-portals.md](design/repo-portals.md): Seikika without a server — `seikika report`, evidence, ratchets, one-file HTML; adopt it with the [report action](../actions/report/README.md) and the [starter](../examples/repo-portal-starter/README.md)
  - [design/ratchets.md](design/ratchets.md): what we learned from Betterer and other baselines (ESLint, PHPStan, basedpyright, OpenSSF Scorecard)
  - [design/scorecards-for-scorecards.md](design/scorecards-for-scorecards.md): what makes a standard good, and the meta scorecards that score ours (and themselves)
  - [design/package-author-sdlc.md](design/package-author-sdlc.md): the fourth SDLC layer, binary distribution (Seikika, connectors, upstream tools) and decoupling connectors from core
  - [design/packages.md](design/packages.md): sharing standards as packages, where they live (Q15), the S4S monolith, what we publish, and fleet rollout
  - [design/github-actions-standards.md](design/github-actions-standards.md): CI security, reliability and supply-chain scorecards (zizmor, actionlint, run history)
  - [design/federation.md](design/federation.md): standards that graduate from one repo to the company (design only)
  - [design/seikika-for-seikika.md](design/seikika-for-seikika.md): Seikika scoring its own repository
  - [design/maintainer-sdlc.md](design/maintainer-sdlc.md): **SDLC layer 1**, how Seikika itself is built and verified (check pyramid, measured loop)
  - [design/user-sdlc.md](design/user-sdlc.md): **SDLC layer 4**, building apps and agents on a portal's context and actions
  - [design/developer-velocity.md](design/developer-velocity.md): fast inner loops for maintainers and operators (SDLC layers 1–2: build speed, selective/parallel/cached tests, watch mode)
  - [design/frontend.md](design/frontend.md): delightful code-defined action UIs, Astro + Vue on `@seikika/ui`, with page stories
  - [design/graphql.md](design/graphql.md): read-only GraphQL with trusted documents for rich catalog pages
  - [design/agent-plugins.md](design/agent-plugins.md): the operator agent plugin (in `plugins/`) and the generated user plugin
  - [design/dashboards.md](design/dashboards.md): dashboards via agent artifacts + MCP; operator-only UI views; history
  - [design/sdlc.md](design/sdlc.md): **SDLC layer 3 (operator)**, local → CI → PR → preview → prod, remote plan, GitHub Actions
  - [design/notifications.md](design/notifications.md): ownership-routed notifications, GitHub App, Actions for service repos
  - [design/identity-resolution.md](design/identity-resolution.md): matching one service across many tools
  - [design/action-entities.md](design/action-entities.md): action-created entities, TTL, day-2 actions
  - [design/typescript-builder.md](design/typescript-builder.md): the typed TS builder that prints the canonical manifest
  - [design/minimal-ui.md](design/minimal-ui.md): the built-in pages, view endpoints and generated forms
  - [design/action-runs.md](design/action-runs.md): preview, confirm and verified completion for self-service actions
  - [design/connector-control.md](design/connector-control.md): commands to connectors (scoped refresh, resync) and read-your-writes
  - [design/secret-vault.md](design/secret-vault.md): connector `${secret:…}` backends (env, file, Vault, AWS/GCP/Azure)
  - [design/context-lake.md](design/context-lake.md): context lakes, and how the catalog, knowledge and security graphs overlap
  - [design/knowledge-graphs.md](design/knowledge-graphs.md): knowledge graphs as agent context: Graphify, its alternatives, the foundations, and why Seikika doesn't build a code graph
  - [design/source-of-truth.md](design/source-of-truth.md): field-level source of truth, and when Seikika owns the data
  - [design/verification.md](design/verification.md): TDD, deterministic simulation, reference models, differential and configuration-space testing
- Worked examples:
  - [examples/approved-github-apps.md](examples/approved-github-apps.md): self-service installation of approved GitHub Apps, and how an operator has an agent build, test and ship it
  - [examples/secret-vault-gcp](../examples/secret-vault-gcp/README.md): GCP Secret Manager holds GitHub App, Jira and GCP SA connector secrets
