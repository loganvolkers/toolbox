# Answers

## Q1

**Outcome.** A rule whose check cannot be evaluated gets the outcome `unknown`. In `crates/seikika-derive/src/lib.rs`, `FullRecompute::score` (the method `fn score(&self, sc, entity, ctx) -> Score`) evaluates each rule's CEL `check`: `Ok(true)` gives `pass`, `Ok(false)` gives `fail`, and `Err(_)` gives `unknown`. Missing connector data reaches this branch because a missing field is a CEL error. Decision D01M7Y5VWKW5TDS0VQWZFPRQ27N says so directly: "Dropping the `has()` guard makes a missing field a CEL error, which is already `unknown`." Only a `fail` can become `waived`; an `unknown` cannot. When the rule names its connector in `dataFrom` (`Rule.data_from`), `evidence()` in the same file records `["no data from connector <src>"]` as the evidence for the unknown rule.

**Effect on the level.** `level_ignoring_unknown(sc, outcomes)` in `crates/seikika-derive/src/lib.rs` works out the level as follows:
- If every outcome is `unknown`, it returns `None`. `score` then displays `none`. `constituent_level` and `constituent_summaries` skip that entity, so it drops out of the repository rollup (D86).
- Otherwise it climbs the levels in order. At each level that has rules:
  - if every rule at that level is `unknown`, the climb stops, so the level is not granted;
  - if not, every *known* rule at that level must pass or be waived (`counts_as_pass`). Unknown rules at that level are ignored.
  - A known failure also stops the climb.

**Ratchet (`seikika report --baseline`).** The code is in `crates/seikika-cli/src/report.rs`.
- `results()` writes each entity's unknown rules into a separate `unknown` list, next to `failing`. A code comment says an unknown is "Never a regression or an improvement."
- `compare_entity()` treats the two cases differently:
  - A rule that becomes `fail` is a regression ("`{rule} now fails`"). A fail whose evidence count grows is also a regression, through `evidence_change`.
  - A rule that becomes `unknown` only goes into `other` ("`{rule} unknown now (its data is missing)`"). It is not counted as an improvement either: a rule that was failing and is now unknown is excluded from "now passes".
  - A rule that was unknown and now fails is reported as "known again: fails" in `other`, not as a regression.
- Test `unknown_outcomes_are_neither_worse_nor_better()` covers this.
- `unknown_count()` counts the unknown outcomes.
- `excused_rules()` and `unexpected_unknowns()` sort them. An unknown whose rule's `dataFrom` connector is in `--skip` is excused. Any other unknown is unexpected.
- `decide()` settles the verdict:
  - Regressions give `regressed`, exit code 1.
  - Otherwise, unexpected unknowns under `--ci`, `--update` or `--fail-on-unknown` give `result: "unknown"`, exit code 1, and the baseline is not written. A code comment says "A connector that ran and still produced no data must not be written into the baseline."
  - Excused unknowns are recorded in the baseline but not compared. The message is "no regressions; N rule outcome(s) unknown (excused by --skip, or not gating), not compared".
- `ratchet()` puts these pieces together.

**Decisions.** The main one is **D01M7Y5VWKW5TDS0VQWZFPRQ27N**, "Missing connector data is unknown, and it does not grant a level that has no data". It amends D70 (`D01M7VZ2DPR0000000000000000`), which had already said "Rules over missing tool data are `unknown` (never a regression; `--ci`/`--update` refuse)". The rollup drop refers to D86.

## Q2

1. **Startup.** `seikika serve` runs `serve()` in `crates/seikika-cli/src/server.rs`. That function validates the portal, builds the store (`PgStore` with `--database-url`, otherwise `SharedMem(MemStore)`), builds `AppState` with `seikika_api::app_state_with`, and calls `seikika_api::serve()` (`crates/seikika-api/src/lib.rs`). `seikika_api::serve()` spawns three things: `run_snapshot_loop`, `connectors::freshness_loop`, and `connectors::spawn_in_image`.

2. **Spawning.** `spawn_in_image` (`crates/seikika-api/src/connectors.rs`) handles every manifest connector with `runtime == "in-image"`, unless `SEIKIKA_CONNECTORS=off`.
   - `binary_for(package)` resolves the binary. For example, `builtin:github` becomes `seikika-connector-github`, found in `SEIKIKA_CONNECTORS_DIR`, next to the running binary, or on `PATH`.
   - `resolve_config` resolves `${env:…}` and `${secret:…}` in the connector's config.
   - Then it starts `supervise()`.

3. **Transport.** The transport is **stdio, NDJSON**. `supervise()` starts the child process with `SEIKIKA_CONNECTOR_CONFIG` set to the config JSON and with stdin and stdout piped.
   - A writer task takes commands from the hub (`s.hub.take`) and writes them to stdin, one JSON object per line.
   - The reader parses each stdout line as JSON and silently drops lines that don't parse. `log` messages go to tracing.
   - It batches messages and calls `flush()` at a control boundary (`register`, `complete`, `snapshot_complete`), at 500 messages, or after 100 ms of quiet.
   - When the process exits, it restarts it with exponential backoff (1 s, doubling to a cap of 30 s).
   - Commands come from `initial_resync` (after `register`), `freshness_loop` (the manifest's `connectors[].freshness`) and refresh requests.

4. **Checks.** `flush()` calls `ingest_messages(s, connector, msgs)`. This is the single ingest path for every transport: remote connectors reach it through `POST /v1/ingest`, the `ingest` handler in `routes.rs`. It applies these checks in order:
   - The connector must be declared in the manifest. Otherwise the result is 403 `unknown-connector`.
   - Every message must validate against the compiled connector-message JSON Schema, through `seikika_model::schema::errors(seikika_model::schema::connector_messages(), m)`.
   - For `record` and `delete` messages, `kind` must be one of the connector's declared `records`.
   - If any check fails, the **whole batch** is rejected with 422 `invalid-messages`. `flush` only logs "connector batch rejected".

5. **Into the store.** If the batch contains a `record` or `delete`, `s.store.ingest(connector, &messages)` applies it atomically and returns a new input version.
   - `MemStore::ingest` upserts records and removes deletes.
   - `PgStore::ingest` (`crates/seikika-store/src/pg.rs`) does the same in one transaction: an upsert into `seikika.raw_records` keyed on (connector, kind, external_id), a delete, and a row in `input_versions`.
   - A batch with only control messages creates no new version.

6. **Bookkeeping after the write.**
   - `s.hub.on_messages()` (`crates/seikika-connect/src/hub.rs`) records registrations, acks and completions. After a `snapshot_complete` for a command with no errors, it returns the external ids that were seen.
   - For each such kind, `reconcile()` deletes stored records that the full read no longer reported. It holds the deletes instead (and logs) when they would remove at least 90% of that kind's records (`DELETION_THRESHOLD`).
   - A `register` message triggers `initial_resync`.

7. **Recompute.** `ingest_messages` ends with `s.changed.notify_one()`. `AppState::run_snapshot_loop` (`crates/seikika-api/src/state.rs`) wakes up and sees that the store's `input_version` differs from the published one. It debounces: it waits until inputs stop changing for `debounce` (default 500 ms), or until `max_lag` (default 5 s) has passed. Then it calls `recompute_and_publish`, which:
   - takes `store.snapshot()`;
   - runs `FullRecompute.recompute(&manifest, &inputs)` on a blocking thread;
   - on the instance holding the Postgres advisory lock, publishes the snapshot and the tuple delta;
   - syncs authorization tuples to the in-memory OpenFGA;
   - swaps the new `Published` snapshot into `current`.

   Record contents are checked once more during the recompute. A `map` rule that fails on a record becomes a `map.error` diagnostic in `Derived.diagnostics` (`seikika-derive/src/lib.rs`); the batch itself is not rejected.

## Q3

**Why ULIDs.** Decision **D01M7XKEXQH4HG5371629M50E2G**, "A decision is one file, and its id is a ULID", explains this, together with `docs/design/decision-files.md` §1–2.
- The log used to be one table, `docs/decisions.md`, and the next id was "one more than the highest `D` number". Two pull requests that each added a decision took the same number, and git cannot merge two appends at the same line.
- Each decision is now its own file, `docs/decisions/<id>.md`. The id is `D` plus a 26-character ULID: 48 bits of Unix time in milliseconds and 80 bits of randomness, in Crockford base32. Ids can therefore be minted with no shared counter, and filename order is time order.
- §2 compares ULID with ObjectId, xid, KSUID, UUIDv7, TypeID and Snowflake. ULID was chosen because it needs no coordinator, its alphabet has no ambiguous `I`, `L`, `O` or `U`, and string order is time order.
- `scripts/decision-id.py` mints new ids: `new_id()` uses 10 bytes from `secrets`. `--self-test` checks the oklog/ulid vector `01ARYZ6S410000000000000000`.

**Why the old ids end in zeros and have `legacy:`.** D1–D98 were backfilled from the old table (`decision-files.md` §4; `docs/decisions/README.md`).
- Each backfilled file keeps its old number as `legacy: Dnn`.
- The random part is zero. The last 16 of the 26 ULID characters are the 80 random bits, so a backfilled id ends in sixteen `0`s and the same inputs always produce the same id.
- `backfill_id(unix_seconds, offset)` in `scripts/decision-id.py` encodes `unix_seconds * 1000 + offset` with `bytes(10)` as the random part.
- New decisions have no `legacy` field.

**How the timestamps were chosen.** The time is that of the **commit that first added the row**, found with `git log --reverse -p`. To that second, the backfill added a millisecond offset: the row's place among the decisions added in the same second. The largest commit added 32 decisions, so every offset stays inside the second.
- The old day headings were deliberately not used. D1–D14 are headed 2026-11-18, but the commit that added them is 2026-11-19T17:09:39Z.
- The commit time is when the row became shared state, and it orders D1…D98 with no inversions.
- The frontmatter field `decided` holds that time. Existing `Dnn` citations were not rewritten.

**What checks that a cited id exists.** The bronze rule **`cited-decisions-exist`** in the `maintainer-docs` scorecard (`portal/standards/docs.yaml`) does this. Its check is `entity.missingDecisions.size() == 0`, and its evidence is `entity.missingDecisions`. The facts come from the workspace connector (`crates/seikika-connector-workspace/src/main.rs`):
- `doc_record()` emits `missingDecisions` for each `doc` record.
- `missing_decisions(text, known, rel)` finds citations with `(?i)\b(D\d{1,4}|D[0-9A-HJKMNP-TV-Z]{26})\b`, so either a legacy `D` plus one to four digits or a ULID id. It canonicalizes each one with `canon_decision` (uppercase) and returns those not in `known`. It skips `docs/decisions.md` and `docs/decisions/*`, which are the index.
- `known_decisions(root)` collects, through `decision_file_ids()`, the `id` and `legacy` frontmatter values of every `docs/decisions/*.md`.

The rule comes from D84 (`D01M7WMSMS30000000000000000`), which originally checked `| Dnn |` table rows. D01M7XKEXQH4HG5371629M50E2G amended it to check files. The design is in `docs/design/decision-citations.md` §3, the fixture test is in `portal/tests/standards.test.yaml` (`D999` fails), and there are unit tests in the workspace connector's `tests` module. The rule runs as part of `seikika report portal … --baseline portal/results.json` in CI. `decision-id.py --self-test` only checks the encoding, not existence.

## Q4

**Where the schema enters the code.** `crates/seikika-model/src/schema.rs` embeds the file at compile time with `CONNECTOR_MESSAGES_SCHEMA = include_str!("../../../schema/connector-messages.schema.json")`. `connector_messages()` compiles it lazily, once (`OnceLock`), as Draft 2020-12 with format validation. `errors()` returns (JSON pointer, message) pairs. Because of `include_str!`, a changed schema takes effect in the Rust binary only after a rebuild ("A Seikika version validates against the schemas it was built with"). If the new schema doesn't compile, `compile()` panics with "embedded schema compiles". The unit test `embedded_schemas_compile` catches that under `cargo test`.

**Runtime: the server.** `ingest_messages()` in `crates/seikika-api/src/connectors.rs` validates every message in a batch and rejects the whole batch with 422 `invalid-messages` if any message fails. It is reached two ways:
- From in-image connectors over stdio: `supervise()` calls `flush()`, which calls `ingest_messages`. A rejected batch is only logged ("connector batch rejected").
- From remote connectors, and from `seikika push`, through `POST /v1/ingest` (the `ingest` handler in `crates/seikika-api/src/routes.rs`).

**Check time.**
- `seikika validate`: `validate_portal()` (`crates/seikika-model/src/validate.rs`) calls `messages::validate_fixtures()` (`crates/seikika-model/src/messages.rs`). That function validates every line of `fixtures/<connector>.ndjson` against the schema (diagnostic `fixture.schema`), and also checks declared record kinds (`fixture.record-kind`). `seikika serve` runs `validate_portal` before it starts, so bad fixtures stop the server from starting (exit code 1).
- `seikika connector test`: `connector_test()` in `crates/seikika-cli/src/main.rs` calls `seikika_connect::conformance::run()` (`crates/seikika-connect/src/conformance.rs`). It validates the connector's live stdio output against the same schema in three checks: "register message is valid", "every resync message is valid" and "every refresh message is valid".
- The Python oracle `examples/reference-portal/tools/validate.py` reads `schema/connector-messages.schema.json` from disk at run time, with no rebuild needed. It validates `fixtures/*.ndjson`, and the connector-side messages in `protocol/*.ndjson`. `scripts/validate-parity.py` compares `seikika validate` with it, and CI runs both.
- Separately, the `maintainer-schema` scorecard (D77, `docs/design/schema-standards.md`) scores the schema file itself, not messages. The workspace connector's `specs.rs` analysis checks, among other things, that it is Draft 2020-12 and is wired into the code (`the_schemas_in_this_repo_are_draft_2020_12_and_wired_in`).

**Paths that do not validate, as far as I can see.** `seikika report` running connectors live (`resync()` and `inputs()` in `crates/seikika-cli/src/report.rs`) puts the `record`/`delete` messages straight into `MemStore::ingest` without a schema check. `seikika_store::load_fixtures` does no schema check either; for `serve --fixtures`, it relies on `validate_portal` having run first. I did not check whether `seikika test` validates fixtures on its own path.

## Q5

1. **CLI.** `Command::Get { id, fields, remote }` is defined in `crates/seikika-cli/src/main.rs`.
   - `remote` is the flattened `Remote` struct: `--url` (env `SEIKIKA_URL`, default `http://127.0.0.1:8080`), `--token` (env `SEIKIKA_TOKEN`), `--as` (env `SEIKIKA_AS`), and `-o md|json|yaml`.
   - In `main()`, `read_call()` maps it to the MCP tool `"get"` with arguments `{"id", "fields"}`. `CLI_MCP_TOOLS` lists the pairings, and its comment says CLI and MCP "share one authorization path, D61".
   - `main()` then calls `client::tool(&remote.target(), "get", args, &remote.output)`.

2. **How the request is carried.** `client::tool()` (`crates/seikika-cli/src/client.rs`) sends **one HTTP POST to `{url}/mcp`**. The body is a JSON-RPC 2.0 `tools/call` message: `{"name": "get", "arguments": …}`.
   - `Target::request()` adds `Authorization: Bearer <token>` when `--token` is given, and `x-seikika-user: <as>` when `--as` is given.
   - The CLI prints `result.content[0].text` (Markdown) or `structuredContent` (json/yaml).
   - It exits 2 on a 401 or a connection error, and 1 when `isError` is true or on another HTTP error.

3. **Server, MCP layer.** The route is `/mcp`, mapped to `crate::mcp::handle` in `routes.rs`.
   - `handle()` (`crates/seikika-api/src/mcp.rs`) first checks identity: `state.caller(&headers)` must succeed ("Every request needs an identity (same as REST)").
   - For `tools/call` it calls `run_tool()`. The `"get"` arm builds `/v1/entities/{id}`, adding `?fields=` when fields are given, and calls `call_rest()`.
   - `call_rest()` does **not** open a network connection. It builds an in-process `Request`, copies over all of the caller's headers except content-length and content-type, and sends it through `router(state).oneshot(req)`. The REST handler therefore sees the same credentials.
   - A non-2xx answer becomes an `isError` result. Otherwise `markdown()` renders the `"get"` answer as `**id** (kind)` plus one line per field with its provenance.

4. **Server, REST handler.** The route is `/v1/entities/{id}`, mapped to `get_entity()` in `crates/seikika-api/src/routes.rs`. It works in this order:
   - **Identity:** `s.caller(&headers)` again. `AppState::caller()` (`crates/seikika-api/src/access.rs`) calls `Authenticator::authenticate()` (`crates/seikika-authz/src/authn.rs`). That tries, in order:
     - an identity-aware proxy JWT (`x-goog-iap-jwt-assertion` or `cf-access-jwt-assertion`, depending on the mode);
     - a bearer token: a `tbx_…` personal access token, looked up by hash in the static or Postgres token store, or, in `oidc` mode, a JWT;
     - in `dev` mode only, the `x-seikika-user` header.

     No credentials give 401 `unauthenticated`; bad ones give 401 `invalid-credentials`. `user_for()` maps the subject to a `user:` entity (by email, else by login), and `is_admin()` checks `admin` on `portal:<name>` in OpenFGA.
   - **Snapshot:** `snapshot_for()` picks the published snapshot, honouring `min_snapshot` if given. The CLI's `get` does not send one.
   - **Visibility:** `s.visibility(&caller, &[id])` in `access.rs` makes batched OpenFGA checks of `can_view` and `can_view_details` on the entity. When the model does not define a relation for that type, it falls back to "visible to signed-in users" for view and "admin" for details; the code comment cites "Public by default (D15)". Admins always get details. With no Authorizer (dev without OpenFGA), everything is visible.
   - **Read and shape:** `s.shape_entity(&p, &caller, &id, vis, fields)` (`access.rs`) is the code that actually reads the entity. It reads `p.derived.entities[id]` from the in-memory `Published` derived snapshot, not from the store, and applies field facets with `facet_needs`:
     - `public` fields are always shown;
     - `operator` fields need admin;
     - any other facet needs `vis.details`.

     Hidden fields are replaced by a `redacted()` marker that names the relation needed and who holds it.
   - The result is filtered by `.filter(|_| vis.view)`. Without view permission, or if the entity is missing, the answer is 404 `not-found` ("no entity {id}"), so a hidden entity is indistinguishable from a missing one.
   - On success it adds `inbound`, `outbound` (from `reads::outgoing`) and `snapshot`, and sets the `x-seikika-snapshot` header (`with_snapshot`). As far as I can see, the neighbour ids in `inbound` and `outbound` are not filtered by the caller's visibility in this handler. I am not certain whether that is filtered elsewhere.
