# Assessment freshness is an opt-in gate

Status: accepted, 2026-11-25.

Supersedes the assessment exit-code rule in D01M7Y5VWKW5TDS0VQWZFPRQ27N.

Routine assessment checks report stale and missing judgments. Invalid files still fail, including
files missing a verdict for any rubric criterion. A change to a rubric and its affected assessment
files should land together. This keeps routine changes from requiring a new judgment and an
independent review when a source hash changes.

Release preflight and explicitly high-risk changes use `toolbox assess check --require-fresh`.
The self-portal CI job selects this mode for pull requests labeled `require-fresh-assessments`.
Reviewers use the reported missing judgments to identify any additional independent review needed.

[Assessment design](../design/scorecards-for-scorecards.md)
[Release preflight](../RELEASING.md#assessment-preflight)
