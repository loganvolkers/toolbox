# Answers

## Q1

**Outcome.** The rule scores `unknown`. In `Derive::score` (`crates/seikika-derive/src/lib.rs`), a rule's `check` is evaluated as CEL. If the evaluation returns an error, the outcome is `"unknown"` (the `Err(_) => "unknown"` arm). Only a real `false` gives `"fail"`, and a `fail` with a matching waiver becomes `"waived"`. A missing connector field produces that error once a rule drops its `has()` guard. If the rule declares `dataFrom` (field `Rule::data_from`, `crates/seikika-model/src/manifest.rs`), `Derive::evidence` attaches the evidence `["no data from connector <src>"]`. `seikika validate` checks that `dataFrom` names a declared connector (`xref.connector` in `crates/seikika-model/src/validate.rs`).

**Level.** The level is computed by `level_ignoring_unknown` (same file):
- If every rule is `unknown`, it returns `None`. The displayed level is then `none` (`unwrap_or_else(|| "none".into())` in `score`).
- Otherwise it climbs the scorecard's levels in order:
  - It stops at the first level whose rules are all `unknown`, so a level that has no data is not granted.
  - At a level with known rules, every known rule must be `pass` or `waived` (`counts_as_pass`). Unknown rules at that level are ignored.
  - A known `fail` stops the climb.
- For the repository rollup (D86, `maintainer-repo`), `constituent_level` and `constituent_summaries` skip an entity whose rules are all unknown (`None`) and entities under `/demo/`. An all-unknown entity therefore drops out of the "lowest constituent" instead of making the repository `none`.

**Ratchet (`seikika report --baseline`)** (`crates/seikika-cli/src/report.rs`):
- `results` records unknown rules under `"unknown"` and failing rules under `"failing"`, with a measure or an evidence count. The comment there says an unknown is "never a regression or an improvement".
- `compare_entity` handles a rule by its old and new outcome:

| Old outcome | New outcome | How `compare_entity` reports it |
|---|---|---|
| any | newly `unknown` | `other` ("unknown now (its data is missing)") |
| `unknown` | `fail` | `other` ("known again: fails"), not a regression |
| `fail` | `unknown` | not an improvement (the "now passes" filter excludes `nu`) |
| pass or absent | `fail` | **regression** ("now fails") |
| `fail` | `fail`, with a larger measure or evidence count | **regression** (`evidence_change`) |

- `decide` checks regressions first: the result is `regressed`, exit 1.
- Next it counts unknowns that are not excused:
  - `unexpected_unknowns` counts unknowns whose rule is not in `excused_rules`, the rules whose `dataFrom` connector is in `--skip`.
  - If that count is above zero and `--ci`, `--update` or `--fail-on-unknown` is set, the result is `unknown`, exit 1, and the baseline is not written.
  - Otherwise unknowns are only counted (`unknown_count`) and reported as "not compared".
- `inputs` does not run connectors listed in `--skip`. The fast ratchet skips `runner` and `github-actions`, so their rules are expected unknowns. `ci-lint` is not skipped, so a missing zizmor or actionlint still fails `--ci`.
- The verdict file carries `result: unknown` and `unknownRules` (D96, `D01M7XJA1FG0000000000000000`).

**Decisions:**
- **`D01M7Y5VWKW5TDS0VQWZFPRQ27N`** ("Missing connector data is unknown, and it does not grant a level that has no data") governs this. It amends D70 (`D01M7VZ2DPR0000000000000000`), which first said "rules over missing tool data are `unknown` (never a regression; `--ci`/`--update` refuse)".
- D86 (`D01M7WMSMS50000000000000000`) covers the rollup.

## Q2

The path is in `crates/seikika-api/src/connectors.rs` unless noted otherwise.

1. **Start.**
   - `seikika serve` runs `server::serve` (`crates/seikika-cli/src/server.rs`). That function validates the portal (`validate_portal`), builds the store (`PgStore`, or `SharedMem(MemStore)`, optionally loaded from fixtures) and calls `seikika_api::serve` (`crates/seikika-api/src/lib.rs`).
   - `seikika_api::serve` spawns `run_snapshot_loop` and `freshness_loop`, then calls `connectors::spawn_in_image`.
2. **Supervision.**
   - `spawn_in_image` does nothing when `SEIKIKA_CONNECTORS=off`. For each connector with `runtime == "in-image"`, it resolves the binary with `binary_for` (`builtin:<name>` becomes `seikika-connector-<name>`) and the config with `resolve_config` (`${env:}` and `${secret:}`). It then spawns `supervise`.
   - `supervise` starts the child process with `SEIKIKA_CONNECTOR_CONFIG` set in its environment, and restarts it with exponential backoff (1 s up to 30 s) when it exits.
3. **Transport: NDJSON over the child's stdin and stdout.**
   - A writer task sends commands from `Hub::take` (`crates/seikika-connect/src/hub.rs`) to the child's stdin, one JSON line each.
   - The reader parses each stdout line as JSON and skips lines that don't parse. It logs and drops `log` messages.
   - It batches the other messages and flushes the batch on a control boundary (`register`, `complete`, `snapshot_complete`), at 500 messages, or after 100 ms of quiet.
   - `flush` calls `ingest_messages`. A rejected batch is logged ("connector batch rejected") and dropped. I found no retry.
4. **Checks in `ingest_messages`.** These are the same checks `POST /v1/ingest` (`routes::ingest`) uses for remote connectors:
   - The connector must be declared in the manifest, or the batch gets 403 `unknown-connector`.
   - Every message is validated against `schema/connector-messages.schema.json` with `seikika_model::schema::errors(schema::connector_messages(), m)`.
   - Every `record` and `delete` must use a `kind` listed in that connector's `records`.
   - Any error rejects the whole batch with 422 `invalid-messages`.
   - I found no check of a record's `data` against the kind's field types at this point.
5. **Store.**
   - If the batch contains a `record` or `delete`, `Store::ingest` applies it atomically and returns a new input version:
     - `MemStore::ingest` (`crates/seikika-store/src/lib.rs`) upserts on `(connector, kind, externalId)` and removes on delete.
     - `PgStore::ingest` (`crates/seikika-store/src/pg.rs`) does the same in one transaction on `seikika.raw_records` and records the version in `seikika.input_versions`.
   - A control-only batch creates no version.
6. **Control bookkeeping.**
   - `Hub::on_messages` handles the control messages:
     - `register` stores the registration.
     - `ack` and `complete` update the command status. A succeeded `complete` also updates `last_read`.
     - It collects the externalIds seen per command and kind, and returns them at `snapshot_complete`.
   - `reconcile` deletes records that a full resync no longer reports. It holds the deletes when they would remove ≥ 90 % of a kind (`DELETION_THRESHOLD`).
   - After a `register`, `initial_resync` enqueues a `resync` for each declared kind that has never been read.
7. **Recompute.**
   - `ingest_messages` ends with `s.changed.notify_one()`.
   - `AppState::run_snapshot_loop` (`crates/seikika-api/src/state.rs`) wakes on that `Notify`, or on its poll interval. It sees that the store's `input_version` differs from the published snapshot.
   - It debounces until inputs stop changing, at most `max_lag`. Then it calls `recompute_and_publish`, which:
     - takes `store.snapshot()`;
     - runs `FullRecompute.recompute` in `spawn_blocking`;
     - publishes tuples to Postgres (only on the advisory-lock leader) and syncs OpenFGA tuples;
     - publishes the new `Published` snapshot (`current.send_replace`).

The design is described in `docs/design/connector-control.md` and v1-spec §7.4, as the module doc says. I did not read those documents for this answer.

## Q3

**Why these ids.** Decision `D01M7XKEXQH4HG5371629M50E2G` ("A decision is one file, and its id is a ULID") and `docs/design/decision-files.md` §1–2 explain it:
- The log used to be one table, `docs/decisions.md`. The next id was "one more than the highest D". Two parallel pull requests took the same number, and git cannot merge two appends at the same line.
- Now each decision is `docs/decisions/<id>.md`, and the id is `D` plus a 26-character ULID: 48 bits of Unix milliseconds and 80 random bits, in Crockford base32. Agents can mint ids without coordinating, and filename order is time order.
- ULID was chosen over ObjectId, xid, KSUID, UUIDv7, TypeID and Snowflake (comparison table in §2).
- `scripts/decision-id.py` mints ids (`new_id`, with 10 random bytes from `secrets`). Its `--self-test` checks the oklog/ulid vector, which CI runs.

**Old ids.** D1–D98 were backfilled (`decision-files.md` §4; `docs/decisions/README.md`):
- Each file has `legacy: Dnn`.
- The random part is zero, so the id ends in sixteen `0`s and the same inputs reproduce it.
- The time part is the second of the commit that first added the row (from `git log --reverse -p`), plus the row's position among the decisions added in that same second, in milliseconds. `backfill_id(unix_seconds, offset)` in `scripts/decision-id.py` implements this.
- Example: D1 to D4 are `...31R`, `...31S`, `...31T` and `...31V`, at 2026-11-19T17:09:39Z with offsets 0 to 3. Their `decided` field is that commit time.
- The day headings in the old table were deliberately not used as the timestamp.
- Existing `Dnn` citations were not rewritten.

**Existence check.**
- The `maintainer-docs` scorecard's bronze rule `cited-decisions-exist` (`portal/standards/docs.yaml`) checks `entity.missingDecisions.size() == 0`, with the missing ids as evidence. It was introduced by D84 (`D01M7WMSMS30000000000000000`), which then checked `| Dnn |` rows of `docs/decisions.md`. `D01M7XKEXQH4HG5371629M50E2G` amended it to read the files (`docs/design/decision-citations.md`).
- The `missingDecisions` field is computed by the workspace connector (`crates/seikika-connector-workspace/src/main.rs`):
  - `known_decisions` reads `id` and `legacy` from the frontmatter of each `docs/decisions/*.md` (`decision_file_ids`, `canon_decision`).
  - `missing_decisions` matches `\b(D\d{1,4}|D[Crockford]{26})\b`, case-insensitive, and keeps the ids that are not known. It skips `docs/decisions/` and `docs/decisions.md`.
- The scorecard's `appliesTo` also skips the docs in `policy.historicalDocs` (`portal/seikika.yaml`, for example `docs/questions.md` and `docs/research-log.md`).
- The rule is tested with a fixture that cites `D999` (`portal/tests/standards.test.yaml`). CI enforces it through the ratchet.
- A separate check covers assessments: `known_id` in `crates/seikika-connect/src/assessment.rs`, used by `cites`. A D/Q id in an assessment's reason only counts as a citation if `docs/decisions/<id>.md` exists for a ULID, or a file has a `legacy: Dnn` line for a short id.

## Q4

The schema is compiled into the binary:
- `CONNECTOR_MESSAGES_SCHEMA = include_str!("../../../schema/connector-messages.schema.json")` and `connector_messages()` (a `OnceLock` Draft 2020-12 `Validator` with format checks) are in `crates/seikika-model/src/schema.rs`.
- `schema::errors` turns the failures into `(pointer, message)` pairs.
- A change therefore takes effect in Rust only after a rebuild. Released binaries keep the schema they were built with, as the module doc says.

Places that enforce it:

| Where | When | What happens to a message that doesn't conform |
|---|---|---|
| `ingest_messages` (`crates/seikika-api/src/connectors.rs`) | Runtime, in `seikika serve`: in-image connectors over stdio (`supervise` → `flush`), and remote connectors and `seikika push` over `POST /v1/ingest` (`routes::ingest`) | The whole batch is rejected with 422 `invalid-messages`. For stdio, the error is logged and the batch dropped |
| `validate_fixtures` (`crates/seikika-model/src/messages.rs`), called from `validate_portal` (`validate.rs`) | Check time: `seikika validate`. Also every command that loads a portal through `validate_portal`: `seikika test`, the local reads in `portal::load`, `report`, and `serve`, which refuses to start | Diagnostic `fixture.schema` per `fixtures/*.ndjson` line, and the portal is invalid |
| `conformance::run` (`crates/seikika-connect/src/conformance.rs`) | Check time: `seikika connector test` (run in CI against workspace, surfaces and ci-lint) | The checks "register message is valid" and "every message validates" fail |
| `examples/reference-portal/tools/validate.py` | Check time, in CI and in `scripts/validate-parity.py`. It reads the schema file at run time, so it sees a change without a rebuild | Errors for fixture lines and `protocol/*.ndjson` transcript messages |

Gaps and side checks:
- `seikika report` running live connectors (`report::resync` → `inputs` → `MemStore::ingest`) does not validate messages against the schema. Its fixture path depends on `validate_portal` having run first.
- `MemStore::ingest`, `PgStore::ingest` and the connector SDK (`crates/seikika-connect/src/sdk.rs`) don't validate either, as far as I found.
- The workspace connector's `json-schema` records and the `maintainer-schema` scorecard (D77; `crates/seikika-connector-workspace/src/specs.rs`) score the schema file itself (draft, `$id`, wired into `schema.rs`). They do not check messages.
- The unit test `embedded_schemas_compile` would fail if the edited schema did not compile.

## Q5

1. **CLI.**
   - `Command::Get { id, fields, remote }` (`crates/seikika-cli/src/main.rs`) is matched by `read_call` before the main dispatch and becomes the MCP tool `get` with `{"id", "fields"}`.
   - `CLI_MCP_TOOLS` maps CLI read commands to MCP tools, as decided in D61 (`D01M7VQJXMR0000000000000000`).
   - `Remote` provides `--url` (env `SEIKIKA_URL`, default `http://127.0.0.1:8080`), `--token` (`SEIKIKA_TOKEN`) and `--as` (`SEIKIKA_AS`, dev only).
   - `main` calls `client::tool(&remote.target(), ...)`.
2. **Transport.**
   - `client::tool` (`crates/seikika-cli/src/client.rs`) sends an HTTP `POST {url}/mcp` with the JSON-RPC body `{"method": "tools/call", "params": {"name": "get", "arguments": ...}}`.
   - `Target::request` adds `Authorization: Bearer <token>` and/or `x-seikika-user`.
   - The output is the tool's Markdown (`content[0].text`) or `structuredContent` for `-o json` and `-o yaml`. The exit code is 1 when `isError`, and 2 on 401 or a connection error.
3. **Server, MCP layer.**
   - The route `/mcp` goes to `mcp::handle` (`crates/seikika-api/src/routes.rs` `router`; `crates/seikika-api/src/mcp.rs`).
   - `handle` first requires an identity with `state.caller(&headers)`, then dispatches `tools/call` to `run_tool`.
   - For `get`, `run_tool` builds `/v1/entities/{id}[?fields=...]` and calls `call_rest`. That function copies every request header except `content-length` and `content-type`, so the credentials travel too, and sends the request in-process through `router(state).oneshot(...)`. No second network hop is made.
4. **Server, REST handler `get_entity`** (`crates/seikika-api/src/routes.rs`):
   1. It calls `s.caller(&headers)` again (identity).
   2. `snapshot_for` picks the published snapshot, or waits for `min_snapshot`.
   3. `s.visibility(&caller, &[id])` gives the entity's visibility.
   4. `s.shape_entity(&p, &caller, &id, vis, fields)` reads `p.derived.entities[id]` from the in-memory published `Derived` snapshot, not the store. The result is filtered by `vis.view`. A caller who can't view the entity gets 404 `not-found`, the same as a missing entity.
   5. It adds `inbound`, `outbound` (`reads::outgoing`) and the snapshot id (`with_snapshot`).
5. **Identity** (`AppState::caller`, `crates/seikika-api/src/access.rs`).
   - `Authenticator::authenticate` (`crates/seikika-authz/src/authn.rs`) accepts, in order:
     - a proxy JWT (`x-goog-iap-jwt-assertion` or `cf-access-jwt-assertion`, by mode);
     - a `Bearer` personal access token (`tbx_…`, static or from Postgres via `PgPats`), or an OIDC JWT in `oidc` mode;
     - in dev mode only, `x-seikika-user`.
   - A missing credential gives 401 `unauthenticated`. An invalid one gives 401 `invalid-credentials`.
   - `user_for` maps the subject to a `user:` entity (by email for email subjects, otherwise by login).
   - `is_admin` checks OpenFGA `admin` on `portal:<name>`. It returns true when no authorizer is configured.
6. **Visibility** (`access.rs`).
   - `visibility` batch-checks OpenFGA `can_view` and `can_view_details` for the entity's type.
   - A type the model doesn't cover is visible to signed-in users; this is public by default, per D15. Its details are visible to admins.
   - With no authorizer (dev mode without the `openfga` binary, see `server::serve`), everything is visible.
   - `shape_entity` applies facets per field (`facet_needs`):
     - `public` or no facet: always shown;
     - `operator`: needs admin;
     - any other facet: needs `can_view_details`.
   - A hidden field is replaced by a `redacted` marker (`{"redacted": true, "requires", "visible_to"}`).

Not certain: in `get_entity`, `inbound` and `outbound` are filled from the derived graph without a further visibility check. As far as the code shows, they can list ids of entities the caller cannot view.
