//! Read commands and the MCP stdio bridge (v1-spec §7.7). Every read command is a call to the
//! server's MCP tool of the same purpose, so CLI, agents and the API share one implementation:
//! `-o md` prints the tool's Markdown, `-o json`/`-o yaml` its structured result.

use serde_json::{Value, json};
use std::process::ExitCode;

pub struct Target {
    pub url: String,
    pub token: Option<String>,
    /// Dev servers only: act as this user (`x-toolbox-user`).
    pub as_user: Option<String>,
}

impl Target {
    pub(crate) fn request(&self, rb: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        let rb = match &self.token {
            Some(t) => rb.bearer_auth(t),
            None => rb,
        };
        match &self.as_user {
            Some(u) => rb.header("x-toolbox-user", u),
            None => rb,
        }
    }
}

pub fn tool(target: &Target, name: &str, args: Value, output: &str) -> ExitCode {
    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
    rt.block_on(async {
        let body = json!({"jsonrpc": "2.0", "id": 1, "method": "tools/call", "params": {"name": name, "arguments": args}});
        let resp = target.request(reqwest::Client::new().post(format!("{}/mcp", target.url)).json(&body)).send().await;
        let v: Value = match resp {
            Ok(r) if r.status().is_success() => r.json().await.unwrap_or_default(),
            Ok(r) => {
                let status = r.status();
                eprintln!("error: {status} {}", r.text().await.unwrap_or_default());
                return ExitCode::from(if status.as_u16() == 401 { 2 } else { 1 });
            }
            Err(e) => {
                eprintln!("error: {}: {e}", target.url);
                return ExitCode::from(2);
            }
        };
        let r = &v["result"];
        let text = match output {
            "json" => format!("{}\n", serde_json::to_string_pretty(&r["structuredContent"]).unwrap_or_default()),
            "yaml" => serde_norway::to_string(&r["structuredContent"]).unwrap_or_default(),
            _ => format!("{}\n", r["content"][0]["text"].as_str().unwrap_or_default()),
        };
        // A reader that stops early (`| head -1`) is not an error: println! would panic on the
        // broken pipe, racing the reader, and fail `set -o pipefail` scripts at random.
        use std::io::Write as _;
        let _ = std::io::stdout().lock().write_all(text.as_bytes());
        if r["isError"] == true { ExitCode::from(1) } else { ExitCode::SUCCESS }
    })
}

/// `toolbox mcp`: a stdio MCP server that forwards each JSON-RPC message to the portal's `/mcp`,
/// for MCP clients that only speak stdio.
pub fn bridge(target: &Target) -> ExitCode {
    use std::io::{BufRead, Write};
    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
    let _telemetry = match toolbox_telemetry::init(toolbox_telemetry::TelemetryOptions {
        mode: None,
        service_name: None,
        prometheus: Some(false),
        log_format: None,
    }) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("error: telemetry: {e}");
            return ExitCode::from(2);
        }
    };
    let client = reqwest::Client::new();
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        if line.trim().is_empty() {
            continue;
        }
        let resp = rt.block_on(async {
            let rb = target.request(client.post(format!("{}/mcp", target.url)).header("content-type", "application/json").body(line.clone()));
            match rb.send().await {
                Ok(r) if r.status() == 202 => None,
                Ok(r) => Some(r.text().await.unwrap_or_default()),
                Err(e) => {
                    let id = serde_json::from_str::<Value>(&line).ok().and_then(|v| v.get("id").cloned()).unwrap_or(Value::Null);
                    Some(json!({"jsonrpc": "2.0", "id": id, "error": {"code": -32000, "message": e.to_string()}}).to_string())
                }
            }
        });
        if let Some(text) = resp {
            let _ = writeln!(stdout, "{}", text.trim());
            let _ = stdout.flush();
        }
    }
    ExitCode::SUCCESS
}
