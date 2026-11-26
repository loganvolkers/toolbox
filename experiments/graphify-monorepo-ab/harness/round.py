#!/usr/bin/env python3
"""Render a round's prompts and build its workspaces (README §6).

    round.py <round> <runs-per-arm> [--workspaces]

Arms A (nothing), B (Graphify) and C (Seikika) get <runs-per-arm> runs each, numbered in a
shuffled order so a workspace's name doesn't say its arm: `codex-<n>`. Prompts go to
prompts/rendered/r<round>/codex-<n>.md and the run-to-arm map to arms.tsv (run, arm, port). With
--workspaces, each run gets a copy of the frozen export at /tmp/graphify-monorepo-ab/r<round>/codex-<n>
and an empty scratch directory at /tmp/graphify-monorepo-ab/scratch/r<round>/codex-<n>, so concurrent
runs don't share scratch files in /tmp.
Arm B's copy also gets the graphify-out/ the build left and the `.cursor/rules/graphify.mdc` that
`graphify cursor install` writes. Arm C's gets the portal at `.seikika/seikika.yaml` and its rule at
`.cursor/rules/seikika.mdc`, naming the port that `serve.sh` starts its server on.
"""
import pathlib
import random
import shutil
import sys

HERE = pathlib.Path(__file__).resolve().parent
PROMPTS = HERE.parent / "prompts"
WORK = pathlib.Path("/tmp/graphify-monorepo-ab")


def rule_text(name: str, port: int | None) -> str:
    text = (PROMPTS / name).read_text()
    return text.replace("{PORT}", str(port)) if port else text


def rule_block(name: str, port: int | None) -> str:
    body = rule_text(name, port).split("---\n", 2)[2].strip()
    return (f"- This project has a Cursor rule, `.cursor/rules/{name}`, that applies to you. Its text:\n\n"
            f"<rule>\n{body}\n</rule>\n")


def main() -> None:
    rnd, runs = sys.argv[1], int(sys.argv[2])
    build = "--workspaces" in sys.argv
    out = PROMPTS / "rendered" / f"r{rnd}"
    out.mkdir(parents=True, exist_ok=True)
    template = (PROMPTS / "task.md").read_text()
    questions = (PROMPTS / "questions-codex.md").read_text().strip()
    rng = random.Random(f"graphify-monorepo-ab-r{rnd}")
    order = ["A"] * runs + ["B"] * runs + ["C"] * runs
    rng.shuffle(order)
    base = 8390 if rnd.startswith("dry") else 8300
    arms = []
    for n, arm in enumerate(order, 1):
        run = f"codex-{n}"
        dest = WORK / f"r{rnd}" / run
        port = base + n if arm == "C" else None
        rule = {"A": "", "B": rule_block("graphify.mdc", None), "C": rule_block("seikika.mdc", port)}[arm]
        scratch = WORK / "scratch" / f"r{rnd}" / run
        prompt = (template.replace("{RULE}", rule).replace("{QUESTIONS}", questions)
                  .replace("{REPO}", "Codex").replace("{DEST}", str(dest))
                  .replace("{SCRATCH}", str(scratch)))
        (out / f"{run}.md").write_text(prompt)
        arms.append(f"{run}\t{arm}\t{port or '-'}")
        if not build:
            continue
        if dest.exists():
            shutil.rmtree(dest)
        if scratch.exists():
            shutil.rmtree(scratch)
        scratch.mkdir(parents=True)
        shutil.copytree(WORK / "src" / "codex", dest, symlinks=True)
        rules = dest / ".cursor" / "rules"
        if arm == "B":
            shutil.copytree(WORK / "build" / "graphify" / "codex" / "graphify-out", dest / "graphify-out", symlinks=True)
            (dest / "graphify-out" / ".graphify_root").write_text(str(dest))
            rules.mkdir(parents=True, exist_ok=True)
            (rules / "graphify.mdc").write_text(rule_text("graphify.mdc", None))
        if arm == "C":
            (dest / ".seikika").mkdir()
            shutil.copy(HERE / "portal" / "seikika.yaml", dest / ".seikika" / "seikika.yaml")
            rules.mkdir(parents=True, exist_ok=True)
            (rules / "seikika.mdc").write_text(rule_text("seikika.mdc", port))
    (out / "arms.tsv").write_text("\n".join(arms) + "\n")
    print("\n".join(arms))


if __name__ == "__main__":
    main()
