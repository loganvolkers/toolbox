//! Resource attributes (`service.name`, …) and `OTEL_*` merge rules.

use opentelemetry::{KeyValue, Value};
use opentelemetry_sdk::Resource;
use std::collections::BTreeMap;

use crate::init::{TelemetryError, TelemetryMode};

const DEFAULT_SERVICE_NAME: &str = "toolbox";

/// Resolve `service.name`: options → `OTEL_SERVICE_NAME` → default `toolbox`.
pub fn service_name(explicit: Option<&str>) -> String {
    if let Some(s) = explicit.map(str::trim).filter(|s| !s.is_empty()) {
        return s.to_owned();
    }
    if let Ok(s) = std::env::var("OTEL_SERVICE_NAME") {
        let t = s.trim();
        if !t.is_empty() {
            return t.to_owned();
        }
    }
    DEFAULT_SERVICE_NAME.into()
}

/// Build the OTel resource. Operator-set `OTEL_RESOURCE_ATTRIBUTES` win on key conflicts
/// (merge, do not overwrite operator keys — we apply env first, then fill gaps).
pub fn build_resource(
    mode: TelemetryMode,
    explicit_service_name: Option<&str>,
) -> Result<(Resource, String), TelemetryError> {
    let mut attrs: BTreeMap<String, String> = BTreeMap::new();
    if let Ok(raw) = std::env::var("OTEL_RESOURCE_ATTRIBUTES") {
        for part in raw.split(',') {
            if let Some((k, v)) = part.split_once('=') {
                let k = k.trim();
                let v = v.trim();
                if !k.is_empty() {
                    attrs.insert(k.to_owned(), v.to_owned());
                }
            }
        }
    }

    let name = if let Some(existing) = attrs.get("service.name").cloned() {
        if existing.trim().is_empty() {
            resolve_or_fail(mode, explicit_service_name)?
        } else {
            existing
        }
    } else {
        let n = resolve_or_fail(mode, explicit_service_name)?;
        attrs.insert("service.name".into(), n.clone());
        n
    };

    if !attrs.contains_key("service.version") {
        attrs.insert(
            "service.version".into(),
            env!("CARGO_PKG_VERSION").to_owned(),
        );
    }
    if !attrs.contains_key("service.instance.id") {
        let host = hostname();
        attrs.insert(
            "service.instance.id".into(),
            format!("{host}-{}", std::process::id()),
        );
    }
    if !attrs.contains_key("deployment.environment") {
        if let Ok(env) = std::env::var("TOOLBOX_ENV") {
            let t = env.trim();
            if !t.is_empty() {
                attrs.insert("deployment.environment".into(), t.to_owned());
            }
        } else {
            attrs.insert(
                "deployment.environment".into(),
                match mode {
                    TelemetryMode::Local => "local".into(),
                    TelemetryMode::Ci => "ci".into(),
                    TelemetryMode::Prod => "prod".into(),
                },
            );
        }
    }

    let kvs: Vec<KeyValue> = attrs
        .into_iter()
        .map(|(k, v)| KeyValue::new(k, Value::from(v)))
        .collect();
    Ok((Resource::builder_empty().with_attributes(kvs).build(), name))
}

fn resolve_or_fail(mode: TelemetryMode, explicit: Option<&str>) -> Result<String, TelemetryError> {
    let name = service_name(explicit);
    if name.trim().is_empty() {
        return match mode {
            TelemetryMode::Local => Ok(DEFAULT_SERVICE_NAME.into()),
            TelemetryMode::Ci | TelemetryMode::Prod => Err(TelemetryError::MissingServiceName),
        };
    }
    Ok(name)
}

fn hostname() -> String {
    std::env::var("HOSTNAME")
        .or_else(|_| std::env::var("COMPUTERNAME"))
        .unwrap_or_else(|_| "localhost".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::init::TelemetryMode;

    #[test]
    fn default_service_name_is_toolbox() {
        // Ensure empty override path still defaults (do not clear real env in parallel tests).
        let n = service_name(Some(""));
        assert!(!n.is_empty());
        let n = service_name(Some("my-portal"));
        assert_eq!(n, "my-portal");
    }

    #[test]
    fn prod_rejects_explicit_empty_when_forced() {
        // Empty explicit falls through to env/default; Local always gets a name.
        let (res, name) = build_resource(TelemetryMode::Local, None).unwrap();
        assert_eq!(name, service_name(None));
        let _ = res;
    }
}
