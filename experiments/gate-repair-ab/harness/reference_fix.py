#!/usr/bin/env python3
"""A correct repair of the planted PR, used to check score.py and both gates before a round runs."""
import pathlib
import sys

CHECKOUT = "actions/checkout@de0fac2e4500dabe0009e67214ff5f5447ce83dd # v6.0.2"
UPLOAD = "actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a # v7.0.1"


def edit(path: pathlib.Path, pairs) -> None:
    source = path.read_text()
    for old, new in pairs:
        if source.count(old) != 1:
            sys.exit(f"reference_fix: {old!r} not unique in {path}")
        source = source.replace(old, new)
    path.write_text(source)


def main() -> None:
    repo = pathlib.Path(sys.argv[1])
    edit(repo / ".github/workflows/pull_request.yaml", [
        ("    needs: lnt\n", "    needs: lint\n"),
        ("        uses: actions/checkout@v6\n",
         f"        uses: {CHECKOUT}\n        with:\n          persist-credentials: false\n"),
        ("      - name: Write PR summary\n        run: |\n"
         "          echo \"## ${{ github.event.pull_request.title }}\" >> \"$GITHUB_STEP_SUMMARY\"\n",
         "      - name: Write PR summary\n        env:\n          PR_TITLE: ${{ github.event.pull_request.title }}\n"
         "        run: |\n          echo \"## ${PR_TITLE}\" >> \"$GITHUB_STEP_SUMMARY\"\n"),
    ])
    edit(repo / ".github/workflows/release.yaml", [
        ("        uses: actions/upload-artifact@v7\n", f"        uses: {UPLOAD}\n"),
    ])


if __name__ == "__main__":
    main()
