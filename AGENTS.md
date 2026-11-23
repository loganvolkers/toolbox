# AGENTS.md: Working on Toolbox

Toolbox is an open-source, agent-first internal developer portal (IDP) framework: first of all a
box for tools, not a wrapper around them ([toolbox-and-tools.md](./docs/design/toolbox-and-tools.md)). It has a Rust core,
is configured entirely in code, and exposes API, MCP, CLI and UI. **Status:** v1 is being implemented
milestone by milestone from [docs/v1-spec.md](./docs/v1-spec.md); M0–M4 are done (see *Building*
below). **Focus (D01M80Y0XEK069GD08VCCNF0WB8):** prove value from both ends. Some sessions work bottom-up, at a single
repository ([docs/design/repo-value.md](./docs/design/repo-value.md)); others work top-down, on the
central portal, inspired by Backstage, DX and Port. Federation is unpaused, and how the two ends meet
is decided once both are proven. Say which end you work on. Self-service actions (M5) stay paused
(D64). The design documents remain the source of intent, and an agent plugin skeleton lives in
`plugins/`.

**Four SDLC layers ([docs/sdlc.md](./docs/sdlc.md), D63, D87).** Say which one you mean:
1. **Maintainer SDLC**: you, building and testing Toolbox itself in this repo
   ([maintainer-sdlc.md](./docs/design/maintainer-sdlc.md)). The working rules below are its rules.
2. **Package author SDLC**: anyone publishing packages, connectors and standards for others
   ([package-author-sdlc.md](./docs/design/package-author-sdlc.md), second-package checklist §6).
   It forks from the maintainer SDLC at *test*: test against released Toolbox versions, then
   distribute.
3. **Operator SDLC**: platform teams building and changing a *portal*
   ([sdlc.md](./docs/design/sdlc.md)).
4. **User SDLC**: developers and agents building *apps* on a portal's context and actions
   ([user-sdlc.md](./docs/design/user-sdlc.md)).

## Read first

1. [README.md](./README.md): the document map.
2. [docs/decisions/](./docs/decisions/README.md): one file per decision. Open questions are
   [docs/questions.md](./docs/questions.md). **This is the source of truth for what's decided.**
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
`yamark format --check` (yamark 0.6.1, D97) checks every tracked `.yaml` and `.yml` except `.github/workflows/release.yml`, which `dist plan` owns (D98). `python3 scripts/decision-id.py --self-test` checks the ULID encoding for decision ids (D01M7XKEXQH4HG5371629M50E2G).
`.github/workflows/ci.yml` runs all of this. **M0–M4 are done.** Commands: `validate`, `test`,
`plan --local --base <manifest>`, `derive`, `serve [--database-url]`, `push`, `token create`, the reads
(`get`, `query`, `search`, `graph`, `owner`, `explain`, `score`, `scorecard`, `refresh`, `actions`), `mcp` (stdio
bridge), `connector test`, `describe` and `report` (repo portals and the ratchet, D66). The server also hosts `/mcp` and the UI at `/`, and supervises the
`toolbox-connector-{github,git-files}` binaries; `toolbox-connector-scim` runs remotely. The
acceptance scripts `scripts/m2…m4-acceptance.sh` need Postgres 16; M3 and M4 also need
`TOOLBOX_OPENFGA_BIN`, and M4 needs Node with Playwright. **M5 (actions) is paused (D64)** until a review of self-service coordination lands; don't start it. Meanwhile [`portal/`](./portal/README.md) is Toolbox for Toolbox: `toolbox report portal --skip runner,github-actions --baseline portal/results.json` (with zizmor 1.32.0, actionlint 1.7.12 and ruff 0.14.0 installed; `toolbox doctor portal` checks) scores this repo against the three SDLC layers' standards (D65), including the agent skills in `plugins/` (`maintainer-skill`, D72) and one skill per CLI command (`cli-skill`, D94) and the first-party connectors (`maintainer-connector`, D73) and direct Cargo dependencies (`maintainer-dependency`, D74) and agent instruction files (`maintainer-agent-doc`, D75) and npm manifests (`maintainer-package`, D76) and the JSON Schemas it compiles (`maintainer-schema`, D77) and the example write-ups agents copy (`maintainer-example`, D78) and the agent plugin packages it ships (`maintainer-plugin`, D81) and harness aliases plus the check commands named in `AGENTS.md` (`maintainer-agent-doc`, D82, D83) and cited decision ids (`maintainer-docs`, D84) and GitHub Advanced Security config (`maintainer-supply-chain`, D85) and the repository level, which is the lowest of those scorecards (`maintainer-repo`, D86) and the public docs pages (`maintainer-site`, D91) and the Python it maintains, through ruff in the SARIF slot (`maintainer-python`) and the HTML report the site publishes (`toolbox report`, D92; host slots, D93), and CI fails on regressions (D66); if you improve something, rerun with `--update` and commit `portal/results.json`. Pilot repos adopt this with one step, the `actions/report` GitHub Action and `examples/repo-portal-starter` (D69). Scorecards are scored too (`meta-rule`, `meta-scorecard`, D71): a new rule must be tested, explained and decided on fixtures from its first commit. Judgement is pushed, not pulled: brief yourself with `toolbox assess brief`, write the assessment to `portal/assessments/`, and run `toolbox assess check portal` (D79, D80; skill `assess-entity`). Every layer's stages are catalogued in `portal/sdlc/` and scored (`sdlc-stage`, `sdlc-layer`, D87). Federation is designed, not built (D68); packages are git-first with an OCI mirror (D89; `packages/` holds ours, included by T4T and tested in their own `tests/` portals), and Toolbox, connectors and tools are distributed per D90.

## Operator plugin

The cloud harness does not inject `plugins/toolbox-operator`. Before a Toolbox command, read
`plugins/toolbox-operator/skills/<command>/SKILL.md`. Those skills are the operator loop for this
repo: validate, test, report, assess. A score change follows the `report` skill.
`site/public/report/` is generated by the pages workflow and is not committed; preview it with
`scripts/regen-site-report.sh`. The ratchet verdict is the gate (D01M7Y5VWKW5TDS0VQWZFPRQ27N).

## Repository layout

```
docs/plan.md                 architecture + roadmap (no time estimates; phases end in exit criteria)
docs/decisions/             one file per decision; the filename is a ULID (D01M7XKEXQH4HG5371629M50E2G)
docs/questions.md            open questions (Q numbers)
docs/research.md             landscape and validation
docs/authorization.md        ReBAC / OpenFGA design
docs/ocean.md                Port Ocean review
docs/red-team.md             competitor-perspective gap review (with status)
docs/principal-review.md     principal-engineer review (with resolutions)
docs/adversarial-critique.md whether to build at all: hypotheses, alternatives, kill criteria
docs/technology-critique.md  sprawl and replaceability review; proposed minimal architecture
docs/sdlc.md                 the four SDLC layers: maintainer, package author, operator, user (D63, D87)
portal/                      Toolbox for Toolbox: this repo's own portal and standards (D65)
docs/design/*.md             one detailed design per capability (maintainer-, user-sdlc.md and sdlc.md = operator)
docs/examples/*.md           worked examples (e.g. approved GitHub Apps)
docs/glossary.md, docs/research-log.md
docs/v1-spec.md              the implementation contract for v1
Cargo.toml, crates/          Rust workspace (v1-spec §4); `toolbox-cli` builds the `toolbox` binary
yamark.toml                  YAML formatting (Yamark 0.6.1, D97)
schema/                      JSON Schemas (manifest, connector and executor messages), compiled into the binary
scripts/                     repo checks and acceptance scripts (validate-parity.py, m2–m4-acceptance.sh)
ui/                          the minimal UI (React + Vite; dist/ is embedded in the binary)
examples/reference-portal/   reference implementation anchor: manifest, schemas, fixtures, oracle, authz tests
experiments/                 reproducible experiments cited by the docs
plugins/toolbox-operator/    Agent Plugins 1.0.0 package (pre-release skeleton)
actions/report/              GitHub Action: `toolbox report` with a ratchet, for pilot repos (D69)
```

## Working rules (maintainer SDLC)

The full loop, check pyramid and measured timings are in
[docs/design/maintainer-sdlc.md](./docs/design/maintainer-sdlc.md).


- **Branching:** during the design phase, commit directly to `main` (decision D32). Pull with rebase
  before you start and again before you push. Keep commits small and scoped to one topic, so parallel
  agents rarely conflict.
- **Generated files:** never hand-merge `portal/results.json`. On a conflict, take upstream's copy
  and rerun `--update` ([portal/README.md](./portal/README.md#layout), D95). After OpenSSF greening
  (or any change that improves `maintainer-openssf`), commit the same baseline CI uses:
  `toolbox report portal --skip runner,github-actions --baseline portal/results.json --update`
  (refresh OpenSSF scores first **with** the runner / Scorecard, then rewrite under `--skip` so crate
  facts are not left `unknown`). The site's copy of the report is generated by the pages workflow,
  not committed (D01M7YAXH89ZJQ7Q4HP77NMHBX8); preview it with `scripts/regen-site-report.sh`
  (installs the same zizmor 1.32.0, actionlint 1.7.12 and ruff 0.14.0 pins as pages CI) — a bare
  `report … -o web` without those tools writes `unknown` for ci-lint rules. New T4T standards go in
  `portal/standards/<topic>.yaml`.
- **YAML format:** before you push workflow or manifest edits, run `yamark format` on the files you
  touched (yamark 0.6.1, D97). CI’s `yamark format --check` covers every tracked `.yaml`/`.yml`
  except `.github/workflows/release.yml` (cargo-dist / D98); whitespace-only CI edits still fail
  until reformatted.
- **When you change a decision:** mint an id with `python3 scripts/decision-id.py` and add
  `docs/decisions/<id>.md`. Do not edit a decision file that is already there. Update the owning
  design doc, and update `plan.md` §4.7/§8/§10 if affected.
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
  skill with `skills-ref validate`. Use `"license": "MIT"` (D88).
- **Writing style:** state conclusions first, then evidence. Prefer tables for comparisons. Keep docs
  self-contained, with relative links that resolve.
- **The box and the tools** ([toolbox-and-tools.md](./docs/design/toolbox-and-tools.md),
  D01M7ZRAJT47W11X5X3FEJ6QTX8). Toolbox is a box for tools, not a wrapper or a factory:
  - Every design doc and package README says whether it is about the box (carrying, pinning,
    running, remembering, reporting, distributing) or a tool (an existing tool, a fitting, a jig,
    or a standalone tool).
  - A tool in the box must still work when taken out: the same command, its own config file, the
    same findings.
  - Before adding an extractor to a Toolbox connector, say why no existing tool, framework plugin
    or standard slot (SARIF, NDJSON, JUnit) covers the fact.

## Charter goals (D33–D35)

Every design choice must serve these. Check new work against them.

1. **Simple deployment:** one container image plus Postgres (Cloud Run or equivalent); custom UIs and
   action code ship inside the image.
2. **Agent SDLC, in four layers (D63, D87):** agents can plan, build, test, deploy and monitor at every
   layer, with the maturity of regular software engineering:
   - *maintainers* build and verify Toolbox in isolation, with executable exit criteria;
   - *package authors* test packages and connectors against released Toolbox versions, then distribute them;
   - *operators* change a portal with validate → test → plan → preview → deploy → verify;
   - *users* build apps on stable, versioned API/MCP contracts with reproducible reads.
3. **Replaceable components:** React, DBOS, OpenFGA packaging and the derived-state engine are defaults
   behind Toolbox-owned contracts, not lock-ins. Respect the sprawl budget (D45).

## Parallel work: independent tracks

These Phase 0 spikes and v1 milestones ([docs/v1-spec.md](./docs/v1-spec.md) §3) can be taken by separate agents with few interactions. Coordinate through `docs/decisions/`. Design
docs put their current v1 design first and keep deferred material in a labelled appendix; where a status banner remains, it wins.

| Track | Scope | Primary docs |
|---|---|---|
| Derived state | Full-recompute pipeline from a real manifest behind `DerivedState`; `plan` as two recomputes + diff; extend `experiments/derived-state-scale` | incremental-computation, technology-critique §7–8 |
| Authorization | OpenFGA inside one container: co-process vs c-archive (Q11); tuple view; conditions and wildcards; `.fga.yaml` corpus through `Authorizer` | authorization |
| Durable runtime | `Durable` on the DBOS-compatible schema (`experiments/durable-schema`); DBOS Rust SDK spike (Q4); crash between steps; cross-instance recovery after scale-in (Q13) | v1-spec §8.1, incremental-computation |
| Deployment | One image on Cloud Run + Cloud SQL with a UI bundle and an in-image executor; instance-based billing; rollback by image | plan §6.2 |
| Maintainer SDLC | Keep the check pyramid green and fast; raise the Toolbox-for-Toolbox scorecards (`scripts/self-portal.sh`); property tests over generated portals; acceptance scripts for M5–M6 | maintainer-sdlc, verification, developer-velocity |
| Operator SDLC | `toolbox dev` / `plan --remote` / `build` / `verify` / preview deploys; dev-loop benchmark with SLOs | sdlc, developer-velocity |
| User SDLC | Per-portal OpenAPI + typed clients; service identities; generated user plugin; stability and deprecation policy | user-sdlc, agent-plugins |
| Connectors & actions runtime | Control channel (stdio / long-poll) with scoped refresh and completion; action run lifecycle with preview tokens and postcondition verification; in-image and GitHub Actions executors | connector-control, action-runs |
| Expressions | CEL conformance (Q10); fixture-based output validation against JSON Schemas | extensibility |
| Config | Canonical manifest, `include`, `toolbox validate`; the `@toolbox/manifest` builder from the prototype in `experiments/typescript-builder` | typescript-builder, v1-spec §5 |
| Frontend | Minimal UI pages and view endpoints; UI extension contract; default React kit (`@toolbox/ui`, Storybook 10); one non-React bundle through the same contract | minimal-ui, frontend |
| Verification | Differential tests (Rust vs SQL, OpenFGA vs SQL); property catalogue | verification |
| Backstage parity audit | D47: Phase 1 features vs Backstage 1.57 + plugins | technology-critique §4 |
| GitHub Apps example | Verify Q8 against the GitHub API | examples/approved-github-apps |
