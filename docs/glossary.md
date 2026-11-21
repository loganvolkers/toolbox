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
| **Package author SDLC** | The fourth SDLC layer (D87): publishing packages, connectors and standards for others. Forks from the maintainer SDLC at *test* (released Toolbox versions, then distribute) | [design/package-author-sdlc.md](./design/package-author-sdlc.md) |
| **SDLC stage** | One step of an SDLC layer, declared in `portal/sdlc/` with an exit criterion, commands, checks, status and gaps, and scored by `sdlc-stage` (D87) | [sdlc.md](./sdlc.md) |
| **Rubric** | Questions a coding agent answers about entities of one kind (criteria with guidance), declared in the manifest; Toolbox calls no model (D79) | [design/scorecards-for-scorecards.md](./design/scorecards-for-scorecards.md) §6 |
| **Assessment** | One agent's or person's verdicts on one entity against a rubric, stored as a file in `assessments/` or pushed with `toolbox assess submit`; stale when the entity's hash changes. Assessments are scored (`meta-assessment`) and reviewed once (`assessment-quality`) (D79, D80) | [design/scorecards-for-scorecards.md](./design/scorecards-for-scorecards.md) §6 |
| **CLI skill** | The agent skill whose text names one `toolbox` command's invocation. `cli-skill` fails a command that has none (D94) | [design/cli-skills.md](./design/cli-skills.md) |
| **YAML format** | The fixed point of Yamark 0.6.1 with `compact`: a collection that fits in 80 columns is flow style (D97) | [design/yaml-format.md](./design/yaml-format.md) |
| **Decision id** | `D` plus a ULID. The file `docs/decisions/<id>.md` is the decision, and the filenames sort by time. D1–D98 remain valid as each file's `legacy` (D01M7XKEXQH4HG5371629M50E2G) | [design/decision-files.md](./design/decision-files.md) |
| **Advisory scorecard** | `ratchet: {advisory: true}`: reported, kept out of the ratchet's baseline, never gating (D79) | [design/scorecards-for-scorecards.md](./design/scorecards-for-scorecards.md) §6 |
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
| **JSON Schema document** | A contract in `schema/`, compiled into the binary as Draft 2020-12. Catalogued and scored (`maintainer-schema`, D77) | [design/schema-standards.md](./design/schema-standards.md) |
| **Example write-up** | A README under `examples/` or `experiments/`, or a document in `docs/examples`. Catalogued and scored (`maintainer-example`, D78) by whether a reader can follow its links and run its check | [design/example-standards.md](./design/example-standards.md) |
| **Site page** | One Starlight content page under `site/src/content/docs`. Catalogued and scored (`maintainer-site`, D91) by title, description, resolving links, sidebar reach, and no secret shape. The site also publishes this repo's `toolbox report` HTML (D92). The report is the `<toolbox-report>` element (Overview, Catalog, Campaigns, Scorecards; D01M7XY1QYKW2D0HMSG50R0XTQH, D01M7Y11G0YZT23W26Z9D3XW68Y); a host fills its header, nav, and footer slots (D93) | [design/site-standards.md](./design/site-standards.md), [design/report-ux.md](./design/report-ux.md) |
| **Track** | A long-term focus program in the manifest that groups scorecards and campaigns (Soundcheck "track") | [design/campaigns.md](./design/campaigns.md) |
| **Campaign** | A time-bound goal over existing scorecards via `includes` (`atLeast` and/or `rules`); progress is `%` of in-scope entities meeting every include (D01M7Y11G0YZT23W26Z9D3XW68Y) | [design/campaigns.md](./design/campaigns.md) |
| **Agent plugin package** | A directory with `plugin.json` (Agent Plugins 1.0.0). Catalogued and scored (`maintainer-plugin`, D81). Skills inside it stay on the skill scorecard | [design/plugin-standards.md](./design/plugin-standards.md) |
| **Repository level** | The lowest level among the scorecards a repository counts. Unknown rules are excluded. Catalogued on the repository record (`maintainer-repo`, D86) | [design/repo-level.md](./design/repo-level.md) |
| **Supply-chain config** | Dependabot ecosystems, a code-scanning workflow, a `SECURITY.md`, and a CODEOWNERS rule for CI. Catalogued on the repository record (`maintainer-supply-chain`, D85). Alert counts are not included | [design/supply-chain-standards.md](./design/supply-chain-standards.md) |
| **Charter goals (G1–G3)** | Simple single-container deployment, agent SDLC (in four layers, D63, D87), replaceable components | [decisions.md](./decisions/README.md) D33–D35 |
| **Snapshot / snapshot id** | A consistent (`REPEATABLE READ`) read of Postgres inputs from which all derived state is recomputed; its id is the consistency token | [decisions.md](./decisions/README.md) D37 |
| **Full recompute** | The v1 derived-state engine: recompute every derived value from a snapshot on change, behind the `DerivedState` contract | [technology-critique.md](./technology-critique.md) §7.1 |
| **Replaceable contract** | A Toolbox-owned trait, protocol or mount contract with conformance tests, behind which a default technology sits (`DerivedState`, `Authorizer`, `Durable`, UI extension contract, executor protocol) | [plan.md](./plan.md) §3 |
| **UI extension contract** | How operator UI bundles (any framework; React by default) are built into the image and mounted | [decisions.md](./decisions/README.md) D40 |
| **In-image executor** | Action code packaged into the Toolbox image and run as a child process over stdio | [decisions.md](./decisions/README.md) D40 |
| **Sprawl budget** | The cap on implementation languages, required services and author-facing languages in v1 | [decisions.md](./decisions/README.md) D45 |
| **Connector command** | A Toolbox → connector instruction (`refresh`, `resync`, `cancel`) with an id, priority, deadline and terminal status | [design/connector-control.md](./design/connector-control.md) |
| **Scoped refresh** | A re-read of specific records (by external ids, selector or `since`); never causes implicit deletes | [design/connector-control.md](./design/connector-control.md) |
| **Postcondition** | A CEL expression an action declares, which must hold after the run for it to succeed | [design/action-runs.md](./design/action-runs.md) |
| **Confirmation token** | Binds a run to the exact preview the requester saw; stale previews are rejected | [design/action-runs.md](./design/action-runs.md) |
| **`unverified`** | Run state when the executor reported success but Toolbox could not observe the effect; never shown as success | [design/action-runs.md](./design/action-runs.md) |
| **v1 spec** | The implementation contract: milestones, contracts, API, MCP, CLI, data model, properties | [v1-spec.md](./v1-spec.md) |
| **Reference portal** | The example portal (GitHub + IdP) whose outputs and scenarios define "done" for v1 | [examples/reference-portal](../examples/reference-portal/README.md) |
| **Reference model / oracle** | `tools/reference.py`: a small executable model of the pipeline that the Rust implementation must match | [v1-spec.md](./v1-spec.md) §6 |
| **Ownership status** | `verified`, `inferred`, `unverified` or `none`, from the CODEOWNERS authority rule | [v1-spec.md](./v1-spec.md) §6.2 |
| **Action context lookup** | Catalog entities an action declares (`{kind, where}`) and passes to its executor as `related` | [decisions.md](./decisions/README.md) D52 |
| **Canonical manifest** | The single key-sorted `manifest.json` that `toolbox build` produces from any source; what the server loads and `plan` diffs | [decisions.md](./decisions/README.md) D53 |
| **`include`** | Source-only list of YAML files merged into the canonical manifest | [decisions.md](./decisions/README.md) D54 |
| **View endpoint** | A composite, facet-shaped REST response serving one UI page in one request | [design/minimal-ui.md](./design/minimal-ui.md) §6 |
| **Form-renderable subset** | The JSON Schema keywords allowed in action inputs so a form can always be generated | [design/minimal-ui.md](./design/minimal-ui.md) §4 |
