---
title: Grafana
description: Read a few Prometheus metrics and link to a Grafana dashboard from a service page.
---

Use `builtin:grafana` with runtime `in-image`. Install `seikika-connector-grafana` from the same source revision as the CLI.

Set `url`, the Prometheus `datasourceUid`, and a secret-backed `token` if required. Give the token read access to the configured dashboards and query access to the data source. Set `publicUrl` when readers reach Grafana through a different URL than the connector.

Each `targets` entry is keyed by `<service>@<environment>` and declares the OpenTelemetry `serviceName`, `dashboardUid`, and one to twelve metrics. Give each metric an `id`, `label`, `unit`, and instant PromQL `expr`. Use `{{service}}` as an unquoted placeholder for the service label value and optionally use `{{environment}}` for the environment label value. Aggregate each query to one scalar. The connector emits `grafana-observation:<service>@<environment>` with the dashboard link and the individual query outcomes. A missing series shows no data. A value of zero stays numeric.

The default service package maps these observations onto declared environments. The local service catalog kit tests this connector with Grafana and Prometheus in a container. [Package and setup example](https://github.com/seikika/seikika/tree/main/packages/service-catalog) · [Local test kit](https://github.com/seikika/seikika/tree/main/examples/service-catalog) · [Grafana data source API](https://grafana.com/docs/grafana/latest/developer-resources/api-reference/http-api/api-legacy/data_source/)
