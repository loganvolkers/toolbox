# Releasing Seikika

A version tag releases every distable binary (D90). Nothing is built by hand.

## Continuous release (Changesets-compatible)

Seikika uses **portable Changesets** (Option B): contributors add `.changeset/*.md`
files; `scripts/apply-changesets.py` bumps Cargo, `CHANGELOG.md`, and package YAML
under **lockstep** (versions match the git tag). `@changesets/cli` is not required.

The registry in `.changeset/config.json` lists special cases (`seikika`, plugins) and
**`includes` globs** for `packages/*/seikika-package.yaml` and `actions/*/action.yml`,
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
   and a GitHub Release. After publication, it builds the GHCR image and checks operator installs with mise.

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
   `.github/workflows/image.yml`, which builds `ghcr.io/<owner>/seikika:X.Y.Z` from the Linux
   archives. A release made with `GITHUB_TOKEN` starts no other workflow, so the image is not
   built on a `release` event. For a tag released before this job existed, run
   `gh workflow run image.yml -f tag=vX.Y.Z`.
6. The `custom-mise` job [checks the operator install](#install-what-a-consumer-needs) after
   the stable release is announced. Its consumer-action smoke test downloads the tagged
   CLI and connectors on Linux x64 and ARM64, then runs the ratchet and builds a static
   repo portal. The scheduled and manual runs use the starter's pinned version.

Before tagging, run the **image rehearsal** workflow against the release commit. It builds
both Linux release archives with cargo-dist, verifies checksums, and builds the amd64 and
arm64 images. It exports an OCI archive on the runner without publishing it. The same
archive staging script runs in the release workflow.

Verify an artifact with `gh attestation verify <file> --repo <owner>/seikika`.

## Verify public container access

The image workflow pushes and attests the release tag. GHCR initially makes a new image
package private, even when the source repository is public. To enable anonymous pulls,
approve the visibility change with the organization owner and review the package's existing
versions. Making the package public exposes its published versions and is permanent.

1. If **Public** is disabled in the package settings, an organization owner can allow it
   under **Package Creation** in the organization settings. This permission applies to the
   organization. [Seikika organization package settings](https://github.com/organizations/seikika/settings/packages)
2. In the `seikika` container's **Package settings**, open **Danger Zone → Change visibility**,
   select **Public**, and confirm. The existing release tag then becomes publicly pullable.
   [Seikika container package settings](https://github.com/orgs/seikika/packages/container/seikika/settings)
3. Run `docker manifest inspect` with an empty Docker config. Check that the index lists
   `linux/amd64` and `linux/arm64`.

   ```bash
   DOCKER_CONFIG="$(mktemp -d)" docker manifest inspect ghcr.io/seikika/seikika:0.2.1
   ```

When the organization keeps the package private, give approved teams and Actions repositories
read access. For Docker outside Actions, use a classic personal access token with `read:packages`
from an account that can read the package. [Authenticate to GHCR](https://docs.github.com/en/packages/working-with-a-github-packages-registry/working-with-the-container-registry#authenticating-to-the-container-registry)

Public operators can [install the CLI and connectors with mise](#install-what-a-consumer-needs)
or build the image locally from the release archives.

### Build a release image locally

Use Bash with Git, an authenticated GitHub CLI, `sha256sum`, and `tar` with xz support.
Use Docker Buildx with Linux containers on amd64 or arm64. The release's staging script
downloads and verifies the public release binaries. The tagged Dockerfile assembles them
into a local image.

```bash
tag=v0.2.1
work=$(mktemp -d)
git clone --depth 1 --branch "$tag" https://github.com/seikika/seikika.git "$work/source"
GITHUB_REPOSITORY=seikika/seikika bash "$work/source/scripts/stage-image-archives.sh" \
  "$work/archives" "$work/source/image" "$tag"
docker buildx build --load --tag "seikika:${tag#v}" "$work/source/image"
docker run --rm --network none "seikika:${tag#v}" --version
```

Use `seikika:0.2.1` as the local base image for your portal.

## Rust crates on crates.io

The workspace publishes the CLI, connectors, SDK, and their Rust libraries in lockstep.
The version applicator updates exact workspace
dependency pins along with the workspace version. Refresh `Cargo.lock` with
`cargo update --workspace` when preparing a version by hand.

Verify the archives and build the standalone connector scaffold against the packaged SDK
from the release commit. CI and the publishing workflow both run these checks.

```bash
cargo package --workspace --locked
python3 scripts/check-crates-release.py sdk target/package
```

For new crate names, use an authenticated crates.io account with permission to publish new
crates. Publish from the verified release commit, then check each version in the registry.

```bash
cargo publish --workspace --locked
```

After the first publish, register a GitHub Actions trusted publisher for each public crate.
Use repository owner and name `seikika/seikika`, workflow filename `publish-crates.yml`,
and environment `crates-io-publish`. In GitHub, restrict that environment to the `main`
branch. Run **Publish Rust crates** from `main` with `publish=true` to publish with OIDC.
The default dispatch verifies the archives and downstream SDK without publishing.

Wait for the publish run's **registry-smoke** jobs to pass before tagging the release.
They install the exact published version with its lockfiles in fresh Cargo directories on
all five release targets. They validate the reference portal, check workspace connector
conformance, and start the other installed connectors. Linux x86-64 also runs the portal's
authorization suite and the GitHub, git-files, and SCIM integration smoke test. A successful
upload records GitHub OIDC in the publish job summary. A rerun of an already published
version repeats the smoke tests without uploading another version.

**Crates.io smoke tests** also runs every Monday and on manual dispatch. It resolves the
latest stable, non-yanked CLI release once, then tests that exact version across all release
targets with both locked and fresh dependency resolution. Check the failed platform and
resolution mode in Actions if the scheduled run fails.

Use the pinned Rust toolchain and Python 3.13 to reproduce the portable checks locally.

```bash
python3 scripts/check-crates-release.py install 0.2.1
python3 scripts/check-crates-release.py install 0.2.1 --resolution fresh
```

The GitHub Release and installer artifacts are built by cargo-dist from the version tag.
See [the crates.io decision](decisions/D01M8BEC7ZX7TFBTSC47W1XQVV4.md) and the
[registry's Trusted Publishing documentation](https://crates.io/docs/trusted-publishing).

## Docs and GitHub Pages

The docs build uses `https://seikika.com/` as its canonical URL. In the
`seikika/seikika` repository's Pages settings, select **GitHub Actions**, set
`seikika.com` as the custom domain, and enforce HTTPS once GitHub issues the certificate.
Verify the domain in the organization's Pages settings and point its apex DNS records
at GitHub Pages. See the [GitHub Pages custom-domain instructions](https://docs.github.com/en/pages/configuring-a-custom-domain-for-your-github-pages-site/managing-a-custom-domain-for-your-github-pages-site).

## Public frontend packages

The [npm package catalog](../site/src/content/docs/authors/npm.mdx) lists the two packages
distributed from the PNPM workspace. Their versions live in their package manifests.
The frontend CI job checks the catalog and packs the same files a registry release ships.

On `main`, run **Publish npm packages** with `publish=true` to release versions listed in
the two manifests. The workflow checks and packs the workspace with pnpm and tests both
tarballs in a clean Astro project. It publishes the UI before the Astro integration using
npm Trusted Publishing and skips versions already in the registry. Wait for the
**registry-smoke** job to pass. It installs the exact versions from npm, checks their
signed provenance against this repository's publish workflow, and builds an Astro report
and a page using the UI exports. Dispatch with `publish=false` to run the tarball checks
and download the `npm-packages` artifact. See [the publishing decision](decisions/D01M8BBPE4KN9NF9E9K3F9ZEE4Q.md).

For the first version of a new package name, publish the tarball from the pack job with
`npm publish <tarball> --access public --ignore-scripts` while signed in as an npm maintainer.
Publish `@seikika/ui` before `@seikika/astro-report`. For each package, add a GitHub Actions
trusted publisher with organization `seikika`, repository `seikika`, workflow filename
`publish-npm.yml`, and environment `npm-publish`. Enable direct publishing. With npm's CLI,
run the following for each package while signed in as its maintainer.

```bash
npm trust github @seikika/ui --repo seikika/seikika --file publish-npm.yml --env npm-publish --allow-publish
npm trust github @seikika/astro-report --repo seikika/seikika --file publish-npm.yml --env npm-publish --allow-publish
```

**npm registry smoke tests** runs every Monday and on manual dispatch against both
packages' `latest` tags. It uses a fresh install to detect registry, dependency, and build
regressions. Use Node 24 and npm 11 to repeat the published package check locally.

```sh
node scripts/check-npm-release.mjs registry latest latest
```

The `npm-publish` environment permits only the `main` branch. Publish a new version through
the workflow within two days after adding each trusted publisher, then verify the package
version and provenance in the npm registry. [npm trusted publishing documentation](https://docs.npmjs.com/trusted-publishers/)

## Install what a consumer needs

Mise installs each app from the platform-specific cargo-dist archive. An operator installs the
CLI and the `seikika-connector-<name>` binaries named by the portal's `builtin:<name>` packages.
The [install guide](../site/src/content/docs/start/install.md) shows the CLI and optional connector
entries. The [starter config](../examples/repo-portal-starter/mise.example.toml) pins the CLI and workspace
connector together.

When updating the recommended operator version, update both starter pins and the matching
examples in the public docs. After cargo-dist announces a stable release, it calls the
[mise workflow](../.github/workflows/mise.yml) for that tag. The workflow installs the CLI and
workspace connector on Linux using the starter config, checks connector discovery and the
report baseline, and resolves the release assets for all five targets with `mise lock`.
The weekly workflow run repeats the install for the version pinned in the starter.

To repeat the check locally for a published tag, run the following from the repository root.

```sh
mise exec -- python3 scripts/check-mise.py release v0.2.1
```

`scripts/install-release.sh` unpacks `seikika`, `seikika-connector-workspace`
and `seikika-connector-runner` into one directory after checking each
`.sha256`, for a checkout that already exists (the same layout as
`actions/report/install.sh`).

In CI, set `seikika-version: "0.2.1"` on `seikika/seikika/actions/report` so the action
downloads checksum-verified binaries instead of compiling.

## Public vs private (required for external consumers)

`uses: seikika/seikika/...` and unauthenticated release downloads only work when this
repository is **public**. A consumer's default `GITHUB_TOKEN` cannot resolve a private action
repo (`repository not found`) and cannot download private release assets.

**Human step (preferred for RTBuddy and other external customers):** GitHub →
`seikika/seikika` → **Settings → General → Danger Zone → Change repository visibility → Public**.

**Workaround if it must stay private:** each consumer repository needs a PAT (or fine-scoped
GitHub App token) with `contents: read` on `seikika/seikika`, stored as a secret
(e.g. `SEIKIKA_READ_TOKEN`), then:

```yaml
- uses: actions/checkout@v4
- uses: seikika/seikika/actions/report@v0.2.1
  with:
    seikika-version: "0.2.1"
  env:
    GH_TOKEN: ${{ secrets.SEIKIKA_READ_TOKEN }}   # install.sh / gh release download
```

Also grant that token (or add the consumer as a collaborator) so GitHub can resolve the `uses:`
line — otherwise the workflow fails before any step runs.

## After the first tag

- Point consumers at the tag, not at `cargo build`.
- Homebrew and aqua registry entries for the CLI are still deferred.
- cargo-binstall metadata lives on each distable crate's `Cargo.toml`.


## Assessment preflight

Build the current checkout and run the freshness gate with the same connector and tool setup used
in CI. Then review missing independent assessments for the change with the release owner.

```bash
cargo build --locked
target/debug/seikika assess check portal --skip runner,github-actions --require-fresh
```

Resolve stale assessments and have a different reviewer review the refreshed judgments before
tagging. For a high-risk pull request, apply `require-fresh-assessments` to run the strict check
in CI. A label change triggers CI.
