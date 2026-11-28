# Service catalog

*Status: implemented in the 0.2 source. Operator and package author SDLC.*

The service catalog is part of the box. It connects versioned inventory to evidence from the source forge, hosting provider, and telemetry backend. Each tool keeps its own credentials, data, and workflows.

## Default model

| Entity | Identity and authority | What it answers |
| --- | --- | --- |
| `Service` | `service:<slug>`, declared in Git | What runs, who is accountable, what lifecycle applies, and where are its code and runbook? |
| `ServiceEnvironment` | `service-environment:<slug>@<environment>`, declared in Git and related to its Service | Where does it run, what was deployed, and where can I troubleshoot it? |
| Repository | Forge connector's repository identity, mapped explicitly from the service descriptor | What source repository implements it? |

Keep the service slug stable across repository moves, runtime renames, and rollouts. The source project slug and its optional catalog repository ID are separate. The repository link selects a visible catalog entity when available and the Source link opens the forge directly. An owning team is accountable for the service. In this package the owner is catalog metadata. See the authorization model before using ownership to grant permissions.

Each environment has a single declared parent. Declare the Worker script, the actual OpenTelemetry `service.name`, and the Grafana dashboard UID independently. Providers add observations keyed by the configured service/environment pair. They cannot create catalog identities. CEL checks the observed Worker and Grafana identifiers against the declaration before it publishes matched observations. For a shared metrics backend, also scope PromQL by `service.namespace` or `deployment.environment.name` when the service name alone is ambiguous.

Cloudflare contributes the latest five Worker deployments, including their UTC times and complete version traffic splits. This supports rollout diagnosis and comparison to the original user journey. Grafana contributes a few instant answers from explicitly scoped PromQL, with collection time, a dashboard link, and separate states for zero, no data, and a query error. Provider failures leave the previous snapshot to age. The service page marks stale deployment evidence and suppresses stale metric values after three refresh intervals. A recent deploy or ingest query is evidence for triage. Assess customer impact with representative HTTP checks and the availability, error, latency, and saturation signals in the operator playbook. The SRE kit still owns readiness, error budgets, paging, and incident context.

Publish service data to guests only when the inventory and every contributing connector are explicitly eligible through `publicReportSources`. Both connectors bound their record shapes. Cloudflare deployment messages and actors, Grafana query errors, and credentials stay out of the emitted records. The demo points Grafana at the existing Collector-filtered public dataset. Adopt private connector instances and private portal visibility for real workloads.

## Portal research

Reviewed first-party descriptors and integration mapping docs against five questions: what establishes identity, where ownership is declared, how a service joins to code, how it joins to a running deployment, and how responders reach operational tools.

| Portal | Practice in the source | Seikika design judgment |
| --- | --- | --- |
| Backstage | A deployable unit is generally a Component with `type: service`, required owner and lifecycle, and explicit relations. Metadata supports named links and integration annotations. | Require owner and lifecycle in the Git declaration. Use catalog relations for running environments and labeled links for responders. Give Service its own Kind so operators can adopt it without a larger Component taxonomy. |
| Port | The default service blueprint is team-owned. Deployment is a separate blueprint related to its service. Integration mappings can select a target by an explicit ID or property. | Model each environment independently so the stable service identity survives continuous deploys. Keep the recent rollout history bounded within its environment until individual deployments need catalog-wide queries or actions. |
| OpsLevel | Services are a component type. A version-controlled descriptor can set ownership and lifecycle by stable aliases and connect repositories, dependencies, and categorized tools such as metrics, deployment, and runbooks. | Keep source and runtime identities explicit and versioned. Show the shared service metadata before the cross-tool links. |
| Cortex | The descriptor uses a stable `x-cortex-tag` and supports owner and repository metadata. The CLI reads a descriptor by tag and can add an owning GitHub group. | Keep the catalog slug stable and make each integration's external identifier visible. A repository may contain several deployable services. |

## Sources

- [Backstage catalog descriptor](https://backstage.io/docs/features/software-catalog/descriptor-format/) and [catalog relations](https://backstage.io/docs/features/software-catalog/well-known-relations/).
- [Port default blueprints](https://docs.port.io/context-lake/data-model/setup-blueprint/default-blueprints/) and [integration mapping](https://docs.port.io/context-lake/ingestion/configure-mapping/overview/).
- [OpsLevel config in code](https://docs.opslevel.com/docs/opslevel-yml) and [repository connections](https://docs.opslevel.com/docs/service-connections).
- [Cortex CLI descriptor examples](https://github.com/cortexapps/cli) and [Cortex catalog entity provider](https://registry.terraform.io/providers/cortexapps/cortex/latest/docs/data-sources/catalog_entities).
- [OpenTelemetry service](https://opentelemetry.io/docs/specs/semconv/resource/service/) and [deployment](https://opentelemetry.io/docs/specs/semconv/registry/entities/deployment/) conventions.
- [Cloudflare Worker Deployments API](https://developers.cloudflare.com/api/resources/workers/subresources/scripts/subresources/deployments/methods/list/) and [Grafana data source API](https://grafana.com/docs/grafana/latest/developer-resources/api-reference/http-api/api-legacy/data_source/).
- [Observability playbook](../../site/src/content/docs/operators/observability.md), [SRE readiness kit](../../packages/sre/README.md), and [Google SRE monitoring guidance](https://sre.google/sre-book/monitoring-distributed-systems/).
