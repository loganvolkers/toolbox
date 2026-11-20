//! The `toolbox` binary (docs/v1-spec.md §7.7). M0 implements `toolbox validate`.

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
}

#[derive(Clone, Copy, ValueEnum)]
enum Format {
    Text,
    Json,
}

fn main() -> ExitCode {
    match Cli::parse().command {
        Command::Validate { path, format } => validate(&path, format),
    }
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
