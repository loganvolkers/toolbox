#!/usr/bin/env python3
"""Check the packaged SDK and exercise Seikika installed from crates.io."""

import argparse
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import tarfile
import tempfile
import tomllib
from urllib.request import Request, urlopen

REPO = Path(__file__).resolve().parents[1]
CLI = "seikika-cli"
SDK = "seikika-connect"
CONNECTOR_CONFIGS = {
    "git-files": {"org": "acme"},
    "github": {"org": "acme"},
    "gitlab": {"group": "acme"},
    "runner": {},
}
BINARIES = [
    CLI,
    *(f"seikika-connector-{name}" for name in CONNECTOR_CONFIGS),
    "seikika-connector-scim",
    "seikika-connector-workspace",
]


def latest_version():
    request = Request(
        "https://index.crates.io/se/ik/seikika-cli",
        headers={"User-Agent": "Seikika registry smoke (https://github.com/seikika/seikika)"},
    )
    with urlopen(request, timeout=30) as response:
        entries = [json.loads(line) for line in response]
    versions = [
        item["vers"] for item in entries
        if not item["yanked"] and re.fullmatch(r"[0-9]+\.[0-9]+\.[0-9]+", item["vers"])
    ]
    if not versions:
        raise SystemExit("crates.io has no stable, non-yanked seikika-cli release")
    return max(versions, key=lambda version: tuple(map(int, version.split("."))))


def cargo_env():
    env = os.environ.copy()
    env.pop("CARGO_REGISTRY_TOKEN", None)
    with (REPO / "rust-toolchain.toml").open("rb") as manifest:
        env.setdefault("RUSTUP_TOOLCHAIN", tomllib.load(manifest)["toolchain"]["channel"])
    env.update(CARGO_PROFILE_DEV_DEBUG="0", CARGO_INCREMENTAL="0")
    return env


def copy_scaffold(destination, dependency):
    consumer = destination / "consumer"
    scaffold = REPO / "examples/connector-scaffold"
    shutil.copytree(scaffold / "src", consumer / "src")
    manifest = (scaffold / "Cargo.toml").read_text(encoding="utf-8")
    manifest, replacements = re.subn(
        r"(?m)^seikika-connect = .*?$", lambda _: f"seikika-connect = {dependency}", manifest
    )
    if replacements != 1:
        raise SystemExit("Expected one seikika-connect dependency in the connector scaffold")
    return consumer, manifest


def check_packaged_sdk(archives, work):
    with (REPO / "Cargo.toml").open("rb") as manifest:
        version = tomllib.load(manifest)["workspace"]["package"]["version"]
    packages = [SDK, "seikika-model", "seikika-expr"]
    for package in packages:
        with tarfile.open(archives / f"{package}-{version}.crate") as archive:
            archive.extractall(work, filter="data")

    # The SDK's normalized manifest refers to registry dependencies. Point its
    # unpublished lockstep dependencies at their normalized archives as well.
    consumer, manifest = copy_scaffold(work, f'{{ path = "../{SDK}-{version}" }}')
    manifest += "\n[patch.crates-io]\n"
    for package in packages[1:]:
        manifest += f'{package} = {{ path = "../{package}-{version}" }}\n'
    (consumer / "Cargo.toml").write_text(manifest, encoding="utf-8")
    subprocess.run(["cargo", "check"], cwd=consumer, env=cargo_env(), check=True)


def smoke_registry(version, resolution, full, work):
    env = cargo_env()
    env.update(CARGO_HOME=str(work / "cargo"), CARGO_TARGET_DIR=str(work / "target"))
    install = work / "install"
    command = [
        "cargo", "install", "--debug", "--registry", "crates-io",
        "--version", f"={version}", "--root", str(install),
    ]
    if resolution == "locked":
        command.append("--locked")
    subprocess.run([*command, *BINARIES], cwd=work, env=env, check=True)

    suffix = ".exe" if os.name == "nt" else ""
    bin_dir = install / "bin"
    cli = bin_dir / f"seikika{suffix}"
    starter = REPO / "examples/repo-portal-starter"
    reference = REPO / "examples/reference-portal"
    installed = subprocess.check_output([cli, "--version"], cwd=work, text=True).strip()
    if installed != f"seikika {version}":
        raise SystemExit(f"Expected seikika {version}, got {installed}")
    print(installed, flush=True)
    subprocess.run([cli, "validate", reference], cwd=work, check=True)

    fixture = work / "repo-fixture"
    fixture.mkdir()
    (fixture / "README.md").write_text("# Registry smoke fixture\n", encoding="utf-8")
    subprocess.run(
        [cli, "connector", "test", "--name", "workspace", "--portal", starter,
         "--config", json.dumps({"root": str(fixture)}),
         "--", bin_dir / f"seikika-connector-workspace{suffix}"],
        cwd=work, check=True, timeout=60,
    )
    for name, config in CONNECTOR_CONFIGS.items():
        result = subprocess.run(
            [bin_dir / f"seikika-connector-{name}{suffix}"], input="",
            cwd=work, env={**env, "SEIKIKA_CONNECTOR_CONFIG": json.dumps(config)},
            text=True, capture_output=True, check=True, timeout=30,
        )
        registration = json.loads(result.stdout)
        if (registration.get("type"), registration.get("connector"), registration.get("version")) != (
            "register", name, version
        ):
            raise SystemExit(f"Unexpected {name} registration: {registration}")
        print(f"{name}: registered over stdio", flush=True)

    # SCIM has a separate HTTP entry point and requires a server URL.
    result = subprocess.run(
        [bin_dir / f"seikika-connector-scim{suffix}"], cwd=work,
        text=True, capture_output=True, timeout=30,
    )
    if result.returncode != 2 or "usage: seikika-connector-scim --seikika" not in result.stderr:
        raise SystemExit(f"SCIM did not start and print usage: {result.stderr}")

    if full:
        subprocess.run([cli, "test", reference], cwd=work, env=env, check=True, timeout=120)
        subprocess.run(
            ["bash", REPO / "scripts/m4-connectors-smoke.sh"],
            env={**env, "T": str(bin_dir), "SEIKIKA_STATE_DIR": str(work / "state")},
            check=True, timeout=180,
        )


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    commands.add_parser("latest", help="print the latest non-yanked stable CLI version")
    sdk = commands.add_parser("sdk", help="compile the scaffold with packaged SDK archives")
    sdk.add_argument("archives", type=Path)
    smoke = commands.add_parser("install", help="install and exercise one public version")
    smoke.add_argument("version")
    smoke.add_argument("--resolution", choices=("locked", "fresh"), default="locked")
    smoke.add_argument("--full", action="store_true", help="run the Linux integration tests")
    args = parser.parse_args()
    if args.command == "latest":
        print(latest_version())
        return

    with tempfile.TemporaryDirectory(prefix="seikika-crates-") as temp:
        work = Path(temp)
        if args.command == "sdk":
            check_packaged_sdk(args.archives, work)
        else:
            smoke_registry(args.version, args.resolution, args.full, work)


if __name__ == "__main__":
    main()
