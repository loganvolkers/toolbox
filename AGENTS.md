# AGENTS.md: Working on Toolbox

Toolbox is an open-source, agent-first internal developer portal (IDP) framework. It has a Rust core,
is configured entirely in code, and exposes API, MCP, CLI and UI. **The project is in the design
phase:** this repository currently holds design documents and a pre-release agent plugin skeleton,
not product code.

## Read first

1. [README.md](./README.md): the document map.
2. [docs/decisions.md](./docs/decisions.md): every decision made so far, plus open questions.
   **This is the source of truth for what's decided.**
3. [docs/plan.md](./docs/plan.md): architecture and roadmap. §4.7 indexes the detailed designs.
4. [docs/glossary.md](./docs/glossary.md): shared vocabulary.
5. [docs/research-log.md](./docs/research-log.md): how research was done, blocked hosts, and what was
   verified hands-on.

**Implementing?** Build from [docs/v1-spec.md](./docs/v1-spec.md) (milestones M0–M6, contracts, API,
MCP, CLI, data model, properties) and test against
[examples/reference-portal](./examples/reference-portal/README.md): your pipeline must reproduce its
`expected/` outputs byte for byte, and its acceptance scenarios define "done".

**Building:** `cargo build` (toolchain pinned in `rust-toolchain.toml`), `cargo test`, `cargo clippy
--all-targets -- -D warnings`, then `target/debug/toolbox validate examples/reference-portal`.
`python scripts/validate-parity.py` checks `toolbox validate` against `tools/validate.py`.
`.github/workflows/ci.yml` runs all of this. **M0 and M1 are done** (`toolbox test`, `toolbox plan --local --base <manifest>`, `toolbox derive`); M2 (Postgres and API) is next.

## Repository layout

```
docs/plan.md                 architecture + roadmap (no time estimates; phases end in exit criteria)
docs/decisions.md            decision log + consolidated open questions
docs/research.md             landscape and validation
docs/authorization.md        ReBAC / OpenFGA design
docs/ocean.md                Port Ocean review
docs/red-team.md             competitor-perspective gap review (with status)
docs/principal-review.md     principal-engineer review (with resolutions)
docs/adversarial-critique.md whether to build at all: hypotheses, alternatives, kill criteria
docs/technology-critique.md  sprawl and replaceability review; proposed minimal architecture
docs/design/*.md             one detailed design per capability
docs/examples/*.md           worked examples (e.g. approved GitHub Apps)
docs/glossary.md, docs/research-log.md
docs/v1-spec.md              the implementation contract for v1
Cargo.toml, crates/          Rust workspace (v1-spec §4); `toolbox-cli` builds the `toolbox` binary
schema/                      JSON Schemas (manifest, connector and executor messages), compiled into the binary
scripts/                     repo checks (validate-parity.py)
examples/reference-portal/   reference implementation anchor: manifest, schemas, fixtures, oracle, authz tests
experiments/                 reproducible experiments cited by the docs
plugins/toolbox-operator/    Agent Plugins 1.0.0 package (pre-release skeleton)
```

## Working rules

- **Branching:** during the design phase, commit directly to `main` (decision D32). Pull with rebase
  before you start and again before you push. Keep commits small and scoped to one topic, so parallel
  agents rarely conflict.
- **When you change a decision:** add an entry to `docs/decisions.md` (never rewrite past entries),
  update the owning design doc, and update `plan.md` §4.7/§8/§10 if affected.
- **Sources:**
  - Prefer primary sources. Many vendor docs sites are blocked in cloud sessions; read their GitHub
    repos instead (see [docs/research-log.md](./docs/research-log.md)).
  - Label secondary or unverified claims as such.
  - Put sources at the end of each design doc.
- **No time estimates** in any document (D25).
- **Verify what you can:**
  - Runnable examples (OpenFGA models, plugin manifests, skills) must be run or validated before they
    are committed. Record the result in `docs/research-log.md`.
  - When you change the reference portal, run its three checks (`tools/validate.py`,
    `tools/reference.py --check`, `fga model test`) and regenerate outputs with `--write` only after
    reviewing the diff.
- **Plugin changes:** validate `plugin.json`/`mcp.json` against the official 1.0.0 schemas, and every
  skill with `skills-ref validate`. Don't add a `license` field until the project licence is decided
  (Q1).
- **Writing style:** state conclusions first, then evidence. Prefer tables for comparisons. Keep docs
  self-contained, with relative links that resolve.

## Charter goals (D33–D35)

Every design choice must serve these. Check new work against them.

1. **Simple deployment:** one container image plus Postgres (Cloud Run or equivalent); custom UIs and
   action code ship inside the image.
2. **Agent SDLC:** plan, build, test, deploy and monitor a portal with a full, mature SDLC.
3. **Replaceable components:** React, DBOS, OpenFGA packaging and the derived-state engine are defaults
   behind Toolbox-owned contracts, not lock-ins. Respect the sprawl budget (D45).

## Parallel work: independent tracks

These Phase 0 spikes and v1 milestones ([docs/v1-spec.md](./docs/v1-spec.md) §3) can be taken by separate agents with few interactions. Coordinate through `docs/decisions.md`. Design
docs put their current v1 design first and keep deferred material in a labelled appendix; where a status banner remains, it wins.

| Track | Scope | Primary docs |
|---|---|---|
| Derived state | Full-recompute pipeline from a real manifest behind `DerivedState`; `plan` as two recomputes + diff; extend `experiments/derived-state-scale` | incremental-computation, technology-critique §7–8 |
| Authorization | OpenFGA inside one container: co-process vs c-archive (Q11); tuple view; conditions and wildcards; `.fga.yaml` corpus through `Authorizer` | authorization |
| Durable runtime | `Durable` on the DBOS-compatible schema (`experiments/durable-schema`); DBOS Rust SDK spike (Q4); crash between steps; cross-instance recovery after scale-in (Q13) | v1-spec §8.1, incremental-computation |
| Deployment | One image on Cloud Run + Cloud SQL with a UI bundle and an in-image executor; instance-based billing; rollback by image | plan §6.2 |
| Operator SDLC | `toolbox dev` / `test` / `plan` / preview deploys; dev-loop benchmark with SLOs | sdlc, developer-velocity |
| Connectors & actions runtime | Control channel (stdio / long-poll) with scoped refresh and completion; action run lifecycle with preview tokens and postcondition verification; in-image and GitHub Actions executors | connector-control, action-runs |
| Expressions | CEL conformance (Q10); fixture-based output validation against JSON Schemas | extensibility |
| Config | Canonical manifest, `include`, `toolbox validate`; the `@toolbox/manifest` builder from the prototype in `experiments/typescript-builder` | typescript-builder, v1-spec §5 |
| Frontend | Minimal UI pages and view endpoints; UI extension contract; default React kit (`@toolbox/ui`, Storybook 10); one non-React bundle through the same contract | minimal-ui, frontend |
| Verification | Differential tests (Rust vs SQL, OpenFGA vs SQL); property catalogue | verification |
| Backstage parity audit | D47: Phase 1 features vs Backstage 1.57 + plugins | technology-critique §4 |
| GitHub Apps example | Verify Q8 against the GitHub API | examples/approved-github-apps |
