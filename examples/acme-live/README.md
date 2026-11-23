# Acme, on a laptop

One portal image and Postgres. The catalog is read from a GitHub API (`GITHUB_API_URL`) and the
directory is pushed to the SCIM connector. Nothing in this portal is loaded from fixtures.
The design is [local simulation](../../docs/design/local-simulation.md).

Check the manifest before starting it:

```
toolbox validate examples/acme-live
```

| | |
|---|---|
| Workload | [deploy/score.yaml](deploy/score.yaml) |
| Chart | [deploy/chart](deploy/chart) |
| Compose, for the dependencies and for a full local stack | [deploy/compose.yaml](deploy/compose.yaml) |

The image listens on `0.0.0.0:$PORT`. `DATABASE_URL` is the portal database. OpenFGA uses
that same database: `serve` adds `sslmode=disable` when the URL has none, and replaces
OpenFGA's tuple table with a view of `toolbox.derived_tuples`. `GITHUB_TOKEN` is a bearer
token for the GitHub API.
`GITHUB_API_URL` is the API root the connectors call (`/orgs/acme/repos`, contents, and the
other reads). The SCIM connector listens on port 9100 inside the same container. Once it is
listening, the image pushes the directory: `bob` in `group:platform`, `alice` and `dave` in
`group:payments`, and `carol` in no group. That push is
[deploy/scim_seed.sh](deploy/scim_seed.sh). It is not a fixture load.

`deploy/up.sh --portal` builds the image from the binaries in `target/debug` and starts the
whole stack, including a local Git server and a GitHub-shaped façade. That script is how this
repository runs the stack. A deploy uses the image, the Score file, or the chart, and points
them at a GitHub API and Postgres that are already running.

Signed-in users are `X-Toolbox-User` while `TOOLBOX_AUTH_MODE=dev` (the stand-in for the
identity-aware proxy). `bob` is in `group:platform` and administers the portal. `alice` is in
`group:payments`. `carol` is in no group.

Entity pages are `/e/<id>`, for example `/e/component:payments-api?as=alice`.
