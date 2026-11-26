# Port Ocean — What Seikika Should Learn

*Reviewed 2026-11-18 from the source of [`port-labs/ocean`](https://github.com/port-labs/ocean) (Apache-2.0,
default branch, shallow clone). Line counts are from that checkout. Companion to
[research.md](./research.md) and [plan.md](./plan.md) §4.6.*

## What Ocean is

Ocean is Port's open-source **Python framework for integrations**. The framework is ~61k lines of
Python, including tests. The monorepo has **52 integrations**: GitHub, GitLab, Jira, PagerDuty, Datadog,
AWS, Azure, GCP, Snyk, Wiz, Okta, ServiceNow, Backstage, Claude/Cursor agents, and a generic HTTP
integration.

An integration runs in the customer's environment and pushes data into Port's SaaS. Its core contract
is small:

```python
@ocean.on_resync("repository")           # 331 uses across integrations
async def resync_repos(kind: str):
    async for page in client.paginate("/repos"):
        yield page                         # raw API objects; no knowledge of Port's data model
```

The integration's mapping config (`port-app-config.yml`) turns those raw objects into Port entities
with **JQ**. That config is stored in Port and edited in Port's UI or API:

```yaml
resources:
  - kind: repository
    selector: { query: 'true', includedFiles: [README.md, CODEOWNERS] }   # per-kind, typed pushdown options
    port:
      entity:
        mappings:
          identifier: .name
          blueprint: '"githubRepository"'
          properties: { url: .html_url, readme: '.__includedFiles["README.md"]' }
          relations:  { organization: .owner.login }
```

Each integration ships `.port/spec.json` (config schema, secrets, features) plus **default blueprints
and default mappings**. Installing one therefore gives you a working catalog immediately.

## Verdict

**Learn from the design. Don't adopt the framework.** Ocean encodes several years of production lessons
about syncing messy third-party APIs. Most of those lessons are protocol and semantics decisions that
Seikika can copy. The implementation itself doesn't fit Seikika:
- It is Python-only.
- It is tightly coupled to Port's SaaS API (`port_client`, Kafka/polling for config and actions).
- Its mapping config lives in Port's database.
- It reconciles inside each connector.

One exception is worth a spike: **41 of 52 integrations never touch the Port client directly**. They
only implement `on_resync` and similar hooks, so a thin compatibility runner could reuse them (§4).

### Reuse decision: don't adopt Ocean as the connector framework

| Question | Evidence (from source) |
|---|---|
| Is the framework neutral, or Port's client? | It is Port's client. The only mapping-config source is `APIPortAppConfig` (read from Port's API), settings require a Port `client_id`, and **39 of 231** framework modules reference `port_client`, including the core sync path, which diffs against Port's entities. |
| Could we fork and decouple it? | That means owning ~61k lines of Python that track a fast-moving 0.x upstream: **497 releases**, three patch releases in the three days before this review. Port controls the direction. |
| Does it fit the polyglot goal? | No. It would make Python the de facto connector language. |
| Does it fit our architecture? | No. Its connectors are thick (each reconciles against the central store), its config lives in a DB, and it maps with JQ. |

**Reuse the integrations, not the framework:**
- **Compatibility runner (§4).** Run unmodified integrations as TCP sources, if the spike's bar is met.
- **Ocean-shaped Python SDK.** Give the TCP Python SDK the same decorator surface
  (`@source.on_resync("repository")`, `@source.on_incremental_resync`, webhook processors), so porting an
  Ocean integration is mostly mechanical, for people and for agents.
- **Converted defaults.** Machine-convert each integration's default JQ mappings and blueprints into
  CEL `defaults()` helpers.
- **API knowledge.** Use the integrations as the reference for each vendor's pagination, rate-limit and
  webhook quirks. This is the most valuable thing in the repo.

## 1. Ideas to adopt (changes to the plan)

| # | Ocean idea | Where in Ocean | Seikika change |
|---|---|---|---|
| 1 | **Connectors emit raw objects; mapping lives in config.** Users reshape data without forking a connector. | `on_resync` returns raw dicts; `port-app-config.yml` JQ mappings | TCP gains a `RECORD {kind, data}` message. The engine maps records to entity fragments with CEL from the portal program. `FRAGMENT` stays for connectors that want to do their own mapping. |
| 2 | **Connector packages ship defaults** (blueprints + mappings), so they work out of the box. | `.port/resources/blueprints.json`, `port-app-config.yml` | Each connector package exports an SDK helper, e.g. `github.defaults(portal)`, that declares default kinds, mappings and (new) starter scorecards *in code*. Every piece can be overridden or omitted. |
| 3 | **Typed, per-kind selectors** pushed down to the source API (`includedFiles`, `states`, `repositoryType`). | `ResourceConfig` subclasses per integration | `discover` returns a JSON Schema per record kind for its selector. The SDKs generate typed selector options, and `seikika build` validates them. |
| 4 | **One record → many entities** (`itemsToParse`), e.g. a repo record producing one entity per file or member. | `PortResourceConfig.items_to_parse` | Mapping option `explode: "<CEL list expr>"`. |
| 5 | **Relations and identifiers by lookup** ("relate to the user whose email is X"), plus **auto-created placeholder targets**. | `IngestSearchQuery` identifiers and relations; `createMissingRelatedEntities` | Mappings may set `rel.lookup(kind, "<CEL predicate>")`. It is resolved in a linking phase *after* the pure merge, so the ingest path still never calls out. Unresolved targets become placeholder entities flagged in `status`. |
| 6 | **Deletion safety valves**: skip deletes if any error occurred or prior state is unknown; refuse deletes above a ratio (`entityDeletionThreshold`, default 0.9); per-kind `enableDelete`. | `sync_raw._resync_reconciliation`, `applier.delete_diff` | Adopt all three in the engine's full-sync reconciliation: no `SNAPSHOT_COMPLETE` means no deletes; any `ERROR` blocks deletes for that kind; a mass-deletion threshold triggers an alert; per-kind `delete: false`. |
| 7 | **Incremental sync strategies**: server-side `since` filters vs. client-side early-stop pagination over a persisted cursor. | `core/incremental/strategies.py`, `cursor_store.py` | The engine persists `STATE` cursors per (connector, kind). SDKs ship both strategy helpers. Periodic full syncs remain the source of truth for deletes. |
| 8 | **Permission probes**: before syncing, check which kinds the credentials can read, and report missing scopes per kind. | `core/probe/*` (`KindPermissionVerdict`) | TCP `check` returns a per-kind verdict (`granted | denied | unknown` + missing scopes). The UI, `seikika connector check` and MCP report it, which turns "why is my catalog empty?" into an actionable answer. |
| 9 | **A webhook-processor lifecycle**: `authenticate → validate_payload → handle_event`, with retryable errors, exponential backoff and routing per event type. | `core/handlers/webhook/*` | Make this the SDK shape for the TCP `webhook` endpoint. Signature verification lives in the connector; the engine provides routing, retries and dead-lettering. |
| 10 | **Connectors can execute actions** too, with a **partition key** (serialize runs per resource), **rate-limit awareness** (back off before quota is exhausted), and **async completion via webhooks**. | `core/handlers/actions/abstract_executor.py`, `execution_manager.py` | The TCP `act` capability adopts `partition_key`, `rate_limit` hints and async completion callbacks as standard fields. |
| 11 | **Identity propagation**: actions run *as the requesting user* in the third-party system, using brokered OAuth tokens kept in a vault and refreshed with per-secret locking. | `identity_propagation/*` | This matters even more for agents: an agent acting for Alice should open the PR *as Alice*. Add an on-behalf-of token broker to `seikika-authz`. `act` requests declare `identity: service | user`. |
| 12 | **Run modes**: long-running server, `ONCE` (cron/CI, see the *ocean-sail* GitHub Action), `WEBHOOKS_ONLY`, `ACTIONS_ONLY`. | `core/event_listener/*` | TCP connectors support `serve` and `once`. `seikika connector run --once --push <portal>` lets CI or cron feed a portal with no extra infrastructure. |
| 13 | **A generic HTTP connector**: endpoints as kinds, configurable auth and pagination, no code. | `integrations/custom` | Ship a first-party **declarative HTTP connector** (OpenAPI-aware). It will cover a large share of internal systems without anyone writing a connector. |
| 14 | **Agents build integrations**: Ocean ships agent skills (`create-ocean-integration`, `create-integration-action`) with execution boundaries, plus reference files on pagination, rate limits and webhooks. | `.cursor/skills/*` | Seikika ships an equivalent connector-authoring skill and an MCP dev tool (`seikika connector new/test`). The plan already does this for portal authoring; extend it to connectors and add the TCP conformance kit as the agent's feedback loop. |

## 2. What to do differently

| Ocean | Problem | Seikika |
|---|---|---|
| Mapping config stored in Port, edited in the UI; a config change triggers a **full re-fetch** (`config_change_processor`) | Config in a DB (our non-goal), and remapping is slow and costly against API rate limits | Mappings live in code. The engine keeps a **bounded raw-record store**, so a mapping change re-maps *without re-fetching*, and `seikika plan` previews mapping changes against stored records. |
| **Thick connector**: reconciliation runs in each connector, which fetches Port's current state to diff before upserting (`sync_raw.py` is 1.7k lines) | Every connector carries sync complexity, plus extra load on the central API | **Thin connector, smart engine**: connectors stream records, cursors and snapshot markers. Diffing, hashing, deletes and relation linking happen once, in Rust, next to the data. |
| JQ evaluated per field per record in Python (`jq_entity_processor.py`, 626 lines + sync variant) | Slow at scale; JQ is powerful but hard to type-check | **CEL** mappings compiled once and type-checked against kind schemas at build time, then evaluated in Rust. A JQ→CEL converter helps import Port configs. |
| Python-only framework; integrations range from 0.2k to **25k lines** (GitHub) | High cost per connector; one language | Language-neutral protocol + SDKs in many languages + a declarative HTTP connector, so most sources need far less code. |
| Requires Port SaaS reachability; Kafka and Redis for live events and stream maintenance | Heavy operational footprint | Self-hosted engine; HTTP push/pull; Postgres only. |

## 3. Small details worth keeping

- Per-kind selector `query` (a filter evaluated before mapping) → a CEL `where` on each mapping.
- `spec.json` marks secrets as `sensitive`, so they are never logged or echoed → the TCP `spec` JSON
  Schema uses `writeOnly`/`x-secret`.
- Lifecycle hooks `on_start`, `on_resync_start` and `on_resync_complete` → SDK hooks.
- ETL phases (extract / transform / load / reconcile) are tagged on every log line and metric → the
  same phase labels on connector OpenTelemetry spans and metrics.
- Integration smoke tests run against a fake integration and fake API, with recorded config sets → the
  basis of the TCP conformance kit.

## 4. Spike: an Ocean compatibility runner

**Hypothesis:** a small Python package, `seikika-ocean-runner`, can load an unmodified Ocean integration,
call its `on_resync` / `on_incremental_resync` / webhook handlers, and emit TCP `RECORD`/`STATE`
messages. It would stub the parts of the `ocean` context it touches and ignore `port_client`.

**Why it's worth a week:**
- It could give Seikika ~40 production-tested sources on day one.
- Ocean's default JQ mappings could be machine-converted to CEL and shipped as `defaults()`.

**Risks:**
- Integrations depend on Ocean internals that change frequently.
- 11 integrations (including GitHub, GitLab, Jira and PagerDuty) call the Port client and would need
  per-integration shims.
- Mixing a Python runtime into a "Rust core" story.

**Decision rule:** pursue it only if ≥ 25 integrations run unmodified with < 500 lines of shim. Otherwise
use Ocean purely as a design reference and port the top connectors by hand (or with agents).

Ocean is Apache-2.0, so reuse is permitted with attribution and the NOTICE file.
