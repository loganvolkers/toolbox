//! `expo-app` records: one per Expo app config matching configured globs
//! (packages/pnpm-expo, D01M7Y6X38AERPBT0YH1AG5TBRD).

use regex::Regex;
use serde_json::{Value, json};
use std::path::Path;

/// One Expo app beside `app_config_rel` (e.g. `apps/mobile/app.json`).
pub fn analyse(root: &Path, app_config_rel: &str, paths: &[String]) -> Value {
    let dir = Path::new(app_config_rel)
        .parent()
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| ".".into());
    let pkg_rel = if dir == "." {
        "package.json".into()
    } else {
        format!("{dir}/package.json")
    };
    let pkg = read_json(root, &pkg_rel);
    let name = pkg["name"].as_str().unwrap_or("").trim().to_owned();
    let expo_version = dep_version(&pkg, "expo");
    let has_expo = !expo_version.is_empty();
    let has_react = dep_present(&pkg, "react");
    let has_react_native = dep_present(&pkg, "react-native");
    let app_config = app_config_path(root, &dir, app_config_rel);
    let eas_path = eas_beside(root, &dir);
    let root_eas = root.join("eas.json").is_file();
    let conflicting_root_eas = !eas_path.is_empty() && root_eas && dir != ".";
    let profiles = eas_profiles(root, &eas_path);
    let metro = metro_uses_expo_default(root, &dir);
    let (doctor_pinned, doctor_pin) = doctor_pinned_in_ci(root, paths, &dir);
    json!({
        "path": dir,
        "name": name,
        "hasExpoDep": has_expo,
        "expoVersion": expo_version,
        "hasAppConfig": !app_config.is_empty(),
        "appConfig": app_config,
        "easPath": eas_path,
        "hasEas": !eas_path.is_empty(),
        "conflictingRootEas": conflicting_root_eas,
        "singleCanonicalEas": !conflicting_root_eas,
        "hasReact": has_react,
        "hasReactNative": has_react_native,
        "metroUsesExpoDefault": metro,
        "easProfiles": profiles,
        "doctorPinnedInCi": doctor_pinned,
        "doctorPin": doctor_pin,
    })
}

fn read_json(root: &Path, rel: &str) -> Value {
    std::fs::read_to_string(root.join(rel))
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or(Value::Null)
}

fn dep_version(pkg: &Value, name: &str) -> String {
    for section in ["dependencies", "devDependencies", "peerDependencies"] {
        if let Some(v) = pkg[section][name].as_str() {
            return v.trim().trim_start_matches(['^', '~', '=']).to_owned();
        }
    }
    String::new()
}

fn dep_present(pkg: &Value, name: &str) -> bool {
    !dep_version(pkg, name).is_empty()
}

fn app_config_path(root: &Path, dir: &str, preferred: &str) -> String {
    if root.join(preferred).is_file() {
        return preferred.to_owned();
    }
    let candidates = [
        "app.json",
        "app.config.js",
        "app.config.ts",
        "app.config.mjs",
        "app.config.cjs",
    ];
    for name in candidates {
        let rel = join(dir, name);
        if root.join(&rel).is_file() {
            return rel;
        }
    }
    String::new()
}

fn eas_beside(root: &Path, dir: &str) -> String {
    let rel = join(dir, "eas.json");
    if root.join(&rel).is_file() {
        rel
    } else {
        String::new()
    }
}

fn eas_profiles(root: &Path, eas_rel: &str) -> Vec<String> {
    if eas_rel.is_empty() {
        return Vec::new();
    }
    let doc = read_json(root, eas_rel);
    let mut names: Vec<String> = doc["build"]
        .as_object()
        .into_iter()
        .flatten()
        .map(|(k, _)| k.clone())
        .collect();
    names.sort();
    names
}

fn metro_uses_expo_default(root: &Path, dir: &str) -> bool {
    for name in [
        "metro.config.js",
        "metro.config.cjs",
        "metro.config.mjs",
        "metro.config.ts",
    ] {
        let rel = join(dir, name);
        let Ok(text) = std::fs::read_to_string(root.join(&rel)) else {
            continue;
        };
        return text.contains("expo/metro-config");
    }
    // No metro.config: Expo's default applies.
    true
}

fn doctor_pinned_in_ci(root: &Path, paths: &[String], dir: &str) -> (bool, String) {
    let re = Regex::new(r"expo-doctor@([0-9]+\.[0-9]+\.[0-9]+)").unwrap();
    let mut pin = String::new();
    for rel in paths.iter().filter(|p| {
        p.starts_with(".github/workflows/") && (p.ends_with(".yml") || p.ends_with(".yaml"))
    }) {
        let text = std::fs::read_to_string(root.join(rel)).unwrap_or_default();
        if !text.contains("expo-doctor") {
            continue;
        }
        // Prefer a pin that mentions the app directory, else any pin in the repo.
        let mentions_dir = dir == "." || text.contains(dir);
        if let Some(c) = re.captures(&text) {
            let found = c.get(1).map(|m| m.as_str().to_owned()).unwrap_or_default();
            if mentions_dir {
                return (true, found);
            }
            if pin.is_empty() {
                pin = found;
            }
        } else if mentions_dir {
            // Present but unpinned (e.g. bare `expo-doctor`).
            return (false, String::new());
        }
    }
    (!pin.is_empty(), pin)
}

fn join(dir: &str, name: &str) -> String {
    if dir == "." {
        name.to_owned()
    } else {
        format!("{dir}/{name}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Tmp(std::path::PathBuf);
    impl Drop for Tmp {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn scratch(name: &str, files: &[(&str, &str)]) -> Tmp {
        let root = std::env::temp_dir().join(format!("seikika-expo-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        for (rel, text) in files {
            let path = root.join(rel);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, text).unwrap();
        }
        Tmp(root)
    }

    #[test]
    fn a_mobile_app_with_pinned_doctor_and_duplicate_root_eas() {
        let tmp = scratch(
            "mobile",
            &[
                (
                    "apps/mobile/package.json",
                    r#"{"name":"@acme/mobile","dependencies":{"expo":"58.0.25","react":"19.3.0","react-native":"0.88.1"}}"#,
                ),
                ("apps/mobile/app.json", r#"{"expo":{"name":"Acme"}}"#),
                (
                    "apps/mobile/eas.json",
                    r#"{"build":{"development":{},"preview":{},"production":{}}}"#,
                ),
                (
                    "apps/mobile/metro.config.js",
                    "const { getDefaultConfig } = require('expo/metro-config');\nmodule.exports = getDefaultConfig(__dirname);\n",
                ),
                (
                    "eas.json",
                    r#"{"cli":{"version":">= 27.9.0"},"build":{"development":{},"preview":{},"production":{}}}"#,
                ),
                (
                    ".github/workflows/ci.yml",
                    "jobs:\n  mobile:\n    defaults: {run: {working-directory: apps/mobile}}\n    steps:\n      - run: npx -y expo-doctor@1.21.5\n",
                ),
            ],
        );
        let paths = vec![
            "apps/mobile/package.json".into(),
            "apps/mobile/app.json".into(),
            "apps/mobile/eas.json".into(),
            "apps/mobile/metro.config.js".into(),
            "eas.json".into(),
            ".github/workflows/ci.yml".into(),
        ];
        let rec = analyse(&tmp.0, "apps/mobile/app.json", &paths);
        assert_eq!(rec["hasExpoDep"], true);
        assert_eq!(rec["expoVersion"], "58.0.25");
        assert_eq!(rec["hasAppConfig"], true);
        assert_eq!(rec["conflictingRootEas"], true);
        assert_eq!(rec["singleCanonicalEas"], false);
        assert_eq!(rec["metroUsesExpoDefault"], true);
        assert_eq!(
            rec["easProfiles"],
            json!(["development", "preview", "production"])
        );
        assert_eq!(rec["doctorPinnedInCi"], true);
        assert_eq!(rec["doctorPin"], "1.21.5");
    }

    #[test]
    fn canonical_eas_when_only_the_app_has_eas_json() {
        let tmp = scratch(
            "canonical",
            &[
                (
                    "apps/mobile/package.json",
                    r#"{"dependencies":{"expo":"58.0.0","react":"19.0.0","react-native":"0.76.0"}}"#,
                ),
                ("apps/mobile/app.json", "{}"),
                (
                    "apps/mobile/eas.json",
                    r#"{"build":{"development":{},"preview":{},"production":{}}}"#,
                ),
            ],
        );
        let paths = vec![
            "apps/mobile/package.json".into(),
            "apps/mobile/app.json".into(),
            "apps/mobile/eas.json".into(),
        ];
        let rec = analyse(&tmp.0, "apps/mobile/app.json", &paths);
        assert_eq!(rec["singleCanonicalEas"], true);
        assert_eq!(rec["conflictingRootEas"], false);
    }
}
