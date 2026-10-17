// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Actual KLU owner counters, not proxies from Jacobian callbacks.
use super::*;
pub(super) unsafe fn snapshot(solver: ffi::SUNLinearSolver) -> Option<ffi::SUNLinearSolver_KLUWork> {
    if solver.is_null() { return None; }
    let mut work = std::mem::MaybeUninit::uninit();
    // SAFETY: caller owns the live KLU solver on this synchronous session thread;
    // the getter initializes the complete output only on success and initiates no work.
    let status = unsafe { ffi::SUNLinSol_KLUGetWork(solver, work.as_mut_ptr()) };
    if status != 0 { return None; }
    // SAFETY: successful getter wrote every scalar in the work record.
    Some(unsafe { work.assume_init() })
}
fn delta(before: Option<&ffi::SUNLinearSolver_KLUWork>, after: Option<&ffi::SUNLinearSolver_KLUWork>) -> std::collections::BTreeMap<String, Metric> {
    let valid = before.zip(after).filter(|(before, after)| before.overflow == 0 && after.overflow == 0);
    let mut values = std::collections::BTreeMap::from([
        ("backend".into(), Metric::Text("kinsol".into())),
        ("linear_solver".into(), Metric::Text("klu".into())),
        ("counter_overflow".into(), Metric::Bool(before.zip(after).is_some_and(|(a,b)| a.overflow != 0 || b.overflow != 0))),
        ("source_available".into(), Metric::Bool(valid.is_some())),
    ]);
    values.insert("setup.issued".into(), valid.and_then(|(a,b)| b.setup_issued.checked_sub(a.setup_issued)).and_then(|value| i64::try_from(value).ok()).map_or(Metric::Unavailable(UnavailableReason::Unknown), Metric::Integer));
    values.insert("setup.completed".into(), valid.and_then(|(a,b)| b.setup_completed.checked_sub(a.setup_completed)).and_then(|value| i64::try_from(value).ok()).map_or(Metric::Unavailable(UnavailableReason::Unknown), Metric::Integer));
    values.insert("symbolic.issued".into(), valid.and_then(|(a,b)| b.symbolic_issued.checked_sub(a.symbolic_issued)).and_then(|value| i64::try_from(value).ok()).map_or(Metric::Unavailable(UnavailableReason::Unknown), Metric::Integer));
    values.insert("symbolic.completed".into(), valid.and_then(|(a,b)| b.symbolic_completed.checked_sub(a.symbolic_completed)).and_then(|value| i64::try_from(value).ok()).map_or(Metric::Unavailable(UnavailableReason::Unknown), Metric::Integer));
    values.insert("numeric_factor.issued".into(), valid.and_then(|(a,b)| b.numeric_factor_issued.checked_sub(a.numeric_factor_issued)).and_then(|value| i64::try_from(value).ok()).map_or(Metric::Unavailable(UnavailableReason::Unknown), Metric::Integer));
    values.insert("numeric_factor.completed".into(), valid.and_then(|(a,b)| b.numeric_factor_completed.checked_sub(a.numeric_factor_completed)).and_then(|value| i64::try_from(value).ok()).map_or(Metric::Unavailable(UnavailableReason::Unknown), Metric::Integer));
    values.insert("refactor.issued".into(), valid.and_then(|(a,b)| b.refactor_issued.checked_sub(a.refactor_issued)).and_then(|value| i64::try_from(value).ok()).map_or(Metric::Unavailable(UnavailableReason::Unknown), Metric::Integer));
    values.insert("refactor.completed".into(), valid.and_then(|(a,b)| b.refactor_completed.checked_sub(a.refactor_completed)).and_then(|value| i64::try_from(value).ok()).map_or(Metric::Unavailable(UnavailableReason::Unknown), Metric::Integer));
    values.insert("rcond.issued".into(), valid.and_then(|(a,b)| b.rcond_issued.checked_sub(a.rcond_issued)).and_then(|value| i64::try_from(value).ok()).map_or(Metric::Unavailable(UnavailableReason::Unknown), Metric::Integer));
    values.insert("rcond.completed".into(), valid.and_then(|(a,b)| b.rcond_completed.checked_sub(a.rcond_completed)).and_then(|value| i64::try_from(value).ok()).map_or(Metric::Unavailable(UnavailableReason::Unknown), Metric::Integer));
    values.insert("condest.issued".into(), valid.and_then(|(a,b)| b.condest_issued.checked_sub(a.condest_issued)).and_then(|value| i64::try_from(value).ok()).map_or(Metric::Unavailable(UnavailableReason::Unknown), Metric::Integer));
    values.insert("condest.completed".into(), valid.and_then(|(a,b)| b.condest_completed.checked_sub(a.condest_completed)).and_then(|value| i64::try_from(value).ok()).map_or(Metric::Unavailable(UnavailableReason::Unknown), Metric::Integer));
    values.insert("solve.issued".into(), valid.and_then(|(a,b)| b.solve_issued.checked_sub(a.solve_issued)).and_then(|value| i64::try_from(value).ok()).map_or(Metric::Unavailable(UnavailableReason::Unknown), Metric::Integer));
    values.insert("solve.completed".into(), valid.and_then(|(a,b)| b.solve_completed.checked_sub(a.solve_completed)).and_then(|value| i64::try_from(value).ok()).map_or(Metric::Unavailable(UnavailableReason::Unknown), Metric::Integer));
    values
}
pub(super) fn current(context: &Context) {
    let Some(producer) = &context.klu_producer else { return; };
    if !producer.detail_selected("native_linear_work.v1") { return; }
    // SAFETY: Context's borrowed handle is installed by its owning Session and
    // callbacks end before Session destroys any native object.
    let after = unsafe { snapshot(context.klu) };
    let values = delta(context.klu_baseline.as_ref(), after.as_ref());
    context.state.execution.progress.push_from(Event {
        source: Some(EventSource::new("native_linear_work", Some("native_linear_system"), None)
            .with_role(pse_diagnostics::observation::ProducerRole::NativeFactor)),
        phase: "kinsol.klu.checkpoint".into(), elapsed: context.state.execution.started.elapsed(),
        values, incumbent: None,
    }, producer);
}
pub(super) fn finish(context: &Context, report: &mut SolveReport) {
    if context.klu.is_null() { return; }
    // SAFETY: the Session still owns all its live native objects at this safe return.
    let after = unsafe { snapshot(context.klu) };
    for (key, value) in delta(context.klu_baseline.as_ref(), after.as_ref()) {
        report.metrics.insert(format!("klu.{key}"), value);
    }
    current(context);
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn actual_linear_delta_refuses_reset_overflow_and_absent_source() {
        // SAFETY: this C record consists only of integer counters and a flag.
        let mut before: ffi::SUNLinearSolver_KLUWork = unsafe { std::mem::zeroed() };
        let mut after = before;
        after.numeric_factor_issued = 2; after.numeric_factor_completed = 1;
        let observed = delta(Some(&before),Some(&after));
        assert_eq!(observed["numeric_factor.issued"],Metric::Integer(2));
        assert_eq!(observed["numeric_factor.completed"],Metric::Integer(1));
        before.numeric_factor_issued = 3;
        assert!(matches!(delta(Some(&before),Some(&after))["numeric_factor.issued"],Metric::Unavailable(_)));
        after.overflow = 1;
        assert!(matches!(delta(Some(&before),Some(&after))["numeric_factor.completed"],Metric::Unavailable(_)));
        assert!(matches!(delta(None,None)["setup.issued"],Metric::Unavailable(_)));
    }
}
