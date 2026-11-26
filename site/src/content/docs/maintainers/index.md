---
title: Build Seikika
description: Build from source, run the checks, and contribute changes to Seikika or its documentation.
sidebar:
  order: 1
---

Use this guide when you change Seikika's source code, first-party connectors, or public docs. For changes to a portal you operate, see [Build a portal](/operators/).

## Build and test a change

Follow [Run Seikika locally](/start/run/) to build and validate the reference portal. It contains fixtures and expected results for entities, scorecards, permissions, and action options. If a behavior change alters those results, review the difference before updating them.

Run the checks for the code you changed from the repository root. Use the mise setup from the build guide.

```bash
mise exec -- cargo fmt --all --check
mise exec -- cargo clippy --all-targets --locked -- -D warnings
mise exec -- cargo test --locked
```

Use the [repository working rules](https://github.com/seikika/seikika/blob/main/AGENTS.md) for the project's acceptance checks, tool pins, and decisions.

## Contribute to the docs

1. Find the page under `site/src/content/docs/`. Follow the [documentation style guide](/maintainers/writing/) and verify any changed command with the stated version.
2. With the checkout's tools installed through mise, run the site checks from the repository root. Build the CLI first because the site reads its help output.

   ```bash
   mise exec -- pnpm install --frozen-lockfile
   mise exec -- pnpm --filter seikika-site check:cli-docs
   mise exec -- pnpm docs:build
   ```

3. Run `mise exec -- pnpm --filter seikika-site preview`. Check the rendered page, local links, headings, and code blocks. Review the change on a narrow screen and check that you can use the page with a keyboard.
4. Include what you ran and the result in your pull request. The [Pages workflow](https://github.com/seikika/seikika/blob/main/.github/workflows/pages.yml) builds the site and publishes changes from `main`.

The [public docs scorecard](/concepts/scorecards/#this-site) checks page structure. The [documentation gap analysis](https://github.com/seikika/seikika/blob/main/docs/audits/public-docs-2026-11-26.md) explains how the docs are evaluated against Backstage and other open-source projects.

## Check the repository standards

Seikika scores its own code, docs, and workflows. From the checkout, with the required tools installed:

```bash
mise exec -- target/debug/seikika report portal --skip runner,github-actions --baseline portal/results.json --ci
```

The check compares the current results against the committed baseline. Review the differences before updating it. See the [repository portal README](https://github.com/seikika/seikika/blob/main/portal/README.md) for the full check and tool setup.

Report bugs and propose changes in [GitHub Issues](https://github.com/seikika/seikika/issues). For vulnerabilities, follow the [security policy](https://github.com/seikika/seikika/security/policy).
