---
id: D01M8FXW9XRMTM2EPQZEZ49HH9F
decided: 2026-11-28T01:57:05Z
---

# A Service is a declared identity with separate environment evidence

A Git declaration establishes a `Service` and its accountable owner. Each `ServiceEnvironment` belongs to one service and names its runtime and telemetry identities explicitly. Cloudflare and Grafana may enrich those declarations with bounded observations. Connector availability and rollouts cannot redefine the service's identity.

This keeps source, runtime, and OpenTelemetry mappings inspectable, allows several environments per service, and preserves the observability playbook's distinction between rollout evidence and user impact. The overview may link to operational tools while leaving histories and investigation workflows with their sources.

[Service catalog design and research](../design/service-catalog.md)
