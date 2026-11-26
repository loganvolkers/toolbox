# Answers

## Q1

**Decision.** `docs/decisions/D01M7Y5VWKW5TDS0VQWZFPRQ27N.md` ("Missing connector data is unknown, and it does not grant a level that has no data"). It amends D70 (`docs/decisions/D01M7VZ2DPR0000000000000000.md`, `legacy: D70`).

**Outcome.** The rule scores `unknown`. The decision says `dataFrom` names the connector a rule reads, and that dropping the `has()` guard turns a missing field into a CEL error, which is already `unknown`. In code, `Engine::score` in `crates/seikika-derive/src/lib.rs` (around L728–782) evaluates each rule's `check`:
- `Ok(true)` gives `pass`.
- `Ok(false)` gives `fail`, or `waived` if a waiver matches.
- `Err(_)` gives `unknown`.

For an `unknown` rule that has `data_from`, `evidence()` (same file, around L786) records `["no data from connector <src>"]` as the evidence.

**Effect on the level.** `level_ignoring_unknown` (`crates/seikika-derive/src/lib.rs`, around L1136–1169) computes the level:
- If every rule is `unknown`, it returns `None`. `score` then shows `none`, and `constituent_level` / `constituent_summaries` skip the entity, so it drops out of the repository rollup (the same handling D86 gives an all-unknown constituent).
- Otherwise it climbs the scorecard's levels in order. A level whose rules are all `unknown` stops the climb, so that level is not granted.
- At any other level, only the known rules count. Every one of them must be `pass` or `waived` (`counts_as_pass`). Unknown rules at a mixed level are ignored.

**Ratchet (`seikika report --baseline`), in `crates/seikika-cli/src/report.rs`:**
- `results()` (around L188) writes each entity's `unknown` rules as a separate `unknown` list, apart from `failing`. The comment there says it is "Never a regression or an improvement".
- `compare_entity()` (around L559–598) handles the comparison:
  - A rule that newly becomes unknown goes to `other` ("`<rule>` unknown now (its data is missing)"). It is not a regression.
  - A rule that was unknown and now fails is `other` ("known again: fails"). It is not a regression.
  - A rule that was failing and is now unknown is not counted as an improvement, because of the `!nu.contains(r)` filter.
  - A level change while the failing set is unchanged (for example `gold` → `none` because data vanished) is only `other`.

  The test `unknown_outcomes_are_neither_worse_nor_better` (around L1809) pins this behaviour.
- By contrast, a rule that newly **fails** is pushed to `regressions` ("`<rule>` now fails"), and so is a failing rule whose evidence or measure grew (`evidence_change`). `decide()` (around L1340) turns any regression into the verdict `regressed` with exit code 1.
- Unknowns are gated separately in `decide()`:
  - `excused_rules()` (around L1267) collects the rules whose `dataFrom` connector is in `--skip`. `unexpected_unknowns()` (around L1281) counts the unknown rules that are not excused.
  - If that count is above 0 and `--ci`, `--update` or `--fail-on-unknown` is set, the verdict is `unknown` with exit 1, and the baseline is not written.
  - Otherwise unknowns are reported ("N rule outcome(s) unknown (excused by --skip, or not gating), not compared") and do not fail the run.
  - `unknown_count()` (around L298) supplies the count shown in the summary.

  The decision gives the reasoning: the fast ratchet skips `runner` and `github-actions`, so their rules are expected unknowns. `ci-lint` is not skipped, so a missing linter still fails `--ci`.
- Each run gets its connector data from `live()` and `inputs()` (around L136 and L1235). `inputs()` does not run connectors that are named in `--skip`.

## Q2

**Start-up.**
1. `server::serve` (`crates/seikika-cli/src/server.rs`, around L101) validates the portal and builds the store: `PgStore` with `--database-url`, otherwise an in-memory `MemStore` wrapped in `SharedMem`.
2. It builds `AppState` with `seikika_api::app_state_with` and calls `seikika_api::serve` (`crates/seikika-api/src/lib.rs`, around L63).
3. `seikika_api::serve` spawns `run_snapshot_loop` and `connectors::freshness_loop`, and calls `connectors::spawn_in_image`.

**Supervision and transport** (`crates/seikika-api/src/connectors.rs`):
- `spawn_in_image` (around L249) skips everything when `SEIKIKA_CONNECTORS=off`. For each manifest connector with `runtime == "in-image"`, it resolves the binary with `binary_for` (`builtin:<name>` becomes `seikika-connector-<name>`) and the config with `resolve_config` (`${env:…}` and `${secret:…}`). It then spawns `supervise`.
- `supervise` (around L267) starts the child with `SEIKIKA_CONNECTOR_CONFIG` in its environment. The transport is **stdio, newline-delimited JSON**:
  - A writer task forwards commands from `hub.take(...)` to the child's stdin.
  - The child's stdout is read line by line. Lines that are not JSON are skipped. `log` messages go to tracing.
  - Messages are batched until a control boundary (`register`, `complete` or `snapshot_complete`), 500 messages, or 100 ms of quiet. Then `flush()` runs.
  - If the child exits, it is restarted with exponential backoff, up to 30 s.
- `flush()` (around L345) calls `ingest_messages`. A rejected batch is logged as "connector batch rejected" and dropped.

The HTTP path (`POST /v1/ingest`, `routes::ingest` in `crates/seikika-api/src/routes.rs`, around L565) is used by `seikika push` and remote connectors. It authenticates the connector through `connector_of` (a bearer token, or `x-seikika-connector` in dev mode), parses JSON or NDJSON, and then calls the same `ingest_messages`.

**Checks in `ingest_messages`** (`connectors.rs`, around L25–96):
1. The connector must be declared in the manifest. Otherwise the batch gets 403 `unknown-connector`.
2. Every message is validated against the compiled `schema/connector-messages.schema.json` (`seikika_model::schema::errors(connector_messages(), m)`).
3. Every `record` or `delete` must have a `kind` listed in that connector's `records`.
4. Any error rejects the **whole batch** with 422 `invalid-messages`.

**Store and recompute.**
- If the batch holds a `record` or `delete`, `s.store.ingest(connector, &messages)` writes it and returns a new input version. Batches with only control messages reuse the current `input_version`.
  - `MemStore::ingest` (`crates/seikika-store/src/lib.rs`, around L111) upserts records and deletes by `(connector, kind, externalId)`.
  - `PgStore::ingest` (`crates/seikika-store/src/pg.rs`, around L62) does the same in one transaction, against `seikika.raw_records` and `seikika.input_versions`.
- `hub.on_messages` (`crates/seikika-connect/src/hub.rs`, around L239) records registrations, acks and completions. For each `snapshot_complete` of a command that did not fail, it returns the external ids seen.
- `reconcile` (`connectors.rs`, around L100) deletes records that a full resync no longer reports. If that would remove at least `DELETION_THRESHOLD` (90%) of a kind, it holds the deletes instead.
- A `register` triggers `initial_resync`.
- Finally `s.changed.notify_one()` wakes `AppState::run_snapshot_loop` (`crates/seikika-api/src/state.rs`, around L197). The loop debounces (500 ms by default, at most `max_lag` of 5 s) and then calls `recompute_and_publish`. That function takes `store.snapshot()`, runs `FullRecompute.recompute(&manifest, &inputs)` on a blocking task, and publishes the result. In Postgres mode on the leader it also records the snapshot and tuple delta, syncs tuples to OpenFGA when it is present, and swaps the new `Published` into the `current` watch channel.

## Q3

**Why these ids.** Decision `docs/decisions/D01M7XKEXQH4HG5371629M50E2G.md`, with design `docs/design/decision-files.md` (§1–§2) and `docs/decisions/README.md`:
- The log used to be one table, `docs/decisions.md`, numbered "one more than the highest". Two pull requests that each added a decision took the same number, and git cannot merge two appends at the same line.
- Now each decision is a file, `docs/decisions/<id>.md`. The id is `D` plus a ULID: 48 bits of Unix milliseconds and 80 random bits, in Crockford base32. Two agents can mint ids without a coordinator, and string order is time order.
- §2 compares ULID with ObjectId, xid, KSUID, UUIDv7, TypeID and Snowflake, and explains why ULID won.
- `scripts/decision-id.py` mints ids. `new_id` takes 10 bytes from `secrets`, and `--self-test` (`self_test`) checks the oklog/ulid vector `01ARYZ6S410000000000000000`.

**Old ids and timestamps.** `decision-files.md` §4 ("Backfill") and `backfill_id()` in `scripts/decision-id.py` describe the backfill:
- D1–D98 were copied out of the table, and each file keeps `legacy: Dnn`.
- The id's time is the second of the commit that first added the row (found with `git log --reverse -p`), plus the row's position among the decisions added in that same second, as a millisecond offset (`unix_seconds * 1000 + offset`, with `offset` from 0 to 999). The largest commit added 32 decisions, so every offset stays inside its second.
- The random part is zero, which is why a backfilled id ends in sixteen `0`s and the same inputs always give the same id.
- The old day headings were deliberately not used. For example, D1–D14 are headed 2026-11-18, but they were committed at 2026-11-19T17:09:39Z. The commit time is when the row became shared state, and it orders D1…D98 with no inversions.
- Existing `Dnn` citations were not rewritten. Example: `docs/decisions/D01M7VZ2DPR0000000000000000.md` has `legacy: D70`.

**What checks that a cited id exists.**
- The rule is `cited-decisions-exist` in the `maintainer-docs` scorecard (`portal/standards/docs.yaml`, around L100). It is a bronze rule with `check: entity.missingDecisions.size() == 0` and evidence `entity.missingDecisions`. It was introduced by D84 (`docs/decisions/D01M7WMSMS30000000000000000.md`) and amended by D01M7XKEXQH4HG5371629M50E2G. The design is `docs/design/decision-citations.md`.
- The `missingDecisions` field of each `doc` record comes from the workspace connector, `crates/seikika-connector-workspace/src/main.rs`:
  - `known_decisions()` (around L247) reads the frontmatter `id` and `legacy` from every `docs/decisions/*.md`, using `decision_file_ids()`.
  - `missing_decisions()` (around L266) finds citations with the regex `(?i)\b(D\d{1,4}|D[0-9A-HJKMNP-TV-Z]{26})\b`, normalises them with `canon_decision`, and returns the ones that are not known.
  - `docs/decisions.md` and `docs/decisions/` are exempt.
- The rule is scored by `seikika report portal`, and the `portal/tests/standards.test.yaml` fixture expects a `D999` citation to fail.

## Q4

**Where the schema is compiled.** `crates/seikika-model/src/schema.rs` embeds the file at build time: `CONNECTOR_MESSAGES_SCHEMA = include_str!("../../../schema/connector-messages.schema.json")`. `connector_messages()` compiles it once, lazily (`OnceLock`, Draft 2020-12, with format validation), and `errors()` returns `(pointer, message)` pairs. Because the schema is embedded, a change only takes effect in the Rust code after a rebuild. "A Seikika version validates against the schemas it was built with." If the new schema does not compile, `compile()` panics, and the unit test `embedded_schemas_compile` catches that.

**Runtime rejection (server):**
- `ingest_messages` (`crates/seikika-api/src/connectors.rs`, around L25) validates every message. Any error rejects the whole batch with 422 `invalid-messages`, and nothing is written to the store.
- Every message path goes through it:
  - In-image connectors use stdio: `supervise` collects messages and `flush` calls `ingest_messages`. A rejected batch is logged and dropped.
  - Remote connectors and `seikika push` use `POST /v1/ingest` (`routes::ingest`). There the 422 goes back to the caller, and `push` exits 1.

**Check-time rejection:**
- `seikika_model::messages::validate_fixtures` (`crates/seikika-model/src/messages.rs`) checks every `fixtures/*.ndjson` line against the schema. Failures become `fixture.schema` diagnostics.
- `validate_portal` (`crates/seikika-model/src/validate.rs`, around L29) calls it. Its callers are:
  - `seikika validate` (`validate()` in `crates/seikika-cli/src/main.rs`, around L869);
  - `seikika serve` at start-up (`server::serve`), which refuses to start and exits 1;
  - `portal::load` (`crates/seikika-cli/src/portal.rs`), which `seikika test`, `plan` and `derive` use.
- `seikika connector test` (`connector_test` in `main.rs`, which calls `seikika_connect::conformance::run` in `crates/seikika-connect/src/conformance.rs`) fails these checks on schema errors: "register message is valid", "every resync message is valid" and "every refresh message is valid".
- `examples/reference-portal/tools/validate.py` loads the repo-root schema file directly with Python `jsonschema`, with no rebuild needed. It validates the fixture lines and the connector messages in `protocol/*.ndjson`. `scripts/validate-parity.py` compares its result with `seikika validate`.
- The workspace connector's `specs.rs` test `the_schemas_in_this_repo_are_draft_2020_12_and_wired_in` checks that the schema stays Draft 2020-12, has an `https://` `$id`, and is wired in. That is a property check on the schema itself, not a check of messages.

**Not schema-checked, as far as I can tell from the code:**
- `seikika report`'s own connector run: `resync()` and `inputs()` in `crates/seikika-cli/src/report.rs` ingest straight into a `MemStore`.
- `seikika connector drive` (`conformance::drive_resync`).
- `seikika_store::load_fixtures` on its own. When it is reached through `serve` or `portal::load`, `validate_portal` has already run.

## Q5

**CLI side:**
1. `Command::Get { id, fields, remote }` (`crates/seikika-cli/src/main.rs`, around L106) flattens `Remote` (around L389). `Remote` holds:
   - `--url` (env `SEIKIKA_URL`, default `http://127.0.0.1:8080`);
   - `--token` (env `SEIKIKA_TOKEN`);
   - `--as` (env `SEIKIKA_AS`, dev servers only);
   - `-o` (md, json or yaml).
2. `main()` calls `read_call()` (around L556), which maps `get` to the MCP tool `"get"` with the arguments `{"id", "fields"}`.
3. `client::tool(&remote.target(), …)` (`crates/seikika-cli/src/client.rs`) carries the request. It sends **one JSON-RPC 2.0 `tools/call` request, over HTTP POST to `<url>/mcp`**. `Target::request` adds `Authorization: Bearer <token>` and/or `x-seikika-user: <as>`.
4. The CLI prints `structuredContent` for json or yaml, otherwise the Markdown in `content[0].text`. It exits 1 when `isError` is set, 2 on a 401 or a connection error, and 1 on any other non-2xx status.

**Server side:**
1. The router (`crates/seikika-api/src/routes.rs`, around L68) sends `/mcp` to `mcp::handle` (`crates/seikika-api/src/mcp.rs`, around L851).
2. `handle` first checks identity with `state.caller(&headers)` ("Every request needs an identity (same as REST)"). For `tools/call` it then runs `run_tool`.
3. `run_tool`'s `"get"` arm (around L283) builds `/v1/entities/{id}[?fields=…]` and calls `call_rest`. `call_rest` (around L95) copies the original request headers, except content-length and content-type, and replays the request **in-process** through `router(state).oneshot(req)`. The MCP tool and the REST endpoint therefore share one implementation.
4. `routes::get_entity` (`routes.rs`, around L303) does the actual read:
   1. `s.caller(&headers)` authenticates again.
   2. `snapshot_for` picks the published snapshot, honouring `min_snapshot`.
   3. `s.visibility(&caller, &[id])` checks visibility.
   4. `s.shape_entity(&p, &caller, &id, vis, fields)`, filtered by `vis.view`, reads the entity from `p.derived.entities`, the current published `Derived` snapshot. It also adds `inbound` and `outbound` edges (`reads::outgoing`) and the snapshot id, which is also sent in the `x-seikika-snapshot` header.
   5. An entity the caller may not view returns the same 404 `not-found` as a missing one.

**Identity** (`AppState::caller` in `crates/seikika-api/src/access.rs`, and `Authenticator::authenticate` in `crates/seikika-authz/src/authn.rs`, around L221):
- `authenticate` tries the following in order:
  1. An IAP or Cloudflare Access JWT header, depending on the mode.
  2. A `Bearer tbx_…` personal access token. Its hash is looked up in static PATs, then Postgres (`PgPats`).
  3. A Bearer JWT, accepted only in `oidc` mode.
  4. `x-seikika-user`, accepted only in `dev` mode.
- If nothing matches, the call gets 401 `unauthenticated`. A token that is presented and rejected gets 401 `invalid-credentials`.
- `caller` maps the subject to a `user:` entity with `user_for` (by email, else by login) and computes `admin` with `is_admin`, an OpenFGA check of `admin` on `portal:<name>`.

**Visibility** (`access.rs`):
- `visibility()` batch-checks OpenFGA for `can_view` and `can_view_details` on the entity.
  - If the model does not define a relation for that type, the defaults apply: public by default (D15), so `view = true`, and `details = admin`.
  - With no Authorizer (dev without OpenFGA), everything is visible.
  - If OpenFGA errors, the call gets 502 `authz-unavailable`.
- `shape_entity()` applies facets per field through `facet_needs`:
  - `public` (or none) is always shown;
  - `operator` needs admin;
  - any other facet needs `details`.

  A hidden field is replaced by the `redacted()` marker `{"redacted": true, "requires", "visible_to"}`.
