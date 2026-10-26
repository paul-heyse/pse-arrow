// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! TOOL06 inquiry only: no production registration or substrate replacement.
//!
//! Exact API sources: tracing-subscriber 0.3.23 src/layer/{mod,context}.rs and
//! src/registry/extensions.rs; tracing 0.1.44 src/instrument.rs (installed pinned
//! sources). Documentation: https://docs.rs/tracing-subscriber/0.3.23/tracing_subscriber/layer/trait.Layer.html
//! and https://docs.rs/tracing/0.1.44/tracing/instrument/trait.WithSubscriber.html.
//! The Layer copies only actual emitted metadata and scalar field counts. It never
//! turns an observed span into a scientific phase, producer, permission or metric.
#![cfg(all(feature = "solver-ipopt", feature = "solver-highs"))]

use super::{RunReport, fitting::regression, modeling::sensitivity_tests};
use pse_backend_native::solve::{Backend, HessianMode, SolverSelection};
use pse_diagnostics::observation::{
    self, ObservationHub, OperationId, OperationReader, Outcome, Policy,
};
use pse_relations::{columnar::RelationRow, generated::runtime::solve_variables};
use std::{
    collections::{BTreeSet, HashSet, VecDeque},
    fmt,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, AtomicUsize, Ordering},
    },
    time::Duration,
};
use tracing::{
    Dispatch, Event, Subscriber,
    field::{Field, Visit},
    instrument::WithSubscriber,
    span::{Attributes, Id, Record},
};
use tracing_subscriber::{Layer, layer::Context, prelude::*, registry::LookupSpan};

const ACTIVE_SPANS: usize = 64;
const CLOSED_PREFIX: usize = 256;

#[derive(Clone, Copy, Debug, Default)]
struct Fields {
    numeric: u64,
    other: u64,
    explicit_phase_fields: u64,
}
impl Fields {
    fn seen(&mut self, field: &Field, numeric: bool) {
        if numeric {
            self.numeric = self.numeric.saturating_add(1);
        } else {
            self.other = self.other.saturating_add(1);
        }
        if field.name() == "phase" {
            self.explicit_phase_fields = self.explicit_phase_fields.saturating_add(1);
        }
    }
    fn add(&mut self, other: Self) {
        self.numeric = self.numeric.saturating_add(other.numeric);
        self.other = self.other.saturating_add(other.other);
        self.explicit_phase_fields = self
            .explicit_phase_fields
            .saturating_add(other.explicit_phase_fields);
    }
}
impl Visit for Fields {
    fn record_f64(&mut self, field: &Field, _value: f64) {
        self.seen(field, true);
    }
    fn record_i64(&mut self, field: &Field, _value: i64) {
        self.seen(field, true);
    }
    fn record_u64(&mut self, field: &Field, _value: u64) {
        self.seen(field, true);
    }
    fn record_debug(&mut self, field: &Field, _value: &dyn fmt::Debug) {
        self.seen(field, false);
    }
}

#[derive(Clone, Debug)]
struct ClosedSpan {
    name: &'static str,
    target: &'static str,
    parent: Option<u64>,
    observation: Option<OperationId>,
    fields: Fields,
    events: u64,
    native_entries: u64,
}
#[derive(Debug, Default)]
struct Counts {
    active: AtomicUsize,
    peak: AtomicUsize,
    admitted: AtomicU64,
    dropped: AtomicU64,
    refused: AtomicU64,
    closed: AtomicU64,
    events: AtomicU64,
    attributed_events: AtomicU64,
    native_callbacks: AtomicU64,
    native_worker_spans: AtomicU64,
    datafusion_callbacks: AtomicU64,
    prefix: Mutex<Vec<ClosedSpan>>,
}
impl Counts {
    fn bounded() -> Arc<Self> {
        Arc::new(Self {
            prefix: Mutex::new(Vec::with_capacity(CLOSED_PREFIX)),
            ..Self::default()
        })
    }
    fn admit(&self) -> bool {
        if self
            .active
            .try_update(Ordering::AcqRel, Ordering::Acquire, |active| {
                (active < ACTIVE_SPANS).then_some(active + 1)
            })
            .is_err()
        {
            self.refused.fetch_add(1, Ordering::Relaxed);
            return false;
        }
        self.admitted.fetch_add(1, Ordering::Relaxed);
        self.peak
            .fetch_max(self.active.load(Ordering::Acquire), Ordering::Relaxed);
        true
    }
    fn callback(&self, target: &str) {
        if std::thread::current().name() == Some("pse-math") {
            self.native_callbacks.fetch_add(1, Ordering::Relaxed);
        }
        if target.starts_with("datafusion") {
            self.datafusion_callbacks.fetch_add(1, Ordering::Relaxed);
        }
    }
    fn assert_drained(&self) {
        assert_eq!(self.active.load(Ordering::Acquire), 0);
        assert_eq!(
            self.admitted.load(Ordering::Acquire),
            self.dropped.load(Ordering::Acquire)
        );
        assert_eq!(
            self.admitted.load(Ordering::Acquire),
            self.closed.load(Ordering::Acquire)
        );
        assert!(self.peak.load(Ordering::Acquire) <= ACTIVE_SPANS);
        assert!(self.prefix.lock().unwrap().len() <= CLOSED_PREFIX);
    }
}
/// A unique extension type prevents collisions with other Layers. The drop owner
/// retires its slot even when the subscriber/registry is destroyed during unwind.
struct SpanAggregate {
    counts: Arc<Counts>,
    sample: ClosedSpan,
}
impl Drop for SpanAggregate {
    fn drop(&mut self) {
        self.counts.dropped.fetch_add(1, Ordering::Release);
        self.counts.active.fetch_sub(1, Ordering::Release);
    }
}
#[derive(Clone)]
struct CandidateLayer(Arc<Counts>);
impl<S> Layer<S> for CandidateLayer
where
    S: Subscriber + for<'lookup> LookupSpan<'lookup>,
{
    fn on_new_span(&self, attributes: &Attributes<'_>, id: &Id, context: Context<'_, S>) {
        self.0.callback(attributes.metadata().target());
        if attributes.metadata().name() == "native.worker" {
            self.0.native_worker_spans.fetch_add(1, Ordering::Relaxed);
        }
        if !self.0.admit() {
            return;
        }
        let span = context.span(id).unwrap();
        let mut fields = Fields::default();
        attributes.record(&mut fields);
        let sample = ClosedSpan {
            name: attributes.metadata().name(),
            target: attributes.metadata().target(),
            parent: span.parent().map(|parent| parent.id().into_u64()),
            observation: observation::current().map(|context| context.id()),
            fields,
            events: 0,
            native_entries: 0,
        };
        // No retained event strings, scientific payloads or unbounded per-span maps.
        span.extensions_mut().insert(SpanAggregate {
            counts: self.0.clone(),
            sample,
        });
    }
    fn on_record(&self, id: &Id, values: &Record<'_>, context: Context<'_, S>) {
        if let Some(span) = context.span(id) {
            let mut fields = Fields::default();
            values.record(&mut fields);
            if let Some(aggregate) = span.extensions_mut().get_mut::<SpanAggregate>() {
                aggregate.sample.fields.add(fields);
            }
        }
    }
    fn on_enter(&self, id: &Id, context: Context<'_, S>) {
        if let Some(span) = context.span(id) {
            self.0.callback(span.metadata().target());
            if std::thread::current().name() == Some("pse-math")
                && let Some(aggregate) = span.extensions_mut().get_mut::<SpanAggregate>()
            {
                aggregate.sample.native_entries = aggregate.sample.native_entries.saturating_add(1);
            }
        }
    }
    fn on_event(&self, event: &Event<'_>, context: Context<'_, S>) {
        self.0.events.fetch_add(1, Ordering::Relaxed);
        self.0.callback(event.metadata().target());
        if observation::current().is_some() {
            self.0.attributed_events.fetch_add(1, Ordering::Relaxed);
        }
        if let Some(span) = context.event_span(event) {
            let mut fields = Fields::default();
            event.record(&mut fields);
            if let Some(aggregate) = span.extensions_mut().get_mut::<SpanAggregate>() {
                aggregate.sample.events = aggregate.sample.events.saturating_add(1);
                aggregate.sample.fields.add(fields);
            }
        }
    }
    fn on_close(&self, id: Id, context: Context<'_, S>) {
        if let Some(span) = context.span(&id)
            && let Some(aggregate) = span.extensions_mut().remove::<SpanAggregate>()
        {
            self.0.closed.fetch_add(1, Ordering::Release);
            let mut prefix = self.0.prefix.lock().unwrap();
            if prefix.len() < CLOSED_PREFIX {
                prefix.push(aggregate.sample.clone());
            } else {
                self.0.refused.fetch_add(1, Ordering::Relaxed);
            }
            drop(aggregate);
        }
    }
}

#[derive(Debug, Default)]
struct HubCensus {
    operations: usize,
    producers: usize,
    phases: BTreeSet<String>,
    metrics: usize,
    unavailable_children: usize,
}
fn hub_census(hub: &ObservationHub, root: OperationReader) -> HubCensus {
    let mut result = HubCensus::default();
    let mut pending = VecDeque::from([root]);
    let mut seen = HashSet::new();
    while let Some(reader) = pending.pop_front() {
        let snapshot = reader.snapshot(Duration::from_secs(60));
        if !seen.insert(snapshot.id) {
            continue;
        }
        assert!(
            seen.len() <= 128,
            "inquiry exceeded the actual bounded hub catalogue"
        );
        assert!(
            snapshot.phases.is_empty(),
            "a joined workflow retained an active phase"
        );
        result.operations += 1;
        result.producers += snapshot.producers.len();
        result.metrics += snapshot.metrics.len();
        for metric in snapshot.metrics {
            if let Some(phase) = metric
                .name
                .strip_prefix("phase.")
                .and_then(|name| name.strip_suffix(".nanoseconds"))
            {
                result.phases.insert(phase.to_owned());
            }
        }
        for child in snapshot.children {
            if let Some(reader) = hub.reader(child.id) {
                pending.push_back(reader);
            } else {
                result.unavailable_children += 1;
            }
        }
    }
    result
}

#[derive(Clone, Copy, Debug)]
enum Mode {
    Absent,
    CurrentHub,
    HubAndLayer,
}
#[derive(Debug, PartialEq, Eq)]
struct Science {
    steady_x_bits: u64,
    fit_bits: Vec<u64>,
    covariance_bits: Vec<u64>,
}
async fn actual_steady_and_fit() -> Science {
    let (steady, ids) = sensitivity_tests::solve(
        sensitivity_tests::QUADRATIC,
        SolverSelection::Explicit(Backend::Ipopt),
    )
    .await;
    assert!(steady.usable());
    let rows =
        solve_variables::Row::rows(&steady.table("runtime.solve_variables").unwrap()).unwrap();
    let x = rows
        .iter()
        .find(|row| row.symbol_id == ids[2])
        .unwrap()
        .value
        .unwrap();
    assert!(x.is_finite());
    let fit_runtime = crate::workflow::tests::runtime();
    let package = regression::package(&fit_runtime, regression::declared(), [1.; 4]).await;
    let fit = regression::fit(
        &fit_runtime.execution_services(),
        &package,
        regression::profile(HessianMode::GaussNewton, None),
    )
    .await;
    assert!(fit.usable());
    let RunReport::Fit(report) = fit.report().unwrap() else {
        panic!("actual fit report");
    };
    assert!(report.validation_error.is_none());
    let fit_values = report.candidate.as_ref().unwrap();
    let covariance = report.covariance.as_ref().unwrap().values.as_ref().unwrap();
    assert!(
        fit_values
            .iter()
            .chain(covariance)
            .all(|value| value.is_finite())
    );
    Science {
        steady_x_bits: x.to_bits(),
        fit_bits: fit_values.iter().map(|value| value.to_bits()).collect(),
        covariance_bits: covariance.iter().map(|value| value.to_bits()).collect(),
    }
}

#[tokio::test]
async fn observation_layer_inquiry_actual_steady_and_fit_preserve_science_but_expose_phase_gap() {
    let mut scientific_reference = None;
    for mode in [Mode::Absent, Mode::CurrentHub, Mode::HubAndLayer] {
        let hub = ObservationHub::new(Policy {
            enabled: !matches!(mode, Mode::Absent),
            ..Default::default()
        });
        let operation = hub.start("inquiry.actual-steady-and-fit");
        let reader = operation.reader();
        let counts = Counts::bounded();
        let dispatch = if matches!(mode, Mode::HubAndLayer) {
            Dispatch::new(tracing_subscriber::registry().with(CandidateLayer(counts.clone())))
        } else {
            Dispatch::new(tracing::subscriber::NoSubscriber::default())
        };
        let context = (!matches!(mode, Mode::Absent)).then(|| operation.context());
        let science = observation::scope(context, actual_steady_and_fit())
            .with_subscriber(dispatch.clone())
            .await;
        operation.finish(Outcome::Completed);
        let census = hub_census(&hub, reader);
        if let Some(reference) = &scientific_reference {
            assert_eq!(
                &science, reference,
                "observation mode changed original scientific results: {mode:?}"
            );
        } else {
            scientific_reference = Some(science);
        }
        counts.assert_drained();
        if matches!(mode, Mode::Absent) {
            assert_eq!(census.metrics, 0);
        } else {
            assert!(
                census.phases.contains("native.worker"),
                "actual native work was not observed: {census:?}"
            );
        }
        if matches!(mode, Mode::HubAndLayer) {
            let prefix = counts.prefix.lock().unwrap();
            assert!(
                counts.admitted.load(Ordering::Acquire) > 0,
                "the actual function-assembly span did not reach the candidate"
            );
            assert!(
                prefix.iter().any(|span| span.observation.is_some()),
                "actual span had no source observation attribution"
            );
            // The candidate consumes emitted spans, not ObservationContext's private
            // phase lifecycle. This observed gap blocks replacement without new wiring.
            assert_eq!(counts.native_worker_spans.load(Ordering::Acquire), 0);
            assert!(!prefix.iter().any(|span| span.name == "native.worker"));
            let attributed = prefix
                .iter()
                .filter(|span| span.observation.is_some())
                .count();
            let parented = prefix.iter().filter(|span| span.parent.is_some()).count();
            let field_count: u64 = prefix
                .iter()
                .map(|span| span.fields.numeric + span.fields.other)
                .sum();
            let phase_fields: u64 = prefix
                .iter()
                .map(|span| span.fields.explicit_phase_fields)
                .sum();
            let retained_events: u64 = prefix.iter().map(|span| span.events).sum();
            let native_entries: u64 = prefix.iter().map(|span| span.native_entries).sum();
            let datafusion_spans = prefix
                .iter()
                .filter(|span| span.target.starts_with("datafusion"))
                .count();
            eprintln!(
                "TOOL06 actual {mode:?}: hub_operations={} hub_metrics={} hub_producers={} hub_phase_names={:?}; layer_spans={} attributed={} parented={} scalar_fields={} explicit_phase_fields={} retained_events={} events={} attributed_events={} native_entries={} native_callbacks={} datafusion_spans={} datafusion_callbacks={} refused={}",
                census.operations,
                census.metrics,
                census.producers,
                census.phases,
                prefix.len(),
                attributed,
                parented,
                field_count,
                phase_fields,
                retained_events,
                counts.events.load(Ordering::Acquire),
                counts.attributed_events.load(Ordering::Acquire),
                native_entries,
                counts.native_callbacks.load(Ordering::Acquire),
                datafusion_spans,
                counts.datafusion_callbacks.load(Ordering::Acquire),
                counts.refused.load(Ordering::Acquire)
            );
        }
        drop(dispatch);
    }
}

#[tokio::test]
async fn observation_layer_inquiry_bounds_state_and_retires_on_future_drop_and_panic() {
    let counts = Counts::bounded();
    let weak = Arc::downgrade(&counts);
    let dispatch =
        Dispatch::new(tracing_subscriber::registry().with(CandidateLayer(counts.clone())));
    let spans = tracing::dispatcher::with_default(&dispatch, || {
        (0..ACTIVE_SPANS + 8)
            .map(|index| tracing::info_span!("inquiry.capacity-control", index))
            .collect::<Vec<_>>()
    });
    assert_eq!(counts.active.load(Ordering::Acquire), ACTIVE_SPANS);
    assert_eq!(counts.refused.load(Ordering::Acquire), 8);
    drop(spans);
    counts.assert_drained();
    let future = async {
        let _span = tracing::info_span!("inquiry.pending-future-control");
        std::future::pending::<()>().await;
    }
    .with_subscriber(dispatch.clone());
    let mut future = Box::pin(future);
    assert!(futures_util::poll!(future.as_mut()).is_pending());
    assert_eq!(counts.active.load(Ordering::Acquire), 1);
    drop(future);
    counts.assert_drained();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        tracing::dispatcher::with_default(&dispatch, || {
            let span = tracing::info_span!("inquiry.panic-control");
            let _entered = span.enter();
            panic!("controlled tracing-layer unwind");
        })
    }));
    assert!(result.is_err());
    counts.assert_drained();
    drop(dispatch);
    drop(counts);
    assert!(
        weak.upgrade().is_none(),
        "the candidate retained observation state after subscriber release"
    );
}
