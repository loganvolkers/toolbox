# Glossary

Shared vocabulary across the design documents. Keep entries short and link to the defining document.

| Term | Meaning | Defined in |
|---|---|---|
| **Portal** | One organisation's Toolbox deployment, defined by a portal repo | [plan.md](./plan.md) §1 |
| **Operator** | Someone (or an agent) who builds and runs a portal: platform engineers | [plan.md](./plan.md) §6 |
| **Maintainer** | Someone (or an agent) who builds Toolbox itself | [design/maintainer-sdlc.md](./design/maintainer-sdlc.md) |
| **User** | Someone (or an agent, bot or app) who builds on a running portal's context and actions | [design/user-sdlc.md](./design/user-sdlc.md) |
| **Report action** | `actions/report`: the GitHub Action that runs `toolbox report` with a ratchet in one workflow step (D69) | [actions/report/README.md](../actions/report/README.md) |
| **Meta scorecard** | A scorecard over Scorecard and Rule entities (`meta-rule`, `meta-scorecard`); it scores itself, so there is one meta level (D71) | [design/scorecards-for-scorecards.md](./design/scorecards-for-scorecards.md) |
| **New-entity floor** | `ratchet.newEntitiesAtLeast`: entities new since the baseline must reach this level, or the ratchet reports a regression (D71) | [design/scorecards-for-scorecards.md](./design/scorecards-for-scorecards.md) |
| **Unknown outcome** | A rule whose data is missing (e.g. a linter not installed): reported, never a regression or an improvement; `--ci` and `--update` refuse it (D70) | [design/github-actions-standards.md](./design/github-actions-standards.md) |
| **Repo portal** | A portal run by `toolbox report` in one repository, without a server (D66) | [design/repo-portals.md](./design/repo-portals.md) |
| **Ratchet / baseline** | A committed results file (`toolbox-report/v1`); a newly failing rule or growing evidence is a regression (D66, Betterer-style) | [design/ratchets.md](./design/ratchets.md) |
| **Evidence** | A rule's `evidence:` CEL value when it fails: the items that make it fail | [design/ratchets.md](./design/ratchets.md) |
| **Federation / standard package / stage** | Portals in a tree: standards down as versioned packages, results up as report files; a rule's stage is `experimental`, `advisory` or `mandatory` (D68, design only) | [design/federation.md](./design/federation.md) |
| **Toolbox for Toolbox** | The local portal in `portal/` that scores Toolbox's own crates, docs and interfaces per SDLC layer (D65) | [design/toolbox-for-toolbox.md](./design/toolbox-for-toolbox.md) |
| **SDLC layers** | Maintainer SDLC (building Toolbox), operator SDLC (building a portal), user SDLC (building apps on a portal); documents always say which (D63) | [sdlc.md](./sdlc.md) |
| **Portal program** | The TypeScript/Python/YAML code in a portal repo that declares kinds, connectors, scorecards, actions, views and policies | [plan.md](./plan.md) §2 |
| **Manifest** | The canonical, versioned JSON emitted by the portal program at build time; the real contract | [plan.md](./plan.md) §2 |
| **Kind** | An entity type (e.g. `Service`), declared in code; Backstage-compatible envelope | [plan.md](./plan.md) §4.1 |
| **Fragment** | A partial entity contributed by one source; fragments are merged by `sources` precedence | [plan.md](./plan.md) §4.1 |
| **Sources (per field)** | The code-declared, ordered list of who owns a field's value: `git:`, `connector:`, `native`, `computed`, `action` | [design/source-of-truth.md](./design/source-of-truth.md) |
| **Native data** | Field values whose source of truth is Toolbox itself, where the schema declares it | [design/source-of-truth.md](./design/source-of-truth.md) |
| **TCP (Toolbox Connector Protocol)** | The open HTTP+JSON (and stdio, push) protocol for connectors and executors; capabilities `source`, `act`, `resolve` | [plan.md](./plan.md) §4.6 |
| **Raw record** | An unmodified connector payload stored once as input; mapping runs in the engine | [design/context-lake.md](./design/context-lake.md) §5.1 |
| **Change log** | *(Deferred, D37.)* The commit-ordered log of input-table changes (Postgres logical decoding) | [design/incremental-computation.md](./design/incremental-computation.md) §3 |
| **Log position** | *(Deferred, D37; replaced by the snapshot id.)* A position in the change log | [design/incremental-computation.md](./design/incremental-computation.md) §3 |
| **Circuit** | *(Deferred, D37.)* The DBSP dataflow, built at runtime from the manifest, that incrementally maintains every derived value | [design/incremental-computation.md](./design/incremental-computation.md) |
| **Epoch** | *(Deferred, D37.)* A circuit version identified by `(engine version, manifest hash)`; built in the background and cut over atomically | [design/incremental-computation.md](./design/incremental-computation.md) §3 |
| **Durable workflow** | A side-effecting process checkpointed in Postgres (DBOS model): actions, syncs, notifications, timers | [design/incremental-computation.md](./design/incremental-computation.md) §4 |
| **Facet** | A named group of fields guarded by a permission (e.g. score summary vs details) | [authorization.md](./authorization.md) §4.2 |
| **Declassification** | An explicit, reviewed declaration that lowers a derived value's facet label | [authorization.md](./authorization.md) §4.2 |
| **Derived tuples** | OpenFGA relationship tuples computed from catalog data as a circuit view; never written separately | [authorization.md](./authorization.md) §4.1 |
| **OpenFGA in the container** | Unmodified OpenFGA shipped in the Toolbox image, reading derived tuples through a Postgres view; co-process or c-archive is Q11 (D38) | [authorization.md](./authorization.md) §4.3 |
| **Meta catalog** | Catalog entities describing the portal itself (kinds, connectors, scorecards, rules, actions, views…), with owners and default scorecards | [design/meta-catalog.md](./design/meta-catalog.md) |
| **Inventory tier** | Bounded, discovered entities (workloads, images, cloud identities) that bridge the catalog to security graphs | [design/context-lake.md](./design/context-lake.md) §5.2 |
| **Finding** | An imported security result (e.g. a Wiz toxic combination) attached to entities | [design/context-lake.md](./design/context-lake.md) §5.4 |
| **Context pack** | A token-budgeted bundle of context about an entity for agents | [plan.md](./plan.md) §4.4 |
| **Action scope** | `create` / `day2` / `delete` / `global`: how an action attaches to entities | [design/action-entities.md](./design/action-entities.md) §2 |
| **`enumFrom`** | An action input whose options are computed per entity by the circuit, with rich labels and disabled reasons | [design/action-entities.md](./design/action-entities.md) §5b, [design/frontend.md](./design/frontend.md) |
| **Action UI tiers** | Generated form → field widgets → custom React component | [design/frontend.md](./design/frontend.md) §3 |
| **Trusted documents** | Pre-registered GraphQL operations, the only ones executed in production | [design/graphql.md](./design/graphql.md) |
| **Remote plan** | Server-side evaluation of a candidate manifest against production state, posted to the PR | [design/sdlc.md](./design/sdlc.md) §4 |
| **Gates** | Code-defined thresholds that block a deploy unless acknowledged (deletion ratio, authz loss, …) | [design/sdlc.md](./design/sdlc.md) §6 |
| **`toolbox dev`** | The local daemon: watch → manifest diff → circuit reconfiguration → affected tests → UI hot reload | [design/developer-velocity.md](./design/developer-velocity.md) §4 |
| **`Env` seam** | The single crate through which all nondeterminism flows, enabling deterministic simulation | [design/verification.md](./design/verification.md) §3 |
| **Operator plugin / user plugin** | The two Agent Plugins packages: authoring (in this repo) and generated per portal | [design/agent-plugins.md](./design/agent-plugins.md) |
| **Agent skill** | A directory with `SKILL.md` (Agent Skills spec). This repo catalogues and scores the ones it ships (`maintainer-skill`, D72) | [design/skill-standards.md](./design/skill-standards.md) |
| **Connector (catalog)** | A first-party `toolbox-connector-*` crate. This repo catalogues and scores the ones it ships (`maintainer-connector`, D73). Distinct from a portal's configured connector instance | [design/connector-standards.md](./design/connector-standards.md) |
| **Library dependency** | A direct external crate a workspace member names. Catalogued and scored from the manifests and `Cargo.lock` (`maintainer-dependency`, D74). Dev-dependencies are not included | [design/dependency-standards.md](./design/dependency-standards.md) |
| **Agent instruction file** | `AGENTS.md`, or an alias such as `CLAUDE.md` or `GEMINI.md` that points at it. Catalogued and scored (`maintainer-agent-doc`, D75) | [design/agents-md-standards.md](./design/agents-md-standards.md) |
| **npm manifest** | A `package.json`. Catalogued and scored as an application (`maintainer-package`, D76): a lockfile is the pin, and a licence field is not required | [design/package-standards.md](./design/package-standards.md) |
| **Charter goals (G1–G3)** | Simple single-container deployment, agent SDLC (in three layers, D63), replaceable components | [decisions.md](./decisions.md) D33–D35 |
| **Snapshot / snapshot id** | A consistent (`REPEATABLE READ`) read of Postgres inputs from which all derived state is recomputed; its id is the consistency token | [decisions.md](./decisions.md) D37 |
| **Full recompute** | The v1 derived-state engine: recompute every derived value from a snapshot on change, behind the `DerivedState` contract | [technology-critique.md](./technology-critique.md) §7.1 |
| **Replaceable contract** | A Toolbox-owned trait, protocol or mount contract with conformance tests, behind which a default technology sits (`DerivedState`, `Authorizer`, `Durable`, UI extension contract, executor protocol) | [plan.md](./plan.md) §3 |
| **UI extension contract** | How operator UI bundles (any framework; React by default) are built into the image and mounted | [decisions.md](./decisions.md) D40 |
| **In-image executor** | Action code packaged into the Toolbox image and run as a child process over stdio | [decisions.md](./decisions.md) D40 |
| **Sprawl budget** | The cap on implementation languages, required services and author-facing languages in v1 | [decisions.md](./decisions.md) D45 |
| **Connector command** | A Toolbox → connector instruction (`refresh`, `resync`, `cancel`) with an id, priority, deadline and terminal status | [design/connector-control.md](./design/connector-control.md) |
| **Scoped refresh** | A re-read of specific records (by external ids, selector or `since`); never causes implicit deletes | [design/connector-control.md](./design/connector-control.md) |
| **Postcondition** | A CEL expression an action declares, which must hold after the run for it to succeed | [design/action-runs.md](./design/action-runs.md) |
| **Confirmation token** | Binds a run to the exact preview the requester saw; stale previews are rejected | [design/action-runs.md](./design/action-runs.md) |
| **`unverified`** | Run state when the executor reported success but Toolbox could not observe the effect; never shown as success | [design/action-runs.md](./design/action-runs.md) |
| **v1 spec** | The implementation contract: milestones, contracts, API, MCP, CLI, data model, properties | [v1-spec.md](./v1-spec.md) |
| **Reference portal** | The example portal (GitHub + IdP) whose outputs and scenarios define "done" for v1 | [examples/reference-portal](../examples/reference-portal/README.md) |
| **Reference model / oracle** | `tools/reference.py`: a small executable model of the pipeline that the Rust implementation must match | [v1-spec.md](./v1-spec.md) §6 |
| **Ownership status** | `verified`, `inferred`, `unverified` or `none`, from the CODEOWNERS authority rule | [v1-spec.md](./v1-spec.md) §6.2 |
| **Action context lookup** | Catalog entities an action declares (`{kind, where}`) and passes to its executor as `related` | [decisions.md](./decisions.md) D52 |
| **Canonical manifest** | The single key-sorted `manifest.json` that `toolbox build` produces from any source; what the server loads and `plan` diffs | [decisions.md](./decisions.md) D53 |
| **`include`** | Source-only list of YAML files merged into the canonical manifest | [decisions.md](./decisions.md) D54 |
| **View endpoint** | A composite, facet-shaped REST response serving one UI page in one request | [design/minimal-ui.md](./design/minimal-ui.md) §6 |
| **Form-renderable subset** | The JSON Schema keywords allowed in action inputs so a form can always be generated | [design/minimal-ui.md](./design/minimal-ui.md) §4 |
