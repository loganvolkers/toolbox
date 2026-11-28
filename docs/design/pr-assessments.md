# Pull request assessment pilot

*Status: implemented for the maintainer SDLC under D01M8D9AK7Q9B18B10FC38WB2BQ.*

This is about the box. Seikika for Seikika determines which checks and review questions apply to a pull request, saves the evidence, and gives an optional judge a bounded packet. The ordinary CI ratchet remains the merge check. An advisory finding or a high-risk review obligation can be taken to the owner through the existing GitHub review process.

## Plan and check

The planner reads [the policy](../../.seikika/assessment-pr.json) at the base ref and diffs the merge base against the requested head. Both paths of a rename, deleted paths, and shared dependency files can trigger a check. An unmapped path or a base without the policy gives `scope_unknown` and names who can triage it. Changes to the policy in a PR take effect on later PRs once the trusted base contains the new version.

CI uses the exact candidate commit that the existing PR jobs test. The Rust job archives the plan and evidence for onboarding, acceptance diagnostics, and SDK coverage. The UI job archives the rendered-state evidence. Existing Storybook CI includes the campaign hatch and unmeasured-comparison assertions. A missing artefact, a failed check, or an unanswered criterion leaves the affected review obligation incomplete.

Run the same workflow from a clean candidate checkout with the repository's usual dependencies and built CLI:

```sh
base=origin/main
head=$(git rev-parse HEAD)
python3 scripts/assessment-pr.py plan --base "$base" --head "$head"
pnpm storybook:build
python3 scripts/assessment-pr.py check --base "$base" --head "$head" --evidence-dir /tmp/seikika-pr-evidence
```

Use repeatable `--only CHECK` options when CI splits checks across jobs. Each evidence directory contains `evidence.json` and the named logs. Preserve the logs alongside their manifest. Run the acceptance diagnostic reproducer directly with `python3 scripts/assessment-probes.py acceptance`. It drives the real acceptance scripts through a forced failure with fixture subprocesses and checks exit, diagnostics, and cleanup.

## Delegate from a trusted runner

Configure the harness outside the candidate checkout. For Codex, the adapter uses the installed CLI and its existing login in ephemeral read-only mode. The runner launches it in a temporary directory and sends the plan, check status and artifact digests, and scoped diff over stdin. Use a shared local SQLite path across workers on the same runner. For each PR, pass a monotonically increasing generation from the runner's event sequence. Reuse the generation for retries of the same event. Call `activate` with the newer generation when it arrives to supersede an in-flight assessment, even if its checks are still pending.

```sh
pilot="$PWD/scripts/assessment-pr.py"
adapter="$PWD/scripts/assessment-codex.py"
ledger="$HOME/.seikika-pr-assessments.sqlite"
python3 "$pilot" delegate --base "$base" --head "$head" \
  --ledger "$ledger" --repository seikika/seikika --pr 142 --generation 1 \
  --adapter codex --evidence /tmp/seikika-pr-evidence/evidence.json \
  -- python3 "$adapter"
python3 "$pilot" status --ledger "$ledger" --repository seikika/seikika --pr 142
```

Another harness can use the same entry point. It reads the JSON packet from stdin and emits one JSON result containing the plan digest and exactly one verdict per planned route and criterion. Cite a planned check with `{"check":"onboarding"}` or a changed file with `{"path":"path/to/file","line":10,"side":"head"}`. Use `base` for a line in the comparison merge base. A failed verdict includes severity 1–4 and a concrete suggestion. An unsure verdict can record missing evidence. A routed high-risk scope with completed judgment shows `needs_owner_decision`. The maintainer can read prior and superseded verdicts in `status`.

## Admission and budgets

| Control | Pilot behavior |
|---|---|
| Admission | One concurrent invocation per PR, two starts per PR per rolling hour and eight per repository per rolling hour. The same PR carries a cycle limit of three starts across head changes. |
| Retries and overrides | One retry for an adapter transport error, timeout, or abandoned lease on the same immutable plan and review slot. A maintainer can request `--slot adjudication` for a second independent review on the same head, or extend a cycle or design plateau with `--override-reason`. Hourly and concurrency limits still apply. |
| Design loop | Two completed design-only reviews with zero new finding identities pause further automatic judgment. Findings are tracked by criterion and evidence paths so a line move alone does not count as new. The executable checks continue to run. |
| Cancellation | The runner supervises and terminates the adapter's process group at the deadline, on interrupt, or when a newer plan supersedes it. A crashed runner leaves its reservation in the ledger until the lease expires. |
| Work per invocation | A 48 KB serialized input packet, a five-minute deadline, and bounded subprocess output. The 32,000-token and six-turn settings are adapter allowances. The runner marks a measured overrun separately. The Codex adapter records token totals from completed turn events. Pre-request token enforcement depends on capabilities exposed by the caller's harness. |

This pilot keeps CI model-free. A maintainer supplies the delegate and its authentication through the normal local setup. Forward environment variables to the adapter explicitly with `--forward-env` when needed. Plan, check, and result files are bound to exact Git inputs. Treat the ledger and check artifact directories as runner-owned.

## Entity assessment compatibility

`assess brief` now emits `rubricHash` alongside `subjectHash`. Include both when saving a new assessment. `assess check` flags an assessment with a changed rubric as stale when the assessment includes the rubric hash, and `assess submit` checks it against the local rubric before upload. Existing files that predate the field still work. Duplicate verdicts and duplicate files for a rubric/subject pair fail validation.

## Sources

- [Assessment audit and archived replays](../audits/agentic-assessments-2026-11-26.md)
- [Maintainer CI](../../.github/workflows/ci.yml) and [report state probes](../../npm/report-ui/scripts/probe.mjs)
- [Delegate mode in Open Code Review](https://github.com/alibaba/open-code-review/blob/main/pages/src/content/docs/en/integrations/delegate.md)
