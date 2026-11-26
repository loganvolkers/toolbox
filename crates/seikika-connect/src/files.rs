//! Files in a local checkout, for connectors that read one (`workspace`, `runner`): globs over
//! `/`-separated relative paths and a walk that skips build and VCS directories.

use regex::Regex;
use std::path::Path;

/// Directories never walked.
pub const SKIP_DIRS: &[&str] = &["target", "node_modules", ".git", "dist", ".venv"];

/// A glob over `/`-separated relative paths: `**` spans directories, `*` and `?` don't.
pub fn glob(pattern: &str) -> Result<Regex, String> {
    let mut re = String::from("^");
    let mut chars = pattern.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '*' if chars.peek() == Some(&'*') => {
                chars.next();
                if chars.peek() == Some(&'/') {
                    chars.next();
                    re.push_str("(?:.*/)?");
                } else {
                    re.push_str(".*");
                }
            }
            '*' => re.push_str("[^/]*"),
            '?' => re.push_str("[^/]"),
            c => re.push_str(&regex::escape(&c.to_string())),
        }
    }
    re.push('$');
    Regex::new(&re).map_err(|e| format!("glob {pattern}: {e}"))
}

/// Relative paths (sorted) of every file under `root`, skipping [`SKIP_DIRS`]. In a git work tree
/// these are the files git sees (tracked, or untracked and not ignored), so a developer's checkout
/// scores like a fresh clone; elsewhere, or when git lists nothing, every file on disk.
pub fn walk(root: &Path) -> Vec<String> {
    match git_files(root) {
        Some(out) if !out.is_empty() => out,
        _ => walk_disk(root),
    }
}

fn git_files(root: &Path) -> Option<Vec<String>> {
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(root)
        .args([
            "ls-files",
            "-z",
            "--cached",
            "--others",
            "--exclude-standard",
        ])
        .stderr(std::process::Stdio::null())
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let mut paths: Vec<String> = String::from_utf8_lossy(&out.stdout)
        .split('\0')
        .filter(|p| !p.is_empty())
        .filter(|p| {
            let dirs = p.rsplit_once('/').map_or("", |(d, _)| d);
            !dirs.split('/').any(|seg| SKIP_DIRS.contains(&seg))
        })
        .filter(|p| root.join(p).is_file())
        .map(str::to_owned)
        .collect();
    paths.sort();
    paths.dedup();
    Some(paths)
}

fn walk_disk(root: &Path) -> Vec<String> {
    let mut out: Vec<String> = walkdir::WalkDir::new(root)
        .into_iter()
        .filter_entry(|e| {
            !(e.file_type().is_dir()
                && SKIP_DIRS.contains(&e.file_name().to_string_lossy().as_ref()))
        })
        .flatten()
        .filter(|e| e.file_type().is_file())
        .filter_map(|e| {
            e.path()
                .strip_prefix(root)
                .ok()
                .map(|p| p.to_string_lossy().replace('\\', "/"))
        })
        .collect();
    out.sort();
    out
}

/// The files under `root` matching any of `globs`.
pub fn matching(root: &Path, globs: &[Regex]) -> Vec<String> {
    walk(root)
        .into_iter()
        .filter(|p| globs.iter().any(|g| g.is_match(p)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{glob, walk};

    #[test]
    fn walk_skips_what_git_ignores() {
        let dir = std::env::temp_dir().join(format!("seikika-walk-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        for f in [
            "AGENTS.md",
            "checkout/AGENTS.md",
            "node_modules/x/AGENTS.md",
            "new.md",
        ] {
            let p = dir.join(f);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(p, "x").unwrap();
        }
        std::fs::write(dir.join(".gitignore"), "checkout/\n").unwrap();
        let disk = walk(&dir);
        assert!(disk.contains(&"checkout/AGENTS.md".to_owned()));
        assert!(!disk.iter().any(|p| p.starts_with("node_modules/")));
        let git = |args: &[&str]| {
            std::process::Command::new("git")
                .arg("-C")
                .arg(&dir)
                .args(args)
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
        };
        if git(&["init", "-q"]) && git(&["add", "AGENTS.md", ".gitignore"]) {
            assert_eq!(walk(&dir), [".gitignore", "AGENTS.md", "new.md"]);
        }
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn globs_match_segments() {
        let g = glob("docs/*.md").unwrap();
        assert!(g.is_match("docs/plan.md"));
        assert!(!g.is_match("docs/design/sdlc.md"));
        let g = glob("docs/**/*.md").unwrap();
        assert!(g.is_match("docs/plan.md") && g.is_match("docs/design/sdlc.md"));
        let g = glob(".github/workflows/*.y*ml").unwrap();
        assert!(g.is_match(".github/workflows/ci.yml") && g.is_match(".github/workflows/x.yaml"));
    }
}
