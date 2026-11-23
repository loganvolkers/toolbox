#!/usr/bin/env python3
"""One correct repair of the planted pull request: pin each action to the SHA the repository
already pins it to, with its version comment. On it both gates pass and score.py says correct.

    reference_fix.py <repo>
"""
import pathlib
import sys

PINS = {
    "actions/checkout@v6": "actions/checkout@de0fac2e4500dabe0009e67214ff5f5447ce83dd # v6.0.2",
    "actions/setup-go@v6": "actions/setup-go@4a3601121dd01d1626a1e23e37211e3254c1c06c # v6.4.0",
    "golangci/golangci-lint-action@v9": "golangci/golangci-lint-action@1e7e51e771db61008b38414a730f564565cf7c20 # v9.2.0",
    "golang/govulncheck-action@v1": "golang/govulncheck-action@b625fbe08f3bccbe446d94fbf87fcc875a4f50ee # v1.0.4",
    "actions/cache@v5": "actions/cache@27d5ce7f107fe9357f9df03efb73ab90386fccae # v5.0.5",
    "codecov/codecov-action@v6": "codecov/codecov-action@57e3a136b779b570ffcdbf80b3bdc90e7fab3de2 # v6.0.0",
    "actions/upload-artifact@v7": "actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a # v7.0.1",
}


def main() -> None:
    path = pathlib.Path(sys.argv[1]) / ".github/workflows/weekly.yaml"
    lines = path.read_text().splitlines(keepends=True)
    for i, line in enumerate(lines):
        ref = line.strip().removeprefix("uses: ")
        if line.strip().startswith("uses: ") and ref in PINS:
            lines[i] = line.replace(ref, PINS[ref])
    path.write_text("".join(lines))


if __name__ == "__main__":
    main()
