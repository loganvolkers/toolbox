# Seikika docs

Astro Starlight publishes the official Seikika documentation, the catalog report, and Storybook at [seikika.com](https://seikika.com/) through GitHub Pages. Follow the [documentation style guide](src/content/docs/maintainers/writing.md) when you change a page.

```sh
corepack enable pnpm
pnpm install --frozen-lockfile
pnpm docs:build
pnpm --filter seikika-site preview
```

The report comes from `site/.generated/report.json` when present. Run `scripts/regen-site-report.sh` to generate it with the Pages workflow's pinned tools. A local docs build can use the checked-in Seikika-for-Seikika fixture. Set `SEIKIKA_REPORT_DATA` to an absolute path to use another snapshot.

The CLI reference is built from `seikika describe` and `seikika <command> --help`. Set `SEIKIKA_BIN` to use a particular binary. After a CLI change, regenerate and check its documentation fingerprint.

```sh
pnpm --filter seikika-site generate:cli-docs
pnpm --filter seikika-site check:cli-docs
```

Run `pnpm check:packages --write` when a public package enters or leaves the workspace.
