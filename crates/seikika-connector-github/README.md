# seikika-connector-github

Reads GitHub repositories, CODEOWNERS, apps, and installations. It can also collect recent
workflow run statistics for selected repositories.

This connector brings data from its source into a Seikika portal. Use the binary
with the matching Seikika release or build both from the same source checkout.

## Use the connector

Configure the organization and give the connector a token or GitHub App credentials with
access to the data you want to collect. Use the reference portal fixtures to inspect
record shapes.

- [Install Seikika and connectors](https://seikika.com/start/install/)
- [Set up and check a connector](https://seikika.com/authors/connectors/)
- [Connector source and tests](https://github.com/seikika/seikika/blob/25a208a3bc1a97f184588c512afcb12d636ce7be/crates/seikika-connector-github/src)
