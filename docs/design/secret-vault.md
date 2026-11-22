# Secret vault for connector auth

**Layer:** operator SDLC (runtime binding) and connector-author SDLC (local env/file).  
**Box vs tools:** this is the **box** — carrying and resolving secret *references* into connector
config. Cloud secret managers and Vault remain external **tools**; Toolbox fits them behind one
trait.  
**Decision:** [D01M8174KFJCBKKG37B9SS8P9RF](../decisions/D01M8174KFJCBKKG37B9SS8P9RF.md) (extends D55).  
**Crate:** `crates/toolbox-secrets`.  
**Status:** implemented for env, file mount, Vault KV v2, AWS Secrets Manager, GCP Secret Manager,
Azure Key Vault.

## 1. Summary

Manifests keep `${env:…}` and `${secret:…}` placeholders through `build` (D55). At serve / report /
connector-test time, `resolve_config` expands them through a chained [`SecretBackend`] list.
Connectors keep stable logical names (`github-app-private-key`); operators map those names to
provider resources without editing connector code.

## 2. Resolution order (first hit wins)

1. After placeholder resolve, merge `TOOLBOX_CONNECTOR_CONFIG_OVERRIDES` (dev/test).
2. `${env:NAME}` → process environment.
3. `${secret:NAME}` → backend chain:
   1. **Env** — `TOOLBOX_SECRET_<NAME>` (`-` → `_`, uppercased)
   2. **File** — `TOOLBOX_SECRETS_DIR/<name>` (also `NAME`, `*.txt`) — v1-spec §10.2
   3. **Configured remotes** — Vault, AWS SM, GCP SM, Azure KV (order from config)

Nesting: string leaves that are exactly `${…}` resolve recursively through objects and arrays.
Partial-string interpolation is unsupported.

Failure default: omit the key and warn. `secrets.required: true` / `TOOLBOX_SECRETS_REQUIRED=1`
fails instead.

## 3. Configuration

| Source | Purpose |
|---|---|
| `TOOLBOX_SECRETS_DIR` | Mounted secrets directory |
| `TOOLBOX_SECRETS_CONFIG` | JSON/YAML path or inline JSON (`backends`, `map`, `required`, `refs`) |
| `TOOLBOX_SECRETS_REFS` | JSON logical → URI (`gcp-sm:projects/…/secrets/…`) |
| `TOOLBOX_GCP_SECRETS_PROJECT` + `TOOLBOX_GCP_SECRETS_MAP` | Opt-in GCP SM |
| `TOOLBOX_VAULT_ADDR` / `VAULT_TOKEN` | Opt-in Vault |
| `TOOLBOX_AWS_SECRETS_ENABLE` / `TOOLBOX_AWS_SECRETS_ENDPOINT` | Opt-in AWS SM |
| `TOOLBOX_AZURE_KEY_VAULT_URL` + token env | Opt-in Azure KV |

Remote backends use HTTPS via `reqwest` (Cargo features `vault` / `aws` / `gcp` / `azure`); there
are no cloud SDKs on the default CI path. Auth to the backend is env/workload-identity tokens
injected as headers; HTTP is pluggable for tests.

## 4. GCP worked example

See [examples/secret-vault-gcp](../../examples/secret-vault-gcp/README.md):

| `${secret:…}` | GCP SM id | Consumer |
|---|---|---|
| `github-app-private-key` | `toolbox-github-app-pem` | `builtin:github` |
| `jira-app-token` | `toolbox-jira-token` | example jira config |
| `gcp-sa-key` | `toolbox-gcp-connector-sa` | example gcp config |

Toolbox’s own access to SM should use ADC / Workload Identity. The GCP connector’s SA JSON is a
*payload* secret stored in SM, not the resolver’s credentials when avoidable.

## 5. SDLC

| Mode | Typical backends |
|---|---|
| Connector-author local | Env + file under gitignored `.secrets/` |
| CI | Env / file from the CI secret store; fakes in unit tests |
| Operator prod | GCP/AWS/Azure/Vault (+ optional CSI file mount) |

## 6. Tests

`cargo test -p toolbox-secrets` — unit fakes per backend, chain precedence, nested resolve,
redaction, wiremock HTTP for remotes.

## Sources

- [secret-vault-design.md](https://github.com/loganvolkers/toolbox) (project store design, mirrored here)
- D55 — placeholders until server start
- [v1-spec.md](../v1-spec.md) §5, §10.2
- [connector-control.md](./connector-control.md)
