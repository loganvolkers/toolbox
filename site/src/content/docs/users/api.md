---
title: HTTP API reference
description: Read entities, scorecards, and evidence through the portal's HTTP API.
sidebar:
  order: 2
---

Use the HTTP API to read the same catalog that the CLI and Model Context Protocol (MCP) expose. The portal applies the caller's permissions before returning fields and scorecard details.

## Make a request

For a shared token-based portal, set `SEIKIKA_URL` to its HTTPS URL and load `SEIKIKA_TOKEN` from your secret store. The HTTP header is `Authorization: Bearer …`. Run this in Bash and substitute an entity ID from your catalog.

```bash
curl -fsS -H "Authorization: Bearer $SEIKIKA_TOKEN" \
  "$SEIKIKA_URL/v1/entities/repository:checkout-web/scorecards"
```

With [the fixture portal](/start/portal/) running, you can make the read on loopback with a fixture identity:

```bash
curl -fsS -H 'x-seikika-user: bob' \
  'http://127.0.0.1:8080/v1/entities/repository:checkout-web/scorecards'
```

In the fixture response, `scorecards.ownership.level` is `bronze`. The `snapshot` field identifies the catalog version used for the read. A caller can receive a level while rule details are replaced with a redaction marker.

## Common reads

All paths below use `GET`. Replace each path parameter with a URL-encoded value from your portal.

| Path | Result |
|---|---|
| `/v1/me` | The identity the portal uses for this caller |
| `/v1/entities?kind=Repository` | Visible entities. Use `q` for text search, or `filter` for a Common Expression Language (CEL) filter. Pass the returned `next_cursor` as `cursor` when there are more results. |
| `/v1/entities/{id}` | An entity and its field sources. Use `fields` to select a comma-separated field list. |
| `/v1/entities/{id}/graph` | Related entities and edges |
| `/v1/entities/{id}/scorecards` | Scorecard levels and visible rule results for one entity |
| `/v1/entities/{id}/explain?rule=ownership%2Fbranch-protected` | Evidence for one rule. Use `field` to explain a field instead. |
| `/v1/scorecards/{name}` | Results for one scorecard across visible entities |
| `/v1/campaigns/{name}` | Goals and progress for one campaign |

For connector and operator endpoints, see the [HTTP router](https://github.com/seikika/seikika/blob/main/crates/seikika-api/src/routes.rs) and [API design](https://github.com/seikika/seikika/blob/main/docs/v1-spec.md#72-rest-api-v1-openapi-31-generated-per-portal). Check [releases and upgrades](/start/versions/) before changing a client or the portal version.

## Snapshots and errors

Many catalog reads include an `x-seikika-snapshot` response header. For entity and entity scorecard reads, pass `min_snapshot` with a version or a full snapshot ID to wait for a catalog at least that recent. The server returns 503 with `snapshot-not-ready` if it cannot serve that version within its wait limit. Retry with backoff when the application still needs the update.

| Result | What to do |
|---|---|
| 400 with `bad-min-snapshot` | Use a numeric version or an ID in the form `version.hash`. |
| 401 with `unauthenticated` or `invalid-credentials` | Check how your operator expects clients to authenticate, then check the credential. |
| 404 with `not-found` | Check the ID and your access. An entity you cannot see can return 404. |
| 503 with `starting` or `snapshot-not-ready` | Check [readiness](/operators/deploy/#verify-the-rollout) and retry. |

Errors from the portal use `application/problem+json` and include a `code` and human-readable `detail`. Use the status and code for client logic.
