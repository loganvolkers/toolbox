/**
 * Scaffold: implement a Backstage EntityProvider for Forge.
 *
 * Read the pinned Backstage tree in BACKSTAGE_ROOT.txt, especially
 * plugins/catalog-backend-module-github/src/providers/GithubEntityProvider.ts
 * for the EntityProvider pattern (getProviderName, connect, mutation).
 *
 * Also write mapped entities to OUT/entities.ndjson in the experiment schema
 * (a small Node script or the provider's run may do this).
 */

export class ForgeEntityProvider {
  getProviderName(): string {
    return 'forge';
  }

  // async connect(connection: EntityProviderConnection): Promise<void> { ... }
  // async run(): Promise<void> { ... }
}
