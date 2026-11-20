# Extensibility and Languages: The Right Tool for Each Job

> **Status 2026-11-20 (D41, D42):** v1 authoring is a **YAML/JSON manifest + JSON Schema, plus one TypeScript typed builder** that prints it. Pulumi-style language hosts, the Python SDK and WASM functions are **deferred**; in-image executors over stdio cover custom action logic (D40). CEL stays. Ingestion is an OpenAPI endpoint, not a governed protocol. See [decisions.md](../decisions.md).

*Status: research + decision. Research date 2026-11-19. Revisits [plan.md](../plan.md) §2 (the Pulumi
model) and §4.6, and [principal-review.md](../principal-review.md) §4.1–4.3. The goal is the **agent
developer experience**: each extension point uses the language best suited to it, and every language
ships with the same tooling (types generated from the portal's schema, LSP, formatter,
machine-readable diagnostics with fixes, tests, examples and Agent Skills).*

---

## 1. How others are extended

| Platform | Extension model | Lesson |
|---|---|---|
| **Backstage** | **Everything is TypeScript code**: frontend extensions and **extension blueprints** (`EntityCardBlueprint`, `PageBlueprint`…), backend modules with `createExtensionPoint`, catalog providers and processors, scaffolder actions, permission policies. `app-config.yaml` wires and configures. In the new frontend system, plugins declare defaults **in code** and adopters override them through `app.extensions` config | Code-first extensibility works and is what platform teams expect. The pain was owning a scaffolded app, not writing code |
| **Port** | **No customer code runs in Port.** Data model and UI are configured in the product (or via Terraform/Pulumi as API clients). Logic lives *outside*: webhooks to your services, the **Port execution agent** (Kafka → your internal APIs, with a payload-mapping file), Ocean integrations (Python), JQ mappings | Flexible data model, but custom logic means running and securing your own services |
| **Salesforce** | **S-controls → Visualforce** (Visualforce superseded s-controls) **→ Aura** (2014, proprietary component model) **→ Lightning Web Components** (GA Spring '19, **built on web standards**; Aura kept for compatibility). Server logic stays in sandboxed Apex. **Salesforce Functions** (customer Node/Java code run as a service) **was retired**: end of sale Oct 2023, end of life 31 Jan 2025 | Moving the UI to **standard web languages** improved developer experience. **Hosting customers' general-purpose server code as a service failed commercially.** Sandboxed, platform-native execution endured |
| **Shopify Functions** | Customer logic compiled to **WebAssembly**, run by Shopify with **deterministic instruction counting**. Rust recommended; JavaScript via **Javy** (an embedded JS engine, so more instructions per line) | Sandboxed, deterministic WASM is a proven way to run third-party logic *inside* a platform at scale, in many languages |

**Conclusion:** keep code-first extensibility (the Pulumi-style program), but put each kind of logic
where it runs best:
- **Configuration programs:** real languages, run at build time.
- **In-engine logic:** sandboxed, deterministic WASM (Shopify), never a service the customer must host
  (Salesforce Functions).
- **Side effects:** executors and connectors out of process.
- **UI:** a single, widely known web stack (React + TypeScript on the `@toolbox/ui` design system), for
  operators only ([frontend.md](./frontend.md), [dashboards.md](./dashboards.md)).

## 2. The language map

| Job | Language | Why it's the right tool | Tooling shipped |
|---|---|---|---|
| Portal program: kinds, sources, connectors, scorecards, actions, views, policies | **TypeScript** (first-class), **Python** (second), **YAML/JSON manifest** (no runtime). Others via codegen when contributors appear | Real abstraction, packages, tests and IDEs; the languages agents write best. Pulumi model retained | Generated SDK types for the portal's own schema; `toolbox check` diagnostics with fixes; test runner |
| Predicates: rules, filters, conditions, policies, `where`, applies-to | **CEL** | Formal spec + **conformance suite**; non-Turing-complete; used by Kubernetes and **by OpenFGA conditions**, so one predicate language everywhere | **Typed CEL:** checked at build time against the generated kind schemas; LSP completion; cost estimation |
| Field mappings (record → fragment) | **CEL with map/list construction** + `optional` types | Kubernetes **MutatingAdmissionPolicy** precedent: CEL builds partial objects (apply configurations) declaratively. Same language and type checker as rules; missing-field semantics defined by the spec | Typed against both the connector's `discover` schema and the target kind |
| Complex transforms (recursion, grouping, folds, heavy reshaping) | **TypeScript or Rust functions compiled to WASM** (Javy/StarlingMonkey for TS; native for Rust/Go) | Deterministic, fuel-limited, sandboxed (Shopify model); unit-testable in the author's language; no service to host | `toolbox ext new transform --lang ts`; fixtures; property tests |
| Authorization model | **OpenFGA DSL** | The evaluator *is* OpenFGA ([authorization.md](../authorization.md)) | `fga model test` + `toolbox test` |
| Connectors, action executors | **Any language** (connector protocol; DBOS SDKs for durable steps) | I/O-heavy and vendor-SDK-heavy; out of process by nature | SDKs, conformance kit |
| Operator UI extensions | **React/TSX** on `@toolbox/ui`; Web Component island as an escape hatch | One design system, one test stack; agent fluency ([frontend.md](./frontend.md) §4) | `@toolbox/ui`, Storybook 10, story-based tests |
| Ad-hoc graph queries (agents, later) | Read-only **GQL** subset | ISO standard, Cypher-like, familiar to agents | — |

**Why not JSONata or jq for mappings:**
- **JSONata:** too many under-specified and implementation-divergent edge cases.
- **jq:** now has formal semantics (Färber's denotational semantics and formal specification papers,
  2023/2024, implemented by **jaq** in Rust). But those cover "a large subset" of the language, and jq,
  jaq and gojq still diverge in places, which is the same portability concern.
- Choosing CEL keeps **one** rigorously specified expression language for predicates and mappings, with
  a single type checker. Logic that exceeds CEL goes to typed, tested WASM functions rather than a
  second expression language.

**Port/Ocean import:** a JQ→CEL converter handles the common mapping subset (field paths, conditionals,
string concatenation, `map`/`select`). Anything else is emitted as a generated TypeScript transform
function with the original JQ in a comment for review.

**Risk:** cel-rust's conformance level. Mitigation: run the cel-spec conformance suite in CI, contribute
fixes upstream, and fall back to running cel-go through the same Go FFI boundary used for OpenFGA if
gaps block us.

## 2b. Where CEL and WASM fit: the four tiers

Custom behaviour uses the **first tier that can express it**:

| Tier | Used for | Runs |
|---|---|---|
| 1. Declarative config (portal program) | Kinds, sources, scorecards, actions, permissions, views | Build time only |
| 2. **CEL** | Predicates (rules, `appliesTo`, eligibility, approval and automation conditions, filters, OpenFGA conditions, `calc`) and **field mappings** (object construction) | In-engine, on every change |
| 3. **WASM functions** (TS via Javy, Rust, Go) | Logic CEL can't express: recursion, grouping, parsing, heavy reshaping, complex rule checks | In-engine, sandboxed, fuel-limited |
| 4. Connectors and executors (any language) | Anything with I/O or side effects | Out of process (TCP) |

**CEL**
- *Pros:*
  - Formally specified, with a conformance suite.
  - Non-Turing-complete and cost-bounded.
  - Type-checked at build against generated schemas.
  - **Analysable:** the fields it reads drive incremental updates, information-flow labels and
    selective tests.
  - Identical meaning in every SDK; agents know it.
- *Cons:*
  - Limited expressiveness (no recursion or general fold), and awkward deep reshaping.
  - Lives in strings inside TS or Python (mitigated by Toolbox's CEL checker and LSP).
  - cel-rust conformance must be verified (Q10).

**WASM**
- *Pros:*
  - Real languages and unit tests.
  - Sandboxed and deterministic, which keeps simulation and result caching valid.
  - No service for customers to host.
  - Fast iteration (Javy dynamic linking, precompile cache).
- *Cons:*
  - **Opaque:** functions must declare their inputs, so incremental updates are coarser and
    information flow is more conservative.
  - Slower per call than CEL (JS via QuickJS slower still).
  - Uneven toolchains beyond Rust.
  - Harder to debug.
  - An extra build step.

**Rule of thumb:** CEL by default; WASM only when a real programming language is needed; never either
for I/O.

## 3. What changed from the previous plan

- **Kept:** the Pulumi model (code-first config), TypeScript first and Python second, a
  YAML/JSON manifest as the stable contract, and WASM for in-engine logic.
- **Removed:** the connector protocol's `evaluate` capability as an *HTTP service* for rules and
  transforms. That would be a service the customer must host, the Salesforce Functions lesson. Rules
  and transforms run as WASM in-engine. Connectors and executors stay out of process.
- **Clarified:** CEL is used for mappings *with object construction*. JQ is not adopted, and JSONata
  is rejected.
- **Restricted:** UI customisation is for operators only, in code
  ([dashboards.md](./dashboards.md)).

## Sources

- Backstage:
  [extension blueprints](https://backstage.io/docs/next/frontend-system/architecture/extension-blueprints) ·
  [common blueprints](https://backstage.io/docs/frontend-system/building-plugins/common-extension-blueprints) ·
  [new backend system](https://backstage.io/docs/plugins/new-backend-system) ·
  [Red Hat: preparing plug-ins for RHDH (new frontend system)](https://developers.redhat.com/articles/2026/10/20/prepare-your-custom-plug-ins-red-hat-developer-hub-21)
- Port:
  [Port execution agent](https://docs.port.io/actions-and-automations/setup-backend/webhook/port-execution-agent/usage) ·
  [webhook backend](https://www.docs.port.io/actions-and-automations/setup-backend/webhook/)
- Salesforce:
  [Introducing Visualforce (supersedes s-controls)](https://developer.salesforce.com/docs/atlas.en-us.230.0.pages.meta/pages/) ·
  [LWC announcement coverage](https://www.crmscience.com/single-post/2019/01/22/what-you-need-to-know-about-lightning-web-components) ·
  [Salesforce Functions retirement](https://devcenter.heroku.com/articles/salesforce-functions-retirement) ·
  [Functions release notes](https://developer.salesforce.com/docs/platform/functions/guide/release-notes-intro.html)
- Shopify Functions:
  [instruction counts](https://shopify.dev/docs/apps/build/functions/optimize-instruction-counts) ·
  [JavaScript for Functions (Javy)](https://shopify.dev/docs/apps/build/functions/programming-languages/javascript-for-functions)
- jq semantics:
  [Denotational semantics and a fast interpreter for jq (arXiv 2302.10576)](https://arxiv.org/pdf/2302.10576) ·
  [A formal specification of the jq language (arXiv 2403.20132)](https://arxiv.org/pdf/2403.20132.pdf)
- CEL object construction:
  [Kubernetes Mutating Admission Policy](https://kubernetes.io/docs/reference/access-authn-authz/mutating-admission-policy) ·
  [KEP-3962](https://www.kubernetes.dev/resources/keps/3962/)
