# Seikika demo observability

The hosted demo sends OTLP/HTTP protobuf to a Cloudflare Worker. The Worker checks an ingest-only bearer token and forwards just the three OTLP paths to a private OpenTelemetry Collector. The Collector keeps an allowlist before storing data in Grafana LGTM. Grafana's anonymous Viewer can open Explore and query the demo's Loki, Tempo, and Prometheus data. The Worker denies login, admin routes, and writes other than Grafana's read-only query endpoint. [Grafana Viewer settings](https://grafana.com/docs/grafana/latest/setup-grafana/configure-grafana/#viewers_can_edit)

[Guest walkthrough](https://seikika.com/demo/observability/) · [Operator playbook](https://seikika.com/operators/observability/)

PostHog documents a hobby Docker Compose deployment sized for a Linux VM with
4 vCPUs, 16 GB RAM, and more than 30 GB of storage. Its application metrics
are in open alpha. Grafana LGTM packages the three signal backends, Collector,
and dashboard UI into one demo container. [PostHog self-hosting](https://posthog.com/docs/self-host) ·
[PostHog application metrics](https://posthog.com/docs/metrics) ·
[Grafana LGTM](https://grafana.com/docs/opentelemetry/docker-lgtm/)

## Deploy

From the parent Cloudflare example directory, install the locked Wrangler dependencies. Use a Cloudflare account with Containers enabled, a delegated hostname in your zone, and Docker.

```sh
npm ci --ignore-scripts
npm exec -- wrangler deploy --config observability/wrangler.toml
npm exec -- wrangler secret put OTLP_INGEST_TOKEN --config observability/wrangler.toml
```

Generate a 32-byte hex token using `openssl rand -hex 32` and store it in your secret manager. Copy the same value to the source Worker's `SEIKIKA_OTLP_TOKEN` secret. Add `SEIKIKA_OTLP_ENDPOINT` to the source Worker with the HTTPS base URL of the observability Worker. These bindings are optional and both are needed to enable export. Roll the source Container after changing the runtime secrets. Use a dedicated Collector destination if configuring a preview. [Worker secrets](https://developers.cloudflare.com/workers/configuration/secrets/) · [Container lifecycle](https://developers.cloudflare.com/containers/concepts/architecture/)

Open the dashboard in a browser context with empty storage. Generate a guest catalog read on the portal and confirm that Grafana can retrieve fresh series, a request log, and its trace. Verify a POST to each OTLP signal path without a token returns 401 and that guest writes to a dashboard return 405. Always verify the query results, since a receiver can accept a batch that the filter subsequently drops.

## Data and operation

The allowlist keeps read requests for `/`, `/search`, `/v1/views/home`, `/v1/entities`, `/healthz`, and `/readyz` for the `seikika-demo` resource. It drops other application records and keeps bounded HTTP attributes, the fixed request-completed log, trace correlation IDs, and the three named ingest and freshness metrics. Avoid adding new paths that contain identifiers or derive attributes from user input. Verify the boundary again after instrumenting a new route.

The Container runs one instance with local storage. Dashboard provisioning is in Git and reloads on a replacement. Recent telemetry can disappear when the Container is replaced. This demo has no data durability objective or customer telemetry. For a production environment, use durable backends, private operator access, and monitoring for export failures and storage pressure. [Cloudflare Container storage](https://developers.cloudflare.com/containers/faq/) · [Grafana LGTM demo stack](https://grafana.com/docs/opentelemetry/docker-lgtm/)

If the public dashboard needs to be withdrawn, detach its custom domain from the observability Worker and stop the demo producer by removing both optional source bindings. Roll the source Container so it picks up the change. Restore guest access only after the boundary and happy-path checks pass.
