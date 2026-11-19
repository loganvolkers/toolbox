use cel::{Context, Env, Value};
use rayon::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

// Naive full recomputation: N entities x R CEL rules, plus transitive closure of a team hierarchy
// and ownership roll-up. No incrementality, no caching.
fn main() {
    let n: usize = std::env::args().nth(1).map(|s| s.parse().unwrap()).unwrap_or(100_000);
    let r: usize = std::env::args().nth(2).map(|s| s.parse().unwrap()).unwrap_or(100);
    let teams = 1_000usize;
    let env = Arc::new(Env::stdlib());
    // 100 rules of varied shape over typical scorecard fields
    let templates = [
        "e.owner != '' && e.tier <= {k}",
        "has(e.annotations.runbook) || e.tier > {k}",
        "e.lang in ['go','rust','python'] && e.coverage >= {k}",
        "e.vulns_critical == 0 || e.tier > ({k} % 4)",
        "e.tags.exists(t, t == 'pci') ? e.coverage > {k} : true",
        "size(e.description) > {k} && e.on_call != ''",
        "e.deps.all(d, d != 'legacy-{k}')",
        "e.image_age_days < 30 + {k}",
        "e.slo_defined && e.alerts >= ({k} % 5)",
        "e.name.startsWith('svc') || e.coverage > ({k} % 90)",
    ];
    let progs: Vec<_> = (0..r)
        .map(|i| env.compile(&templates[i % templates.len()].replace("{k}", &(i % 97).to_string())).unwrap())
        .collect();
    // Build entities as CEL values once (ingest cost is not part of recompute)
    let t0 = Instant::now();
    let ents: Vec<Value> = (0..n)
        .map(|i| {
            let mut m: HashMap<String, Value> = HashMap::new();
            m.insert("name".into(), format!("svc-{i}").into());
            m.insert("owner".into(), (if i % 13 == 0 { String::new() } else { format!("team-{}", i % teams) }).into());
            m.insert("tier".into(), ((i % 4) as i64).into());
            m.insert("lang".into(), ["go", "rust", "python", "java", "ts"][i % 5].into());
            m.insert("coverage".into(), ((i * 7 % 100) as i64).into());
            m.insert("vulns_critical".into(), ((i % 17 == 0) as i64).into());
            m.insert("description".into(), "a service that does things for other services".into());
            m.insert("on_call".into(), (if i % 3 == 0 { "" } else { "pd-x" }).into());
            m.insert("image_age_days".into(), ((i % 90) as i64).into());
            m.insert("slo_defined".into(), (i % 2 == 0).into());
            m.insert("alerts".into(), ((i % 7) as i64).into());
            let tags: Vec<Value> = if i % 11 == 0 { vec!["pci".into(), "prod".into()] } else { vec!["prod".into()] };
            m.insert("tags".into(), tags.into());
            let deps: Vec<Value> = (0..5).map(|j| format!("svc-{}", (i + j * 31) % n).into()).collect();
            m.insert("deps".into(), deps.into());
            let mut ann: HashMap<String, Value> = HashMap::new();
            if i % 2 == 0 { ann.insert("runbook".into(), "https://x".into()); }
            m.insert("annotations".into(), ann.into());
            m.into()
        })
        .collect();
    eprintln!("built {} entities in {:?}", n, t0.elapsed());

    // Full rescore, single thread and parallel
    for threads in [1usize, 4] {
        let pool = rayon::ThreadPoolBuilder::new().num_threads(threads).build().unwrap();
        let t = Instant::now();
        let passed: usize = pool.install(|| {
            ents.par_iter()
                .map(|e| {
                    let mut ctx = Context::with_env(env.clone());
                    ctx.add_variable_from_value("e", e.clone());
                    progs.iter().filter(|p| matches!(p.execute(&ctx), Ok(Value::Bool(true)))).count()
                })
                .sum()
        });
        let el = t.elapsed();
        println!("threads={threads} entities={n} rules={r} evals={} passed={passed} time={:?} per_eval={:.0}ns",
            n * r, el, el.as_nanos() as f64 / (n * r) as f64);
    }

    // Transitive closure: 1,000 teams in a 5-level hierarchy, 10k users with 3 memberships each;
    // compute every user's effective groups (nested membership) by naive BFS.
    let t = Instant::now();
    let parent: Vec<Option<usize>> = (0..teams).map(|i| if i == 0 { None } else { Some((i - 1) / 5) }).collect();
    let users = 10_000usize;
    let mut total = 0usize;
    for u in 0..users {
        let mut seen = std::collections::HashSet::new();
        for k in 0..3 {
            let mut g = Some((u * 7 + k * 131) % teams);
            while let Some(x) = g { if !seen.insert(x) { break; } g = parent[x]; }
        }
        total += seen.len();
    }
    // ownership roll-up: services per team including descendants
    let mut counts = vec![0usize; teams];
    for i in 0..n { let mut g = Some(i % teams); while let Some(x) = g { counts[x] += 1; g = parent[x]; } }
    println!("closure users={users} memberships_total={total} rollup_root={} time={:?}", counts[0], t.elapsed());
}
