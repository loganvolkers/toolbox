//! Full-pipeline recompute at Seikika's v1.0 scale target (plan.md §6.6: 100k entities, 2M relations,
//! 10k users, 1k teams, 100 rules), with no incremental engine. Answers adversarial-critique.md E4.
//!
//! Pipeline (every stage recomputed from scratch, from two source fragments per entity):
//!   merge (field precedence git > connector) -> CEL values -> 100 CEL rules -> levels
//!   -> cross-entity aggregates over 2M dependsOn edges (in-degree; min level of dependencies)
//!   -> team closure + team roll-ups (inherited ownership) -> domain roll-ups
//!   -> OpenFGA-style tuple derivation -> materialised permission view (user -> visible services)
//! Then a targeted single-entity update (hand-written invalidation, no IVM engine).
//!
//! Usage: derived-state-scale [entities=100000] [threads=4] [--csv DIR]
use cel::{Context, Env, Value};
use rayon::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

const TEAMS: usize = 1_000;
const USERS: usize = 10_000;
const SYSTEMS: usize = 2_000;
const DOMAINS: usize = 50;
const DEG: usize = 20; // out-degree: 100k entities x 20 = 2M dependsOn relations
const LANGS: [&str; 5] = ["go", "rust", "python", "java", "ts"];

fn mix(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}
fn h(i: usize, salt: u64) -> u64 { mix((i as u64) ^ salt.wrapping_mul(0xA24B_AED4_963E_E407)) }
fn team_parent(t: usize) -> Option<usize> { if t == 0 { None } else { Some((t - 1) / 5) } }

/// Fragment from git (catalog-info.yaml). `None` = field absent in this source.
#[derive(Clone)]
struct Git { owner: Option<i32>, tier: Option<i64>, lang: usize, desc_len: usize, pci: bool, runbook: bool, system: usize }
/// Fragment from a connector (GitHub/PagerDuty/Snyk-like facts).
#[derive(Clone)]
struct Conn { owner: Option<i32>, tier: i64, on_call: bool, coverage: i64, vulns: i64, image_age: i64, slo: bool, alerts: i64 }
#[derive(Clone)]
struct Merged { owner: i32, tier: i64, lang: usize, desc_len: usize, pci: bool, runbook: bool, system: usize,
                on_call: bool, coverage: i64, vulns: i64, image_age: i64, slo: bool, alerts: i64 }

fn git_frag(i: usize) -> Git {
    let a = h(i, 1);
    Git {
        owner: if a % 10 == 9 { None } else if i % 13 == 0 { Some(-1) } else { Some((i % TEAMS) as i32) },
        tier: if (a >> 8) % 10 < 7 { Some(((a >> 16) % 4) as i64) } else { None },
        lang: ((a >> 20) % 5) as usize,
        desc_len: 20 + ((a >> 24) % 60) as usize,
        pci: (a >> 32) % 11 == 0,
        runbook: (a >> 36) % 2 == 0,
        system: ((a >> 40) % SYSTEMS as u64) as usize,
    }
}
fn conn_frag(i: usize) -> Conn {
    let b = h(i, 2);
    Conn {
        owner: if b % 10 == 0 { Some(((b >> 4) % TEAMS as u64) as i32) } else { None },
        tier: ((b >> 8) % 4) as i64,
        on_call: (b >> 12) % 3 != 0,
        coverage: ((b >> 16) % 101) as i64,
        vulns: ((b >> 24) % 17 == 0) as i64,
        image_age: ((b >> 32) % 120) as i64,
        slo: (b >> 40) % 2 == 0,
        alerts: ((b >> 44) % 7) as i64,
    }
}
fn merge(g: &Git, c: &Conn) -> Merged {
    Merged { owner: g.owner.or(c.owner).unwrap_or(-1), tier: g.tier.unwrap_or(c.tier), lang: g.lang, desc_len: g.desc_len,
             pci: g.pci, runbook: g.runbook, system: g.system, on_call: c.on_call, coverage: c.coverage, vulns: c.vulns,
             image_age: c.image_age, slo: c.slo, alerts: c.alerts }
}
fn edges_of(i: usize, n: usize) -> impl Iterator<Item = usize> {
    (0..DEG).map(move |j| {
        let u = (h(i * DEG + j, 7) >> 11) as f64 / (1u64 << 53) as f64; // skewed fan-in: target ~ n*u^2
        let t = ((u * u) * n as f64) as usize % n;
        if t == i { (t + 1) % n } else { t }
    })
}
fn cel_value(i: usize, m: &Merged, deps: &[usize]) -> Value {
    let mut e: HashMap<String, Value> = HashMap::new();
    e.insert("name".into(), format!("svc-{i}").into());
    e.insert("owner".into(), (if m.owner < 0 { String::new() } else { format!("team-{}", m.owner) }).into());
    e.insert("tier".into(), m.tier.into());
    e.insert("lang".into(), LANGS[m.lang].into());
    e.insert("coverage".into(), m.coverage.into());
    e.insert("vulns_critical".into(), m.vulns.into());
    e.insert("description".into(), "d".repeat(m.desc_len).into());
    e.insert("on_call".into(), (if m.on_call { "pd-x" } else { "" }).into());
    e.insert("image_age_days".into(), m.image_age.into());
    e.insert("slo_defined".into(), m.slo.into());
    e.insert("alerts".into(), m.alerts.into());
    let tags: Vec<Value> = if m.pci { vec!["pci".into(), "prod".into()] } else { vec!["prod".into()] };
    e.insert("tags".into(), tags.into());
    let d: Vec<Value> = deps.iter().map(|t| Value::from(format!("svc-{t}"))).collect();
    e.insert("deps".into(), d.into());
    let mut ann: HashMap<String, Value> = HashMap::new();
    if m.runbook { ann.insert("runbook".into(), "https://x".into()); }
    e.insert("annotations".into(), ann.into());
    e.into()
}
const TEMPLATES: [&str; 10] = [
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
fn level(passed: usize) -> u8 { if passed >= 90 { 3 } else if passed >= 75 { 2 } else if passed >= 60 { 1 } else { 0 } }
fn rss_mb() -> u64 {
    std::fs::read_to_string("/proc/self/status").ok().and_then(|s| s.lines().find(|l| l.starts_with("VmHWM"))
        .and_then(|l| l.split_whitespace().nth(1)).and_then(|v| v.parse::<u64>().ok())).unwrap_or(0) / 1024
}
macro_rules! timed { ($name:expr, $tot:ident, $e:expr) => {{ let t = Instant::now(); let r = $e; let el = t.elapsed();
    $tot += el.as_secs_f64(); println!("{:<44} {:>10.1} ms", $name, el.as_secs_f64() * 1e3); r }}; }

fn write_csv(dir: &str, n: usize, git: &[Git], conn: &[Conn], out: &[Vec<usize>]) {
    use std::io::Write;
    std::fs::create_dir_all(dir).unwrap();
    let f = |name: &str| std::io::BufWriter::new(std::fs::File::create(format!("{dir}/{name}")).unwrap());
    let opt = |o: Option<i64>| o.map(|v| v.to_string()).unwrap_or_default();
    let mut w = f("git.csv");
    for i in 0..n { let g = &git[i];
        let deps: Vec<String> = out[i].iter().map(|t| format!("svc-{t}")).collect();
        writeln!(w, "{i},svc-{i},{},{},{},{},{},{},{},\"{{{}}}\"", opt(g.owner.map(|v| v as i64)), opt(g.tier), LANGS[g.lang],
            "d".repeat(g.desc_len), g.pci, g.runbook, g.system, deps.join(",")).unwrap(); }
    let mut w = f("conn.csv");
    for i in 0..n { let c = &conn[i];
        writeln!(w, "{i},{},{},{},{},{},{},{},{}", opt(c.owner.map(|v| v as i64)), c.tier, if c.on_call { "pd-x" } else { "" },
            c.coverage, c.vulns, c.image_age, c.slo, c.alerts).unwrap(); }
    let mut w = f("edges.csv");
    for i in 0..n { for t in &out[i] { writeln!(w, "{i},{t}").unwrap(); } }
    let mut w = f("teams.csv");
    for t in 0..TEAMS { writeln!(w, "{t},{}", team_parent(t).map(|p| p.to_string()).unwrap_or_default()).unwrap(); }
    let mut w = f("members.csv");
    for u in 0..USERS { for k in 0..3 { writeln!(w, "{u},{}", (u * 7 + k * 131) % TEAMS).unwrap(); } }
    let mut w = f("systems.csv");
    for s in 0..SYSTEMS { writeln!(w, "{s},{}", s % DOMAINS).unwrap(); }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let n: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(100_000);
    let threads: usize = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(4);
    let csv = args.iter().position(|a| a == "--csv").map(|p| args[p + 1].clone());
    rayon::ThreadPoolBuilder::new().num_threads(threads).build_global().unwrap();

    // ---- inputs (not timed: this is what the store already holds) ----
    let git: Vec<Git> = (0..n).map(git_frag).collect();
    let conn: Vec<Conn> = (0..n).map(conn_frag).collect();
    let out: Vec<Vec<usize>> = (0..n).map(|i| edges_of(i, n).collect()).collect();
    let user_teams: Vec<[usize; 3]> = (0..USERS).map(|u| [(u * 7) % TEAMS, (u * 7 + 131) % TEAMS, (u * 7 + 262) % TEAMS]).collect();
    if let Some(dir) = &csv { write_csv(dir, n, &git, &conn, &out); println!("wrote CSVs to {dir}"); return; }
    println!("entities={n} relations={} users={USERS} teams={TEAMS} rules=100 threads={threads}", n * DEG);

    let env = Arc::new(Env::stdlib());
    let progs: Vec<_> = (0..100).map(|i| env.compile(&TEMPLATES[i % 10].replace("{k}", &(i % 97).to_string())).unwrap()).collect();
    let mut total = 0.0f64;

    // 1. merge fragments by per-field precedence
    let merged: Vec<Merged> = timed!("1 merge 2 fragments/entity", total, (0..n).into_par_iter().map(|i| merge(&git[i], &conn[i])).collect());
    // 2. build CEL values (mapping output)
    let vals: Vec<Value> = timed!("2 build CEL entity values", total, (0..n).into_par_iter().map(|i| cel_value(i, &merged[i], &out[i])).collect());
    // 3. 100 CEL rules per entity
    let passed: Vec<usize> = timed!("3 100 CEL rules x entities", total, vals.par_iter().map(|v| {
        let mut ctx = Context::with_env(env.clone());
        ctx.add_variable_from_value("e", v.clone());
        progs.iter().filter(|p| matches!(p.execute(&ctx), Ok(Value::Bool(true)))).count()
    }).collect());
    let levels: Vec<u8> = timed!("4 levels", total, passed.par_iter().map(|p| level(*p)).collect());
    // 5. aggregate over 2M edges: dependents (in-degree)
    let indeg: Vec<u32> = timed!("5 dependents count (2M edges)", total, {
        let mut d = vec![0u32; n]; for o in &out { for &t in o { d[t] += 1; } } d });
    // 6. cross-entity rule: min level of direct dependencies ("all deps >= bronze")
    let dep_min: Vec<u8> = timed!("6 min dependency level (2M-edge join)", total,
        out.par_iter().map(|o| o.iter().map(|&t| levels[t]).min().unwrap_or(3)).collect());
    let deps_ok = dep_min.iter().filter(|&&m| m >= 1).count();
    // 7. team closure (ancestors) + inherited-ownership roll-ups
    let (anc, roll_cnt, roll_sum): (Vec<Vec<usize>>, Vec<u64>, Vec<u64>) = timed!("7 team closure + team roll-ups", total, {
        let anc: Vec<Vec<usize>> = (0..TEAMS).map(|t| { let mut v = vec![t]; let mut g = team_parent(t); while let Some(x) = g { v.push(x); g = team_parent(x); } v }).collect();
        let (mut c, mut s) = (vec![0u64; TEAMS], vec![0u64; TEAMS]);
        for i in 0..n { let o = merged[i].owner; if o >= 0 { for &a in &anc[o as usize] { c[a] += 1; s[a] += passed[i] as u64; } } }
        (anc, c, s) });
    // 8. domain roll-up (service -> system -> domain)
    let dom: Vec<(u64, u64)> = timed!("8 domain roll-ups", total, {
        let mut d = vec![(0u64, 0u64); DOMAINS]; for i in 0..n { let x = &mut d[merged[i].system % DOMAINS]; x.0 += 1; x.1 += passed[i] as u64; } d });
    // 9. OpenFGA-style tuple derivation
    let tuples: Vec<(u8, u32, u32)> = timed!("9 derive authz tuples", total, {
        let mut v = Vec::with_capacity(250_000);
        for i in 0..n { if merged[i].owner >= 0 { v.push((0, i as u32, merged[i].owner as u32)); } v.push((1, i as u32, merged[i].system as u32)); }
        for s in 0..SYSTEMS { v.push((2, s as u32, (s % DOMAINS) as u32)); }
        for t in 1..TEAMS { v.push((3, t as u32, team_parent(t).unwrap() as u32)); }
        for (u, ts) in user_teams.iter().enumerate() { for &t in ts { v.push((4, t as u32, u as u32)); } }
        v });
    // 10. materialised permission view: user -> services they can view details of
    //     (member of the owner team or of any ancestor team) = ListObjects for every user
    let (perm_pairs, subtree_sizes): (usize, Vec<usize>) = timed!("10 permission view (ListObjects x all users)", total, {
        let mut owned: Vec<Vec<u32>> = vec![Vec::new(); TEAMS];
        for i in 0..n { if merged[i].owner >= 0 { owned[merged[i].owner as usize].push(i as u32); } }
        let mut sub: Vec<Vec<u32>> = owned.clone();
        for t in (1..TEAMS).rev() { let p = team_parent(t).unwrap(); let c = std::mem::take(&mut sub[t]); sub[p].extend_from_slice(&c); sub[t] = c; }
        let views: Vec<Vec<u32>> = user_teams.par_iter().map(|ts| {
            let keep: Vec<usize> = ts.iter().copied().filter(|&t| !ts.iter().any(|&o| o != t && anc[t][1..].contains(&o))).collect();
            let mut v: Vec<u32> = keep.iter().flat_map(|&t| sub[t].iter().copied()).collect(); v.sort_unstable(); v.dedup(); v }).collect();
        (views.iter().map(|v| v.len()).sum(), sub.iter().map(|s| s.len()).collect()) });
    println!("{:<44} {:>10.1} ms", "TOTAL full recompute", total * 1e3);
    println!("summary: avg_passed={:.1} levels(g/s/b/none)={:?} deps_ok={} max_indeg={} root_rollup={} tuples={} perm_pairs={} domain0={:?} peak_rss_mb={}",
        passed.iter().sum::<usize>() as f64 / n as f64,
        [3u8, 2, 1, 0].map(|l| levels.iter().filter(|&&x| x == l).count()), deps_ok, indeg.iter().max().unwrap(),
        roll_cnt[0], tuples.len(), perm_pairs, dom[0], rss_mb());
    let _ = &subtree_sizes;

    // ---- targeted single-entity update with hand-written invalidation (no IVM engine) ----
    let mut rev: Vec<Vec<u32>> = vec![Vec::new(); n];
    for (i, o) in out.iter().enumerate() { for &t in o { rev[t].push(i as u32); } }
    let median_e = (0..n).find(|&i| rev[i].len() == 20).unwrap_or(n / 2);
    for (label, x) in [("hot entity (max fan-in)", 0usize), ("median entity", median_e)] {
        let mut levels = levels.clone(); let mut dep_min = dep_min.clone();
        let mut roll_cnt = roll_cnt.clone(); let mut roll_sum = roll_sum.clone();
        let t = Instant::now();
        let mut g = git[x].clone(); g.owner = Some(((merged[x].owner.max(0) as usize + 17) % TEAMS) as i32); g.tier = Some(3);
        let old = merged[x].clone(); let old_passed = passed[x];
        let m = merge(&g, &conn[x]);
        let v = cel_value(x, &m, &out[x]);
        let mut ctx = Context::with_env(env.clone()); ctx.add_variable_from_value("e", v);
        let p = progs.iter().filter(|p| matches!(p.execute(&ctx), Ok(Value::Bool(true)))).count();
        let lv = level(p); let lv_changed = lv != levels[x]; levels[x] = lv;
        let mut touched = 0;
        if lv_changed { for &d in &rev[x] { dep_min[d as usize] = out[d as usize].iter().map(|&t| levels[t]).min().unwrap_or(3); touched += 1; } }
        if old.owner >= 0 { for &a in &anc[old.owner as usize] { roll_cnt[a] -= 1; roll_sum[a] -= old_passed as u64; } }
        if m.owner >= 0 { for &a in &anc[m.owner as usize] { roll_cnt[a] += 1; roll_sum[a] += p as u64; } }
        // permission-view delta: users in old owner's ancestor teams lose x, new owner's gain x
        let affected = user_teams.iter().filter(|ts| ts.iter().any(|&t| (old.owner >= 0 && anc[old.owner as usize].contains(&t))
            || (m.owner >= 0 && anc[m.owner as usize].contains(&t)))).count();
        let el = t.elapsed();
        println!("update {label:<24} id={x} fan_in={} level_changed={lv_changed} deps_rechecked={touched} users_in_perm_delta={affected} time={:.1} us",
            rev[x].len(), el.as_secs_f64() * 1e6);
        let _ = (dep_min, roll_cnt, roll_sum);
    }
    println!("peak_rss_mb={}", rss_mb());
}
