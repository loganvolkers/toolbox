//! GitHub Actions workflows and action metadata, read statically (docs/design/github-actions-standards.md
//! §4): triggers, permissions, timeouts, concurrency, and every `uses:` with how it is pinned.
//! Security findings come from zizmor and actionlint (the `runner`); these are the structural facts
//! scorecards need besides them, and the inventory behind "which workflows use action X?".

use serde_json::{Map, Value, json};
use std::collections::{BTreeMap, BTreeSet};

/// Triggers that run with the base repository's secrets and a write token on behalf of code or
/// input from outside the repository (GitHub "secure use"; OpenSSF Scorecard Dangerous-Workflow).
const RISKY_TRIGGERS: &[&str] = &["pull_request_target", "workflow_run", "issue_comment"];

/// How a `uses:` reference is pinned: `sha` (a full 40-hex commit), `tag`, `branch`, `local`
/// (`./…`), `docker-digest` or `docker-tag`.
pub fn pin(reference: &str) -> &'static str {
    if reference.starts_with("./") {
        return "local";
    }
    if let Some(image) = reference.strip_prefix("docker://") {
        return if image.contains("@sha256:") {
            "docker-digest"
        } else {
            "docker-tag"
        };
    }
    let version = reference.rsplit_once('@').map_or("", |(_, v)| v);
    if version.len() == 40 && version.chars().all(|c| c.is_ascii_hexdigit()) {
        "sha"
    } else if version.starts_with('v') && version[1..].starts_with(|c: char| c.is_ascii_digit())
        || version.starts_with(|c: char| c.is_ascii_digit())
    {
        "tag"
    } else {
        "branch"
    }
}

/// `owner/repo` of an external action or reusable workflow (`owner/repo/path@ref` → `owner/repo`).
fn action_name(reference: &str) -> Option<String> {
    if reference.starts_with("./") || reference.starts_with("docker://") {
        return None;
    }
    let path = reference.split('@').next()?;
    let mut parts = path.split('/');
    Some(format!("{}/{}", parts.next()?, parts.next()?))
}

fn triggers(on: &Value) -> Vec<String> {
    match on {
        Value::String(s) => vec![s.clone()],
        Value::Array(a) => a
            .iter()
            .filter_map(|v| v.as_str().map(str::to_owned))
            .collect(),
        Value::Object(m) => m.keys().cloned().collect(),
        _ => vec![],
    }
}

/// `write-all`, or the scopes granted `write` in a permissions block.
fn write_scopes(p: &Value) -> Vec<String> {
    match p {
        Value::String(s) if s == "write-all" => vec!["*".into()],
        Value::Object(m) => m
            .iter()
            .filter(|(_, v)| v.as_str() == Some("write"))
            .map(|(k, _)| k.clone())
            .collect(),
        _ => vec![],
    }
}

/// Whether a `concurrency:` value cancels a superseded run of the same branch or pull request: its
/// group must vary by ref (otherwise every branch shares one queue) and `cancel-in-progress` must
/// not be false, which is GitHub's default (so `concurrency: ci` cancels nothing).
fn cancels_superseded(c: &Value) -> bool {
    const BY_REF: [&str; 5] = [
        "github.ref",
        "github.head_ref",
        "pull_request.number",
        "event.number",
        "github.event.pull_request",
    ];
    let group = c["group"].as_str().unwrap_or_default();
    let cancels = match &c["cancel-in-progress"] {
        Value::Bool(b) => *b,
        Value::String(s) => !s.trim().is_empty() && s.trim() != "false",
        _ => false,
    };
    cancels && BY_REF.iter().any(|r| group.contains(r))
}

/// Truthy `continue-on-error`: `true`, or an expression (which may be true at run time).
fn continues_on_error(v: &Value) -> bool {
    match v {
        Value::Bool(b) => *b,
        Value::String(s) => s.trim() != "false" && !s.trim().is_empty(),
        _ => false,
    }
}

/// Where a failure is hidden: every job with `continue-on-error`, and every step with it whose
/// outcome no later step of the job reads (`steps.<id>.outcome` or `.conclusion`).
fn hidden_failures(id: &str, job: &Value) -> Vec<String> {
    let mut out = Vec::new();
    if continues_on_error(&job["continue-on-error"]) {
        out.push(format!("jobs.{id}"));
    }
    let steps = job["steps"].as_array().cloned().unwrap_or_default();
    for (i, step) in steps.iter().enumerate() {
        if !continues_on_error(&step["continue-on-error"]) {
            continue;
        }
        let checked = step["id"].as_str().is_some_and(|sid| {
            let later = Value::Array(steps[i + 1..].to_vec()).to_string();
            later.contains(&format!("steps.{sid}.outcome"))
                || later.contains(&format!("steps.{sid}.conclusion"))
        });
        if !checked {
            out.push(format!("jobs.{id}.steps[{i}]"));
        }
    }
    out
}

/// One `uses:` occurrence: `{ref, action, pin, at}`.
fn uses(reference: &str, at: String) -> Value {
    json!({"ref": reference, "action": action_name(reference), "pin": pin(reference), "at": at})
}

/// The `workflow` record for a workflow file or action metadata file, from its YAML text.
pub fn record(path: &str, text: &str) -> Value {
    let doc: Value = match serde_norway::from_str(text) {
        Ok(v) => v,
        Err(e) => return json!({"path": path, "error": e.to_string()}),
    };
    let is_action = path.ends_with("action.yml") || path.ends_with("action.yaml");
    let mut refs = Vec::new();
    let mut jobs = Vec::new();
    if is_action {
        for (i, step) in doc["runs"]["steps"]
            .as_array()
            .into_iter()
            .flatten()
            .enumerate()
        {
            if let Some(r) = step["uses"].as_str() {
                refs.push(uses(r, format!("runs.steps[{i}]")));
            }
        }
    } else {
        for (id, job) in doc["jobs"].as_object().into_iter().flatten() {
            if let Some(r) = job["uses"].as_str() {
                refs.push(uses(r, format!("jobs.{id}")));
            }
            for (i, step) in job["steps"].as_array().into_iter().flatten().enumerate() {
                if let Some(r) = step["uses"].as_str() {
                    refs.push(uses(r, format!("jobs.{id}.steps[{i}]")));
                }
            }
            jobs.push(json!({
                "id": id,
                "timeout": job.get("timeout-minutes").is_some() || job.get("uses").is_some(),
                "permissions": job.get("permissions").is_some(),
                "writeScopes": write_scopes(&job["permissions"]),
                "concurrency": job.get("concurrency").is_some(),
                "cancelsSuperseded": cancels_superseded(&job["concurrency"]),
                "hiddenFailures": hidden_failures(id, job),
            }));
        }
    }
    // YAML 1.1 readers turn the `on` key into `true`; accept both.
    let on = doc
        .get("on")
        .or_else(|| doc.get("true"))
        .cloned()
        .unwrap_or(Value::Null);
    let trig = triggers(&on);
    let names = |pred: &dyn Fn(&Value) -> bool| -> Vec<Value> {
        jobs.iter()
            .filter(|j| pred(j))
            .map(|j| j["id"].clone())
            .collect()
    };
    let has_top_permissions = doc.get("permissions").is_some();
    let actions: BTreeSet<String> = refs
        .iter()
        .filter_map(|r| r["action"].as_str().map(str::to_owned))
        .collect();
    json!({
        "path": path,
        "type": if is_action { "action" } else { "workflow" },
        "name": doc["name"].as_str().unwrap_or(path),
        "triggers": trig,
        "riskyTriggers": trig.iter().filter(|t| RISKY_TRIGGERS.contains(&t.as_str())).collect::<Vec<_>>(),
        "topPermissions": has_top_permissions,
        "topWriteScopes": write_scopes(&doc["permissions"]),
        "jobs": jobs.len(),
        "jobsWithoutTimeout": names(&|j| j["timeout"] == false),
        "jobsWithoutPermissions": if has_top_permissions { vec![] } else { names(&|j| j["permissions"] == false) },
        "jobsWithWrite": names(&|j| j["writeScopes"].as_array().is_some_and(|a| !a.is_empty())),
        "concurrency": doc.get("concurrency").is_some() || (!jobs.is_empty() && jobs.iter().all(|j| j["concurrency"] == true)),
        "cancelsSuperseded": cancels_superseded(&doc["concurrency"]) || (!jobs.is_empty() && jobs.iter().all(|j| j["cancelsSuperseded"] == true)),
        "hiddenFailures": jobs.iter().flat_map(|j| j["hiddenFailures"].as_array().cloned().unwrap_or_default()).collect::<Vec<_>>(),
        "uses": refs,
        "unpinned": refs.iter().filter(|r| !matches!(r["pin"].as_str(), Some("sha" | "local" | "docker-digest"))).filter_map(|r| r["ref"].as_str()).collect::<BTreeSet<_>>(),
        "actions": actions,
    })
}

/// For one external action: the refs used, the files using them, and how they're pinned.
type Usage = (BTreeSet<String>, BTreeSet<String>, BTreeSet<String>);

/// `action-ref` records: one per external action, with every ref used and the files using it.
pub fn action_refs(workflows: &[Value]) -> Vec<(String, Value)> {
    let mut by: BTreeMap<String, Usage> = BTreeMap::new();
    for w in workflows {
        for u in w["uses"].as_array().into_iter().flatten() {
            let Some(name) = u["action"].as_str() else {
                continue;
            };
            let e = by.entry(name.to_owned()).or_default();
            e.0.insert(u["ref"].as_str().unwrap_or("").to_owned());
            e.1.insert(w["path"].as_str().unwrap_or("").to_owned());
            e.2.insert(u["pin"].as_str().unwrap_or("").to_owned());
        }
    }
    by.into_iter()
        .map(|(name, (refs, used_by, pins))| {
            let mut data = Map::new();
            data.insert("name".into(), json!(name));
            data.insert("owner".into(), json!(name.split('/').next()));
            data.insert("refs".into(), json!(refs));
            data.insert("usedBy".into(), json!(used_by));
            data.insert("pins".into(), json!(pins));
            (name, Value::Object(data))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_static_or_non_cancelling_group_does_not_cancel_superseded_runs() {
        let rec = |c: &str| {
            super::record(
                ".github/workflows/w.yml",
                &format!("on: {{pull_request: {{}}}}\n{c}\njobs:\n  a:\n    runs-on: x\n    steps: [{{run: echo}}]\n"),
            )["cancelsSuperseded"]
                .clone()
        };
        assert_eq!(rec("concurrency: ci"), false);
        assert_eq!(
            rec("concurrency: {group: ci, cancel-in-progress: true}"),
            false
        );
        assert_eq!(
            rec("concurrency: {group: 'w-${{ github.ref }}', cancel-in-progress: false}"),
            false
        );
        assert_eq!(
            rec("concurrency: {group: 'w-${{ github.ref }}', cancel-in-progress: true}"),
            true
        );
        assert_eq!(
            rec(
                "concurrency:\n  group: w-${{ github.ref }}\n  cancel-in-progress: ${{ github.event_name == 'pull_request' }}"
            ),
            true
        );
    }

    #[test]
    fn continue_on_error_hides_a_failure_unless_a_later_step_reads_its_outcome() {
        let yml = "on: push\njobs:\n  a:\n    runs-on: x\n    steps:\n      - {id: t, run: false, continue-on-error: true}\n      - {run: 'test ${{ steps.t.outcome }} = failure'}\n      - {run: flaky, continue-on-error: true}\n  b:\n    runs-on: x\n    continue-on-error: true\n    steps: [{run: echo}]\n";
        let r = super::record(".github/workflows/w.yml", yml);
        assert_eq!(
            r["hiddenFailures"],
            serde_json::json!(["jobs.a.steps[2]", "jobs.b"])
        );
    }

    use super::*;

    #[test]
    fn classifies_pins() {
        assert_eq!(pin("actions/checkout@v4"), "tag");
        assert_eq!(
            pin("actions/checkout@11bd71901bbe5b1630ceea73d27597364c9af683"),
            "sha"
        );
        assert_eq!(pin("org/repo/.github/workflows/x.yml@main"), "branch");
        assert_eq!(pin("./actions/report"), "local");
        assert_eq!(pin("docker://alpine:3"), "docker-tag");
        assert_eq!(pin("docker://alpine@sha256:abc"), "docker-digest");
    }

    #[test]
    fn reads_a_workflow() {
        let w = record(
            ".github/workflows/ci.yml",
            "name: ci\non:\n  pull_request_target:\n  push:\njobs:\n  a:\n    runs-on: ubuntu-latest\n    timeout-minutes: 5\n    permissions: {contents: write}\n    steps:\n      - uses: actions/checkout@v4\n      - uses: ./actions/report\n  b:\n    uses: org/shared/.github/workflows/w.yml@0123456789012345678901234567890123456789\n",
        );
        assert_eq!(w["riskyTriggers"], json!(["pull_request_target"]));
        assert_eq!(w["topPermissions"], false);
        assert_eq!(w["jobsWithoutPermissions"], json!(["b"]));
        assert_eq!(w["jobsWithWrite"], json!(["a"]));
        assert_eq!(
            w["jobsWithoutTimeout"],
            json!([]),
            "reusable-workflow jobs can't set a timeout"
        );
        assert_eq!(w["unpinned"], json!(["actions/checkout@v4"]));
        assert_eq!(w["actions"], json!(["actions/checkout", "org/shared"]));
        let refs = action_refs(&[w]);
        assert_eq!(refs[0].1["usedBy"], json!([".github/workflows/ci.yml"]));
    }
}
