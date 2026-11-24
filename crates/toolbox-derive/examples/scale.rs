//! M1 scale check (docs/v1-spec.md §10.4): full recompute at 100k entities, 2M relations, 10k
//! users, 1k teams and 100 rules must finish within 10 s on 4 vCPU.
//!
//! Run: cargo run --release -p toolbox-derive --example scale [-- <services>]
//! `SCALE_RUNS` (default 3) sets how many recomputes to time; `SCALE_BUDGET_SECS` the budget.
//! `scripts/scale-gate.sh` counts this program's instructions at a smaller size, for CI.

use serde_json::{Value, json};
use std::time::Instant;
use toolbox_derive::{DerivedState, FullRecompute};

/// The allocator the `toolbox` binary uses, so this measures what the server runs.
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() {
    let n: usize = std::env::args()
        .nth(1)
        .map(|s| s.parse().unwrap())
        .unwrap_or(100_000);
    let (teams, users, deps_per_service) = (1_000usize, 10_000usize, 20usize);
    let t0 = Instant::now();
    let mut store = toolbox_store::MemStore::new();

    // Teams in a 5-ary tree; each user in 3 teams.
    let mut members: Vec<Vec<String>> = vec![Vec::new(); teams];
    for u in 0..users {
        for k in 0..3 {
            members[(u * 7 + k * 331) % teams].push(format!("u{u}"));
        }
    }
    let groups: Vec<Value> = (0..teams)
        .map(|g| {
            let mut data = json!({"slug": format!("t{g}"), "githubTeam": format!("acme/t{g}"), "members": members[g]});
            if g > 0 {
                data["parent"] = json!(format!("t{}", (g - 1) / 5));
            }
            json!({"type": "record", "kind": "group", "externalId": format!("idp:group/t{g}"), "data": data})
        })
        .collect();
    store.ingest("idp", &groups);

    // Services with skewed dependencies (2M edges at 100k), CODEOWNERS and catalog-info claims.
    let langs = ["rust", "go", "ts", "python", "java", "ruby"];
    let mut github = Vec::with_capacity(n * 2);
    let mut git = Vec::with_capacity(n);
    for i in 0..n {
        let team = i % teams;
        let deps: Vec<String> = (0..deps_per_service)
            .map(|k| {
                let r = (i.wrapping_mul(2654435761).wrapping_add(k * 40503)) % n;
                format!("s{}", (r * r / n.max(1)) % n) // squaring skews towards low indexes
            })
            .collect();
        let topics: Vec<String> = (0..(i % 5)).map(|k| format!("t{}", (i + k) % 10)).collect();
        github.push(json!({"type": "record", "kind": "repository", "externalId": format!("gh:{i}"), "data": {
            "name": format!("s{i}"), "archived": i % 50 == 0, "protected": i % 3 != 0, "readme": i % 4 != 0,
            "stars": (i * 37) % 500, "lang": langs[i % langs.len()], "topics": topics, "deps": deps}}));
        github.push(json!({"type": "record", "kind": "codeowners", "externalId": format!("gh:co:{i}"),
            "data": {"repo": format!("s{i}"), "rules": [{"pattern": "*", "owners": [format!("@acme/t{team}")]}]}}));
        if i % 5 != 0 {
            let claim = if i % 17 == 0 {
                (team + 1) % teams
            } else {
                team
            };
            git.push(json!({"type": "record", "kind": "catalog-info", "externalId": format!("git:{i}"),
                "data": {"repo": format!("s{i}"), "owner": format!("group:t{claim}"), "tier": format!("{}", i % 4)}}));
        }
    }
    store.ingest("github", &github);
    store.ingest("git", &git);

    // 100 rules from 10 templates, spread over three levels.
    let templates = [
        "entity.tier in ['0', '1'] || entity.protected",
        "entity.readme",
        "entity.stars > {k}",
        "entity.topics.exists(t, t == 't{d}')",
        "entity.lifecycle != 'deprecated'",
        "size(entity.name) > {d}",
        "entity.ownership.status in ['verified', 'inferred']",
        "entity.dependents < {k}",
        "entity.lang.startsWith('r') || entity.lang == 'go'",
        "entity.topics.all(t, size(t) > 1) && has(entity.tier)",
    ];
    let rules: Vec<Value> = (0..100)
        .map(|r| {
            let check = templates[r % 10]
                .replace("{k}", &(r * 5).to_string())
                .replace("{d}", &(r % 10).to_string());
            let level = ["bronze", "silver", "gold"][r % 3];
            json!({"name": format!("r{r}"), "level": level, "check": check})
        })
        .collect();
    let manifest_doc = json!({
        "apiVersion": "toolbox.dev/v1alpha1",
        "portal": {"name": "scale", "owner": "group:t0", "toolbox": "0.1"},
        "policies": {"langs": langs},
        "connectors": [
            {"name": "idp", "package": "builtin:scim", "runtime": "remote", "records": ["group"]},
            {"name": "github", "package": "builtin:github", "runtime": "in-image", "records": ["repository", "codeowners"]},
            {"name": "git", "package": "builtin:git-files", "runtime": "in-image", "records": ["catalog-info"]}
        ],
        "kinds": [
            {"name": "Group", "identityFrom": ["connector:idp"],
             "fields": {"parent": {"type": "string"}, "githubTeam": {"type": "string"}, "members": {"type": "list", "items": "string"}},
             "sources": {"default": ["connector:idp"]},
             "map": [{"from": "idp/group", "id": "'group:' + record.slug",
                      "set": "{'parent': has(record.parent) ? 'group:' + record.parent : null, 'githubTeam': record.githubTeam, 'members': record.members}"}]},
            {"name": "Service", "identityFrom": ["connector:github"],
             "fields": {"name": {"type": "string"}, "archived": {"type": "boolean"}, "protected": {"type": "boolean"},
                        "readme": {"type": "boolean"}, "stars": {"type": "integer"}, "lang": {"type": "string"},
                        "topics": {"type": "list", "items": "string"}, "tier": {"type": "string"},
                        "lifecycle": {"type": "string"}, "ownerClaim": {"type": "string"},
                        "codeowners": {"type": "list", "items": "object"},
                        "ownership": {"type": "object", "authority": {"type": "codeowners", "claim": "ownerClaim",
                                      "rules": "codeowners", "path": "catalog-info.yaml", "teamField": "Group.githubTeam"}},
                        "dependents": {"type": "integer", "computed": "inbound.dependsOn.size()"}},
             "relations": {"dependsOn": {"to": "Service", "many": true}},
             "sources": {"default": ["connector:github"], "fields": {"tier": ["git:catalog"], "ownerClaim": ["git:catalog"]}},
             "map": [
                {"from": "github/repository", "id": "'service:' + record.name",
                 "set": "{'name': record.name, 'archived': record.archived, 'protected': record.protected, 'readme': record.readme, 'stars': record.stars, 'lang': record.lang, 'topics': record.topics, 'lifecycle': record.archived ? 'deprecated' : 'production'}",
                 "relations": {"dependsOn": "record.deps.map(d, 'service:' + d)"}},
                {"from": "github/codeowners", "id": "'service:' + record.repo", "set": "{'codeowners': record.rules}"},
                {"from": "git/catalog-info", "source": "git:catalog", "id": "'service:' + record.repo",
                 "set": "{'ownerClaim': record.owner, 'tier': record.tier}"}]}
        ],
        "scorecards": [{"name": "standards", "kind": "Service", "owner": "group:t0", "levels": ["bronze", "silver", "gold"],
                        "appliesTo": "!entity.archived", "rules": rules}],
        "authz": {"model": "authz/model.fga", "tuples": [
            {"kind": "Group", "emit": "entity.members.map(m, {'user': 'user:' + m, 'relation': 'member', 'object': entity.id})"},
            {"kind": "Group", "emit": "has(entity.parent) ? [{'user': entity.id + '#member', 'relation': 'member', 'object': entity.parent}] : []"},
            {"kind": "Service", "emit": "entity.ownership.status in ['verified', 'inferred'] ? [{'user': entity.ownership.owner, 'relation': 'owner', 'object': entity.id}] : []"}
        ]},
        "actions": [{"name": "switch-lang", "kind": "Service", "scope": "day2", "description": "Pick a new primary language.",
                     "inputs": {"type": "object", "properties": {"lang": {"type": "string", "enumFrom": "policy.langs.filter(l, l != entity.lang)"}}},
                     "permission": "can_edit", "executor": {"type": "command", "run": "true"}, "unverified": true}]
    });
    let (m, errors) = toolbox_model::validate_manifest(&manifest_doc, "scale");
    assert!(errors.is_empty(), "{errors:#?}");
    let m = m.unwrap();
    let snap = store.snapshot();
    let edges = n * deps_per_service;
    println!(
        "generated {} records ({n} services, {teams} teams, {users} users, {edges} relations) in {:.2?}",
        snap.records.len(),
        t0.elapsed()
    );
    let budget: f64 = std::env::var("SCALE_BUDGET_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(10.0);
    let runs: usize = std::env::var("SCALE_RUNS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(3);
    let mut best = f64::MAX;
    for run in 1..=runs {
        let t = Instant::now();
        let d = FullRecompute.recompute(&m, &snap);
        let elapsed = t.elapsed();
        best = best.min(elapsed.as_secs_f64());
        let scored = d.scores.values().map(|s| s.len()).sum::<usize>();
        println!(
            "run {run}: full recompute {elapsed:.2?} — {} entities, {scored} scores (100 rules each), {} tuples, {} options, {} diagnostics",
            d.entities.len(),
            d.tuples.len(),
            d.options
                .values()
                .flat_map(|i| i.values())
                .map(|e| e.len())
                .sum::<usize>(),
            d.diagnostics.len()
        );
        if let Some(first) = d.diagnostics.first() {
            println!("  first diagnostic: {first:?}");
        }
    }
    println!(
        "threads: {}; best {best:.2}s; budget {budget:.1}s (docs/v1-spec.md §10.4)",
        rayon::current_num_threads()
    );
    if best > budget {
        eprintln!("over budget");
        std::process::exit(1);
    }
}
