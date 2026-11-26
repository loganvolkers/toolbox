# seikika-connector-git-files

Reads catalog files from local checkouts or through the GitHub contents API.

This connector brings data from its source into a Seikika portal. Use the binary
with the matching Seikika release or build both from the same source checkout.

## Use the connector

Configure the organization and file paths. For local reads, also set the checkout root.
With `splitKinds` enabled, one multi-document Backstage file can produce several catalog
records.

- [Install Seikika and connectors](https://seikika.com/start/install/)
- [Set up and check a connector](https://seikika.com/authors/connectors/)
- [Connector source and tests](https://github.com/seikika/seikika/blob/25a208a3bc1a97f184588c512afcb12d636ce7be/crates/seikika-connector-git-files/src)
