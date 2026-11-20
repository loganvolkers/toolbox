//! `toolbox install` and `toolbox doctor` (D90): released executable connectors and the upstream
//! tools a portal needs, without building anything.
//!
//! `install` downloads every `github:owner/repo[/app]@version` connector for this platform, checks
//! the archive's SHA-256 against `toolbox.lock` (recording it from the release's published `.sha256`
//! the first time, unless `--frozen`), and unpacks the binary into the cache. `doctor` reports
//! whether each connector resolves and each declared tool prints its pinned version; `--mise` prints
//! the `[tools]` section that installs them.

use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use toolbox_api::artifacts::{self, Spec};

const LOCK: &str = "toolbox.lock";

fn fail(m: impl std::fmt::Display) -> ExitCode {
    eprintln!("error: {m}");
    ExitCode::from(2)
}

fn manifest(portal: &Path) -> Result<(PathBuf, toolbox_model::Manifest), String> {
    let src = toolbox_model::load::load(portal).map_err(|e| e.to_string())?;
    let m = crate::portal::manifest_from(&src.document, &src.manifest_file)?;
    Ok((src.root, m))
}

fn read_lock(root: &Path) -> Value {
    std::fs::read_to_string(root.join(LOCK))
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_else(|| json!({"format": "toolbox-lock/v1", "connectors": {}, "packages": {}}))
}

fn hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

async fn get(client: &reqwest::Client, url: &str) -> Result<Vec<u8>, String> {
    let r = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("{url}: {e}"))?;
    if !r.status().is_success() {
        return Err(format!("{url}: {}", r.status()));
    }
    Ok(r.bytes().await.map_err(|e| e.to_string())?.to_vec())
}

/// Download, verify and unpack one connector; returns the archive's SHA-256.
async fn install_one(
    client: &reqwest::Client,
    spec: &Spec,
    pinned: Option<&str>,
) -> Result<String, String> {
    let base =
        std::env::var("TOOLBOX_RELEASES_URL").unwrap_or_else(|_| "https://github.com".into());
    let url = spec.url(&base, artifacts::target());
    let archive = get(client, &url).await?;
    let got = hex(&archive);
    let want = match pinned {
        Some(p) => p.to_owned(),
        None => {
            // First install: trust the release's published checksum, and record it in the lock.
            let text =
                String::from_utf8_lossy(&get(client, &format!("{url}.sha256")).await?).to_string();
            text.split_whitespace()
                .next()
                .unwrap_or_default()
                .to_owned()
        }
    };
    if got != want {
        return Err(format!(
            "{url}: SHA-256 {got} doesn't match {want} ({})",
            if pinned.is_some() {
                "toolbox.lock"
            } else {
                "the release's .sha256"
            }
        ));
    }
    let dest = spec.installed();
    let dir = dest.parent().ok_or("no cache directory")?;
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let file = dir.join(spec.asset(artifacts::target()));
    std::fs::write(&file, &archive).map_err(|e| e.to_string())?;
    let ok = std::process::Command::new("tar")
        .arg("-xf")
        .arg(&file)
        .arg("-C")
        .arg(dir)
        .status()
        .map_err(|e| format!("tar: {e}"))?
        .success();
    let _ = std::fs::remove_file(&file);
    if !ok {
        return Err(format!("{url}: could not unpack"));
    }
    let name = dest.file_name().unwrap_or_default().to_owned();
    let found = walkdir::WalkDir::new(dir)
        .into_iter()
        .filter_map(Result::ok)
        .find(|e| e.file_type().is_file() && e.file_name() == name && e.path() != dest)
        .ok_or(format!(
            "{url}: no {} in the archive",
            name.to_string_lossy()
        ))?;
    std::fs::copy(found.path(), &dest).map_err(|e| e.to_string())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&dest, std::fs::Permissions::from_mode(0o755));
    }
    Ok(got)
}

/// Clone each git package into the cache at its tag and check or record its content hash.
fn install_packages(
    root: &Path,
    manifest_file: &str,
    lock: &mut Value,
    frozen: bool,
) -> Result<bool, String> {
    let text = std::fs::read_to_string(root.join(manifest_file)).map_err(|e| e.to_string())?;
    let doc: Value = serde_norway::from_str(&text).map_err(|e| e.to_string())?;
    let base = std::env::var("TOOLBOX_GIT_URL").unwrap_or_else(|_| "https://github.com".into());
    let mut changed = false;
    for spec in doc["include"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|i| i["package"].as_str())
    {
        let Some(g) = toolbox_model::package::parse_git(spec) else {
            continue; // a directory in this repository: nothing to fetch
        };
        let checkout = g.checkout();
        if !checkout.is_dir() {
            let url = format!("{}/{}/{}.git", base.trim_end_matches('/'), g.owner, g.repo);
            let ok = std::process::Command::new("git")
                .args([
                    "clone",
                    "-q",
                    "--depth",
                    "1",
                    "--branch",
                    &format!("v{}", g.version),
                    &url,
                ])
                .arg(&checkout)
                .status()
                .map_err(|e| format!("git: {e}"))?
                .success();
            if !ok {
                return Err(format!("{spec}: could not clone {url} at v{}", g.version));
            }
        }
        let got = toolbox_model::package::content_hash(&toolbox_model::package::locate(root, spec));
        match lock["packages"][spec].as_str() {
            Some(p) if p != got => {
                return Err(format!(
                    "{spec}: content {got} doesn't match toolbox.lock ({p})"
                ));
            }
            Some(_) => {}
            None if frozen => return Err(format!("{spec}: not pinned in {LOCK} (--frozen)")),
            None => {
                lock["packages"][spec] = json!(got);
                changed = true;
            }
        }
        println!("package {spec} → {}", checkout.display());
    }
    Ok(changed)
}

/// `toolbox install`: fetch git packages and released connectors, verify them against
/// toolbox.lock, and pin new ones.
pub fn install(portal: &Path, frozen: bool) -> ExitCode {
    let (root, manifest_file) = match toolbox_model::load::resolve(portal) {
        Ok(x) => x,
        Err(e) => return fail(e),
    };
    let mut lock = read_lock(&root);
    let mut changed = match install_packages(&root, &manifest_file, &mut lock, frozen) {
        Ok(c) => c,
        Err(e) => return fail(e),
    };
    let m = match manifest(portal) {
        Ok((_, m)) => m,
        Err(e) => return fail(e),
    };
    let target = artifacts::target();
    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
    let client = reqwest::Client::new();
    for c in &m.connectors {
        let Some(spec) = artifacts::parse(&c.package) else {
            continue;
        };
        let pinned = lock["connectors"][&c.package][target]
            .as_str()
            .map(str::to_owned);
        if frozen && pinned.is_none() {
            return fail(format!(
                "{}: not pinned in {LOCK} for {target} (--frozen)",
                c.package
            ));
        }
        match rt.block_on(install_one(&client, &spec, pinned.as_deref())) {
            Ok(sha) => {
                println!("{} {} → {}", c.name, c.package, spec.installed().display());
                if pinned.is_none() {
                    lock["connectors"][&c.package][target] = json!(sha);
                    changed = true;
                }
            }
            Err(e) => return fail(format!("connector {}: {e}", c.name)),
        }
    }
    if changed {
        let text = toolbox_derive::pyjson::pretty(&lock);
        if let Err(e) = std::fs::write(root.join(LOCK), text) {
            return fail(e);
        }
        println!("pinned in {}", root.join(LOCK).display());
    }
    ExitCode::SUCCESS
}

/// The first line a tool prints for its version arguments, if it runs.
fn tool_version(name: &str, args: &[String]) -> Option<String> {
    let out = std::process::Command::new(name).args(args).output().ok()?;
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    Some(text.lines().next().unwrap_or_default().trim().to_owned())
}

/// `toolbox doctor`: report connectors and tools (exit 1 on problems), or print the mise config.
pub fn doctor(portal: &Path, mise: bool) -> ExitCode {
    let (_, m) = match manifest(portal) {
        Ok(x) => x,
        Err(e) => return fail(e),
    };
    if mise {
        println!("[tools]");
        for (name, t) in &m.tools {
            let id = t.mise.clone().unwrap_or_else(|| name.clone());
            println!("\"{id}\" = \"{}\"", t.version);
        }
        return ExitCode::SUCCESS;
    }
    let mut problems = 0;
    for c in m.connectors.iter().filter(|c| c.runtime == "in-image") {
        let bin = toolbox_api::connectors::binary_for(&c.package);
        let found = Path::new(&bin).is_file() || which(&bin);
        if !found {
            problems += 1;
        }
        let hint = if artifacts::parse(&c.package).is_some() {
            "run toolbox install"
        } else {
            "build or install it"
        };
        println!(
            "{} connector {} ({}){}",
            if found { "ok     " } else { "MISSING" },
            c.name,
            c.package,
            if found {
                String::new()
            } else {
                format!(": {bin} not found; {hint}")
            }
        );
    }
    for (name, t) in &m.tools {
        let args = t
            .version_args
            .clone()
            .unwrap_or_else(|| vec!["--version".into()]);
        let line = match tool_version(name, &args) {
            Some(v) if v.contains(&t.version) => format!("ok      tool {name} {}", t.version),
            Some(v) => {
                problems += 1;
                format!("WRONG   tool {name}: wants {}, found \"{v}\"", t.version)
            }
            None => {
                problems += 1;
                format!(
                    "MISSING tool {name} {}: install it (toolbox doctor --mise prints the mise config)",
                    t.version
                )
            }
        };
        println!("{line}");
    }
    if problems == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn which(bin: &str) -> bool {
    std::env::var_os("PATH")
        .map(|p| std::env::split_paths(&p).any(|d| d.join(bin).is_file()))
        .unwrap_or(false)
}
