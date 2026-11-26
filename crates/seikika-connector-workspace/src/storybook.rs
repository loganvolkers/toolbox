//! `storybook` records: one per Storybook main config matching configured globs
//! (packages/storybook, D01M7YNZFP68W0DR24PEA6T3FVM).

use regex::Regex;
use serde_json::{Value, json};
use std::path::Path;

/// One Storybook project beside `config_rel` (e.g. `apps/mobile/.storybook/main.ts`).
pub fn analyse(root: &Path, config_rel: &str, paths: &[String]) -> Value {
    let config_path = config_rel.replace('\\', "/");
    let storybook_dir = Path::new(&config_path)
        .parent()
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|| ".storybook".into());
    let dir = Path::new(&storybook_dir)
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
    let storybook_version = dep_version(&pkg, "storybook");
    let has_storybook_dep = !storybook_version.is_empty();
    let a11y_in_deps = dep_present(&pkg, "@storybook/addon-a11y");

    let main_text = std::fs::read_to_string(root.join(&config_path)).unwrap_or_default();
    let stories_glob_declared = main_text.contains("stories:");
    let a11y_in_main =
        main_text.contains("@storybook/addon-a11y") || main_text.contains("addon-a11y");
    let has_a11y_addon = a11y_in_deps || a11y_in_main;
    let telemetry_disabled = main_text.contains("disableTelemetry")
        && (main_text.contains("disableTelemetry: true")
            || main_text.contains("disableTelemetry:true")
            || main_text.contains("\"disableTelemetry\": true")
            || main_text.contains("\"disableTelemetry\":true"));
    let framework = framework_name(&main_text);

    let story_file_count = count_story_files(paths, &dir);
    let has_stories = story_file_count > 0;
    let a11y_fails = a11y_fails_on_violation(root, &storybook_dir);
    let (ci_builds, ci_tests) = ci_storybook_signals(root, paths, &dir);

    json!({
        "path": dir,
        "name": name,
        "configPath": config_path,
        "hasConfig": root.join(&config_path).is_file(),
        "storiesGlobDeclared": stories_glob_declared,
        "storyFileCount": story_file_count,
        "hasStories": has_stories,
        "hasStorybookDep": has_storybook_dep,
        "storybookVersion": storybook_version,
        "hasA11yAddon": has_a11y_addon,
        "telemetryDisabled": telemetry_disabled,
        "a11yFailsOnViolation": a11y_fails,
        "ciBuildsStorybook": ci_builds,
        "ciTestsStories": ci_tests,
        "framework": framework,
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

fn framework_name(main_text: &str) -> String {
    // Prefer quoted package names in framework.name / framework: { name: '…' }.
    let re = Regex::new(r#"@storybook/[a-zA-Z0-9_/-]+"#).unwrap();
    for cap in re.find_iter(main_text) {
        let s = cap.as_str();
        if s.contains("addon") {
            continue;
        }
        return s.to_owned();
    }
    String::new()
}

fn count_story_files(paths: &[String], dir: &str) -> i64 {
    let prefix = if dir == "." {
        String::new()
    } else {
        format!("{dir}/")
    };
    paths
        .iter()
        .filter(|p| {
            if !prefix.is_empty() && !p.starts_with(&prefix) {
                return false;
            }
            let name = Path::new(p)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            name.contains(".stories.")
        })
        .count() as i64
}

fn a11y_fails_on_violation(root: &Path, storybook_dir: &str) -> bool {
    for name in [
        "preview.tsx",
        "preview.ts",
        "preview.jsx",
        "preview.js",
        "preview.mjs",
    ] {
        let rel = format!("{storybook_dir}/{name}");
        let Ok(text) = std::fs::read_to_string(root.join(&rel)) else {
            continue;
        };
        if !text.contains("a11y") {
            continue;
        }
        // Storybook 8+/9+/10: parameters.a11y.test = 'error' (or "error").
        if text.contains("test:")
            && (text.contains("'error'") || text.contains("\"error\"") || text.contains("`error`"))
        {
            return true;
        }
        if text.contains("test: 'error'")
            || text.contains("test: \"error\"")
            || text.contains("test:'error'")
            || text.contains("test:\"error\"")
        {
            return true;
        }
    }
    false
}

fn is_github_workflow(path: &str) -> bool {
    path.starts_with(".github/workflows/") && (path.ends_with(".yml") || path.ends_with(".yaml"))
}

fn workflow_builds_storybook(low: &str) -> bool {
    low.contains("storybook:build")
        || low.contains("storybook build")
        || low.contains("build-storybook")
}

fn workflow_tests_stories(low: &str) -> bool {
    if low.contains("test:stories") || low.contains("test-storybook") || low.contains("test-runner")
    {
        return true;
    }
    low.contains("playwright") && (low.contains("storybook") || low.contains("stories.spec"))
}

fn ci_storybook_signals(root: &Path, paths: &[String], dir: &str) -> (bool, bool) {
    let mut builds = false;
    let mut tests = false;
    for rel in paths.iter().filter(|p| is_github_workflow(p)) {
        let text = std::fs::read_to_string(root.join(rel)).unwrap_or_default();
        let low = text.to_lowercase();
        let mentions_dir = dir == "." || text.contains(dir);
        if workflow_builds_storybook(&low) && (mentions_dir || !builds) {
            builds = true;
        }
        if workflow_tests_stories(&low) && (mentions_dir || !tests) {
            tests = true;
        }
    }
    (builds, tests)
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
        let root =
            std::env::temp_dir().join(format!("seikika-storybook-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        for (rel, text) in files {
            let path = root.join(rel);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, text).unwrap();
        }
        Tmp(root)
    }

    #[test]
    fn a_healthy_mobile_storybook_reaches_gold_signals() {
        let tmp = scratch(
            "healthy",
            &[
                (
                    "apps/mobile/package.json",
                    r#"{"name":"@acme/mobile","devDependencies":{"storybook":"10.6.3","@storybook/addon-a11y":"10.6.3","@storybook/react-native-web-vite":"10.6.3"}}"#,
                ),
                (
                    "apps/mobile/.storybook/main.ts",
                    r#"
import type { StorybookConfig } from '@storybook/react-native-web-vite';
const config: StorybookConfig = {
  stories: ['../src/**/*.stories.@(ts|tsx)'],
  addons: ['@storybook/addon-a11y'],
  framework: { name: '@storybook/react-native-web-vite', options: {} },
  core: { disableTelemetry: true },
};
export default config;
"#,
                ),
                (
                    "apps/mobile/.storybook/preview.tsx",
                    r#"
const preview = { parameters: { a11y: { test: 'error' } } };
export default preview;
"#,
                ),
                ("apps/mobile/src/ui.stories.tsx", "export default {};\n"),
                (
                    ".github/workflows/ci.yml",
                    r#"
jobs:
  storybook:
    steps:
      - run: pnpm storybook:build
        working-directory: apps/mobile
      - run: pnpm test:stories
        working-directory: apps/mobile
"#,
                ),
            ],
        );
        let paths = vec![
            "apps/mobile/package.json".into(),
            "apps/mobile/.storybook/main.ts".into(),
            "apps/mobile/.storybook/preview.tsx".into(),
            "apps/mobile/src/ui.stories.tsx".into(),
            ".github/workflows/ci.yml".into(),
        ];
        let rec = analyse(&tmp.0, "apps/mobile/.storybook/main.ts", &paths);
        assert_eq!(rec["path"], "apps/mobile");
        assert_eq!(rec["hasConfig"], true);
        assert_eq!(rec["hasStorybookDep"], true);
        assert_eq!(rec["storybookVersion"], "10.6.3");
        assert_eq!(rec["hasStories"], true);
        assert_eq!(rec["storyFileCount"], 1);
        assert_eq!(rec["hasA11yAddon"], true);
        assert_eq!(rec["telemetryDisabled"], true);
        assert_eq!(rec["a11yFailsOnViolation"], true);
        assert_eq!(rec["ciBuildsStorybook"], true);
        assert_eq!(rec["ciTestsStories"], true);
        assert_eq!(rec["framework"], "@storybook/react-native-web-vite");
    }

    #[test]
    fn a_bare_config_without_deps_or_ci_fails_signals() {
        let tmp = scratch(
            "bare",
            &[
                ("apps/web/package.json", r#"{"name":"@acme/web"}"#),
                (
                    "apps/web/.storybook/main.js",
                    "module.exports = { stories: [] };\n",
                ),
            ],
        );
        let paths = vec![
            "apps/web/package.json".into(),
            "apps/web/.storybook/main.js".into(),
        ];
        let rec = analyse(&tmp.0, "apps/web/.storybook/main.js", &paths);
        assert_eq!(rec["hasConfig"], true);
        assert_eq!(rec["hasStorybookDep"], false);
        assert_eq!(rec["hasStories"], false);
        assert_eq!(rec["hasA11yAddon"], false);
        assert_eq!(rec["telemetryDisabled"], false);
        assert_eq!(rec["a11yFailsOnViolation"], false);
        assert_eq!(rec["ciBuildsStorybook"], false);
        assert_eq!(rec["ciTestsStories"], false);
    }
}
