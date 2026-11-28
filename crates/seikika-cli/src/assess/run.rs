//! Content digests and collector coverage for a local assessment check.

use super::Check;
use seikika_model::Manifest;
use seikika_store::InputSnapshot;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::io::Read as _;
use std::path::Path;
use std::process::Command;
use std::time::Instant;

pub(super) struct Record<'a> {
    pub options: &'a Check,
    pub root: &'a Path,
    pub manifest: &'a Manifest,
    pub inputs: &'a InputSnapshot,
    pub files_hash: String,
    pub output: &'a str,
    pub code: u8,
    pub started: Instant,
}

fn hex(hash: impl AsRef<[u8]>) -> String {
    hash.as_ref()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

pub(super) fn json_digest(value: &impl serde::Serialize) -> String {
    let bytes = serde_json::to_vec(value).expect("assessment run input serializes");
    hex(Sha256::digest(bytes))
}

fn binary_digest() -> Option<String> {
    let path = std::env::current_exe().ok()?;
    // On hosts with coreutils, hashing a large debug build is much cheaper in sha256sum.
    if let Ok(output) = Command::new("sha256sum").arg("--").arg(&path).output()
        && output.status.success()
        && let Ok(text) = std::str::from_utf8(&output.stdout)
        && let Some(digest) = text.split_ascii_whitespace().next()
        && digest.len() == 64
        && digest.bytes().all(|b| b.is_ascii_hexdigit())
    {
        return Some(digest.to_ascii_lowercase());
    }
    let mut file = std::fs::File::open(path).ok()?;
    let mut hash = Sha256::new();
    let mut buffer = [0; 16_384];
    loop {
        let count = file.read(&mut buffer).ok()?;
        if count == 0 {
            return Some(hex(hash.finalize()));
        }
        hash.update(&buffer[..count]);
    }
}

fn capture(root: &Path, args: &[&str]) -> Option<String> {
    let result = Command::new("git")
        .arg("--no-optional-locks")
        .args(args)
        .current_dir(root)
        .output()
        .ok()?;
    result
        .status
        .success()
        .then(|| String::from_utf8_lossy(&result.stdout).trim().to_owned())
}

fn common(options: &Check, started: Instant, code: u8) -> Value {
    json!({
        "schemaVersion": 1,
        "seikikaVersion": env!("CARGO_PKG_VERSION"),
        "mode": if options.fixtures {"fixtures"} else {"live"},
        "skippedConnectors": options.skip,
        "scope": {
            "campaign": options.campaign, "rubric": options.rubric, "entities": options.entities,
            "queue": options.queue, "action": options.action, "limit": options.limit,
            "withBriefs": options.with_briefs, "reviewer": options.reviewer,
            "requireFresh": options.require_fresh, "output": options.output,
        },
        "elapsedMs": started.elapsed().as_millis(),
        "exitCode": code,
    })
}

fn save(path: &Path, doc: &Value) -> Result<(), String> {
    std::fs::write(path, format!("{}\n", seikika_derive::pyjson::pretty(doc)))
        .map_err(|e| format!("write assessment run manifest {}: {e}", path.display()))
}

pub(super) fn write_failure(
    options: &Check,
    started: Instant,
    stage: &str,
    connector: Option<&str>,
) -> Result<(), String> {
    let Some(path) = &options.run_manifest else {
        return Ok(());
    };
    let mut doc = common(options, started, 2);
    doc["status"] = json!("error");
    doc["stage"] = json!(stage);
    if let Some(connector) = connector {
        doc["connectors"] = json!({(connector): {"status": "error"}});
    }
    save(path, &doc)
}

fn connector_coverage(
    options: &Check,
    manifest: &Manifest,
    inputs: &InputSnapshot,
) -> (Value, Value) {
    let mut counts = BTreeMap::new();
    for record in &inputs.records {
        *counts.entry(record.connector.as_str()).or_insert(0usize) += 1;
    }
    let coverage: BTreeMap<_, _> = manifest
        .connectors
        .iter()
        .map(|connector| {
            let name = connector.name.as_str();
            let count = counts.remove(name).unwrap_or_default();
            let status = if options.fixtures {
                if count > 0 {
                    "fixture"
                } else {
                    "no-fixture-records"
                }
            } else if options.skip.contains(&connector.name) {
                "skipped"
            } else if connector.runtime != "in-image" {
                "remote"
            } else {
                "collected"
            };
            (
                name,
                json!({"status": status, "records": count, "package": connector.package}),
            )
        })
        .collect();
    (json!(coverage), json!(counts))
}

fn tool_versions<'a>(options: &Check, manifest: &'a Manifest) -> BTreeMap<&'a String, Value> {
    manifest
        .tools
        .iter()
        .map(|(name, tool)| {
            let observed = if options.fixtures {
                None
            } else {
                let default = ["--version".to_owned()];
                crate::install::tool_version(name, tool.version_args.as_deref().unwrap_or(&default))
            };
            (
                name,
                json!({"expected": tool.version, "observed": observed}),
            )
        })
        .collect()
}

pub(super) fn write(path: &Path, run: Record<'_>) -> Result<(), String> {
    let Record {
        options,
        root,
        manifest,
        inputs,
        files_hash,
        output,
        code,
        started,
    } = run;
    let (coverage, extra_records) = connector_coverage(options, manifest, inputs);
    let rubrics: BTreeMap<_, _> = manifest
        .rubrics
        .iter()
        .map(|r| (&r.name, super::assess::rubric_hash(r)))
        .collect();
    let mut doc = common(options, started, code);
    doc["status"] = json!("checked");
    doc["portal"] = json!(manifest.portal.name);
    doc["binarySha256"] = json!(binary_digest());
    doc["repository"] = json!({
        "commit": capture(root, &["rev-parse", "HEAD"]),
        "tree": capture(root, &["rev-parse", "HEAD^{tree}"]),
        "dirty": capture(root, &["status", "--porcelain"]).map(|s| !s.is_empty()),
    });
    doc["manifestSha256"] = json!(json_digest(manifest));
    doc["inputsSha256"] = json!(json_digest(&(&inputs.records, &inputs.native)));
    doc["parsedAssessmentFilesSha256"] = json!(files_hash);
    doc["outputSha256"] = json!(hex(Sha256::digest(output.as_bytes())));
    doc["connectors"] = coverage;
    doc["additionalInputRecords"] = extra_records;
    doc["rubricHashes"] = json!(rubrics);
    doc["tools"] = json!(tool_versions(options, manifest));
    doc["elapsedMs"] = json!(started.elapsed().as_millis());
    save(path, &doc)
}
