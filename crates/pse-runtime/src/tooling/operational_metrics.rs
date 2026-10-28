// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Owner-issued operational populations, independent of recording or sampled spans.
//! Totals start at first owner activity in this process and never reset in-process.
use std::{
    sync::{
        OnceLock,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    time::{Duration, Instant, SystemTime},
};

static SATURATED: AtomicBool = AtomicBool::new(false);
static START: OnceLock<u128> = OnceLock::new();
static NATIVE: [AtomicU64; NativeEvent::ALL.len()] =
    [const { AtomicU64::new(0) }; NativeEvent::ALL.len()];
static SUPERVISORS: AtomicU64 = AtomicU64::new(0);
static PREPARATION: [[AtomicU64; PreparationEvent::ALL.len()]; PreparationStage::ALL.len()] =
    [const { [const { AtomicU64::new(0) }; PreparationEvent::ALL.len()] };
        PreparationStage::ALL.len()];
/// Closed identities issued by the performing metric owner.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NativeEvent {
    /// `attempt` population.
    Attempt,
    /// `admitted` population.
    Admitted,
    /// `dispatched` population.
    Dispatched,
    /// `joined` population.
    Joined,
    /// `cancel_requested` population.
    CancelRequested,
    /// `deadline_requested` population.
    DeadlineRequested,
}
impl NativeEvent {
    pub(crate) const ALL: [Self; 6] = [
        Self::Attempt,
        Self::Admitted,
        Self::Dispatched,
        Self::Joined,
        Self::CancelRequested,
        Self::DeadlineRequested,
    ];
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Attempt => "attempt",
            Self::Admitted => "admitted",
            Self::Dispatched => "dispatched",
            Self::Joined => "joined",
            Self::CancelRequested => "cancel_requested",
            Self::DeadlineRequested => "deadline_requested",
        }
    }
}
impl NativeEvent {
    fn increment(self) {
        increment(&NATIVE[self as usize]);
    }
    pub(crate) fn count(self, sample: &OperationalMetricsSnapshot) -> u64 {
        match self {
            Self::Attempt => sample.native_attempts,
            Self::Admitted => sample.native_admitted,
            Self::Dispatched => sample.native_dispatched,
            Self::Joined => sample.native_joined,
            Self::CancelRequested => sample.native_cancel_requests,
            Self::DeadlineRequested => sample.native_deadline_requests,
        }
    }
}
/// Closed identities issued by the performing metric owner.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PreparationEvent {
    /// `acquisition` population.
    Acquisition,
    /// `retained_reuse` population.
    RetainedReuse,
    /// `producer` population.
    Producer,
    /// `flight_waiter` population.
    FlightWaiter,
}
impl PreparationEvent {
    pub(crate) const ALL: [Self; 4] = [
        Self::Acquisition,
        Self::RetainedReuse,
        Self::Producer,
        Self::FlightWaiter,
    ];
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Acquisition => "acquisition",
            Self::RetainedReuse => "retained_reuse",
            Self::Producer => "producer",
            Self::FlightWaiter => "flight_waiter",
        }
    }
}
impl PreparationEvent {
    pub(crate) fn count(self, sample: &PreparationMetricsSnapshot) -> u64 {
        match self {
            Self::Acquisition => sample.acquisitions,
            Self::RetainedReuse => sample.retained_reuses,
            Self::Producer => sample.producers,
            Self::FlightWaiter => sample.flight_waiters,
        }
    }
}

fn start() -> u128 {
    *START.get_or_init(|| {
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_micros()
    })
}
fn increment(counter: &AtomicU64) {
    let _ = start();
    let _ = counter.try_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
        let next = current.checked_add(1);
        if next.is_none() {
            SATURATED.store(true, Ordering::Relaxed);
        }
        Some(next.unwrap_or(u64::MAX))
    });
}
pub(crate) fn preparation_activity() {
    let _ = start();
}

/// Bounded actual shared immutable-product owner.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum PreparationStage {
    /// Shared modeling frontier.
    Frontier,
    /// Shared prepared basis.
    Basis,
}
impl PreparationStage {
    pub(super) const ALL: [Self; 2] = [Self::Frontier, Self::Basis];
    fn index(self) -> usize {
        match self {
            Self::Frontier => 0,
            Self::Basis => 1,
        }
    }
    pub(super) fn name(self) -> &'static str {
        match self {
            Self::Frontier => "frontier",
            Self::Basis => "basis",
        }
    }
    /// A caller entered a valid acquisition; this includes retained reads and flight waiters.
    pub(crate) fn acquire(self) {
        increment(&PREPARATION[self.index()][PreparationEvent::Acquisition as usize]);
    }
    /// An actual valid retained product was returned, including a producer's cache recheck.
    pub(crate) fn reuse(self) {
        increment(&PREPARATION[self.index()][PreparationEvent::RetainedReuse as usize]);
    }
    /// A caller attached to a shared flight after the retained-product probe missed.
    pub(crate) fn waiter(self) {
        increment(&PREPARATION[self.index()][PreparationEvent::FlightWaiter as usize]);
    }
    /// Called only inside the single-flight producer future, never once per waiter.
    pub(crate) fn producer(self) -> OwnerInterval {
        increment(&PREPARATION[self.index()][PreparationEvent::Producer as usize]);
        OwnerInterval::new(OwnerIntervalFamily::Preparation, self.name())
    }
}
/// Copied owner totals. Acquisition callers and shared producers are overlapping populations.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct OperationalMetricsSnapshot {
    /// First owner activity in Unix microseconds; SDK collection start is different.
    pub source_started_unix_micros: Option<u128>,
    /// A process counter saturated; exact increase queries must decline this source.
    pub saturated: bool,
    /// Attempts entering the native supervisor admission route; not scientific solves.
    pub native_attempts: u64,
    /// Actual active CPU/memory admissions in this supervisor route.
    pub native_admitted: u64,
    /// Worker threads successfully spawned by this route.
    pub native_dispatched: u64,
    /// Threads witnessed joined, including TLS/foreign destruction.
    pub native_joined: u64,
    /// User-cancellation branch selections while join was still pending.
    pub native_cancel_requests: u64,
    /// Deadline branch selections while join was still pending.
    pub native_deadline_requests: u64,
    /// Current outstanding supervisor owners; not a physical-release proof.
    pub native_supervisors: u64,
    /// Exactly frontier and basis shared-preparation populations.
    pub preparation: Vec<PreparationMetricsSnapshot>,
}
/// One shared-product stage, process cumulative; misses supply no inferred invalidation cause.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PreparationMetricsSnapshot {
    /// Fixed stage, frontier or basis.
    pub stage: PreparationStage,
    /// Valid acquisition callers, including retained readers and all flight waiters.
    pub acquisitions: u64,
    /// Retained products reused, including producer cache rechecks.
    pub retained_reuses: u64,
    /// Polled caller flight attempts after the retained probe missed; includes capacity/retiring refusals. Producer ownership is separate.
    pub flight_waiters: u64,
    /// Single-flight producer executions, including a retained-product recheck.
    pub producers: u64,
}
/// Passive reads never initialize an executor or perform scientific work.
pub fn snapshot() -> OperationalMetricsSnapshot {
    let read = |event: NativeEvent| NATIVE[event as usize].load(Ordering::Relaxed);
    OperationalMetricsSnapshot {
        source_started_unix_micros: START.get().copied(),
        saturated: SATURATED.load(Ordering::Relaxed),
        native_attempts: read(NativeEvent::Attempt),
        native_admitted: read(NativeEvent::Admitted),
        native_dispatched: read(NativeEvent::Dispatched),
        native_joined: read(NativeEvent::Joined),
        native_cancel_requests: read(NativeEvent::CancelRequested),
        native_deadline_requests: read(NativeEvent::DeadlineRequested),
        native_supervisors: SUPERVISORS.load(Ordering::Relaxed),
        preparation: PreparationStage::ALL
            .into_iter()
            .map(|stage| {
                let read = |event: PreparationEvent| {
                    PREPARATION[stage.index()][event as usize].load(Ordering::Relaxed)
                };
                PreparationMetricsSnapshot {
                    stage,
                    acquisitions: read(PreparationEvent::Acquisition),
                    retained_reuses: read(PreparationEvent::RetainedReuse),
                    producers: read(PreparationEvent::Producer),
                    flight_waiters: read(PreparationEvent::FlightWaiter),
                }
            })
            .collect(),
    }
}
/// The existing supervisor retains this observer through admission and actual join.
pub(crate) struct NativeJobLifecycle {
    _interval: OwnerInterval,
}
impl NativeJobLifecycle {
    pub(crate) fn new() -> Self {
        NativeEvent::Attempt.increment();
        increment(&SUPERVISORS);
        Self {
            _interval: OwnerInterval::new(OwnerIntervalFamily::Native, SUPERVISOR_STAGE),
        }
    }
    pub(crate) fn joined(&self) {
        NativeEvent::Joined.increment();
    }
    pub(crate) fn observe(
        &self,
        boundary: pse_diagnostics::lifecycle::LifecycleBoundary,
        value: bool,
    ) {
        if let Some(context) = pse_diagnostics::observation::current() {
            context.boolean(boundary.native_job_key(), value);
        }
        let code = match boundary {
            pse_diagnostics::lifecycle::LifecycleBoundary::StopRequested => {
                "native.cancel.requested"
            }
            pse_diagnostics::lifecycle::LifecycleBoundary::DeadlineExpired => {
                "native.deadline.expired"
            }
            pse_diagnostics::lifecycle::LifecycleBoundary::Joined => "native.join.settled",
            _ => boundary.native_job_key(),
        };
        tracing::info!(target: "pse.telemetry.diagnostic", code, value, "native job lifecycle");
    }
    pub(crate) fn returned(&self, failed: bool) {
        use pse_diagnostics::lifecycle::LifecycleBoundary as B;
        if let Some(context) = pse_diagnostics::observation::current() {
            context.boolean(B::Joined.native_job_key(), true);
            context.boolean(B::ReturnFailed.native_job_key(), failed);
        }
        tracing::info!(target: "pse.telemetry.diagnostic", code = "native.join.settled", native.failed = failed, "native worker and foreign destructors joined");
    }
    pub(crate) fn admitted(&self) {
        NativeEvent::Admitted.increment();
    }
    pub(crate) fn dispatched(&self) {
        NativeEvent::Dispatched.increment();
    }
    pub(crate) fn cancelled(&self) {
        NativeEvent::CancelRequested.increment();
    }
    pub(crate) fn expired(&self) {
        NativeEvent::DeadlineRequested.increment();
    }
}
impl Drop for NativeJobLifecycle {
    fn drop(&mut self) {
        SUPERVISORS.fetch_sub(1, Ordering::Relaxed);
    }
}

pub(crate) const SUPERVISOR_STAGE: &str = "supervisor";
/// Families of actual inclusive owner intervals.
#[derive(Clone, Copy, Debug)]
pub(crate) enum OwnerIntervalFamily {
    Native,
    Preparation,
}
impl OwnerIntervalFamily {
    pub(crate) const ALL: [Self; 2] = [Self::Native, Self::Preparation];
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Native => "native",
            Self::Preparation => "preparation",
        }
    }
}

/// Exact completed inclusive wall interval. Histograms cover intervals whose owner
/// starts after SDK activation; no pre-activation distribution is reconstructed.
pub(crate) struct OwnerInterval {
    started: Instant,
    histogram: opentelemetry::metrics::Histogram<f64>,
    attributes: [opentelemetry::KeyValue; 2],
}
impl OwnerInterval {
    fn new(family: OwnerIntervalFamily, stage: &'static str) -> Self {
        let meter = opentelemetry::global::meter(super::telemetry::INSTRUMENTATION_SCOPE);
        let histogram = owner_histogram(&meter);
        Self {
            started: Instant::now(),
            histogram,
            attributes: [
                opentelemetry::KeyValue::new("owner.family", family.as_str()),
                opentelemetry::KeyValue::new("stage", stage),
            ],
        }
    }
    fn record(&self, elapsed: Duration) {
        self.histogram
            .record(elapsed.as_secs_f64(), &self.attributes);
    }
}
fn owner_histogram(
    meter: &opentelemetry::metrics::Meter,
) -> opentelemetry::metrics::Histogram<f64> {
    meter
        .f64_histogram(super::metric_declarations::PSE_OWNER_INTERVAL.name)
        .with_description(super::metric_declarations::PSE_OWNER_INTERVAL.population)
        .with_unit(super::metric_declarations::PSE_OWNER_INTERVAL.unit)
        .with_boundaries(
            super::metric_declarations::PSE_OWNER_INTERVAL
                .buckets
                .to_vec(),
        )
        .build()
}
impl Drop for OwnerInterval {
    fn drop(&mut self) {
        self.record(self.started.elapsed());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opentelemetry::{
        metrics::MeterProvider as _,
        trace::{TraceContextExt as _, Tracer as _, TracerProvider as _},
    };
    use opentelemetry_sdk::{
        metrics::{
            PeriodicReader, SdkMeterProvider, Temporality,
            data::{AggregatedMetrics, MetricData, ResourceMetrics},
            exporter::PushMetricExporter,
        },
        trace::{Sampler, SdkTracerProvider},
    };
    #[derive(Debug, Clone, Default)]
    struct Recorded(std::sync::Arc<std::sync::Mutex<Vec<Image>>>);
    #[derive(Debug, Default)]
    struct Image {
        native_attempts: u64,
        histogram: Option<(u64, f64, Vec<u64>, Vec<f64>)>,
    }
    impl PushMetricExporter for Recorded {
        async fn export(
            &self,
            metrics: &ResourceMetrics,
        ) -> opentelemetry_sdk::error::OTelSdkResult {
            let mut image = Image::default();
            for scope in metrics.scope_metrics() {
                for metric in scope.metrics() {
                    match (metric.name(), metric.data()) {
                        (
                            "pse_native_supervisor_events",
                            AggregatedMetrics::U64(MetricData::Sum(sum)),
                        ) => {
                            assert_eq!(sum.temporality(), Temporality::Cumulative);
                            for point in sum.data_points() {
                                let attributes = point.attributes().collect::<Vec<_>>();
                                assert_eq!(
                                    attributes.len(),
                                    1,
                                    "exact IDs must never become dimensions"
                                );
                                if attributes[0].value.as_str() == "attempt" {
                                    image.native_attempts = point.value();
                                }
                            }
                        }
                        (
                            "pse_owner_interval",
                            AggregatedMetrics::F64(MetricData::Histogram(histogram)),
                        ) => {
                            assert_eq!(histogram.temporality(), Temporality::Cumulative);
                            let points = histogram.data_points().collect::<Vec<_>>();
                            assert_eq!(points.len(), 1);
                            let point = points[0];
                            image.histogram = Some((
                                point.count(),
                                point.sum(),
                                point.bucket_counts().collect(),
                                point.bounds().collect(),
                            ));
                        }
                        _ => {}
                    }
                }
            }
            self.0.lock().unwrap().push(image);
            Ok(())
        }
        fn force_flush(&self) -> opentelemetry_sdk::error::OTelSdkResult {
            Ok(())
        }
        fn shutdown_with_timeout(&self, _: Duration) -> opentelemetry_sdk::error::OTelSdkResult {
            Ok(())
        }
        fn temporality(&self) -> Temporality {
            Temporality::Cumulative
        }
    }
    #[test]
    fn operational_metrics_keys_preserve_populations_when_enumeration_reorders() {
        let sample = OperationalMetricsSnapshot {
            source_started_unix_micros: Some(1),
            saturated: false,
            native_attempts: 11,
            native_admitted: 13,
            native_dispatched: 17,
            native_joined: 19,
            native_cancel_requests: 23,
            native_deadline_requests: 29,
            native_supervisors: 31,
            preparation: vec![],
        };
        let native = NativeEvent::ALL
            .into_iter()
            .rev()
            .map(|key| (key.as_str(), key.count(&sample)))
            .collect::<std::collections::BTreeMap<_, _>>();
        assert_eq!(
            native,
            [
                ("attempt", 11),
                ("admitted", 13),
                ("dispatched", 17),
                ("joined", 19),
                ("cancel_requested", 23),
                ("deadline_requested", 29),
            ]
            .into()
        );
        let sample = PreparationMetricsSnapshot {
            stage: PreparationStage::Basis,
            acquisitions: 37,
            retained_reuses: 41,
            flight_waiters: 43,
            producers: 47,
        };
        let preparation = PreparationEvent::ALL
            .into_iter()
            .rev()
            .map(|key| (key.as_str(), key.count(&sample)))
            .collect::<std::collections::BTreeMap<_, _>>();
        assert_eq!(
            preparation,
            [
                ("acquisition", 37),
                ("retained_reuse", 41),
                ("flight_waiter", 43),
                ("producer", 47),
            ]
            .into()
        );
        assert_eq!(serde_json::to_value(sample).unwrap()["stage"], "basis");
        assert!(serde_json::from_str::<PreparationStage>("\"unknown_stage\"").is_err());
    }
    #[tokio::test]
    async fn operational_metrics_shared_flight_counts_one_producer_two_waiters() {
        let before = snapshot().preparation.remove(0);
        let flights = pse_columnar::flight::Flights::<u8, usize, &'static str>::new(2);
        let (release, pending) = tokio::sync::oneshot::channel();
        PreparationStage::Frontier.acquire();
        let mut first = Box::pin(async {
            PreparationStage::Frontier.waiter();
            flights
                .load_owned(1, |_| async {
                    let _interval = PreparationStage::Frontier.producer();
                    pending.await.unwrap();
                    Ok(std::sync::Arc::new(42))
                })
                .await
        });
        assert!(futures_util::poll!(first.as_mut()).is_pending());
        PreparationStage::Frontier.acquire();
        let mut second = Box::pin(async {
            PreparationStage::Frontier.waiter();
            flights
                .load_owned(1, |_| async { panic!("duplicate producer") })
                .await
        });
        assert!(futures_util::poll!(second.as_mut()).is_pending());
        release.send(()).unwrap();
        assert_eq!(*first.await.unwrap(), 42);
        assert_eq!(*second.await.unwrap(), 42);
        let after = snapshot().preparation.remove(0);
        assert_eq!(after.acquisitions - before.acquisitions, 2);
        assert_eq!(after.flight_waiters - before.flight_waiters, 2);
        assert_eq!(after.producers - before.producers, 1);
        assert_eq!(after.retained_reuses, before.retained_reuses);
    }
    #[test]
    fn operational_metrics_cumulative_population_and_histogram_ignore_head_sampling() {
        let original = snapshot().native_attempts;
        for (index, sampler) in [Sampler::AlwaysOff, Sampler::AlwaysOn]
            .into_iter()
            .enumerate()
        {
            let traces = SdkTracerProvider::builder().with_sampler(sampler).build();
            traces.tracer("fixture").in_span("fixture", |context| {
                assert_eq!(context.span().span_context().is_sampled(), index == 1);
                let attempt = NativeJobLifecycle::new();
                attempt.admitted();
                attempt.dispatched();
                attempt.joined();
                attempt.cancelled();
                drop(attempt);
            });
            let recorded = Recorded::default();
            let provider = SdkMeterProvider::builder()
                .with_reader(
                    PeriodicReader::builder(recorded.clone())
                        .with_interval(Duration::from_secs(3600))
                        .build(),
                )
                .build();
            crate::tooling::telemetry::install_runtime_metrics(&provider);
            // Independent known observations include an exact boundary and its next bucket.
            let histogram = owner_histogram(&provider.meter("fixture"));
            for value in [0.000_1, 0.000_11, 601.0] {
                histogram.record(value, &[opentelemetry::KeyValue::new("stage", "fixture")]);
            }
            provider.force_flush().unwrap();
            provider.force_flush().unwrap();
            let images = recorded.0.lock().unwrap();
            assert_eq!(images.len(), 2);
            for image in images.iter() {
                assert_eq!(image.native_attempts, original + index as u64 + 1);
                let (count, sum, buckets, bounds) = image.histogram.as_ref().unwrap();
                assert_eq!(*count, 3);
                assert!((*sum - 601.000_21).abs() < 1e-10);
                assert_eq!(bounds, &[0.000_1, 0.001, 0.01, 0.1, 1.0, 10.0, 60.0, 600.0]);
                assert_eq!(buckets, &[1, 1, 0, 0, 0, 0, 0, 0, 1]);
            }
            drop(images);
            provider.shutdown().unwrap();
            traces.shutdown().unwrap();
        }
        assert_eq!(snapshot().native_supervisors, 0);
    }
}
