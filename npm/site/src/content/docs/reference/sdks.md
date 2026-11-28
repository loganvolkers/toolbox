---
title: SDK reference
description: Find the Rust connector SDK and the Vue and Astro package API references.
---

Open the reference for the package you install. Select the matching version on docs.rs or npm when checking a released API.

| Package | Use it to | Reference |
|---|---|---|
| `seikika-connect` | Implement connector reads and serve them over stdio or HTTP. | [Rustdoc on docs.rs](https://docs.rs/seikika-connect/) |
| `@seikika/ui` | Render catalog entities and scorecards with Vue components, theme tokens, and entity helpers. | [Vue package README on npm](https://www.npmjs.com/package/@seikika/ui) |
| `@seikika/astro-report` | Publish a static catalog and scorecard report in Astro. | [Astro report README on npm](https://www.npmjs.com/package/@seikika/astro-report) |

The npm package READMEs describe imports, props, slots, and configuration. Editors can show the comments shipped with the types at the call site. The [npm package directory](/authors/npm/) also links to the source for each package.

For a first connector, follow [Write a connector](/authors/connectors/). For a static report, follow [Generate and host a static report](/operators/publish/). Browse [Storybook](/storybook/) for rendered component examples.
