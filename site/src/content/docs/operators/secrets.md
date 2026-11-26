---
title: Connector secrets
description: Supply connector credentials through environment variables, mounted files, or a secret manager.
sidebar:
  order: 7
---

Keep connector credentials outside the manifest. Put a logical name such as `${secret:github-app-private-key}` in the connector configuration. Seikika resolves it when the connector starts. The same manifest can run in different environments with different credentials.

## Supply a local or CI secret

Use an environment variable such as `SEIKIKA_SECRET_GITHUB_APP_PRIVATE_KEY`, or mount a private directory and set `SEIKIKA_SECRETS_DIR` to its path. For a mounted file, use the exact logical name as the file name. An unresolved secret causes a warning and the connector configuration omits that value. If a connector cannot authenticate, check the logical name and the server process's access to the secret.

## Supply secrets in a shared deployment

Set `SEIKIKA_SECRETS_CONFIG` to the path of a JSON or YAML configuration file. The lookup checks environment variables and files before configured providers. You can configure Vault, AWS Secrets Manager, Google Cloud Secret Manager, or Azure Key Vault. Map each logical name to the matching provider ID.

The [Google Cloud example](https://github.com/seikika/seikika/tree/main/examples/secret-vault-gcp) shows a worked configuration. For the provider fields and lookup order, see the [secrets configuration reference](https://github.com/seikika/seikika/blob/main/docs/design/secret-vault.md).
