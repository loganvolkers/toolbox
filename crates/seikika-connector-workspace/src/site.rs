//! Public documentation pages (`site-page` records): one per Starlight content file
//! (docs/design/site-standards.md). Facts only. The manifest decides the levels.

use regex::Regex;
use serde_json::{Value, json};
use std::path::Path;

/// One content page: frontmatter, whether the sidebar reaches it, and links that do not resolve.
pub fn analyse(root: &Path, rel: &str) -> Value {
    let text = std::fs::read_to_string(root.join(rel)).unwrap_or_default();
    let fm = parsed(frontmatter(&text));
    let title = fm_string(&fm, "title");
    let description = fm_string(&fm, "description");
    let draft = fm["draft"] == true;
    let sidebar_hidden = fm["sidebar"]["hidden"] == true || fm["hidden"] == true;
    let slug = slug(rel);
    json!({
        "path": rel,
        "slug": slug,
        "title": title,
        "duplicateTitle": !title.is_empty() && title_elsewhere(root, rel, &title),
        "descriptionChars": description.chars().count(),
        "draft": draft,
        "sidebarHidden": sidebar_hidden,
        "inSidebar": in_sidebar(root, rel, &slug, draft, sidebar_hidden),
        "missingFiles": missing_links(root, rel, &text),
        "secretHits": secrets(rel, &text),
    })
}

/// The directory builds its entries from public connector landing pages.
pub(crate) fn connector_guide(root: &Path, package: &str) -> Option<String> {
    let name = package.strip_prefix("seikika-connector-")?;
    if name.is_empty()
        || !name
            .bytes()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
    {
        return None;
    }
    let slug = format!("connectors/{name}");
    if !is_published_page(
        root,
        "site/src/content/docs/connectors/index.mdx",
        "connectors",
    ) {
        return None;
    }
    [
        format!("{slug}.md"),
        format!("{slug}.mdx"),
        format!("{slug}/index.md"),
        format!("{slug}/index.mdx"),
    ]
    .iter()
    .any(|page| is_published_page(root, &format!("site/src/content/docs/{page}"), &slug))
    .then(|| format!("/{slug}/"))
}

fn is_published_page(root: &Path, rel: &str, slug: &str) -> bool {
    let Ok(text) = std::fs::read_to_string(root.join(rel)) else {
        return false;
    };
    let fm = parsed(frontmatter(&text));
    if fm_string(&fm, "title").is_empty() || fm_string(&fm, "description").is_empty() {
        return false;
    }
    let slug_override = fm_string(&fm, "slug");
    let hidden = fm["sidebar"]["hidden"] == true || fm["hidden"] == true;
    (slug_override.is_empty() || slug_override == slug)
        && in_sidebar(root, rel, slug, fm["draft"] == true, hidden)
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

/// The frontmatter as YAML (Starlight reads it that way), or null when it doesn't parse.
fn parsed(fm: &str) -> Value {
    serde_norway::from_str(fm).unwrap_or(Value::Null)
}

/// A top-level string key, trimmed: a nested `hero: {title: …}` or a blank `title: " "` is no
/// title.
fn fm_string(fm: &Value, key: &str) -> String {
    fm[key]
        .as_str()
        .map(|t| t.trim().to_owned())
        .unwrap_or_default()
}

/// Whether another content page has the same title (a reader can't tell them apart in search or
/// the sidebar).
fn title_elsewhere(root: &Path, rel: &str, title: &str) -> bool {
    let dir = root.join("site/src/content/docs");
    walkdir::WalkDir::new(&dir)
        .into_iter()
        .flatten()
        .filter(|e| e.file_type().is_file())
        .filter(|e| {
            matches!(
                e.path().extension().and_then(|x| x.to_str()),
                Some("md" | "mdx")
            )
        })
        .filter(|e| !e.path().ends_with(rel) && root.join(rel) != e.path())
        .any(|e| {
            let text = std::fs::read_to_string(e.path()).unwrap_or_default();
            fm_string(&parsed(frontmatter(&text)), "title").eq_ignore_ascii_case(title)
        })
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

/// `base: '/seikika'` → `seikika`. Empty when the site is served at the domain root.
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
    static LINKS: std::sync::LazyLock<Regex> =
        std::sync::LazyLock::new(|| compile(r"\]\(([^)\s#]+)(?:#[^)]*)?\)"));
    let mut missing = Vec::new();
    for capture in LINKS.captures_iter(text) {
        let (_, [target]) = capture.extract();
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
        || generated_page(root, dir, &slug)
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

/// Routes supplied by site integrations and the build rather than by files in src/pages.
fn generated_page(root: &Path, site: &str, slug: &str) -> bool {
    let Ok(text) = std::fs::read_to_string(root.join(site).join("src/data/site-routes.json"))
    else {
        return false;
    };
    let Ok(routes) = serde_json::from_str::<std::collections::BTreeMap<String, String>>(&text)
    else {
        return false;
    };
    routes
        .values()
        .any(|path| path.starts_with('/') && path.trim_matches('/') == slug)
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
    crate::hygiene::secrets(rel, text)
}

fn compile(pat: &str) -> Regex {
    Regex::new(pat).unwrap_or_else(|_| panic!("static regex {pat}"))
}

#[cfg(test)]
mod tests {
    #[test]
    fn only_a_top_level_non_blank_title_counts() {
        let title = |fm: &str| super::fm_string(&super::parsed(fm), "title");
        assert_eq!(title("hero:\n  title: Nested\n"), "");
        assert_eq!(title("title: \"  \"\n"), "");
        assert_eq!(title("title: >-\n  Folded\n  title\n"), "Folded title");
        assert_eq!(title("title: DX Benchmarks\n"), "DX Benchmarks");
    }

    use super::*;
    use serde_json::json;

    struct Tmp(std::path::PathBuf);
    impl Drop for Tmp {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn scratch(name: &str, files: &[(&str, &str)]) -> Tmp {
        let root = std::env::temp_dir().join(format!("seikika-site-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        for (rel, body) in files {
            let path = root.join(rel);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, body).unwrap();
        }
        Tmp(root)
    }

    #[test]
    fn a_public_connector_guide_requires_no_manual_directory_link() {
        let config = "site/astro.config.mjs";
        let directory = "site/src/content/docs/connectors/index.mdx";
        let guide = "site/src/content/docs/connectors/github.md";
        let header = "---\ntitle: Connector\ndescription: What it reads.\n---\n";
        let tmp = scratch(
            "connector-guide",
            &[
                (
                    config,
                    "sidebar: [{ autogenerate: { directory: 'connectors' } }]",
                ),
                (directory, header),
                (guide, header),
            ],
        );
        let route = || connector_guide(&tmp.0, "seikika-connector-github");
        assert_eq!(route().as_deref(), Some("/connectors/github/"));
        for change in [
            "draft: true",
            "sidebar: {hidden: true}",
            "slug: another-url",
        ] {
            std::fs::write(
                tmp.0.join(guide),
                header.replace("\n---", &format!("\n{change}\n---")),
            )
            .unwrap();
            assert_eq!(route(), None, "{change}");
        }
        std::fs::remove_file(tmp.0.join(guide)).unwrap();
        assert_eq!(route(), None, "the connector needs its own landing page");
        std::fs::write(tmp.0.join(guide), header).unwrap();
        std::fs::write(tmp.0.join(config), "sidebar: ['connectors']").unwrap();
        assert_eq!(route(), None, "sidebar only lists the directory");
    }

    #[test]
    fn a_generated_page_resolves_under_the_project_base() {
        let root = scratch(
            "generated-page",
            &[
                (
                    "site/astro.config.mjs",
                    "export default { base: '/seikika' };",
                ),
                (
                    "site/src/data/site-routes.json",
                    r#"{"report":"/report","storybook":"/storybook"}"#,
                ),
            ],
        );
        let from = "site/src/content/docs/index.md";
        assert!(site_link_exists(&root.0, from, "/seikika/report/"));
        assert!(site_link_exists(&root.0, from, "/seikika/storybook/"));
        assert!(!site_link_exists(&root.0, from, "/report/"));
        assert!(!site_link_exists(&root.0, from, "/seikika/missing/"));
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
                ("site/astro.config.mjs", "base: '/seikika',"),
                (
                    "site/src/content/docs/index.md",
                    "---\ntitle: Home\ndescription: Home.\n---\n[Run](/start/run/) [ok](/seikika/start/run/)\n",
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
                ("site/astro.config.mjs", "base: '/seikika',"),
                ("site/public/report/index.html", "<h1>report</h1>"),
                (
                    "site/src/content/docs/demo/report.md",
                    "---\ntitle: Report\ndescription: The report.\n---\n[Open it](/seikika/report/)\n",
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
                ("site/astro.config.mjs", "base: '/seikika',"),
                (
                    "site/src/pages/report-hosted.astro",
                    "<seikika-report></seikika-report>\n",
                ),
                (
                    "site/src/content/docs/demo/report.md",
                    "---\ntitle: Report\ndescription: The report.\n---\n[Hosted](/seikika/report-hosted/)\n",
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
