---
title: Secrets in connector config
description: Declare a logical secret name and read the resolved connector configuration.
sidebar:
  order: 3
---

In an example manifest, use a logical placeholder for each credential:

```yaml
config:
  privateKey: ${secret:github-app-private-key}
  token: ${secret:jira-app-token}
```

Read the resolved fields from `SEIKIKA_CONNECTOR_CONFIG` in your connector. Document which fields are required and give a clear configuration error if a required value is missing. Keep credential values out of logs and records.

During local development, supply a secret through its environment variable or a file in a private directory. See [Connector secrets](/operators/secrets/) for the naming rules and for the secret stores an operator can use.
