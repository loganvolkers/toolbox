use super::remove_child_actions;
use seikika_model::Manifest;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::io::Read;
use std::path::{Component, Path};

const MAX_BYTES: u64 = 16 * 1024 * 1024;

/// Parent-staged artifact data, loaded once per server or offline report invocation.
#[derive(Debug, Default)]
pub struct ReportPublications(Vec<(bool, Value)>);

impl ReportPublications {
    /// Load bounded JSON documents from the portal directory. Missing or invalid artifacts
    /// are represented as unavailable sources so the parent and healthy siblings still render.
    pub fn load(root: &Path, manifest: &Manifest) -> Self {
        let Some(federation) = &manifest.federation else {
            return Self::default();
        };
        Self(
            federation
                .report_sources
                .iter()
                .map(|(key, source)| {
                    let mut entry = json!({"key": key, "title": source.title});
                    match read(root, Path::new(&source.path)) {
                        Ok((digest, report)) => {
                            entry["digest"] = json!(digest);
                            entry["report"] = report;
                        }
                        Err(error) => {
                            tracing::warn!(source = key, error, "report publication unavailable");
                            entry["error"] = json!(error);
                        }
                    }
                    (source.public, entry)
                })
                .collect(),
        )
    }

    /// Add approved publications to a report. An explicit source owns its namespace even
    /// when its publication is unavailable. Private artifacts are visible to operators only.
    pub(super) fn append(&self, children: &mut Vec<Value>, admin: bool) {
        for (public, publication) in &self.0 {
            if *public || admin {
                children.retain(|child| child["key"] != publication["key"]);
                children.push(publication.clone());
            }
        }
    }
}

fn read(root: &Path, relative: &Path) -> Result<(String, Value), &'static str> {
    if !relative
        .components()
        .all(|part| matches!(part, Component::Normal(_)))
    {
        return Err("The report publication path must stay inside the portal.");
    }
    let root = root
        .canonicalize()
        .map_err(|_| "The portal directory could not be read.")?;
    let mut path = root;
    for part in relative {
        path.push(part);
        let metadata = path
            .symlink_metadata()
            .map_err(|_| "The report publication is missing.")?;
        if metadata.is_symlink() {
            return Err("The report publication path cannot contain a symbolic link.");
        }
    }
    let metadata = path
        .metadata()
        .map_err(|_| "The report publication is missing.")?;
    if !metadata.is_file() || metadata.len() > MAX_BYTES {
        return Err("The report publication must be a regular file of at most 16 MiB.");
    }
    let mut bytes = Vec::new();
    std::fs::File::open(path)
        .and_then(|file| file.take(MAX_BYTES + 1).read_to_end(&mut bytes))
        .map_err(|_| "The report publication could not be read.")?;
    if bytes.len() as u64 > MAX_BYTES {
        return Err("The report publication exceeds 16 MiB.");
    }
    let mut report: Value =
        serde_json::from_slice(&bytes).map_err(|_| "The report publication is invalid JSON.")?;
    if report["portal"].as_str().is_none()
        || ["catalog", "scorecards", "scores"]
            .iter()
            .any(|field| !report[*field].is_object())
    {
        return Err("The publication is missing required Seikika report fields.");
    }
    remove_child_actions(&mut report);
    let digest = Sha256::digest(&bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    Ok((digest, report))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_public_artifacts_replace_a_guest_source() {
        let public = json!({"key": "public", "report": {"portal": "staged"}});
        let private = json!({"key": "private", "report": {"portal": "restricted"}});
        let publications =
            ReportPublications(vec![(true, public.clone()), (false, private.clone())]);
        let previous = json!({"key": "public", "report": {"portal": "git"}});
        let sibling = json!({"key": "sibling"});
        let mut guest = vec![previous.clone(), sibling.clone()];
        publications.append(&mut guest, false);
        assert_eq!(guest, vec![sibling.clone(), public.clone()]);
        let mut operator = vec![previous, sibling.clone()];
        publications.append(&mut operator, true);
        assert_eq!(operator, vec![sibling, public, private]);
    }

    #[test]
    fn publication_sanitizes_nested_actions_and_handles_bad_files() {
        let root = std::env::temp_dir().join(format!("seikika-publication-{}", std::process::id()));
        std::fs::create_dir(&root).unwrap();
        std::fs::write(root.join("broken.json"), b"{").unwrap();
        let expected = json!({"portal": "test", "catalog": {}, "scorecards": {}, "scores": {}});
        let mut report = expected.clone();
        report["actions"] = json!([{"name": "test"}]);
        report["children"] = json!([{"key": "nested", "report": report.clone()}]);
        std::fs::write(root.join("report.json"), report.to_string()).unwrap();
        assert!(read(&root, Path::new("missing.json")).is_err());
        assert!(read(&root, Path::new("broken.json")).is_err());
        assert!(read(&root, Path::new("../report.json")).is_err());
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(root.join("report.json"), root.join("symlink.json"))
                .unwrap();
            assert!(read(&root, Path::new("symlink.json")).is_err());
        }
        let (digest, report) = read(&root, Path::new("report.json")).unwrap();
        assert_eq!(digest.len(), 64);
        assert!(report.get("actions").is_none());
        assert_eq!(report["children"][0]["report"], expected);
        std::fs::remove_dir_all(root).unwrap();
    }
}
