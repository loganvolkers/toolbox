pub fn eval(expr: &str, tier: i64) -> bool {
    let p = cel::Program::compile(expr).unwrap();
    let mut c = cel::Context::default();
    c.add_variable("tier", tier).unwrap();
    matches!(p.execute(&c), Ok(cel::Value::Bool(true)))
}
