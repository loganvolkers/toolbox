//! The `toolbox` binary (docs/v1-spec.md §7.7).

mod portal;
mod server;

use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser)]
#[command(
    name = "toolbox",
    version,
    about = "Agent-first internal developer portal"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Validate a portal: manifest schema, cross-references, action input forms, CEL, files and
    /// fixtures. Exit code 0 when valid, 1 when there are errors, 2 when it can't be loaded.
    Validate {
        /// A portal directory (containing toolbox.yaml) or a manifest file.
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Output format.
        #[arg(long, value_enum, default_value_t = Format::Text)]
        format: Format,
    },
    /// Recompute derived state from the portal's fixtures and run its expectation files
    /// (tests/*.test.yaml). Exit code 1 if any expectation fails.
    Test {
        #[arg(default_value = ".")]
        path: PathBuf,
        #[arg(long, value_enum, default_value_t = Format::Text)]
        format: Format,
    },
    /// Show what a manifest change would do: two recomputes over the same inputs, then a diff.
    Plan {
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Compare against this manifest file (the "before"); the portal's manifest is the "after".
        #[arg(long)]
        base: PathBuf,
        /// Use the portal's fixtures (the only mode until M6 adds --remote).
        #[arg(long, default_value_t = true)]
        local: bool,
        #[arg(long, value_enum, default_value_t = PlanFormat::Md)]
        format: PlanFormat,
    },
    /// Run the server: snapshot loop + REST API. Uses Postgres when --database-url (or
    /// DATABASE_URL) is set, otherwise an in-memory store (optionally preloaded with fixtures).
    Serve {
        #[arg(default_value = ".")]
        path: PathBuf,
        #[arg(long, env = "TOOLBOX_LISTEN", default_value = "127.0.0.1:8080")]
        listen: String,
        #[arg(long, env = "DATABASE_URL")]
        database_url: Option<String>,
        /// In-memory mode only: load the portal's fixtures at start.
        #[arg(long)]
        fixtures: bool,
    },
    /// Send a portal's fixtures to a running server through the ingest API (one batch per
    /// connector, then native values), and wait for the snapshot that contains them.
    Push {
        #[arg(default_value = ".")]
        path: PathBuf,
        #[arg(long, default_value = "http://127.0.0.1:8080")]
        url: String,
    },
    /// Recompute derived state from fixtures and print a summary, or write the reference-format
    /// files (expected/*.json, authz/tuples.expected.json) to --out.
    Derive {
        #[arg(default_value = ".")]
        path: PathBuf,
        #[arg(long)]
        out: Option<PathBuf>,
    },
}

#[derive(Clone, Copy, ValueEnum)]
enum PlanFormat {
    Md,
    Json,
}

#[derive(Clone, Copy, PartialEq, ValueEnum)]
enum Format {
    Text,
    Json,
}

fn main() -> ExitCode {
    match Cli::parse().command {
        Command::Validate { path, format } => validate(&path, format),
        Command::Test { path, format } => test(&path, format),
        Command::Plan {
            path, base, format, ..
        } => plan(&path, &base, format),
        Command::Derive { path, out } => derive(&path, out.as_deref()),
        Command::Serve {
            path,
            listen,
            database_url,
            fixtures,
        } => server::serve(&path, &listen, database_url, fixtures),
        Command::Push { path, url } => server::push(&path, &url),
    }
}

fn fail(msg: impl std::fmt::Display) -> ExitCode {
    eprintln!("{msg}");
    ExitCode::from(2)
}

fn test(path: &std::path::Path, format: Format) -> ExitCode {
    let local = match portal::load(path) {
        Ok(l) => l,
        Err(e) => return fail(e),
    };
    let derived = std::sync::Arc::new(local.derived);
    let results =
        match toolbox_derive::testing::run(&local.root, &local.manifest.tests, derived.clone()) {
            Ok(r) => r,
            Err(e) => return fail(e),
        };
    let authz_tests = local
        .manifest
        .authz
        .as_ref()
        .map(|a| a.tests.clone())
        .unwrap_or_default();
    let failed = results.iter().filter(|r| !r.passed).count();
    if format == Format::Json {
        let v = serde_json::json!({"results": results, "failed": failed, "diagnostics": derived.diagnostics,
                                   "authz_tests_pending": authz_tests});
        println!("{}", serde_json::to_string_pretty(&v).unwrap());
    } else {
        for d in &derived.diagnostics {
            println!("WARN  {}: {} [{}]", d.location, d.message, d.code);
        }
        for r in &results {
            let err = r
                .error
                .as_ref()
                .map(|e| format!("  ({e})"))
                .unwrap_or_default();
            println!(
                "{}  {}: {}{err}",
                if r.passed { "PASS" } else { "FAIL" },
                r.suite,
                r.name
            );
        }
        for t in &authz_tests {
            println!("SKIP  {t}: authorization tests run through the Authorizer (M3)");
        }
        println!("{} passed, {failed} failed", results.len() - failed);
    }
    if failed == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}

fn plan(path: &std::path::Path, base: &std::path::Path, format: PlanFormat) -> ExitCode {
    use toolbox_derive::DerivedState;
    let after = match portal::load(path) {
        Ok(l) => l,
        Err(e) => return fail(e),
    };
    let base_src = match toolbox_model::load::load(base) {
        Ok(s) => s,
        Err(e) => return fail(e),
    };
    let before_m = match portal::manifest_from(&base_src.document, &base_src.manifest_file) {
        Ok(m) => m,
        Err(e) => return fail(format!("base manifest is invalid:\n{e}")),
    };
    let snapshot = match toolbox_store::load_fixtures(&after.root) {
        Ok(s) => s.snapshot(),
        Err(e) => return fail(e),
    };
    let before = toolbox_derive::FullRecompute.recompute(&before_m, &snapshot);
    let diff = toolbox_derive::plan::diff(&before_m, &before, &after.manifest, &after.derived);
    match format {
        PlanFormat::Md => print!("{}", diff.markdown()),
        PlanFormat::Json => println!("{}", serde_json::to_string_pretty(&diff).unwrap()),
    }
    ExitCode::SUCCESS
}

fn derive(path: &std::path::Path, out: Option<&std::path::Path>) -> ExitCode {
    let local = match portal::load(path) {
        Ok(l) => l,
        Err(e) => return fail(e),
    };
    let d = &local.derived;
    if let Some(dir) = out {
        for (rel, text) in d.reference_files() {
            let target = dir.join(rel);
            if let Some(parent) = target.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            if let Err(e) = std::fs::write(&target, text) {
                return fail(format!("{}: {e}", target.display()));
            }
        }
    }
    let scored: usize = d.scores.values().map(|m| m.len()).sum();
    println!(
        "snapshot {}: {} entities, {} scorecard results, {} tuples, {} orphans, {} diagnostics",
        d.snapshot,
        d.entities.len(),
        scored,
        d.tuples.len(),
        d.orphans.len(),
        d.diagnostics.len()
    );
    for diag in &d.diagnostics {
        println!("WARN  {}: {} [{}]", diag.location, diag.message, diag.code);
    }
    ExitCode::SUCCESS
}

fn validate(path: &std::path::Path, format: Format) -> ExitCode {
    let report = match toolbox_model::validate_portal(path) {
        Ok(r) => r,
        Err(e) => {
            match format {
                Format::Text => eprintln!("error: {e}"),
                Format::Json => println!(
                    "{}",
                    serde_json::json!({ "ok": false, "load_error": e.to_string() })
                ),
            }
            return ExitCode::from(2);
        }
    };
    match format {
        Format::Text => {
            for d in &report.errors {
                println!("ERROR {d}");
            }
            let included = if report.included.is_empty() {
                String::new()
            } else {
                format!(" (+ {} included file(s))", report.included.len())
            };
            println!(
                "validated {}{} + {} fixture message(s): {} error(s)",
                report.manifest,
                included,
                report.messages,
                report.errors.len()
            );
        }
        Format::Json => {
            let mut v = serde_json::to_value(&report).expect("report serialises");
            v["ok"] = report.ok().into();
            println!("{}", serde_json::to_string_pretty(&v).expect("json"));
        }
    }
    if report.ok() {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}
