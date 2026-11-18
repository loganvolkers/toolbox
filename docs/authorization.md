# Toolbox Authorization — Relationship-Based Access Control on the Catalog Graph

*Research date 2026-11-18. Primary sources:*
- *The OpenFGA docs repo (`openfga/openfga.dev`: testing, store file format, conditions, modular
  models, agent/RAG guides).*
- *The `openfga/language` repo (grammar and conformance data).*
- *The OpenFGA server's test corpus (`assets/tests/*.yaml`).*
- *The `zvectorlabs/authz-core` source.*

*Other projects are covered from their docs via search excerpts. Repository stats come from GitHub
search on the same date. Companion to [plan.md](./plan.md) §5.*

---

## 1. TL;DR

**Recommendation:**
- **Adopt OpenFGA's modelling language, semantics and test file format (`.fga.yaml`) as Toolbox's
  authorization language.**
- **Implement the evaluator natively in Rust over Toolbox's own catalog graph**, instead of running
  OpenFGA (or SpiceDB) as a separate service.
- Keep the real OpenFGA as a **test oracle** in CI, and as an **optional external decision service**.

This gives you:
1. **The OpenFGA testing story, unchanged.** An authorization model plus small fixture files, runnable
   both by `toolbox test` and by the stock `fga model test` CLI.
2. **No synchronization problem.** The catalog graph *is* the relationship store. Permission tuples are
   *derived* from entity relations (`spec.owner`, group membership, `partOf` and so on) by rules
   declared in code, so there is nothing to dual-write and no lag between "ownership changed" and
   "access changed".
3. **Field-level visibility.** Kinds and scorecards declare **facets**, each guarded by a permission.
   For example, a scorecard's summary is visible to everyone, but its details (the list of critical
   issues) only to the owning team and security. One shared response-shaping layer enforces this
   identically in API, MCP, CLI, UI, search, context packs and the changefeed.
4. **One expression language.** OpenFGA conditions are written in CEL, which Toolbox already uses for
   filters, rules and mappings. This proposal replaces Cedar (see the earlier plan) with the OpenFGA
   model plus CEL conditions.
5. **Performance beyond the hosted engines' list limits.** OpenFGA caps ListObjects at 1,000 results
   by default, and SpiceDB documents LookupResources degrading past ~10k results. Toolbox can avoid
   this because the graph is in memory, so it can precompute group closures (Zanzibar's "Leopard"
   idea) and per-user permission bitmaps.

The cost is building a correct ReBAC evaluator. OpenFGA publishes a language-neutral conformance
corpus (Apache-2.0) that de-risks this substantially (§4.3).

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
 catalog graph (entities + relations, from connectors and catalog-info.yaml)
        │  derivation rules (portal program, build-time validated)
        ▼
 authz tuples (virtual: computed views over the graph, never separately stored)
        │  + static tuples in code (e.g. break-glass, portal-wide roles)
        │  + contextual tuples from the request token (e.g. IdP group claims)
        ▼
 OpenFGA-semantics evaluator (Rust, in-process)  →  decisions, lists, explanations
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

### 4.2 Facets: field-level visibility

Kinds, facts and scorecards declare **facets**: named groups of fields, each guarded by a permission
in the model. The scorecard example from the requirements:

```
# authz/core.fga
module core

type user
type agent
  relations
    define acts_for: [user]

type group
  relations
    define member: [user, group#member]

type portal
  relations
    define member: [user, user:*, group#member]
    define security: [group#member]
    define admin: [group#member]

type service
  relations
    define portal: [portal]
    define owner: [group]
    define can_view: member from portal
    define can_view_score_summary: can_view
    define can_view_score_details: member from owner or security from portal or admin from portal
    define can_run_actions: member from owner or admin from portal
```

```ts
new tb.Scorecard("security", {
  appliesTo: service,
  visibility: {
    summary: "can_view_score_summary",   // level, colour, pass/fail counts, trend
    details: "can_view_score_details",   // rule-level outcomes, observed values, remarks, issue lists
  },
  // ...rules
});

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

### 4.3 The evaluator (Rust, `toolbox-authz`)

- **Language front end:** a parser for the OpenFGA DSL and `fga.mod` that produces OpenFGA's JSON model
  (`schema 1.1`), from which a typed IR is compiled.
  - It is validated against `openfga/language`'s shared test data: DSL syntax and semantic validation
    cases, and DSL↔JSON transformer cases (~29 transformer fixtures plus module cases).
  - The ANTLR grammar (`OpenFGALexer.g4`/`OpenFGAParser.g4`) is the reference.
- **APIs** (OpenFGA semantics):
  - `Check`, `BatchCheck`
  - `ListObjects`, `ListUsers`
  - `Expand` (for explanations)
  - Conditions (CEL, via the same `toolbox-expr` crate)
  - Wildcards (`user:*`), usersets (`group#member`), tuple-to-userset (`X from Y`), `and`/`or`/`but not`
  - Contextual tuples, cycle detection, depth limits
- **Conformance:**
  - Every build runs OpenFGA's server test corpus: `consolidated_1_1_tests.yaml` (153 cases, ~960
    check assertions, plus list-objects and list-users assertions) and `abac_tests.yaml` (34 cases,
    ~260 checks). Both are Apache-2.0.
  - A nightly **differential fuzzer** generates random models and tuples and compares answers with a
    real OpenFGA server in a container. Any divergence is a bug.
- **Performance design** (the in-memory graph makes this cheap):
  - *Principal context:* on the first request in a graph version, compute the principal's transitive
    usersets, such as all `group#member` they reach through nested groups. This is Zanzibar's Leopard
    idea, made cheap by the in-memory graph. It is cached per (principal, graph version) and
    invalidated by the changefeed.
  - *Permission bitmaps:* for hot (type, permission) pairs (`service#can_view`,
    `service#can_view_score_details`), materialize a roaring bitmap of accessible object ids per
    principal on demand. List, search and context-pack filtering become bitmap intersections.
  - *Targets:* Check p99 < 1 ms; ListObjects over 100k services < 20 ms warm; principal context
    < 5 ms for users in ≤ 200 groups. Verified with `toolbox-synth` (10k users, 1k teams, nested
    groups, 100k entities).
- **Consistency:**
  - Every decision is made against a graph version, returned as `X-Toolbox-Graph-Version`.
  - Clients can request "at least version N" for read-after-write, like Zanzibar's zookies. An agent
    that just changed ownership via an action can rely on that.
- **Explain:**
  - `explain(principal, object, permission)` returns the satisfied path (e.g. `anne → member
    group:payments → owner service:payments-api`), or for a denial the nearest missing edges and
    who can create them.
  - It is exposed as an MCP tool and a CLI command, and used by the UI's "request access" hint.

### 4.4 Enforcement points (one decision engine, every surface)

| Surface | What is checked | How |
|---|---|---|
| REST API | Object permission + facets on every read; action permissions on writes | Middleware resolves the principal; handlers declare `(object, permission)`; the shared serializer shapes responses |
| MCP | Same as API. Tools are filtered from `tools/list` when the principal can't use them at all | MCP handlers call the same service layer. Context packs are built only from allowed facets, and token budgets are computed *after* redaction |
| CLI | Via API | No local enforcement logic to diverge |
| UI | Via API | Hidden facets render as "restricted, owned by X, request access" |
| Catalog lists and queries | `ListObjects` bitmap ∩ query results | Counts and pagination are computed after filtering, so totals don't leak |
| Search | Per-facet index documents with object ids; results ∩ permission bitmaps | Snippets, highlights and facet counts come only from allowed documents. Detail text is indexed in a separate facet document, so it can never rank or snippet for an unauthorized user |
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
- A headless agent with a service token is just an `agent` principal, with no user.
- The audit log and analytics record both principals.
- Context packs and search follow the RAG pre-filter pattern via permission bitmaps, so an agent never
  receives a chunk it could not have opened directly.

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

**CI oracle:** in portal repos, the stock `fga model test` runs alongside `toolbox test` on Level-1
files, and any divergence fails the build. Users get two independent implementations agreeing on their
authorization model.

### 4.7 Interop and replaceability

- **AuthZEN 1.0 PDP endpoint:** other tools can ask Toolbox "can X do Y on Z?" using the standard API.
  Examples: an API gateway, an MCP gateway, or an internal admin tool that wants to reuse
  catalog-derived permissions.
- **External PDP mode (optional):** for organisations that already run OpenFGA centrally, Toolbox
  streams derived tuples to it and delegates decisions. Because the model is OpenFGA's language, this
  is a configuration switch rather than a rewrite. Expect higher latency and lower list-filtering
  limits; the in-process engine stays the default.
- **`toolbox authz export --fga`** writes a standard OpenFGA store file (model plus derived tuples from
  a snapshot) for offline analysis in the OpenFGA tooling.

---

## 5. Changes to the plan

1. **Replace Cedar with OpenFGA model + CEL conditions** as the single authorization language. Cedar's
   formal verification and policy analysis are real strengths, but it lacks graph traversal and list
   primitives. Running two languages (ReBAC for relationships plus Cedar for policies) costs more than
   it gains at this stage. Revisit if attribute-heavy guardrails outgrow CEL conditions.
2. `toolbox-authz` becomes the OpenFGA-compatible parser and evaluator, principal context, permission
   bitmaps, explain, AuthZEN endpoint, and the external-PDP adapter.
3. Kinds, facts and scorecards gain **facets**. A single response-shaping layer is mandatory for all
   surfaces.
4. `toolbox test` runs Level 1–3 authorization tests. `toolbox plan` reports authorization impact.
5. Ship a default module (`@toolbox/authz-default`) with portal membership, ownership, security and
   admin roles, plus facet conventions, so most portals start secure and only extend.

## 6. Phasing

| Phase | Authorization scope |
|---|---|
| 0 (spike, 2–3 weeks) | DSL parser + Check evaluator. Exit: ≥ 95% of OpenFGA's consolidated Check assertions pass. Bitmap list filtering is benchmarked at 100k objects. |
| 1 (catalog) | Full conformance (Check/BatchCheck/ListObjects/ListUsers, conditions, modules); derivation rules; facets + response shaping; Level 1–2 tests; contextual tuples from OIDC |
| 2 (scorecards) | Scorecard summary/detail facets; facts with facets; `plan` authz diff |
| 3 (actions) | `can_run` at request and execution; approvers via ListUsers; agent ceilings |
| 4 (knowledge/search) | Per-facet search documents, bitmap-filtered search and context packs |
| 5 (ecosystem) | AuthZEN endpoint; connector-contributed modules; external OpenFGA mode; `authz export` |
| 6 (hardening) | Nightly differential fuzzing vs OpenFGA; side-channel review; Level-3 surface snapshot suite |

## 7. Risks

| Risk | Mitigation |
|---|---|
| Semantic divergence from OpenFGA (especially `but not`, intersections, conditions, list APIs) | Conformance corpus on every build; differential fuzzing; the stock `fga model test` oracle in portal CI |
| Authorization data leaks through side channels (counts, search snippets, timing, 403 vs 404, changefeed) | One shaping layer; per-facet search documents; post-filter aggregates; uniform 404s; Level-3 surface tests; a dedicated security review in phase 6 |
| Model complexity for portal authors | Default module with secure defaults; facets as a simple declarative concept; agent skill for "add a permission + tests"; `explain` |
| Performance of list operations with exclusions or intersections on huge sets | Bitmaps for hot permissions; fall back to batched Check post-filtering with over-fetch; benchmarks in CI |
| Stale permissions after IdP changes | IdP/HR connectors on webhooks; contextual tuples from fresh tokens; connector-lag SLOs (plan §6) |

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
