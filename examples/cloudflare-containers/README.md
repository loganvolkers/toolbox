# Deploy Seikika on Cloudflare Containers

Run a Seikika portal and its OpenFGA co-process in one Cloudflare Container. The Worker routes HTTPS requests to one named instance. A direct PostgreSQL connection stores portal data, authorization tuples, personal access tokens, and action runs. This example is about the box, from packaging the portal to keeping its state across deployments. [Federated actions design](../../docs/design/federated-actions.md)

Cloudflare offers PlanetScale PostgreSQL through its dashboard with billing on the Cloudflare account. Use the database's **direct primary endpoint** and `sslmode=verify-full`. A direct connection preserves the session-level advisory lock Seikika uses to keep action execution on one instance. Hyperdrive and PgBouncer transaction pooling cannot preserve that lock. [PlanetScale billing through Cloudflare](https://developers.cloudflare.com/changelog/post/2026-06-18-planetscale-databases-cloudflare-billing/) · [Hyperdrive connection pooling](https://developers.cloudflare.com/hyperdrive/concepts/connection-pooling/)

## Prepare your portal

1. Copy this directory into your deployment repository. Replace `portal/` with your validated portal, or edit the example. Keep the includes, authorization model, and runtime files the manifest needs under `portal/` so the image can copy them. Keep `federation.signingKey` set to `SEIKIKA_SIGNING_KEY` so that container restarts preserve the public key children trust.
2. Replace the example child with your repositories under `federation.children`. Commit each child manifest at `.seikika/seikika.yaml` on the branch you list. The example expects public HTTPS Git repositories. If a child offers a `local` runner, it is trusted to execute code in the portal container. Review its action definitions and control who can change them.
3. Replace `operator` and `reader` with your own identities. The model lets each child inherit permissions from the parent. Add a static tuple with the `parent` relation for each child, as shown below. For browser sign-in, use Cloudflare Access as described under **Expose the portal**.

```yaml
# In authz.tuples[0].static
- {user: 'portal:cloudflare-demo', relation: parent, object: 'portal:example-repo'}

# In federation.children
- {git: 'https://github.com/example/example-repo.git', branch: main, key: example-repo}
```

```sh
seikika validate portal
seikika test portal
seikika doctor portal
```

## Build the image

The example uses the same entrypoint and co-process as the Seikika server image. Stage Linux `amd64` binaries for Seikika, the connectors your portal uses, and OpenFGA. The image uses Debian Trixie, so build on a system with a compatible glibc. Choose a source revision with SQLx TLS support and run these commands from that checkout. Set `example` to your copy of this directory if you moved it. [Seikika server image](../../image/Dockerfile)

```sh
example=examples/cloudflare-containers
cargo build --locked --release --workspace
for binary in seikika seikika-connector-workspace seikika-connector-github \
    seikika-connector-git-files seikika-connector-gitlab seikika-connector-runner \
    seikika-connector-scim; do
  install -D -m 0755 "target/release/$binary" "$example/bin/$binary"
done
scripts/ci-tools.sh "$example/bin" openfga
```

When a release includes SQLx TLS support, you can stage its checksum-verified Linux `amd64` binaries in `bin/` in place of building Seikika from source. [Stage release archives](../../scripts/stage-image-archives.sh)

## Expose the portal

Use a Cloudflare account with Containers enabled, Node.js, and Docker. Change the Worker `name` to the deployment name in your account. Verify Wrangler can build the image and bundle the Worker with `npm exec --package wrangler@4.168.0 -- wrangler deploy --dry-run`. Provision a dedicated database and a role that can manage schema objects for Seikika and OpenFGA. Both use the same database and role in this example. Store the direct connection URL as the `DATABASE_URL` Worker secret and generate the signing seed once. Keep a copy of the seed in your secret manager for redeploys and disaster recovery.

Run the following commands from this example directory. The first command prompts for the database connection URL. Generate the seed with `openssl rand -hex 32` and load it from your secret manager into `SEIKIKA_SIGNING_KEY`. Use an authenticated Wrangler session or set the scoped `CLOUDFLARE_API_TOKEN` and `CLOUDFLARE_ACCOUNT_ID` in the deployment environment.

```sh
npm exec --package wrangler@4.168.0 -- wrangler secret put DATABASE_URL
printf '%s\n' "$SEIKIKA_SIGNING_KEY" | npm exec --package wrangler@4.168.0 -- wrangler secret put SEIKIKA_SIGNING_KEY
npm exec --package wrangler@4.168.0 -- wrangler deploy
```

The defaults serve the Worker on `workers.dev` and use Seikika personal access tokens for API clients. For an interactive portal, provision a custom domain and a Cloudflare Access application for it. Set `workers_dev = false` and leave `preview_urls = false`. Add the domain as a Wrangler custom-domain route, set `SEIKIKA_AUTH_MODE` to `cloudflare-access`, and add `SEIKIKA_CF_TEAM_DOMAIN` (the HTTPS issuer URL without a trailing slash) and `SEIKIKA_AUDIENCE` (the Access application audience) under `[vars]`. Seikika validates the signed Access assertion. [Configure Seikika access](../../site/src/content/docs/operators/access.md) · [Cloudflare Worker custom domains](https://developers.cloudflare.com/workers/configuration/routing/custom-domains/)

## Verify a rollout

Check `/readyz` at the deployed hostname. On a cold start, the Worker can return `503` with `Retry-After` while the first child Git clones finish. A successful readiness response confirms the first snapshot. To detect a child clone or manifest error, check the authenticated `/v1/actions` response and require an empty `unreadable` array and each expected action. In token mode, issue a token in a trusted shell using the same `DATABASE_URL`. Store the token in your secret manager and pass it to the CLI via `SEIKIKA_TOKEN`. [Token creation](../../site/src/content/docs/operators/access.md#issue-a-personal-access-token)

```sh
curl -fsS --retry 12 "$SEIKIKA_URL/readyz"
seikika run --url "$SEIKIKA_URL" -o json
```

Use a second identity to confirm it cannot preview an action granted only to the operator. For actions with approval requirements, add the `approval_key` returned by the parent to each child under `federation.acceptApprovalsFrom` and verify a test run. [Federate repository actions](../federated-actions/README.md)

Use the same Durable Object name and a single container instance. The container's checkout cache is rebuilt after an idle shutdown or replacement. Size its disk for full clones of the configured children. PostgreSQL and the signing secret survive replacement. If you change the portal image or a secret passed at process startup, roll the container and recheck readiness and discovery. Keep snapshot refresh and long actions in mind when tuning the idle timeout in `worker.mjs`. [Container operation](https://developers.cloudflare.com/containers/) · [Container inactivity](https://developers.cloudflare.com/containers/api/durable-object-container/#setinactivitytimeout)
