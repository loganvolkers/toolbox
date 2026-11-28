---
title: Service catalog
description: Declare software services and connect their code, deployments, and operational metrics.
---

Include the service catalog package to add searchable `Service` and `ServiceEnvironment` kinds. Give each service a stable slug, title, description, accountable team, lifecycle, and source repository. Use `{title, url}` entries for the source, runbook, dashboard, or on-call links. [Pin and install a package](/packages/#pin-and-install) · [Manifest and inventory examples](https://github.com/seikika/seikika/tree/main/examples/cloudflare-containers/seikika-forks)

Declare each environment under the same service slug. Use the actual Worker name, OpenTelemetry `service.name`, and dashboard UID for that environment. A Service page can show active version traffic splits, recent deployments, and a few metric values with the time of observation. Use the dashboard for time-series investigation. [Cloudflare connector](/connectors/cloudflare/) · [Grafana connector](/connectors/grafana/) · [Observability playbook](/operators/observability/)

Run `seikika validate` on the adopting manifest and use the local integration kit to try the joins with Grafana in a container and emulated Cloudflare and GitHub. [Package reference](https://github.com/seikika/seikika/tree/main/packages/service-catalog) · [Local test kit](https://github.com/seikika/seikika/tree/main/examples/service-catalog)
