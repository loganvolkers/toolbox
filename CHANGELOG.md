# Changelog

## Unreleased

## 0.1.1 - 2026-11-21

### Added

- `packages/pnpm-expo` (0.1.0): `pnpm-workspace` and `expo-app` scorecards for pnpm monorepos and Expo apps (D01M7Y6X38AERPBT0YH1AG5TBRD).
- Workspace connector facts: `jsWorkspace` → `js-workspace` records; `expoApps` globs → `expo-app` records (`packageManager`, linker, mise, frozen CI, EAS, doctor-in-CI).

## 0.1.0 - 2026-11-21

First installable release (D90).

### Added

- Attested per-platform archives and shell/PowerShell installers for `toolbox` and the first-party connectors (`github`, `git-files`, `runner`, `workspace`, `scim`).
- Container image workflow (`ghcr.io/<owner>/toolbox:<version>`) built from the Linux archives.
- `actions/report` input `toolbox-version` to download checksum-verified binaries instead of compiling.
- Repo portal starter (`examples/repo-portal-starter`) and packages (`packages/github-actions`).
- `toolbox install` / `toolbox doctor` for released executable connectors and declared tools.

### Notes

- M0–M4 are implemented; M5 (actions) and M6 (hosted operator loop) are paused while repo-level value is proven.
- Homebrew, npm and aqua install channels are not shipped in this release.
