//! Comparative scorecards (D01M857ZB468ZM2YS6GNSEG6YB9), end to end through the binary: one
//! scorecard scores a subject and its peers, and the report says, rule by rule, where the subject
//! stands against each peer.

use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command;

const MANIFEST: &str = r#"
apiVersion: toolbox.dev/v1alpha1
portal: {name: demo, title: Demo, owner: group:maintainers, toolbox: "0.1"}
connectors:
  - {name: products, package: builtin:git-files, runtime: remote, records: [product], freshness: {product: 1h}}
kinds:
  - name: Product
    description: A product being compared.
    identityFrom: [connector:products]
    fields:
      title: {type: string, description: Its name.}
      coldStartS: {type: number, description: Seconds from start to ready.}
      rssMb: {type: number, description: Resident memory when idle.}
      secureDefault: {type: boolean, description: Whether it refuses anonymous reads by default.}
      federates: {type: boolean, description: Whether it federates catalogs.}
    sources: {default: [connector:products]}
    map:
      - from: products/product
        id: "'product:' + record.name"
        set: "{'title': record.title, 'coldStartS': record.coldStartS, 'rssMb': record.rssMb, 'secureDefault': record.secureDefault, 'federates': record.federates}"
scorecards:
  - name: against-rival
    kind: Product
    owner: group:maintainers
    levels: [bronze]
    compare: {subject: product:ours, peers: [product:rival]}
    rules:
      - {name: cold-start, level: bronze, check: "entity.coldStartS <= 5.0", value: "entity.coldStartS", better: lower, unit: s, fix: "Start faster."}
      - {name: memory, level: bronze, check: "entity.rssMb <= 1000.0", value: "entity.rssMb", better: lower, unit: MB, tolerance: 0.1, fix: "Use less memory."}
      - {name: secure-default, level: bronze, check: "entity.secureDefault", fix: "Refuse anonymous reads."}
      - {name: federation, level: bronze, check: "entity.federates", fix: "Federate."}
"#;

fn portal(dir: &Path, products: &[Value]) {
    std::fs::create_dir_all(dir.join("fixtures")).unwrap();
    std::fs::write(dir.join("toolbox.yaml"), MANIFEST).unwrap();
    let lines: Vec<String> = products
        .iter()
        .map(|p| {
            serde_json::json!({"type": "record", "kind": "product",
                "externalId": format!("product:{}", p["name"].as_str().unwrap()), "data": p})
            .to_string()
        })
        .collect();
    std::fs::write(dir.join("fixtures/products.ndjson"), lines.join("\n")).unwrap();
}

fn report(dir: &Path, output: &str) -> String {
    let o = Command::new(env!("CARGO_BIN_EXE_toolbox"))
        .arg("report")
        .arg(dir)
        .args(["--fixtures", "-o", output])
        .output()
        .expect("toolbox runs");
    String::from_utf8(o.stdout).unwrap()
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("toolbox-compare-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

fn product(name: &str, cold: f64, rss: f64, secure: bool, federates: bool) -> Value {
    serde_json::json!({"name": name, "title": name.to_uppercase(), "coldStartS": cold, "rssMb": rss,
        "secureDefault": secure, "federates": federates})
}

#[test]
fn a_comparison_says_where_the_subject_stands_rule_by_rule() {
    let dir = scratch("standing");
    portal(
        &dir,
        &[
            product("ours", 0.7, 34.0, true, false),
            product("rival", 3.7, 36.0, false, false),
        ],
    );
    let v: Value = serde_json::from_str(&report(&dir, "json")).unwrap();
    let c = &v["comparisons"]["against-rival"];
    assert_eq!(c["subject"], "product:ours");
    let standing = |rule: &str| c["rules"][rule]["product:rival"].clone();
    // Lower is better and both have a value: the values decide, though both pass.
    assert_eq!(standing("cold-start"), "ahead");
    // Within 10% of each other is parity.
    assert_eq!(standing("memory"), "parity");
    // No value: the outcomes decide.
    assert_eq!(standing("secure-default"), "ahead");
    // Both fail: parity.
    assert_eq!(standing("federation"), "parity");
    assert_eq!(c["verdict"]["product:rival"], "ahead");
    assert_eq!(
        v["scores"]["against-rival"]["product:rival"]["values"]["cold-start"],
        3.7
    );

    let md = report(&dir, "md");
    assert!(
        md.contains("## against-rival  (OURS: ahead against RIVAL)"),
        "{md}"
    );
    assert!(
        md.contains("| cold-start | pass, 0.7 s | pass, 3.7 s | ahead |"),
        "{md}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn one_rule_behind_makes_the_subject_behind() {
    let dir = scratch("behind");
    portal(
        &dir,
        &[
            product("ours", 0.7, 34.0, true, false),
            product("rival", 3.7, 36.0, false, true),
        ],
    );
    let v: Value = serde_json::from_str(&report(&dir, "json")).unwrap();
    let c = &v["comparisons"]["against-rival"];
    assert_eq!(c["rules"]["federation"]["product:rival"], "behind");
    assert_eq!(c["verdict"]["product:rival"], "behind");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_majority_comparison_counts_rules_ahead_against_rules_behind() {
    let dir = scratch("majority");
    portal(
        &dir,
        &[
            product("ours", 0.7, 34.0, true, false),
            product("rival", 3.7, 36.0, false, true),
        ],
    );
    let majority = MANIFEST.replace(
        "peers: [product:rival]}",
        "peers: [product:rival], verdict: majority}",
    );
    std::fs::write(dir.join("toolbox.yaml"), &majority).unwrap();
    let v: Value = serde_json::from_str(&report(&dir, "json")).unwrap();
    let c = &v["comparisons"]["against-rival"];
    // Ahead on cold start and secure default, behind on federation, parity on memory.
    assert_eq!(c["rules"]["federation"]["product:rival"], "behind");
    assert_eq!(c["verdictRule"], "majority");
    assert_eq!(c["verdict"]["product:rival"], "ahead");
    let md = report(&dir, "md");
    assert!(
        md.contains("(OURS: ahead against RIVAL, by the majority of rules)"),
        "{md}"
    );

    // Without values, both pass cold start: one rule ahead, one behind, so parity.
    std::fs::write(
        dir.join("toolbox.yaml"),
        majority.replace(
            "entity.coldStartS <= 5.0\", value: \"entity.coldStartS\", better: lower, unit: s,",
            "entity.coldStartS <= 5.0\",",
        ),
    )
    .unwrap();
    let v: Value = serde_json::from_str(&report(&dir, "json")).unwrap();
    let c = &v["comparisons"]["against-rival"];
    assert_eq!(c["rules"]["federation"]["product:rival"], "behind");
    assert_eq!(c["rules"]["memory"]["product:rival"], "parity");
    assert_eq!(c["verdict"]["product:rival"], "parity", "{c}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_missing_peer_is_unknown_not_parity() {
    let dir = scratch("missing");
    portal(&dir, &[product("ours", 0.7, 34.0, true, true)]);
    let v: Value = serde_json::from_str(&report(&dir, "json")).unwrap();
    let c = &v["comparisons"]["against-rival"];
    assert_eq!(c["rules"]["cold-start"]["product:rival"], "unknown");
    assert_eq!(c["verdict"]["product:rival"], "unknown");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn validate_refuses_a_peer_that_is_the_subject() {
    let dir = scratch("validate");
    portal(&dir, &[]);
    let bad = MANIFEST.replace("peers: [product:rival]", "peers: [product:ours]");
    std::fs::write(dir.join("toolbox.yaml"), bad).unwrap();
    let o = Command::new(env!("CARGO_BIN_EXE_toolbox"))
        .arg("validate")
        .arg(&dir)
        .output()
        .expect("toolbox runs");
    assert!(!o.status.success());
    let text = String::from_utf8_lossy(&o.stdout).to_string() + &String::from_utf8_lossy(&o.stderr);
    assert!(text.contains("compares product:ours twice"), "{text}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn peers_are_not_this_portals_debt() {
    let dir = scratch("ratchet");
    portal(
        &dir,
        &[
            product("ours", 0.7, 34.0, true, false),
            product("rival", 9.0, 2000.0, false, false),
        ],
    );
    let o = Command::new(env!("CARGO_BIN_EXE_toolbox"))
        .arg("report")
        .arg(&dir)
        .args(["--fixtures", "-o", "json", "--baseline"])
        .arg(dir.join("results.json"))
        .arg("--update")
        .output()
        .expect("toolbox runs");
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let baseline = std::fs::read_to_string(dir.join("results.json")).unwrap();
    assert!(baseline.contains("product:ours"), "{baseline}");
    assert!(!baseline.contains("product:rival"), "{baseline}");

    let web = dir.join("web");
    let o = Command::new(env!("CARGO_BIN_EXE_toolbox"))
        .arg("report")
        .arg(&dir)
        .args(["--fixtures", "-o", "web", "--out"])
        .arg(&web)
        .output()
        .expect("toolbox runs");
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let doc: Value =
        serde_json::from_str(&std::fs::read_to_string(web.join("report.json")).unwrap()).unwrap();
    assert!(doc["scores"]["against-rival"]["product:rival"].is_null());
    let c = &doc["comparisons"]["against-rival"];
    assert_eq!(c["titles"]["product:rival"], "RIVAL");
    assert_eq!(c["scores"]["product:rival"]["values"]["memory"], 2000.0);
    let _ = std::fs::remove_dir_all(&dir);
}
