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
4. **Dump the catalog** (no hand-built stdio JSON, no map script):
   ```bash
   toolbox connector drive --kinds project,issue --format catalog \
     --out OUT/entities.ndjson \
     --config '{"baseUrl":"http://127.0.0.1:9743","token":"tenant-a-token"}' \
     -- target/debug/forge-connector
   ```
   `--format catalog` writes harness-shaped `{kind,id,data}` NDJSON (`id` =
   `externalId` with a leading `{kind}:` stripped). Record `externalId`s must
   look like `project:ALPHA` / `issue:ALPHA-1` / `component:ALPHA:api`
   (`^[a-z0-9-]+:.+$`). For components, put the Forge numeric id in `data.id`
   only — never as the `externalId` suffix (or catalog `id` becomes `1`).
5. Agents: read `plugins/toolbox-operator/skills/author-connector/SKILL.md` and
   `examples/connector-scaffold/` before inventing a second protocol.

## Toolchain

This directory includes `rust-toolchain.toml` (1.97.0). Use `cargo` from rustup so edition 2024 path deps build.
