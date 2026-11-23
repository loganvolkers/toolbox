//! Agent skills (`SKILL.md`), read statically (docs/design/skill-standards.md).
//!
//! One record per skill directory. Facts only: the Agent Skills frontmatter checks (the same
//! ones `skills-ref validate` applies), and the counts the scorecard thresholds sit on. A
//! leading BOM is ignored; other hidden characters are counted.

use regex::Regex;
use serde_json::{Map, Value, json};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

const NAME_MAX: usize = 64;
const DESCRIPTION_MAX: usize = 1024;
const COMPATIBILITY_MAX: usize = 500;
const ALLOWED: &[&str] = &[
    "name",
    "description",
    "license",
    "compatibility",
    "metadata",
    "allowed-tools",
];

/// The `skill` record for the `SKILL.md` at `rel` (a `/`-separated path under `root`).
pub(crate) fn analyse(root: &Path, rel: &str) -> Value {
    let dir = parent(rel).to_string();
    let dir_name = dir.rsplit('/').next().unwrap_or(&dir).to_string();
    let raw = std::fs::read_to_string(root.join(rel)).unwrap_or_default();
    let text = strip_bom(&raw).replace("\r\n", "\n").replace('\r', "\n");
    let (frontmatter_errors, name, description, body) = split_and_check(&text, &dir_name);
    let (files, unreadable) = files_under(root, &dir);
    let windows_re = compile(r"[A-Za-z]:\\[^\s]+|\\\\[^\s]+");
    let (hidden, secrets, windows) = scan_files(&files, &windows_re);
    // A file that isn't UTF-8 can't be inspected, so it counts as hidden content.
    let hidden = hidden + unreadable;
    let description_chars = description.chars().count() as i64;
    json!({
        "path": rel,
        "dir": dir,
        "name": name,
        "plugin": plugin_of(&dir),
        "description": description,
        "descriptionChars": description_chars,
        "triggerCues": listed_words(&description, &["when", "whenever", "before", "after"]),
        "pronouns": listed_words(&description, &["you", "your", "yours", "i", "we", "my", "our", "ours"]),
        "frontmatterErrors": frontmatter_errors,
        "bodyLines": body.lines().count() as i64,
        "estimatedTokens": body.chars().count() as i64 / 4,
        "numberedSteps": numbered_steps(&body),
        "codeFences": code_fences(&body),
        "missingFiles": missing_files(root, rel, &body),
        "windowsPaths": windows,
        "boundaryCues": boundary_cues(&body),
        "evalCases": eval_cases(root, &dir),
        "hiddenChars": hidden,
        "secretHits": secrets,
        "linkDepth": link_depth(root, rel),
        "supportFiles": support_files(&files, rel),
    })
}

fn strip_bom(text: &str) -> &str {
    text.strip_prefix('\u{feff}').unwrap_or(text)
}

fn parent(path: &str) -> &str {
    match path.rsplit_once('/') {
        Some((dir, _)) => dir,
        None => "",
    }
}

fn plugin_of(dir: &str) -> String {
    let mut parts = dir.split('/');
    match (parts.next(), parts.next()) {
        (Some("plugins"), Some(name)) => name.to_string(),
        _ => String::new(),
    }
}

fn compile(pat: &str) -> Regex {
    Regex::new(pat).unwrap_or_else(|_| panic!("static regex {pat}"))
}

fn split_and_check(text: &str, dir_name: &str) -> (Vec<String>, String, String, String) {
    match split_frontmatter(text) {
        Ok((yaml, body)) => {
            let (errors, name, description) = check_frontmatter(&yaml, dir_name);
            (errors, name, description, body)
        }
        Err(e) => (
            vec![e.to_string()],
            String::new(),
            String::new(),
            text.to_string(),
        ),
    }
}

fn split_frontmatter(text: &str) -> Result<(String, String), &'static str> {
    let Some(after_open) = text.strip_prefix("---") else {
        return Err("SKILL.md must start with YAML frontmatter (---)");
    };
    let after_open = after_open.strip_prefix('\n').unwrap_or(after_open);
    let Some(close) = after_open.find("\n---") else {
        return Err("SKILL.md frontmatter not properly closed with ---");
    };
    let yaml = after_open[..close].to_string();
    let mut body = &after_open[close + 1..];
    body = body.strip_prefix("---").unwrap_or(body);
    body = body.strip_prefix('\n').unwrap_or(body);
    Ok((yaml, body.to_string()))
}

fn parse_meta(yaml: &str) -> Result<Map<String, Value>, String> {
    let v: Value =
        serde_norway::from_str(yaml).map_err(|e| format!("Invalid YAML in frontmatter: {e}"))?;
    v.as_object()
        .cloned()
        .ok_or_else(|| "SKILL.md frontmatter must be a YAML mapping".into())
}

fn check_frontmatter(yaml: &str, dir_name: &str) -> (Vec<String>, String, String) {
    let meta = match parse_meta(yaml) {
        Ok(m) => m,
        Err(e) => return (vec![e], String::new(), String::new()),
    };
    let mut errors = unexpected(&meta);
    let (name_errs, name) = name_errors(&meta, dir_name);
    errors.extend(name_errs);
    let description = description_of(&meta, &mut errors);
    optional_strings(&meta, &mut errors);
    metadata_errors(&meta, &mut errors);
    (errors, name, description)
}

fn unexpected(meta: &Map<String, Value>) -> Vec<String> {
    let extra: Vec<&String> = meta
        .keys()
        .filter(|k| !ALLOWED.contains(&k.as_str()))
        .collect();
    if extra.is_empty() {
        return Vec::new();
    }
    let names: Vec<&str> = extra.iter().map(|s| s.as_str()).collect();
    vec![format!(
        "Unexpected fields in frontmatter: {}. Only name, description, license, compatibility, metadata, allowed-tools are allowed.",
        names.join(", ")
    )]
}

fn name_errors(meta: &Map<String, Value>, dir_name: &str) -> (Vec<String>, String) {
    match meta.get("name") {
        None => (
            vec!["Missing required field in frontmatter: name".into()],
            String::new(),
        ),
        Some(Value::String(s)) => shape_errors(s, dir_name),
        Some(_) => (
            vec!["Field 'name' must be a non-empty string".into()],
            String::new(),
        ),
    }
}

fn shape_errors(raw: &str, dir_name: &str) -> (Vec<String>, String) {
    let name = raw.trim();
    let mut errors = Vec::new();
    if name.is_empty() {
        errors.push("Field 'name' must be a non-empty string".into());
        return (errors, String::new());
    }
    push_name_shape(&mut errors, name);
    if dir_name != name {
        errors.push(format!(
            "Directory name '{dir_name}' must match skill name '{name}'"
        ));
    }
    (errors, name.to_string())
}

fn push_name_shape(errors: &mut Vec<String>, name: &str) {
    let chars = name.chars().count();
    if chars > NAME_MAX {
        errors.push(format!(
            "Skill name '{name}' exceeds {NAME_MAX} character limit ({chars} chars)"
        ));
    }
    if name.chars().any(|c| c.is_uppercase()) {
        errors.push(format!("Skill name '{name}' must be lowercase"));
    }
    if name.starts_with('-') || name.ends_with('-') {
        errors.push("Skill name cannot start or end with a hyphen".into());
    }
    if name.contains("--") {
        errors.push("Skill name cannot contain consecutive hyphens".into());
    }
    if name.chars().any(|c| c != '-' && !c.is_alphanumeric()) {
        errors.push(format!(
            "Skill name '{name}' contains invalid characters. Only letters, digits, and hyphens are allowed."
        ));
    }
}

fn description_of(meta: &Map<String, Value>, errors: &mut Vec<String>) -> String {
    match meta.get("description") {
        None => {
            errors.push("Missing required field in frontmatter: description".into());
            String::new()
        }
        Some(Value::String(s)) if s.trim().is_empty() => {
            errors.push("Field 'description' must be a non-empty string".into());
            String::new()
        }
        Some(Value::String(s)) => {
            push_description_length(errors, s);
            s.clone()
        }
        Some(_) => {
            errors.push("Field 'description' must be a non-empty string".into());
            String::new()
        }
    }
}

fn push_description_length(errors: &mut Vec<String>, description: &str) {
    let n = description.chars().count();
    if n > DESCRIPTION_MAX {
        errors.push(format!(
            "Description exceeds {DESCRIPTION_MAX} character limit ({n} chars)"
        ));
    }
}

fn optional_strings(meta: &Map<String, Value>, errors: &mut Vec<String>) {
    string_type(meta, "license", errors);
    string_limit(meta, "compatibility", COMPATIBILITY_MAX, errors);
    string_type(meta, "allowed-tools", errors);
}

fn string_type(meta: &Map<String, Value>, key: &str, errors: &mut Vec<String>) {
    if meta.get(key).is_some_and(|v| !v.is_string()) {
        errors.push(format!("Field '{key}' must be a string"));
    }
}

fn string_limit(meta: &Map<String, Value>, key: &str, max: usize, errors: &mut Vec<String>) {
    match meta.get(key) {
        Some(Value::String(s)) => {
            let n = s.chars().count();
            if n > max {
                errors.push(format!(
                    "Field '{key}' exceeds {max} character limit ({n} chars)"
                ));
            }
        }
        Some(_) => errors.push(format!("Field '{key}' must be a string")),
        None => {}
    }
}

fn metadata_errors(meta: &Map<String, Value>, errors: &mut Vec<String>) {
    let Some(v) = meta.get("metadata") else {
        return;
    };
    let Some(obj) = v.as_object() else {
        errors.push("Field 'metadata' must be a mapping of strings".into());
        return;
    };
    for (k, val) in obj {
        if !val.is_string() {
            errors.push(format!("metadata.{k} must be a string"));
        }
    }
}

fn words(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_alphanumeric() && c != '\'')
        .filter(|w| !w.is_empty())
        .map(str::to_lowercase)
        .collect()
}

fn listed_words(text: &str, want: &[&str]) -> Vec<String> {
    let mut found = BTreeSet::new();
    for word in words(text) {
        if want.contains(&word.as_str()) {
            found.insert(word);
        }
    }
    found.into_iter().collect()
}

fn numbered_steps(body: &str) -> i64 {
    body.lines().filter(|l| step_line(l)).count() as i64
}

fn step_line(line: &str) -> bool {
    let t = line.trim_start();
    let Some((n, rest)) = t.split_once('.') else {
        return false;
    };
    !n.is_empty()
        && n.bytes().all(|b| b.is_ascii_digit())
        && rest.starts_with(|c: char| c.is_whitespace())
        && !rest.trim().is_empty()
}

fn code_fences(body: &str) -> i64 {
    let mut open = false;
    let mut blocks = 0i64;
    for line in body.lines() {
        if line.trim_start().starts_with("```") {
            if !open {
                blocks += 1;
            }
            open = !open;
        }
    }
    blocks
}

fn boundary_cues(body: &str) -> Vec<String> {
    let lower = body.to_lowercase();
    let mut found = BTreeSet::new();
    for phrase in [
        "do not",
        "on error",
        "if it fails",
        "if this fails",
        "error handling",
    ] {
        if lower.contains(phrase) {
            found.insert(phrase.to_string());
        }
    }
    for word in words(&lower) {
        if is_boundary_word(&word) {
            found.insert(word);
        }
    }
    found.into_iter().collect()
}

fn is_boundary_word(w: &str) -> bool {
    matches!(
        w,
        "never"
            | "don't"
            | "limitation"
            | "limitations"
            | "prerequisite"
            | "prerequisites"
            | "troubleshooting"
    )
}

/// The skill's files as text, and how many couldn't be read as UTF-8.
/// Images, fonts and archives a skill may ship. An agent doesn't read them as instructions, so they
/// are neither scanned nor counted as unreadable.
const BINARY: [&str; 12] = [
    "png", "jpg", "jpeg", "gif", "webp", "ico", "pdf", "woff", "woff2", "ttf", "zip", "gz",
];

fn files_under(root: &Path, dir: &str) -> (Vec<(String, String)>, i64) {
    let (mut out, mut unreadable) = (Vec::new(), 0);
    for path in walk_files(&root.join(dir)) {
        let binary = path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| BINARY.contains(&e.to_ascii_lowercase().as_str()));
        if binary {
            continue;
        }
        let rel = rel_of(root, &path);
        match std::fs::read_to_string(&path) {
            Ok(text) => out.push((rel, text)),
            Err(_) => unreadable += 1,
        }
    }
    out.sort();
    (out, unreadable)
}

fn walk_files(dir: &Path) -> Vec<PathBuf> {
    walkdir::WalkDir::new(dir)
        .into_iter()
        .flatten()
        .filter(|e| e.file_type().is_file())
        .map(|e| e.into_path())
        .collect()
}

fn rel_of(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn support_files(files: &[(String, String)], skill_md: &str) -> i64 {
    files.iter().filter(|(p, _)| p != skill_md).count() as i64
}

fn scan_files(files: &[(String, String)], windows_re: &Regex) -> (i64, Vec<String>, Vec<String>) {
    let mut hidden = 0i64;
    let mut secrets = Vec::new();
    let mut windows = Vec::new();
    for (path, content) in files {
        let content = strip_bom(content);
        hidden += hidden_count(content);
        secrets.extend(crate::hygiene::secrets(path, content));
        windows.extend(windows_in(path, content, windows_re));
    }
    (hidden, secrets, windows)
}

fn hidden_count(text: &str) -> i64 {
    crate::hygiene::hidden_count(text) + crate::hygiene::html_comments(text)
}

fn windows_in(rel: &str, text: &str, re: &Regex) -> Vec<String> {
    let mut hits = Vec::new();
    for (n, line) in text.lines().enumerate() {
        if re.is_match(line) {
            hits.push(format!("{}:{}", rel, n + 1));
        }
    }
    hits
}

fn eval_cases(root: &Path, dir: &str) -> i64 {
    let path = root.join(dir).join("evals").join("evals.json");
    let Ok(text) = std::fs::read_to_string(path) else {
        return 0;
    };
    let Ok(v) = serde_json::from_str::<Value>(&text) else {
        return 0;
    };
    cases_in(&v)
}

fn cases_in(v: &Value) -> i64 {
    if let Some(a) = v.as_array() {
        return a.len() as i64;
    }
    v.get("evals")
        .and_then(Value::as_array)
        .map(|a| a.len() as i64)
        .unwrap_or(0)
}

fn md_targets(text: &str) -> Vec<String> {
    let re = compile(r"\]\(([^)\s]+)\)");
    re.captures_iter(text)
        .filter_map(|c| c.get(1).map(|m| m.as_str().to_string()))
        .collect()
}

fn resolve(from_file: &str, target: &str) -> Option<String> {
    let bare = target.split(['#', '?']).next().unwrap_or(target);
    if bare.is_empty() || bare.contains("://") || bare.starts_with("mailto:") {
        return None;
    }
    let combined = if let Some(abs) = bare.strip_prefix('/') {
        abs.to_string()
    } else {
        let base = parent(from_file);
        if base.is_empty() {
            bare.to_string()
        } else {
            format!("{base}/{bare}")
        }
    };
    normalize(&combined)
}

fn normalize(path: &str) -> Option<String> {
    let mut stack = Vec::new();
    for part in path.split('/') {
        if part.is_empty() || part == "." {
            continue;
        }
        if part == ".." {
            stack.pop()?;
            continue;
        }
        stack.push(part);
    }
    Some(stack.join("/"))
}

fn missing_files(root: &Path, rel: &str, text: &str) -> Vec<String> {
    let mut out = BTreeSet::new();
    for target in md_targets(text) {
        match resolve(rel, &target) {
            Some(path) if root.join(&path).is_file() => {}
            Some(path) => {
                out.insert(path);
            }
            None if local_target(&target) => {
                out.insert(target);
            }
            None => {}
        }
    }
    out.into_iter().collect()
}

fn local_target(target: &str) -> bool {
    let bare = target.split(['#', '?']).next().unwrap_or(target);
    !bare.is_empty() && !bare.contains("://") && !bare.starts_with("mailto:")
}

fn link_depth(root: &Path, start: &str) -> i64 {
    let mut seen = BTreeSet::new();
    let mut max = 0i64;
    let mut stack = vec![(start.to_string(), 0i64)];
    while let Some((file, depth)) = stack.pop() {
        if depth >= 4 || !seen.insert(file.clone()) {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(root.join(&file)) else {
            continue;
        };
        follow_links(root, &file, depth, &text, &mut max, &mut stack);
    }
    max
}

fn follow_links(
    root: &Path,
    file: &str,
    depth: i64,
    text: &str,
    max: &mut i64,
    stack: &mut Vec<(String, i64)>,
) {
    for target in md_targets(text) {
        let Some(next) = resolve(file, &target) else {
            continue;
        };
        if next.ends_with(".md") && root.join(&next).is_file() {
            *max = (*max).max(depth + 1);
            stack.push((next, depth + 1));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Tmp(PathBuf);
    impl Drop for Tmp {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn scratch(name: &str, files: &[(&str, &str)]) -> Tmp {
        let root = std::env::temp_dir().join(format!(
            "toolbox-skill-{name}-{}-{}",
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

    fn good_md() -> &'static str {
        "---\nname: focused\ndescription: Parses portal plans. Use when reviewing a toolbox plan before merge.\n---\n\n1. Read the plan.\n2. Note gates.\n3. Stop before acknowledging.\n\nNever merge it yourself.\n\n```\ntoolbox plan\n```\n\nSee [the reference](references/REFERENCE.md).\n"
    }

    #[test]
    fn reads_a_focused_skill() {
        let reference = "Further detail. Do not follow another file.\n";
        let evals = r#"{"evals":[{"prompt":"a"},{"prompt":"b"},{"prompt":"c"}]}"#;
        let tmp = scratch(
            "good",
            &[
                ("plugins/demo/skills/focused/SKILL.md", good_md()),
                (
                    "plugins/demo/skills/focused/references/REFERENCE.md",
                    reference,
                ),
                ("plugins/demo/skills/focused/evals/evals.json", evals),
            ],
        );
        let rec = analyse(&tmp.0, "plugins/demo/skills/focused/SKILL.md");
        assert_eq!(rec["frontmatterErrors"], json!([]));
        assert_eq!(rec["name"], "focused");
        assert_eq!(rec["plugin"], "demo");
        assert_eq!(rec["triggerCues"], json!(["before", "when"]));
        assert_eq!(rec["pronouns"], json!([]));
        assert_eq!(rec["numberedSteps"], 3);
        assert_eq!(rec["codeFences"], 1);
        assert_eq!(rec["missingFiles"], json!([]));
        assert_eq!(rec["boundaryCues"], json!(["never"]));
        assert_eq!(rec["evalCases"], 3);
        assert_eq!(rec["linkDepth"], 1);
        assert_eq!(rec["supportFiles"], 2);
        assert_eq!(rec["hiddenChars"], 0);
        assert!(rec["estimatedTokens"].as_i64().unwrap() < 5000);
    }

    #[test]
    fn reports_spec_errors_secrets_and_hidden_text() {
        let md = "---\nname: Unsafe\nversion: 1\ndescription: You should use this.\n---\n\nRun C:\\Windows\\cmd.exe\nAKIAIOSFODNN7EXAMPLE\n\u{200b}\n";
        let tmp = scratch("bad", &[("plugins/demo/skills/unsafe/SKILL.md", md)]);
        let rec = analyse(&tmp.0, "plugins/demo/skills/unsafe/SKILL.md");
        let errors = rec["frontmatterErrors"].as_array().unwrap();
        assert!(
            errors
                .iter()
                .any(|e| e.as_str().unwrap().contains("lowercase")),
            "{errors:?}"
        );
        assert!(
            errors
                .iter()
                .any(|e| e.as_str().unwrap().contains("Unexpected")),
            "{errors:?}"
        );
        assert!(
            errors
                .iter()
                .any(|e| e.as_str().unwrap().contains("must match")),
            "{errors:?}"
        );
        assert_eq!(rec["pronouns"], json!(["you"]));
        assert_eq!(rec["triggerCues"], json!([]));
        assert_eq!(
            rec["secretHits"],
            json!(["plugins/demo/skills/unsafe/SKILL.md:8 aws-access-key"])
        );
        assert_eq!(
            rec["windowsPaths"],
            json!(["plugins/demo/skills/unsafe/SKILL.md:7"])
        );
        assert_eq!(rec["hiddenChars"], 1);
    }

    #[test]
    fn counts_a_json_array_of_evals_and_a_broken_link() {
        let md = "---\nname: partial\ndescription: Use before editing a portal manifest that already exists.\n---\n\nSee [missing](references/missing.md).\n";
        let tmp = scratch(
            "partial",
            &[
                ("skills/partial/SKILL.md", md),
                ("skills/partial/evals/evals.json", "[{\"q\":1},{\"q\":2}]"),
            ],
        );
        let rec = analyse(&tmp.0, "skills/partial/SKILL.md");
        assert_eq!(rec["frontmatterErrors"], json!([]));
        assert_eq!(rec["plugin"], "");
        assert_eq!(rec["evalCases"], 2);
        assert_eq!(
            rec["missingFiles"],
            json!(["skills/partial/references/missing.md"])
        );
        assert_eq!(rec["supportFiles"], 1);
    }

    #[test]
    fn rejects_a_file_without_frontmatter() {
        let tmp = scratch("none", &[("skills/none/SKILL.md", "# No frontmatter\n")]);
        let rec = analyse(&tmp.0, "skills/none/SKILL.md");
        assert_eq!(
            rec["frontmatterErrors"],
            json!(["SKILL.md must start with YAML frontmatter (---)"])
        );
    }

    #[test]
    fn operator_skills_meet_the_bronze_facts() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        for name in [
            "add-permission",
            "add-scorecard",
            "review-plan",
            "toolbox-authoring",
        ] {
            let rel = format!("plugins/toolbox-operator/skills/{name}/SKILL.md");
            let rec = analyse(&root, &rel);
            assert_eq!(
                rec["frontmatterErrors"],
                json!([]),
                "{name}: {}",
                rec["frontmatterErrors"]
            );
            assert!(
                !rec["triggerCues"].as_array().unwrap().is_empty(),
                "{name} description has no when/before/after cue: {}",
                rec["description"]
            );
            assert_eq!(rec["pronouns"], json!([]), "{name}");
            assert!(rec["descriptionChars"].as_i64().unwrap() >= 20, "{name}");
            assert!(rec["numberedSteps"].as_i64().unwrap() >= 3, "{name}");
        }
    }
}
