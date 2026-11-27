#!/usr/bin/env python3
"""Execute the maintainer assessment pilot's reproducible defect checks."""

import argparse
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
from tempfile import TemporaryDirectory


ROOT = Path(__file__).resolve().parents[1]


def script(path, body):
    path.write_text("#!/usr/bin/env bash\nset -eu\n" + body)
    path.chmod(0o700)


def onboarding(work):
    source = (ROOT / "site/src/content/docs/start/quickstart.md").read_text()
    blocks = re.findall(r"^```bash\n(.*?)^```", source, re.M | re.S)
    starter = re.search(
        r"^starter=(https://raw\.githubusercontent\.com/seikika/seikika/([a-f0-9]{40})/examples/repo-portal-starter)$",
        source, re.M,
    )
    if not starter or len(blocks) != 4:
        raise ValueError("quickstart must provide the pinned starter and four walkthrough steps")
    snapshot = work / "starter"
    for name in ("seikika.yaml", "tests/standards.test.yaml", "fixtures/workspace.ndjson"):
        target = snapshot / name
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(subprocess.check_output([
            "git", "show", f"{starter[2]}:examples/repo-portal-starter/{name}",
        ], cwd=ROOT))
    local_bin = work / "bin"
    local_bin.mkdir()
    script(local_bin / "curl", """
[[ $# == 4 && $1 == -fsSL && $2 == "$STARTER_URL/"* && $3 == -o ]]
case "${2#"$STARTER_URL/"}" in
    seikika.yaml|tests/standards.test.yaml|fixtures/workspace.ndjson)
        cp "$STARTER_DIR/${2#"$STARTER_URL/"}" "$4" ;;
    *) exit 2 ;;
esac
""")
    built = ROOT / "target/debug/seikika"
    seikika = built if built.is_file() else shutil.which("seikika")
    if seikika is None:
        raise ValueError("build Seikika and its workspace connector before running the walkthrough")
    env = os.environ | {
        "PATH": f"{local_bin}:{Path(seikika).parent}:{os.environ['PATH']}",
        "STARTER_URL": starter[1], "STARTER_DIR": str(snapshot),
    }
    subprocess.run(["bash", "-ec", blocks[0]], cwd=work, env=env, check=True, timeout=30)
    repo = work / "seikika-practice"
    for block, expected in zip(blocks[1:], (0, 1, 0), strict=True):
        run = subprocess.run(["bash", "-ec", block], cwd=repo, env=env, capture_output=True,
                             text=True, timeout=30)
        print(run.stdout, end="")
        print(run.stderr, end="")
        if run.returncode != expected:
            raise ValueError(f"quickstart returned {run.returncode}, expected {expected}")
        if expected == 1 and not all(word in run.stdout + run.stderr for word in ("links-resolve", "portal.md")):
            raise ValueError("quickstart failed without naming the broken link")


def acceptance(work):
    bin_dir = work / "bin"
    bin_dir.mkdir()
    script(bin_dir / "initdb", "exit 0\n")
    script(bin_dir / "curl", "exit 0\n")
    script(bin_dir / "pg_ctl", """
while [[ $# -gt 0 ]]; do
    if [[ $1 == -l ]]; then
        echo 'fixture postgres diagnostic' > "$2"
        exit "$PG_START_EXIT"
    fi
    shift
done
""")
    script(bin_dir / "seikika", """
if [[ $1 == serve ]]; then
    echo 'fixture server diagnostic'
    touch "$TMPDIR/server-ready"
    exit 0
fi
while [[ ! -e "$TMPDIR/server-ready" ]]; do sleep 0.02; done
exit 42
""")
    for stage in ("m2", "m3", "m4"):
        tmp = work / stage
        tmp.mkdir()
        # M2 reaches the fixture push. For M3 and M4, fail at database startup before spawning services.
        env = os.environ | {
            "PATH": f"{bin_dir}:{os.environ['PATH']}", "PGBIN": str(bin_dir),
            "PGDIR": str(work / f"{stage}-db"), "TMPDIR": str(tmp), "SEIKIKA": str(bin_dir / "seikika"),
            "SEIKIKA_OPENFGA_BIN": str(bin_dir / "seikika"), "PG_START_EXIT": "0" if stage == "m2" else "42",
        }
        run = subprocess.run(["bash", f"scripts/{stage}-acceptance.sh"], cwd=ROOT, env=env, timeout=10,
                             capture_output=True, text=True)
        required = ["fixture postgres diagnostic"] + (["fixture server diagnostic"] if stage == "m2" else [])
        if run.returncode != 42 or any(message not in run.stderr for message in required):
            raise ValueError(f"{stage} did not preserve the failure and diagnostics: {run.returncode}\n{run.stderr}")
        if any(path.is_dir() for path in tmp.iterdir()):
            raise ValueError(f"{stage} left its temporary directory")
        print(f"{stage}: exit 42, expected diagnostics emitted, temporary directory removed")


def sdk(_):
    output = subprocess.check_output(["node", "scripts/sdk-docs.mjs"], cwd=ROOT, timeout=240)
    records = [json.loads(line) for line in output.splitlines()]
    print(output.decode(), end="")
    expected = {"seikika-connect", "@seikika/ui", "@seikika/astro-report"}
    if expected - {r["data"]["name"] for r in records} or any(
        r["data"]["undocumented"] or r["data"]["referenceGaps"] for r in records
    ):
        raise ValueError("SDK coverage check found undocumented entry points or reference gaps")


def main():
    cases = {"onboarding": onboarding, "acceptance": acceptance, "sdk": sdk}
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("case", choices=cases)
    args = parser.parse_args()
    with TemporaryDirectory(prefix="seikika-probe-") as directory:
        cases[args.case](Path(directory))


if __name__ == "__main__":
    main()
