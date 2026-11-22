//! `TOOLBOX_SECRET_<NAME>` environment backend.

use std::collections::BTreeMap;
use std::sync::Arc;

use crate::backends::SecretBackend;
use crate::bytes::SecretBytes;
use crate::config::env_var_for;
use crate::error::SecretError;

type EnvLookup = Arc<dyn Fn(&str) -> Option<String> + Send + Sync>;

/// Reads secrets from the process environment (or an injected map in tests).
pub struct EnvBackend {
    lookup: EnvLookup,
}

impl Default for EnvBackend {
    fn default() -> Self {
        Self {
            lookup: Arc::new(|logical| std::env::var(env_var_for(logical)).ok()),
        }
    }
}

impl EnvBackend {
    /// Production: `std::env::var(TOOLBOX_SECRET_*)`.
    pub fn process() -> Self {
        Self::default()
    }

    /// Test helper: logical name → value (skips process env).
    pub fn from_map(map: BTreeMap<String, String>) -> Self {
        Self {
            lookup: Arc::new(move |logical| map.get(logical).cloned()),
        }
    }
}

impl SecretBackend for EnvBackend {
    fn name(&self) -> &'static str {
        "env"
    }

    fn get(&self, logical: &str) -> Result<Option<SecretBytes>, SecretError> {
        Ok((self.lookup)(logical).map(SecretBytes::new))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn env_var_naming() {
        assert_eq!(
            env_var_for("github-app-private-key"),
            "TOOLBOX_SECRET_GITHUB_APP_PRIVATE_KEY"
        );
    }

    #[test]
    fn map_lookup() {
        let b = EnvBackend::from_map(BTreeMap::from([(
            "github-app-private-key".into(),
            "pem-bytes".into(),
        )]));
        assert_eq!(
            b.get("github-app-private-key").unwrap().unwrap().expose(),
            "pem-bytes"
        );
        assert!(b.get("missing").unwrap().is_none());
    }
}
