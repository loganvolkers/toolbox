# Standards package directory research and gap analysis

*Status: Implemented in the public docs. Reviewed November 26, 2026.*

Seikika had nine first-party standards packages at the audited source revision. All nine had repository READMEs and test portals, but the public site had no standards package directory or individual package guides. The public npm directory lists JavaScript libraries. The Seikika standards package format adds checks to a portal manifest and uses a Git pin and lockfile.

## Method

Follow the same reader task in five nearby ecosystems. Find a package, determine its publisher and applicable version, copy its configuration, identify the inputs and results, and find a check that confirms it works. Review the directory and one representative detail page or linked source. These are documentation scores. They describe the sampled journey and do not measure package safety, popularity, or runtime behavior.

| Criterion | 1 point | 2 points |
|---|---|---|
| Discovery | A browsable repository index or scattered links | A dedicated searchable or browsable directory with purpose descriptions |
| Provenance | Publisher or source is visible | Publisher and a specific version, release, or pin are visible |
| Setup | Partial instructions or a configuration fragment | Prerequisites and configuration a reader can adapt |
| Inputs and results | Inputs or results are described | Both are identified with names, defaults, or examples |
| Verification | A sample result or a check command | A check with an expected result or recovery advice |

Zero indicates the sampled journey does not meet the one-point criteria.

| Sampled path | Discovery | Provenance | Setup | Inputs and results | Verification |
|---|---:|---:|---:|---:|---:|
| Backstage directory to Backstage installation docs | 2 | 1 | 2 | 1 | 1 |
| GitHub Marketplace to Upload a Build Artifact | 2 | 2 | 2 | 2 | 2 |
| Artifact Hub to its Helm chart | 2 | 2 | 2 | 2 | 1 |
| OpenTofu Registry to the AWS VPC module | 2 | 2 | 2 | 2 | 1 |
| Gatekeeper Library to Required Labels on Artifact Hub | 2 | 2 | 2 | 2 | 1 |
| Seikika before, public author docs to the package repository | 1 | 2 | 2 | 2 | 1 |
| Seikika after, public directory to operator guide | 2 | 2 | 2 | 2 | 2 |

- **Backstage** lists publishers and separates core features from active community plugins. Search and category filters help with a large ecosystem. General installation docs cover feature discovery and manual registration. State that this directory contains Seikika-maintained standards. [Plugin directory](https://backstage.io/plugins/) · [Installing plugins](https://backstage.io/docs/frontend-system/building-apps/installing-plugins/)
- **GitHub Marketplace** includes a version selector and links the listing to its publisher and source. Upload a Build Artifact shows workflow examples, inputs, outputs, and an example of using the artifact ID. GitHub's security guide explains why a commit SHA makes a useful pin. Give each guide a pinned include and a named result to check. [Marketplace](https://github.com/marketplace?type=actions) · [Upload a Build Artifact](https://github.com/marketplace/actions/upload-a-build-artifact) · [GitHub pinning guidance](https://docs.github.com/en/actions/reference/security/secure-use)
- **Artifact Hub** extracts Helm metadata from chart files and adds a versioned installation panel. Repository ownership and the official or verified publisher labels are separate signals. Generate listings from the manifests Seikika actually ships and link to their source. [Artifact Hub](https://artifacthub.io/) · [Artifact Hub chart](https://artifacthub.io/packages/helm/artifact-hub/artifact-hub) · [Chart metadata](https://artifacthub.io/docs/topics/repositories/helm-charts/) · [Repository ownership labels](https://artifacthub.io/docs/topics/repositories/)
- **OpenTofu Registry** separates an example's usage from required inputs, optional defaults, outputs, dependencies, and resources. Seikika's guides surface the settings needed to start and link to the package reference for the full list. [Registry search](https://search.opentofu.org/) · [AWS VPC example and inputs](https://search.opentofu.org/module/terraform-aws-modules/vpc/aws/v3.7.0/example/issues/inputs) · [Module resources and reference tabs](https://search.opentofu.org/module/terraform-aws-modules/vpc/aws/v3.2.0/resources)
- **Gatekeeper** gives a policy a versioned listing with a constraint template, parameters, and examples of what violates or satisfies the policy. Seikika pairs its operator guides with fixture portals that exercise passing and failing rules. [Gatekeeper policy library](https://github.com/open-policy-agent/gatekeeper-library) · [Required Labels listing](https://artifacthub.io/packages/gatekeeper/gatekeeper-policies/k8srequiredlabels)

## Gaps and fixes

| Gap | Implemented fix |
|---|---|
| An operator had to find the repository or the authoring docs first. | Put the standards package directory on the homepage and in the sidebar. Link it from the operator, author, and npm pages. |
| Package names and descriptions were maintained in a repository table. | Match published landing pages to package manifests and generate the list at build time. Show the guide title and purpose on each card. |
| A reader had to assemble prerequisites, a Git pin, and expected results from multiple places. | Give each of the nine packages a guide with version guidance, a pinned include, required data or tools, a verification step, and recovery advice. |
| There was no check that a shipped standards package had a public guide. | Add a bronze documentation rule to the maintainer portal. Drive it from package manifests so a missing guide produces a visible failure. |

At this catalog size, the public site's existing search and the directory cover discovery. Evaluate faceted search or publisher trust signals when the catalog grows or accepts packages from other publishers.

## Sources

- [Public directory and adoption workflow](../../site/src/content/docs/packages/index.mdx)
- [Standards package scorecard](../../portal/standards/standard-packages.yaml)
- [Documentation style guide](../../site/src/content/docs/maintainers/writing.md)
- [Decision](../decisions/D01M8CRS50G5086P5DCFQN636BA.md)
