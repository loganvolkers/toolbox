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
- **Plugin changes:** validate `plugin.json`/`mcp.json` against the official 1.0.0 schemas, and every
  skill with `skills-ref validate`. Don't add a `license` field until the project licence is decided
  (Q1).
- **Writing style:** state conclusions first, then evidence. Prefer tables for comparisons. Keep docs
  self-contained, with relative links that resolve.

## Parallel work: independent tracks

These Phase 0 spikes ([docs/plan.md](./docs/plan.md) §8) can be taken by separate agents with few
interactions. Coordinate through `docs/decisions.md`.

| Track | Scope | Primary docs |
|---|---|---|
| Circuit | Manifest → DBSP circuit at runtime (mapping, merge, recursive closure, CEL rule); reconfiguration time; sub-circuit state reuse (Q9) | incremental-computation, developer-velocity |
| Authorization | `toolbox-fga` c-archive + read-only datastore; OpenFGA corpus through the adapter; cgo overhead and signals | authorization |
| Change log | Logical-decoding change log; log positions as tokens | incremental-computation |
| Durable runtime | DBOS Rust SDK behind a `Durable` trait; crash-between-steps tests (Q4) | incremental-computation |
| Expressions | Typed CEL with object construction; cel-spec conformance (Q10) | extensibility |
| Config SDK | Schema → TS/Python SDK codegen; language hosts; manifest determinism | plan §2, extensibility |
| Verification | `Env` seam, `MemStore` isolation model, minimal `toolbox-sim`, property catalogue | verification |
| Velocity | Workspace skeleton (interface crates, profiles, hakari, nextest, Bacon, `xtask affected`); dev-loop benchmark harness | developer-velocity |
| Frontend | `@toolbox/ui` primitives, Storybook 10, story-based tests | frontend |
| GitHub Apps example | Verify Q8 against the GitHub API | examples/approved-github-apps |
