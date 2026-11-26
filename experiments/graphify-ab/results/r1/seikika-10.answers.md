# Answers

## Q1

**Outcome: `unknown`.** In `crates/seikika-derive/src/lib.rs`, `score()` evaluates each rule's CEL `check`. It maps `Ok(true)` to `pass`, `Ok(false)` to `fail` and any evaluation error to `unknown`. An error in `appliesTo` also makes every rule `unknown`. Missing connector data shows up as a CEL error (a missing field), so the rule scores `unknown`. Decision `D01M7Y5VWKW5TDS0VQWZFPRQ27N` says this directly: dropping the `has()` guard turns a missing field into a CEL error, which is already `unknown`. A rule names the connector it reads with `dataFrom` (`Rule::data_from` in `crates/seikika-model/src/manifest.rs`). `validate.rs` checks that `dataFrom` names a declared connector. For an `unknown` rule with `dataFrom`, `evidence()` records `["no data from connector <src>"]`. A `fail` can be turned into `waived` by a waiver, but an `unknown` cannot.

**Effect on the level.** `level_ignoring_unknown()` in `seikika-derive/src/lib.rs` decides the level:
- It climbs the scorecard's levels in order.
- At each level, it ignores `unknown` rules. The level is granted only if every known rule passes or is waived (`counts_as_pass`).
- If every rule at a level is `unknown`, the climb stops, so a level with no data is never granted.
- If every rule of the card is `unknown`, it returns `None`. `score()` then shows the level as `none`.

A failing rule is different: it stops the climb at its level.

For the repository rollup, `constituent_summaries()` and `constituent_level()` also use `level_ignoring_unknown`. They skip an entity whose rules are all unknown, as well as `/demo/` fixtures and `skipped` entities. The decisions behind this are D86 (`D01M7WMSMS50000000000000000`) and `D01M7Y5VWKW5TDS0VQWZFPRQ27N`.

**How the ratchet treats it** (`crates/seikika-cli/src/report.rs`):
- `results()` writes `failing` (a rule mapped to its measure or evidence count) and a separate `unknown` list for each entity. The comment there says unknown is "never a regression or an improvement".
- `compare_entity()` handles the changes between baseline and now:
  - A rule that becomes unknown is reported under "other" as `"<rule> unknown now (its data is missing)"`.
  - A rule that goes from unknown to fail is "known again: fails", which is also "other" and not a regression.
  - A failing rule that becomes unknown is not counted as an improvement, because `!nu.contains(r)` filters it out.
  - A rule that newly fails is a regression (`"<rule> now fails"`). So is a failing rule whose evidence count or measure grew (`evidence_change`).
- `decide()` is the gate:
  - Any regression gives `regressed` and exit 1.
  - Otherwise, if `unexpected_unknowns()` is above 0 and `--ci`, `--update` or `--fail-on-unknown` is set, the result is `unknown` with exit 1, and `--update` does not write the baseline.
  - `unexpected_unknowns()` counts unknown rules except those from `excused_rules()`, which are rules whose `dataFrom` connector is in `--skip`. Skipped connectors are not run at all (`inputs()` filters them out).
  - Excused unknowns are recorded and not compared ("no regressions; N rule outcome(s) unknown … not compared").
- The verdict file (`verdict_json`, D96 = `D01M7XJA1FG0000000000000000`) carries `result` (`unknown` is one of the values) and `unknownRules`.

The decision is `D01M7Y5VWKW5TDS0VQWZFPRQ27N` ("Missing connector data is unknown, and it does not grant a level that has no data"). It amends D70 (`D01M7VZ2DPR0000000000000000`): "Rules over missing tool data are `unknown` (never a regression; `--ci`/`--update` refuse)". D86 covers the rollup.

## Q2

All of this is in `crates/seikika-api/src/connectors.rs` unless noted.

1. **Start.** `seikika serve` goes to `server::serve` in `crates/seikika-cli/src/server.rs`. That validates the portal, builds the store (Postgres `PgStore` or `SharedMem(MemStore)`) and `AppState`, then calls `seikika_api::serve` in `seikika-api/src/lib.rs`. `serve` spawns `run_snapshot_loop`, `freshness_loop` and `connectors::spawn_in_image`.
2. **Spawn.**
   - `spawn_in_image()` skips everything if `SEIKIKA_CONNECTORS=off`. Otherwise it picks each manifest connector with `runtime == "in-image"`.
   - It finds the binary with `binary_for()`. `builtin:<name>` becomes `seikika-connector-<name>`, looked up in `SEIKIKA_CONNECTORS_DIR`, next to the running exe, then on PATH; a `github:` spec resolves to the installed cache.
   - It resolves `${env:…}`/`${secret:…}` with `resolve_config()`.
   - It spawns `supervise()`, which starts the child process with `SEIKIKA_CONNECTOR_CONFIG` set. The child is restarted with exponential backoff from 1 s up to 30 s.
3. **Transport: NDJSON over the child's stdio.**
   - A writer task takes commands from `Hub::take()` (`seikika-connect/src/hub.rs`) and writes one JSON line each to the connector's stdin.
   - `supervise()` reads stdout line by line. Lines that are not JSON are skipped, and `log` messages go to tracing.
   - It batches the other messages and flushes on a control boundary (`register`, `complete`, `snapshot_complete`), at 500 messages, or after 100 ms of quiet. `flush()` calls `ingest_messages()`.
   - Remote connectors use the same function, through `POST /v1/ingest` (`routes.rs::ingest`, authenticated by `connector_of`) and the long-poll `GET /v1/connectors/{name}/commands` (`poll_commands`).
4. **Checks in `ingest_messages()`:**
   - The connector must be declared in the manifest. Otherwise the batch is rejected with 403 `unknown-connector`.
   - Every message is validated against the compiled `schema/connector-messages.schema.json` (`seikika_model::schema::connector_messages()` and `schema::errors`).
   - Every `record`/`delete` must use a `kind` listed in that connector's `records`.
   - Any error rejects the **whole batch** with 422 `invalid-messages`. For in-image connectors, `flush()` only logs "connector batch rejected" and drops the batch.
5. **Store.**
   - Only batches that contain `record`/`delete` call `Store::ingest`, which creates a new input version. Control-only batches just read `input_version()`.
   - `PgStore::ingest` (`seikika-store/src/pg.rs`) works in one transaction. It creates a new version, upserts into `seikika.raw_records` (keyed by connector, kind and externalId), deletes rows for deletes, and records counts and `command_id` on `seikika.input_versions`.
   - `MemStore::ingest` (`seikika-store/src/lib.rs`) does the same in memory and bumps `version`.
   - Next, `Hub::on_messages()` records `register`, `ack` and `complete`, and returns resync reconciliations on `snapshot_complete`. `reconcile()` then deletes records that a full read no longer reports, but holds the deletes if they would remove at least 90% of a kind (`DELETION_THRESHOLD`). A `register` triggers `initial_resync()`.
6. **Recompute.**
   - `ingest_messages` ends with `s.changed.notify_one()`.
   - `AppState::run_snapshot_loop` (`seikika-api/src/state.rs`) wakes up and sees `input_version` ahead of the published snapshot. It debounces, waiting for `debounce` (500 ms) of no change but at most `max_lag` (5 s).
   - It then calls `recompute_and_publish()`. That function takes `store.snapshot()` and runs `FullRecompute.recompute` in `spawn_blocking`. If this instance is leader, it publishes the snapshot and the tuple delta to Postgres (`pg.publish`). It syncs tuples to the in-memory Authorizer, then swaps the new `Published` into the `watch` channel. The snapshot id is `<input_version>.<manifest_hash>`.

## Q3

**Why ULIDs.** Decision `D01M7XKEXQH4HG5371629M50E2G` (design in `docs/design/decision-files.md` §1–§2) says the log used to be one table, `docs/decisions.md`, and the next id was "one more than the highest D number". Two PRs that each added a decision took the same number, and git cannot merge two appends to the same line. Each decision is now its own file, `docs/decisions/<id>.md`, and the id is `D` plus a ULID: 48 bits of Unix milliseconds and 80 random bits in Crockford base32. Anyone can mint one without a coordinator, and string order is time order, so filename order is time order. `scripts/decision-id.py` mints them (`new_id`, with 10 bytes from `secrets`). Its `--self-test` checks the oklog/ulid `TestAlizainCompatibility` vector.

**Old ids.** D1–D98 were backfilled, with `legacy: Dnn` in the frontmatter. `decision-files.md` §4, `docs/decisions/README.md` and `backfill_id()` in `decision-id.py` describe the scheme:
- The random part is zero, so the id ends in sixteen `0`s and the same inputs always give the same id.
- The time part is the second of the commit that first added the row (from `git log --reverse -p`), plus the row's position among decisions added in that same second as a millisecond offset. `backfill_id` allows 0–999, and the largest commit added 32.
- The day headings in the old table were deliberately not used. For example, D1–D14 were headed 2026-11-18, but their commit is 2026-11-19T17:09:39Z.

Decoding the files confirms this. D1 is `D01M7TCJ31R…`, which is 17:09:39 plus 0 ms, and D2 is the same second plus 1 ms. Old `Dnn` citations were not rewritten.

**What checks citations.** The `maintainer-docs` scorecard rule `cited-decisions-exist` (`portal/standards/docs.yaml`, check `entity.missingDecisions.size() == 0`, level bronze). It originally came from D84 (`D01M7WMSMS30000000000000000`), which checked `| Dnn |` rows of the table. `D01M7XKEXQH4HG5371629M50E2G` amended it (`docs/design/decision-citations.md`). The data comes from the workspace connector, `crates/seikika-connector-workspace/src/main.rs`:
- `known_decisions()` reads the `id` and `legacy` frontmatter of every `docs/decisions/*.md` (`decision_file_ids`, normalised by `canon_decision`).
- `missing_decisions()` finds citations with `(?i)\b(D\d{1,4}|D[0-9A-HJKMNP-TV-Z]{26})\b` and returns the ones that are not known. It skips `docs/decisions.md` and `docs/decisions/`.
- `doc_record()` emits the result as the `doc` record field `missingDecisions`.

The scorecard's `appliesTo` excludes `policy.historicalDocs` and `docs/decisions/`. The unit tests are in `mod decision_ids`, and there is a fixture with `D999` in `portal/tests/standards.test.yaml`. CI runs it through `seikika report portal … --baseline portal/results.json`, so a new missing citation is a ratchet regression.

## Q4

**What enforces it.** The schema is compiled into the binary. `crates/seikika-model/src/schema.rs` embeds it with `CONNECTOR_MESSAGES_SCHEMA = include_str!("../../../schema/connector-messages.schema.json")`, and `connector_messages()` compiles it once (Draft 2020-12, formats validated, in a `OnceLock`). A change therefore takes effect only after a rebuild. A schema that does not compile panics ("embedded schema compiles"), and `tests::embedded_schemas_compile` catches that. All Rust checks go through `schema::errors()`. The Python reference validator is the only check that reads the file directly.

**Runtime rejection.** `seikika_api::connectors::ingest_messages()` validates every message:
- For in-image connectors, batches arrive over stdio through `supervise()`/`flush()`. A batch with any invalid message is rejected whole (422 `invalid-messages`), logged as "connector batch rejected", and dropped.
- For remote connectors and `seikika push`, the route is `POST /v1/ingest` (`routes.rs::ingest`), and the 422 goes back to the caller. `seikika push` itself does not validate on the client side.

**Check-time rejection:**
- `seikika validate` calls `validate_portal()` (`seikika-model/src/validate.rs`), which calls `messages::validate_fixtures()`. That validates every `fixtures/*.ndjson` line against the schema and reports error code `fixture.schema`. The same `validate_portal` runs before `seikika serve` starts (an invalid portal exits 1), and inside `portal::load` for `seikika test`, `plan`, `derive` and `describe portal`.
- `seikika connector test` runs the conformance kit, `seikika_connect::conformance::run` (`seikika-connect/src/conformance.rs`). It checks "register message is valid", "every resync message is valid" and "every refresh message is valid".
- `examples/reference-portal/tools/validate.py` loads the schema file directly, without a rebuild, using Python `jsonschema` (Draft 2020-12). It validates fixture lines and the connector messages in `protocol/*.ndjson` transcripts. `scripts/validate-parity.py` compares it with `seikika validate`, and CI runs both.
- The `seikika-connector-workspace/src/specs.rs` test `the_schemas_in_this_repo_are_draft_2020_12_and_wired_in` checks the schema file itself (its dialect, `$id`, and that it is wired into `schema.rs`), not messages.

**Not enforced.** As far as I can see:
- `seikika report`, `campaign --portal` and `connector records` go through `report.rs` (`live()`, `inputs()`, `resync()`). They use `manifest_from` instead of `validate_portal` and feed connector output or fixtures straight into `MemStore::ingest`, with no schema check.
- `seikika_store::read_fixtures`/`load_fixtures` (used by `serve --fixtures` and `push`) only parse the JSON.
- The connector SDK (`seikika-connect/src/sdk.rs`) does not validate the messages it sends.

## Q5

1. **CLI.**
   - `Command::Get { id, fields, remote }` in `crates/seikika-cli/src/main.rs` flattens `Remote`: `--url` (env `SEIKIKA_URL`, default `http://127.0.0.1:8080`), `--token` (`SEIKIKA_TOKEN`) and `--as` (`SEIKIKA_AS`).
   - In `main()`, `read_call()` turns it into the MCP tool `"get"` with arguments `{"id", "fields"}`. `Remote::target()` builds a `client::Target`.
   - `client::tool()` (`seikika-cli/src/client.rs`) sends a JSON-RPC 2.0 `tools/call` request as HTTP `POST <url>/mcp` with reqwest. `Target::request` adds `Authorization: Bearer <token>` and/or `x-seikika-user: <as>`.
   - The output is `result.content[0].text` (Markdown) or `structuredContent` (`-o json`/`yaml`). Exit code 2 means a 401 or transport error, and 1 means `isError`.
2. **Server MCP.**
   - `/mcp` is routed to `mcp::handle` (`seikika-api/src/mcp.rs`, registered in `routes.rs::router`). It calls `state.caller(&headers)` before any method ("Every request needs an identity (same as REST)").
   - `tools/call` goes to `run_tool()`. For `"get"`, that builds `GET /v1/entities/{id}[?fields=…]` and calls `call_rest()`, which sends the request through the same axum router in process (`router(state).oneshot`) with **the caller's own headers copied**. MCP therefore gets exactly the REST authorization.
   - `markdown("get", …)` renders the text.
3. **REST handler.** `routes.rs::get_entity`:
   - `s.caller(&headers)` establishes identity.
   - `snapshot_for()` picks the published snapshot, honouring `min_snapshot`.
   - `s.visibility(&caller, &[id])` computes visibility.
   - `s.shape_entity(&p, &caller, &id, vis, fields)` reads the entity from `p.derived.entities` (the `Arc<Derived>` in the `Published` snapshot from `AppState::current`).
   - `.filter(|_| vis.view)` returns 404 `not-found` for an entity the caller may not view, which is the same response as a missing entity.
   - It adds `inbound` and `outbound` (`reads::outgoing`), the snapshot id, and an `x-seikika-snapshot` header.
4. **Identity.**
   - `AppState::caller` (`seikika-api/src/access.rs`) calls `Authenticator::authenticate` (`seikika-authz/src/authn.rs`). The order is: an identity-aware proxy JWT (`iap-google`/`cloudflare-access`); then a Bearer token, either a `tbx_…` personal access token (static or looked up in Postgres via `PgPats`) or a JWT in `oidc` mode; then, in `dev` mode only, the `x-seikika-user` header. With none of these, the response is 401.
   - `user_for` maps the subject to a `user:` entity (by email for email subjects, else by login), and `is_admin` checks `admin` on `portal:<name>`.
5. **Visibility.**
   - `AppState::visibility` (access.rs) batches OpenFGA checks of `can_view` and `can_view_details` (`authz.batch_check`). A type with no such relation in the model is visible to signed-in users, and its details are visible to admins (D15). With no Authorizer (dev without OpenFGA), everything is visible.
   - `shape_entity` applies facet shaping per field (`facet_needs`):
     - `public`, or no facet: shown.
     - `operator`: needs admin.
     - any other facet: needs `can_view_details`.
   - A hidden field is replaced by the `redacted()` marker (`requires`, `visible_to`).
