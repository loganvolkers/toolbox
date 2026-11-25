//! OpenFGA modelling language (schema 1.1) → authorization model JSON, the form OpenFGA's API
//! accepts. Covers types, relations, direct types (`user`, `user:*`, `group#member`), computed
//! relations, `X from Y`, `or`, `and`, `but not` and parentheses. Conditions and modules are
//! rejected with a clear error. Checked against `fga model transform` (tests/fga_dsl.rs).

use serde_json::{Map, Value, json};

#[derive(Debug, Clone, PartialEq)]
enum Tok {
    Ident(String),
    LBracket,
    RBracket,
    LParen,
    RParen,
    Comma,
}

fn lex(s: &str, line: usize) -> Result<Vec<Tok>, String> {
    let mut out = Vec::new();
    let mut chars = s.chars().peekable();
    while let Some(&c) = chars.peek() {
        match c {
            ' ' | '\t' => {
                chars.next();
            }
            '[' => {
                chars.next();
                out.push(Tok::LBracket)
            }
            ']' => {
                chars.next();
                out.push(Tok::RBracket)
            }
            '(' => {
                chars.next();
                out.push(Tok::LParen)
            }
            ')' => {
                chars.next();
                out.push(Tok::RParen)
            }
            ',' => {
                chars.next();
                out.push(Tok::Comma)
            }
            c if c.is_alphanumeric() || "_-:#*.".contains(c) => {
                let mut id = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_alphanumeric() || "_-:#*.".contains(c) {
                        id.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }
                out.push(Tok::Ident(id));
            }
            other => return Err(format!("line {line}: unexpected character {other:?}")),
        }
    }
    Ok(out)
}

struct Parser {
    toks: Vec<Tok>,
    pos: usize,
    line: usize,
    direct: Option<Vec<Value>>,
}

impl Parser {
    fn peek_ident(&self) -> Option<&str> {
        match self.toks.get(self.pos) {
            Some(Tok::Ident(s)) => Some(s),
            _ => None,
        }
    }

    fn err<T>(&self, msg: &str) -> Result<T, String> {
        Err(format!("line {}: {msg}", self.line))
    }

    fn expr(&mut self) -> Result<Value, String> {
        let first = self.term()?;
        if self.peek_ident() == Some("but") {
            self.pos += 1;
            if self.peek_ident() != Some("not") {
                return self.err("expected `not` after `but`");
            }
            self.pos += 1;
            let subtract = self.term()?;
            return Ok(json!({"difference": {"base": first, "subtract": subtract}}));
        }
        let op = match self.peek_ident() {
            Some("or") => "union",
            Some("and") => "intersection",
            _ => return Ok(first),
        };
        let word = if op == "union" { "or" } else { "and" };
        let mut children = vec![first];
        while self.peek_ident() == Some(word) {
            self.pos += 1;
            children.push(self.term()?);
        }
        if matches!(self.peek_ident(), Some("or" | "and" | "but")) {
            return self.err("mixing `or`, `and` and `but not` needs parentheses");
        }
        Ok(json!({op: {"child": children}}))
    }

    fn term(&mut self) -> Result<Value, String> {
        match self.toks.get(self.pos).cloned() {
            Some(Tok::LParen) => {
                self.pos += 1;
                let e = self.expr()?;
                if self.toks.get(self.pos) != Some(&Tok::RParen) {
                    return self.err("expected `)`");
                }
                self.pos += 1;
                Ok(e)
            }
            Some(Tok::LBracket) => {
                self.pos += 1;
                let mut types = Vec::new();
                loop {
                    match self.toks.get(self.pos).cloned() {
                        Some(Tok::Ident(t)) => {
                            self.pos += 1;
                            if self.peek_ident() == Some("with") {
                                return self.err("conditions (`with`) are not supported yet");
                            }
                            types.push(direct_type(&t));
                        }
                        _ => return self.err("expected a type in `[...]`"),
                    }
                    match self.toks.get(self.pos) {
                        Some(Tok::Comma) => self.pos += 1,
                        Some(Tok::RBracket) => {
                            self.pos += 1;
                            break;
                        }
                        _ => return self.err("expected `,` or `]`"),
                    }
                }
                if self.direct.is_some() {
                    return self.err("a relation can have only one list of direct types");
                }
                self.direct = Some(types);
                Ok(json!({"this": {}}))
            }
            Some(Tok::Ident(rel)) => {
                self.pos += 1;
                if self.peek_ident() == Some("from") {
                    self.pos += 1;
                    let Some(tupleset) = self.peek_ident().map(str::to_owned) else {
                        return self.err("expected a relation after `from`");
                    };
                    self.pos += 1;
                    return Ok(
                        json!({"tupleToUserset": {"tupleset": {"relation": tupleset}, "computedUserset": {"relation": rel}}}),
                    );
                }
                Ok(json!({"computedUserset": {"relation": rel}}))
            }
            _ => self.err("expected a relation, `[types]` or `(`"),
        }
    }
}

fn direct_type(t: &str) -> Value {
    if let Some(base) = t.strip_suffix(":*") {
        json!({"type": base, "wildcard": {}})
    } else if let Some((ty, rel)) = t.split_once('#') {
        json!({"type": ty, "relation": rel})
    } else {
        json!({"type": t})
    }
}

/// A type being parsed: (name, relations, relation metadata).
type TypeDef = (String, Map<String, Value>, Map<String, Value>);

/// Transform a model in the OpenFGA DSL to its JSON form.
pub fn to_json(dsl: &str) -> Result<Value, String> {
    let mut types: Vec<Value> = Vec::new();
    let mut schema = None;
    let mut current: Option<TypeDef> = None;
    let flush = |cur: &mut Option<TypeDef>, types: &mut Vec<Value>| {
        if let Some((name, rels, meta)) = cur.take() {
            types.push(if rels.is_empty() {
                json!({"type": name})
            } else {
                json!({"type": name, "relations": rels, "metadata": {"relations": meta}})
            });
        }
    };
    for (i, raw) in dsl.lines().enumerate() {
        let line_no = i + 1;
        let line = strip_comment(raw).trim();
        if line.is_empty() {
            continue;
        }
        let mut words = line.split_whitespace();
        match words.next() {
            Some("model") => {}
            Some("schema") => schema = words.next().map(str::to_owned),
            Some("module") | Some("extend") => {
                return Err(format!(
                    "line {line_no}: modular models are not supported yet"
                ));
            }
            Some("condition") => {
                return Err(format!("line {line_no}: conditions are not supported yet"));
            }
            Some("type") => {
                flush(&mut current, &mut types);
                let name = words
                    .next()
                    .ok_or(format!("line {line_no}: type needs a name"))?;
                current = Some((name.to_owned(), Map::new(), Map::new()));
            }
            Some("relations") => {}
            Some("define") => {
                let rest = line.trim_start_matches("define").trim();
                let (name, body) = rest
                    .split_once(':')
                    .ok_or(format!("line {line_no}: expected `define <name>: <expr>`"))?;
                let Some((_, rels, meta)) = current.as_mut() else {
                    return Err(format!("line {line_no}: `define` outside a type"));
                };
                let mut p = Parser {
                    toks: lex(body, line_no)?,
                    pos: 0,
                    line: line_no,
                    direct: None,
                };
                let expr = p.expr()?;
                if p.pos != p.toks.len() {
                    return Err(format!(
                        "line {line_no}: unexpected tokens after the expression"
                    ));
                }
                rels.insert(name.trim().to_owned(), expr);
                meta.insert(
                    name.trim().to_owned(),
                    match p.direct {
                        Some(t) => json!({"directly_related_user_types": t}),
                        None => json!({}),
                    },
                );
            }
            Some(other) => return Err(format!("line {line_no}: unexpected `{other}`")),
            None => {}
        }
    }
    flush(&mut current, &mut types);
    match schema.as_deref() {
        Some("1.1") => Ok(json!({"schema_version": "1.1", "type_definitions": types})),
        Some(v) => Err(format!("schema {v} is not supported (use 1.1)")),
        None => Err("missing `schema 1.1`".into()),
    }
}

/// Remove a trailing `# comment` (a `#` preceded by whitespace; `group#member` is not a comment).
fn strip_comment(line: &str) -> &str {
    let bytes = line.as_bytes();
    for i in 0..bytes.len() {
        if bytes[i] == b'#' && (i == 0 || bytes[i - 1].is_ascii_whitespace()) {
            return &line[..i];
        }
    }
    line
}

/// Whether `type_name` defines `relation`.
pub fn has_relation(model: &Value, type_name: &str, relation: &str) -> bool {
    model["type_definitions"]
        .as_array()
        .into_iter()
        .flatten()
        .any(|t| t["type"] == type_name && t["relations"].get(relation).is_some())
}
