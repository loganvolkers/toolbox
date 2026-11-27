---
id: D01M8DJJTZ4JPBHYDS7VN9TV21J
decided: 2026-11-27T04:00:29Z
---

# Check maintained shell scripts with ShellCheck, shfmt and Bats

Use ShellCheck warnings and errors and shfmt with two-space indentation and indented
case arms as the shell source gate. Use Bats for behavior tests with local fixtures
and substitutes for external commands. New behavior and bug fixes need a Bats
regression case when they can be exercised in isolation.

Carry ShellCheck and shfmt results through the runner's NDJSON slot in the
[shareable shell package](../../packages/shell/README.md). Run Bats directly in
CI so a failing test blocks the pull request and its TAP output is visible. Adopt
the package in Seikika's own portal for maintained scripts and the report action
and operator launcher. Archived experiment harnesses and examples are outside
this first rollout.

The package uses ShellCheck's JSON output and shfmt's formatted output. Each record
includes the standalone tool commands so a developer can reproduce findings and
apply formatting without Seikika.
