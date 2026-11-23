# Gold key facts: Toolbox at a8f7e03

Graded per fact: covered (1), partial (0.5) or missed (0). A fact is covered when the answer states
it, in any words, with the names that identify it (a decision id, function, file or flag). Partial:
the gist without the identifying names, or right with one material error. A wrong claim that
contradicts a fact makes it missed.

## Q1. Missing connector data and the ratchet (4 facts)

- **T1a.** The rule scores `unknown` (a missing field is a CEL error, which is unknown); the decision is D01M7Y5VWKW5TDS0VQWZFPRQ27N, which amends D70.
- **T1b.** The level ignores unknown rules: `level_ignoring_unknown` (crates/toolbox-derive/src/lib.rs) climbs only through levels with at least one known rule, all of whose known rules pass or are waived. An entity whose every rule is unknown stays `none` and drops out of the repository rollup.
- **T1c.** The ratchet (`compare_entity` and its callers in crates/toolbox-cli/src/report.rs) does not count unknown as a regression: a rule that becomes unknown, or that was unknown and is now known and failing, is reported but not a regression. A rule that newly fails is a regression.
- **T1d.** `--ci` and `--update` refuse an unknown whose `dataFrom` connector was not in `--skip` (`unexpected_unknowns`); an unknown from a skipped connector is expected and not compared.

## Q2. In-image connector ingest (5 facts)

- **T2a.** `spawn_in_image` starts a `supervise` task per in-image connector (crates/toolbox-api/src/connectors.rs). It runs the binary as a child process, with its config in `TOOLBOX_CONNECTOR_CONFIG`, and restarts it with exponential backoff (1 s up to 30 s) when it exits.
- **T2b.** Transport: NDJSON over stdio. Commands from the hub are written to stdin, and messages are read from stdout.
- **T2c.** Messages are batched until a control boundary (`register`, `complete`, `snapshot_complete`), 500 messages or 100 ms of quiet, then `flush` calls `ingest_messages`.
- **T2d.** `ingest_messages` validates each message against the connector-messages JSON Schema and checks the connector and each record kind are declared in the manifest (rejecting the batch otherwise). It then stores the batch, creating a new input version only when records or deletes change.
- **T2e.** After a full read, `reconcile` deletes records the connector no longer reports, unless that would remove 90% or more of the kind (`DELETION_THRESHOLD`; the deletes are held). A `register` triggers `initial_resync`, and `s.changed.notify_one()` wakes the recompute.

## Q3. Decision ids (4 facts)

- **T3a.** Decision D01M7XKEXQH4HG5371629M50E2G made a decision one file (`docs/decisions/<id>.md`) with a ULID id, so parallel agents no longer append to one table or claim the same number. Design: docs/design/decision-files.md.
- **T3b.** The id is `D` plus a 26-character ULID: 48 bits of millisecond time and 80 random bits, Crockford base32, so filename order is time order. `scripts/decision-id.py` mints one.
- **T3c.** The old D1–D98 were backfilled: the timestamp is that of the commit that introduced the row, the random part is zero, and the file records `legacy: Dnn`, so old citations keep working.
- **T3d.** The `cited-decisions-exist` rule (portal/standards/docs.yaml) checks a document's `missingDecisions`, which the workspace connector computes (`missing_decisions` and `known_decisions`, crates/toolbox-connector-workspace/src/main.rs) from both `id` and `legacy` in decision frontmatter.

## Q4. Connector-messages schema enforcement (4 facts)

- **T4a.** The schema is compiled into the binary by toolbox-model (`include_str!` in crates/toolbox-model/src/schema.rs, `connector_messages()`).
- **T4b.** At runtime, the server's `ingest_messages` (crates/toolbox-api/src/connectors.rs) rejects a non-conforming batch (422, `invalid-messages`), for in-image and pushed batches alike.
- **T4c.** `toolbox validate` checks fixture files (`fixtures/*.ndjson`) against it (`validate_fixtures`, crates/toolbox-model/src/messages.rs).
- **T4d.** `toolbox connector test` runs the conformance kit (crates/toolbox-connect/src/conformance.rs), which checks every message a connector emits against it.

## Q5. `toolbox get` path (4 facts)

- **T5a.** CLI reads are MCP tool calls: `client::tool` (crates/toolbox-cli/src/client.rs) posts a JSON-RPC `tools/call` to the server's `/mcp`. (`toolbox mcp`, the stdio bridge, forwards to the same endpoint.)
- **T5b.** `mcp::handle` (crates/toolbox-api/src/mcp.rs) requires a caller identity, then `run_tool` dispatches by tool name.
- **T5c.** `run_tool` maps `get` to the REST route `GET /v1/entities/{id}` and calls it in-process (`call_rest`, the router's `oneshot`) with the caller's headers, so REST and MCP share one implementation.
- **T5d.** `get_entity` (crates/toolbox-api/src/routes.rs) resolves the caller, the snapshot (`min_snapshot`), and the caller's visibility of the entity, returning it only if visible. The CLI prints the tool's Markdown or structured result (`-o md|json|yaml`).
