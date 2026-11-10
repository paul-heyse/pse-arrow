// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit native-server export selection and bounded, incarnation-qualified observations.

use super::NativeServiceError;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, num::NonZeroU64};

/// A selected local Collector gRPC destination. Secrets are persisted only in private startup state.
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeTelemetry {
    endpoint: String,
    headers: BTreeMap<String, String>,
    interval_ms: NonZeroU64,
}
impl std::fmt::Debug for NativeTelemetry {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("NativeTelemetry")
            .field("endpoint", &self.endpoint)
            .field("header_count", &self.headers.len())
            .field("interval_ms", &self.interval_ms)
            .finish()
    }
}
impl NativeTelemetry {
    /// Select an existing loopback Collector's gRPC listener, not its HTTP exporter address.
    /// # Errors
    /// Nonlocal destinations, malformed headers or a zero export interval are refused.
    pub fn collector(
        endpoint: String,
        headers: BTreeMap<String, String>,
        interval_ms: u64,
    ) -> Result<Self, NativeServiceError> {
        let value = Self {
            endpoint,
            headers,
            interval_ms: NonZeroU64::new(interval_ms).ok_or(NativeServiceError::Refused(
                "telemetry interval must be positive",
            ))?,
        };
        value.validate()?;
        Ok(value)
    }
    pub(super) fn validate(&self) -> Result<(), NativeServiceError> {
        let endpoint = url::Url::parse(&self.endpoint)
            .map_err(|_| NativeServiceError::Refused("invalid Collector endpoint"))?;
        let local = match endpoint.host() {
            Some(url::Host::Ipv4(address)) => address.is_loopback(),
            Some(url::Host::Ipv6(address)) => address.is_loopback(),
            _ => false,
        };
        if endpoint.scheme() != "http"
            || !local
            || endpoint.port().is_none()
            || !matches!(endpoint.path(), "" | "/")
            || endpoint.query().is_some()
            || endpoint.fragment().is_some()
            || !endpoint.username().is_empty()
            || endpoint.password().is_some()
            || self.headers.len() > 16
        {
            return Err(NativeServiceError::Refused(
                "native telemetry requires an explicit local Collector gRPC endpoint",
            ));
        }
        for (name, value) in &self.headers {
            if http::header::HeaderName::from_bytes(name.as_bytes()).is_err()
                || name.to_ascii_lowercase() != *name
                || value.is_empty()
                || value.len() > 4096
                || !value.is_ascii()
                || value.contains([',', '\r', '\n'])
                || http::header::HeaderValue::from_str(value).is_err()
            {
                return Err(NativeServiceError::Refused(
                    "invalid native telemetry exporter header",
                ));
            }
        }
        Ok(())
    }
    pub(super) fn environment(
        &self,
        invocation: &str,
    ) -> Result<Vec<(&'static str, String)>, NativeServiceError> {
        self.validate()?;
        if invocation.len() != 32 || !invocation.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(NativeServiceError::Refused(
                "native telemetry requires the original service invocation",
            ));
        }
        Ok(vec![
            ("SURREAL_TELEMETRY_PROVIDER", "otlp".into()),
            ("OTEL_EXPORTER_OTLP_ENDPOINT", self.endpoint.clone()),
            (
                "OTEL_EXPORTER_OTLP_HEADERS",
                self.headers
                    .iter()
                    .map(|(name, value)| {
                        format!("{name}={}", value.replace('%', "%25").replace(' ', "%20"))
                    })
                    .collect::<Vec<_>>()
                    .join(","),
            ),
            (
                "OTEL_RESOURCE_ATTRIBUTES",
                format!("service.instance.id={invocation}"),
            ),
            ("OTEL_METRIC_EXPORT_INTERVAL", self.interval_ms.to_string()),
        ])
    }
}

/// Native MX samples with explicit evidence origin, never estimates of filesystem release.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct NativePersistenceObservation {
    /// Evidence origin. Supplied records alone do not establish authenticated delivery.
    pub source: NativeMetricSource,
    /// Selected original server invocation; supplied-record projection checks its label only.
    pub service_invocation: String,
    /// Unix milliseconds at which the snapshot was read; not scientific time.
    pub observed_at_ms: u64,
    /// Supplied native gauges only. Absence remains unknown.
    pub gauges: BTreeMap<String, u64>,
    /// The selected integrated MX adapter does not retain queryable VERSION history.
    pub recorded_versions_available: bool,
    /// No integrated operator snapshot-completion API is exposed by this backend.
    pub manual_snapshot_available: bool,
}
/// How a native metric sample reached this projection.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum NativeMetricSource {
    /// Parsed caller-supplied record; resource labels are checked, not producer custody.
    SuppliedCollectorRecord,
    /// Bounded authenticated response from the original physically owned server.
    OriginalServer,
}
impl NativePersistenceObservation {
    /// Decode a bounded Collector file-exporter OTLP JSON record for an exact original server.
    /// # Errors
    /// Oversized, malformed, foreign-incarnation or contradictory samples are refused.
    pub fn from_collector_json(
        bytes: &[u8],
        invocation: &str,
        observed_at_ms: u64,
    ) -> Result<Self, NativeServiceError> {
        if bytes.len() > 4 * 1024 * 1024
            || invocation.len() != 32
            || !invocation.bytes().all(|byte| byte.is_ascii_hexdigit())
        {
            return Err(NativeServiceError::Refused(
                "invalid bounded native telemetry snapshot",
            ));
        }
        let record: serde_json::Value = serde_json::from_slice(bytes)
            .map_err(|_| NativeServiceError::Refused("malformed Collector metric record"))?;
        let resources = record
            .get("resourceMetrics")
            .and_then(serde_json::Value::as_array)
            .ok_or(NativeServiceError::Refused(
                "Collector metrics record is absent",
            ))?;
        let mut gauges = BTreeMap::new();
        for resource in resources {
            let matching = resource
                .pointer("/resource/attributes")
                .and_then(serde_json::Value::as_array)
                .is_some_and(|attributes| {
                    attributes.iter().any(|attribute| {
                        attribute.get("key").and_then(serde_json::Value::as_str)
                            == Some("service.instance.id")
                            && attribute
                                .pointer("/value/stringValue")
                                .and_then(serde_json::Value::as_str)
                                == Some(invocation)
                    })
                });
            if !matching {
                continue;
            }
            let scopes = resource
                .get("scopeMetrics")
                .and_then(serde_json::Value::as_array)
                .ok_or(NativeServiceError::Refused(
                    "Collector scope metrics are absent",
                ))?;
            for scope in scopes {
                let Some(metrics) = scope.get("metrics").and_then(serde_json::Value::as_array)
                else {
                    continue;
                };
                for metric in metrics {
                    let Some(name) = metric
                        .get("name")
                        .and_then(serde_json::Value::as_str)
                        .and_then(|name| name.strip_prefix("surrealdb.storage."))
                        .filter(|name| MX_GAUGES.contains(name))
                    else {
                        continue;
                    };
                    let gauge = metric
                        .get("gauge")
                        .and_then(serde_json::Value::as_object)
                        .ok_or(NativeServiceError::Refused(
                            "native MX metric is not a gauge",
                        ))?;
                    let Some(points) = gauge.get("dataPoints") else {
                        continue;
                    };
                    let points = points.as_array().ok_or(NativeServiceError::Refused(
                        "native MX gauge points are malformed",
                    ))?;
                    // An asynchronous gauge may omit an observation (notably
                    // oldest_pinned_version when no reader exists). Keep it unknown.
                    if points.is_empty() {
                        continue;
                    }
                    if points.len() != 1 {
                        return Err(NativeServiceError::Refused(
                            "native MX gauge has ambiguous scope",
                        ));
                    }
                    let value = points[0]
                        .get("asInt")
                        .and_then(|value| {
                            value
                                .as_u64()
                                .or_else(|| value.as_str().and_then(|value| value.parse().ok()))
                        })
                        .ok_or(NativeServiceError::Refused(
                            "native MX gauge is not an unsigned count",
                        ))?;
                    if gauges.insert(name.into(), value).is_some() {
                        return Err(NativeServiceError::Refused("native MX gauge is duplicated"));
                    }
                }
            }
        }
        if gauges.is_empty() {
            return Err(NativeServiceError::Refused(
                "original server has no observed native MX metrics",
            ));
        }
        Ok(Self {
            source: NativeMetricSource::SuppliedCollectorRecord,
            service_invocation: invocation.into(),
            observed_at_ms,
            gauges,
            recorded_versions_available: false,
            manual_snapshot_available: false,
        })
    }
    pub(super) fn from_prometheus(
        bytes: &[u8],
        invocation: String,
        observed_at_ms: u64,
    ) -> Result<Self, NativeServiceError> {
        let text = std::str::from_utf8(bytes)
            .map_err(|_| NativeServiceError::Refused("native metrics are not UTF-8"))?;
        let mut gauges = BTreeMap::new();
        for line in text.lines().filter(|line| !line.starts_with('#')) {
            for &native in MX_GAUGES {
                let rendered = format!("surrealdb_storage_{}", native.replace('.', "_"));
                let Some(suffix) = line.strip_prefix(&rendered) else {
                    continue;
                };
                let suffix = if suffix.starts_with('{') {
                    suffix.rsplit_once('}').map(|(_, suffix)| suffix).ok_or(
                        NativeServiceError::Refused("malformed native metric labels"),
                    )?
                } else {
                    suffix
                };
                if !suffix.starts_with(char::is_whitespace) {
                    continue;
                }
                let value = suffix
                    .split_ascii_whitespace()
                    .next()
                    .and_then(|value| value.parse::<u64>().ok())
                    .ok_or(NativeServiceError::Refused(
                        "native MX gauge count is inexact",
                    ))?;
                if gauges.insert(native.into(), value).is_some() {
                    return Err(NativeServiceError::Refused(
                        "native MX gauge has ambiguous scope",
                    ));
                }
            }
        }
        if gauges.is_empty() {
            return Err(NativeServiceError::Refused(
                "native MX metrics are unavailable or denied",
            ));
        }
        Ok(Self {
            source: NativeMetricSource::OriginalServer,
            service_invocation: invocation,
            observed_at_ms,
            gauges,
            recorded_versions_available: false,
            manual_snapshot_available: false,
        })
    }
}
const MX_GAUGES: &[&str] = &[
    "surrealmx.live_keys",
    "surrealmx.live_versions",
    "surrealmx.approx_data_bytes",
    "surrealmx.active_readers",
    "surrealmx.oldest_pinned_version",
    "surrealmx.gc_candidate_keys",
    "surrealmx.commit_ring_occupancy",
    "surrealmx.async_queue_depth",
    "surrealmx.async_queue_bytes",
    "surrealmx.async_dropped_batches",
    "surrealmx.poisoned",
];

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    reason = "native telemetry contract tests assert exact checked inputs"
)]
mod tests {
    use super::*;
    #[test]
    fn native_telemetry_is_explicit_local_and_secret_redacted() {
        let telemetry = NativeTelemetry::collector(
            "http://127.0.0.1:4317".into(),
            BTreeMap::from([("authorization".into(), "Bearer private-token".into())]),
            1000,
        )
        .unwrap();
        assert!(!format!("{telemetry:?}").contains("private-token"));
        let environment = telemetry.environment(&"a".repeat(32)).unwrap();
        assert!(environment.contains(&("SURREAL_TELEMETRY_PROVIDER", "otlp".into())));
        assert!(environment.contains(&(
            "OTEL_RESOURCE_ATTRIBUTES",
            format!("service.instance.id={}", "a".repeat(32))
        )));
        assert!(environment.contains(&(
            "OTEL_EXPORTER_OTLP_HEADERS",
            "authorization=Bearer%20private-token".into()
        )));
        let escaped = NativeTelemetry::collector(
            "http://[::1]:4317".into(),
            BTreeMap::from([("authorization".into(), "literal%20value".into())]),
            1000,
        )
        .unwrap()
        .environment(&"a".repeat(32))
        .unwrap();
        assert!(escaped.contains(&(
            "OTEL_EXPORTER_OTLP_HEADERS",
            "authorization=literal%2520value".into()
        )));
        assert!(telemetry.environment("unowned").is_err());
        for endpoint in [
            "http://example.com:4317",
            "http://user:secret@127.0.0.1:4317",
            "http://127.0.0.1:4317/v1/metrics",
            "http://127.0.0.1",
        ] {
            assert!(NativeTelemetry::collector(endpoint.into(), BTreeMap::new(), 1000).is_err());
        }
    }
    #[test]
    fn native_persistence_observation_requires_original_incarnation_and_gauge() {
        let invocation = "a".repeat(32);
        let value = serde_json::json!({"resourceMetrics":[{"resource":{"attributes":[{"key":"service.instance.id","value":{"stringValue":invocation}}]},"scopeMetrics":[{"metrics":[{"name":"surrealdb.storage.surrealmx.live_versions","gauge":{"dataPoints":[{"asInt":"17"}]}}]}]}]});
        let bytes = serde_json::to_vec(&value).unwrap();
        let observed =
            NativePersistenceObservation::from_collector_json(&bytes, &invocation, 123).unwrap();
        assert_eq!(observed.gauges["surrealmx.live_versions"], 17);
        assert!(!observed.recorded_versions_available);
        assert!(!observed.manual_snapshot_available);
        assert!(
            NativePersistenceObservation::from_collector_json(&bytes, &"b".repeat(32), 123)
                .is_err()
        );
        assert!(
            NativePersistenceObservation::from_collector_json(b"{}", &invocation, 123).is_err()
        );
        let mut value = value;
        value["resourceMetrics"][0]["scopeMetrics"][0]["metrics"].as_array_mut().unwrap().push(serde_json::json!({"name":"surrealdb.storage.surrealmx.oldest_pinned_version","gauge":{"dataPoints":[]}}));
        let observed = NativePersistenceObservation::from_collector_json(
            &serde_json::to_vec(&value).unwrap(),
            &invocation,
            123,
        )
        .unwrap();
        assert!(
            !observed
                .gauges
                .contains_key("surrealmx.oldest_pinned_version")
        );
    }
    #[test]
    fn native_persistence_prometheus_preserves_exact_counts_and_refuses_ambiguity() {
        let observed=NativePersistenceObservation::from_prometheus(b"# TYPE surrealdb_storage_surrealmx_live_versions gauge\nsurrealdb_storage_surrealmx_live_versions 18446744073709551615\n", "a".repeat(32),123).unwrap();
        assert_eq!(observed.gauges["surrealmx.live_versions"], u64::MAX);
        assert_eq!(observed.source, NativeMetricSource::OriginalServer);
        for invalid in [b"surrealdb_storage_surrealmx_live_versions 17.5\n".as_slice(),b"surrealdb_storage_surrealmx_live_versions 17\nsurrealdb_storage_surrealmx_live_versions 17\n",b"surrealdb_process_cpu_percent 0\n"] {
            assert!(NativePersistenceObservation::from_prometheus(invalid,"a".repeat(32),123).is_err());
        }
    }
}
