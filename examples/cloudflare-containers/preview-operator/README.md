# Manage isolated Cloudflare Previews

Deploy this Worker separately from the portal. The default-branch GitHub workflow sends a list of open, same-repository PRs targeting `main` with the `cloudflare-preview` label. Apply the label only after reviewing and trusting the branch. The operator verifies a short-lived GitHub Actions OIDC token for that workflow, repository ID, branch, and audience before accepting a request. The Cloudflare GitHub App continues to build the portal from each branch. The operator creates an exact-host Access application, provisions its own Cloudflare-billed database, and sets runtime secrets on the branch Preview. It keeps a Preview’s signing key across subsequent builds.

The operator uses the following Worker variables.

| Variable | Purpose |
| --- | --- |
| `GITHUB_REPOSITORY` | Repository in `owner/name` form |
| `GITHUB_REPOSITORY_ID` | GitHub's numeric repository ID, as a string |
| `GITHUB_OIDC_AUDIENCE` | Exact operator URL used as the workflow's token audience |
| `CLOUDFLARE_ACCOUNT_ID` | Cloudflare account owning the portal |
| `CLOUDFLARE_WORKER` | Portal Worker name |
| `WORKER_SUBDOMAIN` | Account's workers.dev subdomain label |
| `CLOUDFLARE_ACCESS_ISSUER` | Zero Trust issuer URL |
| `CLOUDFLARE_IDP_ID` | Restrict-to-account-members Cloudflare identity provider |
| `PLANETSCALE_ORG` | Cloudflare-linked PlanetScale organization |
| `PLANETSCALE_PREVIEW_REGION` | Region available in that organization |
| `PLANETSCALE_PREVIEW_SKU` | Enabled single-node PostgreSQL cluster SKU |

Set `CLOUDFLARE_API_TOKEN`, `PLANETSCALE_SERVICE_TOKEN_ID`, `PLANETSCALE_SERVICE_TOKEN`, and `PREVIEW_ADMIN_EMAIL` as Worker secrets. Use an account-owned Cloudflare token with Workers Scripts Write, Workers Containers Write, Access Apps and Policies Write, and Hyperdrive PlanetScale Setup. Grant the PlanetScale service token the organization-level `create_databases` permission. It receives database permissions on the databases it creates. Set the admin email to an accepted Cloudflare account member. The GitHub workflow uses its built-in repository token to read the PR list and OIDC to authenticate to Cloudflare. [Cloudflare account tokens](https://developers.cloudflare.com/fundamentals/api/get-started/account-owned-tokens/) · [PlanetScale service tokens](https://planetscale.com/docs/api/service-tokens/#creating-databases-with-service-tokens) · [GitHub Actions OIDC](https://docs.github.com/en/actions/concepts/security/openid-connect)

To retire a managed Preview, close the PR or remove its label. The next scheduled workflow reconciles it. To run reconciliation immediately, dispatch the Cloudflare Preview lifecycle workflow on `main`. The operator uses the exact managed Access application as its ownership marker. It deletes the matching Preview, its exact-name container and its PR database before removing the Access app. [Preview container cleanup](https://developers.cloudflare.com/workers/previews/resources/#container-application-cleanup)

## Verify the operator

From `examples/cloudflare-containers`, check that Wrangler can bundle the operator.

```sh
npm ci --ignore-scripts
npm exec -- wrangler deploy --dry-run --config preview-operator/wrangler.toml
```

Label two trusted PRs and run the lifecycle workflow on `main`. Verify that each Preview has an Access policy, a distinct database, and its own approval public key in the authenticated action response. Remove one label and confirm that the other Preview still works after reconciliation. [Federated actions design](../../../docs/design/federated-actions.md)
