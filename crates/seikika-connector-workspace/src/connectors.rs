//! First-party connectors (`Cargo.toml` of a `seikika-connector-*` crate), read statically
//! (docs/design/connector-standards.md).
//!
//! One record per crate. Facts only: the description and crate docs, which transport it speaks,
//! whether a resync closes with `snapshot_complete` and a refresh reports `not_found`, the record
//! kinds named in the source, unit tests, and whether CI actually runs the binary. Thresholds
//! live in the manifest.

use regex::Regex;
use serde_json::{Value, json};
use std::collections::BTreeSet;
use std::path::Path;

pub(crate) fn analyse(root: &Path, rel: &str, site: &str) -> Value {
    let toml = read(root, rel);
    let dir = parent(rel);
    let name = package_field(&toml, "name");
    let description = package_field(&toml, "description");
    let sources = rust_files(root, &dir);
    let doc = crate_doc(&sources);
    let source = sources
        .iter()
        .map(|(_, text)| text.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    let transport = transport_of(&source);
    let (conformance, ci_files) = ci_mentions(root, &name);
    json!({
        "path": rel,
        "dir": dir,
        "name": name,
        "description": description,
        "descriptionChars": description.chars().count() as i64,
        "crateDoc": !doc.is_empty(),
        "configDocumented": config_documented(&doc),
        "publicGuide": crate::site::connector_guide(root, site, &name),
        "transport": transport,
        "pushOnly": push_only(&source, &doc),
        "documentedKinds": documented_kinds(&source, &doc),
        "genericKinds": generic_kinds(&doc),
        "snapshotComplete": source.contains("snapshot_complete"),
        "reportsNotFound": source.contains("not_found"),
        "tests": test_count(&source),
        "conformanceMentions": conformance,
        "ciMentions": ci_files,
        "secretHits": secrets(root, &dir, &sources),
    })
}

fn read(root: &Path, rel: &str) -> String {
    std::fs::read_to_string(root.join(rel)).unwrap_or_default()
}

fn parent(rel: &str) -> String {
    Path::new(rel)
        .parent()
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .unwrap_or_default()
}

fn package_field(toml: &str, key: &str) -> String {
    let mut in_package = false;
    let prefix = format!("{key} = \"");
    for line in toml.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_package = trimmed == "[package]";
            continue;
        }
        if !in_package {
            continue;
        }
        let Some(rest) = trimmed.strip_prefix(&prefix) else {
            continue;
        };
        let Some(end) = rest.find('"') else {
            continue;
        };
        return rest[..end].to_owned();
    }
    String::new()
}

fn rust_files(root: &Path, dir: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let prefix = format!("{dir}/");
    for rel in seikika_connect::files::walk(root) {
        if rel.starts_with(&prefix) && rel.ends_with(".rs") && !rel.contains("/target/") {
            out.push((rel.clone(), read(root, &rel)));
        }
    }
    out
}

/// The `//!` block at the top of `src/main.rs`, or `src/lib.rs` when there is no binary.
fn crate_doc(sources: &[(String, String)]) -> String {
    let text = sources
        .iter()
        .find(|(p, _)| p.ends_with("/src/main.rs"))
        .or_else(|| sources.iter().find(|(p, _)| p.ends_with("/src/lib.rs")))
        .map(|(_, t)| t.as_str())
        .unwrap_or("");
    let mut lines = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() && lines.is_empty() {
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("//!") {
            lines.push(rest.trim().to_owned());
        } else if trimmed.starts_with("#!") {
            continue;
        } else {
            break;
        }
    }
    lines.join("\n")
}

fn config_documented(doc: &str) -> bool {
    doc.contains("SEIKIKA_CONNECTOR_CONFIG")
        || doc.contains("--seikika")
        || doc.contains("--listen")
}

fn transport_of(source: &str) -> &'static str {
    match (source.contains("run_stdio"), source.contains("run_remote")) {
        (true, true) => "both",
        (true, false) => "stdio",
        (false, true) => "remote",
        (false, false) => "none",
    }
}

fn push_only(source: &str, doc: &str) -> bool {
    doc.contains("push-only") || (source.contains("run_remote") && !source.contains("run_stdio"))
}

fn documented_kinds(source: &str, doc: &str) -> Vec<String> {
    let mut kinds = BTreeSet::new();
    for re in kind_patterns() {
        for cap in re.captures_iter(source) {
            if let Some(m) = cap.get(1) {
                kinds.insert(m.as_str().to_owned());
            }
        }
    }
    let bullets = compile(r"(?m)^//!\s*-\s+`([a-z][a-z0-9-]*)`");
    for cap in bullets.captures_iter(doc) {
        if let Some(m) = cap.get(1) {
            kinds.insert(m.as_str().to_owned());
        }
    }
    // The doc passed in is the joined crate doc without `//!` prefixes. Scan that form too.
    let plain = compile(r"(?m)^-\s+`([a-z][a-z0-9-]*)`");
    for cap in plain.captures_iter(doc) {
        if let Some(m) = cap.get(1) {
            kinds.insert(m.as_str().to_owned());
        }
    }
    kinds.into_iter().collect()
}

fn kind_patterns() -> Vec<Regex> {
    [
        r#"(?:record|snapshot_complete)\(\s*"([a-z][a-z0-9-]*)""#,
        r#""kind"\s*:\s*"([a-z][a-z0-9-]*)""#,
    ]
    .into_iter()
    .map(compile)
    .collect()
}

fn generic_kinds(doc: &str) -> bool {
    doc.contains("kind `<") || doc.contains("declares their `kinds`")
}

fn test_count(source: &str) -> i64 {
    compile(r"#\[(?:[\w:]+::)*test\]").find_iter(source).count() as i64
}

fn ci_mentions(root: &Path, package: &str) -> (i64, i64) {
    if package.is_empty() {
        return (0, 0);
    }
    let mut conformance = 0i64;
    let mut files = 0i64;
    for rel in seikika_connect::files::walk(root) {
        if !is_ci_script(&rel) {
            continue;
        }
        let text = read(root, &rel);
        let mut mentions = text
            .lines()
            .filter(|line| !line.trim_start().starts_with('#') && line.contains(package))
            .peekable();
        if mentions.peek().is_none() {
            continue;
        }
        files += 1;
        conformance += mentions
            .filter(|line| line.contains("connector test"))
            .count() as i64;
    }
    (conformance, files)
}

fn is_ci_script(rel: &str) -> bool {
    (rel.starts_with("scripts/") && rel.ends_with(".sh"))
        || (rel.starts_with(".github/workflows/")
            && (rel.ends_with(".yml") || rel.ends_with(".yaml")))
}

/// Credentials in the connector's Rust sources (outside test modules) and in its other text files,
/// which `include_str!` can compile in. Binary files are skipped.
fn secrets(root: &Path, dir: &str, sources: &[(String, String)]) -> Vec<String> {
    let prefix = format!("{dir}/");
    let data = seikika_connect::files::walk(root)
        .into_iter()
        .filter(|rel| {
            rel.starts_with(&prefix) && !rel.ends_with(".rs") && !rel.contains("/target/")
        })
        .filter_map(|rel| {
            std::fs::read_to_string(root.join(&rel))
                .ok()
                .map(|text| crate::hygiene::secrets(&rel, &text))
        })
        .flatten();
    let mut hits: Vec<String> = sources
        .iter()
        .flat_map(|(rel, text)| crate::hygiene::secrets_in_lines(rel, production_lines(text), true))
        .chain(data)
        .collect();
    hits.sort();
    hits
}

/// Lines outside `#[cfg(test)]` modules. A fixture key in a unit test is not a shipped secret.
fn production_lines(text: &str) -> Vec<(usize, &str)> {
    let mut out = Vec::new();
    let mut skip = 0i32;
    let mut arming = false;
    for (n, line) in text.lines().enumerate() {
        if take_test_module(line, &mut skip, &mut arming) {
            continue;
        }
        out.push((n, line));
    }
    out
}

fn take_test_module(line: &str, skip: &mut i32, arming: &mut bool) -> bool {
    let trimmed = line.trim();
    if *skip == 0 && !*arming && trimmed.starts_with("#[cfg(test)]") {
        *arming = true;
        return true;
    }
    if *arming {
        if trimmed.is_empty() || trimmed.starts_with("#[") || trimmed.starts_with("//") {
            return true; // more attributes or comments before the item
        }
        // `#[cfg(test)]` covers exactly the next item: a module, or a single `use`, `fn` or
        // `const`. Skip that item's braces, then scan again (a `use` line alone hides nothing).
        let opened = brace_delta(line);
        let is_mod = trimmed.starts_with("mod ") || trimmed.starts_with("pub mod ");
        *arming = is_mod && opened == 0 && !trimmed.ends_with(';'); // `mod tests` with `{` below
        *skip = opened.max(0);
        return true;
    }
    if *skip > 0 {
        *skip += brace_delta(line);
        if *skip < 0 {
            *skip = 0;
        }
        return true;
    }
    false
}

fn brace_delta(line: &str) -> i32 {
    let code = line.split("//").next().unwrap_or(line);
    let open = code.chars().filter(|c| *c == '{').count() as i32;
    let close = code.chars().filter(|c| *c == '}').count() as i32;
    open - close
}

fn compile(pat: &str) -> Regex {
    Regex::new(pat).unwrap_or_else(|_| panic!("static regex {pat}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_test_only_use_line_hides_only_itself() {
        let key = format!("AKIA{}", "A".repeat(16));
        let text = format!(
            "#[cfg(test)]\nuse std::collections::HashMap;\nconst K: &str = \"{key}\";\n#[cfg(test)]\nmod tests {{\n    const T: &str = \"{key}\";\n}}\n"
        );
        let lines: Vec<usize> = production_lines(&text).iter().map(|(n, _)| *n).collect();
        assert_eq!(
            lines,
            vec![2],
            "only the constant outside the test items is scanned"
        );
    }
    use serde_json::json;

    struct Tmp(std::path::PathBuf);
    impl Drop for Tmp {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn scratch(name: &str, files: &[(&str, &str)]) -> Tmp {
        let root = std::env::temp_dir().join(format!(
            "seikika-connector-{name}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        for (rel, text) in files {
            let path = root.join(rel);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, text).unwrap();
        }
        Tmp(root)
    }

    fn cargo(name: &str) -> String {
        format!("[package]\nname = \"{name}\"\ndescription = \"Reads widgets.\"\n")
    }

    #[test]
    fn reads_a_stdio_connector_that_ci_runs() {
        let doc = "//! The widget connector (in-image, stdio).\n//! `SEIKIKA_CONNECTOR_CONFIG`: `{\"org\"}`.\n//! Record: `widget:<id>`.\n";
        let src = format!(
            "{doc}\nuse seikika_connect::sdk::run_stdio;\nfn handle() {{\n  out.record(\"widget\", id, data);\n  out.snapshot_complete(\"widget\");\n  out.not_found(\"widget\", id);\n}}\n#[test]\nfn t() {{}}\n"
        );
        let tmp = scratch(
            "good",
            &[
                (
                    "crates/demo/widget/Cargo.toml",
                    &cargo("seikika-connector-widget"),
                ),
                ("crates/demo/widget/src/main.rs", &src),
                (
                    "scripts/check.sh",
                    "seikika connector test -- target/debug/seikika-connector-widget\n  # seikika connector test -- seikika-connector-widget\n",
                ),
                (
                    ".github/workflows/comment-only.yml",
                    "# seikika connector test -- seikika-connector-widget\n",
                ),
            ],
        );
        let rec = analyse(&tmp.0, "crates/demo/widget/Cargo.toml", "site");
        assert_eq!(rec["name"], "seikika-connector-widget");
        assert_eq!(rec["transport"], "stdio");
        assert_eq!(rec["pushOnly"], false);
        assert_eq!(rec["documentedKinds"], json!(["widget"]));
        assert_eq!(rec["tests"], 1);
        assert_eq!(rec["conformanceMentions"], 1);
        assert_eq!(rec["ciMentions"], 1);
        assert_eq!(rec["crateDoc"], true);
        assert_eq!(rec["configDocumented"], true);
        assert_eq!(rec["secretHits"], json!([]));
    }

    #[test]
    fn a_push_connector_names_kinds_and_is_not_a_stdio_resync() {
        let doc = "//! The idp connector (remote).\n//! Usage: `seikika-connector-idp --seikika URL --listen ADDR`\n//! - `user` `idp:user/<id>`\n//! SCIM is push-only.\n";
        let src = format!(
            "{doc}\nfn user() {{ json!({{\"kind\": \"user\"}}); }}\nuse seikika_connect::sdk::run_remote;\n"
        );
        let tmp = scratch(
            "push",
            &[
                ("crates/idp/Cargo.toml", &cargo("seikika-connector-idp")),
                ("crates/idp/src/main.rs", &src),
                (
                    "scripts/m4.sh",
                    "\"$T/seikika-connector-idp\" --seikika \"$URL\" --listen \"$SCIM\"\n",
                ),
            ],
        );
        let rec = analyse(&tmp.0, "crates/idp/Cargo.toml", "site");
        assert_eq!(rec["transport"], "remote");
        assert_eq!(rec["pushOnly"], true);
        assert_eq!(rec["documentedKinds"], json!(["user"]));
        assert_eq!(rec["snapshotComplete"], false);
        assert_eq!(rec["conformanceMentions"], 0);
        assert_eq!(rec["ciMentions"], 1);
        assert_eq!(rec["configDocumented"], true);
        assert_eq!(rec["tests"], 0);
    }

    #[test]
    fn an_undocumented_binary_is_not_a_connector() {
        let src = "fn main() {\n  let key = \"AKIAIOSFODNN7EXAMPLE\";\n  let _ = key;\n}\n";
        let tmp = scratch(
            "bare",
            &[
                (
                    "crates/bare/Cargo.toml",
                    "[package]\nname = \"seikika-connector-bare\"\n",
                ),
                ("crates/bare/src/main.rs", src),
            ],
        );
        let rec = analyse(&tmp.0, "crates/bare/Cargo.toml", "site");
        assert_eq!(rec["descriptionChars"], 0);
        assert_eq!(rec["crateDoc"], false);
        assert_eq!(rec["transport"], "none");
        assert_eq!(rec["configDocumented"], false);
        assert_eq!(rec["documentedKinds"], json!([]));
        assert_eq!(
            rec["secretHits"],
            json!(["crates/bare/src/main.rs:2 aws-access-key"])
        );
    }

    #[test]
    fn a_fixture_key_inside_cfg_test_is_not_a_shipped_secret() {
        let src = "fn main() {}\n#[cfg(test)]\nmod tests {\n    fn t() { let _ = \"AKIAIOSFODNN7EXAMPLE\"; }\n}\n";
        let tmp = scratch(
            "fixture-key",
            &[
                (
                    "crates/demo/Cargo.toml",
                    "[package]\nname = \"seikika-connector-demo\"\ndescription = \"Demo.\"\n",
                ),
                ("crates/demo/src/main.rs", src),
            ],
        );
        let rec = analyse(&tmp.0, "crates/demo/Cargo.toml", "site");
        assert_eq!(rec["secretHits"], json!([]));
    }

    #[test]
    fn the_connectors_we_ship_match_the_scorecard_facts() {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let github = analyse(&root, "crates/seikika-connector-github/Cargo.toml", "site");
        assert_eq!(github["transport"], "stdio");
        assert!(github["tests"].as_i64().unwrap() >= 1);
        assert!(github["conformanceMentions"].as_i64().unwrap() >= 1);
        assert!(
            github["documentedKinds"]
                .as_array()
                .unwrap()
                .iter()
                .any(|k| k == "repository")
        );

        let scim = analyse(&root, "crates/seikika-connector-scim/Cargo.toml", "site");
        assert_eq!(scim["pushOnly"], true);
        assert_eq!(scim["transport"], "remote");
        assert!(scim["tests"].as_i64().unwrap() >= 3);
        assert!(scim["ciMentions"].as_i64().unwrap() >= 1);
        assert_eq!(scim["conformanceMentions"], 0);

        let files = analyse(
            &root,
            "crates/seikika-connector-git-files/Cargo.toml",
            "site",
        );
        assert!(files["tests"].as_i64().unwrap() >= 1);
        assert!(files["conformanceMentions"].as_i64().unwrap() >= 1);
        assert!(files["snapshotComplete"].as_bool().unwrap());

        let runner = analyse(&root, "crates/seikika-connector-runner/Cargo.toml", "site");
        assert_eq!(runner["genericKinds"], true);
        assert!(runner["tests"].as_i64().unwrap() >= 1);

        let workspace = analyse(
            &root,
            "crates/seikika-connector-workspace/Cargo.toml",
            "site",
        );
        assert!(workspace["descriptionChars"].as_i64().unwrap() > 0);
        assert_eq!(workspace["crateDoc"], true);
        assert_eq!(workspace["configDocumented"], true);
        assert!(workspace["tests"].as_i64().unwrap() >= 1);
        for (label, rec) in [
            ("github", github),
            ("scim", scim),
            ("git-files", files),
            ("runner", runner),
            ("workspace", workspace),
        ] {
            assert_eq!(rec["secretHits"], json!([]), "{label}");
        }
    }
}
