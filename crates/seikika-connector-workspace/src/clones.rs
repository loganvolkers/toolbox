//! Child portals by git repository (`"git": [{key, url, branch}]`, from a parent's
//! `federation.children`): each is cloned into `<root>/<key>` and kept at its branch, so the
//! `portal` records are read from what is committed there and nothing else. A clone that cannot be
//! brought to its branch is a `portal` record with an error, which offers no actions.

use serde_json::{Value, json};
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone)]
pub(crate) struct Source {
    pub key: String,
    pub url: String,
    pub branch: String,
}

pub(crate) fn sources(cfg: &Value) -> Vec<Source> {
    cfg["git"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|g| {
            Some(Source {
                key: g["key"].as_str()?.to_owned(),
                url: g["url"].as_str()?.to_owned(),
                branch: g["branch"].as_str().unwrap_or("main").to_owned(),
            })
        })
        .collect()
}

fn git(dir: &Path, args: &[&str]) -> Result<String, String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .env("GIT_TERMINAL_PROMPT", "0")
        .output()
        .map_err(|e| format!("git: {e}"))?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_owned())
    } else {
        Err(format!(
            "git {}: {}",
            args.first().copied().unwrap_or(""),
            String::from_utf8_lossy(&out.stderr).trim()
        ))
    }
}

/// Bring `<root>/<key>` to the tip of the source's branch: clone it, or fetch and reset it. A
/// clone of another URL is replaced. Returns the commit.
pub(crate) fn sync(root: &Path, s: &Source) -> Result<String, String> {
    let dir = root.join(&s.key);
    if dir.join(".git").is_dir()
        && git(&dir, &["remote", "get-url", "origin"]).ok().as_deref() != Some(s.url.as_str())
    {
        std::fs::remove_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    if !dir.join(".git").is_dir() {
        std::fs::create_dir_all(root).map_err(|e| format!("{}: {e}", root.display()))?;
        git(root, &["clone", "-q", "--", &s.url, &s.key])?;
    }
    // Commits a parent makes here (a `change` that lands) are the portal's, not whoever the
    // host's global git config names; `GIT_AUTHOR_*` and `GIT_COMMITTER_*` still override.
    git(&dir, &["config", "user.name", "Seikika"])?;
    git(&dir, &["config", "user.email", "seikika@localhost"])?;
    git(&dir, &["fetch", "-q", "origin", "--", &s.branch])?;
    git(&dir, &["checkout", "-q", "-B", &s.branch, "FETCH_HEAD"])?;
    git(&dir, &["reset", "-q", "--hard", "FETCH_HEAD"])?;
    git(&dir, &["clean", "-qfdx"])?;
    git(&dir, &["rev-parse", "HEAD"])
}

/// The manifest path (relative to the clones' root) a child's `portal` record is read from.
pub(crate) fn manifest(s: &Source) -> String {
    format!("{}/.seikika/seikika.yaml", s.key)
}

/// What a parent needs to sync a child again before a run: where it comes from and the commit
/// its record was read at.
pub(crate) fn origin(s: &Source, commit: &str) -> Value {
    json!({"url": s.url, "branch": s.branch, "commit": commit})
}

/// The record for a child that could not be read: no actions, and why.
pub(crate) fn broken(root: &Path, s: &Source, error: String) -> Value {
    json!({"name": s.key, "key": s.key, "path": format!("{}/.seikika", s.key),
        "dir": root.join(&s.key).join(".seikika").display().to_string(),
        "file": manifest(s), "error": error, "title": null, "owner": null,
        "actions": [], "scorecards": [], "packages": [],
        "runner": null, "git": {"url": s.url, "branch": s.branch}})
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(dir: &Path, args: &[&str]) {
        assert!(
            Command::new("git")
                .arg("-C")
                .arg(dir)
                .args(args)
                .status()
                .unwrap()
                .success()
        );
    }

    #[test]
    fn a_clone_follows_its_branch() {
        let t = std::env::temp_dir().join(format!("clones-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&t);
        let (origin, work, root) = (t.join("o.git"), t.join("w"), t.join("clones"));
        std::fs::create_dir_all(&work).unwrap();
        run(&t, &["init", "-q", "--bare", "-b", "main", "o.git"]);
        run(&work, &["init", "-q", "-b", "main"]);
        for args in [
            ["config", "user.name", "t"],
            ["config", "user.email", "t@t"],
        ] {
            run(&work, &args);
        }
        std::fs::write(work.join("a"), "1").unwrap();
        run(&work, &["add", "a"]);
        run(&work, &["commit", "-qm", "one"]);
        run(&work, &["push", "-q", origin.to_str().unwrap(), "main"]);
        let s = Source {
            key: "alpha".into(),
            url: origin.display().to_string(),
            branch: "main".into(),
        };
        let first = sync(&root, &s).unwrap();
        std::fs::write(root.join("alpha/stray"), "x").unwrap();
        std::fs::write(work.join("a"), "2").unwrap();
        run(&work, &["commit", "-qam", "two"]);
        run(&work, &["push", "-q", origin.to_str().unwrap(), "main"]);
        let second = sync(&root, &s).unwrap();
        assert_ne!(first, second);
        assert_eq!(std::fs::read_to_string(root.join("alpha/a")).unwrap(), "2");
        assert!(!root.join("alpha/stray").exists());
        let missing = Source {
            branch: "nope".into(),
            ..s
        };
        assert!(sync(&root, &missing).is_err());
        std::fs::remove_dir_all(&t).ok();
    }
}
