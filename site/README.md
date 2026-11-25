# Toolbox docs

Astro Starlight publishes the official Toolbox documentation, the catalog report, and Storybook to GitHub Pages under `/toolbox/`.

```sh
corepack enable pnpm
pnpm install --frozen-lockfile
pnpm docs:build
pnpm --filter toolbox-site preview
```

The report comes from `site/.generated/report.json` when present. Run `scripts/regen-site-report.sh` to generate it with the Pages workflow's pinned tools. A local docs build can use the checked-in Toolbox-for-Toolbox fixture. Set `TOOLBOX_REPORT_DATA` to an absolute path to use another snapshot.

The CLI reference is built from `toolbox describe` and `toolbox <command> --help`. Set `TOOLBOX_BIN` to use a particular binary. After a CLI change, regenerate and check its documentation fingerprint.

```sh
pnpm --filter toolbox-site generate:cli-docs
pnpm --filter toolbox-site check:cli-docs
```

Run `pnpm check:packages --write` when a public package enters or leaves the workspace.
