---
title: SCIM connector
description: Receive user and group changes from an identity provider over SCIM 2.0.
sidebar:
  order: 7
---

Use the System for Cross-domain Identity Management (SCIM) connector to receive user and group changes from an identity provider. Your portal can use those records to represent teams and grant or revoke access when group membership changes. [Authentication and authorization](/operators/access/)

## Set up provisioning

You need a running portal, the `seikika-connector-scim` binary from the same 0.2 source revision as the portal, and an identity provider that can send resource requests to a SCIM 2.0 endpoint. You also need two secrets, one for each connection. [Install a connector](/start/install/#build-from-source) · [Deploy and operate a portal](/operators/deploy/)

Add this entry to the `connectors` list in the portal's `seikika.yaml`. Add mappings for the `user` and `group` records your portal uses.

```yaml
connectors:
  - name: idp
    package: builtin:scim
    runtime: remote
    records: [user, group]
```

1. On the portal server, set `SEIKIKA_CONNECTOR_TOKENS` to include `idp=<ingest-token>`. Replace the placeholder with a secret. Set `SEIKIKA_CONNECTOR_TOKEN` in the SCIM process to the same secret. The connector uses this token when it sends records to the portal.
2. Set `SEIKIKA_SCIM_TOKEN` in the SCIM process to a separate secret. Give that value to your identity provider as its provisioning bearer token. Keep both values in the service's secret configuration.
3. Start the connector. The example assumes the portal is on the same host at port 8080. Set `--seikika` to the URL the connector can reach. The `--connector` value must match the manifest name.

```bash
seikika-connector-scim \
  --seikika http://127.0.0.1:8080 \
  --listen 127.0.0.1:9100 \
  --connector idp
```

Expose the connector to your identity provider through an HTTPS proxy that enforces the provider's bearer token on every request. The connector itself checks the token on write requests. Keep the internal listener on loopback when the proxy runs on that host. Set the provider's provisioning base URL to the proxy URL with `/scim/v2` appended. Start with test users and groups. [Sample user and group mappings](https://github.com/seikika/seikika/blob/main/examples/reference-portal/seikika.yaml)

## Check a membership change

1. Provision one test user and one group containing that user. Provision the user first so the connector can resolve the group's member ID.
2. Read the mapped test user and group from the portal. Check the user's login and the group's membership list.
3. Remove the member through your identity provider and read the group again. If membership controls access, check that the user loses the access granted by that group.

| Record kind | Data it contains |
|---|---|
| `user` | Login, display name, and email |
| `group` | Slug, display name, member logins, and optional parent and GitHub team |

SCIM is push-only. The provider sends changes and deletes, so resync and refresh commands are unavailable. The current connector expects complete user or group objects for updates. Check that your provider can send them, especially if it uses SCIM PatchOp operations. The connector keeps resource IDs in memory. After restarting it, replay users before groups to restore that lookup state. [Provisioning script](https://github.com/seikika/seikika/blob/main/examples/reference-portal/tools/scim_feed.py) · [Connector acceptance test](https://github.com/seikika/seikika/blob/main/scripts/m4-acceptance.sh)

## Troubleshoot provisioning

- If the provider gets `401`, check the bearer token it sends against `SEIKIKA_SCIM_TOKEN`.
- If the provider gets `502`, check the connector's reachability to the portal, its connector name, and the ingest token on both processes.
- If a group arrives without members, check that its member IDs refer to users the current connector process has received.
