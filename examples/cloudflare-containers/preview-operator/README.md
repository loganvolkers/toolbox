# Manage isolated Cloudflare Previews

Deploy this scheduled Worker separately from the portal. It reads the public GitHub API for open, same-repository PRs targeting `main` with the `cloudflare-preview` label. Apply the label only after reviewing and trusting the branch. The Cloudflare GitHub App continues to build the portal from each branch. The operator creates an exact-host Access application, provisions its own Cloudflare-billed database, and sets runtime secrets on the branch Preview. It keeps a Preview’s signing key across subsequent builds.

The operator uses the following Worker variables.

| Variable | Purpose |
| --- | --- |
| `GITHUB_REPOSITORY` | Repository in `owner/name` form |
| `CLOUDFLARE_ACCOUNT_ID` | Cloudflare account owning the portal |
| `CLOUDFLARE_WORKER` | Portal Worker name |
| `WORKER_SUBDOMAIN` | Account's workers.dev subdomain label |
| `CLOUDFLARE_ACCESS_ISSUER` | Zero Trust issuer URL |
| `CLOUDFLARE_IDP_ID` | Restrict-to-account-members Cloudflare identity provider |
| `PLANETSCALE_ORG` | Cloudflare-linked PlanetScale organization |
| `PLANETSCALE_PREVIEW_REGION` | Region available in that organization |
| `PLANETSCALE_PREVIEW_SKU` | Enabled single-node PostgreSQL cluster SKU |

Set `CLOUDFLARE_API_TOKEN`, `PLANETSCALE_SERVICE_TOKEN_ID`, `PLANETSCALE_SERVICE_TOKEN`, and `PREVIEW_ADMIN_EMAIL` as Worker secrets. Use an account-owned Cloudflare token with Workers Scripts Write, Workers Containers Write, Access Apps and Policies Write, and Hyperdrive PlanetScale Setup. Grant the PlanetScale service token the organization-level `create_databases` permission. It receives database permissions on the databases it creates. Set the admin email to an accepted Cloudflare account member. GitHub requires no token for this public repository. [Cloudflare account tokens](https://developers.cloudflare.com/fundamentals/api/get-started/account-owned-tokens/) · [PlanetScale service tokens](https://planetscale.com/docs/api/service-tokens/#creating-databases-with-service-tokens)

To retire a managed Preview, close the PR or remove its label. The operator uses the exact managed Access application as its ownership marker. It deletes the matching Preview, its exact-name container and its PR database before removing the Access app. A manually configured Preview keeps its independent Access application and is outside this cleanup. [Preview container cleanup](https://developers.cloudflare.com/workers/previews/resources/#container-application-cleanup)
