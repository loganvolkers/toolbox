//! Secret backend implementations.

mod env;
mod file;

#[cfg(feature = "aws")]
pub mod aws;
#[cfg(feature = "azure")]
pub mod azure;
#[cfg(feature = "gcp")]
pub mod gcp;
#[cfg(feature = "vault")]
pub mod vault;

pub use env::EnvBackend;
pub use file::FileBackend;

use crate::bytes::SecretBytes;
use crate::error::SecretError;

/// A source of secret values keyed by logical name (`github-app-private-key`).
pub trait SecretBackend: Send + Sync {
    fn name(&self) -> &'static str;
    fn get(&self, logical: &str) -> Result<Option<SecretBytes>, SecretError>;
}

/// Apply an optional logical→provider map, then fall back to the logical name.
pub(crate) fn mapped<'a>(
    map: &'a std::collections::BTreeMap<String, String>,
    logical: &'a str,
) -> &'a str {
    map.get(logical).map(String::as_str).unwrap_or(logical)
}
