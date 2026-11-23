# Answers

## Q1

**Outcome.** A rule whose data is missing scores `unknown`, not `fail`. The governing decision is `D01M7Y5VWKW5TDS0VQWZFPRQ27N` ("Missing connector data is unknown, and it does not grant a level that has no data", `docs/decisions/D01M7Y5VWKW5TDS0VQWZFPRQ27N.md`). It amends D70 (`docs/decisions/D01M7VZ2DPR0000000000000000.md`, which already said rules over missing tool data are `unknown`, never a regression, and that `--ci`/`--update` refuse them). It relies on D86 (`D01M7WMSMS50000000000000000.md`, repository level = lowest constituent, ignoring unknown rules).

- `crates/toolbox-derive/src/lib.rs`, `score()` evaluates each rule's CEL `check`. `Ok(true)` gives `pass`, `Ok(false)` gives `fail`, and an evaluation error (`Err(_)`) gives `unknown`. The decision drops the `has()` guard, so a missing field becomes a CEL error and therefore `unknown`. A `fail` with a matching waiver becomes `waived`.
- `evidence()` (same file) attaches `["no data from connector <src>"]` to an `unknown` rule that names `dataFrom` (`Rule::data_from`).

**Level.** `level_ignoring_unknown()` (`toolbox-derive/src/lib.rs`) computes the entity's level:

- If every rule is `unknown`, it returns `None`, and `score()` displays `none` (`unwrap_or_else(|| "none")`).
- Otherwise it climbs `sc.levels` in order. A level whose rules are all `unknown` stops the climb, so a level with no data is not granted. At any other level, every *known* rule must pass or be waived (`counts_as_pass`). Unknown rules at that level are ignored. The first level that fails stops the climb.

For the repository rollup, `constituent_level()` and `constituent_summaries()` (same file) use `level_ignoring_unknown` too. A constituent whose rows are all unknown (`None`) is skipped, so it drops out of the rollup instead of making the repository `none` (D86, D01M7Y5VWKW5TDS0VQWZFPRQ27N). `/demo/` fixture ids and `skipped` rows are also excluded.

**Ratchet (`toolbox report --baseline`)**, in `crates/toolbox-cli/src/report.rs`:

- **How each outcome is recorded.** `results()` records `fail` rules under `failing` (with a measure or evidence count) and `unknown` rules under a separate `unknown` list. A code comment there says unknown is "Never a regression or an improvement".
- **A failing rule.** In `compare_entity()`, a rule that newly appears in `failing` is a **regression** ("`<rule> now fails`"), and so is one whose evidence count grew (`evidence_change()`). A rule that leaves `failing` is an improvement.
- **An unknown rule.** A rule that becomes unknown is only reported under `other` ("`<rule> unknown now (its data is missing)`"). A rule that goes from unknown to failing is also `other` ("known again: fails"), not a regression. A rule that goes from failing to unknown is *not* counted as an improvement: the improvement filter excludes rules in `nu`.
- **The gate.** `decide()` makes regressions give `result = "regressed"` and exit code 1. Unknowns are counted separately:
  - `excused_rules()` treats a rule as an expected unknown when its `dataFrom` connector is in `--skip`. `unexpected_unknowns()` counts the rest.
  - If any unexcused unknowns exist and `--ci`, `--update` or `--fail-on-unknown` is set, `decide()` returns `result = "unknown"` with exit code 1 and does not write the baseline (comment: "A connector that ran and still produced no data must not be written into the baseline").
  - Otherwise the unknowns are reported as "not compared".
  - `unknown_count()` feeds `Verdict.unknown`, and the `toolbox-verdict/v1` file carries `unknownRules` (D96, `D01M7XJA1FG0000000000000000.md`).

The difference: a failing rule can regress the ratchet. An unknown rule never regresses or improves it, but in `--ci`/`--update` it blocks the run unless its connector was deliberately skipped. Example from D01M7Y5VWKW5TDS0VQWZFPRQ27N: `ci-lint` is not skipped, so a missing linter fails `--ci`.

## Q2

All of this is in `crates/toolbox-api` unless another path is given.

1. **Start-up.** `toolbox serve` (`crates/toolbox-cli/src/server.rs`, `serve()`) first runs `toolbox_model::validate_portal` on the portal, builds the store (`PgStore` or an in-memory `SharedMem`) and calls `toolbox_api::serve()` (`src/lib.rs`). That function spawns `AppState::run_snapshot_loop`, `connectors::freshness_loop` and `connectors::spawn_in_image`.
2. **Spawn.** `spawn_in_image()` (`src/connectors.rs`) does nothing when `TOOLBOX_CONNECTORS=off`. Otherwise, for each manifest connector with `runtime == "in-image"`, it resolves the binary with `binary_for()`:
   - `builtin:<name>` becomes `toolbox-connector-<name>`, looked up in `TOOLBOX_CONNECTORS_DIR`, then next to the executable, then on `PATH`;
   - `github:…@version` becomes the installed release.

   It resolves `${env:…}`/`${secret:…}` with `resolve_config()`, then starts `supervise()`.
3. **Transport: stdio, NDJSON, one JSON message per line.** `supervise()` spawns the child with `TOOLBOX_CONNECTOR_CONFIG` in its environment and piped stdin and stdout.
   - A writer task pulls commands from the hub (`toolbox_connect::Hub::take`, `crates/toolbox-connect/src/hub.rs`) and writes them to the child's stdin.
   - The reader reads stdout line by line. Lines that are not JSON are skipped silently. `log` messages go to tracing and are not ingested.
   - Everything else is batched and flushed at a control boundary (`register`, `complete`, `snapshot_complete`), at 500 messages, or after 100 ms of quiet.
   - If the process exits, it is restarted with backoff (1 s doubling to 30 s).
   - On the connector side, `toolbox_connect::sdk::run_stdio` (`crates/toolbox-connect/src/sdk.rs`) sends `register`, `ack` and `complete`.
4. **Ingest.** `flush()` calls `ingest_messages()` (`src/connectors.rs`). This is the same function `POST /v1/ingest` (`routes.rs`, `ingest()`) uses for remote connectors. Its checks:
   - the connector must be declared in the manifest, otherwise `403 unknown-connector`;
   - every message is validated against the embedded `schema/connector-messages.schema.json` (`toolbox_model::schema::errors(toolbox_model::schema::connector_messages(), m)`);
   - every `record`/`delete` kind must be in that connector's declared `records`.

   Any error rejects the **whole batch** with `422 invalid-messages`. For an in-image connector, `flush()` only logs "connector batch rejected", so those records are dropped.
5. **Store.** Only if the batch contains a `record` or `delete` does `s.store.ingest(connector, &messages)` run, which creates a new input version. Control-only batches just read `input_version()`.
   - `PgStore::ingest` (`crates/toolbox-store/src/pg.rs`) runs one transaction: it creates a new version, upserts `toolbox.raw_records` on `(connector, kind, external_id)`, applies deletes, and records counts and `command_id` in `toolbox.input_versions`.
   - `MemStore::ingest` (`crates/toolbox-store/src/lib.rs`) upserts or removes, then bumps `version`.
6. **Control bookkeeping.** `Hub::on_messages()` handles the control side:
   - it records registrations, acks and the external ids seen per command;
   - on `complete` it sets the command status and `last_read`;
   - on `snapshot_complete` for a command without errors, it returns `(kind, seen)`.

   `reconcile()` then deletes records of that kind that the resync no longer reported. The safety valve: if the deletes would remove at least 90% of a kind's records (`DELETION_THRESHOLD = 0.9`, with more than one record), they are held and logged. A `register` triggers `initial_resync()`, which enqueues resyncs for declared kinds never read.
7. **Recompute.** `ingest_messages()` ends with `s.changed.notify_one()`. `AppState::run_snapshot_loop()` (`src/state.rs`) wakes up, debounces (500 ms of quiet, at most `max_lag` 5 s, from `Config`) and calls `recompute_and_publish()`. That function:
   - takes `store.snapshot()` and runs `FullRecompute.recompute(&manifest, &inputs)` (`toolbox-derive`) on a blocking thread;
   - on the advisory-lock leader with Postgres, publishes the snapshot and tuple delta (`pg.publish`);
   - syncs tuples to the in-memory OpenFGA (`authz.sync_tuples`);
   - swaps the new `Published` in with `current.send_replace`.

There is no per-record check against the manifest's record data schemas on this path. Mapping failures inside derive surface as non-fatal `map.error` diagnostics (`toolbox-derive/src/lib.rs`). I did not find any further record-level validation between ingest and derive.

## Q3

**Why ULIDs.** Decision `D01M7XKEXQH4HG5371629M50E2G` ("A decision is one file, and its id is a ULID", amends D84) and `docs/design/decision-files.md` §1 explain the change. The old log was one table, `docs/decisions.md`, and the next id was "one more than the highest D number". Two pull requests that each added a decision took the same number and conflicted on the same appended line.

Now each decision is its own file, `docs/decisions/<id>.md`. The id is `D` plus a 26-character ULID: 48 bits of Unix milliseconds plus 80 random bits, in Crockford base32. Agents can mint ids without a shared counter, and filename order is time order. `decision-files.md` §2 compares this against ObjectId, xid, KSUID, UUIDv7, TypeID and Snowflake. `scripts/decision-id.py` mints ids:

- `new_id()` uses 80 bits from `secrets`;
- `encode()`/`decode()` implement the encoding;
- `--self-test` checks oklog/ulid's `TestAlizainCompatibility` vector `01ARYZ6S410000000000000000`, and CI runs it (`.github/workflows/ci.yml`).

**Old ids.** D1–D98 were backfilled. Each file keeps `legacy: Dnn`, and old `Dnn` citations were not rewritten (`docs/decisions/README.md`; `decision-files.md` §4). Their timestamps come from `backfill_id(unix_seconds, offset)` in `scripts/decision-id.py`:

- the time is the **second of the commit that first added the row** (found with `git log --reverse -p`), plus the row's place among decisions added in that same second as a millisecond offset (0–999);
- the random part is ten zero bytes, which is why the id ends in sixteen `0`s and is reproducible.

The old day headings were deliberately not used. D1–D14 are headed 2026-11-18, but their commit is 2026-11-19T17:09:39Z (§4). Example: D1 is `D01M7TCJ31R0000000000000000` and D2 is `D01M7TCJ31S0000000000000000`, both `decided: 2026-11-19T17:09:39Z`. Their ids decode to …39.000 and …39.001.

**The existence check.** The `cited-decisions-exist` rule (bronze) on the `maintainer-docs` scorecard, in `portal/standards/docs.yaml`, checks `entity.missingDecisions.size() == 0`, with the missing ids as evidence. It was introduced by D84 (`D01M7WMSMS30000000000000000.md`, then checking `| Dnn |` table rows) and amended by D01M7XKEXQH4HG5371629M50E2G. It is documented in `docs/design/decision-citations.md` §3.

The `workspace` connector (`crates/toolbox-connector-workspace/src/main.rs`) computes the `missingDecisions` field:

- `doc_record()` sets `"missingDecisions": missing_decisions(&text, &known_decisions(&self.root), rel)`.
- `known_decisions()` reads every `docs/decisions/*.md`, and `decision_file_ids()` takes `id` and `legacy` from the frontmatter.
- `missing_decisions()` finds citations with the regex `(?i)\b(D\d{1,4}|D[0-9A-HJKMNP-TV-Z]{26})\b`. These are legacy `D` plus 1–4 digits, or a ULID id; `canon_decision()` uppercases ULIDs. The function returns those not in the known set, and skips `docs/decisions.md` and `docs/decisions/` itself.

The scorecard is enforced through the `toolbox report` ratchet on the T4T portal, where a new failure is a regression. Fixtures and tests are `portal/fixtures/workspace.ndjson` (`D999` on `doc:docs/v1-spec.md`) and `portal/tests/standards.test.yaml`.

## Q4

**Which code enforces it.** The Rust binary does not read the file at run time. `crates/toolbox-model/src/schema.rs` embeds it with `include_str!` as `CONNECTOR_MESSAGES_SCHEMA` and compiles it once (Draft 2020-12, formats on) in `connector_messages()`. `errors()` turns violations into `(pointer, message)` pairs. A changed schema is therefore enforced by Rust only after the binaries are rebuilt; the module doc says "A Toolbox version validates against the schemas it was built with". The `embedded_schemas_compile` test in that file, and the `expect("embedded schema compiles")` panic, catch a schema that no longer compiles.

**Run time (server).** `ingest_messages()` (`crates/toolbox-api/src/connectors.rs`) validates every message with `schema::connector_messages()`. A non-conforming message rejects the whole batch with `422 invalid-messages`. This one function is reached from two places:

- **in-image connectors under `toolbox serve`**: `supervise()` then `flush()`, which only logs "connector batch rejected";
- **`POST /v1/ingest`** (`routes.rs`, `ingest()`), used by remote connectors through `toolbox_connect::sdk::run_remote`/`push`, by `toolbox push` (`crates/toolbox-cli/src/server.rs`, `push()`) and by `toolbox assess submit`. The HTTP caller gets the 422.

**Check time.**

- **`toolbox validate`**: `validate_portal()` (`crates/toolbox-model/src/validate.rs`) calls `messages::validate_fixtures()` (`crates/toolbox-model/src/messages.rs`), which validates every `fixtures/*.ndjson` line and emits `fixture.schema` diagnostics. Other paths also go through `validate_portal`, so they refuse invalid fixtures too:
  - `toolbox serve` at start-up;
  - local commands via `crate::portal::load()` (`crates/toolbox-cli/src/portal.rs`).
- **`toolbox connector test`**: the conformance kit `toolbox_connect::conformance::run()` (`crates/toolbox-connect/src/conformance.rs`) checks that every message from the live connector validates (check 2 in its module doc). CI runs it for `workspace`, `surfaces` and `ci-lint` (`.github/workflows/ci.yml`).
- **Python reference validator**: `examples/reference-portal/tools/validate.py` loads `schema/connector-messages.schema.json` **from disk**, so it sees the change without a rebuild. It checks fixture lines and the protocol transcripts in `protocol/*.ndjson`. `scripts/validate-parity.py` compares it with `toolbox validate` in CI, so a Rust binary built from an older schema would show up as a parity difference.
- **`maintainer-schema` scorecard**: the workspace connector's `specs.rs` analyses the schema file (draft, `$id`, `wired`, …). Its test `the_schemas_in_this_repo_are_draft_2020_12_and_wired_in` asserts the file stays Draft 2020-12 and wired into `schema.rs`. This checks the schema itself, not messages.

**Paths that do not validate.**

- `toolbox report`'s local connector run (`report.rs`, `resync()` and `inputs()`) parses connector stdout and ingests `record`/`delete` messages into a `MemStore` without schema validation.
- `report::live()` uses `manifest_from()` (manifest validation only), so `--fixtures` is not message-validated there either.
- The SDK does not validate the messages it sends.

These observations come from reading the code; I found no other validation call.

## Q5

1. **CLI.** `toolbox get <id> [--fields …] --url <server>` parses into `Command::Get { id, fields, remote: Remote }` (`crates/toolbox-cli/src/main.rs`). `Remote` holds:
   - `--url` (env `TOOLBOX_URL`, default `http://127.0.0.1:8080`);
   - `--token` (env `TOOLBOX_TOKEN`);
   - `--as` (env `TOOLBOX_AS`, dev servers only);
   - `-o`.

   `main()` calls `read_call()`, which maps `Get` to the MCP tool `"get"` with `{"id", "fields"}` (see `CLI_MCP_TOOLS`), then calls `client::tool(&remote.target(), …)`.
2. **Transport.** `client::tool()` (`crates/toolbox-cli/src/client.rs`) sends one HTTP `POST {url}/mcp` with the JSON-RPC 2.0 body `{"method": "tools/call", "params": {"name": "get", "arguments": …}}`. `Target::request()` adds the identity as `Authorization: Bearer <token>` when `--token` is set and `x-toolbox-user: <as>` when `--as` is set. The client prints `content[0].text` (Markdown) or `structuredContent` (`-o json|yaml`). It exits 2 on HTTP 401 and 1 on `isError`. This follows D61 (`D01M7VQJXMR0000000000000000.md`): CLI reads call the MCP tools, which call REST in-process, so CLI, MCP, REST and UI share one authorization and facet-shaping path.
3. **Server, MCP.** `routes::router` maps `/mcp` to `mcp::handle()` (`crates/toolbox-api/src/mcp.rs`). `handle()` first authenticates with `state.caller(&headers)`; a failure returns the 401 problem. For `tools/call` it calls `run_tool()`. For `"get"`, `run_tool()` builds `/v1/entities/{id}[?fields=…]` and calls `call_rest()`. `call_rest()` copies the caller's original headers, minus content-length and content-type, onto an in-process request and runs it through `router(state).oneshot(req)`. There is no second network hop.
4. **Server, REST.** `/v1/entities/{id}` maps to `get_entity()` (`crates/toolbox-api/src/routes.rs`), which applies identity and visibility in this order:
   - **Identity**: `s.caller(&headers)` (`crates/toolbox-api/src/access.rs`, `AppState::caller`) calls `Authenticator::authenticate()` (`crates/toolbox-authz/src/authn.rs`). Methods are tried in order:
     1. an identity-aware-proxy JWT (`iap-google`, `cloudflare-access`);
     2. a bearer token: a `tbx_…` PAT, looked up by hash in static PATs or the PAT store, or an OIDC JWT in `oidc` mode;
     3. in `dev` mode only, the `x-toolbox-user` header.

     Then `user_for()` maps the subject to a `user:` entity (by email for email subjects), and `is_admin()` checks `admin` on `portal:<name>` in OpenFGA.
   - **Snapshot**: `snapshot_for()` picks the published snapshot, honouring `min_snapshot` (read-your-writes). It returns 503 if none is published yet.
   - **Visibility**: `s.visibility(&caller, &[id])` (`access.rs`) batch-checks `can_view` and `can_view_details` in OpenFGA (`authz.batch_check`). If the model lacks a relation for that type, `view` defaults to true (public by default, D15) and `details` to `caller.admin`. Admins always get details. With no Authorizer, everything is visible. When OpenFGA is unavailable it returns 502 `authz-unavailable`.
   - **Read and facet shaping**: `s.shape_entity(&p, &caller, &id, vis, fields)` (`access.rs`) reads the entity from `p.derived.entities` (the published `Derived` snapshot) and filters `fields`. Fields whose facet requires more access are replaced with `redacted()` markers (`facet_needs()`: `operator` needs admin, any non-public facet needs `can_view_details`). The result is `.filter(|_| vis.view)`: an entity the caller may not view returns **404 `not-found`**, the same response as a missing entity.
   - `get_entity()` then adds `inbound`, `outbound` (from `reads::outgoing()`) and `snapshot`, and sets the `x-toolbox-snapshot` header (`with_snapshot()`).

As far as I can see from the code, `inbound` and `outbound` list related entity ids without a per-target visibility check. I have not verified whether that is intended.
