# How to run (inherited connector — currently failing truth)

This tree is a near-complete Forge connector that **fails** the harness truth check.
Your job in phase 1 is to diagnose why; do not fix until phase 2.


1. **Build** (use the pinned toolchain in `rust-toolchain.toml`):
   ```bash
   cargo build
   ```
2. **Config** is JSON in `SEIKIKA_CONNECTOR_CONFIG`, e.g.
   `{"baseUrl":"http://127.0.0.1:9743","token":"tenant-a-token"}`.
   For a dump-only tenant switch, pass `--config` to `connector drive` and leave
   the portal alone; update `portal/seikika.yaml` only when also running
   `connector test`.
3. **Conformance** (portal stub is included):
   ```bash
   seikika connector test --name forge --portal portal -- target/debug/forge-connector
   ```
4. **Dump the catalog** (no hand-built stdio JSON, no map script):
   ```bash
   seikika connector drive --kinds project,issue,component --format catalog \
     --out OUT/entities.ndjson \
     --config '{"baseUrl":"http://127.0.0.1:9743","token":"tenant-a-token"}' \
     -- target/debug/forge-connector
   ```
   `--format catalog` writes harness-shaped `{kind,id,data}` NDJSON (`id` =
   `externalId` with a leading `{kind}:` stripped). Record `externalId`s must
   look like `project:ALPHA` / `issue:ALPHA-1` / `component:ALPHA:api`
   (`^[a-z0-9-]+:.+$`). For components, catalog `id` is `{project}:{name}`
   (e.g. `ALPHA:api`); put the Forge numeric id in `data.id` only — never as
   the `externalId` suffix. Component `data` is exactly `{id,name,project}`
   (no `description`).
5. Agents: read `plugins/seikika-operator/skills/author-connector/SKILL.md` and
   `examples/connector-scaffold/` before inventing a second protocol.

## Last dump (reuse → tenant B)

```bash
cargo build
seikika connector drive --kinds project,issue,component --format catalog \
  --out OUT/entities.ndjson \
  --config '{"baseUrl":"http://127.0.0.1:9744","token":"tenant-b-token"}' \
  -- target/debug/forge-connector
```

Tenant B: 1 project (`GAMMA`), 2 issues (`GAMMA-1`, `GAMMA-2`), 1 component
(`GAMMA:core`). Same binary and sync logic as tenant A; only `--config` changed.
Portal left on tenant A (dump-only).

## Toolchain

This directory includes `rust-toolchain.toml` (1.97.0). Use `cargo` from rustup so edition 2024 path deps build.
