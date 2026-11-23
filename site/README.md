# Toolbox docs site

Public documentation for Toolbox, built with [Astro Starlight](https://starlight.astro.build/)
and published to GitHub Pages (`https://loganvolkers.github.io/toolbox/`).

```bash
npm ci
npm run build
```

## CLI reference

`src/content/docs/start/cli.mdx` is a Starlight page that renders Astro/TypeScript
components. At build time it loads the live `toolbox` binary (`toolbox describe`
plus each leaf command’s `--help`) — not a committed markdown dump.

Drift guard: a fingerprint of the catalog is committed at
`src/data/cli-catalog.fingerprint.json`. Refresh and check:

```bash
# from site/ (needs TOOLBOX_BIN or ../target/debug/toolbox)
npm run generate:cli-docs   # write fingerprint
npm run check:cli-docs      # exit 1 if fingerprint is stale
```

Optional: `TOOLBOX_BIN=/path/to/toolbox`. The pages workflow runs `check:cli-docs`
after building `toolbox` and before `astro build`.
