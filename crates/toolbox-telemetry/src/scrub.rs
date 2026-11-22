//! Shared redaction for operational logs and (later) analytics.

/// Replace values that look like secrets or high-cardinality PII with a placeholder.
pub fn scrub_value(key: &str, value: &str) -> String {
    let k = key.to_ascii_lowercase();
    if k.contains("password")
        || k.contains("secret")
        || k.contains("token")
        || k.contains("authorization")
        || k.contains("passphrase")
        || k.contains("api_key")
        || k.contains("apikey")
    {
        return "[redacted]".into();
    }
    // Keep operational values short so entity bodies / query text cannot leak by accident.
    const MAX: usize = 256;
    if value.len() > MAX {
        format!("{}…", &value[..MAX])
    } else {
        value.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_token_keys() {
        assert_eq!(scrub_value("Bearer_Token", "abc"), "[redacted]");
        assert_eq!(scrub_value("path", "ok"), "ok");
    }
}
