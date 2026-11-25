//! One detector for credentials and hidden text, shared by every record that reports
//! `secretHits` or `hiddenChars` (connectors, skills, agent instruction files, docs pages,
//! examples, npm manifests, plugins, schemas), so a gap is fixed once.
//!
//! Why not an existing tool: gitleaks and trufflehog are the right home for secret scanning, but
//! running them is not available in every environment this connector runs in; these patterns are
//! the floor, not the scanner (docs/design/meta-standards-review.md, M4).

use regex::Regex;
use std::sync::OnceLock;

/// Credential shapes: provider tokens with fixed prefixes, private keys, and URLs with a password.
fn patterns() -> &'static [(&'static str, Regex)] {
    static P: OnceLock<Vec<(&'static str, Regex)>> = OnceLock::new();
    P.get_or_init(|| {
        [
            ("aws-access-key", r"\b(?:AKIA|ASIA)[0-9A-Z]{16}\b"),
            ("github-token", r"\bgh[pousr]_[A-Za-z0-9]{36}\b"),
            ("github-token", r"\bgithub_pat_[A-Za-z0-9_]{22,}"),
            ("api-key", r"\bsk-(?:ant|proj|live)-[A-Za-z0-9_-]{20,}"),
            ("api-key", r"\bsk-[A-Za-z0-9]{32,}\b"),
            ("google-api-key", r"\bAIza[0-9A-Za-z_-]{35}\b"),
            ("stripe-key", r"\b(?:sk|rk)_live_[A-Za-z0-9]{16,}"),
            ("slack-token", r"\bxox[baprs]-[A-Za-z0-9-]{10,}"),
            ("private-key", r"-----BEGIN (?:[A-Z]+ )*PRIVATE KEY-----"),
            (
                "url-password",
                r"\b[a-z][a-z0-9+.-]*://[^\s/:@]+:([^\s/@]{3,})@([^\s/:@]+)",
            ),
        ]
        .into_iter()
        .map(|(k, p)| (k, Regex::new(p).expect("static regex")))
        .collect()
    })
}

/// A string literal that is exactly a token prefix: the cheapest way to dodge a whole-token
/// pattern is `concat!("ghp_", "…")` or `"ghp_" + rest`.
fn literal() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r#""((?:[^"\\]|\\.)*)""#).expect("static regex"))
}

fn split_prefix() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(
            r#""(?:gh[pousr]_|github_pat_|AKIA|ASIA|AIza|sk_live_|rk_live_|sk-ant-|sk-|xox[baprs]-)""#,
        )
        .expect("static regex")
    })
}

/// Placeholders in documentation URLs are not credentials (`user:pass@example.com`,
/// `postgres:password@localhost`, `${TOKEN}`).
fn placeholder(password: &str, host: &str) -> bool {
    let p = password.to_ascii_lowercase();
    let h = host.to_ascii_lowercase();
    ["localhost", "127.0.0.1", "0.0.0.0", "example."]
        .iter()
        .any(|x| h.starts_with(x))
        || ["pass", "password", "secret", "token", "changeme", "xxx"].contains(&p.as_str())
        || p.starts_with('$')
        || p.starts_with('<')
        || p.starts_with('{')
        || p.starts_with('%')
}

/// A name that says it holds a credential (`token`, `secret`, `password`, `api_key`) bound to a
/// string literal: `let password = "…"`, `const SCIM_TOKEN: &str = "…"`, `"apiKey": "…"`,
/// `token: "…"`. Prefixless credentials have no other shape to match.
fn assignment() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(
            r#"(?i)\b\w*(?:token|secret|password|passwd|api_?key)\w*["']?\s*(?::\s*&?(?:'static\s+)?str\s*)?[:=]\s*["']([^"'\s]{16,})["']"#,
        )
        .expect("static regex")
    })
}

/// Bits per character of `s` (Shannon entropy over its characters).
fn entropy(s: &str) -> f64 {
    let mut counts = std::collections::HashMap::new();
    for c in s.chars() {
        *counts.entry(c).or_insert(0usize) += 1;
    }
    let n = s.chars().count() as f64;
    counts
        .values()
        .map(|&k| {
            let p = k as f64 / n;
            -p * p.log2()
        })
        .sum()
}

/// A value that reads as a fixture or a placeholder rather than a credential: it says so (`fake`,
/// `example`, `test`, `dummy`, `placeholder`, `xxxx`), or one character repeats through it.
fn fixture(value: &str) -> bool {
    let v = value.to_ascii_lowercase();
    [
        "fake",
        "example",
        "test",
        "dummy",
        "placeholder",
        "xxxx",
        "redacted",
    ]
    .iter()
    .any(|w| v.contains(w))
        || v.chars()
            .rev()
            .take(16)
            .collect::<std::collections::HashSet<_>>()
            .len()
            <= 1
}

/// Whether the literal bound to a credential-named identifier looks random enough to be one:
/// letters and digits, no path or template syntax, and at least 3.5 bits per character.
fn assigned_secret(line: &str) -> bool {
    assignment().captures_iter(line).any(|c| {
        let v = &c[1];
        v.chars().any(|c| c.is_ascii_digit())
            && v.chars().any(|c| c.is_ascii_alphabetic())
            && !v.contains(['/', '.', '$', '{', '<', '%'])
            && !fixture(v)
            && entropy(v) >= 3.5
    })
}

/// Whole provider tokens in test code, where fixtures are expected: a match counts unless it
/// reads as a fixture (see [`fixture`]). A real token is published with the tree wherever it sits.
pub fn test_secrets_in_lines<'a>(
    rel: &str,
    lines: impl IntoIterator<Item = (usize, &'a str)>,
) -> Vec<String> {
    let mut hits = Vec::new();
    for (n, line) in lines {
        for (kind, re) in patterns() {
            if *kind != "url-password" && re.find_iter(line).any(|m| !fixture(m.as_str())) {
                hits.push(format!("{rel}:{} {kind}", n + 1));
            }
        }
    }
    hits.dedup();
    hits
}

/// `path:line kind` for each credential in `lines` (line numbers are 0-based on input).
pub fn secrets_in_lines<'a>(
    rel: &str,
    lines: impl IntoIterator<Item = (usize, &'a str)>,
    code: bool,
) -> Vec<String> {
    let mut hits = Vec::new();
    // A `concat!(` that spans lines: its literals are joined across them until it closes.
    let mut open: Option<(usize, i32, String)> = None;
    for (n, line) in lines {
        for (kind, re) in patterns() {
            let found = if *kind == "url-password" {
                re.captures_iter(line).any(|c| !placeholder(&c[1], &c[2]))
            } else {
                re.is_match(line)
            };
            if found {
                hits.push(format!("{rel}:{} {kind}", n + 1));
            }
        }
        if assigned_secret(line) {
            hits.push(format!("{rel}:{} assigned-secret", n + 1));
        }
        // A prefix split off in a comment hides nothing; whole credentials in comments still count.
        let comment = line.trim_start().starts_with("//");
        if !code || comment {
            continue;
        }
        let literals: Vec<String> = literal()
            .captures_iter(line)
            .map(|c| c[1].to_owned())
            .collect();
        // The line's string literals joined: `concat!("gh", "p_…")` or `"gh" + "p_…"` is the
        // credential `ghp_…`, which no single literal shows.
        if split_prefix().is_match(line)
            || (literals.len() > 1 && joins_a_token(&literals.concat()))
        {
            hits.push(format!("{rel}:{} split-token-prefix", n + 1));
        }
        let depth = |t: &str| t.matches('(').count() as i32 - t.matches(')').count() as i32;
        if let Some((start, d, mut buf)) = open.take() {
            buf.push_str(&literals.concat());
            let d = d + depth(line);
            if d > 0 {
                open = Some((start, d, buf));
            } else if joins_a_token(&buf) {
                hits.push(format!("{rel}:{} split-token-prefix", start + 1));
            }
        } else if let Some(at) = line.find("concat!(") {
            let d = depth(&line[at..]);
            if d > 0 {
                open = Some((n, d, literals.concat()));
            }
        }
    }
    hits.dedup();
    hits
}

/// Whether literals joined together make a credential or start with a token prefix.
fn joins_a_token(joined: &str) -> bool {
    split_prefix().is_match(&format!("\"{joined}\""))
        || patterns()
            .iter()
            .any(|(k, re)| *k != "url-password" && re.is_match(joined))
}

/// Credentials in a text file (documentation, configuration).
pub fn secrets(rel: &str, text: &str) -> Vec<String> {
    secrets_in_lines(rel, text.lines().enumerate(), false)
}

/// Characters a reader doesn't see and a model does: Unicode's default-ignorable code points
/// (zero-width, joiners, word joiners, invisible operators, Hangul fillers, tags), bidirectional
/// controls (including U+061C), and variation selectors, except U+FE0F right after a symbol or
/// emoji, where it only selects the emoji presentation.
pub fn hidden_count(text: &str) -> i64 {
    let mut n = 0;
    let mut prev = ' ';
    for c in text.chars() {
        let u = c as u32;
        let emoji_presentation = u == 0xFE0F && (prev as u32) >= 0x2000;
        if !emoji_presentation && is_hidden(u) {
            n += 1;
        }
        prev = c;
    }
    n
}

fn is_hidden(u: u32) -> bool {
    matches!(u,
        0x00AD | 0x034F | 0x061C | 0x115F | 0x1160 | 0x17B4 | 0x17B5 | 0x180B..=0x180F
        | 0x200B..=0x200F | 0x202A..=0x202E | 0x2060..=0x206F | 0x3164 | 0xFE00..=0xFE0F
        | 0xFEFF | 0xFFA0 | 0xFFF0..=0xFFF8 | 0x1BCA0..=0x1BCA3 | 0x1D173..=0x1D17A
        | 0xE0000..=0xE0FFF)
}

/// Comments hidden in rendered Markdown and read in full by an agent: HTML comments (`<!--`) and
/// link-reference definitions that point nowhere (`[//]: # (...)`, `[x]: <> (...)`, any label),
/// outside fenced code blocks, which a reader sees as written.
pub fn html_comments(text: &str) -> i64 {
    static REF: OnceLock<Regex> = OnceLock::new();
    let reference = REF.get_or_init(|| {
        Regex::new(r"^\s{0,3}\[[^\]]+\]:\s*(?:#|<>)(?:\s|$)").expect("static regex")
    });
    let (mut n, mut fenced) = (0, false);
    for line in text.lines() {
        let t = line.trim_start();
        if t.starts_with("```") || t.starts_with("~~~") {
            fenced = !fenced;
            continue;
        }
        if !fenced {
            n += line.matches("<!--").count() as i64 + i64::from(reference.is_match(line));
        }
    }
    n
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn credentials_the_old_patterns_missed_are_found() {
        let fake = |p: &str| format!("{p}{}", "a".repeat(36));
        let text = [
            format!("t = \"{}\"", fake("gho_")),
            format!("k = -----BEGIN RSA {} KEY-----", "PRIVATE"),
            format!("s = sk_live_{}", "b".repeat(24)),
            "db = postgres://admin:hunter22@db.internal.corp:5432/x".to_owned(),
            "docs = postgres://postgres:password@localhost:5432/dev".to_owned(),
        ]
        .join("\n");
        let hits = secrets("f", &text);
        assert_eq!(
            hits,
            vec![
                "f:1 github-token",
                "f:2 private-key",
                "f:3 stripe-key",
                "f:4 url-password"
            ]
        );
    }

    #[test]
    fn a_random_literal_bound_to_a_credential_name_is_found() {
        let hex: String = (0..40)
            .map(|i| b"0123456789abcdef"[(i * 7 + 3) % 16] as char)
            .collect();
        let mixed = ["Xk29!qPz", "7wLmN3vR"].concat();
        for line in [
            format!(r#"const SCIM_TOKEN: &str = "{hex}";"#),
            format!(r#"let password = "{mixed}";"#),
            format!(r#""apiKey": "{mixed}""#),
        ] {
            assert_eq!(secrets("f", &line), vec!["f:1 assigned-secret"], "{line}");
        }
        for line in [
            r#"const TOKEN_ENV: &str = "TOOLBOX_CONNECTOR_TOKEN_FILE";"#,
            r#"let token = "test-token-for-the-fixture-1";"#,
            r#"password: "${{ secrets.DB_PASSWORD }}""#,
            r#"let secret = "aaaaaaaaaaaaaaaaaaaa1";"#,
        ] {
            assert!(secrets("f", line).is_empty(), "{line}");
        }
    }

    #[test]
    fn a_real_token_in_test_code_counts_and_a_fixture_does_not() {
        let real: String = (0..36)
            .map(|i| b"aZ09bY18cX27"[(i * 5 + 1) % 12] as char)
            .collect();
        let line = format!(r#"let t = "ghp_{real}";"#);
        assert_eq!(
            test_secrets_in_lines("a.rs", [(0, line.as_str())]),
            vec!["a.rs:1 github-token"]
        );
        let fake = format!(r#"let t = "ghp_{}";"#, "a".repeat(36));
        assert!(test_secrets_in_lines("a.rs", [(0, fake.as_str())]).is_empty());
    }

    #[test]
    fn a_split_token_prefix_is_found_in_code() {
        let line = r#"let t = concat!("ghp_", "rest");"#;
        assert_eq!(
            secrets_in_lines("a.rs", [(0, line)], true),
            vec!["a.rs:1 split-token-prefix"]
        );
        assert!(secrets_in_lines("a.md", [(0, line)], false).is_empty());
        let token = format!("{}{}", "p_", "a".repeat(36));
        for line in [
            format!(r#"let t = concat!("gh", "{token}");"#),
            format!(r#"let t = String::from("g") + "h" + "{token}";"#),
        ] {
            assert_eq!(
                secrets_in_lines("a.rs", [(0, line.as_str())], true),
                vec!["a.rs:1 split-token-prefix"],
                "{line}"
            );
        }
        let lines = [
            (0, "let t = concat!("),
            (1, "    \"gh\","),
            (2, "    \"p_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\","),
            (3, ");"),
        ];
        assert_eq!(
            secrets_in_lines("a.rs", lines, true),
            vec!["a.rs:1 split-token-prefix"]
        );
        let google = format!("k = \"AIza{}\"", "b".repeat(35));
        assert_eq!(secrets("f", &google), vec!["f:1 google-api-key"]);
        let fine = r#"let p = format!("{}/{}", "gh", "pages");"#;
        assert!(secrets_in_lines("a.rs", [(0, fine)], true).is_empty());
    }

    #[test]
    fn hidden_characters_include_word_joiners_bidi_and_variation_selectors() {
        assert_eq!(hidden_count("a\u{2060}b\u{061C}c\u{E0100}d"), 3);
        assert_eq!(
            hidden_count("ok \u{2705}\u{FE0F}"),
            0,
            "emoji presentation is visible"
        );
        assert_eq!(hidden_count("x\u{FE0F}"), 1);
        assert_eq!(html_comments("a <!-- do this --> b"), 1);
    }

    #[test]
    fn markdown_comments_of_both_kinds_count() {
        let text = "# Skill\n<!-- hidden -->\n[//]: # (do something else)\n  [comment]: <> (and this)\n[x]: # (renamed)\n[docs]: https://example.com\n```md\n<!-- shown in a fence -->\n```\n";
        assert_eq!(html_comments(text), 4);
    }
}
