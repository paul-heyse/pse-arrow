// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Rust-only Diffsol adapter. All operator failures exit through one owned catch boundary.
//! The scheme (BDF, TR-BDF2, ESDIRK34, Tsit45) and the Newton linear solver (faer LU or
//! SuiteSparse KLU) are typed profile fields (ADR-0110 item 2). Both linear solvers are
//! pse-owned and fail through Diffsol's typed step recovery, not by panicking (I9).
use super::*;
use diffsol::{
    ConstantOp, ConstantOpSens, ConstantOpSensAdjoint, FaerContext, FaerSparseMat, FaerVec,
    LinearOp, LinearOpTranspose, LinearSolver, Matrix, NonLinearOp, NonLinearOpAdjoint,
    NonLinearOpJacobian, NonLinearOpSens, NonLinearOpSensAdjoint, OdeBuilder, OdeEquations,
    OdeEquationsRef, OdeSolverMethod, OdeSolverStopReason, Op, Vector, VectorHost,
};

mod adjoint;
pub(super) use adjoint::gradient;
use std::{
    cell::{Cell, RefCell},
    panic::{AssertUnwindSafe, catch_unwind, resume_unwind},
    rc::Rc,
    sync::atomic::Ordering,
    time::Instant,
};
type V = FaerVec<f64>;
type M = FaerSparseMat<f64>;
type Pattern = faer::sparse::SymbolicSparseColMat<usize>;
#[derive(Debug)]
struct Abort;
struct Shared<'o> {
    oracle: RefCell<&'o mut dyn Oracle>,
    contract: Contract,
    /// The integration parameter vector: unscheduled parameters, then every interval of
    /// every scheduled input (I6). Diffsol's parameters and sensitivity columns.
    parameters: RefCell<Vec<f64>>,
    /// The integration column of each contract parameter in the current segment.
    columns: RefCell<Vec<usize>>,
    mode: Cell<usize>,
    integrals: RefCell<Vec<f64>>,
    seed: RefCell<Option<Vec<f64>>>,
    transition: RefCell<Option<Transition>>,
    seed_sens: RefCell<Option<Vec<V>>>,
    failure: RefCell<Option<(Termination, ProblemError)>>,
    cancel: Cancellation,
    deadline: Instant,
    started: Instant,
    progress: Arc<crate::solve::Progress>,
    context: FaerContext,
}
impl Shared<'_> {
    fn abort(&self, status: Termination, error: ProblemError) -> ! {
        *self.failure.borrow_mut() = Some((status, error));
        resume_unwind(Box::new(Abort))
    }
    fn check(&self) {
        if self.cancel.load(Ordering::Acquire) {
            self.abort(
                Termination::Cancelled,
                pse_math::MathError::Cancelled.into(),
            );
        }
        if Instant::now() >= self.deadline {
            self.abort(
                Termination::TimeLimit,
                ProblemError::Limit {
                    kind: crate::LimitKind::Time,
                    detail: "dynamic deadline".into(),
                },
            );
        }
    }
    fn evaluate(&self, f: Function, t: f64, x: &[f64], derivative: bool) -> Evaluation {
        self.evaluate_mode(self.mode.get(), f, t, x, derivative)
    }
    /// The contract parameter values of the current segment.
    fn values(&self) -> Vec<f64> {
        let parameters = self.parameters.borrow();
        self.columns
            .borrow()
            .iter()
            .map(|c| parameters.get(*c).copied().unwrap_or(f64::NAN))
            .collect()
    }
    /// The integration parameter count: Diffsol's parameter and sensitivity width.
    fn width(&self) -> usize {
        self.parameters.borrow().len()
    }
    fn evaluate_mode(
        &self,
        mode: usize,
        f: Function,
        t: f64,
        x: &[f64],
        derivative: bool,
    ) -> Evaluation {
        self.check();
        let parameters = self.values();
        let result = self
            .oracle
            .borrow_mut()
            .evaluate(mode, f, t, x, &parameters, derivative);
        match result {
            Ok(v) => {
                let n = self.nout_mode(mode, f);
                if v.values.len() != n
                    || derivative
                        && v.jacobian.as_ref().is_none_or(|j| {
                            j.nrows() != n
                                || j.ncols()
                                    != self.contract.states.len() + self.contract.parameters.len()
                        })
                {
                    self.abort(
                        Termination::Failed,
                        ProblemError::internal("dynamic function value/derivative dimensions"),
                    );
                }
                if v.values.iter().any(|x| !x.is_finite())
                    || derivative
                        && v.jacobian
                            .as_ref()
                            .is_some_and(|j| j.val().iter().any(|v| !v.is_finite()))
                {
                    self.abort(
                        Termination::Failed,
                        ProblemError::numerical(
                            "dynamic function value or derivative is nonfinite",
                        ),
                    );
                }
                v
            }
            Err(e) => self.abort(Termination::Failed, e),
        }
    }
    fn inventories(&self, time: f64, state: &[f64]) -> Vec<f64> {
        if self.contract.balances.is_empty() {
            vec![]
        } else {
            self.evaluate(Function::Inventory, time, state, false)
                .values
        }
    }
    fn conserve(
        &self,
        report: &mut Report,
        time: f64,
        state: &[f64],
        integrals: Vec<f64>,
    ) -> Result<(), ProblemError> {
        let transfers = cumulative_transfers(&self.contract, report);
        observe_conservation(
            &self.contract,
            report,
            time,
            self.mode.get(),
            self.inventories(time, state),
            integrals,
            transfers,
        )
    }
    fn transition(
        &self,
        report: &Report,
        event: Option<usize>,
        time: f64,
        state: &[f64],
    ) -> Result<(), ProblemError> {
        let mut transfers = vec![0.0; self.contract.balances.len()];
        if let Some(index) = event {
            let id = self.contract.events[self.mode.get()][index].id;
            if self
                .contract
                .balances
                .iter()
                .any(|b| b.transfers.contains(&id))
            {
                transfers = self
                    .evaluate(Function::Transfer(index), time, state, false)
                    .values;
                for (value, balance) in transfers.iter_mut().zip(&self.contract.balances) {
                    if !balance.transfers.contains(&id) && *value != 0.0 {
                        return Err(contract(
                            "event transfer supplied for an unauthorized conserved subject",
                        ));
                    }
                }
            }
        }
        *self.transition.borrow_mut() = Some(Transition {
            record: report.events.len() - 1,
            inventories: self.inventories(time, state),
            transfers,
        });
        Ok(())
    }
    fn nout_mode(&self, mode: usize, f: Function) -> usize {
        match f {
            Function::QuadratureFlux => self.contract.quadratures.len(),
            Function::Output => self.contract.outputs.len(),
            Function::Inventory | Function::Transfer(_) => self.contract.balances.len(),
            Function::Roots => self.contract.events[mode].len(),
            _ => self.contract.states.len(),
        }
    }
}
#[derive(Clone)]
struct Operator<'o> {
    shared: Rc<Shared<'o>>,
    function: Function,
    mode: usize,
    state_pattern: Pattern,
    /// Partials in the segment's integration columns: each contract parameter column is
    /// relabeled to its interval's column; the other intervals' columns are empty.
    parameter_pattern: Pattern,
    /// The integration column of each contract parameter in this operator's segment.
    columns: Vec<usize>,
    /// The contract parameter of each integration column in this segment, if any.
    contract: Vec<Option<usize>>,
    direction: RefCell<Vec<f64>>,
    /// The transposed patterns of the adjoint operators −Jₓᵀ and −Jₚᵀ.
    state_adjoint_pattern: Pattern,
    parameter_adjoint_pattern: Pattern,
    /// One transposed-product buffer over the state followed by the contract parameters.
    transposed: RefCell<Vec<f64>>,
}
impl<'o> Operator<'o> {
    fn new(shared: Rc<Shared<'o>>, function: Function) -> Result<Self, ProblemError> {
        let mode = shared.mode.get();
        Self::for_mode(shared, function, mode)
    }
    fn for_mode(
        shared: Rc<Shared<'o>>,
        function: Function,
        mode: usize,
    ) -> Result<Self, ProblemError> {
        let n = shared.contract.states.len();
        let np = shared.contract.parameters.len();
        let width = shared.width();
        let columns = shared.columns.borrow().clone();
        let mut contract_columns = vec![None; width];
        for (k, c) in columns.iter().enumerate() {
            if let Some(slot) = contract_columns.get_mut(*c) {
                *slot = Some(k);
            }
        }
        if columns.len() != np || contract_columns.iter().flatten().count() != np {
            return Err(ProblemError::internal("dynamic segment parameter columns"));
        }
        let m = shared.nout_mode(mode, function);
        // The oracle's typed support becomes faer positions here.
        let mut pairs = shared
            .oracle
            .borrow()
            .support(mode, function)
            .into_iter()
            .map(|entry| (entry.row.get(), entry.col.get()))
            .collect::<Vec<_>>();
        if function == Function::Initial {
            pairs.extend((0..n).flat_map(|r| (n..n + np).map(move |c| (r, c))));
        }
        if pairs.iter().any(|&(r, c)| r >= m || c >= n + np) {
            return Err(ProblemError::internal("dynamic support bounds"));
        }
        // Contract parameter columns become the segment's integration columns.
        for pair in &mut pairs {
            if pair.1 >= n {
                pair.1 = n + columns[pair.1 - n];
            }
        }
        let pattern = |parameter: bool| {
            let indices: Vec<_> = pairs
                .iter()
                .filter_map(|&(r, c)| {
                    if parameter && c >= n {
                        Some(faer::sparse::Pair::new(r, c - n))
                    } else if !parameter && c < n {
                        Some(faer::sparse::Pair::new(r, c))
                    } else {
                        None
                    }
                })
                .collect();
            Pattern::try_new_from_indices(m, if parameter { width } else { n }, &indices)
                .map(|v| v.0)
                .map_err(|e| ProblemError::internal(format!("dynamic support pattern: {e}")))
        };
        let state_pattern = pattern(false)?;
        let parameter_pattern = pattern(true)?;
        let transpose = |p: &Pattern| {
            p.as_ref()
                .transpose()
                .to_col_major()
                .map_err(|e| ProblemError::memory(format!("dynamic adjoint pattern: {e}")))
        };
        Ok(Self {
            state_adjoint_pattern: transpose(&state_pattern)?,
            parameter_adjoint_pattern: transpose(&parameter_pattern)?,
            state_pattern,
            parameter_pattern,
            columns,
            contract: contract_columns,
            direction: RefCell::new(vec![0.0; n + np]),
            transposed: RefCell::new(vec![0.0; n + np]),
            shared,
            function,
            mode,
        })
    }
    /// The raw partials at `(x, t)`, which Diffsol's callbacks cannot fail: a missing
    /// Jacobian exits through the owned abort.
    fn raw_partials(&self, x: &V, t: f64) -> faer::sparse::SparseColMat<usize, f64> {
        let result = self
            .shared
            .evaluate_mode(self.mode, self.function, t, x.as_slice(), true);
        match result.jacobian {
            Some(j) => j,
            None => self.shared.abort(
                Termination::Failed,
                ProblemError::internal("missing dynamic partials"),
            ),
        }
    }
    /// −Jᵀv from the oracle's one CSC by a faer transpose product: over the state, or over
    /// the segment's integration columns, where a column this segment maps no contract
    /// parameter to is zero.
    fn transpose_product(&self, x: &V, t: f64, v: &V, y: &mut V, parameter: bool) {
        let j = self.raw_partials(x, t);
        let n = self.nstates();
        let mut product = self.transposed.borrow_mut();
        let width = product.len();
        faer::sparse::linalg::matmul::sparse_dense_matmul(
            faer::MatMut::from_column_major_slice_mut(&mut product, width, 1),
            faer::Accum::Replace,
            j.as_ref().transpose(),
            faer::MatRef::from_column_major_slice(v.as_slice(), v.len(), 1),
            -1.0,
            faer::Par::Seq,
        );
        if parameter {
            for (k, slot) in y.as_mut_slice().iter_mut().enumerate() {
                *slot = self
                    .contract
                    .get(k)
                    .copied()
                    .flatten()
                    .map_or(0.0, |c| product[n + c]);
            }
        } else {
            y.as_mut_slice().copy_from_slice(&product[..n]);
        }
    }
    /// −Jᵀ in its transposed pattern: each column is a function row, each row a state or
    /// an integration column of the segment.
    fn transpose_partials(&self, x: &V, t: f64, matrix: &mut M, parameter: bool) {
        let j = self.raw_partials(x, t);
        let n = self.nstates();
        let target = matrix.inner_mut();
        for c in 0..target.ncols() {
            for k in target.col_range(c) {
                let r = target.symbolic().row_idx()[k];
                let source = if parameter {
                    self.contract.get(r).copied().flatten().map(|p| n + p)
                } else {
                    Some(r)
                };
                target.val_mut()[k] = source
                    .and_then(|source| j.get(c, source).copied())
                    .map_or(0.0, |v| -v);
            }
        }
    }
    fn partials(&self, x: &V, t: f64, matrix: &mut M, parameter: bool) {
        let j = self.raw_partials(x, t);
        let n = self.nstates();
        let target = matrix.inner_mut();
        for c in 0..target.ncols() {
            // A parameter column reads its contract parameter's partials, if this segment
            // maps one to it.
            let source = if parameter {
                self.contract.get(c).copied().flatten().map(|k| n + k)
            } else {
                Some(c)
            };
            for k in target.col_range(c) {
                let r = target.symbolic().row_idx()[k];
                target.val_mut()[k] = source
                    .and_then(|source| j.get(r, source).copied())
                    .unwrap_or(0.0);
            }
        }
    }
    fn product(&self, x: &V, t: f64, v: &V, y: &mut V, parameter: bool) {
        let j = self.raw_partials(x, t);
        // Keep one direction buffer per operator attempt. faer multiplies the
        // full admitted CSC matrix; a fresh Diffsol matrix is unnecessary.
        let mut direction = self.direction.borrow_mut();
        direction.fill(0.0);
        let n = self.nstates();
        if parameter {
            for (k, c) in self.columns.iter().enumerate() {
                direction[n + k] = v[*c];
            }
        } else {
            direction[..n].copy_from_slice(v.as_slice());
        }
        let nout = self.nout();
        faer::sparse::linalg::matmul::sparse_dense_matmul(
            faer::MatMut::from_column_major_slice_mut(y.as_mut_slice(), nout, 1),
            faer::Accum::Replace,
            j.as_ref(),
            faer::MatRef::from_column_major_slice(&direction, direction.len(), 1),
            1.0,
            faer::Par::Seq,
        );
    }
}
impl Op for Operator<'_> {
    type T = f64;
    type V = V;
    type M = M;
    type C = FaerContext;
    fn context(&self) -> &FaerContext {
        &self.shared.context
    }
    fn nstates(&self) -> usize {
        self.shared.contract.states.len()
    }
    fn nparams(&self) -> usize {
        self.shared.width()
    }
    fn nout(&self) -> usize {
        self.shared.nout_mode(self.mode, self.function)
    }
}
impl NonLinearOp for Operator<'_> {
    fn call_inplace(&self, x: &V, t: f64, y: &mut V) {
        y.as_mut_slice().copy_from_slice(
            &self
                .shared
                .evaluate_mode(self.mode, self.function, t, x.as_slice(), false)
                .values,
        );
    }
}
impl NonLinearOpJacobian for Operator<'_> {
    fn jac_mul_inplace(&self, x: &V, t: f64, v: &V, y: &mut V) {
        self.product(x, t, v, y, false);
    }
    fn jacobian_inplace(&self, x: &V, t: f64, y: &mut M) {
        self.partials(x, t, y, false);
    }
    fn jacobian_sparsity(&self) -> Option<Pattern> {
        Some(self.state_pattern.clone())
    }
}
impl NonLinearOpSens for Operator<'_> {
    fn sens_mul_inplace(&self, x: &V, t: f64, v: &V, y: &mut V) {
        self.product(x, t, v, y, true);
    }
    fn sens_inplace(&self, x: &V, t: f64, y: &mut M) {
        self.partials(x, t, y, true);
    }
    fn sens_sparsity(&self) -> Option<Pattern> {
        Some(self.parameter_pattern.clone())
    }
}
/// The adjoint operators, −Jₓᵀ and −Jₚᵀ, from the same CSC as the forward products; the
/// matrix forms fill the transposed pattern directly instead of Diffsol's default of one
/// product per column (ADR-0110 item 3).
impl NonLinearOpAdjoint for Operator<'_> {
    fn jac_transpose_mul_inplace(&self, x: &V, t: f64, v: &V, y: &mut V) {
        self.transpose_product(x, t, v, y, false);
    }
    fn adjoint_inplace(&self, x: &V, t: f64, y: &mut M) {
        self.transpose_partials(x, t, y, false);
    }
    fn adjoint_sparsity(&self) -> Option<Pattern> {
        Some(self.state_adjoint_pattern.clone())
    }
}
impl NonLinearOpSensAdjoint for Operator<'_> {
    fn sens_transpose_mul_inplace(&self, x: &V, t: f64, v: &V, y: &mut V) {
        self.transpose_product(x, t, v, y, true);
    }
    fn sens_adjoint_inplace(&self, x: &V, t: f64, y: &mut M) {
        self.transpose_partials(x, t, y, true);
    }
    fn sens_adjoint_sparsity(&self) -> Option<Pattern> {
        Some(self.parameter_adjoint_pattern.clone())
    }
}
/// −(∂x₀/∂p)ᵀv of the initial values. A seeded segment starts from the previous segment's
/// state, whose parameter dependence the adjoint state carries across the boundary, so its
/// own initial values contribute nothing.
impl ConstantOpSensAdjoint for Operator<'_> {
    fn sens_transpose_mul_inplace(&self, t: f64, v: &V, y: &mut V) {
        if self.shared.seed.borrow().is_some() {
            y.as_mut_slice().fill(0.0);
            return;
        }
        self.transpose_product(
            &V::zeros(self.nstates(), self.shared.context),
            t,
            v,
            y,
            true,
        );
    }
    fn sens_adjoint_sparsity(&self) -> Option<Pattern> {
        Some(self.parameter_adjoint_pattern.clone())
    }
}
impl ConstantOp for Operator<'_> {
    fn call_inplace(&self, t: f64, y: &mut V) {
        if let Some(seed) = self.shared.seed.borrow().as_ref() {
            y.as_mut_slice().copy_from_slice(seed);
        } else {
            let x = vec![0.0; self.nstates()];
            y.as_mut_slice()
                .copy_from_slice(&self.shared.evaluate(Function::Initial, t, &x, false).values);
        }
    }
}
impl ConstantOpSens for Operator<'_> {
    fn sens_mul_inplace(&self, t: f64, v: &V, y: &mut V) {
        if let Some(seed) = self.shared.seed_sens.borrow().as_ref() {
            for i in 0..self.nstates() {
                y[i] = seed.iter().enumerate().map(|(j, s)| s[i] * v[j]).sum();
            }
            return;
        }
        self.product(
            &V::zeros(self.nstates(), self.shared.context),
            t,
            v,
            y,
            true,
        );
    }
    fn sens_inplace(&self, t: f64, y: &mut M) {
        if let Some(seed) = self.shared.seed_sens.borrow().as_ref() {
            let target = y.inner_mut();
            for (c, col) in seed.iter().enumerate() {
                for k in target.col_range(c) {
                    let r = target.symbolic().row_idx()[k];
                    target.val_mut()[k] = col[r];
                }
            }
            return;
        }
        self.partials(&V::zeros(self.nstates(), self.shared.context), t, y, true);
    }
    fn sens_sparsity(&self) -> Option<Pattern> {
        Some(self.parameter_pattern.clone())
    }
}
#[derive(Clone)]
struct Mass<'o>(Rc<Shared<'o>>);
impl Op for Mass<'_> {
    type T = f64;
    type V = V;
    type M = M;
    type C = FaerContext;
    fn context(&self) -> &FaerContext {
        &self.0.context
    }
    fn nstates(&self) -> usize {
        self.0.contract.states.len()
    }
    fn nparams(&self) -> usize {
        self.0.width()
    }
    fn nout(&self) -> usize {
        self.nstates()
    }
}
impl LinearOp for Mass<'_> {
    /// `y = M·x + β·y`; with β = 0 the previous `y` is not read, as in BLAS.
    fn gemv_inplace(&self, x: &V, _t: f64, beta: f64, y: &mut V) {
        for (i, d) in self.0.contract.differential.iter().enumerate() {
            let mass = if *d { x[i] } else { 0.0 };
            y[i] = if beta == 0.0 {
                mass
            } else {
                mass + beta * y[i]
            };
        }
    }
    fn sparsity(&self) -> Option<Pattern> {
        Some(
            <Pattern as diffsol::matrix::sparsity::MatrixSparsity<M>>::new_diagonal(self.nstates()),
        )
    }
}
/// The fixed diagonal mass is its own transpose.
impl LinearOpTranspose for Mass<'_> {
    fn gemv_transpose_inplace(&self, x: &V, t: f64, beta: f64, y: &mut V) {
        self.gemv_inplace(x, t, beta, y);
    }
    fn transpose_inplace(&self, _t: f64, y: &mut M) {
        let target = y.inner_mut();
        // The contract has one mass entry per state, so this spans every column.
        for (c, &differential) in self.0.contract.differential.iter().enumerate() {
            for k in target.col_range(c) {
                let r = target.symbolic().row_idx()[k];
                target.val_mut()[k] = f64::from(r == c && differential);
            }
        }
    }
    fn transpose_sparsity(&self) -> Option<Pattern> {
        self.sparsity()
    }
}
struct Equation<'o> {
    rhs: Operator<'o>,
    init: Operator<'o>,
    out: Operator<'o>,
    root: Operator<'o>,
    mass: Mass<'o>,
    reset: Option<Operator<'o>>,
}
impl Op for Equation<'_> {
    type T = f64;
    type V = V;
    type M = M;
    type C = FaerContext;
    fn context(&self) -> &FaerContext {
        self.rhs.context()
    }
    fn nstates(&self) -> usize {
        self.rhs.nstates()
    }
    fn nparams(&self) -> usize {
        self.rhs.nparams()
    }
    fn nout(&self) -> usize {
        self.out.nout()
    }
}
impl<'a, 'o> OdeEquationsRef<'a> for Equation<'o> {
    type Mass = Mass<'o>;
    type Rhs = Operator<'o>;
    type Root = Operator<'o>;
    type Init = Operator<'o>;
    type Out = Operator<'o>;
    type Reset = Operator<'o>;
}
impl<'o> OdeEquations for Equation<'o> {
    fn rhs(&self) -> Operator<'o> {
        self.rhs.clone()
    }
    fn init(&self) -> Operator<'o> {
        self.init.clone()
    }
    fn out(&self) -> Option<Operator<'o>> {
        Some(self.out.clone())
    }
    fn root(&self) -> Option<Operator<'o>> {
        (self.root.nout() > 0).then(|| self.root.clone())
    }
    fn reset(&self) -> Option<Operator<'o>> {
        self.reset.clone()
    }
    fn mass(&self) -> Option<Mass<'o>> {
        self.mass
            .0
            .contract
            .differential
            .contains(&false)
            .then(|| self.mass.clone())
    }
    fn set_params(&mut self, p: &V) {
        *self.rhs.shared.parameters.borrow_mut() = p.as_slice().to_vec();
    }
    fn get_params(&self, p: &mut V) {
        p.as_mut_slice()
            .copy_from_slice(&self.rhs.shared.parameters.borrow());
    }
}
/// Classify Diffsol's typed errors by cause. The matches are exhaustive over the
/// pinned enums, so an upgrade that adds a variant fails to compile (F06, F10).
fn native(error: diffsol::DiffsolError) -> ProblemError {
    use diffsol::{
        DiffsolError as D,
        error::{NonLinearSolverError as N, OdeSolverError as O},
    };
    let detail = format!("Diffsol {error}");
    match &error {
        D::LaError(_) => ProblemError::numerical(detail),
        D::NonLinearSolverError(e) => match e {
            N::InitialConditionDidNotConverge
            | N::NewtonMaxIterations
            | N::NewtonDiverged
            | N::LinesearchFailedMaxIterations
            | N::LinesearchFailedMinStep
            | N::LuSolveFailed
            | N::Other(_) => ProblemError::numerical(detail),
            N::JacobianNotReset | N::WrongStateLength { .. } => ProblemError::internal(detail),
        },
        D::OdeSolverError(e) => match e {
            O::TooManyNonlinearSolverFailures { .. }
            | O::TooManyErrorTestFailures { .. }
            | O::StepSizeTooSmall { .. }
            | O::SensitivitySolveFailed
            | O::SundialsError(_) => ProblemError::numerical(detail),
            O::MassMatrixNotSupported
            | O::SensitivityNotSupported
            | O::ResetRequiresRootOperator
            | O::JacobianNotAvailable => ProblemError::unsupported(detail),
            O::StopTimeBeforeCurrentTime { .. }
            | O::StopTimeAtCurrentTime
            | O::InterpolationVectorWrongSize { .. }
            | O::SensitivityCountMismatch { .. }
            | O::InterpolationTimeAfterCurrentTime
            | O::InterpolationTimeOutsideCurrentStep
            | O::InterpolationTimeGreaterThanCurrentTime
            | O::StateNotSet
            | O::FailedToGetMutableReference
            | O::BuilderError(_)
            | O::StateProblemMismatch
            | O::InvalidTEval
            | O::ProblemNotSet
            | O::InvalidTableau(_)
            | O::Other(_) => ProblemError::internal(detail),
        },
        D::DiffslParserError(_) | D::DiffslCompilerError(_) | D::Other(_) => {
            ProblemError::internal(detail)
        }
    }
}

/// Same owned integration using the caller's bounded progress source.
pub(super) fn integrate_with_progress(
    oracle: &mut dyn Oracle,
    profile: &Profile,
    parameters: &[f64],
    cancel: Cancellation,
    progress: Arc<crate::solve::Progress>,
) -> Result<Report, ProblemError> {
    profile.validate(oracle.contract(), parameters)?;
    let shared = Shared::new(oracle, profile, parameters, cancel, progress)?;
    let mut report = Report::new(profile.start);
    let result = catch_unwind(AssertUnwindSafe(|| match profile.diffsol.linear {
        DiffsolLinear::FaerLu => run::<linear::FaerLu>(shared.clone(), profile, &mut report),
        DiffsolLinear::Klu => run::<linear::Klu>(shared.clone(), profile, &mut report),
    }));
    shared.contain(result, &mut report);
    Ok(report)
}
impl<'o> Shared<'o> {
    /// The owned context of one integration attempt, starting in the first segment.
    fn new(
        oracle: &'o mut dyn Oracle,
        profile: &Profile,
        parameters: &[f64],
        cancel: Cancellation,
        progress: Arc<crate::solve::Progress>,
    ) -> Result<Rc<Self>, ProblemError> {
        let contract_value = oracle.contract().clone();
        Ok(Rc::new(Shared {
            oracle: RefCell::new(oracle),
            columns: RefCell::new(
                profile.columns_at(contract_value.parameters.len(), profile.start),
            ),
            integrals: RefCell::new(vec![0.0; contract_value.quadratures.len()]),
            contract: contract_value,
            parameters: RefCell::new(parameters.to_vec()),
            mode: Cell::new(0),
            seed: RefCell::new(None),
            transition: RefCell::new(None),
            seed_sens: RefCell::new(None),
            failure: RefCell::new(None),
            cancel,
            deadline: Instant::now()
                .checked_add(profile.time_limit)
                .ok_or_else(|| ProblemError::unsupported("dynamic deadline overflow"))?,
            started: Instant::now(),
            progress,
            context: FaerContext {
                par: faer::Par::Seq,
            },
        }))
    }
    /// The boundary exits Diffsol's infallible operator callbacks with the recorded typed
    /// failure (`Shared::abort`) and contains any other library panic. A failed Newton
    /// factorization never reaches it: the linear solvers return it as an error.
    fn contain(&self, result: std::thread::Result<Result<(), ProblemError>>, report: &mut Report) {
        match result {
            Ok(Ok(())) => {}
            Ok(Err(e)) => {
                report.termination = Termination::Failed;
                report.error = Some(e);
            }
            Err(payload) => {
                if payload.is::<Abort>() {
                    if let Some((status, error)) = self.failure.borrow_mut().take() {
                        report.termination = status;
                        report.error = Some(error);
                    } else {
                        report.termination = Termination::Panic;
                        report.error = Some(ProblemError::internal("unattributed dynamic abort"));
                    }
                } else {
                    report.termination = Termination::Panic;
                    report.error = Some(ProblemError::internal("panic inside Diffsol operation"));
                }
            }
        }
        (report.progress, report.dropped_progress) = self.progress.snapshot();
    }
}
/// Statistics of one finished segment, labelled with the scheme and linear solver.
fn segment_statistics(
    statistics: &diffsol::ode_solver::OdeSolverStatistics,
    settings: &DiffsolSettings,
) -> Result<serde_json::Value, ProblemError> {
    let mut value = serde_json::to_value(statistics)
        .map_err(|e| ProblemError::internal(format!("Diffsol statistics: {e}")))?;
    if let Some(fields) = value.as_object_mut() {
        fields.insert(
            "diffsol".into(),
            serde_json::to_value(settings)
                .map_err(|e| ProblemError::internal(format!("Diffsol settings: {e}")))?,
        );
    }
    Ok(value)
}
/// One segment's problem from `time`, whose operators hold the segment's column map.
fn build_problem<'o>(
    shared: &Rc<Shared<'o>>,
    p: &Profile,
    time: f64,
    scales: &[f64],
) -> Result<diffsol::OdeSolverProblem<Equation<'o>>, ProblemError> {
    let eq = Equation {
        rhs: Operator::new(shared.clone(), Function::Rhs)?,
        init: Operator::new(shared.clone(), Function::Initial)?,
        out: Operator::new(
            shared.clone(),
            if shared.contract.quadratures.is_empty() {
                Function::Output
            } else {
                Function::QuadratureFlux
            },
        )?,
        root: Operator::new(shared.clone(), Function::Roots)?,
        mass: Mass(shared.clone()),
        reset: None,
    };
    let params = shared.parameters.borrow().clone();
    let mut builder = OdeBuilder::<M>::new()
        .context(shared.context)
        .t0(time)
        .h0(p.initial_step)
        .rtol(p.rtol)
        .atol(p.atol.clone())
        .p(params)
        .param_scales(scales.to_vec());
    if let Some(tolerance) = p.out_rtol {
        builder = builder
            .integrate_out(true)
            .out_rtol(tolerance)
            .out_atol(p.out_atol.clone());
    }
    if p.forward() {
        builder = builder.sens_rtol(p.rtol).sens_atol(p.atol.clone());
    }
    if p.sensitivity == DynamicSensitivity::Adjoint {
        let atol = p.atol.iter().copied().fold(f64::INFINITY, f64::min);
        builder = builder
            .param_rtol(p.rtol)
            .param_atol(vec![atol; scales.len()]);
    }
    let mut problem = builder.build_from_eqn(eq).map_err(native)?;
    problem.ic_options = copy_initial(&p.initialization);
    problem.ode_options = copy_native(&p.native);
    Ok(problem)
}
fn run<LS: LinearSolver<M>>(
    shared: Rc<Shared<'_>>,
    p: &Profile,
    r: &mut Report,
) -> Result<(), ProblemError> {
    let mut time = p.start;
    // Scheduled-input changes split the horizon into segments; every segment integrates
    // the same integration parameters with its own column map (I6).
    let boundaries = p.boundaries();
    let scales = p.integration_parameters(&p.parameter_scales);
    let np = shared.contract.parameters.len();
    let mut segment = 0;
    let mut steps = 0usize;
    loop {
        shared.check();
        let problem = build_problem(&shared, p, time, &scales)?;
        let requested = ConstantOp::call(&problem.eqn.init(), time)
            .as_slice()
            .to_vec();
        if r.requested_initial.is_empty() {
            r.requested_initial = requested;
        }
        let stop = boundaries.get(segment).copied().unwrap_or(p.end);
        // One segment on the selected library scheme (ADR-0110 item 2). Every scheme
        // shares root finding, interpolation, output quadrature and reset sensitivities.
        // The segment's own unwind catch records its statistics before an abort continues
        // to the outer boundary.
        macro_rules! segment {
            ($solver:expr, sensitivities) => {{
                let mut solver = $solver.map_err(native)?;
                record_start(&shared, &solver, p, r, time)?;
                if time >= p.end && boundaries.get(segment) != Some(&time) {
                    capture_endpoint(&shared, r, time, solver.state().y.as_slice(), None)?;
                    r.termination = Termination::Completed;
                    return Ok(());
                }
                let attempt = catch_unwind(AssertUnwindSafe(|| {
                    if stop == time {
                        return Ok((None, solver.state().y.as_slice().to_vec()));
                    }
                    drive(&shared, &mut solver, p, r, (stop, &boundaries), &mut steps)
                }));
                r.statistics
                    .push(segment_statistics(solver.get_statistics(), &p.diffsol)?);
                if let Some(statistics) = r
                    .statistics
                    .last_mut()
                    .and_then(serde_json::Value::as_object_mut)
                {
                    statistics.insert("sensitivity_partials".into(), serde_json::json!({"state":"analytic", "parameter":"analytic", "event_time":"Diffsol finite-difference root/reset time partials", "reset":"Diffsol event-time correction and consistent mass reset"}));
                }
                let result = match attempt {
                    Ok(result) => result?,
                    Err(payload) => resume_unwind(payload),
                };
                if let Some(index) = result.0 {
                    let event = &shared.contract.events[shared.mode.get()][index];
                    if !event.terminal {
                        reset_sens::<LS, _>(&shared, &mut solver, p, &scales, index)?;
                        *shared.seed.borrow_mut() = Some(solver.state().y.as_slice().to_vec());
                    }
                }
                *shared.seed_sens.borrow_mut() = Some(solver.state().s.to_vec());
                result
            }};
            ($solver:expr) => {{
                let mut solver = $solver.map_err(native)?;
                record_start(&shared, &solver, p, r, time)?;
                if time >= p.end && boundaries.get(segment) != Some(&time) {
                    capture_endpoint(&shared, r, time, solver.state().y.as_slice(), None)?;
                    r.termination = Termination::Completed;
                    return Ok(());
                }
                let attempt = catch_unwind(AssertUnwindSafe(|| {
                    if stop == time {
                        return Ok((None, solver.state().y.as_slice().to_vec()));
                    }
                    drive(&shared, &mut solver, p, r, (stop, &boundaries), &mut steps)
                }));
                r.statistics
                    .push(segment_statistics(solver.get_statistics(), &p.diffsol)?);
                match attempt {
                    Ok(result) => result?,
                    Err(payload) => resume_unwind(payload),
                }
            }};
        }
        let (event, state) = match (p.forward(), p.diffsol.method) {
            (true, DiffsolMethod::Bdf) => {
                let solver = problem.bdf_sens::<LS>().map(|mut solver| {
                    // Diffsol 0.16.2 leaves the forward sensitivity BDF coefficient
                    // at zero in its constructor. Its public state mutation path
                    // rebuilds both main and sensitivity coefficients before stepping,
                    // using the already consistent state/rates and unchanged controls.
                    let _ = solver.state_mut();
                    solver
                });
                segment!(solver, sensitivities)
            }
            (true, DiffsolMethod::TrBdf2) => {
                segment!(problem.tr_bdf2_sens::<LS>(), sensitivities)
            }
            (true, DiffsolMethod::Esdirk34) => {
                segment!(problem.esdirk34_sens::<LS>(), sensitivities)
            }
            (true, DiffsolMethod::Tsit45) => segment!(problem.tsit45_sens(), sensitivities),
            (false, DiffsolMethod::Bdf) => segment!(problem.bdf::<LS>()),
            (false, DiffsolMethod::TrBdf2) => segment!(problem.tr_bdf2::<LS>()),
            (false, DiffsolMethod::Esdirk34) => segment!(problem.esdirk34::<LS>()),
            (false, DiffsolMethod::Tsit45) => segment!(problem.tsit45()),
        };
        if matches!(
            r.termination,
            Termination::StepLimit | Termination::EventLimit
        ) {
            return Ok(());
        }
        time = r.completed_time;
        shared.conserve(r, time, &state, shared.integrals.borrow().clone())?;
        if let Some(index) = event {
            let guards = shared.evaluate(Function::Roots, time, &state, false).values;
            let events = &shared.contract.events[shared.mode.get()];
            if guards_at_zero(&guards, events) > 1 {
                return Err(contract("ambiguous simultaneous dynamic events"));
            }
            let e = events
                .get(index)
                .ok_or_else(|| ProblemError::internal("native root index"))?
                .clone();
            if r.events.len() >= p.max_events {
                r.termination = Termination::EventLimit;
                return Ok(());
            }
            r.events.push(EventRecord {
                event: Some(e.id),
                time,
                before: state.clone(),
                after: None,
            });
            if e.terminal {
                capture_endpoint(&shared, r, time, &state, Some(e.id))?;
                if p.samples.get(r.samples.len()).is_some_and(|t| *t == time) {
                    let outputs = shared
                        .evaluate(Function::Output, time, &state, false)
                        .values;
                    r.samples.push(Sample {
                        mode: shared.mode.get(),
                        integrals: shared.integrals.borrow().clone(),
                        time,
                        state: state.clone(),
                        outputs,
                        state_sensitivities: vec![],
                        output_sensitivities: vec![],
                    });
                }
                r.termination = Termination::Event;
                return Ok(());
            }
            shared.transition(r, Some(index), time, &state)?;
            if !p.forward() {
                *shared.seed.borrow_mut() = Some(
                    shared
                        .evaluate(Function::Reset(index), time, &state, false)
                        .values,
                );
            }
            shared.mode.set(e.next_mode);
            // Settle the reset under its active input segment before any coincident
            // schedule changes are consistently initialized.
            continue;
        } else {
            *shared.seed.borrow_mut() = Some(state.clone());
        }
        // Roots precede a scheduled change at the same native stop time.
        let changed = boundaries.get(segment).is_some_and(|b| *b == time);
        if changed {
            if r.events.len() >= p.max_events {
                r.termination = Termination::EventLimit;
                return Ok(());
            }
            r.events.push(EventRecord {
                event: None,
                time,
                before: shared.seed.borrow().clone().unwrap_or_default(),
                after: None,
            });
            shared.transition(
                r,
                None,
                time,
                &shared.seed.borrow().clone().unwrap_or_default(),
            )?;
            // The integration parameters stay; the next intervals' columns take effect.
            *shared.columns.borrow_mut() = p.columns_at(np, time);
            segment += 1;
        }
        if time >= p.end && event.is_none() && !changed {
            capture_endpoint(&shared, r, time, &state, None)?;
            r.termination = Termination::Completed;
            return Ok(());
        }
    }
}
// The transition equation combines pre-event root/reset derivatives with post-event rates.
// Diffsol owns the saltation and mass-matrix consistency operations.
fn reset_sens<'p, 'o: 'p, LS: LinearSolver<M>, S: OdeSolverMethod<'p, Equation<'o>>>(
    shared: &Rc<Shared<'o>>,
    solver: &mut S,
    p: &Profile,
    scales: &[f64],
    index: usize,
) -> Result<(), ProblemError> {
    let mode = shared.mode.get();
    let event = &shared.contract.events[mode][index];
    let state = solver.state();
    let roots = shared.evaluate(Function::Roots, state.t, state.y.as_slice(), true);
    if guards_at_zero(&roots.values, &shared.contract.events[mode]) != 1 {
        return Err(contract("ambiguous simultaneous sensitivity events"));
    }
    if p.samples
        .iter()
        .any(|t| (*t - state.t).abs() <= 8.0 * f64::EPSILON * (1.0 + state.t.abs()))
    {
        let j = roots
            .jacobian
            .ok_or_else(|| ProblemError::internal("missing root derivatives"))?;
        let columns = shared.columns.borrow().clone();
        for (k, s) in state.s.iter().enumerate() {
            let direct = columns
                .iter()
                .position(|c| *c == k)
                .and_then(|contract| j.get(index, state.y.len() + contract).copied())
                .unwrap_or(0.0);
            let moving = (0..state.y.len())
                .map(|i| j.get(index, i).copied().unwrap_or(0.0) * s[i])
                .sum::<f64>()
                + direct;
            if moving.abs() > 100.0 * f64::EPSILON {
                return Err(contract(
                    "fixed-time observation coincides with a parameter-dependent jump",
                ));
            }
        }
    }
    let eq = Equation {
        rhs: Operator::for_mode(shared.clone(), Function::Rhs, event.next_mode)?,
        root: Operator::for_mode(shared.clone(), Function::Roots, mode)?,
        reset: Some(Operator::for_mode(
            shared.clone(),
            Function::Reset(index),
            mode,
        )?),
        init: Operator::for_mode(shared.clone(), Function::Initial, mode)?,
        out: Operator::for_mode(shared.clone(), Function::Output, event.next_mode)?,
        mass: Mass(shared.clone()),
    };
    let parameters = shared.parameters.borrow().clone();
    let mut transition = OdeBuilder::<M>::new()
        .context(shared.context)
        .t0(state.t)
        .h0(p.initial_step)
        .rtol(p.rtol)
        .atol(p.atol.clone())
        .sens_rtol(p.rtol)
        .sens_atol(p.atol.clone())
        .p(parameters)
        .param_scales(scales.to_vec())
        .build_from_eqn(eq)
        .map_err(native)?;
    transition.ic_options = copy_initial(&p.initialization);
    if shared.contract.differential.contains(&false) {
        solver
            .state_mut()
            .apply_reset_with_sens_mass::<LS, _>(&transition, index)
            .map_err(native)?;
    } else {
        solver
            .state_mut()
            .apply_reset_with_sens(&transition, index)
            .map_err(native)?;
    }
    Ok(())
}
fn record_start<'p, 'o: 'p, S: OdeSolverMethod<'p, Equation<'o>>>(
    shared: &Rc<Shared<'o>>,
    s: &S,
    p: &Profile,
    r: &mut Report,
    time: f64,
) -> Result<(), ProblemError> {
    let state = s.state().y.as_slice().to_vec();
    if r.consistent_initial.is_empty() {
        r.consistent_initial = state.clone();
    }
    let inventories = shared.inventories(time, &state);
    let integrals = shared.integrals.borrow().clone();
    if let Some(transition) = shared.transition.borrow_mut().take() {
        settle_transition(
            &shared.contract,
            r,
            transition,
            ConservationPoint {
                time,
                mode: shared.mode.get(),
                inventories,
                integrals,
                transfers: vec![],
                defects: vec![],
            },
            &state,
        )?;
    } else {
        let transfers = cumulative_transfers(&shared.contract, r);
        observe_conservation(
            &shared.contract,
            r,
            time,
            shared.mode.get(),
            inventories,
            integrals,
            transfers,
        )?;
    }
    {
        let roots = shared.evaluate(Function::Roots, time, &state, false).values;
        if guards_at_zero(&roots, &shared.contract.events[shared.mode.get()]) > 0 {
            return Err(contract("initial or post-reset root is ambiguous"));
        }
    }
    if p.samples.get(r.samples.len()).is_some_and(|t| *t == time)
        && *shared.columns.borrow() == p.columns_at(shared.contract.parameters.len(), time)
    {
        r.samples.push(sample(shared, s, time, p.forward())?);
    }
    r.completed_time = time;
    Ok(())
}
fn drive<'p, 'o: 'p, S: OdeSolverMethod<'p, Equation<'o>>>(
    shared: &Rc<Shared<'o>>,
    s: &mut S,
    p: &Profile,
    r: &mut Report,
    (stop, boundaries): (f64, &[f64]),
    steps: &mut usize,
) -> Result<(Option<usize>, Vec<f64>), ProblemError> {
    s.set_stop_time(stop).map_err(native)?;
    loop {
        shared.check();
        if *steps >= p.max_steps {
            r.termination = Termination::StepLimit;
            return Ok((None, s.state().y.as_slice().to_vec()));
        }
        *steps += 1;
        let reason = s.step().map_err(native)?;
        let (time, root) = match reason {
            OdeSolverStopReason::RootFound(t, i) => (t, Some(i)),
            _ => (s.state().t, None),
        };
        while let Some(&t) = p.samples.get(r.samples.len()) {
            // A sample at a root or a scheduled change observes the post-transition state.
            if t > time || (t == time && (root.is_some() || boundaries.contains(&t))) {
                break;
            }
            let point = sample(shared, s, t, p.forward())?;
            shared.conserve(r, t, &point.state, point.integrals.clone())?;
            r.samples.push(point);
        }
        r.completed_time = time;
        shared.progress.push(crate::solve::Event {
            phase: "simulation.step".into(),
            elapsed: shared.started.elapsed(),
            values: std::collections::BTreeMap::from([
                ("time".into(), crate::solve::Metric::Real(time)),
                ("steps".into(), crate::solve::Metric::Integer(*steps as i64)),
            ]),
            incumbent: None,
        });
        if root.is_some() {
            s.state_mut_back(time).map_err(native)?;
        }
        if root.is_some() || matches!(reason, OdeSolverStopReason::TstopReached) {
            if !shared.contract.quadratures.is_empty() {
                let g = s.state().g;
                for (total, value) in shared.integrals.borrow_mut().iter_mut().zip(g.as_slice()) {
                    *total += value;
                }
            }
            return Ok((root, s.state().y.as_slice().to_vec()));
        }
    }
}
fn capture_endpoint(
    shared: &Rc<Shared<'_>>,
    r: &mut Report,
    time: f64,
    state: &[f64],
    event: Option<SemanticId>,
) -> Result<(), ProblemError> {
    shared.check();
    let outputs = shared.evaluate(Function::Output, time, state, false).values;
    if let Some((_, error)) = shared.failure.borrow_mut().take() {
        return Err(error);
    }
    let point = r
        .samples
        .last()
        .filter(|s| s.time == time && s.mode == shared.mode.get())
        .cloned()
        .unwrap_or_else(|| Sample {
            mode: shared.mode.get(),
            integrals: shared.integrals.borrow().clone(),
            time,
            state: state.to_vec(),
            outputs,
            state_sensitivities: vec![],
            output_sensitivities: vec![],
        });
    r.endpoint = Some(TrajectoryEndpoint {
        point,
        event,
        input_columns: shared.columns.borrow().clone(),
        inputs: shared.values(),
    });
    Ok(())
}
fn sample<'p, 'o: 'p, S: OdeSolverMethod<'p, Equation<'o>>>(
    shared: &Rc<Shared<'o>>,
    s: &S,
    t: f64,
    sens: bool,
) -> Result<Sample, ProblemError> {
    let y = if t == s.state().t {
        s.state().y.clone()
    } else {
        s.interpolate(t).map_err(native)?
    };
    let eval = shared.evaluate(Function::Output, t, y.as_slice(), sens);
    let np = shared.width();
    let n = shared.contract.states.len();
    let columns = shared.columns.borrow().clone();
    let mut dy = Vec::new();
    let mut dh = Vec::new();
    if sens {
        let vectors = if t == s.state().t {
            s.state().s.to_vec()
        } else {
            s.interpolate_sens(t).map_err(native)?
        };
        if vectors.len() != np {
            return Err(ProblemError::internal("native sensitivity count"));
        }
        let states = faer::Mat::from_fn(n, np, |i, j| vectors[j][i]);
        let Some(jac) = eval.jacobian else {
            return Err(ProblemError::internal("output sensitivity partials"));
        };
        // Output partials along [S; P], where P selects each contract parameter's
        // integration column in the sample's segment.
        let chain = faer::Mat::from_fn(n + columns.len(), np, |i, j| {
            if i < n {
                states[(i, j)]
            } else {
                f64::from(columns[i - n] == j)
            }
        });
        let mut result = faer::Mat::zeros(jac.nrows(), chain.ncols());
        faer::sparse::linalg::matmul::sparse_dense_matmul(
            result.as_mut(),
            faer::Accum::Replace,
            jac.as_ref(),
            chain.as_ref(),
            1.0,
            faer::Par::Seq,
        );
        for i in 0..n {
            for j in 0..np {
                dy.push(states[(i, j)]);
            }
        }
        for i in 0..eval.values.len() {
            for j in 0..np {
                dh.push(result[(i, j)]);
            }
        }
        if dy.iter().chain(&dh).any(|v| !v.is_finite()) {
            return Err(ProblemError::numerical("nonfinite output sensitivity"));
        }
    }
    let integrals = if shared.contract.quadratures.is_empty() {
        vec![]
    } else {
        let g = s.interpolate_out(t).map_err(native)?;
        g.as_slice()
            .iter()
            .zip(shared.integrals.borrow().iter())
            .map(|(v, total)| v + total)
            .collect()
    };
    Ok(Sample {
        mode: shared.mode.get(),
        integrals,
        time: t,
        state: y.as_slice().to_vec(),
        outputs: eval.values,
        state_sensitivities: dy,
        output_sensitivities: dh,
    })
}

fn copy_initial(
    p: &diffsol::InitialConditionSolverOptions<f64>,
) -> diffsol::InitialConditionSolverOptions<f64> {
    diffsol::InitialConditionSolverOptions {
        use_linesearch: p.use_linesearch,
        max_linesearch_iterations: p.max_linesearch_iterations,
        max_newton_iterations: p.max_newton_iterations,
        max_linear_solver_setups: p.max_linear_solver_setups,
        step_reduction_factor: p.step_reduction_factor,
        armijo_constant: p.armijo_constant,
    }
}

fn copy_native(p: &diffsol::OdeSolverOptions<f64>) -> diffsol::OdeSolverOptions<f64> {
    diffsol::OdeSolverOptions {
        max_nonlinear_solver_iterations: p.max_nonlinear_solver_iterations,
        max_error_test_failures: p.max_error_test_failures,
        max_nonlinear_solver_failures: p.max_nonlinear_solver_failures,
        nonlinear_solver_tolerance: p.nonlinear_solver_tolerance,
        min_timestep: p.min_timestep,
        max_timestep_growth: p.max_timestep_growth,
        min_timestep_growth: p.min_timestep_growth,
        max_timestep_shrink: p.max_timestep_shrink,
        min_timestep_shrink: p.min_timestep_shrink,
        update_jacobian_after_steps: p.update_jacobian_after_steps,
        update_rhs_jacobian_after_steps: p.update_rhs_jacobian_after_steps,
        threshold_to_update_jacobian: p.threshold_to_update_jacobian,
        threshold_to_update_rhs_jacobian: p.threshold_to_update_rhs_jacobian,
        pi_control_proportional: p.pi_control_proportional,
        pi_control_integral: p.pi_control_integral,
    }
}

#[cfg(test)]
mod forward_initialization_tests {
    use super::*;

    /// The public affine experiment, in canonical seconds: x'=rate, x(0)=2,
    /// output=x. Exact sensitivities follow the elapsed input interval.
    #[derive(Debug)]
    struct Affine(Contract);
    impl Affine {
        fn new() -> Self {
            let id = |n| SemanticId::from_bytes([n; 16]);
            Self(Contract {
                identity: ContentHash::from_bytes([81; 32]),
                states: vec![id(82)],
                differential: vec![true],
                parameters: vec![id(83)],
                outputs: vec![id(84)],
                events: vec![vec![]],
                quadratures: vec![],
                balances: vec![],
                signs: vec![],
                derivatives: pse_kernels::DerivativeOrder::First,
            })
        }
    }
    impl Oracle for Affine {
        fn contract(&self) -> &Contract {
            &self.0
        }
        fn support(&self, _: usize, function: Function) -> Vec<SupportEntry> {
            entries(match function {
                Function::Rhs => vec![(0, 1)],
                Function::Output => vec![(0, 0)],
                _ => vec![],
            })
        }
        fn evaluate(
            &mut self,
            _: usize,
            function: Function,
            _: f64,
            state: &[f64],
            parameters: &[f64],
            derivatives: bool,
        ) -> Result<Evaluation, ProblemError> {
            let (values, partials) = match function {
                Function::Initial => (vec![2.0], vec![]),
                Function::Rhs => (vec![parameters[0]], vec![(0, 1, 1.0)]),
                Function::Output => (vec![state[0]], vec![(0, 0, 1.0)]),
                _ => (vec![], vec![]),
            };
            let jacobian = derivatives.then(|| {
                faer::sparse::SparseColMat::try_new_from_triplets(
                    values.len(),
                    2,
                    &partials
                        .into_iter()
                        .map(|(row, col, value)| faer::sparse::Triplet::new(row, col, value))
                        .collect::<Vec<_>>(),
                )
                .unwrap()
            });
            Ok(Evaluation { values, jacobian })
        }
    }
    fn affine_profile(linear: DiffsolLinear) -> Profile {
        Profile {
            method: Method::Diffsol,
            samples: vec![0.0, 0.0005, 0.5, 1.0],
            atol: vec![0.001],
            rtol: 0.001,
            initial_step: 1e-4,
            parameter_scales: vec![1.0],
            sensitivity: DynamicSensitivity::Forward,
            diffsol: DiffsolSettings {
                linear,
                ..Default::default()
            },
            ..Default::default()
        }
    }
    #[test]
    fn affine_bdf_forward_response_respects_physical_allowance_at_public_controls() {
        // This empirical affine control compares a half-scale parameter effect
        // against the public state's frozen physical budget. The native local
        // integration tolerances are inputs, not a derivative-error certificate.
        let (delta, budget) = (0.5, 0.001);
        for linear in [DiffsolLinear::FaerLu, DiffsolLinear::Klu] {
            for rate in [1.0, 3.0] {
                let report = integrate(
                    &mut Affine::new(),
                    &affine_profile(linear),
                    &[rate],
                    Arc::default(),
                )
                .unwrap();
                assert_eq!(
                    report.termination,
                    Termination::Completed,
                    "{:?}",
                    report.error
                );
                assert_eq!(report.samples.len(), 4);
                for sample in &report.samples {
                    assert!(
                        (sample.state[0] - (2.0 + rate * sample.time)).abs() <= budget,
                        "{linear:?}: {sample:?}"
                    );
                    assert!(
                        (delta * sample.state_sensitivities[0] - delta * sample.time).abs()
                            <= budget,
                        "{linear:?}: {sample:?}"
                    );
                    assert!(
                        (delta * sample.output_sensitivities[0] - delta * sample.time).abs()
                            <= budget,
                        "{linear:?}: {sample:?}"
                    );
                }
            }
        }
    }
    #[test]
    fn affine_bdf_forward_response_reinitializes_each_scheduled_segment() {
        let (delta, budget) = (0.5, 0.001);
        for linear in [DiffsolLinear::FaerLu, DiffsolLinear::Klu] {
            let mut profile = affine_profile(linear);
            profile.schedule = vec![ScheduledInput {
                parameter: 0,
                times: vec![0.5],
            }];
            let report =
                integrate(&mut Affine::new(), &profile, &[3.0, 1.0], Arc::default()).unwrap();
            assert_eq!(
                report.termination,
                Termination::Completed,
                "{:?}",
                report.error
            );
            assert_eq!(report.samples.len(), 4);
            for sample in &report.samples {
                let intervals = [sample.time.min(0.5), (sample.time - 0.5).max(0.0)];
                assert!(
                    (sample.state[0] - (2.0 + 3.0 * intervals[0] + intervals[1])).abs() <= budget,
                    "{linear:?}: {sample:?}"
                );
                assert_eq!(sample.state_sensitivities.len(), 2);
                assert_eq!(sample.output_sensitivities.len(), 2);
                for (column, duration) in intervals.into_iter().enumerate() {
                    assert!(
                        (delta * sample.state_sensitivities[column] - delta * duration).abs()
                            <= budget,
                        "{linear:?}: {sample:?}"
                    );
                    assert!(
                        (delta * sample.output_sensitivities[column] - delta * duration).abs()
                            <= budget,
                        "{linear:?}: {sample:?}"
                    );
                }
            }
        }
    }
}
