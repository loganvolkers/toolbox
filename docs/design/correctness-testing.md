# Testing functional correctness

*Status:* Guide and initial correctness scorecards implemented in the maintainer portal.

For the maintainer SDLC, start with the behavior that could break and the
consequence for a caller. Record the expected result and choose a test that
would fail if the result were wrong. Run the smallest test that crosses the
boundary where the bug could occur. Keep a few end-to-end acceptance tests
for workflows whose outcome depends on the whole system. Seikika carries the
test evidence and review in its portal. Cargo, the reference implementation
and the acceptance scripts execute the tests directly.

## Choose the test by failure mode

| Failure mode | Test and oracle | Seikika example |
| --- | --- | --- |
| Wrong branch, boundary value or error | Use a focused table of representative inputs, including empty, boundary and invalid values. Assert the result or typed error and any relevant state change. | CEL rules and scorecard levels in [portal fixtures](../../.seikika/tests/standards.test.yaml). |
| State changes under replay, reorder or retries | State the invariant and exercise multiple histories. Use deterministic inputs or record failing seeds and preserve minimized cases as regressions. | Compare outputs across record arrival orders in [reference.rs](../../crates/seikika-derive/tests/reference.rs). |
| Implementations or formats drift | Compare against a separately maintained oracle or a reviewed fixture. Check the complete set of expected outputs and review any update to golden files. | Compare Rust outputs with the [reference model](../../examples/reference-portal/README.md). |
| Callers see the wrong result | Exercise public CLI, API or MCP entry points and assert observable responses, error semantics and postconditions. Where a real boundary matters, use a real compatible service or run shared contract tests against the real service and its fake. | CLI exit status and evidence in [ratchet.rs](../../crates/seikika-cli/tests/ratchet.rs). |
| End-to-end wiring or recovery fails | Use a narrow acceptance journey with isolated state and explicit prerequisites. Include a failure or cancellation case when a partial write would matter. For browser flows, use user-facing locators and condition-based assertions. | Local and federated action journeys in [M5 acceptance](../../scripts/m5-acceptance.sh). |

A regression test should fail on the buggy behavior and pass with the fix.
Use a negative control when feasible. For example, change the expected
boundary outcome or reverse the comparison in a scratch copy and confirm
the intended assertion fails. An oracle should establish what the caller
observes, including protected state after a rejection. A test that only
checks a process exit code may miss a wrong output or partial write.

## Run and interpret evidence

- Keep normal presubmit coverage fast and isolated. Control the filesystem,
  clock and data as needed. Put Postgres, OpenFGA and browser checks in visible
  CI jobs with the dependencies provisioned and explicit failures when required
  services are missing. Preserve logs, failing inputs and first-attempt results.
- Treat skips, missing reports, empty runs and retries that pass as distinct
  evidence. Record and investigate repeated flakes. CI discovery in a
  workflow shows wiring only. Check the run itself for execution and outcomes.
- Use line and branch coverage to find unexercised code. Inspect gaps on
  changed and high-consequence paths before introducing thresholds. Work from
  a stated denominator and a report for the same revision.
- Use targeted mutation testing to check whether assertions detect plausible
  errors such as inverted comparisons or dropped side effects. Review
  surviving, equivalent and unexecuted mutations separately. Start on
  high-consequence changes where a maintainer can act on the result.
- Keep assertions close to the named behavior. When correcting a regression,
  prefer the narrowest test that reproduces it and add a broader case when
  the boundary itself was part of the bug.

## Seikika scorecards

Crates remain the execution unit for Cargo, with structural and runner
evidence on `maintainer-crate`. CLI and MCP remain the interface inventory.
`TestingContract` covers behavior that can span multiple crates, tools and
interfaces. The [initial contracts](../../.seikika/testing/contracts.yaml)
cover derivation and plan, the scorecard gate, and action postconditions.
Add contracts by consequence and identify the workflow files that exercise them.

`maintainer-testing` checks that the links exist and the workspace connector
finds evidence of CI discovery. That discovery is a repository-wide heuristic
based on workflow steps and test conventions. `review-testing` is advisory and
asks an agent to read the tests, specifications and CI steps for risk coverage,
meaningful oracles and reliable execution. The review distinguishes failed,
uncertain, missing and stale evidence. Changes to the declaration or its
listed files invalidate a review. List critical fixtures and dependencies in
the contract when a change to those files would invalidate the conclusions.

Use `seikika assess brief testing-contract:<id> --portal .seikika` and
`seikika assess check .seikika` for review. Run the cited suite, record
before-and-after evidence, and use the report ratchet after the improvement.

## Sources

- Google describes test size, scope, fidelity, coverage and the engineering
  costs of tests that are slow or unreliable. Its unit-testing chapter argues
  for observable behavior and clear oracles. [Testing Overview](https://abseil.io/resources/swe-book/html/ch11.html) · [Unit Testing](https://abseil.io/resources/swe-book/html/ch12.html)
- Ham Vocke explains why to combine focused integration and contract tests
  with a small set of broad journeys. [The Practical Test Pyramid](https://martinfowler.com/articles/practical-test-pyramid.html)
- Playwright documents test isolation, user-facing locators and web-first
  assertions. [Playwright Best Practices](https://playwright.dev/docs/best-practices)
- Inozemtseva and Holmes measured coverage and effectiveness against mutants in
  Java suites. The comparison controlled for suite size and supports using
  coverage to find untested code while requiring separate evidence of fault
  detection. [ICSE 2014 study](https://cs.uwaterloo.ca/~rtholmes/papers/icse_2014_inozemtseva.pdf)
- Google reports deploying mutation testing by focusing on changed code and
  selecting actionable mutants. [Practical Mutation Testing at Scale](https://research.google/pubs/practical-mutation-testing-at-scale-a-view-from-google/)
- Hypothesis documents how generated examples and recorded failures support
  reproducible property tests. [Replaying Failures](https://hypothesis.readthedocs.io/en/latest/tutorial/replaying-failures.html)
