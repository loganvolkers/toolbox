# How to run

1. **Build** (use the pinned toolchain in `rust-toolchain.toml`):
   ```bash
   cargo build
   ```
2. **Config** is JSON in `TOOLBOX_CONNECTOR_CONFIG`, e.g.
   `{"baseUrl":"http://127.0.0.1:9743","token":"tenant-a-token"}`.
   For a dump-only tenant switch, pass `--config` to `connector drive` and leave
   the portal alone; update `portal/toolbox.yaml` only when also running
   `connector test`.
3. **Conformance** (portal stub is included):
   ```bash
   toolbox connector test --name forge --portal portal -- target/debug/forge-connector
   ```
4. **Dump connector records** (no hand-built stdio JSON):
   ```bash
   toolbox connector drive --kinds project,issue \
     --config '{"baseUrl":"http://127.0.0.1:9743","token":"tenant-a-token"}' \
     -- target/debug/forge-connector
   ```
   Map those lines into `OUT/entities.ndjson` for the harness schema.
   Record `externalId`s must look like `project:ALPHA` / `issue:ALPHA-1`
   (`^[a-z0-9-]+:.+$`); catalog entity `id`s in OUT may still be bare keys.
5. Agents: read `plugins/toolbox-operator/skills/author-connector/SKILL.md` and
   `examples/connector-scaffold/` before inventing a second protocol.

## Toolchain

This directory includes `rust-toolchain.toml` (1.97.0). Use `cargo` from rustup so edition 2024 path deps build.
