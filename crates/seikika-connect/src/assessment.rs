//! Convert rubric assessments into catalog records for the portal.
//!
//! Assessments written to files and those pushed via `seikika assess submit` produce the same
//! `assessment` record. Validate the document against the [assessment schema] before passing it
//! to these helpers.
//!
//! [assessment schema]: https://github.com/seikika/seikika/blob/main/crates/seikika-model/schema/assessment.schema.json

use seikika_model::{Manifest, manifest::Rubric};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, sync::OnceLock};

/// Version of the rubric used to prepare an assessment. Documentation links can change without
/// invalidating a verdict.
pub fn rubric_hash(r: &Rubric) -> String {
    let mut definition = r.clone();
    definition.links.clear();
    let body = serde_json::to_vec(&definition).expect("rubric serializes");
    Sha256::digest(body)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// Problems with an assessment document against the schema and the portal's rubrics.
pub fn problems(m: &Manifest, doc: &Value) -> Vec<String> {
    let mut out = schema_problems(doc);
    out.extend(rubric_problems(m, doc));
    out
}

/// Schema errors that prevent an assessment from being read as a record.
pub fn schema_problems(doc: &Value) -> Vec<String> {
    seikika_model::schema::errors(seikika_model::schema::assessment(), doc)
        .into_iter()
        .map(|(p, e)| format!("{}: {e}", if p.is_empty() { "/" } else { &p }))
        .collect()
}

/// Missing or unrecognized criteria relative to the current portal rubric.
pub fn rubric_problems(m: &Manifest, doc: &Value) -> Vec<String> {
    let Some(r) = m
        .rubrics
        .iter()
        .find(|r| Some(r.name.as_str()) == doc["rubric"].as_str())
    else {
        return vec![format!("unknown rubric {}", doc["rubric"])];
    };
    let mut out = Vec::new();
    let mut named = BTreeSet::new();
    for n in doc["verdicts"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|v| v["criterion"].as_str())
    {
        if !named.insert(n) {
            out.push(format!("duplicate verdict for criterion {n}"));
        } else if !r.criteria.iter().any(|c| c.name == n) {
            out.push(format!("{n} is not a criterion of {}", r.name));
        }
    }
    for c in &r.criteria {
        if !named.contains(&c.name.as_str()) {
            out.push(format!("no verdict for criterion {}", c.name));
        }
    }
    out
}

/// Something a reader can check. Without a repository (`root` is `None`, e.g. `assess submit`
/// before the files are at hand), the shape alone: a path with a directory or a file extension, a
/// decision or question id (`D71`, `Q10`, a `D01…` ULID), or a URL. With the repository, each
/// path or file name must exist, and each id must be one the repository has, so "good and/or bad"
/// or "in 2026" cites nothing.
fn cites(reason: &str, root: Option<&std::path::Path>) -> bool {
    static PATH: OnceLock<regex::Regex> = OnceLock::new();
    static NAME: OnceLock<regex::Regex> = OnceLock::new();
    static ID: OnceLock<regex::Regex> = OnceLock::new();
    let path =
        PATH.get_or_init(|| regex::Regex::new(r"/?[\w.-]+(?:/[\w.<>*-]+)+").expect("valid regex"));
    let name = NAME.get_or_init(|| {
        regex::Regex::new(
            r"\b[\w-]+\.(?:md|mdx|rs|ya?ml|json|ndjson|toml|sh|py|[cm]?js|ts|tsx|fga)\b",
        )
        .expect("valid regex")
    });
    let id = ID.get_or_init(|| {
        regex::Regex::new(r"\b(?:D01[0-9A-Z]{20,}|D\d{1,3}|Q\d{1,3})\b").expect("valid regex")
    });
    static URL: OnceLock<regex::Regex> = OnceLock::new();
    let url = URL.get_or_init(|| {
        regex::Regex::new(r"https?://[A-Za-z0-9-]+(?:\.[A-Za-z0-9-]+)+(?:[/:#?]\S*)?")
            .expect("valid regex")
    });
    if url.is_match(reason) {
        return true;
    }
    let Some(root) = root else {
        return path
            .find_iter(reason)
            .any(|m| m.as_str().contains('.') || m.as_str().matches('/').count() >= 2)
            || name.is_match(reason)
            || id.is_match(reason);
    };
    !cited_paths(reason, root).is_empty()
        || path
            .find_iter(reason)
            .any(|m| relative_citation(m.as_str()).is_some_and(|p| root.join(p).is_dir()))
        || id.find_iter(reason).any(|m| known_id(root, m.as_str()))
}

fn relative_citation(path: &str) -> Option<&std::path::Path> {
    let path = std::path::Path::new(path.trim_end_matches(['.', ',', ')', ':']));
    (!path.as_os_str().is_empty()
        && path.components().all(|part| {
            matches!(
                part,
                std::path::Component::Normal(_) | std::path::Component::CurDir
            )
        }))
    .then_some(path)
}

/// The repository files a reason cites: paths that exist (trailing `.`, `,`, `)` or `:` and a
/// `:line` suffix dropped) and bare file names that exactly one file has, as repo-relative paths.
fn cited_paths(reason: &str, root: &std::path::Path) -> Vec<String> {
    static PATH: OnceLock<regex::Regex> = OnceLock::new();
    static NAME: OnceLock<regex::Regex> = OnceLock::new();
    let path =
        PATH.get_or_init(|| regex::Regex::new(r"/?[\w.-]+(?:/[\w.<>*-]+)+").expect("valid regex"));
    let name = NAME.get_or_init(|| {
        regex::Regex::new(
            r"\b[\w-]+\.(?:md|mdx|rs|ya?ml|json|ndjson|toml|sh|py|[cm]?js|ts|tsx|fga)\b",
        )
        .expect("valid regex")
    });
    let mut out = Vec::new();
    let explicit: Vec<_> = path.find_iter(reason).collect();
    for m in &explicit {
        if let Some(path) = relative_citation(m.as_str())
            && root.join(path).is_file()
        {
            out.push(path.to_string_lossy().trim_start_matches("./").to_owned());
        }
    }
    let names = file_names(root);
    for m in name.find_iter(reason) {
        if explicit
            .iter()
            .any(|path| path.start() <= m.start() && m.end() <= path.end())
        {
            continue;
        }
        if let Some([only]) = names.get(m.as_str()).map(Vec::as_slice) {
            out.push(only.clone());
        }
    }
    out.sort();
    out.dedup();
    out
}

/// Where each file name occurs in the repository (outside build output and dependencies), read
/// once per root. A bare name cites a file only when exactly one file has it: `README.md` names
/// dozens.
fn file_names(
    root: &std::path::Path,
) -> std::sync::Arc<std::collections::HashMap<String, Vec<String>>> {
    type Names = std::collections::HashMap<String, Vec<String>>;
    static CACHE: OnceLock<
        std::sync::Mutex<std::collections::HashMap<std::path::PathBuf, std::sync::Arc<Names>>>,
    > = OnceLock::new();
    let cache = CACHE.get_or_init(Default::default);
    let mut cache = cache.lock().expect("cache lock");
    cache
        .entry(root.to_path_buf())
        .or_insert_with(|| {
            let mut names = Names::new();
            for e in walkdir::WalkDir::new(root)
                .into_iter()
                .filter_entry(|e| {
                    let n = e.file_name().to_string_lossy();
                    !matches!(n.as_ref(), "target" | "node_modules" | ".git" | "dist")
                })
                .flatten()
                .filter(|e| e.file_type().is_file())
            {
                let rel = e
                    .path()
                    .strip_prefix(root)
                    .unwrap_or(e.path())
                    .to_string_lossy()
                    .into_owned();
                names
                    .entry(e.file_name().to_string_lossy().into_owned())
                    .or_default()
                    .push(rel);
            }
            std::sync::Arc::new(names)
        })
        .clone()
}

/// Whether the repository has this decision (a file in docs/decisions, or a backfilled file whose
/// `legacy:` field is the short id) or question (in docs/questions.md).
fn known_id(root: &std::path::Path, id: &str) -> bool {
    let read = |p: &str| std::fs::read_to_string(root.join(p)).unwrap_or_default();
    let word = |text: &str| {
        text.match_indices(id).any(|(i, _)| {
            let after = text[i + id.len()..].chars().next();
            !after.is_some_and(|c| c.is_ascii_alphanumeric())
        })
    };
    if id.starts_with("D01") {
        return root.join(format!("docs/decisions/{id}.md")).exists();
    }
    if id.starts_with('D') {
        let legacy = format!("legacy: {id}");
        return std::fs::read_dir(root.join("docs/decisions"))
            .into_iter()
            .flatten()
            .flatten()
            .any(|e| {
                std::fs::read_to_string(e.path())
                    .unwrap_or_default()
                    .lines()
                    .any(|l| l.trim_end() == legacy)
            });
    }
    word(&read("docs/questions.md"))
}

/// Returns the first 12 lowercase hex digits of the SHA-256 of the serialized assessment.
///
/// Use this value as the subject hash when writing a review of the assessment.
pub fn hash(doc: &Value) -> String {
    let text = serde_json::to_string(doc).unwrap_or_default();
    Sha256::digest(text.as_bytes())
        .iter()
        .take(6)
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// Builds the stable external ID `assessment:<rubric>/<entity>` from the document.
///
/// An assessment replaces an earlier one with the same rubric and entity. Validate both fields
/// first. A missing or non-string field becomes an empty ID segment.
pub fn external_id(doc: &Value) -> String {
    format!(
        "assessment:{}/{}",
        doc["rubric"].as_str().unwrap_or_default(),
        doc["entity"].as_str().unwrap_or_default()
    )
}

/// Builds catalog record data from a validated assessment document.
///
/// The overall verdict is `fail` if any criterion fails, otherwise `unsure` if any is unsure,
/// otherwise `pass`. The record includes the verdict lists, `hash`, and `path` when supplied.
/// For criteria that failed or were unsure, it identifies missing suggestions and citations.
/// This form recognizes citation shapes without checking the local filesystem. Use [`record_in`]
/// to resolve repository references.
pub fn record(doc: &Value, path: Option<&str>) -> Value {
    record_in(doc, path, None)
}

/// Builds assessment record data and checks local citations against `root` when supplied.
///
/// With a checkout root, the `cited` field lists resolved file paths for review grounding. With
/// `None`, citation checks use the same shape rules as [`record`].
pub fn record_in(doc: &Value, path: Option<&str>, root: Option<&std::path::Path>) -> Value {
    let by = |v: &str| -> Vec<String> {
        doc["verdicts"]
            .as_array()
            .into_iter()
            .flatten()
            .filter(|c| c["verdict"] == v)
            .filter_map(|c| c["criterion"].as_str().map(str::to_owned))
            .collect()
    };
    let (failing, unsure) = (by("fail"), by("unsure"));
    let verdicts = doc["verdicts"].as_array().cloned().unwrap_or_default();
    let names = |keep: &dyn Fn(&Value) -> bool| -> Vec<String> {
        verdicts
            .iter()
            .filter(|v| keep(v))
            .filter_map(|v| v["criterion"].as_str().map(str::to_owned))
            .collect()
    };
    let unsuggested = names(&|v| {
        v["verdict"] != "pass" && v["suggestion"].as_str().is_none_or(|s| s.trim().is_empty())
    });
    let uncited = names(&|v| !cites(v["reason"].as_str().unwrap_or_default(), root));
    let verdict = if !failing.is_empty() {
        "fail"
    } else if !unsure.is_empty() {
        "unsure"
    } else {
        "pass"
    };
    let mut data = json!({
        "entity": doc["entity"], "rubric": doc["rubric"], "subjectHash": doc["subjectHash"],
        "assessor": doc["assessor"], "at": doc["at"], "verdicts": doc["verdicts"],
        "summary": doc.get("summary").cloned().unwrap_or(Value::Null),
        "verdict": verdict, "failing": failing, "unsure": unsure,
        "unsuggested": unsuggested, "uncited": uncited, "hash": hash(doc),
    });
    if let Some(name) = doc["assessor"]["name"].as_str() {
        let name = name.trim();
        let placeholder = matches!(
            name.to_ascii_lowercase().as_str(),
            "tbd" | "unknown" | "anonymous" | "n/a" | "placeholder"
        );
        data["assessor"]["name"] =
            json!(if !placeholder && name.chars().any(char::is_alphanumeric) {
                name
            } else {
                ""
            });
    }
    if let Some(p) = path {
        data["path"] = json!(p);
    }
    if let Some(hash) = doc["rubricHash"].as_str() {
        data["rubricHash"] = json!(hash);
    }
    if let Some(root) = root {
        let mut cited: Vec<String> = verdicts
            .iter()
            .flat_map(|v| cited_paths(v["reason"].as_str().unwrap_or_default(), root))
            .collect();
        cited.sort();
        cited.dedup();
        data["cited"] = json!(cited);
    }
    data
}

/// Adds review grounding and independent review verdicts to assessment records in place.
///
/// Pass pairs of external ID and [`record_in`] data from the same checkout. A review's `groundedIn`
/// lists citations it shares with its subject or the path of the subject itself. An assessment's
/// `independentReview` holds the verdict of a grounded review with a matching subject hash and a
/// different named assessor. An empty string means no qualifying review was found.
pub fn link_reviews(records: &mut [(String, Value)]) {
    let name = |d: &Value| {
        d["assessor"]["name"]
            .as_str()
            .unwrap_or_default()
            .trim()
            .to_lowercase()
    };
    let strings = |v: &Value| -> Vec<String> {
        v.as_array()
            .into_iter()
            .flatten()
            .filter_map(|x| x.as_str().map(str::to_owned))
            .collect()
    };
    // What each assessment cites, plus its own file: a review grounded in its subject cites one.
    let sources: std::collections::HashMap<String, Vec<String>> = records
        .iter()
        .map(|(id, d)| {
            let mut s = strings(&d["cited"]);
            s.extend(d["path"].as_str().map(str::to_owned));
            (id.clone(), s)
        })
        .collect();
    for (_, d) in records.iter_mut() {
        let subject = d["entity"].as_str().unwrap_or_default();
        if let Some(own) = sources.get(subject) {
            let grounded: Vec<String> = strings(&d["cited"])
                .into_iter()
                .filter(|c| own.contains(c))
                .collect();
            d["groundedIn"] = json!(grounded);
        }
    }
    type Review = (String, String, String, String, bool);
    let reviews: Vec<Review> = records
        .iter()
        .filter(|(_, d)| {
            d["rubric"] == "assessment-quality"
                && d["rubricCurrent"] == true
                && d["uncited"].as_array().is_some_and(Vec::is_empty)
                && d["verdicts"]
                    .as_array()
                    .is_some_and(|rows| !rows.is_empty())
        })
        .map(|(_, d)| {
            (
                d["entity"].as_str().unwrap_or_default().to_owned(),
                d["subjectHash"].as_str().unwrap_or_default().to_owned(),
                name(d),
                d["verdict"].as_str().unwrap_or_default().to_owned(),
                d["groundedIn"].as_array().is_some_and(|g| !g.is_empty()),
            )
        })
        .collect();
    for (id, d) in records.iter_mut() {
        let (hash, own) = (d["hash"].as_str().unwrap_or_default().to_owned(), name(d));
        let verdict = reviews
            .iter()
            .find(|(e, h, by, _, grounded)| {
                e == id
                    && *h == hash
                    && !own.is_empty()
                    && !by.is_empty()
                    && *by != own
                    && *grounded
            })
            .map(|r| r.3.clone())
            .unwrap_or_default();
        d["independentReview"] = json!(verdict);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verdict_rolls_up() {
        let doc = json!({"entity": "rule:p/c/r", "rubric": "rule-quality", "verdicts": [
            {"criterion": "a", "verdict": "pass"}, {"criterion": "b", "verdict": "unsure"}]});
        let r = record(&doc, Some("assessments/x.yaml"));
        assert_eq!(r["verdict"], "unsure");
        assert_eq!(r["unsure"], json!(["b"]));
        assert_eq!(external_id(&doc), "assessment:rule-quality/rule:p/c/r");
        let failed = json!({"verdicts": [{"criterion": "b", "verdict": "unsure"},
                                         {"criterion": "a", "verdict": "fail"}]});
        assert_eq!(record(&failed, None)["verdict"], "fail");
        assert_eq!(record(&failed, None)["unsuggested"], json!(["b", "a"]));
    }

    #[test]
    fn citations_are_checkable_references() {
        for yes in [
            "see docs/plan.md",
            "per D71",
            "crates/seikika-api/tests",
            "lazy.rs",
            "Q10",
        ] {
            assert!(cites(yes, None), "{yes}");
        }
        for no in [
            "it seems fine",
            "good and/or bad",
            "in 2026",
            "20% failed",
            "yes/no",
        ] {
            assert!(!cites(no, None), "{no}");
        }
    }

    #[test]
    fn with_the_repository_a_citation_must_resolve() {
        let root = std::env::temp_dir().join(format!("seikika-cites-{}", std::process::id()));
        std::fs::create_dir_all(root.join("docs/decisions")).unwrap();
        std::fs::write(root.join("docs/plan.md"), "").unwrap();
        std::fs::write(
            root.join("docs/decisions/D01M7VZ2DPS0000000000000000.md"),
            "---\nid: D01M7VZ2DPS0000000000000000\nlegacy: D71\n---\n",
        )
        .unwrap();
        std::fs::write(root.join("docs/questions.md"), "## Q10 a question").unwrap();
        std::fs::write(root.join("README.md"), "").unwrap();
        std::fs::write(root.join("docs/README.md"), "").unwrap();
        let r = Some(root.as_path());
        for yes in [
            "docs/plan.md:12",
            "see plan.md",
            "per D71",
            "open in Q10",
            "https://x.dev",
        ] {
            assert!(cites(yes, r), "{yes}");
        }
        for no in [
            "docs/gone.md",
            "per D7",
            "Q1",
            "good and/or bad",
            "gone.rs",
            "see README.md",
            "a bare https:// prefix",
        ] {
            assert!(!cites(no, r), "{no}");
        }
        assert!(cites("docs/README.md", r), "a path is unambiguous");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn only_a_current_review_by_someone_else_counts() {
        let a = json!({"entity": "crate:x", "rubric": "crate-design", "subjectHash": "c1",
            "assessor": {"kind": "agent", "name": "ann"}, "verdicts": []});
        let rec = |doc: &Value| (external_id(doc), record_in(doc, None, None));
        let target = rec(&a);
        let hash = target.1["hash"].as_str().unwrap().to_owned();
        let review = |by: &str, h: &str, verdict: &str| {
            json!({"entity": "assessment:crate-design/crate:x", "rubric": "assessment-quality",
                "subjectHash": h, "assessor": {"kind": "agent", "name": by},
                "verdicts": [{"criterion": "c", "verdict": verdict, "reason": "See src/lib.rs."}]})
        };
        let mut target = target;
        target.1["cited"] = json!(["src/lib.rs"]);
        let cases = [
            ("bob", hash.as_str(), json!(["src/lib.rs"]), "fail"),
            ("bob", hash.as_str(), json!(["README.md"]), ""),
            ("ann", hash.as_str(), json!(["src/lib.rs"]), ""),
            (" ann ", hash.as_str(), json!(["src/lib.rs"]), ""),
            ("  ", hash.as_str(), json!(["src/lib.rs"]), ""),
            ("bob", "old", json!(["src/lib.rs"]), ""),
        ];
        for (by, h, cited, want) in cases {
            let mut r = rec(&review(by, h, "fail"));
            r.1["cited"] = cited.clone();
            r.1["rubricCurrent"] = json!(true);
            let mut rs = vec![target.clone(), r];
            link_reviews(&mut rs);
            assert_eq!(rs[0].1["independentReview"], want, "{by} {h} {cited}");
            assert_eq!(rs[1].1["independentReview"], "");
        }
    }
}
