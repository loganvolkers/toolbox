//! What `federation.children` and actions on a portal's own entity declare for the author
//! (docs/design/federated-actions.md §5.1). Loading a manifest fills them in, so `validate`, `run`,
//! `serve` and a parent reading this portal as a child all see the same manifest.
//!
//! - **`federation.children`** (a parent): the `federation` connector (builtin:workspace), which
//!   keeps a clone of each child at its branch under the state directory; the `Portal` kind its
//!   `portal` records map to, keyed `portal:<key>`; `federation.portals: Portal`; and
//!   `runners.local.portals` listing the children, unless `runners.local` is declared.
//! - **An action on kind `Portal` in a portal that declares no `Portal` kind** (a repo portal):
//!   the `self` connector (builtin:workspace, `repo` record of the portal's repository) and the
//!   `Portal` kind with one entity, `portal:<name>`, the portal itself.
//!
//! Whatever the author declares (a connector named `federation` or `self`, a `Portal` kind) is
//! left as it is.

use serde_json::{Value, json};
use std::path::{Path, PathBuf};

/// Where Toolbox keeps local state: `TOOLBOX_STATE_DIR`, else `$XDG_STATE_HOME/toolbox`, else
/// `~/.local/state/toolbox`.
pub fn state_dir() -> PathBuf {
    std::env::var_os("TOOLBOX_STATE_DIR")
        .filter(|d| !d.is_empty())
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("XDG_STATE_HOME").map(|d| PathBuf::from(d).join("toolbox")))
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/state/toolbox")))
        .unwrap_or_else(|| std::env::temp_dir().join("toolbox"))
}

/// The kind child portals and a portal's own entity have when the author declares none.
/// The connector `federation.children` declares.
pub const CONNECTOR: &str = "federation";

pub const PORTAL_KIND: &str = "Portal";

/// A child's key when it names none: the URL's last segment, without `.git`.
pub fn default_key(git: &str) -> String {
    let last = git
        .trim_end_matches('/')
        .rsplit(['/', ':'])
        .next()
        .unwrap_or(git);
    last.strip_suffix(".git")
        .unwrap_or(last)
        .to_ascii_lowercase()
}

/// A path `git clone` would read as local, resolved against the manifest's directory; URLs
/// (`scheme://…`, `user@host:path`) are left as they are.
fn clone_url(root: &Path, git: &str) -> String {
    let remote = git.contains("://") || git.split_once(':').is_some_and(|(h, _)| h.contains('@'));
    if remote {
        return git.to_owned();
    }
    let p = root.join(git);
    p.canonicalize()
        .unwrap_or_else(|_| lexical(&p))
        .display()
        .to_string()
}

fn lexical(p: &Path) -> std::path::PathBuf {
    use std::path::Component;
    let mut out = std::path::PathBuf::new();
    for c in p.components() {
        match c {
            Component::ParentDir => {
                out.pop();
            }
            Component::CurDir => {}
            c => out.push(c),
        }
    }
    out
}

/// FNV-1a, for a short stable directory name: two portals with one name in different checkouts
/// keep separate clones.
fn short_hash(s: &str) -> String {
    let h = s.bytes().fold(0xcbf2_9ce4_8422_2325_u64, |h, b| {
        (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3)
    });
    format!("{:08x}", h >> 32)
}

/// The directory a parent keeps its children's clones in.
pub fn clones_dir(root: &Path, portal: &str) -> PathBuf {
    let abs = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    let name: String = portal
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect();
    state_dir()
        .join("federation")
        .join(format!("{name}-{}", short_hash(&abs.display().to_string())))
}

fn has_named(doc: &Value, list: &str, name: &str) -> bool {
    doc[list]
        .as_array()
        .is_some_and(|a| a.iter().any(|x| x["name"] == name))
}

fn push(doc: &mut Value, list: &str, item: Value) {
    let Some(o) = doc.as_object_mut() else {
        return;
    };
    if let Value::Array(a) = o.entry(list).or_insert_with(|| json!([])) {
        a.push(item);
    }
}

fn portal_kind(connector: &str, id: &str, set: &str) -> Value {
    json!({
        "name": PORTAL_KIND,
        "identityFrom": [format!("connector:{connector}")],
        "fields": {
            "name": {"type": "string"}, "key": {"type": "string"}, "title": {"type": "string"},
            "owner": {"type": "string"}, "path": {"type": "string"}, "dir": {"type": "string"},
            "actions": {"type": "list", "items": "object"}, "runner": {"type": "object"},
            "git": {"type": "object"}, "error": {"type": "string"}, "warning": {"type": "string"}
        },
        "sources": {"default": [format!("connector:{connector}")]},
        "map": [{"from": format!("{connector}/{}", if connector == "self" { "repo" } else { "portal" }),
                 "id": id, "set": set}]
    })
}

fn expand_children(root: &Path, doc: &mut Value) {
    let Some(children) = doc["federation"]["children"].as_array().cloned() else {
        return;
    };
    if children.is_empty() {
        return;
    }
    let portal = doc["portal"]["name"]
        .as_str()
        .unwrap_or("portal")
        .to_owned();
    let git: Vec<Value> = children
        .iter()
        .filter_map(|c| {
            let url = c["git"].as_str()?;
            let key = c["key"]
                .as_str()
                .map_or_else(|| default_key(url), str::to_owned);
            Some(json!({"key": key, "url": clone_url(root, url),
                        "branch": c["branch"].as_str().unwrap_or("main")}))
        })
        .collect();
    let keys: Vec<Value> = git.iter().map(|g| g["key"].clone()).collect();
    if !has_named(doc, "connectors", CONNECTOR) {
        push(
            doc,
            "connectors",
            json!({
                "name": CONNECTOR, "package": "builtin:workspace", "runtime": "in-image",
                "config": {"root": clones_dir(root, &portal).display().to_string(), "crates": false,
                           "portals": ["*/.toolbox/toolbox.yaml"], "git": git},
                "records": ["portal"], "refresh": {"portal": ["external_ids"]},
                "freshness": {"portal": "5m"}
            }),
        );
    }
    if !has_named(doc, "kinds", PORTAL_KIND) {
        let set = "{'name': record.name, 'key': record.key, \
                   'title': has(record.title) ? record.title : null, \
                   'owner': record.owner, 'path': record.path, 'dir': record.dir, \
                   'actions': record.actions, 'runner': record.runner, \
                   'git': has(record.git) ? record.git : null, 'error': record.error, \
                   'warning': has(record.warning) ? record.warning : null}";
        push(
            doc,
            "kinds",
            portal_kind(CONNECTOR, "'portal:' + record.key", set),
        );
    }
    let fed = &mut doc["federation"];
    if fed.get("portals").is_none_or(Value::is_null) {
        fed["portals"] = json!(PORTAL_KIND);
    }
    if fed["runners"]["local"].is_null() {
        if !fed["runners"].is_object() {
            fed["runners"] = json!({});
        }
        fed["runners"]["local"] = json!({"portals": keys});
    }
}

fn expand_self(root: &Path, doc: &mut Value) {
    let on_portal = doc["actions"]
        .as_array()
        .is_some_and(|a| a.iter().any(|x| x["kind"] == PORTAL_KIND));
    if !on_portal || has_named(doc, "kinds", PORTAL_KIND) || has_named(doc, "connectors", "self") {
        return;
    }
    let p = &doc["portal"];
    let name = p["name"].as_str().unwrap_or("portal").to_owned();
    let lit = |v: &Value| serde_json::to_string(v.as_str().unwrap_or("")).unwrap_or_default();
    let set = format!(
        "{{'name': {}, 'key': {}, 'title': {}, 'owner': {}}}",
        lit(&p["name"]),
        lit(&p["name"]),
        lit(if p["title"].is_string() {
            &p["title"]
        } else {
            &p["name"]
        }),
        lit(&p["owner"]),
    );
    let repo = if root.file_name().is_some_and(|n| n == ".toolbox") {
        root.parent().unwrap_or(root)
    } else {
        root
    };
    let repo = repo.canonicalize().unwrap_or_else(|_| repo.to_path_buf());
    push(
        doc,
        "connectors",
        json!({
            "name": "self", "package": "builtin:workspace", "runtime": "in-image",
            "config": {"root": repo.display().to_string(), "crates": false, "repo": true},
            "records": ["repo"], "refresh": {"repo": ["external_ids"]}, "freshness": {"repo": "5m"}
        }),
    );
    let id = serde_json::to_string(&format!("portal:{name}")).unwrap_or_default();
    push(doc, "kinds", portal_kind("self", &id, &set));
}

/// Fill in what `federation.children` and actions on the portal's own entity declare.
pub fn expand(root: &Path, doc: &mut Value) {
    expand_children(root, doc);
    expand_self(root, doc);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_come_from_the_url() {
        assert_eq!(default_key("../origins/alpha.git"), "alpha");
        assert_eq!(
            default_key("https://github.com/acme/Pay-Svc.git/"),
            "pay-svc"
        );
        assert_eq!(default_key("git@github.com:acme/web"), "web");
    }

    #[test]
    fn children_declare_a_connector_a_kind_and_the_local_runner() {
        let root = Path::new("/org");
        let mut doc = json!({"portal": {"name": "org", "owner": "group:p"},
            "federation": {"children": [{"git": "../origins/alpha.git"},
                                        {"git": "https://x/acme/beta.git", "branch": "trunk", "key": "b"}]}});
        expand(root, &mut doc);
        let c = &doc["connectors"][0];
        assert_eq!(c["name"], "federation");
        assert_eq!(
            c["config"]["git"][0],
            json!({"key": "alpha", "url": "/origins/alpha.git", "branch": "main"})
        );
        assert_eq!(c["config"]["git"][1]["url"], "https://x/acme/beta.git");
        assert_eq!(c["config"]["git"][1]["branch"], "trunk");
        assert_eq!(doc["kinds"][0]["name"], "Portal");
        assert_eq!(doc["federation"]["portals"], "Portal");
        assert_eq!(
            doc["federation"]["runners"]["local"]["portals"],
            json!(["alpha", "b"])
        );
    }

    #[test]
    fn what_the_author_declares_wins() {
        let mut doc = json!({"portal": {"name": "org", "owner": "group:p"},
            "kinds": [{"name": "Portal"}],
            "federation": {"portals": "Portal", "children": [{"git": "/o/a.git"}],
                           "runners": {"local": {"portals": []}}}});
        expand(Path::new("/org"), &mut doc);
        assert_eq!(doc["kinds"].as_array().unwrap().len(), 1);
        assert_eq!(doc["federation"]["runners"]["local"]["portals"], json!([]));
    }

    #[test]
    fn an_action_on_portal_is_on_the_portal_itself() {
        let mut doc = json!({"portal": {"name": "alpha", "owner": "group:m"},
            "actions": [{"name": "x", "kind": "Portal"}]});
        expand(Path::new("/r/alpha/.toolbox"), &mut doc);
        assert_eq!(doc["connectors"][0]["config"]["root"], "/r/alpha");
        assert_eq!(doc["kinds"][0]["map"][0]["id"], "\"portal:alpha\"");
        assert_eq!(doc["kinds"][0]["map"][0]["from"], "self/repo");
        let mut doc = json!({"portal": {"name": "alpha", "owner": "group:m"},
            "actions": [{"name": "x", "kind": "Repository"}]});
        expand(Path::new("/r"), &mut doc);
        assert!(doc.get("connectors").is_none());
    }
}
