# openssf-scorecard: a Seikika package

**Box or tool:** this package is a **kit**. It carries an existing standard and its tool, OpenSSF
Scorecard, as they are ([openssf-template.md](../../docs/design/openssf-template.md)). The checks,
their 0–10 scores, reasons, details and documentation are OpenSSF's. The box adds only:
- a ratchet: a check's gap to 10 is the measure, so a score that drops fails a pull request;
- a campaign to bring every check to 10, counted from a recorded start;
- the command that reproduces each check with Scorecard alone.

| Brings | Named |
|---|---|
| Connector | `<name>` (`builtin:runner`, `parse: scorecard`, `tracked: true`) |
| Tool pin | `scorecard` at `<version>` (`seikika doctor` reads `scorecard version`) |
| Kind | `<kind>`: `check`, `score` (null when inconclusive), `reason`, `details`, `documentation`, `aggregate`, `command` |
| Scorecard | `<name>`, one level `ten`, rule `ten`; new checks don't fail the gate (`newEntitiesAtLeast: none`) |
| Campaign | `<name>-ten` |

## Use it

**Minimum Seikika:** **0.2.0** supports `parse: scorecard` and array `with:` parameters. To
work with an unreleased package revision, use `seikika vendor` or a path include.

```yaml
include:
  - package: github:seikika/seikika/packages/openssf-scorecard@<tag>   # or path / seikika vendor
    with: {}
```

It runs `scorecard --local . --format json --show-details` in an export of the git-tracked files,
with their uncommitted edits, so `node_modules` or `__pycache__` aren't read as binary artifacts.
Results then match a fresh CI checkout. `--local` needs no GitHub token.

**Ratchet-safe `--local` checks:** Binary-Artifacts, Dangerous-Workflow, Dependency-Update-Tool,
Fuzzing, License, Pinned-Dependencies, SAST, Security-Policy, Token-Permissions. **Omit**
Packaging (often `-1` / inconclusive offline — blocks `--update` / `--ci`) and **Vulnerabilities**
(OSV at runtime; not reproducible). S4S “green” means those nine (or ten when Packaging scores) at
level `ten` — not Scorecard’s default aggregate including Vulnerabilities.

**Installing Scorecard:** from a seikika checkout that includes the pin,
`scripts/ci-tools.sh <dir> scorecard` downloads the release named in the package’s `tools:` entry
and checks its SHA-256. A **stale** consumer checkout of this repo (or of `actions/report`) that
predates the scorecard case in that script prints `unknown tool scorecard` — fetch / pin a SHA that
contains the pin before installing. Consumers who do not vendor the script can still install the
same version from Scorecard’s GitHub Releases and verify the published checksum.

**Consumer CI:** pin `uses: seikika/seikika/actions/report@<sha>`, **omit** `seikika-version`
so the action builds from that SHA; install Scorecard with `scripts/ci-tools.sh <dir> scorecard`
from the same SHA.

**Refreshing OpenSSF scores:** the kit’s connector is the **runner**. Do **not** pass
`--skip runner` when you need live `maintainer-openssf` / Scorecard facts. After recording those
scores, rewrite a CI baseline that skips the runner with the usual
`seikika report portal --skip runner,github-actions --baseline … --update` so crate clippy/test
facts are not left `unknown` under the skip.

**Why a fitting, not a slot:** Scorecard's SARIF output needs a policy file and `ENABLE_SARIF`
(experimental in 5.7.0), and a check scored 0–10 isn't a SARIF concept. Its probe format has no
scores. Its JSON is the tool's own stable output, and the fitting reads it without reinterpreting.

## Greening notes (operator)

**Security-Policy:** a minimal private-reporting stub often scores ~4. Ship a richer policy (linked
contact, free text, vulnerability / disclosure / N-day language). Use
[examples/SECURITY.md](examples/SECURITY.md) as a template that aims at ~10, and follow Scorecard’s
remediation in
[docs/checks.md#security-policy](https://github.com/ossf/scorecard/blob/main/docs/checks.md#security-policy).

**Fuzzing (Rust):** Scorecard’s language fuzz probes omit Rust. A Rust-only repo does not reach
Fuzzing=10 from `cargo fuzz` alone — use OSS-Fuzz or ClusterFuzzLite (or accept a lower score).

**License under `--local`:** root MIT body with `LICENSES/MIT.txt` can still score **9**; the
classifier may not award the FSF/OSI point offline. Scorecard 5.7.0’s local path can take SPDX from
a filename such as `LICENSE-MIT`. Prefer that (or the GitHub License API path without `--local`)
when chasing ten.

**Token-Permissions + cargo-dist:** generated `release.yml` often uses top-level `contents: write`
(score 0). Escape hatch ([cargo-dist#2320](https://github.com/axodotdev/cargo-dist/issues/2320)):
top-level `contents: read`, job-level write on plan/host/announce, and
`allow-dirty = ["ci"]` in `dist-workspace.toml` so `dist generate` keeps the hand patch.

**Pinned-Dependencies:** Scorecard treats `curl … | python` / `curl … | sh` as
`downloadThenRun` even for small helpers. Write the download to a file, then interpret
(`curl -o file; python3 … < file`, or use `jq`). Actions SHA pins alone do not clear pip /
installer / Dockerfile probe classes.

**npm:** OpenSSF Pinned-Dependencies and Seikika’s package scorecard both want lockfile-faithful
CI: `npm ci` (or `pnpm install --frozen-lockfile`). The workspace fact `ciUsesInstall: true` means
an **unlocked** install (`npm install`) was detected — keep that name; do not invert the signal.

**Vulnerabilities (Cargo):** Scorecard’s count can include aliases and other ecosystems and may
differ from a local `cargo-audit` or `osv-scanner` run. Before planning bumps, re-scan with
`osv-scanner -L Cargo.lock`. Treat **unmaintained** advisories with no fixed version (for example
`chrono` / `chrono-tz` pulled in via `cel`) separately from upgradeable CVEs — there may be no
upgrade path in-tree. After a **sqlx** major that clears a sqlite CVE, still run
`cargo update -p libsqlite3-sys` (Dependabot may leave it behind); expect small call-site edits for
`SqlSafeStr` / `AssertSqlSafe` on `include_str` migrations, not a lockfile-only PR.

## Parameters

| Parameter | Default | Meaning |
|---|---|---|
| `name`, `kind` | `openssf`, `OpenSSFCheck` | The connector, scorecard and campaign name; the entity kind |
| `version` | `5.7.0` | The Scorecard version pinned under `tools:` |
| `command` | `[scorecard, --local, ., --format, json, --show-details]` | Scorecard's own command |
| `title`, `owner` | `OpenSSF Scorecard`, `group:maintainers` | The scorecard's heading and who owns it in this portal |

## Sources

- `ossf/scorecard` v5.7.0: `docs/checks.md`, `options/options.go`
- [docs/reference/runner-connector.md](../../docs/reference/runner-connector.md): the `scorecard` fitting and `tracked`
