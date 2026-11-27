//! The ratchet as a pipeline sees it (D66, D96), end to end through the binary: `seikika report
//! --baseline` exits 1 when something gets worse, and `--verdict` says what, as data any CI system
//! can read. Each step rewrites the portal's fixtures, as a pull request would change the code.

use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command;

const MANIFEST: &str = r#"
apiVersion: seikika.com/v1alpha1
portal: {name: demo, title: Demo, owner: group:maintainers, seikika: "0.1"}
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
    std::fs::write(dir.join("seikika.yaml"), MANIFEST).unwrap();
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

/// `seikika report --fixtures --baseline … --verdict …` plus `extra`: (exit code, verdict).
fn report(dir: &Path, extra: &[&str]) -> (i32, Value) {
    let verdict = dir.join("verdict.json");
    let _ = std::fs::remove_file(&verdict);
    let o = Command::new(env!("CARGO_BIN_EXE_seikika"))
        .arg("report")
        .arg(dir)
        .args(["--fixtures", "-o", "json", "--baseline"])
        .arg(dir.join("results.json"))
        .arg("--verdict")
        .arg(&verdict)
        .args(extra)
        .output()
        .expect("seikika runs");
    let v = std::fs::read_to_string(&verdict).expect("a verdict is always written");
    (
        o.status.code().unwrap_or(-1),
        serde_json::from_str(&v).unwrap(),
    )
}

fn scratch() -> PathBuf {
    let dir = std::env::temp_dir().join(format!("seikika-ratchet-{}", std::process::id()));
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
    // Each regression carries its root cause: the rule's evidence now (the baseline recorded
    // its items' keys, so only the new one is listed), and where it came from.
    assert_eq!(
        v["regressions"][0]["detail"],
        serde_json::json!([
            "documented: + usage",
            "documented: and 1 already in the baseline",
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
    std::fs::write(dir.join("seikika.yaml"), "kinds: 3\n").unwrap();
    let (code, v) = report(&dir, &["--ci"]);
    assert_eq!((code, v["result"].as_str()), (2, Some("error")));
    assert!(v["message"].as_str().is_some_and(|m| !m.is_empty()));
    let _ = std::fs::remove_dir_all(&dir);
}

/// A gate prints its verdict, not the report: with --baseline and neither -o nor --out, stdout
/// says what changed, marking new evidence, and nothing else. A baseline written before
/// evidence keys still works (the whole grown list is shown); it isn't stale for lacking them,
/// and --update adds them.
#[test]
fn a_gate_prints_what_changed() {
    let dir = std::env::temp_dir().join(format!("seikika-gate-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let run = |extra: &[&str]| {
        let o = Command::new(env!("CARGO_BIN_EXE_seikika"))
            .arg("report")
            .arg(&dir)
            .args(["--fixtures", "--baseline"])
            .arg(dir.join("results.json"))
            .args(extra)
            .output()
            .expect("seikika runs");
        let text = |b: &[u8]| String::from_utf8_lossy(b).into_owned();
        (
            o.status.code().unwrap_or(-1),
            text(&o.stdout),
            text(&o.stderr),
        )
    };
    portal(&dir, &[("a", true, &["intro", "setup 1"])]);
    assert_eq!(run(&["--update"]).0, 0);
    let baseline = dir.join("results.json");
    let doc: Value = serde_json::from_str(&std::fs::read_to_string(&baseline).unwrap()).unwrap();
    assert_eq!(
        doc["evidence"]["quality"]["thing:a"]["documented"]
            .as_array()
            .map(Vec::len),
        Some(2)
    );

    // `setup 1` moved to `setup 2` (same key); `usage` is new.
    portal(&dir, &[("a", true, &["intro", "setup 2", "usage"])]);
    let (code, out, err) = run(&[]);
    assert_eq!((code, err.as_str()), (1, ""), "{out}");
    assert!(out.starts_with("✗ worse   thing:a · quality\n"), "{out}");
    assert!(out.contains("    documented: + usage\n"), "{out}");
    assert!(
        out.contains("and 2 finding(s) already in the baseline\n"),
        "{out}"
    );
    assert!(!out.contains("from: connector"), "{out}");
    assert!(out.contains("every scorecard: add -o md"), "{out}");
    assert!(!out.contains("# "), "no report body: {out}");
    // Asked for, the report is on stdout and the verdict on stderr, as before.
    let (_, out, err) = run(&["-o", "md"]);
    assert!(
        out.contains("quality") && err.contains("✗ worse   thing:a"),
        "{out}"
    );

    // An old baseline, without keys: the whole grown list, and not stale once fixed.
    let mut old = doc.clone();
    old.as_object_mut().unwrap().remove("evidence");
    std::fs::write(&baseline, serde_json::to_string(&old).unwrap()).unwrap();
    let (_, out, _) = run(&[]);
    assert!(
        out.contains("documented: intro\n") && out.contains("documented: usage\n"),
        "{out}"
    );
    portal(&dir, &[("a", true, &["intro", "setup 3"])]);
    let (code, out, _) = run(&["--ci"]);
    assert_eq!(code, 0, "{out}");
    let (code, out, _) = run(&["--update"]);
    assert!(
        code == 0 && out.contains("updated its evidence keys"),
        "{out}"
    );
    let doc: Value = serde_json::from_str(&std::fs::read_to_string(&baseline).unwrap()).unwrap();
    assert_eq!(
        doc["evidence"]["quality"]["thing:a"]["documented"]
            .as_array()
            .map(Vec::len),
        Some(2)
    );
    let _ = std::fs::remove_dir_all(&dir);
}

/// One finding, one line: when several rules, or several entities, report the same new item, the
/// summary shows it once, after every rule (and other entity) that reports it. The verdict
/// document keeps each regression's own detail.
#[test]
fn a_finding_several_rules_report_is_one_line() {
    let dir = std::env::temp_dir().join(format!("seikika-fold-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let both = format!(
        "{MANIFEST}  - name: docs\n    kind: Thing\n    owner: group:maintainers\n    levels: [gold]\n    rules:\n      - {{name: all-documented, level: gold, check: \"entity.undocumented.size() == 0\", evidence: \"entity.undocumented\"}}\n"
    );
    let write = |things: &[(&str, &[&str])]| {
        portal(
            &dir,
            &things
                .iter()
                .map(|(n, u)| (*n, true, *u))
                .collect::<Vec<_>>(),
        );
        std::fs::write(dir.join("seikika.yaml"), &both).unwrap();
    };
    let run = |extra: &[&str]| {
        let o = Command::new(env!("CARGO_BIN_EXE_seikika"))
            .arg("report")
            .arg(&dir)
            .args(["--fixtures", "--baseline"])
            .arg(dir.join("results.json"))
            .args(extra)
            .output()
            .expect("seikika runs");
        (
            o.status.code().unwrap_or(-1),
            String::from_utf8_lossy(&o.stdout).into_owned(),
        )
    };
    write(&[("a", &[]), ("b", &[])]);
    assert_eq!(run(&["--update"]).0, 0);
    write(&[("a", &["usage"]), ("b", &["usage"])]);
    let (code, out) = run(&[]);
    assert_eq!(code, 1, "{out}");
    assert!(
        out.starts_with("✗ worse   thing:a · docs, quality\n"),
        "{out}"
    );
    assert!(
        out.contains("    all-documented, all-documented (thing:b), documented, documented (thing:b): + usage\n"),
        "{out}"
    );
    assert_eq!(out.matches("usage").count(), 1, "{out}");
    assert!(!out.contains("thing:b ·"), "{out}");
    assert!(out.contains("baseline: 4 regression(s)"), "{out}");
    let _ = std::fs::remove_dir_all(&dir);
}

/// A rule an agent's assessment decides is marked as judged wherever text names it: in the gate's
/// summary and in the markdown report, each with the legend that explains the mark.
#[test]
fn judged_rules_are_marked_in_the_gate_and_the_markdown() {
    let dir = std::env::temp_dir().join(format!("seikika-judged-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let manifest = MANIFEST
        .replace("records: [thing], freshness: {thing: 1h}", "records: [thing, assessment], freshness: {thing: 1h, assessment: 1h}")
        .replace(
            "      undocumented: {type: list, items: string, description: Its undocumented parts.}\n",
            "      undocumented: {type: list, items: string, description: Its undocumented parts.}\n      reviewFailing: {type: list, items: string, description: What its review failed., computed: \"inbound.assesses.size() == 0 ? [] : inbound.assesses[0].failing\"}\n",
        )
        .replace("scorecards:\n", r#"  - name: Assessment
    description: An agent's review of a thing.
    identityFrom: [connector:things]
    fields:
      failing: {type: list, items: string, description: The criteria it failed.}
    relations:
      assesses: {to: Thing, many: false, unresolved: ignore}
    sources: {default: [connector:things]}
    map:
      - {from: things/assessment, id: "'assessment:' + record.entity", set: "{'failing': record.failing}", relations: {assesses: "[record.entity]"}}
scorecards:
  - name: review
    kind: Thing
    owner: group:maintainers
    levels: [bronze]
    rules:
      - {name: passes-review, level: bronze, check: "entity.reviewFailing.size() == 0", evidence: "entity.reviewFailing"}
"#);
    let write = |failing: &[&str]| {
        portal(&dir, &[("a", true, &[])]);
        std::fs::write(dir.join("seikika.yaml"), &manifest).unwrap();
        let review = serde_json::json!({"type": "record", "kind": "assessment", "externalId": "assessment:thing:a",
            "data": {"entity": "thing:a", "failing": failing}});
        let things = dir.join("fixtures/things.ndjson");
        let thing = std::fs::read_to_string(&things).unwrap();
        std::fs::write(things, format!("{thing}\n{review}")).unwrap();
    };
    let run = |extra: &[&str]| {
        let o = Command::new(env!("CARGO_BIN_EXE_seikika"))
            .arg("report")
            .arg(&dir)
            .args(["--fixtures", "--baseline"])
            .arg(dir.join("results.json"))
            .args(extra)
            .output()
            .expect("seikika runs");
        let text = |b: &[u8]| String::from_utf8_lossy(b).into_owned();
        (
            o.status.code().unwrap_or(-1),
            text(&o.stdout),
            text(&o.stderr),
        )
    };
    write(&[]);
    let (code, out, err) = run(&["--update"]);
    assert_eq!(code, 0, "{out}{err}");
    write(&["cites-nothing"]);
    let (code, out, _) = run(&[]);
    assert_eq!(code, 1, "{out}");
    assert!(
        out.contains("    passes-review ✦: + cites-nothing\n"),
        "{out}"
    );
    assert!(
        out.contains("✦ agent-judged: an agent's assessment decides the rule"),
        "{out}"
    );
    assert!(!out.contains("tested ✦"), "{out}");
    let (_, md, _) = run(&["-o", "md"]);
    assert!(
        md.contains("## review  (none: 1) · every rule agent-judged ✦"),
        "{md}"
    );
    assert!(
        md.contains("`thing:a`: passes-review ✦\n  - passes-review ✦: cites-nothing"),
        "{md}"
    );
    assert!(md.contains("\n## quality  (silver: 1)\n"), "{md}");

    // The same failing assessment remains visible when the scorecard is advisory.
    let advisory = manifest.replace(
        "  - name: review\n    kind: Thing",
        "  - name: review\n    ratchet: {advisory: true}\n    kind: Thing",
    );
    std::fs::write(dir.join("seikika.yaml"), advisory).unwrap();
    let (code, md, err) = run(&["-o", "md"]);
    assert_eq!(code, 0, "{md}{err}");
    assert!(md.contains("passes-review ✦: cites-nothing"), "{md}");
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
    let dir = std::env::temp_dir().join(format!("seikika-causes-{}", std::process::id()));
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
        std::fs::write(dir.join("seikika.yaml"), &manifest).unwrap();
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
    let dir = std::env::temp_dir().join(format!("seikika-unknown-{}", std::process::id()));
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

    // When the same absence is caused by an explicit skip, strict mode
    // preserves the unknown evidence and permits the report.
    let manifest = MANIFEST.replace(
        "{name: tested, level: bronze, check:",
        "{name: tested, level: bronze, dataFrom: things, check:",
    );
    std::fs::write(dir.join("seikika.yaml"), manifest).unwrap();
    let (code, v) = report(&dir, &["--fail-on-unknown", "--skip", "things"]);
    assert_eq!((code, v["unknownRules"].as_u64()), (0, Some(1)), "{v}");
    let _ = std::fs::remove_dir_all(&dir);
}

/// `--accept` restarts a campaign over the accepted scorecard, keeping the start it replaced
/// under `restartedFrom`; with `--keep-campaigns` the start stays, and the campaign goes backwards.
#[test]
fn accept_restarts_campaigns_unless_kept() {
    let dir = std::env::temp_dir().join(format!("seikika-keep-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    portal(&dir, &[("a", false, &[]), ("b", true, &[])]);
    let manifest = MANIFEST.to_owned()
        + "campaigns:\n  - name: tested\n    owner: group:maintainers\n    title: Every thing tested\n    includes: [{scorecard: quality, rules: [tested]}]\n";
    std::fs::write(dir.join("seikika.yaml"), manifest).unwrap();
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
