#!/usr/bin/env python3
"""Render a round's prompts and build its workspaces (README §6).

    round.py <round> <runs-per-arm> [--workspaces] [--only <repo>]

For each repo, arms A (no graph) and B (Graphify) get <runs-per-arm> runs each, numbered in a
shuffled order so a workspace's name doesn't say its arm: `<repo>-<n>`. Prompts go to
prompts/rendered/r<round>/<repo>-<n>.md and the run-to-arm map to arms.tsv. With --workspaces,
each run gets a copy of the frozen export at /tmp/graphify-ab/r<round>/<repo>-<n>; arm B's copy
also gets the prebuilt graphify-out/ and the `.cursor/rules/graphify.mdc` that
`graphify install --platform cursor` writes.
"""
import pathlib
import random
import shutil
import sys

HERE = pathlib.Path(__file__).resolve().parent
PROMPTS = HERE.parent / "prompts"
WORK = pathlib.Path("/tmp/graphify-ab")
REPOS = {"seikika": "Seikika", "openfga": "OpenFGA"}
BUILD_ONLY = (".graphify_detect.json", ".graphify_extract.json", ".graphify_ast.json", ".graphify_semantic*.json",
              ".graphify_analysis.json", ".graphify_partial.txt", ".graphify_chunk_*.json", ".graphify_cached.json",
              ".graphify_uncached.txt", ".chunks.json", ".needs_update")


def rule_block() -> str:
    text = (PROMPTS / "graphify.mdc").read_text().split("---\n", 2)[2].strip()
    return ("- This project has a Cursor rule, `.cursor/rules/graphify.mdc`, that applies to you. Its text:\n\n"
            f"<rule>\n{text}\n</rule>\n")


def main() -> None:
    rnd, runs = sys.argv[1], int(sys.argv[2])
    build = "--workspaces" in sys.argv
    out = PROMPTS / "rendered" / f"r{rnd}"
    out.mkdir(parents=True, exist_ok=True)
    template = (PROMPTS / "task.md").read_text()
    rng = random.Random(f"graphify-ab-r{rnd}")
    arms = []
    only = sys.argv[sys.argv.index("--only") + 1] if "--only" in sys.argv else None
    for repo, name in REPOS.items():
        if only and repo != only:
            continue
        order = ["A"] * runs + ["B"] * runs
        rng.shuffle(order)
        questions = (PROMPTS / f"questions-{repo}.md").read_text().strip()
        for n, arm in enumerate(order, 1):
            run = f"{repo}-{n}"
            dest = WORK / f"r{rnd}" / run
            prompt = (template.replace("{RULE}", rule_block() if arm == "B" else "")
                      .replace("{QUESTIONS}", questions).replace("{REPO}", name).replace("{DEST}", str(dest)))
            (out / f"{run}.md").write_text(prompt)
            arms.append(f"{run}\t{arm}")
            if build:
                if dest.exists():
                    shutil.rmtree(dest)
                shutil.copytree(WORK / "src" / repo, dest, symlinks=True)
                if arm == "B":
                    # What the skill leaves after step 9's cleanup, with the scan root moved here.
                    shutil.copytree(WORK / "build" / repo / "graphify-out", dest / "graphify-out", symlinks=True,
                                    ignore=shutil.ignore_patterns(*BUILD_ONLY))
                    (dest / "graphify-out" / ".graphify_root").write_text(str(dest))
                    shutil.copy(WORK / "build" / repo / ".graphifyignore", dest / ".graphifyignore")
                    (dest / ".cursor" / "rules").mkdir(parents=True, exist_ok=True)
                    shutil.copy(PROMPTS / "graphify.mdc", dest / ".cursor" / "rules" / "graphify.mdc")
    (out / "arms.tsv").write_text("\n".join(arms) + "\n")
    print("\n".join(arms))


if __name__ == "__main__":
    main()
