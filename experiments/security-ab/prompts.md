# Prompts for the Security A/B

Each phase starts a fresh general-purpose agent. `{WS}` is the phase's
workspace, which holds `repo/`, an export of the repository's tracked
files. `{PATH}` is the arm A tool path (`/tmp/box-ab/bin` plus the
system's Go, Cargo, Node and Python).

## A1 and A3: build

> You are a platform engineer. Build a security scorecard for software
> repositories, then run it on the repository in `{WS}/repo`.
>
> - The scorecard is a program you write, in any language, in
>   `{WS}/scorecard/`. It runs as `{WS}/scorecard/run <repo-path>` and
>   prints JSON to stdout:
>   `{"repo": ..., "checks": [{"id", "title", "score": 0-10 or null if it can't be evaluated,
>   "why": what it checks and why it matters, "findings": [{"message", "path", "line"}]}]}`.
> - Other teams will run it on their own repositories, so it must be
>   reusable and not special to this one.
> - Decide for yourself what a repository's security should be judged
>   on.
> - Use only these tools, with `PATH={PATH}`: zizmor, actionlint,
>   semgrep (with no registry rules), ruff, go, cargo, node, python3 and
>   jq. There is no network: package registries and APIs are blocked.
>   Don't install anything.
> - Work only inside `{WS}`. Don't read other directories on this
>   machine.
> - Write the results for this repository to `{WS}/results.json`. Write
>   a short `{WS}/REPORT.md` covering:
>   - the checks you chose, and why;
>   - the most important findings;
>   - what you would add with more time.
> - Budget: about 60 tool calls.
>
> Finish with a three-line summary: number of checks, overall score, and
> top finding.

## A2: reuse

> A colleague built a security scorecard for software repositories: it is in `{WS}/scorecard/`, and
> their notes are in `{WS}/scorecard/REPORT.md`. Run it on the repository in `{WS}/repo` (another
> team's) as `{WS}/scorecard/run {WS}/repo`.
>
> - If it fails, or gives wrong results for this repository, fix the scorecard. Keep it general:
>   don't special-case this repository.
> - The tool, network and directory rules are the same as the colleague's:
>   - use only `PATH={PATH}`: zizmor, actionlint, semgrep with no registry rules, ruff, go, cargo,
>     node, python3 and jq;
>   - no network, and don't install anything;
>   - work only inside `{WS}`.
> - Write the results to `{WS}/results.json`. Write `{WS}/REUSE.md`, listing:
>   - every change you made to the scorecard, and why;
>   - any results you think are still wrong.
> - Budget: about 40 tool calls.
>
> Finish with a three-line summary: whether it ran unchanged, number of changes, overall score.

## Deviations

- **A1 and A3** couldn't write `REPORT.md`: the harness refuses report files from subagents
  ("Subagents should return findings as text"). Both returned the text in their final message, and
  it was saved verbatim to `data/a1/REPORT.md` and `data/a3/REPORT.md`.
- **A2's prompt was changed before it ran**, for the same reason. Instead of writing `REUSE.md`,
  it put the same notes in its final message, under a `REUSE.md` heading. Its changes are
  `data/a2/scorecard.diff`.
