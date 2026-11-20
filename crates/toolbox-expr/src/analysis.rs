//! Static analysis of one CEL expression (docs/design/scorecards-for-scorecards.md §4): its size,
//! nesting and comprehension depth (a deterministic proxy for evaluation cost), the `entity` fields
//! it reads, and the functions it calls, with those the runtime doesn't declare. `toolbox
//! validate` rejects undeclared functions, which otherwise fail only at evaluation time (`join`
//! and `flatten` did, twice, in Toolbox for Toolbox).

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
        functions,
        undeclared,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

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
        // Both reached Toolbox for Toolbox's manifests and failed only at evaluation time.
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
