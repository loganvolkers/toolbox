# Monitor and recover Seikika production

The private monitor uses a Cloudflare service binding to probe `/readyz`. It permits a cold container to load the child repositories before recording an error. A separate schedule reads the primary’s connection count and connection limit from PlanetScale and logs an error at 70% utilization. Configure `PLANETSCALE_ORG` and `DATABASE` as Worker variables and `PLANETSCALE_SERVICE_TOKEN_ID` and `PLANETSCALE_SERVICE_TOKEN` as Worker secrets. Grant that token `read_database` and `read_branch` on the production database.

Enable Workers Observability Issues on the portal and monitor. Create a Cloudflare Notifications **Real-Time Issue** policy for your operators and attach an occurrence-threshold automation to each Worker. A portal `authz-unavailable` response logs an error with no request body or database details. Readiness timeouts and high connection utilization appear as monitor issues. [Workers Issues](https://developers.cloudflare.com/workers/observability/issues/) · [Issue automation](https://developers.cloudflare.com/workers/observability/issues/automations/)

## Roll back an image or Worker deployment

1. Record the known-good tracked Git revision, Worker version, and Container image digest after each verified deployment.
2. Revert the failing change on `main` and let Workers Builds deploy that revision with `wrangler deploy`. That updates both the Worker and Container image. A Worker-version rollback alone does not roll back a Container image.
3. Keep the existing production database and signing bindings. Verify `/readyz`, Access sign-in, child discovery, and an existing action run. If the database schema is involved, check migration compatibility before deploying old code. [Worker rollbacks](https://developers.cloudflare.com/workers/configuration/versions-and-deployments/rollbacks/) · [Container deployment](https://developers.cloudflare.com/workers/ci-cd/builds/configuration/)

## Restore a database backup

1. Create or select a completed PlanetScale backup. Restore to a fresh branch. Use a region and size available in the organization and verify the restore against the same PostgreSQL major version.
2. Use a temporary credential scoped to the restored branch. Check Seikika’s migrations, OpenFGA tables, and an action run known to exist when the backup was taken.
3. For incident recovery, stop the original publisher before directing the Worker to the restored branch. Set its direct TLS `DATABASE_URL` in Cloudflare, keep its signing key, and roll the Container before reopening traffic. Verify readiness, Access, discovery, and the persisted run.
4. Retire a drill branch after verification and retain the source backup according to the database backup policy. Keep the signing seed in an operator-controlled secret manager for recovery on a new Worker. [PlanetScale backup commands](https://planetscale.com/docs/cli/backup/)
