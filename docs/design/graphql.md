# GraphQL for Rich Catalog Pages: Evaluation and Decision

> **Status 2026-11-20 (D43): deferred design.** Nothing in this document is in v1. The minimal UI uses composite REST view endpoints instead ([minimal-ui.md](./minimal-ui.md) §6). v1's UI uses a handful of REST view endpoints. This design returns if a non-first-party UI client appears or entity pages need more than three round trips at the p95 budget ([technology-critique.md](../technology-critique.md) §8.1).

*Status: research + decision, 2026-11-19. Related: [plan.md](../plan.md) §4.5 (interfaces),
[authorization.md](../authorization.md), [incremental-computation.md](./incremental-computation.md),
[dashboards.md](./dashboards.md), [developer-velocity.md](./developer-velocity.md).*

## 1. The question

Entity pages need one entity plus its neighbourhood: owner and on-call, system and domain, APIs
provided and consumed, dependencies and dependents, scorecard summaries, recent runs. All of it is
permission-shaped. Doing this with REST either means many round trips or an ever-growing `include=`
mini-language. Should Seikika expose GraphQL for this?

## 2. Verdict

**Yes, but as a read-only data layer for the UI and operator views, executed from trusted documents.
Not as an open-ended public query API, and not for agents.**

| Decision | Why |
|---|---|
| **Read-only schema** (queries only; no mutations, no subscriptions) | Writes already have one well-guarded path (actions with dry-run, approvals and audit; `update_entity` routing). The changefeed already uses SSE. A second write surface would duplicate semantics and risk |
| **Trusted documents (persisted operations) only, in production** | graphql.org's own security guidance recommends an allowlist of operations for first-party clients. A graphql-js maintainer argues "90+%" of GraphQL users should use trusted documents. This removes most GraphQL-specific attack and cost risk (arbitrary depth, aliases, batching) |
| **The documents come from code:** built-in UI pages and operator views declare their GraphQL queries, and `seikika build` validates, cost-analyses and registers them | One reviewable source; breaking schema changes fail the build, not production |
| **Ad-hoc GraphQL** in `seikika dev` (with GraphiQL) and, optionally, for authenticated *operators* in production behind cost limits | A fast authoring loop without opening production to arbitrary queries |
| **Agents keep the MCP task tools** (`get`, `graph`, `context`, …) and the later read-only GQL graph tool | Agents work best with task-level tools. A second query language on MCP would duplicate `graph` and weaken the token-efficient design |

**Why GraphQL rather than a custom view-query DSL:**
- **Agents and developers already write GraphQL fluently.**
- **Fragments colocate** a component's data needs with the component.
- **The client tooling exists:** typed code generation, normalised caches.
- It describes nested relation fetches exactly, which is what entity pages need.

With trusted documents it carries the safety properties of a fixed set of view endpoints.

## 3. Prior art

- **Backstage:** the core `@backstage/plugin-catalog-graphql` is **deprecated**. The community
  Frontside module serves `/api/graphql` with catalog types and an `@relation` directive. GraphQL over
  a catalog is useful, but it hasn't been core to Backstage's value, which argues for keeping it narrow.
- **GitHub GraphQL:**
  - every connection requires `first`/`last` between 1 and 100
  - a 500,000-node cap per call
  - a points-based rate limit derived from connection sizes
- **Shopify GraphQL:** per-field calculated cost, checked before and after execution, with a per-query
  cost cap and a leaky-bucket throttle.
- **IBM GraphQL Cost Directives** (`@cost`, `@listSize`): a schema-level cost vocabulary implemented by
  IBM API Connect, Apollo Router (demand control) and Cosmo Router.
- **Apollo MCP Server** turns persisted operations into MCP tools, and warns that its schema "isn't a
  security boundary": authorization must live in the backend. This supports keeping authorization in
  Seikika's service layer, not in GraphQL.

## 4. Performance

| Practice | Seikika implementation |
|---|---|
| **Avoid N+1 data access** | Resolvers read the circuit's **in-memory indexed views** (graph, entities, scores), so per-field data access is a memory lookup, not a database query |
| **Avoid N+1 authorization** | Authorization is the expensive part: an OpenFGA call per object. Resolvers collect object refs per depth level and call **batched** checks (BatchCheck, or ListObjects results cached per principal and log position): the DataLoader pattern applied to permissions and facets |
| **A consistent snapshot per query** | The whole operation executes at **one log position**, returned in `extensions.logPosition`. A page never mixes data from two moments |
| **Mandatory pagination** | Every list relation is a **Relay-style connection**: `first`/`after` required, `first ≤ 100` (the GitHub convention), and a per-operation node cap |
| **Static cost analysis** | The schema is generated with `@cost`/`@listSize` annotations. Each trusted document's worst-case cost is computed **at build time**, and documents over budget fail `seikika build`. Ad-hoc queries are costed before execution, with per-principal point budgets (GitHub/Shopify model) |
| **Depth and complexity limits** | `limit_depth` and `limit_complexity` are set explicitly. They are **off by default** in async-graphql, and its README "strongly recommends" them in production |
| **Caching** | Trusted documents allow `GET /graphql?doc=<hash>&variables=…` with an **ETag of `(log position, principal permission fingerprint)`**. `Cache-Control: private` only; responses are never shared across principals |
| **No speculative features** | No reliance on `@defer`/`@stream` or federation |

## 5. Security

| Risk | Control |
|---|---|
| **Authorization bypass or IDOR** | Resolvers call the **same service layer** as REST and MCP, which enforces ReBAC, facet shaping and information-flow labels ([authorization.md](../authorization.md) §4.4). There is no authorization logic in resolvers; graphql.org recommends enforcing it in the business-logic layer |
| **Facet leakage via schema shape** | **Every faceted field is nullable.** Restricted fields return `null` plus an error entry with `extensions: { code: "REDACTED", requires: "can_view_score_details", ask: [...] }`, matching the REST redaction marker. Uniform `null` is returned for invisible objects (`node(id)`), matching REST's 404 behaviour |
| **Counts and aggregates leaking** | `totalCount` is computed after permission filtering. Declassified aggregates follow minimum-cardinality rules |
| **Denial of service** (depth, aliases, batching, huge lists) | Trusted documents in production. For ad-hoc queries: cost, depth and alias limits, **no multi-operation batching**, connection caps, timeouts, response-size caps. These follow OWASP's GraphQL guidance (depth/amount limiting; preventing batching abuse) |
| **Schema discovery** | The schema is derived from config visible to portal members. Introspection is **enabled for authenticated users** (tooling needs it) and **disabled for unauthenticated requests**, along with field suggestions. OWASP lists introspection and GraphiQL as insecure defaults |
| **CSRF** (cookie sessions behind an identity-aware proxy) | Accept `POST` only with `application/json` (or the GraphQL-over-HTTP media type) and `GET` only for trusted-document ids with a required custom header. No simple-request side effects, and there are no mutations anyway |
| **Error information disclosure** | Masked errors with stable codes; internals stay in traces |
| **Observability** | Operation name and document hash on every trace, metric and analytics event |

## 6. Simplicity

- **No code generation into Rust.** The GraphQL schema is built **at runtime from the manifest** with
  async-graphql's dynamic schema (the `dynamic-schema` feature), consistent with the "no per-portal
  codegen" rule ([developer-velocity.md](./developer-velocity.md)). An SDL file is emitted at build
  for client code generation and review.
- **One query layer underneath.** REST `GET /v1/entities/{ref}?include=…`, GraphQL and MCP `get` all
  call the same internal read API. GraphQL is a projection, like the others.
- **Mapping from the model:**

  | Seikika concept | GraphQL shape |
  |---|---|
  | Kind | Object type implementing `Entity` |
  | Relation type | Connection fields in both directions |
  | Facet | Nullable field group |
  | Scorecard | `scorecards(names:)` field with summary/details sub-objects |

  Identifiers are global ids (entity refs).
- **Schema changes are reviewed:** `seikika plan` shows a GraphQL schema diff (breaking/dangerous/safe)
  alongside the OpenAPI and MCP diffs. Trusted documents are re-validated against the new schema at
  build.
- **Operators** write view queries as `.graphql` files or tagged templates next to their TSX views.
  `@seikika/ui-sdk` generates typed hooks. The same documents are snapshot-tested as two principals
  (Level-3 surface tests).

```graphql
# views/service-page.graphql (a trusted document, registered at build)
query ServicePage($ref: ID!, $deps: Int = 20) {
  entity(ref: $ref) {
    ... on Service {
      title
      owner { ref title notify { github } }
      system { ref title domain { ref title } }
      providesApis(first: 20) { nodes { ref title lifecycle } }
      dependsOn(first: $deps) { totalCount nodes { ref title owner { ref } } }
      scorecards(names: ["production-readiness", "security"]) {
        name level summary { passed failed }
        details { rules { id outcome remark } }   # nullable: REDACTED for non-owners
      }
    }
  }
}
```

## 7. Costs and cons (accepted)

- **Another API surface** to version and document. Mitigated by generating it from the same manifest
  and diffing it in `plan`.
- **Per-portal dynamic schemas** make generic third-party clients less useful. That's acceptable,
  because the audience is first-party UI and operator views.
- **GraphQL caching is subtler than REST's.** Mitigated by trusted documents with `GET`, ETags and
  private caching.
- **The team must hold the security discipline** (nullable facets, batched authorization, limits).
  Enforced by the Level-3 surface tests and the S1 non-interference simulation checks, which now
  include GraphQL.

## 8. Changes to the plan

1. **Crate `seikika-graphql`:** async-graphql dynamic schema from the manifest; read-only; batched
   authorization and facet loaders; cost directives; trusted-document registry; GET with ETags.
2. **Operator views** declare GraphQL documents. `seikika build` validates and costs them, and plan
   shows schema diffs.
3. **Verification:** add GraphQL to S1 (non-interference) and E6 (surface equivalence); fuzz the
   GraphQL endpoint (L4); cost-regression checks on the built-in documents.
4. **Phasing:**
   - **Phase 1:** the internal read API plus REST `include` for the minimal UI.
   - **Phase 2:** GraphQL with trusted documents for the built-in entity pages and operator views.
   - **Later:** optional operator-only ad-hoc GraphQL in production, if demand appears.

## Sources

- [OWASP GraphQL Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/GraphQL_Cheat_Sheet.html)
- graphql.org:
  [Security (trusted documents)](https://www.graphql.org/learn/security) ·
  [Authorization](https://graphql.org/learn/authorization/) ·
  [Best practices](https://graphql.org/learn/best-practices) ·
  [Trusted documents (GraphQL golden path)](https://goldenpath.graphql.org/practices/trusted-documents/)
- [GraphQL over HTTP (draft)](https://graphql.github.io/graphql-over-http/draft)
- Cost and limits:
  [IBM GraphQL Cost Directives](https://ibm.github.io/graphql-specs/cost-spec.html) ·
  [Apollo demand control](https://www.apollographql.com/docs/graphos/routing/security/demand-control) ·
  [GitHub GraphQL rate and node limits](https://docs.github.com/en/enterprise-server@3.15/graphql/overview/rate-limits-and-node-limits-for-the-graphql-api) ·
  [Shopify GraphQL rate limits](https://shopify.dev/docs/apps/build/apis/graphql-admin/rate-limits) ·
  [Shopify: calculating query complexity](https://shopify.engineering/rate-limiting-graphql-apis-calculating-query-complexity)
- async-graphql:
  [repo](https://github.com/async-graphql/async-graphql) ·
  [depth and complexity](https://async-graphql.github.io/async-graphql/en/depth_and_complexity.html)
- Backstage:
  [@backstage/plugin-catalog-graphql (deprecated)](https://www.npmjs.com/package/@backstage/plugin-catalog-graphql) ·
  [Frontside catalog module](https://www.npmjs.com/package/@frontside/backstage-plugin-graphql-backend-module-catalog)
- [Apollo MCP Server: define tools](https://www.apollographql.com/docs/apollo-mcp-server/define-tools)
