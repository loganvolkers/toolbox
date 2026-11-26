//! Loading portal sources: one YAML/JSON manifest, plus files merged through `include` (D54).
//!
//! Merge rules for `include`: objects merge key by key, arrays concatenate (root file first, then
//! included files in sorted path order), and two different scalars at the same path are an error.
//! `include` itself never reaches the canonical manifest (D53).

use serde_json::Value;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LoadError {
    #[error("{path}: {source}")]
    Io {
        path: String,
        source: std::io::Error,
    },
    #[error("{path}: not valid YAML or JSON: {message}")]
    Parse { path: String, message: String },
    #[error("{path}: package {spec}: {message}")]
    Package {
        path: String,
        spec: String,
        message: String,
    },
    #[error("{path}: `include` must be a list of file paths, globs or {{package, with}} entries")]
    BadInclude { path: String },
    #[error("{path}: include pattern `{pattern}` matched no files")]
    NoMatch { path: String, pattern: String },
    #[error("{a} and {b} both set {pointer} to different values")]
    Conflict {
        a: String,
        b: String,
        pointer: String,
    },
    #[error(
        "{path}: TypeScript sources need the builder, which `seikika validate` doesn't run yet; print the manifest with it and validate that file"
    )]
    Builder { path: String },
}

/// A loaded portal source.
#[derive(Debug, Clone)]
pub struct Source {
    /// Directory the manifest lives in; relative paths in the manifest resolve against it.
    pub root: PathBuf,
    /// The root manifest file, relative to `root`.
    pub manifest_file: String,
    /// Included files, relative to `root`, in merge order.
    pub included: Vec<String>,
    /// The merged document, without `include`.
    pub document: Value,
    /// Where each included value landed: merged pointers that came from another file.
    pub origins: Vec<Origin>,
}

/// One value merged from an included file: its pointer in the merged document, the file, and its
/// pointer in that file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Origin {
    pub merged: String,
    pub file: String,
    pub local: String,
}

impl Source {
    /// The file and file-local pointer for a pointer into the merged document: the included file
    /// the value came from, else the root manifest.
    pub fn locate(&self, pointer: &str) -> (String, String) {
        let within = |o: &&Origin| {
            pointer == o.merged
                || pointer
                    .strip_prefix(o.merged.as_str())
                    .is_some_and(|r| r.starts_with('/'))
        };
        match self
            .origins
            .iter()
            .filter(within)
            .max_by_key(|o| o.merged.len())
        {
            Some(o) => (
                o.file.clone(),
                format!("{}{}", o.local, &pointer[o.merged.len()..]),
            ),
            None => (self.manifest_file.clone(), pointer.to_owned()),
        }
    }

    /// Point each diagnostic labelled with the root manifest at the file its value came from.
    pub fn relocate(&self, diagnostics: &mut [crate::Diagnostic]) {
        for d in diagnostics
            .iter_mut()
            .filter(|d| d.file == self.manifest_file)
        {
            (d.file, d.path) = self.locate(&d.path);
        }
    }
}

/// Find the manifest for `path`: a file, or a directory containing `seikika.yaml`, `seikika.yml` or
/// `seikika.json`.
pub fn resolve(path: &Path) -> Result<(PathBuf, String), LoadError> {
    if path.is_dir() {
        for name in ["seikika.yaml", "seikika.yml", "seikika.json"] {
            if path.join(name).is_file() {
                return Ok((path.to_path_buf(), name.to_string()));
            }
        }
        if path.join("seikika.config.ts").is_file() {
            return Err(LoadError::Builder {
                path: path.join("seikika.config.ts").display().to_string(),
            });
        }
        return Err(LoadError::Io {
            path: path.display().to_string(),
            source: std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "no seikika.yaml, seikika.yml or seikika.json here",
            ),
        });
    }
    if path.extension().is_some_and(|e| e == "ts") {
        return Err(LoadError::Builder {
            path: path.display().to_string(),
        });
    }
    let root = path.parent().map(Path::to_path_buf).unwrap_or_default();
    let root = if root.as_os_str().is_empty() {
        PathBuf::from(".")
    } else {
        root
    };
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    Ok((root, name))
}

fn read_doc(root: &Path, rel: &str) -> Result<Value, LoadError> {
    let full = root.join(rel);
    let text = std::fs::read_to_string(&full).map_err(|source| LoadError::Io {
        path: rel.to_string(),
        source,
    })?;
    if rel.ends_with(".json") {
        serde_json::from_str(&text).map_err(|e| LoadError::Parse {
            path: rel.to_string(),
            message: e.to_string(),
        })
    } else {
        serde_norway::from_str(&text).map_err(|e| LoadError::Parse {
            path: rel.to_string(),
            message: e.to_string(),
        })
    }
}

/// Expand an include pattern relative to `root`. Supports literal paths and `*` within the final
/// path segment (e.g. `policies/*.yaml`).
/// [`expand`] for other directories (package fragments).
pub(crate) fn expand_in(root: &Path, pattern: &str) -> Vec<String> {
    expand(root, pattern)
}

fn expand(root: &Path, pattern: &str) -> Vec<String> {
    let (dir, file_pat) = match pattern.rsplit_once('/') {
        Some((d, f)) => (d.to_string(), f.to_string()),
        None => (String::new(), pattern.to_string()),
    };
    if !file_pat.contains('*') {
        return if root.join(pattern).is_file() {
            vec![pattern.to_string()]
        } else {
            vec![]
        };
    }
    let parts: Vec<&str> = file_pat.split('*').collect();
    let mut out: Vec<String> = std::fs::read_dir(root.join(&dir))
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.path().is_file())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|name| glob_match(&parts, name))
        .map(|name| {
            if dir.is_empty() {
                name
            } else {
                format!("{dir}/{name}")
            }
        })
        .collect();
    out.sort();
    out
}

fn glob_match(parts: &[&str], name: &str) -> bool {
    let (first, rest) = (parts[0], &parts[1..]);
    let Some(mut tail) = name.strip_prefix(first) else {
        return false;
    };
    for (i, part) in rest.iter().enumerate() {
        if i == rest.len() - 1 {
            return tail.ends_with(part);
        }
        match tail.find(part) {
            Some(pos) => tail = &tail[pos + part.len()..],
            None => return false,
        }
    }
    true
}

/// Where a merge reads from and records to: the included file, its pointer, and the origins list.
struct Incoming<'a> {
    file: &'a str,
    local: String,
    origins: &'a mut Vec<Origin>,
}

impl Incoming<'_> {
    fn record(&mut self, merged: String, local: String) {
        self.origins.push(Origin {
            merged,
            file: self.file.to_owned(),
            local,
        });
    }
}

fn merge(
    into: &mut Value,
    from: Value,
    pointer: &str,
    a: &str,
    src: &mut Incoming,
) -> Result<(), LoadError> {
    match (into, from) {
        (Value::Object(x), Value::Object(y)) => {
            let base = src.local.clone();
            for (k, v) in y {
                let p = format!("{pointer}/{k}");
                let l = format!("{base}/{k}");
                match x.get_mut(&k) {
                    Some(existing) => {
                        src.local = l;
                        merge(existing, v, &p, a, src)?
                    }
                    None => {
                        src.record(p, l);
                        x.insert(k, v);
                    }
                }
            }
            src.local = base;
            Ok(())
        }
        // Top-level lists (connectors, kinds, scorecards, ...) concatenate. A list nested inside a
        // merged object, such as tools.<name>.versionArgs from two includes of one kit, must be
        // equal: concatenating it ran `semgrep --version --version`.
        (Value::Array(x), Value::Array(y)) if pointer.matches('/').count() <= 1 => {
            for (j, v) in y.into_iter().enumerate() {
                src.record(
                    format!("{pointer}/{}", x.len()),
                    format!("{}/{j}", src.local),
                );
                x.push(v);
            }
            Ok(())
        }
        (x, y) if *x == y => Ok(()),
        _ => Err(LoadError::Conflict {
            a: a.to_string(),
            b: src.file.to_string(),
            pointer: pointer.to_string(),
        }),
    }
}

/// Merge one included document into `document`, recording where its values landed.
fn merge_file(
    document: &mut Value,
    mut doc: Value,
    manifest_file: &str,
    file: &str,
    origins: &mut Vec<Origin>,
) -> Result<(), LoadError> {
    if let Some(o) = doc.as_object_mut() {
        o.remove("include"); // includes don't nest
    }
    let mut src = Incoming {
        file,
        local: String::new(),
        origins,
    };
    merge(document, doc, "", manifest_file, &mut src)
}

fn include_files(
    root: &Path,
    manifest_file: &str,
    pattern: &str,
    document: &mut Value,
    included: &mut Vec<String>,
    origins: &mut Vec<Origin>,
) -> Result<(), LoadError> {
    let files = expand(root, pattern);
    if files.is_empty() {
        return Err(LoadError::NoMatch {
            path: manifest_file.to_owned(),
            pattern: pattern.to_owned(),
        });
    }
    for file in files {
        let doc = read_doc(root, &file)?;
        merge_file(document, doc, manifest_file, &file, origins)?;
        included.push(file);
    }
    Ok(())
}

/// A package (D89): its fragments with `with` applied, after checking its pin in seikika.lock.
fn include_package(
    root: &Path,
    manifest_file: &str,
    entry: &serde_json::Map<String, Value>,
    document: &mut Value,
    included: &mut Vec<String>,
    origins: &mut Vec<Origin>,
) -> Result<(), LoadError> {
    let spec = entry["package"].as_str().unwrap_or_default();
    let err = |message: String| LoadError::Package {
        path: manifest_file.to_owned(),
        spec: spec.to_owned(),
        message,
    };
    let dir = crate::package::locate(root, spec);
    let lock: Value = std::fs::read_to_string(root.join("seikika.lock"))
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or(Value::Null);
    if let Some(pinned) = lock["packages"][spec].as_str() {
        let got = crate::package::content_hash(&dir);
        if got != pinned {
            return Err(err(format!(
                "content {got} doesn't match seikika.lock ({pinned})"
            )));
        }
    }
    let with = entry
        .get("with")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    for (label, doc) in crate::package::fragments(&dir, &with).map_err(err)? {
        merge_file(document, doc, manifest_file, &label, origins)?;
        included.push(label);
    }
    Ok(())
}

/// Load the portal at `path` (a manifest file or a portal directory).
pub fn load(path: &Path) -> Result<Source, LoadError> {
    let (root, manifest_file) = resolve(path)?;
    let mut document = read_doc(&root, &manifest_file)?;
    let mut included = Vec::new();
    let mut origins = Vec::new();
    if let Some(inc) = document.as_object_mut().and_then(|o| o.remove("include")) {
        let Value::Array(items) = inc else {
            return Err(LoadError::BadInclude {
                path: manifest_file.clone(),
            });
        };
        for item in items {
            match item {
                Value::String(pattern) => include_files(
                    &root,
                    &manifest_file,
                    &pattern,
                    &mut document,
                    &mut included,
                    &mut origins,
                )?,
                Value::Object(o) if o.get("package").is_some_and(Value::is_string) => {
                    include_package(
                        &root,
                        &manifest_file,
                        &o,
                        &mut document,
                        &mut included,
                        &mut origins,
                    )?
                }
                _ => {
                    return Err(LoadError::BadInclude {
                        path: manifest_file.clone(),
                    });
                }
            }
        }
    }
    crate::federation::expand(&root, &mut document);
    Ok(Source {
        root,
        manifest_file,
        included,
        document,
        origins,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn glob_matching() {
        let parts: Vec<&str> = "*.yaml".split('*').collect();
        assert!(glob_match(&parts, "approved-apps.yaml"));
        assert!(!glob_match(&parts, "approved-apps.yml"));
        let parts: Vec<&str> = "apps*.yaml".split('*').collect();
        assert!(!glob_match(&parts, "approved-apps.yaml"));
        assert!(glob_match(&parts, "apps-security.yaml"));
    }

    #[test]
    fn merge_rules() {
        let mut a = json!({"policies": {"x": 1}, "kinds": [{"name": "A"}]});
        let mut origins = Vec::new();
        let b = json!({"policies": {"y": 2}, "kinds": [{"name": "B", "fields": {}}]});
        merge_file(&mut a, b, "a", "b", &mut origins).unwrap();
        assert_eq!(
            a,
            json!({"policies": {"x": 1, "y": 2}, "kinds": [{"name": "A"}, {"name": "B", "fields": {}}]})
        );
        let err = merge_file(
            &mut a,
            json!({"policies": {"x": 3}}),
            "a",
            "b",
            &mut origins,
        );
        assert!(err.unwrap_err().to_string().contains("/policies/x"));
        // Diagnostics on included values name the file they came from, at their pointer there.
        let src = Source {
            root: PathBuf::from("."),
            manifest_file: "a".into(),
            included: vec!["b".into()],
            document: a,
            origins,
        };
        assert_eq!(
            src.locate("/kinds/1/fields"),
            ("b".into(), "/kinds/0/fields".into())
        );
        assert_eq!(
            src.locate("/policies/y"),
            ("b".into(), "/policies/y".into())
        );
        assert_eq!(
            src.locate("/kinds/0/name"),
            ("a".into(), "/kinds/0/name".into())
        );
        assert_eq!(src.locate("/kinds/10"), ("a".into(), "/kinds/10".into()));
    }

    #[test]
    fn nested_lists_from_two_includes_are_equal_or_conflict() {
        // Two includes of one kit that both declare the tool (a second semgrep rule set).
        let tool =
            || json!({"tools": {"semgrep": {"version": "1.163.0", "versionArgs": ["--version"]}}});
        let mut a = tool();
        let mut origins = Vec::new();
        merge_file(&mut a, tool(), "a", "b", &mut origins).unwrap();
        assert_eq!(a, tool());
        let other = json!({"tools": {"semgrep": {"versionArgs": ["-V"]}}});
        let err = merge_file(&mut a, other, "a", "c", &mut origins).unwrap_err();
        assert!(
            err.to_string().contains("/tools/semgrep/versionArgs"),
            "{err}"
        );
    }
}
