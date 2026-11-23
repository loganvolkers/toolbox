#!/usr/bin/env python3
"""Plant the pull request every run repairs: the same new workflow in both arms.

The PR adds a weekly workflow that lints, checks for vulnerabilities, and runs the tests with
coverage. Its author wrote every action as a tag (`actions/checkout@v6`) instead of the commit SHA
the rest of the repository pins it to. Each of the seven actions is pinned elsewhere in the
repository, with its version in a comment, so the fix needs no network.
"""
import pathlib
import sys

WORKFLOW = """name: Weekly checks

on:
  schedule:
    - cron: "0 6 * * 1" # At 06:00 on Monday
  workflow_dispatch:

permissions:
  contents: read

jobs:
  lint:
    runs-on: ubuntu-latest
    timeout-minutes: 15
    steps:
      - name: Checkout code
        uses: actions/checkout@v6
        with:
          persist-credentials: false
      - name: Set up Go
        uses: actions/setup-go@v6
        with:
          go-version-file: go.mod
          cache: false
      - name: Run golangci-lint
        uses: golangci/golangci-lint-action@v9
        with:
          version: latest
          args: -v -c .golangci.yaml

  govulncheck:
    runs-on: ubuntu-latest
    timeout-minutes: 15
    steps:
      - name: Checkout code
        uses: actions/checkout@v6
        with:
          persist-credentials: false
      - name: Run govulncheck
        uses: golang/govulncheck-action@v1
        with:
          go-version-file: go.mod
          repo-checkout: false

  coverage:
    runs-on: ubuntu-latest
    timeout-minutes: 30
    steps:
      - name: Checkout code
        uses: actions/checkout@v6
        with:
          persist-credentials: false
      - name: Set up Go
        uses: actions/setup-go@v6
        with:
          go-version-file: go.mod
          cache: false
      - name: Cache Go build
        uses: actions/cache@v5
        with:
          path: ~/.cache/go-build
          key: ${{ runner.os }}-go-build-${{ hashFiles('**/*.go', 'go.sum') }}
      - name: Run tests with coverage
        run: go test -coverprofile=coverage.out ./...
      - name: Upload coverage to Codecov
        uses: codecov/codecov-action@v6
        with:
          files: ./coverage.out
          flags: weekly
          token: ${{ secrets.CODECOV_TOKEN }}
      - name: Keep the coverage profile
        uses: actions/upload-artifact@v7
        with:
          name: coverage
          path: coverage.out
"""
PATH = ".github/workflows/weekly.yaml"


def main() -> None:
    repo = pathlib.Path(sys.argv[1])
    target = repo / PATH
    if target.exists():
        sys.exit(f"plant: {PATH} exists")
    target.write_text(WORKFLOW)


if __name__ == "__main__":
    main()
