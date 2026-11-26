# seikika-connector-scim

Receives user and group changes from an identity provider over System for Cross-domain
Identity Management (SCIM) 2.0 and sends them to Seikika.

This connector brings data from its source into a Seikika portal. Use the binary
with the matching Seikika release or build both from the same source checkout.

## Use the connector

Run this connector separately from the portal. It needs a token for incoming
identity-provider requests and a separate connector token for Seikika. It pushes updates
as they arrive.

- [Install Seikika and connectors](https://seikika.com/start/install/)
- [Inspect sample identity records](https://github.com/seikika/seikika/blob/25a208a3bc1a97f184588c512afcb12d636ce7be/examples/reference-portal/fixtures/idp.ndjson)
- [Connector source and tests](https://github.com/seikika/seikika/blob/25a208a3bc1a97f184588c512afcb12d636ce7be/crates/seikika-connector-scim/src)
