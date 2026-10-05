// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Scoped public POUNCE QP homotopy plus its library-owned corrector.
use super::super::activity::Limits;
use crate::{
    NativeStatus, ProblemError,
    solve::{Backend, Execution, WorkEvidence},
};
pub use pounce_common::types::{lower_bound_present, upper_bound_present};
pub use pounce_feral::FeralConfig;
use pounce_feral::FeralSolverInterface;
use pounce_linsol::{
    EMatrixFormat, ESymSolverStatus, LinearSolverSummary, SparseSymLinearSolverInterface,
};
pub use pounce_qp::{HessianInertia, QpOptions, QpProblem, QpSolution, QpStatus, WorkingSet};
use pounce_qp::{ParametricActiveSetSolver, QpSolver};
#[cfg(feature = "pounce")]
pub use pounce_rs::qp::{GenTMatrix, GenTMatrixSpace, SymTMatrix, SymTMatrixSpace};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    sync::Arc,
    time::Instant,
};
/// Admitted convex QP family with unchanged H, A, variable bounds and equation topology.
#[derive(Clone, Copy)]
pub struct Request<'a> {
    /// Original library problem, in explicit identical coordinates.
    pub previous: &'a QpProblem<'a>,
    /// Qualified previous library solution; no independent task start is substituted.
    pub source: &'a QpSolution,
    /// Target QP; only objective linear term and row limits may change.
    pub target: &'a QpProblem<'a>,
    /// Explicit bounded native path/corrector options.
    pub options: &'a QpOptions,
    /// Complete shared FERAL configuration, under serial/FMA-free admission.
    pub linear: &'a FeralConfig,
    /// Explicit finite linear work/layout allowance.
    pub limits: Limits,
}
impl std::fmt::Debug for Request<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Request")
            .field("variables", &self.target.n)
            .field("rows", &self.target.m)
            .field("options", self.options)
            .field("limits", &self.limits)
            .finish_non_exhaustive()
    }
}
/// Copied actual library result; Optimal remains a QP result, never an original NLP claim.
#[derive(Clone, Debug)]
pub struct Outcome {
    /// Actual native status, working set, point, canonical QP multipliers and path source.
    pub solution: QpSolution,
    /// Actual forwarded linear backend calls, including native refusals.
    pub linear_calls: usize,
    /// Actual completed FERAL factors and their source statistics.
    pub linear: LinearSolverSummary,
    /// Effective native options, including the original remaining task deadline.
    pub options: QpOptions,
}
#[derive(Default)]
struct State {
    calls: Cell<usize>,
    attempted_factors: Cell<usize>,
    failure: RefCell<Option<Arc<ProblemError>>>,
    summary: RefCell<LinearSolverSummary>,
}
/// Observe actual source primitives, including quality-triggered replacement factors.
struct Observation {
    execution: Execution,
    limits: Limits,
    state: Rc<State>,
    storage: RefCell<std::collections::BTreeMap<(&'static str, usize), usize>>,
}
impl Observation {
    fn fail(&self, error: ProblemError) -> pounce_common::observed::Abort {
        let abort = if matches!(error, ProblemError::Cancelled) {
            pounce_common::observed::Abort::Cancelled
        } else {
            pounce_common::observed::Abort::Resource(error.to_string())
        };
        if self.state.failure.borrow().is_none() {
            *self.state.failure.borrow_mut() = Some(Arc::new(error));
        }
        abort
    }
}
fn factor_unit() -> WorkEvidence {
    WorkEvidence {
        evaluations: Some(0),
        iterations: Some(0),
        factorizations: Some(1),
        proof_steps: Some(0),
    }
}
impl pounce_common::observed::Observer for Observation {
    fn bind_layout(
        &self,
        _: &pounce_common::observed::Layout,
    ) -> Result<Option<Vec<usize>>, pounce_common::observed::Abort> {
        Ok(None)
    }
    fn event(
        &self,
        event: &pounce_common::observed::Event,
    ) -> Result<(), pounce_common::observed::Abort> {
        use pounce_common::observed::{Event, Primitive};
        if !matches!(event, Event::End { .. }) {
            self.execution.check().map_err(|e| self.fail(e))?;
        }
        match event {
            Event::Begin {
                primitive: Primitive::Factor,
                ..
            } => {
                let count = self.state.attempted_factors.get();
                if count >= self.limits.refactorizations {
                    return Err(self.fail(ProblemError::Limit {
                        kind: crate::LimitKind::Work,
                        detail: "QP actual factor allowance exhausted".into(),
                    }));
                }
                if let Some(owner) = &self.execution.work_admission {
                    crate::quality::contained(|| owner.admit(factor_unit()))
                        .map_err(|e| self.fail(e))?;
                }
                self.state.attempted_factors.set(count + 1);
            }
            Event::End {
                primitive: Primitive::Factor,
                ..
            } => {
                if let Some(owner) = &self.execution.work_admission {
                    crate::quality::contained(|| owner.observe(factor_unit()))
                        .map_err(|e| self.fail(e))?;
                }
            }
            Event::Storage {
                scope,
                owner,
                instance,
                known_bytes,
                opaque,
            } => {
                let previous = self
                    .storage
                    .borrow()
                    .get(&(*owner, *instance))
                    .copied()
                    .unwrap_or(0);
                if let Some(admission) = &self.execution.work_admission {
                    let scope = match scope {
                        pounce_common::observed::StorageScope::Application => {
                            crate::solve::NativeStorageScope::Application
                        }
                        pounce_common::observed::StorageScope::Linear => {
                            crate::solve::NativeStorageScope::Linear
                        }
                    };
                    admission
                        .admit_storage(scope, owner, known_bytes.saturating_sub(previous), *opaque)
                        .map_err(|e| self.fail(e))?;
                }
                self.storage
                    .borrow_mut()
                    .insert((*owner, *instance), previous.max(*known_bytes));
            }
            _ => {}
        }
        Ok(())
    }
}

struct Scoped {
    backend: FeralSolverInterface,
    execution: Execution,
    limits: Limits,
    state: Rc<State>,
}
impl Scoped {
    fn checkpoint(&self) -> bool {
        if self.state.failure.borrow().is_some() {
            return false;
        }
        if let Err(error) = self.execution.check() {
            self.fail(error);
            return false;
        }
        true
    }
    fn fail(&self, error: ProblemError) {
        if self.state.failure.borrow().is_none() {
            *self.state.failure.borrow_mut() = Some(Arc::new(error));
        }
    }
    fn status(&self, status: ESymSolverStatus) -> ESymSolverStatus {
        if status == ESymSolverStatus::FatalError && self.state.failure.borrow().is_none() {
            // The public pounce-feral seam supplies this typed native status only;
            // it does not expose its internal FERAL error payload.
            self.fail(ProblemError::native(
                NativeStatus {
                    backend: Backend::Pounce,
                    code: status as i64,
                    name: "SYMSOLVER_FATAL_ERROR".into(),
                },
                crate::solve::Termination::Invalid,
                "POUNCE QP linear backend refused its operation",
            ));
        }
        status
    }
}
impl SparseSymLinearSolverInterface for Scoped {
    fn initialize_structure(
        &mut self,
        dim: i32,
        nonzeros: i32,
        ia: &[i32],
        ja: &[i32],
    ) -> ESymSolverStatus {
        if !self.checkpoint() {
            return ESymSolverStatus::FatalError;
        }
        let extent = usize::try_from(nonzeros)
            .ok()
            .and_then(|n| n.checked_mul(32))
            .and_then(|n| {
                usize::try_from(dim)
                    .ok()
                    .and_then(|d| d.checked_mul(32))
                    .and_then(|d| n.checked_add(d))
            });
        if dim < 0 || nonzeros < 0 || ia.len() != nonzeros as usize || ja.len() != nonzeros as usize
        {
            self.fail(ProblemError::Contract(
                "QP linear structure coordinates".into(),
            ));
            return ESymSolverStatus::FatalError;
        }
        if extent.is_none_or(|n| n > self.limits.bytes) {
            self.fail(ProblemError::memory("QP linear layout allowance exhausted"));
            return ESymSolverStatus::FatalError;
        }
        let status = self.backend.initialize_structure(dim, nonzeros, ia, ja);
        self.status(status)
    }
    fn values_array_mut(&mut self) -> &mut [f64] {
        self.backend.values_array_mut()
    }
    fn multi_solve(
        &mut self,
        new_matrix: bool,
        ia: &[i32],
        ja: &[i32],
        nrhs: i32,
        rhs: &mut [f64],
        check: bool,
        negative: i32,
    ) -> ESymSolverStatus {
        if !self.checkpoint() {
            return ESymSolverStatus::FatalError;
        }
        let calls = self.state.calls.get();
        if calls >= self.limits.backsolves {
            self.fail(ProblemError::Limit {
                kind: crate::LimitKind::Work,
                detail: "QP linear work allowance exhausted".into(),
            });
            return ESymSolverStatus::FatalError;
        }
        let work = WorkEvidence {
            evaluations: Some(0),
            // The source QP/refinement loops do not expose iteration admission here.
            iterations: None,
            factorizations: Some(0),
            proof_steps: Some(0),
        };
        let execution = self.execution.clone();
        let status = match execution.counted(work, || {
            self.state.calls.set(calls + 1);
            let status = self
                .backend
                .multi_solve(new_matrix, ia, ja, nrhs, rhs, check, negative);
            Ok(self.status(status))
        }) {
            Ok(status) => status,
            Err(error) => {
                self.fail(error);
                return ESymSolverStatus::FatalError;
            }
        };
        *self.state.summary.borrow_mut() = self.backend.summary();
        if !self.checkpoint() {
            return ESymSolverStatus::FatalError;
        }
        self.status(status)
    }
    fn number_of_neg_evals(&self) -> i32 {
        self.backend.number_of_neg_evals()
    }
    fn increase_quality(&mut self) -> bool {
        self.checkpoint() && self.backend.increase_quality()
    }
    fn provides_inertia(&self) -> bool {
        self.backend.provides_inertia()
    }
    fn multi_solve_matches_single_solve(&self, nrhs: usize) -> bool {
        self.backend.multi_solve_matches_single_solve(nrhs)
    }
    fn matrix_format(&self) -> EMatrixFormat {
        self.backend.matrix_format()
    }
}
fn error(error: pounce_qp::QpError) -> ProblemError {
    use pounce_qp::QpError as E;
    match error {
        E::DimensionMismatch(detail)
        | E::InvertedBounds(detail)
        | E::WarmStartDimensionMismatch(detail) => ProblemError::Contract(detail),
        E::UnsupportedFeature(detail) => ProblemError::Unsupported(detail),
        E::LinearSolverFailure(detail) => ProblemError::numerical(detail),
        E::DeadlineExpired => ProblemError::Limit {
            kind: crate::LimitKind::Time,
            detail: "POUNCE QP native task deadline".into(),
        },
    }
}
fn same_values(a: &[f64], b: &[f64]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(a, b)| a.to_bits() == b.to_bits())
}
/// Execute the exact admitted QP family under the original task scope.
///
/// # Errors
/// Changed coordinates/operator/bounds, unsupported profile, typed scope/allowance
/// stop, or an actual library failure. A late stop vetoes every returned candidate.
pub fn predict(request: Request<'_>, execution: Execution) -> Result<Outcome, Arc<ProblemError>> {
    execution.check()?;
    request.limits.validate()?;
    let previous = request.previous;
    let target = request.target;
    previous.validate().map_err(error)?;
    target.validate().map_err(error)?;
    if previous
        .h
        .values()
        .iter()
        .chain(previous.a.values())
        .chain(previous.g)
        .chain(target.g)
        .chain(&request.source.x)
        .chain(&request.source.lambda_g)
        .chain(&request.source.lambda_x)
        .any(|v| !v.is_finite())
        || previous
            .bl
            .iter()
            .chain(previous.bu)
            .chain(previous.xl)
            .chain(previous.xu)
            .chain(target.bl)
            .chain(target.bu)
            .chain(target.xl)
            .chain(target.xu)
            .any(|v| v.is_nan())
    {
        return Err(Arc::new(ProblemError::Contract(
            "QP path requires finite coefficients/point and valid explicit bounds".into(),
        )));
    }
    if previous.n != target.n
        || previous.m != target.m
        || previous.h.irows() != target.h.irows()
        || previous.h.jcols() != target.h.jcols()
        || !same_values(previous.h.values(), target.h.values())
        || previous.a.irows() != target.a.irows()
        || previous.a.jcols() != target.a.jcols()
        || !same_values(previous.a.values(), target.a.values())
        || !same_values(previous.xl, target.xl)
        || !same_values(previous.xu, target.xu)
        || previous
            .bl
            .iter()
            .zip(previous.bu)
            .zip(target.bl.iter().zip(target.bu))
            .any(|((l, u), (tl, tu))| (l == u) != (tl == tu))
        || request.source.x.len() != previous.n
        || request.source.lambda_g.len() != previous.m
        || request.source.lambda_x.len() != previous.n
    {
        return Err(Arc::new(ProblemError::Contract("QP path requires identical explicit coordinates, H, A, variable bounds and equation topology".into())));
    }
    if previous.hessian_inertia != HessianInertia::Psd
        || target.hessian_inertia != HessianInertia::Psd
        || request.source.status != QpStatus::Optimal
    {
        return Err(Arc::new(ProblemError::Unsupported(
            "QP path requires an admitted convex family and a qualified original library point"
                .into(),
        )));
    }
    if request.linear.parallel != Some(false)
        || request.linear.fma
        || request.options.max_iter == 0
        || !request.options.feas_tol.is_finite()
        || request.options.feas_tol <= 0.
        || !request.options.opt_tol.is_finite()
        || request.options.opt_tol <= 0.
    {
        return Err(Arc::new(ProblemError::Unsupported("QP path requires serial FMA-free factors and finite positive native work/accuracy controls".into())));
    }
    let remaining = execution
        .scope()?
        .deadline()
        .ok_or_else(|| ProblemError::Contract("QP path requires finite task deadline".into()))?
        .saturating_duration_since(Instant::now());
    if remaining.is_zero() {
        execution.check()?;
        return Err(Arc::new(ProblemError::Limit {
            kind: crate::LimitKind::Time,
            detail: "QP path task deadline expired".into(),
        }));
    }
    let mut options = request.options.clone();
    options.time_limit = Some(options.time_limit.map_or(remaining, |v| v.min(remaining)));
    let state = Rc::new(State::default());
    let backend = Scoped {
        backend: FeralSolverInterface::with_config(request.linear.clone()),
        execution: execution.clone(),
        limits: request.limits,
        state: state.clone(),
    };
    let _observation = pounce_common::observed::Scope::enter(Rc::new(Observation {
        execution: execution.clone(),
        limits: request.limits,
        state: state.clone(),
        storage: RefCell::default(),
    }));
    pounce_common::observed::set_linear_bounded(pounce_feral::complete_storage_profile(
        request.linear,
    ));
    pounce_common::observed::set_linear_maximum(request.linear.bounded_storage_max_dimension);
    let mut solver = ParametricActiveSetSolver::new(Box::new(backend));
    // Refuse an unobserved capped QP iteration count before entering its loop.
    let result = execution.counted(
        WorkEvidence {
            evaluations: Some(0),
            iterations: None,
            factorizations: Some(0),
            proof_steps: Some(0),
        },
        || {
            solver
                .solve_parametric(previous, request.source, target, &options)
                .map_err(error)
        },
    );
    if let Some(cause) = state.failure.borrow().clone() {
        return Err(cause);
    }
    execution.check()?;
    let solution = result?;
    if solution
        .x
        .iter()
        .chain(&solution.lambda_g)
        .chain(&solution.lambda_x)
        .any(|v| !v.is_finite())
    {
        return Err(Arc::new(ProblemError::numerical(
            "nonfinite QP path result",
        )));
    }
    let linear = state.summary.borrow().clone();
    Ok(Outcome {
        solution,
        linear_calls: state.calls.get(),
        linear,
        options,
    })
}

#[cfg(all(test, feature = "pounce"))]
mod tests {
    use super::*;
    use std::time::Duration;
    fn matrices() -> (SymTMatrix, GenTMatrix) {
        let mut h = SymTMatrix::new(SymTMatrixSpace::new(1, vec![1], vec![1]));
        h.set_values(&[1.]);
        let mut a = GenTMatrix::new(GenTMatrixSpace::new(1, 1, vec![1], vec![1]));
        a.set_values(&[1.]);
        (h, a)
    }
    fn qp<'a>(h: &'a SymTMatrix, a: &'a GenTMatrix, g: &'a [f64]) -> QpProblem<'a> {
        QpProblem {
            n: 1,
            m: 1,
            h,
            g,
            a,
            bl: &[0.],
            bu: &[1e20],
            xl: &[-1e20],
            xu: &[1e20],
            hessian_inertia: HessianInertia::Psd,
        }
    }
    fn configuration() -> (FeralConfig, QpOptions, Limits, Execution) {
        let linear = FeralConfig {
            parallel: Some(false),
            fma: false,
            ..FeralConfig::default()
        };
        let options = QpOptions {
            max_iter: 50,
            time_limit: Some(Duration::from_secs(5)),
            feas_tol: 1e-9,
            opt_tol: 1e-9,
            ..QpOptions::default()
        };
        let limits = Limits {
            backsolves: 1000,
            refactorizations: 200,
            bytes: 1 << 20,
        };
        let controls = crate::solve::Controls {
            time_limit: Duration::from_secs(5),
            ..crate::solve::Controls::default()
        };
        (
            linear,
            options,
            limits,
            Execution::new(Arc::default(), &controls),
        )
    }
    #[derive(Debug, Default)]
    struct Admission {
        seen: std::sync::Mutex<Vec<WorkEvidence>>,
        factor_cap: Option<u64>,
        iterations: Option<u64>,
    }
    impl crate::solve::WorkAdmission for Admission {
        fn admit(&self, work: WorkEvidence) -> Result<(), ProblemError> {
            let factors: u64 = self
                .seen
                .lock()
                .unwrap()
                .iter()
                .map(|w| w.factorizations.unwrap())
                .sum();
            if self
                .factor_cap
                .is_some_and(|cap| factors + work.factorizations.unwrap() > cap)
                || self.iterations.is_some() && work.iterations.is_none()
            {
                return Err(ProblemError::Limit {
                    kind: crate::LimitKind::Work,
                    detail: "QP task test cap".into(),
                });
            }
            Ok(())
        }
        fn observe(&self, work: WorkEvidence) -> Result<(), ProblemError> {
            self.seen.lock().unwrap().push(work);
            Ok(())
        }
    }
    fn scoped(admission: Arc<Admission>) -> (Scoped, pounce_common::observed::Scope) {
        let (mut linear, _, limits, mut execution) = configuration();
        linear.scaling = feral::scaling::ScalingStrategy::Identity;
        linear.increase_quality = true;
        execution.work_admission = Some(admission);
        let state = Rc::new(State::default());
        let scope = pounce_common::observed::Scope::enter(Rc::new(Observation {
            execution: execution.clone(),
            limits,
            state: state.clone(),
            storage: RefCell::default(),
        }));
        (
            Scoped {
                backend: FeralSolverInterface::with_config(linear),
                execution,
                limits,
                state,
            },
            scope,
        )
    }
    #[test]
    fn shared_factor_cap_covers_actual_quality_retry_without_charging_reused_actions() {
        let admission = Arc::new(Admission {
            factor_cap: Some(1),
            ..Default::default()
        });
        let (mut backend, _scope) = scoped(admission.clone());
        assert_eq!(
            backend.initialize_structure(1, 1, &[1], &[1]),
            ESymSolverStatus::Success
        );
        backend.values_array_mut()[0] = 2.;
        assert_eq!(
            backend.multi_solve(true, &[1], &[1], 1, &mut [1.], false, 0),
            ESymSolverStatus::Success
        );
        assert_eq!(
            backend.multi_solve(false, &[1], &[1], 1, &mut [2.], false, 0),
            ESymSolverStatus::Success
        );
        assert_eq!(backend.state.attempted_factors.get(), 1);
        assert_eq!(
            admission
                .seen
                .lock()
                .unwrap()
                .iter()
                .map(|w| w.factorizations.unwrap())
                .sum::<u64>(),
            1
        );
        assert!(backend.increase_quality());
        assert_eq!(
            backend.multi_solve(false, &[1], &[1], 1, &mut [1.], false, 0),
            ESymSolverStatus::CallAgain
        );
        backend.values_array_mut()[0] = 2.;
        assert_eq!(
            backend.multi_solve(false, &[1], &[1], 1, &mut [1.], false, 0),
            ESymSolverStatus::FatalError
        );
        assert_eq!(backend.state.attempted_factors.get(), 1);
        assert!(matches!(
            backend.state.failure.borrow().as_deref(),
            Some(ProblemError::Limit { .. })
        ));
    }
    #[test]
    fn failed_qp_factor_is_counted_and_strict_opaque_iteration_cap_refuses_before_dispatch() {
        let admission = Arc::new(Admission::default());
        {
            let (mut backend, _scope) = scoped(admission.clone());
            assert_eq!(
                backend.initialize_structure(1, 1, &[1], &[1]),
                ESymSolverStatus::Success
            );
            backend.values_array_mut()[0] = f64::NAN;
            assert_eq!(
                backend.multi_solve(true, &[1], &[1], 1, &mut [1.], false, 0),
                ESymSolverStatus::FatalError
            );
            assert_eq!(backend.state.attempted_factors.get(), 1);
            assert!(backend.state.failure.borrow().is_some());
        }
        assert_eq!(
            admission
                .seen
                .lock()
                .unwrap()
                .iter()
                .map(|w| w.factorizations.unwrap())
                .sum::<u64>(),
            1
        );
        let capped = Arc::new(Admission {
            iterations: Some(0),
            ..Default::default()
        });
        let (mut backend, _scope) = scoped(capped.clone());
        assert_eq!(
            backend.initialize_structure(1, 1, &[1], &[1]),
            ESymSolverStatus::Success
        );
        backend.values_array_mut()[0] = 2.;
        assert_eq!(
            backend.multi_solve(true, &[1], &[1], 1, &mut [1.], false, 0),
            ESymSolverStatus::FatalError
        );
        assert_eq!(backend.state.calls.get(), 0);
        assert_eq!(backend.state.attempted_factors.get(), 0);
        assert!(capped.seen.lock().unwrap().is_empty());
    }
    #[test]
    fn strict_qp_iteration_cap_refuses_before_the_native_path_loop() {
        let (h, a) = matrices();
        let previous = qp(&h, &a, &[1.]);
        let target = qp(&h, &a, &[-1.]);
        let (linear, options, limits, mut execution) = configuration();
        let mut cold = ParametricActiveSetSolver::new(Box::new(FeralSolverInterface::with_config(
            linear.clone(),
        )));
        let source = cold.solve(&previous, None, &options).unwrap();
        let admission = Arc::new(Admission {
            iterations: Some(0),
            ..Default::default()
        });
        execution.work_admission = Some(admission.clone());
        let refused = predict(
            Request {
                previous: &previous,
                source: &source,
                target: &target,
                options: &options,
                linear: &linear,
                limits,
            },
            execution,
        );
        assert!(matches!(refused,Err(cause) if matches!(&*cause,ProblemError::Limit{..})));
        assert!(admission.seen.lock().unwrap().is_empty());
    }
    #[test]
    fn actual_convex_qp_homotopy_runs_library_corrector_and_records_real_source() {
        let (h, a) = matrices();
        let previous = qp(&h, &a, &[1.]);
        let target = qp(&h, &a, &[-1.]);
        let (linear, options, limits, execution) = configuration();
        let mut cold = ParametricActiveSetSolver::new(Box::new(FeralSolverInterface::with_config(
            linear.clone(),
        )));
        let source = cold.solve(&previous, None, &options).unwrap();
        assert_eq!(source.status, QpStatus::Optimal);
        assert!(source.x[0].abs() < 1e-9);
        let outcome = predict(
            Request {
                previous: &previous,
                source: &source,
                target: &target,
                options: &options,
                linear: &linear,
                limits,
            },
            execution,
        )
        .unwrap();
        assert_eq!(outcome.solution.status, QpStatus::Optimal);
        assert!((outcome.solution.x[0] - 1.).abs() < 1e-9);
        assert_eq!(
            outcome.solution.stats.parametric_source,
            Some(pounce_qp::ParametricSource::Homotopy)
        );
        assert!(outcome.linear_calls > 0);
        assert!(outcome.linear.n_factors > 0);
        assert!(outcome.options.time_limit.unwrap() <= Duration::from_secs(5));
    }
    #[test]
    fn changed_row_operator_is_refused_before_unmodelled_path_or_cold_fallback() {
        let (h, a) = matrices();
        let (linear, options, limits, execution) = configuration();
        let mut other = GenTMatrix::new(GenTMatrixSpace::new(1, 1, vec![1], vec![1]));
        other.set_values(&[2.]);
        let previous = qp(&h, &a, &[1.]);
        let target = qp(&h, &other, &[-1.]);
        let mut cold = ParametricActiveSetSolver::new(Box::new(FeralSolverInterface::with_config(
            linear.clone(),
        )));
        let source = cold.solve(&previous, None, &options).unwrap();
        let cause = predict(
            Request {
                previous: &previous,
                source: &source,
                target: &target,
                options: &options,
                linear: &linear,
                limits,
            },
            execution,
        )
        .unwrap_err();
        assert!(matches!(cause.as_ref(), ProblemError::Contract(_)));
    }
    #[test]
    fn cancel_and_linear_work_limit_cannot_be_relabelled_native_optimal() {
        let (h, a) = matrices();
        let previous = qp(&h, &a, &[1.]);
        let target = qp(&h, &a, &[-1.]);
        let (linear, options, limits, execution) = configuration();
        let mut cold = ParametricActiveSetSolver::new(Box::new(FeralSolverInterface::with_config(
            linear.clone(),
        )));
        let source = cold.solve(&previous, None, &options).unwrap();
        let request = Request {
            previous: &previous,
            source: &source,
            target: &target,
            options: &options,
            linear: &linear,
            limits,
        };
        execution
            .cancel
            .store(true, std::sync::atomic::Ordering::Relaxed);
        assert!(matches!(
            predict(request, execution).unwrap_err().as_ref(),
            ProblemError::Cancelled
        ));
        let (_, _, _, execution) = configuration();
        let cause = predict(
            Request {
                limits: Limits {
                    backsolves: 1,
                    refactorizations: 1,
                    ..limits
                },
                ..request
            },
            execution,
        )
        .unwrap_err();
        assert!(matches!(
            cause.as_ref(),
            ProblemError::Limit {
                kind: crate::LimitKind::Work,
                ..
            }
        ));
    }
    #[test]
    fn native_fatal_status_is_terminal_even_when_the_library_erases_its_cause() {
        let (linear, _, limits, execution) = configuration();
        let state = Rc::new(State::default());
        let scoped = Scoped {
            backend: FeralSolverInterface::with_config(linear),
            execution,
            limits,
            state: state.clone(),
        };
        assert_eq!(
            scoped.status(ESymSolverStatus::FatalError),
            ESymSolverStatus::FatalError
        );
        assert!(matches!(
            state.failure.borrow().as_deref(),
            Some(ProblemError::Internal(_))
        ));
        assert!(!scoped.checkpoint());
        assert_eq!(state.calls.get(), 0);
    }
}
