# Root Causes: Every Regression Says Why

*Status: built, 2026-11-21 (D01M7ZZYZ1SXSTHDZQK8A1CZMMN). About the box
([toolbox-and-tools.md](./toolbox-and-tools.md)): what the box remembers about its tools, and how
it explains a change. It serves the operator SDLC (a gate that explains itself) and the user SDLC
(an agent fixing what failed).*

---

## 1. Conclusion

**A failing gate must say why, down to a file, a line, a rule and a command.** Toolbox's ratchet
compares each entity's scorecard results with a baseline (or another commit, `--against`). Every
regression it reports now carries a root-cause chain:

```
✗ worse   maintainer-repo · repo:loganvolkers/toolbox: constituents-at-least-bronze now fails
    constituents-at-least-bronze: maintainer-docs is none
  ↳ maintainer-docs · its lowest entity: fell silver → none
    ↳ maintainer-docs · doc:docs/design/zz-probe.md: new (none)
        names-sdlc-layer: 1 mention(s) of "SDLC" without a layer
        from: connector workspace, record doc doc:docs/design/zz-probe.md
```

A rule that reports new findings lists each one once, under its entity. A tool's finding ends at
the command that reproduces it (D01M84CCDH1ZWF9726SVDDM1JDX):

```
✗ worse   maintainer-python:scripts/take-it-out.py · maintainer-python
    no-findings: + line 87: F811 Redefinition of unused `os` from line 16
    reproduce: ruff check --no-cache --output-format sarif --exit-zero scripts/take-it-out.py
```

When several rules report the same text, the line names them all, for example
`no-high-findings, zizmor-clean: + high template-injection at line 40: …`.

The first example is this repository's own gate, on a probe doc. Before this, it printed only
its first line. The cause, a new doc below the floor,
appeared among the changes, as "· changed … new (none)", with nothing linking the two
([experiments/box-ab](../../experiments/box-ab/README.md) §1).

## 2. The chain

| Link | What it says | Where it comes from |
|---|---|---|
| Aggregate | The repository-level rule that failed, and its evidence | A scorecard that reads `cards` (D86) |
| Constituent | Which scorecard's level fell, from what to what | The lowest counted entity of that scorecard, before and now (fixture samples under `/demo/` and skipped entities don't count) |
| Change | The entity changes now at that lower level: a rule that started failing, evidence that grew, a new entity below the floor | The same comparison's changes, filtered to that scorecard and level |
| Evidence | What the rule found now: up to three items, then a count. When the evidence grew, only the new items, marked `+`, then how many were already in the baseline: the baseline keeps a key per item, its text with digits folded, so a finding that moved keeps its key (D01M841J2SN4V3AVZZ5D91YAR51). A baseline without keys shows up to ten items of the whole list. Using this repository's own ratchet, the new long function was hidden behind "… 1 more" until the ten-item rule | The rule's `evidence`, and the baseline's `evidence` keys |
| Reproduce | The command that gives the same finding with the tool alone, or the record the fact came from | The entity's `command` field (lint records, take-it-out), else its source record |

When a scorecard fell but no change at the new level explains it, for example because a better
entity was removed, the chain says so instead of guessing.

## 3. Where it appears

| Surface | Form |
|---|---|
| `toolbox report --baseline` and `--against` | New findings, each once, under their entity, with the rules that report them; the chain indented under every other `✗ worse` line (D01M84CCDH1ZWF9726SVDDM1JDX). Without `-o` or `--out`, the verdict is the whole output, on stdout (D01M841J2SN4V3AVZZ5D91YAR51) |
| `--verdict FILE` | Each regression has `detail` (strings, including the source record) and `causes` (nested regressions of the same shape) |
| The `actions/report` action | One annotation per regression, with the whole chain |

## 4. Why it is core

**A gate that fails without a cause teaches people to update the baseline blindly.** Ownership
tells you who should fix something; a root cause tells them what to fix. The two together turn a
red check into a task an agent can complete: the chain names the file, the evidence and the
command, so the agent can rerun the tool, fix the finding and watch the gate go green.

The user's assessment is that Backstage and Port don't trace a scorecard's change to its cause.
Their scorecards report rule outcomes per entity, and roll-ups are computed separately. This was
not re-verified for this doc ([research.md](../research.md) compares their scorecard languages,
not their explanations).

## 5. Next

| # | Item |
|---|---|
| C1 | ✅ Evidence identity: the baseline records a key per evidence item, and a grown list marks the new items (D01M841J2SN4V3AVZZ5D91YAR51; measured by [gate-repair-ab](../../experiments/gate-repair-ab/README.md)). Judging by keys, so that "fixed one, broke another" is worse, stays with [ratchets.md](./ratchets.md) R1 |
| C2 | Causes for campaigns: when a campaign's paydown goes backwards, name the entities that moved it |
| C3 | Causes in the UI and over MCP (`explain_entity` already explains a score; extend it to a change) |

## Sources

- [crates/toolbox-cli/src/causes.rs](../../crates/toolbox-cli/src/causes.rs): the chain
- [crates/toolbox-cli/tests/ratchet.rs](../../crates/toolbox-cli/tests/ratchet.rs): `a_repository_level_regression_names_its_root_cause`, and the evidence on a rule's regression
- [actions/report/run.sh](../../actions/report/run.sh): the annotations
