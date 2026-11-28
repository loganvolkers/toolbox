# Federated inventory and report publications

*Status: implemented for the report read model, 2026-11-28 (D01M8FRFDEKE0F5XHG7ZSNEJ9S8). This is about the box: carrying, identifying, composing, and reporting snapshots. The maintainer SDLC validates the checkout and the operator SDLC stages a reviewed artifact.*

The federation demo composes the Seikika for Seikika report from the same checkout that builds the parent. The parent keeps the independent organization inventory and standards. Each source keeps its definitions, evidence, and local record IDs. The operator can link observations of the same resource using explicit catalog aliases.

## Industry research

| System | Observed practice | Consequence for Seikika |
| --- | --- | --- |
| Google Cloud Asset Inventory | Lists, searches, exports, and versions inventory at project, folder, and organization scope. An asset has a full resource name, type, and ancestors. Relationships have types and can have a separate update time. | Qualify an observation by source and keep the provider identity and acquisition revision separately. Carry explicit relationships and remap their endpoints through approved catalog aliases. [Overview](https://docs.cloud.google.com/asset-inventory/docs/asset-inventory-overview) · [Asset schema](https://docs.cloud.google.com/asset-inventory/docs/reference/rest/v1/Asset) |
| Wiz Security Graph and Service Catalog | API connectors inventory multiple clouds. The graph links workloads, identities, data, code, and other resources. Service Catalog uses ownership to navigate to related security context. Wiz's public material describes this behavior and leaves identity arbitration internals unspecified. | Preserve source observations and allow typed links across records. Component ownership and provider topology can coexist. The published report can deliver these views from existing records and relations. [Wiz Cloud](https://www.wiz.io/platform/wiz-cloud) · [Service Catalog](https://www.wiz.io/blog/wiz-service-catalog) |
| Backstage Software Catalog | Entity references use kind, namespace, and name. Providers own their contributions. Catalog stitching assembles a read model and catalog annotations record the location and origin. | Stable source keys can distinguish two catalogs with identical names. Separate importing a publication from evaluating parent standards. Keep provenance even after the UI composes a canonical row. [Entity references](https://backstage.io/docs/features/software-catalog/references/) · [Entity life cycle](https://backstage.io/docs/features/software-catalog/life-of-an-entity/) · [Origin annotations](https://backstage.io/docs/features/software-catalog/well-known-annotations/) |
| AWS Config and Azure Resource Graph | AWS Config uses scoped, authorized source account and region replication. Azure Resource Graph applies resource read permissions to results within the query scope. | Make publication access explicit at ingress and filter the parent inventory before rendering or reconciling records for an audience. [AWS Config authorization](https://docs.aws.amazon.com/config/latest/developerguide/aggregated-add-authorization.html) · [Azure Resource Graph](https://learn.microsoft.com/en-us/azure/governance/resource-graph/overview) |
| GitHub repositories and W3C PROV | GitHub returns both repository `id` and `full_name`. Repository names can change. PROV models derivation and attribution as separately traversable relations. | Capture the provider ID alongside the slug used in report links. Record which artifact and source record contributed to a canonical entity. [GitHub repositories](https://docs.github.com/en/rest/repos/repos) · [GitHub renames](https://docs.github.com/en/repositories/creating-and-managing-repositories/renaming-a-repository) · [PROV-O](https://www.w3.org/TR/prov-o/) |

## Gap assessment

| Before this change | Implementation |
| --- | --- |
| GitHub inventory finds `.seikika/report.json` on the default branch. This checkout builds its public report from `portal/`. | `federation.reportSources` declares parent-staged files. Pages reuses its generated self report. The hosted example bundles a file in its build context. |
| Identical source-local IDs were isolated, including multiple observations of the same repository. The plan assumed globally unique IDs. | `federation.catalogAliases` declares exact source paths, kinds, and refs. The organization standards score the canonical GitHub slug, URL, and provider ID. |
| The comparison summary treated all factors as one set of participants. | Comparison groups preserve the source, participants, and verdict rule. Detail links stay within their group. |
| A child repository observation could lose its relationship to its parent repository or its rollup explanation. | Entity references, scores, assessment targets, and deep links resolve to the canonical row. Source-specific facts and rollup explanations remain available. |
| The browser test covered synthetic Git children, and the organization example required a GitHub token for live inventory. | The parity command uses the checked-in organization fixture and an actual artifact from the checkout. The same input can be replayed as an offline report or a loopback server. |

## Composition and access

- Report sources use keys chosen by the parent. An explicit staged artifact takes precedence over a Git observation with the same key for its approved audience. Staging records the SHA-256 digest of the file. Git publications continue to identify the scanned commit.
- The parent loads staged files once per invocation or server start. A path stays inside the manifest directory, uses regular files, and has a 16 MiB limit per artifact. A missing or invalid artifact appears as unavailable beside healthy sources. Git child publications keep their existing limits.
- `public: true` is an operator decision to publish the artifact to guests and offline web readers. Review all data in the file and nested publications. Artifacts without `public: true` appear to authenticated operators. Staged artifacts carry report data and have their action definitions removed during import. The git action federation path remains responsible for action authorization.
- Without an alias, two records with the same name and kind remain independently namespaced. Aliases target already visible parent entities of the same kind. Each source can contribute one observation to a target. Multiple sources can point to that target. The visible catalog row has the parent ID. Child claims remain separately attributed and cannot change parent evaluation or fields.
- The alias is an operator decision about an observation, independent of the GitHub provider ID. A future inventory reconciler can propose aliases based on verified provider IDs, track renames and deletions, and project authorized edges into persisted APIs. Dynamic cross-portal queries, reconciliation suggestions, and subtree permissions remain in the [federation roadmap](federation.md) and [identity resolution design](identity-resolution.md).

See [the local example](../../examples/github-org-federation/README.md) for the fixture and exact parity check.

## Sources

- [Google Cloud Asset Inventory API](https://docs.cloud.google.com/asset-inventory/docs/reference/rest/v1/Asset)
- [Wiz Service Catalog and Security Graph](https://www.wiz.io/blog/wiz-service-catalog)
- [Backstage entity processing](https://backstage.io/docs/features/software-catalog/life-of-an-entity/)
- [AWS Config aggregation](https://docs.aws.amazon.com/config/latest/developerguide/aggregated-add-authorization.html) and [Azure Resource Graph](https://learn.microsoft.com/en-us/azure/governance/resource-graph/overview)
- [GitHub repository API](https://docs.github.com/en/rest/repos/repos) and the [W3C PROV ontology](https://www.w3.org/TR/prov-o/)
