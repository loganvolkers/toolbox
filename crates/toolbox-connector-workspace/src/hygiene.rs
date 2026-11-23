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
fn split_prefix() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(
            r#""(?:gh[pousr]_|github_pat_|AKIA|ASIA|sk_live_|rk_live_|sk-ant-|xox[baprs]-)""#,
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

/// `path:line kind` for each credential in `lines` (line numbers are 0-based on input).
pub fn secrets_in_lines<'a>(
    rel: &str,
    lines: impl IntoIterator<Item = (usize, &'a str)>,
    code: bool,
) -> Vec<String> {
    let mut hits = Vec::new();
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
        // A prefix split off in a comment hides nothing; whole credentials in comments still count.
        let comment = line.trim_start().starts_with("//");
        if code && !comment && split_prefix().is_match(line) {
            hits.push(format!("{rel}:{} split-token-prefix", n + 1));
        }
    }
    hits
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

/// HTML comments: hidden in rendered Markdown, read in full by an agent.
pub fn html_comments(text: &str) -> i64 {
    text.matches("<!--").count() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn credentials_the_old_patterns_missed_are_found() {
        let fake = |p: &str| format!("{p}{}", "a".repeat(36));
        let text = [
            format!("t = \"{}\"", fake("gho_")),
            format!("k = {}", "-----BEGIN RSA PRIVATE KEY-----"),
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
    fn a_split_token_prefix_is_found_in_code() {
        let line = r#"let t = concat!("ghp_", "rest");"#;
        assert_eq!(
            secrets_in_lines("a.rs", [(0, line)], true),
            vec!["a.rs:1 split-token-prefix"]
        );
        assert!(secrets_in_lines("a.md", [(0, line)], false).is_empty());
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
}
