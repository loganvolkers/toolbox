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
- `actions/report/`
- Root changelog-worthy behaviour consumers depend on

## File format

```markdown
---
"toolbox": minor
"pnpm-expo": minor
---

Short summary for consumers — becomes a changelog bullet.
```

Logical names are defined in `config.json` → `registry`:

| Name | Kind | Version source |
|---|---|---|
| `toolbox` | cargo-workspace | `Cargo.toml` `[workspace.package].version` |
| `github-actions`, `pnpm-expo`, `storybook` | toolbox-package | `packages/<name>/toolbox-package.yaml` |
| `actions/report` | action | same git tag as `toolbox` (lockstep) |
| `toolbox-operator` | plugin | Phase 3 |

## Lockstep policy (default)

One release version ships binaries, in-repo packages, and `actions/report`.
Package YAML `version:` **must match** the git tag (`vX.Y.Z` ↔ `X.Y.Z`).
The applicator sets every package YAML to that version when applying.

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
python3 scripts/apply-changesets.py --self-test
```
