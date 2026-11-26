---
id: D01M8CHMD6TH2AXRWT9E49G4V6V
decided: 2026-11-26
---

# Require public guides for first-party connectors

Every first-party connector needs an operator landing page under `/connectors/<name>/`. The connector directory and sidebar must reach that page. The guide can link to more pages for configuration, permissions, or other tasks. It should tell an operator what the connector reads, what it needs, how to configure it, and how to verify the result.

`maintainer-connector` checks this at bronze. The workspace collector reports the guide route when it finds titled and described pages for the directory and guide, and both appear in the sidebar. The directory builds its entries from the published landing pages. Draft or hidden pages do not count. The site scorecard checks the docs themselves, and the [documentation style guide](../../site/src/content/docs/maintainers/writing.md) defines the editorial review. Adding the rule to the existing connector scorecard gives each connector an individual result and applies the existing bronze requirement to new connectors.

**Status** Implemented.

## Sources

- [Connector directory research](../audits/connector-directory.md)
- [Connector scorecard design](../design/connector-standards.md)
- [Connector scorecard](../../portal/standards/connectors.yaml)
