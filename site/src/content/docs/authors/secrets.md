---
title: Secrets in connector config
description: Use logical ${secret:…} names; document local env and file fallbacks; never commit secret values.
sidebar:
  order: 2
---

When a connector needs a credential, put a **logical** placeholder in the example manifest:

```yaml
config:
  privateKey: ${secret:github-app-private-key}
  token: ${secret:jira-app-token}
```

Do not paste PEMs, tokens or service-account JSON into the repository. Connector authors run with:

- `TOOLBOX_SECRET_<NAME>` (hyphens become underscores), or
- files under `TOOLBOX_SECRETS_DIR` (gitignored `.secrets/`).

Operators map those logical names to Vault or cloud secret managers. Your connector code only reads
the resolved `TOOLBOX_CONNECTOR_CONFIG` JSON. See the
[operator secrets page](/toolbox/operators/secrets/) and
[`examples/secret-vault-gcp`](https://github.com/loganvolkers/toolbox/tree/main/examples/secret-vault-gcp).
