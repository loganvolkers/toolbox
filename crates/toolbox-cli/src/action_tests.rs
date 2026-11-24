//! `toolbox test`'s action cases (`runs:` in a test file, docs/design/federated-actions.md §6.5):
//! each case previews, requests, approves and runs one action through the lifecycle `toolbox run`
//! and `toolbox serve` use, in a scratch clone of the repository whose origin is a scratch copy of
//! the base, then checks the run's state and the files on that base. Nothing reaches the real
//! origin, and no run is saved.

use serde::Deserialize;
use serde_json::{Map, Value};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;
use toolbox_derive::Derived;
use toolbox_derive::testing::TestResult;
use toolbox_exec::executor::{Executor, portal_repo_dir};
use toolbox_exec::federation::{self, Offered};
use toolbox_exec::prepare::{Refused, Request, prepare};
use toolbox_exec::run::{self as lifecycle, Observer, Run};
use toolbox_model::Manifest;
use toolbox_model::manifest::Expect;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Suite {
    name: String,
    /// Fields every case starts from. A case's own fields replace them, except `inputs`, which
    /// merge key by key; an input set to `null` there is left out.
    #[serde(default)]
    defaults: Map<String, Value>,
    #[serde(default)]
    runs: Vec<Map<String, Value>>,
}

/// `{portal}` in any string of a case is the name of the portal under test, so one test file can
/// serve every repository that includes the same standard action.
fn with_portal(v: &mut Value, portal: &str) {
    match v {
        Value::String(s) if s.contains("{portal}") => *s = s.replace("{portal}", portal),
        Value::Array(xs) => xs.iter_mut().for_each(|x| with_portal(x, portal)),
        Value::Object(o) => o.values_mut().for_each(|x| with_portal(x, portal)),
        _ => {}
    }
}

impl Suite {
    fn cases(&self, file: &str, portal: &str) -> Result<Vec<Case>, String> {
        self.runs
            .iter()
            .enumerate()
            .map(|(i, run)| {
                let mut c = self.defaults.clone();
                for (k, v) in run {
                    match (k.as_str(), c.get_mut(k), v) {
                        ("inputs", Some(Value::Object(base)), Value::Object(over)) => {
                            for (name, value) in over {
                                if value.is_null() {
                                    base.remove(name);
                                } else {
                                    base.insert(name.clone(), value.clone());
                                }
                            }
                        }
                        _ => {
                            c.insert(k.clone(), v.clone());
                        }
                    }
                }
                let mut c = Value::Object(c);
                with_portal(&mut c, portal);
                serde_json::from_value(c).map_err(|e| {
                    let name = run.get("name").and_then(Value::as_str).unwrap_or("");
                    format!("{file}: runs[{i}] {name}: {e}")
                })
            })
            .collect()
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct Case {
    name: String,
    action: String,
    #[serde(default)]
    entity: Option<String>,
    #[serde(rename = "as", default = "tester")]
    as_user: String,
    #[serde(default)]
    inputs: Map<String, Value>,
    /// People who approve the run, in order.
    #[serde(default)]
    approve: Vec<String>,
    /// The scratch origin refuses pushes with this message (a `pre-receive` hook).
    #[serde(default)]
    reject_push: Option<String>,
    expect: Want,
}

fn tester() -> String {
    "user:test".into()
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Want {
    /// A run state, or `invalid` (the inputs were refused), `blocked` (a precondition or the
    /// preview blocked it) or `refused` (no such action here).
    state: String,
    /// Each file's content on the scratch origin's base afterwards; `null` means absent.
    #[serde(default)]
    files: BTreeMap<String, Option<String>>,
    /// The approvers in `approve` whose approval was refused.
    #[serde(default)]
    refused: Vec<String>,
    /// Text the run's error, the refusal or a refused approval contains.
    #[serde(default)]
    error: Option<String>,
}

/// What a case ended with.
#[derive(Default)]
struct Got {
    state: String,
    refused: Vec<String>,
    errors: Vec<String>,
}

/// Run the action cases in the manifest's test files (paths relative to `root`). The portal is
/// read as `toolbox run` reads it, by its in-image connectors, and only when there are cases.
pub fn run(portal: &Path, root: &Path, tests: &[String]) -> Result<Vec<TestResult>, String> {
    let mut suites = Vec::new();
    for file in tests {
        let text = std::fs::read_to_string(root.join(file)).map_err(|e| format!("{file}: {e}"))?;
        let suite: Suite = serde_norway::from_str(&text).map_err(|e| format!("{file}: {e}"))?;
        if !suite.runs.is_empty() {
            suites.push((file, suite));
        }
    }
    if suites.is_empty() {
        return Ok(vec![]);
    }
    let (root, m, d) = crate::report::live(portal, false, &[])?;
    let (m, d) = (&m, &d);
    let all = federation::offered(m, d);
    let mut out = Vec::new();
    for (file, suite) in suites {
        for case in &suite.cases(file, &m.portal.name)? {
            let root = root.as_path();
            let error = one(root, m, d, &all, case).err();
            out.push(TestResult {
                file: file.clone(),
                suite: suite.name.clone(),
                name: case.name.clone(),
                passed: error.is_none(),
                error,
            });
        }
    }
    Ok(out)
}

fn one(root: &Path, m: &Manifest, d: &Derived, all: &[Offered], c: &Case) -> Result<(), String> {
    let program = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("toolbox"));
    let (off, _) = federation::resolve(all, d, &c.action, c.entity.as_deref())?;
    if off.child.is_some() {
        return Err(format!(
            "{} is a child portal's action: test it in that portal",
            c.action
        ));
    }
    let (remote, base) = match off.executor(&program)? {
        Executor::Change { remote, base, .. } => (remote, base),
        Executor::Command { .. } => ("origin".into(), None),
        other => {
            return Err(format!(
                "toolbox test runs change and command actions, and {} is {}",
                c.action,
                other.kind()
            ));
        }
    };
    let repo = portal_repo_dir(root);
    let s = Scratch::new(&repo, &remote, base.as_deref(), c.reject_push.as_deref())?;
    let got = attempt(m, d, c, &s.work, &program);
    let files: BTreeMap<String, Option<String>> = c
        .expect
        .files
        .keys()
        .map(|f| (f.clone(), s.file(f)))
        .collect();
    drop(s);
    compare(&c.expect, &got, &files)
}

fn attempt(m: &Manifest, d: &Derived, c: &Case, work: &Path, program: &Path) -> Got {
    let req = Request {
        action: &c.action,
        entity: c.entity.as_deref(),
        inputs: c.inputs.clone(),
        actor: &c.as_user,
        cwd: work.to_path_buf(),
        program,
    };
    let p = match prepare(m, d, req) {
        Ok(p) => p,
        Err(Refused::Invalid(e)) => return got("invalid", e),
        Err(e) => return got("refused", vec![e.to_string()]),
    };
    if !p.blockers.is_empty() {
        return got("blocked", p.blockers.clone());
    }
    let mut run = p.new_run(
        lifecycle::new_id("run"),
        &c.as_user,
        "test".into(),
        &m.portal.name,
        &d.snapshot,
    );
    let mut g = Got::default();
    if !p.approvers.is_empty() {
        run.set("awaiting_approval", None, None);
    }
    for who in &c.approve {
        if let Err(e) = run.approve(who, |_| false) {
            g.refused.push(who.clone());
            g.errors.push(e);
        }
    }
    if p.approvers.is_empty() || p.reviewed || run.pending_approvers().is_empty() {
        let mut obs = Fixtures(d);
        let ctx = p.context(&run);
        lifecycle::execute(
            &mut run,
            &p.exec,
            &ctx,
            p.offered.action.expect.as_ref(),
            &m.policies,
            &mut obs,
            Duration::from_millis(200),
        );
    }
    g.state = run.state.clone();
    g.errors.extend(run.error.clone());
    g
}

fn got(state: &str, errors: Vec<String>) -> Got {
    Got {
        state: state.into(),
        errors,
        ..Got::default()
    }
}

fn compare(want: &Want, got: &Got, files: &BTreeMap<String, Option<String>>) -> Result<(), String> {
    let mut wrong = Vec::new();
    if got.state != want.state {
        let why = if got.errors.is_empty() {
            String::new()
        } else {
            format!(" ({})", got.errors.join("; "))
        };
        wrong.push(format!("state is {}, not {}{why}", got.state, want.state));
    }
    let (mut a, mut b) = (got.refused.clone(), want.refused.clone());
    a.sort();
    b.sort();
    if a != b {
        wrong.push(format!("refused approvals {a:?}, not {b:?}"));
    }
    if let Some(e) = &want.error
        && !got.errors.iter().any(|x| x.contains(e.as_str()))
    {
        wrong.push(format!(
            "no error contains {e:?} (errors: {:?})",
            got.errors
        ));
    }
    for (path, content) in &want.files {
        let seen = files.get(path).cloned().flatten();
        if &seen != content {
            wrong.push(match (&seen, content) {
                (None, _) => format!("{path} is not on the base"),
                (Some(_), None) => format!("{path} is on the base, and should not be"),
                (Some(s), Some(_)) => format!("{path} on the base is {s:?}"),
            });
        }
    }
    if wrong.is_empty() {
        Ok(())
    } else {
        Err(wrong.join("; "))
    }
}

/// A test run's postcondition is checked against the portal's fixtures: no connector runs.
struct Fixtures<'a>(&'a Derived);

impl Observer for Fixtures<'_> {
    fn observe(&mut self, run: &Run, _: &Expect) -> Result<(Value, String), String> {
        let e = self
            .0
            .entity_json(&run.entity)
            .ok_or_else(|| format!("{} is not in the fixtures", run.entity))?;
        Ok((e, self.0.snapshot.clone()))
    }
}

/// A bare origin holding only the base as it would be pushed, and a clone of it whose remote has
/// the executor's remote name.
struct Scratch {
    dir: PathBuf,
    origin: PathBuf,
    work: PathBuf,
    branch: String,
}

impl Scratch {
    fn new(
        repo: &Path,
        remote: &str,
        base: Option<&str>,
        reject: Option<&str>,
    ) -> Result<Self, String> {
        let branch = match base {
            Some(b) => b.to_owned(),
            None => git(repo, &["symbolic-ref", "--short", "HEAD"]).unwrap_or("main".into()),
        };
        let dir = std::env::temp_dir().join(lifecycle::new_id("toolbox-test"));
        let s = Scratch {
            origin: dir.join("origin.git"),
            work: dir.join("work"),
            dir,
            branch,
        };
        std::fs::create_dir_all(&s.dir).map_err(|e| e.to_string())?;
        let at = working_tree(repo, remote, &s.branch, &s.dir.join("index"))?;
        let origin = s.origin.to_string_lossy().into_owned();
        git(&s.dir, &["init", "-q", "--bare", &origin])?;
        git(
            repo,
            &[
                "push",
                "-q",
                &origin,
                &format!("{at}:refs/heads/{}", s.branch),
            ],
        )?;
        git(
            &s.origin,
            &["symbolic-ref", "HEAD", &format!("refs/heads/{}", s.branch)],
        )?;
        let work = s.work.to_string_lossy().into_owned();
        git(&s.dir, &["clone", "-q", "-o", remote, &origin, &work])?;
        if let Some(msg) = reject {
            let hook = s.origin.join("hooks/pre-receive");
            let quoted = msg.replace('\'', r"'\''");
            std::fs::write(&hook, format!("#!/bin/sh\necho '{quoted}' >&2\nexit 1\n"))
                .map_err(|e| e.to_string())?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&hook, std::fs::Permissions::from_mode(0o755))
                    .map_err(|e| e.to_string())?;
            }
        }
        Ok(s)
    }

    /// A file's content on the origin's base.
    fn file(&self, path: &str) -> Option<String> {
        git(&self.origin, &["show", &format!("{}:{path}", self.branch)]).ok()
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

/// A commit of the repository's working tree on top of its base, so cases test what would be
/// pushed, including uncommitted edits and a repository with no commit yet. It goes through a
/// scratch index; the repository's own index and refs are untouched.
fn working_tree(repo: &Path, remote: &str, branch: &str, index: &Path) -> Result<String, String> {
    let parent = [
        format!("{remote}/{branch}"),
        branch.to_owned(),
        "HEAD".into(),
    ]
    .into_iter()
    .find_map(|r| {
        git(
            repo,
            &["rev-parse", "-q", "--verify", &format!("{r}^{{commit}}")],
        )
        .ok()
    });
    let env = [("GIT_INDEX_FILE", index.as_os_str())];
    if let Some(p) = &parent {
        git_env(repo, &["read-tree", p], &env)?;
    }
    git_env(repo, &["add", "-A"], &env)?;
    let tree = git_env(repo, &["write-tree"], &env)?;
    let mut args = vec![
        "-c",
        "user.name=toolbox test",
        "-c",
        "user.email=test@toolbox.invalid",
        "commit-tree",
        &tree,
        "-m",
        "toolbox test: the working tree",
    ];
    if let Some(p) = &parent {
        args.extend(["-p", p]);
    }
    git(repo, &args)
}

/// `git` in `dir`, its stdout as given (a file's content keeps its trailing newline).
fn git(dir: &Path, args: &[&str]) -> Result<String, String> {
    git_env(dir, args, &[])
}

fn git_env(dir: &Path, args: &[&str], env: &[(&str, &std::ffi::OsStr)]) -> Result<String, String> {
    let out = Command::new("git")
        .args(args)
        .envs(env.iter().copied())
        .current_dir(dir)
        .output()
        .map_err(|e| format!("git: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "git {}: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    let s = String::from_utf8_lossy(&out.stdout).into_owned();
    Ok(if args.first() == Some(&"show") {
        s
    } else {
        s.trim().to_owned()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn want(yaml: &str) -> Want {
        serde_norway::from_str(yaml).unwrap()
    }

    #[test]
    fn a_case_passes_only_when_state_refusals_error_and_files_all_match() {
        let got = Got {
            state: "failed".into(),
            refused: vec!["user:carol".into(), "user:ann".into()],
            errors: vec!["remote: main is protected".into()],
        };
        let files = BTreeMap::from([("A.md".to_owned(), None)]);
        let w = "{state: failed, refused: [user:ann, user:carol], error: protected, files: {A.md: null}}";
        assert!(compare(&want(w), &got, &files).is_ok());
        let e = compare(&want("{state: succeeded}"), &got, &files).unwrap_err();
        assert!(
            e.contains("state is failed, not succeeded") && e.contains("refused approvals"),
            "{e}"
        );
        let e = compare(
            &want("{state: failed, refused: [user:ann, user:carol], files: {A.md: x}}"),
            &got,
            &files,
        )
        .unwrap_err();
        assert_eq!(e, "A.md is not on the base");
    }

    #[test]
    fn the_scratch_base_is_the_working_tree_even_before_the_first_commit() {
        let dir = std::env::temp_dir().join(lifecycle::new_id("toolbox-test-repo"));
        std::fs::create_dir_all(&dir).unwrap();
        let repo = dir.as_path();
        git(repo, &["init", "-q", "-b", "main"]).unwrap();
        std::fs::write(repo.join("README.md"), "uncommitted\n").unwrap();
        let s = Scratch::new(repo, "origin", Some("main"), None).unwrap();
        assert_eq!(s.file("README.md").as_deref(), Some("uncommitted\n"));
        assert!(s.work.join("README.md").exists());
        assert!(git(repo, &["rev-parse", "-q", "--verify", "HEAD"]).is_err());
        assert_eq!(
            git(repo, &["status", "--porcelain"]).unwrap(),
            "?? README.md"
        );
        drop(s);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_misspelt_case_field_is_an_error_not_ignored() {
        let e = serde_norway::from_str::<Case>(
            "{name: n, action: a, aprove: [user:bob], expect: {state: x}}",
        )
        .err()
        .unwrap()
        .to_string();
        assert!(e.contains("aprove"), "{e}");
    }

    #[test]
    fn cases_start_from_the_defaults_and_merge_inputs() {
        let s: Suite = serde_norway::from_str(
            "name: s
defaults: {action: a, as: user:ann, inputs: {x: '1', y: '2'}, approve: [user:bob]}
runs:
  - {name: all of them, expect: {state: succeeded}}
  - {name: one replaced, inputs: {x: '3'}, expect: {state: succeeded}}
  - {name: one left out, inputs: {y: null}, approve: [], expect: {state: invalid}}
  - {name: another action, action: b, expect: {state: refused}}",
        )
        .unwrap();
        let c = s.cases("t.yaml", "alpha").unwrap();
        let inputs = |i: usize| serde_json::to_string(&c[i].inputs).unwrap();
        assert_eq!(inputs(0), r#"{"x":"1","y":"2"}"#);
        assert_eq!(inputs(1), r#"{"x":"3","y":"2"}"#);
        assert_eq!(inputs(2), r#"{"x":"1"}"#);
        assert_eq!((c[0].as_user.as_str(), c[0].approve.len()), ("user:ann", 1));
        assert!(c[2].approve.is_empty());
        assert_eq!((c[3].action.as_str(), c[0].action.as_str()), ("b", "a"));
        let bad: Suite =
            serde_norway::from_str("name: s\ndefaults: {aprove: [user:bob]}\nruns: [{name: n, action: a, expect: {state: x}}]")
                .unwrap();
        assert!(
            bad.cases("t.yaml", "alpha")
                .err()
                .unwrap()
                .contains("aprove")
        );
    }

    #[test]
    fn portal_in_a_case_is_the_name_of_the_portal_under_test() {
        let s: Suite = serde_norway::from_str(
            "name: s
defaults: {action: a, inputs: {successor: 'acme/{portal}'}}
runs:
  - name: '{portal} is not its own successor'
    expect: {state: blocked, error: '{portal}', files: {OWNER.md: 'owned by {portal}'}}",
        )
        .unwrap();
        let c = &s.cases("t.yaml", "beta").unwrap()[0];
        assert_eq!(c.name, "beta is not its own successor");
        assert_eq!(c.inputs["successor"], "acme/beta");
        assert_eq!(c.expect.error.as_deref(), Some("beta"));
        assert_eq!(c.expect.files["OWNER.md"].as_deref(), Some("owned by beta"));
    }
}
