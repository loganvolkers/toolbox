# Releasing Toolbox

A version tag releases every distable binary (D90). Nothing is built by hand.

## Continuous release (Changesets-compatible)

Toolbox uses **portable Changesets** (Option B): contributors add `.changeset/*.md`
files; `scripts/apply-changesets.py` bumps Cargo, `CHANGELOG.md`, and package YAML
under **lockstep** (versions match the git tag). `@changesets/cli` is not required.

The registry in `.changeset/config.json` lists special cases (`toolbox`, plugins) and
**`includes` globs** for `packages/*/toolbox-package.yaml` and `actions/*/action.yml`,
so new packages/Actions are versioned without editing every name into the config.
Changeset frontmatter still uses logical names: package YAML `name:` for packages,
`actions/<dirname>` for Actions (see `.changeset/README.md`).

1. Merge feature PRs that include changeset files (soft guidance — see `.changeset/README.md`).
2. On `main`, `.github/workflows/version-packages.yml` opens or updates a **Version Packages**
   PR that runs the applicator and deletes consumed changesets.
3. Merge the Version Packages PR when ready.
4. Run the [assessment preflight](#assessment-preflight), then tag and push (human step for now):

   ```bash
   VERSION=$(python3 -c 'import tomllib,pathlib; print(tomllib.loads(pathlib.Path("Cargo.toml").read_text())["workspace"]["package"]["version"])')
   git tag "v$VERSION"
   git push origin "v$VERSION"
   ```

5. `.github/workflows/release.yml` (cargo-dist) builds archives, installers, attestations,
   and a GitHub Release. Its last job calls `.github/workflows/image.yml`, which builds the GHCR image.

Maintainer-run apply (without the Version PR):

```bash
python3 scripts/apply-changesets.py --list-registry
python3 scripts/apply-changesets.py --dry-run
python3 scripts/apply-changesets.py
```

Fixture check (fixtures live under `pkg-fixture/`, not `packages/**`):

```bash
python3 scripts/test-apply-changesets.py
```

## Cut a release (manual / after Version PR)

1. `version` in the root `Cargo.toml` (`[workspace.package]`) matches the tag you will push.
   Package YAML `version:` fields match that same version (lockstep).
2. `dist plan` (cargo-dist 0.34.0) lists the archives and installers. `.github/workflows/release.yml`
   is dist's output plus hand patches kept by `allow-dirty = ["ci"]` (D98 — do not format it, and
   reapply the patches if you run `dist generate`).
3. On a clean `main` with CI green, run the [assessment preflight](#assessment-preflight):

   ```bash
   git tag vX.Y.Z
   git push origin vX.Y.Z
   ```

4. `.github/workflows/release.yml` builds Linux / macOS / Windows archives, shell and PowerShell
   installers, attestations, and a GitHub Release.
5. In the same run, after the release is announced, the `custom-image` job calls
   `.github/workflows/image.yml`, which builds `ghcr.io/<owner>/toolbox:X.Y.Z` from the Linux
   archives. A release made with `GITHUB_TOKEN` starts no other workflow, so the image is not
   built on a `release` event. For a tag released before this job existed, run
   `gh workflow run image.yml -f tag=vX.Y.Z`.

Verify an artifact with `gh attestation verify <file> --repo <owner>/toolbox`.

## Public frontend packages

The [npm package catalog](../site/src/content/docs/authors/npm.mdx) lists the two packages
distributed from the PNPM workspace. Their versions live in their package manifests.
The frontend CI job checks the catalog and packs the same files a registry release ships.

Run `pnpm check` before packing. Release `@toolbox-dev/ui` first when the Astro integration's
dependency range requires a new UI version, then release `@toolbox-dev/astro-report`. Package
publishing uses the maintainers' npm scope access.

## Install what a consumer needs

cargo-dist publishes one installer per app. The public page is
`site/src/content/docs/start/install.md`.

An operator installs the CLI. A portal whose manifest names `builtin:<name>`
also installs `toolbox-connector-<name>`. A repo portal names `builtin:workspace`.
`builtin:runner` is a second installer when that connector is enabled. A
connector author installs the CLI and builds the scaffold with Rust
(`site/src/content/docs/authors/connectors.md`).

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/loganvolkers/toolbox/releases/download/v0.1.1/toolbox-cli-installer.sh | sh
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/loganvolkers/toolbox/releases/download/v0.1.1/toolbox-connector-workspace-installer.sh | sh
export PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH"
toolbox --version   # 0.1.1
```

Windows uses the matching `*-installer.ps1` (`irm … | iex`).

`scripts/install-release.sh` unpacks `toolbox`, `toolbox-connector-workspace`
and `toolbox-connector-runner` into one directory after checking each
`.sha256`, for a checkout that already exists (the same layout as
`actions/report/install.sh`).

In CI, set `toolbox-version: "0.1.1"` on `loganvolkers/toolbox/actions/report` so the action
downloads checksum-verified binaries instead of compiling.

## Public vs private (required for external consumers)

`uses: loganvolkers/toolbox/...` and unauthenticated release downloads only work when this
repository is **public**. A consumer's default `GITHUB_TOKEN` cannot resolve a private action
repo (`repository not found`) and cannot download private release assets.

**Human step (preferred for RTBuddy and other external customers):** GitHub →
`loganvolkers/toolbox` → **Settings → General → Danger Zone → Change repository visibility → Public**.

**Workaround if it must stay private:** each consumer repository needs a PAT (or fine-scoped
GitHub App token) with `contents: read` on `loganvolkers/toolbox`, stored as a secret
(e.g. `TOOLBOX_READ_TOKEN`), then:

```yaml
- uses: actions/checkout@v4
- uses: loganvolkers/toolbox/actions/report@v0.1.1
  with:
    toolbox-version: "0.1.1"
  env:
    GH_TOKEN: ${{ secrets.TOOLBOX_READ_TOKEN }}   # install.sh / gh release download
```

Also grant that token (or add the consumer as a collaborator) so GitHub can resolve the `uses:`
line — otherwise the workflow fails before any step runs.

## After the first tag

- Point consumers at the tag, not at `cargo build`.
- Homebrew, npm and aqua channels are still deferred (see `portal/sdlc/stages/maintainer.yaml` gaps).
- cargo-binstall metadata lives on each distable crate's `Cargo.toml`.


## Assessment preflight

Build the current checkout and run the freshness gate with the same connector and tool setup used
in CI. Then review missing independent assessments for the change with the release owner.

```bash
cargo build --locked
target/debug/toolbox assess check portal --skip runner,github-actions --require-fresh
```

Resolve stale assessments and have a different reviewer review the refreshed judgments before
tagging. For a high-risk pull request, apply `require-fresh-assessments` to run the strict check
in CI. A label change triggers CI.
