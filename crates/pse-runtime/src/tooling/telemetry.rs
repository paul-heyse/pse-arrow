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
    Resource,
    logs::SdkLoggerProvider,
    metrics::SdkMeterProvider,
    propagation::TraceContextPropagator,
    trace::{SdkTracerProvider, SpanProcessor},
};
use serde::Serialize;
use std::{
    collections::{BTreeMap, HashMap},
    sync::{Arc, Mutex, OnceLock},
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tracing::instrument::WithSubscriber;
use tracing_opentelemetry::OpenTelemetrySpanExt;
use tracing_subscriber::{Layer, layer::SubscriberExt};

/// Instrumentation scope shared by native spans and the cheap host activation read.
pub const INSTRUMENTATION_SCOPE: &str = "pse-runtime";

fn exported_owner_span(metadata: &tracing::Metadata<'_>) -> bool {
    metadata.is_span()
        && (metadata.fields().field("pse.owner.stage").is_some()
            || pse_operations::transport::client::is_request_span(metadata))
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

/// Lifecycle of the owned trace processor, independent of SDK loss observations.
#[derive(Clone, Copy, Debug, Serialize, schemars::JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TraceShutdown {
    /// Native trace export was not selected.
    Unselected,
    /// An exit without a shutdown observation leaves buffered loss unknown.
    NotAttempted,
    /// Shutdown was requested; no completion is claimed.
    Attempted,
    /// SDK shutdown returned successfully, which is not ingestion proof.
    Succeeded,
    /// SDK shutdown failed; an earlier cumulative diagnostic remains an observation.
    Failed,
}

/// Native instrumentation scope observed by this finite adapter.
#[derive(Clone, Copy, Debug, Serialize, schemars::JsonSchema)]
pub enum SdkLossScope {
    /// Native process runtime instrumentation.
    #[serde(rename = "pse-runtime")]
    Runtime,
}
/// SDK signal covered by this loss observer.
#[derive(Clone, Copy, Debug, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SdkLossSignal {
    /// Completed native spans submitted to the trace processor.
    Traces,
}
/// SDK processor whose own structured diagnostics are observed.
#[derive(Clone, Copy, Debug, Serialize, schemars::JsonSchema)]
pub enum SdkLossProcessor {
    /// Unmodified SDK thread-based batch processor.
    BatchSpanProcessor,
}
/// Knowledge of telemetry lost on abrupt process exit.
#[derive(Clone, Copy, Debug, Serialize, schemars::JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SdkAbruptLoss {
    /// Neither queue-drop diagnostic measures abrupt buffered loss.
    Unknown,
}
/// Backend delivery evidence supplied by this observer.
#[derive(Clone, Copy, Debug, Serialize, schemars::JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SdkLossDelivery {
    /// Local processor diagnostics do not establish ingestion or drain.
    NotObserved,
}

/// Compact, passive observations for exactly one owned SDK trace processor.
/// No missing diagnostic is interpreted as zero and no count proves delivery.
#[derive(Clone, Debug, Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SdkLossHealth {
    /// Exact host incarnation; observations cannot be combined across incarnations.
    pub process_incarnation: String,
    /// Native instrumentation scope.
    pub scope: SdkLossScope,
    /// Signal observed by this finite adapter.
    pub signal: SdkLossSignal,
    /// SDK processor class; Python/log/metric queues are outside this observation.
    pub processor: SdkLossProcessor,
    /// Time this passive snapshot was copied.
    pub observed_at_unix_ns: Option<u64>,
    /// SDK queue capacity, not current occupancy.
    pub max_queue_size: usize,
    /// Whether the SDK's first full-queue diagnostic was observed, independently of clock availability.
    pub first_drop_observed: bool,
    /// Time of that first diagnostic, if the Unix clock was available.
    pub first_drop_observed_at_unix_ns: Option<u64>,
    /// SDK-reported cumulative count at its shutdown diagnostic, not a live counter.
    pub sdk_reported_dropped_span_count: Option<u64>,
    /// Time of that diagnostic; the SDK emits it before sending shutdown control.
    pub cumulative_count_observed_at_unix_ns: Option<u64>,
    /// Independent completion of the shutdown attempt.
    pub shutdown: TraceShutdown,
    /// Abrupt exit loss cannot be inferred from these two SDK diagnostics.
    pub abrupt_exit_loss: SdkAbruptLoss,
    /// Export, drain and backend ingestion are not established by a drop diagnostic.
    pub delivery: SdkLossDelivery,
}

fn observation_time_unix_ns() -> Option<u64> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|duration| u64::try_from(duration.as_nanos()).ok())
}

/// A clone retains local observations after the Host has shut down or been consumed.
#[derive(Clone, Debug)]
pub struct LossObserver(Arc<Mutex<SdkLossHealth>>);
impl LossObserver {
    fn new(settings: &Settings) -> Self {
        Self(Arc::new(Mutex::new(SdkLossHealth {
            process_incarnation: settings.process_incarnation.clone(),
            scope: SdkLossScope::Runtime,
            signal: SdkLossSignal::Traces,
            processor: SdkLossProcessor::BatchSpanProcessor,
            observed_at_unix_ns: observation_time_unix_ns(),
            max_queue_size: settings.max_queue_size,
            first_drop_observed: false,
            first_drop_observed_at_unix_ns: None,
            sdk_reported_dropped_span_count: None,
            cumulative_count_observed_at_unix_ns: None,
            shutdown: if settings.traces.enabled && !settings.disabled {
                TraceShutdown::NotAttempted
            } else {
                TraceShutdown::Unselected
            },
            abrupt_exit_loss: SdkAbruptLoss::Unknown,
            delivery: SdkLossDelivery::NotObserved,
        })))
    }
    /// Copy current local state without initialization, exporting or flushing.
    pub fn snapshot(&self) -> SdkLossHealth {
        let mut snapshot = self
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        snapshot.observed_at_unix_ns = observation_time_unix_ns();
        snapshot
    }
    fn shutdown(&self, status: TraceShutdown) {
        self.0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .shutdown = status;
    }
}

fn sdk_loss_event(metadata: &tracing::Metadata<'_>) -> bool {
    metadata.is_event()
        && metadata.target() == "opentelemetry_sdk"
        && matches!(
            metadata.name(),
            "BatchSpanProcessor.SpanDroppingStarted" | "BatchSpanProcessor.SpansDropped"
        )
}

#[derive(Default)]
struct LossFields {
    name: Option<&'static str>,
    count: Option<u64>,
    capacity: Option<u64>,
}
impl tracing::field::Visit for LossFields {
    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "name" {
            self.name = match value {
                "BatchSpanProcessor.SpanDroppingStarted" => {
                    Some("BatchSpanProcessor.SpanDroppingStarted")
                }
                "BatchSpanProcessor.SpansDropped" => Some("BatchSpanProcessor.SpansDropped"),
                _ => None,
            };
        }
    }
    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        match field.name() {
            "dropped_span_count" => self.count = Some(value),
            "max_queue_size" => self.capacity = Some(value),
            _ => {}
        }
    }
    fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
        if let Ok(value) = u64::try_from(value) {
            self.record_u64(field, value);
        }
    }
    fn record_debug(&mut self, _field: &tracing::field::Field, _value: &dyn std::fmt::Debug) {
        // Never render or parse diagnostic prose, errors, headers or arbitrary values.
    }
}
#[derive(Clone, Debug)]
struct SdkLossLayer(LossObserver);
impl<S: tracing::Subscriber> Layer<S> for SdkLossLayer {
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _context: tracing_subscriber::layer::Context<'_, S>,
    ) {
        if !sdk_loss_event(event.metadata()) {
            return;
        }
        let mut fields = LossFields::default();
        event.record(&mut fields);
        if fields.name != Some(event.metadata().name()) {
            return;
        }
        let mut state = self
            .0
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        match fields.name {
            Some("BatchSpanProcessor.SpanDroppingStarted") => {
                if !state.first_drop_observed {
                    state.first_drop_observed = true;
                    state.first_drop_observed_at_unix_ns = observation_time_unix_ns();
                }
            }
            Some("BatchSpanProcessor.SpansDropped")
                if fields.capacity == u64::try_from(state.max_queue_size).ok() =>
            {
                if let Some(count) = fields.count.filter(|count| *count > 0) {
                    state.sdk_reported_dropped_span_count = Some(count);
                    state.cumulative_count_observed_at_unix_ns = observation_time_unix_ns();
                }
            }
            _ => {}
        }
    }
}

/// Delegates all processing to the SDK's unchanged bounded batch processor.
/// Its diagnostics carry no processor identity: a private observer-only Dispatch
/// scopes synchronous queue-drop/shutdown events to this exact owned processor.
/// It contains no export layer and never changes a global subscriber/provider.
#[derive(Debug)]
struct ObservedBatchProcessor {
    batch: opentelemetry_sdk::trace::BatchSpanProcessor,
    diagnostics: tracing::Dispatch,
    observer: LossObserver,
}
impl ObservedBatchProcessor {
    fn new(batch: opentelemetry_sdk::trace::BatchSpanProcessor, observer: LossObserver) -> Self {
        let diagnostics = tracing::Dispatch::new(
            tracing_subscriber::registry().with(
                SdkLossLayer(observer.clone())
                    .with_filter(tracing_subscriber::filter::filter_fn(sdk_loss_event)),
            ),
        );
        Self {
            batch,
            diagnostics,
            observer,
        }
    }
}
impl SpanProcessor for ObservedBatchProcessor {
    fn on_start(&self, span: &mut opentelemetry_sdk::trace::Span, context: &Context) {
        self.batch.on_start(span, context);
    }
    fn on_end(&self, span: opentelemetry_sdk::trace::SpanData) {
        tracing::dispatcher::with_default(&self.diagnostics, || self.batch.on_end(span));
    }
    fn force_flush(&self) -> opentelemetry_sdk::error::OTelSdkResult {
        self.batch.force_flush()
    }
    fn shutdown_with_timeout(&self, timeout: Duration) -> opentelemetry_sdk::error::OTelSdkResult {
        self.observer.shutdown(TraceShutdown::Attempted);
        let result = tracing::dispatcher::with_default(&self.diagnostics, || {
            self.batch.shutdown_with_timeout(timeout)
        });
        self.observer.shutdown(if result.is_ok() {
            TraceShutdown::Succeeded
        } else {
            TraceShutdown::Failed
        });
        result
    }
    fn set_resource(&mut self, resource: &Resource) {
        self.batch.set_resource(resource);
    }
}

pub use pse_diagnostics::telemetry_settings::{
    SETTINGS_ENV, SETTINGS_VERSION, Settings, SignalSettings, TelemetryError, TelemetrySampler,
};
#[cfg(test)]
use pse_diagnostics::telemetry_settings::{
    TELEMETRY_METRIC_INTERVAL_MAX_MS, TELEMETRY_MINIMUM, TELEMETRY_QUEUE_MAX, TELEMETRY_RATIO_MAX,
    TELEMETRY_RATIO_MIN, TELEMETRY_TIMEOUT_MAX_MS,
};

trait SdkSampler {
    fn sdk(self, argument: f64) -> opentelemetry_sdk::trace::Sampler;
}
impl SdkSampler for TelemetrySampler {
    fn sdk(self, argument: f64) -> opentelemetry_sdk::trace::Sampler {
        use opentelemetry_sdk::trace::Sampler;
        match self {
            Self::AlwaysOn => Sampler::AlwaysOn,
            Self::AlwaysOff => Sampler::AlwaysOff,
            Self::TraceIdRatio => Sampler::TraceIdRatioBased(argument),
            Self::ParentBasedAlwaysOn => Sampler::ParentBased(Box::new(Sampler::AlwaysOn)),
            Self::ParentBasedAlwaysOff => Sampler::ParentBased(Box::new(Sampler::AlwaysOff)),
            Self::ParentBasedTraceIdRatio => {
                Sampler::ParentBased(Box::new(Sampler::TraceIdRatioBased(argument)))
            }
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
        let span = tracing::info_span!(parent: None, "pse.native.shared_producer", pse.owner.stage = true, owner);
        let context = self.context();
        if context.span().span_context().is_valid() {
            span.add_link(context.span().span_context().clone());
        }
        span
    }
    /// Set the explicit parent before entry; report unavailable attachment on the span.
    pub fn operation_span(&self, owner: &'static str) -> tracing::Span {
        let span = tracing::info_span!(parent: None, "pse.native.operation", pse.owner.stage = true, owner, telemetry.parent = tracing::field::Empty);
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
/// Poll-scoped propagation across owned tasks. No attached guard crosses await.
pub(crate) fn inherit<F: Future>(future: F) -> impl Future<Output = F::Output> {
    tracing::Instrument::instrument(future, tracing::Span::current())
        .with_subscriber(tracing::dispatcher::get_default(Clone::clone))
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
    loss: LossObserver,
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
            for event in super::operational_metrics::NativeEvent::ALL {
                let count = event.count(&sample);
                observer.observe(count, &[KeyValue::new("event", event.as_str())]);
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
                for event in super::operational_metrics::PreparationEvent::ALL {
                    let count = event.count(&stage);
                    observer.observe(
                        count,
                        &[
                            KeyValue::new("stage", stage.stage.name()),
                            KeyValue::new("event", event.as_str()),
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
            for product in crate::math::PreparationProduct::ALL {
                let count = product.count(&sample);
                observer.observe(count as u64, &[KeyValue::new("product", product.as_str())]);
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
            for event in pse_operations::canonical::CanonicalQueryEvent::ALL {
                let count = event.count(&sample);
                observer.observe(count, &[KeyValue::new("event", event.as_str())]);
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
            for owner in super::runtime_metrics::OperationalSource::ALL {
                if let Some(start) = owner.started() {
                    observer.observe(
                        start as f64 / 1_000_000.0,
                        &[KeyValue::new("owner", owner.as_str())],
                    );
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
                for event in super::runtime_metrics::TaskEvent::ALL {
                    let count = event.value(&task);
                    observer.observe(
                        count,
                        &[
                            KeyValue::new("task.class", task.class.as_str()),
                            KeyValue::new("event", event.as_str()),
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
                for phase in super::runtime_metrics::TaskPhase::ALL {
                    let nanos = phase.value(&task);
                    observer.observe(
                        nanos as f64 / 1_000_000_000.0,
                        &[
                            KeyValue::new("task.class", task.class.as_str()),
                            KeyValue::new("phase", phase.as_str()),
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
                for threshold in super::runtime_metrics::TaskThreshold::ALL {
                    let nanos = threshold.value(&task);
                    observer.observe(
                        nanos as f64 / 1_000_000_000.0,
                        &[
                            KeyValue::new("task.class", task.class.as_str()),
                            KeyValue::new("threshold", threshold.as_str()),
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
        let loss = LossObserver::new(settings);
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
                    .with_sampler(settings.sampler.sdk(settings.sampler_argument))
                    .with_span_processor(ObservedBatchProcessor::new(
                        opentelemetry_sdk::trace::BatchSpanProcessor::builder(exporter)
                            .with_batch_config(batch)
                            .build(),
                        loss.clone(),
                    ))
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
            loss,
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
    /// Retain passive observations independently of Host consumption/shutdown.
    pub fn loss_observer(&self) -> LossObserver {
        self.loss.clone()
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
    struct Recorded(Arc<Mutex<Vec<opentelemetry_sdk::trace::SpanData>>>);
    impl Recorded {
        fn snapshot(&self) -> Vec<opentelemetry_sdk::trace::SpanData> {
            // Assertions must not hold this lock: unwinding can close live spans
            // and synchronously call the exporter again.
            self.0.lock().unwrap().clone()
        }
    }
    impl opentelemetry_sdk::trace::SpanExporter for Recorded {
        async fn export(
            &self,
            batch: Vec<opentelemetry_sdk::trace::SpanData>,
        ) -> opentelemetry_sdk::error::OTelSdkResult {
            self.0
                .lock()
                .map_err(|error| {
                    opentelemetry_sdk::error::OTelSdkError::InternalFailure(format!(
                        "recorded span mutex poisoned: {error}"
                    ))
                })?
                .extend(batch);
            Ok(())
        }
    }
    #[test]
    fn recorded_exporter_reports_poison_without_panicking_during_cleanup() {
        let recorded = Recorded::default();
        let poisoned = std::panic::catch_unwind(|| {
            let _records = recorded.0.lock().unwrap();
            panic!("fixture assertion while holding exporter storage");
        });
        assert!(poisoned.is_err());
        let runtime = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();
        let result = runtime.block_on(opentelemetry_sdk::trace::SpanExporter::export(
            &recorded,
            Vec::new(),
        ));
        assert!(matches!(
            result,
            Err(opentelemetry_sdk::error::OTelSdkError::InternalFailure(_))
        ));
    }
    #[test]
    fn production_span_filter_retains_exact_request_parent_without_events_or_other_library_spans() {
        use pse_operations::transport::client::{REQUEST_SPAN_NAME, REQUEST_SPAN_TARGET};
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
            let native = carrier.operation_span("fixture.canonical");
            let native_id = native.context().span().span_context().span_id();
            native.in_scope(|| {
                let request = tracing::info_span!(
                    target: REQUEST_SPAN_TARGET,
                    REQUEST_SPAN_NAME
                );
                assert!(!request.is_disabled());
                request.in_scope(|| {
                    tracing::warn!(target: REQUEST_SPAN_TARGET, private = "must-not-export", "private request event");
                    let unrelated = tracing::info_span!(target: "unrelated.grpc", REQUEST_SPAN_NAME);
                    assert!(unrelated.is_disabled());
                    let wrong_request_name = tracing::info_span!(target: REQUEST_SPAN_TARGET, "unrelated.lifecycle.span");
                    assert!(wrong_request_name.is_disabled());
                    let inner = tracing::info_span!(target: "pse_runtime::math", "pse.inner.expression");
                    assert!(inner.is_disabled());
                    let lookalike = tracing::info_span!(target: "pse_unrelated", "pse.lookalike");
                    assert!(lookalike.is_disabled());
                });
            });
            native_id
        });
        provider.force_flush().unwrap();
        let records = recorded.snapshot();
        assert_eq!(records.len(), 2);
        let native = records
            .iter()
            .find(|span| span.name == "pse.native.operation")
            .unwrap();
        let request = records
            .iter()
            .find(|span| span.name == REQUEST_SPAN_NAME)
            .unwrap();
        assert_eq!(native.parent_span_id.to_string(), "9123456789abcdef");
        assert_eq!(request.parent_span_id, native_id);
        assert_eq!(
            request.span_context.trace_id().to_string(),
            "8123456789abcdef0123456789abcdef"
        );
        assert!(records.iter().all(|span| span.events.events.is_empty()));
        provider.shutdown().unwrap();
    }
    #[test]
    fn production_span_filter_preserves_inner_spans_in_finite_capture() {
        use crate::tooling::capture::{
            CaptureKind, CaptureOutputKind, CaptureRequest, start_capture,
        };
        use prost::Message;
        use tracing_perfetto_sdk_schema::{Trace, trace_packet, track_event};
        let directory = tempfile::tempdir().unwrap();
        let diagnostics =
            crate::observation::Diagnostics::new(Default::default(), directory.path().to_owned());
        let operation = diagnostics.hub().start("finite-capture-owner");
        diagnostics.publish().unwrap();
        let session = start_capture(CaptureRequest {
            version: pse_model::document::Version,
            checkpoint: diagnostics.checkpoint_path(),
            operation: operation.context().id(),
            kind: CaptureKind::Timeline,
            output: directory.path().join("capture"),
            duration: Duration::from_millis(100),
            budget: Default::default(),
        })
        .unwrap();
        let recorded = Recorded::default();
        let provider = SdkTracerProvider::builder()
            .with_simple_exporter(recorded.clone())
            .build();
        let subscriber = tracing_subscriber::registry().with(session.layer()).with(
            tracing_opentelemetry::layer()
                .with_tracer(provider.tracer("fixture"))
                .with_filter(tracing_subscriber::filter::filter_fn(exported_owner_span)),
        );
        tracing::subscriber::with_default(subscriber, || {
            let owner = Carrier::default().operation_span("fixture.capture");
            owner.in_scope(|| {
                let inner =
                    tracing::info_span!(target: "pse_runtime::math", "fixture.inner.expression");
                assert!(!inner.is_disabled());
                inner.in_scope(|| {});
            });
        });
        provider.force_flush().unwrap();
        let exported = recorded.snapshot();
        assert_eq!(exported.len(), 1);
        assert_eq!(exported[0].name, "pse.native.operation");
        let captured = session.wait().unwrap();
        let output = captured
            .outputs
            .iter()
            .find(|output| output.kind == CaptureOutputKind::Perfetto)
            .unwrap();
        let mut bytes = std::fs::read(&output.path).unwrap();
        bytes.truncate(output.readable_prefix_bytes as usize);
        let timeline = Trace::decode(bytes.as_slice()).unwrap();
        assert!(timeline.packet.iter().any(|packet| {
            matches!(&packet.data,
                Some(trace_packet::Data::TrackEvent(event))
                if matches!(&event.name_field, Some(track_event::NameField::Name(name)) if name == "fixture.inner.expression"))
        }));
        assert!(operation.context().is_active());
        provider.shutdown().unwrap();
    }

    #[derive(Debug, Default)]
    struct ExportGate {
        state: Mutex<(bool, bool)>,
        changed: std::sync::Condvar,
    }
    impl ExportGate {
        fn wait_started(&self) {
            let state = self.state.lock().unwrap();
            let (state, _) = self
                .changed
                .wait_timeout_while(state, Duration::from_secs(5), |state| !state.0)
                .unwrap();
            assert!(
                state.0,
                "SDK exporter did not begin its first blocked batch"
            );
        }
        fn release(&self) {
            self.state.lock().unwrap().1 = true;
            self.changed.notify_all();
        }
    }
    #[derive(Clone, Debug)]
    struct BlockedExporter(Arc<ExportGate>);
    impl opentelemetry_sdk::trace::SpanExporter for BlockedExporter {
        async fn export(
            &self,
            _batch: Vec<opentelemetry_sdk::trace::SpanData>,
        ) -> opentelemetry_sdk::error::OTelSdkResult {
            let mut state = self.0.state.lock().unwrap();
            state.0 = true;
            self.0.changed.notify_all();
            while !state.1 {
                state = self.0.changed.wait(state).unwrap();
            }
            Ok(())
        }
    }
    struct ReleaseExporter(Arc<ExportGate>);
    impl Drop for ReleaseExporter {
        fn drop(&mut self) {
            self.0.release();
        }
    }
    fn pressured_trace_processor() -> (
        SdkTracerProvider,
        LossObserver,
        Arc<ExportGate>,
        ReleaseExporter,
    ) {
        let mut selected = settings();
        selected.disabled = false;
        selected.traces.enabled = true;
        selected.max_queue_size = 2;
        let observer = LossObserver::new(&selected);
        let gate = Arc::new(ExportGate::default());
        let batch =
            opentelemetry_sdk::trace::BatchSpanProcessor::builder(BlockedExporter(gate.clone()))
                .with_batch_config(
                    opentelemetry_sdk::trace::BatchConfigBuilder::default()
                        .with_max_queue_size(2)
                        .with_max_export_batch_size(1)
                        .with_scheduled_delay(Duration::from_secs(60))
                        .build(),
                )
                .build();
        let processor = ObservedBatchProcessor::new(batch, observer.clone());
        let provider = SdkTracerProvider::builder()
            .with_span_processor(processor)
            .build();
        // Construct the release guard before assertion/timeout paths so test failure
        // cannot leave an SDK thread blocked on a test-owned gate.
        let release = ReleaseExporter(gate.clone());
        let subscriber = tracing_subscriber::registry().with(
            tracing_opentelemetry::layer()
                .with_tracer(provider.tracer("queue-pressure"))
                .with_filter(tracing_subscriber::filter::filter_fn(exported_owner_span)),
        );
        // Exercise the production tracing layer's on_close path, including nested
        // dispatch during SDK on_end, rather than only direct SDK API emissions.
        tracing::subscriber::with_default(subscriber, || {
            tracing::info_span!("fixture.in-flight", pse.owner.stage = true).in_scope(|| {});
            gate.wait_started();
            for _ in 0..7 {
                tracing::info_span!("fixture.queued-or-dropped", pse.owner.stage = true)
                    .in_scope(|| {});
            }
        });
        (provider, observer, gate, release)
    }
    #[test]
    fn sdk_loss_reports_first_drop_then_shutdown_cumulative_without_delivery_claim() {
        let (provider, observer, gate, _release) = pressured_trace_processor();
        let live = observer.snapshot();
        assert_eq!(live.process_incarnation, "fixture-instance");
        assert!(matches!(live.signal, SdkLossSignal::Traces));
        assert!(matches!(
            live.processor,
            SdkLossProcessor::BatchSpanProcessor
        ));
        assert!(live.first_drop_observed);
        assert!(live.first_drop_observed_at_unix_ns.is_some());
        assert_eq!(live.sdk_reported_dropped_span_count, None);
        assert_eq!(live.shutdown, TraceShutdown::NotAttempted);
        assert_eq!(live.abrupt_exit_loss, SdkAbruptLoss::Unknown);
        gate.release();
        provider.shutdown().unwrap();
        let final_observation = observer.snapshot();
        assert_eq!(final_observation.sdk_reported_dropped_span_count, Some(5));
        assert!(
            final_observation
                .cumulative_count_observed_at_unix_ns
                .is_some()
        );
        assert_eq!(final_observation.shutdown, TraceShutdown::Succeeded);
        assert_eq!(final_observation.delivery, SdkLossDelivery::NotObserved);
        assert_eq!(final_observation.abrupt_exit_loss, SdkAbruptLoss::Unknown);
    }
    #[test]
    fn sdk_loss_shutdown_diagnostic_survives_failed_shutdown_without_claiming_drain() {
        let (provider, observer, _gate, _release) = pressured_trace_processor();
        let result = provider.shutdown_with_timeout(Duration::from_millis(1));
        assert!(result.is_err());
        let snapshot = observer.snapshot();
        assert_eq!(snapshot.shutdown, TraceShutdown::Failed);
        assert_eq!(snapshot.sdk_reported_dropped_span_count, Some(5));
        assert_eq!(snapshot.delivery, SdkLossDelivery::NotObserved);
        assert_eq!(snapshot.abrupt_exit_loss, SdkAbruptLoss::Unknown);
    }
    #[test]
    #[expect(
        clippy::exit,
        reason = "this subprocess control deliberately omits Rust drops and SDK shutdown"
    )]
    fn sdk_loss_abrupt_exit_control() {
        let Some(output) = std::env::var_os("PSE_SDK_LOSS_ABRUPT_SNAPSHOT") else {
            return;
        };
        let (_provider, observer, _gate, _release) = pressured_trace_processor();
        std::fs::write(output, serde_json::to_vec(&observer.snapshot()).unwrap()).unwrap();
        // Deliberately omit all SDK shutdown and Rust drops, as a process exit does.
        std::process::exit(0);
    }
    #[test]
    fn sdk_loss_abrupt_exit_retains_first_drop_with_cumulative_and_buffered_loss_unknown() {
        let directory = tempfile::tempdir().unwrap();
        let snapshot = directory.path().join("abrupt-observation.json");
        let result = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "tooling::telemetry::tests::sdk_loss_abrupt_exit_control",
                "--nocapture",
            ])
            .env("PSE_SDK_LOSS_ABRUPT_SNAPSHOT", &snapshot)
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "abrupt-loss control failed: {}",
            String::from_utf8_lossy(&result.stderr)
        );
        let retained: serde_json::Value =
            serde_json::from_slice(&std::fs::read(snapshot).unwrap()).unwrap();
        assert_eq!(retained["first_drop_observed"], true);
        assert!(retained["sdk_reported_dropped_span_count"].is_null());
        assert_eq!(retained["shutdown"], "not_attempted");
        assert_eq!(retained["abrupt_exit_loss"], "unknown");
        assert_eq!(retained["delivery"], "not_observed");
    }
    #[test]
    fn sdk_loss_observer_ignores_prose_other_targets_processors_and_malformed_fields() {
        let observer = LossObserver::new(&settings());
        let diagnostics = tracing::Dispatch::new(
            tracing_subscriber::registry().with(SdkLossLayer(observer.clone())),
        );
        tracing::dispatcher::with_default(&diagnostics, || {
            tracing::warn!(name: "BatchSpanProcessor.SpanDroppingStarted", target: "other_sdk", name = "BatchSpanProcessor.SpanDroppingStarted");
            tracing::warn!(name: "BatchLogProcessor.LogsDropped", target: "opentelemetry_sdk", name = "BatchLogProcessor.LogsDropped", dropped_span_count = 999u64);
            tracing::warn!(name: "BatchSpanProcessor.SpansDropped", target: "opentelemetry_sdk", name = "BatchSpanProcessor.SpansDropped", dropped_span_count = "999", max_queue_size = 64u64, message = "private-secret-must-not-parse");
            tracing::warn!(name: "BatchSpanProcessor.SpansDropped", target: "opentelemetry_sdk", name = "BatchSpanProcessor.SpansDropped", dropped_span_count = 999u64, max_queue_size = 32u64);
        });
        let snapshot = observer.snapshot();
        assert_eq!(snapshot.first_drop_observed_at_unix_ns, None);
        assert_eq!(snapshot.sdk_reported_dropped_span_count, None);
        assert_eq!(snapshot.shutdown, TraceShutdown::Unselected);
        assert!(
            !serde_json::to_string(&snapshot)
                .unwrap()
                .contains("private-secret")
        );
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
        let records = recorded.snapshot();
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
        let records = recorded.snapshot();
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
            initiator.in_scope(crate::ExecutionContext::capture)
        });
        let initiator_context = initiator.context().span().span_context().clone();
        let producer_context = context.shared_producer("fixture.continued");
        // The captured context owns an initiator span clone. Release both
        // owners before polling the independent producer to exercise an ended
        // initiator rather than just a dropped local span handle.
        drop(context);
        drop(initiator);
        provider.force_flush().unwrap();
        let ended = recorded.snapshot();
        assert_eq!(ended.len(), 1);
        assert_eq!(ended[0].name, "pse.native.operation");
        assert_eq!(ended[0].span_context, initiator_context);
        assert_eq!(ended[0].parent_span_id.to_string(), "6123456789abcdef");
        assert!(ended[0].parent_span_is_remote);
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let continued = runtime.block_on(producer_context.instrument_with(
            async {
                tokio::task::yield_now().await;
                Carrier::current().context().span().span_context().clone()
            },
            |_| Some(pse_diagnostics::observation::Outcome::Completed),
        ));
        provider.force_flush().unwrap();
        let records = recorded.snapshot();
        provider.shutdown().unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].name, "pse.native.operation");
        let producer = &records[1];
        assert_eq!(producer.name, "pse.native.shared_producer");
        assert_eq!(
            producer.parent_span_id,
            opentelemetry::trace::SpanId::INVALID
        );
        assert_eq!(producer.links.links.len(), 1);
        assert_eq!(producer.links.links[0].span_context, initiator_context);
        assert!(continued.is_valid());
        assert_eq!(continued.trace_id(), producer.span_context.trace_id());
        assert_eq!(continued.span_id(), producer.span_context.span_id());
        assert_eq!(continued.trace_flags(), producer.span_context.trace_flags());
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
    fn resolved_settings_sampler_adapters_preserve_parent_semantics() {
        use opentelemetry::trace::{
            SpanContext, SpanId, SpanKind, TraceFlags, TraceId, TraceState,
        };
        use opentelemetry_sdk::trace::{SamplingDecision, ShouldSample};
        let trace_id = TraceId::from(1_u128);
        for (name, root_sampled, parent_based) in [
            ("always_on", true, false),
            ("always_off", false, false),
            ("traceidratio", false, false),
            ("parentbased_always_on", true, true),
            ("parentbased_always_off", false, true),
            ("parentbased_traceidratio", false, true),
        ] {
            let selection: TelemetrySampler =
                serde_json::from_value(serde_json::json!(name)).unwrap();
            assert_eq!(serde_json::to_value(selection).unwrap(), name);
            let sampler = selection.sdk(0.0);
            let decision = |parent: Option<&Context>| {
                sampler
                    .should_sample(parent, trace_id, "fixture", &SpanKind::Internal, &[], &[])
                    .decision
            };
            let sampled = |value| {
                if value {
                    SamplingDecision::RecordAndSample
                } else {
                    SamplingDecision::Drop
                }
            };
            assert_eq!(decision(None), sampled(root_sampled), "{name} root");
            for remote in [false, true] {
                for parent_sampled in [false, true] {
                    let parent = Context::new().with_remote_span_context(SpanContext::new(
                        trace_id,
                        SpanId::from(1_u64),
                        if parent_sampled {
                            TraceFlags::SAMPLED
                        } else {
                            TraceFlags::default()
                        },
                        remote,
                        TraceState::default(),
                    ));
                    assert_eq!(
                        decision(Some(&parent)),
                        sampled(if parent_based {
                            parent_sampled
                        } else {
                            root_sampled
                        }),
                        "{name} parent"
                    );
                }
            }
        }
    }

    #[test]
    fn resolved_settings_scalar_schema_and_receiving_admission_agree() {
        let schema = schemars::schema_for!(Settings).to_value();
        let original = serde_json::to_value(settings()).unwrap();
        for (field, maximum) in [
            ("max_queue_size", TELEMETRY_QUEUE_MAX as u64),
            ("max_export_batch_size", TELEMETRY_QUEUE_MAX as u64),
            ("schedule_delay_ms", TELEMETRY_TIMEOUT_MAX_MS),
            ("export_timeout_ms", TELEMETRY_TIMEOUT_MAX_MS),
            (
                "metric_export_interval_ms",
                TELEMETRY_METRIC_INTERVAL_MAX_MS,
            ),
        ] {
            assert_eq!(schema["properties"][field]["minimum"], TELEMETRY_MINIMUM);
            assert_eq!(schema["properties"][field]["maximum"], maximum);
            for endpoint in [TELEMETRY_MINIMUM, maximum] {
                let mut wire = original.clone();
                wire["max_queue_size"] = serde_json::json!(TELEMETRY_QUEUE_MAX);
                wire["max_export_batch_size"] = serde_json::json!(1);
                wire[field] = serde_json::json!(endpoint);
                assert!(
                    Settings::from_json(&wire.to_string()).is_ok(),
                    "{field}={endpoint}"
                );
            }
            for outside in [0, maximum + 1] {
                let mut wire = original.clone();
                wire[field] = serde_json::json!(outside);
                assert!(
                    Settings::from_json(&wire.to_string()).is_err(),
                    "{field}={outside}"
                );
            }
        }
        for endpoint in [TELEMETRY_MINIMUM, TELEMETRY_TIMEOUT_MAX_MS] {
            let mut wire = original.clone();
            wire["traces"]["timeout_ms"] = serde_json::json!(endpoint);
            assert!(Settings::from_json(&wire.to_string()).is_ok());
        }
        for outside in [0, TELEMETRY_TIMEOUT_MAX_MS + 1] {
            let mut wire = original.clone();
            wire["traces"]["timeout_ms"] = serde_json::json!(outside);
            assert!(Settings::from_json(&wire.to_string()).is_err());
        }
        for ratio in [TELEMETRY_RATIO_MIN, TELEMETRY_RATIO_MAX] {
            let mut wire = original.clone();
            wire["sampler_argument"] = serde_json::json!(ratio);
            assert!(Settings::from_json(&wire.to_string()).is_ok());
        }
        for ratio in [-0.01, 1.01] {
            let mut wire = original.clone();
            wire["sampler_argument"] = serde_json::json!(ratio);
            assert!(Settings::from_json(&wire.to_string()).is_err());
        }
        let mut direct = settings();
        direct.sampler_argument = f64::NAN;
        assert!(direct.validate().is_err());
        direct.sampler_argument = f64::INFINITY;
        assert!(direct.validate().is_err());
        let mut wire = original.clone();
        wire["max_export_batch_size"] = serde_json::json!(65);
        assert!(Settings::from_json(&wire.to_string()).is_err());
        wire = original;
        wire["sampler"] = serde_json::json!("unknown_sampler");
        assert!(Settings::from_json(&wire.to_string()).is_err());
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
            tracing::info_span!("pse.fixture.http", pse.owner.stage = true)
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
        tracing::info_span!("pse.fixture.private_exit", pse.owner.stage = true).in_scope(|| {});
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
