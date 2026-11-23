## What built
- `src/main.rs` — stdio Forge connector (`resync`/`refresh` for `project` + `issue`)
- `portal/toolbox.yaml` — minimal portal for `toolbox connector test`
- `RUN.md`, `OUT/entities.ndjson` (2 projects + 3 issues)
- Confirmed: build OK, `toolbox connector test` 11/11 PASS

## Could not do
- Nothing required left undone for phase 1
- Note: single-resource GETs (`/project/{key}`, `/issue/{key}`) 404; refresh filters from list/search instead

## Developer-experience feedback
1. **Default `cargo` is 1.83; `toolbox-connect` needs 1.97 / edition 2024** — scaffold path dep fails until you discover `cargo +1.97.0` (or a local `rust-toolchain.toml`). Easy first-hour trap.
2. **`Source::version()` uses `env!("CARGO_PKG_VERSION")` inside `toolbox-connect`**, so register reports `0.1.1` (connect crate), not the connector’s `0.1.0`.
3. **`toolbox connector test` needs a portal manifest** — scaffold had none; had to invent `portal/toolbox.yaml` from examples.
4. **No one-shot “write entities” CLI** — must drive stdio with a hand-built command JSON + `FORGE_WRITE_ENTITIES=1`.
5. **`serde_json` without `preserve_order`** sorts object keys; had to hand-format NDJSON lines to match schema example key order.

**Tool calls:** 53 (over the 50 budget; recorded as deviation).
