# seikika-connector-cloudflare

A tool in the box that reads Cloudflare Workers deployment history. The portal passes `accountId`, `token`, optional `apiUrl`, and a map of `targets` in `SEIKIKA_CONNECTOR_CONFIG`.

- `worker-deployments` reports one bounded rollout history per configured service environment.
- Run the [service catalog integration kit](../../examples/service-catalog/README.md) to conformance-test this connector with a local emulator.

[Operator guide](https://seikika.com/connectors/cloudflare/)
