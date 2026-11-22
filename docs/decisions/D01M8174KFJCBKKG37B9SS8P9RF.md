---
id: D01M8174KFJCBKKG37B9SS8P9RF
decided: 2026-11-22T09:00:00Z
---

# Connector `${secret:…}` resolves through a pluggable backend chain

**Extends D55.** `${env:…}` and `${secret:…}` stay placeholders until runtime. At resolve time,
`${secret:name}` walks a chained `SecretBackend` in `crates/toolbox-secrets`: env
(`TOOLBOX_SECRET_*`), mounted directory (`TOOLBOX_SECRETS_DIR`), then opt-in remotes (HashiCorp
Vault KV, AWS Secrets Manager, GCP Secret Manager, Azure Key Vault). Manifests keep stable logical
names; operators map them to provider resources in runtime config. Unresolvable secrets are omitted
with a warning unless `required` is set. Cloud access uses HTTPS clients behind Cargo features —
no cloud SDKs required for default CI.

**Status:** ✅

**Where:** [design/secret-vault.md](../design/secret-vault.md), [v1-spec.md](../v1-spec.md) §10.2,
[examples/secret-vault-gcp](../../examples/secret-vault-gcp/README.md)
