# Research Log: Methods, Constraints and Hands-On Verifications

How the design research was done, so parallel agents can reproduce, extend or challenge it.

## Environment constraints (cloud session, Nov 2026)

- **Blocked hosts** (proxy refused): backstage.io, docs.port.io, ocean.getport.io, docs.cortex.io,
  docs.opslevel.com, docs.getdx.com, warpstream.com, github.blog, agent-plugins.org, agentskills.io,
  modelcontextprotocol.io, docs.tilt.dev, developer.hashicorp.com, crates.io API.
- **What worked:**
  - `raw.githubusercontent.com` and shallow `git clone` of public GitHub repos.
  - `gh api search/*` (repositories, code, issues).
  - Web search for indexed excerpts.
- **Rule used throughout:** prefer primary sources, read from GitHub where the website is blocked.
  Label search-excerpt claims as secondary, and never fill a gap from memory without saying so.

## Primary sources read locally (not just via search)

| Source | Used for |
|---|---|
| `backstage/backstage` docs (`life-of-an-entity`, `descriptor-format`, `faq`, `ai/mcp-actions`, `ai/ai-in-the-catalog`, templates, TechDocs, search, well-known annotations) | [research.md](./research.md), authority/ownership semantics |
| `port-labs/terraform-provider-port-labs` resource docs (blueprint, scorecard, action) | Port data model, actions, timers, `upsert_entity_method` |
| `port-labs/ocean` source (~61k lines): mapping models, `sync_raw`, deletion threshold, incremental strategies, probes, identity propagation, actions, webhooks, the **lakehouse client** | [ocean.md](./ocean.md), [design/context-lake.md](./design/context-lake.md) |
| `openfga/openfga.dev` docs, `openfga/language` grammar and tests, `openfga/openfga` `assets/tests` corpus and `pkg/server` + `pkg/storage` | [authorization.md](./authorization.md) |
| `zvectorlabs/authz-core` | Rust ReBAC landscape |
| `cartography-cncf/cartography` README | Security graph landscape |
| `feldera/feldera` LICENSE and `crates/dbsp` | DBSP licence (MIT) and capabilities |
| `TimelyDataflow/differential-dataflow`, `salsa-rs/salsa` licences | IVM landscape |
| `dbos-inc/dbos-transact-rust` README | Durable execution |
| `agentplugins/agent-plugins-spec` (1.0.0, 1.1.0 draft, schemas, maintainers) and `agentskills/agentskills` (spec, `skills-ref`) | [design/agent-plugins.md](./design/agent-plugins.md) |
| `rust-lang/rust-project-goals` 2026 "Fast Builds" | [design/developer-velocity.md](./design/developer-velocity.md) |
| GitHub changelog 2026-10-13 (via the `forks-felickz/github-changelog-reader` mirror issue) and `github/roadmap#1376` | [examples/approved-github-apps.md](./examples/approved-github-apps.md) |
| Competitor READMEs: Frontkeep, OpsCat, Lekton, Decathlon idp-core, Backline, ForgePortal, Torii, Clutch, Harmonix, Portway, Junction, Cortex engrams | [research.md](./research.md) §5 |

## Hands-on verifications

| What | Result |
|---|---|
| The authorization doc's OpenFGA model + `.fga.yaml` test, run with the real `fga` CLI (`go install github.com/openfga/cli/cmd/fga`) | 6/6 checks, 2/2 ListObjects, 1/1 ListUsers pass; flipping one expectation fails as expected. Found that `fga model test` refuses `../` model paths, so tests sit beside `fga.mod` |
| `plugins/toolbox-operator/plugin.json` and `mcp.json` against the official Agent Plugins 1.0.0 JSON schemas (`jsonschema`, Draft 2020-12) | Valid |
| All operator-plugin skills with `skills-ref validate` | Valid |
| Naive full recompute, 100 CEL rules × 100k entities (cel-rust 0.16 at `004a4b8`, 4 vCPU), [`experiments/full-recompute`](../experiments/full-recompute) | 6.3–7.1 s single-threaded, 1.75–2.0 s on 4 threads; team closure + roll-up 5–7 ms. Basis for [adversarial-critique.md](./adversarial-critique.md) §7 (H4) |
| Measured coupling of Ocean integrations to Port's API | 41 of 52 integrations never use `port_client`; 39 of 231 framework modules do |
| Full-pipeline recompute at 100k entities / 2M relations / 10k users / 100 CEL rules (merge, rules, cross-entity aggregates, closures, roll-ups, tuples, permission view), [`experiments/derived-state-scale`](../experiments/derived-state-scale) | Rust + cel-rust: 2.97 s on 4 threads (11.8 s on 1); ~98% of it is CEL. Postgres 16 materialised views: 2.61 s (+18.5 s to materialise ListObjects for all users). Identical outputs. Single-entity update 0.33–1.55 ms. Basis for [technology-critique.md](./technology-critique.md) §7.1 |
| Unmodified OpenFGA v1.22.1 (Postgres datastore) reading derived tuples through a read-only `tuple` view, same experiment (`openfga/run.sh`) | 2,000/2,000 checks agree with the SQL permission view; ListObjects 667 = 667; Check p50 2.4 ms, p95 3.6 ms over HTTP; an ownership change is visible after a 2.6 s refresh. Writes, `ReadChanges`, conditions not tested |
| Build cost of dbsp 0.384, wasmtime 50.0.3, tantivy 0.27, async-graphql 7.2.1 vs axum/sqlx/rmcp/cel, [`experiments/build-cost`](../experiments/build-cost) | **rustc 1.97.0 ICE compiling dbsp** at opt-level 2 (Feldera pins 1.99.0). On 1.99.0: clean dev build 496 s vs 96 s, release 363 s vs 64 s, 494 vs 167 crates, release binary 52.9 vs 10.1 MB; edit rebuilds 1.2–1.3 s vs 0.45–0.5 s |
| OpenFGA v1.22.1 as a Go c-archive (`-buildmode=c-archive`, Go 1.26.11), `experiments/build-cost/fga-carchive` | 45 s cold build, 2.0 s warm, 78 MB archive |

## How to re-run the verifications

```bash
# Agent Plugins schemas (clone the spec repo to a scratch dir first)
python3 -c "import json,jsonschema; jsonschema.validate(json.load(open('plugins/toolbox-operator/plugin.json')), json.load(open('<spec>/schemas/1.0.0/plugin.schema.json')))"
# Skills
skills-ref validate plugins/toolbox-operator/skills/<skill>
# OpenFGA example: put the model from authorization.md §4.2 in authz/core.fga, an fga.mod beside it,
# and the §4.6 test file in authz/scorecards.fga.yaml, then:
fga model test --tests authz/scorecards.fga.yaml
```
