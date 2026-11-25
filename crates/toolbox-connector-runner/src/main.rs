//! The `runner` connector (in-image, stdio): runs local commands and reports their results.
//! Where the `workspace` connector reads files, this one *executes* things, so its records are
//! slower to produce and are refreshed per entity: `refresh_entity crate:toolbox-derive` reruns
//! only that crate's jobs (docs/design/toolbox-for-toolbox.md §3).
//!
//! `TOOLBOX_CONNECTOR_CONFIG`: `{"name"?: "runner", "root"?: ".", "jobs": [job]}` where a job is
//! `{"name", "command": [argv], "per"?: "crate" | "once", "parse": "cargo-test" | "clippy" | "ndjson" | "tally" | "sarif" | "zizmor" | "actionlint" | "semgrep" | "scorecard"}`, and `"tracked": true` to run in an export of the git-tracked files.
//! In `per: crate` jobs, `{crate}` in the command is replaced by each workspace member's name and
//! the job emits one record of kind `<name>` with external id `<name>:<crate>`. `ndjson` jobs print
//! connector `record` messages themselves (e.g. `toolbox describe`); they are passed through, and
//! the job declares their `kinds`. `sarif` jobs run any tool that prints SARIF 2.1.0 (or writes it
//! to `output`), the standard slot (docs/design/toolbox-and-tools.md); `zizmor`, `actionlint` and
//! `semgrep` are fittings for those tools' own JSON. Every lint record carries `command`, the
//! command that reproduces its findings with the tool alone. These jobs run that linter and
//! report one record per file matching their `files` globs, and for semgrep per file it scanned
//! (findings with severity and line, counts, and `suppressed`/`active`: findings marked
//! `nosemgrep` inline, when semgrep runs with `--disable-nosem`). An
//! argument that is exactly `{files}` expands to those files, so a linter gets only paths that
//! exist; with none, the job doesn't run. `optional: true` skips a job whose program isn't installed. `tally` jobs run their command, then count the lines of the NDJSON
//! `file` it wrote by the value of `key` (one record per value: `{<key>, calls, errors}`, where a
//! line with `"error": true` is an error, and a line with `"declared": true` only declares the key,
//! so keys nothing used are reported with zero calls); the file is removed before each run.

mod lint;
mod scorecard;

use regex::Regex;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Instant;
use toolbox_connect::sdk::{Out, Source, run_stdio};

#[derive(Clone)]
struct Job {
    name: String,
    command: Vec<String>,
    per_crate: bool,
    parse: String,
    /// Record kinds an `ndjson` job produces (for refresh routing and resync deletions).
    kinds: Vec<String>,
    /// `tally` jobs: the NDJSON file the command writes, and the field to count by.
    file: Option<String>,
    key: Option<String>,
    /// `zizmor` / `actionlint` jobs: the files they cover (globs); each gets a record.
    files: Vec<Regex>,
    /// A missing program is skipped with a log line instead of failing the command; its records
    /// are then absent, so rules over them evaluate to `unknown`, not `fail`.
    optional: bool,
    /// Lint jobs: the file the tool writes its report to (relative to the root), instead of stdout.
    output: Option<String>,
    /// Run in an export of the git-tracked files (with their uncommitted edits), as a fresh CI
    /// checkout would be, for tools that walk the whole directory (Scorecard's `--local`).
    tracked: bool,
}

impl Job {
    /// Jobs whose record ids carry the job name (`<job>:<…>`), as opposed to `ndjson` jobs.
    fn prefixed(&self) -> bool {
        self.per_crate || self.parse != "ndjson"
    }
}

struct Runner {
    /// The connector's name in the manifest (`name` in the config; default `runner`).
    name: String,
    root: PathBuf,
    jobs: Vec<Job>,
}

/// Captured output of one command.
struct Run {
    ok: bool,
    stdout: String,
    stderr: String,
    millis: u64,
}

/// The last `max` characters of `s`, on a line boundary.
fn tail(s: &str, max: usize) -> String {
    if s.len() <= max {
        return s.to_owned();
    }
    let start = s.len() - max;
    let start = (start..s.len())
        .find(|&i| s.is_char_boundary(i))
        .unwrap_or(s.len());
    let cut = &s[start..];
    cut.find('\n').map_or(cut, |i| &cut[i + 1..]).to_owned()
}

/// Sum of `test result:` lines from `cargo test`.
fn parse_cargo_test(run: &Run) -> Value {
    let re = Regex::new(r"test result: \w+\. (\d+) passed; (\d+) failed; (\d+) ignored").unwrap();
    let (mut passed, mut failed, mut ignored, mut suites) = (0u64, 0u64, 0u64, 0u64);
    let text = format!("{}\n{}", run.stdout, run.stderr);
    for c in re.captures_iter(&text) {
        suites += 1;
        passed += c[1].parse::<u64>().unwrap_or(0);
        failed += c[2].parse::<u64>().unwrap_or(0);
        ignored += c[3].parse::<u64>().unwrap_or(0);
    }
    let status = match (run.ok, failed) {
        (true, _) => "pass",
        (false, n) if n > 0 => "fail",
        _ => "error",
    };
    let mut v = json!({"status": status, "passed": passed, "failed": failed, "ignored": ignored,
                       "suites": suites, "durationMs": run.millis});
    if status != "pass" {
        v["output"] = json!(tail(&text, 2000));
    }
    v
}

/// Warnings and errors from `cargo clippy --message-format=json` that belong to `manifest`.
fn parse_clippy(run: &Run, manifest: &str) -> Value {
    let (mut warnings, mut errors, mut messages) = (0u64, 0u64, Vec::new());
    for line in run.stdout.lines() {
        let Ok(m) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        if m["reason"] != "compiler-message" || m["manifest_path"] != manifest {
            continue;
        }
        let msg = &m["message"];
        match msg["level"].as_str() {
            Some("warning") => warnings += 1,
            Some("error") => errors += 1,
            _ => continue,
        }
        if messages.len() < 20 {
            let span = msg["spans"].get(0);
            let at = span.map_or(String::new(), |s| {
                format!(
                    "{}:{}",
                    s["file_name"].as_str().unwrap_or("?"),
                    s["line_start"]
                )
            });
            let code = msg["code"]["code"].as_str().unwrap_or("");
            messages.push(format!(
                "{at} {code} {}",
                msg["message"].as_str().unwrap_or("")
            ));
        }
    }
    let status = if !run.ok || errors > 0 {
        "error"
    } else if warnings > 0 {
        "fail"
    } else {
        "pass"
    };
    let mut v = json!({"status": status, "warnings": warnings, "errors": errors,
                       "messages": messages, "durationMs": run.millis});
    if !run.ok && errors == 0 {
        v["output"] = json!(tail(&run.stderr, 2000));
    }
    v
}

/// The SDK's output, narrowed to what a command asked for: on refresh only the requested ids, on a
/// resync of named kinds only those kinds (the conformance kit checks both). Remembers what it
/// emitted, so ids nothing produced are reported `not_found`.
struct Sink<'o> {
    out: &'o mut Out,
    ids: Option<Vec<String>>,
    kinds: Option<Vec<String>>,
    emitted: std::collections::BTreeSet<String>,
}

impl<'o> Sink<'o> {
    fn new(out: &'o mut Out, ids: Option<Vec<String>>, kinds: Option<Vec<String>>) -> Self {
        Sink {
            out,
            ids,
            kinds,
            emitted: Default::default(),
        }
    }

    fn wanted_kind(&self, kind: &str) -> bool {
        self.kinds
            .as_ref()
            .is_none_or(|k| k.iter().any(|x| x == kind))
    }

    fn wants_any(&self, kinds: &[&str]) -> bool {
        kinds.iter().any(|k| self.wanted_kind(k))
    }

    fn record(&mut self, kind: &str, id: &str, data: Value) {
        if self.wanted_kind(kind)
            && self
                .ids
                .as_ref()
                .is_none_or(|ids| ids.iter().any(|i| i == id))
        {
            self.emitted.insert(id.to_owned());
            self.out.record(kind, id, data);
        }
    }

    fn not_found(&mut self, kind: &str, id: &str) {
        self.out.not_found(kind, id);
    }

    fn snapshot_complete(&mut self, kind: &str) {
        if self.wanted_kind(kind) {
            self.out.snapshot_complete(kind);
        }
    }

    fn log(&mut self, level: &str, message: &str) {
        self.out.log(level, message);
    }
}

impl Runner {
    fn from_env() -> Result<Self, String> {
        let cfg: Value = serde_json::from_str(
            &std::env::var("TOOLBOX_CONNECTOR_CONFIG").unwrap_or_else(|_| "{}".into()),
        )
        .map_err(|e| format!("TOOLBOX_CONNECTOR_CONFIG: {e}"))?;
        let mut jobs = Vec::new();
        for j in cfg["jobs"].as_array().into_iter().flatten() {
            let name = j["name"]
                .as_str()
                .ok_or("every job needs a `name`")?
                .to_owned();
            let command: Vec<String> = j["command"]
                .as_array()
                .ok_or(format!("job {name}: `command` must be a list"))?
                .iter()
                .filter_map(|a| a.as_str().map(str::to_owned))
                .collect();
            if command.is_empty() {
                return Err(format!("job {name}: empty command"));
            }
            let parse = j["parse"].as_str().unwrap_or("ndjson").to_owned();
            if !matches!(
                parse.as_str(),
                "cargo-test"
                    | "clippy"
                    | "ndjson"
                    | "tally"
                    | "sarif"
                    | "zizmor"
                    | "actionlint"
                    | "semgrep"
                    | "scorecard"
            ) {
                return Err(format!("job {name}: unknown parser {parse}"));
            }
            let kinds = j["kinds"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|k| k.as_str().map(str::to_owned))
                .collect();
            let s = |k: &str| j[k].as_str().map(str::to_owned);
            if parse == "tally" && (s("file").is_none() || s("key").is_none()) {
                return Err(format!("job {name}: tally needs `file` and `key`"));
            }
            let files = j["files"]
                .as_array()
                .into_iter()
                .flatten()
                .map(|g| toolbox_connect::files::glob(g.as_str().unwrap_or("")))
                .collect::<Result<Vec<_>, _>>()?;
            jobs.push(Job {
                per_crate: j["per"] == "crate",
                files,
                optional: j["optional"].as_bool().unwrap_or(false),
                output: s("output"),
                tracked: j["tracked"].as_bool().unwrap_or(false),
                file: s("file"),
                key: s("key"),
                name,
                command,
                parse,
                kinds,
            });
        }
        Ok(Runner {
            name: cfg["name"].as_str().unwrap_or("runner").to_owned(),
            root: PathBuf::from(cfg["root"].as_str().unwrap_or(".")),
            jobs,
        })
    }

    async fn run(&self, argv: &[String]) -> Run {
        self.run_in(argv, &self.root).await
    }

    async fn run_in(&self, argv: &[String], dir: &std::path::Path) -> Run {
        let start = Instant::now();
        let out = tokio::process::Command::new(&argv[0])
            .args(&argv[1..])
            .current_dir(dir)
            .output()
            .await;
        let millis = start.elapsed().as_millis() as u64;
        match out {
            Ok(o) => Run {
                ok: o.status.success(),
                stdout: String::from_utf8_lossy(&o.stdout).into_owned(),
                stderr: String::from_utf8_lossy(&o.stderr).into_owned(),
                millis,
            },
            Err(e) => Run {
                ok: false,
                stdout: String::new(),
                stderr: format!("cannot run {}: {e}", argv[0]),
                millis,
            },
        }
    }

    /// Workspace members: name → manifest path.
    async fn crates(&self) -> Result<BTreeMap<String, String>, String> {
        let run = self
            .run(&["cargo", "metadata", "--no-deps", "--format-version", "1"].map(String::from))
            .await;
        if !run.ok {
            return Err(format!("cargo metadata: {}", run.stderr.trim()));
        }
        let meta: Value = serde_json::from_str(&run.stdout).map_err(|e| e.to_string())?;
        Ok(meta["packages"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|p| {
                Some((
                    p["name"].as_str()?.to_owned(),
                    p["manifest_path"].as_str()?.to_owned(),
                ))
            })
            .collect())
    }

    async fn run_for_crate(&self, job: &Job, name: &str, manifest: &str, out: &mut Sink<'_>) {
        let argv: Vec<String> = job
            .command
            .iter()
            .map(|a| a.replace("{crate}", name))
            .collect();
        let run = self.run(&argv).await;
        let mut data = match job.parse.as_str() {
            "cargo-test" => parse_cargo_test(&run),
            _ => parse_clippy(&run, manifest),
        };
        data["crate"] = json!(name);
        out.record(&job.name, &format!("{}:{name}", job.name), data);
    }

    /// Run a linter over the repository and emit one record per covered file (`<job>:<path>`),
    /// clean files included. A missing optional program emits nothing (its rules become unknown).
    async fn run_lint(&self, job: &Job, out: &mut Sink<'_>) -> Result<(), String> {
        let files = toolbox_connect::files::matching(&self.root, &job.files);
        let Some(argv) = with_files(&job.command, &files) else {
            return Ok(()); // `{files}` and nothing to lint: no records, nothing failed
        };
        let run = self.run(&argv).await;
        if run.stderr.starts_with("cannot run ") {
            if job.optional {
                out.log(
                    "warn",
                    &format!("job {}: {}; skipped (optional)", job.name, run.stderr),
                );
                return Ok(());
            }
            return Err(format!("job {}: {}", job.name, run.stderr));
        }
        // A tool that writes its report to a file (`output`) is read from there, else stdout.
        let text = match &job.output {
            Some(f) => std::fs::read_to_string(self.root.join(f))
                .map_err(|e| format!("job {}: {f}: {e}", job.name))?,
            None => run.stdout.clone(),
        };
        let parsed = match job.parse.as_str() {
            "sarif" => lint::sarif(&text, &self.root),
            "zizmor" => lint::zizmor(&text),
            "semgrep" => lint::semgrep(&text, &self.root),
            _ => lint::actionlint(&text),
        }
        .map_err(|e| {
            format!(
                "job {}: {e}; stderr: {}",
                job.name,
                tail(&run.stderr, 500).trim()
            )
        })?;
        let mut by_file = parsed;
        // Take-it-out: each record names the command that gives its file's findings with the tool
        // alone (`{files}` as that one file; a whole-tree run as it was run).
        let command = |path: &str| {
            shell_words(&with_files(&job.command, &[path.to_owned()]).unwrap_or_default())
        };
        for path in files {
            let findings = by_file.remove(&path).unwrap_or_default();
            out.record(
                &job.name,
                &format!("{}:{path}", job.name),
                lint::record(&path, &command(&path), &job.parse, findings),
            );
        }
        for (path, findings) in by_file {
            // Findings in a file outside `files` (e.g. an action used by a workflow): still report.
            if !path.is_empty() {
                out.record(
                    &job.name,
                    &format!("{}:{path}", job.name),
                    lint::record(&path, &command(&path), &job.parse, findings),
                );
            }
        }
        Ok(())
    }

    /// Run the command, then count the lines of its NDJSON `file` by `key`.
    async fn run_tally(&self, job: &Job, out: &mut Sink<'_>) -> Result<(), String> {
        let (file, key) = (
            job.file.as_deref().unwrap_or(""),
            job.key.as_deref().unwrap_or(""),
        );
        let path = self.root.join(file);
        let _ = std::fs::remove_file(&path);
        let run = self.run(&job.command).await;
        if !run.ok {
            return Err(format!(
                "job {}: {}",
                job.name,
                tail(&run.stderr, 500).trim()
            ));
        }
        let text = std::fs::read_to_string(&path).unwrap_or_default();
        let mut counts: BTreeMap<String, (u64, u64)> = BTreeMap::new();
        for line in text.lines() {
            let Ok(v) = serde_json::from_str::<Value>(line) else {
                continue;
            };
            if let Some(k) = v[key].as_str() {
                let c = counts.entry(k.to_owned()).or_default();
                if v["declared"] == true {
                    continue; // a declared key: reported even with no lines counted
                }
                c.0 += 1;
                c.1 += u64::from(v["error"] == true);
            }
        }
        for (k, (calls, errors)) in counts {
            out.record(
                &job.name,
                &format!("{}:{k}", job.name),
                json!({key: k, "calls": calls, "errors": errors}),
            );
        }
        Ok(())
    }

    /// OpenSSF Scorecard: one record per check (`<job>:<check>`). With `tracked`, it runs in an
    /// export of the tracked files, so untracked build output doesn't count as binaries.
    async fn run_scorecard(&self, job: &Job, out: &mut Sink<'_>) -> Result<(), String> {
        let export = if job.tracked {
            Some(export_tracked(&self.root)?)
        } else {
            None
        };
        let run = self
            .run_in(
                &job.command,
                export
                    .as_ref()
                    .map_or(self.root.as_path(), |d| d.0.as_path()),
            )
            .await;
        drop(export);
        if !run.ok {
            if job.optional && run.stderr.starts_with("cannot run ") {
                out.log(
                    "warning",
                    &format!("job {}: {}; skipped (optional)", job.name, run.stderr),
                );
                return Ok(());
            }
            return Err(format!(
                "job {}: {}",
                job.name,
                tail(&run.stderr, 500).trim()
            ));
        }
        let mut command = shell_words(&job.command);
        if job.tracked {
            // The same export as `export_tracked`, as one shell command a person can paste.
            command = format!(
                "d=$(mktemp -d); rev=$(git stash create); git archive \"${{rev:-HEAD}}\" | tar -x -C \"$d\" && cd \"$d\" && {command}"
            );
        }
        for (check, mut data) in
            scorecard::checks(&run.stdout).map_err(|e| format!("job {}: {e}", job.name))?
        {
            data["command"] = json!(command);
            out.record(&job.name, &format!("{}:{check}", job.name), data);
        }
        Ok(())
    }

    async fn run_ndjson(&self, job: &Job, out: &mut Sink<'_>) -> Result<(), String> {
        let run = self.run(&job.command).await;
        if !run.ok {
            return Err(format!(
                "job {}: {}",
                job.name,
                tail(&run.stderr, 500).trim()
            ));
        }
        for line in run.stdout.lines().filter(|l| !l.trim().is_empty()) {
            let m: Value =
                serde_json::from_str(line).map_err(|e| format!("job {}: {e}", job.name))?;
            if m["type"] == "record" {
                let (Some(kind), Some(id)) = (m["kind"].as_str(), m["externalId"].as_str()) else {
                    continue;
                };
                out.record(kind, id, m["data"].clone());
            }
        }
        Ok(())
    }

    /// Run every job (resync), or only the jobs and crates named by `ids` (refresh).
    async fn execute(
        &self,
        ids: Option<&[String]>,
        out: &mut Sink<'_>,
        resync: bool,
    ) -> Result<(), String> {
        // Only per-crate jobs need Cargo: zizmor, actionlint and NDJSON jobs run in any repository.
        let crates = if self.jobs.iter().any(|j| j.per_crate) {
            self.crates().await?
        } else {
            BTreeMap::new()
        };
        for job in &self.jobs {
            if !out.wants_any(&self.kinds_of(job)) {
                continue;
            }
            if job.per_crate {
                self.execute_per_crate(job, &crates, ids, out).await;
            } else if self.selected(job, ids) {
                if job.parse == "tally" {
                    self.run_tally(job, out).await?;
                } else if job.parse == "scorecard" {
                    self.run_scorecard(job, out).await?;
                } else if matches!(
                    job.parse.as_str(),
                    "sarif" | "zizmor" | "actionlint" | "semgrep"
                ) {
                    self.run_lint(job, out).await?;
                } else {
                    self.run_ndjson(job, out).await?;
                }
            } else {
                continue;
            }
            if resync {
                for k in self.kinds_of(job) {
                    out.snapshot_complete(k);
                }
            }
        }
        // Refresh: anything asked for that no job produced (and no per-crate job already reported).
        for id in ids.into_iter().flatten() {
            let per_crate = self
                .jobs
                .iter()
                .any(|j| j.per_crate && id.starts_with(&format!("{}:", j.name)));
            if !per_crate && !out.emitted.contains(id) {
                let kind = id.split(':').next().unwrap_or("record").to_owned();
                out.not_found(&kind, id);
            }
        }
        Ok(())
    }

    /// A per-crate job for every crate (resync) or the crates its refresh ids name.
    async fn execute_per_crate(
        &self,
        job: &Job,
        crates: &BTreeMap<String, String>,
        ids: Option<&[String]>,
        out: &mut Sink<'_>,
    ) {
        for (name, manifest) in crates {
            let id = format!("{}:{name}", job.name);
            if ids.is_none_or(|ids| ids.contains(&id)) {
                self.run_for_crate(job, name, manifest, out).await;
            }
        }
        let prefix = format!("{}:", job.name);
        for id in ids.into_iter().flatten().filter(|i| i.starts_with(&prefix)) {
            if !crates.contains_key(&id[prefix.len()..]) {
                out.not_found(&job.name, id);
            }
        }
    }

    /// Whether a once-per-run job runs: always on resync; on refresh when an id is its own (tally
    /// ids carry the job name; ndjson ids are whatever the command printed).
    fn selected(&self, job: &Job, ids: Option<&[String]>) -> bool {
        let Some(ids) = ids else { return true };
        if job.prefixed() {
            ids.iter().any(|i| i.starts_with(&format!("{}:", job.name)))
        } else {
            self.owns_any(ids)
        }
    }

    /// The record kinds a job produces.
    fn kinds_of<'a>(&self, job: &'a Job) -> Vec<&'a str> {
        if job.prefixed() {
            vec![job.name.as_str()]
        } else {
            job.kinds.iter().map(String::as_str).collect()
        }
    }

    /// Whether a refresh names a record that this ndjson job produced (its ids carry no job name,
    /// so any id without a `<per-crate job>:` prefix belongs to the ndjson jobs).
    fn owns_any(&self, ids: &[String]) -> bool {
        ids.iter().any(|id| {
            !self
                .jobs
                .iter()
                .any(|j| j.prefixed() && id.starts_with(&format!("{}:", j.name)))
        })
    }
}

impl Source for Runner {
    fn name(&self) -> &str {
        &self.name
    }

    fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }

    fn capabilities(&self) -> Value {
        let kinds: Vec<&str> = self.jobs.iter().flat_map(|j| self.kinds_of(j)).collect();
        let refresh: serde_json::Map<String, Value> = kinds
            .iter()
            .map(|k| (k.to_string(), json!(["external_ids"])))
            .collect();
        json!({"refresh": refresh, "resync": kinds, "max_concurrency": 1})
    }

    async fn handle(&mut self, cmd: &Value, out: &mut Out) -> Result<(), String> {
        match cmd["command"].as_str() {
            Some("resync") => {
                let kinds: Option<Vec<String>> = cmd["kinds"].as_array().map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_str().map(str::to_owned))
                        .collect()
                });
                let mut sink = Sink::new(out, None, kinds);
                self.execute(None, &mut sink, true).await
            }
            Some("refresh") => {
                let ids: Vec<String> = cmd["external_ids"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(|v| v.as_str().map(str::to_owned))
                    .collect();
                let mut sink = Sink::new(out, Some(ids.clone()), None);
                self.execute(Some(&ids), &mut sink, false).await
            }
            other => Err(format!("unsupported command {other:?}")),
        }
    }
}

/// `command` with an argument that is exactly `{files}` replaced by `files`; `None` when it asks for
/// files and there are none.
/// A temporary directory removed when dropped: on success, on any error after it was made, and when
/// the job's future is cancelled, so a failed or interrupted export never accumulates on disk.
struct TempTree(PathBuf);

impl Drop for TempTree {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// The git-tracked files under `root`, with their uncommitted edits (`git stash create`), exported
/// to a new temporary directory: the tree a fresh CI checkout sees, without untracked build output.
fn export_tracked(root: &std::path::Path) -> Result<TempTree, String> {
    let git = |args: &[&str]| {
        std::process::Command::new("git")
            .arg("-C")
            .arg(root)
            .args(args)
            .output()
            .map_err(|e| format!("git: {e}"))
    };
    let stash = git(&["stash", "create"])?;
    let rev = String::from_utf8_lossy(&stash.stdout).trim().to_owned();
    let rev = if rev.is_empty() {
        "HEAD".to_owned()
    } else {
        rev
    };
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    let dir = TempTree(
        std::env::temp_dir().join(format!("toolbox-tracked-{}-{nanos}", std::process::id())),
    );
    std::fs::create_dir_all(&dir.0).map_err(|e| format!("{}: {e}", dir.0.display()))?;
    let tar = dir.0.join(".tracked.tar");
    let archive = git(&[
        "archive",
        "--format=tar",
        "-o",
        &tar.to_string_lossy(),
        &rev,
    ])?;
    if !archive.status.success() {
        return Err(format!(
            "git archive: {}",
            String::from_utf8_lossy(&archive.stderr).trim()
        ));
    }
    let untar = std::process::Command::new("tar")
        .arg("-xf")
        .arg(&tar)
        .arg("-C")
        .arg(&dir.0)
        .status()
        .map_err(|e| format!("tar: {e}"))?;
    let _ = std::fs::remove_file(&tar);
    if !untar.success() {
        return Err("tar: could not unpack the tracked files".into());
    }
    Ok(dir)
}

/// An argv as one shell command line: words that need it are single-quoted.
fn shell_words(argv: &[String]) -> String {
    argv.iter()
        .map(|a| {
            if !a.is_empty()
                && a.chars()
                    .all(|c| c.is_ascii_alphanumeric() || "-_./=:,+@%".contains(c))
            {
                a.clone()
            } else {
                format!("'{}'", a.replace('\'', "'\\''"))
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn with_files(command: &[String], files: &[String]) -> Option<Vec<String>> {
    if !command.iter().any(|a| a == "{files}") {
        return Some(command.to_vec());
    }
    if files.is_empty() {
        return None;
    }
    Some(
        command
            .iter()
            .flat_map(|a| {
                if a == "{files}" {
                    files.to_vec()
                } else {
                    vec![a.clone()]
                }
            })
            .collect(),
    )
}

#[tokio::main]
async fn main() {
    match Runner::from_env() {
        Ok(src) => run_stdio(src).await,
        Err(e) => {
            eprintln!("toolbox-connector-runner: {e}");
            std::process::exit(2);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(ok: bool, stdout: &str) -> Run {
        Run {
            ok,
            stdout: stdout.into(),
            stderr: String::new(),
            millis: 5,
        }
    }

    #[test]
    fn sums_test_results_across_suites() {
        let r = parse_cargo_test(&run(
            false,
            "test result: ok. 3 passed; 0 failed; 1 ignored; 0 measured\n\
             test result: FAILED. 2 passed; 1 failed; 0 ignored; 0 measured\n",
        ));
        assert_eq!(
            (
                r["status"].as_str(),
                r["passed"].as_u64(),
                r["failed"].as_u64()
            ),
            (Some("fail"), Some(5), Some(1))
        );
        let r = parse_cargo_test(&run(false, "error[E0425]: cannot find value"));
        assert_eq!(r["status"], "error");
    }

    #[test]
    fn counts_only_this_crates_clippy_messages() {
        let mine = r#"{"reason":"compiler-message","manifest_path":"/w/a/Cargo.toml","message":{"level":"warning","message":"x","code":{"code":"clippy::needless_return"},"spans":[{"file_name":"src/lib.rs","line_start":3}]}}"#;
        let other = r#"{"reason":"compiler-message","manifest_path":"/w/b/Cargo.toml","message":{"level":"warning","message":"y","code":null,"spans":[]}}"#;
        let r = parse_clippy(&run(true, &format!("{mine}\n{other}\n")), "/w/a/Cargo.toml");
        assert_eq!(
            (r["status"].as_str(), r["warnings"].as_u64()),
            (Some("fail"), Some(1))
        );
        assert_eq!(r["messages"][0], "src/lib.rs:3 clippy::needless_return x");
    }

    #[test]
    fn a_failed_tracked_export_leaves_no_directory_behind() {
        let tmp = std::env::temp_dir();
        let before = |p: &str| {
            std::fs::read_dir(&tmp)
                .into_iter()
                .flatten()
                .flatten()
                .filter(|e| e.file_name().to_string_lossy().starts_with(p))
                .count()
        };
        let prefix = format!("toolbox-tracked-{}-", std::process::id());
        let n = before(&prefix);
        let not_git = tmp.join(format!("toolbox-not-git-{}", std::process::id()));
        std::fs::create_dir_all(&not_git).unwrap();
        assert!(super::export_tracked(&not_git).is_err());
        assert_eq!(before(&prefix), n, "the export directory was removed");
        let _ = std::fs::remove_dir_all(&not_git);
    }
}
