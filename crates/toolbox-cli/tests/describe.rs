//! `toolbox describe scorecards` reports the facts the meta scorecards judge. These cases are the
//! cheapest ways to game them (docs/design/meta-standards-review.md): a map that reads nothing, and
//! rubric guidance that is blank or says nothing about failing.

use serde_json::Value;
use std::path::PathBuf;
use std::process::Command;

const MANIFEST: &str = r#"
apiVersion: toolbox.dev/v1alpha1
portal: {name: demo, title: Demo, owner: group:maintainers, toolbox: "0.1"}
connectors:
  - {name: things, package: builtin:git-files, runtime: remote, records: [thing, extra], freshness: {thing: 1h, extra: 1h}}
kinds:
  - name: Thing
    description: A thing.
    identityFrom: [connector:things]
    fields:
      title: {type: string, description: Its name.}
      hash: {type: string, description: Version of the thing.}
    sources: {default: [connector:things]}
    map:
      - {from: things/thing, id: "'thing:' + record.name", set: "{'title': record.name, 'hash': record.hash}"}
      - {from: things/extra, id: "'thing:x'", set: "{}"}
rubrics:
  - name: blank
    kind: Thing
    description: Judges a thing.
    criteria:
      - {name: empty, question: Is it useful?, guidance: ""}
      - {name: restated, question: Is it good?, guidance: Is the thing good enough overall?}
      - {name: real, question: Is it named?, guidance: Fail when the thing has no title at all in its file.}
"#;

fn describe(dir: &std::path::Path) -> Vec<Value> {
    let out = Command::new(env!("CARGO_BIN_EXE_toolbox"))
        .args(["describe", "scorecards"])
        .arg(dir)
        .output()
        .expect("run toolbox");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter_map(|l| serde_json::from_str(l).ok())
        .collect()
}

#[test]
fn a_map_that_reads_nothing_and_guidance_that_says_nothing_are_reported() {
    let dir: PathBuf =
        std::env::temp_dir().join(format!("toolbox-describe-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("toolbox.yaml"), MANIFEST).unwrap();
    let records = describe(&dir);
    let of = |kind: &str| -> Value {
        records
            .iter()
            .find(|r| r["kind"] == kind)
            .map(|r| r["data"].clone())
            .unwrap_or_else(|| panic!("no {kind} record"))
    };
    assert_eq!(of("connector")["unmapped"], serde_json::json!(["extra"]));
    assert_eq!(
        of("rubric")["unguided"],
        serde_json::json!(["empty", "restated"])
    );
    let _ = std::fs::remove_dir_all(&dir);
}
