From [Connector A/B phase4 Backstage](bc-cad824e0-f9e1-5022-9121-e0a860a9e9b9):

Only `RUN.md` changed (tenant A/B table). Sync code untouched. Tenant B resync via env → 4 entities.

**DX:** Clean env-only reuse. Defaults still bake in tenant A; a named profile (`TENANT=B`) would be clearer than two env vars.

**Tool calls:** 16 (transcript; agent self-reported 17).
