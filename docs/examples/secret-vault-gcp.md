# GCP Secret Manager for GitHub, Jira and GCP connector secrets

Operators store SaaS credentials in GCP Secret Manager; manifests only reference logical names.
Full walkthrough, YAML and local fallback: [examples/secret-vault-gcp](../../examples/secret-vault-gcp/README.md).

| `${secret:…}` | GCP SM secret id | Used by |
|---|---|---|
| `github-app-private-key` | `toolbox-github-app-pem` | GitHub App `privateKey` |
| `jira-app-token` | `toolbox-jira-token` | Jira `token` (example connector) |
| `gcp-sa-key` | `toolbox-gcp-connector-sa` | GCP `serviceAccountJson` (example) |

Design: [secret-vault.md](../design/secret-vault.md). Decision: D01M8174KFJCBKKG37B9SS8P9RF.
