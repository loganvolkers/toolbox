---
id: D01M7Y5VWNAWB8K1DNMY31KQPQ0
decided: 2026-11-21T04:29:37Z
---

# A waiver cites a decision, and the supply-chain files are now the standard

**An accepted failure is a waiver, and a waiver cites a decision that already exists.** `portal/waivers.yaml` names one entity, one scorecard and one rule. The scorer records the outcome as `waived`, keeps the evidence, and counts it as a pass for the level. Removing the waiver makes a still-failing rule a regression. Closing the gap, so the rule passes with the waiver deleted, is a new decision. There is no detector that invents that decision; the validator only checks that the cited file exists.

**`release.yml` template findings are waived, not silenced in zizmor.** The file is the bytes dist generates (D98) and is not edited by hand. `.github/zizmor.yml` no longer ignores `excessive-permissions`, `template-injection` or `unpinned-images` on that file. `no-high-findings`, `no-medium-findings` and `zizmor-clean` on `workflow:.github/workflows/release.yml` are waived under this decision. `ci.yml` and `codeql.yml` stay fully checked.

**The supply-chain files are the standard for this repository.** Dependabot, the pinned CodeQL workflow, `SECURITY.md` and `CODEOWNERS` exist, and `maintainer-supply-chain` scores the live entity gold. D85 left the entity at none until those files existed, and said they were not added in order to pass. This decision is that adoption. D85 is not rewritten.

**Where:** [design/waivers.md](../design/waivers.md), [design/supply-chain-standards.md](../design/supply-chain-standards.md), [design/github-actions-standards.md](../design/github-actions-standards.md)
