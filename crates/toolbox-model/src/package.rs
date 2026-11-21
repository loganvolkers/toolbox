//! Toolbox packages (D89): standards as data, included by a portal with typed parameters.
//!
//! An `include` entry may be `{package: <spec>, with: {param: value}}`. The spec is a directory
//! relative to the portal (`../packages/sdlc`) or `github:owner/repo[/subdir]@ref`, which
//! `toolbox install` clones into the cache (tag `v<semver>`, branch name, or commit SHA) and pins
//! in `toolbox.lock` by content hash. Loading never touches the network: it reads the cached copy
//! and checks the pin.
//!
//! A package directory holds `toolbox-package.yaml`:
//! `{apiVersion: toolbox.dev/package/v0, name, version, description?, parameters?: {name: {type,
//! default, description?, tighten?: lower|higher}}, fragments?: [glob]}` (default
//! `fragments/*.yaml`). Fragments are manifest YAML. Every string `${param:name}` is replaced by
//! the parameter's value (keeping its type when it is the whole string). A parameter with `tighten`
//! may only move in that direction from its default: a portal can make a package stricter, never
//! weaker.

use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

/// A git-hosted package: repository, optional subdirectory, version.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitSpec {
    pub owner: String,
    pub repo: String,
    pub subdir: Option<String>,
    /// Cache key and include pin: leading `v` stripped (so `@v0.1.1` and `@0.1.1` share a checkout).
    pub version: String,
}

/// How `toolbox install` should fetch [`GitSpec::version`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GitRef {
    /// `git clone --branch <name>` (a tag such as `v0.1.1`, or a branch such as `main`).
    Named(String),
    /// Fetch and check out this commit (full or abbreviated SHA).
    Commit(String),
}

/// Classify a parsed include version for clone: semver → `v…` tag, hex SHA → commit, else branch.
pub fn git_ref(version: &str) -> GitRef {
    if (7..=40).contains(&version.len()) && version.bytes().all(|b| b.is_ascii_hexdigit()) {
        return GitRef::Commit(version.to_owned());
    }
    if version.starts_with(|c: char| c.is_ascii_digit()) {
        return GitRef::Named(format!("v{version}"));
    }
    GitRef::Named(version.to_owned())
}

/// Parse `github:owner/repo[/subdir…]@version`.
pub fn parse_git(spec: &str) -> Option<GitSpec> {
    let rest = spec.strip_prefix("github:")?;
    let (path, version) = rest.rsplit_once('@')?;
    let mut parts = path.splitn(3, '/');
    let (owner, repo) = (parts.next()?, parts.next()?);
    if owner.is_empty() || repo.is_empty() || version.is_empty() {
        return None;
    }
    Some(GitSpec {
        owner: owner.into(),
        repo: repo.into(),
        subdir: parts.next().map(str::to_owned),
        version: version.trim_start_matches('v').into(),
    })
}

/// `TOOLBOX_CACHE_DIR`, else `$XDG_CACHE_HOME/toolbox`, else `~/.cache/toolbox`.
pub fn cache_dir() -> PathBuf {
    if let Some(d) = std::env::var_os("TOOLBOX_CACHE_DIR") {
        return d.into();
    }
    if let Some(d) = std::env::var_os("XDG_CACHE_HOME") {
        return PathBuf::from(d).join("toolbox");
    }
    let home = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE"));
    PathBuf::from(home.unwrap_or_default()).join(".cache/toolbox")
}

impl GitSpec {
    /// The cached checkout of the repository at this version.
    pub fn checkout(&self) -> PathBuf {
        cache_dir()
            .join("packages")
            .join(&self.owner)
            .join(&self.repo)
            .join(&self.version)
    }
}

/// The package directory for `spec`, relative to the portal `root`.
pub fn locate(root: &Path, spec: &str) -> PathBuf {
    match parse_git(spec) {
        Some(g) => {
            let dir = g.checkout();
            g.subdir.map_or(dir.clone(), |s| dir.join(s))
        }
        None => root.join(spec),
    }
}

/// SHA-256 over the package's files (sorted relative paths and contents), as pinned in toolbox.lock.
pub fn content_hash(dir: &Path) -> String {
    let mut files: Vec<PathBuf> = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(&d).into_iter().flatten().flatten() {
            let p = e.path();
            if p.file_name().is_some_and(|n| n == ".git") {
                continue;
            }
            if p.is_dir() {
                stack.push(p);
            } else {
                files.push(p);
            }
        }
    }
    files.sort();
    let mut h = Sha256::new();
    for f in files {
        h.update(
            f.strip_prefix(dir)
                .unwrap_or(&f)
                .to_string_lossy()
                .as_bytes(),
        );
        h.update([0]);
        h.update(std::fs::read(&f).unwrap_or_default());
    }
    h.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

/// Check `with` against the declared parameters and return every parameter's value.
fn values(
    params: &Map<String, Value>,
    with: &Map<String, Value>,
) -> Result<Map<String, Value>, String> {
    if let Some(k) = with.keys().find(|k| !params.contains_key(*k)) {
        return Err(format!("unknown parameter `{k}`"));
    }
    let mut out = Map::new();
    for (name, p) in params {
        let default = p.get("default").cloned().unwrap_or(Value::Null);
        let v = with.get(name).cloned().unwrap_or_else(|| default.clone());
        let ok = match p["type"].as_str().unwrap_or("string") {
            "integer" => v.is_i64() || v.is_u64(),
            "number" => v.is_number(),
            "boolean" => v.is_boolean(),
            _ => v.is_string(),
        };
        if !ok {
            return Err(format!("parameter `{name}` must be {}", p["type"]));
        }
        if let (Some(dir), Some(a), Some(b)) = (p["tighten"].as_str(), v.as_f64(), default.as_f64())
        {
            let weaker = match dir {
                "lower" => a > b,
                "higher" => a < b,
                _ => false,
            };
            if weaker {
                return Err(format!(
                    "parameter `{name}` = {v} weakens the package's default {default} (it may only go {dir})"
                ));
            }
        }
        out.insert(name.clone(), v);
    }
    Ok(out)
}

/// Replace `${param:name}` in every string; a string that is exactly one reference takes the value's type.
fn substitute(v: &mut Value, vals: &Map<String, Value>) -> Result<(), String> {
    match v {
        Value::String(s) => {
            if let Some(name) = s.strip_prefix("${param:").and_then(|r| r.strip_suffix('}'))
                && !name.contains('}')
            {
                *v = vals
                    .get(name)
                    .cloned()
                    .ok_or(format!("unknown parameter `{name}` in a fragment"))?;
                return Ok(());
            }
            while let Some(i) = s.find("${param:") {
                let end = s[i..].find('}').ok_or("unterminated ${param:…}")? + i;
                let name = &s[i + 8..end];
                let val = vals
                    .get(name)
                    .ok_or(format!("unknown parameter `{name}` in a fragment"))?;
                let text = val.as_str().map_or_else(|| val.to_string(), str::to_owned);
                s.replace_range(i..=end, &text);
            }
            Ok(())
        }
        Value::Array(a) => a.iter_mut().try_for_each(|x| substitute(x, vals)),
        Value::Object(o) => o.values_mut().try_for_each(|x| substitute(x, vals)),
        _ => Ok(()),
    }
}

/// The package's fragments, parameters applied: `(label, document)` in fragment path order.
pub fn fragments(dir: &Path, with: &Map<String, Value>) -> Result<Vec<(String, Value)>, String> {
    let text = std::fs::read_to_string(dir.join("toolbox-package.yaml")).map_err(|e| {
        format!(
            "{}: {e} (run `toolbox install` for git packages)",
            dir.join("toolbox-package.yaml").display()
        )
    })?;
    let meta: Value =
        serde_norway::from_str(&text).map_err(|e| format!("toolbox-package.yaml: {e}"))?;
    if meta["apiVersion"] != "toolbox.dev/package/v0" {
        return Err("toolbox-package.yaml: apiVersion must be toolbox.dev/package/v0".into());
    }
    let params = meta["parameters"].as_object().cloned().unwrap_or_default();
    let vals = values(&params, with)?;
    let globs: Vec<String> = meta["fragments"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|g| g.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_else(|| vec!["fragments/*.yaml".into()]);
    let mut out = Vec::new();
    for g in globs {
        for rel in crate::load::expand_in(dir, &g) {
            let t = std::fs::read_to_string(dir.join(&rel)).map_err(|e| format!("{rel}: {e}"))?;
            let mut doc: Value = serde_norway::from_str(&t).map_err(|e| format!("{rel}: {e}"))?;
            substitute(&mut doc, &vals).map_err(|e| format!("{rel}: {e}"))?;
            out.push((
                format!("{}:{rel}", meta["name"].as_str().unwrap_or("package")),
                doc,
            ));
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parameters_type_check_and_only_tighten() {
        let params = json!({"max": {"type": "integer", "default": 20, "tighten": "lower"}, "label": {"type": "string", "default": "x"}});
        let p = params.as_object().unwrap();
        let ok = values(p, json!({"max": 10}).as_object().unwrap()).unwrap();
        assert_eq!(ok["max"], 10);
        assert!(
            values(p, json!({"max": 30}).as_object().unwrap())
                .unwrap_err()
                .contains("weakens")
        );
        assert!(values(p, json!({"max": "a"}).as_object().unwrap()).is_err());
        assert!(
            values(p, json!({"nope": 1}).as_object().unwrap())
                .unwrap_err()
                .contains("unknown")
        );
        let mut doc = json!({"check": "entity.n <= ${param:max}", "n": "${param:max}", "l": ["${param:label}-y"]});
        substitute(&mut doc, &ok).unwrap();
        assert_eq!(
            doc,
            json!({"check": "entity.n <= 10", "n": 10, "l": ["x-y"]})
        );
    }

    #[test]
    fn git_specs() {
        let g = parse_git("github:acme/standards/ci@v1.2.0").unwrap();
        assert_eq!(
            (g.subdir.as_deref(), g.version.as_str()),
            (Some("ci"), "1.2.0")
        );
        assert_eq!(parse_git("github:acme/standards@1").unwrap().subdir, None);
        assert_eq!(parse_git("../packages/x"), None);
    }

    #[test]
    fn git_ref_tag_branch_or_commit() {
        assert_eq!(git_ref("1.2.0"), GitRef::Named("v1.2.0".into()));
        assert_eq!(git_ref("0.1.1"), GitRef::Named("v0.1.1".into()));
        assert_eq!(git_ref("main"), GitRef::Named("main".into()));
        assert_eq!(git_ref("develop"), GitRef::Named("develop".into()));
        let sha = "e213947a1b2c3d4e5f678901234567890abcdef0";
        assert_eq!(git_ref(sha), GitRef::Commit(sha.into()));
        assert_eq!(git_ref("e213947"), GitRef::Commit("e213947".into()));
    }
}
