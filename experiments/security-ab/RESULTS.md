# Security A/B: Results

Run 2026-11-22 on OpenFGA (R1) and Codex (R2), as pre-registered in
[README.md](./README.md). The raw outputs, both arms' code, costs and
judgements are in [data/](./data). This is about tools and standards,
not the box: Toolbox wasn't used in either arm.

## 1. Conclusion

**OpenSSF Scorecard doesn't find more than a home build; it finds less. Its value is everything else.**
Each home-built scorecard: - took one agent about 160k tokens; - was
right on every shared area but one, where it was partly right; Scorecard
was wrong on four of the thirteen shared verdicts; - found six or seven
confirmed problems per repository that Scorecard can't see.

Scorecard was better where a standard should be: - it costs nothing to
run again; - it means the same thing on every repository; - it runs
unchanged on a new repository; - it caught two things no home build
checked, Fuzzing and unpinned `pip install`.

The decisive number:
**reusing the home build on a second repository cost as much as building it**
(162k tokens against 161k). It ran unchanged but gave wrong verdicts
until an agent made 12 fixes.

| Hypothesis                                        | Result    | Evidence                                                                                                                                                                                   |
| ------------------------------------------------- | --------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| H1. On shared ground, OpenSSF is more often right | **Fails** | Wrong verdicts in shared areas: Scorecard 4 of 13, the home builds 0, plus 1 partly right (§3)                                                                                             |
| H2. A home build finds what OpenSSF can't         | **Holds** | 5 confirmed findings beyond Scorecard's checks on R1 and 6 on R2 (§4)                                                                                                                      |
| H3. A home build misses what OpenSSF catches      | **Holds** | Neither build checks Fuzzing (R2 has none, confirmed). None of the three versions caught `pip install` without hashes in two R2 workflows                                                  |
| H4. A home build doesn't reuse cleanly            | **Holds** | A1's scorecard ran on R2, but its secrets score (4.4) came from 20 test fixtures. Its reuse took 12 fixes (+161/−34 lines) and 162k tokens, and still misses the prompt injection A3 found |
| H5. Home builds don't converge                    | **Fails** | The areas overlap with Jaccard 0.76. **The scores don't converge**: on R2 the reused build gives 6.4 and the fresh build 5.6, and one area differs by 4.4 points                           |

**So "better" splits in two.** For one review of one repository, an
agent working from scratch does better than the standard. For a
portfolio of repositories, measured over time, the standard wins: its
numbers are free to rerun, compare across repositories, and support a
ratchet.

The box A/B reached the same conclusion, one level down
([../box-ab](../box-ab/README.md) §0): the value of reuse lies in
repetition, not in a single run.

## 2. Cost

|                        | Tokens  | Tool calls | Time       | Output                                                                       |
| ---------------------- | ------- | ---------- | ---------- | ---------------------------------------------------------------------------- |
| A1: build, on R1       | 161,017 | 26         | 780 s      | 11 checks, 922 lines of Python plus 20 semgrep rules and a fixture test; 7.9 |
| A3: build, on R2       | 162,582 | 31         | 765 s      | 16 checks, 964 lines plus 18 semgrep rules and a self-test; 5.6              |
| A2: reuse A1's on R2   | 162,519 | 29         | 519 s      | 12 fixes, +161/−34 lines; 5.1 → 6.4. Rerun on R1, every score is unchanged   |
| B: Scorecard on R1, R2 | 0       | 0          | 24 s, 82 s | 11 checks; 9.1 and 7.2                                                       |

Both build agents were blocked by the harness from writing `REPORT.md`
("subagents should return findings as text"). Their reports were saved
verbatim from their hand-backs ([data/](./data)).

## 3. The shared areas

Scorecard's 11 `--local` checks, judged against the files. The truth
column uses one definition per area for both arms: is there a real
weakness of this kind? Where Scorecard's own definition is narrower, the
table says so.

| Area                   | R1 truth                                                                                     | B                       | A1     | R2 truth                                                                          | B                       | A3                 | A1 reused (A2)           |
| ---------------------- | -------------------------------------------------------------------------------------------- | ----------------------- | ------ | --------------------------------------------------------------------------------- | ----------------------- | ------------------ | ------------------------ |
| Binary-Artifacts       | pass                                                                                         | ✓                       | —      | pass                                                                              | ✓                       | —                  | —                        |
| Dangerous-Workflow     | **fail**: any commenter triggers a third-party workflow at `@main` with write and `id-token` | ✗ (✓ by its definition) | ✓      | **fail**: issue title and body go into an agent prompt, `allow-users: "*"`        | ✗ (✓ by its definition) | ✓                  | partly: permissions only |
| Dependency-Update-Tool | pass, one dead entry                                                                         | ✓                       | ✓      | **fail**: the bun and docker entries point at nothing; npm and pip aren't covered | ✗                       | ✓                  | ✓                        |
| Fuzzing                | pass                                                                                         | ✓                       | —      | **fail**: no fuzzers                                                              | ✓                       | —                  | —                        |
| License                | pass (Apache-2.0)                                                                            | ✗ "not OSI"             | —      | pass (Apache-2.0)                                                                 | ✗ "not OSI"             | —                  | —                        |
| Packaging              | pass                                                                                         | ✓                       | —      | pass                                                                              | ✓                       | —                  | —                        |
| Pinned-Dependencies    | fail: `@main`                                                                                | ✓                       | ✓      | fail: images, curl-pipe-sh, pip                                                   | ✓                       | partly: misses pip | partly: misses pip       |
| SAST                   | pass                                                                                         | ✓                       | ✓      | fail                                                                              | ✓                       | ✓                  | ✓                        |
| Security-Policy        | fail (in-repo)                                                                               | ✓                       | ✓      | pass                                                                              | ✓                       | ✓                  | ✓                        |
| Token-Permissions      | fail: write on a comment trigger                                                             | ✗ (warns, scores 10)    | ✓      | fail                                                                              | ✓                       | ✓                  | ✓                        |
| Vulnerabilities        | unknown offline                                                                              | **fails open**: 10      | ✓ null | unknown                                                                           | **fails open**: 10      | —                  | ✓ null                   |

**Shared verdicts** are the areas both arms check: 7 on R1 (A1 also
reports Vulnerabilities, as null) and 6 on R2, which makes 13. -
Scorecard was wrong on 4 of them, each because its definition stops at
the presence of a tool or pattern: - Dangerous-Workflow reads `run:` but
not an action's `with:` inputs, and ignores a
    third-party reusable workflow on a comment trigger;
  - Dependency-Update-Tool is satisfied by a config file whose entries
    update nothing;
  - Token-Permissions warns about a write scope but doesn't deduct.
- The home builds were wrong on none, and partly right on one (pip
  pinning).
- Outside the shared areas, Scorecard was also wrong on License twice:
  `--local` can't match an Apache-2.0 file.
- Vulnerabilities reads 10 when `api.osv.dev` is unreachable. The home
  builds said "null" honestly
  ([../../docs/research-log.md](../../docs/research-log.md)).

**Areas neither home build checked:** Binary-Artifacts, Fuzzing, License
and Packaging. These are Scorecard's signals of a well-run open-source
project. Both agents judged security by attack paths instead, and never
mentioned OpenSSF as a model. A1 counts the repository's own
`ossf/scorecard-action` as one CI tool.

## 4. Beyond Scorecard

Confirmed by reading the files
([data/beyond-scorecard.tsv](./data/beyond-scorecard.tsv)):

| Repo | Finding                                                                                              | Arm         |
| ---- | ---------------------------------------------------------------------------------------------------- | ----------- |
| R1   | Four `http.Server`s without `ReadHeaderTimeout` (`cmd/run/run.go:784, 898, 1012, 1033`)              | A1          |
| R1   | 21 `actions/checkout` steps keep credentials (artipacked)                                            | A1 (zizmor) |
| R1   | No secret scanner in CI                                                                              | A1          |
| R1   | Dependabot `gomod /tools` points at a directory that doesn't exist                                   | A1          |
| R1   | The `@main` comment-triggered workflow above, as a dangerous workflow                                | A1          |
| R2   | Prompt injection into `openai/codex-action` (`issue-labeler.yml:25, 85, 88`)                         | A3          |
| R2   | `codex-rs/.github/workflows/cargo-audit.yml` never runs: GitHub ignores nested `.github` directories | A3          |
| R2   | 12 `Cargo.lock` entries from git, in 7 forked repositories                                           | A3          |
| R2   | 501 of 548 non-test `unsafe` sites have no SAFETY comment                                            | A3          |
| R2   | CODEOWNERS doesn't cover `.github/workflows`                                                         | A3, A2      |
| R2   | No secret scanner in CI                                                                              | A3, A2      |

**Precision:** - **A1 on R1:** no check's score rests on a false
finding. Its secrets hits are dev defaults (`postgres:password@`),
labelled as such. `InsecureSkipVerify` on a unix socket is justified in
the code. - **A1 reused unchanged on R2:** three checks rest mostly on
false findings. - secrets: 20 of 20 hits are fixtures in Rust
`#[cfg(test)]` modules; - static analysis: 45 of 64 findings are ruff
S603/S607 notes on helper scripts; - container hardening: it treated
dev-container images as shipped.

A2 fixed all three. - **A3 on R2:** clean. It labelled test hits as test
code. - **B:** two false checks per repository: License, and
Vulnerabilities, which fails open.

## 5. The baseline: the OpenSSF delta

The measures from [README.md](./README.md) §6, with Scorecard as the
reference standard *S*:

| Measure                                                              | A1 on R1              | A3 on R2                        | A1 reused on R2 (A2)                                   | *S* (reference)                    |
| -------------------------------------------------------------------- | --------------------- | ------------------------------- | ------------------------------------------------------ | ---------------------------------- |
| Recall of *S* (areas where *S*'s confirmed findings lower its score) | 2 of 2                | 3 of 4 (no Fuzzing; pip partly) | 3 of 4                                                 | 1.0                                |
| Confirmed findings beyond *S* (§3 misses + §4)                       | 6                     | 7                               | 3                                                      | 0                                  |
| Checks whose score rests on false findings                           | 0 of 11               | 0 of 16                         | 3 of 11 before the fixes, about 1 after                | 2 of 11 (License, Vulnerabilities) |
| Build cost                                                           | 161k tokens, 780 s    | 163k tokens, 765 s              | —                                                      | 0                                  |
| Reuse cost on a second repository                                    | —                     | —                               | 162k tokens, 12 fixes, +161/−34 lines; 2 misses remain | 0                                  |
| Convergence with another build                                       | Jaccard 0.76 on areas | (same)                          | Scores differ from A3 by up to 4.4 points per area     | 1.0                                |

**How to use it.** For any domain (observability, quality, architecture
conformance), run the same three phases: an agent's build, its reuse on
a second repository, and a second build. Then compare with the reference
standard on these six measures. The security numbers set expectations: -
**A good reusable standard doesn't win on finding power.** An agent's
build beats it by about six confirmed findings per repository. -
**It wins on reuse cost and convergence.** These measures predict
whether a scorecard's numbers can be compared across repositories and
ratcheted over time. -
**A domain standard Toolbox carries should be at least as good as *S* on reuse cost and convergence.**
It should also beat *S* on recall of the agent's findings, for example
by letting a repository add its own rules on top.

**What this means for Toolbox.** The case for a standard in the box
isn't that it finds more. It is that it costs nothing to repeat and
gives the same answer everywhere. The obvious next step is a
combination: *S* as the floor, plus the home build's extras kept as
local rules beside it. The agent's six findings per repository then join
a ratchet instead of being lost. That is the "A + S" arm in
[README.md](./README.md) §7, and it is where the box can show value over
both arms.

## 6. Limits

- One run per phase. Two repositories, both with mature security
  tooling.
- The adjudicator is this session's agent, which designed the
  experiment. Every judgement is in
  [data/adjudication.tsv](./data/adjudication.tsv) with its evidence, so
  it can be checked.
- The truth columns use one definition per area. Three of Scorecard's
  four wrong verdicts are right under its own, narrower definitions,
  which it documents. The disagreement is about what to check, not about
  a bug.
- No network: Vulnerabilities and Scorecard's API checks are out, and Go
  1.25 analysis (`govulncheck`) couldn't run.
- `unsafe` without a SAFETY comment is counted as a finding, but whether
  it is a security weakness is a judgement.
