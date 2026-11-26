//! Pluggable secret backends for Seikika connector `${secret:…}` resolution.
//!
//! See [docs/design/secret-vault.md]. Extends D55: placeholders stay unresolved until runtime.
//!
//! Backends (first hit wins): env (`SEIKIKA_SECRET_*`), file (`SEIKIKA_SECRETS_DIR`), then
//! configured remotes (Vault KV, AWS Secrets Manager, GCP Secret Manager, Azure Key Vault).

pub mod b64;
pub mod backends;
pub mod bytes;
pub mod config;
pub mod error;
pub mod http;
pub mod resolver;

pub use backends::SecretBackend;
pub use bytes::SecretBytes;
pub use config::{BackendConfig, SecretsConfig};
pub use error::SecretError;
pub use resolver::{
    SecretResolver, global, resolve_config, resolve_value, resolver_from_env, set_global,
};
