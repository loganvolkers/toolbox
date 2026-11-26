# seikika-connector-gitlab

Reads projects and group membership from GitLab.com or a self-managed GitLab instance.

This connector brings data from its source into a Seikika portal. Use the binary
with the matching Seikika release or build both from the same source checkout.

## Use the connector

Configure a group and a token with `read_api`. Declare both `group` and `project` record
kinds. Group membership records cover direct members who meet the configured access level.

- [Install Seikika and connectors](https://seikika.com/start/install/)
- [Build and check the GitLab example](https://github.com/seikika/seikika/blob/25a208a3bc1a97f184588c512afcb12d636ce7be/docs/design/forges.md#6-testing-against-a-real-gitlab)
- [Connector source and tests](https://github.com/seikika/seikika/blob/25a208a3bc1a97f184588c512afcb12d636ce7be/crates/seikika-connector-gitlab/src)
