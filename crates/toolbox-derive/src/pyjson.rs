//! JSON text identical to Python's `json.dumps(..., sort_keys=True)` (`ensure_ascii=True`), so the
//! pipeline's files can be compared byte for byte with the reference model's (P2).

use serde_json::Value;
use std::fmt::Write;

fn string(out: &mut String, s: &str) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{8}' => out.push_str("\\b"),
            '\u{c}' => out.push_str("\\f"),
            c if (c as u32) < 0x20 || (c as u32) > 0x7f => {
                let mut buf = [0u16; 2];
                for unit in c.encode_utf16(&mut buf) {
                    let _ = write!(out, "\\u{unit:04x}");
                }
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

fn scalar(out: &mut String, v: &Value) {
    match v {
        Value::Null => out.push_str("null"),
        Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Value::Number(n) => out.push_str(&n.to_string()),
        Value::String(s) => string(out, s),
        _ => unreachable!(),
    }
}

fn sorted(m: &serde_json::Map<String, Value>) -> Vec<(&String, &Value)> {
    let mut v: Vec<_> = m.iter().collect();
    v.sort_by(|a, b| a.0.cmp(b.0));
    v
}

/// `json.dumps(v, sort_keys=True)`: separators `", "` and `": "`.
pub fn compact(v: &Value) -> String {
    let mut out = String::new();
    compact_into(&mut out, v);
    out
}

fn compact_into(out: &mut String, v: &Value) {
    match v {
        Value::Array(items) => {
            out.push('[');
            for (i, x) in items.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                compact_into(out, x);
            }
            out.push(']');
        }
        Value::Object(m) => {
            out.push('{');
            for (i, (k, x)) in sorted(m).into_iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                string(out, k);
                out.push_str(": ");
                compact_into(out, x);
            }
            out.push('}');
        }
        other => scalar(out, other),
    }
}

/// `json.dumps(v, indent=2, sort_keys=True) + "\n"`.
pub fn pretty(v: &Value) -> String {
    let mut out = String::new();
    pretty_into(&mut out, v, 0);
    out.push('\n');
    out
}

fn pretty_into(out: &mut String, v: &Value, depth: usize) {
    let pad = |out: &mut String, d: usize| out.extend(std::iter::repeat_n("  ", d));
    match v {
        Value::Array(items) if items.is_empty() => out.push_str("[]"),
        Value::Object(m) if m.is_empty() => out.push_str("{}"),
        Value::Array(items) => {
            out.push_str("[\n");
            for (i, x) in items.iter().enumerate() {
                pad(out, depth + 1);
                pretty_into(out, x, depth + 1);
                out.push_str(if i + 1 < items.len() { ",\n" } else { "\n" });
            }
            pad(out, depth);
            out.push(']');
        }
        Value::Object(m) => {
            out.push_str("{\n");
            let entries = sorted(m);
            let n = entries.len();
            for (i, (k, x)) in entries.into_iter().enumerate() {
                pad(out, depth + 1);
                string(out, k);
                out.push_str(": ");
                pretty_into(out, x, depth + 1);
                out.push_str(if i + 1 < n { ",\n" } else { "\n" });
            }
            pad(out, depth);
            out.push('}');
        }
        other => scalar(out, other),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn matches_python() {
        assert_eq!(
            compact(&json!({"b": 1, "a": ["x", true]})),
            r#"{"a": ["x", true], "b": 1}"#
        );
        assert_eq!(
            pretty(&json!({"a": [], "b": {"c": "é"}})),
            "{\n  \"a\": [],\n  \"b\": {\n    \"c\": \"\\u00e9\"\n  }\n}\n"
        );
    }
}
