#!/usr/bin/env python3
"""Plant the pull request every run repairs: the same two edits in both arms.

The PR adds a job that writes the pull request title to the job summary, and a release step that
uploads digests.txt. Both carry the mistakes a hurried author makes: an unpinned action, a
credential-persisting checkout, a template injection, and a `needs:` typo. The fix needs no
network: the pinned SHAs are already in the repository.
"""
import pathlib
import sys

PR_JOB = """  pr-summary:
    runs-on: ubuntu-latest
    needs: lnt
    timeout-minutes: 5
    steps:
      - name: Checkout code
        uses: actions/checkout@v6
      - name: Write PR summary
        run: |
          echo "## ${{ github.event.pull_request.title }}" >> "$GITHUB_STEP_SUMMARY"
          git log -1 --format=%s >> "$GITHUB_STEP_SUMMARY"

"""
PR_ANCHOR = "  # Fast unit tests"

RELEASE_STEP = """      - name: Upload binary digests
        uses: actions/upload-artifact@v7
        with:
          name: digests-${{ steps.tag_name.outputs.current_version }}
          path: digests.txt

"""
RELEASE_ANCHOR = "      - uses: slsa-framework/slsa-github-generator/actions/generator/generic/create-base64-subjects-from-file@"


def insert_before(path: pathlib.Path, anchor: str, text: str) -> None:
    source = path.read_text()
    if source.count(anchor) != 1:
        sys.exit(f"plant: anchor not unique in {path}: {anchor!r}")
    path.write_text(source.replace(anchor, text + anchor))


def main() -> None:
    repo = pathlib.Path(sys.argv[1])
    insert_before(repo / ".github/workflows/pull_request.yaml", PR_ANCHOR, PR_JOB)
    insert_before(repo / ".github/workflows/release.yaml", RELEASE_ANCHOR, RELEASE_STEP)


if __name__ == "__main__":
    main()
