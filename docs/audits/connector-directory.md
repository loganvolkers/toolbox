# Connector directory research and gap analysis

*Status: Implemented in the public docs. Reviewed November 26, 2026.*

Seikika shipped six first-party connectors. Its public docs explained how to install a binary and write a custom connector. Operators could find examples and two detailed references in the repository, but there was no directory or public setup page for any of the six. The connector scorecard checked the presence of crate docs. It did not check whether an operator could reach setup instructions from the public site.

## Method

Compare public directories and their GitHub entries using the same operator task. Find the GitHub integration, identify who maintains it and where it runs, choose credentials and configuration, determine what it collects, then find a way to verify or recover. The sample includes two developer portals, two data integration products, and an observability platform. The scores describe what a reader can find on the linked pages. They do not measure connector quality or confirm live authentication across products.

| Criterion | 1 point | 2 points |
|---|---|---|
| Discovery | A provider link or a list to scan | A dedicated directory with categories, search, or filters |
| Availability | One of install target, compatibility, or maintenance information | At least two of those with a clear provider identity |
| Setup | Settings or a partial example | Procedure with prerequisites and usable configuration |
| Data coverage | Description of supported data | Named streams, resource kinds, tables, or sample records |
| Verification | A success check or guidance for a likely connector failure | Both a success check and guidance for a likely connector failure |

A zero means the linked path does not satisfy the 1-point description. For Seikika, the starting path includes public pages and their linked repository references. The revised path uses the directory and the GitHub guide.

## What the baselines show

| Product and GitHub path | Discovery | Availability | Setup | Data | Verify | Total |
|---|---:|---:|---:|---:|---:|---:|
| Airbyte | 2 | 2 | 2 | 2 | 2 | 10 |
| Fivetran | 2 | 2 | 2 | 2 | 1 | 9 |
| Port | 2 | 2 | 2 | 2 | 2 | 10 |
| Backstage | 2 | 1 | 2 | 1 | 2 | 8 |
| Grafana | 2 | 2 | 2 | 2 | 2 | 10 |
| Seikika before | 0 | 1 | 1 | 1 | 0 | 3 |
| Seikika after | 2 | 2 | 2 | 2 | 2 | 10 |

- **Airbyte** groups sources and destinations and shows support levels. The GitHub page names sync modes and streams and keeps setup, troubleshooting, and changelog information together. Adopt the visible maintainer, explicit record names, and checks next to setup. [Connector catalog](https://docs.airbyte.com/integrations) · [GitHub source](https://docs.airbyte.com/integrations/sources/github) · [Connector support levels](https://docs.airbyte.com/integrations/connector-support-levels)
- **Fivetran** lists SaaS and hybrid deployment on the capabilities page, alongside the destination schema. It moves credential choices and a connection test into a separate setup checklist. Keep the setup guide short and link to deeper references where a connector has many settings. [Connector directory](https://fivetran.com/docs/connectors) · [GitHub connector](https://fivetran.com/docs/connectors/applications/github) · [GitHub setup](https://fivetran.com/docs/connectors/applications/github/setup-guide)
- **Port** groups integrations by upstream system. GitHub ingestion shows hosted and self-hosted setup, record mappings, and a link to monitor sync status. Troubleshooting covers GitHub-specific errors. Make run location visible and give operators one known record to check after setup. [Available integrations](https://docs.port.io/context-lake/ingestion/ingest-data-into-port/native-integrations/available-integrations/) · [GitHub Ocean integration](https://docs.port.io/context-lake/ingestion/ingest-data-into-port/native-integrations/available-integrations/git/github-ocean/) · [GitHub troubleshooting](https://docs.port.io/troubleshooting/github/)
- **Backstage** has a provider index and breaks GitHub into tasks such as locations and discovery. Discovery explains how to check and retry a trial webhook delivery. Its separate plugin directory helps readers distinguish integration setup from extensions. Keep one operator landing page per shipped connector and link to additional tasks where useful. [Integrations](https://backstage.io/docs/integrations/) · [GitHub locations](https://backstage.io/docs/integrations/github/locations/) · [GitHub discovery](https://backstage.io/docs/integrations/github/discovery/) · [Plugin directory](https://backstage.io/plugins/)
- **Grafana** features GitHub and identifies the publisher, version, and minimum Grafana version. The linked docs describe how to test credentials, list query types and response fields, and resolve common failures. Give GitHub a direct home page link and state who maintains the connectors and which Seikika version the guides cover. [Plugin catalog](https://grafana.com/grafana/plugins/) · [GitHub listing](https://grafana.com/grafana/plugins/grafana-github-datasource/) · [Setup](https://grafana.com/docs/plugins/grafana-github-datasource/latest/configure/) · [Query types](https://grafana.com/docs/plugins/grafana-github-datasource/latest/query-editor/) · [Troubleshooting](https://grafana.com/docs/plugins/grafana-github-datasource/latest/troubleshooting/)

## Gaps and fixes

| Gap in Seikika | Fix |
|---|---|
| Connecting a data source was hard to find from the home page. | Add direct links to GitHub and the directory near the top of the home page. Put Connectors next to Start in the sidebar. Link from installation, the operator index, and the manifest reference. |
| Operators could not compare providers, run locations, or record coverage. | Generate the directory from public connector guides. Show each guide's title and description, and explain the source version and the two ways connectors run. |
| Setup, credentials, and recovery were spread across examples, crate docs, and references. | Publish one operator guide for each of GitHub, GitLab, Git files, Workspace, Runner, and SCIM. Each guide gives a manifest example, sample evidence of success, and recovery steps. Link further references for detailed configuration. |
| The connector scorecard could pass a connector with no public setup page. | Add the bronze `public-guide` rule. Require a published landing page linked from the connector directory and sidebar. Use the site scorecard and the documentation style guide to review the page itself. |

With six entries, the site's existing search and the directory cover discovery. A searchable provider filter or separate publisher tiers become candidates when the number or ownership of entries changes.

## Sources

- [Public directory source](../../npm/site/src/content/docs/connectors/index.mdx)
- [Connector scorecard design](../design/connector-standards.md)
- [Documentation style guide](../../npm/site/src/content/docs/maintainers/writing.md)
- [Decision](../decisions/D01M8CHMD6TH2AXRWT9E49G4V6V.md)
