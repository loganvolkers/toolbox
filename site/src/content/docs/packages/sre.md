---
title: SRE readiness
description: Score one service journey using evidence from reliability, paging, and recovery tests.
---

Use the site reliability engineering (SRE) package to review production readiness for one critical journey, such as checkout. It scores evidence for the service objective, paging, recovery, and incident response. A served portal can also show a read-only health observation.

## Connect a readiness command

You need Seikika 0.2, the [Runner connector](/connectors/runner/), and a command that prints one `readiness` connector record as newline-delimited JSON. Start from the Prometheus adapter and a copy of its rule tests. In this example, those files and your evidence file are under `ops/` in the adopting checkout. Python 3 and `promtool` must be available to the adapter. [Prometheus example](https://github.com/seikika/seikika/tree/5818e032cac3cefec1fe9f6a2a4559bd410174cf/packages/sre/examples/prometheus) · [Pin and install a package](/packages/#pin-and-install)

```yaml
include:
  - package: github:seikika/seikika/packages/sre@5818e032cac3cefec1fe9f6a2a4559bd410174cf
    with:
      service: checkout
      owner: group:checkout
      command:
        - python3
        - ops/readiness.py
        - ops/evidence.json
        - ops/rules.test.yaml
        - --max-drill-age-days
        - "90"
```

The default environment is `production`. The record must name the same service and environment and give the drill's observation time as `recovery.observedAt`. Set the maximum drill age to match your team's policy. Supply evidence from your own tests and drill artifacts. Give each additional service or environment a distinct PascalCase `kind` prefix. [Package reference](https://github.com/seikika/seikika/blob/5818e032cac3cefec1fe9f6a2a4559bd410174cf/packages/sre/README.md)

## Verify one service

Run this from the checkout root to score readiness while you prepare the optional live health adapter.

```bash
seikika report .seikika --skip checkout-production-sre-health --all
```

Look for `checkout-production-sre-readiness`. Confirm the scorecard has an entity for checkout in production and inspect the evidence behind a result. If the entity is missing, run the readiness command from the checkout root and check its exit status, service, environment, and record kind.

Use the [test portal](https://github.com/seikika/seikika/tree/5818e032cac3cefec1fe9f6a2a4559bd410174cf/packages/sre/tests) for examples of passing and failing evidence. To add live health context, configure a separate read-only `healthCommand` as described in the package reference.
