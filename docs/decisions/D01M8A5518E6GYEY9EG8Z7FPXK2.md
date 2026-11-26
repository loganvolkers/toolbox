---
id: D01M8A5518E6GYEY9EG8Z7FPXK2
decided: 2026-11-25T20:22:02Z
---

# Astro and Vue for Seikika user interfaces

Seikika authors its first-party UI in Astro and Vue. PNPM manages the frontend workspace. The shared
Vue components and tokens live in `@seikika/ui`. Use named slots and props to customize the page
frame, entity page and scorecard page across the static report and the hosted portal.

`@seikika/astro-report` mounts the report from CLI snapshot JSON and accepts a site layout.
GitHub Pages uses this integration and publishes the Storybook for the whole workspace. The CLI
bundles the same Astro report into its offline HTML artifact. The hosted portal's Astro assets stay
embedded in the Rust server. Its viewer authorization, refresh and action APIs stay in Rust.

Every first-party UI surface has stories. CI builds the public npm packages, validates their catalog,
and checks the stories at desktop and mobile widths in both color schemes. Seikika catalogs public
registry access and links as package manifest facts.

Operators can compose the shared components in their own Astro builds. Independently loaded
UI modules are part of the proposed extension design.

See [frontend architecture](../design/frontend.md) and [Vue components](../../ui-kit/README.md).
