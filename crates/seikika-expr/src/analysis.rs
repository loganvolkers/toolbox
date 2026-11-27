//! Static analysis of one CEL expression (docs/design/scorecards-for-scorecards.md §4): its size,
//! nesting and comprehension depth (a deterministic proxy for evaluation cost), the `entity` fields
//! it reads, and the functions it calls, with those the runtime doesn't declare. `seikika
//! validate` rejects undeclared functions, which otherwise fail only at evaluation time (`join`
//! and `flatten` did, twice, in Seikika for Seikika).

use cel::common::ast::{EntryExpr, Expr, IdedExpr};
use std::collections::BTreeSet;
use std::sync::OnceLock;

/// What an expression is made of.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Analysis {
    /// AST nodes.
    pub nodes: usize,
    /// Longest root-to-leaf path in the AST.
    pub depth: usize,
    /// Deepest nesting of comprehensions (`map`, `filter`, `all`, `exists` …): 2 means a loop
    /// inside a loop, which costs the product of two list lengths per entity.
    pub comprehension_depth: usize,
    /// Fields read as `entity.<field>` (top level only).
    pub fields: Vec<String>,
    /// Every field name selected anywhere (`x.<name>`), e.g. fields of related entities read
    /// through `inbound` lists.
    pub selects: Vec<String>,
    /// Named functions called (operators excluded).
    pub functions: Vec<String>,
    /// Functions the runtime doesn't declare: the expression fails when it reaches them.
    pub undeclared: Vec<String>,
}

#[derive(Default)]
struct Walk {
    nodes: usize,
    depth: usize,
    comprehension_depth: usize,
    fields: BTreeSet<String>,
    selects: BTreeSet<String>,
    functions: BTreeSet<String>,
}

impl Walk {
    fn expr(&mut self, e: &IdedExpr, depth: usize, loops: usize) {
        self.nodes += 1;
        self.depth = self.depth.max(depth);
        self.comprehension_depth = self.comprehension_depth.max(loops);
        for (child, l) in self.children(e, loops) {
            self.expr(child, depth + 1, l);
        }
    }

    /// The sub-expressions of `e` with their loop depth, noting calls and `entity` fields on the way.
    fn children<'e>(&mut self, e: &'e IdedExpr, loops: usize) -> Vec<(&'e IdedExpr, usize)> {
        match &e.expr {
            Expr::Call(c) => {
                if c.func_name.starts_with(|ch: char| ch.is_ascii_alphabetic()) {
                    self.functions.insert(c.func_name.clone());
                }
                c.target
                    .iter()
                    .map(|t| &**t)
                    .chain(&c.args)
                    .map(|x| (x, loops))
                    .collect()
            }
            Expr::Comprehension(c) => std::iter::once((&c.iter_range, loops))
                .chain(
                    [&c.accu_init, &c.loop_cond, &c.loop_step, &c.result].map(|x| (x, loops + 1)),
                )
                .collect(),
            Expr::List(l) => l.elements.iter().map(|x| (x, loops)).collect(),
            Expr::Map(m) => entries(&m.entries)
                .into_iter()
                .map(|x| (x, loops))
                .collect(),
            Expr::Struct(s) => entries(&s.entries)
                .into_iter()
                .map(|x| (x, loops))
                .collect(),
            Expr::Select(s) => {
                self.selects.insert(s.field.clone());
                if matches!(&s.operand.expr, Expr::Ident(root) if root == "entity") {
                    self.fields.insert(s.field.clone());
                }
                vec![(&*s.operand, loops)]
            }
            Expr::Ident(_) | Expr::Literal(_) | Expr::Unspecified => vec![],
        }
    }
}

/// The expressions inside map and struct literal entries.
fn entries(entries: &[cel::common::ast::IdedEntryExpr]) -> Vec<&IdedExpr> {
    entries
        .iter()
        .flat_map(|e| match &e.expr {
            EntryExpr::MapEntry(me) => vec![&me.key, &me.value],
            EntryExpr::StructField(f) => vec![&f.value],
        })
        .collect()
}

/// Whether the default CEL context declares `name`, as a global or a member function.
pub fn declared(name: &str) -> bool {
    static CACHE: OnceLock<std::sync::Mutex<std::collections::HashMap<String, bool>>> =
        OnceLock::new();
    let cache = CACHE.get_or_init(Default::default);
    if let Some(&known) = cache
        .lock()
        .ok()
        .and_then(|c| c.get(name).copied())
        .as_ref()
    {
        return known;
    }
    // A global (`size()`) or a member function (`'a'.startsWith()`): either call fails with "no
    // such overload" rather than "undeclared reference" when the function exists.
    let probe = |src: String| match cel::Program::compile(&src) {
        Ok(p) => !matches!(
            p.execute(&cel::Context::default()),
            Err(cel::ExecutionError::UndeclaredReference(_))
        ),
        Err(_) => true, // not a plain identifier; let evaluation decide
    };
    let ok = probe(format!("{name}()")) || probe(format!("'a'.{name}()"));
    if let Ok(mut c) = cache.lock() {
        c.insert(name.to_owned(), ok);
    }
    ok
}

/// Analyse `expr`, or the parse error.
pub fn analyse(expr: &str) -> Result<Analysis, String> {
    let program = cel::Program::compile(expr).map_err(|e| e.to_string())?;
    let mut w = Walk::default();
    w.expr(program.expression(), 1, 0);
    let functions: Vec<String> = w.functions.into_iter().collect();
    let undeclared = functions.iter().filter(|f| !declared(f)).cloned().collect();
    Ok(Analysis {
        nodes: w.nodes,
        depth: w.depth,
        comprehension_depth: w.comprehension_depth,
        fields: w.fields.into_iter().collect(),
        selects: w.selects.into_iter().collect(),
        functions,
        undeclared,
    })
}

/// The `entity.<field> == '<string>'` comparisons that `expr` requires: its top-level `&&`
/// conjuncts of that form, either way round. An entity the expression accepts has every one of
/// them, so a caller may narrow candidates by them before evaluating it.
pub fn required_equalities(expr: &str) -> Vec<(String, String)> {
    fn walk(e: &IdedExpr, out: &mut Vec<(String, String)>) {
        let Expr::Call(c) = &e.expr else { return };
        match (c.func_name.as_str(), c.target.is_none(), c.args.as_slice()) {
            ("_&&_", true, [a, b]) => {
                walk(a, out);
                walk(b, out);
            }
            ("_==_", true, [a, b]) => {
                if let Some(eq) = field_literal(a, b).or_else(|| field_literal(b, a)) {
                    out.push(eq);
                }
            }
            _ => {}
        }
    }
    fn field_literal(f: &IdedExpr, l: &IdedExpr) -> Option<(String, String)> {
        let Expr::Select(s) = &f.expr else {
            return None;
        };
        let Expr::Literal(cel::common::ast::LiteralValue::String(v)) = &l.expr else {
            return None;
        };
        (matches!(&s.operand.expr, Expr::Ident(root) if root == "entity") && !s.test)
            .then(|| (s.field.clone(), v.inner().to_owned()))
    }
    let mut out = Vec::new();
    if let Ok(p) = cel::Program::compile(expr) {
        walk(p.expression(), &mut out);
    }
    out
}

/// A scorecard level or rule outcome required by a fixture assertion.
#[derive(Debug, PartialEq, Eq)]
pub struct ScoreAssertion {
    pub card: String,
    pub entity: String,
    /// A rule and its asserted status, or `evidence` for an asserted diagnostic.
    pub rule: Option<(String, String)>,
}

fn string_literal(e: &IdedExpr) -> Option<&str> {
    match &e.expr {
        Expr::Literal(cel::common::ast::LiteralValue::String(s)) => Some(s.inner()),
        _ => None,
    }
}

fn score_selection(e: &IdedExpr) -> Option<(&str, &str, &str)> {
    let Expr::Select(select) = &e.expr else {
        return None;
    };
    let Expr::Call(score) = &select.operand.expr else {
        return None;
    };
    match score.args.as_slice() {
        [card, id] if score.func_name == "score" && score.target.is_none() && !select.test => {
            Some((string_literal(card)?, string_literal(id)?, &select.field))
        }
        _ => None,
    }
}

fn is_literal(e: &IdedExpr) -> bool {
    match &e.expr {
        Expr::Literal(_) => true,
        Expr::List(list) => list.elements.iter().all(is_literal),
        _ => false,
    }
}

fn score_comparison(left: &IdedExpr, right: &IdedExpr) -> Option<ScoreAssertion> {
    if let Some((card, entity, "level")) = score_selection(left) {
        string_literal(right)?;
        return Some(ScoreAssertion {
            card: card.into(),
            entity: entity.into(),
            rule: None,
        });
    }
    let Expr::Call(index) = &left.expr else {
        return None;
    };
    let [map, key] = index.args.as_slice() else {
        return None;
    };
    if index.func_name != "_[_]" || index.target.is_some() {
        return None;
    }
    let (card, entity, field) = score_selection(map)?;
    let rule = string_literal(key)?;
    let outcome = match field {
        "rules" => match string_literal(right)? {
            o @ ("pass" | "fail" | "unknown" | "skipped" | "waived") => o,
            _ => return None,
        },
        "evidence" if is_literal(right) => "evidence",
        _ => return None,
    };
    Some(ScoreAssertion {
        card: card.into(),
        entity: entity.into(),
        rule: Some((rule.into(), outcome.into())),
    })
}

/// Recognize direct score comparisons in required branches of an assertion. For example, an
/// operand of `||` may be skipped, and text inside a string is never a score comparison.
pub(crate) fn score_assertions(expr: &IdedExpr) -> Vec<ScoreAssertion> {
    fn walk(e: &IdedExpr, out: &mut Vec<ScoreAssertion>) {
        let Expr::Call(c) = &e.expr else { return };
        match (c.func_name.as_str(), c.target.is_none(), c.args.as_slice()) {
            ("_&&_", true, [a, b]) => {
                walk(a, out);
                walk(b, out);
            }
            ("_==_", true, [a, b]) => {
                out.extend(score_comparison(a, b).or_else(|| score_comparison(b, a)));
            }
            _ => {}
        }
    }

    let mut out = Vec::new();
    walk(expr, &mut out);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_only_required_score_comparisons() {
        let assertions = |src| crate::Expr::compile(src).unwrap().score_assertions();
        let a = |card: &str, entity: &str, rule: &str, outcome: &str| ScoreAssertion {
            card: card.into(),
            entity: entity.into(),
            rule: Some((rule.into(), outcome.into())),
        };
        assert_eq!(
            assertions(
                "score('a', 'x:1').rules['first'] == 'pass' &&
                 score('b', 'x:2').evidence['second'] == ['missing'] &&
                 'gold' == score('c', 'x:3').level",
            ),
            [
                a("a", "x:1", "first", "pass"),
                a("b", "x:2", "second", "evidence"),
                ScoreAssertion {
                    card: "c".into(),
                    entity: "x:3".into(),
                    rule: None
                },
            ]
        );
        for source in [
            "true || score('a', 'x:1').rules['first'] == 'pass'",
            "score('a', 'x:1').rules['first'] == score('a', 'x:1').rules['first']",
            "score('a', 'x:1').rules['first'] != 'fail'",
            "score('a', 'x:1').evidence['first'] == score('a', 'x:1').evidence['first']",
            "true || string(\"score('a', 'x:1').rules['first'] == 'pass'\") == ''",
        ] {
            assert!(assertions(source).is_empty(), "{source}");
        }
    }

    #[test]
    fn finds_the_equalities_a_filter_requires() {
        let eq = |f: &str, v: &str| (f.to_owned(), v.to_owned());
        assert_eq!(
            required_equalities("entity.owner == 'group:g3'"),
            [eq("owner", "group:g3")]
        );
        assert_eq!(
            required_equalities("'prod' == entity.tier && entity.lines > 3 && entity.a == 'x'"),
            [eq("tier", "prod"), eq("a", "x")]
        );
        for none in [
            "entity.owner == 'a' || entity.owner == 'b'",
            "!(entity.owner == 'a')",
            "entity.owner == other.owner",
            "entity.size == 3",
            "entity.meta.owner == 'a'",
            "has(entity.owner) == 'a'",
            "not valid CEL ((",
        ] {
            assert!(required_equalities(none).is_empty(), "{none}");
        }
    }

    #[test]
    fn measures_size_fields_and_loops() {
        let a = analyse("entity.functions.filter(f, f.lines > 80).size() == 0 && entity.crateDoc")
            .unwrap();
        assert_eq!(a.fields, ["crateDoc", "functions"]);
        assert_eq!(a.comprehension_depth, 1);
        assert!(a.functions.contains(&"size".to_owned()));
        assert!(a.undeclared.is_empty());
        let nested = analyse("entity.a.all(x, entity.b.exists(y, y == x))").unwrap();
        assert_eq!(nested.comprehension_depth, 2);
        assert!(analyse("entity.x == 1").unwrap().nodes < a.nodes);
    }

    #[test]
    fn finds_undeclared_functions() {
        // Both reached Seikika for Seikika's manifests and failed only at evaluation time.
        assert_eq!(
            analyse("entity.xs.join(', ')").unwrap().undeclared,
            ["join"]
        );
        assert_eq!(
            analyse("[[1], [2]].flatten()").unwrap().undeclared,
            ["flatten"]
        );
        for ok in [
            "entity.s.startsWith('a')",
            "entity.s.matches('^a')",
            "string(1)",
            "size([1])",
            "has(entity.x)",
        ] {
            assert!(analyse(ok).unwrap().undeclared.is_empty(), "{ok}");
        }
    }
}
