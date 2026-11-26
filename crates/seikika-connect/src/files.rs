//! Match repository-relative paths for connectors that read a local checkout.

use regex::Regex;
use std::path::Path;

/// Directory names omitted from checkout scans at every depth.
pub const SKIP_DIRS: &[&str] = &["target", "node_modules", ".git", "dist", ".venv"];

/// Compiles a glob that matches an entire `/`-separated relative path.
///
/// `**` crosses directory boundaries, and `**/` matches zero or more directories. `*` matches
/// zero or more characters in one path segment, and `?` matches one. Other characters are literal.
///
/// # Errors
///
/// Returns a diagnostic if the translated expression exceeds the regex compiler's limits.
///
/// # Examples
///
/// ```
/// use seikika_connect::files::glob;
///
/// let manifests = glob("**/catalog-info.yaml")?;
/// assert!(manifests.is_match("catalog-info.yaml"));
/// assert!(manifests.is_match("services/checkout/catalog-info.yaml"));
/// # Ok::<(), String>(())
/// ```
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

/// Lists sorted, `/`-separated paths relative to `root`, omitting [`SKIP_DIRS`].
///
/// In a Git checkout, includes tracked files and unignored untracked files. If Git is unavailable
/// or lists no existing files after filtering, falls back to a disk walk with no Git ignore rules.
/// The fallback skips entries it cannot read.
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

/// Returns the paths from [`walk`] that match at least one of `globs`.
///
/// Compile each pattern with [`glob`]. An empty pattern list returns an empty result.
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
