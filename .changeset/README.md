# Changesets (portable)

This directory holds **Changesets-compatible** release-intent files for Toolbox.
The format matches [Changesets](https://changesets.dev/) (YAML frontmatter + summary),
but versions are applied by `scripts/apply-changesets.py`, not `@changesets/cli`.

cargo-dist (`.github/workflows/release.yml`) remains the binary publisher. Do not replace it.

## When to add a changeset

**Soft guidance:** any PR that changes a releasable artifact should add a file here.
CI does not fail PRs that omit one (Phase 1). Maintainers can add changesets on the
Version Packages PR if needed.

Releasable paths include:

- Rust crates / connectors (`crates/`, root `Cargo.toml`)
- Toolbox packages (`packages/*/`)
- `actions/*/`
- Root changelog-worthy behaviour consumers depend on

## File format

```markdown
---
"toolbox": minor
"pnpm-expo": minor
"actions/report": minor
---

Short summary for consumers — becomes a changelog bullet.
```

### How names map

| Frontmatter name | How it is resolved | Version source |
|---|---|---|
| `toolbox` | Explicit in `config.json` → `registry` | `Cargo.toml` `[workspace.package].version` |
| `<package>` (e.g. `pnpm-expo`, `file-size`) | Discovered by `includes` glob `packages/*/toolbox-package.yaml`; name = YAML `name:` | that package's `toolbox-package.yaml` `version:` |
| `actions/<dir>` (e.g. `actions/report`) | Discovered by `includes` glob `actions/*/action.yml` | same git tag as `toolbox` (lockstep; no separate Action semver file yet) |
| `toolbox-operator` | Explicit `registry` entry | Phase 3 plugin versioning |

New packages under `packages/<name>/toolbox-package.yaml` and new Actions under
`actions/<name>/action.yml` are picked up automatically — you do **not** need to edit
`config.json` for each one. List only special cases (Cargo workspace, plugins) under
`registry`.

**Fixtures:** keep test packages under `pkg-fixture/` (or outside `packages/`), never as
`packages/**/toolbox-package.yaml`, so Toolbox-for-Toolbox discovery and the ratchet stay
clean.

Inspect the resolved set:

```bash
python3 scripts/apply-changesets.py --list-registry
```

## Lockstep policy (default)

One release version ships binaries, in-repo packages, and Actions.
Package YAML `version:` **must match** the git tag (`vX.Y.Z` ↔ `X.Y.Z`).
The applicator sets every discovered package YAML to that version when applying.

## Workflow

1. Contributors merge feature PRs **with** `.changeset/*.md` files (versions on `main` stay unchanged).
2. On `main`, `.github/workflows/version-packages.yml` opens or updates a **Version Packages** PR that runs `scripts/apply-changesets.py`.
3. Merge that PR when ready → versions + `CHANGELOG.md` land on `main`, consumed changesets are deleted.
4. **Human or script tags** `vX.Y.Z` on green `main` (see `docs/RELEASING.md`).
5. Existing `release.yml` / `image.yml` build archives, installers, attestations, and the GHCR image.

Local apply (dry-run first):

```bash
python3 scripts/apply-changesets.py --dry-run
python3 scripts/apply-changesets.py
```

Self-test (also run in CI):

```bash
python3 scripts/test-apply-changesets.py
```
