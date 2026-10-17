// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Passive projections at native-owner checkpoints. No getter here enters a solver.
use crate::solve::{Event, Metric};
use pse_diagnostics::observation::current;
static GENERATION: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

pub(crate) fn generation() -> u64 {
    GENERATION.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

pub(crate) fn report(report: &mut crate::solve::SolveReport, execution: &crate::solve::Execution, producer: Option<&pse_diagnostics::observation::Producer>) {
    let name = format!("{}.final", format!("{:?}", report.backend).to_lowercase());
    metrics(&name, &report.metrics);
    let event = Event {
        source: Some(crate::solve::EventSource::new("native_owner_return", None, None)
            .with_role(execution.callback_role)),
        phase: name, elapsed: execution.started.elapsed(),
        values: std::collections::BTreeMap::from([
            ("status".into(), Metric::Integer(report.termination.code)),
            ("termination".into(), Metric::Text(report.termination.name.clone())),
            ("category".into(), Metric::Text(format!("{:?}", report.termination.category))),
            ("assurance".into(), Metric::Text(format!("{:?}", report.termination.assurance))),
            ("candidate_present".into(), Metric::Bool(report.candidate.is_some())),
        ]), incumbent: None,
    };
    if let Some(producer) = producer { execution.progress.push_from(event, producer); }
    else { execution.progress.push(event); }
    let (events, dropped) = execution.progress.snapshot();
    report.events = events;
    report.dropped_events = dropped;
}

/// Observe an actual synchronous owner call, including failure and unwind duration.
pub(crate) fn scoped<T>(name: &str, work: impl FnOnce() -> T) -> T {
    let context = current();
    let _phase = context.as_ref().map(|context| context.phase(name));
    if let Some(context) = &context {
        context.count(&format!("{name}.calls"), "count", 1);
    }
    work()
}

/// Distinct native work owner. Its result class describes return/failure only,
/// and its child keeps bounded factor/inner facts separate from outer callbacks.
pub(crate) fn operation<T, E>(name: &str, work: impl FnOnce() -> Result<T, E>) -> Result<T, E> {
    let Some(parent) = current() else {
        return work();
    };
    let operation = parent.child(name, pse_diagnostics::observation::Relationship::Nested);
    let _entered = operation.context().enter();
    let result = scoped(name, work);
    operation.finish(if result.is_ok() {
        pse_diagnostics::observation::Outcome::Completed
    } else {
        pse_diagnostics::observation::Outcome::Failed
    });
    result
}

pub(crate) fn count(name: &str, count: u64) {
    if let Some(context) = current() {
        context.count(name, "count", count);
    }
}

pub(crate) fn integer(name: &str, unit: &str, value: i64) {
    if let Some(context) = current() {
        context.integer(name, unit, value);
    }
}

pub(crate) fn boolean(name: &str, value: bool) {
    if let Some(context) = current() {
        context.boolean(name, value);
    }
}

pub(crate) fn gauge(name: &str, unit: &str, value: f64) {
    if let Some(context) = current() {
        context.gauge(name, unit, value);
    }
}

pub(crate) fn unavailable(name: &str, reason: &str) {
    if let Some(context) = current() {
        context.unavailable(name, reason);
    }
}

/// Preserve supplied method names and values. These are latest checkpoint values,
/// never generic iteration counters or deltas inferred from retained history.
pub(crate) fn metrics(phase: &str, values: &std::collections::BTreeMap<String, Metric>) {
    let Some(context) = current() else { return };
    for (name, value) in values {
        // Callback populations already have prepared producer-scoped facts. A final
        // report is not a second unscoped publication of every per-block key.
        if phase.ends_with(".final") && name.starts_with("callback.") { continue; }
        let key = format!("{phase}.{name}");
        match value {
            Metric::Integer(value) => context.integer(&key, "native", *value),
            Metric::Real(value) => context.gauge(&key, "native", *value),
            Metric::Bool(value) => context.boolean(&key, *value),
            Metric::Unavailable(reason) => context.unavailable(&key, &format!("{reason:?}")),
            Metric::Text(value) => context.text(&key, value),
        }
    }
}

pub(crate) fn event(event: &mut Event, producer: Option<&pse_diagnostics::observation::Producer>) {
    use pse_diagnostics::observation::{RecordAvailability, RecordField, Value};
    let (Some(producer), Some(source)) = (producer, event.source.as_mut()) else { return; };
    fn number(value: f64) -> Value {
        if value.is_nan() { Value::NonFinite(pse_diagnostics::NonFinite::Nan) }
        else if value == f64::INFINITY { Value::NonFinite(pse_diagnostics::NonFinite::PositiveInfinity) }
        else if value == f64::NEG_INFINITY { Value::NonFinite(pse_diagnostics::NonFinite::NegativeInfinity) }
        else { Value::Number(value) }
    }
    fn unit(family: &str, name: &str) -> &'static str {
        match (family, name) {
            ("nonlinear_iteration", "iteration" | "line_search.trials" | "regime.crossings") => "count",
            ("nonlinear_iteration", "obj_value" | "objective.normalized") => "native_objective",
            ("nonlinear_iteration", "inf_pr" | "primal.native") => "native_primal_residual",
            ("nonlinear_iteration", "inf_du" | "dual.native") => "native_dual_residual",
            ("nonlinear_iteration", "alpha_du" | "alpha_pr" | "alpha.dual" | "alpha.primal") => "ratio",
            ("linear_search" | "branch_and_bound", "nodes" | "simplex.iterations" | "ipm.iterations") => "count",
            ("linear_search" | "branch_and_bound", "primal_bound" | "dual_bound") => "original_objective",
            ("linear_search" | "branch_and_bound", "gap") => "ratio",
            ("linear_search", "seconds") | ("conic_iteration", "solve_time") => "seconds",
            ("conic_iteration" | "convex_return", "iterations") => "count",
            ("artificial_time_step", "artificial_time" | "proposed_artificial_step") => "artificial_time",
            ("artificial_time_step", "steps" | "nonlinear_iterations" | "rejections" | "failures") => "count",
            ("dynamic_output" | "dynamic_adjoint" | "dynamic_native_return", "time") => "model_time",
            ("dynamic_output" | "dynamic_adjoint" | "dynamic_native_return", "steps" | "segment") => "count",
            ("root_raw_map", "ordinal") | ("root_evaluation", "evaluation_ordinal") => "evaluation_ordinal",
            ("root_evaluation", "captured_coordinates") => "count",
            ("root_evaluation", "scalar.0.residual" | "scalar.1.residual") => "source_residual",
            ("root_raw_map", "captured_prefix.coordinates") => "count",
            ("root_raw_map", "captured_prefix.max_physical_error_ratio" | "scalar.0.physical_error_ratio" | "scalar.1.physical_error_ratio") => "ratio",
            ("root_raw_map", "scalar.0.residual" | "scalar.1.residual") => "source_coordinate",
            _ => "native_unspecified",
        }
    }
    let mut values: Vec<_> = event.values.iter().map(|(name, metric)| {
        let (value, availability) = match metric {
            Metric::Integer(value) => (Value::Integer(*value), None),
            Metric::Real(value) => (number(*value), None),
            Metric::Bool(value) => (Value::Boolean(*value), None),
            Metric::Text(value) => (Value::Text(value.clone()), None),
            Metric::Unavailable(reason) => (Value::Unavailable(format!("{reason:?}")), Some(match reason {
                crate::solve::UnavailableReason::NotRequested => RecordAvailability::NotRequested,
                crate::solve::UnavailableReason::NotComputed => RecordAvailability::NotSupplied,
                _ => RecordAvailability::Unavailable,
            })),
        };
        RecordField { name: name.clone(), unit: unit(&source.family, name).into(), value, availability }
    }).collect();
    if let Some(incumbent) = &event.incumbent {
        for (name, value, unit) in [
            ("incumbent.objective", Some(incumbent.objective), "original_objective"),
            ("incumbent.dual_bound", incumbent.dual_bound, "original_objective"),
            ("incumbent.gap", incumbent.gap, "native_relative_gap"),
            ("incumbent.native_seconds", Some(incumbent.seconds), "seconds"),
        ] {
            values.push(RecordField { name: name.into(), unit: unit.into(),
                value: value.map_or_else(|| Value::Unavailable("not supplied at this incumbent checkpoint".into()), number),
                availability: value.is_none().then_some(RecordAvailability::NotSupplied) });
        }
        values.push(RecordField { name: "incumbent.nodes".into(), unit: "count".into(), value: Value::Integer(incumbent.nodes), availability: None });
    }
    values.push(RecordField { name: "execution_elapsed_seconds".into(), unit: "seconds".into(), value: number(event.elapsed.as_secs_f64()), availability: None });
    let record = producer.record(&source.family, &event.phase, source.coordinates.as_deref(), source.norm.as_deref(), values);
    producer.publish(&record);
    source.producer = Some(record.producer);
    source.producer_generation = Some(record.generation);
    source.producer_sequence = Some(record.sequence);
    source.observed_at = Some(record.observed_at);
}

/// Project a source-owned structured return without adding any native getter.
#[cfg(any(feature = "diffsol", feature = "idas"))]
pub(crate) fn structured(
    progress: &crate::solve::Progress,
    started: std::time::Instant,
    producer: Option<&pse_diagnostics::observation::Producer>,
    family: &str,
    phase: &str,
    value: &serde_json::Value,
) {
    fn collect(values: &mut std::collections::BTreeMap<String, Metric>, prefix: &str, value: &serde_json::Value) {
        match value {
            serde_json::Value::Object(fields) => for (name, value) in fields {
                collect(values, &if prefix.is_empty() { name.clone() } else { format!("{prefix}.{name}") }, value);
            },
            serde_json::Value::Number(value) => {
                let metric = value.as_i64().map(Metric::Integer)
                    .or_else(|| value.as_u64().map(|value| Metric::Text(value.to_string())))
                    .or_else(|| value.as_f64().map(Metric::Real));
                if let Some(metric) = metric { values.insert(prefix.into(), metric); }
            }
            serde_json::Value::Bool(value) => { values.insert(prefix.into(), Metric::Bool(*value)); }
            serde_json::Value::String(value) => { values.insert(prefix.into(), Metric::Text(value.clone())); }
            serde_json::Value::Null => { values.insert(prefix.into(), Metric::Unavailable(crate::solve::UnavailableReason::NotComputed)); }
            serde_json::Value::Array(_) => { values.insert(prefix.into(), Metric::Unavailable(crate::solve::UnavailableReason::Unsupported)); }
        }
    }
    let mut values = std::collections::BTreeMap::new();
    collect(&mut values, "", value);
    let event = Event {
        source: Some(crate::solve::EventSource::new(family, Some("physical_model"), None)),
        phase: phase.into(), elapsed: started.elapsed(), values, incumbent: None,
    };
    if let Some(producer) = producer { progress.push_from(event, producer); }
    else { progress.push(event); }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solve::{Progress, ProgressTap};
    use pse_diagnostics::observation::{ObservationHub, Policy, Value};
    use std::{
        collections::BTreeMap,
        sync::{
            Arc,
            atomic::{AtomicUsize, Ordering},
        },
        time::Duration,
    };

    fn event_at(step: i64) -> Event {
        Event {
            source: Some(crate::solve::EventSource::new("test_checkpoint", None, None)), phase: "test.step".into(),
            elapsed: Duration::ZERO,
            values: BTreeMap::from([("step".into(), Metric::Integer(step))]),
            incumbent: None,
        }
    }

    #[test]
    fn rolling_progress_preserves_latest_checkpoint_and_exact_loss() {
        let progress = Progress::new(2);
        for step in 0..5 {
            progress.push(event_at(step));
        }
        let (events, dropped) = progress.snapshot();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].values["step"], Metric::Integer(3));
        assert_eq!(events[1].values["step"], Metric::Integer(4));
        assert_eq!(dropped, 3);
        assert_eq!(
            progress.current().unwrap().values["step"],
            Metric::Integer(4)
        );
    }

    #[test]
    fn disabled_history_keeps_current_and_durable_tap_receives_all_events() {
        #[derive(Debug, Default)]
        struct Tap(AtomicUsize);
        impl ProgressTap for Tap {
            fn observe(&self, _: &Event) {
                self.0.fetch_add(1, Ordering::Relaxed);
            }
        }
        let tap = Arc::new(Tap::default());
        let progress = Progress::tapped(0, tap.clone());
        for step in 0..3 {
            progress.push(event_at(step));
        }
        assert_eq!(progress.snapshot().1, 3);
        assert!(progress.snapshot().0.is_empty());
        assert_eq!(
            progress.current().unwrap().values["step"],
            Metric::Integer(2)
        );
        assert_eq!(tap.0.load(Ordering::Relaxed), 3);
    }

    #[test]
    fn source_identity_precedes_ring_loss_and_is_shared_with_tap() {
        #[derive(Debug, Default)]
        struct Tap(std::sync::Mutex<Vec<(u64, u64)>>);
        impl ProgressTap for Tap {
            fn observe(&self, event: &Event) {
                let source = event.source.as_ref().unwrap();
                self.0.lock().unwrap().push((source.generation, source.sequence));
            }
        }
        let tap = Arc::new(Tap::default());
        let progress = Progress::tapped(1, tap.clone());
        for step in 0..3 { progress.push(event_at(step)); }
        let observed = tap.0.lock().unwrap();
        assert_eq!(observed.iter().map(|(_, sequence)| *sequence).collect::<Vec<_>>(), [1, 2, 3]);
        assert!(observed.iter().all(|(generation, _)| *generation == observed[0].0));
        let (events, dropped) = progress.snapshot();
        assert_eq!(dropped, 2);
        assert_eq!(events[0].source.as_ref().unwrap().sequence, 3);
        assert_eq!(progress.current().unwrap().source.as_ref().unwrap().sequence, 3);
    }
    #[test]
    fn refused_oversized_checkpoint_keeps_previous_record_with_visible_source_gap() {
        let progress = Progress::new(1);
        progress.push(event_at(1));
        let mut oversized = event_at(2);
        oversized.values.insert("too_large".into(), Metric::Text("x".repeat(8192)));
        progress.push(oversized);
        let latest = progress.current().unwrap();
        assert_eq!(latest.source.unwrap().sequence, 1);
        assert_eq!(progress.source_identity().1, 2);
        assert_eq!(progress.snapshot().1, 1);
    }
    #[test]
    fn checkpoint_projection_preserves_lossless_counter_and_nonfinite() {
        let hub = ObservationHub::new(Policy::default());
        let operation = hub.start("native.test");
        let _entered = operation.context().enter();
        let mut event = event_at(i64::MAX);
        event
            .values
            .insert("residual".into(), Metric::Real(f64::NAN));
        event
            .values
            .insert("role".into(), Metric::Text("trial".into()));
        Progress::new(0).push(event);
        let snapshot = operation.reader().snapshot(Duration::from_secs(60));
        let record = snapshot.producers.iter().find_map(|p| p.current.as_ref()).unwrap();
        let value = |name: &str| &record.values.iter().find(|field| field.name == name).unwrap().value;
        assert_eq!(*value("step"), Value::Integer(i64::MAX));
        assert!(matches!(value("residual"), Value::NonFinite(_)));
        assert_eq!(*value("role"), Value::Text("trial".into()));
        assert!(snapshot.metrics.iter().all(|m| !m.name.starts_with("test.step.")));

    }
}
