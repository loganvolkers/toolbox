# Connector scaffold

Minimal stdio connector for the **package author** SDLC
([connector-control.md](../../docs/design/connector-control.md),
decision [D01M83FFM2K6XS4EAVHVP92TBDZ](../../docs/decisions/D01M83FFM2K6XS4EAVHVP92TBDZ.md)).
Copy this directory, rename the crate, and point `seikika-connect` at a released
Seikika checkout or path dep.

## Check

```bash
cargo build
seikika connector test --name example --portal portal -- target/debug/example-connector
mkdir -p OUT
seikika connector drive --kinds example-item --format catalog --out OUT/entities.ndjson \
  --config '{"seed":"alpha"}' \
  -- target/debug/example-connector
```

Both commands must exit 0. `--format catalog` writes `{kind,id,data}` NDJSON
(decision [D01M83GQ1KP8A0WR3P2S7AG54WQ](../../docs/decisions/D01M83GQ1KP8A0WR3P2S7AG54WQ.md)).
Agents: follow the `author-connector` skill in
`plugins/seikika-operator/skills/author-connector/`.
