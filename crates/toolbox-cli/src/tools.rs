//! `toolbox tools`: the box's tools, installed by the installer you choose
//! (docs/design/tool-installation.md, D01M800DHD0Z1RTPH7CM46GZS3B).
//!
//! Toolbox doesn't download tools itself (D90): installers already do that well, with checksums,
//! signatures and lockfiles. It owns the pin, one `version` per tool in `tools:`, and derives each
//! installer's input from it. `tools install --via` then runs that installer:
//! - **mise** (`install.mise`, e.g. `aqua:rhysd/actionlint`): any of its backends;
//! - **aqua** (`install.aqua`, e.g. `rhysd/actionlint`): its registry; config only, aqua installs;
//! - **uv**, **pipx** or **pip** (`install.pypi`, e.g. `zizmor`): tools published on PyPI.
//!
//! No installer is required, and none is privileged: a tool can declare several.

use serde_json::Value;
use std::path::Path;
use std::process::{Command, ExitCode};
use toolbox_model::manifest::Tool;

/// The installers Toolbox knows how to drive, in the order `auto` tries them.
const AUTO: &[&str] = &["mise", "uv", "pipx", "pip"];

/// This tool's id for an installer, if it declares one.
pub(crate) fn id_for(t: &Tool, installer: &str) -> Option<String> {
    let key = match installer {
        "uv" | "pipx" | "pip" => "pypi",
        other => other,
    };
    t.install
        .as_ref()
        .and_then(|m| m.get(key))
        .and_then(Value::as_str)
        .map(str::to_owned)
        .or_else(|| (key == "mise").then(|| t.mise.clone()).flatten())
}

fn on_path(program: &str) -> bool {
    std::env::var_os("PATH")
        .map(|p| std::env::split_paths(&p).any(|d| d.join(program).is_file()))
        .unwrap_or(false)
}

/// The program an installer is run as, and whether it is available here.
fn available(installer: &str) -> bool {
    match installer {
        "pip" => on_path("python3"),
        other => on_path(other),
    }
}

/// The command that installs one tool with one installer.
fn command(installer: &str, id: &str, version: &str) -> Vec<String> {
    let pinned = format!("{id}=={version}");
    let words: Vec<&str> = match installer {
        "mise" => return vec!["mise".into(), "install".into(), format!("{id}@{version}")],
        "uv" => vec!["uv", "tool", "install", &pinned],
        "pipx" => vec!["pipx", "install", &pinned],
        _ => vec!["python3", "-m", "pip", "install", &pinned],
    };
    words.into_iter().map(str::to_owned).collect()
}

/// `toolbox tools config --for <installer>`: the installer's input, from the portal's pins.
pub fn config(portal: &Path, installer: &str) -> ExitCode {
    let m = match crate::install::manifest(portal) {
        Ok((_, m)) => m,
        Err(e) => return crate::install::fail(e),
    };
    let mut missing = Vec::new();
    let mut lines = Vec::new();
    for (name, t) in &m.tools {
        let Some(id) = id_for(t, installer) else {
            missing.push(name.clone());
            continue;
        };
        lines.push(match installer {
            "mise" => format!("\"{id}\" = \"{}\"", t.version),
            "aqua" => format!("  - name: {id}@v{}", t.version.trim_start_matches('v')),
            _ => format!("{id}=={}", t.version),
        });
    }
    match installer {
        "mise" => println!("[tools]"),
        "aqua" => println!(
            "# aqua.yaml: pin the standard registry to a release (aqua's docs: `aqua init`)\nregistries:\n  - type: standard\n    ref: <registry release>\npackages:"
        ),
        "pypi" | "uv" | "pipx" | "pip" => {}
        other => {
            return crate::install::fail(format!(
                "--for {other}: expected mise, aqua or pypi (requirements for uv, pipx or pip)"
            ));
        }
    }
    for l in lines {
        println!("{l}");
    }
    for name in missing {
        eprintln!("note: {name} declares no {installer} id (tools.{name}.install)");
    }
    ExitCode::SUCCESS
}

/// Whether the tool is already on the PATH at its pinned version.
fn installed(name: &str, t: &Tool) -> bool {
    let args = t
        .version_args
        .clone()
        .unwrap_or_else(|| vec!["--version".into()]);
    crate::install::tool_version(name, &args).is_some_and(|v| v.contains(&t.version))
}

/// The installer for one tool: the one asked for, or the first of `AUTO` that is available and
/// that the tool declares an id for.
fn choose(t: &Tool, via: &str) -> Result<(String, String), String> {
    if via != "auto" {
        if via == "aqua" {
            return Err("aqua installs from its own aqua.yaml: run `toolbox tools config --for aqua`, then `aqua i`".into());
        }
        let id = id_for(t, via).ok_or(format!("declares no {via} id"))?;
        return Ok((via.to_owned(), id));
    }
    AUTO.iter()
        .filter(|i| available(i))
        .find_map(|i| id_for(t, i).map(|id| ((*i).to_owned(), id)))
        .ok_or_else(|| {
            "no installer available for it: install one of mise, uv, pipx or pip, or declare an id for one that is (tools.<name>.install)".into()
        })
}

/// `toolbox tools install`: install every tool the portal pins that isn't already there, with
/// the installer asked for (`--via`) or the first one available (`auto`).
pub fn install(portal: &Path, via: &str, dry_run: bool) -> ExitCode {
    let m = match crate::install::manifest(portal) {
        Ok((_, m)) => m,
        Err(e) => return crate::install::fail(e),
    };
    let mut failed = 0;
    for (name, t) in &m.tools {
        if installed(name, t) {
            println!("ok      {name} {} (already installed)", t.version);
            continue;
        }
        let (installer, id) = match choose(t, via) {
            Ok(x) => x,
            Err(e) => {
                failed += 1;
                println!("SKIP    {name} {}: {e}", t.version);
                continue;
            }
        };
        let argv = command(&installer, &id, &t.version);
        println!(
            "install {name} {} via {installer}: {}",
            t.version,
            argv.join(" ")
        );
        if dry_run {
            continue;
        }
        let ok = Command::new(&argv[0])
            .args(&argv[1..])
            .status()
            .is_ok_and(|s| s.success());
        if !ok {
            failed += 1;
            println!("FAILED  {name}: {} exited with an error", argv[0]);
        }
    }
    if !dry_run {
        println!(
            "then: toolbox doctor {} (checks every pin)",
            portal.display()
        );
    }
    if failed == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn tool(v: Value) -> Tool {
        serde_json::from_value(v).unwrap()
    }

    #[test]
    fn ids_per_installer_and_the_legacy_mise_key() {
        let t = tool(
            json!({"version": "1.32.0", "mise": "pipx:zizmor", "install": {"pypi": "zizmor", "aqua": "zizmorcore/zizmor"}}),
        );
        assert_eq!(id_for(&t, "mise").as_deref(), Some("pipx:zizmor"));
        assert_eq!(id_for(&t, "uv").as_deref(), Some("zizmor"));
        assert_eq!(id_for(&t, "aqua").as_deref(), Some("zizmorcore/zizmor"));
        let t = tool(json!({"version": "1.7.12", "install": {"mise": "aqua:rhysd/actionlint"}}));
        assert_eq!(id_for(&t, "mise").as_deref(), Some("aqua:rhysd/actionlint"));
        assert_eq!(id_for(&t, "pip"), None);
    }

    #[test]
    fn each_installer_gets_the_pin() {
        assert_eq!(
            command("mise", "aqua:rhysd/actionlint", "1.7.12"),
            ["mise", "install", "aqua:rhysd/actionlint@1.7.12"]
        );
        assert_eq!(
            command("uv", "ruff", "0.14.0"),
            ["uv", "tool", "install", "ruff==0.14.0"]
        );
        assert_eq!(
            command("pip", "ruff", "0.14.0"),
            ["python3", "-m", "pip", "install", "ruff==0.14.0"]
        );
        let t = tool(json!({"version": "1.7.12", "install": {"mise": "aqua:rhysd/actionlint"}}));
        assert!(choose(&t, "aqua").unwrap_err().contains("aqua i"));
        assert!(choose(&t, "pip").unwrap_err().contains("no pip id"));
    }
}
