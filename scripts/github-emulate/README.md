# GitHub connector integration fixture

These maintainer tests run the GitHub connector against Emulate, which serves
a local GitHub API fixture. The tests check the records the connector emits
without requiring a GitHub token.

The [connector integration job](../../.github/workflows/ci.yml) builds the
connector, installs the locked fixture dependencies with Node 24, and runs
`connector.test.mjs`. See the [test file](connector.test.mjs) for the requests
and expected records.
