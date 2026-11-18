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
- Detailed designs:
  - [design/meta-catalog.md](docs/design/meta-catalog.md): the portal catalogs, owns and scores itself
  - [design/canvases.md](docs/design/canvases.md): dashboards as agent-written canvases
  - [design/sdlc.md](docs/design/sdlc.md): local → CI → PR → preview → prod, remote plan, GitHub Actions
  - [design/notifications.md](docs/design/notifications.md): ownership-routed notifications, GitHub App, Actions for service repos
  - [design/identity-resolution.md](docs/design/identity-resolution.md): matching one service across many tools
  - [design/action-entities.md](docs/design/action-entities.md): action-created entities, TTL, day-2 actions
