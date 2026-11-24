---
id: D01M85MK7FCRXH2TRMRKXTNA5GT
decided: 2026-11-24T02:01:43Z
---

# UI changes are judged by a blind design audit, and the hosted report leads with its verdict

**A change to a rendered surface is judged by an independent auditor, never by its implementer.
The auditor sees only screenshots and metrics, and scores ten usability and four joy dimensions
from 0 to 4 against named reader tasks. A second auditor, who never sees the first scores,
re-audits the change. The change is done when no usability score falls, the targeted scores
rise, and serious axe-core violations do not grow. The hosted report is redesigned to that bar
(below).** The process is in [design-audit.md](../design/design-audit.md), with the skill
[`design-audit`](../../plugins/toolbox-operator/skills/design-audit/SKILL.md). The first audit is
[toolbox-report-2026-11-24.md](../audits/toolbox-report-2026-11-24.md).

1. **Why a blind auditor.** `report-ux` (D01M7XY1QYKW2D0HMSG50R0XTQH) gave builders a
   checklist, and the page passed it while readers still found it hard to understand and joyless.
   The baseline audit found faults the builders had missed: failing rules listed under "passing",
   a campaign shown as both 0% and fully met, and 986 serious axe-core nodes across eight routes.
2. **What the report now does.**
   - Overview opens on the repository level as a medal, with a one-sentence verdict, the next
     level and the standards that hold it there.
   - Scorecards and campaigns use human titles; scorecards carry `title` and `description` in
     the report JSON.
   - Levels are one badge component everywhere, with text colours that pass WCAG AA contrast.
   - Each scorecard lists what each level needs and its most common failures.
   - Each entity shows its next step.
   - A campaign with nothing in scope says so.
   - Rows are single links, and unknown routes show Not found.
3. **Contract.** The report JSON adds `title` and `description` per scorecard. Nothing is
   removed. Deep links keep their hashes.
4. **Joy is second to usability.** A delighter that hides data, slows a task or fails contrast
   is a regression, whatever it adds to a joy score.
5. **Reopen when** five real readers on the audit's tasks disagree with the agent auditor's
   outcomes.

## Where

- [design-audit.md](../design/design-audit.md), [report-ux.md](../design/report-ux.md)
- `plugins/toolbox-operator/skills/design-audit/`, `plugins/toolbox-operator/skills/report-ux/`
- `crates/toolbox-cli/src/toolbox-report.js`, `crates/toolbox-cli/src/report.rs`
