#!/usr/bin/env python3
"""Check mise's maintainer tools and the operator install contract."""

import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import tomllib


ROOT = Path(__file__).resolve().parents[1]
STARTER = ROOT / "examples/repo-portal-starter"
REPOSITORY = "seikika/seikika"
APPS = {"seikika": "seikika-cli", "seikika-connector-workspace": "seikika-connector-workspace"}
TARGETS = {
    "linux-arm64": "aarch64-unknown-linux-gnu",
    "linux-x64": "x86_64-unknown-linux-gnu",
    "macos-arm64": "aarch64-apple-darwin",
    "macos-x64": "x86_64-apple-darwin",
    "windows-x64": "x86_64-pc-windows-msvc",
}


def archive_name(app, platform):
    ext = "zip" if platform == "windows-x64" else "tar.xz"
    return f"{app}-{TARGETS[platform]}.{ext}"


def check_maintainer(config):
    rust_pin = tomllib.loads((ROOT / "rust-toolchain.toml").read_text())["toolchain"]["channel"]
    selected_rust = subprocess.check_output(["mise", "current", "rust"], text=True).strip()
    actual_rust = subprocess.check_output(["rustc", "--version"], text=True).split()[1]
    if selected_rust != rust_pin or actual_rust != rust_pin:
        raise SystemExit(f"mise must select Rust {rust_pin} from rust-toolchain.toml")
    pnpm_pin = json.loads((ROOT / "package.json").read_text())["packageManager"]
    config_pnpm = tomllib.loads((ROOT / "mise.toml").read_text())["tools"]["pnpm"]
    pnpm_version = subprocess.check_output(["pnpm", "--version"], text=True).strip()
    if config_pnpm != pnpm_version or f"pnpm@{config_pnpm}" != pnpm_pin:
        raise SystemExit(
            f"mise pins pnpm@{config_pnpm} and runs pnpm@{pnpm_version}, but package.json requires {pnpm_pin}"
        )

    dist_config = tomllib.loads((ROOT / "dist-workspace.toml").read_text())["dist"]
    if set(dist_config["targets"]) != set(TARGETS.values()):
        raise SystemExit("Update the mise smoke test platform mapping to match dist-workspace.toml")
    plan = json.loads(subprocess.check_output(
        ["dist", "plan", "--output-format=json"], cwd=ROOT, text=True,
    ))
    artifacts = plan["artifacts"]
    for tool, app in APPS.items():
        matching = config["tools"][tool]["matching"]
        for platform, target in TARGETS.items():
            expected_archive = archive_name(app, platform)
            candidates = [
                name for name, artifact in artifacts.items()
                if artifact["kind"] == "executable-zip"
                and target in artifact["target_triples"] and matching in name
            ]
            if candidates != [expected_archive]:
                raise SystemExit(f"{tool} on {platform} should match {expected_archive}, found {candidates}")
            binary = tool + (".exe" if platform == "windows-x64" else "")
            if not any(asset.get("path") == binary for asset in artifacts[expected_archive]["assets"]):
                raise SystemExit(f"{expected_archive} is missing {binary}")
            if f"{expected_archive}.sha256" not in artifacts:
                raise SystemExit(f"{expected_archive} is missing its checksum artifact")
    print(f"mise selected Rust {rust_pin} and {pnpm_pin}. Release plan matches all starter archives.")


def check_release(config_text, pinned_version, tag):
    if not tag and pinned_version == "0.2.1":
        # The first branded install cannot run before the tag exists. Once pushed,
        # even a missing GitHub release must fail the check.
        tagged = subprocess.run(
            ["git", "ls-remote", "--exit-code", "--refs", f"https://github.com/{REPOSITORY}.git",
             "refs/tags/v0.2.1"],
            stdout=subprocess.DEVNULL,
        )
        if tagged.returncode == 2:
            print("v0.2.1 has not been tagged. The live install starts with the first branded release.")
            return
        tagged.check_returncode()

    version = tag.removeprefix("v") if tag else pinned_version
    if version != pinned_version:
        # A new release can be tested before it becomes the version recommended in the docs.
        config_text = config_text.replace(f'version = "{pinned_version}"', f'version = "{version}"')
        tools = tomllib.loads(config_text)["tools"]
        if any(tools[tool]["version"] != version for tool in APPS):
            raise SystemExit(f"Could not set starter pins to {version}")

    with tempfile.TemporaryDirectory(prefix="seikika-mise-") as tmp:
        work = Path(tmp)
        project = work / "portal"
        project.mkdir()
        (project / ".seikika").mkdir()
        shutil.copy(STARTER / "seikika.yaml", project / ".seikika")
        (project / "mise.toml").write_text(config_text)
        env = os.environ.copy()
        for name in ("GH_TOKEN", "GITHUB_TOKEN", "MISE_GITHUB_TOKEN", "SEIKIKA_CONNECTORS_DIR"):
            env.pop(name, None)
        for name in ("CACHE", "CONFIG", "DATA", "STATE"):
            env[f"MISE_{name}_DIR"] = str(work / name.lower())

        def run(*args, **kwargs):
            return subprocess.run(args, cwd=project, env=env, check=True, text=True, **kwargs)

        run("git", "init", "--quiet")
        run("mise", "trust")
        run("mise", "install")
        for tool in APPS:
            executable = Path(run("mise", "which", tool, capture_output=True).stdout.strip())
            if not executable.is_relative_to(work):
                raise SystemExit(f"mise resolved {tool} outside the clean install: {executable}")
        actual_version = run("mise", "exec", "--", "seikika", "--version", capture_output=True).stdout.strip()
        if actual_version != f"seikika {version}":
            raise SystemExit(f"Expected seikika {version}, got {actual_version}")
        run("mise", "exec", "--", "seikika", "doctor", ".seikika")
        run("mise", "exec", "--", "seikika", "report", ".seikika", "--baseline", ".seikika/results.json", "--update")
        run("mise", "exec", "--", "seikika", "report", ".seikika", "--baseline", ".seikika/results.json", "--ci")

        run("mise", "lock", "--platform", ",".join(TARGETS))
        locked_tools = tomllib.loads((project / "mise.lock").read_text())["tools"]
        for tool, app in APPS.items():
            [locked] = locked_tools[tool]
            for platform in TARGETS:
                asset = locked.get(f"platforms.{platform}", {})
                name = asset.get("url", "").rsplit("/", 1)[-1]
                if name != archive_name(app, platform) or not asset.get("checksum"):
                    raise SystemExit(f"mise did not lock the expected archive and checksum for {tool} on {platform}")
        print(f"Installed Seikika {version}, scored the starter, and resolved archives for every target.")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("check", choices=("maintainer", "release"))
    parser.add_argument("tag", nargs="?", help="Published release to test. Defaults to the starter pin.")
    args = parser.parse_args()

    config_text = (STARTER / "mise.example.toml").read_text()
    config = tomllib.loads(config_text)
    for tool in APPS:
        if config["tool_alias"].get(tool) != f"github:{REPOSITORY}":
            raise SystemExit(f"{tool} must install from {REPOSITORY} releases")
    versions = {config["tools"][tool]["version"] for tool in APPS}
    if len(versions) != 1:
        raise SystemExit(f"The starter CLI and workspace connector must pin one release, found {sorted(versions)}")
    if args.check == "maintainer":
        check_maintainer(config)
    else:
        check_release(config_text, versions.pop(), args.tag)


if __name__ == "__main__":
    main()
