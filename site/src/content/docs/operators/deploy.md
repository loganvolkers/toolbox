---
title: Deploy and operate a portal
description: Start a portal with Postgres and OpenFGA, check readiness, and verify the rollout.
sidebar:
  order: 5
---

A shared portal needs a validated configuration, a persistent Postgres database, identities for callers, and the connector binaries and credentials the manifest requires. Use the [local tutorial](/start/portal/) to try the product with fixtures before configuring a shared deployment.

## Prepare the deployment

1. Choose a [release or source revision](/start/versions/) and pin it. When release images are available, use the [Seikika image](https://github.com/seikika/seikika/blob/main/image/Dockerfile) and add your portal directory at `/portal`. For a source build, put the CLI and required connectors on the host, and install OpenFGA.
2. Provision Postgres. Use a database for this portal and store its connection URL as a secret. Configure transport security for the database. If OpenFGA needs a different connection URL, set `SEIKIKA_OPENFGA_DATASTORE_URI`.
3. Configure [authentication and authorization](/operators/access/) and [connector secrets](/operators/secrets/). Keep the service behind HTTPS and limit access to internal endpoints such as `/metrics`. Set `SEIKIKA_CONNECTOR_CONFINEMENT=required` on a host with Landlock if the deployment depends on connector confinement.
4. Run `seikika validate PATH_TO_PORTAL`, `seikika test PATH_TO_PORTAL`, and `seikika doctor PATH_TO_PORTAL`. Replace `PATH_TO_PORTAL` with the directory that holds the manifest.

If your deployment cannot pull from GHCR, build the container locally from the public release archives. [Build a release image locally](https://github.com/seikika/seikika/blob/main/docs/RELEASING.md#build-a-release-image-locally)

## Start the service

Set `DATABASE_URL`, the settings for your [authentication mode](/operators/access/), and the connector settings in your service environment. On a source-built host, set the shell variable `PATH_TO_PORTAL` to the absolute path to the portal directory. The following command uses token authentication and binds loopback for a reverse proxy on the same machine.

```bash
SEIKIKA_AUTH_MODE=token seikika serve "$PATH_TO_PORTAL" \
  --listen 127.0.0.1:8080 --database-url "$DATABASE_URL"
```

A container host can set `PORT` to bind on all interfaces inside the container. An explicit `--listen` wins over `PORT`. Route traffic through your host's access controls before exposing the port.

For a full local simulation with Postgres, a Git service, and connector inputs, follow [the Acme example](https://github.com/seikika/seikika/tree/main/examples/acme-live). Its credentials and development identity settings are for a practice environment.

For Cloudflare, deploy a Worker and one Container with a direct PostgreSQL connection. The example includes authorization for a parent portal and the configuration for passing secrets into the container. [Cloudflare Containers example](https://github.com/seikika/seikika/tree/main/examples/cloudflare-containers)

## Verify the rollout

```bash
curl -fsS http://127.0.0.1:8080/healthz
curl -fsS http://127.0.0.1:8080/readyz
```

`/healthz` shows that the process answers. `/readyz` returns `"ready":true` and a snapshot after the first catalog is available. Check its connector confinement counts. Verify that an authenticated user can read a known entity, that a caller without access receives a redacted result or denial, and that a connector has supplied recent data. See [the HTTP API reference](/users/api/) for requests you can adapt.

Monitor readiness, connector logs, and catalog freshness. Back up Postgres and retain the manifest revision and image digest. Follow [Releases and upgrades](/start/versions/#verify-an-upgrade) to rehearse rollback. Keep action execution on one owning instance until shared execution and durable recovery are available. An interrupted action can fail after its effect occurred, so inspect the run before retrying.

If readiness fails or catalog data is missing, start with [portal troubleshooting](/start/troubleshooting/#running-a-portal). For a parent portal, also check [child access and runners](/operators/federate-actions/#configure-a-shared-parent). If your readers need a saved snapshot, see [static report hosting](/operators/publish/).
