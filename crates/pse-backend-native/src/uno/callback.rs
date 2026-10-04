// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
#![allow(
    unsafe_code,
    reason = "contained callbacks behind worker-owned C++ bridges; all pointer extents and activation lifetimes are checked"
)]
//! Shared dispatch for the shielded Uno and Ipopt sequence boundaries.
//! Retained foreign objects point to an owned slot, never to a retained borrowed oracle.
use crate::{
    NlpOracle, ProblemError,
    callback::CallbackState,
    nlp_pattern::Pattern,
    solve::{Execution, Termination},
};
use std::{
    cell::Cell,
    ffi::c_void,
    panic::{AssertUnwindSafe, catch_unwind},
    ptr,
};

pub(crate) struct Context<'a> {
    pub oracle: &'a mut dyn NlpOracle,
    pub state: CallbackState,
    pub n: usize,
    pub m: usize,
    pub jac: Pattern,
    #[cfg(feature = "ipopt")]
    pub hess: Pattern,
}
impl<'a> Context<'a> {
    pub(crate) fn new(
        oracle: &'a mut dyn NlpOracle,
        execution: Execution,
        jac: Pattern,
        hess: Pattern,
    ) -> Self {
        #[cfg(not(feature = "ipopt"))]
        let _ = hess;
        Self {
            n: oracle.contract().variables.len(),
            m: oracle.contract().rows.len(),
            oracle,
            state: CallbackState::new(execution),
            jac,
            #[cfg(feature = "ipopt")]
            hess,
        }
    }
    fn dimensions(&self, n: i32, m: Option<i32>) -> Result<(), ProblemError> {
        if usize::try_from(n).ok() != Some(self.n)
            || m.is_some_and(|m| usize::try_from(m).ok() != Some(self.m))
        {
            Err(ProblemError::internal("foreign NLP callback dimensions"))
        } else {
            Ok(())
        }
    }
    fn evaluate(
        &mut self,
        demand: &str,
        work: impl FnOnce(&mut dyn NlpOracle) -> Result<(), ProblemError>,
    ) -> i32 {
        if self.state.evaluate(demand, || work(self.oracle)).is_some() {
            0
        } else if self.state.terminal.is_none() {
            1
        } else {
            2
        }
    }
}
#[derive(Default)]
pub(crate) struct Slot {
    active: Cell<*mut c_void>,
}
impl Slot {
    pub(crate) fn data(&self) -> *mut c_void {
        ptr::from_ref(self).cast_mut().cast()
    }
    /// Activates exactly one borrowed oracle for this synchronous foreign call.
    /// The guard clears activation on normal return and unwinding.
    pub(crate) fn with_active<T>(&self, context: &mut Context<'_>, work: impl FnOnce() -> T) -> T {
        struct Clear<'a>(&'a Cell<*mut c_void>);
        impl Drop for Clear<'_> {
            fn drop(&mut self) {
                self.0.set(ptr::null_mut());
            }
        }
        self.active.set(ptr::from_mut(context).cast());
        let clear = Clear(&self.active);
        let output = work();
        drop(clear);
        output
    }
}
unsafe fn input<'a>(p: *const f64, n: usize) -> Result<&'a [f64], ProblemError> {
    if n == 0 {
        return Ok(&[]);
    }
    if p.is_null() {
        return Err(ProblemError::internal("null foreign NLP input"));
    }
    // SAFETY: bridge supplies n readable initialized doubles during this callback.
    Ok(unsafe { std::slice::from_raw_parts(p, n) })
}
unsafe fn publish(p: *mut f64, values: &[f64]) -> Result<(), ProblemError> {
    if values.is_empty() {
        return Ok(());
    }
    if p.is_null() {
        return Err(ProblemError::internal("null foreign NLP output"));
    }
    // SAFETY: bridge supplies output capacity matching the checked admitted extent.
    unsafe {
        ptr::copy_nonoverlapping(values.as_ptr(), p, values.len());
    }
    Ok(())
}
pub(crate) fn finite(values: &[f64]) -> Result<(), ProblemError> {
    if values.iter().any(|v| !v.is_finite()) {
        Err(ProblemError::numerical(
            "nonfinite foreign NLP callback output",
        ))
    } else {
        Ok(())
    }
}
unsafe fn boundary(data: *mut c_void, work: impl FnOnce(&mut Context<'_>) -> i32) -> i32 {
    // SAFETY: data points to the stable owned Slot registered in the foreign model.
    let Some(slot) = (unsafe { data.cast::<Slot>().as_ref() }) else {
        return 2;
    };
    // SAFETY: activation exists only during the synchronous with_active call; no
    // other Rust code borrows the Context while the bridge calls back.
    let Some(context) = (unsafe { slot.active.get().cast::<Context<'_>>().as_mut() }) else {
        return 2;
    };
    match catch_unwind(AssertUnwindSafe(|| work(context))) {
        Ok(status) => status,
        Err(_) => {
            context.state.terminal =
                Some((Termination::Panic, "panic in foreign NLP callback".into()));
            context.state.last_failure =
                Some(ProblemError::internal("panic in foreign NLP callback"));
            2
        }
    }
}
pub(crate) unsafe extern "C" fn objective(
    n: i32,
    x: *const f64,
    out: *mut f64,
    data: *mut c_void,
) -> i32 {
    let work = |c: &mut Context<'_>| {
        let extent = c.n;
        let valid = c.dimensions(n, None);
        c.evaluate("objective", |o| {
            valid?;
            // SAFETY: the bridge supplies this readable vector for the checked variable extent.
            let x = unsafe { input(x, extent) }?;
            let value = o.objective(x)?;
            finite(&[value])?;
            // SAFETY: the bridge supplies one writable objective scalar, disjoint from values.
            unsafe { publish(out, &[value]) }
        })
    };
    // SAFETY: the bridge retains the live owned Slot; boundary checks activation
    // before borrowing its Context and contains callback unwinding.
    unsafe { boundary(data, work) }
}
pub(crate) unsafe extern "C" fn gradient(
    n: i32,
    x: *const f64,
    out: *mut f64,
    data: *mut c_void,
) -> i32 {
    let work = |c: &mut Context<'_>| {
        let extent = c.n;
        let valid = c.dimensions(n, None);
        c.evaluate("gradient", |o| {
            valid?;
            let mut values = vec![0.; extent];
            // SAFETY: the bridge supplies this readable vector for the checked variable extent.
            let x = unsafe { input(x, extent) }?;
            o.gradient(x, &mut values)?;
            finite(&values)?;
            // SAFETY: output has the checked variable extent and cannot alias the owned values.
            unsafe { publish(out, &values) }
        })
    };
    // SAFETY: the bridge retains the live owned Slot; boundary checks activation
    // before borrowing its Context and contains callback unwinding.
    unsafe { boundary(data, work) }
}
pub(crate) unsafe extern "C" fn constraints(
    n: i32,
    m: i32,
    x: *const f64,
    out: *mut f64,
    data: *mut c_void,
) -> i32 {
    let work = |c: &mut Context<'_>| {
        let (variables, rows) = (c.n, c.m);
        let valid = c.dimensions(n, Some(m));
        c.evaluate("constraints", |o| {
            valid?;
            let mut values = vec![0.; rows];
            // SAFETY: the bridge supplies this readable vector for the checked variable extent.
            let x = unsafe { input(x, variables) }?;
            o.constraints(x, &mut values)?;
            finite(&values)?;
            // SAFETY: output has the checked row extent and cannot alias the owned values.
            unsafe { publish(out, &values) }
        })
    };
    // SAFETY: the bridge retains the live owned Slot; boundary checks activation
    // before borrowing its Context and contains callback unwinding.
    unsafe { boundary(data, work) }
}
pub(crate) unsafe extern "C" fn jacobian(
    n: i32,
    nnz: i32,
    x: *const f64,
    out: *mut f64,
    data: *mut c_void,
) -> i32 {
    let work = |c: &mut Context<'_>| {
        let (variables, count) = (c.n, c.jac.rows.len());
        let valid = c.dimensions(n, None);
        c.evaluate("jacobian", |o| {
            valid?;
            if usize::try_from(nnz).ok() != Some(count) {
                return Err(ProblemError::internal("foreign Jacobian extent"));
            }
            let mut values = vec![0.; count];
            // SAFETY: the bridge supplies this readable vector for the checked variable extent.
            let x = unsafe { input(x, variables) }?;
            o.jacobian(x, &mut values)?;
            finite(&values)?;
            // SAFETY: output has the checked nonzero extent and cannot alias the owned values.
            unsafe { publish(out, &values) }
        })
    };
    // SAFETY: the bridge retains the live owned Slot; boundary checks activation
    // before borrowing its Context and contains callback unwinding.
    unsafe { boundary(data, work) }
}
#[cfg(feature = "ipopt")]
pub(crate) unsafe extern "C" fn hessian(
    n: i32,
    m: i32,
    nnz: i32,
    x: *const f64,
    weight: f64,
    multipliers: *const f64,
    out: *mut f64,
    data: *mut c_void,
) -> i32 {
    let work = |c: &mut Context<'_>| {
        let (variables, rows, count) = (c.n, c.m, c.hess.rows.len());
        let valid = c.dimensions(n, Some(m));
        c.evaluate("hessian", |o| {
            valid?;
            if usize::try_from(nnz).ok() != Some(count) {
                return Err(ProblemError::internal("foreign Hessian extent"));
            }
            finite(&[weight])?;
            let mut values = vec![0.; count];
            // SAFETY: the bridge supplies this readable vector for the checked variable extent.
            let x = unsafe { input(x, variables) }?;
            // SAFETY: the bridge supplies readable multipliers for the checked row extent.
            let multipliers = unsafe { input(multipliers, rows) }?;
            o.hessian(x, weight, multipliers, &mut values)?;
            finite(&values)?;
            // SAFETY: output has the checked nonzero extent and cannot alias the owned values.
            unsafe { publish(out, &values) }
        })
    };
    // SAFETY: the bridge retains the live owned Slot; boundary checks activation
    // before borrowing its Context and contains callback unwinding.
    unsafe { boundary(data, work) }
}
pub(crate) unsafe extern "C" fn poll(data: *mut c_void) -> i32 {
    // SAFETY: boundary contains every callback; no borrowed oracle is evaluated here.
    unsafe {
        boundary(data, |c| {
            if let Some((stop, _)) = c.state.terminal.as_ref() {
                return match stop {
                    Termination::Cancelled => 1,
                    Termination::TimeLimit => 2,
                    Termination::Limit => 3,
                    _ => 4,
                };
            }
            if let Some(stop) = c.state.execution.stopped() {
                c.state.last_failure = c.state.execution.check().err();
                c.state.terminal = Some((stop, "foreign NLP execution checkpoint".into()));
                match stop {
                    Termination::Cancelled => 1,
                    Termination::TimeLimit => 2,
                    Termination::Limit => 3,
                    _ => 4,
                }
            } else {
                0
            }
        })
    }
}
pub(crate) fn remaining(execution: &Execution) -> Result<f64, ProblemError> {
    execution.check()?;
    let deadline = execution
        .scope()?
        .deadline()
        .ok_or_else(|| ProblemError::Contract("finite native task scope required".into()))?;
    Ok(deadline
        .saturating_duration_since(std::time::Instant::now())
        .as_secs_f64())
}

#[cfg(feature = "ipopt")]
pub(crate) unsafe extern "C" fn iteration(
    mode: i32,
    number: i32,
    objective: f64,
    primal: f64,
    dual: f64,
    barrier: f64,
    step: f64,
    regularization: f64,
    alpha_dual: f64,
    alpha_primal: f64,
    trials: i32,
    n: i32,
    current: *const f64,
    lagrangian: *const f64,
    data: *mut c_void,
) -> i32 {
    let work = |c: &mut Context<'_>| {
        let valid = c.dimensions(n, None);
        let extent = c.n;
        let crossings = c.state.complete_iteration();
        let execution = c.state.execution.clone();
        c.evaluate("intermediate", |_| {
            valid?;
            let mut values = std::collections::BTreeMap::from([
                (
                    "iteration".into(),
                    crate::solve::Metric::Integer(i64::from(number)),
                ),
                (
                    "regime.crossings".into(),
                    crate::solve::Metric::Integer(i64::try_from(crossings).unwrap_or(i64::MAX)),
                ),
                ("restoration".into(), crate::solve::Metric::Bool(mode == 1)),
                (
                    "objective.normalized".into(),
                    crate::solve::Metric::Real(objective),
                ),
                ("primal.native".into(), crate::solve::Metric::Real(primal)),
                ("dual.native".into(), crate::solve::Metric::Real(dual)),
                ("barrier".into(), crate::solve::Metric::Real(barrier)),
                ("step.norm".into(), crate::solve::Metric::Real(step)),
                (
                    "regularization".into(),
                    crate::solve::Metric::Real(regularization),
                ),
                ("alpha.dual".into(), crate::solve::Metric::Real(alpha_dual)),
                (
                    "alpha.primal".into(),
                    crate::solve::Metric::Real(alpha_primal),
                ),
                (
                    "line_search.trials".into(),
                    crate::solve::Metric::Integer(i64::from(trials)),
                ),
            ]);
            if !current.is_null() {
                // SAFETY: the bridge supplies a nonnull current vector of the checked extent,
                // immutable for this callback; only copied scalar observations are retained.
                let current = unsafe { input(current, extent) }?;
                finite(current)?;
                values.insert(
                    "iterate.normalized.infinity_norm".into(),
                    crate::solve::Metric::Real(current.iter().map(|v| v.abs()).fold(0., f64::max)),
                );
            }
            if !lagrangian.is_null() {
                // SAFETY: the bridge supplies a nonnull gradient vector of the checked extent,
                // immutable for this callback; only copied scalar observations are retained.
                let lagrangian = unsafe { input(lagrangian, extent) }?;
                finite(lagrangian)?;
                values.insert(
                    "stationarity.normalized".into(),
                    crate::solve::Metric::Real(
                        lagrangian.iter().map(|v| v.abs()).fold(0., f64::max),
                    ),
                );
            }
            execution.progress.push(crate::solve::Event {
                phase: "ipopt.iteration".into(),
                elapsed: execution.started.elapsed(),
                values,
                incumbent: None,
            });
            Ok(())
        })
    };
    // SAFETY: the bridge retains the live owned Slot; boundary checks activation
    // before borrowing its Context and contains callback unwinding.
    unsafe { boundary(data, work) }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn context(oracle: &mut dyn NlpOracle) -> Context<'_> {
        let jac = Pattern::new(oracle.jacobian_pattern(), false).unwrap();
        let hess = Pattern::new(oracle.hessian_pattern().unwrap(), true).unwrap();
        #[cfg(any(feature = "ipopt", feature = "pounce", feature = "kinsol"))]
        let execution = crate::solver_tests::execution();
        #[cfg(not(any(feature = "ipopt", feature = "pounce", feature = "kinsol")))]
        let execution = Execution::new(
            std::sync::Arc::default(),
            &crate::solve::Controls::default(),
        );
        Context::new(oracle, execution, jac, hess)
    }
    #[test]
    fn owned_slot_deactivates_and_failed_partial_output_never_publishes() {
        let mut oracle = crate::solver_tests::Polynomial::new();
        oracle.fail = true;
        let mut context = context(&mut oracle);
        let slot = Slot::default();
        let data = slot.data();
        let mut out = 91.;
        let code = slot.with_active(&mut context, || {
            // SAFETY: test-owned scalar buffers and active context have exact extents.
            unsafe { gradient(1, &2., &mut out, data) }
        });
        assert_eq!(code, 2);
        assert_eq!(out, 91.);
        assert!(
            matches!(context.state.last_failure, Some(ProblemError::Contract(_))),
            "{:?}",
            context.state
        );
        // SAFETY: stable slot remains live but inactive, so no buffer is touched.
        assert_eq!(unsafe { objective(1, &2., &mut out, data) }, 2);
        assert_eq!(out, 91.);
    }
    #[test]
    fn panic_is_contained_and_latched_before_any_second_oracle_call() {
        let mut oracle = crate::solver_tests::Polynomial::new();
        oracle.panic = true;
        let mut context = context(&mut oracle);
        let slot = Slot::default();
        let data = slot.data();
        let mut out = 91.;
        slot.with_active(&mut context, || {
            // SAFETY: test-owned scalar buffers and active context have exact extents.
            assert_eq!(unsafe { objective(1, &2., &mut out, data) }, 2);
            // SAFETY: the same live scalar buffers and activation remain valid;
            // the latched failure prevents any second oracle evaluation.
            assert_eq!(unsafe { objective(1, &2., &mut out, data) }, 2);
        });
        assert_eq!(out, 91.);
        assert_eq!(
            context.state.terminal.as_ref().unwrap().0,
            Termination::Panic
        );
        assert_eq!(context.state.counts["objective"], 1);
    }
    #[derive(Debug)]
    struct Domain(crate::solver_tests::Polynomial);
    impl NlpOracle for Domain {
        fn contract(&self) -> &crate::OracleContract {
            NlpOracle::contract(&self.0)
        }
        fn jacobian_pattern(&self) -> faer::sparse::SymbolicSparseColMatRef<'_, usize> {
            NlpOracle::jacobian_pattern(&self.0)
        }
        fn hessian_pattern(&self) -> Option<faer::sparse::SymbolicSparseColMatRef<'_, usize>> {
            NlpOracle::hessian_pattern(&self.0)
        }
        fn constraint_bounds(&self) -> &[(f64, f64)] {
            NlpOracle::constraint_bounds(&self.0)
        }
        fn objective(&mut self, x: &[f64]) -> Result<f64, ProblemError> {
            if x[0] < 0. {
                Err(pse_math::MathError::Domain {
                    source_id: crate::solver_tests::id(9),
                    requirement: "positive scripted trial",
                }
                .into())
            } else {
                self.0.objective(x)
            }
        }
        fn constraints(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
            self.0.constraints(x, out)
        }
        fn gradient(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
            self.0.gradient(x, out)
        }
        fn jacobian(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
            NlpOracle::jacobian(&mut self.0, x, out)
        }
        fn hessian(
            &mut self,
            x: &[f64],
            w: f64,
            l: &[f64],
            out: &mut [f64],
        ) -> Result<(), ProblemError> {
            self.0.hessian(x, w, l, out)
        }
    }
    #[test]
    fn typed_domain_trial_recovers_without_terminal_latch_or_partial_values() {
        let mut oracle = Domain(crate::solver_tests::Polynomial::new());
        let mut context = context(&mut oracle);
        let slot = Slot::default();
        let data = slot.data();
        let mut out = 91.;
        slot.with_active(&mut context, || {
            // SAFETY: test-owned scalar buffers and active context have exact extents.
            assert_eq!(unsafe { objective(1, &-1., &mut out, data) }, 1);
            assert_eq!(out, 91.);
        });
        assert!(matches!(
            context.state.last_failure,
            Some(ProblemError::Math(pse_math::MathError::Domain { .. }))
        ));
        assert!(context.state.terminal.is_none());
        slot.with_active(&mut context, || {
            // SAFETY: as above, with a valid trial and fresh activation.
            assert_eq!(unsafe { objective(1, &2., &mut out, data) }, 0);
        });
        assert_eq!(out, 4.);
        assert_eq!(context.state.trial_rejections, 1);
        assert!(context.state.last_failure.is_none());
    }
}
