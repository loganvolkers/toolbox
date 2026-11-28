# Internal capability documentation audit

The public docs covered the first repo score and the live catalog. They left major parts of action federation, action authoring, and static report hosting in repository examples or design notes. This revision adds task guides for those workflows and documents where the current implementation stops. It also covers campaigns, waivers, assessments, and targeted refresh.

The main audience is portal operators connecting repo portals to a central portal. The revised docs also serve users requesting changes, package authors supplying action definitions, and maintainers who publish these docs.

## Method

Review date is November 26, 2026. The baseline is [`25a208a`](https://github.com/seikika/seikika/tree/25a208a3bc1a97f184588c512afcb12d636ce7be), after the first [public docs audit](./public-docs-2026-11-26.md) and onboarding reconciliation. That audit compared the docs to other projects. This one compares the reader journeys to Seikika's own implementation.

I mapped the site navigation and pages to the [implementation milestones and interface contracts](../v1-spec.md), current CLI commands and manifest schema, the decisions behind each capability, and the repository's worked examples and acceptance checks. I treated the implementation and passing tests as evidence of availability. I kept unimplemented design proposals out of the coverage score.

Coverage per journey is **0** for no discoverable route, **1** for a mention or a design link without usable steps, **2** for practical but incomplete guidance, and **3** for a findable guide with a runnable or validated path, a result to check, and relevant limits or recovery. The rows are equally weighted. This is a coverage score for the sampled workflows, separate from the first audit's comparative editorial score.

## Coverage and fixes

| Reader journey | Before | Revised | Evidence and fix |
|---|---:|---:|---|
| Score one checkout and gate regressions | 3 | 3 | The quickstart and CI guide already show a baseline, a failed check, and a fix. [Quickstart](../../npm/site/src/content/docs/start/quickstart.md), [CI guide](../../npm/site/src/content/docs/operators/ci.md) |
| Serve a catalog and read it through CLI, HTTP, or MCP | 3 | 3 | The fixture tutorial and interface guides show catalog reads. [Local portal](../../npm/site/src/content/docs/start/portal.md), [HTTP](../../npm/site/src/content/docs/users/api.md), [MCP](../../npm/site/src/content/docs/agents/reads.md) |
| Deploy a shared portal and configure permissions | 3 | 3 | The deployment and access guides cover service checks and authentication. Added paths for parent portals and static snapshots. [Deployment guide](../../npm/site/src/content/docs/operators/deploy.md), [access guide](../../npm/site/src/content/docs/operators/access.md) |
| Author packages and connectors | 3 | 3 | Existing guides cover pinning, tests, and connector conformance. [Package guide](../../npm/site/src/content/docs/authors/index.md), [connector guide](../../npm/site/src/content/docs/authors/connectors.md) |
| Discover and run actions from child repositories | 1 | 3 | The manifest table mentioned federation, and the example README held the walkthrough. The new guide previews, confirms, approves, and verifies a disposable Git change. It also covers trust, runner policy, and failure recovery for shared parents. [Federated actions example](../../examples/federated-actions/README.md), [federation guide](../../npm/site/src/content/docs/operators/federate-actions.md) |
| Define and test a self-service action | 1 | 3 | The existing user guide linked to the example but did not teach the manifest fields. The operator guide now explains input validation, approval groups, preview effects, executor options, and tests for allowed and refused runs. [Action schema](../../crates/seikika-model/schema/manifest.schema.json), [action authoring guide](../../npm/site/src/content/docs/operators/actions.md) |
| Request, approve, and investigate a run | 2 | 3 | Preview and verification were documented. The revision explains who can approve a run, supported interfaces, and how to respond to waiting, failed, and unverified states. [Action guide](../../npm/site/src/content/docs/users/actions.md), [HTTP reference](../../npm/site/src/content/docs/users/api.md#action-runs) |
| Generate a report that works offline | 1 | 3 | The demo mentioned an offline file. The operator guide now shows a direct fixture export and clarifies what readers can see in the embedded data. [Report implementation](../../crates/seikika-cli/src/report.rs), [HTML export](../../npm/site/src/content/docs/operators/publish.md#generate-an-html-report) |
| Host a report or embed it in a static site | 2 | 3 | The demo had an Astro fragment. The guide adds a complete Pages workflow, an Astro build and preview, and checks for site base and stale deploys. [Project Pages workflow](../../.github/workflows/pages.yml), [report integration](../../npm/report-ui/README.md), [hosting guide](../../npm/site/src/content/docs/operators/publish.md) |
| Track standards rollout and review an exception | 1 | 3 | Campaigns appeared in the reference and demo, and waiver behavior lived in a design note. The new guide adds a verified campaign and explains scoped baseline acceptance, campaign starts, and the decision a waiver needs. [Campaign design](../design/campaigns.md), [waiver design](../design/waivers.md), [standards guide](../../npm/site/src/content/docs/operators/standards.md) |
| Record and check a judgment about an entity | 1 | 3 | The generated CLI documented flags. The guide now takes readers from a rubric and subject hash to an evidence-backed assessment and a freshness check. [Assessment acceptance tests](../../crates/seikika-cli/tests/assess.rs), [assessment guide](../../npm/site/src/content/docs/operators/assessments.md) |
| Refresh the facts for one entity | 1 | 3 | Readers could look up the refresh command. The new tutorial shows a connector updating a field and identifies its source and snapshot. [Refresh handler](../../crates/seikika-api/src/connectors.rs), [refresh guide](../../npm/site/src/content/docs/users/refresh.md) |
| **Coverage points** | **22/36** | **36/36** | The edited navigation links these guides from the operator, user, and troubleshooting entry points. |

## Product boundaries

| Capability | Current path | Proposal or limit |
|---|---|---|
| Federation | A parent discovers child actions, handles approvals, dispatches, and checks the result. A child pins the trusted parent key. | Catalog and score aggregation across children and a static index across separate reports. [Federation design](../design/federation.md), [static proposals](../design/repo-portals.md#5-proposed-a-static-site-and-static-federation) |
| Actions | The CLI, HTTP API, and MCP cover the action operations documented for each surface. The built-in web interface lists actions. | Dedicated action and approval pages, shared run ownership across server instances, and durable executor recovery. [Current UI](../../npm/report-ui/src/ReportActions.vue), [action design](../design/federated-actions.md#72-deferred-with-the-reason) |
| Static generation and hosting | The CLI exports one HTML page and a JSON snapshot. Astro can build a report route into its static output, and a static host can serve that output. | A native multi-page generator and static federation remain proposals. Hosting uses a service you configure, such as GitHub Pages. [Report integration](../../npm/report-ui/README.md), [static proposals](../design/repo-portals.md#5-proposed-a-static-site-and-static-federation) |

The [capability map](../../npm/site/src/content/docs/concepts/capabilities.md) publishes these distinctions. When a proposed capability ships, give it a reader task, a validated example and a version before promoting it on that page.

## Verification

- Ran the action federation tutorial against local bare Git. The parent found the action, previewed the diff, received exit status 3 while waiting for approval, and verified Bob's approval and the committed file. The full M5 actions acceptance suite also passed.
- Generated the standalone report and the web files. Checked the report in a browser and built an independent Astro project with the published integration. Linted the GitHub Pages recipe and exercised its report and staging commands locally. The repository's own Pages workflow is the deployment reference.
- Ran the campaign and reviewed the recorded start. Checked an assessment brief and the check and submit paths. Ran the refresh exercise and checked that the workspace connector's new snapshot reported the broken link.
- Reviewed the new pages against the [writing standards](../../npm/site/src/content/docs/maintainers/writing.md). Checked prerequisites, commands, visible results, troubleshooting, scope, headings, and links. The repository research log records the command and environment details. [Research log](../research-log.md)
