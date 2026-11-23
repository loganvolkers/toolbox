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
4. Tag and push (human step for now):

   ```bash
   VERSION=$(python3 -c 'import tomllib,pathlib; print(tomllib.loads(pathlib.Path("Cargo.toml").read_text())["workspace"]["package"]["version"])')
   git tag "v$VERSION"
   git push origin "v$VERSION"
   ```

5. `.github/workflows/release.yml` (cargo-dist) builds archives, installers, attestations,
   and a GitHub Release. `.github/workflows/image.yml` builds the GHCR image.

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
2. `dist plan` (cargo-dist 0.34.0) lists the archives and installers; it also checks that
   `.github/workflows/release.yml` still matches the generator (D98 — do not format that file by hand).
3. On a clean `main` with CI green:

   ```bash
   git tag vX.Y.Z
   git push origin vX.Y.Z
   ```

4. `.github/workflows/release.yml` builds Linux / macOS / Windows archives, shell and PowerShell
   installers, attestations, and a GitHub Release.
5. `.github/workflows/image.yml` then builds `ghcr.io/<owner>/toolbox:X.Y.Z` from the Linux archives.

Verify an artifact with `gh attestation verify <file> --repo <owner>/toolbox`.

## Install what a repo portal needs

`toolbox report` needs the `toolbox` binary plus `toolbox-connector-workspace`
(and `toolbox-connector-runner` when the portal uses the runner). cargo-dist
publishes one installer per app. Consumers run those, the same commands the
release notes print:

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/loganvolkers/toolbox/releases/download/v0.1.1/toolbox-cli-installer.sh | sh
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/loganvolkers/toolbox/releases/download/v0.1.1/toolbox-connector-workspace-installer.sh | sh
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/loganvolkers/toolbox/releases/download/v0.1.1/toolbox-connector-runner-installer.sh | sh
export PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH"
toolbox --version   # 0.1.1
```

Windows uses the matching `*-installer.ps1` (`irm … | iex`). The public install
page is `site/src/content/docs/start/install.md`.

`scripts/install-release.sh` unpacks that same trio into one directory after
checking each `.sha256`, for a checkout that already exists (the same layout as
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
