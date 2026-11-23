# Toolbox docs site

Public documentation for Toolbox, built with [Astro Starlight](https://starlight.astro.build/)
and published to GitHub Pages (`https://loganvolkers.github.io/toolbox/`).

```bash
npm ci
npm run build
```

## CLI reference

`src/content/docs/start/cli.md` is **generated** from the `toolbox` binary
(`toolbox describe` plus each leaf command’s `--help`). Do not edit that page by hand.

From the repository root:

```bash
python3 scripts/generate-cli-docs.py
python3 scripts/generate-cli-docs.py --check   # exit 1 if the committed page is stale
```

From `site/`:

```bash
npm run generate:cli-docs
npm run check:cli-docs
```

Optional: `TOOLBOX_BIN=/path/to/toolbox` to skip the default `target/debug/toolbox` lookup.
The pages workflow runs `--check` after building `toolbox`, so a stale page fails CI.
