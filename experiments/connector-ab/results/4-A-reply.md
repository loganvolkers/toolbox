From [Connector A/B phase4 Toolbox](bc-c9a3934b-6678-5d51-a752-bea6934b3a50):

No sync-logic rewrite. Tenant via `TOOLBOX_CONNECTOR_CONFIG` / portal config. Pointed portal + env at tenant B; `OUT/entities.ndjson` has GAMMA (+ issues, `GAMMA:core`). `toolbox connector test` 12/12.

**DX:** Switching tenants was config-only. Mild friction: two places to keep in sync (env for stdio export vs `portal/toolbox.yaml` for `connector test`) unless you use `--config`.

**Tool calls:** 15 (transcript).
