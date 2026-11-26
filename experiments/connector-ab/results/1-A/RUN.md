# How to run (arm A — Forge / Seikika)

Work from `/tmp/connector-ab/arms/A`.

## 1. Build

```bash
export PATH=/tmp/connector-ab/bin:$PATH
# seikika-connect needs edition 2024 (Rust 1.97); the arm's default cargo may be older.
cargo +1.97.0 build
```

Binary: `target/debug/forge-connector`.

## 2. `SEIKIKA_CONNECTOR_CONFIG`

JSON object (secrets already resolved; no env expansion inside the connector):

```json
{"baseUrl":"http://127.0.0.1:9743","token":"tenant-a-token"}
```

| Field     | Meaning                          |
|-----------|----------------------------------|
| `baseUrl` | Forge tenant base (no trailing `/`) |
| `token`   | Bearer token (`Authorization`)   |

## 3. Produce `OUT/entities.ndjson`

Resync over stdio with `FORGE_WRITE_ENTITIES=1` (optional `FORGE_ENTITIES_PATH`, default `OUT/entities.ndjson`):

```bash
cd /tmp/connector-ab/arms/A
export PATH=/tmp/connector-ab/bin:$PATH
export SEIKIKA_CONNECTOR_CONFIG='{"baseUrl":"http://127.0.0.1:9743","token":"tenant-a-token"}'
export FORGE_WRITE_ENTITIES=1

printf '%s\n' '{"type":"command","command_id":"cmd_resync1","command":"resync","kinds":["project","issue"],"priority":"background","deadline":"2099-01-01T00:00:00Z"}' \
  | ./target/debug/forge-connector >/tmp/forge-resync.ndjson
```

Mapped lines are sorted by `kind` then `id`. Kinds in phase 1: `project`, `issue` (no `component`).

## 4. `seikika connector test`

Minimal portal: `portal/seikika.yaml` (connector name `forge`, records `project` + `issue`).

```bash
cd /tmp/connector-ab/arms/A
export PATH=/tmp/connector-ab/bin:$PATH
seikika connector test --name forge --portal portal -- ./target/debug/forge-connector
```

Override config (e.g. another tenant) with `--config '{"baseUrl":"…","token":"…"}'`.
