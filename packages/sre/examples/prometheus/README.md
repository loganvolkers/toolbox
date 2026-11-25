# Prometheus checkout example

The example counts eligible checkout attempts and uses the error
fraction to measure burn against a 99.9% objective over 30 days. It
assumes enough traffic to evaluate request ratios. Use a separate
end-to-end synthetic signal when traffic is sparse. The missing-data
ticket detects absent series, and zero eligible attempts produce no burn
measurement.

```sh
cd packages/sre/examples/prometheus
promtool check rules rules.yaml
promtool test rules rules.test.yaml
```

Route `severity=page` to your pager and `severity=ticket` to your ticket
integration. In production, deduplicate or inhibit the ticket during a
page. Replace the example runbook link and send a test notification
before setting `paging.routeVerified`. A unit test exercises rule
evaluation, while a test notification verifies routing. Set
`paging.ruleTestsPassed` from the `promtool` exit status when producing
the readiness record.

If you already generate Prometheus rules with Sloth, feed the generated
rule file into the same native test flow and use the generated alert
names in the expectations. Keep the error-budget policy in Git and the
page route and drill results in their respective systems. The
[package README](../../README.md) describes the evidence your adapter
emits.

[Prometheus rule tests](
  https://prometheus.io/docs/prometheus/latest/configuration/unit_testing_rules/
) ·
[Sloth rule generation](https://sloth.dev/introduction/architecture/)
