# OpenSSF as the Template: Others' Opinionated Standards, Carried by the Box

*Status: built for security, 2026-11-22 (D01M82E93XAPNE0WFXEPJZ59WG4). About the box
([seikika-and-tools.md](./seikika-and-tools.md)): carrying someone else's standard and its tool as
they are, and what an author needs to make one like it. Package author SDLC (authors publish
standards); operator SDLC (a portal carries them). Bottom-up end
(D01M80Y0XEK069GD08VCCNF0WB8).*

---

## 1. Conclusion

**Seikika doesn't define scorecards of its own where an established standard exists. It carries
them, and it is a template for people who want to build the next one.**
- **OpenSSF Scorecard is the model.** It is an opinionated standard for one domain, software supply
  chain security. It has:
  - **probes**, true-or-false facts with evidence;
  - **checks**, 0–10 scores built from those probes, each with documentation and a risk level;
  - **one tool** that computes them anywhere: a CLI, a GitHub Action, an API, and `--local` with
    no token;
  - **catalogs**: its conformance mode judges the OSPS Baseline, and writes Gemara Evaluation Logs;
  - **governance:** an OpenSSF project with versioned releases, public check definitions and
    adopters.
- **The opportunity is every other domain.** Nobody has an OpenSSF Scorecard for observability,
  code quality, correctness, or conformance with an architecture ("our services are hexagonal",
  "every handler emits a span"). Seikika's job is to make building one cheap, and to carry it beside
  the others once it exists.
- **Built:** the [`openssf-scorecard` kit](../../packages/openssf-scorecard/README.md) carries
  Scorecard 5.7.0 unchanged. Its checks, scores and documentation stay OpenSSF's. The box adds:
  - a ratchet, so a check's score can't drop;
  - a campaign to bring every check to 10;
  - the reproduce command.

  Seikika for Seikika includes the kit as `maintainer-openssf`, and has retired its own versions
  of two of Scorecard's checks (§4).

## 2. What an opinionated standard is made of, in OpenSSF and in Seikika

| Part | OpenSSF Scorecard | Seikika's primitive | Gap |
|---|---|---|---|
| **Facts with evidence** | Probes (`def.yml`: motivation, implementation, outcomes; findings with locations) | Connector records and SARIF findings (the slot), entity fields | Distinct outcomes: probes have five, Seikika has `unknown` for three of them ([results-spec.md](./results-spec.md) §4) |
| **Judgements** | Checks: a 0–10 score from probes, with a risk level | Scorecard rules in CEL, with levels, measures and evidence | Scores are rules plus levels; a 0–10 scale is a measure, as the kit shows |
| **Documentation per judgement** | `docs/checks.md`: what, why, how to fix | Rule `fix` text, `-o standard` (STANDARD.md) | — |
| **One tool, many places** | CLI, GitHub Action, API, `--local` | `seikika report`, `actions/report`, `serve`, MCP | — |
| **Catalogs and conformance** | Conformance mode over the OSPS Baseline, as Gemara | Campaigns and tracks; standards packages | Mapping a rule to a catalog control ([results-spec.md](./results-spec.md) §4, `standard`) |
| **Ratchet over time** | None: a score is a snapshot (the API keeps history) | Baselines, the ratchet, campaigns from a recorded start, root causes | **The box's addition.** It applies to any standard it carries |
| **Distribution and versions** | Releases; checks change between releases | Packages pinned by version and digest (D89), tools pinned in `tools:` | — |
| **Governance** | An OpenSSF technical initiative, public RFCs and adopters | None yet | Out of scope: a standard's author governs it, and Seikika hosts it |

## 3. A standard for another domain, built the same way

The author of, say, *an observability scorecard* writes a package, not a tool inside Seikika:

| Step | In the package | Precedent here |
|---|---|---|
| 1. The facts | A tool, or an existing one, that emits records or SARIF: "every HTTP handler opens a span", "metric names follow the convention" | `seikika-connector-workspace` observability facts; semgrep rules through the SARIF slot |
| 2. The judgements | Rules over those facts, with levels, a measure and fix text | `portal/standards/observability.yaml` (`maintainer-observability`) |
| 3. Documentation | Each rule's fix, generated into STANDARD.md | `-o standard` |
| 4. Tests | Fixtures where each rule both passes and fails, and expectations | `packages/*/tests`, the meta-standards (`meta-rule`: decided on fixtures) |
| 5. Distribution | A versioned package that others `include` with parameters | `packages/sarif`, `packages/github-actions`, `packages/openssf-scorecard` |

**The box's meta-standards enforce the template's discipline on every author.** Adding this kit
failed this repository's ratchet until its kind, fields, rule and connector each had fixtures, tests and
descriptions (`meta-kind`, `meta-field`, `meta-rule`, `meta-connector`, D71). An author gets that
discipline for free.

## 4. What Seikika for Seikika stopped doing itself

| This portal's rule | OpenSSF check | Now |
|---|---|---|
| `maintainer-supply-chain` / `code-scanning` | SAST (10 here) | **Retired**; Scorecard carries it |
| `maintainer-supply-chain` / `security-policy` | Security-Policy (4 here: Scorecard also asks for linked content) | **Retired**; Scorecard's is stricter |
| `maintainer-supply-chain` / `code-scanning-pinned` | Pinned-Dependencies (4 here) | Kept until Pinned-Dependencies reaches 10, because the repository level is the lowest constituent and this rule holds the gold level |
| `maintainer-supply-chain` / `dependabot` | Dependency-Update-Tool (10 here) | Kept: it checks each ecosystem (cargo, npm, actions); Scorecard checks that a tool exists |
| `maintainer-supply-chain` / `workflows-have-owners` | None in `--local` (Code-Review needs the API) | Kept |
| `github-actions` kit / `pinned-to-sha`, zizmor | Pinned-Dependencies, Token-Permissions, Dangerous-Workflow | Kept in the kit, which pilots use for per-workflow evidence; noted as overlapping |

**This repository on Scorecard 5.7.0 (`--local`, tracked files):** aggregate 7.2. Binary-Artifacts,
Dangerous-Workflow, Dependency-Update-Tool, Packaging and SAST are at 10 (Vulnerabilities read 10 too, wrongly: see below), License
at 9, Pinned-Dependencies and Security-Policy at 4, Fuzzing and Token-Permissions at 0. The
campaign `maintainer-openssf-ten` starts there.

**Vulnerabilities is carried, but not ratcheted here.** On CI, with network access, it found 23
advisories against `Cargo.lock`. Here `api.osv.dev` is blocked, and Scorecard scored it 10, exited
0 and printed nothing: a tool error that reads as a pass. Its score also changes whenever an
advisory is published, with no change to the repository. A ratchet needs a result that reproduces
in every environment, so Seikika for Seikika runs the other ten checks with Scorecard's own
`--checks` flag. The kit's README warns other portals. The 23 advisories are real and open (O5).

**Advisories that reproduce (O5, D01M885GT32STM506V32XKPP37G).** `maintainer-advisories` scores
`Cargo.lock` with osv-scanner offline, against RustSec's OSV export at a commit pinned in
`scripts/ci-tools.sh`, through the SARIF slot. At that pin, two advisories remain, both
informational "unmaintained" notices for chrono and chrono-tz, which come in through cel. Both are
justified in `osv-scanner.toml`. The 23 that Scorecard counted were from an older `Cargo.lock`,
over the network, so the two counts can't be compared.

## 5. Is the template better than building from scratch?

[experiments/security-ab](../../experiments/security-ab/RESULTS.md) asked an agent to build a
security scorecard from scratch on two repositories, and compared it with Scorecard. The agent
found more:
- 6 to 7 confirmed problems per repository that Scorecard can't see;
- the right verdict in 4 places where Scorecard's checks stop at the presence of a tool or
  pattern.

Scorecard won on everything a standard is for:
- it costs nothing to run again (each build took about 160k tokens);
- it runs unchanged on a new repository (reusing the agent's build cost as much as building it);
- it gives one definition, so scores compare (two builds differed by up to 4.4 points in an area).

So the template's value is cost and comparability, not detection. A domain standard Seikika
carries should be judged on the same six measures (that experiment's §5). The way to keep the
agent's extra findings is local rules beside the standard, the A + S arm that comes next.

## 6. Next

| # | Item |
|---|---|
| O1 | Pay down Token-Permissions and Pinned-Dependencies, then retire `code-scanning-pinned` |
| O2 | Scorecard's conformance mode (OSPS Baseline) through the same kit: a catalog, not a score |
| O3 | Scorecard's probe format as facts, so a rule can name a probe and not only a check |
| O4 | A second domain standard, such as observability, published as a package by the template in §3, ideally by someone other than us (R5) |
| O5 | ✅ for Cargo.lock (§4, D01M885GT32STM506V32XKPP37G): osv-scanner offline against a pinned RustSec snapshot, with justifications in `osv-scanner.toml`. The npm lockfiles remain, and need a pinned GitHub advisory snapshot |
| O6 | The A + S arm of the security A/B: Scorecard as the floor plus an agent's extra checks as local rules, ratcheted together; then native Scorecard against the kit |

## Sources

- `ossf/scorecard` (2026-11-19): `docs/checks.md`, `docs/probes.md`, `docs/conformance.md`, `finding/finding.go`, `options/options.go` (`ENABLE_SARIF`)
- Scorecard v5.7.0 release (`scorecard_5.7.0_linux_amd64.tar.gz`, SHA-256 `3b08ec0a…`), run here with `--local` on an export of the tracked files
- [results-spec.md](./results-spec.md): the outcome model and catalogs
- [supply-chain-standards.md](./supply-chain-standards.md) (D85): the rules retired in §4
