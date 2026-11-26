#!/usr/bin/env python3
"""Render the box A/B's twelve prompts from one template: arm A's and arm B's differ only in the
arm block, which `--check` verifies. Run: python3 -I experiments/box-ab/prompts/make.py [--check]"""

import difflib
import sys
from pathlib import Path

HERE = Path(__file__).parent
AB_HOME = "/tmp/box-ab"

COMMON = """You are working in a copy of the PostHog repository at {arm_dir}. Work from earlier phases, if any, is committed there: read it before you start. Change files only inside that directory, and don't commit; the harness commits when you finish.

## Environment

- First run `export PATH={path}:$PATH`. That holds the tools CI runs: semgrep 1.163.0, ruff {ruff} ({ruff_note}), actionlint 1.7.12 and zizmor 1.32.0. shellcheck isn't installed, so actionlint runs without it (`-shellcheck= -pyflakes=`).
- No network and no installs. Budget: 40 tool calls.
- Don't read anything under {ab_home} (only run the binaries on that PATH), and don't read /home/user/seikika/experiments/ or /home/user/seikika/docs/research-log.md.

{arm_block}

## The tools in scope

These are CI's own runs. Keep every tool usable on its own, with its own config files.
- **semgrep**: the rules `.semgrep/rules/no-direct-persons-db-orm.yaml` and `.semgrep/rules/prefer-codegen-api.yaml`, over CI's paths (`bin/ common/ ee/ frontend/ posthog/ products/ services/llm-gateway/ services/stripe-mock/`). Findings suppressed inline with `nosemgrep` count: a new suppression is a regression.
- **ruff**: `ruff check .` with the repository's config.
- **actionlint**: the workflows, with `.github/actionlint.yaml`.{zizmor_scope}{wf001_scope}

## The interface

The harness reads this; it doesn't count as glue. Keep a `TOOLS.md` at the repository root holding one ```json block:

```json
{{"gate": "...", "accept": "...", "progress": "...", "tools": [{{"name": "semgrep", "version": "1.163.0", "command": "..."}}]}}
```

- **gate**: one shell command, run from the repository root, that exits non-zero when any tool got worse than its recorded baseline (any file with more findings of that tool) and zero otherwise.
- **accept**: records the current state as the baseline.
- **progress**: prints each campaign's start, current count and paydown (empty until there is a campaign).
- **tools**: for each tool, its version and a command that runs the tool alone (no wrapper of yours), with the tool's own config, printing SARIF or the tool's own JSON on stdout. The harness reruns these and compares the findings with the tool run directly.

Everything else is your choice: where baselines live, scripts, workflow steps. CI is GitHub Actions; wire the gate into a workflow as you would for real.

## Task

{task}

When you finish, reply with:
1. What you built or changed, and the gate's result on the current state.
2. Anything you couldn't do.
3. **Developer-experience feedback**, most important first: anything that was confusing, missing, slow, wrong in the docs or help, or that you had to work around, with the command or file it concerned. Say "none" if there was none."""

ARM = {
    "A": """## Your setup

You have the repository, its tools, Python 3 and the shell. Write whatever glue you need: scripts, baseline files, workflow steps. Don't use or read Seikika (/home/user/seikika).""",
    "B": """## Your setup

You have the repository, its tools, Python 3, the shell and Seikika: `export PATH={ab_home}/box-bin:$PATH` gives `seikika`. Its docs and kits are in /home/user/seikika, where you may read only `README.md`, `docs/v1-spec.md`, `docs/design/`, `docs/reference/`, `examples/`, `packages/`, `crates/`, `schema/` and `plugins/`. The operator skills in `plugins/seikika-operator/skills/` are written for agents. There is no network, so bring kits in with `seikika vendor` (or a relative path to /home/user/seikika/packages), as a repository without network access would.""",
}

TASKS = {
    1: """**Gather.** Put semgrep, ruff and actionlint under one gate: a pull request fails when any of them gets worse. One baseline per tool or one for all, your choice. Record today's findings as the baseline, so the gate passes now. In your reply, give each tool's current findings (count and files) and the total.""",
    2: """**Campaign.** Start a campaign to pay semgrep's findings, suppressed ones included, down to zero. Its start is today's count, recorded so that progress is measured from it, and `progress` prints its start, current count and paydown. ruff has no findings today; say whether it needs a campaign. In your reply, give the campaign's starting count.""",
    3: """**Add a tool.** Add zizmor (offline, over `.github/`, its default audits, no config file) to the gate. Its findings today must not fail CI; new ones must. Keep it usable on its own.""",
    4: """**Upgrade.** CI's ruff is now 0.15.0: it is first on the PATH above. Upgrade the repository to it (the pinned version wherever you keep one, and the baseline). ruff 0.15.0 reports findings 0.14.11 didn't. Accept those for ruff only: no other tool's baseline may move, and a regression in another tool must still fail.""",
    5: """**Swap.** hogli's `lint:workflows` check WF001 (`tools/hogli-commands/hogli_commands/workflow_lint/checks/job_timeouts.py`) requires every job to declare `timeout-minutes`, except reusable-workflow calls (`jobs.<id>.uses`). Replace it with a semgrep rule over the workflows: add the rule, put it in the gate as the tool `job-timeouts` in TOOLS.md, and remove WF001 from hogli's checks. It must flag exactly what WF001 flags. In your reply, say whether its history (baseline) was kept or restarted.""",
    6: """**Read.** The last pull request is the latest commit. CI's gate output for it is in `last-pr-gate.log`. Using that log and the repository's artifacts (baselines, reports, TOOLS.md), and without rerunning the gate or the tools, answer:
1. Which tool or tools failed the gate?
2. For each new finding: the file, the line, the rule, and why it is a finding.
3. The exact command that reproduces each one with the tool alone.
4. Each campaign's progress: start, now and paid.
Don't change anything.""",
}

ZIZMOR = "\n- **zizmor**: offline, over `.github/`, default audits, no config file."
WF001 = "\n- **job-timeouts**: the semgrep rule that replaced hogli's WF001 (phase 5)."


def render(phase, arm):
    path = f"{AB_HOME}/bin" if phase < 4 else f"{AB_HOME}/upgrade/ruff-0.15.0/bin:{AB_HOME}/bin"
    if arm == "B":
        path = f"{AB_HOME}/box-bin:" + path
    return COMMON.format(
        arm_dir=f"{AB_HOME}/arms/{arm}", path=path, ab_home=AB_HOME,
        ruff="0.15.0" if phase >= 4 else "0.14.11",
        ruff_note="the upgrade, from phase 4" if phase >= 4 else "PostHog's pin",
        arm_block=ARM[arm].format(ab_home=AB_HOME),
        zizmor_scope=ZIZMOR if phase >= 3 else "",
        wf001_scope=WF001 if phase >= 6 else "",
        task=TASKS[phase],
    ) + "\n"


def main(check):
    bad = 0
    for phase in TASKS:
        texts = {arm: render(phase, arm) for arm in "AB"}
        for arm, text in texts.items():
            p = HERE / f"{phase}-{arm}.md"
            if check:
                bad += p.read_text() != text
            else:
                p.write_text(text)
        # The arms' prompts differ only in the setup block and the PATH line.
        same_dir = {arm: t.replace(f"/arms/{arm}", "/arms/<arm>") for arm, t in texts.items()}
        diff = [line for line in difflib.unified_diff(same_dir["A"].splitlines(), same_dir["B"].splitlines(), lineterm="", n=0)
                if line[:1] in "+-" and not line.startswith(("+++", "---"))]
        allowed = {ARM["A"].splitlines()[-1], ARM["B"].format(ab_home=AB_HOME).splitlines()[-1]}
        stray = [line for line in diff if line[1:] not in allowed and "export PATH=" not in line]
        if stray:
            print(f"phase {phase}: prompts differ outside the arm block: {stray}")
            bad += 1
    print("prompts:", "out of date or differing" if bad else "ok")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main("--check" in sys.argv))
