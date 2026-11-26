//! Reading Betterer's results file (`.betterer.results`, "BETTERER RESULTS V2"), so repositories that
//! already ratchet with Betterer show up in Seikika scorecards without changing anything
//! (docs/design/ratchets.md §5). The file is JavaScript; it is parsed here without running it: each
//! entry is ``exports[`<test>`] = { value: `<printed value>` };`` with Betterer's escaping.

use regex::Regex;
use serde_json::{Map, Value, json};

/// Undo Betterer's `escape()` (packages/betterer/src/results/print.ts).
fn unescape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('b') => out.push('\u{8}'),
            Some('f') => out.push('\u{c}'),
            Some('r') => out.push('\r'),
            Some('t') => out.push('\t'),
            Some('u') => {
                let hex: String = chars.by_ref().take(4).collect();
                out.extend(u32::from_str_radix(&hex, 16).ok().and_then(char::from_u32));
            }
            Some(other) => out.push(other), // \\, \`, \', \{ (from `$\{`)
            None => out.push('\\'),
        }
    }
    out
}

/// One record per test: its raw printed value and, for file tests (a JSON object of
/// `"<path>:<hash>": [[line, column, length, message, hash], …]`), issue counts per file.
pub fn parse(text: &str) -> Vec<(String, Value)> {
    let entry =
        Regex::new(r"(?s)exports\[`((?:[^`\\]|\\.)*)`\] = \{\s*value: `((?:[^`\\]|\\.)*)`\s*\};")
            .expect("valid regex");
    entry
        .captures_iter(text)
        .map(|c| {
            let test = unescape(&c[1]);
            let value = unescape(&c[2]);
            let mut data = json!({"test": test, "value": value});
            match serde_json::from_str::<Value>(&value) {
                Ok(Value::Object(files)) => {
                    let mut per_file = Map::new();
                    let mut issues = 0;
                    for (key, list) in files {
                        let path = key
                            .rsplit_once(':')
                            .map_or(key.as_str(), |(p, _)| p)
                            .to_owned();
                        let n = list.as_array().map_or(0, Vec::len);
                        issues += n;
                        per_file.insert(path, json!(n));
                    }
                    data["kind"] = json!("file");
                    data["issues"] = json!(issues);
                    data["files"] = Value::Object(per_file);
                }
                Ok(n @ Value::Number(_)) => {
                    data["kind"] = json!("number");
                    data["number"] = n;
                }
                _ => data["kind"] = json!("other"),
            }
            (test, data)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    // From Betterer's own test snapshots (test/__snapshots__/file-test-goal.spec.ts.snap, with Jest's
    // snapshot escaping removed), plus a
    // numeric test and a name that needs escaping.
    const RESULTS: &str = r#"// BETTERER RESULTS V2.
//
// If this file contains merge conflicts, use `betterer merge` to automatically resolve them:
// https://phenomnomnominal.github.io/betterer/docs/results-file/#merge
//
exports[`test`] = {
  value: `{
    "src/index.ts:2053668783": [
      [0, 0, 9, "Unexpected \'debugger\' statement.", "3201740415"],
      [1, 0, 9, "Unexpected \'debugger\' statement.", "3201740415"]
    ]
  }`
};

exports[`bundle size \`main\``] = {
  value: `1234`
};
"#;

    #[test]
    fn parses_file_and_number_tests() {
        let r = parse(RESULTS);
        assert_eq!(r.len(), 2);
        assert_eq!(r[0].0, "test");
        assert_eq!(r[0].1["kind"], "file");
        assert_eq!(r[0].1["issues"], 2);
        assert_eq!(r[0].1["files"], json!({"src/index.ts": 2}));
        assert_eq!(r[1].0, "bundle size `main`");
        assert_eq!(
            (&r[1].1["kind"], &r[1].1["number"]),
            (&json!("number"), &json!(1234))
        );
    }
}
