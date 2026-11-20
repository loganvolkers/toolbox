//! The `runner` connector (in-image, stdio): runs local commands and reports their results.
//! Where the `workspace` connector reads files, this one *executes* things, so its records are
//! slower to produce and are refreshed per entity: `refresh_entity crate:toolbox-derive` reruns
//! only that crate's jobs (docs/design/toolbox-for-toolbox.md §3).
//!
//! `TOOLBOX_CONNECTOR_CONFIG`: `{"root"?: ".", "jobs": [job]}` where a job is
//! `{"name", "command": [argv], "per"?: "crate" | "once", "parse": "cargo-test" | "clippy" | "ndjson"}`.
//! In `per: crate` jobs, `{crate}` in the command is replaced by each workspace member's name and
//! the job emits one record of kind `<name>` with external id `<name>:<crate>`. `ndjson` jobs print
//! connector `record` messages themselves (e.g. `toolbox describe`); they are passed through, and
//! the job declares their `kinds`.

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
}

struct Runner {
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
            if !matches!(parse.as_str(), "cargo-test" | "clippy" | "ndjson") {
                return Err(format!("job {name}: unknown parser {parse}"));
            }
            let kinds = j["kinds"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|k| k.as_str().map(str::to_owned))
                .collect();
            jobs.push(Job {
                per_crate: j["per"] == "crate",
                name,
                command,
                parse,
                kinds,
            });
        }
        Ok(Runner {
            root: PathBuf::from(cfg["root"].as_str().unwrap_or(".")),
            jobs,
        })
    }

    async fn run(&self, argv: &[String]) -> Run {
        let start = Instant::now();
        let out = tokio::process::Command::new(&argv[0])
            .args(&argv[1..])
            .current_dir(&self.root)
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

    async fn run_for_crate(&self, job: &Job, name: &str, manifest: &str, out: &mut Out) {
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

    async fn run_ndjson(&self, job: &Job, out: &mut Out) -> Result<(), String> {
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
        out: &mut Out,
        resync: bool,
    ) -> Result<(), String> {
        let crates = self.crates().await?;
        for job in &self.jobs {
            if job.per_crate {
                for (name, manifest) in &crates {
                    let id = format!("{}:{name}", job.name);
                    if ids.is_none_or(|ids| ids.contains(&id)) {
                        self.run_for_crate(job, name, manifest, out).await;
                    }
                }
                for id in ids
                    .into_iter()
                    .flatten()
                    .filter(|i| i.starts_with(&format!("{}:", job.name)))
                {
                    if !crates.contains_key(&id[job.name.len() + 1..]) {
                        out.not_found(&job.name, id);
                    }
                }
                if resync {
                    out.snapshot_complete(&job.name);
                }
            } else if ids.is_none() || self.owns_any(ids.unwrap_or_default()) {
                self.run_ndjson(job, out).await?;
                if resync {
                    for k in &job.kinds {
                        out.snapshot_complete(k);
                    }
                }
            }
        }
        Ok(())
    }

    /// Whether a refresh names a record that this ndjson job produced (its ids carry no job name,
    /// so any id without a `<per-crate job>:` prefix belongs to the ndjson jobs).
    fn owns_any(&self, ids: &[String]) -> bool {
        ids.iter().any(|id| {
            !self
                .jobs
                .iter()
                .any(|j| j.per_crate && id.starts_with(&format!("{}:", j.name)))
        })
    }
}

impl Source for Runner {
    fn name(&self) -> &str {
        "runner"
    }

    fn capabilities(&self) -> Value {
        let kinds: Vec<&str> = self
            .jobs
            .iter()
            .flat_map(|j| {
                if j.per_crate {
                    vec![j.name.as_str()]
                } else {
                    j.kinds.iter().map(String::as_str).collect()
                }
            })
            .collect();
        let refresh: serde_json::Map<String, Value> = kinds
            .iter()
            .map(|k| (k.to_string(), json!(["external_ids"])))
            .collect();
        json!({"refresh": refresh, "resync": kinds, "max_concurrency": 1})
    }

    async fn handle(&mut self, cmd: &Value, out: &mut Out) -> Result<(), String> {
        match cmd["command"].as_str() {
            Some("resync") => self.execute(None, out, true).await,
            Some("refresh") => {
                let ids: Vec<String> = cmd["external_ids"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(|v| v.as_str().map(str::to_owned))
                    .collect();
                self.execute(Some(&ids), out, false).await
            }
            other => Err(format!("unsupported command {other:?}")),
        }
    }
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
}
