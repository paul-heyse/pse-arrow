// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Scoped public POUNCE QP homotopy plus its library-owned corrector.
use super::super::activity::Limits;
use crate::{
    NativeStatus, ProblemError,
    solve::{Backend, Execution},
};
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
        let factors = self.state.attempted_factors.get();
        if calls >= self.limits.backsolves || new_matrix && factors >= self.limits.refactorizations
        {
            self.fail(ProblemError::Limit {
                kind: crate::LimitKind::Work,
                detail: "QP linear work allowance exhausted".into(),
            });
            return ESymSolverStatus::FatalError;
        }
        self.state.calls.set(calls + 1);
        if new_matrix {
            self.state.attempted_factors.set(factors + 1);
        }
        let status = self
            .backend
            .multi_solve(new_matrix, ia, ja, nrhs, rhs, check, negative);
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
    let mut solver = ParametricActiveSetSolver::new(Box::new(backend));
    let result = crate::quality::contained(|| {
        solver
            .solve_parametric(previous, request.source, target, &options)
            .map_err(error)
    });
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
