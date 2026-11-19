use async_graphql::dynamic::*;
use async_graphql::Value;
/// A dynamic schema built at runtime from a "manifest" (no codegen), as docs/design/graphql.md proposes.
pub fn schema_sdl(kinds: &[&str]) -> String {
    let mut q = Object::new("Query");
    let mut b = Schema::build("Query", None, None);
    for k in kinds {
        let obj = Object::new(*k).field(Field::new("name", TypeRef::named_nn(TypeRef::STRING), |_| {
            FieldFuture::new(async { Ok(Some(Value::from("x"))) })
        }));
        q = q.field(Field::new(k.to_lowercase(), TypeRef::named(*k), |_| FieldFuture::new(async { Ok(Some(FieldValue::NULL)) })));
        b = b.register(obj);
    }
    b.register(q).finish().unwrap().sdl()
}
