# Seikika docs

Astro Starlight publishes the official Seikika documentation, the catalog report, and Storybook at [seikika.com](https://seikika.com/) through GitHub Pages. Follow the [documentation style guide](src/content/docs/maintainers/writing.md) when you change a page.

```sh
corepack enable pnpm
pnpm install --frozen-lockfile
pnpm docs:build
pnpm --filter seikika-site preview
```

The report comes from `site/.generated/report.json` when present. Run `scripts/regen-site-report.sh` to generate it with the Pages workflow's pinned tools. A local docs build can use the checked-in Seikika-for-Seikika fixture. Set `SEIKIKA_REPORT_DATA` to an absolute path to use another snapshot.

The Reference group in `astro.config.mjs` collects interface lookups. The CLI and MCP references are built from `seikika describe` and the commands' `--help` output. Set `SEIKIKA_BIN` to use a particular build of Seikika from this checkout. After a CLI or MCP tool change, regenerate and check the documentation fingerprint.

```sh
pnpm --filter seikika-site generate:cli-docs
pnpm --filter seikika-site check:cli-docs
```

Run `pnpm check:packages --write` when a public package enters or leaves the workspace.
