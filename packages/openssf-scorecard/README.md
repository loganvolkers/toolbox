# openssf-scorecard: a Toolbox package

**Box or tool:** this package is a **kit**. It carries an existing standard and its tool, OpenSSF
Scorecard, as they are ([openssf-template.md](../../docs/design/openssf-template.md)). The checks,
their 0–10 scores, reasons, details and documentation are OpenSSF's. The box adds only:
- a ratchet: a check's gap to 10 is the measure, so a score that drops fails a pull request;
- a campaign to bring every check to 10, counted from a recorded start;
- the command that reproduces each check with Scorecard alone.

| Brings | Named |
|---|---|
| Connector | `<name>` (`builtin:runner`, `parse: scorecard`, `tracked: true`) |
| Tool pin | `scorecard` at `<version>` (`toolbox doctor` reads `scorecard version`) |
| Kind | `<kind>`: `check`, `score` (null when inconclusive), `reason`, `details`, `documentation`, `aggregate`, `command` |
| Scorecard | `<name>`, one level `ten`, rule `ten`; new checks don't fail the gate (`newEntitiesAtLeast: none`) |
| Campaign | `<name>-ten` |

## Use it

```yaml
include:
  - package: github:loganvolkers/toolbox/packages/openssf-scorecard@<tag>   # or a path, or `toolbox vendor`
    with: {}
```

It runs `scorecard --local . --format json --show-details` in an export of the git-tracked files,
with their uncommitted edits, so `node_modules` or `__pycache__` aren't read as binary artifacts.
Results then match a fresh CI checkout. `--local` needs no GitHub token. It runs the checks
Scorecard supports on a directory: Binary-Artifacts, Dangerous-Workflow, Dependency-Update-Tool,
Fuzzing, License, Packaging, Pinned-Dependencies, SAST, Security-Policy, Token-Permissions and
Vulnerabilities. Choose checks with Scorecard's own flag in `command`, e.g. `--checks=SAST,License`.

**Vulnerabilities and the ratchet:** Scorecard's Vulnerabilities check asks `api.osv.dev` at run
time. Its score changes when a new advisory is published, with no change to the repository, so a
baseline made one day can fail the next. When OSV is unreachable, 5.7.0 reports 10 ("0 existing
vulnerabilities detected"), exits 0 and prints nothing on stderr: an error that reads as a pass. A
portal that ratchets should leave it out with `--checks` and carry vulnerabilities through a tool
pinned to an advisory database snapshot. Toolbox for Toolbox does this.

**Installing Scorecard:** `scripts/ci-tools.sh <dir> scorecard` downloads the v5.7.0 release and
checks its SHA-256 against Scorecard's published checksums.

**Why a fitting, not a slot:** Scorecard's SARIF output needs a policy file and `ENABLE_SARIF`
(experimental in 5.7.0), and a check scored 0–10 isn't a SARIF concept. Its probe format has no
scores. Its JSON is the tool's own stable output, and the fitting reads it without reinterpreting.

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
