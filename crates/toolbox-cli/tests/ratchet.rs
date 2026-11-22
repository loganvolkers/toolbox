//! The ratchet as a pipeline sees it (D66, D96), end to end through the binary: `toolbox report
//! --baseline` exits 1 when something gets worse, and `--verdict` says what, as data any CI system
//! can read. Each step rewrites the portal's fixtures, as a pull request would change the code.

use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command;

const MANIFEST: &str = r#"
apiVersion: toolbox.dev/v1alpha1
portal: {name: demo, title: Demo, owner: group:maintainers, toolbox: "0.1"}
connectors:
  - {name: things, package: builtin:git-files, runtime: remote, records: [thing], freshness: {thing: 1h}}
kinds:
  - name: Thing
    description: A thing with docs and tests.
    identityFrom: [connector:things]
    fields:
      tested: {type: boolean, description: Whether it has a test.}
      undocumented: {type: list, items: string, description: Its undocumented parts.}
    sources: {default: [connector:things]}
    map:
      - {from: things/thing, id: "'thing:' + record.name", set: "{'tested': record.tested, 'undocumented': record.undocumented}"}
scorecards:
  - name: quality
    kind: Thing
    owner: group:maintainers
    levels: [bronze, silver]
    ratchet: {newEntitiesAtLeast: bronze}
    rules:
      - {name: tested, level: bronze, check: "entity.tested", fix: "Add a test."}
      - {name: documented, level: silver, check: "entity.undocumented.size() == 0", evidence: "entity.undocumented", fix: "Document them."}
"#;

/// A portal whose fixtures hold `things`: (name, tested, undocumented parts).
fn portal(dir: &Path, things: &[(&str, bool, &[&str])]) {
    std::fs::create_dir_all(dir.join("fixtures")).unwrap();
    std::fs::write(dir.join("toolbox.yaml"), MANIFEST).unwrap();
    let lines: Vec<String> = things
        .iter()
        .map(|(name, tested, undocumented)| {
            serde_json::json!({"type": "record", "kind": "thing", "externalId": format!("thing:{name}"),
                "data": {"name": name, "tested": tested, "undocumented": undocumented}})
            .to_string()
        })
        .collect();
    std::fs::write(dir.join("fixtures/things.ndjson"), lines.join("\n")).unwrap();
}

/// `toolbox report --fixtures --baseline … --verdict …` plus `extra`: (exit code, verdict).
fn report(dir: &Path, extra: &[&str]) -> (i32, Value) {
    let verdict = dir.join("verdict.json");
    let _ = std::fs::remove_file(&verdict);
    let o = Command::new(env!("CARGO_BIN_EXE_toolbox"))
        .arg("report")
        .arg(dir)
        .args(["--fixtures", "-o", "json", "--baseline"])
        .arg(dir.join("results.json"))
        .arg("--verdict")
        .arg(&verdict)
        .args(extra)
        .output()
        .expect("toolbox runs");
    let v = std::fs::read_to_string(&verdict).expect("a verdict is always written");
    (
        o.status.code().unwrap_or(-1),
        serde_json::from_str(&v).unwrap(),
    )
}

fn scratch() -> PathBuf {
    let dir = std::env::temp_dir().join(format!("toolbox-ratchet-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

#[test]
fn a_falling_ratchet_fails_the_pipeline_and_says_why() {
    let dir = scratch();
    portal(&dir, &[("a", true, &["intro"])]);

    // No baseline yet: --ci fails, --update creates it, then the ratchet passes.
    let (code, v) = report(&dir, &["--ci"]);
    assert_eq!((code, v["result"].as_str()), (1, Some("missing-baseline")));
    let (code, v) = report(&dir, &["--update"]);
    assert_eq!(
        (code, &v["result"], &v["written"]),
        (0, &"pass".into(), &true.into())
    );
    let (code, v) = report(&dir, &["--ci"]);
    assert_eq!(
        (code, &v["result"], &v["mode"]),
        (0, &"pass".into(), &"ci".into())
    );

    // Worse: a failing rule's evidence grows, and another rule starts failing.
    portal(&dir, &[("a", false, &["intro", "usage"])]);
    let (code, v) = report(&dir, &["--ci"]);
    assert_eq!((code, v["result"].as_str()), (1, Some("regressed")));
    let messages: Vec<(&str, &str, &str)> = v["regressions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| {
            (
                r["scorecard"].as_str().unwrap(),
                r["entity"].as_str().unwrap(),
                r["message"].as_str().unwrap(),
            )
        })
        .collect();
    assert_eq!(
        messages,
        [
            ("quality", "thing:a", "documented evidence grew 1 → 2"),
            ("quality", "thing:a", "tested now fails")
        ]
    );
    // Each regression carries its root cause: the rule's evidence now, and where it came from.
    assert_eq!(
        v["regressions"][0]["detail"],
        serde_json::json!([
            "documented: intro",
            "documented: usage",
            "from: connector things, record thing thing:a"
        ])
    );
    // --update refuses to record a regression.
    let (code, v) = report(&dir, &["--update"]);
    assert_eq!((code, &v["written"]), (1, &false.into()));

    // A new entity below the scorecard's floor is a regression too.
    portal(&dir, &[("a", true, &["intro"]), ("b", false, &[])]);
    let (code, v) = report(&dir, &["--ci"]);
    assert_eq!((code, v["result"].as_str()), (1, Some("regressed")));
    assert_eq!(v["regressions"][0]["entity"], "thing:b");

    // Better but not recorded: stale fails --ci only; --update records it.
    portal(&dir, &[("a", true, &[])]);
    let (code, v) = report(&dir, &[]);
    assert_eq!(
        (code, &v["result"], &v["mode"]),
        (0, &"stale".into(), &"check".into())
    );
    assert_eq!(v["improvements"][0]["message"], "documented now passes");
    assert_eq!(report(&dir, &["--ci"]).0, 1);
    assert_eq!(report(&dir, &["--update"]).0, 0);
    assert_eq!(report(&dir, &["--ci"]).1["result"], "pass");

    // A portal that doesn't load is an error (exit 2), and still has a verdict.
    std::fs::write(dir.join("toolbox.yaml"), "kinds: 3\n").unwrap();
    let (code, v) = report(&dir, &["--ci"]);
    assert_eq!((code, v["result"].as_str()), (2, Some("error")));
    assert!(v["message"].as_str().is_some_and(|m| !m.is_empty()));
    let _ = std::fs::remove_dir_all(&dir);
}

/// A repository level over `quality`: its regression names the scorecard that fell and the
/// entity that made it fall, with that entity's evidence (docs/design/root-causes.md).
const AGGREGATE: &str = r#"
  - name: Repo
    description: The repository, rolling up its scorecards.
    identityFrom: [connector:things]
    fields:
      watch: {type: list, items: string, description: The scorecards it rolls up.}
    sources: {default: [connector:things]}
    map:
      - {from: things/repo, id: "'repo:' + record.name", set: "{'watch': record.watch}"}
"#;

#[test]
fn a_repository_level_regression_names_its_root_cause() {
    let dir = std::env::temp_dir().join(format!("toolbox-causes-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let manifest = MANIFEST
        .replace("records: [thing]", "records: [thing, repo]")
        .replace(
            "scorecards:\n",
            &format!("{}scorecards:\n", AGGREGATE.trim_start_matches('\n')),
        )
        + r#"  - name: repo-level
    kind: Repo
    owner: group:maintainers
    levels: [bronze]
    rules:
      - {name: constituents-at-least-bronze, level: bronze, check: "cards.all(c, c.rank >= 1)",
         evidence: "cards.filter(c, c.rank < 1).map(c, c.name + ' is ' + c.level)", fix: "Raise each one to bronze."}
"#;
    let write = |things: &[(&str, bool)]| {
        std::fs::create_dir_all(dir.join("fixtures")).unwrap();
        std::fs::write(dir.join("toolbox.yaml"), &manifest).unwrap();
        let mut lines = vec![
            serde_json::json!({"type": "record", "kind": "repo", "externalId": "repo:r",
            "data": {"name": "r", "watch": ["quality"]}})
            .to_string(),
        ];
        lines.extend(things.iter().map(|(name, tested)| {
            serde_json::json!({"type": "record", "kind": "thing", "externalId": format!("thing:{name}"),
                "data": {"name": name, "tested": tested, "undocumented": []}})
            .to_string()
        }));
        std::fs::write(dir.join("fixtures/things.ndjson"), lines.join("\n")).unwrap();
    };
    write(&[("a", true)]);
    let (code, _) = report(&dir, &["--update"]);
    assert_eq!(code, 0);
    // `a` loses its test: quality falls to none, so the repository level fails.
    write(&[("a", false)]);
    let (code, v) = report(&dir, &[]);
    assert_eq!(code, 1);
    let level = v["regressions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["scorecard"] == "repo-level")
        .expect("the repository level regressed");
    let fell = &level["causes"][0];
    assert_eq!(
        (fell["scorecard"].as_str(), fell["message"].as_str()),
        (Some("quality"), Some("fell silver → none"))
    );
    let cause = &fell["causes"][0];
    assert_eq!(
        (cause["entity"].as_str(), cause["message"].as_str()),
        (Some("thing:a"), Some("tested now fails"))
    );
    assert_eq!(
        cause["detail"][0],
        "from: connector things, record thing thing:a"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

/// A rule whose data is missing although its connector ran (a kit's `optional: true` tool that
/// isn't installed) is unknown. A plain check passes then; `--fail-on-unknown` fails it.
#[test]
fn fail_on_unknown_catches_a_missing_tool() {
    let dir = std::env::temp_dir().join(format!("toolbox-unknown-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    portal(&dir, &[("a", true, &[])]);
    assert_eq!(report(&dir, &["--update"]).0, 0);
    // The record arrives with no value for `tested`: the rule reading it can't be evaluated.
    std::fs::write(
        dir.join("fixtures/things.ndjson"),
        serde_json::json!({"type": "record", "kind": "thing", "externalId": "thing:a",
            "data": {"name": "a", "tested": null, "undocumented": []}})
        .to_string(),
    )
    .unwrap();
    let (code, v) = report(&dir, &[]);
    assert_eq!((code, v["unknownRules"].as_u64()), (0, Some(1)), "{v}");
    let (code, v) = report(&dir, &["--fail-on-unknown"]);
    assert_eq!((code, v["result"].as_str()), (1, Some("unknown")), "{v}");
    let _ = std::fs::remove_dir_all(&dir);
}

/// `--accept` restarts a campaign over the accepted scorecard, keeping the start it replaced
/// under `restartedFrom`; with `--keep-campaigns` the start stays, and the campaign goes backwards.
#[test]
fn accept_restarts_campaigns_unless_kept() {
    let dir = std::env::temp_dir().join(format!("toolbox-keep-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    portal(&dir, &[("a", false, &[]), ("b", true, &[])]);
    let manifest = MANIFEST.to_owned()
        + "campaigns:\n  - name: tested\n    owner: group:maintainers\n    title: Every thing tested\n    includes: [{scorecard: quality, rules: [tested]}]\n";
    std::fs::write(dir.join("toolbox.yaml"), manifest).unwrap();
    let campaign = |dir: &Path| {
        let b: Value =
            serde_json::from_str(&std::fs::read_to_string(dir.join("results.json")).unwrap())
                .unwrap();
        b["campaigns"]["tested"].clone()
    };
    assert_eq!(report(&dir, &["--update"]).0, 0);
    assert_eq!(campaign(&dir)["short"], 1);
    // A second thing stops being tested: a regression, accepted for the quality scorecard.
    let reset = |dir: &Path, tested_b: bool| {
        let lines: Vec<String> = [("a", false), ("b", tested_b)]
            .iter()
            .map(|(n, t)| serde_json::json!({"type": "record", "kind": "thing", "externalId": format!("thing:{n}"),
                "data": {"name": n, "tested": t, "undocumented": []}}).to_string())
            .collect();
        std::fs::write(dir.join("fixtures/things.ndjson"), lines.join("\n")).unwrap();
    };
    reset(&dir, false);
    let before = std::fs::read_to_string(dir.join("results.json")).unwrap();
    assert_eq!(
        report(
            &dir,
            &["--update", "--accept", "quality", "--keep-campaigns"]
        )
        .0,
        0
    );
    assert_eq!(
        campaign(&dir)["short"],
        1,
        "kept: the start stays at 1 short"
    );
    std::fs::write(dir.join("results.json"), before).unwrap();
    assert_eq!(report(&dir, &["--update", "--accept", "quality"]).0, 0);
    let c = campaign(&dir);
    assert_eq!(
        (c["short"].as_u64(), c["restartedFrom"]["short"].as_u64()),
        (Some(2), Some(1)),
        "{c}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}
