// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Pure interpretations of already-published observations; no science or I/O.
pub use crate::{NextObservation, StatementKind};
mod performance;
pub use performance::{NumericalHistoryPolicy, explain_numerical_history};

use crate::observation::{MetricSnapshot, OperationId, OperationSnapshot, Value};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Exact observed location supporting an explanatory statement.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EvidenceRef {
    /// Actual diagnostic operation, including its process instance.
    pub operation: OperationId,
    /// Exact performing producer, when the fact is producer-scoped.
    pub producer: Option<crate::observation::ProducerId>,
    /// Source record sequence, independent of a durable append ordinal.
    pub record_sequence: Option<u64>,
    /// Producer metric name, or absent for an operation/phase fact.
    pub metric: Option<String>,
    /// Actual checkpoint time; absence never means now.
    #[schemars(with = "Option<crate::ClosedDuration>")]
    pub observed_at: Option<Duration>,
    /// Metric generation, if this is a metric reference.
    pub generation: Option<u64>,
    /// The referenced value retains unavailable and nonfinite categories.
    pub value: Option<Value>,
    /// Evidence has loss, reset or bounded temporal/detail resolution.
    pub partial: bool,
}
/// A bounded source-linked statement. Consumers render it; they do not reinterpret it.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Statement {
    /// Evidence strength, never an inferred numerical outcome.
    pub kind: StatementKind,
    /// Stable interpretation code, not a severity or convergence score.
    pub code: String,
    /// Plain explanation of the observed meaning and its limits.
    pub message: String,
    /// Actual evidence supporting this statement.
    pub evidence: Vec<EvidenceRef>,
    /// Discriminating optional observations; none are executed by inspection.
    pub next: Vec<NextObservation>,
}
impl Statement {
    /// Construct an operation/coverage statement without inventing a metric.
    pub fn operation(
        snapshot: &OperationSnapshot,
        kind: StatementKind,
        code: &str,
        message: impl Into<String>,
        next: Vec<NextObservation>,
    ) -> Self {
        Self {
            kind,
            code: code.into(),
            message: message.into(),
            evidence: vec![EvidenceRef {
                operation: snapshot.id,
                producer: None,
                record_sequence: None,
                metric: None,
                observed_at: snapshot.observed_at,
                generation: None,
                value: None,
                partial: snapshot.interval_partial
                    || snapshot.coverage.curtailed
                    || snapshot.coverage.stale,
            }],
            next,
        }
    }
    /// Reference one exact producer checkpoint with its actual generation/value.
    pub fn metric(
        snapshot: &OperationSnapshot,
        metric: &MetricSnapshot,
        kind: StatementKind,
        code: &str,
        message: impl Into<String>,
        next: Vec<NextObservation>,
    ) -> Self {
        Self {
            kind,
            code: code.into(),
            message: message.into(),
            evidence: vec![EvidenceRef {
                operation: snapshot.id,
                producer: metric.producer,
                record_sequence: None,
                metric: Some(metric.name.clone()),
                observed_at: Some(metric.observed_at),
                generation: Some(metric.generation),
                value: Some(metric.latest.clone()),
                partial: metric.recent_partial
                    || metric.history_partial
                    || metric.cumulative_partial
                    || snapshot.coverage.curtailed,
            }],
            next,
        }
    }
}
/// Common interpretations over one immutable snapshot. Caller bounds the returned list.
pub fn explain(snapshot: &OperationSnapshot) -> Vec<Statement> {
    let mut statements = Vec::new();
    if !snapshot.coverage.enabled {
        statements.push(Statement::operation(
            snapshot,
            StatementKind::Limitation,
            "collection.disabled",
            "Lightweight collection was disabled; missing observations are not zero work.",
            vec![],
        ));
        return statements;
    }
    if snapshot.coverage.stale {
        statements.push(Statement::operation(snapshot, StatementKind::Limitation, "checkpoint.stale", "The latest producer checkpoint is stale. This does not establish a hang or a known wait.", vec![NextObservation::CpuProfile, NextObservation::AwaitOwnerCheckpoint]));
    }
    if snapshot.interval_partial
        || snapshot.detail_partial
        || snapshot.coverage.curtailed
        || snapshot.coverage.dropped_updates > 0
    {
        statements.push(Statement::operation(snapshot, StatementKind::Limitation, "coverage.partial", "Requested history or detail is partial. Retained subtotals and checkpoints do not establish missing work.", vec![]));
    }
    if !snapshot.phases.is_empty() {
        statements.push(Statement::operation(snapshot, StatementKind::Established, "phase.active", "Owner-published phases identify last observed activity; overlapping phases are not exclusive CPU intervals.", vec![NextObservation::CpuProfile]));
    }
    if snapshot.outcome.is_some() {
        statements.push(Statement::operation(snapshot, StatementKind::Established, "operation.outcome", "The diagnostic owner recorded an operational outcome. Scientific permission, publication and native drain remain separate facts.", vec![]));
    }
    if !snapshot.children.is_empty() {
        statements.push(Statement::operation(snapshot, StatementKind::Established, "relationships.known", "Known children, dependencies and shared producers can be inspected independently. Their inclusive work must not be summed as exclusive time.", vec![NextObservation::InspectRelated]));
    }
    for producer in &snapshot.producers {
        if let Some(record) = &producer.current {
            statements.push(Statement {
                kind: StatementKind::Established, code: "record.current".into(),
                message: format!("{} ({:?}) last published {} / {}. This coherent source checkpoint is not original scientific qualification.", producer.descriptor.name, producer.descriptor.role, record.family, record.phase),
                evidence: vec![EvidenceRef { operation: snapshot.id, producer: Some(record.producer), record_sequence: Some(record.sequence), metric: None, observed_at: Some(record.observed_at), generation: Some(record.generation), value: None, partial: producer.coverage.partial || snapshot.coverage.curtailed }],
                next: vec![],
            });
        }
        if producer.coverage.partial || producer.coverage.history_dropped != 0 {
            statements.push(Statement::operation(snapshot, StatementKind::Limitation, "record.history_partial", "Coherent source records have loss or bounded retention. Missing iterations and continuous convergence cannot be reconstructed.", vec![NextObservation::TimelineCapture]));
        }
    }
    for metric in &snapshot.metrics {
        if metric.resets > 0 {
            statements.push(Statement::metric(snapshot, metric, StatementKind::Limitation, "metric.reset", "This metric has reset/generation boundaries; subtraction across them does not establish work.", vec![]));
        }
        if matches!(
            metric.latest,
            Value::Unavailable(_) | Value::NonFinite(_) | Value::Overflow
        ) {
            continue;
        }
        let points: Vec<_> = metric
            .history
            .iter()
            .filter(|point| !point.predecessor)
            .collect();
        if let (Some(first), Some(last)) = (points.first(), points.last()) {
            if first.first_generation != last.generation {
                continue;
            }
            let equal = first.first_value == last.value;
            statements.push(Statement::metric(snapshot, metric, StatementKind::Established,
                if equal { "checkpoint.endpoints_equal" } else { "checkpoint.changed" },
                if equal { "The first and latest retained checkpoints agree. Coalesced or missing intermediate values do not establish continuous stagnation." }
                else { "The retained first and latest checkpoints differ. Their timestamps and partiality delimit this comparison; the producer defines whether it is trial, accepted or physical progress." },
                if equal { vec![NextObservation::TimelineCapture] } else { vec![] }));
        }
    }
    statements
}
/// Derive inclusive seconds per actual call only from compatible complete cumulative facts.
/// Partial/restarted mismatched facts never produce a misleading mean.
pub fn seconds_per_call(
    snapshot: &OperationSnapshot,
    duration: &MetricSnapshot,
    calls: &MetricSnapshot,
) -> Option<Statement> {
    // Phase time is published on return, whereas native call counters advance at entry.
    // An active matching phase means these populations are not yet comparable.
    let name = duration
        .name
        .strip_prefix("phase.")
        .unwrap_or(&duration.name);
    let stem = name
        .strip_suffix(".nanoseconds")
        .or_else(|| name.strip_suffix(".wall"))
        .or_else(|| name.strip_suffix(".elapsed_nanoseconds"));
    if stem.is_some_and(|stem| snapshot.phases.iter().any(|phase| phase.kind == stem)) {
        return None;
    }
    if duration.producer != calls.producer
        || duration.unit != "nanoseconds"
        || duration.resets != calls.resets
        || duration.generation != calls.generation
        || duration.cumulative_partial
        || calls.cumulative_partial
        || duration.resets > 0
        || snapshot.coverage.dropped_updates > 0
        || snapshot.coverage.curtailed
    {
        return None;
    }
    let (Value::Count(nanoseconds), Value::Count(count)) =
        (&duration.cumulative, &calls.cumulative)
    else {
        return None;
    };
    if *count == 0 {
        return None;
    }
    if let Some(summary) = &duration.duration_summary
        && (summary.overflow || summary.samples != *count)
    {
        return None;
    }
    let seconds = *nanoseconds as f64 / 1_000_000_000.0 / *count as f64;
    let mut statement = Statement::metric(
        snapshot,
        duration,
        StatementKind::Established,
        "work.seconds_per_call",
        format!(
            "Inclusive observed duration per actual call is {seconds:.6} seconds ({count} calls). This is not exclusive CPU time or a performance threshold."
        ),
        vec![NextObservation::CpuProfile],
    );
    statement.evidence.extend(
        Statement::metric(snapshot, calls, StatementKind::Established, "", "", vec![]).evidence,
    );
    Some(statement)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::observation::{ObservationHub, Policy};
    #[test]
    fn inspection_equal_endpoints_never_prove_stagnation() {
        let hub = ObservationHub::new(Policy::default());
        let op = hub.start("native");
        op.context().gauge("residual", "scaled", 1.0);
        op.context().gauge("residual", "scaled", 3.0);
        op.context().gauge("residual", "scaled", 1.0);
        let snapshot = op.reader().snapshot(Duration::from_secs(60));
        let statements = explain(&snapshot);
        assert!(statements.iter().any(|statement| {
            statement.code == "checkpoint.endpoints_equal"
                && statement
                    .message
                    .contains("do not establish continuous stagnation")
        }));
        assert!(
            statements
                .iter()
                .all(|statement| statement.kind != StatementKind::Hypothesis)
        );
    }
    #[test]
    fn inspection_disabled_is_absence_not_zero() {
        let hub = ObservationHub::new(Policy {
            enabled: false,
            ..Policy::default()
        });
        let op = hub.start("off");
        let statements = explain(&op.reader().snapshot(Duration::from_secs(60)));
        assert_eq!(statements.len(), 1);
        assert_eq!(statements[0].code, "collection.disabled");
    }
    #[test]
    fn inspection_reset_refuses_mean_and_keeps_evidence() {
        let hub = ObservationHub::new(Policy::default());
        let op = hub.start("calls");
        op.context()
            .duration("callback.wall", Duration::from_secs(2));
        op.context().count("callback.calls", "count", 2);
        let mut snapshot = op.reader().snapshot(Duration::from_secs(60));
        let duration = snapshot
            .metrics
            .iter()
            .find(|metric| metric.name == "callback.wall")
            .unwrap();
        let calls = snapshot
            .metrics
            .iter()
            .find(|metric| metric.name == "callback.calls")
            .unwrap();
        assert!(seconds_per_call(&snapshot, duration, calls).is_some());
        snapshot
            .metrics
            .iter_mut()
            .find(|metric| metric.name == "callback.calls")
            .unwrap()
            .resets = 1;
        assert!(seconds_per_call(&snapshot, &snapshot.metrics[0], &snapshot.metrics[1]).is_none());
    }
}

/// Select a requested read window from copied retained intervals without asking a producer.
/// Missing transported buckets remain unavailable rather than reconstructed from point values.
pub fn select_window(
    mut snapshot: OperationSnapshot,
    read_at: Duration,
    interval: Duration,
) -> OperationSnapshot {
    let retained_start = snapshot
        .read_at
        .saturating_sub(Duration::from_secs(60))
        .max(snapshot.started_at);
    let old_request = snapshot.requested_interval;
    let old_read = snapshot.read_at;
    let end = snapshot.finished_at.unwrap_or(read_at).min(read_at);
    let start = read_at
        .saturating_sub(interval)
        .max(snapshot.started_at)
        .min(end);
    let empty = read_at.saturating_sub(interval) > end;
    snapshot.read_at = read_at;
    snapshot.requested_interval = interval;
    snapshot.interval_start = start;
    snapshot.interval_end = end;
    let uncovered_suffix = !empty && end > old_read;
    snapshot.interval_partial |= start < retained_start || uncovered_suffix;
    snapshot.coverage.stale |= snapshot.outcome.is_none()
        && read_at.saturating_sub(snapshot.observed_at.unwrap_or(snapshot.started_at))
            > snapshot.stale_after;
    snapshot.apply_summary_expiry(read_at);
    snapshot
        .detail
        .retain(|event| !empty && event.at >= start && event.at <= end);
    for producer in &mut snapshot.producers {
        producer
            .recent
            .retain(|record| !empty && record.observed_at >= start && record.observed_at <= end);
        producer.coverage.partial |= uncovered_suffix || start < retained_start;
    }
    for metric in &mut snapshot.metrics {
        let predecessor = metric
            .history
            .iter()
            .filter(|sample| sample.last_observed_at < start)
            .map(|sample| sample.last_observed_at)
            .max();
        metric.history.retain(|sample| {
            !empty
                && (sample.last_observed_at >= start
                    || Some(sample.last_observed_at) == predecessor)
                && sample.first_observed_at <= end
        });
        for sample in &mut metric.history {
            sample.predecessor = sample.last_observed_at < start;
        }
        metric.history_partial |= snapshot.interval_partial
            || metric
                .history
                .iter()
                .any(|sample| sample.first_observed_at < start || sample.last_observed_at > end);
        if let Some(window) = &metric.counter_window {
            metric.recent_observed_start = None;
            metric.recent_observed_end = None;
            if !empty && start > old_read {
                metric.recent = Value::Unavailable(
                    "requested interval follows the transported checkpoint".into(),
                );
                metric.recent_partial = true;
                continue;
            }
            if !window.observed {
                metric.recent = Value::Unavailable("no observed counter interval".into());
                metric.recent_partial = true;
                continue;
            }
            let mut sum = Some(0_u64);
            let mut partial = window.partial || snapshot.interval_partial;
            for sample in &window.intervals {
                let first = Duration::from_nanos(sample.first_ns);
                let last = Duration::from_nanos(sample.last_ns);
                if empty
                    || last < start
                    || first > end
                    || (window.duration && (last <= start || first >= end))
                {
                    continue;
                }
                partial |= first < start || last > end || sample.reset;
                metric.recent_observed_start = Some(
                    metric
                        .recent_observed_start
                        .map_or(first, |old| old.min(first)),
                );
                metric.recent_observed_end =
                    Some(metric.recent_observed_end.map_or(last, |old| old.max(last)));
                sum = sum
                    .zip(sample.delta)
                    .and_then(|(sum, delta)| sum.checked_add(delta));
            }
            metric.recent = sum.map_or(Value::Overflow, Value::Count);
            metric.recent_partial = partial;
        } else if old_request != interval || old_read != read_at {
            // Nonadditive fields stay nonadditive; curtailed counter transport has no buckets.
            metric.recent = Value::Unavailable(
                "no retained additive intervals for this requested window".into(),
            );
            metric.recent_partial = true;
            metric.recent_observed_start = None;
            metric.recent_observed_end = None;
        }
    }
    snapshot
}

#[cfg(test)]
mod window_tests {
    use super::*;
    use crate::observation::{Clock, ObservationHub, Policy};
    use std::sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    };
    struct TestClock(AtomicU64);
    impl Clock for TestClock {
        fn now(&self) -> Duration {
            Duration::from_secs(self.0.load(Ordering::Relaxed))
        }
    }
    #[test]
    fn inspection_frozen_checkpoint_never_reports_unobserved_suffix_as_zero() {
        let clock = Arc::new(TestClock(AtomicU64::new(0)));
        let hub = ObservationHub::with_clock(Policy::default(), clock.clone());
        let op = hub.start("busy");
        clock.0.store(10, Ordering::Relaxed);
        op.context().count("calls", "count", 3);
        let frozen = op.reader().snapshot(Duration::from_secs(60));
        let later = select_window(
            frozen.clone(),
            Duration::from_secs(20),
            Duration::from_secs(5),
        );
        assert!(later.interval_partial);
        assert!(later.metrics[0].recent_partial);
        assert!(matches!(later.metrics[0].recent, Value::Unavailable(_)));
        let overlap = select_window(frozen, Duration::from_secs(20), Duration::from_secs(15));
        assert_eq!(overlap.metrics[0].recent, Value::Count(3));
        assert!(overlap.metrics[0].recent_partial);
    }
    #[test]
    fn inspection_remote_freshness_uses_the_producers_policy() {
        let clock = Arc::new(TestClock(AtomicU64::new(0)));
        let hub = ObservationHub::with_clock(
            Policy {
                stale_after: Duration::from_secs(60),
                ..Policy::default()
            },
            clock.clone(),
        );
        let op = hub.start("slow-checkpoints");
        op.context().count("calls", "count", 1);
        let frozen = op.reader().snapshot(Duration::from_secs(60));
        assert!(
            !select_window(
                frozen.clone(),
                Duration::from_secs(20),
                Duration::from_secs(5)
            )
            .coverage
            .stale
        );
        assert!(
            select_window(frozen, Duration::from_secs(61), Duration::from_secs(5))
                .coverage
                .stale
        );
    }
    #[test]
    fn inspection_started_call_population_cannot_divide_completed_phase_time() {
        let clock = Arc::new(TestClock(AtomicU64::new(0)));
        let hub = ObservationHub::with_clock(Policy::default(), clock.clone());
        let op = hub.start("native");
        op.context().count("callback.calls", "calls", 1);
        let first = op.context().phase("callback");
        clock.0.store(2, Ordering::Relaxed);
        drop(first);
        op.context().count("callback.calls", "calls", 1);
        let second = op.context().phase("callback");
        let active = op.reader().snapshot(Duration::from_secs(60));
        let mean = |snapshot: &OperationSnapshot| {
            seconds_per_call(
                snapshot,
                snapshot
                    .metrics
                    .iter()
                    .find(|metric| metric.name == "phase.callback.nanoseconds")
                    .unwrap(),
                snapshot
                    .metrics
                    .iter()
                    .find(|metric| metric.name == "callback.calls")
                    .unwrap(),
            )
        };
        assert!(mean(&active).is_none());
        let mut compact = active.clone();
        compact.compact();
        assert!(compact.phases.is_empty());
        assert!(compact.coverage.curtailed);
        assert!(mean(&compact).is_none());
        clock.0.store(6, Ordering::Relaxed);
        drop(second);
        assert!(
            mean(&op.reader().snapshot(Duration::from_secs(60)))
                .unwrap()
                .message
                .contains("3.000000")
        );
    }
    #[test]
    fn inspection_remote_windows_use_actual_counter_intervals_without_prorating() {
        let clock = Arc::new(TestClock(AtomicU64::new(0)));
        let hub = ObservationHub::with_clock(Policy::default(), clock.clone());
        let op = hub.start("actual-counts");
        clock.0.store(10, Ordering::Relaxed);
        op.context().count("calls", "count", 2);
        clock.0.store(20, Ordering::Relaxed);
        op.context().count("calls", "count", 3);
        op.context().duration("work", Duration::from_secs(10));
        let snapshot = op.reader().snapshot(Duration::from_secs(60));
        let narrowed = select_window(
            snapshot.clone(),
            Duration::from_secs(20),
            Duration::from_secs(9),
        );
        assert_eq!(
            narrowed
                .metrics
                .iter()
                .find(|m| m.name == "calls")
                .unwrap()
                .recent,
            Value::Count(3)
        );
        assert_eq!(
            narrowed
                .metrics
                .iter()
                .find(|m| m.name == "work")
                .unwrap()
                .recent,
            Value::Count(9_000_000_000)
        );
        assert!(
            !narrowed
                .metrics
                .iter()
                .find(|m| m.name == "work")
                .unwrap()
                .recent_partial
        );
        let boundary = select_window(
            snapshot,
            Duration::from_millis(20500),
            Duration::from_secs(9),
        );
        let work = boundary.metrics.iter().find(|m| m.name == "work").unwrap();
        assert_eq!(work.recent, Value::Count(9_000_000_000));
        assert!(
            work.recent_partial,
            "intersecting bucket is retained whole with explicit partiality"
        );
    }
    #[test]
    fn inspection_curtailed_counter_transport_never_invents_zero() {
        let hub = ObservationHub::new(Policy::default());
        let op = hub.start("curtailed");
        op.context().count("calls", "count", 7);
        let mut snapshot = op.reader().snapshot(Duration::from_secs(60));
        snapshot.metrics[0].counter_window = None;
        let at = snapshot.read_at;
        let narrowed = select_window(snapshot, at, Duration::from_secs(1));
        assert!(matches!(narrowed.metrics[0].recent, Value::Unavailable(_)));
        assert!(narrowed.metrics[0].recent_partial);
    }
    #[test]
    fn inspection_phase_detail_retains_exact_parallel_tokens_and_ends() {
        let clock = Arc::new(TestClock(AtomicU64::new(0)));
        let hub = ObservationHub::with_clock(Policy::default(), clock.clone());
        let op = hub.start("parallel");
        let first = op.context().phase("work");
        let second = op.context().phase("work");
        clock.0.store(5, Ordering::Relaxed);
        drop(first);
        clock.0.store(7, Ordering::Relaxed);
        drop(second);
        let snapshot = op.reader().snapshot(Duration::from_secs(60));
        let ends: Vec<_> = snapshot
            .detail
            .iter()
            .filter_map(|event| event.phase.as_ref())
            .filter(|phase| phase.finished_at.is_some())
            .collect();
        assert_eq!(ends.len(), 2);
        assert_ne!(ends[0].token, ends[1].token);
        assert_eq!(ends[0].finished_at, Some(Duration::from_secs(5)));
        assert_eq!(ends[1].finished_at, Some(Duration::from_secs(7)));
    }
}
