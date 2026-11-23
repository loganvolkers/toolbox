# Context Lake: Where the Catalog Graph, Knowledge Graphs and Security Graphs Meet

*Status: research + design. Research date 2026-11-18.
Primary sources:*
- *Port Ocean source code (`port_ocean/clients/port/mixins/integrations.py`, `core/models.py`,
  `config/settings.py`).*
- *The Cartography README.*
- *Earlier primary reads (Backstage docs, Port's Terraform schemas).*

*Vendor claims (Port, Cortex, Atlassian, Wiz, Zep) come from their own pages via search excerpts and are
marked as such. Related: [plan.md](../plan.md) §4.1 and §4.4, [identity-resolution.md](./identity-resolution.md),
[meta-catalog.md](./meta-catalog.md), [authorization.md](../authorization.md).*

---

## 1. TL;DR

- **"Context lake" is a positioning term, not a technology.** Port coined it for "a unified engineering
  knowledge layer… typed nodes with explicit relationships", consumed by agents through MCP. Others
  build the same thing under other names: Cortex **Context Graph**, Atlassian **Teamwork Graph** (opened
  to third-party agents in May 2026), Glean's enterprise graph, GitLab "Orbit". Zep uses "context lake"
  for governed agent memory.
- **Port's context lake is literally backed by a lake.** Ocean ships *raw* third-party API
  `request`/`response`/`items` with extraction timestamps and selector hashes to a
  `/lake/write/...` endpoint, with `lakehouse_enabled: True` by default. The pattern is: raw lake →
  typed graph → agent interface. This validates the bounded raw-record store we already designed
  ([ocean.md](../ocean.md)).
- **Three graphs overlap, but they are different instruments:**
  - **Catalog graph:** curated, logical, low-churn. Answers *who owns what and how it fits together*.
  - **Security graph** (Wiz, Cartography): discovered, physical, high-churn, millions of nodes. Answers
    *what can reach what, and which combinations are dangerous*.
  - **Knowledge graph** (GraphRAG-style): extracted from text, fuzzy, high-volume. Answers *what do we
    know and why*.
- **They join on a small set of bridge entities:** repository, container image, workload/deployment,
  service, team/person, cloud identity, API, datastore. These join keys are exactly the external ids from
  [identity-resolution.md](./identity-resolution.md).
- **The market is converging from both sides.** Wiz launched a **Service Catalog** with ownership,
  owner suggestions and **Backstage owner import**. Wiz Code maps cloud risks back to repos, commits and
  owners. Meanwhile IDPs integrate Wiz. Ownership is the contested asset.
- **Recommendation:** make Toolbox's context lake a **layered, catalog-anchored, permission-aware
  lake**:
  1. A **raw layer** (bronze).
  2. A **two-tier graph**: curated catalog plus a bounded *inventory* tier for bridge resources.
  3. A **catalog-anchored knowledge layer**: no full GraphRAG; entity linking against the catalog
     dictionary; typed LLM extraction only for declared relation types.
  4. A **derived layer** (gold): scores, findings, context packs.
  5. **Federation instead of replication** for deep security graphs: import findings and toxic
     combinations, query attack paths live.
  6. **Export ownership** to security tools, so Toolbox becomes the **ownership system of record**.

---

## 2. What "context lake" means across the market

| Vendor / project | Name | What it is (per their materials) | Interface for agents |
|---|---|---|---|
| **Port** | Context Lake | Unified engineering knowledge layer: services, teams, incidents, deployments, runbooks, workflows, decisions as typed nodes with explicit relationships. "Not a separate feature" but the result of the catalog + integrations + AI working together. **Under the hood (Ocean source):** raw API payloads go to a lakehouse (`/lake/write/integration-type/.../`), `LakehouseDataEntry {request, response, items, metadata{operation, extractionTimestamp, resourceIndex, selectorHash}}`, buffered (10 s / 50 entries), enabled by default | MCP server (remote) |
| **Cortex** | Context Graph | AI/ML-assisted map of services, dependencies, ownership, standards, runbooks; also catalogs AI models and agents | Cortex MCP, AI assistant |
| **Atlassian** | Teamwork Graph | Unified data layer over Atlassian apps and 100+ connected apps. **Opened to third-party agents (Claude Code, Codex) in May 2026.** Retrieved items keep the same permissions, audit boundary and policy enforcement as for a human | MCP server, Graph CLI, graph query language |
| **Glean** | Enterprise graph | Neutral enterprise search and reasoning layer (details not verified here) | Search/agents APIs |
| **Zep / Graphiti** | Context lake (as agent memory) | **Bitemporal** knowledge graph: edges carry `valid_at`/`invalid_at`/`created_at`; contradictions invalidate rather than delete (Apache-2.0) | SDKs, MCP |
| **Microsoft GraphRAG** | (pattern) | LLM extracts entities and relations from text, Leiden clustering into communities, community summaries; local/global/DRIFT search. **Costly extraction, re-extraction on updates, quality bounded by entity resolution** (secondary sources) | Library |

**Common denominators:**
1. A typed graph of entities and relations.
2. Connectors feeding it continuously.
3. Permission preservation at retrieval.
4. MCP as the agent interface.
5. (Port, Zep) Retention of raw or temporal history.

## 3. Security graphs: Wiz and Cartography

**Wiz Security Graph** (vendor and secondary sources):
- Agentless API scanning into a continuously updated graph. A secondary source says it is built on
  Amazon Neptune.
- **Toxic combinations:** individually low-risk issues (exposure + vulnerability + excessive permission
  + sensitive data) that chain into a critical path.
- **Attack path analysis:** traces sequences an attacker could take.
- **Runtime signals:** live network connections layered into the graph to validate paths.
- Reported effect: 10,000+ vulnerabilities reduced to dozens of critical combinations.
- Text-to-graph-query (AskAI), built by retrieving example queries and graph-entity descriptions.
- GraphQL API.
- **Wiz Code:** maps cloud risks back to the repository, branch, Dockerfile and **commit author**;
  routes fix PRs to owners.
- **Wiz Service Catalog:** auto-groups resources into services with ownership metadata, an **owner
  suggestion flow, Backstage owner import**, ownership-gap boards and posture policies ("every
  production service must have an owner").
- **Projects:** organisational boundaries and RBAC scopes.
- **Google completed the $32B acquisition of Wiz on 11 March 2026.** Wiz keeps its brand and
  multi-cloud support.

**Cartography** (CNCF Sandbox since Aug 2024; Apache-2.0; ~4.2k stars; created at Lyft; basis of
SubImage):
- Python ingestion of 30+ platforms (AWS, GCP, Azure, Kubernetes, GitHub, Okta, Entra, CrowdStrike,
  AI BOMs…) into **Neo4j**.
- **Cypher** queries.
- `cartography-rules` for security rules evaluated over the graph. These are effectively scorecards over
  a security graph.

**Takeaways for Toolbox:**
1. Security graphs are *path-reasoning* engines over high-cardinality, high-churn infrastructure. That's
   a different workload from a curated catalog. **Don't rebuild one; federate.**
2. Their value multiplies when joined with *ownership and business criticality*, which they lack and
   keep trying to import (Wiz from Backstage). **Toolbox should be the source they import from.**
3. "Rules over a graph" (Cartography) and "toxic combinations" (Wiz) map naturally onto **scorecards
   with graph predicates**, which our rule language should support (§6.4).

## 4. Three graphs compared

| Dimension | Catalog graph | Security graph | Knowledge graph |
|---|---|---|---|
| Purpose | Ownership, structure, standards, golden paths | Exposure, reachability, privilege, risk prioritisation | Understanding, rationale, history ("why") |
| Typical nodes | Service, System, Domain, API, Team, User, Repo, Scorecard | Cloud resources, identities, roles, policies, network paths, packages, CVEs, secrets, data stores | Docs, ADRs, incidents, tickets, conversations, concepts |
| Cardinality / churn | 10³–10⁵, low churn (human-curated) | 10⁶–10⁸, high churn (machine-discovered) | 10⁵–10⁷ chunks, medium churn |
| Source of truth | Code (`catalog-info.yaml`) + connectors | Cloud/APIs (agentless scans), runtime | Text (repos, wikis, trackers) |
| Precision | High (declared) | High (discovered), but no business meaning | Variable (extraction quality) |
| Core query | Lookup, neighbourhood, impact, ownership | Path finding, combinations | Retrieval + entity-centric summarisation |
| Freshness need | Minutes | Minutes to hours | Hours |
| Who owns the truth | Teams (declared) | Nobody (observed) | Authors |

**Bridge entities** (where the graphs must join):

| Bridge | Catalog side | Security side | Knowledge side | Join key (external id) |
|---|---|---|---|---|
| Repository | `Repository` (+ owner) | Code scanning, Wiz Code | READMEs, ADRs, PR discussions | `github:repo/org/name` |
| Container image | (implicit) | Image vulns, layers, SBOM | — | image digest `oci:sha256:…` |
| Workload / deployment | `Deployment`/`Environment` | K8s workloads, VMs, functions | Runbooks | `k8s:workload/...`, cloud ARN |
| Service | `Service`/`Component` | Wiz *Service*; resource groups | Service docs | catalog ref + `wiz:service/...` |
| Team / person | `Group`/`User` | IAM principals, commit authors | Authors, assignees | IdP ids, emails, VCS logins |
| Cloud identity | (inventory tier) | Roles, service accounts | — | ARN / SA email |
| API | `API` (OpenAPI) | API gateway exposure | API docs | spec URL / gateway id |
| Datastore | `Resource` | DB, sensitive-data findings | Data docs | ARN / connection id |

## 5. Design: Toolbox's context lake

```
                         ┌──────────────── interfaces: API · MCP · CLI · UI (operator views) ────────────────┐
  GOLD (derived)         │ scores · findings (incl. imported toxic combos) · facts · context packs ·  │
                         │ embeddings · history (daily)                                                │
                         ├─────────────────────────────────────────────────────────────────────────────┤
  SILVER (modelled)      │ CATALOG tier (curated, owned)  ⇄ bridge edges ⇄  INVENTORY tier (discovered, │
                         │                                                   bounded kinds)            │
                         │ KNOWLEDGE layer: docs/chunks/incidents/ADRs linked to catalog entities       │
                         ├─────────────────────────────────────────────────────────────────────────────┤
  BRONZE (raw)           │ raw records: connector payloads, timestamps, selector hash, retention in code│
                         └─────────────────────────────────────────────────────────────────────────────┘
  FEDERATED (not copied):  security graph (Wiz, Cartography) · observability · code intelligence
                           → queried live via connector `resolve`, summarised into findings
```

### 5.1 Bronze: raw layer (already designed; now formalised)
- Every connector `RECORD` is stored with source, kind, extraction time, selector hash and payload
  hash. This is the same shape Port's lake uses.
- Uses:
  - re-map without re-fetching
  - `plan` previews of mapping changes
  - audit ("what did the source say on date X?")
  - backfills for new derived facts
- **Retention and facets are set in code.** Raw payloads are the *most* sensitive data in the system:
  full API responses, possibly containing secrets or PII. They are readable only by `operator` and
  connector owners via an `ops` facet. They are **never** used directly in context packs or embeddings.

### 5.2 Silver: two-tier graph
- **Catalog tier:** what we have today. Curated, owned and scored.
- **Inventory tier (new):** a bounded set of *discovered* kinds needed for bridging, such as
  `Workload`, `Image`, `CloudAccount`, `Datastore` and `CloudIdentity`. These are opt-in per portal,
  with per-kind caps.
  - Inventory entities have **no required owner**. Ownership is **inherited** through bridge edges
    (`Workload runs Service` → service owner), with provenance.
  - That inherited ownership is what security tools need.
- **Bridge edges** (`implementedBy`, `deployedAs`, `runsImage`, `buildsFrom`, `assumesIdentity`,
  `storesDataIn`) come from identity resolution and structural inference: image labels, CI provenance
  attestations, k8s annotations.
- **Temporal model** (Graphiti-inspired, kept simple): every edge and fragment carries `observed_from`
  / `observed_to` (validity) and `ingested_at`. Superseded facts are closed, not deleted.
  - This enables **"as-of" graph queries** for incident analysis ("what depended on X at 14:00
    yesterday?") on top of the daily history store.

### 5.3 Knowledge layer: catalog-anchored, not full GraphRAG
Full GraphRAG (LLM extraction over everything plus community detection) is expensive, must be re-run on
updates, and is only as good as its entity resolution. Toolbox has an advantage: **the catalog is a
high-precision entity dictionary, and its hierarchy (Domain → System → Component, teams) already
provides human-curated "communities".**

1. **Deterministic entity linking first.** Mentions of refs, aliases, external ids, repo paths, API
   operation ids and team handles in docs, ADRs, incidents and PRs are linked by dictionary matching
   (cheap, exact, explainable).
2. **Typed LLM extraction only for declared relation types**, validated against the schema. Examples:
   `ADR decides-for Service`, `Incident affected Service`, `Runbook remediates Alert`. Extraction is
   incremental per changed document, with provenance and confidence. Low confidence becomes a proposal,
   the same flow as identity proposals.
3. **"Global" questions use catalog communities.** Summaries per System, Domain and team replace Leiden
   clusters. They are regenerated when membership or linked documents change.
4. **Permissions are preserved** (same principle as Atlassian's Teamwork Graph). Documents carry their
   source ACLs as derived tuples ([red-team](../red-team.md) B6); chunks inherit them.

### 5.4 Gold: derived layer
- **Findings:** a new first-class kind of fact. Findings are imported from security tools (Wiz issues,
  toxic combinations, Cartography rule results) and attached to the bridge entities they concern, then
  propagated to owning services via bridge edges.
  - Finding details are a restricted facet: the scorecard summary-vs-details example made real.
- Scores, facts, history, embeddings (optional) and **context packs**. Packs now draw from all three
  graphs:
  - catalog (owners, dependencies)
  - inventory (where it runs, which images)
  - findings (critical risks, as a details facet)
  - knowledge (relevant ADRs, runbooks, incidents)

### 5.5 Federation, not replication
- **Code intelligence** stays federated. Toolbox doesn't build or ingest a code knowledge graph
  (Graphify and the tools like it): [knowledge-graphs.md](./knowledge-graphs.md) and
  [D01M84ZW6K2V9CV2SD644DE5M0S](../decisions/D01M84ZW6K2V9CV2SD644DE5M0S.md), after
  [graphify-ab](../../experiments/graphify-ab/README.md) showed neither help nor harm.
- **Security graph:** connector packages for Wiz (GraphQL) and Cartography (Neo4j/Cypher).
  - `source` imports *findings* and *bridge resources* (bounded).
  - `resolve` answers live, deep questions ("attack paths reaching `service:payments-api`") by
    translating to the tool's query language with the caller's identity mapped to a scoped token.
    Results are cached briefly and shown as a live panel.
- **Exporting ownership (strategic):** Toolbox exposes ownership in the formats security tools already
  ingest.
  - A **Backstage-compatible catalog API** (`/api/catalog/entities`), which Wiz's documented Backstage
    owner import can consume.
  - Push connectors that set owner/project tags in Wiz and Cartography.
  - Toolbox then becomes the **ownership system of record**: security, cost and observability tools
    route their issues to the owners the portal defines.

### 5.6 Querying the lake

| Need | Mechanism | Phase |
|---|---|---|
| Lookup, filter, neighbourhood, impact | Existing task-level tools (`get`, `query`, `graph`, `impact`, `context`) | 1 |
| Cross-graph rules ("toxic combinations" for engineering) | **Graph predicates in CEL** for scorecard rules: bounded traversal functions such as `entity.related('dependsOn', depth: 2)`, `entity.inventory('runsImage').findings('critical')`, `entity.path_exists(to: 'internet', via: [...], maxHops: 4)`. Cost-limited and statically analysed (feeds meta-catalog `reads` edges) | 2–3 |
| Ad-hoc graph questions by agents | **Read-only GQL/openCypher subset** (`graph_query` tool). GQL is ISO/IEC 39075:2024 and Cypher-based, and agents write Cypher fluently (Cartography, Neo4j). Compiled onto the in-memory graph with ReBAC filtering, cost limits and no writes | 4–5 |
| Text questions | Hybrid search + catalog-community summaries | 4 |
| As-of questions | Temporal edges + daily history | 3–4 |

**Example: an engineering "toxic combination" as a rule:**

```ts
rule("internet-exposed-critical-without-oncall", {
  level: "gold",
  check: `!(
     entity.spec.tier in ['0','1'] &&
     entity.inventory('deployedAs').exists(w, w.facts['exposure.internet'] == true) &&
     entity.findings(severity: 'critical').size() > 0 &&
     !has(entity.spec.oncall)
  )`,
  fix: "Assign on-call (PagerDuty) or remove public exposure; see linked findings",
  visibility: { details: "can_view_score_details" },
});
```

## 6. Implications for the plan

1. **§4.1 (catalog):** add the **inventory tier**, bridge edge types, temporal validity fields and
   inherited ownership.
2. **§4.2 (scorecards):** add **graph predicates** in CEL with cost limits; add **findings** as a fact
   type with severity and facets.
3. **§4.4 (knowledge):**
   - Replace "entity linking" with the catalog-anchored pipeline (dictionary linking + typed extraction
     + community summaries per System/Domain/team).
   - Raw layer never feeds packs or embeddings.
   - Third-party source ACLs carried as tuples.
4. **Connectors:**
   - Add **Wiz** and **Cartography** packages (source: findings + bridge resources; resolve: live paths).
   - Add a **Backstage-compatible catalog read API** so other tools can import ownership.
5. **MCP:** extend `context` packs with inventory and findings sections. Add `graph_query` (read-only
   GQL subset) later.
6. **Meta catalog:** the inventory-tier kinds and Findings sources become meta entities with owners
   (e.g. security owns the Wiz connector and finding mappings). The new scorecard `inventory-coverage`
   (percentage of workloads bridged to a service) is the security-to-catalog join health metric.
7. **Positioning:** say "context lake" descriptively (layered lake + graph + knowledge, open, self-hosted,
   permission-preserving). Differentiate on **open formats, code-defined semantics, and being the
   ownership system of record**, rather than on the term Port popularised.

## 7. Risks

| Risk | Mitigation |
|---|---|
| Inventory tier balloons into a second Wiz | Opt-in kinds, hard caps, bridge-only purpose; deep reasoning stays federated |
| Raw layer becomes a sensitive-data honeypot | Ops-only facet, short default retention, secret scanning on ingest, never embedded |
| LLM extraction noise pollutes the graph | Only declared relation types; confidence + proposals; provenance; per-document incremental re-extraction |
| Live federation latency and outages | `resolve` results cached and degraded gracefully; imported findings remain as the offline view |
| Permission mismatch between Toolbox and source systems (Wiz Projects, Confluence spaces) | Import source ACLs as tuples where APIs allow; otherwise owner-only default for imported details |

## 8. Phasing

| Phase | Scope |
|---|---|
| 1 | Raw layer formalised (facets, retention); temporal validity fields on fragments and edges |
| 2 | Findings fact type; Wiz + Cartography source connectors (findings + bridge resources); inventory tier for `Workload`/`Image`; graph predicates v1 in CEL |
| 3 | Bridge inference (image labels, provenance attestations, k8s annotations); inherited ownership; Backstage-compatible catalog read API; ownership push to security tools; as-of queries |
| 4 | Catalog-anchored knowledge pipeline (dictionary linking, typed extraction, community summaries); context packs with inventory + findings; `resolve` live attack-path panels |
| 5 | Read-only GQL/openCypher `graph_query` for agents |

## Sources

- Port:
  - [Context lake glossary](https://www.port.io/glossary/context-lake) ·
    [Context Lake docs](https://docs.port.io/context-lake/overview/) ·
    [Why AI agents need a context lake](https://www.port.io/blog/why-ai-agents-need-a-context-lake)
  - Ocean lakehouse client: [port-labs/ocean](https://github.com/port-labs/ocean)
    (`port_ocean/clients/port/mixins/integrations.py`, `core/models.py`, `config/settings.py`)
- Cortex: [Context Graph](https://www.cortex.io/products/context-graph) · [Agents](https://www.cortex.io/solutions/agents)
- Atlassian Teamwork Graph:
  [Nasdaq/Business Wire release (May 2026)](https://www.nasdaq.com/press-release/atlassian-opens-its-teamwork-graph-power-agentic-work-across-enterprise-2026-05-06) ·
  [Futurum analysis](https://futurumgroup.com/insights/atlassian-teamwork-graph-the-secret-weapon-thats-no-longer-a-secret/)
- Zep / Graphiti:
  [What is a context lake](https://www.getzep.com/ai-agents/what-is-a-context-lake/) ·
  [Temporal KG architecture (arXiv 2501.13956)](https://blog.getzep.com/zep-a-temporal-knowledge-graph-architecture-for-agent-memory)
- GraphRAG (secondary):
  [implementation guide](https://blog.premai.io/graphrag-implementation-guide-entity-extraction-query-routing-when-it-beats-vector-rag-2026)
- Wiz:
  - [AI security graph](https://www.wiz.io/academy/ai-security/ai-security-graph) ·
    [Attack path analysis](https://www.wiz.io/academy/attack-path-analysis) ·
    [Runtime signals in the Security Graph](https://www.wiz.io/blog/runtime-signals-in-security-graph)
  - [AskAI text-to-graph-query](https://www.wiz.io/blog/askai-text-to-security-graph-query) ·
    [Introducing Wiz Code](https://wiz.io/blog/introducing-wiz-code) ·
    [Code to cloud](https://www.wiz.io/blog/the-missing-link-in-code-to-cloud-visibility-is-finally-here)
  - [Wiz Service Catalog](https://wiz.io/blog/introducing-wiz-service-catalog) ·
    [Service Catalog updates](https://www.wiz.io/blog/wiz-service-catalog-updates) ·
    [Google completes acquisition](https://www.nasdaq.com/press-release/google-completes-acquisition-wiz-2026-03-11)
  - [PuppyGraph on Wiz's graph (secondary)](https://www.puppygraph.com/blog/wiz-security-graph)
- Cartography: [cartography-cncf/cartography](https://github.com/cartography-cncf/cartography) ·
  [CNCF project page](https://cncf.io/projects/cartography)
- GQL: [ISO/IEC 39075 (Wikipedia)](https://en.wikipedia.org/wiki/Graph_Query_Language) ·
  [Neo4j on GQL and Cypher](https://neo4j.com/blog/cypher-and-gql/gql-database-language-standard)
