# Playwright for acceptance checks

This private package pins the Playwright version used by the maintainer CI
browser check. The [acceptance job](../../.github/workflows/ci.yml) installs
it in a temporary directory, installs Chromium, and runs the browser check
against a local portal.

See the [browser check](../ui-a11.cjs) for the catalog interactions it covers.
Use [Explore a portal](https://seikika.com/start/portal/) to try the UI yourself.
