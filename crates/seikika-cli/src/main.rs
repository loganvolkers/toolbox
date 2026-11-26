//! The `seikika` binary (docs/v1-spec.md §7.7).

mod action_tests;
mod assess;
mod causes;
mod client;
#[cfg(unix)]
mod detach;
mod find;
mod install;
mod judged;
mod meta;
mod model;
mod portal;
mod report;
mod run;
mod server;
mod tools;

use clap::{CommandFactory, Parser, Subcommand, ValueEnum};

/// mimalloc, not the system allocator: the recompute allocates from every thread, and glibc's
/// malloc was 38% of its profile (docs/research-log.md, *Scale profile*).
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser)]
#[command(
    name = "seikika",
    version,
    about = "Agent-first internal developer portal",
    after_help = "Each command has an agent skill of the same name. Task skills, such as `add-action`, \
                  `add-scorecard` and `author-connector`, cover work that spans commands."
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
        /// A portal directory (containing seikika.yaml) or a manifest file.
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Output format.
        #[arg(long, value_enum, default_value_t = Format::Text)]
        format: Format,
    },
    /// Recompute derived state from the portal's fixtures and run its expectation files
    /// (tests/*.test.yaml), and the action cases its manifest lists under `tests`. Exit code 1 if
    /// any expectation fails.
    ///
    /// An action case runs the real lifecycle (requester, approvers, an origin that rejects the
    /// push) in a scratch clone of the repository, so nothing reaches its origin. The operator
    /// skill `add-action` shows the format.
    Test {
        /// A portal directory (containing seikika.yaml) or a manifest file.
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
    ///
    /// Sign-in: on a loopback address callers say who they are (`--as user:<id>` on `seikika run
    /// --url` and the reads); elsewhere they need a token. `SEIKIKA_AUTH_MODE` overrides (`dev`,
    /// `token`, `iap-google`, `cloudflare-access`, `oidc`).
    ///
    /// Connectors: each in-image connector starts in its own Landlock domain. On a host without
    /// Landlock, `SEIKIKA_CONNECTOR_CONFINEMENT=best-effort` (the default) starts it unconfined
    /// and says so in `/readyz` and `/v1/connectors`; `required` refuses to start it.
    ///
    /// State: without a database, action runs are kept in `SEIKIKA_STATE_DIR` (default
    /// `~/.local/state/seikika`) under `served/<portal>/runs` and survive a restart. A portal with
    /// `federation.children` keeps their clones under `federation/` there, and, unless
    /// `federation.signingKey` names a variable, its approval signing key in
    /// `served/<portal>/signing-key`; the key children pin is printed at start and by `seikika run
    /// --url <server>`.
    ///
    /// Stop it with Ctrl-C or SIGTERM. `--detach` runs it in the background instead: it returns once
    /// the server answers, printing its URL, pid, log and approval key, and `--stop` stops it.
    Serve(Box<ServeArgs>),
    /// Score a repository locally: run the portal's in-image connectors once (or use its fixtures),
    /// recompute, and print every scorecard with the evidence for failing rules. With --baseline,
    /// compare against a committed results file (a ratchet): regressions exit 1, --update records
    /// improvements, --ci also fails when the file is out of date.
    Report(report::Options),
    /// Preview and run a self-service action without a server, in a repo or parent portal:
    /// `seikika run` lists the actions, `seikika run <action> --input k=v` previews (effects and a
    /// confirm token), `--confirm <token>` runs it and verifies its postcondition, and a child
    /// portal's actions are dispatched to the child (federation).
    ///
    /// A parent portal (`federation.children`) holds approvals in its checkout: `--as <person>`
    /// requests, `--approve <run>` or `--reject <run>` decides, and the last approval signs the
    /// grant with the key `seikika run` lists and runs the child. The operator skill `add-action`
    /// walks through declaring, testing and offering an action.
    Run(Box<run::Options>),
    /// Send a portal's fixtures to a running server through the ingest API (one batch per
    /// connector, then native values), and wait for the snapshot that contains them.
    Push {
        /// A portal directory (containing seikika.yaml) or a manifest file.
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
        /// Only follow edges with this relation name, e.g. dependsOn (default: every relation).
        #[arg(long)]
        relation: Option<String>,
        /// Follow outgoing edges, incoming edges or both.
        #[arg(long, default_value = "both", value_parser = ["in", "out", "both"])]
        direction: String,
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
        /// A portal directory (containing seikika.yaml) or a manifest file.
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Directory to write the reference-format files to.
        #[arg(long)]
        out: Option<PathBuf>,
    },
    /// Print Seikika's own interfaces (CLI commands and built-in MCP tools) as connector record
    /// messages (NDJSON), for contract snapshots and the Seikika-for-Seikika portal; or, with
    /// `scorecards`, a portal's scorecards and rules.
    Describe {
        #[command(subcommand)]
        what: Option<DescribeWhat>,
    },
    /// Agent assessments: brief your coding agent on an entity, check the assessment files it
    /// writes, or push them to a server. Seikika calls no model; your agent does the judging.
    Assess {
        #[command(subcommand)]
        what: AssessWhat,
    },
    /// Download the portal's released executable connectors (`github:owner/repo[/app]@version`)
    /// for this platform, verify them against seikika.lock (pinning them the first time), and
    /// unpack them into the cache.
    Install {
        /// A portal directory (containing seikika.yaml) or a manifest file.
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Fail instead of pinning connectors that seikika.lock doesn't list yet (for CI).
        #[arg(long)]
        frozen: bool,
    },
    /// Copy a package into the portal (`<portal>/packages/<name>`, without its tests) so the portal
    /// needs no network: from a directory, or a `github:` package `seikika install` cached.
    Vendor {
        /// A package directory, or `github:owner/repo[/subdir]@version`.
        spec: String,
        /// The portal to vendor it into (a directory containing seikika.yaml).
        #[arg(long, default_value = ".")]
        portal: PathBuf,
    },
    /// The portal's tools: install them at their pinned versions with an installer you choose,
    /// or print an installer's input. Seikika doesn't download tools itself.
    Tools {
        #[command(subcommand)]
        command: ToolsCommand,
    },
    /// Check that every in-image connector resolves, whether this host would confine it
    /// (Landlock), and that every declared tool is at its pinned version; exit 1 when something
    /// is missing.
    Doctor {
        /// A portal directory (containing seikika.yaml) or a manifest file.
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
        /// A portal directory (containing seikika.yaml) or a manifest file.
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
        /// A portal directory (containing seikika.yaml) or a manifest file.
        #[arg(default_value = ".")]
        portal: PathBuf,
        /// Use the portal's fixtures instead of running its connectors.
        #[arg(long)]
        fixtures: bool,
        /// Connectors not to run (comma-separated), e.g. a slow runner.
        #[arg(long, value_delimiter = ',')]
        skip: Vec<String>,
        /// Fail when a recorded assessment or review is stale.
        #[arg(long)]
        require_fresh: bool,
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
        #[arg(long, env = "SEIKIKA_URL", default_value = "http://127.0.0.1:8080")]
        url: String,
        /// The remote connector the records belong to (dev servers, without a token).
        #[arg(long, default_value = "assessments")]
        connector: String,
        /// That connector's token.
        #[arg(long, env = "SEIKIKA_CONNECTOR_TOKEN")]
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
    #[arg(long, env = "SEIKIKA_URL", default_value = "http://127.0.0.1:8080")]
    url: String,
    /// Personal access token.
    #[arg(long, env = "SEIKIKA_TOKEN")]
    token: Option<String>,
    /// Dev servers only: act as this user.
    #[arg(long = "as", env = "SEIKIKA_AS")]
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
    /// Drive a connector binary over stdio: one `resync` of the named kinds, print each `record`
    /// as NDJSON (`kind`, `externalId`, `data`). No portal required — for local dumps while
    /// authoring. Pass config with --config (JSON) or rely on SEIKIKA_CONNECTOR_CONFIG in the
    /// environment. Use `--format catalog` to emit harness-shaped `{kind,id,data}` lines (catalog
    /// `id` = `externalId` with a leading `{kind}:` stripped) and `--out` to write a file.
    Drive {
        /// Record kinds to resync (comma-separated), e.g. project,issue.
        #[arg(long, value_delimiter = ',')]
        kinds: Vec<String>,
        /// JSON object set as SEIKIKA_CONNECTOR_CONFIG for the child.
        #[arg(long)]
        config: Option<String>,
        /// Output shape: `records` (default) or `catalog` (`kind`/`id`/`data`, sorted).
        #[arg(long, default_value = "records", value_parser = ["records", "catalog"])]
        format: String,
        /// Write NDJSON to this path instead of stdout (parent dirs must exist).
        #[arg(long)]
        out: Option<PathBuf>,
        /// The connector program and its arguments.
        #[arg(required = true, last = true)]
        program: Vec<String>,
    },
}

#[derive(Subcommand)]
enum ToolsCommand {
    /// Install every tool the portal pins that isn't already on the PATH at its version, by
    /// running an installer: mise, uv, pipx or pip (auto: the first available that the tool
    /// declares an id for). Then run `seikika doctor`.
    Install {
        /// A portal directory (containing seikika.yaml) or a manifest file.
        #[arg(default_value = ".")]
        path: PathBuf,
        /// The installer: auto, mise, uv, pipx or pip (aqua installs from `tools config --for aqua`).
        #[arg(long, default_value = "auto")]
        via: String,
        /// Print the install commands without running them.
        #[arg(long)]
        dry_run: bool,
    },
    /// Print an installer's input from the portal's pins: a mise `[tools]` table, an aqua.yaml
    /// package list, or PyPI requirements (for uv, pipx or pip).
    Config {
        /// A portal directory (containing seikika.yaml) or a manifest file.
        #[arg(default_value = ".")]
        path: PathBuf,
        /// mise, aqua or pypi.
        #[arg(long = "for")]
        installer: String,
    },
    /// List the tools a repository already uses (config files, CI and hook commands, package
    /// manifests), whether each is in the portal's box, and the slot that would carry it. Evidence
    /// for an agent to check (skill find-tools), not a verdict.
    Find {
        /// The repository root.
        #[arg(default_value = ".")]
        path: PathBuf,
        /// The portal to compare with (default: the root's seikika.yaml, else portal/seikika.yaml).
        #[arg(long)]
        portal: Option<PathBuf>,
        /// Output format.
        #[arg(long, value_enum, default_value_t = Format::Text)]
        format: Format,
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

/// A reader that closed the pipe (`seikika describe portal . | head`) ends the command quietly,
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

/// The read commands: each is one MCP tool call against a server or a local portal.
fn read_call(c: &Command) -> Option<(&Remote, &'static str, serde_json::Value)> {
    use serde_json::json;
    Some(match c {
        Command::Get { id, fields, remote } => (remote, "get", json!({"id": id, "fields": fields})),
        Command::Query {
            kind,
            filter,
            fields,
            remote,
        } => (
            remote,
            "query",
            json!({"kind": kind, "filter": filter, "fields": fields}),
        ),
        Command::Search { query, remote } => (remote, "search", json!({"query": query})),
        Command::Graph {
            id,
            depth,
            relation,
            direction,
            remote,
        } => (
            remote,
            "graph",
            json!({"id": id, "depth": depth, "relation": relation, "direction": direction}),
        ),
        Command::Owner { id, remote } => (remote, "owner_of", json!({"id": id})),
        Command::Explain {
            id,
            field,
            rule,
            remote,
        } => (
            remote,
            "explain",
            json!({"id": id, "field": field, "rule": rule}),
        ),
        Command::Score { id, remote } => (remote, "scorecard_status", json!({"id": id})),
        Command::Scorecard {
            name,
            rule,
            outcome,
            level,
            remote,
        } => (
            remote,
            "scorecard_report",
            json!({"scorecard": name, "rule": rule, "outcome": outcome, "level": level}),
        ),
        Command::Campaign {
            name,
            portal: None,
            remote,
            ..
        } => (remote, "campaign_report", json!({"campaign": name})),
        Command::Refresh { id, remote } => (remote, "refresh_entity", json!({"id": id})),
        Command::Actions { id, remote } => (remote, "list_actions", json!({"id": id})),
        _ => return None,
    })
}

/// `seikika tools …`: find, pin and install the box's tools.
fn tools_command(command: ToolsCommand) -> ExitCode {
    match command {
        ToolsCommand::Install { path, via, dry_run } => tools::install(&path, &via, dry_run),
        ToolsCommand::Config { path, installer } => tools::config(&path, &installer),
        ToolsCommand::Find {
            path,
            portal,
            format,
        } => find::find(&path, portal.as_deref(), matches!(format, Format::Json)),
    }
}

fn campaign_command(
    portal: &std::path::Path,
    name: Option<String>,
    baseline: Option<std::path::PathBuf>,
    skip: &[String],
    remote: &Remote,
) -> ExitCode {
    report::campaign_local(
        portal,
        name.as_deref(),
        baseline.as_deref(),
        skip,
        &remote.output,
    )
}

fn serve_command(o: ServeArgs) -> ExitCode {
    let listen = server::listen_addr(o.listen.as_deref(), std::env::var("PORT").ok().as_deref());
    #[cfg(unix)]
    if o.stop {
        return detach::stop(&o.path);
    } else if o.detach {
        return detach::start(&o.path, &listen, o.database_url, o.fixtures);
    }
    #[cfg(not(unix))]
    if o.stop || o.detach {
        return fail("--detach and --stop need a Unix host; use a service manager");
    }
    server::serve(&o.path, &listen, o.database_url, o.fixtures)
}

#[derive(clap::Args)]
struct ServeArgs {
    /// A portal directory (containing seikika.yaml) or a manifest file.
    #[arg(default_value = ".")]
    path: PathBuf,
    /// Address to listen on. `SEIKIKA_LISTEN` and `--listen` win. Otherwise, when
    /// `PORT` is set, the server binds `0.0.0.0:$PORT` (the Cloud Run contract).
    /// With neither, it binds `127.0.0.1:8080`.
    #[arg(long, env = "SEIKIKA_LISTEN")]
    listen: Option<String>,
    /// Postgres URL; without it the store is in memory and lost on exit.
    #[arg(long, env = "DATABASE_URL")]
    database_url: Option<String>,
    /// In-memory mode only: load the portal's fixtures at start.
    #[arg(long)]
    fixtures: bool,
    /// Start in the background and return once it answers. The pid and log are kept in
    /// `served/<portal>/` under the state directory, so it survives the shell that started it.
    #[arg(long, conflicts_with = "stop")]
    detach: bool,
    /// Stop the server `--detach` started for this portal, and wait for it to exit.
    #[arg(long)]
    stop: bool,
}

fn main() -> ExitCode {
    quiet_on_closed_pipe();
    let command = Cli::parse().command;
    // Telemetry for serve/mcp is installed in those entrypoints (Tokio runtime first).
    // Other subcommands stay quiet so stdout JSON (describe, report) stays clean.
    if let Some((remote, tool, args)) = read_call(&command) {
        return client::tool(&remote.target(), tool, args, &remote.output);
    }
    match command {
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
        Command::Tools { command } => tools_command(command),
        Command::Vendor { spec, portal } => install::vendor(&spec, &portal),
        Command::Serve(o) => serve_command(*o),
        Command::Push { path, url } => server::push(&path, &url),
        Command::Campaign {
            name,
            portal: Some(portal),
            baseline,
            skip,
            remote,
        } => campaign_command(&portal, name, baseline, &skip, &remote),
        Command::Report(o) => report::run(&o),
        Command::Run(o) => run::run(&o),
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
        Command::Connector {
            command:
                ConnectorCommand::Drive {
                    kinds,
                    config,
                    format,
                    out,
                    program,
                },
        } => connector_drive(&kinds, config.as_deref(), &format, out.as_deref(), &program),
        Command::Token {
            command:
                TokenCommand::Create {
                    user,
                    label,
                    database_url,
                },
        } => server::create_token(&user, label.as_deref(), &database_url),
        _ => unreachable!("read commands are handled by read_call"),
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
    let mut results =
        match seikika_derive::testing::run(&local.root, &local.manifest.tests, derived.clone()) {
            Ok(r) => r,
            Err(e) => return fail(e),
        };
    match action_tests::run(path, &local.root, &local.manifest.tests) {
        Ok(r) => results.extend(r),
        Err(e) => return fail(e),
    }
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
    let authz_failed = authz_failures(&local.root, &authz_tests);
    if failed == 0 && authz_failed == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}

/// Run the `.fga.yaml` files through the Authorizer, printing each; how many failed.
fn authz_failures(root: &std::path::Path, tests: &[String]) -> usize {
    let mut authz_failed = 0;
    if !tests.is_empty() {
        let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
        for t in tests {
            match rt.block_on(seikika_authz::fgatest::run_file(&root.join(t))) {
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
    authz_failed
}

fn plan(path: &std::path::Path, base: &std::path::Path, format: PlanFormat) -> ExitCode {
    use seikika_derive::DerivedState;
    let after = match portal::load(path) {
        Ok(l) => l,
        Err(e) => return fail(e),
    };
    let base_src = match seikika_model::load::load(base) {
        Ok(s) => s,
        Err(e) => return fail(e),
    };
    let before_m = match portal::manifest_from(&base_src) {
        Ok(m) => m,
        Err(e) => return fail(format!("base manifest is invalid:\n{e}")),
    };
    let snapshot = match seikika_store::load_fixtures(&after.root) {
        Ok(s) => s.snapshot(),
        Err(e) => return fail(e),
    };
    let before = seikika_derive::FullRecompute.recompute(&before_m, &snapshot);
    let diff = seikika_derive::plan::diff(&before_m, &before, &after.manifest, &after.derived);
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
    let report = match seikika_model::validate_portal(path) {
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

/// Catalog entity id from a connector `externalId`: strip a leading `{kind}:` when present.
fn catalog_id_from_external(kind: &str, external_id: &str) -> String {
    let prefix = format!("{kind}:");
    external_id
        .strip_prefix(&prefix)
        .unwrap_or(external_id)
        .to_owned()
}

/// Prefer `{project}:{name}` from record data when the stripped externalId is a bare
/// numeric token (common mistake for SaaS components whose upstream id is numeric).
fn catalog_id(kind: &str, external_id: &str, data: &serde_json::Value) -> String {
    let stripped = catalog_id_from_external(kind, external_id);
    let bare_numeric = !stripped.is_empty() && stripped.bytes().all(|b| b.is_ascii_digit());
    if bare_numeric
        && let (Some(project), Some(name)) = (data["project"].as_str(), data["name"].as_str())
        && !project.is_empty()
        && !name.is_empty()
    {
        return format!("{project}:{name}");
    }
    stripped
}

fn drive_output_lines(format: &str, msgs: &[serde_json::Value]) -> Result<Vec<String>, String> {
    match format {
        "records" => Ok(msgs
            .iter()
            .filter(|m| m["type"] == "record")
            .map(|m| {
                serde_json::json!({
                    "kind": m["kind"],
                    "externalId": m["externalId"],
                    "data": m["data"],
                })
                .to_string()
            })
            .collect()),
        "catalog" => {
            let mut ents: Vec<serde_json::Value> = msgs
                .iter()
                .filter(|m| m["type"] == "record")
                .map(|m| {
                    let kind = m["kind"].as_str().unwrap_or("");
                    let ext = m["externalId"].as_str().unwrap_or("");
                    serde_json::json!({
                        "kind": kind,
                        "id": catalog_id(kind, ext, &m["data"]),
                        "data": m["data"].clone(),
                    })
                })
                .collect();
            ents.sort_by(|a, b| {
                (
                    a["kind"].as_str().unwrap_or(""),
                    a["id"].as_str().unwrap_or(""),
                )
                    .cmp(&(
                        b["kind"].as_str().unwrap_or(""),
                        b["id"].as_str().unwrap_or(""),
                    ))
            });
            Ok(ents.into_iter().map(|e| e.to_string()).collect())
        }
        other => Err(format!("unknown --format {other}")),
    }
}

fn connector_drive(
    kinds: &[String],
    config: Option<&str>,
    format: &str,
    out: Option<&std::path::Path>,
    program: &[String],
) -> ExitCode {
    if kinds.is_empty() {
        return fail("--kinds needs at least one record kind");
    }
    if program.is_empty() {
        return fail("pass the connector program after --");
    }
    let mut envs = Vec::new();
    if let Some(cfg) = config {
        if serde_json::from_str::<serde_json::Value>(cfg).is_err() {
            return fail("--config must be JSON");
        }
        envs.push(("SEIKIKA_CONNECTOR_CONFIG".to_owned(), cfg.to_owned()));
    }
    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
    match rt.block_on(seikika_connect::conformance::drive_resync(
        &program[0],
        &program[1..],
        &envs,
        kinds,
    )) {
        Ok(msgs) => {
            let lines = match drive_output_lines(format, &msgs) {
                Ok(lines) => lines,
                Err(e) => return fail(e),
            };
            let body = lines.join("\n") + if lines.is_empty() { "" } else { "\n" };
            if let Some(path) = out {
                if let Err(e) = std::fs::write(path, &body) {
                    return fail(format!("cannot write {}: {e}", path.display()));
                }
            } else {
                print!("{body}");
            }
            let ok = msgs
                .iter()
                .any(|m| m["type"] == "complete" && m["status"] == "succeeded");
            if ok {
                ExitCode::SUCCESS
            } else {
                fail("resync did not complete successfully")
            }
        }
        Err(e) => fail(e),
    }
}

fn connector_test(
    name: &str,
    portal: &std::path::Path,
    config: Option<&str>,
    program: &[String],
) -> ExitCode {
    let src = match seikika_model::load::load(portal) {
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
    // `${env:…}` and `${secret:…}` resolve as `serve` resolves them; when they can't (a kit run
    // against a mock, say), the connector gets the references as written.
    let written = c.config.clone().unwrap_or(serde_json::json!({}));
    let mut cfg = seikika_api::connectors::resolve_config(name, &written);
    if cfg.as_object().is_some_and(|m| m.is_empty()) {
        cfg = written;
    }
    if let Some(extra) = config {
        match serde_json::from_str::<serde_json::Value>(extra) {
            Ok(serde_json::Value::Object(m)) => cfg.as_object_mut().unwrap().extend(m),
            _ => return fail("--config must be a JSON object"),
        }
    }
    let envs = vec![("SEIKIKA_CONNECTOR_CONFIG".to_owned(), cfg.to_string())];
    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
    let results = rt.block_on(seikika_connect::conformance::run(
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

/// Dispatch `seikika assess …`.
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
            require_fresh,
        } => assess::check(&portal, fixtures, &skip, &output, require_fresh),
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
    for t in seikika_api::mcp::builtin_tools()
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

#[cfg(test)]
mod drive_catalog_tests {
    use super::{catalog_id, catalog_id_from_external};
    use serde_json::json;

    #[test]
    fn strips_kind_prefix() {
        assert_eq!(
            catalog_id_from_external("project", "project:ALPHA"),
            "ALPHA"
        );
        assert_eq!(
            catalog_id_from_external("issue", "issue:ALPHA-1"),
            "ALPHA-1"
        );
        assert_eq!(
            catalog_id_from_external("component", "component:ALPHA:api"),
            "ALPHA:api"
        );
    }

    #[test]
    fn leaves_bare_external_id() {
        assert_eq!(catalog_id_from_external("project", "ALPHA"), "ALPHA");
    }

    #[test]
    fn prefers_project_name_when_external_id_is_numeric() {
        let data = json!({"id": "1", "name": "api", "project": "ALPHA"});
        assert_eq!(catalog_id("component", "component:1", &data), "ALPHA:api");
        assert_eq!(
            catalog_id("component", "component:ALPHA:api", &data),
            "ALPHA:api"
        );
    }
}

#[cfg(test)]
mod read_call_tests {
    use super::{Cli, read_call};
    use clap::Parser;
    use serde_json::json;

    #[test]
    fn graph_passes_relation_and_direction() {
        let cli = Cli::try_parse_from([
            "seikika",
            "graph",
            "crate:codex-tools",
            "--relation",
            "dependsOn",
            "--direction",
            "in",
            "--depth",
            "3",
        ])
        .unwrap();
        let (_, tool, args) = read_call(&cli.command).unwrap();
        assert_eq!(tool, "graph");
        assert_eq!(
            args,
            json!({"id": "crate:codex-tools", "depth": 3, "relation": "dependsOn", "direction": "in"})
        );
    }

    #[test]
    fn graph_defaults_to_every_relation_both_ways() {
        let cli = Cli::try_parse_from(["seikika", "graph", "crate:x"]).unwrap();
        let (_, _, args) = read_call(&cli.command).unwrap();
        assert_eq!(
            args,
            json!({"id": "crate:x", "depth": 1, "relation": null, "direction": "both"})
        );
    }

    #[test]
    fn graph_rejects_an_unknown_direction() {
        assert!(Cli::try_parse_from(["seikika", "graph", "crate:x", "--direction", "up"]).is_err());
    }
}
