//! Static metrics for one crate from a `syn` parse of its sources. Thresholds are not applied
//! here: the connector reports every function and every undocumented public item, and the
//! portal's scorecards decide what "too long" or "too complex" means.

use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::path::Path;
use syn::spanned::Spanned;
use syn::visit::{self, Visit};

/// One function's measurements.
struct Function {
    name: String,
    at: String,
    lines: usize,
    complexity: usize,
}

#[derive(Default)]
struct Metrics {
    files: usize,
    code_lines: usize,
    pub_items: usize,
    undocumented: Vec<String>,
    functions: Vec<Function>,
    unit_tests: usize,
    /// Tests whose body asserts nothing: no `assert*!`/`panic!`, no `.unwrap()`/`.expect()`,
    /// and no `#[should_panic]`. `#[test] fn t() {}` passes `cargo test` and proves nothing.
    tests_without_asserts: Vec<String>,
    unwraps: usize,
    parse_errors: Vec<String>,
}

fn has_doc(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|a| a.path().is_ident("doc"))
}

fn is_test_fn(attrs: &[syn::Attribute]) -> bool {
    attrs
        .iter()
        .any(|a| a.path().segments.last().is_some_and(|s| s.ident == "test"))
}

fn is_cfg_test(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|a| {
        a.path().is_ident("cfg")
            && a.meta
                .require_list()
                .is_ok_and(|l| l.tokens.to_string().trim() == "test")
    })
}

fn lines_of(span: proc_macro2::Span) -> usize {
    span.end().line.saturating_sub(span.start().line) + 1
}

/// Cyclomatic complexity of a function body: 1 + branches.
#[derive(Default)]
struct Complexity(usize);

impl<'ast> Visit<'ast> for Complexity {
    fn visit_expr(&mut self, e: &'ast syn::Expr) {
        match e {
            syn::Expr::If(_) | syn::Expr::While(_) | syn::Expr::ForLoop(_) | syn::Expr::Loop(_) => {
                self.0 += 1
            }
            syn::Expr::Match(m) => self.0 += m.arms.len().saturating_sub(1),
            syn::Expr::Binary(b) if matches!(b.op, syn::BinOp::And(_) | syn::BinOp::Or(_)) => {
                self.0 += 1
            }
            _ => {}
        }
        visit::visit_expr(self, e);
    }
}

struct Walker<'a> {
    file: &'a str,
    m: &'a mut Metrics,
    in_test: bool,
    scope: Vec<String>,
}

impl Walker<'_> {
    fn qualified(&self, name: &str) -> String {
        self.scope
            .iter()
            .map(String::as_str)
            .chain([name])
            .collect::<Vec<_>>()
            .join("::")
    }

    fn public(&mut self, vis: &syn::Visibility, attrs: &[syn::Attribute], name: &str, line: usize) {
        if self.in_test || !matches!(vis, syn::Visibility::Public(_)) {
            return;
        }
        self.m.pub_items += 1;
        if !has_doc(attrs) {
            let q = self.qualified(name);
            self.m
                .undocumented
                .push(format!("{q} ({}:{line})", self.file));
        }
    }

    fn function(
        &mut self,
        attrs: &[syn::Attribute],
        name: &str,
        span: proc_macro2::Span,
        block: &syn::Block,
    ) {
        if is_test_fn(attrs) {
            self.m.unit_tests += 1;
            let mut a = Asserts(false);
            a.visit_block(block);
            let should_panic = attrs.iter().any(|x| x.path().is_ident("should_panic"));
            if !a.0 && !should_panic {
                let q = self.qualified(name);
                self.m.tests_without_asserts.push(format!(
                    "{q} ({}:{})",
                    self.file,
                    span.start().line
                ));
            }
            return;
        }
        if self.in_test {
            return;
        }
        let mut c = Complexity(1);
        c.visit_block(block);
        self.m.functions.push(Function {
            name: self.qualified(name),
            at: format!("{}:{}", self.file, span.start().line),
            lines: lines_of(span),
            complexity: c.0,
        });
    }
}

/// Whether a block asserts anything (see `Metrics::tests_without_asserts`).
struct Asserts(bool);

impl<'ast> Visit<'ast> for Asserts {
    fn visit_macro(&mut self, m: &'ast syn::Macro) {
        if let Some(seg) = m.path.segments.last() {
            let n = seg.ident.to_string();
            if n.starts_with("assert") || n == "panic" || n == "unreachable" {
                self.0 = true;
            }
        }
        visit::visit_macro(self, m);
    }

    fn visit_expr_method_call(&mut self, c: &'ast syn::ExprMethodCall) {
        let n = c.method.to_string();
        if matches!(
            n.as_str(),
            "unwrap" | "expect" | "unwrap_err" | "expect_err"
        ) {
            self.0 = true;
        }
        visit::visit_expr_method_call(self, c);
    }
}

impl<'ast> Visit<'ast> for Walker<'_> {
    fn visit_item(&mut self, item: &'ast syn::Item) {
        let line = item.span().start().line;
        match item {
            syn::Item::Struct(i) => self.public(&i.vis, &i.attrs, &i.ident.to_string(), line),
            syn::Item::Enum(i) => self.public(&i.vis, &i.attrs, &i.ident.to_string(), line),
            syn::Item::Trait(i) => self.public(&i.vis, &i.attrs, &i.ident.to_string(), line),
            syn::Item::Type(i) => self.public(&i.vis, &i.attrs, &i.ident.to_string(), line),
            syn::Item::Const(i) => self.public(&i.vis, &i.attrs, &i.ident.to_string(), line),
            syn::Item::Static(i) => self.public(&i.vis, &i.attrs, &i.ident.to_string(), line),
            syn::Item::Fn(i) => self.public(&i.vis, &i.attrs, &i.sig.ident.to_string(), line),
            _ => {}
        }
        visit::visit_item(self, item);
    }

    fn visit_item_mod(&mut self, m: &'ast syn::ItemMod) {
        let was = self.in_test;
        self.in_test |= is_cfg_test(&m.attrs);
        self.scope.push(m.ident.to_string());
        visit::visit_item_mod(self, m);
        self.scope.pop();
        self.in_test = was;
    }

    fn visit_item_impl(&mut self, i: &'ast syn::ItemImpl) {
        let ty = match &*i.self_ty {
            syn::Type::Path(p) => p
                .path
                .segments
                .last()
                .map(|s| s.ident.to_string())
                .unwrap_or_default(),
            _ => "impl".into(),
        };
        self.scope.push(ty);
        visit::visit_item_impl(self, i);
        self.scope.pop();
    }

    fn visit_item_fn(&mut self, f: &'ast syn::ItemFn) {
        self.function(&f.attrs, &f.sig.ident.to_string(), f.span(), &f.block);
        let was = self.in_test;
        self.in_test |= is_test_fn(&f.attrs);
        visit::visit_item_fn(self, f);
        self.in_test = was;
    }

    fn visit_impl_item_fn(&mut self, f: &'ast syn::ImplItemFn) {
        let line = f.span().start().line;
        self.public(&f.vis, &f.attrs, &f.sig.ident.to_string(), line);
        self.function(&f.attrs, &f.sig.ident.to_string(), f.span(), &f.block);
        visit::visit_impl_item_fn(self, f);
    }

    fn visit_trait_item_fn(&mut self, f: &'ast syn::TraitItemFn) {
        if let Some(block) = &f.default {
            self.function(&f.attrs, &f.sig.ident.to_string(), f.span(), block);
        }
        visit::visit_trait_item_fn(self, f);
    }

    fn visit_expr_method_call(&mut self, c: &'ast syn::ExprMethodCall) {
        if !self.in_test && (c.method == "unwrap" || c.method == "expect") {
            self.m.unwraps += 1;
        }
        visit::visit_expr_method_call(self, c);
    }
}

fn rust_files(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut v: Vec<_> = walkdir::WalkDir::new(dir)
        .into_iter()
        .flatten()
        .filter(|e| e.file_type().is_file() && e.path().extension().is_some_and(|x| x == "rs"))
        .map(|e| e.into_path())
        .collect();
    v.sort();
    v
}

fn parse(path: &Path, m: &mut Metrics, rel: &str) -> Option<syn::File> {
    let text = std::fs::read_to_string(path).ok()?;
    m.files += 1;
    m.code_lines += text
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with("//"))
        .count();
    match syn::parse_file(&text) {
        Ok(f) => Some(f),
        Err(e) => {
            m.parse_errors.push(format!("{rel}: {e}"));
            None
        }
    }
}

/// The first 12 hex digits of the SHA-256 over the crate's `src/` files (paths and contents): an
/// assessment of the crate is stale when it changes.
/// What a crate-design assessment judges: its sources, its tests and its manifest (the
/// dependencies behind `replaceable-defaults`), so a change to any of them makes the review stale.
fn source_hash(dir: &Path) -> String {
    let mut h = Sha256::new();
    let files = rust_files(&dir.join("src"))
        .into_iter()
        .chain(rust_files(&dir.join("tests")))
        .chain(Some(dir.join("Cargo.toml")).filter(|p| p.exists()));
    for path in files {
        h.update(
            path.strip_prefix(dir)
                .unwrap_or(&path)
                .to_string_lossy()
                .as_bytes(),
        );
        h.update(std::fs::read(&path).unwrap_or_default());
    }
    h.finalize()
        .iter()
        .take(6)
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// Metrics for the crate in `dir`; `entry` is its lib or bin root (for crate-level docs).
pub fn analyse(dir: &Path, entry: Option<&Path>) -> Value {
    let mut m = Metrics::default();
    let mut crate_doc = false;
    for path in rust_files(&dir.join("src")) {
        let rel = path
            .strip_prefix(dir)
            .unwrap_or(&path)
            .to_string_lossy()
            .into_owned();
        let Some(file) = parse(&path, &mut m, &rel) else {
            continue;
        };
        if entry.is_some_and(|e| e == path) {
            crate_doc = has_doc(&file.attrs);
        }
        let mut w = Walker {
            file: &rel,
            m: &mut m,
            in_test: false,
            scope: vec![],
        };
        w.visit_file(&file);
    }
    let mut integration_tests = 0;
    let mut it = Metrics::default();
    for path in rust_files(&dir.join("tests")) {
        let rel = path
            .strip_prefix(dir)
            .unwrap_or(&path)
            .to_string_lossy()
            .into_owned();
        if let Some(file) = parse(&path, &mut it, &rel) {
            let mut w = Walker {
                file: &rel,
                m: &mut it,
                in_test: true,
                scope: vec![],
            };
            w.visit_file(&file);
        }
    }
    integration_tests += it.unit_tests;
    m.parse_errors.extend(it.parse_errors);
    m.functions
        .sort_by(|a, b| b.lines.cmp(&a.lines).then(a.name.cmp(&b.name)));
    json!({
        "crateDoc": crate_doc,
        "files": m.files,
        "codeLines": m.code_lines,
        "testCodeLines": it.code_lines,
        "pubItems": m.pub_items,
        "pubDocumented": m.pub_items - m.undocumented.len(),
        "undocumented": m.undocumented,
        "functions": m.functions.iter().map(|f| json!({
            "name": f.name, "at": f.at, "lines": f.lines, "complexity": f.complexity})).collect::<Vec<_>>(),
        "maxFnLines": m.functions.iter().map(|f| f.lines).max().unwrap_or(0),
        "maxComplexity": m.functions.iter().map(|f| f.complexity).max().unwrap_or(0),
        "unitTests": m.unit_tests,
        "testsWithoutAsserts": m.tests_without_asserts.iter().chain(&it.tests_without_asserts).collect::<Vec<_>>(),
        "integrationTests": integration_tests,
        "unwraps": m.unwraps,
        "parseErrors": m.parse_errors,
        "hash": source_hash(dir),
    })
}
