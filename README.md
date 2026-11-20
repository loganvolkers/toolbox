# toolbox

An open-source, agent-first internal developer portal (IDP) framework: Rust core, configured entirely in
code, deployed by you. Catalog, scorecards, self-service actions, knowledge/context lake and search —
exposed identically over API, MCP, CLI and UI.

**Status:** v1 implementation in progress: M0–M4 are done (validate, derived state, Postgres store and REST reads, authentication and OpenFGA facets, MCP, CLI, minimal UI and connectors); M5 (actions) is paused for a review of self-service coordination (D64) ([docs/v1-spec.md](docs/v1-spec.md) §3). Toolbox also runs on itself: [portal/](portal/README.md) scores this repo's crates, docs, CLI and MCP tools against the three SDLC layers (D65).

**SDLC has three layers** ([docs/sdlc.md](docs/sdlc.md)): the *maintainer* SDLC for building Toolbox, the *operator* SDLC for building a portal, and the *user* SDLC for building apps on a portal.

```bash
cargo build && target/debug/toolbox validate examples/reference-portal
```

**Start here:** [AGENTS.md](AGENTS.md) for working rules and parallel tracks, [docs/v1-spec.md](docs/v1-spec.md) for what v1 must do, [examples/reference-portal](examples/reference-portal/README.md) for the reference implementation anchor. Also: [docs/decisions.md](docs/decisions.md) for every decision and open question, [docs/glossary.md](docs/glossary.md) for vocabulary, and [docs/research-log.md](docs/research-log.md) for methods and hands-on verifications.

- [docs/research.md](docs/research.md) — landscape (Backstage, Port, Cortex, DX, OpsLevel, Datadog, OSS alternatives) and validation
- [docs/v1-spec.md](docs/v1-spec.md) — the v1 implementation contract (milestones, contracts, API, MCP, CLI, data model, properties)
- [examples/reference-portal/](examples/reference-portal/README.md) — reference implementation anchor: manifest, schemas, fixtures, oracle and authz tests
- [docs/plan.md](docs/plan.md) — architecture and delivery roadmap
- [docs/ocean.md](docs/ocean.md) — lessons from Port Ocean for the connector protocol
- [docs/authorization.md](docs/authorization.md) — relationship-based access control on the catalog graph (OpenFGA-compatible)
- [docs/red-team.md](docs/red-team.md) — devil's-advocate review: gaps a competitor would exploit, ranked by deal risk
- [docs/principal-review.md](docs/principal-review.md) — principal-engineer review: security, consistency, scope holes and proposed ADRs
- [docs/adversarial-critique.md](docs/adversarial-critique.md) — adversarial critique: should Toolbox be built at all? Hypotheses, alternatives, kill criteria
- [docs/technology-critique.md](docs/technology-critique.md) — technology-only critique: sprawl, replaceability by generic components, the Backstage compatibility claim, a minimal architecture
- [experiments/](experiments/) — small reproducible experiments cited by the docs
- Detailed designs:
  - [design/meta-catalog.md](docs/design/meta-catalog.md): the portal catalogs, owns and scores itself
  - [design/incremental-computation.md](docs/design/incremental-computation.md): DBSP incremental circuit + DBOS-model durable workflows over one change log
  - [design/extensibility.md](docs/design/extensibility.md): code-first extensibility and the right language for each job
  - [design/repo-portals.md](docs/design/repo-portals.md): Toolbox without a server — `toolbox report`, evidence, ratchets, one-file HTML
  - [design/ratchets.md](docs/design/ratchets.md): what we learned from Betterer and other baselines (ESLint, PHPStan, basedpyright, OpenSSF Scorecard)
  - [design/federation.md](docs/design/federation.md): standards that graduate from one repo to the company (design only)
  - [design/toolbox-for-toolbox.md](docs/design/toolbox-for-toolbox.md): Toolbox scoring its own repository
  - [design/maintainer-sdlc.md](docs/design/maintainer-sdlc.md): **SDLC layer 1**, how Toolbox itself is built and verified (check pyramid, measured loop)
  - [design/user-sdlc.md](docs/design/user-sdlc.md): **SDLC layer 3**, building apps and agents on a portal's context and actions
  - [design/developer-velocity.md](docs/design/developer-velocity.md): fast inner loops for maintainers and operators (SDLC layers 1–2: build speed, selective/parallel/cached tests, watch mode)
  - [design/frontend.md](docs/design/frontend.md): delightful code-defined action UIs, React + `@toolbox/ui`, Storybook page-level stories
  - [design/graphql.md](docs/design/graphql.md): read-only GraphQL with trusted documents for rich catalog pages
  - [design/agent-plugins.md](docs/design/agent-plugins.md): the operator agent plugin (in `plugins/`) and the generated user plugin
  - [design/dashboards.md](docs/design/dashboards.md): dashboards via agent artifacts + MCP; operator-only UI views; history
  - [design/sdlc.md](docs/design/sdlc.md): **SDLC layer 2 (operator)**, local → CI → PR → preview → prod, remote plan, GitHub Actions
  - [design/notifications.md](docs/design/notifications.md): ownership-routed notifications, GitHub App, Actions for service repos
  - [design/identity-resolution.md](docs/design/identity-resolution.md): matching one service across many tools
  - [design/action-entities.md](docs/design/action-entities.md): action-created entities, TTL, day-2 actions
  - [design/typescript-builder.md](docs/design/typescript-builder.md): the typed TS builder that prints the canonical manifest
  - [design/minimal-ui.md](docs/design/minimal-ui.md): the built-in pages, view endpoints and generated forms
  - [design/action-runs.md](docs/design/action-runs.md): preview, confirm and verified completion for self-service actions
  - [design/connector-control.md](docs/design/connector-control.md): commands to connectors (scoped refresh, resync) and read-your-writes
  - [design/context-lake.md](docs/design/context-lake.md): context lakes, and how the catalog, knowledge and security graphs overlap
  - [design/source-of-truth.md](docs/design/source-of-truth.md): field-level source of truth, and when Toolbox owns the data
  - [design/verification.md](docs/design/verification.md): TDD, deterministic simulation, reference models, differential and configuration-space testing
- Worked examples:
  - [examples/approved-github-apps.md](docs/examples/approved-github-apps.md): self-service installation of approved GitHub Apps, and how an operator has an agent build, test and ship it
