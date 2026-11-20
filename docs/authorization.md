# Toolbox Authorization — Relationship-Based Access Control on the Catalog Graph

> **Implemented (M3):** OpenFGA v1.22.1 runs as a supervised co-process reading derived tuples through the Postgres view; Toolbox converts the DSL itself (D59); facets follow D60; `toolbox test` runs the `.fga.yaml` corpus. See [v1-spec.md](./v1-spec.md) §9.
>
> **Status 2026-11-20:** §4.3 updated for D38 (OpenFGA inside the container, co-process by default, reading derived tuples through a Postgres view). The rest of the design stands; the reference model and tests are in [examples/reference-portal](../examples/reference-portal/README.md).

*Research date 2026-11-18, updated 2026-11-19 (embedded OpenFGA, information flow, CODEOWNERS
authority, proxy authentication). Primary sources:*
- *The OpenFGA docs repo (`openfga/openfga.dev`: testing, store file format, conditions, modular
  models, agent/RAG guides).*
- *The `openfga/language` repo (grammar and conformance data).*
- *The OpenFGA server's test corpus (`assets/tests/*.yaml`).*
- *The `zvectorlabs/authz-core` source.*

*Other projects are covered from their docs via search excerpts. Repository stats come from GitHub
search on the same date. Companion to [plan.md](./plan.md) §5.*

---

## 1. TL;DR

**Decisions (updated 2026-11-19):**
- **Language and tests:** adopt OpenFGA's modelling language, semantics and test file format
  (`.fga.yaml`) as Toolbox's authorization contract.
- **Evaluator:** **OpenFGA itself, embedded as a Go module** and linked into the Rust binary through a
  C ABI (§4.3). This replaces the earlier plan to write our own evaluator. There is no separate
  OpenFGA service, and **no tuple writes anywhere**. A custom read-only `OpenFGADatastore` serves
  tuples straight from the incremental circuit's output index
  ([incremental-computation.md](./design/incremental-computation.md)).
- **Information flow:** explicit facets, taint propagation and reviewed declassification (§4.2), so
  "summary public, details private" is principled rather than accidental.
- **Authority:** ownership and other permission-relevant claims are accepted only when backed by
  **CODEOWNERS** (§4.8).
- **Authentication:** delegated to the organisation's **identity-aware proxy** (Google IAP, Cloudflare
  Access and similar) or OIDC. **Toolbox owns the permissions boundary.** Revocation has an SLO (§4.9).

This gives you:
1. **The OpenFGA testing story, unchanged and exact.** The same engine runs in `fga model test`, in
   `toolbox test`, and in production.
2. **No synchronization problem.** Tuples are *derived views* of catalog data, read in-process by
   OpenFGA. "Ownership changed" and "access changed" are the same log position.
3. **Field-level visibility.** Facets are guarded by permissions, and a single response-shaping layer
   is shared by API, MCP, CLI, UI, search, context packs and the changefeed.
4. **One predicate language.** OpenFGA conditions are CEL, as are Toolbox's rules and mappings.

---

## 2. Requirements

| # | Requirement | Why |
|---|---|---|
| R1 | Authorization decisions use the **portal's own graph**: every portal has different kinds and relations | Ownership, team membership and system hierarchy already live in the catalog |
| R2 | **Facet-level** visibility on the same entity | Scorecard summary for everyone; details for owners |
| R3 | **Identical enforcement** across API, MCP, CLI, UI, search, context packs, changefeed/webhooks | Agents must not see more via MCP than humans see in the UI, or vice versa |
| R4 | **Testable from a model plus fixtures**, OpenFGA-style, in CI | User requirement; agents can author and verify permission changes |
| R5 | Configuration in code; **no runtime grant API** | Project non-goal: config is never stored in the database |
| R6 | Agents as principals, **acting on behalf of users**, with their own ceilings | Agent-first product |
| R7 | Fast list filtering at 100k entities and 10k users | Catalog lists, search and context packs filter large sets |
| R8 | Explainability: "why can't I see this, and who can grant it?" | Humans and agents need actionable denials |
| R9 | Replaceable / interoperable | Same stance as observability: open standards at the boundary |

---

## 3. Landscape

| Project | Lang / license | Stars | Model and test story | Fit for Toolbox |
|---|---|---|---|---|
| **OpenFGA** | Go, Apache-2.0, **CNCF Incubating** (Oct/Nov 2025) | ~6.2k | Zanzibar-style DSL (`type`/`relations`/`define`, `X from Y`, `or`/`and`/`but not`), **CEL conditions**, **modular models** (`fga.mod`, `module`, `extend type`), contextual tuples. **`.fga.yaml` store files**: `model`/`model_file`, `tuples`/`tuple_file(s)`, `tests` with `check`, `list_objects`, `list_users` assertions, run by `fga model test`, with a VS Code extension and a GitHub Action. | **Adopt the language and test format.** Running the server means syncing tuples from the catalog, an extra service, and ListObjects capped at 1,000 by default. |
| **SpiceDB** (AuthZed) | Go, Apache-2.0 | ~7.2k | Schema language with `definition`/`relation`/`permission`, CEL caveats. `zed validate` files with `schema`, `relationships`, `assertions` (`assertTrue`/`assertFalse`) and exhaustive `validation` (expected relations). | Strong engine. Its own docs say LookupResources degrades significantly beyond 10k results and suggest the commercial Materialize. Same sync problem. A different language from OpenFGA's. |
| **Permify** | Go, **AGPL-3.0** | ~6.0k | Schema DSL; validation YAML with `scenarios` (`checks`, `entity_filters`, `subject_filters`), plus `permify coverage` | Good testing ideas (scenarios, coverage). The AGPL license is a problem for an embeddable dependency. |
| **Ory Keto** | Go, Apache-2.0 | ~5.4k | Ory Permission Language (TypeScript-like) | Active but less momentum; no comparable test format. |
| **Topaz** (Aserto) | Go, Apache-2.0 | ~1.4k | OPA/Rego + a ReBAC directory | Future less certain per secondary sources; two languages. |
| **Cedar** (AWS) | **Rust**, Apache-2.0 | ~1.9k | Policies over an entity hierarchy (`principal in Group::"x"`); formally verified; validator | Embeddable and Rust, but relationships must be *supplied as entity data* per request. There is no list-objects primitive and no graph traversal beyond ancestry. Weak for "derive access from the catalog graph". |
| **Cerbos** | Go, Apache-2.0 | ~4.7k | YAML policies + CEL; policy test suites | A policy decision point for ABAC rather than a relationship engine. |
| **Oso** (open-source library) | Rust, Apache-2.0 | ~3.5k | Polar | **Repository archived**; Oso Cloud is the product. Not viable. |
| **authz-core** (zvectorlabs) | **Rust**, Apache-2.0 | 3 | OpenFGA-*like* DSL (braces, `permissions` with `+`), CEL conditions, datastore traits, `CoreResolver` | Closest Rust engine: v0.1.0 (Mar 2026), ~4.7k lines, single release. Its syntax is **not** OpenFGA-compatible. Useful reference, not a dependency. |
| Other Rust crates (`zanzibar`, `simple-zanzibar`, `fga-rs`) | Rust | small | Own DSLs | Hobby or early-stage; not OpenFGA-compatible. |
| **AuthZEN Authorization API 1.0** | OpenID Foundation standard, **Final Jan 2026** | — | Standard PDP HTTP API: evaluation, batched evaluations, subject/resource/action search | **Adopt at the boundary** so external tools can query Toolbox, and Toolbox can delegate to an external PDP. |

**Why OpenFGA's language rather than SpiceDB's:**
- The user prefers OpenFGA's testing story.
- It is a CNCF project with Apache-2.0 tooling and multi-language parsers.
- It uses CEL, our expression language.
- It has modular models, which match portal and connector packages.
- It has a public conformance corpus (§4.3).

---

## 4. Design

### 4.1 Where relationships come from: derived, not written

```
 input tables (raw records, native values) ──► change log ──► DBSP circuit
        │  derivation rules (portal program, build-time validated) are circuit operators
        ▼
 authz tuples = an indexed circuit OUTPUT VIEW (in memory, per log position; never written to a table)
        │  + static tuples in code (portal-wide roles, break-glass)
        │  + contextual tuples from the request identity (e.g. IdP group claims)
        ▼
 embedded OpenFGA (Go module, in-process) reads tuples via a read-only datastore → decisions, lists, explain
```

Three sources of tuples, and **none of them is a runtime "grant" API** (R5):

| Source | Example | Defined in |
|---|---|---|
| **Derived** from catalog data | `group:payments` is `owner` of `service:payments-api` (from `spec.owner`); `user:anne` is `member` of `group:payments` (from the IdP/HR connector) | Derivation rules in the portal program |
| **Static** | `group:security#member` is `security` on `portal:acme` | Portal program, reviewed in PRs |
| **Contextual** | `user:anne` is `member` of `group:oncall` because her OIDC token says so | Token-claim mapping in code |

Access changes when the catalog changes (ownership moves via a `catalog-info.yaml` PR, someone joins a
team in the IdP) or when code changes. Both paths are audited, and both are visible to `toolbox plan`.
Just-in-time or temporary grants are an open question (§8). If added, they would be time-bound *state*
with an expiry, created through an approved action, never a config bypass.

Derivation in the portal program (TypeScript shown; every SDK has the same shape):

```ts
const authz = new tb.Authz("default", {
  model: "authz/fga.mod",                           // OpenFGA modular model: authz/*.fga
});
authz.derive(service, "owner",  { from: "spec.owner" });            // entity ref → tuple
authz.derive(service, "portal", { constant: "portal:acme" });
authz.derive(bs.Group, "member", { from: "relations.hasMember" });   // users and nested groups
authz.derive(bs.Component, "system", { from: "relations.partOf", targetKind: bs.System });
authz.contextual({ claim: "groups", as: "member", objectType: "group" });  // OIDC claims → tuples
```

`toolbox build` validates the derivation rules against the model:
- Every derived relation exists on the target type.
- Its type restrictions (`[group]`, `[user, group#member]`) admit the derived subjects.
- Kinds map to model types (`Service` → `service`).

Violations are build errors, not runtime surprises.

**Modules come from packages.** Using OpenFGA's modular models, a connector package can contribute an
`.fga` module. For example, the PagerDuty package could `extend type service` with
`define oncall: [user]` and ship the matching derivation rule. The portal's `fga.mod` lists the
modules it accepts.

### 4.2 Facets, information flow and declassification

Most catalogs are **public by default**, and Toolbox keeps that default: the shipped model grants
`can_view` to every portal member. Facets exist for the data that genuinely needs protection.

Kinds, facts and scorecards declare **facets**: named groups of fields, each guarded by a permission
in the model.

**Information-flow rules** (enforced by the circuit, [incremental-computation.md](./design/incremental-computation.md) §3):
- Every circuit column carries a **facet label**.
- A derived value (rule outcome, `calc`/`mirror`/`aggregate` property, roll-up, `explain` text,
  search count) takes the **join of its inputs' labels** by default.
- **Declassification** lowers a label only where the program says so, explicitly. Each
  declassification is listed by `toolbox plan` and reviewed like any other change.
- **Minimum cardinality:** an aggregate that is declassified over restricted inputs is suppressed when
  it covers fewer than `k` entities (configurable, default 3). Otherwise a roll-up of one service
  reveals that service's details.

The scorecard example from the requirements:

```ts
new tb.Scorecard("security", {
  appliesTo: service,
  visibility: {
    summary: "can_view_score_summary",   // level, colour, pass/fail counts, trend
    details: "can_view_score_details",   // rule-level outcomes, observed values, remarks, issue lists
  },
  declassify: { outcome: "summary" },    // explicit: rule outcomes computed from details are visible as summary
  // ...rules
});
```

```ts
new tb.Kind("Service", {
  // ...
  facets: { default: "can_view", oncall: "can_view", costs: "can_view_costs" },
});
// Facts from connectors carry a facet: snyk.critical_issues → facet "score_details"
```

Different scorecards can use different permissions. The documentation-quality scorecard might expose
details to everyone, while the security scorecard restricts them.

**Response shaping is one layer, shared by every surface:**
1. The handler loads the entity and computes the set of facets the principal may read. This is one
   `BatchCheck` over the facet permissions of the object.
2. A single serializer removes disallowed facets *before* any surface formats the output. REST JSON,
   MCP Markdown, CLI tables and UI pages all go through it. The CLI and UI are API clients, so they
   cannot bypass it.
3. Redacted facets leave a marker by default, so humans and agents know the data exists and whom to
   ask:
   ```json
   "scorecards": { "security": { "level": "yellow", "passed": 6, "failed": 2,
     "details": { "redacted": true, "requires": "can_view_score_details",
                  "ask": ["group:payments"] } } }
   ```
   Portals that must hide even the *existence* of a facet can switch markers off per facet.

### 4.3 The evaluator: unmodified OpenFGA inside the container (v1)

**v1 (D38, Q11 default):** the Toolbox image contains the pinned, unmodified OpenFGA server binary,
started and supervised by `toolbox serve` as a co-process on localhost, using its Postgres datastore.
OpenFGA's `tuple` table is replaced by a **view over `toolbox.derived_tuples`**, which the derive loop
publishes as deltas. OpenFGA never receives writes, so tuples exist once, as derived data.

| Property | v1 |
|---|---|
| Semantics | Exactly OpenFGA's (it *is* OpenFGA); the reference portal's `.fga.yaml` tests run against the derived tuples |
| Measured | 2,000/2,000 checks agree with a SQL permission view; Check p50 2.4 ms over HTTP; an ownership change visible after a 2.6 s refresh ([technology-critique.md](./technology-critique.md) §7.2) |
| Consistency | Decisions use the latest *published* tuples. A response computed at snapshot N may be authorised against N+1 tuples, never older than the snapshot being served (newer only tightens revocation) |
| Contract | `Authorizer` (v1-spec §7.1): an external OpenFGA, or the linked variant below, can replace the co-process |
| Not yet tested over the view | Conditions, wildcards beyond `user:*`, `ListUsers` at scale (Phase 0 spike) |

#### Alternative kept for Q11: OpenFGA linked in-process as a Go module

OpenFGA is designed to be embeddable:
- `server.NewServerWithOpts(server.WithDatastore(ds))` builds an in-process server.
- `(*Server).Check`, `ListObjects`, `ListUsers` and `Expand` are ordinary Go methods.
- The datastore is a public interface, `storage.OpenFGADatastore`, with `RelationshipTupleReader`
  methods (`Read`, `ReadPage`, `ReadUserTuple`, `ReadUsersetTuples`, `ReadStartingWithUser`, …).
- Verified against `openfga/openfga` `main` (Go 1.26 module).

**Integration:**
1. **`toolbox-fga` (Go)** wraps the OpenFGA server and implements a **read-only `OpenFGADatastore`**.
   - Tuple reads call back into Rust through exported C functions, which read the derived tuple
     index at the request's snapshot.
   - Model reads return the compiled model from the manifest.
   - All write methods return "unsupported".
   - The optional typed `Querier` surface returns `nil`, which the interface permits.
2. **Built as a `c-archive`** and linked into the Rust binary, so Toolbox stays a single static binary.
3. **The C ABI is batched to amortise cgo overhead:**
   - Requests are sent as protobuf bytes.
   - Callbacks return pages of tuples rather than one tuple per call. Published measurements put a
     cgo call in the tens of nanoseconds.
   - No Go pointers cross the boundary, and memory is freed explicitly per cgo rules.
4. **Pinned version.** The OpenFGA module version is pinned in `toolbox.lock` and upgraded deliberately
   with its own conformance run.

**Consequences:**
- **Exact semantics.** OpenFGA's own conformance corpus (`assets/tests`, ~1,200 checks) now tests *our
  datastore adapter*, not a re-implementation. `fga model test` in portal CI uses the same engine as
  production.
- **No duplicate writes.** OpenFGA never stores tuples. They exist once, as a derived view.
- **Consistency.** Every decision is evaluated at a snapshot, returned as `X-Toolbox-Snapshot`;
  `min_snapshot` gives read-your-writes across instances.
- **List filtering.** ListObjects and ListUsers limits are configured for in-process use. Toolbox
  caches *results* per (principal, permission, snapshot), with LRU bounds. Unconditional hot
  permissions can also be maintained in the derived state as `principal → objects` views; that path doesn't
  apply to conditional or contextual tuples.
- **Explain** uses OpenFGA `Expand` plus derivation provenance: which source fact produced each tuple.

**Risks and mitigations:**

| Risk | Mitigation |
|---|---|
| **Go runtime in a Rust process:** GC, its own threads, signal handlers | Isolate the boundary in `toolbox-fga`; stress-test signals and shutdown in CI |
| **Determinism under simulation** | Results are pure functions of the datastore reads, which come from the deterministic circuit. Go-internal scheduling affects ordering, not answers. The simulator runs with `GOMAXPROCS=1` and treats calls as synchronous. If strict replay is needed, a `GOOS=wasip1` build of the same module runs under wasmtime |
| **API churn** in `pkg/server`/`pkg/storage` | Pin; upgrade with the conformance corpus; contribute an "embedded datastore" example upstream |
| **Build needs a Go toolchain** | Hermetic build images; prebuilt `toolbox-fga` archives per target |

### 4.4 Enforcement points (one decision engine, every surface)

| Surface | What is checked | How |
|---|---|---|
| REST API | Object permission + facets on every read; action permissions on writes | Middleware resolves the principal; handlers declare `(object, permission)`; the shared serializer shapes responses |
| MCP | Same as API. Tools are filtered from `tools/list` when the principal can't use them at all | MCP handlers call the same service layer. Context packs are built only from allowed facets, and token budgets are computed *after* redaction |
| CLI | Via API | No local enforcement logic to diverge |
| UI | Via API | Hidden facets render as "restricted, owned by X, request access" |
| Catalog lists and queries | `ListObjects` (cached per log position) or circuit permission views ∩ query results | Counts and pagination are computed after filtering, so totals don't leak |
| Search | Per-facet index documents with object ids; results ∩ the principal's accessible set | Snippets, highlights and facet counts come only from allowed documents. Detail text is indexed in a separate facet document, so it can never rank or snippet for an unauthorized user |
| Changefeed / SSE / webhooks | Each event is checked against the subscriber principal; facet fields are stripped | Outbound webhooks run as a declared service principal |
| Actions | `can_run` at request time **and again at execution**. Approvers = `ListUsers(run, can_approve)` | Agents acting on behalf of a user get the intersection of both principals (§4.5) |
| AuthZEN endpoint | Other systems ask Toolbox for decisions | `POST /access/v1/evaluation(s)` and the search endpoints, backed by the same engine |

**Side channels are handled deliberately:**
- Forbidden and nonexistent objects both return `404` unless the principal can at least `can_view` the
  object.
- Aggregate endpoints (counts by owner, scorecard roll-ups) aggregate only over visible objects, or
  over summary facets.

### 4.5 Agents as principals

The OpenFGA docs include agent-specific patterns: agents as principals, task-based authorization, MCP
authorization, and RAG filtering with pre-filter (ListObjects) and post-filter (BatchCheck). Toolbox
applies them as follows:
- Every MCP session has a **user** (from OAuth) and an **agent** (the registered client, e.g.
  `agent:claude-code`).
- **Effective permission = user permission ∧ agent ceiling.** The ceiling is defined in the model, e.g.
  `define can_run_destructive: [agent]` granted only to specific agents in code.
- **Prompt-injection defence is delegated to other layers.** That means the MCP client's approval UI,
  agent-harness permissions, network egress controls and the identity-aware proxy. Toolbox does not
  taint sessions. It provides accurate MCP tool annotations (`readOnlyHint`, `destructiveHint`,
  `idempotentHint`, `openWorldHint`), marks untrusted text in responses, enforces approval policies on
  actions, and audits everything.
- A headless agent with a service token is just an `agent` principal, with no user.
- The audit log and analytics record both principals.
- Context packs and search follow the RAG pre-filter pattern via the principal's accessible set, so
  an agent never receives a chunk it could not have opened directly.

### 4.6 Testing: OpenFGA's story, extended to the catalog

Portal repo layout:

```
authz/
├── fga.mod                     # modular model manifest (stock OpenFGA format: schema "1.2", contents: [...])
├── core.fga                    # module core (above)
├── pagerduty.fga               # contributed by a connector package (vendored, pinned)
├── scorecards.fga.yaml         # stock OpenFGA store-file tests: model + tuples + assertions
└── scorecards.catalog.yaml     # Toolbox catalog fixtures → derived tuples (tests the derivation too)
```

**Level 1: pure model tests (100% stock OpenFGA format).** These run with `toolbox test` *and* with
`fga model test --tests "authz/*.fga.yaml"`.
Test files sit beside `fga.mod`, because the `fga` CLI refuses `../` model references.

*Verified: this exact model and test file pass with the OpenFGA CLI (`fga model test`: 6/6 checks,
2/2 ListObjects, 1/1 ListUsers). Flipping one expectation makes it fail as expected.*

```yaml
name: Scorecard visibility
model_file: ./fga.mod
tuples:
  - { user: "user:*",                  relation: member,   object: "portal:acme" }
  - { user: "group:security#member",   relation: security, object: "portal:acme" }
  - { user: "user:anne",               relation: member,   object: "group:payments" }
  - { user: "user:bob",                relation: member,   object: "group:search" }
  - { user: "user:sam",                relation: member,   object: "group:security" }
  - { user: "portal:acme",             relation: portal,   object: "service:payments-api" }
  - { user: "group:payments",          relation: owner,    object: "service:payments-api" }
tests:
  - name: summary is public, details are owner + security only
    check:
      - user: user:anne
        object: service:payments-api
        assertions: { can_view_score_summary: true, can_view_score_details: true }
      - user: user:bob
        object: service:payments-api
        assertions: { can_view_score_summary: true, can_view_score_details: false }
      - user: user:sam
        object: service:payments-api
        assertions: { can_view_score_summary: true, can_view_score_details: true }
    list_objects:
      - user: user:bob
        type: service
        assertions:
          can_view_score_details: []
          can_view_score_summary: [service:payments-api]
    list_users:
      - object: service:payments-api
        user_filter: [{ type: user }]
        assertions:
          can_view_score_details: { users: [user:anne, user:sam] }
```

**Level 2: derivation tests (Toolbox).** Catalog fixtures (entities in `catalog-info.yaml` format)
are run through the portal's real derivation rules, and the same assertions are checked. This catches
the most common real bug: the model is right, but `spec.owner` was mapped to the wrong relation.

```yaml
# scorecards.catalog.yaml
entities:
  - { kind: Group, metadata: { name: payments }, spec: { members: [user:anne] } }
  - { kind: Service, metadata: { name: payments-api }, spec: { owner: group:payments, tier: "1" } }
assertions_from: scorecards.fga.yaml   # reuse the check/list assertions above
```

**Level 3: surface tests (Toolbox).** These assert what each surface actually returns for a principal.
They are snapshot-based, so redaction regressions show up in review:

```yaml
- as: user:bob
  mcp: { tool: get, args: { ref: "service:default/payments-api" } }
  expect: { contains: "level: yellow", redacted: ["scorecards.security.details"] }
```

**`toolbox plan` adds an authorization diff.** Against a production snapshot, it reports the effect of
the change, for example: "+12 users gain `can_view_score_details` on 3 services; 1,240 users lose
`can_run_actions` on `system:checkout`". It is computed by ListUsers before and after, and reported
as a PR comment. Permission changes become reviewable by impact, not just by diff.

**Coverage**, borrowed from `permify coverage`: report every relation and permission that no
assertion touches.

**Same engine everywhere:** Level-1 files run unchanged under the stock `fga model test` and
`toolbox test`, because both use OpenFGA. Toolbox's added value is Levels 2 and 3: derivation from
catalog data, and what each surface actually returns.

### 4.7 Interop and replaceability

- **AuthZEN 1.0 PDP endpoint:** other tools can ask Toolbox "can X do Y on Z?" using the standard API.
  Examples: an API gateway, an MCP gateway, or an internal admin tool that wants to reuse
  catalog-derived permissions.
- **`toolbox authz export --fga`** writes a standard OpenFGA store file (model plus derived tuples from
  a snapshot) for offline analysis in the OpenFGA tooling.
- *Removed:* the "external OpenFGA service" mode. Streaming tuples to a central OpenFGA would
  reintroduce duplicate writes. Organisations with a central OpenFGA can call Toolbox's AuthZEN
  endpoint instead.

### 4.8 Authority of permission-relevant claims (CODEOWNERS)

Permission-relevant fields (`spec.owner`, `system`/`partOf` links that confer access, external-id
annotations) are accepted from git only when **CODEOWNERS** backs them:
- **Ownership claim:** a `catalog-info.yaml` may set `spec.owner: group:X` only if team X (mapped from
  its GitHub team via external ids) is a code owner of that file's path in the repo's CODEOWNERS.
  Otherwise the claim is recorded but **not applied**: the entity gets an `ownership: unverified`
  status, and the `ownership` scorecard fails at bronze.
- **External-id claims** (e.g. `pagerduty.com/service-id`) are applied only from files whose code
  owners include the entity's owner. A conflicting claim from another repo **freezes both claims**
  until resolved. First claim does not win.
- **Central data** (the catalog-data repo, native fields) is governed by that repo's CODEOWNERS and by
  ReBAC write permissions respectively.
- `toolbox plan` and the GitHub check report unverified claims on the PR that introduces them.

### 4.9 Authentication, identity-aware proxies and revocation

**Toolbox authenticates through what the organisation already runs, and owns the permissions
boundary.**

| Mode | How | Notes |
|---|---|---|
| **Identity-aware proxy** (recommended for the UI and API) | Google **IAP** (`x-goog-iap-jwt-assertion`), Cloudflare **Access** (`Cf-Access-Jwt-Assertion`, per-application AUD), or generic JWKS-verified headers for Pomerium, oauth2-proxy and similar | Toolbox **always verifies the signed JWT** (signature, `aud`, issuer, expiry). Google notes signed headers protect against requests that bypass the proxy. Identity maps to a `User` entity by external id or email |
| **OIDC** (built-in) | Standard authorization-code flow | For deployments without a proxy |
| **Agents / MCP** | MCP OAuth 2.1, PATs, or the proxy's programmatic access (IAP service-account tokens, Access service tokens) | Browser-only proxy flows don't suit agents, so the MCP endpoint may use a separate hostname or path with OAuth |
| **CI** | GitHub OIDC tokens | Repo-scoped principals |

**Revocation SLO.** The default is 60 s from an IdP event to enforcement:
- The proxy revokes *sessions* at the edge, but team-based *permissions* come from the IdP connector.
- **Fast path:** SCIM deprovisioning and IdP group-change webhooks are written to the input tables
  immediately, flow through the change log, and update the circuit's membership views. Cached
  permission results are keyed by log position, so they go stale automatically.
- Group claims in the proxy or OIDC token are used as a **ceiling**: a membership absent from a fresh
  token is treated as revoked even before the connector catches up.
- Revocation latency is a published metric and alert ([plan.md](./plan.md) §6).

---

## 5. Changes to the plan

1. **Replace Cedar with the OpenFGA model + CEL conditions** as the single authorization language.
2. **`toolbox-authz` + `toolbox-fga`:**
   - **`toolbox-fga`** is the Go module: embedded OpenFGA with a read-only datastore over the circuit
     index.
   - **`toolbox-authz`** is the Rust side: principals, proxy/OIDC/PAT authentication, facet shaping,
     result caches, explain, AuthZEN, CODEOWNERS authority checks.
3. Kinds, facts and scorecards gain **facets with information-flow labels and explicit
   declassification**. A single response-shaping layer is mandatory for all surfaces.
4. `toolbox test` runs Level 1–3 authorization tests. `toolbox plan` reports authorization impact and
   declassifications.
5. Ship a default module (`@toolbox/authz-default`) that is **public by default**: portal membership,
   ownership, security and admin roles, plus facet conventions.

## 6. Phasing

| Phase | Authorization scope |
|---|---|
| 0 (spike) | `toolbox-fga` c-archive linked into Rust; read-only datastore over an in-memory tuple index; OpenFGA conformance corpus green through the adapter; cgo overhead and signal behaviour measured |
| 1 (catalog) | Derivation rules as circuit operators; facets + information-flow labels + response shaping; CODEOWNERS authority; proxy/OIDC authentication; revocation fast path; Level 1–2 tests |
| 2 (scorecards) | Scorecard summary/detail facets with declassification and minimum cardinality; facts with facets; `plan` authz diff |
| 3 (actions) | `can_run` at request and execution; approvers via ListUsers; agent ceilings |
| 4 (knowledge/search) | Per-facet search documents, filtered search and context packs |
| 5 (ecosystem) | AuthZEN endpoint; connector-contributed modules; `authz export` |
| 6 (hardening) | Continuous conformance + fuzzing of the datastore adapter; side-channel review; Level-3 surface snapshot suite |

## 7. Risks

| Risk | Mitigation |
|---|---|
| FFI boundary bugs (memory, panics across the boundary, signals) | Narrow C ABI with byte buffers; fuzz the ABI; panic and recover guards on both sides; CI stress tests |
| Authorization data leaks through side channels (counts, search snippets, timing, 403 vs 404, changefeed) | Information-flow labels; explicit declassification with minimum cardinality; per-facet search documents; uniform 404s; Level-3 surface tests; a dedicated security review |
| Model complexity for portal authors | Public-by-default module; facets as a simple declarative concept; agent skill for "add a permission + tests"; `explain` |
| Stale permissions after IdP changes | Revocation SLO with the SCIM/webhook fast path and token claims as a ceiling |

## 8. Open questions

1. **Temporary or just-in-time grants:** allow time-bound grant *state* created only via an approved
   action, or keep all grants derived from the catalog and code?
2. **Redaction markers on by default** (helps agents ask for access), or off by default (hides that
   the data exists)?
3. Should agent ceilings live in the authorization model (type `agent`) or in a separate, simpler
   allowlist in the portal program?

## Sources

- OpenFGA:
  - [Testing models](https://openfga.dev/docs/modeling/testing) · [Conditions (CEL)](https://openfga.dev/docs/modeling/conditions) ·
    [Modular models](https://openfga.dev/docs/modeling/modular-models)
  - [RAG authorization](https://openfga.dev/docs/modeling/agents/rag-authorization) ·
    [CNCF incubation announcement](https://www.cncf.io/blog/2025/11/11/openfga-becomes-a-cncf-incubating-project/) ·
    [openfga/language](https://github.com/openfga/language)
  - [Server test corpus](https://github.com/openfga/openfga/tree/main/assets/tests) ·
    [Continuous authorization testing (Auth0 blog)](https://auth0.com/blog/continuous-authorization-testing-fga-github-ci-cd/)
  - Embedding API: `pkg/server` (`NewServerWithOpts`, `WithDatastore`) and `pkg/storage`
    (`OpenFGADatastore`, `RelationshipTupleReader`) in [openfga/openfga](https://github.com/openfga/openfga)
- Go/Rust FFI: [embedding Go in a Rust program](https://engineered.at/articles/embedding-go-in-a-rust-program) ·
  [rust2go](https://github.com/ShabbirHasan1/rust2go)
- Identity-aware proxies:
  [Google IAP signed headers](https://docs.cloud.google.com/iap/docs/signed-headers-howto) ·
  [Cloudflare Access: validate JWTs](https://developers.cloudflare.com/cloudflare-one/access-controls/applications/http-apps/authorization-cookie/validating-json/)
- SpiceDB:
  - [Validation & testing](https://authzed.com/docs/spicedb/modeling/validation-testing-debugging) ·
    [Querying data (LookupResources limits)](https://authzed.com/docs/spicedb/concepts/querying-data) ·
    [Protecting a list endpoint](https://authzed.com/docs/spicedb/modeling/protecting-a-list-endpoint) ·
    [Materialize](https://authzed.com/docs/authzed/concepts/authzed-materialize)
- Permify: [Testing](https://docs.permify.co/getting-started/testing) ·
  [coverage action](https://github.com/permify/permify-coverage-action)
- Cedar: [Entities](https://docs.cedarpolicy.com/policies/syntax-entity.html) ·
  [Design patterns](https://docs.cedarpolicy.com/overview/patterns.html) ·
  [How we built Cedar](https://arxiv.org/pdf/2407.01688)
- Ory Keto: [github.com/ory/keto](https://github.com/ory/keto)
- Oso (archived): [github.com/osohq/oso](https://github.com/osohq/oso)
- Rust engines: [authz-core](https://github.com/zvectorlabs/authz-core) ·
  [fga-rs](https://github.com/yoogoc/fga-rs) · [zanzibar crate](https://crates.io/crates/zanzibar) ·
  [simple-zanzibar](https://lib.rs/crates/simple-zanzibar)
- AuthZEN: [Authorization API 1.0 Final](https://openid.net/specs/authorization-api-1_0.html) ·
  [Approval announcement](https://openid.net/authorization-api-1-0-final-specification-approved/)
- Zanzibar: [paper summary (AuthZed)](https://authzed.com/zanzibar) ·
  [Building Zanzibar from scratch (Oso)](https://osohq.com/post/zanzibar)
