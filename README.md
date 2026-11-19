# toolbox

An open-source, agent-first internal developer portal (IDP) framework: Rust core, configured entirely in
code, deployed by you. Catalog, scorecards, self-service actions, knowledge/context lake and search —
exposed identically over API, MCP, CLI and UI.

**Status:** research & planning.

- [docs/research.md](docs/research.md) — landscape (Backstage, Port, Cortex, DX, OpsLevel, Datadog, OSS alternatives) and validation
- [docs/plan.md](docs/plan.md) — architecture and delivery roadmap
- [docs/ocean.md](docs/ocean.md) — lessons from Port Ocean for the connector protocol
- [docs/authorization.md](docs/authorization.md) — relationship-based access control on the catalog graph (OpenFGA-compatible)
- [docs/red-team.md](docs/red-team.md) — devil's-advocate review: gaps a competitor would exploit, ranked by deal risk
- [docs/principal-review.md](docs/principal-review.md) — principal-engineer review: security, consistency, scope holes and proposed ADRs
- Detailed designs:
  - [design/meta-catalog.md](docs/design/meta-catalog.md): the portal catalogs, owns and scores itself
  - [design/incremental-computation.md](docs/design/incremental-computation.md): DBSP incremental circuit + DBOS-model durable workflows over one change log
  - [design/extensibility.md](docs/design/extensibility.md): code-first extensibility and the right language for each job
  - [design/dashboards.md](docs/design/dashboards.md): dashboards via agent artifacts + MCP; operator-only UI views; history
  - [design/sdlc.md](docs/design/sdlc.md): local → CI → PR → preview → prod, remote plan, GitHub Actions
  - [design/notifications.md](docs/design/notifications.md): ownership-routed notifications, GitHub App, Actions for service repos
  - [design/identity-resolution.md](docs/design/identity-resolution.md): matching one service across many tools
  - [design/action-entities.md](docs/design/action-entities.md): action-created entities, TTL, day-2 actions
  - [design/context-lake.md](docs/design/context-lake.md): context lakes, and how the catalog, knowledge and security graphs overlap
  - [design/source-of-truth.md](docs/design/source-of-truth.md): field-level source of truth, and when Toolbox owns the data
  - [design/verification.md](docs/design/verification.md): TDD, deterministic simulation, reference models, differential and configuration-space testing
