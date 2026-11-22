//! Mounted secrets directory (`TOOLBOX_SECRETS_DIR` / v1-spec §10.2).

use std::path::{Path, PathBuf};

use crate::backends::SecretBackend;
use crate::bytes::SecretBytes;
use crate::error::SecretError;

#[derive(Debug, Clone)]
pub struct FileBackend {
    dir: PathBuf,
}

impl FileBackend {
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        Self { dir: dir.into() }
    }

    fn candidates(dir: &Path, logical: &str) -> Vec<PathBuf> {
        let upper = logical.to_uppercase().replace('-', "_");
        vec![
            dir.join(logical),
            dir.join(format!("{logical}.txt")),
            dir.join(&upper),
            dir.join(format!("{upper}.txt")),
        ]
    }
}

impl SecretBackend for FileBackend {
    fn name(&self) -> &'static str {
        "file"
    }

    fn get(&self, logical: &str) -> Result<Option<SecretBytes>, SecretError> {
        if !self.dir.exists() {
            return Ok(None);
        }
        for path in Self::candidates(&self.dir, logical) {
            if path.is_file() {
                let text = std::fs::read_to_string(&path)?;
                // Trim a single trailing newline common in mounted secret files.
                let trimmed = text.strip_suffix('\n').unwrap_or(&text);
                let trimmed = trimmed.strip_suffix('\r').unwrap_or(trimmed);
                return Ok(Some(SecretBytes::new(trimmed.to_owned())));
            }
        }
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_logical_and_upper_names() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("jira-app-token"), "tok\n").unwrap();
        let b = FileBackend::new(dir.path());
        assert_eq!(b.get("jira-app-token").unwrap().unwrap().expose(), "tok");

        std::fs::write(dir.path().join("GCP_SA_KEY.txt"), "json").unwrap();
        assert_eq!(b.get("gcp-sa-key").unwrap().unwrap().expose(), "json");
    }
}
