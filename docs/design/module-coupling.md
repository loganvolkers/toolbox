# Module coupling

> Status: Implemented in the maintainer SDLC.

The maintainer SDLC gates dependencies between shipped modules. Seikika carries the findings and
tracks the campaign. The standalone checker owns the policy and produces the same evidence from
the same checkout. This work supports the single-repository end of Seikika.

## Sources and tool comparison

| Option | What it can enforce | Fit for Seikika |
| --- | --- | --- |
| [ArchUnit for Java](https://www.archunit.org/userguide/html/000_Index.html) | Bytecode-based dependency direction, independent or acyclic slices, and freezing of existing violations. | Adopt the slice and layer rules and report exact evidence so a falling count cannot hide a different violation. The analyzer is JVM-specific. |
| [Cargo metadata](https://doc.rust-lang.org/stable/cargo/commands/cargo-metadata.html) and [Guppy](https://docs.rs/guppy/latest/guppy/) | Cargo exposes direct workspace declarations. Guppy adds queries over resolved package and feature graphs. | Direct declarations are enough for the crate policy, including renamed, target-specific, optional, build, and development edges. Use Guppy if feature-sensitive transitive constraints become necessary. |
| [ArchUnitRust](https://docs.rs/archunit/latest/archunit/) and [cargo-bylaw](https://github.com/danielgerlag/cargo-bylaw) | Rust source-level architecture tests and dependency rules. | Candidates for enforcing module boundaries *within* a crate. Evaluate source and macro coverage before adding a second Rust analyzer. |
| [dependency-cruiser](https://github.com/sverweij/dependency-cruiser) | JavaScript, TypeScript and Vue graphs with cycle, grouped cycle, forbidden edge, and declaration rules. | Strong choice for a broader source graph. Astro is absent from its documented source formats. Verify Astro and MDX coverage through an adapter before using it as the sole gate. |
| [Nx project boundaries](https://nx.dev/docs/features/enforce-module-boundaries) | Project tags express dependency directions and can share a graph across projects. | The role model is useful. Cargo and pnpm already provide workspace inventories, and adopting an additional project runner for four npm modules would add configuration. |
| [Backstage package roles](https://backstage.io/docs/tooling/cli/build-system/) and [Node package exports](https://nodejs.org/api/packages.html#package-entry-points) | Package roles distinguish shipping units from reusable libraries. Package exports declare entry points. | Use package names and enforce exports at cross-package imports. Node's export map alone still permits relative or absolute filesystem bypasses. |
| [Port scorecards](https://docs.port.io/governance/standards-and-compliance/overview/) | Conditions on catalog data plus ongoing compliance visibility. | Keep findings per module and expose the denominator through a campaign so progress can be tracked. |
| [Playwright testing guidance](https://playwright.dev/docs/best-practices) and [metamorphic testing](https://idus.us.es/items/a8835a99-987a-4841-8f14-a4a824507ebd) | Control inputs and test user-visible outcomes, including the response to changes in input. | Exercise an empty catalog and a portal with a different scorecard ID and display name. Assertions on static screenshots or the production demo can pass with hard-coded data. |

## Enforced policy

Run `pnpm check:coupling` after the frozen install. The command is offline and exits 1 with
repository-relative evidence for rule failures. `node scripts/check-coupling.mjs --ndjson` sends
the same results through the runner's NDJSON slot. The scorecard reports all four finding
counts per module, and the campaign targets all modules clean.

| Rule | Boundary |
| --- | --- |
| Acyclic | The union of direct internal dependency edges is acyclic in each workspace. Cargo declarations include all dependency kinds and targets. npm includes dependency declarations and literal production imports. Each cyclic edge counts once and has a return path. |
| Deployment independence | Keep Rust members and local crate dependencies inside `crates/`. Product source may not import deployment files from `.seikika/`, `examples/`, or `experiments/`, and reusable npm components may not depend on the docs site. The site is a composition layer and can use product components and deployment data. |
| Public entry points | Internal npm imports require a runtime workspace dependency and a public package export. Reject filesystem paths across npm packages and file-linked manifest dependencies. |
| Deployment links | Reusable frontend files may not pin Seikika demo URLs or concrete scorecard anchors. Derive the destination from available data. Stable routes such as `#scorecards` and `/ops` are application contracts. |

The source scan covers npm `src/` and the Astro integration entry, including Vue script and
style blocks, Astro frontmatter and script blocks, MDX imports, and CSS imports.
TypeScript resolves Astro’s tsconfig path aliases, including inherited configuration. Alias
targets use the same boundary checks as relative imports. An unresolved explicit path alias
fails the gate. Use tsconfig paths for application aliases. Review custom bundler plugins and
configuration that rewrite imports. The scan checks Rust
literal compile-time includes in crate source and build scripts. Test and Storybook composition
may read fixtures and exercise several packages. These tools do not resolve arbitrary runtime
I/O, generated imports, or computed strings. Review changes to these mechanisms
and extend the scan when a production use is introduced.

The first cleanup gives the live Vue app and documentation site public report exports. The guest
browser probes exercise a catalog without a report and a guest report with a synthetic readiness
card. They check that the link appears only when useful and that it opens the current portal's list.

## Assessment and next restrictions

When a change touches production modules or the gate, review the evidence under the
`module-coupling` testing contract. Check the import direction, read the actual payload and
assertions, and confirm that changing a portal's identity, data, and configuration changes the
display. Look for generated state copied into production, private API response assumptions,
hard-coded fallbacks, and skipped or overly specific assertions. Cite source and test evidence.
The person or agent implementing the change can record observations, and a separate reviewer
should adjudicate them before treating the assessment as independent.

Candidate restrictions for follow-up are an explicit Rust layer matrix for domain, storage,
adapters, and composition, isolation of server-only imports from browser bundles, published
package smoke builds from tarballs, and compatibility tests for the report payload. Record
outgoing and incoming internal edge counts before adding a fan-out policy. Set a budget only
where measured change propagation or review evidence shows a problem. Any temporary exception
should identify the specific edge, owner, reason, and removal condition.
