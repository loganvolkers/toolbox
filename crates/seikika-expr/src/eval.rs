//! CEL evaluation behind Seikika-owned types: [`ExprValue`] (converted from and to JSON),
//! [`Scope`] (bound variables), [`Env`] (extra functions) and [`Expr`]. Callers never name a
//! `cel` type, so the CEL implementation (cel-rust today, Q10) can be swapped inside this crate.

use cel::objects::{Key, Map};
use cel::{Context, ExecutionError, Program, ResolveResult, Value};
use std::collections::HashMap;
use std::sync::Arc;

/// A value an expression reads or returns. Built from JSON; integers become CEL `int`, as in
/// cel-python's `json_to_cel`.
#[derive(Clone, PartialEq)]
pub struct ExprValue(pub(crate) Value);

impl std::fmt::Debug for ExprValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl ExprValue {
    /// Convert JSON into an expression value.
    pub fn from_json(v: &serde_json::Value) -> Self {
        ExprValue(to_cel(v))
    }

    /// Convert back to JSON; values with no JSON form (durations, types) are an error.
    pub fn to_json(&self) -> Result<serde_json::Value, String> {
        from_cel(&self.0)
    }

    /// A map from string keys.
    pub fn map(entries: impl IntoIterator<Item = (String, ExprValue)>) -> Self {
        ExprValue(map_value(entries.into_iter().map(|(k, v)| (k, v.0))))
    }

    /// The boolean, if this is one.
    pub fn as_bool(&self) -> Option<bool> {
        match self.0 {
            Value::Bool(b) => Some(b),
            _ => None,
        }
    }
}

fn to_cel(v: &serde_json::Value) -> Value {
    match v {
        serde_json::Value::Null => Value::Null,
        serde_json::Value::Bool(b) => Value::Bool(*b),
        serde_json::Value::Number(n) => match (n.as_i64(), n.as_u64()) {
            (Some(i), _) => Value::Int(i),
            (None, Some(u)) => Value::UInt(u),
            _ => Value::Float(n.as_f64().unwrap_or(f64::NAN)),
        },
        serde_json::Value::String(s) => Value::String(Arc::new(s.clone())),
        serde_json::Value::Array(items) => {
            Value::List(Arc::new(items.iter().map(to_cel).collect()))
        }
        serde_json::Value::Object(map) => {
            map_value(map.iter().map(|(k, v)| (k.clone(), to_cel(v))))
        }
    }
}

fn map_value(entries: impl IntoIterator<Item = (String, Value)>) -> Value {
    let map: HashMap<Key, Value> = entries
        .into_iter()
        .map(|(k, v)| (Key::String(Arc::new(k)), v))
        .collect();
    Value::Map(Map { map: Arc::new(map) })
}

fn from_cel(v: &Value) -> Result<serde_json::Value, String> {
    Ok(match v {
        Value::Null => serde_json::Value::Null,
        Value::Bool(b) => (*b).into(),
        Value::Int(i) => (*i).into(),
        Value::UInt(u) => (*u).into(),
        Value::Float(f) => serde_json::Number::from_f64(*f)
            .map(Into::into)
            .ok_or("non-finite float")?,
        Value::String(s) => s.as_str().into(),
        Value::List(items) => items
            .iter()
            .map(from_cel)
            .collect::<Result<Vec<_>, _>>()?
            .into(),
        Value::Map(m) => {
            let mut out = serde_json::Map::new();
            for (k, v) in m.map.iter() {
                let key = match k {
                    Key::String(s) => s.as_str().to_owned(),
                    Key::Int(i) => i.to_string(),
                    Key::Uint(u) => u.to_string(),
                    Key::Bool(b) => b.to_string(),
                };
                out.insert(key, from_cel(v)?);
            }
            out.into()
        }
        other => return Err(format!("CEL value {other:?} has no JSON form")),
    })
}

/// Variables bound for evaluation, from [`with_vars`], [`with_vals`] or [`Env::scope`].
#[derive(Clone, Copy)]
pub struct Scope<'s>(&'s Context<'s, 's>);

/// A variable whose elements are converted only when an expression visits them (see
/// [`crate::lazy_map`]).
pub struct Lazy(pub(crate) Box<dyn cel::common::value::Val>);

thread_local! {
    static ROOT: Context<'static, 'static> = Context::default();
}

/// Bind `vars` once (each is converted into CEL's runtime form here) in a child of a per-thread
/// root scope with the standard library, and run `f` with it.
pub fn with_vars<R>(vars: &[(&str, &ExprValue)], f: impl FnOnce(&Scope<'_>) -> R) -> R {
    with_vals(vars, Vec::new(), f)
}

/// Like [`with_vars`], plus lazy variables (e.g. [`crate::lazy_map`]).
pub fn with_vals<R>(
    vars: &[(&str, &ExprValue)],
    vals: Vec<(&str, Lazy)>,
    f: impl FnOnce(&Scope<'_>) -> R,
) -> R {
    ROOT.with(|root| {
        let mut ctx = root.new_inner_scope();
        for (name, value) in vars {
            ctx.add_variable_from_value(*name, value.0.clone());
        }
        for (name, val) in vals {
            ctx.add_variable_as_val(name, val.0);
        }
        f(&Scope(&ctx))
    })
}

/// A standard library plus caller functions over strings that return JSON, such as
/// `seikika test`'s `entity(id)` and `score(card, id)`.
#[derive(Default)]
pub struct Env(Context<'static, 'static>);

type Fallible = Result<serde_json::Value, String>;

impl Env {
    fn result(name: &'static str, r: Fallible) -> ResolveResult {
        r.map(|v| to_cel(&v))
            .map_err(|m| ExecutionError::function_error(name, m))
    }

    /// Register `name()`. Fails if it would shadow a standard function.
    pub fn function0(
        &mut self,
        name: &'static str,
        f: impl Fn() -> Fallible + Send + Sync + 'static,
    ) -> Result<(), String> {
        self.0
            .add_function(name, move || -> ResolveResult { Self::result(name, f()) })
            .map_err(|e| e.to_string())
    }

    /// Register `name(a)` over a string argument.
    pub fn function1(
        &mut self,
        name: &'static str,
        f: impl Fn(&str) -> Fallible + Send + Sync + 'static,
    ) -> Result<(), String> {
        self.0
            .add_function(name, move |a: Arc<String>| -> ResolveResult {
                Self::result(name, f(&a))
            })
            .map_err(|e| e.to_string())
    }

    /// Register `name(a, b)` over two string arguments.
    pub fn function2(
        &mut self,
        name: &'static str,
        f: impl Fn(&str, &str) -> Fallible + Send + Sync + 'static,
    ) -> Result<(), String> {
        self.0
            .add_function(
                name,
                move |a: Arc<String>, b: Arc<String>| -> ResolveResult {
                    Self::result(name, f(&a, &b))
                },
            )
            .map_err(|e| e.to_string())
    }

    /// The scope to evaluate in.
    pub fn scope(&self) -> Scope<'_> {
        Scope(&self.0)
    }
}

/// A compiled expression, shareable across threads.
#[derive(Clone)]
pub struct Expr {
    pub source: String,
    program: Arc<Program>,
}

impl std::fmt::Debug for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Expr({:?})", self.source)
    }
}

impl Expr {
    /// Parse `source` once; the error is the parser's first line, for a diagnostic.
    pub fn compile(source: &str) -> Result<Expr, String> {
        Program::compile(source)
            .map(|p| Expr {
                source: source.to_owned(),
                program: Arc::new(p),
            })
            .map_err(|e| e.to_string().lines().next().unwrap_or_default().to_owned())
    }

    /// Evaluate with the given variables. Binding converts values into CEL's runtime form, so when
    /// several expressions share variables, bind once with [`with_vars`] and call [`Expr::eval_in`].
    pub fn eval(&self, vars: &[(&str, &ExprValue)]) -> Result<ExprValue, String> {
        with_vars(vars, |ctx| self.eval_in(ctx))
    }

    /// Evaluate in a caller-provided scope (e.g. one with test functions registered).
    pub fn eval_in(&self, scope: &Scope<'_>) -> Result<ExprValue, String> {
        self.program
            .execute(scope.0)
            .map(ExprValue)
            .map_err(|e| e.to_string())
    }

    /// Evaluate and convert the result to JSON.
    pub fn json_in(&self, scope: &Scope<'_>) -> Result<serde_json::Value, String> {
        self.eval_in(scope)?.to_json()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn lazy_lists_size_and_iterate() {
        let items = Arc::new(vec![
            ExprValue::from_json(&json!({"app": "a"})),
            ExprValue::from_json(&json!({"app": "b"})),
        ]);
        let e = Expr::compile("inbound.r.size() == 2 && inbound.r.map(i, i.app) == ['a', 'b'] && inbound.r.exists(i, i.app == 'b') && inbound.r[1].app == 'b'").unwrap();
        let out = with_vals(
            &[],
            vec![("inbound", crate::lazy_map([("r".to_string(), items)]))],
            |ctx| e.eval_in(ctx),
        );
        assert_eq!(out.map(|v| v.as_bool()), Ok(Some(true)));
        let e = Expr::compile("inbound.r[2]").unwrap();
        let out = with_vals(
            &[],
            vec![(
                "inbound",
                crate::lazy_map([("r".to_string(), Default::default())]),
            )],
            |ctx| e.eval_in(ctx),
        );
        assert!(out.is_err(), "out of bounds: {out:?}");
        for idx in ["'a'", "-1"] {
            let e = Expr::compile(&format!("inbound.r[{idx}]")).unwrap();
            let items = Arc::new(vec![ExprValue::from_json(&json!(1))]);
            let out = with_vals(
                &[],
                vec![("inbound", crate::lazy_map([("r".to_string(), items)]))],
                |ctx| e.eval_in(ctx),
            );
            assert!(out.is_err(), "inbound.r[{idx}]: {out:?}");
        }
    }

    #[test]
    fn map_keys_that_are_not_strings_become_json_strings() {
        let m = Expr::compile("{1: 'i', 2u: 'u', true: 'b'}")
            .unwrap()
            .eval(&[])
            .unwrap();
        assert_eq!(m.to_json(), Ok(json!({"1": "i", "2": "u", "true": "b"})));
    }

    #[test]
    fn round_trip_and_eval() {
        let record = ExprValue::from_json(
            &json!({"id": 1001, "name": "x", "tags": ["a"], "m": {"k": true}}),
        );
        let e = Expr::compile(
            "{'id': string(record.id), 'n': record.name, 'missing': has(record.z) ? 1 : null}",
        )
        .unwrap();
        let out = e.eval(&[("record", &record)]).unwrap().to_json().unwrap();
        assert_eq!(out, json!({"id": "1001", "n": "x", "missing": null}));
        let e = Expr::compile("record.m == {'k': true} && 'a' in record.tags").unwrap();
        assert_eq!(
            e.eval(&[("record", &record)]).unwrap().as_bool(),
            Some(true)
        );
        let m = ExprValue::map([("k".to_string(), ExprValue::from_json(&json!(1)))]);
        assert_eq!(m.to_json(), Ok(json!({"k": 1})));
        assert!(ExprValue::from_json(&json!(1)).as_bool().is_none());
    }

    #[test]
    fn env_functions_return_json_and_name_their_errors() {
        let mut env = Env::default();
        env.function1("twice", |s| Ok(json!(format!("{s}{s}"))))
            .unwrap();
        env.function2("pick", |a, b| {
            if a == "x" {
                Ok(json!(b))
            } else {
                Err(format!("no {a}"))
            }
        })
        .unwrap();
        env.function0("zero", || Ok(json!(0))).unwrap();
        let e =
            Expr::compile("twice('a') == 'aa' && pick('x', 'y') == 'y' && zero() == 0").unwrap();
        assert_eq!(e.eval_in(&env.scope()).unwrap().as_bool(), Some(true));
        let err = Expr::compile("pick('q', 'y')")
            .unwrap()
            .eval_in(&env.scope())
            .unwrap_err();
        assert!(err.contains("no q"), "{err}");
    }

    #[test]
    fn numbers_keep_their_kind_and_values_without_json_fail() {
        let big = json!(u64::MAX);
        assert_eq!(ExprValue::from_json(&big).to_json(), Ok(big.clone()));
        let e = Expr::compile("x == 18446744073709551615u && type(x) == uint").unwrap();
        let x = ExprValue::from_json(&big);
        assert_eq!(e.eval(&[("x", &x)]).unwrap().as_bool(), Some(true));
        let half = ExprValue::from_json(&json!(0.5));
        assert_eq!(half.to_json(), Ok(json!(0.5)));
        let e = Expr::compile("h + h == 1.0").unwrap();
        assert_eq!(e.eval(&[("h", &half)]).unwrap().as_bool(), Some(true));
        // cel-rust compares int and double by value, where the CEL spec has no such overload
        // (Q10). JSON numbers rely on it: a rule compares an integer field with `0.5`.
        let one = ExprValue::from_json(&json!(1));
        let e = Expr::compile("i == 1.0 && i < 1.5").unwrap();
        assert_eq!(e.eval(&[("i", &one)]).unwrap().as_bool(), Some(true));
        let inf = Expr::compile("1.0 / 0.0").unwrap().eval(&[]).unwrap();
        assert_eq!(inf.to_json(), Err("non-finite float".into()));
        let ty = Expr::compile("duration('1s')").unwrap().eval(&[]).unwrap();
        assert!(ty.to_json().unwrap_err().contains("has no JSON form"));
        let bad = Expr::compile("1 +").unwrap_err();
        assert!(!bad.is_empty() && !bad.contains('\n'), "{bad}");
    }
}
