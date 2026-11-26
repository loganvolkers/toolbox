# Answers

## Q1

**The rule's outcome is `unknown`.** It is never a regression or an improvement in the ratchet. A rule that fails is different: a rule that starts failing is a regression and exits 1. An unknown makes the ratchet exit 1 only under `--ci`, `--update` or `--fail-on-unknown`, and only when the rule's connector was not skipped.

**Decision.** `docs/decisions/D01M7Y5VWKW5TDS0VQWZFPRQ27N.md` ("Missing connector data is unknown, and it does not grant a level that has no data"). It amends D70 (`D01M7VZ2DPR0000000000000000.md`), which first said "Rules over missing tool data are `unknown` (never a regression; `--ci`/`--update` refuse)". It relies on D86 (`D01M7WMSMS50000000000000000.md`) for the repository rollup. The verdict field `unknownRules` and the `unknown` result come from `D01M7XJA1FG0000000000000000.md`.

**How the outcome is produced** (`crates/seikika-derive/src/lib.rs`, `score`):
- Each rule's `check` CEL expression is evaluated. `Ok(true)` gives `pass`, `Ok(false)` gives `fail`, and an evaluation error gives `unknown`.
- Missing connector data is a missing field, and a missing field is a CEL error. The decision drops the `has()` guards so that a missing field produces that error.
- If `appliesTo` itself errors, every rule of that card is `unknown`.
- A `fail` can become `waived`. An `unknown` cannot.
- Evidence for an unknown rule that has `dataFrom` is `["no data from connector <src>"]` (`evidence`). Measures are recorded only for `fail` (`measure`).

**Effect on the level** (`level_ignoring_unknown`, same file):
- If every rule is unknown, the function returns `None`, and `score` displays the level as `none`.
- Otherwise it climbs the levels in order:
  - A level whose rules are all unknown stops the climb, so a level with no data is not granted.
  - At a level with known rules, every known rule must be `pass` or `waived` (`counts_as_pass`). Unknown rules at that level are ignored.
- In the repository rollup (`constituent_summaries` and `constituent_level`, which feed `maintainer-repo`), an entity whose rules are all unknown is skipped. It does not drag the repository level down (D86).

**Ratchet** (`crates/seikika-cli/src/report.rs`):
- `results` writes each entity's unknown rules as a separate `"unknown"` list, next to `"failing"` (which carries a measure or an evidence count) and `"waived"`.
- `compare_entity` handles each case as follows:
  - A rule that becomes unknown is a change in `other` ("unknown now (its data is missing)"). It is not a regression or an improvement.
  - A rule that was unknown and now fails is in `other` ("known again: fails"). It is not a regression.
  - A rule that failed before and is now unknown is not counted as an improvement, because the filter excludes rules in `nu`.
  - A newly failing rule is a regression ("now fails"). A rule whose evidence grew is handled by `evidence_change`.
  - Test: `unknown_outcomes_are_neither_worse_nor_better`.
- `excused_rules` and `unexpected_unknowns`: an unknown is excused when the rule's `dataFrom` connector is in `--skip`.
- `decide` reaches its verdict in this order:
  1. Any regression gives `regressed`, exit 1.
  2. Otherwise, unexcused unknowns with `--ci`, `--update` or `--fail-on-unknown` give the result `unknown`, exit 1. Under `--update`, the baseline is not written.
  3. Otherwise, the check passes: "no regressions; N rule outcome(s) unknown … not compared". If the results differ from the baseline, the result is `stale` instead (exit 1 only with `--ci`).
- The unknown count is reported as `Verdict.unknown`, and as `unknownRules` in the `--verdict` JSON (`ratchet`).

## Q2

**Short answer.** Commands go to the connector on its stdin, and messages come back on its stdout, one JSON object per line. The messages are batched and passed to `ingest_messages`. That function checks the batch and writes it to the store, then wakes the snapshot loop. The snapshot loop recomputes.

**Startup.**
- `crates/seikika-cli/src/server.rs` `serve`:
  - validates the portal (`seikika_model::validate_portal`);
  - opens a `PgStore` (with `--database-url`) or an in-memory `SharedMem(MemStore)`, which is loaded from fixtures with `--fixtures`;
  - builds `AppState` (`seikika_api::app_state_with`) and calls `seikika_api::serve`.
- `crates/seikika-api/src/lib.rs` `serve` spawns `AppState::run_snapshot_loop` and `connectors::freshness_loop`, and calls `connectors::spawn_in_image`.

**Running the connector** (`crates/seikika-api/src/connectors.rs`).
- `spawn_in_image` starts a supervisor for each manifest connector with `runtime == "in-image"`, unless `SEIKIKA_CONNECTORS=off`.
  - `binary_for` resolves the package: `builtin:<name>` becomes `seikika-connector-<name>`, found in `SEIKIKA_CONNECTORS_DIR`, next to the binary, or on `PATH`. A `github:` package is the release installed in the cache.
  - `resolve_config` resolves `${env:…}` and `${secret:…}`.
- `supervise` spawns the child process. The config is passed in `SEIKIKA_CONNECTOR_CONFIG`, stdin and stdout are piped, and stderr is inherited.
  - **Transport:** stdio, newline-delimited JSON.
  - A writer task drains `Hub::take` (`crates/seikika-connect/src/hub.rs`) and writes each command as one line on stdin.
  - The reader parses each stdout line:
    - A line that is not JSON is skipped silently.
    - `log` messages go to `tracing` and are not ingested.
    - Other messages are batched. A batch is flushed at a `register`, `complete` or `snapshot_complete` message, at 500 messages, or after 100 ms of quiet.
  - When the process exits, the supervisor restarts it with exponential backoff, from 1 s up to 30 s.
- `flush` calls `ingest_messages`. A rejected batch is only logged ("connector batch rejected") and dropped.

**Checks in `ingest_messages`.**
1. The connector must be declared in the manifest. Otherwise it returns 403 `unknown-connector`.
2. Every message is validated against `schema/connector-messages.schema.json`, via `seikika_model::schema::errors(connector_messages(), m)`.
3. A `record` or `delete` must have a `kind` listed in that connector's `records`.

Any error rejects the whole batch with 422 `invalid-messages`. Remote connectors go through the same function through `POST /v1/ingest` (`routes.rs` `ingest`, which needs a connector token).

**Into the store.**
- A batch that contains a `record` or `delete` is applied by `Store::ingest` as one batch, which returns a new input version:
  - `PgStore::ingest` (`crates/seikika-store/src/pg.rs`) runs one transaction. It upserts into `seikika.raw_records`, keyed by connector, kind and `externalId`, and adds a row to `seikika.input_versions`.
  - `MemStore::ingest` (`crates/seikika-store/src/lib.rs`) does the same in memory.
- A batch with only control messages does not create a version.
- `seikika_telemetry::catalog::record_ingest` counts the records.
- `Hub::on_messages` records the registration, acks, the ids seen per command, and command completion and `last_read`.
- A `snapshot_complete` for a command with no errors triggers `reconcile`. It deletes records that the resync no longer reports, unless that would delete at least 90% of the kind (`DELETION_THRESHOLD`). In that case the deletes are held and logged.
- A `register` triggers `initial_resync`, which enqueues resyncs for kinds never read.

**Recompute.**
- `ingest_messages` ends with `s.changed.notify_one()`.
- `run_snapshot_loop` (`crates/seikika-api/src/state.rs`) sees that the input version has moved. It debounces (`Config.debounce`, 500 ms by default, capped by `max_lag`, 5 s), then calls `recompute_and_publish`. That function:
  - reads `store.snapshot()` and runs `FullRecompute.recompute` in `spawn_blocking`;
  - builds the snapshot id `<input_version>.<manifest_hash>`;
  - on the advisory-lock leader, publishes to Postgres with the tuple delta;
  - syncs the tuple delta to the in-memory Authorizer;
  - swaps the result into the `watch` channel (`current.send_replace`).

## Q3

**Why ULIDs.** The decision is `docs/decisions/D01M7XKEXQH4HG5371629M50E2G.md` ("A decision is one file, and its id is a ULID"). The design is `docs/design/decision-files.md`.
- The log used to be one table, and the next id was the next `D` number. Two pull requests both took the same number and both appended at the same line, which git cannot merge.
- Now each decision is a file, `docs/decisions/<id>.md`. The id is `D` plus a 26-character ULID: 48 bits of Unix milliseconds and 80 random bits, in Crockford base32.
  - Agents can mint ids without a coordinator.
  - Filename order is time order.
  - The alphabet has no ambiguous `I`, `L`, `O` or `U`.
- §2 of the design compares ObjectId, xid, KSUID, UUIDv7, TypeID and Snowflake, and explains why ULID won.
- `scripts/decision-id.py` mints a new id: `new_id` takes 10 bytes from `secrets`. `--self-test` checks the oklog/ulid vector, and CI runs it (`.github/workflows/ci.yml`).

**The old ids.** D1–D98 were backfilled.
- Each file has `legacy: Dnn`, and its `decided` time is the commit that first added the row (from `git log --reverse -p`).
- The random part is zero (`backfill_id`, `bytes(10)`), so a backfilled id ends in sixteen `0`s and is reproducible (`docs/decisions/README.md`; decision-files §4).
- The id's timestamp is that commit's second, in milliseconds, plus the row's position among the decisions added in the same second (`backfill_id(unix_seconds, offset)`, with offset 0..999).
  - For example, D1 is `D01M7TCJ31R…` = 2026-11-19T17:09:39.000Z, and D32 is `D01M7TCJ32Q…` = 17:09:39.031Z. The largest commit added 32 decisions, so every offset stays inside its second.
  - The day headings in the old table were not used. D1–D14 are headed 2026-11-18, but their commit is 2026-11-19.
- Existing `Dnn` citations were not rewritten.

**What checks a cited id exists.** The rule is `cited-decisions-exist` in the `maintainer-docs` scorecard, `portal/standards/docs.yaml`. It is bronze, checks `entity.missingDecisions.size() == 0`, and gives the missing ids as evidence. It comes from D84 (`D01M7WMSMS30000000000000000.md`), as amended by the ULID decision.
- The `Doc` field `missingDecisions` is computed by the workspace connector, `crates/seikika-connector-workspace/src/main.rs`:
  - `known_decisions` reads `docs/decisions/*.md`.
  - `decision_file_ids` takes both `id` and `legacy` from the frontmatter.
  - `canon_decision` keeps `Dnn` as written and uppercases a ULID.
  - `missing_decisions` matches `D\d{1,4}` or `D` plus 26 Crockford characters, case-insensitively, and returns the ones not known.
- Files under `docs/decisions/` and `docs/decisions.md` are skipped. The scorecard's `appliesTo` also excludes `docs/decisions/` and `policy.historicalDocs`.
- Tested by the `decision_ids` unit tests in that file, and by `portal/tests/standards.test.yaml` (`docs/v1-spec.md` fixture citing `D999` fails).
- A second check covers assessment reasons: `known_id` in `crates/seikika-connect/src/assessment.rs`, used by `cites`. A `D01…` id must be an existing file. A short `D` id must match a `legacy: Dnn` line. A `Q` id must appear in `docs/questions.md`.

## Q4

**Where the schema comes from.** `crates/seikika-model/src/schema.rs` embeds the file at compile time: `CONNECTOR_MESSAGES_SCHEMA = include_str!("../../../schema/connector-messages.schema.json")`. `connector_messages()` compiles it once (`OnceLock`), as Draft 2020-12 with format validation, and `errors()` returns pointer and message pairs. A changed schema therefore takes effect in the Rust code only after a rebuild: "A Seikika version validates against the schemas it was built with". The unit test `embedded_schemas_compile` panics if the new schema does not compile.

**Runtime rejection.**
- `ingest_messages` in `crates/seikika-api/src/connectors.rs` validates every message, and any error rejects the whole batch. This one function serves two paths:
  - In-image connectors over stdio (`supervise` → `flush`). The batch is dropped and logged.
  - Remote connectors over `POST /v1/ingest` (`routes.rs` `ingest`). The caller gets 422 `invalid-messages`.
- The conformance kit, `seikika connector test` (`crates/seikika-connect/src/conformance.rs` `run`, called from `main.rs`), validates every message the connector emits. For example, it runs the "register message is valid" check. CI runs it for the `workspace`, `surfaces` and `ci-lint` connectors.

**Check-time rejection.**
- `seikika validate` calls `validate_portal` (`crates/seikika-model/src/validate.rs`), which calls `messages::validate_fixtures` (`crates/seikika-model/src/messages.rs`). Every line of `fixtures/*.ndjson` is checked, and each failure is a `fixture.schema` diagnostic.
- `validate_portal` also gates other commands. A non-conforming fixture makes them refuse to run:
  - `seikika serve` (`server.rs` `serve`, exit 1);
  - `seikika test`, `plan`, `derive` and `describe` (through `portal::load`).
- The Python oracle, `examples/reference-portal/tools/validate.py`, reads `schema/connector-messages.schema.json` from disk, so it needs no rebuild. It validates fixtures and also the `protocol/*.ndjson` transcripts.
- `scripts/validate-parity.py` compares `seikika validate` with that oracle. CI runs both.
- The schema file itself is scored by `maintainer-schema` (D77) through the workspace connector (`crates/seikika-connector-workspace/src/specs.rs`). That rule checks the file's draft, `$id`, refs and wiring, not message conformance.

**Not enforced.** As far as I can tell, these paths only parse JSON and do not schema-check messages:
- `seikika report`'s own connector runs (`report.rs` `resync` and `inputs`, reached from `live`, which uses `manifest_from`, not `validate_portal`);
- `seikika_store::read_fixtures` and `load_fixtures`.

## Q5

**Short answer.** The CLI sends the `get` MCP tool call to the server's `/mcp`. The server turns it into an in-process call to the REST route `GET /v1/entities/{id}`, and that route reads the entity from the published snapshot. Identity is checked twice, at `/mcp` and again on the REST route. Visibility is applied only on the REST route.

**CLI** (`crates/seikika-cli/src/main.rs`).
- `main` calls `read_call`, which maps `Command::Get { id, fields, remote }` to the tool `"get"` with args `{"id", "fields"}`. `CLI_MCP_TOOLS` lists `("get", "get")`, so CLI and MCP share one authorization path (D61).
- `Remote::target` builds a `client::Target` from `--url` (env `SEIKIKA_URL`), `--token` (`SEIKIKA_TOKEN`) and `--as` (`SEIKIKA_AS`).
- `client::tool` (`crates/seikika-cli/src/client.rs`) POSTs a JSON-RPC 2.0 `tools/call` to `<url>/mcp` with reqwest.
  - `Target::request` adds `Authorization: Bearer <token>`, and `x-seikika-user` when `--as` is set (dev servers only).
  - It prints `structuredContent` (`-o json` or `-o yaml`) or the Markdown text (`-o md`).
  - Exit codes: 2 on a 401 or a connection error, 1 when `isError`.

**Server** (`crates/seikika-api`).
- `routes.rs` `router` sends `POST /mcp` to `mcp::handle`. That function:
  - parses the JSON-RPC message;
  - **authenticates first** with `state.caller(&headers)`, so every request needs an identity;
  - dispatches `tools/call` to `run_tool`.
- In `run_tool`, `"get"` builds the URI `/v1/entities/{id}[?fields=…]` and calls `call_rest`. `call_rest` copies all request headers except content-length and content-type, and runs the request through `router(state).oneshot(req)`. This is an in-process REST call, not a network hop.
- `routes.rs` `get_entity` handles `GET /v1/entities/{id}`:
  1. `s.caller(&headers)` authenticates again.
  2. `snapshot_for` picks the published snapshot, honouring `min_snapshot`.
  3. `s.visibility(&caller, &[id])` computes visibility.
  4. `s.shape_entity(...)` builds the entity from `p.derived.entities`, and `.filter(|_| vis.view)` hides it unless the caller may view it.
  5. The response adds `inbound`, `outbound` (`reads::outgoing`) and `snapshot`, and `with_snapshot` sets the `x-seikika-snapshot` header.
  6. An entity that is missing or not visible gives 404 `not-found`, so the two look the same.
- The entity data is read from the in-memory `Published.derived` snapshot, not from the store.

**Identity** (`crates/seikika-api/src/access.rs` `AppState::caller`). It calls `Authenticator::authenticate` (`crates/seikika-authz/src/authn.rs`), which tries these in order:
1. an identity-aware-proxy JWT (`x-goog-iap-jwt-assertion` or `cf-access-jwt-assertion`);
2. a bearer token: a `tbx_` personal access token (static or from Postgres), or an OIDC JWT in `oidc` mode;
3. in dev mode only, the `x-seikika-user` header.

The resulting subject is mapped to a `user:` entity by `user_for` (by email, else by login). `is_admin` checks `admin` on `portal:<name>`.

**Visibility** (`access.rs`).
- `visibility` batch-checks `can_view` and `can_view_details` through the OpenFGA `Authorizer`.
- A type the model does not define is public by default for `can_view` (D15). `can_view_details` falls back to admin, and admins always get details.
- With no Authorizer (dev without OpenFGA), everything is visible.
- `shape_entity` replaces each field whose facet the caller lacks with a `redacted` marker. An `operator` facet needs admin, and any other non-public facet needs `can_view_details`.

**Uncertain.** In `get_entity`, the `inbound` and `outbound` neighbour ids appear to be returned without a per-neighbour visibility filter. I did not find code that filters them.
