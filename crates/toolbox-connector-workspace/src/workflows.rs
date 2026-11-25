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

/// What a workflow's `on:` says about pushes and schedules: the branch filters of `push` (`["*"]`
/// when it has none, empty when it runs on no branch push, as for tags only), its
/// `branches-ignore`, and whether it is scheduled. Whether it runs on the default branch, where run
/// history is read, is decided against the configured branch (the package's `branch` parameter).
fn push_triggers(on: &Value) -> (Vec<String>, Vec<String>, bool) {
    let all = || vec!["*".to_string()];
    match on {
        Value::String(s) => (
            if s == "push" { all() } else { vec![] },
            vec![],
            s == "schedule",
        ),
        Value::Array(a) => {
            let has = |t: &str| a.iter().any(|v| v.as_str() == Some(t));
            (
                if has("push") { all() } else { vec![] },
                vec![],
                has("schedule"),
            )
        }
        Value::Object(m) => {
            let scheduled = m.contains_key("schedule");
            let Some(push) = m.get("push") else {
                return (vec![], vec![], scheduled);
            };
            let names = |k: &str| -> Option<Vec<String>> {
                push[k].as_array().map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_str().map(str::to_owned))
                        .collect()
                })
            };
            let ignores = names("branches-ignore").unwrap_or_default();
            let branches = match (names("branches"), names("tags")) {
                (Some(b), _) => b,
                // `tags:` alone limits a push to tags; neither key means every branch push.
                (None, Some(_)) if ignores.is_empty() => vec![],
                _ => all(),
            };
            (branches, ignores, scheduled)
        }
        _ => (vec![], vec![], false),
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
    let cancels = continues_on_error(&c["cancel-in-progress"]);
    cancels && BY_REF.iter().any(|r| group.contains(r))
}

/// A truthy flag: `true`, or an expression that isn't literally false (it may be true at run
/// time). `${{ false }}` is false.
fn continues_on_error(v: &Value) -> bool {
    match v {
        Value::Bool(b) => *b,
        Value::String(s) => {
            let e = s
                .trim()
                .trim_start_matches("${{")
                .trim_end_matches("}}")
                .trim();
            !e.is_empty() && e != "false"
        }
        _ => false,
    }
}

/// A `run:` script that swallows its own exit status: `|| true`, `|| :`, `|| exit 0`, an
/// `|| echo`/`printf` fallback, `|| { … }` or `|| ret=$?` (with or without spaces), a one-line
/// `if ! cmd; then echo …; fi` (a multi-line `if !` that installs or exits is a real branch),
/// `set +e`, or a pipe into `tee` without `pipefail` (the pipeline's status is `tee`'s).
fn masks_exit(run: &str) -> bool {
    static R: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    static TEE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    let masked = R.get_or_init(|| {
        regex::Regex::new(
            r"\|\|\s*(?:true\b|:(?:\s|$)|exit\s+0\b|echo\b|printf\b|\{|\w+=\$\?)|\bset\s+\+e\b|\bif\s+!\s[^;\n]*;\s*then\s+(?:echo|true|:)[^;\n]*;\s*fi\b",
        )
        .expect("static regex")
    });
    let tee = TEE.get_or_init(|| regex::Regex::new(r"[^|]\|\s*tee\b").expect("static regex"));
    masked.is_match(run) || (tee.is_match(run) && !run.contains("pipefail"))
}

/// A step that retries inside the run, so a first-try failure never reaches the run's result: a
/// retry action, or a test runner's retry flag (`--retries`, `--reruns`, `--retry-failed`,
/// `retryTimes`). `curl --retry` retries a download, not a test, and isn't counted.
fn retries(step: &Value) -> bool {
    static R: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    let flag = R.get_or_init(|| {
        regex::Regex::new(r"--retries\b|--reruns\b|--retry-failed\b|\bretryTimes\b")
            .expect("static regex")
    });
    let action = step["uses"].as_str().is_some_and(|u| {
        let u = u.to_ascii_lowercase();
        [
            "nick-fields/retry",
            "wandalen/wretry",
            "step-security/retry",
        ]
        .iter()
        .any(|a| u.starts_with(a))
    });
    action || step["run"].as_str().is_some_and(|r| flag.is_match(r))
}

/// A step `shell:` without `-e`: GitHub's default bash is `bash -e {0}`, so `bash {0}` or `sh {0}`
/// lets a failing command in the middle of the script pass.
fn shell_ignores_errors(shell: &Value) -> bool {
    shell.as_str().is_some_and(|s| {
        let words: Vec<&str> = s.split_whitespace().collect();
        words.contains(&"{0}")
            && matches!(words.first(), Some(&"bash") | Some(&"sh"))
            && !words.iter().any(|w| w.starts_with('-') && w.contains('e'))
    })
}

/// Where an `if:` keeps a job or step from running, if it does: `never runs` (literally false),
/// `runs only on pull requests` (an event or ref test for one), `skipped on push`, or
/// `skipped on <branch>` (a test that the ref is not that branch). The scorecard counts the last
/// only when it is the default branch.
fn skip_of(cond: &Value) -> Option<String> {
    let c = match cond {
        Value::Bool(false) => return Some("never runs".into()),
        Value::String(s) => s.replace(' ', ""),
        _ => return None,
    };
    let e = c.trim_start_matches("${{").trim_end_matches("}}");
    if e == "false" {
        return Some("never runs".into());
    }
    if e == "github.event_name=='pull_request'"
        || e == "github.event_name=='pull_request_target'"
        || e.contains("startsWith(github.ref,'refs/pull/')")
    {
        return Some("runs only on pull requests".into());
    }
    if e.contains("github.event_name!='push'") {
        return Some("skipped on push".into());
    }
    static R: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    let branch = R.get_or_init(|| {
        regex::Regex::new(r"github\.ref!='refs/heads/([^']+)'|github\.ref_name!='([^']+)'")
            .expect("static regex")
    });
    branch
        .captures(e)
        .and_then(|c| c.get(1).or_else(|| c.get(2)))
        .map(|b| format!("skipped on {}", b.as_str()))
}

/// Where a failure is hidden from the fail rate: a job with `continue-on-error` or skipped on the
/// default branch; a step with `continue-on-error` whose outcome no later step of the job reads
/// (`steps.<id>.outcome` or `.conclusion`), a step skipped by its `if:` (see [`skip_of`]), a step
/// that retries inside the run, and a step whose
/// script swallows its exit status.
fn hidden_failures(id: &str, job: &Value) -> Vec<String> {
    let mut out = Vec::new();
    if continues_on_error(&job["continue-on-error"]) {
        out.push(format!("jobs.{id}"));
    }
    if let Some(skip) = skip_of(&job["if"]) {
        out.push(format!("jobs.{id} ({skip})"));
    }
    let steps = job["steps"].as_array().cloned().unwrap_or_default();
    for (i, step) in steps.iter().enumerate() {
        if let Some(skip) = skip_of(&step["if"]) {
            out.push(format!("jobs.{id}.steps[{i}] ({skip})"));
        }
        if retries(step) {
            out.push(format!("jobs.{id}.steps[{i}] (retries inside the run)"));
        }
        let shell = if step.get("shell").is_some() {
            &step["shell"]
        } else {
            &job["defaults"]["run"]["shell"]
        };
        if step["run"].as_str().is_some_and(masks_exit)
            || (step.get("run").is_some() && shell_ignores_errors(shell))
        {
            out.push(format!(
                "jobs.{id}.steps[{i}] (the script swallows its exit status)"
            ));
        }
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

/// What only the text as written says: each ref's version comment (set on `refs`), and every
/// unpinned `uses:` as `<ref> at line <n>`.
fn as_written(text: &str, refs: &mut [Value]) -> Vec<String> {
    let written = uses_lines(text);
    for r in refs.iter_mut() {
        let comment = written
            .iter()
            .find(|(_, w, c)| Some(w.as_str()) == r["ref"].as_str() && c.is_some())
            .and_then(|(_, _, c)| c.clone());
        if let Some(c) = comment {
            r["comment"] = json!(c);
        }
    }
    written
        .iter()
        .filter(|(_, w, _)| !matches!(pin(w), "sha" | "local" | "docker-digest"))
        .map(|(n, w, _)| format!("{w} at line {n}"))
        .collect()
}

/// One `uses:` occurrence: `{ref, action, pin, at}`.
fn uses(reference: &str, at: String) -> Value {
    json!({"ref": reference, "action": action_name(reference), "pin": pin(reference), "at": at})
}

/// Every `uses:` line of the text as written: its 1-based line, the reference, and the comment
/// after it (`v6.0.2` in `uses: actions/checkout@<sha> # v6.0.2`). The YAML reader drops both.
fn uses_lines(text: &str) -> Vec<(usize, String, Option<String>)> {
    let mut out = Vec::new();
    for (i, line) in text.lines().enumerate() {
        let item = line.trim_start().trim_start_matches("- ").trim_start();
        let Some(rest) = item.strip_prefix("uses:") else {
            continue;
        };
        let (value, comment) = match rest.split_once(" #") {
            Some((v, c)) => (v, Some(c.trim().to_owned()).filter(|c| !c.is_empty())),
            None => (rest, None),
        };
        let value = value.trim().trim_matches(|c| c == '"' || c == '\'');
        if !value.is_empty() {
            out.push((i + 1, value.to_owned(), comment));
        }
    }
    out
}

/// A workflow's `on:`. YAML 1.1 readers turn the key into `true`; accept both.
fn on(doc: &Value) -> &Value {
    doc.get("on")
        .or_else(|| doc.get("true"))
        .unwrap_or(&Value::Null)
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
    let trig = triggers(on(&doc));
    let push = push_triggers(on(&doc));
    let names = |pred: &dyn Fn(&Value) -> bool| -> Vec<Value> {
        jobs.iter()
            .filter(|j| pred(j))
            .map(|j| j["id"].clone())
            .collect()
    };
    let unpinned_at = as_written(text, &mut refs);
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
        "pushBranches": if is_action { vec![] } else { push.0 },
        "pushIgnores": if is_action { vec![] } else { push.1 },
        "scheduled": !is_action && push.2,
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
        "unpinnedAt": unpinned_at,
        "actions": actions,
    })
}

/// For one external action: the refs used, the files using them, how they're pinned, and its
/// SHA pins as written.
#[derive(Default)]
struct Usage {
    refs: BTreeSet<String>,
    used_by: BTreeSet<String>,
    pins: BTreeSet<String>,
    sha_pins: BTreeSet<String>,
    /// Each SHA pin as (path, sha, version comment).
    pinned: BTreeSet<(String, String, Option<String>)>,
}

/// The pin to suggest for an unpinned `ref`: one of the action's SHAs, written on the ref's own
/// path (`actions/cache@v5` gets `actions/cache@<sha>`, never `actions/cache/restore@<sha>`, which
/// is another action in the same repository). Prefers a pin whose version comment is within the
/// ref's tag (`v5` → `v5.0.5`), then one on the same path, then one with a comment.
fn pin_for(reference: &str, pinned: &BTreeSet<(String, String, Option<String>)>) -> Option<String> {
    let (path, tag) = reference.rsplit_once('@')?;
    let within = |c: &str| {
        c == tag
            || c.strip_prefix(tag)
                .is_some_and(|rest| rest.starts_with('.'))
    };
    let (_, sha, comment) = pinned.iter().max_by_key(|(p, _, c)| {
        (
            c.as_deref().is_some_and(within),
            p == path,
            c.is_some(),
            std::cmp::Reverse((p.clone(), c.clone())),
        )
    })?;
    Some(match comment {
        Some(c) => format!("{path}@{sha} # {c}"),
        None => format!("{path}@{sha}"),
    })
}

/// `action-ref` records: one per external action, with every ref used and the files using it.
/// `shaPins` are its SHA refs as the workflows write them, with the version comment when there is
/// one (`actions/checkout@<sha> # v6.0.2`); the commented ones come first. `pinHints` names each ref
/// that isn't a SHA with the pin to write instead (`pin_for`), so a hint can be copied as it stands.
pub fn action_refs(workflows: &[Value]) -> Vec<(String, Value)> {
    let mut by: BTreeMap<String, Usage> = BTreeMap::new();
    for w in workflows {
        for u in w["uses"].as_array().into_iter().flatten() {
            let Some(name) = u["action"].as_str() else {
                continue;
            };
            let e = by.entry(name.to_owned()).or_default();
            let reference = u["ref"].as_str().unwrap_or("");
            e.refs.insert(reference.to_owned());
            e.used_by
                .insert(w["path"].as_str().unwrap_or("").to_owned());
            e.pins.insert(u["pin"].as_str().unwrap_or("").to_owned());
            if u["pin"] == "sha" {
                let comment = u["comment"].as_str();
                e.sha_pins.insert(match comment {
                    Some(c) => format!("{reference} # {c}"),
                    None => reference.to_owned(),
                });
                if let Some((path, sha)) = reference.rsplit_once('@') {
                    e.pinned
                        .insert((path.to_owned(), sha.to_owned(), comment.map(str::to_owned)));
                }
            }
        }
    }
    by.into_iter()
        .map(|(name, u)| {
            let mut data = Map::new();
            data.insert("name".into(), json!(name));
            data.insert("owner".into(), json!(name.split('/').next()));
            data.insert("refs".into(), json!(u.refs));
            data.insert("usedBy".into(), json!(u.used_by));
            data.insert("pins".into(), json!(u.pins));
            let mut sha_pins: Vec<String> = u.sha_pins.into_iter().collect();
            sha_pins.sort_by_key(|p| !p.contains(" # "));
            data.insert("shaPins".into(), json!(sha_pins));
            let hints: Vec<String> = u
                .refs
                .iter()
                .filter(|r| pin(r) != "sha")
                .map(|r| match pin_for(r, &u.pinned) {
                    Some(p) => format!("{r}, pinned elsewhere as {p}"),
                    None => r.clone(),
                })
                .collect();
            data.insert("pinHints".into(), json!(hints));
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
        assert_eq!(w["unpinnedAt"], json!(["actions/checkout@v4 at line 11"]));
        assert_eq!(w["actions"], json!(["actions/checkout", "org/shared"]));
        let refs = action_refs(&[w]);
        assert_eq!(refs[0].1["usedBy"], json!([".github/workflows/ci.yml"]));
    }

    #[test]
    fn sha_pins_keep_their_version_comment_and_unpinned_refs_their_line() {
        let sha = "de0fac2e4500dabe0009e67214ff5f5447ce83dd";
        let old = "11bd71901bbe5b1630ceea73d27597364c9af683";
        let yml = format!(
            "on: push\njobs:\n  a:\n    runs-on: x\n    steps:\n      - name: Checkout\n        uses: actions/checkout@{sha} # v6.0.2\n  b:\n    runs-on: x\n    steps:\n      - uses: \"actions/checkout@v6\"\n      - uses: actions/checkout@{sha}\n      - uses: actions/checkout@{old}\n"
        );
        let w = record(".github/workflows/w.yml", &yml);
        assert_eq!(w["unpinnedAt"], json!(["actions/checkout@v6 at line 11"]));
        let refs = action_refs(&[w]);
        assert_eq!(
            refs[0].1["shaPins"],
            json!([
                format!("actions/checkout@{sha} # v6.0.2"),
                format!("actions/checkout@{old}")
            ])
        );
        assert_eq!(
            refs[0].1["pinHints"],
            json!([format!(
                "actions/checkout@v6, pinned elsewhere as actions/checkout@{sha} # v6.0.2"
            )])
        );
    }

    #[test]
    fn a_pin_hint_keeps_the_refs_own_path_and_prefers_its_version() {
        let sha = "27d5ce7f107fe9357f9df03efb73ab90386fccae";
        let v4 = "0057852bfaa89a56745cba8c7296529d2fc39830";
        let yml = format!(
            "on: push\njobs:\n  a:\n    runs-on: x\n    steps:\n      - uses: actions/cache/restore@{sha} # v5.0.5\n      - uses: actions/cache/save@{sha} # v5.0.5\n      - uses: actions/cache@{v4} # v4.2.0\n      - uses: actions/cache@v5\n      - uses: actions/cache/save@v4\n      - uses: actions/cache@main\n"
        );
        let refs = action_refs(&[record(".github/workflows/w.yml", &yml)]);
        assert_eq!(
            refs[0].1["pinHints"],
            json!([
                format!(
                    "actions/cache/save@v4, pinned elsewhere as actions/cache/save@{v4} # v4.2.0"
                ),
                format!("actions/cache@main, pinned elsewhere as actions/cache@{v4} # v4.2.0"),
                format!("actions/cache@v5, pinned elsewhere as actions/cache@{sha} # v5.0.5"),
            ])
        );
    }

    #[test]
    fn masked_exits_and_default_branch_skips_hide_failures() {
        let job = json!({"if": "${{ github.ref != 'refs/heads/main' }}", "steps": [
            {"run": "cargo test ||   true"},
            {"run": "set -e\ncargo test"},
            {"if": false, "run": "x"},
            {"if": "github.event_name == 'push'", "run": "y"},
            {"run": "cargo test ||true"},
            {"run": "make || echo flaky"},
            {"shell": "bash {0}", "run": "a\nb"},
            {"shell": "bash -e {0}", "run": "a"},
            {"if": "${{ github.event_name == 'pull_request' }}", "run": "z"},
            {"run": "cargo test | tee log"},
            {"run": "set -o pipefail\ncargo test | tee log"},
            {"run": "if ! cargo test; then echo failed; fi"},
            {"run": "cargo test || { true; }"},
            {"run": "cargo test || ret=$?"},
            {"if": "github.event_name != 'push'", "run": "a"},
            {"if": "startsWith(github.ref, 'refs/pull/')", "run": "a"},
            {"uses": "nick-fields/retry@v3", "with": {"command": "cargo test"}},
            {"run": "cargo nextest run --retries 3"},
            {"run": "curl --retry 3 https://example.com"},
            {"if": "github.ref_name != 'develop'", "run": "a"}
        ]});
        assert_eq!(
            hidden_failures("t", &job),
            vec![
                "jobs.t (skipped on main)",
                "jobs.t.steps[0] (the script swallows its exit status)",
                "jobs.t.steps[2] (never runs)",
                "jobs.t.steps[4] (the script swallows its exit status)",
                "jobs.t.steps[5] (the script swallows its exit status)",
                "jobs.t.steps[6] (the script swallows its exit status)",
                "jobs.t.steps[8] (runs only on pull requests)",
                "jobs.t.steps[9] (the script swallows its exit status)",
                "jobs.t.steps[11] (the script swallows its exit status)",
                "jobs.t.steps[12] (the script swallows its exit status)",
                "jobs.t.steps[13] (the script swallows its exit status)",
                "jobs.t.steps[14] (skipped on push)",
                "jobs.t.steps[15] (runs only on pull requests)",
                "jobs.t.steps[16] (retries inside the run)",
                "jobs.t.steps[17] (retries inside the run)",
                "jobs.t.steps[19] (skipped on develop)",
            ]
        );
        assert!(!continues_on_error(&json!("${{ false }}")));
        assert!(continues_on_error(&json!("${{ matrix.experimental }}")));
    }

    #[test]
    fn push_branches_and_schedules_are_recorded_as_written() {
        let on = |y: &str| push_triggers(&serde_norway::from_str(y).unwrap());
        let v = |x: &[&str]| x.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        assert_eq!(on("push"), (v(&["*"]), v(&[]), false));
        assert_eq!(
            on("{push: {branches: [develop]}, pull_request: {}}"),
            (v(&["develop"]), v(&[]), false)
        );
        assert_eq!(
            on("{schedule: [{cron: '0 0 * * 0'}]}"),
            (v(&[]), v(&[]), true)
        );
        assert_eq!(
            on("{push: {tags: ['v*']}, pull_request: {}}"),
            (v(&[]), v(&[]), false)
        );
        assert_eq!(
            on("{pull_request: {}, workflow_dispatch: {}}"),
            (v(&[]), v(&[]), false)
        );
        assert_eq!(
            on("{push: {branches-ignore: [main]}}"),
            (v(&["*"]), v(&["main"]), false)
        );
    }
}
