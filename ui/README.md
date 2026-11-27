# Hosted portal UI

This Astro and Vue app renders the interface that the Seikika server serves.
The portal and the [Astro report integration](../report-ui/README.md) mount the same Vue app.
The portal loads an authorized snapshot from `/v1/views/report?partial=true` and adds live search,
CEL filters, entity refresh, and operations. The partial snapshot keeps local views available while federation
sources load or fail. Views share the displayed snapshot until the user refreshes an entity or
updates to newer data. The server checks permissions when loading that snapshot and for each live
request. When anonymous demo access is configured, `/v1/views/home?public_catalog=true` supplies
the allowlist to the same catalog view. Previous portal paths still open the corresponding report view.

To try the UI with data, follow [Explore a portal](https://seikika.com/start/portal/).
The walkthrough shows a catalog entry, its owner, and its score.

For changes to the app, use the repository's pinned Node and pnpm versions.
Run `pnpm --filter seikika-portal-ui check` from the repository root. Follow
[Build Seikika](https://seikika.com/maintainers/) to rebuild and check the assets
embedded in the Rust server. `pnpm build` generates the Git-ignored
`crates/seikika-api/ui-dist/` directory.
