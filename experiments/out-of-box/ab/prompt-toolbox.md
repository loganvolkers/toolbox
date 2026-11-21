# Arm B prompt (Toolbox)

Sent as written, with `<repo>` = `openfga` or `codex`, `<dir>` = `/tmp/claude-0/ab/<repo>-B`, and
`<portal>` = `portal` (OpenFGA) or `portal-pnpm` (Codex).

> You are improving the CI and repository hygiene of a checkout of the <project> project at <dir>.
> This is an experiment, so follow these rules exactly:
>
> - Work ONLY inside <dir>, apart from running the `toolbox` command below. Do not read or modify
>   anything else, and do not look for other tools or repositories on this machine.
> - Do not commit, push, or use the network.
> - Make at most 40 tool calls in total, then stop.
> - Don't remove or disable functionality: no deleting workflows or jobs, no removing triggers the
>   project relies on, no deleting docs. Fix things properly.
> - Start every shell command with `export PATH=/tmp/claude-0/ab/bin:$PATH;` (toolbox, zizmor and
>   actionlint are installed there).
>
> Goal: make the repository meet as many of its standards as you can within your budget,
> prioritising the higher-impact ones. The standards are scored by Toolbox. This command scores the
> checkout and prints every scorecard, each failing rule with its fix and its evidence (the exact
> files, lines and findings):
>
>     cd <dir> && toolbox report /tmp/claude-0/ab/<portal> --skip github-actions
>
> Rules about run history (fail rates, reruns, `main-not-stuck-red`, `fast-feedback`) can't be met
> offline: ignore them. Before you start, record a baseline so the ratchet tells you if you make
> anything worse:
>
>     cd <dir> && toolbox report /tmp/claude-0/ab/<portal> --skip github-actions --baseline /tmp/claude-0/ab/<repo>-B.results.json --update -o json --out /dev/null
>
> and afterwards check with the same command without `--update` (it prints "worse" lines for
> regressions and "better" lines for improvements). Re-run the report as often as you find useful.
>
> Note: you cannot look up commit SHAs on the network. Where pinning needs a SHA you don't know,
> leave that action unpinned rather than inventing a SHA.
>
> When you finish, reply with: (1) the files you changed and what you changed in each, (2) which
> rules you believe now pass that didn't before, and (3) how many tool calls you used.
