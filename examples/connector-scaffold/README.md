# Connector scaffold

Minimal stdio connector for the **package author** SDLC
([connector-control.md](../../docs/design/connector-control.md),
decision [D01M83FFM2K6XS4EAVHVP92TBDZ](../../docs/decisions/D01M83FFM2K6XS4EAVHVP92TBDZ.md)).
Copy this directory, rename the crate, and point `toolbox-connect` at a released
Toolbox checkout or path dep.

## Check

```bash
cargo build
toolbox connector test --name example --portal portal -- target/debug/example-connector
toolbox connector drive --kinds example-item \
  --config '{"seed":"alpha"}' \
  -- target/debug/example-connector
```

Both commands must exit 0. Agents: follow the `author-connector` skill in
`plugins/toolbox-operator/skills/author-connector/`.
