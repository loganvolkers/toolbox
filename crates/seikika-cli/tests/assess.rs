//! The push model for judgement (D79), end to end through the binary: a portal with a rubric is
//! described (`seikika describe portal`), an agent is briefed (`seikika assess brief`), its
//! assessment file is checked (`seikika assess check`), then pushed to a running server
//! (`seikika assess submit`), where a scorecard reads it as data.

use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::Duration;

const MANIFEST: &str = r#"
apiVersion: seikika.com/v1alpha1
portal: {name: demo, title: Demo, owner: group:maintainers, seikika: "0.1"}
connectors:
  - {name: things, package: builtin:git-files, runtime: remote, records: [thing], freshness: {thing: 1h}}
  - {name: assessments, package: builtin:git-files, runtime: remote, records: [assessment]}
kinds:
  - name: Thing
    description: A thing worth judging.
    identityFrom: [connector:things]
    fields:
      title: {type: string, description: Its name.}
      hash: {type: string, description: Version of the thing.}
      reviewState:
        type: string
        computed: "inbound.assesses.size() == 0 ? 'missing' : (inbound.assesses[0].subjectHash == entity.hash ? 'fresh' : 'stale')"
    sources: {default: [connector:things]}
    map:
      - {from: things/thing, id: "'thing:' + record.name", set: "{'title': record.name, 'hash': record.hash}"}
  - name: Assessment
    identityFrom: [connector:assessments]
    fields:
      subjectHash: {type: string}
      verdict: {type: string}
    relations:
      assesses: {to: Thing, many: false}
    sources: {default: [connector:assessments]}
    map:
      - from: assessments/assessment
        id: "'assessment:' + record.rubric + '/' + record.entity"
        set: "{'subjectHash': record.subjectHash, 'verdict': record.verdict}"
        relations: {assesses: "[record.entity]"}
scorecards:
  - name: reviewed
    kind: Thing
    owner: group:maintainers
    levels: [bronze]
    rules:
      - {name: assessed, level: bronze, check: "entity.reviewState == 'fresh'", fix: "Run seikika assess brief and write the file."}
rubrics:
  - name: thing-review
    kind: Thing
    criteria:
      - {name: useful, question: Is it useful?}
tests: [tests/t.test.yaml]
"#;

const ASSESSMENT: &str = r#"apiVersion: seikika.com/assessment/v1
entity: thing:a
rubric: thing-review
subjectHash: "0123456789ab"
assessor: {kind: agent, name: test}
at: "2026-11-20"
verdicts:
  - {criterion: useful, verdict: pass, reason: It does what it says on the tin.}
"#;

fn seikika(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_seikika"))
        .args(args)
        .env("SEIKIKA_OPENFGA_BIN", "/nonexistent") // dev mode, no authorization
        .output()
        .expect("seikika runs")
}

fn stdout(o: &Output) -> String {
    assert!(
        o.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&o.stdout),
        String::from_utf8_lossy(&o.stderr)
    );
    String::from_utf8_lossy(&o.stdout).into_owned()
}

fn portal() -> PathBuf {
    portal_with("brief", MANIFEST)
}

fn portal_with(name: &str, manifest: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("seikika-assess-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    for sub in ["fixtures", "tests", "assessments/thing-review/thing"] {
        std::fs::create_dir_all(dir.join(sub)).unwrap();
    }
    std::fs::write(dir.join("seikika.yaml"), manifest).unwrap();
    std::fs::write(
        dir.join("fixtures/things.ndjson"),
        r#"{"type": "record", "kind": "thing", "externalId": "thing:a", "data": {"name": "a", "hash": "0123456789ab"}}"#,
    )
    .unwrap();
    std::fs::write(
        dir.join("tests/t.test.yaml"),
        "name: t\nassert:\n  - {name: unassessed, expect: \"score('reviewed', 'thing:a').level == 'none'\"}\n",
    )
    .unwrap();
    dir
}

fn free_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

fn wait_for(url: &str, id: &str, want: &str) -> Value {
    for _ in 0..100 {
        let o = seikika(&["get", id, "--url", url, "--as", "dev", "-o", "json"]);
        if o.status.success() {
            let v: Value = serde_json::from_slice(&o.stdout).unwrap_or_default();
            if v["fields"]["reviewState"] == want {
                return v;
            }
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    panic!("{id} never reached reviewState {want}");
}

#[test]
fn describe_brief_check_and_submit() {
    let dir = portal();
    let p = dir.to_str().unwrap();
    stdout(&seikika(&["validate", p]));

    // The model as records, with content hashes.
    let described = stdout(&seikika(&["describe", "portal", p]));
    let kinds: Vec<Value> = described
        .lines()
        .map(|l| serde_json::from_str::<Value>(l).unwrap())
        .collect();
    let thing = kinds
        .iter()
        .find(|r| r["externalId"] == "kind:demo/Thing")
        .expect("kind record");
    assert_eq!(thing["data"]["described"], true);
    assert!(
        kinds
            .iter()
            .any(|r| r["externalId"] == "field:demo/Thing.hash" && r["data"]["described"] == true)
    );
    assert!(
        kinds
            .iter()
            .any(|r| r["externalId"] == "connector:demo/assessments"
                && r["data"]["withoutFreshness"] == serde_json::json!(["assessment"]))
    );

    // The brief names the rubric, the subject hash and where to write the file.
    let brief: Value = serde_json::from_str(&stdout(&seikika(&[
        "assess",
        "brief",
        "thing:a",
        "--portal",
        p,
        "--fixtures",
        "-o",
        "json",
    ])))
    .unwrap();
    assert_eq!(brief["subjectHash"], "0123456789ab");
    assert_eq!(brief["file"], "assessments/thing-review/thing/a.yaml");

    // Before the file exists the entity is missing; a valid file makes it fresh; a bad one fails.
    let check = stdout(&seikika(&["assess", "check", p, "--fixtures"]));
    assert!(
        check.contains("thing-review: 0 fresh, 0 stale, 1 missing"),
        "{check}"
    );
    let file = dir.join("assessments/thing-review/thing/a.yaml");
    std::fs::write(&file, ASSESSMENT).unwrap();
    let check = stdout(&seikika(&["assess", "check", p, "--fixtures"]));
    assert!(check.contains("- fresh `thing:a` pass"), "{check}");
    std::fs::write(&file, ASSESSMENT.replace("0123456789ab", "abcdef012345")).unwrap();
    let check = stdout(&seikika(&["assess", "check", p, "--fixtures"]));
    assert!(check.contains("- stale `thing:a` pass"), "{check}");
    let strict = seikika(&["assess", "check", p, "--fixtures", "--require-fresh"]);
    assert_eq!(strict.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&strict.stdout).contains("- stale `thing:a` pass"));
    std::fs::write(&file, ASSESSMENT).unwrap();
    stdout(&seikika(&[
        "assess",
        "check",
        p,
        "--fixtures",
        "--require-fresh",
    ]));
    let bad = dir.join("assessments/thing-review/thing/bad.yaml");
    std::fs::write(&bad, ASSESSMENT.replace("useful", "pretty")).unwrap();
    let o = seikika(&["assess", "check", p, "--fixtures"]);
    assert_eq!(o.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&o.stdout).contains("no verdict for criterion useful"));
    std::fs::remove_file(&bad).unwrap();

    // Push it to a running server: the scorecard reads it like any other record.
    let url = format!("http://127.0.0.1:{}", free_port());
    let mut server = Command::new(env!("CARGO_BIN_EXE_seikika"))
        .args([
            "serve",
            p,
            "--fixtures",
            "--listen",
            url.trim_start_matches("http://"),
        ])
        .env("SEIKIKA_OPENFGA_BIN", "/nonexistent")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .unwrap();
    wait_for(&url, "thing:a", "missing");
    let submitted = stdout(&seikika(&[
        "assess",
        "submit",
        file.to_str().unwrap(),
        "--portal",
        p,
        "--url",
        &url,
    ]));
    assert!(submitted.starts_with("1 assessment(s)"), "{submitted}");
    wait_for(&url, "thing:a", "fresh");
    let _ = server.kill();
    let _ = server.wait();
    let _ = std::fs::remove_dir_all(Path::new(&dir));
}

/// A rubric for assessments reviews each first-level assessment once, and never its own reviews,
/// so `assess check` can reach zero missing (docs/design/scorecards-for-scorecards.md §6.1).
#[test]
fn reviews_stop_at_one_level() {
    let manifest = MANIFEST
        .replace(
            "      subjectHash: {type: string}\n",
            "      subjectHash: {type: string}\n      rubric: {type: string}\n      hash: {type: string}\n",
        )
        .replace(
            "'verdict': record.verdict}",
            "'verdict': record.verdict, 'rubric': record.rubric, 'hash': record.hash}",
        )
        .replace("relations: {assesses: \"[record.entity]\"}", "relations: {assesses: \"record.entity.startsWith('thing:') ? [record.entity] : []\"}")
        .replace(
            "tests: [",
            "  - name: assessment-review\n    kind: Assessment\n    criteria:\n      - {name: supported, question: Is it supported?}\ntests: [",
        );
    let dir = portal_with("review", &manifest);
    let p = dir.to_str().unwrap();
    let first = "assessment:thing-review/thing:a";
    let review = "assessment:assessment-review/assessment:thing-review/thing:a";
    let record = |id: &str, entity: &str, rubric: &str, hash: &str| {
        format!(
            r#"{{"type": "record", "kind": "assessment", "externalId": "{id}", "data": {{"entity": "{entity}", "rubric": "{rubric}", "subjectHash": "0123456789ab", "verdict": "pass", "hash": "{hash}"}}}}"#
        )
    };
    std::fs::write(
        dir.join("fixtures/assessments.ndjson"),
        [
            record(first, "thing:a", "thing-review", "aaaaaaaaaaaa"),
            record(review, first, "assessment-review", "bbbbbbbbbbbb"),
        ]
        .join("\n"),
    )
    .unwrap();
    stdout(&seikika(&["validate", p]));

    let check = stdout(&seikika(&["assess", "check", p, "--fixtures"]));
    assert!(
        check.contains("assessment-review: 0 fresh, 0 stale, 1 missing"),
        "{check}"
    );
    let file = dir.join("assessments/assessment-review/assessment/thing-review/thing/a.yaml");
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    std::fs::write(
        &file,
        format!(
            "apiVersion: seikika.com/assessment/v1\nentity: {first}\nrubric: assessment-review\n\
             subjectHash: \"aaaaaaaaaaaa\"\nassessor: {{kind: agent, name: reviewer}}\nat: \"2026-11-25\"\n\
             verdicts:\n  - {{criterion: supported, verdict: pass, reason: It cites the tin.}}\n"
        ),
    )
    .unwrap();

    // The review is not itself a subject of the rubric that made it.
    let check = stdout(&seikika(&["assess", "check", p, "--fixtures"]));
    assert!(
        check.contains("assessment-review: 1 fresh, 0 stale, 0 missing"),
        "{check}"
    );
    let o = seikika(&["assess", "brief", review, "--portal", p, "--fixtures"]);
    assert_eq!(o.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&o.stderr).contains("does not review its own assessments"),
        "{}",
        String::from_utf8_lossy(&o.stderr)
    );
    let _ = std::fs::remove_dir_all(&dir);
}
