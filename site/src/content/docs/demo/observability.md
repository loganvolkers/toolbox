---
title: Live runtime dashboard
description: Follow a real request through the Seikika demo's OpenTelemetry metrics, logs, and traces.
---

[Open the live runtime dashboard](https://observe.seikika.com/) to see real requests and connector activity from the hosted Seikika demo. Grafana opens in a read-only guest session without an account.

## Follow a request

1. Open the [hosted demo](https://demo.seikika.com/) in a second tab. Refresh its home page or select **Catalog**. Those actions make real HTTP requests.
2. In Grafana, choose **Last 15 minutes** and refresh. In **Recent request logs**, look for a new `GET /`, `GET /v1/views/home`, or `GET /v1/entities` row.
3. Expand the row and select the trace link. In Tempo, open the `http.server.request` span and inspect its duration, path, and HTTP status. The trace and log share the same trace ID.
4. Check **Snapshot lag** for the last reported snapshot age and **Connector records and batches** for the process's running ingest totals. Refreshing the portal exercises HTTP. The connector counters move when a connector publishes data.

**Request rate by outcome** and **95th-percentile request latency** are computed from traces for the routes included in the guest view. A quiet demo may have too few requests for a useful percentile. New batches take a short while to reach the panels.

## What guests can see

The demo exports a short list of public or probe paths, request methods and status codes, and three ingest and freshness metrics. The dashboard combines Grafana, Loki, Tempo, Prometheus, and an OpenTelemetry Collector running in a dedicated Cloudflare Container. A replacement of the container can reset the visible history.

To investigate readiness failures, missing exports, or stale catalog data on your own portal, follow the [operator observability playbook](/operators/observability/).
