# seikika-connector-grafana

A tool in the box that reads Prometheus instant queries through Grafana and links back to a dashboard. The portal passes `url`, optional `token` and `publicUrl`, `datasourceUid`, and a map of `targets` in `SEIKIKA_CONNECTOR_CONFIG`.

- `grafana-observation` reports one dashboard and set of metric results per configured service environment.
- Run the [service catalog integration kit](../../examples/service-catalog/README.md) to conformance-test this connector with local Grafana.

[Operator guide](https://seikika.com/connectors/grafana/)
