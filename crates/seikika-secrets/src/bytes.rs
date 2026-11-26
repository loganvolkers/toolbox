//! Secret payload that never prints its contents.

use std::fmt;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Bytes of a resolved secret. `Debug` / `Display` always redact; drops zeroize the buffer.
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct SecretBytes {
    value: String,
}

impl SecretBytes {
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
        }
    }

    /// Borrow the secret. Prefer not retaining the return value beyond the injection site.
    pub fn expose(&self) -> &str {
        &self.value
    }

    pub fn into_string(mut self) -> String {
        std::mem::take(&mut self.value)
    }
}

impl fmt::Debug for SecretBytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SecretBytes([redacted])")
    }
}

impl fmt::Display for SecretBytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[redacted]")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_debug_and_display() {
        let s = SecretBytes::new("super-secret-pem");
        assert!(!format!("{s:?}").contains("super-secret"));
        assert!(!format!("{s}").contains("super-secret"));
        assert!(format!("{s:?}").contains("redacted"));
        assert_eq!(s.expose(), "super-secret-pem");
    }
}
