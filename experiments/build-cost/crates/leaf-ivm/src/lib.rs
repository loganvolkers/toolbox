use dbsp::{utils::Tup2, IndexedZSetReader, OrdZSet, Runtime, Stream};
/// Transitive closure of `edges` (the nested-group-membership shape), returns the closure size.
pub fn closure_size(edges: &[(u64, u64)]) -> usize {
    let (mut c, (h, out)) = Runtime::init_circuit(1, |circuit| {
        let (e, h) = circuit.add_input_zset::<Tup2<u64, u64>>();
        let paths = circuit.recursive(|child, paths: Stream<_, OrdZSet<Tup2<u64, u64>>>| {
            let e = e.delta0(child);
            let by_t = paths.map_index(|Tup2(s, t)| (*t, *s));
            let by_s = e.map_index(|Tup2(s, t)| (*s, *t));
            Ok(e.plus(&by_t.join(&by_s, |_v, s, t| Tup2(*s, *t))))
        })?;
        Ok((h, paths.integrate().output()))
    }).unwrap();
    for (a, b) in edges { h.push(Tup2(*a, *b), 1); }
    c.transaction().unwrap();
    let n = out.consolidate().iter().count();
    c.kill().unwrap();
    n
}
