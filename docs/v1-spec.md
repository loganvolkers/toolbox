# Seikika v1: Implementation Specification

*Status: implementation contract, 2026-11-20 (decision D50). This is the document an implementing
agent builds from. [decisions.md](./decisions/README.md) says what was decided and why; the design docs
explain the reasoning; this spec says **exactly what v1 must do**, in what order, and how to tell when
it's done. The [reference portal](../examples/reference-portal/README.md) is the executable anchor.*

**Precedence when documents disagree:** [decisions.md](./decisions/README.md) → this spec → design-doc status
banners → design-doc bodies → [plan.md](./plan.md). Anything a design doc describes but this spec
doesn't list is **not in v1**.

---

## 1. What v1 is

One Rust server, shipped as **one container image plus Postgres** (G1), that:

1. ingests records from connectors (GitHub, git catalog files, an IdP) and asks them for scoped
   refreshes;
2. recomputes every derived value (entities, ownership, scorecards, authorization tuples, action
   options) from a consistent snapshot whenever inputs change;
3. answers agents and people through **MCP, REST, a CLI and a minimal UI**, with OpenFGA-backed
   permissions and owner-only facets;
4. runs **self-service actions** with preview, confirmation, durable execution and verification;
5. is configured only by a **manifest in git**, with `validate`, `test`, `plan`, `build`, `deploy` and
   `verify` as the operator SDLC (G2, layer 3 of [the four SDLC layers](./sdlc.md));
6. gives apps built on a portal (the user SDLC, layer 4) a versioned `/v1` API, stable MCP tool
   names and snapshot ids for reproducible reads; and is itself built under the maintainer SDLC
   (layer 1): every milestone below ends in executable checks
   ([maintainer-sdlc.md](./design/maintainer-sdlc.md)).

The acceptance bar is the reference portal: an implementation is done when it passes §11 against
[`examples/reference-portal`](../examples/reference-portal/README.md).

## 2. Scope

| In v1 | Deferred (and the decision that deferred it) |
|---|---|
| Manifest (YAML/JSON) + JSON Schema; TypeScript builder that prints it | Python SDK, Pulumi-style language hosts (D41) |
| CEL for mappings, computed fields, rules, conditions, options | WASM functions (D41); own CEL type checker (critique C4) |
| Full recompute from a Postgres snapshot | DBSP, logical-decoding change log, checkpoints, epochs (D37) |
| OpenFGA in the image (co-process by default), reading a tuple view | c-archive linking (Q11), external OpenFGA as the default |
| Durable runs in-process on Postgres behind `Durable` | Temporal/Restate; DBOS SDK adoption is Q4 |
| Ingest API + connector control channel | A governed connector standard (D42) |
| MCP (curated tools), REST + OpenAPI, CLI, minimal UI + UI extension contract | GraphQL (D43); `@seikika/ui` persona Storybook beyond the default kit (D40) |
| In-image, HTTP, GitHub Actions and webhook executors | Argo, k8s Job, template + PR executors |
| Differential + property tests; the reference model | Deterministic simulation, Antithesis (D44) |
| Postgres full-text search over entities | tantivy, vectors, context lake (plan Phase 4) |
| Postgres-backed history of snapshots (summary counts) | History archive, analysis tools, dashboards |
| Run results through API, MCP and UI only | Notifications: GitHub App dashboard issues, PR checks, email (Phase 2) |

## 3. Milestones

Each milestone ends in exit criteria an agent can check. Build in this order; later milestones assume
earlier ones. Tracks in [AGENTS.md](../AGENTS.md) map onto these. Every milestone is closed under the
[maintainer SDLC](./design/maintainer-sdlc.md) (its exit criteria are scripts and tests). By the
[SDLC layer](./sdlc.md) each mainly serves: M1 and M6 the operator SDLC (`test`, `plan`, then
`build`/`verify`/deploy); M2–M5 the user SDLC (REST, authorization, MCP, CLI, actions), with M3's
`.fga.yaml` tests and M4's `connector test` also serving operators.

| # | Milestone | Delivers | Exit criteria |
|---|---|---|---|
| **M0** ✅ | Skeleton | Cargo workspace (§4), `seikika validate` (done 2026-11-20: [research-log.md](./research-log.md)) | Validates the reference `seikika.yaml` with the same results as `tools/validate.py`, including cross-reference errors; rejects the negative cases recorded in [research-log.md](./research-log.md) |
| **M1** ✅ | Derived state | `MemStore`, NDJSON fixture loader, the §6 pipeline behind `DerivedState`, `seikika test`, `seikika plan --local` | **Byte-identical** `expected/*.json` and `authz/tuples.expected.json` vs `tools/reference.py`; all `tests/*.test.yaml` pass; a plan between two manifests lists changed fields, levels, tuples and options; 100k-entity synthetic run under the §10.4 budget |
| **M2** ✅ | Postgres and API | `PgStore` (§8), ingest endpoint, snapshot loop, REST reads (§7.2), snapshot ids | Same outputs as M1 when fixtures arrive through `POST /v1/ingest`; `min_snapshot` read-your-writes works; restart loses nothing |
| **M3** ✅ | Authorization | Authn (IAP JWT, OIDC, PAT), OpenFGA co-process over the tuple view, facet shaping, `seikika test` runs `.fga.yaml` | Reference `authz/model.fga.yaml` passes through `Authorizer`; A3/A4/A10 behave as specified; a membership removal is enforced within 60 s |
| **M4** ✅ | Agents and connectors | MCP (§7.3), CLI (§7.7), the read pages of the [minimal UI](./design/minimal-ui.md), connector runtime with control channel (§7.4), GitHub, git-files and SCIM connectors | Acceptance A1–A5 and A11; connector conformance kit passes for all three connectors |
| **M5** 🔶 | Actions, federated (**unpaused**, D01M86BY3YQX3SA1W99QXF1QA1H, after the [self-service review](./self-service-review.md); design: [federated-actions.md](./design/federated-actions.md)) | Executor protocol (§7.5), run lifecycle (§6.6), previews and tokens, approvals, verification, federation (actions up, runs down, signed approvals), durable tables (§8.1), UI extension contract (§7.6), the minimal UI's action, run and approval pages, the `github-apps` executor. **Built:** the lifecycle in `seikika run` and `seikika serve` (REST, MCP), the `command`, `change`, `github-workflow`, `gitlab-pipeline`, `in-image` (a command or a pinned released `package`), `http`, `webhook` and federated `seikika` executors, with the protocol's `status`, `cancel` and `verify` messages sent (action-runs §5), approvals and federation, children listed by git URL with kept clones and a kept signing key (D01M86J769VGE6K5FJJQ76EP4AA). `change` executors that propose or land (`land: push`), runs written through to Postgres (`seikika.runs`). **Not yet:** durable steps across instances, the UI pages, the `github-apps` executor | `scripts/m5-acceptance.sh` (built); acceptance A6–A10 with simulated executor and connector; failure-injection properties P9–P12 hold; the sandbox-org integration test passes |
| **M6** ⏸ | Ship (**paused**, D01M7Y46VDDBK2SW6H96ZSG89MP: repo-level value first, [repo-value.md](./design/repo-value.md)) | Image build, Cloud Run deployment, `plan --remote`, `verify`, the operator SDLC workflow, metrics | A12; the reference workflow runs end to end on a sandbox project; `v0.1` tagged |

## 4. Architecture and crates

```
seikika serve (one binary, roles: all | api | worker; default all)
 ├─ api:      REST · MCP · UI host · authn · facet shaping
 ├─ derive:   snapshot loop → DerivedState::recompute → atomic in-memory swap → tuple delta to Postgres
 ├─ connect:  connector runtime (stdio children, long-poll endpoints) · command scheduler
 ├─ runs:     Durable workflows: previews, runs, approvals, verification
 ├─ exec:     executor clients (stdio children, HTTP, GitHub Actions, webhook)
 └─ openfga:  supervised co-process (unmodified upstream binary) on localhost, Postgres datastore
Postgres: inputs, derived tables, workflows, audit (§8)
```

Small, shallow workspace; heavy dependencies isolated in leaf crates (D44, D46):

| Crate | Responsibility |
|---|---|
| `seikika-model` | Manifest types (serde), JSON Schema validation, cross-reference checks, entity/value types |
| `seikika-expr` | CEL compile and evaluate (cel-rust), custom functions, error mapping to `unknown` |
| `seikika-derive` | The §6 pipeline behind `DerivedState`; plan diffs |
| `seikika-store` | `Store` trait; `MemStore`; `PgStore` (sqlx); migrations |
| `seikika-authz` | Authn; `Authorizer` trait; OpenFGA client + supervisor; facet shaping; audit |
| `seikika-durable` | `Durable` trait; Postgres implementation (Q4 decides SDK vs own) |
| `seikika-connect` | Connector runtime, control channel, command scheduler, conformance kit |
| `seikika-exec` | `Executor` trait; stdio, HTTP, GitHub Actions, webhook implementations |
| `seikika-api` | axum REST + OpenAPI generation; MCP (rmcp); UI host |
| `seikika-cli` | The `seikika` binary (server and CLI subcommands) |
| `seikika-secrets` | Secret backends for connector `${secret:…}` resolution (D55) |
| `seikika-telemetry` | OpenTelemetry and Prometheus bootstrap, metric catalog, log bridge (§10.5) |
| `seikika-connector-*` | First-party connectors as separate binaries: `github`, `gitlab`, `git-files` and `scim` read sources; `workspace` and `runner` serve Seikika for Seikika. Third-party connectors may be in any language |
| `executors/*` | First-party executors as separate binaries (any language; Rust or TS) |
| `ui/` | Embedded Astro/Vue UI (TypeScript) |

## 5. The manifest

The schema is [`crates/seikika-model/schema/manifest.schema.json`](../crates/seikika-model/schema/manifest.schema.json)
(`apiVersion: seikika.com/v1alpha1`), compiled into the `seikika` binary;
[`seikika.yaml`](../examples/reference-portal/seikika.yaml) uses every v1 feature.

| Section | Meaning |
|---|---|
| `portal` | Name, owner, pinned Seikika version |
| `policies` | Named values, readable in CEL as `policy.<name>` |
| `connectors` | Package, runtime (`in-image` or `remote`), config with `${env:…}` and `${secret:…}`, record kinds, required refresh scopes, freshness per record kind |
| `kinds` | Fields (types, facets, computed, authority), relations, identity sources, per-field source precedence, write-back, mapping rules |
| `scorecards` | Kind, owner, ordered levels, `appliesTo`, rules (`check`, `fix`), facets; optional `track`; optional `compare: {subject, peers, verdict}` (`verdict`: `anyBehind` or `majority`, D01M87WF2QZVPTD8CGDZKY133CD), with per-rule `value`, `better` (`lower` or `higher`), `unit` and `tolerance`, for a comparative scorecard (D01M857ZB468ZM2YS6GNSEG6YB9) |
| `tracks` | Long-term programs: name, owner, optional title/description (D01M7Y11G0YZT23W26Z9D3XW68Y) |
| `campaigns` | Time-bound goals: owner, `includes` (`scorecard` + `atLeast` and/or `rules`), dates, `targetPercent`, milestones, optional `track` / `draft` / `status` |
| `authz` | OpenFGA model and tests; tuple derivations (static or CEL per kind) |
| `actions` | Inputs (JSON Schema + `enumFrom`), permission, approval, preconditions, context lookups, executor, `expect` (postcondition, refresh, within) or `unverified: true`, annotations, MCP promotion, UI bundle |
| `ui.bundles` | Operator UI bundles and their mount slots |
| `include` | Source-only: other YAML files merged into the canonical manifest (D54) |
| `mcp.instructions` | Server instructions shown to agents |
| `tests` | Test files (§11.2) |

**Sources and the canonical manifest (D53–D55).** A portal's source is one YAML file, several YAML files
joined by a top-level `include` list (globs; sources only), or a TypeScript program using the
[builder](./design/typescript-builder.md). `seikika build` turns any of them into **one canonical,
key-sorted `manifest.json`**, which is what the server loads, `plan` diffs and the image hash covers.
Neither the builder nor `build` resolves `${env:…}` or `${secret:…}`; the server does at start-up.
Diagnostics on a value from an included file or package fragment name that file and the JSON
pointer inside it, not the merged document (D95).

**Action inputs (D58)** use the form-renderable JSON Schema subset in
[minimal-ui.md](./design/minimal-ui.md) §4, so every action has a generated form.

**Validation (`seikika validate`)** runs the JSON Schema, then cross-reference checks: map rules name
declared connectors and record kinds; precedence lists name sources some rule provides (or `native`);
scorecards and actions name declared kinds and levels; refreshes name declared connectors; UI bundles
exist; action inputs stay within the form-renderable subset; every CEL expression compiles with the
variables available at its site (§6.2).

**The TypeScript builder** (`@seikika/manifest`, [design](./design/typescript-builder.md)) is a typed
library whose only output is this JSON. CI keeps it honest: the TS reference portal must equal
`seikika.yaml` ([prototype](../experiments/typescript-builder/README.md): identical), its output must
pass the schema, and its negative type tests must fail to compile.

## 6. Derived-state pipeline (semantics)

The reference model is [`tools/reference.py`](../examples/reference-portal/tools/reference.py); where
prose and the model disagree, fix one of them in the same commit.

### 6.1 Snapshots

- Inputs: raw records (latest per connector + record kind + external id), deletions, native values.
- Every committed ingest batch or native write increments an **input version** (a Postgres sequence).
- The derive loop takes a `REPEATABLE READ` snapshot, recomputes everything, and swaps the result in
  memory atomically. Recompute is debounced (default 500 ms after the last change, at most 5 s
  behind under continuous change).
- **Snapshot id** = `<input version>.<manifest hash prefix>`. Every response carries it
  (`X-Seikika-Snapshot`, MCP `_meta`); reads accept `min_snapshot` and wait up to 10 s for it.

### 6.2 Stages and CEL variables

| Stage | Input | Output | CEL variables |
|---|---|---|---|
| 1. Map | Records | Fragments per `(entity id, source)`; relation edges | `record` |
| 2. Native | Native values | Fragments with source `native` | — |
| 3. Existence | Fragments | Entities (only if a source in `identityFrom` has a fragment); orphans | — |
| 4. Merge | Fragments | Field values + provenance per field | — |
| 5. Authority | Merged fields, Groups | Authority fields (e.g. `ownership`) | — |
| 6. Relations | Edges | `inbound.<relation>` lists (empty list when declared but absent) | — |
| 7. Computed | Entities | Computed fields | `entity`, `inbound`, `policy` |
| 8. Scorecards | Entities | Per rule `pass | fail | unknown`; level or `skipped` | `entity`, `policy` |
| 8b. Campaigns | Scorecard results | Per campaign: status, `inScope` / `meeting` / `percent`, per-entity gaps | (from scores) |
| 9. Tuples | Entities | Sorted, de-duplicated OpenFGA tuples | `entity`, `policy` |
| 10. Options | Entities | Sorted `enumFrom` options per action input and entity | `entity`, `policy` |

Rules, in order of importance:

- **Map:** `id` must return a string `<kind-lowercase>:<name>`; `set` returns a map whose `null`
  values mean "no value from this source". Fragments from several records with the same id and source
  are merged in record order. A map error rejects the record and records a diagnostic; it never stops
  the pipeline.
- **Merge:** for each non-computed, non-authority field, take the first source in
  `sources.fields[field]` (else `sources.default`) that has a value. Provenance records which source
  won. Sources not in the list are ignored for that field, never silently used.
- **Authority (`codeowners`):** owners of `path` = the last matching CODEOWNERS rule (`*`, `/exact`,
  glob). Teams map to Groups through `teamField`. Claim present and backed → `verified` (owner = claim);
  claim not backed → `unverified` (no owner; claim kept); no claim and exactly one owning team →
  `inferred`; otherwise `none`. Only `verified` and `inferred` owners produce authorization tuples.
- **Scorecards:** `appliesTo` false → `skipped`. A rule whose evaluation errors (missing field, type
  error) is `unknown`, not `fail`. The level is the highest level whose rules, and all lower levels'
  rules, all `pass`; otherwise `none`.
- **Determinism:** every output is sorted; the same manifest and inputs produce byte-identical JSON
  (P1). Records are processed in (connector, record kind, external id) order, so arrival order never
  matters.
- **Computed fields** see the entity's merged and authority fields, not other computed fields, so
  their evaluation order doesn't matter. `inbound.<relation>` is a lazy list: `size()` and the
  iteration macros (`map`, `exists`, `all`, `filter`) work; indexing and `in` don't (implementation
  note from M1, where converting 2M inbound elements eagerly cost seconds).

### 6.3 Deletions and safety valves

A `delete` message removes one record. A full `resync` ends with `snapshot_complete` per kind;
records not seen are deleted **only if** no `error` was reported for that kind and the deletion ratio
is below the connector's threshold (default 0.9); otherwise the deletes are held and a diagnostic
raised ([ocean.md](./ocean.md) lesson 6). Scoped refreshes never delete implicitly
([connector-control.md](./design/connector-control.md) §2).

### 6.4 Plan

`seikika plan` recomputes twice over one snapshot (current manifest, candidate manifest) and diffs
entities (fields and provenance), scorecard levels and rule outcomes, tuples (gained and lost access,
as user-level `ListObjects` deltas for affected objects), options, actions and MCP tools. `--local`
uses fixtures; `--remote` uploads the candidate manifest to the portal, which runs the plan
server-side as a durable workflow and returns the diff (no production data leaves the portal except
the diff, posted on the PR; the portal repo is the boundary, D16).

### 6.5 Facets

Each field and each scorecard part has a facet. `public` facets need `can_view`; `owner` facets need
`can_view_details`; `operator` facets need portal `admin`. The service layer removes fields the caller
can't see and adds a redaction marker `{"redacted": true, "requires": "<relation>"}`. A computed value
inherits the most restrictive facet of the fields its CEL reads, unless the manifest declares
`declassify` (D15).

### 6.6 Action runs

Exactly as [action-runs.md](./design/action-runs.md) §2–§4: preview (Seikika layer always, executor
layer when supported) → confirmation token `sha256(action version, inputs, effects, facts read,
requester, entity)`, valid 10 minutes and spent once by a server → optional approval (durable
wait; in memory until §8.1 lands) → dispatch with idempotency key → progress → executor result →
verification (refresh commands, wait for the snapshot containing the results, evaluate the
postcondition) → terminal state. Preconditions and permission are re-checked at dispatch; a
changed preview returns `preview_stale`, and so does a definition or effects that changed between
the request and the last approval. Approval is atomic: the last approval leaves
`awaiting_approval` under one lock, and an ended run is never overwritten. A postcondition that
held before the run ends `unverified`; a change counts only once its commit is on the base.
Across portals, a parent signs the approvals it passes to a child, and the child checks them
against a key it pinned ([federated-actions.md](./design/federated-actions.md) §5.3).

## 7. Interfaces and contracts

### 7.1 Rust contracts (G3)

Sketches; names and shapes are binding, details are the implementer's.

```rust
/// Inputs. MemStore for tests and `seikika dev`; PgStore in production.
#[async_trait] pub trait Store: Send + Sync {
    async fn ingest(&self, connector: &str, batch: Vec<ConnectorMessage>) -> Result<InputVersion>;
    async fn put_native(&self, write: NativeWrite) -> Result<InputVersion>;
    async fn snapshot(&self) -> Result<InputSnapshot>;          // consistent read of all inputs
}

/// Derived state. Full recompute in v1; an incremental engine may replace it (D37).
pub trait DerivedState: Send + Sync {
    fn recompute(&self, manifest: &Manifest, inputs: &InputSnapshot) -> Result<Derived>;
}
pub struct Derived { pub snapshot: SnapshotId, pub entities: Entities, pub scores: Scores,
                     pub tuples: Vec<Tuple>, pub options: Options, pub orphans: Vec<EntityId>,
                     pub diagnostics: Vec<Diagnostic> }

/// Authorization. OpenFGA co-process in v1 (Q11); external OpenFGA as a modular option.
#[async_trait] pub trait Authorizer: Send + Sync {
    async fn publish_tuples(&self, delta: TupleDelta) -> Result<()>;   // one transaction
    async fn check(&self, user: &str, relation: &str, object: &str) -> Result<bool>;
    async fn batch_check(&self, checks: &[CheckRequest]) -> Result<Vec<bool>>;
    async fn list_objects(&self, user: &str, relation: &str, object_type: &str) -> Result<Vec<String>>;
}

/// Durable workflows on Postgres (DBOS model, D39).
#[async_trait] pub trait Durable: Send + Sync {
    async fn start(&self, workflow: &str, input: Value, idempotency_key: &str) -> Result<WorkflowId>;
    async fn signal(&self, id: WorkflowId, event: &str, payload: Value) -> Result<()>;  // approvals, results
    async fn recover(&self) -> Result<usize>;   // claim workflows whose owner instance died (Q13)
}
// Inside a workflow: ctx.step(name, f) runs f at most once per (workflow, step) and checkpoints its
// output; ctx.sleep_until / ctx.wait_for(event, deadline) are durable.

/// Executors (action-runs §5).
#[async_trait] pub trait Executor: Send + Sync {
    async fn preview(&self, req: PreviewRequest) -> Result<PreviewResult>;
    async fn run(&self, req: RunRequest) -> Result<RunHandle>;          // stream of progress + result
    async fn cancel(&self, run: &RunId) -> Result<()>;
    async fn verify(&self, req: VerifyRequest) -> Result<Option<VerifyResult>>;
}
```

### 7.2 REST API (`/v1`, OpenAPI 3.1 generated per portal)

| Method and path | Purpose |
|---|---|
| `GET /v1/entities?kind=&filter=<CEL>&fields=&cursor=` | List, facet-shaped |
| `GET /v1/entities/{id}` | One entity with provenance |
| `GET /v1/entities/{id}/scorecards` | Levels and rule outcomes (facet-shaped) |
| `POST /v1/entities/{id}/refresh` | Request a refresh; returns a command id |
| `GET /v1/commands/{id}?wait=` | Command status and the snapshot that holds its results |
| `PUT /v1/entities/{id}/native/{field}` | Native write (if the field's sources include `native`) with `If-Match` revision |
| `GET /v1/actions?entity=` / `GET /v1/actions/{name}` | Available actions and schemas |
| `GET /v1/actions/{name}/options/{input}?entity=&q=` | Options with labels and disabled reasons |
| `POST /v1/actions/{name}/previews` | Preview → effects, warnings, blockers, `confirm_token` |
| `POST /v1/actions/{name}/runs` | Start a run with `confirm_token` |
| `GET /v1/runs/{id}` · `GET /v1/runs/{id}/events` (SSE) · `POST /v1/runs/{id}/cancel` · `GET /v1/runs?mine=true` | Run status, events, the caller's runs |
| `GET /v1/approvals?mine=true` · `POST /v1/approvals/{id}/decision` | Approvals for the caller; decisions need a human session (D57) |
| `GET /v1/scorecards` · `/v1/scorecards/{name}?rule=&outcome=&level=` | Scorecards with level counts; one scorecard's per-entity results with failing rules and evidence (D66). `GET /v1/entities?filter=` may read `scores['<card>']` |
| `GET /v1/campaigns` · `/v1/campaigns/{name}` · `/v1/views/campaign/{name}` | Campaign progress over visible entities (D01M7Y11G0YZT23W26Z9D3XW68Y) |
| `GET /v1/views/home` · `/v1/views/entity/{id}` · `/v1/views/scorecard/{name}` · `/v1/views/ops` | Composite, facet-shaped page views for the UI and operator bundles (D56) |
| `GET /v1/events` (SSE) | Snapshot published, command completed and run transition events visible to the caller |
| `POST /v1/ingest` | Connector ingest (connector token) |
| `GET /v1/connectors/{name}/commands?wait=` | Long-poll command stream (connector token) |
| `POST /v1/plans` | Remote plan (CI OIDC token) |
| `GET /v1/snapshot` · `/healthz` · `/readyz` · `/metrics` | Snapshot info, health, Prometheus |

Errors use RFC 9457 problem details with `code`, and for authorization failures the missing
`relation`. Pagination is cursor-based; `fields` projects.

### 7.3 MCP tools (Streamable HTTP at `/mcp`; `seikika mcp` stdio bridge)

| Tool | Inputs | Returns |
|---|---|---|
| `search` | `query`, `kind?`, `limit?` | Ranked entity summaries |
| `get` | `id`, `fields?` | Entity (facet-shaped) with provenance and snapshot |
| `query` | `kind`, `filter` (CEL), `fields?` | Matching entities |
| `graph` | `id`, `relation?`, `direction?`, `depth?` | Neighbours |
| `owner_of` | `id` | Owner, ownership status, evidence (claim, CODEOWNERS) |
| `scorecard_status` | `id`, `scorecard?` | Levels, rule outcomes, fix text and evidence for failing rules (facet-shaped) |
| `scorecard_report` | `scorecard?`, `rule?`, `outcome?`, `level?` | One scorecard across visible entities: level counts and failing rules with evidence; without `scorecard`, every scorecard (D66) |
| `campaign_report` | `campaign?` | One campaign's progress and non-meeting entities; without `campaign`, every campaign (D01M7Y11G0YZT23W26Z9D3XW68Y) |
| `assessment_brief` | `id`, `rubric?` | What a coding agent judges the entity on: the portal's rubric for its kind, the entity's facts, its subject hash, the assessment file path and a template. Read-only; the agent writes the assessment (D79) |
| `explain` | `id`, `field` or `rule` | Why: sources, precedence, CEL, observed values |
| `refresh_entity` | `id` | Waits for completion (MCP task if long); returns the new snapshot and what changed (fields, entities over the snapshot range) |
| `list_actions` | `id` | Actions available to the caller on that entity |
| `action_options` | `action`, `input`, `id`, `q?` | Options with disabled reasons |
| `preview_action` | `action`, `id`, `inputs` | Effects and `confirm_token` |
| `run_action` | `action`, `id`, `inputs`, `confirm_token` | Run id; final state as an MCP task |
| `get_run` | `run_id` | State and timeline |
| `list_runs` | `id?`, `state?` | Runs the caller may see. There is no approval tool: approvals need an interactive sign-in (D57) |
| promoted tools | per `mcp.promote` | Same as preview + run, specialised |

Responses default to compact Markdown with a JSON option, hard size caps and continuation cursors.
Annotations come from action metadata. Destructive actions always require the two-step
preview → run flow.

### 7.4 Connector API

Messages: [`crates/seikika-model/schema/connector-messages.schema.json`](../crates/seikika-model/schema/connector-messages.schema.json);
semantics: [connector-control.md](./design/connector-control.md). Golden conversation:
[`protocol/install-approved-app.connector.ndjson`](../examples/reference-portal/protocol/install-approved-app.connector.ndjson).
In-image connectors: NDJSON over stdin/stdout, started and supervised by the server (restart with
backoff). Remote connectors: `POST /v1/ingest` and the long-poll stream, authenticated with a
per-connector token scoped to its record kinds.

**v1 connectors:** `github` (repositories with admin collaborators, CODEOWNERS, apps, installations;
webhooks for repository and push events; installation visibility per Q14), `git-files`
(catalog-info.yaml), `scim` (users, groups, memberships; the revocation fast path).

### 7.5 Executor protocol

Messages: [`crates/seikika-model/schema/executor-messages.schema.json`](../crates/seikika-model/schema/executor-messages.schema.json);
semantics: [action-runs.md](./design/action-runs.md) §5. Golden conversation:
[`protocol/install-approved-app.executor.ndjson`](../examples/reference-portal/protocol/install-approved-app.executor.ndjson).
In-image executors are child processes over stdio, started with a cleared environment: a fixed
base set plus only their declared secrets (built). Running them as a separate Unix user is still
open (Q12). `http` executors receive the same messages as POST bodies. An executor that answers
`accepted` with an `executor_run_id` and no `result` keeps the run `dispatched`; Seikika then asks
after it with `status` and stops it with `cancel`. An executor that declares `verify: true` is sent
`verify` when the action has no postcondition (built). Credentials for the acting identity arrive
per request in `identity.credentials` and are never logged. **GitHub Actions executor:** dispatch with `return_run_details` and track that
run id through the runs API; a server that returns no id falls back to the one run named for the
Seikika run, on the dispatched ref, created after the dispatch. Conclusions map to `result`, a run
that never finishes is `timed_out`, and previews come from the Seikika layer only. The dispatch
token and API are the portal's, never a child manifest's. **CI hosts** (D01M87T16JRH53YV6VVWM8PH169,
[forges.md](./design/forges.md) §4.7): a manifest declares the CI systems it starts runs on under
`ci:`, apart from `forges:`. Each is `github-actions`, `gitlab-ci`, or `http`, a service that
speaks this protocol (`run`, `status`, `cancel`) for any other CI, with the child's run record in
the result's `outputs.record`. `forges[].ci` picks the host a forge's `runner: {type: ci}`
children run on, and the default is the forge's own CI. `github-workflow` and `gitlab-pipeline`
may take their endpoint and token from a host (`ci:`) (built).

### 7.6 UI extension contract

- A bundle is a static ES module built by the operator (any framework) and copied into the image by
  `seikika build`. The manifest declares `ui.bundles[] {name, path, entry, mounts[]}`.
- Slots: `action:<name>` (replaces the generated form), `entity-tab:<Kind>`, `page:<name>`.
- The module exports `mount(element, ctx): { unmount(): void }`. A Web Component may be used inside;
  the contract is the function.
- `ctx` provides: `entity`, `user`, `snapshot`, `api` (typed client generated from the portal's
  OpenAPI, same-origin session), `action.options/preview/run`, `runs.watch`, `navigate`, `theme`
  (design tokens).
- Served same-origin under a strict CSP; the server enforces every permission, so a bundle can't
  widen access.
- The first-party kit is Astro + Vue on `@seikika/ui`. Browser extensions use the module contract above.

### 7.7 CLI

| Command | Purpose |
|---|---|
| `seikika validate` | §5 validation |
| `seikika test` | `tests/*.test.yaml`, `.fga.yaml` through `Authorizer`, executor cassettes |
| `seikika plan [--local | --remote URL] [--format md|json]` | §6.4 |
| `seikika dev` | Watch, recompute over fixtures, serve API/MCP/UI locally, re-run affected tests |
| `seikika build [--image REF] [--push]` | Build the portal image |
| `seikika verify --url URL [--rollback-on-failure]` | Post-deploy smoke checks (MCP tools present, snapshot fresh, scorecards evaluating) |
| `seikika serve [--role all|api|worker]` | Run the server |
| `seikika push [--url URL]` | Send a portal's fixtures through the ingest API and wait for the snapshot (dev and tests) |
| `seikika derive [--out DIR]` | Recompute from fixtures; print a summary or write the reference-format outputs |
| `seikika report [--skip C] [-o md\|json\|html] [--baseline F [--update\|--ci]] [--verdict F]` | Score a repository without a server: connectors once, recompute, scorecards with evidence; a ratchet against a committed baseline (D66, [repo-portals.md](./design/repo-portals.md)); `--verdict` writes the outcome as JSON for any CI system (D96) |
| `seikika scorecard [NAME] [--rule R]` | One scorecard across entities: a work list (MCP `scorecard_report`) |
| `seikika campaign [NAME]` | One campaign's progress and gaps (MCP `campaign_report`) |
| `seikika assess brief ID` / `assess check` / `assess submit FILE` | Brief a coding agent on an entity (MCP `assessment_brief`), validate a portal's `assessments/`, or push an assessment to a server (D79) |
| `seikika install [PORTAL] [--frozen]` | Download released executable connectors (`package: github:owner/repo[/app]@version`) for this platform, verify them against `seikika.lock` (pinning new ones), and unpack them into the cache (D90) |
| `seikika doctor [PORTAL] [--mise]` | Check every in-image connector and executor package resolves and every declared `tools:` entry is at its pinned version; `--mise` prints the mise config that installs them (D90) |
| `include: [{package: <dir or github:owner/repo[/subdir]@ref>, with: {...}}]` | A package (D89): fragments merged with typed parameters (`${param:name}`; parameters with `tighten` may only move that way); git packages are cloned by `seikika install` (semver tag `v…`, branch, or commit SHA) and pinned in `seikika.lock` by content hash, checked on every load |
| `seikika describe` | Print Seikika's own CLI commands and MCP tools as connector records (contract snapshots; Seikika for Seikika, D65) |
| `seikika token create --user U` | Create a personal access token (printed once) |
| `seikika get|query|search|graph|owner|explain|score|refresh|actions` | Read commands, each calling the MCP tool of the same purpose (`-o md|json|yaml`) |
| `seikika mcp` | Stdio MCP server bridging to a portal's `/mcp` (for agent harnesses) |
| `seikika run [<action>] [--entity ID] [-i k=v] [--confirm TOKEN \| --yes] [--verify ID] [--runs] [--wait] [--approve\|--reject ID] [--url URL]` | Actions: list, preview, run, approve and verify in a checkout (a parent's holds approvals), or with `--url` on a served portal (federated-actions §6.4) |
| `seikika connector refresh|resync|test` | Connector operations and the conformance kit |
| `seikika login` | OIDC device flow or PAT |

## 8. Data model (Postgres)

Migrations are expand/contract only. Tables (schema `seikika`):

| Table | Key | Contents |
|---|---|---|
| `input_versions` | `version` (sequence) | Committed batch metadata: connector, counts, command id, time |
| `raw_records` | `(connector, kind, external_id)` | Latest `data`, `observed_at`, `version`, `deleted` |
| `native_values` | `(entity_id, field, revision)` | Value, author, reason, time; latest revision wins |
| `connector_state` | `(connector, kind)` | Cursor, last full sync, last error |
| `connectors` | `(name, instance)` | Registration, capabilities, heartbeat |
| `commands` | `command_id` | Command, scope, priority, deadline, status, requester, result snapshot |
| `snapshots` | `snapshot_id` | Input version, manifest hash, published at, summary counts, diagnostics |
| `derived_tuples` | `(object, relation, subject)` | Current tuples, written as deltas by the leader |
| `seikika_durable.*` | — | Durable workflow state (§8.1) |
| `runs` | `run_id` | Action, entity, inputs, requester, state, confirm token hash, executor run id, timeline |
| `audit` | `id` | Every authorization decision that denied, every write, command and run transition |

### 8.1 Durable workflow tables

[`experiments/durable-schema/schema.sql`](../experiments/durable-schema/schema.sql) is the v1 schema:
a column subset of DBOS's `workflow_status`, `operation_outputs` and `notifications` (same names), plus
a Seikika `executors` heartbeat table. Semantics, each shown by the experiment's test:

| Operation | Implementation |
|---|---|
| `start(workflow, input, key)` | Insert `workflow_status` (`PENDING`, `deduplication_id = key`); a duplicate key returns the existing workflow |
| `ctx.step(n, name, f)` | If `operation_outputs(workflow, n)` exists, return it; else run `f`, then insert the output (`ON CONFLICT DO NOTHING`) |
| `ctx.wait_for(topic, deadline)` / `signal` | `notifications` rows consumed with `FOR UPDATE SKIP LOCKED`; `LISTEN/NOTIFY` wakes waiters; deadline via `workflow_deadline_epoch_ms` |
| `recover()` | Every 15 s, claim `PENDING` workflows whose `executor_id` heartbeat is older than 60 s; increment `recovery_attempts`; replay from the last stored step |
| Completion | `status = SUCCESS | ERROR | CANCELLED`, `output` / `error` JSON |

Adopting the DBOS SDK later (Q4) means migrating these tables to DBOS's full schema, not redesigning.

### 8.2 OpenFGA tables

OpenFGA uses its own schema with its Postgres datastore; its `tuple` table is replaced by a view over
`seikika.derived_tuples` (the critique's §7.2 experiment). OpenFGA's write API is never used.

**Multiple instances:** every instance recomputes in memory from the same snapshot (deterministic, P1).
One instance, holding a Postgres advisory lock, publishes `snapshots` rows and the tuple delta. Any
instance can run workflows; `Durable::recover` reclaims those of dead instances.

## 9. Authentication and authorization

- **Authn:** in order: identity-aware-proxy JWT (`x-goog-iap-jwt-assertion`, `Cf-Access-Jwt-Assertion`,
  audience checked), OIDC session (UI) or bearer token (API/MCP via MCP OAuth 2.1), PAT (CLI, headless
  agents), connector tokens, CI OIDC tokens (plan only). Users map to `user:<login>` through IdP data.
- **Authz:** every read and action goes through one service layer that calls `Authorizer`. Summaries
  need `can_view`; facets per §6.5; actions need their `permission` on the target entity; refresh needs
  `can_refresh`. Denials return the missing relation.
- **Revocation:** a SCIM membership removal is ingested, recomputed and published within 60 s
  (P7).
- **Agents:** an agent acts as a user (OAuth or PAT); agent ceilings stay open (Q7).
- **Facet rules as implemented in M3 (D60):** a field facet named `public` (or none) needs `can_view`;
  `operator` needs portal `admin`; any other name (e.g. `contact`) is owner-only and needs
  `can_view_details`. Kinds the OpenFGA model doesn't define are visible to every signed-in user
  (public by default) and their owner-only facets to portal admins. Filters are evaluated over the
  shaped entity, so they can't probe hidden values. Native writes need `can_edit` (if the model
  defines it) or portal `admin`.
- **M4 implementation notes (D61, D62):** MCP is Seikika's own JSON-RPC handler at `/mcp` (plain
  JSON responses over Streamable HTTP; no SSE streams or MCP tasks yet), whose tools call the REST
  handlers in-process. Connector commands are held in memory (lost on restart; they become durable
  with M5). The UI polls `/v1/snapshot` every 10 s for "newer data" instead of the `/v1/events`
  SSE stream, which isn't built yet. The Astro and Vue UI shares components and Storybook stories with the published report. The stdio conformance kit applies to in-image connectors; the
  push-only SCIM connector is checked end to end by `scripts/m4-acceptance.sh`.
- **Browser sign-in:** v1 relies on the identity-aware proxy for browsers; `oidc` mode verifies bearer
  JWTs (API, MCP) but has no OAuth code flow yet.
- **Approvals are human-only in v1 (D57):** approval decisions require an interactive session (proxy
  or OIDC), not a PAT or MCP token, and aren't exposed as MCP tools.

## 10. Deployment, configuration and operations

### 10.1 Image

`FROM ghcr.io/seikika/seikika:<version>` plus `/portal/seikika.yaml`, `authz/`, UI bundle
`dist/` directories and executor binaries (reference [`Dockerfile`](../examples/reference-portal/Dockerfile)).
The base image contains `seikika` and the pinned OpenFGA binary.

### 10.2 Runtime configuration (environment)

| Variable | Meaning |
|---|---|
| `DATABASE_URL` | Postgres (Cloud SQL via connector or private IP) |
| `SEIKIKA_MANIFEST` | Manifest path (default `/portal/seikika.yaml`) |
| `SEIKIKA_PUBLIC_URL` | External URL (OAuth, links) |
| `SEIKIKA_AUTH_MODE` | `iap-google`, `cloudflare-access`, `oidc`, `token` (personal access tokens only), `dev` (trusts `x-seikika-user`). Unset: `dev` when the server binds loopback, otherwise `token`, so the image (`PORT` binds `0.0.0.0`) never trusts a header unless told to. Explicit `dev` off loopback also needs `SEIKIKA_ALLOW_DEV_AUTH_OFF_LOOPBACK`, or the server refuses to start (D01M885TA0R4HCZTVDSG9F0PHDQ) |
| `SEIKIKA_ALLOW_DEV_AUTH_OFF_LOOPBACK` | `1`, `true` or `yes` acknowledges `SEIKIKA_AUTH_MODE=dev` on a non-loopback address. Without it, `serve` exits 2. Unset, `dev` still works on loopback (D01M885TA0R4HCZTVDSG9F0PHDQ) |
| `SEIKIKA_CONNECTOR_CONFINEMENT` | `best-effort` (the default, including unset or empty) or `required`. Where the host has Landlock, in-image connectors start confined either way. Without it, `best-effort` starts them unconfined and says so in `/readyz`, `/v1/connectors`, field explain and `seikika doctor`; `required` refuses to start them. Any other value makes `serve` exit 2 (D01M88668MR8K559A1NYSNQQTX0) |
| `SEIKIKA_SECRET_<NAME>` | Env backend for `${secret:…}` (uppercase, `-` → `_`) |
| `SEIKIKA_SECRETS_DIR` | Mounted secrets directory (files named by logical secret); first hit after env |
| `SEIKIKA_SECRETS_CONFIG` | JSON/YAML path (or inline JSON) listing backends: env, file, Vault KV, AWS/GCP/Azure secret managers, plus logical→provider maps ([secret-vault.md](./design/secret-vault.md)) |
| `SEIKIKA_SECRETS_REQUIRED` | When `1`/`true`, fail if a `${secret:…}` cannot be resolved (default: omit key + warn) |
| `SEIKIKA_ROLE` | `all` (default), `api`, `worker` |

### 10.3 Cloud Run reference settings

One service, instance-based billing, minimum instances ≥ 1, 1–2 vCPU and ≥ 1 GiB per instance at the
reference scale, startup probe on `/readyz` (ready after the first recompute and OpenFGA start).
Webhooks and MCP share the service URL.

### 10.4 Budgets (checked in CI)

| Measure | Budget | Evidence |
|---|---|---|
| Full recompute, 100k entities / 2M relations / 100 rules, 4 vCPU | ≤ 10 s (the prototype measured ~3 s; the pipeline measures 4.8–4.9 s, and 8.0 s on 2 vCPU, after the scale fixes, 8.0–8.6 s before; see [research-log.md](./research-log.md), *Scale profile*). Each push is gated on instruction counts (`scripts/scale-gate.py`), and the budget is timed weekly on 4 vCPU (`scale-slo.yml`) | [technology-critique.md](./technology-critique.md) §7.1 |
| Webhook → visible | p95 ≤ 60 s | plan §6.6 |
| Check latency (OpenFGA over the view) | p95 ≤ 10 ms in-container | critique §7.2 (2.4 ms p50 over HTTP) |
| Seikika preview | p95 ≤ 150 ms | [frontend.md](./design/frontend.md) §6 |
| Cold start to ready (reference portal) | ≤ 15 s | — (measure in M6) |

### 10.5 Observability

OTel traces and logs via `seikika-telemetry` (init before listeners; honour `OTEL_*`); Prometheus
`/metrics` from the same catalog: `seikika_recompute_seconds`, `seikika_snapshot_lag_seconds`
(canonical; temporary alias `seikika_snapshot_age_seconds`), `seikika_ingest_records_total`,
`seikika_command_duration_seconds{status}`, `seikika_run_transitions_total{state}`,
`seikika_authz_checks_total{decision}`, `seikika_mcp_tool_calls_total{tool}`. Analytics sinks are
deferred beyond `none`/`otlp` (D6 keeps PostHog as a later reference sink). See
[observability.md](./design/observability.md).

## 11. Acceptance and testing

### 11.1 Acceptance

The scenarios A1–A12 in the [reference portal README](../examples/reference-portal/README.md#acceptance-scenarios-with-the-seikika-binary),
plus every milestone exit in §3.

### 11.2 Test formats

- `tests/*.test.yaml`: `name` + `assert[] {name, expect}`; `expect` is CEL with `entity(id)`,
  `score(scorecard, id)`, `options(action, id)` and `orphans()`. `seikika test` evaluates them
  against the fixtures listed by the portal (default `fixtures/`). The same file may hold
  `runs[] {name, action, entity?, as?, inputs, approve[], rejectPush?, expect {state, files?,
  refused?, error?}}`: action cases, each run through the real lifecycle in a scratch copy of the
  repository and its base ([federated-actions.md](./design/federated-actions.md) §6.5). A file's
  `defaults` holds case fields every case starts from; a case's `inputs` merge into its inputs.
- `authz/*.fga.yaml`: OpenFGA's test format, with tuples taken from the derived output.
- Executor cassettes: recorded HTTP interactions per executor test.

### 11.3 Properties (named, each with a test)

| # | Property |
|---|---|
| P1 | Determinism: the same manifest and inputs produce byte-identical derived output |
| P2 | Equivalence: Rust pipeline output equals `tools/reference.py` output on the reference portal and on generated portals |
| P3 | Precedence: a field's value always comes from the first listed source that has one; provenance names it |
| P4 | Authority: no owner tuple exists for an `unverified` or `none` ownership |
| P5 | No implicit deletes from scoped refreshes; resync deletes respect the safety valves |
| P6 | Facets: no response contains a field the caller lacks the relation for |
| P7 | Revocation: a membership removal is enforced within 60 s |
| P8 | Read-your-writes: a read with `min_snapshot` ≥ a command's result snapshot reflects it |
| P9 | A run never reaches `succeeded` unless its postcondition held at a snapshot after the executor result |
| P10 | A run never dispatches without a valid, unexpired confirm token matching the current preview |
| P11 | Required approvals always precede dispatch; permission and preconditions are re-checked at dispatch |
| P12 | Crash between any two run steps causes no duplicate external effect (idempotency key) |
| P13 | OpenFGA over the tuple view agrees with the SQL permission view (differential, critique §7.2) |

## 12. Open questions and the default to use meanwhile

| Q | Default until decided |
|---|---|
| Q1 licence | No licence field anywhere; don't publish packages |
| Q4 durable runtime | Implement `Durable` on the §8.1 schema (DBOS-compatible names); keep the DBOS SDK spike separate |
| Q7 agent ceilings | Agents act as users only |
| Q8 GitHub API details | Path confirmed (§7.5); token types verified in the sandbox spike before M5 |
| Q11 OpenFGA packaging | Co-process |
| Q12 executor isolation | Separate Unix user + declared secrets only |
| Q13 Cloud Run | One service, min instances 1, instance-based billing |
| Q14 seeing other apps' installations | Read-back through the acting user's token; mark runs `unverified` when nothing can observe the effect |

## Sources

All decisions: [decisions.md](./decisions/README.md). Measurements: [technology-critique.md](./technology-critique.md)
§7 and [experiments/](../experiments/README.md). Reference portal and its validation:
[examples/reference-portal](../examples/reference-portal/README.md),
[research-log.md](./research-log.md). GitHub endpoints: `github/rest-api-description` at `c9c95fe`
(see [examples/approved-github-apps.md](./examples/approved-github-apps.md) §2.6).
