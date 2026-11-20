# `github-apps` in-image executor (specification)

The executor behind `install-approved-app` and `remove-app`. It ships inside the portal image at
`/portal/executors/github-apps/bin/github-apps` (D40) and speaks the executor protocol over stdio
([`schema/executor-messages.schema.json`](../../schema/executor-messages.schema.json); a full
conversation is in [`protocol/install-approved-app.executor.ndjson`](../../protocol/install-approved-app.executor.ndjson)).
Any language works; this spec is what the implementation must do.

## Behaviour

| Message | Behaviour |
|---|---|
| `preview` | Read-only. Use `related.installation[0]` (resolved by Toolbox from the catalog). Read the app's current permissions with `GET /apps/{app_slug}`. Return one `grant` effect (installation → repository, with permissions). Warn if the app's permissions differ from `policy.approvedApps[app].permissions`. Block if no installation exists in the org, or if the app holds `administration`, `organization_administration` or `repository_creation` write permissions (owner-only, see the GitHub Apps example §2.3) |
| `run` | `PUT /user/installations/{installation_id}/repositories/{repository_id}` (install) or `DELETE` (remove), API version `2026-03-10`, with `identity.credentials.github.token`. Must not exceed `confirmed_effects`. Treat "already added/removed" as success. Map 403 → `forbidden` (not retryable), 404 → `not_found` (not retryable), 422 → `invalid` (not retryable), 5xx and secondary rate limits → retryable |
| `verify` | Used only if the connector refresh can't observe the change: `GET /user/installations/{installation_id}/repositories` with the user's token and check membership. Returns `observed` |
| `cancel` | Before the API call: stop and report `failed` with `cancelled`. After: report the real outcome |

Endpoints are from GitHub's REST OpenAPI description (`github/rest-api-description` at `c9c95fe`):
the add/remove operations are `apps/add-repo-to-installation-for-authenticated-user` and
`apps/remove-repo-from-installation-for-authenticated-user`, both marked usable by GitHub Apps. The
description at that commit doesn't yet mention the `installation_repositories` permission from the
13 Oct 2026 changelog, so which token types work is still verified in the sandbox spike (Q8).

## Tests (G2)

- Recorded cassettes for each status above; preview snapshot tests; idempotent retry with the same
  `idempotency_key`; a run whose `confirmed_effects` differ from a fresh preview must fail with
  `preview_stale` before calling GitHub.
- Sandbox-org integration test in CI against a real test app.
