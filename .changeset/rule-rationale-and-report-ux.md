---
"seikika": minor
"file-size": minor
"github-actions": minor
"openssf-scorecard": minor
"pattern-count": minor
"pnpm-expo": minor
"sarif": minor
"semgrep": minor
"storybook": minor
---

Scorecards and rules take a `rationale` and `links`, manifests group kinds with `kindGroups`, a kind may carry a reader-facing `title` (the `sarif` package sets it through `kindTitle`), and every rule has a computed basis (measured or agent-judged). Every package scorecard and rule now says why it exists. The HTML report gains Tracks, kind groups in the catalog, rule pages, Measured and Agent-judged badges wherever a result is shown, and a Compare view that explains its verdict and how it was measured. The markdown report and the ratchet's gate summary mark agent-judged rules with ✦. Links are titled by what they say rather than by a decision id, and the HTML report drops parentheticals that only cite internal ids or doc paths. The overview marks standards at their top level and names a standard that caps the repository level; a level that rests on unrun checks is drawn as provisional everywhere; no page scrolls sideways on a phone. A kind page says why some of its entities are not scored, and the design-audit capture counts WCAG 2.2 rules such as target size.
