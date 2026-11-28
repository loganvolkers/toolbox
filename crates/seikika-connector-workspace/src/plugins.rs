//! Agent plugin packages (`plugin.json` plus `mcp.json`), read statically
//! (docs/design/plugin-standards.md).
//!
//! One record per `plugin.json`. Facts only: the Agent Plugins 1.0.0 manifest rules a file can
//! prove, whether `mcp.json` is the spec file beside it, and whether a skill sits at the one
//! portable path. Thresholds live in the manifest.

use regex::Regex;
use seikika_model::schema;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

pub(crate) fn analyse(root: &Path, rel: &str) -> Value {
    let text = std::fs::read_to_string(root.join(rel)).unwrap_or_default();
    let doc: Value = serde_json::from_str(&text).unwrap_or(Value::Null);
    let dir = Path::new(rel)
        .parent()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default();
    let name = doc["name"].as_str().unwrap_or("").to_owned();
    let description = doc["description"].as_str().unwrap_or("").trim();
    let mcp_path = format!("{dir}/mcp.json");
    let mcp_text = std::fs::read_to_string(root.join(&mcp_path)).unwrap_or_default();
    let mcp: Value = serde_json::from_str(&mcp_text).unwrap_or(Value::Null);
    let has_mcp = root.join(&mcp_path).is_file();
    let (command_token, header_secrets, path_escapes) = mcp_facts(&mcp);
    let mut secret_hits = secrets(rel, &text);
    if has_mcp {
        secret_hits.extend(secrets(&mcp_path, &mcp_text));
    }
    secret_hits.sort();
    secret_hits.dedup();
    json!({
        "path": rel,
        "dir": dir,
        "name": name,
        "nameOk": name_ok(&name),
        "descriptionChars": description.chars().count() as i64,
        "schemaOk": schema::agent_plugin_manifest_valid(&doc),
        "hasMcp": has_mcp,
        "mcpSchemaOk": has_mcp && schema::agent_plugin_mcp_valid(&mcp),
        "commandToken": command_token,
        "headerSecrets": header_secrets,
        "pathEscapes": path_escapes,
        "misplacedSkills": misplaced_skills(root, &dir),
        "secretHits": secret_hits,
    })
}

/// Agent Plugins 1.0.0 `name`: 1–64 chars, `a-z`, digits, `.` and `-`, alphanumeric at both ends,
/// no `--` and no `..`.
fn name_ok(name: &str) -> bool {
    let n = name.chars().count();
    (1..=64).contains(&n)
        && !name.contains("--")
        && !name.contains("..")
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '.')
        && name
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphanumeric())
        && name
            .chars()
            .last()
            .is_some_and(|c| c.is_ascii_alphanumeric())
}

/// `(every stdio command is one token, header values that look like secrets, paths that leave the root)`.
fn mcp_facts(mcp: &Value) -> (bool, Vec<String>, Vec<String>) {
    let Some(servers) = mcp["mcpServers"].as_object() else {
        return (true, Vec::new(), Vec::new());
    };
    let mut command_token = true;
    let mut header_secrets = Vec::new();
    let mut path_escapes = Vec::new();
    for (name, server) in servers {
        if let Some(command) = server["command"].as_str() {
            if !command_token_ok(command) {
                command_token = false;
            }
            if escapes(command) {
                path_escapes.push(format!("{name} command"));
            }
        }
        for arg in server["args"].as_array().into_iter().flatten() {
            if let Some(arg) = arg.as_str()
                && escapes(arg)
            {
                path_escapes.push(format!("{name} arg"));
            }
        }
        if let Some(headers) = server["headers"].as_object() {
            for (header, value) in headers {
                let value = value.as_str().unwrap_or("");
                if secret_shape(value) {
                    header_secrets.push(format!("{name} header {header}"));
                }
            }
        }
    }
    header_secrets.sort();
    path_escapes.sort();
    (command_token, header_secrets, path_escapes)
}

/// A single token: bare, `./…`, or rooted at `${PLUGIN_ROOT}` / `${PLUGIN_DATA}`. No spaces.
fn command_token_ok(command: &str) -> bool {
    let re =
        compile(r"^(?:(?:\./|\$\{PLUGIN_ROOT\}/|\$\{PLUGIN_DATA\}/)?)[A-Za-z0-9][A-Za-z0-9._/-]*$");
    !command.is_empty() && !escapes(command) && re.is_match(command)
}

fn escapes(path: &str) -> bool {
    path.split(['/', '\\']).any(|p| p == "..")
}

/// `SKILL.md` files that are not `skills/<name>/SKILL.md` under the plugin root.
fn misplaced_skills(root: &Path, dir: &str) -> Vec<String> {
    let mut bad = Vec::new();
    let plugin_root = root.join(dir);
    let mut stack = vec![plugin_root.clone()];
    let reverse_domain = compile(r"^[a-z0-9]+(?:-[a-z0-9]+)*(?:\.[a-z0-9]+(?:-[a-z0-9]+)*)+$");
    while let Some(current) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&current) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if matches!(name.as_ref(), "node_modules" | "target" | ".git") {
                continue;
            }
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            if file_type.is_dir() {
                if current != plugin_root || !reverse_domain.is_match(&name) {
                    stack.push(path);
                }
                continue;
            }
            if name != "SKILL.md" {
                continue;
            }
            if skill_path(root, dir, &path).is_none() {
                bad.push(rel_under(root, &path));
            }
        }
    }
    bad.sort();
    bad
}

/// `skills/<one segment>/SKILL.md`, relative to the plugin directory.
fn skill_path(root: &Path, dir: &str, file: &Path) -> Option<()> {
    let rel = rel_under(root, file);
    let rest = rel.strip_prefix(&format!("{dir}/skills/"))?;
    let mut parts = rest.split('/');
    let skill = parts.next()?;
    let leaf = parts.next()?;
    if parts.next().is_some() || leaf != "SKILL.md" || skill.is_empty() || skill.contains('.') {
        return None;
    }
    Some(())
}

fn rel_under(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .components()
        .map(|c| c.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

fn secret_shape(text: &str) -> bool {
    compile(r"AKIA[0-9A-Z]{16}|ghp_[A-Za-z0-9]{36}").is_match(text)
}

fn secrets(rel: &str, text: &str) -> Vec<String> {
    let name = PathBuf::from(rel)
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| rel.to_owned());
    crate::hygiene::secrets(&name, text)
}

fn compile(src: &str) -> Regex {
    Regex::new(src).unwrap()
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
        let root =
            std::env::temp_dir().join(format!("seikika-plugin-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        for (rel, text) in files {
            let path = root.join(rel);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, text).unwrap();
        }
        Tmp(root)
    }

    #[test]
    fn schema_urls_alone_do_not_make_plugin_files_valid() {
        let plugin = r#"{
            "$schema": "https://agent-plugins.org/schemas/1.0.0/plugin.schema.json",
            "name": "example",
            "version": 1,
            "description": "   "
        }"#;
        let mcp = r#"{
            "$schema": "https://agent-plugins.org/schemas/1.0.0/mcp.schema.json"
        }"#;
        let tmp = scratch(
            "invalid-schema",
            &[
                ("plugins/example/plugin.json", plugin),
                ("plugins/example/mcp.json", mcp),
            ],
        );
        let rec = analyse(&tmp.0, "plugins/example/plugin.json");
        assert_eq!(rec["nameOk"], true);
        assert_eq!(rec["descriptionChars"], 0);
        assert_eq!(rec["schemaOk"], false);
        assert_eq!(rec["hasMcp"], true);
        assert_eq!(rec["mcpSchemaOk"], false);
    }

    #[test]
    fn a_spec_plugin_with_stdio_mcp_is_closed() {
        let plugin = r#"{
            "$schema": "https://agent-plugins.org/schemas/1.0.0/plugin.schema.json",
            "name": "seikika-operator",
            "description": "Skills and a local MCP server for operators."
        }"#;
        let mcp = r#"{
            "$schema": "https://agent-plugins.org/schemas/1.0.0/mcp.schema.json",
            "mcpServers": {"dev": {"type": "stdio", "command": "./bin/seikika-mcp", "args": ["--data", "${PLUGIN_DATA}"]}}
        }"#;
        let tmp = scratch(
            "good",
            &[
                ("plugins/demo/focused/plugin.json", plugin),
                ("plugins/demo/focused/mcp.json", mcp),
                ("plugins/demo/focused/skills/author/SKILL.md", "# Skill\n"),
            ],
        );
        let rec = analyse(&tmp.0, "plugins/demo/focused/plugin.json");
        assert_eq!(rec["nameOk"], true);
        assert_eq!(rec["schemaOk"], true);
        assert_eq!(rec["hasMcp"], true);
        assert_eq!(rec["mcpSchemaOk"], true);
        assert_eq!(rec["commandToken"], true);
        assert_eq!(rec["misplacedSkills"], json!([]));
        assert_eq!(rec["headerSecrets"], json!([]));
        assert_eq!(rec["pathEscapes"], json!([]));
        assert_eq!(rec["secretHits"], json!([]));
    }

    #[test]
    fn a_shell_command_a_secret_header_and_a_skill_outside_skills_fail() {
        let plugin = r#"{
            "$schema": "https://agent-plugins.org/schemas/1.0.0/plugin.schema.json",
            "name": "Bad--name",
            "description": ""
        }"#;
        let mcp = r#"{
            "$schema": "https://agent-plugins.org/schemas/1.0.0/mcp.schema.json",
            "mcpServers": {"dev": {
                "command": "sh -c ./bin/tool",
                "args": ["../outside"],
                "headers": {"Authorization": "AKIAIOSFODNN7EXAMPLE"}
            }}
        }"#;
        let tmp = scratch(
            "bad",
            &[
                ("plugins/demo/unsafe/plugin.json", plugin),
                ("plugins/demo/unsafe/mcp.json", mcp),
                ("plugins/demo/unsafe/extra/SKILL.md", "# Nope\n"),
                (
                    "plugins/demo/unsafe/.claude/skills/example/SKILL.md",
                    "# Vendor location\n",
                ),
                (
                    "plugins/demo/unsafe/com.example.client/skills/example/SKILL.md",
                    "# Client extension\n",
                ),
            ],
        );
        let rec = analyse(&tmp.0, "plugins/demo/unsafe/plugin.json");
        assert_eq!(rec["nameOk"], false);
        assert_eq!(rec["descriptionChars"], 0);
        assert_eq!(rec["commandToken"], false);
        assert_eq!(rec["pathEscapes"], json!(["dev arg"]));
        assert_eq!(rec["headerSecrets"], json!(["dev header Authorization"]));
        assert_eq!(
            rec["misplacedSkills"],
            json!([
                "plugins/demo/unsafe/.claude/skills/example/SKILL.md",
                "plugins/demo/unsafe/extra/SKILL.md",
            ])
        );
        assert_eq!(rec["secretHits"], json!(["mcp.json:6 aws-access-key"]));
    }
}
