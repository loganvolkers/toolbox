# SRE readiness kit

This opt-in package gives one service journey a production-readiness
scorecard and a read-only incident brief. It serves the
**operator SDLC** and starts in one repository. The package is the box.
Your existing monitoring, pager, deployment and incident tools own the
measurements and live workflows.

## Adopt

Include a pinned release in your repo portal. Use a relative package
path when trying it from a checkout.

```yaml
include:
  - package: github:loganvolkers/toolbox/packages/sre@<tag>
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

Use lowercase letters, digits and hyphens in `service` and
`environment`. The defaults are `production`, `critical: true`, and a
`5m` connector refresh interval. For a second include, choose a distinct
PascalCase `kind` prefix and a distinct service/environment combination.
The same service and environment must appear in the emitted record.

The readiness command runs from the checkout root. It must exit nonzero
if it cannot emit its one readiness record. Derive the flags from tests
and source artifacts, rather than filling in checkboxes. Start with the
[Prometheus example](examples/prometheus/README.md).

| Evidence          | Readiness requirement                                                                                                                                                                                                 |
| ----------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Identity          | Service, environment, source, observed time and artifact revision                                                                                                                                                     |
| Journey           | Good and eligible events, a fractional SLO target, window and error-budget policy with exception ownership                                                                                                            |
| Paging            | Tested page, ticket, spike and missing-data behavior, a verified route, and a linked runbook                                                                                                                          |
| Recovery          | Verified rollout stop, mitigation and customer-visible recovery, with a current drill artifact. Critical services also exercise dependency loss under peak load and reduced capacity, then verify controlled re-entry |
| Incident response | Named roles, shared timeline, fallback communications and a follow-up policy requiring owners, due dates and verification                                                                                             |

The [fixture messages](tests/fixtures) define the connector NDJSON
shape. Use RFC 3339 timestamps. In `recovery.fresh`, use the drill's
actual observation time and the team's agreed drill cadence. The
connector refresh interval governs collection and is independent of that
cadence. Capture mitigation and backlog clearance separately in the
drill artifact. Scope the incident record to qualifying incidents under
the team's policy.

```sh
toolbox validate packages/sre/tests
toolbox test packages/sre/tests

# In the adopting repository, review the initial verdict before committing a baseline.
toolbox report .toolbox --skip checkout-production-sre-health -o failures
toolbox report .toolbox --skip checkout-production-sre-health --baseline .toolbox/results.json --update
toolbox report .toolbox --skip checkout-production-sre-health --baseline .toolbox/results.json
```

A launch reviewer should require a current bronze result or a documented
exception. The ratchet catches degraded controls. Check that the
expected readiness entity exists when using a custom evidence command.

## Incident context

Set `healthCommand` in the served portal only when a read-only adapter
is available. It emits one `health` record with `source`, `revision`,
`observedAt`, `validUntil`, `status`, and `burnRate`. Put source links
for dependencies, recent changes, incidents and dashboards in `context`.
Keep incident details, credentials and customer data in their source
systems.

Use the [incident brief skill](skills/incident-brief/SKILL.md) with the
portal's existing read permissions. An absent or expired health
observation is unknown. Burn is independent of readiness and has no
scorecard rule.

For a pilot, compare this brief with the same dashboards, pager and
runbook used directly in a staged dependency failure. Measure time to
the first correct mitigation decision, unsupported claims, and
independent verification of customer recovery.

[Google SRE Workbook](https://sre.google/workbook/table-of-contents/) ·
[Microsoft post-incident report, public mirror](
  https://www.365fanatic.com/wp-content/uploads/2026/11/postincidentreport-mo1550821.pdf
) · [Toolbox package model](../../docs/design/packages.md)
