# Answers

## Q1

**Outcome.** The rule scores `unknown`. In `crates/seikika-derive/src/lib.rs`, the scorecard scoring method `score` (an inherent method on the evaluator, documented "One scorecard for the entity bound in `ctx`") maps the rule's CEL `check` result as `Ok(true) => "pass"`, `Ok(false) => "fail"`, `Err(_) => "unknown"`. Missing connector data shows up as a CEL error, because a missing field with no `has()` guard is an error. If `appliesTo` itself errors, every rule of that card becomes `unknown`. Waivers only turn a `fail` into `waived`, so an `unknown` is never waived. When the rule declares `dataFrom` (`Rule.data_from` in `crates/seikika-model/src/manifest.rs`), `evidence` records `["no data from connector <src>"]` for the unknown rule.

**Decision.** This is D01M7Y5VWKW5TDS0VQWZFPRQ27N, "Missing connector data is unknown, and it does not grant a level that has no data" (`docs/decisions/D01M7Y5VWKW5TDS0VQWZFPRQ27N.md`). It amends D70 (`docs/decisions/D01M7VZ2DPR0000000000000000.md`, `legacy: D70`).

**Effect on the level.** `level_ignoring_unknown` in `crates/seikika-derive/src/lib.rs` computes the level as follows:
- If every rule is `unknown`, it returns `None`. `score` then displays the level as `none`, and `constituent_level` / `constituent_summaries` drop the entity from the repository rollup.
- Otherwise it climbs through `sc.levels` in order. A level with no rules is passed over.
- A level whose rules are all `unknown` stops the climb, so that level is not granted.
- At a level with at least one known rule, every known rule must be `pass` or `waived` (`counts_as_pass`). `unknown` rules at that level are ignored. One known failure stops the climb.

**The ratchet (`seikika report --baseline`, `crates/seikika-cli/src/report.rs`).**
- `results` writes each entity's row with `level`, a `failing` map (rule → measure or evidence count) and a separate `unknown` list. The code comment says an unknown is "Never a regression or an improvement".
- `compare_with` → `compare_entity` compares the rows:
  - A rule that is newly `unknown` is reported in `other` as "`<rule>` unknown now (its data is missing)".
  - A rule that was unknown and now fails is reported in `other` as "known again: fails", not as a regression.
  - A rule that failed and is now unknown is not counted as an improvement (the filter is `!nu.contains(r)`).
  - A level change while the failing set stays the same is reported in `other` (`level X → Y`). So a level drop caused only by unknowns is not a regression. The test `unknown_outcomes_are_neither_worse_nor_better` covers this: `gold → none` lands in `other`.
  - By contrast, a newly failing rule is a regression ("`<rule>` now fails"), and so is a failing rule whose evidence or measure grew (`evidence_change`).
- `decide` checks regressions first. Any regression gives result `regressed` and exit 1.
- Unknowns are then gated separately:
  - `excused_rules` collects every rule whose `dataFrom` connector is in `--skip`.
  - `unexpected_unknowns` counts the unknowns that are not excused.
  - Under `--ci`, `--update` or `--fail-on-unknown`, any unexpected unknown gives result `unknown`, exit 1, and the baseline is not written. The comment says a connector that ran and produced no data "must not be written into the baseline".
  - Excused unknowns are recorded and not compared. The summary then reads "N rule outcome(s) unknown (excused by --skip, or not gating), not compared".
- `unknown_count` feeds `unknownRules` in the `--verdict` JSON (`verdict_json`).
- The decision gives the CI consequence. `--skip runner,github-actions` excuses `tests-pass`, `clippy-clean` and the run-history rules. Zizmor and actionlint rules are `dataFrom: ci-lint`, which CI does not skip, so a missing linter still fails `--ci`.

## Q2

**Startup.**
1. `serve` (`crates/seikika-cli/src/server.rs`) validates the portal with `seikika_model::validate_portal`.
2. It picks the store: `PgStore` with `--database-url`, otherwise `SharedMem(MemStore)`, which `load_fixtures` can preload under `--fixtures`.
3. It builds `AppState` with `seikika_api::app_state_with` and calls `seikika_api::serve`.
4. `seikika_api::serve` (`crates/seikika-api/src/lib.rs`) spawns `run_snapshot_loop` and `connectors::freshness_loop`, and calls `connectors::spawn_in_image`.

**Supervision and transport** (all in `crates/seikika-api/src/connectors.rs`):
- `spawn_in_image` does nothing if `SEIKIKA_CONNECTORS=off`. Otherwise it starts one `supervise` task for each manifest connector with `runtime: in-image`.
  - It resolves the binary with `binary_for`. `builtin:<name>` becomes `seikika-connector-<name>`, looked up in `SEIKIKA_CONNECTORS_DIR`, next to the binary, then on `PATH`. A `github:` spec points at the cache that `seikika install` fills.
  - It resolves the config with `resolve_config`, which fills `${env:…}` and `${secret:…}` placeholders.
- `supervise` runs the connector as a child process. It passes `SEIKIKA_CONNECTOR_CONFIG` in the environment, pipes stdin and stdout, and sets `kill_on_drop`. It restarts the child with exponential backoff from 1 s up to 30 s.
- **The transport is newline-delimited JSON over stdio.**
  - A writer task sends the hub's queued commands (`hub.take`) to the child's stdin.
  - The reader parses each stdout line as JSON and drops lines that are not JSON.
  - `log` messages go to tracing.
  - Other messages are batched. A batch is flushed at a control boundary (`register`, `complete`, `snapshot_complete`), at 500 messages, after 100 ms of quiet, and when the process exits.
- `flush` calls `ingest_messages`. If the batch is rejected, it logs "connector batch rejected" and drops the batch.

**Checks in `ingest_messages`.** Remote connectors use the same function through `POST /v1/ingest` (`routes::ingest`, with the connector identified by `connector_of`).
1. The connector must be declared in the manifest. Otherwise the result is 403 `unknown-connector`.
2. Every message must validate against the compiled connector-message JSON Schema (`seikika_model::schema::errors(seikika_model::schema::connector_messages(), m)`).
3. Every `record` or `delete` message's `kind` must be in that connector's declared `records`.
4. Any error rejects the whole batch with 422 `invalid-messages`.

**Store write and recompute.**
- Only a batch that contains `record` or `delete` messages calls `store.ingest`, which creates a new input version. Control-only batches just read `input_version`.
  - `MemStore::ingest` (`crates/seikika-store/src/lib.rs`) upserts or removes keys `(connector, kind, externalId)` and increments the version.
  - `PgStore`'s `ingest` (`crates/seikika-store/src/pg.rs`) runs in one transaction. It creates a version with `new_version`, runs `INSERT … ON CONFLICT DO UPDATE` into `seikika.raw_records`, and runs `DELETE` for deletes.
- `hub.on_messages` (`crates/seikika-connect/src/hub.rs`) tracks the external ids seen per command. On `snapshot_complete` for a command with no errors, it returns the kind for `reconcile`.
- `reconcile` deletes the records the resync no longer reported. It holds the deletes and logs instead when they would remove at least 90% of more than one existing record (`DELETION_THRESHOLD`).
- A `register` triggers `initial_resync`.
- Finally, `s.changed.notify_one()` runs.
- `run_snapshot_loop` (`crates/seikika-api/src/state.rs`) wakes on `changed` or on `config.poll`. When the published `input_version` differs from the store's, it debounces with `config.debounce`, bounded by `config.max_lag`. It then calls `recompute_and_publish`:
  1. `store.snapshot()`.
  2. `FullRecompute.recompute` in `spawn_blocking`.
  3. With Postgres, and only when this instance is the advisory-lock leader (`try_lead`), it publishes the snapshot and tuple delta with `pg.publish`.
  4. It syncs tuples to OpenFGA (`authz.sync_tuples`).
  5. It publishes the new `Published` snapshot with `current.send_replace`.

## Q3

**Why these ids.** D01M7XKEXQH4HG5371629M50E2G ("A decision is one file, and its id is a ULID") and `docs/design/decision-files.md` §1–2 explain the format.
- The old log was one table, `docs/decisions.md`, where the next id was "one more than the highest D number". Two PRs both took that number, and git cannot merge two appends at the same line.
- Now each decision is `docs/decisions/<id>.md`. The id is `D` plus a 26-character ULID: 48 bits of Unix milliseconds and 80 random bits, in Crockford base32. Ids are lexicographically time-ordered, so filename order is time order, and agents can mint ids without coordinating.
- §2 compares ObjectId, xid, KSUID, UUIDv7, TypeID and Snowflake, and gives the reason each lost. `docs/decisions/README.md` restates the format.
- `python3 scripts/decision-id.py` mints a new id with `new_id`, which reads 10 bytes from `secrets`.
- `--self-test` (`self_test`) checks oklog/ulid's vector `01ARYZ6S410000000000000000`. CI runs it (`.github/workflows/ci.yml`). It checks the encoding only, not citations.

**The old ids** (`decision-files.md` §4, "Backfill"; `backfill_id` in `scripts/decision-id.py`):
- D1–D98 were copied out of the table. Each file keeps `legacy: Dnn`, and there are 98 files whose names end in sixteen zeros.
- `decided` is the commit that first added the row, found with `git log --reverse -p`. The old day headings were not used. For example, D1–D14 are headed 2026-11-18, but their commit is 2026-11-19T17:09:39Z.
- The id time is that commit's second, plus the row's place among the decisions added in the same second, in milliseconds: `unix_seconds * 1000 + offset`, with offset 0–999. The largest commit added 32 decisions, so the offset stays inside the second.
- The random part is zero, so the id is reproducible and ends in sixteen `0`s (the 80 random bits).
- Decoding confirms this. D1 = `D01M7TCJ31R…` is 2026-11-19T17:09:39.000Z, and D2 = `…31S…` is the next millisecond.
- Existing `Dnn` citations were not rewritten. New decisions have no `legacy` field.

**What checks citations.** The `cited-decisions-exist` rule (bronze) of the `maintainer-docs` scorecard, in `portal/standards/docs.yaml`. Its check is `entity.missingDecisions.size() == 0`, and the evidence is `entity.missingDecisions`.
- That field is computed by the workspace connector (`crates/seikika-connector-workspace/src/main.rs`) for each `doc` record: `"missingDecisions": missing_decisions(&text, &known_decisions(&self.root), rel)`.
  - `known_decisions` reads `docs/decisions/*.md`.
  - `decision_file_ids` takes `id` and `legacy` from each file's frontmatter.
  - `missing_decisions` matches `(?i)\b(D\d{1,4}|D[0-9A-HJKMNP-TV-Z]{26})\b`. `canon_decision` uppercases ULID citations. Files under `docs/decisions/` and `docs/decisions.md` are skipped, because the log is the index.
- The documents scanned are the connector's `docs` globs in `portal/seikika.yaml`.
- History: the rule comes from D84 (`docs/decisions/D01M7WMSMS30000000000000000.md`), which originally matched `| Dnn |` rows in `docs/decisions.md`. The ULID decision amends it to check against the files.
- It is enforced through `seikika report portal` and the CI ratchet. The tests are in the `decision_ids` module of the same file.

## Q4

**Compiled in.** `crates/seikika-model/src/schema.rs` embeds the file with `include_str!` as `CONNECTOR_MESSAGES_SCHEMA`. `connector_messages()` compiles it once, through a `OnceLock`, as Draft 2020-12 with format validation on. `errors()` returns `(pointer, message)` pairs.
- A changed schema takes effect only in a rebuilt binary ("A Seikika version validates against the schemas it was built with").
- An embedded schema that does not compile panics on first use. The `embedded_schemas_compile` test catches that.

**Runtime rejection.** `ingest_messages` in `crates/seikika-api/src/connectors.rs` validates every message of every batch. Any error rejects the whole batch (422 `invalid-messages`). This covers every transport:
- In-image connectors under `seikika serve`: `supervise` → `flush` → `ingest_messages`. A rejected batch is logged and dropped.
- Remote connectors: `POST /v1/ingest` (`routes::ingest`). The caller receives the 422 problem.
- `seikika push` (`server.rs` `push`), which posts fixture files to `/v1/ingest`.

**Check-time rejection.**
- `seikika validate` calls `validate_portal` (`crates/seikika-model/src/validate.rs`), which calls `messages::validate_fixtures` (`crates/seikika-model/src/messages.rs`). Every `fixtures/*.ndjson` line is checked, and failures are diagnostics with code `fixture.schema`.
  - The same `validate_portal` runs at `seikika serve` startup: an invalid fixture makes it exit 1 before listening.
  - It also runs in `portal::load` (`crates/seikika-cli/src/portal.rs`), which `test`, `derive` and `plan` use.
- `seikika connector test` calls `seikika_connect::conformance::run` (`crates/seikika-connect/src/conformance.rs`). Check 2 of that kit is "every message validates against schema/connector-messages.schema.json". It applies to the `register` message, the resync output and the refresh output. CI runs it for `workspace`, `surfaces` and `ci-lint` (`.github/workflows/ci.yml`).
- Outside the Rust binary, `examples/reference-portal/tools/validate.py` loads `schema/connector-messages.schema.json` directly from disk, so a changed schema applies there without a rebuild. It validates the reference portal's fixtures and its connector-direction `protocol/*.ndjson` messages. `scripts/validate-parity.py` checks that `seikika validate` agrees with it, and CI runs both.
- The `maintainer-schema` facts (`crates/seikika-connector-workspace/src/specs.rs`) score the schema file itself: draft 2020-12, `$id`, no external refs, wired in. They do not check messages.

**Paths that do not validate, as far as I can tell:**
- `seikika report`'s live path: `resync` and `inputs` in `crates/seikika-cli/src/report.rs` put connector output straight into `MemStore::ingest`.
- `report --fixtures`: `live` uses `manifest_from` and `load_fixtures`, not `validate_portal`.
- `seikika connector drive`: I found no schema check in `conformance::drive_resync`, but I did not read all of it.

## Q5

1. **CLI.** `seikika get <id>` is `Command::Get { id, fields, remote }` in `crates/seikika-cli/src/main.rs`.
   - `Remote` provides `--url` (or `SEIKIKA_URL`, default `http://127.0.0.1:8080`), `--token` (`SEIKIKA_TOKEN`), `--as` (`SEIKIKA_AS`) and `-o`.
   - `main` calls `read_call`, which maps the command to the MCP tool `"get"` with arguments `{"id", "fields"}`. `CLI_MCP_TOOLS` records that CLI reads share one authorization path (D61).
   - `main` then calls `client::tool(&remote.target(), …)`.

2. **Transport.** `client::tool` (`crates/seikika-cli/src/client.rs`) sends one JSON-RPC 2.0 `tools/call` as an HTTP `POST {url}/mcp`.
   - `Target::request` adds `Authorization: Bearer <token>` when a token is set, and `x-seikika-user` when `--as` is set.
   - The CLI prints `result.content[0].text` (Markdown) for `-o md`, or `structuredContent` for `-o json` / `-o yaml`.
   - It exits 2 on 401 or a connection error, and 1 when `isError` is true.

3. **MCP handler.** The `/mcp` route (`routes::router`) goes to `mcp::handle` (`crates/seikika-api/src/mcp.rs`).
   - It first authenticates with `state.caller(&headers)`.
   - For `tools/call` it runs `run_tool`. For `"get"`, that builds `/v1/entities/{id}[?fields=…]` and calls `call_rest`.
   - `call_rest` sends the request in-process through `router(state).oneshot(…)`, copying the caller's own headers. MCP therefore goes through the same REST handler and authorization.
   - A non-2xx status becomes an `isError` result. Otherwise `markdown("get", …)` renders the fields with their provenance.

4. **Read.** `get_entity` (`crates/seikika-api/src/routes.rs`):
   - **Identity:** `s.caller(&headers)` again. `AppState::caller` (`crates/seikika-api/src/access.rs`) calls `Authenticator::authenticate` (`crates/seikika-authz/src/authn.rs`), which tries these in order:
     - the IAP or Cloudflare proxy JWT;
     - a bearer `tbx_` personal access token, matched by hash against static PATs or Postgres (`PgPats`);
     - a bearer JWT, accepted only in OIDC mode;
     - `x-seikika-user`, accepted only in dev mode.
     It returns 401 otherwise. `user_for` maps the subject to a `user:` entity (by email for email subjects, else by login), and `is_admin` checks `admin` on `portal:<name>`.
   - **Snapshot:** `snapshot_for` answers from the published snapshot, waiting up to 10 s for `min_snapshot`.
   - **Visibility:** `s.visibility(&caller, &[id])` runs OpenFGA batch checks of `can_view` and `can_view_details` for the entity's type. A type the model does not cover is visible by default (D15), with details for admins only. Without an Authorizer, everything is visible. An OpenFGA failure returns 502 `authz-unavailable`.
   - **Entity read:** `shape_entity` reads `p.derived.entities[id]` from the published `Derived` snapshot and keeps only the requested `fields`. It shapes facets with `facet_needs`: `public` is shown to everyone, `operator` needs admin, and any other facet needs `can_view_details`. A hidden field is replaced by a `redacted(...)` marker.
   - If `vis.view` is false, or the entity does not exist, the response is 404 `not-found`, so existence is not revealed.
   - The handler then adds `inbound`, `outbound` (`reads::outgoing`) and `snapshot`, and sets the `x-seikika-snapshot` header (`with_snapshot`).
   - I am not sure whether the ids in `inbound` / `outbound` are filtered by visibility. `get_entity` itself does not filter them.
