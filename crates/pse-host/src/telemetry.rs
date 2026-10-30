// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Passive native host resolution of the shared bounded telemetry contract.
//! No SDK, exporter, service, database or global environment is initialized here.
use crate::environment::Environment;
use percent_encoding::percent_decode_str;
use pse_diagnostics::telemetry_settings::{
    HEADERS_MAX_BYTES, HEADERS_MAX_COUNT, SETTINGS_ENV, SETTINGS_MAX_BYTES, SETTINGS_VERSION,
    Settings, SignalSettings, TELEMETRY_TIMEOUT_MAX_MS, TelemetryError, TelemetrySampler,
};
use serde::Deserialize;
use std::{
    collections::{BTreeMap, HashMap},
    ffi::OsStr,
    io::Read as _,
};

/// Public status contains no endpoint, credential, resource attribute or configuration path.
pub const STATUS_ENV: &str = "PSE_TELEMETRY_CONFIGURATION_STATUS";

fn invalid(message: &str) -> TelemetryError {
    TelemetryError(message.into())
}
fn text<'a>(environment: &'a Environment, name: &str) -> Result<Option<&'a str>, TelemetryError> {
    environment
        .get(OsStr::new(name))
        .map(|value| {
            value
                .to_str()
                .ok_or_else(|| invalid("telemetry environment selection is not UTF-8"))
        })
        .transpose()
}
fn number(environment: &Environment, name: &str, default: f64) -> Result<f64, TelemetryError> {
    let number = text(environment, name)?
        .map(str::parse::<f64>)
        .transpose()
        .map_err(|_| invalid("invalid telemetry numeric setting"))?
        .unwrap_or(default);
    if !number.is_finite() {
        return Err(invalid("invalid telemetry numeric setting"));
    }
    Ok(number)
}
fn integer(environment: &Environment, name: &str, default: u64) -> Result<u64, TelemetryError> {
    let value = number(environment, name, default as f64)?;
    if value.fract() != 0.0 || value < 0.0 || value >= u64::MAX as f64 {
        return Err(invalid("invalid telemetry integer setting"));
    }
    Ok(value as u64)
}
fn timeout(environment: &Environment, name: &str, default: f64) -> Result<f64, TelemetryError> {
    let value = number(environment, name, default)?;
    if value <= 0.0 || value > TELEMETRY_TIMEOUT_MAX_MS as f64 / 1000.0 {
        return Err(invalid(
            "telemetry timeout is outside the shared host bounds",
        ));
    }
    Ok(value)
}
fn attributes(value: &str) -> Result<HashMap<String, String>, TelemetryError> {
    let mut result = HashMap::new();
    for pair in value.split(',').filter(|pair| !pair.trim().is_empty()) {
        let (key, value) = pair
            .split_once('=')
            .ok_or_else(|| invalid("invalid telemetry attribute or header"))?;
        let key = percent_decode_str(key.trim())
            .decode_utf8()
            .map_err(|_| invalid("invalid telemetry attribute or header encoding"))?
            .into_owned();
        let value = percent_decode_str(value.trim())
            .decode_utf8()
            .map_err(|_| invalid("invalid telemetry attribute or header encoding"))?
            .into_owned();
        if key.is_empty() || key.contains(['\r', '\n']) || value.contains(['\r', '\n']) {
            return Err(invalid("invalid telemetry attribute or header"));
        }
        result.insert(key, value);
    }
    if result.len() > HEADERS_MAX_COUNT
        || result
            .iter()
            .map(|(key, value)| key.len() + value.len())
            .sum::<usize>()
            > HEADERS_MAX_BYTES
    {
        return Err(invalid(
            "telemetry attributes or headers exceed the shared bounds",
        ));
    }
    Ok(result)
}
#[derive(Deserialize)]
struct LocalConfiguration {
    schema_version: u32,
    otlp_http_endpoint: Option<String>,
}
fn local_configuration(
    environment: &Environment,
) -> Result<Option<LocalConfiguration>, TelemetryError> {
    let directory = crate::paths::HostPaths::new(environment)
        .telemetry()
        .map_err(|_| invalid("telemetry state selection has no home directory"))?;
    let path = directory.join("configuration.json");
    match std::fs::metadata(&path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err(invalid("telemetry configuration cannot be inspected")),
        Ok(metadata) if !metadata.is_file() => return Ok(None),
        Ok(metadata) if metadata.len() > SETTINGS_MAX_BYTES as u64 => {
            return Err(invalid("telemetry configuration exceeds its bound"));
        }
        Ok(_) => {}
    }
    let mut bytes = Vec::new();
    std::fs::File::open(path)
        .and_then(|file| {
            file.take(SETTINGS_MAX_BYTES as u64 + 1)
                .read_to_end(&mut bytes)
        })
        .map_err(|_| invalid("telemetry configuration cannot be read"))?;
    if bytes.len() > SETTINGS_MAX_BYTES {
        return Err(invalid("telemetry configuration exceeds its bound"));
    }
    let config: LocalConfiguration =
        serde_json::from_slice(&bytes).map_err(|_| invalid("invalid telemetry configuration"))?;
    if config.schema_version != SETTINGS_VERSION {
        return Err(invalid("unsupported telemetry configuration schema"));
    }
    Ok(Some(config))
}

/// Resolve OTEL signal overrides, base selection and optional local state without starting telemetry.
pub fn resolve(service: &str, environment: &Environment) -> Result<Settings, TelemetryError> {
    let local = local_configuration(environment)?;
    let base = text(environment, "OTEL_EXPORTER_OTLP_ENDPOINT")?
        .or_else(|| {
            local
                .as_ref()
                .and_then(|config| config.otlp_http_endpoint.as_deref())
        })
        .unwrap_or("");
    let mut selected = !base.is_empty();
    for signal in ["TRACES", "LOGS", "METRICS"] {
        selected |= text(
            environment,
            &format!("OTEL_EXPORTER_OTLP_{signal}_ENDPOINT"),
        )?
        .is_some_and(|endpoint| !endpoint.is_empty());
    }
    let disabled = text(environment, "OTEL_SDK_DISABLED")?
        .is_some_and(|value| value.eq_ignore_ascii_case("true"))
        || !selected;
    let base_headers = text(environment, "OTEL_EXPORTER_OTLP_HEADERS")?.unwrap_or("");
    let base_protocol =
        text(environment, "OTEL_EXPORTER_OTLP_PROTOCOL")?.unwrap_or("http/protobuf");
    let base_timeout = timeout(environment, "OTEL_EXPORTER_OTLP_TIMEOUT", 5.0)?;
    let signal = |name: &str| -> Result<SignalSettings, TelemetryError> {
        let exporter = text(environment, &format!("OTEL_{name}_EXPORTER"))?.unwrap_or("otlp");
        if !matches!(exporter, "otlp" | "none") {
            return Err(invalid("unsupported telemetry signal exporter"));
        }
        let enabled = !disabled && exporter != "none";
        let protocol = text(environment, &format!("OTEL_EXPORTER_OTLP_{name}_PROTOCOL"))?
            .unwrap_or(base_protocol);
        if enabled && protocol != "http/protobuf" {
            return Err(invalid(
                "selected telemetry transport requires http/protobuf",
            ));
        }
        let endpoint = match text(environment, &format!("OTEL_EXPORTER_OTLP_{name}_ENDPOINT"))? {
            Some(value) => Some(value.to_owned()),
            None if !base.is_empty() => Some(format!(
                "{}/v1/{}",
                base.trim_end_matches('/'),
                name.to_lowercase()
            )),
            None => None,
        }
        .filter(|value| enabled && !value.is_empty());
        let headers = if enabled {
            attributes(
                text(environment, &format!("OTEL_EXPORTER_OTLP_{name}_HEADERS"))?
                    .unwrap_or(base_headers),
            )?
        } else {
            HashMap::new()
        };
        let timeout = timeout(
            environment,
            &format!("OTEL_EXPORTER_OTLP_{name}_TIMEOUT"),
            base_timeout,
        )?;
        Ok(SignalSettings {
            enabled: enabled && endpoint.is_some(),
            endpoint,
            headers,
            timeout_ms: ((timeout * 1000.0) as u64).max(1),
        })
    };
    let incarnation = uuid::Uuid::now_v7().to_string();
    let mut resource: BTreeMap<_, _> =
        attributes(text(environment, "OTEL_RESOURCE_ATTRIBUTES")?.unwrap_or(""))?
            .into_iter()
            .collect();
    resource.extend([
        (
            "service.name".into(),
            text(environment, "OTEL_SERVICE_NAME")?
                .unwrap_or(service)
                .into(),
        ),
        ("service.version".into(), env!("CARGO_PKG_VERSION").into()),
        ("service.instance.id".into(), incarnation.clone()),
        ("process.pid".into(), std::process::id().to_string()),
    ]);
    let sampler = match text(environment, "OTEL_TRACES_SAMPLER")? {
        Some(value) => serde_json::from_value(serde_json::Value::String(value.into()))
            .map_err(|_| invalid("unsupported telemetry sampler"))?,
        None => TelemetrySampler::ParentBasedAlwaysOn,
    };
    let settings = Settings {
        schema_version: SETTINGS_VERSION,
        disabled,
        traces: signal("TRACES")?,
        logs: signal("LOGS")?,
        metrics: signal("METRICS")?,
        resource,
        process_incarnation: incarnation,
        max_queue_size: usize::try_from(integer(environment, "OTEL_BSP_MAX_QUEUE_SIZE", 2048)?)
            .map_err(|_| invalid("telemetry queue exceeds platform range"))?,
        max_export_batch_size: usize::try_from(integer(
            environment,
            "OTEL_BSP_MAX_EXPORT_BATCH_SIZE",
            512,
        )?)
        .map_err(|_| invalid("telemetry batch exceeds platform range"))?,
        schedule_delay_ms: integer(environment, "OTEL_BSP_SCHEDULE_DELAY", 5000)?,
        export_timeout_ms: integer(environment, "OTEL_BSP_EXPORT_TIMEOUT", 5000)?,
        metric_export_interval_ms: integer(environment, "OTEL_METRIC_EXPORT_INTERVAL", 60_000)?,
        sampler,
        sampler_argument: number(environment, "OTEL_TRACES_SAMPLER_ARG", 1.0)?,
    };
    // Encoding applies the owner's scalar, HTTP, identity and total-document bounds once.
    settings.to_json()?;
    Ok(settings)
}

/// Optional observation refusal never rejects the computation or retains stale private settings.
pub fn configure(environment: &mut Environment) {
    environment.remove(OsStr::new(SETTINGS_ENV));
    let selection = resolve("pse-native-host", environment).and_then(|settings| {
        if settings.disabled {
            Ok(None)
        } else {
            settings.to_json().map(Some)
        }
    });
    let status = match selection {
        Ok(Some(document)) => {
            environment.insert(SETTINGS_ENV.into(), document.into());
            "configured"
        }
        Ok(None) => "disabled",
        Err(_) => "initialization_refused",
    };
    environment.insert(STATUS_ENV.into(), status.into());
}

#[cfg(test)]
mod tests {
    use super::*;
    fn selected_environment(root: &std::path::Path, values: &[(&str, &str)]) -> Environment {
        let mut environment = Environment::from([("HOME".into(), root.as_os_str().to_owned())]);
        environment.extend(
            values
                .iter()
                .map(|(key, value)| ((*key).into(), (*value).into())),
        );
        environment
    }
    #[test]
    fn telemetry_signal_overrides_base_and_explicit_empty_disables_one_signal() {
        let root = tempfile::tempdir().unwrap();
        let environment = selected_environment(
            root.path(),
            &[
                (
                    "OTEL_EXPORTER_OTLP_ENDPOINT",
                    "https://collector.test/tenant",
                ),
                (
                    "OTEL_EXPORTER_OTLP_TRACES_ENDPOINT",
                    "https://traces.test/private",
                ),
                ("OTEL_EXPORTER_OTLP_LOGS_ENDPOINT", ""),
                ("OTEL_EXPORTER_OTLP_HEADERS", "authorization=base%3Dsecret"),
                (
                    "OTEL_EXPORTER_OTLP_TRACES_HEADERS",
                    "authorization=trace+secret",
                ),
                ("OTEL_EXPORTER_OTLP_TRACES_TIMEOUT", "0.00001"),
            ],
        );
        let settings = resolve("host", &environment).unwrap();
        assert_eq!(
            settings.traces.endpoint.as_deref(),
            Some("https://traces.test/private")
        );
        assert_eq!(
            settings.metrics.endpoint.as_deref(),
            Some("https://collector.test/tenant/v1/metrics")
        );
        assert!(!settings.logs.enabled);
        assert_eq!(settings.traces.headers["authorization"], "trace+secret");
        assert_eq!(settings.metrics.headers["authorization"], "base=secret");
        assert_eq!(settings.traces.timeout_ms, 1);
        assert!(!format!("{settings:?}").contains("secret"));
    }
    #[test]
    fn telemetry_native_mapping_carries_only_the_validated_shared_contract() {
        let root = tempfile::tempdir().unwrap();
        let mut environment = selected_environment(
            root.path(),
            &[
                (
                    "OTEL_EXPORTER_OTLP_TRACES_ENDPOINT",
                    "https://collector.test/private",
                ),
                (
                    "OTEL_EXPORTER_OTLP_TRACES_HEADERS",
                    "authorization=private-token",
                ),
            ],
        );
        configure(&mut environment);
        assert_eq!(
            environment.get(OsStr::new(STATUS_ENV)),
            Some(&"configured".into())
        );
        let document = environment
            .get(OsStr::new(SETTINGS_ENV))
            .unwrap()
            .to_str()
            .unwrap();
        let settings = Settings::from_json(document).unwrap();
        assert_eq!(settings.traces.headers["authorization"], "private-token");
        assert_eq!(
            settings.resource["service.instance.id"],
            settings.process_incarnation
        );
        assert!(!settings.logs.enabled && !settings.metrics.enabled);
        environment.insert("OTEL_SDK_DISABLED".into(), "true".into());
        configure(&mut environment);
        assert_eq!(
            environment.get(OsStr::new(STATUS_ENV)),
            Some(&"disabled".into())
        );
        assert!(!environment.contains_key(OsStr::new(SETTINGS_ENV)));
    }
    #[test]
    fn telemetry_local_state_is_fallback_and_empty_base_overrides_it() {
        let root = tempfile::tempdir().unwrap();
        let state = root.path().join(".local/state/pse/telemetry");
        std::fs::create_dir_all(&state).unwrap();
        std::fs::write(state.join("configuration.json"), r#"{"schema_version":1,"configuration_id":"local","otlp_http_endpoint":"http://localhost:4318"}"#).unwrap();
        let mut environment = selected_environment(root.path(), &[]);
        assert!(resolve("host", &environment).unwrap().traces.enabled);
        environment.insert("OTEL_EXPORTER_OTLP_ENDPOINT".into(), "".into());
        assert!(resolve("host", &environment).unwrap().disabled);
        assert_eq!(std::fs::read_dir(state).unwrap().count(), 1);
    }
    #[test]
    fn telemetry_disable_and_exporter_none_keep_transport_unselected() {
        let root = tempfile::tempdir().unwrap();
        let environment = selected_environment(
            root.path(),
            &[
                ("OTEL_EXPORTER_OTLP_ENDPOINT", "http://localhost:4318"),
                ("OTEL_SDK_DISABLED", "TRUE"),
                ("OTEL_EXPORTER_OTLP_PROTOCOL", "unsupported"),
                ("OTEL_EXPORTER_OTLP_HEADERS", "bad\nheader"),
            ],
        );
        let settings = resolve("host", &environment).unwrap();
        assert!(settings.disabled && !settings.traces.enabled);
        let environment = selected_environment(
            root.path(),
            &[
                ("OTEL_EXPORTER_OTLP_ENDPOINT", "http://localhost:4318"),
                ("OTEL_TRACES_EXPORTER", "none"),
            ],
        );
        let settings = resolve("host", &environment).unwrap();
        assert!(!settings.traces.enabled && settings.logs.enabled);
    }
    #[test]
    fn telemetry_optional_invalid_configuration_refuses_without_stale_carrier_or_secret_error() {
        let root = tempfile::tempdir().unwrap();
        for values in [
            vec![("OTEL_EXPORTER_OTLP_ENDPOINT", "https://secret@example.test")],
            vec![
                ("OTEL_EXPORTER_OTLP_ENDPOINT", "http://localhost:4318"),
                ("OTEL_EXPORTER_OTLP_HEADERS", "authorization=secret%0d%0a"),
            ],
            vec![
                ("OTEL_EXPORTER_OTLP_ENDPOINT", "http://localhost:4318"),
                ("OTEL_BSP_MAX_QUEUE_SIZE", "0"),
            ],
            vec![("OTEL_EXPORTER_OTLP_TIMEOUT", "NaN")],
            vec![("OTEL_TRACES_SAMPLER", "unknown")],
        ] {
            let mut environment = selected_environment(root.path(), &values);
            environment.insert(SETTINGS_ENV.into(), "stale-secret".into());
            assert!(
                !resolve("host", &environment)
                    .unwrap_err()
                    .to_string()
                    .contains("secret")
            );
            configure(&mut environment);
            assert_eq!(
                environment.get(OsStr::new(STATUS_ENV)),
                Some(&"initialization_refused".into())
            );
            assert!(!environment.contains_key(OsStr::new(SETTINGS_ENV)));
        }
    }
    #[test]
    fn telemetry_queue_sampler_and_configuration_bounds_are_shared() {
        let root = tempfile::tempdir().unwrap();
        for values in [
            vec![
                ("OTEL_BSP_MAX_QUEUE_SIZE", "1"),
                ("OTEL_BSP_MAX_EXPORT_BATCH_SIZE", "2"),
            ],
            vec![("OTEL_TRACES_SAMPLER_ARG", "1.01")],
            vec![("OTEL_METRIC_EXPORT_INTERVAL", "3600001")],
            vec![("OTEL_EXPORTER_OTLP_TIMEOUT", "60.001")],
        ] {
            assert!(resolve("host", &selected_environment(root.path(), &values)).is_err());
        }
        let state = root.path().join(".local/state/pse/telemetry");
        std::fs::create_dir_all(&state).unwrap();
        std::fs::write(
            state.join("configuration.json"),
            vec![b' '; SETTINGS_MAX_BYTES + 1],
        )
        .unwrap();
        assert!(resolve("host", &selected_environment(root.path(), &[])).is_err());
    }
}
