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

/// Relative paths (sorted) of every file under `root`, skipping [`SKIP_DIRS`].
pub fn walk(root: &Path) -> Vec<String> {
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
    use super::glob;

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
