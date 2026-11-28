---
title: Monitor and troubleshoot a portal
description: Use Seikika's runtime signals to triage incidents, verify recovery, and test instrumentation changes.
---

Start with the affected user journey. Test the catalog or another representative read from outside the hosting environment, then compare HTTP status, latency, logs, traces, and dependency health. The [live runtime walkthrough](/demo/observability/) shows the guest view using the hosted demo.

## Configure and verify export

Seikika emits OpenTelemetry traces, structured logs, and selected catalog metrics. Export the three signals to a Collector and keep its receiver credentials in the deployment secret store. Use TLS for cross-host export and keep the Prometheus scrape endpoint on the operator network. The [metric catalog](https://github.com/seikika/seikika/blob/main/crates/seikika-telemetry/METRIC_CATALOG.md) lists the scrape contract and units. The guest demo charts the three native OTLP ingest and snapshot instruments.

| Setting | Example or behavior |
| --- | --- |
| `SEIKIKA_TELEMETRY` | `prod` sets production defaults, including structured stderr. |
| `OTEL_SERVICE_NAME` | A stable identifier for the deployed service, such as `seikika-demo`. |
| `OTEL_EXPORTER_OTLP_PROTOCOL` | `http/protobuf` over HTTPS for the demo gateway. The SDK also supports gRPC. |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | The base Collector URL. With HTTP, the SDK appends `/v1/traces`, `/v1/metrics`, and `/v1/logs`. |
| `OTEL_EXPORTER_OTLP_HEADERS` | Secret-backed OTLP headers, such as `authorization=Bearer%20<TOKEN>`. |
| `OTEL_TRACES_SAMPLER` | Choose a sampling policy for your traffic and retention budget. The demo samples every request at the source so a walkthrough can follow a specific request. |

After every rollout use a fresh client session and record the time, expected HTTP status, service name, matching log and span IDs, and a recent metric timestamp. Confirm a denied guest operation still fails. Check the Collector's internal telemetry separately from the dashboards it feeds. [OTLP exporter specification](https://opentelemetry.io/docs/specs/otel/protocol/exporter/) · [Collector internal telemetry](https://opentelemetry.io/docs/collector/internal-telemetry/)

## Diagnose an issue

1. Record the affected journey, environment, first observed UTC time, and latest deploy or configuration change. Check traffic, error rate, latency, and saturation against the usual baseline. Separate portal availability from instrumentation availability with an independent HTTP check.
2. Pick one affected request. Open its server span, confirm the path and status, and find the log with the same trace ID. Compare successful and failed requests in the same window. Look at Cloudflare, database, or connector telemetry when a span stops at the process boundary.
3. If users are affected, declare an incident. Assign an incident commander, an operations lead to coordinate changes, and a communications lead to track status and updates. Contact the portal owner first and bring in the owner of the affected dependency when the evidence points there. Keep a UTC timeline with changes, results, and handoffs. [Google SRE incident response](https://sre.google/workbook/incident-response/) · [Google SRE monitoring](https://sre.google/sre-book/monitoring-distributed-systems/)

| Symptom | Where to look | Mitigate and verify |
| --- | --- | --- |
| The portal returns 503 or reads fail | Compare external HTTP, private `/healthz` and `/readyz`, Worker deployment events, and portal startup logs. A healthy listener with a failing readiness check points toward dependencies or the first snapshot. | Check database connections, OpenFGA, and child portal refresh errors. If the failure began with a release and its rollback is compatible with the data schema, roll back the release. Verify readiness and the original read. |
| Requests slow down or start returning 5xx | Compare request rate, error counts and latency with a healthy period. Inspect a slow or failed span and its log. Check connection utilization and saturation independently. | Reduce concurrency or use the previous healthy configuration when the evidence supports it. Confirm customer-facing latency and errors return to baseline and the dependency has headroom. |
| The catalog is stale while pages load | Check snapshot lag, ingest counters, connector errors, and whether readiness includes the expected child portals. A counter reset can indicate a process restart. | Restore the failing connector, credential, or source, and perform the supported refresh. Verify the expected entity or score as well as a newer snapshot or advancing counters. |
| All dashboard data goes quiet | Check time window, service name, and a fresh HTTP request. Inspect the portal's exporter errors, Collector accepted and refused signals, queue depth, and backend health. | Trace the missing data hop by hop. For the demo, 401 indicates a rejected token, 404 can indicate an incorrect signal URL, and 415 indicates the wrong wire format. Restore the endpoint, credential, or backend, then verify all three signals with a fresh request. |
| Logs have no linked trace | Check for trace sampling, time range, matching resource names and whether both traces and logs reach the Collector. | Reproduce on a known sampled route. Confirm the same trace and span IDs exist in both exported records, then verify the Loki-to-Tempo link. |
| Suspicious or sensitive data appears in a public view | Record the time range and signal type without copying sensitive values into incident chat. Check the source and Collector allowlist. | Remove public access, rotate any exposed credentials, and purge affected data under your retention policy. Restore guest access after privacy and permission checks pass. |

[OpenTelemetry troubleshooting](https://opentelemetry.io/docs/collector/troubleshooting/) · [OpenTelemetry handling sensitive data](https://opentelemetry.io/docs/security/handling-sensitive-data/)

## Make recovery repeatable

- Set availability and latency objectives for real user journeys. Page on meaningful budget burn with a named owner and a mitigation path. Track freshness and collector drops as supporting signals. Keep notification delivery and external health checks independent of the system they monitor. [Google SRE alerting on SLOs](https://sre.google/workbook/alerting-on-slos/)
- Before a release, validate in a preview with representative traffic. Compare its error rate and duration against the stable build, record the last healthy deployment, and confirm whether schema changes permit a rollback. Keep private operator telemetry separate from intentionally public demo data. [Google SRE canarying releases](https://sre.google/workbook/canarying-releases/) · [AWS operational readiness reviews](https://docs.aws.amazon.com/wellarchitected/latest/framework/ops05-bp04.html)
- Keep known recovery procedures and diagnostic playbooks in version control. Review and rehearse them when the hosting topology, dependencies, or on-call ownership changes. For a production Collector, size its sending queue for expected outages and use persistent buffering where signal loss matters. Test your restore process. [AWS playbooks](https://docs.aws.amazon.com/wellarchitected/latest/framework/ops_ready_to_support_use_playbooks.html) · [OpenTelemetry Collector resiliency](https://opentelemetry.io/docs/collector/resiliency/)
- Close the incident after the original user journey, telemetry, and key dependencies recover. Write a blameless timeline with impact, contributing conditions, and actions with owners. Add a regression reproduction to the [OpenTelemetry testbed](https://github.com/seikika/seikika/tree/main/experiments/otel-testbed) for transport, export, or correlation failures. [Google SRE postmortem culture](https://sre.google/workbook/postmortem-culture/)
