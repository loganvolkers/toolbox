# Answers

## Q1

**Outcome.** A rule whose data is missing gets the outcome `unknown`, not `fail`. In
`crates/seikika-derive/src/lib.rs`, `FullRecompute::score` evaluates each rule's CEL `check`. `Ok(true)` gives `pass`, `Ok(false)` gives `fail`, and an evaluation error gives `unknown`. A missing field is a CEL error, so it scores `unknown`. A `fail` can then become `waived`, but an `unknown` never does. The decision is **D01M7Y5VWKW5TDS0VQWZFPRQ27N**, "Missing connector data is unknown, and it does not grant a level that has no data" (`docs/decisions/D01M7Y5VWKW5TDS0VQWZFPRQ27N.md`). It amends D70 (`D01M7VZ2DPR0000000000000000`, which already said rules over missing tool data are `unknown` and never a regression). It says that dropping the `has()` guard makes a missing field a CEL error. A rule names the connector it reads with `dataFrom` (`Rule::data_from` in `crates/seikika-model/src/manifest.rs`). For an unknown rule with `dataFrom`, `evidence()` in `seikika-derive` records the evidence `["no data from connector <src>"]`.

**Effect on the level.** The level comes from `level_ignoring_unknown` in `seikika-derive/src/lib.rs`, called from `score`:

- If every rule is `unknown`, the function returns `None`. `score` then shows the level as `none`, and `constituent_level` / `constituent_summaries` drop the entity from the repository rollup (D86, `D01M7WMSMS50000000000000000`).
- Otherwise it climbs the levels in order. A level whose rules are all `unknown` stops the climb, so a level with no data is never granted.
- At any other level, unknown rules are ignored. Every *known* rule there must pass or be waived (`counts_as_pass`), and a failing known rule stops the climb.

In short, an unknown rule neither holds the level down (a fail would) nor earns a level by itself.

**Ratchet (`seikika report --baseline`)** (`crates/seikika-cli/src/report.rs`):

- **How results are stored.** `results()` writes each entity's row: failing rules go under `failing` with an evidence count or measure, and unknown rules go under a separate `unknown` list.
- **Comparison.** `compare_entity` (through `compare_with`) never counts an unknown as a regression or an improvement. A rule that turns unknown is reported under "other" as "`<rule>` unknown now (its data is missing)". A rule that was unknown and now fails is reported as "known again: fails", also not a regression. A rule that was failing and is now unknown is not counted as "now passes".
  - Level changes that come only from unknowns (for example gold to none) are reported, not gated.
  - The unit test `unknown_outcomes_are_neither_worse_nor_better` covers this.
  - By contrast, a rule that newly fails is a regression ("`<rule>` now fails"), and so is a failing rule whose evidence count grew (`evidence_change`).
- **Verdict.** `decide` (called from `ratchet`) handles regressions first: result `regressed`, exit 1. Next, `unexpected_unknowns` counts unknown rules that `excused_rules` does not excuse. A rule is excused when its `dataFrom` connector is in `--skip`. If any unexcused unknown remains under `--ci`, `--update` or `--fail-on-unknown`, the result is `unknown` with exit 1, and `--update` refuses to write the baseline. Without those flags, unknowns do not fail the run: the summary says "rule outcome(s) unknown … not compared".
- **Verdict file.** The verdict (`struct Verdict`, D96 `D01M7XJA1FG0000000000000000`) carries `result` = `unknown` and the count of unknown rules.

One possible exception, which I have not confirmed: for a `compact` scorecard, `compare_with` treats an entity missing from the baseline as unclean when it has an `unknown` list. It then pushes a regression for it, even when only unknowns are present.

## Q2

**Startup.**

- `seikika serve` runs `serve` in `crates/seikika-cli/src/server.rs`. That function first runs `seikika_model::validate_portal`, then picks the store: `PgStore` with `--database-url`, otherwise an in-memory `SharedMem`/`MemStore`, optionally loaded with fixtures. It then calls `seikika_api::serve` (`crates/seikika-api/src/lib.rs`).
- `seikika_api::serve` spawns `run_snapshot_loop` and `connectors::freshness_loop`, and calls `connectors::spawn_in_image`.

**Supervision and transport** (`crates/seikika-api/src/connectors.rs`):

- `spawn_in_image` starts one supervisor per manifest connector with `runtime == "in-image"`, unless `SEIKIKA_CONNECTORS=off`. `binary_for` resolves the program (`builtin:<name>` becomes `seikika-connector-<name>`), and `resolve_config` resolves `${env:…}`/`${secret:…}` placeholders.
- `supervise` launches the child with `SEIKIKA_CONNECTOR_CONFIG`, restarting it with exponential backoff (1 s, capped at 30 s). The transport is **NDJSON over stdin/stdout** (v1-spec §7.4).
  - A writer task sends the hub's commands (`hub.take`) to the child's stdin.
  - The reader parses each stdout line as JSON and skips lines that don't parse. `log` messages go to tracing.
  - Other messages are batched. A batch is flushed at a control boundary (`register`, `complete`, `snapshot_complete`), at 500 messages, after 100 ms of quiet, or when the process exits.

**Checks on the way in.** `flush` calls `ingest_messages`, the same ingest path that remote connectors use through `POST /v1/ingest` (`ingest` in `routes.rs`). `ingest_messages` applies these checks:

1. The connector must be declared in the manifest; otherwise 403 `unknown-connector`.
2. Every message is validated against the compiled connector-message JSON Schema (`seikika_model::schema::errors(schema::connector_messages(), m)`).
3. Every `record` or `delete` must have a `kind` listed in that connector's `records`.

If any check fails, the whole batch is rejected with 422 `invalid-messages`, and `flush` logs "connector batch rejected". Nothing from that batch is stored.

**Into the store.**

- If the batch contains records or deletes, `Store::ingest` applies it atomically and returns a new input version.
  - `MemStore::ingest` upserts on `(connector, kind, externalId)` and removes on delete.
  - `PgStore::ingest` (`crates/seikika-store/src/pg.rs`) inserts into `seikika.input_versions` and upserts or deletes `seikika.raw_records` in one transaction.
- Control-only batches don't create a version.
- Next, `hub.on_messages` (`crates/seikika-connect/src/hub.rs`) updates registrations and command state. After a full resync, `reconcile` deletes records that the resync did not report, unless that would remove at least 90% of them (`DELETION_THRESHOLD`). A `register` triggers `initial_resync`.

**Recompute.**

- `ingest_messages` ends with `s.changed.notify_one()`. `AppState::run_snapshot_loop` (`crates/seikika-api/src/state.rs`) waits on `changed`, debounces until the input version stops moving (bounded by `max_lag`), then calls `recompute_and_publish`.
- `recompute_and_publish` takes `store.snapshot()` and runs `FullRecompute.recompute` (`seikika-derive`) on a blocking task. It then publishes the snapshot (`PgStore::publish` if this instance holds the leader lock) and syncs tuple deltas to OpenFGA.

## Q3

**Why ULIDs.** Decisions used to be rows in one table, `docs/decisions.md`, where the next id was "one more than the highest D number". Two pull requests could take the same number, and git cannot merge two appends at the same line. Decision **D01M7XKEXQH4HG5371629M50E2G** ("A decision is one file, and its id is a ULID") made each decision its own file, `docs/decisions/<id>.md`. The id is `D` plus a 26-character ULID: 48 bits of Unix milliseconds and 80 random bits, in Crockford base32. Agents can mint ids without coordinating, and filename order is time order. `docs/design/decision-files.md` §1–2 explains the choice and compares ULID with ObjectId, xid, KSUID, UUIDv7, TypeID and Snowflake. `scripts/decision-id.py` mints ids: `new_id` uses 10 bytes from `secrets`, and `--self-test` checks the oklog/ulid vector and backfill ordering. CI runs that self-test.

**The old ids.** D1–D98 were backfilled. Each file keeps `legacy: Dnn`, and its random part is zero, so the id ends in sixteen `0`s (`docs/decisions/README.md`; decision-files.md §4).

- **Timestamp:** the commit that first added the row, found with `git log --reverse -p`, not the old day headings. It is that commit's second plus the row's position among the decisions added in the same second, as a millisecond offset. `backfill_id(unix_seconds, offset)` in `scripts/decision-id.py` encodes `unix_seconds*1000 + offset` with zero randomness, so the same inputs always give the same id. For example, D84, D85 and D86 share the same second and end `…MS3…`, `…MS4…`, `…MS5…`.
- **Citations:** existing `Dnn` citations were not rewritten.

**What checks a cited id exists.**

- The rule is `cited-decisions-exist` in the `maintainer-docs` scorecard (`portal/standards/docs.yaml`), at bronze. Its check is `entity.missingDecisions.size() == 0`, and the missing ids are the evidence. It comes from D84 (`D01M7WMSMS30000000000000000`), amended by D01M7XKEXQH4HG5371629M50E2G (see `docs/design/decision-citations.md`).
- `missingDecisions` is computed by the workspace connector in `crates/seikika-connector-workspace/src/main.rs`, when it builds each doc record:
  - `known_decisions` reads every `docs/decisions/*.md`. `decision_file_ids` extracts `id` and `legacy` from the frontmatter, and `canon_decision` uppercases ULID ids.
  - `missing_decisions` matches `(?i)\b(D\d{1,4}|D[0-9A-HJKMNP-TV-Z]{26})\b` and returns the citations that are not known. It skips `docs/decisions.md` and `docs/decisions/`.
- Fixtures in `portal/tests/standards.test.yaml` test the rule. A cited `D999` fails.
- The ratchet (`seikika report portal … --baseline portal/results.json` in CI) turns a new failure into a regression.

## Q4

**Where the schema enters the code.** `crates/seikika-model/src/schema.rs` embeds the schema at build time: `CONNECTOR_MESSAGES_SCHEMA = include_str!("../../../schema/connector-messages.schema.json")`. `connector_messages()` compiles it once as Draft 2020-12, with format validation, into a `OnceLock`, and `errors()` returns the violations. A changed file therefore takes effect only once the binary is rebuilt ("A Seikika version validates against the schemas it was built with"). The `embedded_schemas_compile` test panics if the schema no longer compiles.

**Runtime rejection.** `ingest_messages` in `crates/seikika-api/src/connectors.rs` validates every message and rejects the whole batch with 422 `invalid-messages`. Two paths reach it:

- `POST /v1/ingest` (`ingest` in `crates/seikika-api/src/routes.rs`), used by remote connectors and by `seikika push`.
- The stdio supervisor for in-image connectors (`supervise`, then `flush`), which only logs the rejection.

**Check-time rejection.**

- **`seikika validate`, and the startup of `seikika serve`:** `validate_portal` calls `messages::validate_fixtures` (`crates/seikika-model/src/messages.rs`, from `validate.rs`). It validates every line of `fixtures/*.ndjson` against the schema, with diagnostic code `fixture.schema`, and checks declared record kinds (`fixture.record-kind`). `serve` exits 1 if validation fails.
- **`seikika connector test`:** the conformance kit (`run` in `crates/seikika-connect/src/conformance.rs`, check 2) validates every message a connector emits over stdio.
- **The Python reference validator** (`examples/reference-portal/tools/validate.py`) loads the same file with its own validator. It checks the reference portal's fixtures and its `protocol/*.ndjson` golden conversations. `scripts/validate-parity.py` compares it with `seikika validate`.
- **The schema file itself** is catalogued by the workspace connector and scored by `maintainer-schema` (`docs/design/schema-standards.md`; the `analyse` test in `crates/seikika-connector-workspace/src/specs.rs`). That scoring checks the document's own shape (Draft 2020-12, an `https` `$id`, wired into `schema.rs` or `validate.py`), not the messages.

**Not checked.** The local `seikika report` path does not schema-check connector output. `inputs`/`resync` in `crates/seikika-cli/src/report.rs` feed records straight to `MemStore::ingest`, and `seikika connector records` likewise just prints them. I found no other callers of `schema::connector_messages()`.

## Q5

**CLI side.**

1. `seikika get <id> --url <server>` parses into `Command::Get { id, fields, remote }` (`crates/seikika-cli/src/main.rs`). `Remote` holds `--url` (env `SEIKIKA_URL`, default `http://127.0.0.1:8080`), `--token` (`SEIKIKA_TOKEN`), `--as` (`SEIKIKA_AS`) and `-o`.
2. `read_call` maps the command to the MCP tool `get` with `{"id", "fields"}` (`CLI_MCP_TOOLS`). `main` then calls `client::tool(&remote.target(), …)`.
3. `client::tool` (`crates/seikika-cli/src/client.rs`) sends a JSON-RPC 2.0 `tools/call` as an HTTP `POST {url}/mcp`. `Target::request` adds `Authorization: Bearer <token>` and, for dev servers, `x-seikika-user`.
4. It prints `result.content[0].text` (Markdown), or `structuredContent` as JSON or YAML. It exits 1 when `isError` is set, and 2 on a 401 or a connection error.

**Server side.**

1. `POST /mcp` is routed to `mcp::handle` (`crates/seikika-api/src/mcp.rs`; route in `routes.rs`). `handle` first authenticates with `state.caller(&headers)`, then dispatches `tools/call` to `run_tool`.
2. For `get`, `run_tool` builds `/v1/entities/{id}[?fields=…]` and calls `call_rest`. That function copies the original headers into an in-process request and runs it through `router(state).oneshot`, so no second network hop occurs. Per D61 (`D01M7VQJXMR0000000000000000`), MCP tools call the REST handlers in-process so that REST, MCP, CLI and UI share one authorization and facet-shaping path.
3. `get_entity` (`routes.rs`) does the read:
   - `s.caller(&headers)` authenticates again.
   - `snapshot_for` picks the published snapshot (or waits for `min_snapshot`).
   - `s.visibility(&caller, &[id])` checks visibility.
   - `s.shape_entity(&p, &caller, &id, vis, fields)` reads `p.derived.entities[id]` from the in-memory published `Derived` snapshot. The handler then adds `inbound`, `outbound` (`reads::outgoing`) and `snapshot`.

**Identity.** `AppState::caller` (`crates/seikika-api/src/access.rs`) calls `Authenticator::authenticate` (`crates/seikika-authz/src/authn.rs`), which tries these in order:

1. An identity-aware-proxy JWT (`x-goog-iap-jwt-assertion` / `cf-access-jwt-assertion`).
2. A Bearer token: a personal access token `tbx_…` (static hash or database lookup), or an OIDC JWT in `oidc` mode.
3. In dev mode only, the `x-seikika-user` header.

A missing or invalid credential gives 401. `user_for` maps the subject to a `user:` entity (by `email` on a `User` entity, else by login), and `is_admin` checks `admin` on `portal:<name>` in OpenFGA.

**Visibility.**

- `AppState::visibility` (`access.rs`) runs batched OpenFGA checks of `can_view` and `can_view_details` on the entity's type.
  - With no authorizer, everything is visible.
  - A type whose model lacks the relation is public by default (D15): signed-in users can view it, and details fall back to admin.
  - Admins always get details.
- If `view` is false, `get_entity` filters the result out and returns 404 `not-found` ("no entity {id}"), the same response as a nonexistent entity.
- `shape_entity` applies field-level facets (`facet_needs`): `public` fields are shown, `operator` fields need admin, and any other facet needs `can_view_details`. A hidden field is replaced by a `redacted()` marker that names the relation needed and who holds it.
