---
title: Authentication and authorization
description: Choose how callers sign in and verify what each caller can read in a shared portal.
sidebar:
  order: 6
---

Authentication identifies the caller. Authorization decides what that caller can see and do. Configure both before you expose a portal to a team.

:::caution
Development mode accepts the caller's claimed identity. Keep it on a loopback address and leave the off-loopback override unset on shared deployments. A portal without an active authorization model treats authenticated callers as administrators.
:::

## Choose an authentication mode

Set `SEIKIKA_AUTH_MODE` explicitly for a shared portal. The default is `dev` on loopback addresses and `token` elsewhere.

| Mode | Caller supplies | Operator configures |
|---|---|---|
| `dev` | A development identity, such as CLI `--as alice` | A private loopback listener for fixture work |
| `token` | A personal access token in an HTTP Bearer header or `SEIKIKA_TOKEN` for the CLI | A token tied to a user and a secure way to give it to that user |
| `oidc` | An OpenID Connect (OIDC) token | `SEIKIKA_ISSUER`, `SEIKIKA_AUDIENCE`, `SEIKIKA_JWKS_URL`, and optionally `SEIKIKA_SUBJECT_CLAIM` |
| `iap-google` | The signed assertion from Google Identity-Aware Proxy | The proxy and `SEIKIKA_AUDIENCE` |
| `cloudflare-access` | The signed assertion from Cloudflare Access | `SEIKIKA_CF_TEAM_DOMAIN` and `SEIKIKA_AUDIENCE` |

The default subject claim is `email`. Confirm the mapped portal identity with `GET /v1/me` before granting access. Keep the portal behind HTTPS and prevent clients from bypassing the access proxy. See the [authentication implementation](https://github.com/seikika/seikika/blob/main/crates/seikika-authz/src/authn.rs) for exact header names and token validation.

## Configure permissions

1. Add `authz` to the manifest and supply an OpenFGA model and its relationships. OpenFGA checks access based on relationships between people, groups, and catalog entities. Use the [reference portal's model and tests](https://github.com/seikika/seikika/tree/main/examples/reference-portal/authz) as a starting point.
2. Ensure the OpenFGA executable is available to the server. Set `SEIKIKA_OPENFGA_BIN` to its path when needed. The server starts OpenFGA and loads the model.
3. Verify one allowed and one denied or redacted read as different callers. Check the scorecard details as well as the entity itself. A caller may be able to see a score while its evidence stays restricted.

## Issue a personal access token

Use the same Postgres database as the portal. In a trusted operator shell with `DATABASE_URL` already set, run:

```bash
seikika token create --user alice --label "Alice portal access" --database-url "$DATABASE_URL"
```

The command prints the token once. Postgres stores its hash. Give the token to the named user through your secret distribution process. The [HTTP API reference](/users/api/) shows how to use it.

Keep [connector secrets](/operators/secrets/) separate from user credentials. For vulnerability reporting, follow the [security policy](https://github.com/seikika/seikika/security/policy).
