# Changelog

## Unreleased

## 0.2.0 — 2026-11-26

- Build the hosted portal and offline report from Astro and Vue. The report uses the same shared entity and scorecard components as the live portal. Publish the report and UI Storybook with the docs. Embed the Astro builds in Seikika so the binary and offline report can run without a JavaScript build tool.
- Scorecards and rules take a `rationale` and `links`, manifests group kinds with `kindGroups`, a kind may carry a reader-facing `title` (the `sarif` package sets it through `kindTitle`), and every rule has a computed basis (measured or agent-judged). Every package scorecard and rule now says why it exists. The HTML report gains Tracks, kind groups in the catalog, rule pages, Measured and Agent-judged badges wherever a result is shown, and a Compare view that explains its verdict and how it was measured. The markdown report and the ratchet's gate summary mark agent-judged rules with ✦. Links are titled by what they say rather than by a decision id, and the HTML report drops parentheticals that only cite internal ids or doc paths. The overview marks standards at their top level and names a standard that caps the repository level; a level that rests on unrun checks is drawn as provisional everywhere; no page scrolls sideways on a phone. A kind page says why some of its entities are not scored, and the design-audit capture counts WCAG 2.2 rules such as target size.
- Rename the project, CLI, package identifiers, portal manifests, protocol fields, environment

## 0.1.1 - 2026-11-21

### Added

- `packages/pnpm-expo` (0.1.0): `pnpm-workspace` and `expo-app` scorecards for pnpm monorepos and Expo apps (D01M7Y6X38AERPBT0YH1AG5TBRD).
- Workspace connector facts: `jsWorkspace` → `js-workspace` records; `expoApps` globs → `expo-app` records (`packageManager`, linker, mise, frozen CI, EAS, doctor-in-CI).

## 0.1.0 - 2026-11-21

First installable release (D90).

### Added

- Attested per-platform archives and shell/PowerShell installers for `seikika` and the first-party connectors (`github`, `git-files`, `runner`, `workspace`, `scim`).
- Container image workflow (`ghcr.io/<owner>/seikika:<version>`) built from the Linux archives.
- `actions/report` input `seikika-version` to download checksum-verified binaries instead of compiling.
- Repo portal starter (`examples/repo-portal-starter`) and packages (`packages/github-actions`).
- `seikika install` / `seikika doctor` for released executable connectors and declared tools.

### Notes

- M0–M4 are implemented; M5 (actions) and M6 (hosted operator loop) are paused while repo-level value is proven.
- Homebrew, npm and aqua install channels are not shipped in this release.
