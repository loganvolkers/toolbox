//! CEL evaluation: JSON ↔ CEL value conversion and execution with named variables.

use cel::objects::{Key, Map};
use cel::{Context, Program, Value};
use std::collections::HashMap;
use std::sync::Arc;

/// Convert JSON to a CEL value. Integers become `int`, as in cel-python's `json_to_cel`.
pub fn to_cel(v: &serde_json::Value) -> Value {
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

/// Build a CEL map from string-keyed entries.
pub fn map_value(entries: impl IntoIterator<Item = (String, Value)>) -> Value {
    let map: HashMap<Key, Value> = entries
        .into_iter()
        .map(|(k, v)| (Key::String(Arc::new(k)), v))
        .collect();
    Value::Map(Map { map: Arc::new(map) })
}

/// Convert a CEL result back to JSON.
pub fn from_cel(v: &Value) -> Result<serde_json::Value, String> {
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

thread_local! {
    static ROOT: Context<'static, 'static> = Context::default();
}

/// Bind `vars` once (each is converted into CEL's runtime form here) in a child of a per-thread
/// root context with the standard library, and run `f` with it.
pub fn with_vars<R>(vars: &[(&str, &Value)], f: impl FnOnce(&Context<'_, '_>) -> R) -> R {
    ROOT.with(|root| {
        let mut ctx = root.new_inner_scope();
        for (name, value) in vars {
            ctx.add_variable_from_value(*name, (*value).clone());
        }
        f(&ctx)
    })
}

/// Like [`with_vars`], plus variables that are already CEL runtime values (e.g. [`crate::lazy_map`]).
pub fn with_vals<R>(
    vars: &[(&str, &Value)],
    vals: Vec<(&str, Box<dyn cel::common::value::Val>)>,
    f: impl FnOnce(&Context<'_, '_>) -> R,
) -> R {
    ROOT.with(|root| {
        let mut ctx = root.new_inner_scope();
        for (name, value) in vars {
            ctx.add_variable_from_value(*name, (*value).clone());
        }
        for (name, val) in vals {
            ctx.add_variable_as_val(name, val);
        }
        f(&ctx)
    })
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
    pub fn eval(&self, vars: &[(&str, &Value)]) -> Result<Value, String> {
        with_vars(vars, |ctx| self.eval_in(ctx))
    }

    /// Evaluate in a caller-provided context (e.g. one with test functions registered).
    pub fn eval_in(&self, ctx: &Context<'_, '_>) -> Result<Value, String> {
        self.program.execute(ctx).map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn lazy_lists_size_and_iterate() {
        let items = std::sync::Arc::new(vec![
            to_cel(&json!({"app": "a"})),
            to_cel(&json!({"app": "b"})),
        ]);
        let e = Expr::compile("inbound.r.size() == 2 && inbound.r.map(i, i.app) == ['a', 'b'] && inbound.r.exists(i, i.app == 'b')").unwrap();
        let out = with_vals(
            &[],
            vec![("inbound", crate::lazy_map([("r".to_string(), items)]))],
            |ctx| e.eval_in(ctx),
        );
        assert_eq!(out, Ok(Value::Bool(true)));
    }

    #[test]
    fn round_trip_and_eval() {
        let record = to_cel(&json!({"id": 1001, "name": "x", "tags": ["a"], "m": {"k": true}}));
        let e = Expr::compile(
            "{'id': string(record.id), 'n': record.name, 'missing': has(record.z) ? 1 : null}",
        )
        .unwrap();
        let out = from_cel(&e.eval(&[("record", &record)]).unwrap()).unwrap();
        assert_eq!(out, json!({"id": "1001", "n": "x", "missing": null}));
        let e = Expr::compile("record.m == {'k': true} && 'a' in record.tags").unwrap();
        assert_eq!(e.eval(&[("record", &record)]).unwrap(), Value::Bool(true));
    }
}
