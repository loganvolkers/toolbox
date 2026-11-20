# Glossary

Shared vocabulary across the design documents. Keep entries short and link to the defining document.

| Term | Meaning | Defined in |
|---|---|---|
| **Portal** | One organisation's Toolbox deployment, defined by a portal repo | [plan.md](./plan.md) §1 |
| **Operator** | Someone (or an agent) who builds and runs a portal: platform engineers | [plan.md](./plan.md) §6 |
| **Maintainer** | Someone who builds Toolbox itself | [design/developer-velocity.md](./design/developer-velocity.md) |
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
| **Charter goals (G1–G3)** | Simple single-container deployment, agent SDLC, replaceable components | [decisions.md](./decisions.md) D33–D35 |
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
