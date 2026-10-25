// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit process-host telemetry composition. No Runtime installs SDK state.
//!
//! Context is diagnostic only: it never participates in native admission or hashes.
use super::metric_declarations::*;
use opentelemetry::{
    Context, KeyValue,
    propagation::TextMapPropagator,
    trace::{TraceContextExt, TracerProvider},
};
use opentelemetry_otlp::{WithExportConfig, WithHttpConfig};
use opentelemetry_sdk::{
    Resource, logs::SdkLoggerProvider, metrics::SdkMeterProvider,
    propagation::TraceContextPropagator, trace::SdkTracerProvider,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashMap},
    sync::OnceLock,
    time::Duration,
};
use tracing::instrument::WithSubscriber;
use tracing_opentelemetry::OpenTelemetrySpanExt;
use tracing_subscriber::{Layer, layer::SubscriberExt};

/// Instrumentation scope shared by native spans and the cheap host activation read.
pub const INSTRUMENTATION_SCOPE: &str = "pse-runtime";

fn exported_owner_span(metadata: &tracing::Metadata<'_>) -> bool {
    metadata.is_span()
        && (metadata.target().starts_with("pse")
            || metadata.target() == "surrealdb::engine::remote::grpc_lifecycle")
}

/// Actual compiled SDK metadata, supplied by the SDK's own detector without
/// importing endpoint/resource environment interpretation or duplicating pins.
pub fn sdk_metadata() -> BTreeMap<String, String> {
    let resource = Resource::builder_empty()
        .with_detector(Box::new(
            opentelemetry_sdk::resource::TelemetryResourceDetector,
        ))
        .build();
    resource
        .iter()
        .map(|(key, value)| (key.as_str().to_owned(), value.to_string()))
        .collect()
}

/// A configuration/initialization failure never changes a scientific outcome.
#[derive(Debug, thiserror::Error, miette::Diagnostic)]
#[error("telemetry initialization refused: {0}")]
#[diagnostic(code(config::invalid))]
pub struct TelemetryError(pub String);

/// Private resolved telemetry wire version.
pub const SETTINGS_VERSION: u32 = 1;

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
    pub timeout_ms: u64,
}
/// Shared Python/Rust host settings. Credentials are never Debug; serialization is private launcher transport.
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
    /// Incarnation shared with the embedding Python host in this actual process.
    pub process_incarnation: String,
    /// SDK finite batch queue capacity.
    pub max_queue_size: usize,
    /// SDK maximum items per export batch.
    pub max_export_batch_size: usize,
    /// SDK periodic batch delay.
    pub schedule_delay_ms: u64,
    /// Host-requested SDK shutdown wait; transport deadlines remain independent.
    pub export_timeout_ms: u64,
    /// Dedicated metric reader interval.
    pub metric_export_interval_ms: u64,
    /// Standard sampler name selected by the host resolver.
    pub sampler: String,
    /// Ratio argument for ratio-based samplers.
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
        if json.len() > 65536 {
            return Err(TelemetryError("settings exceed 64 KiB".into()));
        }
        let settings: Self = serde_json::from_str(json)
            .map_err(|_| TelemetryError("invalid resolved settings document".into()))?;
        settings.validate()?;
        Ok(settings)
    }
    fn validate(&self) -> Result<(), TelemetryError> {
        if self.schema_version != SETTINGS_VERSION
            || self.process_incarnation.is_empty()
            || self.resource.get("service.instance.id") != Some(&self.process_incarnation)
            || (self.max_queue_size == 0 || self.max_queue_size > 65536)
            || self.max_export_batch_size == 0
            || self.max_export_batch_size > self.max_queue_size
            || (self.schedule_delay_ms == 0 || self.schedule_delay_ms > 60000)
            || (self.export_timeout_ms == 0 || self.export_timeout_ms > 60000)
            || (self.metric_export_interval_ms == 0 || self.metric_export_interval_ms > 3600000)
            || !self.sampler_argument.is_finite()
            || !(0.0..=1.0).contains(&self.sampler_argument)
        {
            return Err(TelemetryError(
                "invalid shared identity, schema or batch bounds".into(),
            ));
        }
        for signal in [&self.traces, &self.logs, &self.metrics] {
            if signal.timeout_ms == 0 || signal.timeout_ms > 60000 {
                return Err(TelemetryError("signal timeout outside host bounds".into()));
            }
            if signal.headers.len() > 32
                || signal
                    .headers
                    .iter()
                    .map(|(key, value)| key.len() + value.len())
                    .sum::<usize>()
                    > 8192
                || signal.headers.iter().any(|(key, value)| {
                    reqwest::header::HeaderName::from_bytes(key.as_bytes()).is_err()
                        || reqwest::header::HeaderValue::from_str(value).is_err()
                })
            {
                return Err(TelemetryError("invalid bounded signal headers".into()));
            }
            if signal.enabled && !self.disabled && signal.endpoint.is_none() {
                return Err(TelemetryError("enabled signal lacks endpoint".into()));
            }
            if let Some(endpoint) = &signal.endpoint {
                let url = reqwest::Url::parse(endpoint)
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
        let _ = self.sampler()?;
        Ok(())
    }
    fn sampler(&self) -> Result<opentelemetry_sdk::trace::Sampler, TelemetryError> {
        use opentelemetry_sdk::trace::Sampler;
        let ratio = || {
            if self.sampler_argument.is_finite() && (0.0..=1.0).contains(&self.sampler_argument) {
                Ok(Sampler::TraceIdRatioBased(self.sampler_argument))
            } else {
                Err(TelemetryError("invalid sampler ratio".into()))
            }
        };
        match self.sampler.as_str() {
            "always_on" => Ok(Sampler::AlwaysOn),
            "always_off" => Ok(Sampler::AlwaysOff),
            "traceidratio" => ratio(),
            "parentbased_always_on" => Ok(Sampler::ParentBased(Box::new(Sampler::AlwaysOn))),
            "parentbased_always_off" => Ok(Sampler::ParentBased(Box::new(Sampler::AlwaysOff))),
            "parentbased_traceidratio" => Ok(Sampler::ParentBased(Box::new(ratio()?))),
            _ => Err(TelemetryError("unsupported sampler".into())),
        }
    }
}

/// Only the two W3C trace fields may cross process/language boundaries. No baggage.
#[derive(Clone, Default, Debug, Serialize)]
#[serde(transparent)]
pub struct Carrier(BTreeMap<String, String>);
impl Carrier {
    /// Validate and copy the allowlisted bounded W3C fields.
    pub fn from_fields(fields: BTreeMap<String, String>) -> Result<Self, TelemetryError> {
        if fields.len() > 2
            || fields.iter().any(|(key, value)| {
                !matches!(key.as_str(), "traceparent" | "tracestate")
                    || value.len() > 512
                    || !value.is_ascii()
                    || value.bytes().any(|b| b.is_ascii_control())
            })
        {
            return Err(TelemetryError("invalid bounded W3C carrier".into()));
        }
        if let Some(state) = fields.get("tracestate")
            && (state.is_empty() || state.parse::<opentelemetry::trace::TraceState>().is_err())
        {
            return Err(TelemetryError("invalid W3C tracestate".into()));
        }
        let carrier = Self(fields);
        if !carrier.0.is_empty()
            && (!carrier.0.contains_key("traceparent")
                || !carrier.context().span().span_context().is_valid())
        {
            return Err(TelemetryError("invalid W3C parent".into()));
        }
        Ok(carrier)
    }
    /// Decode a private launcher carrier with a finite document bound.
    pub fn from_json(json: &str) -> Result<Self, TelemetryError> {
        if json.len() > 1200 {
            return Err(TelemetryError("carrier exceeds bound".into()));
        }
        Self::from_fields(
            serde_json::from_str(json)
                .map_err(|_| TelemetryError("invalid carrier document".into()))?,
        )
    }
    /// Encode the separate private launcher carrier.
    pub fn to_json(&self) -> Result<String, TelemetryError> {
        serde_json::to_string(self).map_err(|_| TelemetryError("carrier encoding".into()))
    }
    /// Capture the authentic current tracing/OpenTelemetry context.
    pub fn current() -> Self {
        let context = tracing::Span::current().context();
        let context = if context.span().span_context().is_valid() {
            context
        } else {
            Context::current()
        };
        let mut fields = HashMap::new();
        TraceContextPropagator::new().inject_context(&context, &mut fields);
        // The Rust propagator emits an empty tracestate; omit that optional field
        // so our strict receiving boundary accepts the SDK-produced carrier.
        fields.retain(|_, value| !value.is_empty());
        Self(fields.into_iter().collect())
    }
    /// Extract with an empty base, preventing inherited context leakage.
    pub fn context(&self) -> Context {
        let fields: HashMap<_, _> = self.0.clone().into_iter().collect();
        // Explicit empty base prevents accidental inheritance from a previous entry.
        TraceContextPropagator::new().extract_with_context(&Context::new(), &fields)
    }
    /// An independent shared producer links its initiator instead of claiming parentage.
    pub fn linked_span(&self, owner: &'static str) -> tracing::Span {
        let span = tracing::info_span!(parent: None, "pse.native.shared_producer", owner);
        let context = self.context();
        if context.span().span_context().is_valid() {
            span.add_link(context.span().span_context().clone());
        }
        span
    }
    /// Set the explicit parent before entry; report unavailable attachment on the span.
    pub fn operation_span(&self, owner: &'static str) -> tracing::Span {
        let span = tracing::info_span!(parent: None, "pse.native.operation", owner, telemetry.parent = tracing::field::Empty);
        let context = self.context();
        let valid = context.span().span_context().is_valid();
        match span.set_parent(context) {
            Ok(()) => {
                span.record(
                    "telemetry.parent",
                    if valid { "attached" } else { "missing" },
                );
            }
            Err(_) => {
                span.record("telemetry.parent", "unavailable");
            }
        }
        span
    }
}
/// A shared producer has independent lifetime and links to its initiating request.
/// Further waiters retain their native SharedProduction relationship rather than
/// becoming invented causal parents.
pub(crate) struct ProducerContext {
    carrier: Carrier,
    dispatch: tracing::Dispatch,
}
impl ProducerContext {
    pub(crate) fn capture() -> Self {
        Self {
            carrier: Carrier::current(),
            dispatch: tracing::dispatcher::get_default(Clone::clone),
        }
    }
    pub(crate) fn run<F: Future>(
        self,
        owner: &'static str,
        future: F,
    ) -> impl Future<Output = F::Output> {
        let span =
            tracing::dispatcher::with_default(&self.dispatch, || self.carrier.linked_span(owner));
        tracing::Instrument::instrument(future, span).with_subscriber(self.dispatch)
    }
}

/// Poll-scoped propagation across owned tasks. No attached guard crosses await.
pub(crate) fn inherit<F: Future>(future: F) -> impl Future<Output = F::Output> {
    tracing::Instrument::instrument(future, tracing::Span::current())
        .with_subscriber(tracing::dispatcher::get_default(Clone::clone))
}

/// Emit a synchronous lifecycle fact under its authentic span context. An explicit
/// tracing event parent alone does not activate the log bridge's OTel context.
pub fn lifecycle(span: &tracing::Span, code: &'static str, owner: &'static str) {
    span.in_scope(|| tracing::info!(target: "pse.telemetry.diagnostic", code, owner, "native lifecycle owner event"));
}

/// Project source-owned diagnostic codes without rendering private scientific inputs.
/// Cancellation and nonconvergence keep their native classes; no eager span Ok status.
pub(crate) fn diagnostic(error: &dyn pse_diagnostics::TypedDiagnostic, owner: &'static str) {
    let operation = pse_diagnostics::observation::current().map(|context| context.id());
    for leaf in pse_diagnostics::diagnostic_leaves(error)
        .into_iter()
        .take(32)
    {
        if let Some(code) = leaf.diagnostic_code() {
            if let Some(operation) = operation {
                tracing::warn!(target: "pse.telemetry.diagnostic", code = %code, owner, pse.operation.id = %operation, "native owner returned a typed diagnostic");
            } else {
                tracing::warn!(target: "pse.telemetry.diagnostic", code = %code, owner, "native owner returned a typed diagnostic");
            }
        }
    }
}

/// Child-launch context is a separate diagnostic environment field, never request source.
pub const CHILD_CARRIER_ENV: &str = "PSE_NATIVE_TRACE_CONTEXT";
/// Resolved host settings passed only by selected fixed executable launchers.
pub const SETTINGS_ENV: &str = "PSE_NATIVE_TELEMETRY_SETTINGS";

/// Process-host handles. Shutdown only after native owners settle, outside async execution.
static ACTIVE_SETTINGS: OnceLock<Settings> = OnceLock::new();

/// Copy the resolved host selection for a fixed child launcher. The child replaces
/// only actual process identity, never scientific or invocation authority.
pub fn child_settings_json() -> Result<Option<String>, TelemetryError> {
    ACTIVE_SETTINGS
        .get()
        .map(|settings| {
            serde_json::to_string(settings)
                .map_err(|_| TelemetryError("child settings encoding".into()))
        })
        .transpose()
}

/// Owned process SDK handles and their exact composed subscriber.
pub struct Host {
    settings: Settings,
    traces: Option<SdkTracerProvider>,
    logs: Option<SdkLoggerProvider>,
    metrics: Option<SdkMeterProvider>,
    dispatch: tracing::Dispatch,
    capture: bool,
}
impl std::fmt::Debug for Host {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TelemetryHost")
            .field("traces", &self.traces.is_some())
            .field("logs", &self.logs.is_some())
            .field("metrics", &self.metrics.is_some())
            .finish_non_exhaustive()
    }
}
struct ResolvedClient {
    client: reqwest::blocking::Client,
    headers: http::HeaderMap,
}
impl std::fmt::Debug for ResolvedClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ResolvedBlockingOtlpClient")
    }
}
#[async_trait::async_trait]
impl opentelemetry_http::HttpClient for ResolvedClient {
    async fn send_bytes(
        &self,
        mut request: http::Request<bytes::Bytes>,
    ) -> Result<http::Response<bytes::Bytes>, opentelemetry_http::HttpError> {
        // SDK0.33.1 overlays environment headers even after with_headers. Keep only
        // protocol-owned fields, then apply the single resolved host selection.
        let protocol: http::HeaderMap = request
            .headers()
            .iter()
            .filter(|(key, _)| matches!(key.as_str(), "content-type" | "content-encoding"))
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect();
        *request.headers_mut() = protocol;
        request.headers_mut().extend(self.headers.clone());
        opentelemetry_http::HttpClient::send_bytes(&self.client, request).await
    }
}
fn client(signal: &SignalSettings) -> Result<ResolvedClient, TelemetryError> {
    let mut headers = http::HeaderMap::new();
    for (key, value) in &signal.headers {
        let key = http::header::HeaderName::from_bytes(key.as_bytes())
            .map_err(|_| TelemetryError("invalid resolved header name".into()))?;
        let mut value = http::HeaderValue::from_str(value)
            .map_err(|_| TelemetryError("invalid resolved header value".into()))?;
        value.set_sensitive(true);
        headers.insert(key, value);
    }
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_millis(signal.timeout_ms))
        .build()
        .map_err(|_| TelemetryError("blocking HTTP client unavailable".into()))?;
    Ok(ResolvedClient { client, headers })
}

/// SDK callbacks read the same native sources as passive checkpoints. The SDK owns
/// collection scheduling; no second sampler, executor or resource owner is installed.
pub(super) fn install_runtime_metrics(provider: &SdkMeterProvider) {
    super::runtime_metrics::initialize_source_identity();
    use opentelemetry::metrics::MeterProvider as _;
    let meter = provider.meter_with_scope(
        opentelemetry::InstrumentationScope::builder(INSTRUMENTATION_SCOPE)
            .with_version(env!("CARGO_PKG_VERSION"))
            .build(),
    );
    for (definition, field) in [
        (PSE_RUNTIME_EXECUTOR_WORKERS, 0),
        (PSE_RUNTIME_EXECUTOR_ALIVE_TASKS, 1),
        (PSE_RUNTIME_EXECUTOR_GLOBAL_QUEUE_DEPTH, 2),
    ] {
        meter
            .u64_observable_gauge(definition.name)
            .with_description(definition.population)
            .with_unit(definition.unit)
            .with_callback(move |observer| {
                let snapshot = super::runtime_metrics::snapshot();
                if !snapshot.coverage.executor_registry_available {
                    return;
                }
                let mut roles = BTreeMap::<&'static str, u64>::new();
                for executor in snapshot.executors {
                    let value = match field {
                        0 => executor.workers,
                        1 => executor.alive_tasks,
                        _ => executor.global_queue_depth,
                    };
                    let total = roles.entry(executor.role.as_str()).or_default();
                    *total = total.saturating_add(value as u64);
                }
                for (role, value) in roles {
                    observer.observe(value, &[KeyValue::new("executor.role", role)]);
                }
            })
            .build();
    }
    // This is a gauge over the current executor population. Removing an executor
    // can decrease it; calling it a process-lifetime monotonic counter is incorrect.
    meter
        .f64_observable_gauge(PSE_RUNTIME_EXECUTOR_WORKER_BUSY.name)
        .with_description(PSE_RUNTIME_EXECUTOR_WORKER_BUSY.population)
        .with_unit(PSE_RUNTIME_EXECUTOR_WORKER_BUSY.unit)
        .with_callback(|observer| {
            let snapshot = super::runtime_metrics::snapshot();
            if !snapshot.coverage.executor_registry_available {
                return;
            }
            let mut roles = BTreeMap::<&'static str, u128>::new();
            for executor in snapshot.executors {
                if let Some(nanos) = executor.worker_busy_nanos {
                    let total = roles.entry(executor.role.as_str()).or_default();
                    *total = total.saturating_add(nanos);
                }
            }
            for (role, nanos) in roles {
                observer.observe(
                    nanos as f64 / 1_000_000_000.0,
                    &[KeyValue::new("executor.role", role)],
                );
            }
        })
        .build();
    meter
        .u64_observable_counter(PSE_NATIVE_SUPERVISOR_EVENTS.name)
        .with_description(PSE_NATIVE_SUPERVISOR_EVENTS.population)
        .with_unit(PSE_NATIVE_SUPERVISOR_EVENTS.unit)
        .with_callback(|observer| {
            let sample = super::operational_metrics::snapshot();
            if sample.source_started_unix_micros.is_none() || sample.saturated {
                return;
            }
            for (event, count) in NATIVE_EVENTS.into_iter().zip([
                sample.native_attempts,
                sample.native_admitted,
                sample.native_dispatched,
                sample.native_joined,
                sample.native_cancel_requests,
                sample.native_deadline_requests,
            ]) {
                observer.observe(count, &[KeyValue::new("event", event)]);
            }
        })
        .build();
    meter
        .u64_observable_gauge(PSE_NATIVE_SUPERVISORS.name)
        .with_description(PSE_NATIVE_SUPERVISORS.population)
        .with_unit(PSE_NATIVE_SUPERVISORS.unit)
        .with_callback(|observer| {
            observer.observe(
                super::operational_metrics::snapshot().native_supervisors,
                &[],
            );
        })
        .build();
    meter
        .u64_observable_counter(PSE_PREPARATION_FLIGHT_EVENTS.name)
        .with_description(PSE_PREPARATION_FLIGHT_EVENTS.population)
        .with_unit(PSE_PREPARATION_FLIGHT_EVENTS.unit)
        .with_callback(|observer| {
            let sample = super::operational_metrics::snapshot();
            if sample.source_started_unix_micros.is_none() || sample.saturated {
                return;
            }
            for stage in sample.preparation {
                for (event, count) in PREPARATION_EVENTS.into_iter().zip([
                    stage.acquisitions,
                    stage.retained_reuses,
                    stage.producers,
                    stage.flight_waiters,
                ]) {
                    observer.observe(
                        count,
                        &[
                            KeyValue::new("stage", stage.stage.clone()),
                            KeyValue::new("event", event),
                        ],
                    );
                }
            }
        })
        .build();
    meter
        .u64_observable_counter(PSE_PREPARATION_PRODUCTS.name)
        .with_description(PSE_PREPARATION_PRODUCTS.population)
        .with_unit(PSE_PREPARATION_PRODUCTS.unit)
        .with_callback(|observer| {
            if super::operational_metrics::snapshot()
                .source_started_unix_micros
                .is_none()
            {
                return;
            }
            let sample = crate::math::preparation_metrics();
            for (product, count) in PRODUCTS.into_iter().zip([
                sample.views,
                sample.observations,
                sample.rebuilt,
                sample.shared,
            ]) {
                observer.observe(count as u64, &[KeyValue::new("product", product)]);
            }
        })
        .build();
    meter
        .u64_observable_counter(PSE_CANONICAL_QUERY_EVENTS.name)
        .with_description(PSE_CANONICAL_QUERY_EVENTS.population)
        .with_unit(PSE_CANONICAL_QUERY_EVENTS.unit)
        .with_callback(|observer| {
            let sample = pse_operations::canonical::query_metrics();
            if sample.source_started_unix_micros.is_none() || sample.saturated {
                return;
            }
            for (event, count) in QUERY_EVENTS.into_iter().zip([
                sample.builders,
                sample.request_failures,
                sample.statements,
                sample.statements_succeeded,
                sample.statements_failed,
                sample.statements_timed,
                sample.statements_untimed,
            ]) {
                observer.observe(count, &[KeyValue::new("event", event)]);
            }
        })
        .build();
    meter
        .f64_observable_counter(PSE_CANONICAL_SERVER_DURATION.name)
        .with_description(PSE_CANONICAL_SERVER_DURATION.population)
        .with_unit(PSE_CANONICAL_SERVER_DURATION.unit)
        .with_callback(|observer| {
            let sample = pse_operations::canonical::query_metrics();
            if sample.source_started_unix_micros.is_some()
                && !sample.saturated
                && sample.statements_timed > 0
            {
                observer.observe(sample.server_duration_nanos as f64 / 1_000_000_000.0, &[]);
            }
        })
        .build();
    meter
        .f64_observable_gauge(PSE_OPERATIONAL_SOURCE_START.name)
        .with_description(PSE_OPERATIONAL_SOURCE_START.population)
        .with_unit(PSE_OPERATIONAL_SOURCE_START.unit)
        .with_callback(|observer| {
            for (owner, start) in [
                (
                    SOURCE_OWNERS[0],
                    super::operational_metrics::snapshot().source_started_unix_micros,
                ),
                (
                    SOURCE_OWNERS[1],
                    pse_operations::canonical::query_metrics().source_started_unix_micros,
                ),
            ] {
                if let Some(start) = start {
                    observer.observe(start as f64 / 1_000_000.0, &[KeyValue::new("owner", owner)]);
                }
            }
        })
        .build();
    meter
        .u64_observable_counter(PSE_RUNTIME_TASK_EVENTS.name)
        .with_description(PSE_RUNTIME_TASK_EVENTS.population)
        .with_unit(PSE_RUNTIME_TASK_EVENTS.unit)
        .with_callback(|observer| {
            for task in super::runtime_metrics::snapshot().task_classes {
                for (event, count) in TASK_EVENTS.into_iter().zip([
                    task.instrumented_count,
                    task.dropped_count,
                    task.poll_count,
                    task.scheduled_count,
                    task.idle_count,
                    task.first_poll_count,
                    task.slow_poll_count,
                    task.long_delay_count,
                ]) {
                    observer.observe(
                        count,
                        &[
                            KeyValue::new("task.class", task.class.as_str()),
                            KeyValue::new("event", event),
                        ],
                    );
                }
            }
        })
        .build();
    meter
        .f64_observable_counter(PSE_RUNTIME_TASK_DURATION.name)
        .with_description(PSE_RUNTIME_TASK_DURATION.population)
        .with_unit(PSE_RUNTIME_TASK_DURATION.unit)
        .with_callback(|observer| {
            for task in super::runtime_metrics::snapshot().task_classes {
                for (phase, nanos) in TASK_PHASES.into_iter().zip([
                    task.poll_nanos,
                    task.scheduled_nanos,
                    task.idle_nanos,
                    task.first_poll_delay_nanos,
                    task.slow_poll_nanos,
                    task.long_delay_nanos,
                ]) {
                    observer.observe(
                        nanos as f64 / 1_000_000_000.0,
                        &[
                            KeyValue::new("task.class", task.class.as_str()),
                            KeyValue::new("phase", phase),
                        ],
                    );
                }
            }
        })
        .build();
    meter
        .f64_observable_gauge(PSE_RUNTIME_TASK_THRESHOLD.name)
        .with_description(PSE_RUNTIME_TASK_THRESHOLD.population)
        .with_unit(PSE_RUNTIME_TASK_THRESHOLD.unit)
        .with_callback(|observer| {
            for task in super::runtime_metrics::snapshot().task_classes {
                for (threshold, nanos) in TASK_THRESHOLDS.into_iter().zip([
                    task.slow_poll_threshold_nanos,
                    task.long_delay_threshold_nanos,
                ]) {
                    observer.observe(
                        nanos as f64 / 1_000_000_000.0,
                        &[
                            KeyValue::new("task.class", task.class.as_str()),
                            KeyValue::new("threshold", threshold),
                        ],
                    );
                }
            }
        })
        .build();
    meter
        .f64_observable_gauge(PSE_RUNTIME_TASK_SOURCE_START.name)
        .with_description(PSE_RUNTIME_TASK_SOURCE_START.population)
        .with_unit(PSE_RUNTIME_TASK_SOURCE_START.unit)
        .with_callback(|observer| {
            for task in super::runtime_metrics::snapshot().task_classes {
                observer.observe(
                    task.source_started_unix_micros as f64 / 1_000_000.0,
                    &[KeyValue::new("task.class", task.class.as_str())],
                );
            }
        })
        .build();
    meter
        .u64_observable_gauge(PSE_RUNTIME_EXECUTOR_REGISTRY_AVAILABLE.name)
        .with_description(PSE_RUNTIME_EXECUTOR_REGISTRY_AVAILABLE.population)
        .with_unit(PSE_RUNTIME_EXECUTOR_REGISTRY_AVAILABLE.unit)
        .with_callback(|observer| {
            observer.observe(
                u64::from(
                    super::runtime_metrics::snapshot()
                        .coverage
                        .executor_registry_available,
                ),
                &[],
            )
        })
        .build();
    meter
        .u64_observable_counter(PSE_RUNTIME_EXECUTOR_REGISTRATION_REFUSED.name)
        .with_description(PSE_RUNTIME_EXECUTOR_REGISTRATION_REFUSED.population)
        .with_unit(PSE_RUNTIME_EXECUTOR_REGISTRATION_REFUSED.unit)
        .with_callback(|observer| {
            observer.observe(
                super::runtime_metrics::snapshot()
                    .coverage
                    .refused_executor_registrations,
                &[],
            )
        })
        .build();
}
impl Host {
    /// Construct on a synchronous host thread; no implicit global mutation.
    pub fn build(
        settings: &Settings,
        capture: Option<super::capture::CaptureLayer>,
    ) -> Result<Self, TelemetryError> {
        settings.validate()?;
        let has_capture = capture.is_some();
        let resource = Resource::builder_empty()
            .with_attributes(
                settings
                    .resource
                    .iter()
                    .map(|(k, v)| KeyValue::new(k.clone(), v.clone())),
            )
            .with_detector(Box::new(
                opentelemetry_sdk::resource::TelemetryResourceDetector,
            ))
            .with_attributes([KeyValue::new("process.pid", i64::from(std::process::id()))])
            .build();
        let traces = if settings.traces.enabled && !settings.disabled {
            let signal = &settings.traces;
            let exporter = opentelemetry_otlp::SpanExporter::builder()
                .with_http()
                .with_protocol(opentelemetry_otlp::Protocol::HttpBinary)
                .with_endpoint(
                    signal
                        .endpoint
                        .clone()
                        .ok_or_else(|| TelemetryError("missing trace endpoint".into()))?,
                )
                .with_timeout(Duration::from_millis(signal.timeout_ms))
                .with_headers(signal.headers.clone())
                .with_http_client(client(signal)?)
                .build()
                .map_err(|_| TelemetryError("OTLP exporter initialization unavailable".into()))?;
            let batch = opentelemetry_sdk::trace::BatchConfigBuilder::default()
                .with_max_queue_size(settings.max_queue_size)
                .with_max_export_batch_size(settings.max_export_batch_size)
                .with_scheduled_delay(Duration::from_millis(settings.schedule_delay_ms))
                .build();
            Some(
                SdkTracerProvider::builder()
                    .with_resource(resource.clone())
                    .with_sampler(settings.sampler()?)
                    .with_span_processor(
                        opentelemetry_sdk::trace::BatchSpanProcessor::builder(exporter)
                            .with_batch_config(batch)
                            .build(),
                    )
                    .build(),
            )
        } else {
            None
        };
        let logs = if settings.logs.enabled && !settings.disabled {
            let signal = &settings.logs;
            let exporter = opentelemetry_otlp::LogExporter::builder()
                .with_http()
                .with_protocol(opentelemetry_otlp::Protocol::HttpBinary)
                .with_endpoint(
                    signal
                        .endpoint
                        .clone()
                        .ok_or_else(|| TelemetryError("missing log endpoint".into()))?,
                )
                .with_timeout(Duration::from_millis(signal.timeout_ms))
                .with_headers(signal.headers.clone())
                .with_http_client(client(signal)?)
                .build()
                .map_err(|_| TelemetryError("OTLP exporter initialization unavailable".into()))?;
            let batch = opentelemetry_sdk::logs::BatchConfigBuilder::default()
                .with_max_queue_size(settings.max_queue_size)
                .with_max_export_batch_size(settings.max_export_batch_size)
                .with_scheduled_delay(Duration::from_millis(settings.schedule_delay_ms))
                .build();
            Some(
                SdkLoggerProvider::builder()
                    .with_resource(resource.clone())
                    .with_log_processor(
                        opentelemetry_sdk::logs::BatchLogProcessor::builder(exporter)
                            .with_batch_config(batch)
                            .build(),
                    )
                    .build(),
            )
        } else {
            None
        };
        let metrics = if settings.metrics.enabled && !settings.disabled {
            let signal = &settings.metrics;
            let exporter = opentelemetry_otlp::MetricExporter::builder()
                .with_temporality(opentelemetry_sdk::metrics::Temporality::Cumulative)
                .with_http()
                .with_protocol(opentelemetry_otlp::Protocol::HttpBinary)
                .with_endpoint(
                    signal
                        .endpoint
                        .clone()
                        .ok_or_else(|| TelemetryError("missing metric endpoint".into()))?,
                )
                .with_timeout(Duration::from_millis(signal.timeout_ms))
                .with_headers(signal.headers.clone())
                .with_http_client(client(signal)?)
                .build()
                .map_err(|_| TelemetryError("OTLP exporter initialization unavailable".into()))?;
            let reader = opentelemetry_sdk::metrics::PeriodicReader::builder(exporter)
                .with_interval(Duration::from_millis(settings.metric_export_interval_ms))
                .build();
            Some(
                SdkMeterProvider::builder()
                    .with_resource(resource)
                    .with_reader(reader)
                    .build(),
            )
        } else {
            None
        };
        if let Some(provider) = &metrics {
            install_runtime_metrics(provider);
        }
        // Export only explicit native diagnostics as logs; other tracing events remain
        // span events. In particular SDK/HTTP exporter internals cannot recurse.
        let bridge = logs.as_ref().map(|provider| {
            opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge::new(provider)
                .with_filter(tracing_subscriber::filter::filter_fn(|metadata| {
                    metadata.target() == "pse.telemetry.diagnostic"
                }))
        });
        // Ordinary traces expose native owners and the patched SDK's request
        // owner, which bridges dispatch/body lifetime to the server. Existing tracing events can
        // contain rendered scientific errors; their finite CaptureLayer consumer
        // retains its separate contract. Typed diagnostics use the log bridge.
        let layer = traces.as_ref().map(|provider| {
            tracing_opentelemetry::layer()
                .with_tracer(
                    provider.tracer_with_scope(
                        opentelemetry::InstrumentationScope::builder(INSTRUMENTATION_SCOPE)
                            .with_version(env!("CARGO_PKG_VERSION"))
                            .build(),
                    ),
                )
                .with_filter(tracing_subscriber::filter::filter_fn(exported_owner_span))
        });
        let stderr = tracing_subscriber::fmt::layer()
            .with_writer(std::io::stderr)
            .with_ansi(false)
            .with_filter(tracing_subscriber::filter::filter_fn(|metadata| {
                metadata.target() == "pse.telemetry.diagnostic"
            }));
        let dispatch = tracing::Dispatch::new(
            tracing_subscriber::registry()
                .with(capture)
                .with(layer)
                .with(bridge)
                .with(stderr),
        );
        Ok(Self {
            settings: settings.clone(),
            traces,
            logs,
            metrics,
            dispatch,
            capture: has_capture,
        })
    }
    /// Explicit composition root only. A preexisting arbitrary Dispatch cannot gain layers.
    pub fn install_global(&self) -> Result<(), TelemetryError> {
        if self.traces.is_none() && self.logs.is_none() && self.metrics.is_none() && !self.capture {
            let _ = ACTIVE_SETTINGS.set(self.settings.clone());
            return Ok(());
        }
        tracing::dispatcher::set_global_default(self.dispatch.clone()).map_err(|_| TelemetryError("preexisting subscriber: export composition unavailable; existing capture preserved".into()))?;
        let _ = ACTIVE_SETTINGS.set(self.settings.clone());
        if let Some(provider) = &self.metrics {
            opentelemetry::global::set_meter_provider(provider.clone());
        }
        Ok(())
    }
    /// A host with caller-scoped capture can compose and enter this exact Dispatch.
    pub fn dispatch(&self) -> tracing::Dispatch {
        self.dispatch.clone()
    }
    /// Shut down all selected SDKs after native owners settle on a synchronous host.
    pub fn shutdown(mut self) -> Result<(), TelemetryError> {
        self.shutdown_inner()
    }
    fn shutdown_inner(&mut self) -> Result<(), TelemetryError> {
        let mut failures = Vec::new();
        if let Some(provider) = self.traces.take()
            && let Err(error) = provider
                .shutdown_with_timeout(Duration::from_millis(self.settings.export_timeout_ms))
        {
            failures.push(error.to_string());
        }
        if let Some(provider) = self.logs.take()
            && let Err(error) = provider
                .shutdown_with_timeout(Duration::from_millis(self.settings.export_timeout_ms))
        {
            failures.push(error.to_string());
        }
        if let Some(provider) = self.metrics.take()
            && let Err(error) = provider
                .shutdown_with_timeout(Duration::from_millis(self.settings.export_timeout_ms))
        {
            failures.push(error.to_string());
        }
        if failures.is_empty() {
            Ok(())
        } else {
            Err(TelemetryError(failures.join("; ")))
        }
    }
    /// Fixed executable roots consume already-resolved settings passed by their launcher.
    pub fn from_environment(service: &str) -> Result<Option<Self>, TelemetryError> {
        let Some(json) = std::env::var_os(SETTINGS_ENV) else {
            return Ok(None);
        };
        let mut settings = Settings::from_json(&json.to_string_lossy())?;
        if settings.disabled
            || !(settings.traces.enabled || settings.logs.enabled || settings.metrics.enabled)
        {
            return Ok(None);
        }
        settings.process_incarnation = pse_operations::mint_id::<pse_ids::SemanticId>().to_string();
        settings.resource.insert(
            "service.instance.id".into(),
            settings.process_incarnation.clone(),
        );
        settings
            .resource
            .insert("service.name".into(), service.into());
        settings
            .resource
            .insert("process.pid".into(), std::process::id().to_string());
        let host = Self::build(&settings, None)?;
        if let Err(error) = host.install_global() {
            let _ = host.shutdown();
            return Err(error);
        }
        Ok(Some(host))
    }
}

/// Initialize the private short-lived child host with process-lifetime ownership.
/// Its final batches may be lost at process exit: no network drain is permitted
/// between native settlement and parent-owned physical process reaping. Long-lived
/// hosts continue to own and shut down their ordinary Host explicitly.
pub fn initialize_private_child_host() -> Result<(), &'static TelemetryError> {
    static HOST: OnceLock<Result<Option<Host>, TelemetryError>> = OnceLock::new();
    HOST.get_or_init(|| Host::from_environment("pse-native-lane"))
        .as_ref()
        .map(|_| ())
}

impl Drop for Host {
    fn drop(&mut self) {
        if self.shutdown_inner().is_err() {
            tracing::warn!(target: "pse.telemetry.diagnostic", code = "telemetry.shutdown.failed", "native telemetry shutdown failed after owner settlement");
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "bounded W3C fixture assertions")]
mod tests {
    use super::*;
    #[test]
    fn bounded_carrier_validates_and_round_trips_independent_parent() {
        let parent = "00-0123456789abcdef0123456789abcdef-0123456789abcdef-01";
        let carrier =
            Carrier::from_fields(BTreeMap::from([("traceparent".into(), parent.into())])).unwrap();
        let context = carrier.context();
        assert_eq!(
            context.span().span_context().trace_id().to_string(),
            "0123456789abcdef0123456789abcdef"
        );
        assert_eq!(
            context.span().span_context().span_id().to_string(),
            "0123456789abcdef"
        );
        assert!(context.span().span_context().is_remote());
        assert_eq!(
            Carrier::from_json(&carrier.to_json().unwrap()).unwrap().0,
            carrier.0
        );
    }
    #[test]
    fn bounded_carrier_rejects_invalid_unknown_and_oversized_fields() {
        for fields in [
            BTreeMap::from([("traceparent".into(), "invalid".into())]),
            BTreeMap::from([("baggage".into(), "secret".into())]),
            BTreeMap::from([("tracestate".into(), "x".repeat(513))]),
        ] {
            assert!(Carrier::from_fields(fields).is_err());
        }
        assert!(
            !Carrier::default()
                .context()
                .span()
                .span_context()
                .is_valid()
        );
    }
    #[derive(Clone, Debug, Default)]
    struct Recorded(std::sync::Arc<std::sync::Mutex<Vec<opentelemetry_sdk::trace::SpanData>>>);
    impl opentelemetry_sdk::trace::SpanExporter for Recorded {
        async fn export(
            &self,
            batch: Vec<opentelemetry_sdk::trace::SpanData>,
        ) -> opentelemetry_sdk::error::OTelSdkResult {
            self.0.lock().unwrap().extend(batch);
            Ok(())
        }
    }
    #[test]
    fn production_span_filter_retains_exact_sdk_parent_without_events_or_other_library_spans() {
        let recorded = Recorded::default();
        let provider = SdkTracerProvider::builder()
            .with_simple_exporter(recorded.clone())
            .build();
        let subscriber = tracing_subscriber::registry().with(
            tracing_opentelemetry::layer()
                .with_tracer(provider.tracer("fixture"))
                .with_filter(tracing_subscriber::filter::filter_fn(exported_owner_span)),
        );
        let dispatch = tracing::Dispatch::new(subscriber);
        let carrier = Carrier::from_fields(BTreeMap::from([(
            "traceparent".into(),
            "00-8123456789abcdef0123456789abcdef-9123456789abcdef-01".into(),
        )]))
        .unwrap();
        let native_id = tracing::dispatcher::with_default(&dispatch, || {
            let native = carrier.operation_span("fixture.sdk");
            let native_id = native.context().span().span_context().span_id();
            native.in_scope(|| {
                let sdk = tracing::info_span!(
                    target: "surrealdb::engine::remote::grpc_lifecycle",
                    "surrealdb.sdk.request"
                );
                assert!(!sdk.is_disabled());
                sdk.in_scope(|| {
                    tracing::warn!(target: "surrealdb::engine::remote::grpc_lifecycle", private = "must-not-export", "private request event");
                    let unrelated = tracing::info_span!(target: "surrealdb::engine::remote::grpc", "unrelated.sdk.span");
                    assert!(unrelated.is_disabled());
                });
            });
            native_id
        });
        provider.force_flush().unwrap();
        let records = recorded.0.lock().unwrap();
        assert_eq!(records.len(), 2);
        let native = records
            .iter()
            .find(|span| span.name == "pse.native.operation")
            .unwrap();
        let sdk = records
            .iter()
            .find(|span| span.name == "surrealdb.sdk.request")
            .unwrap();
        assert_eq!(native.parent_span_id.to_string(), "9123456789abcdef");
        assert_eq!(sdk.parent_span_id, native_id);
        assert_eq!(
            sdk.span_context.trace_id().to_string(),
            "8123456789abcdef0123456789abcdef"
        );
        assert!(records.iter().all(|span| span.events.events.is_empty()));
        drop(records);
        provider.shutdown().unwrap();
    }
    #[test]
    fn native_carrier_retains_parent_across_actual_task_and_thread_hops() {
        let recorded = Recorded::default();
        let provider = SdkTracerProvider::builder()
            .with_simple_exporter(recorded.clone())
            .build();
        let subscriber = tracing_subscriber::registry()
            .with(tracing_opentelemetry::layer().with_tracer(provider.tracer("fixture")));
        let dispatch = tracing::Dispatch::new(subscriber);
        let carrier = Carrier::from_fields(BTreeMap::from([(
            "traceparent".into(),
            "00-1123456789abcdef0123456789abcdef-2123456789abcdef-01".into(),
        )]))
        .unwrap();
        let span =
            tracing::dispatcher::with_default(&dispatch, || carrier.operation_span("fixture"));
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .unwrap();
        let future = tracing::Instrument::instrument(
            async {
                tokio::spawn(inherit(async {
                    tokio::task::yield_now().await;
                    let child = tracing::info_span!("pse.fixture.task");
                    let parent = child.context().span().span_context().clone();
                    let dispatch = tracing::dispatcher::get_default(Clone::clone);
                    std::thread::spawn(move || {
                        tracing::dispatcher::with_default(&dispatch, || {
                            child.in_scope(|| {
                                let carrier = Carrier::current();
                                Carrier::from_json(&carrier.to_json().unwrap())
                                    .unwrap()
                                    .context()
                                    .span()
                                    .span_context()
                                    .clone()
                            })
                        })
                    })
                    .join()
                    .map(|actual| (actual, parent))
                    .unwrap()
                }))
                .await
                .unwrap()
            },
            span,
        )
        .with_subscriber(dispatch);
        let (actual, expected) = runtime.block_on(future);
        assert_eq!(actual.trace_id(), expected.trace_id());
        assert_eq!(actual.span_id(), expected.span_id());
        assert_eq!(actual.trace_flags(), expected.trace_flags());
        assert!(actual.is_remote());
        assert_eq!(
            actual.trace_id().to_string(),
            "1123456789abcdef0123456789abcdef"
        );
        provider.force_flush().unwrap();
        let records = recorded.0.lock().unwrap();
        let root = records
            .iter()
            .find(|span| span.name == "pse.native.operation")
            .unwrap();
        assert_eq!(root.parent_span_id.to_string(), "2123456789abcdef");
        assert!(root.parent_span_is_remote);
        let task = records
            .iter()
            .find(|span| span.name == "pse.fixture.task")
            .unwrap();
        assert_eq!(task.parent_span_id, root.span_context.span_id());
        drop(records);
        provider.shutdown().unwrap();
    }
    #[test]
    fn independent_producer_links_initiator_without_claiming_causal_parent() {
        let recorded = Recorded::default();
        let provider = SdkTracerProvider::builder()
            .with_simple_exporter(recorded.clone())
            .build();
        let dispatch = tracing::Dispatch::new(
            tracing_subscriber::registry()
                .with(tracing_opentelemetry::layer().with_tracer(provider.tracer("fixture"))),
        );
        let carrier = Carrier::from_fields(BTreeMap::from([(
            "traceparent".into(),
            "00-3123456789abcdef0123456789abcdef-4123456789abcdef-01".into(),
        )]))
        .unwrap();
        let producer =
            tracing::dispatcher::with_default(&dispatch, || carrier.linked_span("fixture.shared"));
        producer.in_scope(|| {});
        drop(producer);
        provider.force_flush().unwrap();
        let records = recorded.0.lock().unwrap();
        let root = records
            .iter()
            .find(|span| span.name == "pse.native.shared_producer")
            .unwrap();
        assert_eq!(root.parent_span_id, opentelemetry::trace::SpanId::INVALID);
        assert_eq!(root.links.links.len(), 1);
        assert_eq!(
            root.links.links[0].span_context.span_id().to_string(),
            "4123456789abcdef"
        );
        drop(records);
        provider.shutdown().unwrap();
    }

    #[test]
    fn shared_producer_context_survives_ended_initiator() {
        let recorded = Recorded::default();
        let provider = SdkTracerProvider::builder()
            .with_simple_exporter(recorded.clone())
            .build();
        let dispatch = tracing::Dispatch::new(
            tracing_subscriber::registry()
                .with(tracing_opentelemetry::layer().with_tracer(provider.tracer("fixture"))),
        );
        let remote = Carrier::from_fields(BTreeMap::from([(
            "traceparent".into(),
            "00-5123456789abcdef0123456789abcdef-6123456789abcdef-01".into(),
        )]))
        .unwrap();
        let initiator = tracing::dispatcher::with_default(&dispatch, || {
            remote.operation_span("fixture.initiator")
        });
        let context = tracing::dispatcher::with_default(&dispatch, || {
            initiator.in_scope(ProducerContext::capture)
        });
        let initiator_id = initiator.context().span().span_context().span_id();
        drop(initiator);
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        runtime.block_on(context.run("fixture.continued", async {
            tokio::task::yield_now().await;
            assert!(
                Carrier::current()
                    .context()
                    .span()
                    .span_context()
                    .is_valid()
            );
        }));
        provider.force_flush().unwrap();
        let records = recorded.0.lock().unwrap();
        let producer = records
            .iter()
            .find(|span| span.name == "pse.native.shared_producer")
            .unwrap();
        assert_eq!(records[0].name, "pse.native.operation");
        assert_eq!(
            producer.parent_span_id,
            opentelemetry::trace::SpanId::INVALID
        );
        assert_eq!(producer.links.links[0].span_context.span_id(), initiator_id);
        drop(records);
        provider.shutdown().unwrap();
    }

    #[derive(Clone, Debug, Default)]
    struct RecordedLogs(
        std::sync::Arc<std::sync::Mutex<Vec<opentelemetry_sdk::logs::TraceContext>>>,
    );
    impl opentelemetry_sdk::logs::LogExporter for RecordedLogs {
        async fn export(
            &self,
            batch: opentelemetry_sdk::logs::LogBatch<'_>,
        ) -> opentelemetry_sdk::error::OTelSdkResult {
            self.0.lock().unwrap().extend(
                batch
                    .iter()
                    .filter_map(|(record, _)| record.trace_context().cloned()),
            );
            Ok(())
        }
    }
    #[test]
    fn waiting_thread_lifecycle_logs_use_authentic_native_span_context() {
        let records = RecordedLogs::default();
        let traces = SdkTracerProvider::builder()
            .with_simple_exporter(Recorded::default())
            .build();
        let logs = SdkLoggerProvider::builder()
            .with_simple_exporter(records.clone())
            .build();
        let dispatch = tracing::Dispatch::new(
            tracing_subscriber::registry()
                .with(tracing_opentelemetry::layer().with_tracer(traces.tracer("fixture")))
                .with(
                    opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge::new(&logs),
                ),
        );
        let carrier = Carrier::from_fields(BTreeMap::from([(
            "traceparent".into(),
            "00-7123456789abcdef0123456789abcdef-8123456789abcdef-01".into(),
        )]))
        .unwrap();
        let unrelated = Carrier::from_fields(BTreeMap::from([(
            "traceparent".into(),
            "00-9123456789abcdef0123456789abcdef-a123456789abcdef-01".into(),
        )]))
        .unwrap();
        let span = tracing::dispatcher::with_default(&dispatch, || {
            carrier.operation_span("fixture.native")
        });
        let native_id = span.context().span().span_context().span_id();
        tracing::dispatcher::with_default(&dispatch, || {
            let unrelated = unrelated.operation_span("fixture.waiting_thread");
            unrelated.in_scope(|| {
                lifecycle(&span, "native.cancel.requested", "python.native_pull");
                lifecycle(&span, "native.drain.settled", "python.native_pull");
            });
        });
        logs.force_flush().unwrap();
        let records = records.0.lock().unwrap();
        assert_eq!(records.len(), 2);
        for context in records.iter() {
            assert_eq!(
                context.trace_id.to_string(),
                "7123456789abcdef0123456789abcdef"
            );
            assert_eq!(context.span_id, native_id);
        }
        drop(records);
        drop(span);
        logs.shutdown().unwrap();
        traces.shutdown().unwrap();
    }
    fn settings() -> Settings {
        Settings::from_json(
            r#"{
            "schema_version":1,"disabled":true,
            "traces":{"enabled":false,"endpoint":null,"headers":{},"timeout_ms":200},
            "logs":{"enabled":false,"endpoint":null,"headers":{},"timeout_ms":200},
            "metrics":{"enabled":false,"endpoint":null,"headers":{},"timeout_ms":200},
            "resource":{"service.instance.id":"fixture-instance","service.name":"fixture"},
            "process_incarnation":"fixture-instance","max_queue_size":64,"max_export_batch_size":64,
            "schedule_delay_ms":5000,"export_timeout_ms":2000,"metric_export_interval_ms":60000,
            "sampler":"parentbased_always_on","sampler_argument":1.0
        }"#,
        )
        .unwrap()
    }
    #[test]
    fn resolved_settings_preserve_shared_incarnation_and_refuse_invalid_bounds() {
        let settings = settings();
        let encoded = serde_json::to_string(&settings).unwrap();
        let roundtrip = Settings::from_json(&encoded).unwrap();
        assert_eq!(roundtrip.process_incarnation, "fixture-instance");
        assert!(Settings::from_json(&encoded.replace("fixture-instance", "")).is_err());
        assert!(
            Settings::from_json(&encoded.replace("\"max_queue_size\":64", "\"max_queue_size\":0"))
                .is_err()
        );
        let host = Host::build(&roundtrip, None).unwrap();
        assert!(host.traces.is_none() && host.logs.is_none() && host.metrics.is_none());
        host.shutdown().unwrap();
    }

    #[test]
    fn private_settings_reject_unknown_fields_and_never_render_credentials() {
        let mut settings = settings();
        settings.traces.endpoint = Some("https://example.test/secret-path".into());
        settings
            .traces
            .headers
            .insert("authorization".into(), "secret-token".into());
        settings
            .resource
            .insert("private".into(), "secret-resource".into());
        for secret in ["secret-path", "secret-token", "secret-resource"] {
            assert!(!format!("{settings:?}").contains(secret));
        }
        let wire = serde_json::to_value(&settings).unwrap();
        let mut unknown = wire.clone();
        unknown["private-secret-key"] = serde_json::json!("secret-token");
        let mut nested = wire.clone();
        nested["traces"]["private-secret-key"] = serde_json::json!("secret-token");
        let mut invalid = wire.clone();
        invalid["traces"]["timeout_ms"] = serde_json::json!("secret-token");
        let mut url = wire.clone();
        url["traces"]["endpoint"] = serde_json::json!("https://secret-token@example.test");
        let mut headers = wire.clone();
        headers["traces"]["headers"]["authorization"] = serde_json::json!("secret-token\r\n");
        let mut oversized = wire.clone();
        oversized["resource"]["private"] = serde_json::json!("secret-token".repeat(65536));
        for refused in [unknown, nested, invalid, url, headers, oversized] {
            let error = Settings::from_json(&refused.to_string()).unwrap_err();
            for secret in [
                "private-secret-key",
                "secret-path",
                "secret-token",
                "secret-resource",
            ] {
                assert!(!format!("{error:?} {error}").contains(secret));
            }
        }
    }

    fn http_receiver() -> (String, std::thread::JoinHandle<Vec<u8>>) {
        use std::io::{Read, Write};
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let endpoint = format!("http://{}/v1/traces", listener.local_addr().unwrap());
        listener.set_nonblocking(true).unwrap();
        let thread = std::thread::spawn(move || {
            let deadline = std::time::Instant::now() + Duration::from_secs(5);
            let mut stream = loop {
                match listener.accept() {
                    Ok((stream, _)) => break stream,
                    Err(error)
                        if error.kind() == std::io::ErrorKind::WouldBlock
                            && std::time::Instant::now() < deadline =>
                    {
                        std::thread::sleep(Duration::from_millis(1))
                    }
                    Err(error) => panic!("local OTLP receiver unavailable: {error}"),
                }
            };
            stream
                .set_read_timeout(Some(Duration::from_secs(2)))
                .unwrap();
            stream
                .set_write_timeout(Some(Duration::from_secs(2)))
                .unwrap();
            let mut request = Vec::new();
            let mut buffer = [0; 4096];
            loop {
                let read = stream.read(&mut buffer).unwrap();
                assert!(read > 0);
                request.extend_from_slice(&buffer[..read]);
                assert!(request.len() < 65536);
                if let Some(offset) = request.windows(4).position(|bytes| bytes == b"\r\n\r\n") {
                    let headers = String::from_utf8_lossy(&request[..offset]);
                    let length = headers
                        .lines()
                        .find_map(|line| {
                            let (key, value) = line.split_once(':')?;
                            key.eq_ignore_ascii_case("content-length")
                                .then(|| value.trim().parse::<usize>().unwrap())
                        })
                        .unwrap();
                    if request.len() >= offset + 4 + length {
                        break;
                    }
                }
            }
            stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Type: application/x-protobuf\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").unwrap();
            request
        });
        (endpoint, thread)
    }
    #[test]
    fn thread_batch_http_export_works_with_current_thread_host_runtime() {
        let (endpoint, receiver) = http_receiver();
        let mut settings = settings();
        settings.disabled = false;
        settings.traces.enabled = true;
        settings.traces.endpoint = Some(endpoint);
        settings
            .traces
            .headers
            .insert("authorization".into(), "fixture-credential".into());
        let host = Host::build(&settings, None).unwrap();
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let dispatch = host.dispatch();
        let span = tracing::dispatcher::with_default(&dispatch, || {
            tracing::info_span!("pse.fixture.http")
        });
        runtime.block_on(
            tracing::Instrument::instrument(
                async {
                    tracing::warn!(
                        private_expression = "fixture-private-expression-must-not-export",
                        "fixture-private-error-must-not-export"
                    );
                    tokio::task::yield_now().await;
                },
                span,
            )
            .with_subscriber(dispatch),
        );
        host.shutdown().unwrap();
        let request = receiver.join().unwrap();
        let request = String::from_utf8_lossy(&request);
        assert!(request.starts_with("POST /v1/traces HTTP/1.1"));
        assert!(request.contains("authorization: fixture-credential"));
        assert!(request.contains("content-type: application/x-protobuf"));
        assert!(!request.contains("fixture-private-expression-must-not-export"));
        assert!(!request.contains("fixture-private-error-must-not-export"));
    }
    #[test]
    fn resolved_blocking_client_replaces_exporter_environment_header_overlay() {
        use opentelemetry_http::HttpClient;
        let (endpoint, receiver) = http_receiver();
        let mut settings = settings();
        settings
            .traces
            .headers
            .insert("authorization".into(), "resolved-credential".into());
        let client = client(&settings.traces).unwrap();
        let request = http::Request::post(endpoint)
            .header("authorization", "stale-environment-credential")
            .header("x-unresolved-environment", "must-not-escape")
            .header("content-type", "application/x-protobuf")
            .body(bytes::Bytes::from_static(b"fixture"))
            .unwrap();
        // This adapter delegates a blocking operation and completes its future in one poll.
        futures_util::FutureExt::now_or_never(client.send_bytes(request))
            .unwrap()
            .unwrap();
        let request = receiver.join().unwrap();
        let request = String::from_utf8_lossy(&request);
        assert!(request.contains("authorization: resolved-credential"));
        assert!(!request.contains("stale-environment-credential"));
        assert!(!request.contains("x-unresolved-environment"));
    }

    #[test]
    fn private_child_exit_control() {
        if std::env::var("PSE_TELEMETRY_EXIT_CONTROL").as_deref() != Ok("1") {
            return;
        }
        initialize_private_child_host().unwrap();
        tracing::info_span!("pse.fixture.private_exit").in_scope(|| {});
        let release =
            std::path::PathBuf::from(std::env::var_os("PSE_TELEMETRY_EXIT_READY").unwrap());
        let deadline = std::time::Instant::now() + Duration::from_secs(4);
        while !release.exists() && std::time::Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(1));
        }
        assert!(release.exists());
        // Return a native-control success while the SDK export remains in flight.
        // The actual process exits with its static host still owned by the OS.
    }
    #[test]
    fn private_child_exit_does_not_wait_for_delayed_exporter() {
        use std::io::{Read, Write};
        let directory = tempfile::tempdir().unwrap();
        let release_path = directory.path().join("native-owner-settled");
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let mut settings = settings();
        settings.disabled = false;
        settings.traces.enabled = true;
        settings.traces.endpoint = Some(format!(
            "http://{}/v1/traces",
            listener.local_addr().unwrap()
        ));
        settings.traces.timeout_ms = 5000;
        settings.max_export_batch_size = 1;
        settings.export_timeout_ms = 5000;
        let (started, accepted) = std::sync::mpsc::sync_channel(1);
        let (release, delayed) = std::sync::mpsc::sync_channel(1);
        let server = std::thread::spawn(move || {
            let deadline = std::time::Instant::now() + Duration::from_secs(5);
            let mut stream = loop {
                match listener.accept() {
                    Ok((stream, _)) => break stream,
                    Err(error)
                        if error.kind() == std::io::ErrorKind::WouldBlock
                            && std::time::Instant::now() < deadline =>
                    {
                        std::thread::sleep(Duration::from_millis(1))
                    }
                    Err(error) => panic!("delayed receiver unavailable: {error}"),
                }
            };
            stream
                .set_read_timeout(Some(Duration::from_secs(2)))
                .unwrap();
            let mut bytes = [0; 4096];
            assert!(stream.read(&mut bytes).unwrap() > 0);
            started.send(()).unwrap();
            // Only the parent test can release this response. SDK client/provider
            // deadlines are five seconds, much longer than the process-exit bound.
            let _ = delayed.recv_timeout(Duration::from_secs(5));
            let _ = stream
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");
        });
        struct Owned(std::process::Child);
        impl Drop for Owned {
            fn drop(&mut self) {
                let _ = self.0.kill();
                let _ = self.0.wait();
            }
        }
        let mut child = Owned(
            std::process::Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "tooling::telemetry::tests::private_child_exit_control",
                    "--nocapture",
                ])
                .env("PSE_TELEMETRY_EXIT_CONTROL", "1")
                .env("PSE_TELEMETRY_EXIT_READY", &release_path)
                .env(SETTINGS_ENV, serde_json::to_string(&settings).unwrap())
                .stdout(std::process::Stdio::null())
                .spawn()
                .unwrap(),
        );
        accepted.recv_timeout(Duration::from_secs(4)).unwrap();
        std::fs::write(&release_path, b"native owner settled").unwrap();
        let deadline = std::time::Instant::now() + Duration::from_millis(750);
        let status = loop {
            if let Some(status) = child.0.try_wait().unwrap() {
                break Some(status);
            }
            if std::time::Instant::now() >= deadline {
                break None;
            }
            std::thread::sleep(Duration::from_millis(1));
        };
        let _ = release.send(());
        server.join().unwrap();
        assert!(
            status.is_some_and(|status| status.success()),
            "private child process waited for optional exporter drain"
        );
    }
}
