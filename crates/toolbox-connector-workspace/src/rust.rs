//! Static metrics for one crate from a `syn` parse of its sources. Thresholds are not applied
//! here: the connector reports every function and every undocumented public item, and the
//! portal's scorecards decide what "too long" or "too complex" means.

use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
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
    /// Tests whose body asserts nothing: no `assert*!`/`panic!` other than a trivial one
    /// (`assert!(true)`, `assert_eq!(x, x)`), no `.unwrap()`/`.expect()`, no `?`, no call to a
    /// helper in the same file that asserts, and no `#[should_panic]`. `#[test] fn t() {}` passes
    /// `cargo test` and proves nothing.
    tests_without_asserts: Vec<String>,
    /// Tests compiled out by an always-false `#[cfg(any())]` or `#[cfg(not(all()))]`, on the test
    /// or a module around it: neither run nor reported as ignored.
    disabled_tests: Vec<String>,
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

/// An always-false `cfg`: the item is compiled out.
fn is_cfg_disabled(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|a| {
        a.path().is_ident("cfg")
            && a.meta.require_list().is_ok_and(|l| {
                let t: String = l.tokens.to_string().split_whitespace().collect();
                matches!(t.as_str(), "any()" | "not(all())" | "miri" | "not(test)")
            })
    })
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
    disabled: bool,
    scope: Vec<String>,
    /// Functions in the crate that assert (see [`helpers`]).
    helpers: &'a HashSet<String>,
    /// The file's source, and the crate's own names (see [`exercises`]).
    text: &'a str,
    items: &'a HashSet<String>,
    lib: &'a str,
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
            if self.disabled || is_cfg_disabled(attrs) {
                let q = self.qualified(name);
                self.m
                    .disabled_tests
                    .push(format!("{q} ({}:{})", self.file, span.start().line));
            }
            let mut a = Asserts::new(self.helpers);
            a.visit_block(block);
            let should_panic = attrs.iter().any(|x| x.path().is_ident("should_panic"));
            let touches = exercises(self.text, span, self.items, self.lib);
            if (!a.found && !should_panic) || !touches {
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
struct Asserts<'h> {
    found: bool,
    helpers: &'h HashSet<String>,
}

impl<'h> Asserts<'h> {
    fn new(helpers: &'h HashSet<String>) -> Self {
        Asserts {
            found: false,
            helpers,
        }
    }
}

/// An assertion that can't fail: `assert!(true)`, or `assert_eq!`/`assert_ne!` whose two sides are
/// the same tokens.
fn trivial(name: &str, tokens: &proc_macro2::TokenStream) -> bool {
    let text = tokens.to_string();
    let mut parts = Vec::new();
    let (mut depth, mut cur) = (0i32, String::new());
    for c in text.chars() {
        match c {
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth -= 1,
            ',' if depth == 0 => {
                parts.push(cur.trim().to_owned());
                cur.clear();
                continue;
            }
            _ => {}
        }
        cur.push(c);
    }
    parts.push(cur.trim().to_owned());
    match name {
        "assert" | "debug_assert" => matches!(parts[0].as_str(), "true" | "! false"),
        "assert_eq" | "assert_ne" | "debug_assert_eq" | "debug_assert_ne" => {
            parts.len() >= 2 && parts[0] == parts[1]
        }
        _ => false,
    }
}

impl<'ast> Visit<'ast> for Asserts<'_> {
    fn visit_macro(&mut self, m: &'ast syn::Macro) {
        if let Some(seg) = m.path.segments.last() {
            let n = seg.ident.to_string();
            let asserts = n.starts_with("assert")
                || n.starts_with("debug_assert")
                || n.starts_with("prop_assert")
                || n == "ensure";
            if (asserts && !trivial(&n, &m.tokens)) || n == "panic" || n == "unreachable" {
                self.found = true;
            }
        }
        visit::visit_macro(self, m);
    }

    fn visit_expr_method_call(&mut self, c: &'ast syn::ExprMethodCall) {
        let n = c.method.to_string();
        if matches!(
            n.as_str(),
            "unwrap" | "expect" | "unwrap_err" | "expect_err"
        ) || self.helpers.contains(&n)
        {
            self.found = true;
        }
        visit::visit_expr_method_call(self, c);
    }

    fn visit_expr_call(&mut self, c: &'ast syn::ExprCall) {
        if let syn::Expr::Path(p) = &*c.func
            && p.path
                .segments
                .last()
                .is_some_and(|s| self.helpers.contains(&s.ident.to_string()))
        {
            self.found = true;
        }
        visit::visit_expr_call(self, c);
    }

    // `?` in a test that returns `Result` fails the test on an error.
    fn visit_expr_try(&mut self, t: &'ast syn::ExprTry) {
        self.found = true;
        visit::visit_expr_try(self, t);
    }
}

/// Non-test functions in a file whose bodies assert (directly or through another such function):
/// table-driven tests put their assertions in a helper like these.
fn helpers(files: &[&syn::File]) -> HashSet<String> {
    struct Fns<'ast>(Vec<(String, &'ast syn::Block)>);
    impl<'ast> Visit<'ast> for Fns<'ast> {
        fn visit_item_fn(&mut self, f: &'ast syn::ItemFn) {
            if !is_test_fn(&f.attrs) {
                self.0.push((f.sig.ident.to_string(), &f.block));
            }
            visit::visit_item_fn(self, f);
        }
        fn visit_impl_item_fn(&mut self, f: &'ast syn::ImplItemFn) {
            self.0.push((f.sig.ident.to_string(), &f.block));
            visit::visit_impl_item_fn(self, f);
        }
    }
    let mut fns = Fns(Vec::new());
    for file in files {
        fns.visit_file(file);
    }
    let mut found = HashSet::new();
    loop {
        let before = found.len();
        for (name, block) in &fns.0 {
            if !found.contains(name) {
                let mut a = Asserts::new(&found);
                a.visit_block(block);
                if a.found {
                    found.insert(name.clone());
                }
            }
        }
        if found.len() == before {
            return found;
        }
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
        let (was, was_disabled) = (self.in_test, self.disabled);
        self.in_test |= is_cfg_test(&m.attrs);
        self.disabled |= is_cfg_disabled(&m.attrs);
        self.scope.push(m.ident.to_string());
        visit::visit_item_mod(self, m);
        self.scope.pop();
        self.in_test = was;
        self.disabled = was_disabled;
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

fn parse(path: &Path, m: &mut Metrics, rel: &str) -> Option<(syn::File, String)> {
    let text = std::fs::read_to_string(path).ok()?;
    m.files += 1;
    m.code_lines += text
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with("//"))
        .count();
    match syn::parse_file(&text) {
        Ok(f) => Some((f, text)),
        Err(e) => {
            m.parse_errors.push(format!("{rel}: {e}"));
            None
        }
    }
}

/// The crate's own names (see [`crate_items`]) plus every helper that calls one: a test's helper
/// (`fn errs(..)` in its module, or in `tests/common`) that calls the crate counts as the crate's,
/// so a test calling it exercises the crate too.
fn exercised_names(
    src: &[(String, syn::File, String)],
    tests: &[(String, syn::File, String)],
    lib: &str,
) -> HashSet<String> {
    let mut items = crate_items(&src.iter().map(|(_, f, _)| f).collect::<Vec<_>>());
    let mut via = Vec::new();
    for (_, file, text) in src.iter().chain(tests) {
        for (name, span) in fn_spans(file) {
            if !items.contains(&name) && exercises(text, span, &items, lib) {
                via.push(name);
            }
        }
    }
    items.extend(via);
    items
}

/// The Rust files under `dir/sub`, parsed: `(path relative to dir, file, source)`.
fn parse_all(dir: &Path, sub: &str, m: &mut Metrics) -> Vec<(String, syn::File, String)> {
    let mut out = Vec::new();
    for path in rust_files(&dir.join(sub)) {
        let rel = path
            .strip_prefix(dir)
            .unwrap_or(&path)
            .to_string_lossy()
            .into_owned();
        if let Some((file, text)) = parse(&path, m, &rel) {
            out.push((rel, file, text));
        }
    }
    out
}

/// Names the crate defines outside its tests (functions, types, traits, constants, modules,
/// macros): a test that names none of them, and doesn't run the crate's binary, checks nothing
/// this crate does.
fn crate_items(files: &[&syn::File]) -> HashSet<String> {
    struct Items(HashSet<String>, bool);
    impl<'ast> Visit<'ast> for Items {
        fn visit_item(&mut self, item: &'ast syn::Item) {
            let name = match item {
                syn::Item::Fn(i) if !is_test_fn(&i.attrs) => Some(i.sig.ident.to_string()),
                syn::Item::Struct(i) => Some(i.ident.to_string()),
                syn::Item::Enum(i) => Some(i.ident.to_string()),
                syn::Item::Trait(i) => Some(i.ident.to_string()),
                syn::Item::Type(i) => Some(i.ident.to_string()),
                syn::Item::Const(i) => Some(i.ident.to_string()),
                syn::Item::Static(i) => Some(i.ident.to_string()),
                syn::Item::Macro(i) => i.ident.as_ref().map(|x| x.to_string()),
                _ => None,
            };
            if !self.1 {
                self.0.extend(name);
            }
            visit::visit_item(self, item);
        }
        fn visit_item_mod(&mut self, m: &'ast syn::ItemMod) {
            let was = self.1;
            self.1 |= is_cfg_test(&m.attrs);
            visit::visit_item_mod(self, m);
            self.1 = was;
        }
        fn visit_impl_item_fn(&mut self, f: &'ast syn::ImplItemFn) {
            if !self.1 {
                self.0.insert(f.sig.ident.to_string());
            }
            visit::visit_impl_item_fn(self, f);
        }
    }
    let mut items = Items(HashSet::new(), false);
    for f in files {
        items.visit_file(f);
    }
    items.0
}

/// Every non-test function in a file (helpers included) with its span.
fn fn_spans(file: &syn::File) -> Vec<(String, proc_macro2::Span)> {
    struct Fns(Vec<(String, proc_macro2::Span)>);
    impl<'ast> Visit<'ast> for Fns {
        fn visit_item_fn(&mut self, f: &'ast syn::ItemFn) {
            if !is_test_fn(&f.attrs) {
                self.0.push((f.sig.ident.to_string(), f.span()));
            }
            visit::visit_item_fn(self, f);
        }
    }
    let mut fns = Fns(Vec::new());
    fns.visit_file(file);
    fns.0
}

/// Whether a test's source text names something the crate defines, or runs its binary.
fn exercises(text: &str, span: proc_macro2::Span, items: &HashSet<String>, lib: &str) -> bool {
    let (from, to) = (span.start().line.saturating_sub(1), span.end().line);
    let body: Vec<&str> = text.lines().skip(from).take(to - from).collect();
    let body = body.join("\n");
    if body.contains("CARGO_BIN_EXE_") || body.contains(&format!("{lib}::")) {
        return true;
    }
    body.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
        .any(|w| !w.is_empty() && items.contains(w))
}

/// The first 12 hex digits of the SHA-256 over what a crate-design assessment judges: the crate's
/// `src/` and `tests/` files and its `Cargo.toml` (the dependencies behind
/// `replaceable-defaults`), paths and contents, so a change to any of them makes the review stale.
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
    let mut it = Metrics::default();
    let mut crate_doc = false;
    let src = parse_all(dir, "src", &mut m);
    if let Some(e) = entry {
        let rel = e.strip_prefix(dir).unwrap_or(e).to_string_lossy();
        crate_doc = src.iter().any(|(r, f, _)| *r == rel && has_doc(&f.attrs));
    }
    let tests = parse_all(dir, "tests", &mut it);
    let all: Vec<&syn::File> = src.iter().chain(&tests).map(|(_, f, _)| f).collect();
    let helpers = helpers(&all);
    let lib = dir
        .file_name()
        .map(|n| n.to_string_lossy().replace('-', "_"))
        .unwrap_or_default();
    let items = exercised_names(&src, &tests, &lib);
    for (rel, file, text) in &src {
        let mut w = Walker {
            file: rel,
            m: &mut m,
            in_test: false,
            disabled: false,
            scope: vec![],
            helpers: &helpers,
            text,
            items: &items,
            lib: &lib,
        };
        w.visit_file(file);
    }
    let mut integration_tests = 0;
    for (rel, file, text) in &tests {
        let mut w = Walker {
            file: rel,
            m: &mut it,
            in_test: true,
            disabled: false,
            scope: vec![],
            helpers: &helpers,
            text,
            items: &items,
            lib: &lib,
        };
        w.visit_file(file);
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
        "disabledTests": m.disabled_tests.iter().chain(&it.disabled_tests).collect::<Vec<_>>(),
        "integrationTests": integration_tests,
        "unwraps": m.unwraps,
        "parseErrors": m.parse_errors,
        "hash": source_hash(dir),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tests_must_assert_something_that_can_fail_and_disabled_tests_are_found() {
        let dir = std::env::temp_dir().join(format!("toolbox-rust-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("src")).unwrap();
        std::fs::write(
            dir.join("src/lib.rs"),
            r#"
pub fn add(a: i32, b: i32) -> i32 { a + b }
#[cfg(test)]
mod tests {
    use super::*;
    fn check(a: i32, b: i32, want: i32) { assert_eq!(add(a, b), want); }
    #[test] fn table() { for (a, b, w) in [(1, 2, 3)] { check(a, b, w); } }
    #[test] fn fallible() -> Result<(), String> { let n: i32 = "1".parse().map_err(|_| "no".to_string())?; (add(n, 1) == 2).then_some(()).ok_or("bad".into()) }
    #[test] fn real() { assert_eq!(add(1, 1), 2); }
    #[test] fn vacuous() { assert!(true); }
    #[test] fn same_sides() { let x = add(1, 1); assert_eq!(x, x); }
    #[test] fn empty() {}
    #[cfg(any())] #[test] fn off() { assert_eq!(add(1, 1), 3); }
    #[cfg(miri)] #[test] fn miri_only() { assert_eq!(add(1, 1), 3); }
    #[test] fn unrelated() { "1".parse::<i32>().unwrap(); }
    #[test] fn ensured() -> anyhow::Result<()> { anyhow::ensure!(add(1, 1) == 2); Ok(()) }
}
#[cfg(not(all()))]
mod gone { #[test] fn also_off() { assert_eq!(super::add(0, 0), 1); } }
"#,
        )
        .unwrap();
        let m = analyse(&dir, Some(&dir.join("src/lib.rs")));
        let names = |k: &str| -> Vec<String> {
            m[k].as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_str().unwrap().split(' ').next().unwrap().to_owned())
                .collect()
        };
        assert_eq!(
            names("testsWithoutAsserts"),
            [
                "tests::vacuous",
                "tests::same_sides",
                "tests::empty",
                "tests::unrelated"
            ]
        );
        assert_eq!(
            names("disabledTests"),
            ["tests::off", "tests::miri_only", "gone::also_off"]
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}
