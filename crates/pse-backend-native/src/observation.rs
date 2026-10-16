// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Passive projections at native-owner checkpoints. No getter here enters a solver.
use crate::solve::{Event, Metric};
use pse_diagnostics::observation::current;
static GENERATION: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

pub(crate) fn generation() -> u64 {
    GENERATION.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

pub(crate) fn report(report: &crate::solve::SolveReport) {
    let name = format!("{}.final", format!("{:?}", report.backend).to_lowercase());
    metrics(&name, &report.metrics);
    if let Some(context) = current() {
        context.integer(
            &format!("{name}.status"),
            "native_status",
            report.termination.code,
        );
        context.text(&format!("{name}.termination"), &report.termination.name);
    }
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

pub(crate) fn counter(name: &str, unit: &str, value: u64, generation: u64) {
    if let Some(context) = current() {
        context.count_value(name, unit, value, generation);
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

pub(crate) fn event(event: &Event) {
    if current().is_none() {
        return;
    }
    metrics(&event.phase, &event.values);
    if let (Some(context), Some(incumbent)) = (current(), &event.incumbent) {
        context.gauge(
            &format!("{}.incumbent.objective", event.phase),
            "native_objective",
            incumbent.objective,
        );
        context.integer(
            &format!("{}.incumbent.nodes", event.phase),
            "count",
            incumbent.nodes,
        );
        context.gauge(
            &format!("{}.incumbent.native_seconds", event.phase),
            "seconds",
            incumbent.seconds,
        );
        for (name, value) in [("dual_bound", incumbent.dual_bound), ("gap", incumbent.gap)] {
            if let Some(value) = value {
                context.gauge(
                    &format!("{}.incumbent.{name}", event.phase),
                    "native",
                    value,
                );
            } else {
                context.unavailable(
                    &format!("{}.incumbent.{name}", event.phase),
                    "not supplied by this incumbent checkpoint",
                );
            }
        }
    }
    gauge(
        &format!("{}.checkpoint_seconds", event.phase),
        "seconds",
        event.elapsed.as_secs_f64(),
    );
    count(&format!("{}.checkpoints", event.phase), 1);
}

/// Project a source-owned structured return without adding any native getter.
#[cfg(any(feature = "diffsol", feature = "idas"))]
pub(crate) fn structured(prefix: &str, value: &serde_json::Value) {
    if current().is_none() {
        return;
    }
    match value {
        serde_json::Value::Number(value) => {
            if let (Some(context), Some(value)) = (current(), value.as_i64()) {
                context.integer(prefix, "native", value);
            } else if let (Some(context), Some(value)) = (current(), value.as_u64()) {
                context.count_value(prefix, "native", value, 0);
            } else if let Some(value) = value.as_f64() {
                gauge(prefix, "native", value);
            }
        }
        serde_json::Value::Object(values) => {
            for (name, value) in values {
                structured(&format!("{prefix}.{name}"), value);
            }
        }
        serde_json::Value::Null => unavailable(prefix, "not supplied at this owner checkpoint"),
        serde_json::Value::Bool(value) => {
            if let Some(context) = current() {
                context.boolean(prefix, *value);
            }
        }
        serde_json::Value::String(value) => {
            if let Some(context) = current() {
                context.text(prefix, value);
            }
        }
        _ => {}
    }
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
            phase: "test.step".into(),
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
        assert_eq!(
            snapshot
                .metrics
                .iter()
                .find(|m| m.name == "test.step.step")
                .unwrap()
                .latest,
            Value::Integer(i64::MAX)
        );
        assert!(matches!(
            snapshot
                .metrics
                .iter()
                .find(|m| m.name == "test.step.residual")
                .unwrap()
                .latest,
            Value::NonFinite(_)
        ));
        assert_eq!(
            snapshot
                .metrics
                .iter()
                .find(|m| m.name == "test.step.role")
                .unwrap()
                .latest,
            Value::Text("trial".into())
        );
    }
}
