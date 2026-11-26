# Installing the Box's Tools

*Status: researched and built, 2026-11-21 (D01M800DHD0Z1RTPH7CM46GZS3B, refining D90). About the
box ([seikika-and-tools.md](./seikika-and-tools.md)): pinning and installing the tools it carries.
It serves the operator SDLC (a portal's CI) and the package author SDLC (kits that declare their
tools).*

---

## 1. Conclusion

**Seikika owns the pin; installers install. Seikika still downloads nothing itself (D90), but it
is no longer tied to one installer.**
- **One pin.** Each tool has one `version` in `tools:`, plus an id for each installer it can be
  installed with: `install: {mise, aqua, pypi}`.
- **The installer's input, derived from the pin.** `seikika tools config --for mise|aqua|pypi`
  prints mise's `[tools]` table, an aqua.yaml package list, or PyPI requirements.
- **Running the installer.** `seikika tools install --via auto|mise|uv|pipx|pip` runs the chosen
  installer for every tool not already at its pin. `seikika doctor` then checks each pin.
- **mise is the default recommendation, not a requirement.** It covers the most tools through
  the most backends, verifies the most, and has a lockfile and a CI action. aqua is the
  alternative with the same registry and verification. uv, pipx and pip cover tools published on
  PyPI with no extra installer.

What changed: the pins used to be repeated in four places in this repository (`tools:`, `ci.yml`,
`pages.yml`, the site-report script). Now CI and the script install zizmor and ruff from
`tools:`, so a version bump is one edit. actionlint, which isn't on PyPI, is still installed by
`scripts/ci-tools.sh` from a SHA-256-pinned release. `seikika doctor` fails CI if its version
drifts from the pin.

## 2. Why not have Seikika download tools

D90 rejected it, and the research confirms that. Installing well means:
- per-platform asset templates;
- checksum files;
- GitHub artifact attestations, Cosign signatures and SLSA provenance;
- a registry kept current for thousands of tools.

mise and aqua already do all of it (§3). A downloader inside Seikika would be a worse copy, and
the sprawl the budget forbids (D45). A box holds tools; it doesn't manufacture the shelves.

## 3. The installers, from their sources

Read from clones of each project at the dates shown. Version numbers are as released then.

| | mise | aqua | proto | asdf | DotSlash | uv / pipx / pip |
|---|---|---|---|---|---|---|
| Repository (date read) | `jdx/mise` 2026.11.12 (2026-11-21) | `aquaproj/aqua`, registry `aquaproj/aqua-registry` (2026-11-21) | `moonrepo/proto` 0.64.0 (2026-11-20) | `asdf-vm/asdf` (2026-11-10) | `facebook/dotslash` (2026-11-16) | Python's own installers |
| Licence | MIT | MIT | MIT | MIT | MIT or Apache-2.0 | Apache-2.0 / MIT (from memory, not re-read) |
| What it installs | Many backends: aqua, github, gitlab, http, pypi (uv or pipx; `pipx:` is the same backend), cargo, npm, go, gem, ubi, asdf and vfox plugins (`docs/dev-tools/backends/`) | Its registry: **2,392** packages (`pkgs/**/registry.yaml`), mostly GitHub releases | WASM and TOML plugins | Shell plugins, one per tool | One committed text file per executable, holding URLs and hashes per platform | Python packages, including tools published as wheels (ruff, zizmor) |
| Integrity | Checksums; GitHub artifact attestations, Cosign, SLSA and minisign when the recipe declares them (`docs/dev-tools/backends/aqua.md`) | Checksums (`aqua-checksums.json`); attestations, Cosign, SLSA per recipe | Checksum verification (`README.md`, `crates/pdk-api/src/api/checksum.rs`) | None in core; each plugin decides (its docs never mention checksums) | The hash in the file | Hash-pinning is opt-in (`pip --require-hashes`; from memory) |
| Lockfile | `mise.lock`: versions, URLs and checksums; `mise install --locked` (`docs/dev-tools/mise-lock.md`) | `aqua-checksums.json` | `.prototools` pins; lockfile not checked | `.tool-versions` pins only | The file is the lock | Requirements files |
| CI | `jdx/mise-action`, with a `sha256` input for mise itself and `--locked` added when a lockfile exists | `aquaproj/aqua-installer`, which checks its bootstrap download against embedded checksums | — | — | — | Preinstalled on most runners |
| The pins we need | actionlint `aqua:rhysd/actionlint`; zizmor and ruff `pipx:` | `rhysd/actionlint`, `zizmorcore/zizmor`, `astral-sh/ruff`, each with checksums or attestations in its recipe | Not checked | Plugins exist for some | Hand-written per tool | zizmor, ruff |

**What the comparison decides:**
- **mise** gives the widest reach and the strongest verification, through its aqua backend and
  lockfile, so it is the recommendation for people.
- **aqua** gives the same registry and verification in a narrower, CI-shaped tool, so it is the
  alternative for CI.
- **uv, pipx and pip** cover the PyPI tools with what runners already have.
- **asdf** leaves integrity to each plugin, so Seikika doesn't recommend it.
- **DotSlash** is a good model, a pinned and hashed file per tool, but one hand-maintained per
  tool.

## 4. How it works

```yaml
tools:
  actionlint:
    version: 1.7.12
    install: {mise: "aqua:rhysd/actionlint", aqua: rhysd/actionlint}
    versionArgs: [-version]
  zizmor:
    version: 1.32.0
    install: {pypi: zizmor, mise: "pipx:zizmor", aqua: zizmorcore/zizmor}
```

| Command | Does |
|---|---|
| `seikika tools install [portal] --via auto` | For each tool not at its pin: the first of mise, uv, pipx and pip that is on the `PATH` and that the tool declares an id for |
| `--via mise` / `uv` / `pipx` / `pip` | `mise install <id>@<version>`, `uv tool install <id>==<version>`, `pipx install …`, `python3 -m pip install …` |
| `--dry-run` | Print the commands without running them |
| `seikika tools config --for mise` | A `[tools]` table for `mise.toml` (the same as `seikika doctor --mise`); then `mise lock` adds checksums |
| `seikika tools config --for aqua` | An aqua.yaml package list; pin the registry release, then `aqua i` |
| `seikika tools config --for pypi` | Requirements lines for uv, pipx or pip |
| `seikika doctor` | Checks every tool's version against its pin; exit 1 on a mismatch or a missing tool |

**Checked here:** `tools install --via uv` installed ruff 0.14.0 from the pin into a scratch
directory, and the configs printed correctly for this repository's three tools. The mise
adapter is not verified hands-on: the session's permission policy refused downloading and running
mise. It follows mise's documented `mise install <tool>@<version>`.

## 5. Next

| # | Item |
|---|---|
| T1 | Install actionlint from the pin too: through mise or aqua in CI, or a `release` id holding the SHA-256 that `scripts/ci-tools.sh` pins today |
| T2 | Renovate: keep `tools:` pins current, as Renovate already does for `mise.toml` and `aqua.yaml` |
| T3 | Check `--via mise` in CI, where mise can run |

## Sources

- `jdx/mise`: `docs/dev-tools/mise-lock.md`, `docs/dev-tools/backends/aqua.md`, `github.md`, `pypi.md`, `index.md`, `docs/installing-mise.md`; `jdx/mise-action/action.yml`
- `aquaproj/aqua`: `README.md`, `pkg/checksum/`, `pkg/ghattestation/`; `aquaproj/aqua-registry`: `pkgs/rhysd/actionlint`, `pkgs/zizmorcore/zizmor`, `pkgs/astral-sh/ruff`; `aquaproj/aqua-installer`: `action.yaml`, `aqua-installer`
- `moonrepo/proto`: `README.md`, `crates/pdk-api/src/api/checksum.rs`
- `asdf-vm/asdf`: `docs/`
- `facebook/dotslash`: `README.md`
- [package-author-sdlc.md](./package-author-sdlc.md) §3.3: D90's options, refined here
- actionlint 1.7.12's `actionlint_1.7.12_checksums.txt`: the Linux amd64 hash matches `scripts/ci-tools.sh`
