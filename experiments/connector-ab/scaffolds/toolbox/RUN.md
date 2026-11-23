# How to run

1. **Build** (use the pinned toolchain in `rust-toolchain.toml`):
   ```bash
   cargo build
   ```
2. **Config** is JSON in `TOOLBOX_CONNECTOR_CONFIG`, e.g.
   `{"baseUrl":"http://127.0.0.1:9743","token":"tenant-a-token"}`.
   Keep `portal/toolbox.yaml` connector `config:` in sync when you also run
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
5. Agents: read `plugins/toolbox-operator/skills/author-connector/SKILL.md` and
   `examples/connector-scaffold/` before inventing a second protocol.

## Toolchain

This directory includes `rust-toolchain.toml` (1.97.0). Use `cargo` from rustup so edition 2024 path deps build.
