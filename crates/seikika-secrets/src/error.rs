//! Errors from secret backends and the resolver.

/// Failure talking to a backend or loading configuration. Missing secrets are `Ok(None)`, not errors.
#[derive(Debug, thiserror::Error)]
pub enum SecretError {
    #[error("secrets config: {0}")]
    Config(String),
    #[error("secret backend {backend}: {message}")]
    Backend {
        backend: &'static str,
        message: String,
    },
    #[error("secret {name} is required but unresolved")]
    Required { name: String },
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

impl SecretError {
    pub fn backend(backend: &'static str, message: impl Into<String>) -> Self {
        Self::Backend {
            backend,
            message: message.into(),
        }
    }
}

/// Warnings are printed to stderr; wording matches the historical `resolve_config` message.
pub fn warn_unresolved(connector: &str, placeholder: &str, key: &str) {
    eprintln!("seikika: connector {connector}: {placeholder} is not set; leaving `{key}` out");
}
