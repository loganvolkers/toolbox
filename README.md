# toolbox

An open-source, agent-first internal developer portal (IDP) framework: Rust core, configured entirely in
code, deployed by you. Catalog, scorecards, self-service actions, knowledge/context lake and search —
exposed identically over API, MCP, CLI and UI.

The public docs are a Starlight site in [site/](site/src/content/docs/index.md), published to GitHub Pages (D91).

**Focus (D01M80Y0XEK069GD08VCCNF0WB8):** proving value from both ends: bottom-up at a single repository ([docs/design/repo-value.md](docs/design/repo-value.md)) and top-down at the central portal, then meeting in the middle. Federation is unpaused, and self-service actions are built with it (D01M86BY3YQX3SA1W99QXF1QA1H).

**Status:** v1 implementation in progress: M0–M4 are done (validate, derived state, Postgres store and REST reads, authentication and OpenFGA facets, MCP, CLI, minimal UI and connectors); M5 (actions) is built with federation, after an independent review of self-service coordination (D01M86BY3YQX3SA1W99QXF1QA1H, [docs/design/federated-actions.md](docs/design/federated-actions.md)); durable runs and the UI pages remain ([docs/v1-spec.md](docs/v1-spec.md) §3). Toolbox also runs on itself: [portal/](portal/README.md) scores this repo's crates, docs, CLI and MCP tools against the three SDLC layers (D65).

**DX Benchmarks ([docs/design/dx-benchmarks.md](docs/design/dx-benchmarks.md)):** Toolbox measures its own developer experience in public. Fresh coding agents do real lifecycle work on open-source repositories (OpenFGA, Codex, gh-aw, PostHog), once with Toolbox and once with the strongest alternative: hand-written glue, a checklist, an agent building from scratch, or OpenSSF Scorecard. The bar is set before each run, and the losses are published with the wins. Every benchmark so far has changed Toolbox. The catalog is [portal/benchmarks/](portal/benchmarks).

**SDLC has four layers** ([docs/sdlc.md](docs/sdlc.md)): the *maintainer* SDLC for building Toolbox, the *package author* SDLC for publishing packages and connectors others consume, the *operator* SDLC for building a portal, and the *user* SDLC for building apps on a portal.

**Install (v0.1.1):** the shell and PowerShell installers on the GitHub release ([Install a release](site/src/content/docs/start/install.md), [docs/RELEASING.md](docs/RELEASING.md)). An operator running a repo portal installs the CLI and the workspace connector. Writing a connector is the CLI plus the scaffold in [examples/connector-scaffold](examples/connector-scaffold/README.md). For a repo portal:

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/loganvolkers/toolbox/releases/download/v0.1.1/toolbox-cli-installer.sh | sh
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/loganvolkers/toolbox/releases/download/v0.1.1/toolbox-connector-workspace-installer.sh | sh
```

**Build from source** (changing Toolbox itself):

```bash
cargo build && target/debug/toolbox validate examples/reference-portal
```

**Start here:** [AGENTS.md](AGENTS.md) for working rules and parallel tracks, [docs/v1-spec.md](docs/v1-spec.md) for what v1 must do, [examples/reference-portal](examples/reference-portal/README.md) for the reference implementation anchor. Also: [docs/decisions/](docs/decisions/README.md) for every decision, [docs/questions.md](docs/questions.md) for open questions, [docs/glossary.md](docs/glossary.md) for vocabulary, and [docs/research-log.md](docs/research-log.md) for methods and hands-on verifications.

- [docs/research.md](docs/research.md) — landscape (Backstage, Port, Cortex, DX, OpsLevel, Datadog, OSS alternatives) and validation
- [docs/v1-spec.md](docs/v1-spec.md) — the v1 implementation contract (milestones, contracts, API, MCP, CLI, data model, properties)
- [examples/reference-portal/](examples/reference-portal/README.md) — reference implementation anchor: manifest, schemas, fixtures, oracle and authz tests
- [docs/plan.md](docs/plan.md) — architecture and delivery roadmap
- [docs/ocean.md](docs/ocean.md) — lessons from Port Ocean for the connector protocol
- [docs/authorization.md](docs/authorization.md) — relationship-based access control on the catalog graph (OpenFGA-compatible)
- [docs/red-team.md](docs/red-team.md) — devil's-advocate review: gaps a competitor would exploit, ranked by deal risk
- [docs/principal-review.md](docs/principal-review.md) — principal-engineer review: security, consistency, scope holes and proposed ADRs
- [docs/adversarial-critique.md](docs/adversarial-critique.md) — adversarial critique: should Toolbox be built at all? Hypotheses, alternatives, kill criteria
- [docs/graphify-red-team.md](docs/graphify-red-team.md) — red team of the Graphify verdict by a session without the designer's context: CAFE(S), adopters, doc claims, methodology holes; triaged in [docs/graphify-red-team-response.md](docs/graphify-red-team-response.md)
- [docs/technology-critique.md](docs/technology-critique.md) — technology-only critique: sprawl, replaceability by generic components, the Backstage compatibility claim, a minimal architecture
- [docs/design/dx-benchmarks.md](docs/design/dx-benchmarks.md) — DX Benchmarks: how we benchmark agents' developer experience with Toolbox against the alternatives on open source, the prior art, and the catalog
- [experiments/](experiments/) — small reproducible experiments cited by the docs, and the DX Benchmarks' harnesses
- Detailed designs:
  - [design/meta-catalog.md](docs/design/meta-catalog.md): the portal catalogs, owns and scores itself
  - [design/incremental-computation.md](docs/design/incremental-computation.md): DBSP incremental circuit + DBOS-model durable workflows over one change log
  - [design/extensibility.md](docs/design/extensibility.md): code-first extensibility and the right language for each job
  - [design/repo-value.md](docs/design/repo-value.md): **the current focus**, proving repo-level value: hypotheses, pilots, stop conditions
  - [design/repo-portals.md](docs/design/repo-portals.md): Toolbox without a server — `toolbox report`, evidence, ratchets, one-file HTML; adopt it with the [report action](actions/report/README.md) and the [starter](examples/repo-portal-starter/README.md)
  - [design/ratchets.md](docs/design/ratchets.md): what we learned from Betterer and other baselines (ESLint, PHPStan, basedpyright, OpenSSF Scorecard)
  - [design/scorecards-for-scorecards.md](docs/design/scorecards-for-scorecards.md): what makes a standard good, and the meta scorecards that score ours (and themselves)
  - [design/package-author-sdlc.md](docs/design/package-author-sdlc.md): the fourth SDLC layer, binary distribution (Toolbox, connectors, upstream tools) and decoupling connectors from core
  - [design/packages.md](docs/design/packages.md): sharing standards as packages, where they live (Q15), the T4T monolith, what we publish, and fleet rollout
  - [design/github-actions-standards.md](docs/design/github-actions-standards.md): CI security, reliability and supply-chain scorecards (zizmor, actionlint, run history)
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
  - [design/secret-vault.md](docs/design/secret-vault.md): connector `${secret:…}` backends (env, file, Vault, AWS/GCP/Azure)
  - [design/context-lake.md](docs/design/context-lake.md): context lakes, and how the catalog, knowledge and security graphs overlap
  - [design/knowledge-graphs.md](docs/design/knowledge-graphs.md): knowledge graphs as agent context: Graphify, its alternatives, the foundations, and why Toolbox doesn't build a code graph
  - [design/source-of-truth.md](docs/design/source-of-truth.md): field-level source of truth, and when Toolbox owns the data
  - [design/verification.md](docs/design/verification.md): TDD, deterministic simulation, reference models, differential and configuration-space testing
- Worked examples:
  - [examples/approved-github-apps.md](docs/examples/approved-github-apps.md): self-service installation of approved GitHub Apps, and how an operator has an agent build, test and ship it
  - [examples/secret-vault-gcp](examples/secret-vault-gcp/README.md): GCP Secret Manager holds GitHub App, Jira and GCP SA connector secrets
