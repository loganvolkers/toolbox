---
title: Connector secrets
description: Point ${secret:…} at env, a mounted directory, or cloud secret managers without putting values in git.
sidebar:
  order: 3
---

Connector configs in the manifest use `${env:…}` and `${secret:…}` placeholders. They are not
resolved at build time (one manifest for every environment). The server and `toolbox report`
resolve them when a connector starts.

## Local and CI

```bash
export TOOLBOX_SECRETS_DIR=$PWD/.secrets
echo "$PEM" > .secrets/github-app-private-key
# or: export TOOLBOX_SECRET_GITHUB_APP_PRIVATE_KEY=…
```

Unresolved secrets are left out of the connector config (with a warning) so a connector can tell
“not configured” from “configured”.

## Production backends

Configure a chain with `TOOLBOX_SECRETS_CONFIG` (JSON or YAML file). Env and file stay first; then
opt in to Vault, AWS Secrets Manager, GCP Secret Manager or Azure Key Vault. Map logical names to
provider ids so `${secret:github-app-private-key}` keeps working when the backing store changes.

The GCP worked example (GitHub App PEM, Jira token, GCP SA JSON) lives in the repository at
[`examples/secret-vault-gcp`](https://github.com/loganvolkers/toolbox/tree/main/examples/secret-vault-gcp).
Design notes: [`docs/design/secret-vault.md`](https://github.com/loganvolkers/toolbox/blob/main/docs/design/secret-vault.md).
