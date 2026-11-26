# Answers

## Q1

**Outcome.** A rule whose connector data is missing scores `unknown`. In `seikika-derive`, `score()` (`crates/seikika-derive/src/lib.rs`, the `impl` method `.score()`) evaluates each rule's CEL `check`. `Ok(true)` gives `pass` and `Ok(false)` gives `fail`. Any evaluation error gives `unknown`. A missing field is a CEL error, so missing data is `unknown`. Decision D01M7Y5VWKW5TDS0VQWZFPRQ27N says this happens because the `has()` guard was dropped. For an `unknown` rule that names its connector with `dataFrom`, `evidence()` in the same file records `["no data from connector <src>"]`.

**Effect on the level.** `score()` sets the level with `level_ignoring_unknown(sc, &outcomes).unwrap_or_else(|| "none")`. `level_ignoring_unknown()` (same file) works as follows:
- If every rule is `unknown`, it returns `None`. The displayed level is then `none`.
- Otherwise it climbs the scorecard's `levels` in order.
- It stops at the first level whose rules are all `unknown`. An all-unknown level is not granted.
- At every other level, each known (non-`unknown`) rule must be `pass` or `waived` (`counts_as_pass()`). Otherwise the climb stops.

The repository rollup (D86) uses `constituent_level()`, which skips an entity whose `level_ignoring_unknown` is `None`. An all-unknown entity therefore drops out of the rollup.

**Ratchet (`seikika report --baseline`)** (`crates/seikika-cli/src/report.rs`):
- `ratchet()` loads the baseline and calls `compare_with()` → `compare_entity()`.
- **A failing rule:** in `compare_entity()`, a rule in `failing` now but not before is a **regression** (`"{rule} now fails"`). If it failed on both sides, `evidence_change()` treats grown evidence as a regression and shrunk evidence as an improvement.
- **An unknown rule:** it is **neither worse nor better**. A rule newly `unknown` goes to `other` (`"{rule} unknown now (its data is missing)"`). A rule that was `unknown` and now fails is also `other` (`"known again: fails"`), not a regression. A rule that failed before and is `unknown` now is not counted as an improvement. The test `unknown_outcomes_are_neither_worse_nor_better()` covers this.
- **Gating on unknowns:** `ratchet()` counts all unknowns (`unknown_count()`). It also computes `unexpected_unknowns()`, which are unknowns not excused by `excused_rules()`. A rule is excused only when its `dataFrom` connector is in `--skip`.
- `decide()` returns `regressed` (exit 1) first if there are any regressions. Otherwise, if there are unexpected unknowns and `--ci`, `--update` or `--fail-on-unknown` is set, it returns result `unknown` (exit 1), so an unexcused unknown is never written into the baseline.
- Without those flags, or when every unknown is excused, unknowns are reported as "not compared".

**Decisions.** D01M7Y5VWKW5TDS0VQWZFPRQ27N ("Missing connector data is unknown, and it does not grant a level that has no data") amends D70 (`docs/decisions/D01M7VZ2DPR0000000000000000.md`). D70 already said "Rules over missing tool data are `unknown` (never a regression; `--ci`/`--update` refuse)". The ratchet itself is D66 (`D01M7VWS4ZG0000000000000000.md`).

## Q2

**Start-up.** `seikika serve` runs `serve()` in `crates/seikika-cli/src/server.rs`. It validates the portal (`seikika_model::validate_portal`), builds a `PgStore` (with `--database-url`) or a `MemStore`, builds state with `seikika_api::app_state_with`, then calls `seikika_api::serve()` (`crates/seikika-api/src/lib.rs`). That function spawns three things:
- `run_snapshot_loop()`;
- `connectors::freshness_loop()`;
- `connectors::spawn_in_image()`.

**Supervision and transport.** All of this is in `crates/seikika-api/src/connectors.rs`.
- `spawn_in_image()` handles each manifest connector with `runtime == "in-image"`, unless `SEIKIKA_CONNECTORS=off`. It resolves the binary with `binary_for()` (`builtin:<name>` → `seikika-connector-<name>`) and the config with `resolve_config()` (`${env:}`/`${secret:}`), then spawns `supervise()`.
- `supervise()` starts the child process with the config in `SEIKIKA_CONNECTOR_CONFIG`. **The transport is NDJSON over the child's stdin/stdout.** A writer task forwards commands from `hub.take()` to stdin. The child is restarted with exponential backoff (1 s up to 30 s).
- Stdout lines are read and batched until one of these happens: a control boundary (`register`, `complete` or `snapshot_complete`), 500 messages, or 100 ms of quiet. Then `flush()` calls `ingest_messages(s, name, batch)`.
- The connector name comes from the manifest, not from a token. In contrast, HTTP `POST /v1/ingest` (`routes.rs` `ingest()`) identifies the connector with `connector_of()`, and then calls the same `ingest_messages()`.

**Checks on the way:**
1. `supervise()`: a line that is not valid JSON is silently skipped. `type: "log"` messages go to tracing and are not ingested.
2. `ingest_messages()`: the connector must be declared in the manifest, or the call fails with 403 `unknown-connector`.
3. Every message is validated against the compiled connector-message schema (`seikika_model::schema::errors(seikika_model::schema::connector_messages(), m)`).
4. For `record` and `delete` messages, the `kind` must be one of the connector's declared `records`.
5. Any error rejects the **whole batch** with 422 `invalid-messages`. On the stdio path, `flush()` only logs "connector batch rejected", so the batch is dropped.

**Into the store and the recompute.**
- If the batch has records or deletes, `s.store.ingest(connector, &messages)` writes them and returns a new input version. `PgStore::ingest` in `crates/seikika-store/src/pg.rs` upserts into `seikika.raw_records`. A control-only batch reads the current `input_version()` and does not create a version.
- Next, `s.hub.on_messages()` (`crates/seikika-connect/src/hub.rs`) runs. After a full read of a kind, `reconcile()` deletes the records that the read no longer reports. It holds the deletes if they would remove ≥ 90 % of the kind (`DELETION_THRESHOLD`).
- A `register` triggers `initial_resync()`.
- Finally, `s.changed.notify_one()` wakes `AppState::run_snapshot_loop()` (`crates/seikika-api/src/state.rs`). That loop debounces (`Config.debounce` 500 ms, at most `max_lag` 5 s), then calls `recompute_and_publish()`. That function takes `store.snapshot()` and runs `FullRecompute.recompute(&manifest, &inputs)` in `spawn_blocking`. It publishes tuple deltas to Postgres (leader only) and/or syncs them to the in-memory Authorizer, then swaps in the new `Published` snapshot via `current.send_replace`.

## Q3

**Why these ids.** Decision D01M7XKEXQH4HG5371629M50E2G (`docs/decisions/D01M7XKEXQH4HG5371629M50E2G.md`; design in `docs/design/decision-files.md` §1–2) makes each decision one file, `docs/decisions/<id>.md`, with id `D` + a 26-character ULID. A ULID is 48 bits of Unix milliseconds plus 80 random bits, in Crockford base32. The reasons:
- The old single table (`docs/decisions.md`) used "next = highest D number + 1", so two PRs took the same number and their appends conflicted.
- ULIDs need no coordinator, and string order is time order.
- §2 compares ObjectId, xid, KSUID, UUIDv7, TypeID and Snowflake, and explains why ULID won.

New ids come from `scripts/decision-id.py` (`new_id()`, 80 bits from `secrets`). CI runs its `--self-test` (`self_test()`, which checks the oklog/ulid vector `01ARYZ6S410000000000000000`).

**The old ids.** D1–D98 were backfilled (`decision-files.md` §4; `docs/decisions/README.md`). Each backfilled file has `legacy: Dnn`, and its random part is zero, so the id ends in sixteen `0`s and is deterministic (`backfill_id(unix_seconds, offset)` in `decision-id.py`). The timestamp is:
- the **second of the commit that first added the row**, from `git log --reverse -p`;
- **plus a millisecond offset**, which is the row's place among the decisions added in that same second. The largest commit added 32, so every offset stays inside the second.

The day headings in the old table were deliberately not used. For example, D1–D14 are headed 2026-11-18, but the commit that added them is 2026-11-19T17:09:39Z. Existing `Dnn` citations were not rewritten.

**What checks that a cited id exists.** The `cited-decisions-exist` rule (bronze) on the `maintainer-docs` scorecard does this (`portal/standards/docs.yaml`: `check: entity.missingDecisions.size() == 0`, evidence `entity.missingDecisions`; design in `docs/design/decision-citations.md`, D84 as amended).
- The workspace connector computes `missingDecisions` in `doc_record()` (`crates/seikika-connector-workspace/src/main.rs`) as `missing_decisions(&text, &known_decisions(root), rel)`.
- `known_decisions()` reads the `id` and `legacy` frontmatter of every `docs/decisions/*.md`, using `decision_file_ids()` and `canon_decision()`.
- `missing_decisions()` matches `D\d{1,4}` or `D` + 26 Crockford characters, case-insensitively, and returns the ids that are not known. It skips `docs/decisions/` itself and `docs/decisions.md`.
- The fixture tests are in `portal/tests/standards.test.yaml`: `D999` fails in `docs/v1-spec.md`.

## Q4

**How the schema reaches the code.** The Rust code does not read `schema/connector-messages.schema.json` at run time. `crates/seikika-model/src/schema.rs` embeds it with `include_str!` (`CONNECTOR_MESSAGES_SCHEMA`) and compiles it once as Draft 2020-12 with format checks (`connector_messages()` → `compile()`). A changed schema therefore takes effect only after a rebuild ("A Seikika version validates against the schemas it was built with"). The unit test `embedded_schemas_compile()` makes a schema that does not compile fail `cargo test`.

**Run time (server).** `ingest_messages()` in `crates/seikika-api/src/connectors.rs` validates every message with `schema::errors(connector_messages(), m)`. Any error rejects the whole batch with 422 `invalid-messages`. This is the single ingest path for:
- in-image connectors over stdio: `supervise()` → `flush()`. The batch is dropped and "connector batch rejected" is logged.
- remote connectors: `POST /v1/ingest`, `routes.rs` `ingest()`. The 422 is returned to the caller.

**Check time:**
- `seikika validate`: `validate_portal()` (`crates/seikika-model/src/validate.rs`) calls `messages::validate_fixtures()` (`crates/seikika-model/src/messages.rs`). It validates every line of `fixtures/*.ndjson` against the schema and reports a `fixture.schema` diagnostic. `seikika serve` (`server.rs`) runs the same check at start-up. So do the commands that load through `portal::load()`, such as `seikika test`.
- `seikika connector test`: `seikika_connect::conformance::run()` (`crates/seikika-connect/src/conformance.rs`, check 2: "every message validates against schema/connector-messages.schema.json") validates the `register` message and the messages that later checks receive. CI runs it on the workspace, surfaces and ci-lint connectors (`.github/workflows/ci.yml`).
- `examples/reference-portal/tools/validate.py` (Python `jsonschema`) reads the schema file **from disk**, so it needs no rebuild. It validates the fixture lines and the `protocol/*.ndjson` transcripts, then `scripts/validate-parity.py` compares it with `seikika validate`. Both run in CI.
- `crates/seikika-connector-workspace/src/specs.rs` test `the_schemas_in_this_repo_are_draft_2020_12_and_wired_in()` checks the schema file's shape (Draft 2020-12, `$id`, title, description, no external refs, wired in). It does not check messages.

**Paths that do not validate, as far as I can see.** `seikika report`'s `resync()` and `inputs()` (`crates/seikika-cli/src/report.rs`), and `seikika connector records` (`connector_records()`), keep `record`/`delete` messages without schema validation. `report --fixtures` loads fixtures through `seikika_store::load_fixtures()`, which only parses JSON. `report`'s `live()` uses `portal::manifest_from()`, which validates the manifest but not the fixtures. I did not trace every other command, so there may be further paths.

## Q5

**CLI side** (`crates/seikika-cli/src/main.rs`, `client.rs`):
- `Command::Get { id, fields, remote }` flattens the `Remote` args: `--url`/`SEIKIKA_URL`, `--token`/`SEIKIKA_TOKEN`, `--as`/`SEIKIKA_AS` and `-o`.
- `main()` calls `read_call()`, which maps `get` to the MCP tool `"get"` with `{"id", "fields"}`. It then calls `client::tool(&remote.target(), "get", args, output)`.
- `client::tool()` sends **a JSON-RPC 2.0 `tools/call` request by HTTP `POST {url}/mcp`** (reqwest). `Target::request()` adds `Authorization: Bearer <token>` and/or `x-seikika-user: <as>`.
- The output is `result.content[0].text` (Markdown) or `structuredContent` (json/yaml). Exit code 1 means `isError`, and 2 means a 401 or a connection error.

**Server side:**
- `POST /mcp` routes to `mcp::handle()` (`crates/seikika-api/src/mcp.rs`). It first calls `state.caller(&headers)`: "Every request needs an identity (same as REST)". It then calls `run_tool()`.
- For `"get"`, `run_tool()` builds `/v1/entities/{id}[?fields=…]` and calls `call_rest()`. This replays the request **in-process** through `routes::router(state).oneshot(req)` with the caller's own headers copied over, so MCP gets the same authorization and shaping as REST.
- The router (`crates/seikika-api/src/routes.rs`) dispatches `/v1/entities/{id}` to `get_entity()`.

**Identity.** `get_entity()` calls `AppState::caller()` (`crates/seikika-api/src/access.rs`, "the one policy enforcement point"). That calls `Authenticator::authenticate()` (`crates/seikika-authz/src/authn.rs`), which tries these in order:
1. the IAP or Cloudflare Access JWT header, in those modes;
2. a bearer `tbx_` personal access token (static PAT hashes, then the PAT store);
3. a bearer OIDC JWT, in `oidc` mode;
4. in `dev` mode only, the `x-seikika-user` header.

If none of these succeeds the result is 401. The subject is mapped to a `user:` entity by `user_for()`, which matches a `User` entity by email, or else uses the login. `is_admin()` checks `admin` on `portal:<name>`.

**Reading and visibility:**
- `snapshot_for()` picks the published snapshot. It honours `min_snapshot` (read-your-writes).
- `AppState::visibility(&caller, &[id])` runs batched OpenFGA checks (`Authorizer::batch_check`) of `can_view` and `can_view_details`. Types the model does not cover are visible by default (D15), with details for admins. With no Authorizer, everything is visible.
- The entity is read from `p.derived.entities` in the in-memory derived snapshot. It is not read from the store. `AppState::shape_entity()` filters the requested `fields` and replaces facets the caller may not see with `redacted()` markers. Each field's `facet` is resolved by `facet_needs()`, using `admin` or `vis.details`.
- If `vis.view` is false or the entity does not exist, the response is the same 404 `not-found`.
- Otherwise `get_entity()` adds `inbound`, `outbound` (`reads::outgoing`) and `snapshot`, and sets the `x-seikika-snapshot` header (`with_snapshot()`).
- `mcp.rs` `markdown()` renders the Markdown `content` next to the JSON `structuredContent`.
