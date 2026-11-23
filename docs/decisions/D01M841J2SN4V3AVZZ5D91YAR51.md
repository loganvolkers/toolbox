---
id: D01M841J2SN4V3AVZZ5D91YAR51
decided: 2026-11-23T11:10:00Z
---

# A ratchet check prints its verdict, and the baseline remembers which evidence it has seen

**`toolbox report --baseline` (and `--against`) without `-o` or `--out` is a gate. It now prints
only the verdict, on stdout, not every scorecard followed by the verdict on stderr. The baseline
also records a key for each evidence item of each failing rule, in an `evidence` section beside
`results`. When a rule's evidence grows, the verdict lists only the new items, marked `+`, and
counts the ones already in the baseline. It no longer shows the first ten items of the whole
list.** This closes root-causes C1 ([root-causes.md](../design/root-causes.md)), friction #4 of
[box-ab](../../experiments/box-ab/README.md), and the results spec's "fingerprints per finding"
row ([results-spec.md](../design/results-spec.md)).

1. **Why.** The [gate-repair-ab](../../experiments/gate-repair-ab/README.md) benchmark, round 1,
   timed agents repairing a pull request that the gate rejects. In every Toolbox run the first gate
   output was about 16k characters of report (every scorecard, old findings included), and the
   regressions came after it, on stderr. Every agent then reasoned about whether a finding already
   on `main` was new, because the grown list mixed old and new items without marking them. Two
   piped later runs through `grep` to find the verdict. The reviewdog arm read only its five new
   findings. The same friction was recorded before this benchmark, in box-ab (both arms asked for
   new findings to be marked) and on this repository's own ratchet.
2. **The gate's output.**

   | Invocation | stdout | stderr |
   |---|---|---|
   | `--baseline` or `--against`, no `-o`, no `--out` (a gate) | the verdict | warnings only |
   | with `-o FORMAT` or `--out FILE` | the report, as before | the verdict, as before |
   | no `--baseline` | the report | nothing new |

   The verdict goes on stdout because in a gate it is the output, and some agent harnesses record
   only stdout. A failing gate ends with one line saying how to see every scorecard. The GitHub
   Action passes `--out`, so its job summary and artifacts don't change. `--verdict FILE` doesn't
   change.
3. **Evidence keys.** A key is an FNV-1a hash of the item's text with each run of digits folded to
   `#`. A finding that only moved (its line or column) keeps its key. It is stable across
   platforms and Rust versions, and it is 8 hex characters long. Keys are recorded for rules counted by
   their evidence, not for rules with a `measure`. A grown list is shown in three parts:
   - new items, `+ …`;
   - for a key that is both old and new (one more of a finding already there, at another line),
     `+ N more like: … (M alike in all)`, because the key can't say which line is the new one;
   - `and K already in the baseline`.
4. **Keys explain; they don't judge.** The ratchet still compares counts. A difference in keys
   alone is never a regression and never makes the baseline stale, so `--ci` doesn't fail on an
   upgrade, and an old baseline without keys falls back to the whole list. `--update` writes the
   keys whenever they changed, even when nothing else did. Making "fixed one, broke another" a
   regression, as Betterer does ([ratchets.md](../design/ratchets.md) R1), is now possible but
   is not decided here.
5. **Cost.** This repository's baseline holds about 2,000 evidence items. Pretty-printed with one
   key per line, they took `portal/results.json` from 147 KB to 214 KB. The file is generated and
   never merged by hand (D95), so its size costs reviewers scrolling, not conflicts.
