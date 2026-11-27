# Deploy Seikika on Cloudflare Containers

Run a Seikika portal and its OpenFGA co-process in one Cloudflare Container. The Worker routes HTTPS requests to one named instance. A direct PostgreSQL connection stores portal data, authorization tuples, personal access tokens, and action runs. This example is about the box, from packaging the portal to keeping its state across deployments. [Federated actions design](../../docs/design/federated-actions.md)

Cloudflare offers PlanetScale PostgreSQL through its dashboard with billing on the Cloudflare account. Use the database's **direct primary endpoint** and `sslmode=verify-full`. For a PlanetScale connection string that includes `sslrootcert=system`, omit that parameter so Seikika uses the system trust store. A direct connection preserves the session-level advisory lock Seikika uses to keep action execution on one instance. Hyperdrive and PgBouncer transaction pooling cannot preserve that lock. [PlanetScale billing through Cloudflare](https://developers.cloudflare.com/changelog/post/2026-06-18-planetscale-databases-cloudflare-billing/) · [Hyperdrive connection pooling](https://developers.cloudflare.com/hyperdrive/concepts/connection-pooling/)

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

The example uses the same entrypoint and co-process as the Seikika server image. Stage Linux `amd64` binaries for Seikika, its connectors, and OpenFGA. The image uses Debian Trixie, so build on a system with a compatible glibc. Choose a source revision with SQLx TLS support and run these commands from that Seikika checkout. [Seikika server image](../../image/Dockerfile)

```sh
example=examples/cloudflare-containers
cargo build --locked --release --workspace
deployment=$(mktemp -d)
bash "$example/stage.sh" "$example/portal" "$deployment"
cd "$deployment"
npm ci --ignore-scripts
npm exec -- wrangler deploy --dry-run
```

Run the deployment commands from `$deployment`. The staging helper selects the Dockerfile, Worker, Wrangler configuration, and compiled Linux binaries from the Seikika checkout. Its first argument can point to a portal in another repository. Its optional third argument selects an alternate release binaries directory.

When a release includes SQLx TLS support, you can stage its checksum-verified Linux `amd64` binaries in place of building Seikika from source. [Stage release archives](../../scripts/stage-image-archives.sh)

## Deploy with Cloudflare Workers Builds

Workers Builds compiles the checked-out Seikika revision and deploys the Worker and Container. The Cloudflare Workers & Pages GitHub App only needs access to the parent repository. Child repositories publish their public manifests and use GitHub Actions for repository checks. [GitHub integration](https://developers.cloudflare.com/workers/ci-cd/builds/git-integration/github-integration/)

Start with a separate Worker such as `seikika-portal-staging`. Create a staging PostgreSQL database or branch and add the direct connection URL and a signing seed as `DATABASE_URL` and `SEIKIKA_SIGNING_KEY` under **Settings → Variables & Secrets** for this Worker. Select **Production** within that Worker's settings to apply them to its tracked-branch deployment. Cloudflare also uses the term **Production branch** for the branch that deploys the live version of each Worker, including a staging Worker.

In the Worker's **Settings → Build**, connect the GitHub repository and use these settings.

| Setting | Value |
| --- | --- |
| Repository | `seikika/seikika` |
| Root directory | `examples/cloudflare-containers` |
| Production branch | A trusted staging branch with this example, such as `codex/cloudflare-containers` while reviewing PR #142 |
| Build command | `bash build.sh` |
| Deploy command | `cd .build && npm exec -- wrangler deploy --config ./wrangler.toml` |
| Preview command | `cd .build && npm exec -- wrangler preview --config ./wrangler.toml` |
| Build variable | `SKIP_DEPENDENCY_INSTALL=1` |
| API token | Cloudflare-managed build token |

The build script uses the name of the connected Worker in the staged Wrangler configuration. The build script compiles with the repository's pinned Rust toolchain, stages the portal under `.build`, and installs the locked Wrangler dependencies. It uses `seikika-forks/` by default. Set the build command to `bash build.sh /path/to/portal` or pass a path relative to this example directory to stage another checked-out manifest. [Build settings](https://developers.cloudflare.com/workers/ci-cd/builds/configuration/) · [Worker name override](https://developers.cloudflare.com/workers/ci-cd/builds/troubleshoot/#workers-name-requirement)

If Cloudflare reports that application detection ran from a workspace root, check the **Root directory** setting and retry the build. If Wrangler reports a missing `previews` block, check that the Preview command enters `.build` and passes `--config ./wrangler.toml`. That flag selects the staged config even when Wrangler can find a generated config.

If a build gets a Cloudflare permission error publishing the Container, inspect the build token's permissions. A custom user API token selected in the Worker's Builds settings needs Workers Editor for the existing Worker, and Containers Write for its account. Add Workers Routes Write for any zone whose routes the deploy changes. Keep the token in Cloudflare's build settings and the runtime secrets in the Worker. [Build token configuration](https://developers.cloudflare.com/workers/ci-cd/builds/configuration/#api-token) · [Worker permissions](https://developers.cloudflare.com/workers/authorization/workers/)

Verify the staging rollout before creating the production Worker. Use a separate database or branch and a separate signing key for production. Connect its full deploy to `main` once the example has landed there.

### Preview a trusted branch

Start with preview builds disabled while you verify the tracked-branch deployment. The Wrangler configuration declares a Container and Durable Object binding for each Preview. Use `wrangler preview` so each Preview gets its own image, Container application, and namespace. [Container Preview isolation](https://developers.cloudflare.com/workers/previews/resources/#containers)

1. Protect the Preview URLs with Cloudflare Access and allow your testers. Treat every branch that can deploy a Preview as trusted code.
2. Enable **Preview Builds** on the staging Worker and push a pilot branch other than its tracked branch. In Workers Builds, preview builds apply to non-production branches of the connected repository.
3. Give the pilot Preview a dedicated test database or database branch and a different signing seed. Run the commands below from the example directory after `npm ci --ignore-scripts`. Replace the Worker and Preview names with those in the dashboard. Repeat with separate credentials for other Previews that will run concurrently. Seikika keeps an advisory lock in PostgreSQL, so sharing a database can keep one Container from becoming ready.

```sh
npm exec -- wrangler preview secret put DATABASE_URL --worker-name seikika-portal-staging --name preview-pilot
npm exec -- wrangler preview secret put SEIKIKA_SIGNING_KEY --worker-name seikika-portal-staging --name preview-pilot
```

4. Open the Preview URL posted on the PR. Check `/readyz` and authenticated child discovery before expanding preview use. Delete the Preview when finished and check for a leftover Container application. [Preview secrets](https://developers.cloudflare.com/workers/previews/configuration/#secrets) · [Preview cleanup](https://developers.cloudflare.com/workers/previews/resources/#container-application-cleanup)

For browser sign-in on the seven-fork portal, create an Access application for the branch Preview hostname with a policy that allows your testers. In that Preview's Workers Builds settings, set `SEIKIKA_PREVIEW_CF_TEAM_DOMAIN` to your Access issuer URL and `SEIKIKA_PREVIEW_CF_AUDIENCE` to the Access application's audience. Set either `SEIKIKA_PREVIEW_CF_ADMIN` for a seed portal admin or `SEIKIKA_PREVIEW_CF_VIEWER` for read-only access. Use the login before `@` in the account's verified email address and store it as a secret build variable if it is private. The build stages the Access settings and the selected role. [Cloudflare Access for Workers](https://developers.cloudflare.com/workers/configuration/cloudflare-access/)

## Expose the portal

Use a Cloudflare account with Containers enabled, Node.js, and Docker. Change the Worker `name` to the deployment name in your account. Verify Wrangler can build the image and bundle the Worker with `npm exec -- wrangler deploy --dry-run`. Provision a dedicated database and a role that can manage schema objects for Seikika and OpenFGA. Both use the same database and role in this example. Store the direct connection URL as the `DATABASE_URL` Worker secret and generate the signing seed once. Keep a copy of the seed in your secret manager for redeploys and disaster recovery.

Run the following commands from the staged deployment directory. The first command prompts for the database connection URL. Generate the seed with `openssl rand -hex 32` and load it from your secret manager into `SEIKIKA_SIGNING_KEY`. Use an authenticated Wrangler session or set the scoped `CLOUDFLARE_API_TOKEN` and `CLOUDFLARE_ACCOUNT_ID` in the deployment environment.

```sh
npm exec -- wrangler secret put DATABASE_URL
printf '%s\n' "$SEIKIKA_SIGNING_KEY" | npm exec -- wrangler secret put SEIKIKA_SIGNING_KEY
npm exec -- wrangler deploy
```

The defaults serve the Worker on `workers.dev` and use Seikika personal access tokens for API clients. For an interactive portal, provision a custom domain and a Cloudflare Access application for it. Set `workers_dev = false` and `preview_urls = false` if you want to use only the custom-domain hostname. Configure Access for any Preview hostnames you enable. Add the domain as a Wrangler custom-domain route, set `SEIKIKA_AUTH_MODE` to `cloudflare-access`, and add `SEIKIKA_CF_TEAM_DOMAIN` (the HTTPS issuer URL without a trailing slash) and `SEIKIKA_AUDIENCE` (the Access application audience) under `[vars]`. Seikika validates the signed Access assertion. [Configure Seikika access](../../site/src/content/docs/operators/access.md) · [Cloudflare Worker custom domains](https://developers.cloudflare.com/workers/configuration/routing/custom-domains/)

## Verify a rollout

Check `/readyz` at the deployed hostname. On a cold start, the Worker can return `503` with `Retry-After` while the HTTP server starts. The portal serves the homepage as soon as it is listening, and `/readyz` waits for the first snapshot that includes its child portals. To detect a child clone or manifest error, check the authenticated `/v1/actions` response and require an empty `unreadable` array and each expected action. In token mode, issue a token in a trusted shell using the same `DATABASE_URL`. Store the token in your secret manager and pass it to the CLI via `SEIKIKA_TOKEN`. [Token creation](../../site/src/content/docs/operators/access.md#issue-a-personal-access-token)

```sh
curl -fsS --retry 12 "$SEIKIKA_URL/readyz"
seikika run --url "$SEIKIKA_URL" -o json
```

Use a second identity to confirm it cannot preview an action granted only to the operator. For actions with approval requirements, add the `approval_key` returned by the parent to each child under `federation.acceptApprovalsFrom` and verify a test run. [Federate repository actions](../federated-actions/README.md)

Use the same Durable Object name and a single container instance. The container's checkout cache is rebuilt after an idle shutdown or replacement. Size its disk for full clones of the configured children. PostgreSQL and the signing secret survive replacement. If you change the portal image or a secret passed at process startup, roll the container and recheck readiness and discovery. Keep snapshot refresh and long actions in mind when tuning the idle timeout in `worker.mjs`. [Container operation](https://developers.cloudflare.com/containers/) · [Container inactivity](https://developers.cloudflare.com/containers/api/durable-object-container/#setinactivitytimeout)
