# Releasing Toolbox

A version tag releases every distable binary (D90). Nothing is built by hand.

## Cut a release

1. `version` in the root `Cargo.toml` (`[workspace.package]`) matches the tag you will push (today: `0.1.0`).
2. `dist plan` (cargo-dist 0.34.0) lists the archives and installers; it also checks that `.github/workflows/release.yml` still matches the generator (D98 — do not format that file by hand).
3. On a clean `main` with CI green:

   ```bash
   git tag v0.1.0
   git push origin v0.1.0
   ```

4. `.github/workflows/release.yml` builds Linux / macOS / Windows archives, shell and PowerShell installers, attestations, and a GitHub Release.
5. `.github/workflows/image.yml` then builds `ghcr.io/<owner>/toolbox:0.1.0` from the Linux archives.

Verify an artifact with `gh attestation verify <file> --repo <owner>/toolbox`.

## Install what a repo portal needs

`toolbox report` needs the `toolbox` binary plus sibling connector binaries (`toolbox-connector-workspace`, and `toolbox-connector-runner` when the portal uses the runner). Release installers are one app each; for a single install that unpacks the set used by `actions/report`, run:

```bash
# From a Toolbox checkout, or copy scripts/install-release.sh
TOOLBOX_VERSION=0.1.0 ./scripts/install-release.sh
toolbox --version   # 0.1.0
toolbox doctor examples/repo-portal-starter
toolbox report examples/repo-portal-starter
```

In CI, set `toolbox-version: "0.1.0"` on `loganvolkers/toolbox/actions/report` so the action downloads checksum-verified binaries instead of compiling.

## Public vs private (required for external consumers)

`uses: loganvolkers/toolbox/...` and unauthenticated release downloads only work when this repository is **public**. A consumer's default `GITHUB_TOKEN` cannot resolve a private action repo (`repository not found`) and cannot download private release assets.

**Human step (preferred for RTBuddy and other external customers):** GitHub → `loganvolkers/toolbox` → **Settings → General → Danger Zone → Change repository visibility → Public**.

**Workaround if it must stay private:** each consumer repository needs a PAT (or fine-scoped GitHub App token) with `contents: read` on `loganvolkers/toolbox`, stored as a secret (e.g. `TOOLBOX_READ_TOKEN`), then:

```yaml
- uses: actions/checkout@v4
- uses: loganvolkers/toolbox/actions/report@v0.1.0
  with:
    toolbox-version: "0.1.0"
  env:
    GH_TOKEN: ${{ secrets.TOOLBOX_READ_TOKEN }}   # install.sh / gh release download
```

Also grant that token (or add the consumer as a collaborator) so GitHub can resolve the `uses:` line — otherwise the workflow fails before any step runs.
## After the first tag

- Point consumers at the tag, not at `cargo build`.
- Homebrew, npm and aqua channels are still deferred (see `portal/sdlc/stages/maintainer.yaml` gaps).
- cargo-binstall metadata lives on each distable crate's `Cargo.toml`.
