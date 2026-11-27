---
title: Generate and host a static report
description: Export a catalog snapshot as HTML, publish it on GitHub Pages, or add it to an Astro site.
sidebar:
  order: 10
---

Publish a static report when readers need to browse a catalog and its scores at a fixed point in time. A static host serves the generated files. Use a [running portal](/operators/deploy/) for fresh queries, viewer-specific permissions, connector refresh, or actions.

## Generate an HTML report

You need the Seikika CLI from the [0.2 source install](/start/install/#build-from-source). From that checkout, run the following with the sample data. It uses fixtures, so the report needs no connector credentials.

```bash
report_dir=$(mktemp -d)
seikika report examples/repo-portal-starter --fixtures -o html --out "$report_dir/index.html"
```

Open `index.html` from that directory in your browser. Select **Catalog** to browse the sample repositories and **Scorecards** to see checks and evidence. The file includes the code and data it needs. You can open it offline or upload it as an artifact.

For your own repo portal, run from your repository root. Use its manifest directory in place of `examples/repo-portal-starter` and omit `--fixtures` to score the checkout. Install the connector binaries and pinned tools the manifest requires. Review the snapshot before sharing it. The report has no per-viewer access checks, and the embedded data can include catalog values and rule evidence. Use sample data or export only information approved for the audience.

## Host the report on GitHub Pages

Use a repository with a `.seikika/seikika.yaml` configured for [repository scoring](/operators/ci/).
The workflow builds the HTML for each push to `main` without a baseline. Change the branch name if
your default is different. Review which facts the manifest collects before enabling publication.

1. In the repository, open **Settings**, then **Pages**. Set the publishing source to **GitHub Actions**. A repository admin or maintainer can set the source. [GitHub Pages publishing sources](https://docs.github.com/en/pages/getting-started-with-github-pages/configuring-a-publishing-source-for-your-github-pages-site)
2. Save the workflow as `.github/workflows/seikika-pages.yml`. Replace both `ACTION_SHA`
   placeholders with the same full commit SHA that includes the actions.
   You can use the commit for a [Seikika release](https://github.com/seikika/seikika/releases)
   once it includes them.
   If your portal needs more than the starter's workspace connector, install those connectors and
   tools in the build job before scoring.

```yaml
name: Publish standards report
on:
  push:
    branches: [main]
permissions:
  contents: read
concurrency:
  group: seikika-pages
  cancel-in-progress: true
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@d23441a48e516b6c34aea4fa41551a30e30af803 # v6.1.0
        with:
          persist-credentials: false
      - uses: seikika/seikika/actions/setup@ACTION_SHA
        with:
          seikika-version: "0.2.1"
      - name: Build the portal
        id: portal
        uses: seikika/seikika/actions/build-portal@ACTION_SHA
      - uses: actions/upload-pages-artifact@fc324d3547104276b827a68afc52ff2a11cc49c9 # v5.0.0
        with:
          path: ${{ steps.portal.outputs.directory }}
  deploy:
    needs: build
    runs-on: ubuntu-latest
    permissions:
      pages: write
      id-token: write
    environment:
      name: github-pages
      url: ${{ steps.deployment.outputs.page_url }}
    steps:
      - name: Deploy the report
        id: deployment
        uses: actions/deploy-pages@368f82528645a54fb793d4d04e342629a3f51346 # v5.0.1
```

3. After the deploy job succeeds, open the URL in the `github-pages` environment. Check that
   **Catalog** and **Scorecards** load and that the report describes the intended checkout.
   Enable [the ratchet](/operators/ci/) in CI if you also want to enforce standards on changes.

For another static host, upload the approved `index.html` to its publish directory. The host can serve it from a subdirectory because the file includes its assets.

## Add the report to an Astro site

Use this path when an existing static site should supply the report's navigation or layout. The example uses Node.js 24, Astro 7, and version 0.1.1 of the report integration. From your Astro project, install the integration and Vue:

```bash
npm install @seikika/astro-report@0.1.1 vue@^3.5.0
```

From your repository root, set `astro_project` to the absolute path to your Astro project. Change `.seikika` if your portal manifest is elsewhere.

```bash
seikika report .seikika -o web --out "$astro_project/generated"
```

The output directory contains `report.json`, `seikika-report.js`, and a standalone `index.html`.
In GitHub Actions, `actions/build-portal` with `format: web` provides the same files. Use its
`report-json` output as the integration's input. Keep the input outside Astro's `public/`
directory so the build publishes your chosen pages. The compiled report exposes its embedded
snapshot to its readers.

In `astro.config.mjs`, add the integration. If the file already exists, keep the integrations you use and add this one to the list.

```js
import { defineConfig } from 'astro/config';
import seikikaReport from '@seikika/astro-report';

export default defineConfig({
  integrations: [
    seikikaReport({
      data: new URL('./generated/report.json', import.meta.url),
      path: '/standards',
    }),
  ],
});
```

From the Astro project, run:

```bash
npx astro build
npx astro preview
```

Open `/standards/` under the preview URL and check that you can browse the report. Stop the preview with **Ctrl+C** when finished. Deploy Astro's `dist/` directory. If the host serves your project under a path such as `/my-project/`, configure Astro's `base` and check the report at `/my-project/standards/`. See [the integration options](https://github.com/seikika/seikika/tree/main/report-ui) for a custom layout and route aliases. [Deploy Astro to GitHub Pages](https://docs.astro.build/en/guides/deploy/github/)

## Troubleshoot a published report

| Symptom | Check |
|---|---|
| The site shows old results | Regenerate the snapshot from the intended commit and check the host's latest deployment. |
| The report route returns 404 | Check the build output, publication directory, and site base. |
| A rule reports `unknown` | Check the source data and run `seikika doctor` against the portal before regenerating. |
| Readers need private views or fresh data | Use the [running portal](/operators/deploy/) for those requests. |

Today Seikika generates a report page and the snapshot Astro consumes. Astro generates other site pages. A native multi-page generator and a static index across child reports remain [proposals](/concepts/capabilities/#planned-capabilities).
