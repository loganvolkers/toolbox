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
| Measured coupling of Ocean integrations to Port's API | 41 of 52 integrations never use `port_client`; 39 of 231 framework modules do |

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
