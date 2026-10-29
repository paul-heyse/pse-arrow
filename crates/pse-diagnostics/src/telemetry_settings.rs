// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! SDK-independent resolved process telemetry contract and scalar admission.
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

/// A configuration/initialization failure never changes a scientific outcome.
#[derive(Debug, thiserror::Error, miette::Diagnostic)]
#[error("telemetry initialization refused: {0}")]
#[diagnostic(code(config::invalid))]
pub struct TelemetryError(pub String);

/// Private resolved telemetry wire version.
pub const SETTINGS_VERSION: u32 = 1;
/// Private document size bound for encoding and decoding.
pub const SETTINGS_MAX_BYTES: usize = 65_536;
/// Maximum resolved headers per signal.
pub const HEADERS_MAX_COUNT: usize = 32;
/// Maximum combined header-name and value byte length per signal.
pub const HEADERS_MAX_BYTES: usize = 8192;
/// Fixed executable roots receive their private process selection in this carrier.
pub const SETTINGS_ENV: &str = "PSE_NATIVE_TELEMETRY_SETTINGS";

// Scalar bounds are shared by runtime admission and the generated bootstrap schema.
/// Minimum positive queue or duration value.
pub const TELEMETRY_MINIMUM: u64 = 1;
/// Maximum signal and batch timeout in milliseconds.
pub const TELEMETRY_TIMEOUT_MAX_MS: u64 = 60_000;
/// Maximum batch queue and export batch capacity.
pub const TELEMETRY_QUEUE_MAX: usize = 65_536;
/// Maximum metric export interval in milliseconds.
pub const TELEMETRY_METRIC_INTERVAL_MAX_MS: u64 = 3_600_000;
/// Minimum trace sampling ratio.
pub const TELEMETRY_RATIO_MIN: f64 = 0.0;
/// Maximum trace sampling ratio.
pub const TELEMETRY_RATIO_MAX: f64 = 1.0;

/// Standard sampler selection shared with embedding process hosts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TelemetrySampler {
    /// Sample every root span.
    AlwaysOn,
    /// Sample no root span.
    AlwaysOff,
    /// Sample roots by their trace ID and configured ratio.
    #[serde(rename = "traceidratio")]
    TraceIdRatio,
    /// Honor the parent, sampling every root span.
    #[serde(rename = "parentbased_always_on")]
    ParentBasedAlwaysOn,
    /// Honor the parent, sampling no root span.
    #[serde(rename = "parentbased_always_off")]
    ParentBasedAlwaysOff,
    /// Honor the parent, sampling roots by trace ID and ratio.
    #[serde(rename = "parentbased_traceidratio")]
    ParentBasedTraceIdRatio,
}
/// Resolved signal settings; environment precedence belongs to the host resolver.
#[derive(Clone, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SignalSettings {
    /// Whether the host selected this signal.
    pub enabled: bool,
    /// Fully resolved signal URL; private and never rendered.
    pub endpoint: Option<String>,
    /// Resolved authentication/transport headers; never rendered.
    pub headers: HashMap<String, String>,
    /// Actual transport request timeout in milliseconds.
    #[schemars(range(min = TELEMETRY_MINIMUM, max = TELEMETRY_TIMEOUT_MAX_MS))]
    pub timeout_ms: u64,
}
/// Shared native process-host settings. Credentials are never Debug; serialization is private launcher transport.
#[derive(Clone, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    /// Shared host settings document version.
    #[schemars(extend("const" = SETTINGS_VERSION))]
    pub schema_version: u32,
    /// Process-wide telemetry disable selection.
    pub disabled: bool,
    /// Resolved trace selection.
    pub traces: SignalSettings,
    /// Resolved log selection.
    pub logs: SignalSettings,
    /// Resolved metric selection.
    pub metrics: SignalSettings,
    /// Shared process/service attributes; SDK attributes are Rust-owned.
    pub resource: BTreeMap<String, String>,
    /// Incarnation selected by the launcher and replaced by the actual receiving process.
    pub process_incarnation: String,
    /// SDK finite batch queue capacity.
    #[schemars(range(min = TELEMETRY_MINIMUM, max = TELEMETRY_QUEUE_MAX))]
    pub max_queue_size: usize,
    /// SDK maximum items per export batch.
    #[schemars(range(min = TELEMETRY_MINIMUM, max = TELEMETRY_QUEUE_MAX))]
    pub max_export_batch_size: usize,
    /// SDK periodic batch delay.
    #[schemars(range(min = TELEMETRY_MINIMUM, max = TELEMETRY_TIMEOUT_MAX_MS))]
    pub schedule_delay_ms: u64,
    /// Host-requested SDK shutdown wait; transport deadlines remain independent.
    #[schemars(range(min = TELEMETRY_MINIMUM, max = TELEMETRY_TIMEOUT_MAX_MS))]
    pub export_timeout_ms: u64,
    /// Dedicated metric reader interval.
    #[schemars(range(min = TELEMETRY_MINIMUM, max = TELEMETRY_METRIC_INTERVAL_MAX_MS))]
    pub metric_export_interval_ms: u64,
    /// Standard sampler name selected by the host resolver.
    pub sampler: TelemetrySampler,
    /// Ratio argument for ratio-based samplers.
    #[schemars(range(min = TELEMETRY_RATIO_MIN, max = TELEMETRY_RATIO_MAX))]
    pub sampler_argument: f64,
}
impl std::fmt::Debug for SignalSettings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SignalSettings")
            .field("enabled", &self.enabled)
            .field("timeout_ms", &self.timeout_ms)
            .finish_non_exhaustive()
    }
}
impl std::fmt::Debug for Settings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Settings")
            .field("disabled", &self.disabled)
            .field("traces", &self.traces)
            .field("logs", &self.logs)
            .field("metrics", &self.metrics)
            .finish_non_exhaustive()
    }
}
impl Settings {
    /// Parse and validate the bounded resolved host document without rendering secrets.
    pub fn from_json(json: &str) -> Result<Self, TelemetryError> {
        if json.len() > SETTINGS_MAX_BYTES {
            return Err(TelemetryError("settings exceed 64 KiB".into()));
        }
        let settings: Self = serde_json::from_str(json)
            .map_err(|_| TelemetryError("invalid resolved settings document".into()))?;
        settings.validate()?;
        Ok(settings)
    }
    /// Encode the validated private contract, retaining the same finite wire bound.
    pub fn to_json(&self) -> Result<String, TelemetryError> {
        self.validate()?;
        let json = serde_json::to_string(self)
            .map_err(|_| TelemetryError("private settings encoding failed".into()))?;
        if json.len() > SETTINGS_MAX_BYTES {
            return Err(TelemetryError("settings exceed 64 KiB".into()));
        }
        Ok(json)
    }
    /// Validate an already typed selection without installing SDK state.
    pub fn validate(&self) -> Result<(), TelemetryError> {
        if self.schema_version != SETTINGS_VERSION
            || self.process_incarnation.is_empty()
            || self.resource.get("service.instance.id") != Some(&self.process_incarnation)
            || (self.max_queue_size < TELEMETRY_MINIMUM as usize
                || self.max_queue_size > TELEMETRY_QUEUE_MAX)
            || self.max_export_batch_size < TELEMETRY_MINIMUM as usize
            || self.max_export_batch_size > TELEMETRY_QUEUE_MAX
            || self.max_export_batch_size > self.max_queue_size
            || (self.schedule_delay_ms < TELEMETRY_MINIMUM
                || self.schedule_delay_ms > TELEMETRY_TIMEOUT_MAX_MS)
            || (self.export_timeout_ms < TELEMETRY_MINIMUM
                || self.export_timeout_ms > TELEMETRY_TIMEOUT_MAX_MS)
            || (self.metric_export_interval_ms < TELEMETRY_MINIMUM
                || self.metric_export_interval_ms > TELEMETRY_METRIC_INTERVAL_MAX_MS)
            || !self.sampler_argument.is_finite()
            || !(TELEMETRY_RATIO_MIN..=TELEMETRY_RATIO_MAX).contains(&self.sampler_argument)
        {
            return Err(TelemetryError(
                "invalid shared identity, schema or batch bounds".into(),
            ));
        }
        for signal in [&self.traces, &self.logs, &self.metrics] {
            if signal.timeout_ms < TELEMETRY_MINIMUM || signal.timeout_ms > TELEMETRY_TIMEOUT_MAX_MS
            {
                return Err(TelemetryError("signal timeout outside host bounds".into()));
            }
            if signal.headers.len() > HEADERS_MAX_COUNT
                || signal
                    .headers
                    .iter()
                    .map(|(key, value)| key.len() + value.len())
                    .sum::<usize>()
                    > HEADERS_MAX_BYTES
                || signal.headers.iter().any(|(key, value)| {
                    http::header::HeaderName::from_bytes(key.as_bytes()).is_err()
                        || http::header::HeaderValue::from_str(value).is_err()
                })
            {
                return Err(TelemetryError("invalid bounded signal headers".into()));
            }
            if signal.enabled && !self.disabled && signal.endpoint.is_none() {
                return Err(TelemetryError("enabled signal lacks endpoint".into()));
            }
            if let Some(endpoint) = &signal.endpoint {
                let url = url::Url::parse(endpoint)
                    .map_err(|_| TelemetryError("invalid signal endpoint".into()))?;
                if !matches!(url.scheme(), "http" | "https")
                    || url.host_str().is_none()
                    || !url.username().is_empty()
                    || url.password().is_some()
                    || url.query().is_some()
                    || url.fragment().is_some()
                {
                    return Err(TelemetryError("unsupported signal transport".into()));
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn settings() -> Settings {
        let signal = SignalSettings {
            enabled: false,
            endpoint: None,
            headers: HashMap::new(),
            timeout_ms: 5_000,
        };
        Settings {
            schema_version: SETTINGS_VERSION,
            disabled: false,
            traces: signal.clone(),
            logs: signal.clone(),
            metrics: signal,
            resource: BTreeMap::from([("service.instance.id".into(), "incarnation".into())]),
            process_incarnation: "incarnation".into(),
            max_queue_size: 2048,
            max_export_batch_size: 512,
            schedule_delay_ms: 5_000,
            export_timeout_ms: 5_000,
            metric_export_interval_ms: 60_000,
            sampler: TelemetrySampler::ParentBasedAlwaysOn,
            sampler_argument: 1.0,
        }
    }
    #[test]
    fn telemetry_contract_roundtrip_and_scalar_bounds_are_sdk_independent() {
        let mut settings = settings();
        let json = settings.to_json().unwrap();
        assert_eq!(Settings::from_json(&json).unwrap().max_queue_size, 2048);
        settings.max_queue_size = 0;
        assert!(settings.validate().is_err());
        settings.max_queue_size = TELEMETRY_QUEUE_MAX;
        settings.max_export_batch_size = TELEMETRY_QUEUE_MAX + 1;
        assert!(settings.validate().is_err());
        settings.max_export_batch_size = 1;
        settings.sampler_argument = f64::NAN;
        assert!(settings.validate().is_err());
    }
    #[test]
    fn telemetry_contract_refuses_credentials_and_header_injection_without_logging_them() {
        let mut settings = settings();
        settings.traces.enabled = true;
        settings.traces.endpoint = Some("https://private-token@example.test/secret".into());
        assert!(settings.validate().is_err());
        settings.traces.endpoint = Some("https://example.test/secret-path".into());
        settings
            .traces
            .headers
            .insert("authorization".into(), "private-token\r\n".into());
        let error = settings.validate().unwrap_err();
        assert!(!format!("{error:?} {settings:?}").contains("private-token"));
        assert!(!format!("{settings:?}").contains("secret-path"));
    }
    #[test]
    fn telemetry_contract_both_encoding_and_decoding_enforce_document_bound() {
        let mut settings = settings();
        settings
            .resource
            .insert("large".into(), "x".repeat(SETTINGS_MAX_BYTES));
        assert!(settings.to_json().is_err());
        assert!(Settings::from_json(&" ".repeat(SETTINGS_MAX_BYTES + 1)).is_err());
    }
}
