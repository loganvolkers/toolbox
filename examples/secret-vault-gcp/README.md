# GCP Secret Manager → Toolbox connectors

Worked example: the same logical `${secret:…}` names resolve from a local file mount in
dev, and from GCP Secret Manager in production, without changing connector config.

## Secrets to create

| Logical `${secret:…}` | GCP SM secret id | Payload |
|---|---|---|
| `github-app-private-key` | `toolbox-github-app-pem` | GitHub App PEM |
| `jira-app-token` | `toolbox-jira-token` | Jira API token |
| `gcp-sa-key` | `toolbox-gcp-connector-sa` | SA JSON for the *GCP connector* (data plane) |

`jira` and `gcp` packages are config sketches for connectors that are not first-party yet;
resolution and mapping are what this example proves. GitHub reuses `builtin:github`.

Runtime backend config lives in [`secrets.yaml`](./secrets.yaml) (no secret *values*). Point
`TOOLBOX_SECRETS_CONFIG` at it. The manifest keeps only logical placeholders (D55).

## Operator (prod)

1. Create the three secrets in project `P`.
2. Grant the Toolbox runtime SA `roles/secretmanager.secretAccessor` (prefer WIF/ADC; do not use a long-lived key for Toolbox→SM when avoidable).
3. Set `TOOLBOX_SECRETS_CONFIG` to this directory's `secrets.yaml` (edit `project:`), or equivalent env (`TOOLBOX_GCP_SECRETS_PROJECT` + `TOOLBOX_GCP_SECRETS_MAP`).
4. `toolbox serve` / deploy — placeholders resolve at start-up.

```bash
gcloud secrets create toolbox-github-app-pem --data-file=./github-app.pem
gcloud secrets create toolbox-jira-token --data-file=./jira.token
gcloud secrets create toolbox-gcp-connector-sa --data-file=./gcp-sa.json

export TOOLBOX_SECRETS_CONFIG=$PWD/examples/secret-vault-gcp/secrets.yaml
# local smoke only:
export GOOGLE_ACCESS_TOKEN="$(gcloud auth print-access-token)"
```

## Connector author (local, no GCP)

```bash
mkdir -p .secrets   # gitignore this
cp ~/github-app.pem .secrets/github-app-private-key
echo "$JIRA_TOKEN" > .secrets/jira-app-token
cp ~/sa.json .secrets/gcp-sa-key
export TOOLBOX_SECRETS_DIR=$PWD/.secrets
export GITHUB_APP_ID=…
export GITHUB_INSTALLATION_ID=…
# same toolbox.yaml — file backend satisfies ${secret:…}
```

Or: `export TOOLBOX_SECRET_GITHUB_APP_PRIVATE_KEY="$(cat …)"` (env backend).

## CI

```bash
cargo test -p toolbox-secrets
```

Unit tests use trait fakes; `tests/wiremock_remotes.rs` exercises HTTP against wiremock. Do not
commit PEMs. Optional nightly: OIDC → SM read smoke.

## Files

- [`toolbox.yaml`](./toolbox.yaml) — connector configs with logical `${secret:…}` only
- [`secrets.yaml`](./secrets.yaml) — backend chain + GCP map (runtime config)

Design: [`docs/design/secret-vault.md`](../../docs/design/secret-vault.md).
