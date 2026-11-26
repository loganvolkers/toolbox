# Hosted portal UI

This Astro and Vue app renders the interface that the Seikika server serves.
It uses the [shared Vue components](../ui-kit/README.md).
The [Astro report integration](../report-ui/README.md) renders static snapshots.

To try the UI with data, follow [Explore a portal](https://seikika.com/start/portal/).
The walkthrough shows a catalog entry, its owner, and its score.

For changes to the app, use the repository's pinned Node and pnpm versions.
Run `pnpm --filter seikika-portal-ui check` from the repository root. Follow
[Build Seikika](https://seikika.com/maintainers/) to rebuild and check the assets
embedded in the Rust server. The generated files live in
[`crates/seikika-api/ui-dist`](../crates/seikika-api/ui-dist/).
