---
title: Cloudflare Workers
description: Read recent Workers deployments and version traffic splits into a service catalog.
---

Use `builtin:cloudflare` with runtime `in-image` to read recent deploys for an explicit list of Workers. Install `seikika-connector-cloudflare` from the same source revision as the CLI.

Set `accountId`, a secret-backed `token` with **Metadata Read-Only** access to the selected Workers, and `targets` mapping each `<service>@<environment>` to a Worker script name. The optional `apiUrl` overrides the API base for an emulator. The connector emits one `worker-deployments` record per configured target with the five most recent deployment IDs, timestamps, version IDs and percentages. Scoped refresh accepts IDs such as `worker-deployments:checkout@production`. A request failure leaves the previous observation in the catalog.

The default service package includes the record mapping. Run the local service catalog kit to verify the connector against an emulator before deployment. [Package and setup example](https://github.com/seikika/seikika/tree/main/packages/service-catalog) · [Local test kit](https://github.com/seikika/seikika/tree/main/examples/service-catalog) · [Workers Deployments API](https://developers.cloudflare.com/api/resources/workers/subresources/scripts/subresources/deployments/methods/list/)
