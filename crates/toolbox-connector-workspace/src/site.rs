//! Public documentation pages (`site-page` records): one per Starlight content file
//! (docs/design/site-standards.md). Facts only. The manifest decides the levels.

use regex::Regex;
use serde_json::{Value, json};
use std::path::Path;

/// One content page: frontmatter, whether the sidebar reaches it, and links that do not resolve.
pub fn analyse(root: &Path, rel: &str) -> Value {
    let text = std::fs::read_to_string(root.join(rel)).unwrap_or_default();
    let fm = frontmatter(&text);
    let title = fm_string(fm, "title");
    let description = fm_string(fm, "description");
    let draft = fm_bool(fm, "draft");
    let sidebar_hidden = fm_bool(fm, "sidebar.hidden") || fm_bool(fm, "hidden");
    let slug = slug(rel);
    json!({
        "path": rel,
        "slug": slug,
        "title": title,
        "descriptionChars": description.chars().count(),
        "draft": draft,
        "sidebarHidden": sidebar_hidden,
        "inSidebar": in_sidebar(root, rel, &slug, draft, sidebar_hidden),
        "missingFiles": missing_links(root, rel, &text),
        "secretHits": secrets(rel, &text),
    })
}

fn frontmatter(text: &str) -> &str {
    let rest = text
        .strip_prefix("---\n")
        .or_else(|| text.strip_prefix("---\r\n"));
    let Some(rest) = rest else {
        return "";
    };
    rest.find("\n---").map(|i| &rest[..i]).unwrap_or("")
}

fn fm_string(fm: &str, key: &str) -> String {
    let prefix = format!("{key}:");
    for line in fm.lines() {
        let Some(rest) = line.trim().strip_prefix(&prefix) else {
            continue;
        };
        return rest
            .trim()
            .trim_matches(|c| c == '"' || c == '\'')
            .to_string();
    }
    String::new()
}

fn fm_bool(fm: &str, key: &str) -> bool {
    fm_string(fm, key) == "true"
}

/// Slug under `src/content/docs`. The root `index` page is empty, which Starlight serves at `/`.
fn slug(rel: &str) -> String {
    let rest = rel
        .split_once("/src/content/docs/")
        .map(|(_, r)| r)
        .unwrap_or(rel);
    let no_ext = rest.trim_end_matches(".mdx").trim_end_matches(".md");
    if no_ext == "index" {
        String::new()
    } else if let Some(dir) = no_ext.strip_suffix("/index") {
        dir.to_string()
    } else {
        no_ext.to_string()
    }
}

fn read_config(root: &Path, dir: &str) -> String {
    ["astro.config.mjs", "astro.config.ts", "astro.config.js"]
        .iter()
        .map(|name| root.join(dir).join(name))
        .find(|p| p.is_file())
        .and_then(|p| std::fs::read_to_string(p).ok())
        .unwrap_or_default()
}

/// `base: '/toolbox'` → `toolbox`. Empty when the site is served at the domain root.
fn site_base(config: &str) -> String {
    compile(r#"base\s*:\s*['"]([^'"]+)['"]"#)
        .captures(config)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().trim_matches('/').to_string())
        .unwrap_or_default()
}

fn site_dir(rel: &str) -> Option<&str> {
    rel.split_once("/src/content/docs/").map(|(dir, _)| dir)
}

fn in_sidebar(root: &Path, rel: &str, slug: &str, draft: bool, hidden: bool) -> bool {
    if draft || hidden {
        return false;
    }
    if slug.is_empty() {
        return true;
    }
    let Some(dir) = site_dir(rel) else {
        return false;
    };
    let config = read_config(root, dir);
    let dirs = compile(r#"directory\s*:\s*['"]([^'"]+)['"]"#);
    if dirs.captures_iter(&config).any(|c| {
        let dir = c.get(1).map(|m| m.as_str()).unwrap_or("");
        slug == dir || slug.starts_with(&format!("{dir}/"))
    }) {
        return true;
    }
    config.contains(&format!("'{slug}'")) || config.contains(&format!("\"{slug}\""))
}

fn missing_links(root: &Path, rel: &str, text: &str) -> Vec<String> {
    let re = compile(r"\]\(([^)\s#]+)(?:#[^)]*)?\)");
    let mut missing = Vec::new();
    for c in re.captures_iter(text) {
        let Some(target) = c.get(1).map(|m| m.as_str()) else {
            continue;
        };
        if target.starts_with("http://")
            || target.starts_with("https://")
            || target.starts_with("mailto:")
        {
            continue;
        }
        let ok = if target.starts_with('/') {
            site_link_exists(root, rel, target)
        } else {
            file_exists(root, rel, target)
        };
        if !ok {
            missing.push(target.to_string());
        }
    }
    missing.sort();
    missing.dedup();
    missing
}

/// A Starlight root-relative link names a content page. When `astro.config` sets `base`, the
/// link has to include it: GitHub Pages serves the site under that prefix, and Starlight does
/// not rewrite a markdown link that already starts with `/`.
fn site_link_exists(root: &Path, rel: &str, target: &str) -> bool {
    let Some(dir) = site_dir(rel) else {
        return false;
    };
    let config = read_config(root, dir);
    let prefix = site_base(&config);
    let mut slug = target.trim_matches('/').to_string();
    if !prefix.is_empty() {
        if slug == prefix {
            slug.clear();
        } else if let Some(rest) = slug.strip_prefix(&format!("{prefix}/")) {
            slug = rest.to_string();
        } else {
            return false;
        }
    }
    let base = root.join(dir).join("src/content/docs");
    if slug.is_empty() {
        return base.join("index.md").is_file() || base.join("index.mdx").is_file();
    }
    base.join(format!("{slug}.md")).is_file()
        || base.join(format!("{slug}.mdx")).is_file()
        || base.join(&slug).join("index.md").is_file()
        || base.join(&slug).join("index.mdx").is_file()
        || public_file(root, dir, &slug)
        || astro_page(root, dir, &slug)
}

/// A file shipped next to the site (`public/report/index.html` → `/report/`).
fn public_file(root: &Path, site: &str, slug: &str) -> bool {
    if slug.is_empty() {
        return false;
    }
    let public = root.join(site).join("public");
    public.join(format!("{slug}.html")).is_file()
        || public.join(slug).join("index.html").is_file()
        || public.join(slug).is_file()
}

/// An Astro page under `src/pages` (`report-hosted` → `src/pages/report-hosted.astro`).
fn astro_page(root: &Path, site: &str, slug: &str) -> bool {
    if slug.is_empty() {
        return false;
    }
    let pages = root.join(site).join("src/pages");
    pages.join(format!("{slug}.astro")).is_file() || pages.join(slug).join("index.astro").is_file()
}

fn file_exists(root: &Path, rel: &str, target: &str) -> bool {
    let from = root.join(rel);
    let Some(parent) = from.parent() else {
        return false;
    };
    let path = parent.join(target);
    path.is_file() || path.is_dir()
}

fn secrets(rel: &str, text: &str) -> Vec<String> {
    let patterns = [
        ("aws-access-key", r"AKIA[0-9A-Z]{16}"),
        ("github-token", r"ghp_[A-Za-z0-9]{36}"),
    ];
    let mut hits = Vec::new();
    for (n, line) in text.lines().enumerate() {
        for (kind, pat) in patterns {
            if compile(pat).is_match(line) {
                hits.push(format!("{rel}:{} {kind}", n + 1));
            }
        }
    }
    hits
}

fn compile(pat: &str) -> Regex {
    Regex::new(pat).unwrap_or_else(|_| panic!("static regex {pat}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    struct Tmp(std::path::PathBuf);
    impl Drop for Tmp {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn scratch(name: &str, files: &[(&str, &str)]) -> Tmp {
        let root = std::env::temp_dir().join(format!("toolbox-site-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        for (rel, body) in files {
            let path = root.join(rel);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, body).unwrap();
        }
        Tmp(root)
    }

    #[test]
    fn a_sidebar_page_with_resolving_links_is_complete() {
        let tmp = scratch(
            "good",
            &[
                (
                    "site/astro.config.mjs",
                    "sidebar: [{ autogenerate: { directory: 'concepts' } }]",
                ),
                (
                    "site/src/content/docs/concepts/scorecards.md",
                    "---\ntitle: Scorecards\ndescription: Levels and evidence.\n---\nSee [the catalog](/concepts/catalog/).\n",
                ),
                (
                    "site/src/content/docs/concepts/catalog.md",
                    "---\ntitle: Catalog\ndescription: Entities.\n---\n",
                ),
            ],
        );
        let page = analyse(&tmp.0, "site/src/content/docs/concepts/scorecards.md");
        assert_eq!(page["slug"], "concepts/scorecards");
        assert_eq!(page["title"], "Scorecards");
        assert_eq!(page["descriptionChars"], 20);
        assert_eq!(page["inSidebar"], true);
        assert_eq!(page["missingFiles"], json!([]));
        assert_eq!(page["secretHits"], json!([]));
    }

    #[test]
    fn a_root_relative_link_must_include_the_pages_base() {
        let tmp = scratch(
            "base",
            &[
                ("site/astro.config.mjs", "base: '/toolbox',"),
                (
                    "site/src/content/docs/index.md",
                    "---\ntitle: Home\ndescription: Home.\n---\n[Run](/start/run/) [ok](/toolbox/start/run/)\n",
                ),
                (
                    "site/src/content/docs/start/run.md",
                    "---\ntitle: Run\ndescription: Run.\n---\n",
                ),
            ],
        );
        let page = analyse(&tmp.0, "site/src/content/docs/index.md");
        assert_eq!(page["missingFiles"], json!(["/start/run/"]));
    }

    #[test]
    fn a_link_to_a_published_file_resolves() {
        let tmp = scratch(
            "public",
            &[
                ("site/astro.config.mjs", "base: '/toolbox',"),
                ("site/public/report/index.html", "<h1>report</h1>"),
                (
                    "site/src/content/docs/demo/report.md",
                    "---\ntitle: Report\ndescription: The report.\n---\n[Open it](/toolbox/report/)\n",
                ),
            ],
        );
        let page = analyse(&tmp.0, "site/src/content/docs/demo/report.md");
        assert_eq!(page["missingFiles"], json!([]));
        assert_eq!(page["inSidebar"], false);
    }

    #[test]
    fn a_link_to_an_astro_page_resolves() {
        let tmp = scratch(
            "astro",
            &[
                ("site/astro.config.mjs", "base: '/toolbox',"),
                (
                    "site/src/pages/report-hosted.astro",
                    "<toolbox-report></toolbox-report>\n",
                ),
                (
                    "site/src/content/docs/demo/report.md",
                    "---\ntitle: Report\ndescription: The report.\n---\n[Hosted](/toolbox/report-hosted/)\n",
                ),
            ],
        );
        let page = analyse(&tmp.0, "site/src/content/docs/demo/report.md");
        assert_eq!(page["missingFiles"], json!([]));
    }

    #[test]
    fn a_draft_is_not_in_the_sidebar() {
        let tmp = scratch(
            "draft",
            &[(
                "site/src/content/docs/draft.md",
                "---\ntitle: Later\ndescription: Not published.\ndraft: true\n---\n",
            )],
        );
        let page = analyse(&tmp.0, "site/src/content/docs/draft.md");
        assert_eq!(page["draft"], true);
        assert_eq!(page["inSidebar"], false);
    }

    #[test]
    fn a_broken_link_and_a_secret_are_facts() {
        let token = format!("ghp_{}", "a".repeat(36));
        let tmp = scratch(
            "bad",
            &[(
                "site/src/content/docs/unsafe.md",
                &format!("---\n---\nSee [gone](/missing/).\n{token}\n"),
            )],
        );
        let page = analyse(&tmp.0, "site/src/content/docs/unsafe.md");
        assert_eq!(page["title"], "");
        assert_eq!(page["missingFiles"], json!(["/missing/"]));
        assert_eq!(
            page["secretHits"][0],
            "site/src/content/docs/unsafe.md:4 github-token"
        );
    }
}
