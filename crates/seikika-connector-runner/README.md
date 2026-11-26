# seikika-connector-runner

Runs configured local tools and feeds their results into Seikika. It can collect test
output, lint findings, or records printed as newline-delimited JSON.

This connector brings data from its source into a Seikika portal. Use the binary
with the matching Seikika release or build both from the same source checkout.

## Use the connector

Run it with commands and a checkout you trust. Jobs execute with the connector process’s
permissions. Use each finding’s recorded command to reproduce it with the original tool.

- [Install Seikika and connectors](https://seikika.com/start/install/)
- [Set up and check a connector](https://seikika.com/authors/connectors/)
- [Connector source and tests](https://github.com/seikika/seikika/blob/25a208a3bc1a97f184588c512afcb12d636ce7be/crates/seikika-connector-runner/src)
