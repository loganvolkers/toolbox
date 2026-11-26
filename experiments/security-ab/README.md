# The Security A/B: a Home-Built Scorecard or OpenSSF Scorecard

**Question:** is a reusable standard (OpenSSF Scorecard) better than
asking an agent to build a security scorecard from scratch? The box A/B
([../box-ab](../box-ab/README.md)) found that Seikika added little over
an agent working alone. This tests the same assumption one level up, on
the standard rather than the box: if an agent can home-build a scorecard
as good as OpenSSF's, reusable standards (and a box that carries them)
matter less than we think.

**Status: run 2026-11-22.** This file was written before any arm ran,
and is unchanged since except for this line. Results are in
[RESULTS.md](./RESULTS.md).

This is about tools and standards, not the box
([seikika-and-tools.md](../../docs/design/seikika-and-tools.md)): both
arms run without Seikika. Comparing native Scorecard with Scorecard
through the Seikika kit is a later experiment (§7).

## 1. Repositories

|     | Repository             | Commit                  | Files | Workflows | Language         |
| --- | ---------------------- | ----------------------- | ----- | --------- | ---------------- |
| R1  | `loganvolkers/openfga` | `7d89a566` (2026-06-02) | 559   | 11        | Go               |
| R2  | `loganvolkers/codex`   | `859dbe27` (2026-06-02) | 4,771 | 24        | Rust, TypeScript |

Both arms read an export of the tracked files (`git archive HEAD`), so
neither sees `.git` history. No network: package registries and APIs are
blocked for both.

## 2. Arms

|       | A: home-built                                                                                                         | B: OpenSSF Scorecard                                        |
| ----- | --------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------- |
| Who   | A fresh general-purpose agent per phase, on the same model                                                            | No agent                                                    |
| Given | The export, and the tools below on PATH                                                                               | The same export                                             |
| Does  | Designs and writes a scorecard program, runs it, reports                                                              | `scorecard --local . --format json --show-details` (v5.7.0) |
| Tools | zizmor 1.32.0, actionlint 1.7.12, semgrep 1.163.0 (no registry rules: no network), ruff, go, cargo, node, python3, jq | Scorecard alone                                             |

The arm A prompt doesn't mention OpenSSF, Scorecard or any check list
([prompts.md](./prompts.md)). Whether the agent reaches for OpenSSF's
checks on its own is recorded: agents are trained on it, and copying it
is a realistic way to home-build.

**Phases:**

| Phase | Arm A                                                                                               | Arm B                |
| ----- | --------------------------------------------------------------------------------------------------- | -------------------- |
| A1    | Build a scorecard and run it on R1                                                                  | Run on R1            |
| A2    | **Reuse:** a fresh agent runs A1's scorecard on R2 and fixes what breaks, without special-casing R2 | Run on R2, unchanged |
| A3    | **Second build:** a fresh agent builds a scorecard from scratch on R2                               | —                    |

A3 measures whether two independent home builds converge, which a
standard does by construction.

## 3. Scoring

1. **Taxonomy.** Every check from either arm is mapped to a security
   area. The areas are Scorecard's 11 `--local` checks, plus any others
   the arms raise (such as secrets, workflow injection, container user,
   or unsafe code).
2. **Claims.** Each check's verdict, and each finding it names (an area
   and a file), becomes a claim.
3. **Adjudication.** Each claim is checked against the repository, by
   reading the file it names or by a deterministic command, and judged
   true, false or unverifiable. The claims are pooled and judged without
   regard to which arm made them. Every judgement is recorded in
   `adjudication.tsv` with its evidence.
4. **Measures per repository:**
   - areas covered;
   - confirmed findings;
   - false findings;
   - findings unique to each arm;
   - agreement on the areas both cover;
   - agent tokens, tool calls and time;
   - for A2, lines changed to reuse the scorecard;
   - for A1 against A3, overlap of the areas covered (Jaccard).

## 4. Hypotheses and the bar, set before running

| #   | Hypothesis                                        | Holds if                                                                                 | Fails if                                                                         |
| --- | ------------------------------------------------- | ---------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------- |
| H1  | **On shared ground, OpenSSF is more often right** | In the areas both arms cover, B has no more wrong verdicts than A, summed over R1 and R2 | A has fewer wrong verdicts: a home build matches the standard where they overlap |
| H2  | **A home build finds what OpenSSF can't**         | A has at least one confirmed finding outside Scorecard's 11 checks on each repository    | It has none: OpenSSF's coverage is enough                                        |
| H3  | **A home build misses what OpenSSF catches**      | A misses at least one area where B's confirmed findings lower its score                  | A catches all of them                                                            |
| H4  | **A home build doesn't reuse cleanly**            | A2 changes the scorecard, or it gives at least one wrong verdict on R2                   | It runs unchanged on R2 and is right                                             |
| H5  | **Home builds don't converge**                    | A1 and A3 overlap on fewer than half their areas (Jaccard < 0.5)                         | They converge (≥ 0.5): agents already share a de facto standard                  |

**What "better" means here.** OpenSSF is better than home-building if
H1, H3 and H4 hold. If H2 also holds, a standard is a floor, not a
ceiling. Then the useful baseline is the difference between the two
arms: what a home build adds beyond the standard, and what it drops
(§6).

## 5. Known limits, stated before running

- **Vulnerabilities fails open here.** api.osv.dev is unreachable, and
  Scorecard then scores 10 ("0 existing vulnerabilities")
  ([../../docs/research-log.md](../../docs/research-log.md)). Its
  verdict is judged like any other. Arm A has no vulnerability database
  either.
- **`--local` is a subset.** Checks that need the GitHub API
  (Branch-Protection, Code-Review, Maintained, Signed-Releases,
  CII-Best-Practices, CI-Tests, Contributors, Webhooks) don't run. Arm A
  sees the same files, so this is a fair comparison of what a
  repository's files show, not of Scorecard's full reach.
- **One run per phase.** Variance comes only from A1 against A3.
- **The adjudicator is this session's agent**, which also designed the
  experiment. Claims are judged by reading files or by commands, and
  recorded with that evidence so they can be checked.

## 6. The baseline this produces

For a home-built scorecard *X* against a reference standard *S* on
repository *R*:

| Measure | Meaning |
|---|---|
| **Recall of *S*** | Of the areas where *S*'s confirmed findings lower its score, the share *X* also flags |
| **Beyond *S*** | Confirmed findings of *X* in areas *S* doesn't check |
| **Precision** | Confirmed findings ÷ all of *X*'s findings |
| **Build cost** | Agent tokens, calls and time to build *X* |
| **Reuse cost** | Lines changed, and wrong verdicts, to run *X* on a second repository |
| **Convergence** | Jaccard of two independent builds' areas |

*S* scores 1.0 recall of itself, zero build cost, zero reuse cost and
full convergence. The same measures can be computed for any domain
standard Seikika carries (observability, quality, architecture) against
an agent's home build. That makes OpenSSF the reference point for what a
good reusable standard achieves.

## 7. Later

- **Native Scorecard vs the Seikika kit:** the same B on the same
  repositories, through `packages/openssf-scorecard` and
  `seikika report`. The kit should add the ratchet and the campaign
  without changing a score.
- **A + S:** an agent asked to extend Scorecard rather than replace it.
- **With network:** Vulnerabilities, and the API checks, on GitHub.

## Sources

- OpenSSF Scorecard v5.7.0 (`docs/checks.md`);
  [../../docs/design/openssf-template.md](
    ../../docs/design/openssf-template.md
  )
- [../box-ab](../box-ab/README.md): the lessons reused here (fresh
  agents per phase, a pre-registered bar, judging by evidence)
