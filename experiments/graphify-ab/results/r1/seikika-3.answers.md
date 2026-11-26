# Answers

## Q1

**Outcome.** A rule whose data is missing scores `unknown`. In `crates/seikika-derive/src/lib.rs`, the `score` method evaluates each rule's `check` CEL expression. `Ok(true)` becomes `pass`, `Ok(false)` becomes `fail` (or `waived` when a manifest waiver matches), and `Err(_)` becomes `unknown`. A missing connector field makes the CEL expression error, so the rule is `unknown`. The decision is **D01M7Y5VWKW5TDS0VQWZFPRQ27N**, "Missing connector data is unknown, and it does not grant a level that has no data", which amends D70 (`D01M7VZ2DPR0000000000000000`). It says that dropping the `has()` guard makes a missing field a CEL error, which is already `unknown`. A rule's `dataFrom` names the connector it reads. When an `unknown` rule has `dataFrom`, `evidence` (same file) records `["no data from connector <src>"]` as its evidence.

**Effect on the level.** `level_ignoring_unknown` in `crates/seikika-derive/src/lib.rs` works as follows:
- If every rule is `unknown`, it returns `None`, and `score` shows the level as `none`.
- Otherwise it climbs the scorecard's `levels` in order. If every rule at a level is `unknown`, the climb stops there, so a level with no data is not granted.
- At a level with some known rules, only the known rules are considered. They must all pass or be waived (`counts_as_pass`) for the entity to reach that level.

So `unknown` rules neither fail a level nor help an entity reach one, and an all-unknown level ends the climb. For the repository rollup, `constituent_level` and `constituent_summaries` skip an entity whose rules are all `unknown` (`level_ignoring_unknown` returns `None`). That entity drops out of the "lowest constituent" level. D86 (`D01M7WMSMS50000000000000000`) says this, and so does D01M7Y5VWKW5TDS0VQWZFPRQ27N.

**The ratchet (`seikika report --baseline`).** These functions are in `crates/seikika-cli/src/report.rs`:
- `results` writes each entity's `failing` rules (with measure or evidence counts) and its `unknown` rules as separate lists. The comment there says `unknown` is "Never a regression or an improvement".
- `compare_entity` treats a newly failing rule as a regression (`"{rule} now fails"`), and growing evidence is a regression through `evidence_change`. A newly unknown rule only goes to `other` (`"{rule} unknown now (its data is missing)"`). A rule that was failing and is now unknown is not counted as an improvement, because of the `!nu.contains(r)` filter. A rule that goes from unknown to failing is `other` (`"known again: fails"`), not a regression. The test `unknown_outcomes_are_neither_worse_nor_better` covers this.
- `excused_rules` and `unexpected_unknowns` count the unknowns whose rule's `dataFrom` connector is *not* in `--skip`.
- In `decide`, any regression gives `regressed` with exit 1. If there are no regressions but some unknowns are unexpected, and `--ci`, `--update` or `--fail-on-unknown` is set, the result is `unknown` with exit 1, and `--update` refuses to write the baseline. Without those flags, or when every unknown is excused by `--skip`, the run passes with the summary "N rule outcome(s) unknown … not compared".
- `ratchet` wires this together, and `verdict_json` reports `unknownRules` (D96, `D01M7XJA1FG0000000000000000`).

In short, a failing rule regresses the ratchet. An unknown rule is never a regression, but it blocks `--ci` and `--update` (and `--fail-on-unknown`) unless its connector was skipped. The decision text gives the example: `runner` and `github-actions` are skipped, but `ci-lint` is not, so a missing zizmor or actionlint still fails `--ci`.

## Q2

1. **Startup.** `seikika serve` runs `serve` in `crates/seikika-cli/src/server.rs`. It validates the portal and builds the store: `PgStore` with `--database-url`, otherwise an in-memory `SharedMem(MemStore)`. It then calls `seikika_api::serve` in `crates/seikika-api/src/lib.rs`. That function spawns `run_snapshot_loop` and `connectors::freshness_loop`, and calls `connectors::spawn_in_image`.
2. **Spawning the connector.** These are in `crates/seikika-api/src/connectors.rs`. `spawn_in_image` does nothing if `SEIKIKA_CONNECTORS=off`. For each connector with `runtime == "in-image"`, it resolves the program with `binary_for`: `builtin:<name>` becomes `seikika-connector-<name>`, found in `SEIKIKA_CONNECTORS_DIR`, next to the binary, or on PATH. It resolves the config with `resolve_config`, which handles `${env:…}` and `${secret:…}`, and starts `supervise`.
3. **Transport.** `supervise` starts the binary as a child process. The config goes in the `SEIKIKA_CONNECTOR_CONFIG` environment variable. Messages are newline-delimited JSON over stdio:
   - A writer task takes commands from the hub (`hub.take`) and writes them to the child's stdin.
   - The reader takes stdout lines. Lines that are not valid JSON are skipped, and `log` messages go to tracing.
   - The reader batches messages until a control boundary (`register`, `complete` or `snapshot_complete`), 500 messages, or 100 ms of quiet. Then `flush` calls `ingest_messages`.
   - If the child exits, it is restarted with exponential backoff (1 s, capped at 30 s).
4. **Checks in `ingest_messages`.** This is the single ingest path. HTTP `POST /v1/ingest` (`ingest` in `routes.rs`) uses it too.
   - The connector must be declared in the manifest. Otherwise the result is 403 `unknown-connector`.
   - Every message is validated against the embedded `schema/connector-messages.schema.json`, using `seikika_model::schema::errors(seikika_model::schema::connector_messages(), m)`.
   - Every `record` and `delete` must have a `kind` the connector declares in `records`.

   Any error rejects the whole batch with 422 `invalid-messages`. For in-image connectors, `flush` only logs "connector batch rejected", so the batch is dropped.
5. **Store.** If the batch contains a `record` or `delete`, `s.store.ingest(connector, &messages)` applies it as one batch and returns a new input version. This is the `Store::ingest` trait in `crates/seikika-store/src/lib.rs`:
   - `PgStore::ingest` in `pg.rs` uses one transaction: it upserts into `seikika.raw_records`, deletes rows, and updates `seikika.input_versions`.
   - `MemStore::ingest` upserts or removes records and bumps the version.

   A batch with only control messages does not create a version.
6. **Bookkeeping.**
   - `s.hub.on_messages` (`crates/seikika-connect/src/hub.rs`) records registrations, acks and completions, and returns reconciliations at each `snapshot_complete`.
   - `reconcile` deletes records that a full resync no longer reported. If that would remove at least 90% of a kind (`DELETION_THRESHOLD`), it holds the deletes.
   - A `register` triggers `initial_resync`.
7. **Recompute.** `ingest_messages` ends with `s.changed.notify_one()`. `AppState::run_snapshot_loop` in `crates/seikika-api/src/state.rs` wakes on `changed` and debounces (default 500 ms, at most `max_lag` 5 s). It then calls `recompute_and_publish`, which works as follows:
   - It takes `store.snapshot()` and runs `FullRecompute.recompute` on a blocking task.
   - On the leader, it publishes the snapshot and tuple delta to Postgres (`pg.publish`).
   - It syncs tuples to OpenFGA (`authz.sync_tuples`).
   - It swaps in the new `Published` with `current.send_replace`.

## Q3

**Why these ids.** Decision **D01M7XKEXQH4HG5371629M50E2G** ("A decision is one file, and its id is a ULID") gives the reason. The design is in `docs/design/decision-files.md` §1–2. The decision log used to be one table, `docs/decisions.md`, and the next id was "one more than the highest D number". Two pull requests that each added a decision both took the same number, and appended to the same line. Now each decision is a file named `docs/decisions/<id>.md`. The id is `D` plus a 26-character Crockford-base32 ULID: 48 bits of Unix milliseconds, then 80 random bits. Agents can mint ids without coordinating, and filename order is time order. The design doc compares ULID with ObjectId, xid, KSUID, UUIDv7, TypeID and Snowflake.

`scripts/decision-id.py` mints the ids:
- `new_id` uses 80 bits from `secrets`.
- `encode` and `decode` implement the ULID encoding.
- `--self-test` checks oklog/ulid's `TestAlizainCompatibility` vector. CI runs it in `.github/workflows/ci.yml`, in the step "Decision ids encode as ULIDs".

**Old ids.** D1–D98 were backfilled by `backfill_id(unix_seconds, offset)` in the same script, as described in `decision-files.md` §4 and `docs/decisions/README.md`:
- The random part is zero, so a backfilled id ends in sixteen `0`s and is reproducible.
- The time is the second of the git commit that first added the row, taken from `git log --reverse -p`, not the day heading in the old table.
- The time adds the row's place among the decisions added in that same second, as milliseconds 0–999.

Each file keeps `legacy: Dnn`, and old `Dnn` citations were not rewritten. For example, `D01M7TCJ31R0000000000000000` (`legacy: D1`) decodes to 2026-11-19T17:09:39.000Z, and `…31S…` is the next millisecond. Newer files have random bits and no `legacy` field.

**Checking cited ids.** The bronze rule `cited-decisions-exist` checks that cited decisions exist. It is in the `maintainer-docs` scorecard in `portal/standards/docs.yaml`, with check `entity.missingDecisions.size() == 0`. It was introduced by D84 (`D01M7WMSMS30000000000000000`), amended by D01M7XKEXQH4HG5371629M50E2G, and described in `docs/design/decision-citations.md`.

The data comes from the `workspace` connector, in `crates/seikika-connector-workspace/src/main.rs`:
- `doc_record` sets `missingDecisions` using `missing_decisions(text, &known_decisions(root), rel)`.
- `known_decisions` reads every `docs/decisions/*.md` file, and `decision_file_ids` collects the frontmatter `id` and `legacy` values. `canon_decision` uppercases the ids.
- `missing_decisions` matches `\b(D\d{1,4}|D[Crockford]{26})\b`, case-insensitively, and returns the ids that are not known. It skips `docs/decisions/` and `docs/decisions.md`.

There is a second check in `crates/seikika-connect/src/assessment.rs`. `known_id` is used when deciding whether an assessment's reason cites something real. For a `D01…` id it checks that the file exists, for a short `D` id it looks for a `legacy:` line, and otherwise it looks in `docs/questions.md`. Separately, `check_files` in `crates/seikika-model/src/validate.rs` requires that each waiver's `decision` file exists.

## Q4

**Where the schema is compiled.** `crates/seikika-model/src/schema.rs` embeds the file at compile time: `CONNECTOR_MESSAGES_SCHEMA = include_str!("../../../schema/connector-messages.schema.json")`. `connector_messages()` compiles it once (Draft 2020-12, format validation on), and `errors()` returns `(pointer, message)` pairs. A change therefore takes effect only after the binary is rebuilt; a running or older binary keeps the schema it was built with. The unit test `embedded_schemas_compile` checks that the new schema still compiles.

**Runtime rejection.** This happens in `ingest_messages` in `crates/seikika-api/src/connectors.rs`, which validates every message in a batch:
- **In-image connectors under `seikika serve`:** `supervise` passes batches to `flush` and then `ingest_messages`. A batch with a non-conforming message is rejected whole (422 `invalid-messages`), logged, and dropped.
- **Remote connectors and `seikika push`:** `POST /v1/ingest` (`ingest` in `routes.rs`) calls the same function and returns HTTP 422 to the sender. `seikika push` (`push` in `crates/seikika-cli/src/server.rs`) sends fixtures this way and exits 1 on that response.

**Check-time rejection.**
- **`seikika validate`** (`validate_portal` in `crates/seikika-model/src/validate.rs`) calls `messages::validate_fixtures` in `crates/seikika-model/src/messages.rs`. That function validates every line of `fixtures/*.ndjson` against the schema (`fixture.schema` diagnostics). `seikika serve` also calls `validate_portal` before it starts and refuses to start on errors. The local read and test commands that use `portal::load` in `crates/seikika-cli/src/portal.rs` do the same.
- **`seikika connector test`** runs the conformance kit, `conformance::run` in `crates/seikika-connect/src/conformance.rs`. It validates the `register` message ("register message is valid"), every resync message ("every resync message is valid") and the refresh messages against the same compiled schema.
- **The Python oracle**, `examples/reference-portal/tools/validate.py`, reads `schema/connector-messages.schema.json` from disk, not from the binary. It validates fixture lines and the protocol transcripts in `protocol/*.ndjson`. CI runs it (`python tools/validate.py`), and `scripts/validate-parity.py` compares it with `seikika validate`.

**Paths that do not validate.** I found no schema check on these:
- `seikika report` reads in-image connectors with its own `resync` in `crates/seikika-cli/src/report.rs` and passes the result to `MemStore::ingest`.
- `seikika connector drive` uses `conformance::drive_resync`.
- `Store::ingest` itself does not validate.

`maintainer-schema` (`crates/seikika-connector-workspace/src/specs.rs`) only scores the schema file's metadata and whether it is wired in. It does not validate messages.

## Q5

1. **CLI.** `Command::Get { id, fields, remote }` is defined in `crates/seikika-cli/src/main.rs`. `Remote` carries:
   - `--url`, also `SEIKIKA_URL`, default `http://127.0.0.1:8080`;
   - `--token`, also `SEIKIKA_TOKEN`;
   - `--as`, also `SEIKIKA_AS`;
   - `-o md|json|yaml`.

   `read_call` maps the command to the MCP tool `"get"` with arguments `{"id", "fields"}`. `main` then calls `client::tool(&remote.target(), tool, args, &remote.output)`.
2. **Transport.** `tool` in `crates/seikika-cli/src/client.rs` sends one JSON-RPC 2.0 `tools/call` as an HTTP `POST {url}/mcp`. `Target::request` adds `Authorization: Bearer <token>` and, for `--as`, `x-seikika-user`. The CLI prints `content[0].text` (Markdown) or `structuredContent` (JSON or YAML). It exits 1 when `isError` is set, and exits 2 on HTTP 401 or a connection failure.
3. **MCP handler.** In `crates/seikika-api/src/mcp.rs`, `handle` (the route `POST /mcp` in `routes.rs`) first calls `state.caller(&headers)`, so every request needs an identity. `run_tool` then handles `"get"` by building `/v1/entities/{id}[?fields=…]` and calling `call_rest`. `call_rest` copies the caller's headers onto an in-process request and sends it through `router(state).oneshot(req)`. The same REST handler and authorization therefore apply. A non-2xx status becomes an `isError` result.
4. **REST handler.** `get_entity` in `crates/seikika-api/src/routes.rs` works through these steps:
   - It calls `s.caller(&headers)` again.
   - `snapshot_for` picks the published snapshot, or waits up to 10 s if `min_snapshot` is set.
   - `s.visibility(&caller, &[id])` gives the caller's visibility for the entity.
   - `s.shape_entity(&p, &caller, &id, vis, fields)` builds the entity, which is returned only when `vis.view` is true.

   It adds `inbound` and `outbound` (`reads::outgoing`) and `snapshot`. `with_snapshot` sets the `x-seikika-snapshot` header. If the entity does not exist or `view` is false, the response is the same 404 `not-found`.
5. **Identity.** `AppState::caller` in `crates/seikika-api/src/access.rs` calls `Authenticator::authenticate` in `crates/seikika-authz/src/authn.rs`. Methods are tried in this order:
   - an identity-aware proxy JWT (`iap-google` or `cloudflare-access` modes);
   - a Bearer `tbx_…` personal access token, looked up as a static hash or through `PatLookup`, which is `PgPats` in the server;
   - a Bearer JWT, accepted only in `oidc` mode;
   - `x-seikika-user`, trusted only in `dev` mode.

   `user_for` maps the subject to a `user:` entity, matching by email or by login. `is_admin` checks `admin` on `portal:<name>` in OpenFGA.
6. **Visibility.** This is in `crates/seikika-api/src/access.rs`, which the module doc calls "the one policy enforcement point".
   - `visibility` batches OpenFGA `can_view` and `can_view_details` checks through `Authorizer::batch_check` (`OpenFga::batch_check` in `crates/seikika-authz/src/openfga.rs`).
   - Types the model does not cover are visible by default (D15), and admins get details.
   - With no Authorizer (dev mode without the `openfga` binary), everything is visible and every caller is admin.
   - `shape_entity` reads the entity from the in-memory published snapshot, `p.derived.entities` (the `Published` from the snapshot loop), not from the store. It replaces fields whose kind `facet` needs more access with `redacted(...)` markers: `operator` needs admin, and other non-public facets need `can_view_details`.
