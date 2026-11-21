//! The `toolbox` binary (docs/v1-spec.md §7.7).

mod assess;
mod causes;
mod client;
mod install;
mod meta;
mod model;
mod portal;
mod report;
mod server;

use clap::{CommandFactory, Parser, Subcommand, ValueEnum};

/// mimalloc, not the system allocator: the recompute allocates from every thread, and glibc's
/// malloc was 38% of its profile (docs/research-log.md, *Scale profile*).
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;
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
        /// A portal directory (containing toolbox.yaml) or a manifest file.
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Output format.
        #[arg(long, value_enum, default_value_t = Format::Text)]
        format: Format,
    },
    /// Show what a manifest change would do: two recomputes over the same inputs, then a diff.
    Plan {
        /// The portal with the changed manifest (the "after").
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Compare against this manifest file (the "before"); the portal's manifest is the "after".
        #[arg(long)]
        base: PathBuf,
        /// Use the portal's fixtures (the only mode until M6 adds --remote).
        #[arg(long, default_value_t = true)]
        local: bool,
        /// Output format: Markdown for PR comments, or JSON.
        #[arg(long, value_enum, default_value_t = PlanFormat::Md)]
        format: PlanFormat,
    },
    /// Run the server: snapshot loop + REST API. Uses Postgres when --database-url (or
    /// DATABASE_URL) is set, otherwise an in-memory store (optionally preloaded with fixtures).
    Serve {
        /// A portal directory (containing toolbox.yaml) or a manifest file.
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Address to listen on.
        #[arg(long, env = "TOOLBOX_LISTEN", default_value = "127.0.0.1:8080")]
        listen: String,
        /// Postgres URL; without it the store is in memory and lost on exit.
        #[arg(long, env = "DATABASE_URL")]
        database_url: Option<String>,
        /// In-memory mode only: load the portal's fixtures at start.
        #[arg(long)]
        fixtures: bool,
    },
    /// Score a repository locally: run the portal's in-image connectors once (or use its fixtures),
    /// recompute, and print every scorecard with the evidence for failing rules. With --baseline,
    /// compare against a committed results file (a ratchet): regressions exit 1, --update records
    /// improvements, --ci also fails when the file is out of date.
    Report(report::Options),
    /// Send a portal's fixtures to a running server through the ingest API (one batch per
    /// connector, then native values), and wait for the snapshot that contains them.
    Push {
        /// A portal directory (containing toolbox.yaml) or a manifest file.
        #[arg(default_value = ".")]
        path: PathBuf,
        /// The running server.
        #[arg(long, default_value = "http://127.0.0.1:8080")]
        url: String,
    },
    /// Read one entity (fields and where each came from).
    Get {
        /// Entity id, e.g. repository:payments-api.
        id: String,
        /// Comma-separated fields to return (default: every visible field).
        #[arg(long)]
        fields: Option<String>,
        #[command(flatten)]
        remote: Remote,
    },
    /// Entities of a kind matching a CEL filter over `entity` (and `scores`).
    Query {
        /// Entity kind, e.g. Repository.
        #[arg(long)]
        kind: String,
        /// CEL over `entity` and `scores['<scorecard>']` ({level, rules}); keeps matches.
        #[arg(long)]
        filter: Option<String>,
        /// Comma-separated fields to return (default: every visible field).
        #[arg(long)]
        fields: Option<String>,
        #[command(flatten)]
        remote: Remote,
    },
    /// Find entities by name or title.
    Search {
        /// Text to find in entity ids and titles.
        query: String,
        #[command(flatten)]
        remote: Remote,
    },
    /// Neighbours of an entity.
    Graph {
        /// Entity id.
        id: String,
        /// How many hops to follow (1 to 3).
        #[arg(long, default_value_t = 1)]
        depth: u64,
        #[command(flatten)]
        remote: Remote,
    },
    /// Who owns an entity and why (CODEOWNERS evidence).
    Owner {
        /// Entity id.
        id: String,
        #[command(flatten)]
        remote: Remote,
    },
    /// Why a field has its value (--field) or why a rule passes or fails (--rule scorecard/rule).
    Explain {
        /// Entity id.
        id: String,
        /// Field to explain.
        #[arg(long)]
        field: Option<String>,
        /// Rule to explain, as <scorecard>/<rule>.
        #[arg(long)]
        rule: Option<String>,
        #[command(flatten)]
        remote: Remote,
    },
    /// Scorecard levels and rule outcomes for one entity.
    Score {
        /// Entity id.
        id: String,
        #[command(flatten)]
        remote: Remote,
    },
    /// One scorecard across every entity: level counts, failing rules and evidence (a work list).
    /// Without a name, lists the scorecards.
    Scorecard {
        /// Scorecard name.
        name: Option<String>,
        /// Only entities where this rule has --outcome (default fail).
        #[arg(long)]
        rule: Option<String>,
        /// Outcome to match for --rule: fail, pass or unknown.
        #[arg(long)]
        outcome: Option<String>,
        /// Only entities at this level.
        #[arg(long)]
        level: Option<String>,
        #[command(flatten)]
        remote: Remote,
    },
    /// One campaign's progress over existing scorecards (entities still missing the goal).
    /// Without a name, lists the campaigns.
    Campaign {
        /// Campaign name.
        name: Option<String>,
        /// Run this portal locally instead of asking a server (a repo portal: no server needed).
        #[arg(long)]
        portal: Option<PathBuf>,
        /// With --portal: the baseline whose recorded campaign starts the paydown is counted from.
        #[arg(long, requires = "portal")]
        baseline: Option<PathBuf>,
        /// With --portal: connectors not to run (comma-separated).
        #[arg(long, value_delimiter = ',', requires = "portal")]
        skip: Vec<String>,
        #[command(flatten)]
        remote: Remote,
    },
    /// Ask the connectors behind an entity to re-read it, and wait for the fresh snapshot.
    Refresh {
        /// Entity id.
        id: String,
        #[command(flatten)]
        remote: Remote,
    },
    /// Actions available on an entity.
    Actions {
        /// Entity id.
        id: String,
        #[command(flatten)]
        remote: Remote,
    },
    /// A stdio MCP server bridging to the portal's /mcp endpoint.
    Mcp {
        #[command(flatten)]
        remote: Remote,
    },
    /// Connector tools.
    Connector {
        #[command(subcommand)]
        command: ConnectorCommand,
    },
    /// Manage personal access tokens.
    Token {
        #[command(subcommand)]
        command: TokenCommand,
    },
    /// Recompute derived state from fixtures and print a summary, or write the reference-format
    /// files (expected/*.json, authz/tuples.expected.json) to --out.
    Derive {
        /// A portal directory (containing toolbox.yaml) or a manifest file.
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Directory to write the reference-format files to.
        #[arg(long)]
        out: Option<PathBuf>,
    },
    /// Print Toolbox's own interfaces (CLI commands and built-in MCP tools) as connector record
    /// messages (NDJSON), for contract snapshots and the Toolbox-for-Toolbox portal; or, with
    /// `scorecards`, a portal's scorecards and rules.
    Describe {
        #[command(subcommand)]
        what: Option<DescribeWhat>,
    },
    /// Agent assessments: brief your coding agent on an entity, check the assessment files it
    /// writes, or push them to a server. Toolbox calls no model; your agent does the judging.
    Assess {
        #[command(subcommand)]
        what: AssessWhat,
    },
    /// Download the portal's released executable connectors (`github:owner/repo[/app]@version`)
    /// for this platform, verify them against toolbox.lock (pinning them the first time), and
    /// unpack them into the cache.
    Install {
        /// A portal directory (containing toolbox.yaml) or a manifest file.
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Fail instead of pinning connectors that toolbox.lock doesn't list yet (for CI).
        #[arg(long)]
        frozen: bool,
    },
    /// Copy a package into the portal (`<portal>/packages/<name>`, without its tests) so the portal
    /// needs no network: from a directory, or a `github:` package `toolbox install` cached.
    Vendor {
        /// A package directory, or `github:owner/repo[/subdir]@version`.
        spec: String,
        /// The portal to vendor it into (a directory containing toolbox.yaml).
        #[arg(long, default_value = ".")]
        portal: PathBuf,
    },
    /// Check that every in-image connector resolves and every declared tool is at its pinned
    /// version; exit 1 when something is missing.
    Doctor {
        /// A portal directory (containing toolbox.yaml) or a manifest file.
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Print the mise `[tools]` section that installs the declared tools instead.
        #[arg(long)]
        mise: bool,
    },
}

#[derive(Subcommand)]
enum AssessWhat {
    /// What to judge an entity on: the rubric's criteria, the entity's facts, its subject hash, and
    /// a template and file path for the assessment.
    Brief {
        /// Entity id, e.g. rule:portal/user-mcp/contract-tested.
        entity: String,
        /// Rubric name (default: the only rubric for the entity's kind).
        #[arg(long)]
        rubric: Option<String>,
        /// A portal directory (containing toolbox.yaml) or a manifest file.
        #[arg(long, default_value = ".")]
        portal: PathBuf,
        /// Use the portal's fixtures instead of running its connectors.
        #[arg(long)]
        fixtures: bool,
        /// Connectors not to run (comma-separated), e.g. a slow runner.
        #[arg(long, value_delimiter = ',')]
        skip: Vec<String>,
        /// Output: md (instructions for an agent, default) or json.
        #[arg(short = 'o', long, default_value = "md")]
        output: String,
    },
    /// Validate the portal's assessments/ files and list, per rubric, which entities are assessed
    /// (fresh), assessed against an older version (stale), or not assessed (missing).
    Check {
        /// A portal directory (containing toolbox.yaml) or a manifest file.
        #[arg(default_value = ".")]
        portal: PathBuf,
        /// Use the portal's fixtures instead of running its connectors.
        #[arg(long)]
        fixtures: bool,
        /// Connectors not to run (comma-separated), e.g. a slow runner.
        #[arg(long, value_delimiter = ',')]
        skip: Vec<String>,
        /// Output: md (default) or json.
        #[arg(short = 'o', long, default_value = "md")]
        output: String,
    },
    /// Validate assessment files and push them to a running server as `assessment` records of a
    /// remote connector (for portals whose assessments live in Postgres, not in a repository).
    Submit {
        /// Assessment files (YAML or JSON).
        #[arg(required = true)]
        files: Vec<PathBuf>,
        /// The portal whose rubrics the files must match.
        #[arg(long, default_value = ".")]
        portal: PathBuf,
        /// The running server.
        #[arg(long, env = "TOOLBOX_URL", default_value = "http://127.0.0.1:8080")]
        url: String,
        /// The remote connector the records belong to (dev servers, without a token).
        #[arg(long, default_value = "assessments")]
        connector: String,
        /// That connector's token.
        #[arg(long, env = "TOOLBOX_CONNECTOR_TOKEN")]
        token: Option<String>,
    },
}

#[derive(Subcommand)]
enum DescribeWhat {
    /// Every part of portals as records — `kind`, `field`, `connector`, `scorecard`, `rule`: manifest
    /// facts, content hashes, usage, CEL size and loops, test coverage, outcomes over fixtures and
    /// (with --results) on real data.
    #[command(alias = "scorecards")]
    Portal {
        /// Portal directories to describe.
        #[arg(required = true)]
        portals: Vec<PathBuf>,
        /// A portal's results file for live outcomes, as PORTAL=FILE (repeatable).
        #[arg(long)]
        results: Vec<String>,
    },
}

/// CLI read commands and the MCP tool each one calls (they share one authorization path, D61).
const CLI_MCP_TOOLS: &[(&str, &str)] = &[
    ("get", "get"),
    ("query", "query"),
    ("search", "search"),
    ("graph", "graph"),
    ("owner", "owner_of"),
    ("explain", "explain"),
    ("score", "scorecard_status"),
    ("scorecard", "scorecard_report"),
    ("campaign", "campaign_report"),
    ("refresh", "refresh_entity"),
    ("actions", "list_actions"),
];

#[derive(clap::Args, Clone)]
struct Remote {
    /// Portal URL.
    #[arg(long, env = "TOOLBOX_URL", default_value = "http://127.0.0.1:8080")]
    url: String,
    /// Personal access token.
    #[arg(long, env = "TOOLBOX_TOKEN")]
    token: Option<String>,
    /// Dev servers only: act as this user.
    #[arg(long = "as", env = "TOOLBOX_AS")]
    as_user: Option<String>,
    /// Output: md (default), json or yaml.
    #[arg(short = 'o', long, default_value = "md")]
    output: String,
}

impl Remote {
    fn target(&self) -> client::Target {
        client::Target {
            url: self.url.trim_end_matches('/').to_owned(),
            token: self.token.clone(),
            as_user: self.as_user.clone(),
        }
    }
}

#[derive(Subcommand)]
enum ConnectorCommand {
    /// Run the conformance kit against a connector binary in in-image (stdio) mode. Its config
    /// comes from the portal manifest (with --config overriding it as JSON).
    Test {
        /// The connector's name in the manifest.
        #[arg(long)]
        name: String,
        /// Portal directory or manifest.
        #[arg(long, default_value = ".")]
        portal: PathBuf,
        /// JSON merged over the manifest's connector config (e.g. an apiUrl for a mock).
        #[arg(long)]
        config: Option<String>,
        /// The connector program and its arguments.
        #[arg(required = true, last = true)]
        program: Vec<String>,
    },
    /// Run one resync of a portal's connector and print its records (NDJSON): what a tool
    /// reported before any rule reads it. Lint records carry `command`, which reproduces their
    /// findings with the tool alone.
    Records {
        /// The connector's name in the manifest.
        #[arg(long)]
        name: String,
        /// Portal directory or manifest.
        #[arg(long, default_value = ".")]
        portal: PathBuf,
    },
}

#[derive(Subcommand)]
enum TokenCommand {
    /// Create a token for a user (the subject: a login or an email) and print it once.
    Create {
        /// The token's subject: a login or an email.
        #[arg(long)]
        user: String,
        /// A note to recognise the token by.
        #[arg(long)]
        label: Option<String>,
        /// The portal's Postgres, where token hashes are stored.
        #[arg(long, env = "DATABASE_URL")]
        database_url: String,
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

/// A reader that closed the pipe (`toolbox describe portal . | head`) ends the command quietly,
/// with the status a process killed by SIGPIPE has, instead of a panic and a backtrace.
fn quiet_on_closed_pipe() {
    let default = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let msg = info
            .payload()
            .downcast_ref::<String>()
            .map(String::as_str)
            .or_else(|| info.payload().downcast_ref::<&str>().copied())
            .unwrap_or("");
        if msg.starts_with("failed printing to stdout") && msg.contains("Broken pipe") {
            std::process::exit(141);
        }
        default(info);
    }));
}

fn main() -> ExitCode {
    quiet_on_closed_pipe();
    match Cli::parse().command {
        Command::Validate { path, format } => validate(&path, format),
        Command::Test { path, format } => test(&path, format),
        Command::Plan {
            path, base, format, ..
        } => plan(&path, &base, format),
        Command::Derive { path, out } => derive(&path, out.as_deref()),
        Command::Describe { what: None } => describe(),
        Command::Describe {
            what: Some(DescribeWhat::Portal { portals, results }),
        } => match meta::describe(&portals, &results) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => fail(e),
        },
        Command::Assess { what } => assess_command(what),
        Command::Install { path, frozen } => install::install(&path, frozen),
        Command::Doctor { path, mise } => install::doctor(&path, mise),
        Command::Vendor { spec, portal } => install::vendor(&spec, &portal),
        Command::Serve {
            path,
            listen,
            database_url,
            fixtures,
        } => server::serve(&path, &listen, database_url, fixtures),
        Command::Push { path, url } => server::push(&path, &url),
        Command::Get { id, fields, remote } => client::tool(
            &remote.target(),
            "get",
            serde_json::json!({"id": id, "fields": fields}),
            &remote.output,
        ),
        Command::Query {
            kind,
            filter,
            fields,
            remote,
        } => client::tool(
            &remote.target(),
            "query",
            serde_json::json!({"kind": kind, "filter": filter, "fields": fields}),
            &remote.output,
        ),
        Command::Search { query, remote } => client::tool(
            &remote.target(),
            "search",
            serde_json::json!({"query": query}),
            &remote.output,
        ),
        Command::Graph { id, depth, remote } => client::tool(
            &remote.target(),
            "graph",
            serde_json::json!({"id": id, "depth": depth}),
            &remote.output,
        ),
        Command::Owner { id, remote } => client::tool(
            &remote.target(),
            "owner_of",
            serde_json::json!({"id": id}),
            &remote.output,
        ),
        Command::Explain {
            id,
            field,
            rule,
            remote,
        } => client::tool(
            &remote.target(),
            "explain",
            serde_json::json!({"id": id, "field": field, "rule": rule}),
            &remote.output,
        ),
        Command::Score { id, remote } => client::tool(
            &remote.target(),
            "scorecard_status",
            serde_json::json!({"id": id}),
            &remote.output,
        ),
        Command::Scorecard {
            name,
            rule,
            outcome,
            level,
            remote,
        } => client::tool(
            &remote.target(),
            "scorecard_report",
            serde_json::json!({"scorecard": name, "rule": rule, "outcome": outcome, "level": level}),
            &remote.output,
        ),
        Command::Campaign {
            name,
            portal: Some(portal),
            baseline,
            skip,
            remote,
        } => report::campaign_local(
            &portal,
            name.as_deref(),
            baseline.as_deref(),
            &skip,
            &remote.output,
        ),
        Command::Campaign { name, remote, .. } => client::tool(
            &remote.target(),
            "campaign_report",
            serde_json::json!({"campaign": name}),
            &remote.output,
        ),
        Command::Report(o) => report::run(&o),
        Command::Refresh { id, remote } => client::tool(
            &remote.target(),
            "refresh_entity",
            serde_json::json!({"id": id}),
            &remote.output,
        ),
        Command::Actions { id, remote } => client::tool(
            &remote.target(),
            "list_actions",
            serde_json::json!({"id": id}),
            &remote.output,
        ),
        Command::Mcp { remote } => client::bridge(&remote.target()),
        Command::Connector {
            command:
                ConnectorCommand::Test {
                    name,
                    portal,
                    config,
                    program,
                },
        } => connector_test(&name, &portal, config.as_deref(), &program),
        Command::Connector {
            command: ConnectorCommand::Records { name, portal },
        } => report::connector_records(&portal, &name),
        Command::Token {
            command:
                TokenCommand::Create {
                    user,
                    label,
                    database_url,
                },
        } => server::create_token(&user, label.as_deref(), &database_url),
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
                                   "authz_tests": authz_tests});
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
        println!("{} passed, {failed} failed", results.len() - failed);
    }
    let mut authz_failed = 0;
    if !authz_tests.is_empty() {
        let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
        for t in &authz_tests {
            match rt.block_on(toolbox_authz::fgatest::run_file(&local.root.join(t))) {
                Ok(r) => {
                    for f in &r.failures {
                        println!(
                            "FAIL  {t}: {}: {} (expected {}, got {})",
                            f.test, f.assertion, f.expected, f.got
                        );
                    }
                    println!(
                        "{}  {t}: tests {}/{}, checks {}/{}, list_objects {}/{}",
                        if r.ok() { "PASS" } else { "FAIL" },
                        r.tests_passed,
                        r.tests,
                        r.checks_passed,
                        r.checks,
                        r.list_objects_passed,
                        r.list_objects
                    );
                    authz_failed += usize::from(!r.ok());
                }
                Err(e) => {
                    println!("FAIL  {t}: {e}");
                    authz_failed += 1;
                }
            }
        }
    }
    if failed == 0 && authz_failed == 0 {
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
    let before_m = match portal::manifest_from(&base_src) {
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

fn connector_test(
    name: &str,
    portal: &std::path::Path,
    config: Option<&str>,
    program: &[String],
) -> ExitCode {
    let src = match toolbox_model::load::load(portal) {
        Ok(s) => s,
        Err(e) => return fail(e),
    };
    let manifest = match portal::manifest_from(&src) {
        Ok(m) => m,
        Err(e) => return fail(e),
    };
    let Some(c) = manifest.connectors.iter().find(|c| c.name == name) else {
        return fail(format!("no connector {name} in the manifest"));
    };
    let mut cfg = c.config.clone().unwrap_or(serde_json::json!({}));
    if let Some(extra) = config {
        match serde_json::from_str::<serde_json::Value>(extra) {
            Ok(serde_json::Value::Object(m)) => cfg.as_object_mut().unwrap().extend(m),
            _ => return fail("--config must be a JSON object"),
        }
    }
    let envs = vec![("TOOLBOX_CONNECTOR_CONFIG".to_owned(), cfg.to_string())];
    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
    let results = rt.block_on(toolbox_connect::conformance::run(
        &program[0],
        &program[1..],
        &envs,
        &c.records,
    ));
    let failed = results.iter().filter(|r| !r.passed).count();
    for r in &results {
        let detail = if r.detail.is_empty() {
            String::new()
        } else {
            format!("  ({})", r.detail)
        };
        println!(
            "{}  {}{detail}",
            if r.passed { "PASS" } else { "FAIL" },
            r.name
        );
    }
    println!("{name}: {} passed, {failed} failed", results.len() - failed);
    if failed == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}

/// Dispatch `toolbox assess …`.
fn assess_command(what: AssessWhat) -> ExitCode {
    match what {
        AssessWhat::Brief {
            entity,
            rubric,
            portal,
            fixtures,
            skip,
            output,
        } => assess::brief(
            &portal,
            &entity,
            rubric.as_deref(),
            fixtures,
            &skip,
            &output,
        ),
        AssessWhat::Check {
            portal,
            fixtures,
            skip,
            output,
        } => assess::check(&portal, fixtures, &skip, &output),
        AssessWhat::Submit {
            files,
            portal,
            url,
            connector,
            token,
        } => assess::submit(&assess::Submit {
            portal,
            files,
            url,
            connector,
            token,
        }),
    }
}

/// One `command` record per CLI (sub)command and one `mcp-tool` record per built-in MCP tool.
fn describe() -> ExitCode {
    use serde_json::{Value, json};
    fn walk(cmd: &clap::Command, prefix: &str, out: &mut Vec<Value>) {
        for sub in cmd.get_subcommands().filter(|s| s.get_name() != "help") {
            let name = format!("{prefix}{}", sub.get_name()).trim().to_owned();
            if sub.has_subcommands() {
                walk(sub, &format!("{name} "), out);
                continue;
            }
            let args: Vec<Value> = sub
                .get_arguments()
                .filter(|a| !matches!(a.get_id().as_str(), "help" | "version"))
                .map(|a| {
                    json!({"name": a.get_id().as_str(), "long": a.get_long(), "positional": a.is_positional(),
                           "described": a.get_help().is_some()})
                })
                .collect();
            let undocumented: Vec<&str> = args
                .iter()
                .filter(|a| a["described"] == false)
                .filter_map(|a| a["name"].as_str())
                .collect();
            let json_output = sub
                .get_arguments()
                .any(|a| matches!(a.get_long(), Some("output" | "format")));
            let tool = CLI_MCP_TOOLS
                .iter()
                .find(|(c, _)| *c == name)
                .map(|(_, t)| *t);
            out.push(json!({"type": "record", "kind": "command", "externalId": format!("cli:{}", name.replace(' ', "-")),
                "data": {"name": name, "slug": name.replace(' ', "-"), "about": sub.get_about().map(|s| s.to_string()),
                         "args": args, "undocumentedArgs": undocumented, "structuredOutput": json_output,
                         "mcpTool": tool}}));
        }
    }
    let mut out = Vec::new();
    walk(&Cli::command(), "", &mut out);
    for t in toolbox_api::mcp::builtin_tools()
        .as_array()
        .into_iter()
        .flatten()
    {
        let props = t["inputSchema"]["properties"]
            .as_object()
            .cloned()
            .unwrap_or_default();
        let undescribed: Vec<&String> = props
            .iter()
            .filter(|(_, p)| p.get("description").is_none())
            .map(|(k, _)| k)
            .collect();
        let name = t["name"].as_str().unwrap_or_default();
        let cli: Vec<&str> = CLI_MCP_TOOLS
            .iter()
            .filter(|(_, tool)| *tool == name)
            .map(|(c, _)| *c)
            .collect();
        out.push(json!({"type": "record", "kind": "mcp-tool", "externalId": format!("mcp:{name}"),
            "data": {"name": name, "description": t["description"], "annotations": t["annotations"],
                     "properties": props.keys().collect::<Vec<_>>(), "required": t["inputSchema"]["required"],
                     "undescribedProperties": undescribed, "cliCommands": cli}}));
    }
    for line in out {
        println!("{line}");
    }
    ExitCode::SUCCESS
}
