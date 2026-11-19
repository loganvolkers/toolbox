use wasmtime::*;
pub fn add(a: i32, b: i32) -> i32 {
    let engine = Engine::default();
    let module = Module::new(&engine, r#"(module (func (export "add") (param i32 i32) (result i32) local.get 0 local.get 1 i32.add))"#).unwrap();
    let mut store = Store::new(&engine, ());
    let inst = Instance::new(&mut store, &module, &[]).unwrap();
    let f = inst.get_typed_func::<(i32, i32), i32>(&mut store, "add").unwrap();
    f.call(&mut store, (a, b)).unwrap()
}
