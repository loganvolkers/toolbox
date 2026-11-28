# GCP Secret Manager → Seikika connectors

Worked example: the same logical `${secret:…}` names resolve from a local file mount in
dev, and from GCP Secret Manager in production, without changing connector config.

## Secrets to create

| Logical `${secret:…}` | GCP SM secret id | Payload |
|---|---|---|
| `github-app-private-key` | `seikika-github-app-pem` | GitHub App PEM |
| `jira-app-token` | `seikika-jira-token` | Jira API token |
| `gcp-sa-key` | `seikika-gcp-connector-sa` | SA JSON for the *GCP connector* (data plane) |

`jira` and `gcp` packages are config sketches for connectors that are not first-party yet;
resolution and mapping are what this example proves. GitHub reuses `builtin:github`.

Runtime backend config lives in [`secrets.yaml`](./secrets.yaml) (no secret *values*). Point
`SEIKIKA_SECRETS_CONFIG` at it. The manifest keeps only logical placeholders (D55).

## Operator (prod)

1. Create the three secrets in project `P`.
2. Grant the Seikika runtime SA `roles/secretmanager.secretAccessor` (prefer WIF/ADC; do not use a long-lived key for Seikika→SM when avoidable).
3. Set `SEIKIKA_SECRETS_CONFIG` to this directory's `secrets.yaml` (edit `project:`), or equivalent env (`SEIKIKA_GCP_SECRETS_PROJECT` + `SEIKIKA_GCP_SECRETS_MAP`).
4. `seikika serve` / deploy — placeholders resolve at start-up.

```bash
gcloud secrets create seikika-github-app-pem --data-file=./github-app.pem
gcloud secrets create seikika-jira-token --data-file=./jira.token
gcloud secrets create seikika-gcp-connector-sa --data-file=./gcp-sa.json

export SEIKIKA_SECRETS_CONFIG=$PWD/examples/secret-vault-gcp/secrets.yaml
# local smoke only:
export GOOGLE_ACCESS_TOKEN="$(gcloud auth print-access-token)"
```

## Connector author (local, no GCP)

```bash
mkdir -p .secrets   # gitignore this
cp ~/github-app.pem .secrets/github-app-private-key
echo "$JIRA_TOKEN" > .secrets/jira-app-token
cp ~/sa.json .secrets/gcp-sa-key
export SEIKIKA_SECRETS_DIR=$PWD/.secrets
export GITHUB_APP_ID=…
# same seikika.yaml — file backend satisfies ${secret:…}
```

Install the GitHub App on the configured organization. The connector looks up its installation when it runs.

Or: `export SEIKIKA_SECRET_GITHUB_APP_PRIVATE_KEY="$(cat …)"` (env backend).

## CI

```bash
cargo test -p seikika-secrets
```

Unit tests use trait fakes; `tests/wiremock_remotes.rs` exercises HTTP against wiremock. Do not
commit PEMs. Optional nightly: OIDC → SM read smoke.

## Files

- [`seikika.yaml`](./seikika.yaml) — connector configs with logical `${secret:…}` only
- [`secrets.yaml`](./secrets.yaml) — backend chain + GCP map (runtime config)

Design: [`docs/design/secret-vault.md`](../../docs/design/secret-vault.md). Decision: D01M8174KFJCBKKG37B9SS8P9RF.
