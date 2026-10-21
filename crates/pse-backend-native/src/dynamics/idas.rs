// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
#![allow(
    unsafe_code,
    reason = "owned IDAS/SUNContext resources and panic-contained C callbacks"
)]
//! IDAS owns residual integration, consistent initialization, trial recovery, roots,
//! scheduled reinitialization, sign constraints and forward sensitivities (ADR-0093,
//! ADR-0110 item 1). Each scheduled change or reset restarts the same native memory with
//! `IDAReInit`, `IDASensReInit`, `IDAQuadReInit` and `IDACalcIC`; nothing is integrated by
//! project code. Sensitivities are taken with respect to the integration parameters, one
//! per scheduled-input interval, so they cross every scheduled change (I6). The adjoint
//! route (ADR-0110 item 3) records the forward pass with `IDAAdjInit`/`IDASolveF` and
//! integrates one backward problem with `IDASolveB` and its gradient quadrature.
use super::*;
use crate::{
    NativeStatus,
    callback::CallbackState,
    solve::{Assurance, Backend, Execution, NativeTermination, Preconditioner, Progress},
};
use std::{
    ffi::{c_int, c_long, c_void},
    marker::PhantomData,
    rc::Rc,
    time::Instant,
};
use suitesparse_sys as _;
use sundials_sys as ffi;
mod bbd;
#[cfg(test)]
mod coupled_probe;
mod local_error;
#[cfg(test)]
mod policy_inquiry;
#[cfg(test)]
mod workspace_tests;

type Jacobian = pse_math::SharedAllocation<faer::sparse::SparseColMat<usize, f64>>;
/// Exact fields exposed together by SUNDIALS 7.1.1 IDAGetIntegratorStats.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct IntegratorStatistics {
    steps: c_long,
    residuals: c_long,
    linear_setups: c_long,
    error_test_failures: c_long,
    last_order: c_int,
    current_order: c_int,
    initial_step: f64,
    last_step: f64,
    current_step: f64,
    internal_time: f64,
}

/// A second-order adjoint result: the symmetric Hessian over the directions and its
/// largest relative asymmetry before symmetrization.
type SecondOrder = (faer::Mat<f64>, f64);

fn format_callback(function: Function) -> crate::callback::CallbackKey {
    use crate::callback::CallbackKey;
    match function {
        Function::Rhs => CallbackKey::Named("idas.residual"),
        Function::Initial => CallbackKey::Named("idas.initial"),
        Function::Output => CallbackKey::Named("idas.output"),
        Function::QuadratureFlux => CallbackKey::Named("idas.quadrature"),
        Function::Inventory => CallbackKey::Named("idas.inventory"),
        Function::Roots => CallbackKey::Named("idas.roots"),
        Function::Transfer(index) => CallbackKey::Block {
            index,
            demand: "idas.transfer",
        },
        Function::Reset(index) => CallbackKey::Block {
            index,
            demand: "idas.reset",
        },
    }
}

struct Context<'a> {
    oracle: &'a mut dyn Oracle,
    contract: Contract,
    /// The contract parameter values of the current segment.
    parameters: Vec<f64>,
    /// The integration parameter vector: the sensitivity parameters.
    integration: Vec<f64>,
    /// Physical integration axis for each native forward sensitivity vector.
    /// Exact-curvature sessions preserve their complete forward layout.
    axes: Vec<usize>,
    /// Ordered physical coordinates of parameter quadratures; independent of state adjoints.
    gradient_axes: Vec<usize>,
    /// The integration column of each contract parameter in the current segment.
    map: Vec<usize>,
    mode: usize,
    trial_policy: TrialPolicy,
    callback: CallbackState,
    /// Actual first residual outcome of the current IDACalcIC phase, not a later trial.
    initial_ic_residual: Option<bool>,
    /// One immutable all-mode layout shared with the preparation/admission owner.
    layout: Arc<PreparedLayout>,
    newton_values: Vec<f64>,
    newton_values_b: Vec<f64>,
    /// Selected Krylov callback scratch, retained across native resets.
    rhs_direction: Vec<f64>,
    rhs_product: Vec<f64>,
    /// CSC pattern of the second-order adjoint's block lower-triangular Newton matrix: the
    /// transposed pattern on both diagonal blocks and on the coupling block below them;
    /// empty on the other routes.
    columns_bs: Vec<ffi::sunindextype>,
    rows_bs: Vec<ffi::sunindextype>,
    partial_slots_bs: Vec<[usize; 2]>,
    diagonal_slots_bs: Vec<[usize; 2]>,
    coupling_slots_bs: Vec<usize>,
    /// Residual rows grouped so that no two rows of a group depend on a common state: one
    /// weighted Hessian per group yields their coupling columns (a Curtis–Powell–Reid
    /// colouring of the coupling block).
    colors: Vec<Vec<usize>>,
    /// Inverse Jacobi diagonal of the current Newton matrix (Krylov preconditioner).
    inverse_diagonal: Vec<f64>,
    #[cfg(test)]
    coupled_probe: Option<coupled_probe::FactorState>,
    sample_elimination: SampleEliminationPreparation,
}
/// Stable address copied by native sensitivity/linear modules. Idle storage owns
/// only this nullable pointer; the current request alone owns the target Context.
struct UserDataSlot {
    current: *mut c_void,
}
/// # Safety
/// `data` is this owner-thread session's live registered slot; while nonnull its
/// target Box lives through the whole synchronous native call and is exclusive.
unsafe fn callback_context<'call, 'oracle>(
    data: *mut c_void,
) -> Option<&'call mut Context<'oracle>> {
    if data.is_null() {
        return None;
    }
    // SAFETY: the caller supplies the registered stable slot.
    let slot = unsafe { &*data.cast::<UserDataSlot>() };
    if slot.current.is_null() {
        return None;
    }
    // SAFETY: the owner binds a fresh live exclusive Box before every native call.
    Some(unsafe { &mut *slot.current.cast::<Context<'oracle>>() })
}
impl Context<'_> {
    fn evaluate(
        &mut self,
        f: Function,
        t: f64,
        x: &[f64],
        derivatives: bool,
    ) -> Option<Evaluation> {
        crate::observation::integer(
            "idas.callback.mode",
            "ordinal",
            i64::try_from(self.mode).unwrap_or(i64::MAX),
        );
        crate::observation::boolean("idas.callback.derivatives_requested", derivatives);
        let Self {
            oracle,
            contract: c,
            parameters,
            mode,
            callback,
            layout,
            ..
        } = self;
        let value = callback.evaluate(format_callback(f), || {
            let e = oracle.evaluate(*mode, f, t, x, parameters, derivatives)?;
            let rows = match f {
                Function::Output => c.outputs.len(),
                Function::QuadratureFlux => c.quadratures.len(),
                Function::Inventory | Function::Transfer(_) => c.balances.len(),
                Function::Roots => c.events[*mode].len(),
                _ => c.states.len(),
            };
            if e.values.len() != rows
                || derivatives
                    && e.jacobian.as_ref().is_none_or(|j| {
                        j.nrows() != rows || j.ncols() != c.states.len() + c.parameters.len()
                    })
            {
                return Err(ProblemError::internal(
                    "IDAS function value/derivative dimensions",
                ));
            }
            if e.values.iter().any(|v| !v.is_finite())
                || derivatives
                    && e.jacobian
                        .as_ref()
                        .is_some_and(|j| j.val().iter().any(|v| !v.is_finite()))
            {
                return Err(ProblemError::numerical(
                    "IDAS function value or derivative is nonfinite",
                ));
            }
            if f == Function::Rhs
                && derivatives
                && let Some(partials) = &e.jacobian
                && (0..c.states.len()).any(|column| {
                    partials
                        .col_range(column)
                        .any(|entry| layout.get(column, partials.row_idx()[entry]).is_none())
                })
            {
                return Err(ProblemError::Contract(
                    "IDAS residual partial outside its declared support".into(),
                ));
            }
            Ok(e)
        });
        if value.is_none()
            && callback.terminal.is_none()
            && self.trial_policy == TrialPolicy::Terminal
        {
            callback.terminal = Some((
                crate::solve::Termination::Evaluation,
                "trial failure under terminal policy".into(),
            ));
        }
        value
    }
    /// The analytic state-and-parameter partials of the current mode's residual function;
    /// `evaluate` has checked their presence and extent.
    fn partials(&mut self, t: f64, x: &[f64]) -> Option<Jacobian> {
        self.evaluate(Function::Rhs, t, x, true)
            .and_then(|e| e.jacobian)
    }
    fn rhs_diagonal(&mut self, t: f64, x: &[f64]) -> Option<()> {
        crate::observation::integer(
            "idas.callback.mode",
            "ordinal",
            i64::try_from(self.mode).unwrap_or(i64::MAX),
        );
        crate::observation::boolean("idas.callback.derivatives_requested", true);
        let Self {
            oracle,
            parameters,
            mode,
            callback,
            contract,
            layout,
            rhs_product,
            ..
        } = self;
        let result = callback.evaluate("idas.residual_diagonal", || {
            if rhs_product.len() != contract.states.len() {
                return Err(ProblemError::internal("IDAS diagonal workspace extent"));
            }
            rhs_product.fill(f64::NAN);
            let source = oracle.rhs_state_diagonal(*mode, t, x, parameters, rhs_product)?;
            if rhs_product.iter().any(|v| !v.is_finite()) {
                return Err(ProblemError::numerical("nonfinite IDAS raw RHS diagonal"));
            }
            if let RhsDiagonalSource::FullJacobian(evaluation) = &source {
                let j = evaluation
                    .jacobian
                    .as_ref()
                    .ok_or_else(|| ProblemError::internal("IDAS full diagonal partials absent"))?;
                // A wrapper may append guarded structural-zero states to its
                // source's unchanged state prefix. Parameter columns are not
                // consumed by this product, just as in the ordinary state guard.
                if j.nrows() == 0
                    || j.nrows() > contract.states.len()
                    || evaluation.values.len() != j.nrows()
                    || j.ncols() < j.nrows()
                    || evaluation
                        .values
                        .iter()
                        .chain(j.val())
                        .any(|v| !v.is_finite())
                    || rhs_product[j.nrows()..].iter().any(|v| *v != 0.)
                    || (0..j.nrows()).any(|column| {
                        j.col_range(column)
                            .any(|entry| layout.get(column, j.row_idx()[entry]).is_none())
                    })
                {
                    return Err(ProblemError::Contract(
                        "IDAS full diagonal partial outside its declared support or extent".into(),
                    ));
                }
                for (column, value) in rhs_product[..j.nrows()].iter_mut().enumerate() {
                    *value = j
                        .col_range(column)
                        .find(|entry| j.row_idx()[*entry] == column)
                        .map_or(0., |entry| j.val()[entry]);
                }
            }
            let selected = matches!(source, RhsDiagonalSource::SelectedEntries);
            crate::observation::boolean("idas.jacobi.selected_entries", selected);
            crate::observation::count(
                if selected {
                    "idas.jacobi.selected_setups"
                } else {
                    "idas.jacobi.full_setups"
                },
                1,
            );
            Ok(())
        });
        if result.is_none()
            && callback.terminal.is_none()
            && self.trial_policy == TrialPolicy::Terminal
        {
            callback.terminal = Some((
                crate::solve::Termination::Evaluation,
                "trial failure under terminal policy".into(),
            ));
        }
        result
    }
    fn rhs_action(&mut self, t: f64, x: &[f64], direction: &[f64], cj: f64) -> Option<()> {
        let Self {
            oracle,
            parameters,
            mode,
            callback,
            contract,
            rhs_direction,
            rhs_product,
            ..
        } = self;
        let result = callback.evaluate("idas.residual_action", || {
            if direction.len() != contract.states.len()
                || rhs_product.len() != direction.len()
                || rhs_direction.len() != direction.len() + parameters.len()
            {
                return Err(ProblemError::internal("IDAS action workspace extent"));
            }
            rhs_direction[..direction.len()].copy_from_slice(direction);
            // Parameter seeds remain exactly zero from construction. Input
            // borrows cannot mutate them, and native reinit does not replace them.
            rhs_product.fill(f64::NAN);
            oracle.jacobian_action(
                *mode,
                Function::Rhs,
                t,
                x,
                parameters,
                rhs_direction,
                rhs_product,
            )?;
            for i in 0..direction.len() {
                let mass = if contract.differential[i] { cj } else { 0. };
                rhs_product[i] = mass * rhs_direction[i] - rhs_product[i];
            }
            if rhs_product.iter().any(|v| !v.is_finite()) {
                return Err(ProblemError::numerical("nonfinite IDAS Newton action"));
            }
            Ok(())
        });
        if result.is_none()
            && callback.terminal.is_none()
            && self.trial_policy == TrialPolicy::Terminal
        {
            callback.terminal = Some((
                crate::solve::Termination::Evaluation,
                "trial failure under terminal policy".into(),
            ));
        }
        result
    }
    /// `∇²(Σ weights·f)·d` over the state followed by the contract parameters, from the
    /// oracle's exact weighted Hessian of function `f` (ADR-0110 item 4).
    fn curvature(
        &mut self,
        f: Function,
        t: f64,
        x: &[f64],
        weights: &[f64],
        direction: &[f64],
    ) -> Option<Vec<f64>> {
        let Self {
            oracle,
            contract: c,
            parameters,
            mode,
            callback,
            ..
        } = self;
        let width = c.states.len() + c.parameters.len();
        let value = callback.evaluate("idas.curvature", || {
            let mut product = vec![0.; width];
            oracle.weighted_hessian_action(
                *mode,
                f,
                t,
                x,
                parameters,
                weights,
                direction,
                &mut product,
            )?;
            if product.iter().any(|v| !v.is_finite()) {
                return Err(ProblemError::numerical(
                    "IDAS weighted Hessian action is nonfinite",
                ));
            }
            Ok(product)
        });
        if value.is_none()
            && callback.terminal.is_none()
            && self.trial_policy == TrialPolicy::Terminal
        {
            callback.terminal = Some((
                crate::solve::Termination::Evaluation,
                "trial failure under terminal policy".into(),
            ));
        }
        value
    }
    /// The tangent direction of integration column `k` over the state followed by the
    /// contract parameters: the forward state sensitivity `s_k`, then every contract
    /// parameter that takes column `k` in the current segment.
    fn direction(&self, k: usize, sensitivity: &[f64]) -> Vec<f64> {
        sensitivity
            .iter()
            .copied()
            .chain(self.map.iter().map(|column| f64::from(*column == k)))
            .collect()
    }
    fn failure(&self) -> i32 {
        if self.callback.terminal.is_some() {
            -1
        } else {
            1
        }
    }
    /// The typed cause of a failed callback demand, never a message.
    fn failed(&mut self, operation: &str) -> ProblemError {
        self.callback
            .terminal_error()
            .or_else(|| self.callback.last_failure.take())
            .unwrap_or_else(|| ProblemError::internal(format!("IDAS {operation} failed")))
    }
    /// A failed integration or initialization flag: a latched callback cause wins, and
    /// an evaluation flag reports the callback's typed witness; otherwise the native
    /// status is kept (F06).
    fn native_failure(&mut self, flag: i32, operation: &str) -> ProblemError {
        let status = termination(flag);
        if self.callback.terminal.is_some()
            || status.category == crate::solve::Termination::Evaluation
                && self.callback.last_failure.is_some()
        {
            return self.failed(operation);
        }
        ProblemError::native(
            NativeStatus {
                backend: Backend::Idas,
                code: status.code,
                name: status.name,
            },
            status.category,
            format!("IDAS {operation}"),
        )
    }
}
/// Map every pinned IDA/IDAS return flag to the shared stop vocabulary, mirroring
/// `kinsol::termination`. Only an integer undeclared by the pinned header reaches the
/// final arm, and it is never given a confident category.
pub(crate) fn termination(flag: i32) -> NativeTermination {
    use crate::solve::Termination as T;
    let (name, category) = match flag {
        ffi::IDA_SUCCESS => ("IDA_SUCCESS", T::Success),
        ffi::IDA_TSTOP_RETURN => ("IDA_TSTOP_RETURN", T::Success),
        ffi::IDA_ROOT_RETURN => ("IDA_ROOT_RETURN", T::Success),
        ffi::IDA_WARNING => ("IDA_WARNING", T::Inconclusive),
        ffi::IDA_TOO_MUCH_WORK => ("IDA_TOO_MUCH_WORK", T::IterationLimit),
        ffi::IDA_TOO_MUCH_ACC => ("IDA_TOO_MUCH_ACC", T::Numerical),
        ffi::IDA_ERR_FAIL => ("IDA_ERR_FAIL", T::Numerical),
        ffi::IDA_CONV_FAIL => ("IDA_CONV_FAIL", T::Numerical),
        ffi::IDA_LINIT_FAIL => ("IDA_LINIT_FAIL", T::Numerical),
        ffi::IDA_LSETUP_FAIL => ("IDA_LSETUP_FAIL", T::Numerical),
        ffi::IDA_LSOLVE_FAIL => ("IDA_LSOLVE_FAIL", T::Numerical),
        ffi::IDA_RES_FAIL => ("IDA_RES_FAIL", T::Evaluation),
        ffi::IDA_REP_RES_ERR => ("IDA_REP_RES_ERR", T::Evaluation),
        ffi::IDA_RTFUNC_FAIL => ("IDA_RTFUNC_FAIL", T::Evaluation),
        ffi::IDA_CONSTR_FAIL => ("IDA_CONSTR_FAIL", T::Numerical),
        ffi::IDA_FIRST_RES_FAIL => ("IDA_FIRST_RES_FAIL", T::Evaluation),
        ffi::IDA_LINESEARCH_FAIL => ("IDA_LINESEARCH_FAIL", T::Numerical),
        ffi::IDA_NO_RECOVERY => ("IDA_NO_RECOVERY", T::Numerical),
        ffi::IDA_NLS_INIT_FAIL => ("IDA_NLS_INIT_FAIL", T::Numerical),
        ffi::IDA_NLS_SETUP_FAIL => ("IDA_NLS_SETUP_FAIL", T::Numerical),
        ffi::IDA_NLS_FAIL => ("IDA_NLS_FAIL", T::Numerical),
        ffi::IDA_MEM_NULL => ("IDA_MEM_NULL", T::Invalid),
        ffi::IDA_MEM_FAIL => ("IDA_MEM_FAIL", T::ResourceExhausted),
        ffi::IDA_ILL_INPUT => ("IDA_ILL_INPUT", T::Invalid),
        ffi::IDA_NO_MALLOC => ("IDA_NO_MALLOC", T::Invalid),
        ffi::IDA_BAD_EWT => ("IDA_BAD_EWT", T::Numerical),
        ffi::IDA_BAD_K => ("IDA_BAD_K", T::Invalid),
        ffi::IDA_BAD_T => ("IDA_BAD_T", T::Invalid),
        ffi::IDA_BAD_DKY => ("IDA_BAD_DKY", T::Invalid),
        ffi::IDA_VECTOROP_ERR => ("IDA_VECTOROP_ERR", T::Numerical),
        ffi::IDA_CONTEXT_ERR => ("IDA_CONTEXT_ERR", T::Invalid),
        ffi::IDA_NO_QUAD => ("IDA_NO_QUAD", T::Invalid),
        ffi::IDA_QRHS_FAIL => ("IDA_QRHS_FAIL", T::Evaluation),
        ffi::IDA_FIRST_QRHS_ERR => ("IDA_FIRST_QRHS_ERR", T::Evaluation),
        ffi::IDA_REP_QRHS_ERR => ("IDA_REP_QRHS_ERR", T::Evaluation),
        ffi::IDA_NO_SENS => ("IDA_NO_SENS", T::Invalid),
        ffi::IDA_SRES_FAIL => ("IDA_SRES_FAIL", T::Evaluation),
        ffi::IDA_REP_SRES_ERR => ("IDA_REP_SRES_ERR", T::Evaluation),
        ffi::IDA_BAD_IS => ("IDA_BAD_IS", T::Invalid),
        ffi::IDA_NO_QUADSENS => ("IDA_NO_QUADSENS", T::Invalid),
        ffi::IDA_QSRHS_FAIL => ("IDA_QSRHS_FAIL", T::Evaluation),
        ffi::IDA_FIRST_QSRHS_ERR => ("IDA_FIRST_QSRHS_ERR", T::Evaluation),
        ffi::IDA_REP_QSRHS_ERR => ("IDA_REP_QSRHS_ERR", T::Evaluation),
        ffi::IDA_UNRECOGNIZED_ERROR => ("IDA_UNRECOGNIZED_ERROR", T::Inconclusive),
        ffi::IDA_NO_ADJ => ("IDA_NO_ADJ", T::Invalid),
        ffi::IDA_NO_FWD => ("IDA_NO_FWD", T::Invalid),
        ffi::IDA_NO_BCK => ("IDA_NO_BCK", T::Invalid),
        ffi::IDA_BAD_TB0 => ("IDA_BAD_TB0", T::Invalid),
        ffi::IDA_REIFWD_FAIL => ("IDA_REIFWD_FAIL", T::Numerical),
        ffi::IDA_FWD_FAIL => ("IDA_FWD_FAIL", T::Numerical),
        ffi::IDA_GETY_BADT => ("IDA_GETY_BADT", T::Invalid),
        _ => ("IDA_UNKNOWN", T::Inconclusive),
    };
    NativeTermination {
        code: i64::from(flag),
        name: name.into(),
        message: None,
        category,
        assurance: Assurance::None,
    }
}
/// Setup, option and retrieval calls return module-specific flags (IDA, IDALS or
/// SUNErrCode); a nonzero flag there is an adapter invariant failure.
fn check(code: i32, operation: &str) -> Result<(), ProblemError> {
    if code == 0 {
        Ok(())
    } else {
        Err(ProblemError::internal(format!(
            "IDAS {operation}: native flag {code}"
        )))
    }
}
/// Call IDAS or SUNDIALS and require a zero flag, as [`check`] does.
///
/// Every use passes live native objects of one session: its `SUNContext`, its IDAS memory
/// (which owns the adjoint memory and every backward problem), matrices, linear solvers and
/// serial vectors, all freed only when the session drops. Calls run on the session's owning
/// thread, and out-parameters and vector arrays are caller storage that outlives the call.
macro_rules! native {
    ($call:expr, $operation:expr $(,)?) => {{
        // SAFETY: the macro's contract: live session-owned native objects on the owning
        // thread, with caller storage that outlives the call.
        check(unsafe { $call }, $operation)
    }};
}
fn index(n: usize) -> Result<ffi::sunindextype, ProblemError> {
    n.try_into()
        .map_err(|_| ProblemError::unsupported("IDAS index extent"))
}
/// Trajectory stop implied by a typed failure; stops are never reported as failures.
fn stopped(error: &ProblemError, execution: &Execution) -> Termination {
    match crate::callback::classify(error) {
        crate::callback::Failure::Stopped(crate::solve::Termination::Cancelled) => {
            Termination::Cancelled
        }
        crate::callback::Failure::Stopped(crate::solve::Termination::TimeLimit) => {
            Termination::TimeLimit
        }
        _ if matches!(
            error,
            ProblemError::Limit {
                kind: crate::LimitKind::Work,
                ..
            }
        ) =>
        {
            Termination::StepLimit
        }
        _ => match execution.stopped() {
            Some(crate::solve::Termination::Cancelled) => Termination::Cancelled,
            Some(crate::solve::Termination::TimeLimit) => Termination::TimeLimit,
            _ => Termination::Failed,
        },
    }
}
/// The first `n` values of a serial vector.
///
/// # Safety
/// `v` is a live serial vector of at least `n` values.
unsafe fn read(v: ffi::N_Vector, n: usize) -> Vec<f64> {
    // SAFETY: `v` is a live serial vector (the caller's contract).
    let data = unsafe { ffi::N_VGetArrayPointer(v) };
    // SAFETY: its data array holds at least `n` initialized values.
    unsafe { std::slice::from_raw_parts(data, n) }.to_vec()
}
/// Borrow one callback input without retaining foreign storage.
///
/// # Safety
/// `vector` is null or a live serial vector. Its initialized data stays live and
/// unmodified for `'input`; every returned borrow ends before native publication
/// or a native call that can mutate or invalidate the vector.
unsafe fn callback_input<'input>(
    vector: ffi::N_Vector,
    expected: usize,
) -> Result<&'input [f64], ProblemError> {
    let expected_native = ffi::sunindextype::try_from(expected)
        .map_err(|_| ProblemError::internal("IDAS callback vector extent"))?;
    // SAFETY: non-null vectors are live serial vectors by the caller's contract.
    if vector.is_null() || unsafe { ffi::N_VGetLength(vector) } != expected_native {
        return Err(ProblemError::internal("IDAS callback vector dimensions"));
    }
    if expected == 0 {
        return Ok(&[]);
    }
    // SAFETY: the live serial vector has the checked nonzero extent.
    let pointer = unsafe { ffi::N_VGetArrayPointer(vector) };
    if pointer.is_null() {
        return Err(ProblemError::internal("IDAS null callback vector storage"));
    }
    // SAFETY: the checked initialized serial data stays unmodified for the
    // caller-bounded input lifetime and does not outlive its callback.
    Ok(unsafe { std::slice::from_raw_parts(pointer, expected) })
}
/// The caller supplies a live callback vector and ends its input borrow before
/// any native output publication. Malformed callback storage is terminal, and
/// never supplies a partially computed native output.
macro_rules! input {
    ($context:ident, $vector:expr, $expected:expr) => {{
        // SAFETY: each callback invocation documents the vector's live serial
        // extent and confines returned input borrows to its compute scope.
        match unsafe { callback_input($vector, $expected) } {
            Ok(values) => values,
            Err(error) => {
                if $context.callback.terminal.is_none() {
                    $context.callback.terminal = Some((
                        crate::solve::Termination::Evaluation,
                        "invalid IDAS callback vector".into(),
                    ));
                    $context.callback.last_failure = Some(error);
                }
                return -1;
            }
        }
    }};
}
/// Overwrite the first `values.len()` entries of a serial vector.
///
/// # Safety
/// `v` is a live serial vector of at least `values.len()` values.
unsafe fn write(v: ffi::N_Vector, values: &[f64]) {
    // SAFETY: `v` is a live serial vector (the caller's contract).
    let data = unsafe { ffi::N_VGetArrayPointer(v) };
    // SAFETY: its native data array holds at least `values.len()` values and cannot
    // overlap the Rust slice.
    unsafe { std::ptr::copy_nonoverlapping(values.as_ptr(), data, values.len()) };
}
/// Publish a column of an independent owned product without another output copy.
///
/// # Safety
/// `vector` is a live serial output with `product.nrows()` entries, no reference
/// into its data remains live, and column is within the owned matrix.
unsafe fn write_column(vector: ffi::N_Vector, product: &faer::Mat<f64>, column: usize) {
    // SAFETY: the output is a live serial vector with the caller-stated extent.
    let pointer = unsafe { ffi::N_VGetArrayPointer(vector) };
    for row in 0..product.nrows() {
        // SAFETY: row is below product.nrows(), the caller-guaranteed native extent.
        let cell = unsafe { pointer.add(row) };
        // SAFETY: cell denotes that writable serial output entry, with no live input borrow.
        unsafe { cell.write(product[(row, column)]) };
    }
}
/// The vector at `k` of a SUNDIALS vector array.
///
/// # Safety
/// `array` holds more than `k` initialized vectors.
unsafe fn at(array: *mut ffi::N_Vector, k: usize) -> ffi::N_Vector {
    // SAFETY: `k` is within the caller's array.
    let entry = unsafe { array.add(k) };
    // SAFETY: as above; the entry is initialized.
    unsafe { *entry }
}
/// Copy a CSC pattern and its values into a SUNDIALS sparse matrix.
///
/// # Safety
/// `matrix` is a live CSC `SUNSparseMatrix` with `columns.len() - 1` columns and room for
/// `rows.len()` nonzeros, and `values` has `rows.len()` entries.
unsafe fn publish(
    matrix: ffi::SUNMatrix,
    columns: &[ffi::sunindextype],
    rows: &[ffi::sunindextype],
    values: &[f64],
) {
    // SAFETY: `matrix` is a live sparse matrix (the caller's contract).
    let pointers = unsafe { ffi::SUNSparseMatrix_IndexPointers(matrix) };
    // SAFETY: the matrix holds one index pointer per column plus one, `columns.len()`, in
    // native storage that cannot overlap the Rust slice.
    unsafe { std::ptr::copy_nonoverlapping(columns.as_ptr(), pointers, columns.len()) };
    // SAFETY: as above.
    let indices = unsafe { ffi::SUNSparseMatrix_IndexValues(matrix) };
    // SAFETY: its nonzero capacity holds the `rows.len()` row indices.
    unsafe { std::ptr::copy_nonoverlapping(rows.as_ptr(), indices, rows.len()) };
    // SAFETY: as above.
    let data = unsafe { ffi::SUNSparseMatrix_Data(matrix) };
    // SAFETY: its nonzero capacity holds the `values.len() == rows.len()` values.
    unsafe { std::ptr::copy_nonoverlapping(values.as_ptr(), data, values.len()) };
}
/// `J · [S; P]`: every function row's derivative along each integration parameter, from
/// the state sensitivities `columns` (one per integration parameter) and the direct
/// partials, where `P` selects each contract parameter's integration column `map[k]` in the
/// current segment. One faer sparse × dense product (F12).
fn chained(
    j: &Jacobian,
    columns: &[impl AsRef<[f64]>],
    n: usize,
    map: &[usize],
    axes: &[usize],
) -> faer::Mat<f64> {
    let np = columns.len();
    let chain = faer::Mat::from_fn(n + map.len(), np, |i, k| {
        if i < n {
            columns[k].as_ref()[i]
        } else {
            f64::from(map[i - n] == axes[k])
        }
    });
    let mut result = faer::Mat::zeros(j.nrows(), np);
    faer::sparse::linalg::matmul::sparse_dense_matmul(
        result.as_mut(),
        faer::Accum::Replace,
        j.as_ref().as_ref(),
        chain.as_ref(),
        1.0,
        faer::Par::Seq,
    );
    result
}
/// `Jᵀv` over the state followed by the contract parameters: one faer transpose product.
fn transposed(j: &Jacobian, v: &[f64]) -> Vec<f64> {
    let mut out = vec![0.0; j.ncols()];
    faer::sparse::linalg::matmul::sparse_dense_matmul(
        faer::MatMut::from_column_major_slice_mut(&mut out, j.ncols(), 1),
        faer::Accum::Replace,
        j.as_ref().as_ref().transpose(),
        faer::MatRef::from_column_major_slice(v, v.len(), 1),
        1.0,
        faer::Par::Seq,
    );
    out
}
fn scatter_partials(
    source: faer::sparse::SparseColMatRef<'_, usize, f64>,
    slots: &PreparedLayout,
    transpose: Option<&[usize]>,
    values: &mut [f64],
) -> bool {
    for col in 0..slots.states() {
        for k in source.col_range(col) {
            let Some(target) = slots.get(col, source.row_idx()[k]) else {
                return false;
            };
            if let Some(transpose) = transpose {
                values[transpose[target]] += source.val()[k];
            } else {
                values[target] -= source.val()[k];
            }
        }
    }
    true
}
fn finish_callback(c: &mut Context<'_>, result: std::thread::Result<i32>) -> i32 {
    match result {
        Ok(flag) => flag,
        Err(_) => {
            c.callback.terminal = Some((
                crate::solve::Termination::Panic,
                "panic in IDAS callback".into(),
            ));
            -1
        }
    }
}
// All oracle calls cross CallbackState's catch_unwind boundary; buffers publish only on success.
unsafe extern "C" fn residual(
    t: f64,
    y: ffi::N_Vector,
    dy: ffi::N_Vector,
    out: ffi::N_Vector,
    data: *mut c_void,
) -> i32 {
    // SAFETY: `data` is this session's registered neutral slot; its current Context is live and not otherwise
    // borrowed while IDAS calls back on the owning thread.
    let Some(c) = (unsafe { callback_context(data) }) else {
        return -1;
    };
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let n = c.contract.states.len();
        // SAFETY: IDAS passes live serial vectors of the `n` states for the callback.
        let x = input!(c, y, n);
        let evaluation = c.evaluate(Function::Rhs, t, x, false);
        c.initial_ic_residual.get_or_insert(evaluation.is_some());
        let Some(mut e) = evaluation else {
            return c.failure();
        };
        {
            // SAFETY: IDAS supplies a live n-entry rate input. The possibly
            // aliased foreign input borrow ends before output publication.
            let yp = input!(c, dy, n);
            for (i, value) in e.values.iter_mut().enumerate() {
                *value = if c.contract.differential[i] {
                    yp[i] - *value
                } else {
                    -*value
                };
            }
        }
        // SAFETY: out is a live n-entry vector, possibly aliasing y or dy. All
        // foreign input borrows have ended; e.values is independent owned storage.
        unsafe {
            write(out, &e.values);
        }
        0
    }));
    finish_callback(c, result)
}
/// Newton matrix `cj·M − ∂f/∂y` on the admitted pattern, filled by one traversal of the
/// analytic Jacobian's state columns (F12). A partial outside the declared support is a
/// contract violation, never silently dropped.
unsafe extern "C" fn jacobian(
    t: f64,
    cj: f64,
    y: ffi::N_Vector,
    _dy: ffi::N_Vector,
    _r: ffi::N_Vector,
    matrix: ffi::SUNMatrix,
    data: *mut c_void,
    _a: ffi::N_Vector,
    _b: ffi::N_Vector,
    _d: ffi::N_Vector,
) -> i32 {
    // SAFETY: `data` is this session's registered neutral slot; its current Context is live and not otherwise
    // borrowed while IDAS calls back on the owning thread.
    let Some(c) = (unsafe { callback_context(data) }) else {
        return -1;
    };
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        crate::observation::gauge("idas.newton.shift", "inverse-time", cj);
        let n = c.contract.states.len();
        // SAFETY: IDAS passes a live serial vector of the `n` states for the callback.
        let x = input!(c, y, n);
        let Some(j) = c.partials(t, x) else {
            return c.failure();
        };
        c.newton_values.fill(0.0);
        for (col, &target) in c.layout.diagonal_slots.iter().enumerate() {
            if c.contract.differential[col] {
                c.newton_values[target] += cj;
            }
        }
        if !scatter_partials(j.as_ref().as_ref(), &c.layout, None, &mut c.newton_values) {
            c.callback.terminal = Some((
                crate::solve::Termination::Evaluation,
                "IDAS residual partial outside its declared support".into(),
            ));
            return -1;
        }
        // SAFETY: IDAS passes the KLU matrix `linear_solver` created with `n` columns and
        // room for `c.layout.rows.len()` nonzeros; `values` has one entry per pattern row.
        unsafe { publish(matrix, &c.layout.columns, &c.layout.rows, &c.newton_values) };
        0
    }));
    finish_callback(c, result)
}
/// Analytic Newton-matrix product `(cj·M − ∂f/∂y)·v` for the matrix-free Krylov routes.
unsafe extern "C" fn jtimes(
    t: f64,
    y: ffi::N_Vector,
    _dy: ffi::N_Vector,
    _r: ffi::N_Vector,
    v: ffi::N_Vector,
    jv: ffi::N_Vector,
    cj: f64,
    data: *mut c_void,
    _a: ffi::N_Vector,
    _b: ffi::N_Vector,
) -> i32 {
    // SAFETY: `data` is this session's registered neutral slot; its current Context is live and not otherwise
    // borrowed while IDAS calls back on the owning thread.
    let Some(c) = (unsafe { callback_context(data) }) else {
        return -1;
    };
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        crate::observation::gauge("idas.newton.shift", "inverse-time", cj);
        let n = c.contract.states.len();
        {
            // SAFETY: IDAS supplies live serial input vectors of n initialized
            // values. They are read-only throughout this inner scope; even if a
            // native output aliases either input, all borrows end before publish.
            let x = input!(c, y, n);
            // SAFETY: IDAS supplies a live direction vector of n initialized values.
            let direction = input!(c, v, n);
            if c.rhs_action(t, x, direction, cj).is_none() {
                return c.failure();
            }
        }
        // SAFETY: jv is a live native output of n entries; the fully qualified
        // Rust-owned product cannot overlap it. No foreign input borrow survives.
        unsafe {
            write(jv, &c.rhs_product);
        }
        0
    }));
    finish_callback(c, result)
}
/// Jacobi preconditioner setup from the compiled Newton-matrix diagonal.
unsafe extern "C" fn precondition_setup(
    t: f64,
    y: ffi::N_Vector,
    _dy: ffi::N_Vector,
    _r: ffi::N_Vector,
    cj: f64,
    data: *mut c_void,
) -> i32 {
    // SAFETY: `data` is this session's registered neutral slot; its current Context is live and not otherwise
    // borrowed while IDAS calls back on the owning thread.
    let Some(c) = (unsafe { callback_context(data) }) else {
        return -1;
    };
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        crate::observation::gauge("idas.newton.shift", "inverse-time", cj);
        let n = c.contract.states.len();
        // SAFETY: IDAS passes a live serial vector of the `n` states for the callback.
        let x = input!(c, y, n);
        let Some(()) = c.rhs_diagonal(t, x) else {
            return c.failure();
        };
        for col in 0..n {
            let diagonal = if c.contract.differential[col] {
                cj
            } else {
                0.0
            } - c.rhs_product[col];
            c.inverse_diagonal[col] = if diagonal.is_finite() && diagonal.abs() > f64::MIN_POSITIVE
            {
                diagonal.recip()
            } else {
                1.0
            };
        }
        0
    }));
    finish_callback(c, result)
}
/// Jacobi preconditioner solve `P z = r`.
unsafe extern "C" fn precondition_solve(
    _t: f64,
    _y: ffi::N_Vector,
    _dy: ffi::N_Vector,
    _r: ffi::N_Vector,
    rvec: ffi::N_Vector,
    zvec: ffi::N_Vector,
    _cj: f64,
    _delta: f64,
    data: *mut c_void,
) -> i32 {
    // SAFETY: `data` is this session's registered neutral slot; its current Context is live and not otherwise
    // borrowed while IDAS calls back on the owning thread.
    let Some(c) = (unsafe { callback_context(data) }) else {
        return -1;
    };
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let n = c.inverse_diagonal.len();
        if c.rhs_product.len() != n {
            c.callback.last_failure = Some(ProblemError::internal(
                "IDAS preconditioner workspace extent",
            ));
            return -1;
        }
        {
            // SAFETY: IDAS supplies a live n-entry preconditioner input, borrowed
            // only while the independent admitted Krylov product is filled.
            let r = input!(c, rvec, n);
            for ((output, value), inverse) in
                c.rhs_product.iter_mut().zip(r).zip(&c.inverse_diagonal)
            {
                *output = value * inverse;
            }
        }
        // SAFETY: the live n-entry native output may alias rvec. Its input borrow
        // ended above, and the session-private product cannot overlap it.
        unsafe {
            write(zvec, &c.rhs_product);
        }
        0
    }));
    finish_callback(c, result)
}
/// Forward-sensitivity residuals `M·ṡ − ∂f/∂y·s − ∂f/∂p` for every parameter at once: one
/// faer sparse × dense product replaces the dense (row, column) loops (F12).
unsafe extern "C" fn sensitivities(
    np: i32,
    t: f64,
    y: ffi::N_Vector,
    _dy: ffi::N_Vector,
    _r: ffi::N_Vector,
    ys: *mut ffi::N_Vector,
    yps: *mut ffi::N_Vector,
    rs: *mut ffi::N_Vector,
    data: *mut c_void,
    _a: ffi::N_Vector,
    _b: ffi::N_Vector,
    _d: ffi::N_Vector,
) -> i32 {
    // SAFETY: `data` is this session's registered neutral slot; its current Context is live and not otherwise
    // borrowed while IDAS calls back on the owning thread.
    let Some(c) = (unsafe { callback_context(data) }) else {
        return -1;
    };
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let n = c.contract.states.len();
        if np <= 0 || np as usize != c.axes.len() {
            return -1;
        }
        let np = np as usize;
        // SAFETY: IDAS passes live serial vectors of the `n` states for the callback.
        let x = input!(c, y, n);
        let Some(j) = c.partials(t, x) else {
            return c.failure();
        };
        let mut columns = Vec::with_capacity(np);
        for p in 0..np {
            // SAFETY: IDAS supplies np live n-entry sensitivity vectors.
            let vector = unsafe { at(ys, p) };
            columns.push(input!(c, vector, n));
        }
        let mut product = chained(&j, &columns, n, &c.map, &c.axes);
        // Only the independent matrix survives; no foreign state-sensitivity
        // borrow may remain live while any sensitivity output is published.
        drop(columns);
        for p in 0..np {
            // SAFETY: IDAS supplies np live n-entry rate vectors.
            let vector = unsafe { at(yps, p) };
            let ds = input!(c, vector, n);
            for row in 0..n {
                let rate = if c.contract.differential[row] {
                    ds[row]
                } else {
                    0.0
                };
                product[(row, p)] = rate - product[(row, p)];
            }
        }
        // Every native input borrow ended before this publication pass, even
        // when an output aliases an input belonging to another selected axis.
        for p in 0..np {
            // SAFETY: rs is the native output array with np initialized handles and p < np.
            let output = unsafe { at(rs, p) };
            // SAFETY: output has n entries; product is owned and all input borrows ended.
            unsafe { write_column(output, &product, p) };
        }
        0
    }));
    finish_callback(c, result)
}
unsafe extern "C" fn quadrature(
    t: f64,
    y: ffi::N_Vector,
    _dy: ffi::N_Vector,
    out: ffi::N_Vector,
    data: *mut c_void,
) -> i32 {
    // SAFETY: `data` is this session's registered neutral slot; its current Context is live and not otherwise
    // borrowed while IDAS calls back on the owning thread.
    let Some(c) = (unsafe { callback_context(data) }) else {
        return -1;
    };
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        // SAFETY: IDAS passes a live serial vector of the states for the callback.
        let x = input!(c, y, c.contract.states.len());
        let Some(e) = c.evaluate(Function::QuadratureFlux, t, x, false) else {
            return c.failure();
        };
        // SAFETY: `out` is IDAS's vector of the quadratures, which `evaluate` checked
        // `e.values` against.
        unsafe {
            write(out, &e.values);
        }
        0
    }));
    finish_callback(c, result)
}
/// Exact d(flux)/dp = flux_y * state_sensitivity + direct flux_p. Parameter
/// mapping changes with each scheduled segment; selected physical axes stay fixed.
unsafe extern "C" fn quadrature_sensitivities(
    ns: i32,
    t: f64,
    y: ffi::N_Vector,
    _dy: ffi::N_Vector,
    ys: *mut ffi::N_Vector,
    _yps: *mut ffi::N_Vector,
    _q: ffi::N_Vector,
    out: *mut ffi::N_Vector,
    data: *mut c_void,
    _tmp_y: ffi::N_Vector,
    _tmp_dy: ffi::N_Vector,
    _tmp_q: ffi::N_Vector,
) -> i32 {
    // SAFETY: IDAS invokes this on the owning thread with the registered live context.
    let Some(c) = (unsafe { callback_context(data) }) else {
        return -1;
    };
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        if usize::try_from(ns).ok() != Some(c.axes.len()) {
            c.callback.last_failure =
                Some(ProblemError::internal("IDAS quadrature sensitivity extent"));
            return -1;
        }
        let n = c.contract.states.len();
        // SAFETY: IDAS supplies n state values and ns live sensitivity/output vectors.
        let x = input!(c, y, n);
        let Some(e) = c.evaluate(Function::QuadratureFlux, t, x, true) else {
            return c.failure();
        };
        let Some(j) = e.jacobian else {
            return -1;
        };
        let mut columns = Vec::with_capacity(c.axes.len());
        for k in 0..c.axes.len() {
            // SAFETY: IDAS supplies one live n-entry vector per checked axis.
            let vector = unsafe { at(ys, k) };
            columns.push(input!(c, vector, n));
        }
        let product = chained(&j, &columns, n, &c.map, &c.axes);
        drop(columns);
        // Validate the complete candidate before publishing any axis.
        if (0..product.ncols())
            .any(|column| (0..product.nrows()).any(|row| !product[(row, column)].is_finite()))
        {
            c.callback.last_failure = Some(ProblemError::numerical(
                "nonfinite quadrature sensitivity flux",
            ));
            return -1;
        }
        for k in 0..c.axes.len() {
            // SAFETY: out contains one initialized native handle per checked axis.
            let output = unsafe { at(out, k) };
            // SAFETY: output has nq entries and product owns this checked column;
            // every foreign input borrow ended before this publication pass.
            unsafe { write_column(output, &product, k) };
        }
        0
    }));
    finish_callback(c, result)
}
unsafe extern "C" fn roots(
    t: f64,
    y: ffi::N_Vector,
    _dy: ffi::N_Vector,
    out: *mut f64,
    data: *mut c_void,
) -> i32 {
    // SAFETY: `data` is this session's registered neutral slot; its current Context is live and not otherwise
    // borrowed while IDAS calls back on the owning thread.
    let Some(c) = (unsafe { callback_context(data) }) else {
        return -1;
    };
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        // SAFETY: IDAS passes a live serial vector of the states for the callback.
        let x = input!(c, y, c.contract.states.len());
        let Some(e) = c.evaluate(Function::Roots, t, x, false) else {
            return -1;
        };
        // SAFETY: `out` holds one value per root function of the active mode, which
        // `evaluate` checked `e.values` against; IDAS's array cannot overlap it.
        unsafe {
            std::ptr::copy_nonoverlapping(e.values.as_ptr(), out, e.values.len());
        }
        0
    }));
    finish_callback(c, result)
}
/// The adjoint residual `M·λ' + ∂f/∂yᵀλ` of the backward problem (ADR-0110 item 3): the
/// forward residual is `M·y' − f`, so its adjoint in IDAS's form `(λᵀF_ẏ)' − λᵀF_y` needs
/// only the transposed state partials, one faer product.
unsafe extern "C" fn residual_b(
    t: f64,
    yy: ffi::N_Vector,
    _yp: ffi::N_Vector,
    yb: ffi::N_Vector,
    ypb: ffi::N_Vector,
    rr: ffi::N_Vector,
    data: *mut c_void,
) -> i32 {
    // SAFETY: `data` is this session's registered neutral slot; its current Context is live and not otherwise
    // borrowed while IDAS calls back on the owning thread.
    let Some(c) = (unsafe { callback_context(data) }) else {
        return -1;
    };
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let n = c.contract.states.len();
        // SAFETY: IDAS passes live serial vectors of the `n` states (forward) and of the
        // `n` adjoint values (first-order backward problem) for the callback.
        let x = input!(c, yy, n);
        let Some(j) = c.partials(t, x) else {
            return c.failure();
        };
        // SAFETY: as above.
        let lambda = input!(c, yb, n);
        // SAFETY: as above.
        let rate = input!(c, ypb, n);
        let symbolic = j.symbolic();
        let product = (0..n)
            .map(|col| {
                symbolic
                    .col_range(col)
                    .map(|e| j.val()[e] * lambda[symbolic.row_idx()[e]])
                    .sum::<f64>()
            })
            .collect::<Vec<_>>();
        let out: Vec<_> = (0..n)
            .map(|i| {
                let mass = if c.contract.differential[i] {
                    rate[i]
                } else {
                    0.0
                };
                mass + product[i]
            })
            .collect();
        // SAFETY: as above; `out` has `n` values.
        unsafe {
            write(rr, &out);
        }
        0
    }));
    finish_callback(c, result)
}
/// Adjoint Newton matrix `∂f/∂yᵀ + cjB·M` on the transposed pattern.
unsafe extern "C" fn jacobian_b(
    t: f64,
    cj: f64,
    yy: ffi::N_Vector,
    _yp: ffi::N_Vector,
    _yb: ffi::N_Vector,
    _ypb: ffi::N_Vector,
    _rr: ffi::N_Vector,
    matrix: ffi::SUNMatrix,
    data: *mut c_void,
    _a: ffi::N_Vector,
    _b: ffi::N_Vector,
    _d: ffi::N_Vector,
) -> i32 {
    // SAFETY: `data` is this session's registered neutral slot; its current Context is live and not otherwise
    // borrowed while IDAS calls back on the owning thread.
    let Some(c) = (unsafe { callback_context(data) }) else {
        return -1;
    };
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let n = c.contract.states.len();
        // SAFETY: IDAS passes a live serial vector of the `n` states for the callback.
        let x = input!(c, yy, n);
        let Some(j) = c.partials(t, x) else {
            return c.failure();
        };
        c.newton_values_b.fill(0.0);
        for (col, &target) in c.layout.diagonal_slots_b.iter().enumerate() {
            if c.contract.differential[col] {
                c.newton_values_b[target] += cj;
            }
        }
        if !scatter_partials(
            j.as_ref().as_ref(),
            &c.layout,
            Some(&c.layout.transpose_slots),
            &mut c.newton_values_b,
        ) {
            c.callback.terminal = Some((
                crate::solve::Termination::Evaluation,
                "IDAS residual partial outside its declared support".into(),
            ));
            return -1;
        }
        // SAFETY: IDAS passes the KLU matrix `create_backward` made with `n` columns and
        // room for `c.layout.rows_b.len()` nonzeros; `values` has one entry per pattern row.
        unsafe {
            publish(
                matrix,
                &c.layout.columns_b,
                &c.layout.rows_b,
                &c.newton_values_b,
            )
        };
        0
    }));
    finish_callback(c, result)
}
/// The gradient integrand `−∂f/∂pᵀλ` over the segment's integration columns; IDAS
/// integrates it from the backward problem's start toward the segment's start, so the
/// quadrature at the start is `∫ ∂f/∂pᵀλ dt` over the segment.
unsafe extern "C" fn quadrature_b(
    t: f64,
    yy: ffi::N_Vector,
    _yp: ffi::N_Vector,
    yb: ffi::N_Vector,
    _ypb: ffi::N_Vector,
    out: ffi::N_Vector,
    data: *mut c_void,
) -> i32 {
    // SAFETY: `data` is this session's registered neutral slot; its current Context is live and not otherwise
    // borrowed while IDAS calls back on the owning thread.
    let Some(c) = (unsafe { callback_context(data) }) else {
        return -1;
    };
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let n = c.contract.states.len();
        // SAFETY: IDAS passes live serial vectors of the `n` states (forward) and of the
        // `n` adjoint values for the callback.
        let x = input!(c, yy, n);
        let Some(j) = c.partials(t, x) else {
            return c.failure();
        };
        // SAFETY: as above.
        let lambda = input!(c, yb, n);
        let symbolic = j.symbolic();
        let mut rate = vec![0.0; c.gradient_axes.len()];
        for (selected, axis) in c.gradient_axes.iter().enumerate() {
            if let Some(q) = c.map.iter().position(|column| column == axis) {
                rate[selected] = symbolic
                    .col_range(n + q)
                    .map(|e| -j.val()[e] * lambda[symbolic.row_idx()[e]])
                    .sum();
            }
        }
        // SAFETY: `out` is the problem's quadrature vector of one value per integration
        // column (`create_backward`), the extent of `rate`.
        unsafe {
            write(out, &rate);
        }
        0
    }));
    finish_callback(c, result)
}
/// The context and integration column of a second-order backward problem's user data.
///
/// # Safety
/// `data` is the address of a live [`Direction`] whose context outlives the call.
unsafe fn directed<'c>(data: *mut c_void) -> Option<(&'c mut Context<'c>, usize)> {
    if data.is_null() {
        return None;
    }
    // SAFETY: `data` addresses a live `Direction` (the caller's contract).
    let direction = unsafe { &*data.cast::<Direction>() };
    // SAFETY: the Direction carries the same stable slot as the parent session.
    Some((
        unsafe { callback_context(direction.context) }?,
        direction.column,
    ))
}
/// The second-order adjoint residual (ADR-0110 item 4) of `[λ; μ]`, where `μ` is the
/// tangent of `λ` along one integration column `k`: `M·λ' + f_xᵀλ` as on the first-order
/// route, and `M·μ' + f_xᵀμ + [∇²(λᵀf)·d]_x` with `d = [s_k; π_k]`, the forward state
/// sensitivity and the contract parameters in effect for column `k`. The forward
/// sensitivities arrive from IDAS's checkpoints (`IDAInitBS`).
unsafe extern "C" fn residual_bs(
    t: f64,
    yy: ffi::N_Vector,
    _yp: ffi::N_Vector,
    yys: *mut ffi::N_Vector,
    _yps: *mut ffi::N_Vector,
    yb: ffi::N_Vector,
    ypb: ffi::N_Vector,
    rr: ffi::N_Vector,
    data: *mut c_void,
) -> i32 {
    // SAFETY: `data` is the problem's boxed `Direction`, registered by `create_backward`
    // and live with the session, whose context IDAS reaches only through this callback.
    let Some((c, column)) = (unsafe { directed(data) }) else {
        return -1;
    };
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let n = c.contract.states.len();
        // SAFETY: IDAS passes live serial vectors of the `n` forward states and of the
        // `2n` second-order adjoint values for the callback.
        let x = input!(c, yy, n);
        let Some(j) = c.partials(t, x) else {
            return c.failure();
        };
        // SAFETY: as above.
        let adjoint = input!(c, yb, 2 * n);
        // SAFETY: as above.
        let rate = input!(c, ypb, 2 * n);
        // SAFETY: IDAS passes one forward sensitivity vector per integration column, and
        // `column` is one of them.
        let sensitivity = unsafe { at(yys, column) };
        // SAFETY: each is a live serial vector of the `n` states.
        let sensitivity_values = input!(c, sensitivity, n);
        let d = c.direction(column, sensitivity_values);
        let Some(v) = c.curvature(Function::Rhs, t, x, &adjoint[..n], &d) else {
            return c.failure();
        };
        let first = transposed(&j, &adjoint[..n]);
        let second = transposed(&j, &adjoint[n..]);
        let mut out = vec![0.0; 2 * n];
        for i in 0..n {
            let mass = if c.contract.differential[i] { 1.0 } else { 0.0 };
            out[i] = mass * rate[i] + first[i];
            out[n + i] = mass * rate[n + i] + second[i] + v[i];
        }
        // SAFETY: `rr` is the problem's residual vector of `2n` values, the extent of `out`.
        unsafe {
            write(rr, &out);
        }
        0
    }));
    finish_callback(c, result)
}
/// Newton matrix of the second-order adjoint (ADR-0110 item 4): `∂f/∂yᵀ + cjB·M` on both
/// diagonal blocks and the exact coupling `∂(∇²(λᵀf)·d)_x/∂λ` below them, whose column j
/// is `(∇²f_j·d)_x`, one weighted Hessian per colour of rows with disjoint state support.
/// The residual is linear, so Newton converges in one correction. A block-diagonal matrix
/// was measured first: it leaves the algebraic coupling uncorrected (its error does not
/// shrink with the step), and a DAE's backward corrector failed to converge thousands of
/// times until the error test failed at the minimum step.
unsafe extern "C" fn jacobian_bs(
    t: f64,
    cj: f64,
    yy: ffi::N_Vector,
    _yp: ffi::N_Vector,
    ys: *mut ffi::N_Vector,
    _yps: *mut ffi::N_Vector,
    _yb: ffi::N_Vector,
    _ypb: ffi::N_Vector,
    _rr: ffi::N_Vector,
    matrix: ffi::SUNMatrix,
    data: *mut c_void,
    _a: ffi::N_Vector,
    _b: ffi::N_Vector,
    _d: ffi::N_Vector,
) -> i32 {
    // SAFETY: `data` is the problem's boxed `Direction`, registered by `create_backward`
    // and live with the session, whose context IDAS reaches only through this callback.
    let Some((c, column)) = (unsafe { directed(data) }) else {
        return -1;
    };
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let n = c.contract.states.len();
        // SAFETY: IDAS passes a live serial vector of the `n` forward states.
        let x = input!(c, yy, n);
        let Some(j) = c.partials(t, x) else {
            return c.failure();
        };
        let outside = |c: &mut Context<'_>| {
            c.callback.terminal = Some((
                crate::solve::Termination::Evaluation,
                "IDAS residual partial outside its declared support".into(),
            ));
            -1
        };
        let mut values = vec![0.0; c.rows_bs.len()];
        let symbolic = j.symbolic();
        for col in 0..n {
            if c.contract.differential[col] {
                for target in c.diagonal_slots_bs[col] {
                    values[target] += cj;
                }
            }
            for k in symbolic.col_range(col) {
                let Some(union) = c.layout.get(col, symbolic.row_idx()[k]) else {
                    return outside(c);
                };
                for target in c.partial_slots_bs[union] {
                    values[target] += j.val()[k];
                }
            }
        }
        // SAFETY: IDAS passes one forward sensitivity vector per integration column, and
        // `column` is one of them.
        let sensitivity = unsafe { at(ys, column) };
        // SAFETY: each is a live serial vector of the `n` states.
        let sensitivity_values = input!(c, sensitivity, n);
        let d = c.direction(column, sensitivity_values);
        for color in c.colors.clone() {
            let mut weights = vec![0.0; n];
            for row in &color {
                weights[*row] = 1.0;
            }
            let Some(v) = c.curvature(Function::Rhs, t, x, &weights, &d) else {
                return c.failure();
            };
            // Rows of one colour share no state, so each state entry belongs to one row.
            for row in color {
                for k in c.layout.columns_b[row] as usize..c.layout.columns_b[row + 1] as usize {
                    let state = c.layout.rows_b[k] as usize;
                    let target = c.coupling_slots_bs[k];
                    values[target] += v[state];
                }
            }
        }
        // SAFETY: IDAS passes the KLU matrix `create_backward` made with `2n` columns and
        // room for `c.rows_bs.len()` nonzeros; `values` has one entry per pattern row.
        unsafe { publish(matrix, &c.columns_bs, &c.rows_bs, &values) };
        0
    }));
    finish_callback(c, result)
}
/// The second-order quadrature: the gradient integrand `−f_pᵀλ`, then the Hessian
/// column's integrand `−(f_pᵀμ + [∇²(λᵀf)·d]_p)`, each over the segment's integration
/// columns.
unsafe extern "C" fn quadrature_bs(
    t: f64,
    yy: ffi::N_Vector,
    _yp: ffi::N_Vector,
    yys: *mut ffi::N_Vector,
    _yps: *mut ffi::N_Vector,
    yb: ffi::N_Vector,
    _ypb: ffi::N_Vector,
    out: ffi::N_Vector,
    data: *mut c_void,
) -> i32 {
    // SAFETY: `data` is the problem's boxed `Direction`, registered by `create_backward`
    // and live with the session, whose context IDAS reaches only through this callback.
    let Some((c, column)) = (unsafe { directed(data) }) else {
        return -1;
    };
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let n = c.contract.states.len();
        let width = c.integration.len();
        // SAFETY: IDAS passes live serial vectors of the `n` forward states and of the
        // `2n` second-order adjoint values for the callback.
        let x = input!(c, yy, n);
        let Some(j) = c.partials(t, x) else {
            return c.failure();
        };
        // SAFETY: as above.
        let adjoint = input!(c, yb, 2 * n);
        // SAFETY: IDAS passes one forward sensitivity vector per integration column, and
        // `column` is one of them.
        let sensitivity = unsafe { at(yys, column) };
        // SAFETY: each is a live serial vector of the `n` states.
        let sensitivity_values = input!(c, sensitivity, n);
        let d = c.direction(column, sensitivity_values);
        let Some(v) = c.curvature(Function::Rhs, t, x, &adjoint[..n], &d) else {
            return c.failure();
        };
        let first = transposed(&j, &adjoint[..n]);
        let second = transposed(&j, &adjoint[n..]);
        let mut rate = vec![0.0; 2 * width];
        for (q, target) in c.map.iter().enumerate() {
            rate[*target] = -first[n + q];
            rate[width + *target] = -(second[n + q] + v[n + q]);
        }
        // SAFETY: `out` is the problem's quadrature vector of `2·width` values
        // (`create_backward`), the extent of `rate`.
        unsafe {
            write(out, &rate);
        }
        0
    }));
    finish_callback(c, result)
}
/// The segment a backward pass restarts at: its start time and consistent forward state,
/// with the state sensitivities and their rates when the forward pass integrates them.
struct Start {
    time: f64,
    y: Vec<f64>,
    dy: Vec<f64>,
    sensitivities: Vec<Vec<f64>>,
    rates: Vec<Vec<f64>>,
}
/// The user data of one second-order backward problem: the shared callback context and
/// the integration column whose tangent the problem integrates.
struct Direction {
    context: *mut c_void,
    column: usize,
}
/// Native counters of one backward problem, summed over its reinitializations: steps,
/// nonlinear iterations, nonlinear convergence failures and error-test failures.
type Counters = [i64; 4];
/// One IDAS backward problem: the first-order adjoint (n values, `width` gradient
/// quadratures), or on the second-order route the adjoint with its tangent along one
/// direction (2n values; the gradient and then the Hessian column, 2·width quadratures).
struct Problem {
    which: i32,
    matrix: ffi::SUNMatrix,
    linear: ffi::SUNLinearSolver,
    y: ffi::N_Vector,
    yp: ffi::N_Vector,
    quadrature: ffi::N_Vector,
    quadrature_width: usize,
    /// The second-order user data, boxed so its address is stable while IDAS holds it.
    direction: Option<Box<Direction>>,
    counters: Counters,
    steps: StepBudget,
}
/// The IDAS adjoint of one attempt: `IDAAdjInit` checkpoints and its backward problems,
/// reinitialized at every segment and every observed sample.
struct Backward {
    /// The second-order directions; empty on the first-order route.
    directions: Vec<usize>,
    /// The backward problems, created at the first restart.
    problems: Vec<Problem>,
    /// Forward state and rates at a restart time, for `IDACalcICB`/`IDACalcICBS`.
    forward: ffi::N_Vector,
    forward_rate: ffi::N_Vector,
    /// Forward state sensitivities at a restart time and their rates, which the
    /// second-order residual does not read.
    forward_sensitivities: Vec<ffi::N_Vector>,
    forward_sensitivity_rates: Vec<ffi::N_Vector>,
    /// Every segment's consistent start, from the forward pass.
    starts: Vec<Start>,
    /// Most checkpoints held at once, and the allowance.
    checkpoints: usize,
    limit: usize,
}
impl Backward {
    fn second_order(&self) -> bool {
        !self.directions.is_empty()
    }
}
/// The adjoint values and quadratures of every backward problem.
type Adjoints = (Vec<Vec<f64>>, Vec<Vec<f64>>);
/// The forward state, its rates and its state sensitivities at a restart time.
type Forward = (Vec<f64>, Vec<f64>, Vec<Vec<f64>>);
/// Owner-affine optional IDAS allocations. It contains no current oracle or execution.
pub struct IdasWorkspace {
    identity: Arc<()>,
    held: Option<(StorageKey, NativeStorage)>,
    #[cfg(test)]
    reuses: usize,
}
impl Default for IdasWorkspace {
    fn default() -> Self {
        Self {
            identity: Arc::new(()),
            held: None,
            #[cfg(test)]
            reuses: 0,
        }
    }
}
impl IdasWorkspace {
    pub(super) fn clear(&mut self) {
        self.held = None;
    }
}
impl std::fmt::Debug for IdasWorkspace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IdasWorkspace")
            .field("retained", &self.held.is_some())
            .finish()
    }
}
struct StorageKey {
    contract: ContentHash,
    profile: pse_ids::roles::ProfileHash,
    layout: Arc<PreparedLayout>,
    forward_quadratures: bool,
}
impl StorageKey {
    fn matches(&self, other: &Self) -> bool {
        self.contract == other.contract
            && self.profile == other.profile
            && Arc::ptr_eq(&self.layout, &other.layout)
            && self.forward_quadratures == other.forward_quadratures
    }
}
struct Session<'a> {
    reused: bool,
    vector_cursor: usize,
    storage_lease: Option<Arc<dyn std::any::Any + Send + Sync>>,
    user_data: Option<Box<UserDataSlot>>,
    #[cfg(test)]
    force_backward_ic: bool,
    replay_steps: Option<StepBudget>,
    local_error: Option<local_error::LocalError>,
    observation_generation: u64,
    observation_counters: std::cell::RefCell<BTreeSet<&'static str>>,
    ctx: ffi::SUNContext,
    mem: *mut c_void,
    matrix: ffi::SUNMatrix,
    linear: ffi::SUNLinearSolver,
    vectors: Vec<ffi::N_Vector>,
    y: ffi::N_Vector,
    dy: ffi::N_Vector,
    quad: ffi::N_Vector,
    /// Forward sensitivities are integrated: requested by the profile, or the tangents
    /// of the second-order route.
    forward: bool,
    sens: Vec<ffi::N_Vector>,
    dsens: Vec<ffi::N_Vector>,
    /// Output quadratures of finished segments; IDAS restarts its quadrature at each
    /// reinitialization.
    totals: Vec<f64>,
    /// Selected quadrature-sensitivity vectors and completed-segment offsets.
    quad_sens: Vec<ffi::N_Vector>,
    quad_sens_totals: Vec<f64>,
    transition: Option<Transition>,
    /// The adjoint route's checkpoints and backward problems.
    adjoint: Option<Backward>,
    callback: Box<Context<'a>>,
    _local: PhantomData<Rc<()>>,
}
// Neutral resources transfer exclusively between one live Session and its idle owner.
struct NativeStorage {
    ctx: ffi::SUNContext,
    mem: *mut c_void,
    matrix: ffi::SUNMatrix,
    linear: ffi::SUNLinearSolver,
    vectors: Vec<ffi::N_Vector>,
    y: ffi::N_Vector,
    dy: ffi::N_Vector,
    quad: ffi::N_Vector,
    forward: bool,
    sens: Vec<ffi::N_Vector>,
    dsens: Vec<ffi::N_Vector>,
    totals: Vec<f64>,
    quad_sens: Vec<ffi::N_Vector>,
    quad_sens_totals: Vec<f64>,
    transition: Option<Transition>,
    adjoint: Option<Backward>,
    _lease: Option<Arc<dyn std::any::Any + Send + Sync>>,
    user_data: Option<Box<UserDataSlot>>,
}
impl NativeStorage {
    fn take(session: &mut Session<'_>) -> Self {
        Self {
            ctx: std::mem::replace(&mut session.ctx, std::ptr::null_mut()),
            mem: std::mem::replace(&mut session.mem, std::ptr::null_mut()),
            matrix: std::mem::replace(&mut session.matrix, std::ptr::null_mut()),
            linear: std::mem::replace(&mut session.linear, std::ptr::null_mut()),
            vectors: std::mem::take(&mut session.vectors),
            y: std::mem::replace(&mut session.y, std::ptr::null_mut()),
            dy: std::mem::replace(&mut session.dy, std::ptr::null_mut()),
            quad: std::mem::replace(&mut session.quad, std::ptr::null_mut()),
            forward: std::mem::replace(&mut session.forward, false),
            sens: std::mem::take(&mut session.sens),
            dsens: std::mem::take(&mut session.dsens),
            totals: std::mem::take(&mut session.totals),
            quad_sens: std::mem::take(&mut session.quad_sens),
            quad_sens_totals: std::mem::take(&mut session.quad_sens_totals),
            transition: session.transition.take(),
            adjoint: session.adjoint.take(),
            _lease: session.storage_lease.take(),
            user_data: session.user_data.take(),
        }
    }
    fn install(mut self, session: &mut Session<'_>) {
        session.storage_lease = self._lease.take();
        session.user_data = self.user_data.take();
        session.ctx = std::mem::replace(&mut self.ctx, std::ptr::null_mut());
        session.mem = std::mem::replace(&mut self.mem, std::ptr::null_mut());
        session.matrix = std::mem::replace(&mut self.matrix, std::ptr::null_mut());
        session.linear = std::mem::replace(&mut self.linear, std::ptr::null_mut());
        session.vectors = std::mem::take(&mut self.vectors);
        session.y = std::mem::replace(&mut self.y, std::ptr::null_mut());
        session.dy = std::mem::replace(&mut self.dy, std::ptr::null_mut());
        session.quad = std::mem::replace(&mut self.quad, std::ptr::null_mut());
        session.forward = std::mem::replace(&mut self.forward, false);
        session.sens = std::mem::take(&mut self.sens);
        session.dsens = std::mem::take(&mut self.dsens);
        session.totals = std::mem::take(&mut self.totals);
        session.quad_sens = std::mem::take(&mut self.quad_sens);
        session.quad_sens_totals = std::mem::take(&mut self.quad_sens_totals);
        session.transition = self.transition.take();
        session.adjoint = self.adjoint.take();
    }
}
impl Drop for NativeStorage {
    fn drop(&mut self) {
        // Every object below was created by this session, is freed exactly once, and the
        // integrator memory goes first so nothing native refers to the rest afterwards.
        // `IDAFree` also frees the adjoint memory and every backward problem.
        if !self.mem.is_null() {
            // SAFETY: the session's live IDAS memory; `IDAFree` nulls the pointer.
            unsafe { ffi::IDAFree(&raw mut self.mem) };
        }
        if !self.linear.is_null() {
            // SAFETY: the session's linear solver, no longer attached to any memory.
            unsafe { ffi::SUNLinSolFree(self.linear) };
        }
        if !self.matrix.is_null() {
            // SAFETY: the session's Newton matrix, freed after the solver that used it.
            unsafe { ffi::SUNMatDestroy(self.matrix) };
        }
        if let Some(b) = &self.adjoint {
            for problem in &b.problems {
                if !problem.linear.is_null() {
                    // SAFETY: as above, for the backward problem's solver.
                    unsafe { ffi::SUNLinSolFree(problem.linear) };
                }
                if !problem.matrix.is_null() {
                    // SAFETY: as above, for the backward problem's matrix.
                    unsafe { ffi::SUNMatDestroy(problem.matrix) };
                }
            }
        }
        for v in self.vectors.drain(..) {
            // SAFETY: every serial vector the session allocated, each listed once.
            unsafe { ffi::N_VDestroy(v) };
        }
        if !self.ctx.is_null() {
            // SAFETY: the session's context, freed last, after every object created in it.
            unsafe { ffi::SUNContext_Free(&raw mut self.ctx) };
        }
        // The slot allocation is covered by the same lease, which drops last.
        self.user_data = None;
    }
}
impl Drop for Session<'_> {
    fn drop(&mut self) {
        // Native teardown completes while the current callback Box still exists.
        drop(NativeStorage::take(self));
        self.local_error = None;
    }
}
/// Consistent-initialization option of `IDACalcIC`.
fn initialization_option(start: IdasInitialization) -> i32 {
    match start {
        IdasInitialization::AlgebraicAndRates => ffi::IDA_YA_YDP_INIT,
        IdasInitialization::SteadyStates => ffi::IDA_Y_INIT,
    }
}
struct StatePreparation<'a> {
    layout: Option<Arc<PreparedLayout>>,
    time: f64,
    parameters: Vec<f64>,
    map: Vec<usize>,
    trial_failures: TrialPolicy,
    rtol: f64,
    atol: &'a [f64],
    initial_step: Option<f64>,
    linear: IdasLinear,
    forward: bool,
}
impl<'a> Session<'a> {
    fn user_data(&mut self) -> *mut c_void {
        self.user_data
            .as_mut()
            .map_or(std::ptr::null_mut(), |slot| (&raw mut **slot).cast())
    }
    fn bind_current_context(&mut self) -> Result<(), ProblemError> {
        let current = (&raw mut *self.callback).cast();
        self.user_data
            .get_or_insert_with(|| {
                Box::new(UserDataSlot {
                    current: std::ptr::null_mut(),
                })
            })
            .current = current;
        let data = self.user_data();
        native!(ffi::IDASetUserData(self.mem, data), "current user data")
    }
    fn vector(&mut self, values: &[f64]) -> Result<ffi::N_Vector, ProblemError> {
        if let Some(&vector) = self.vectors.get(self.vector_cursor) {
            // SAFETY: every entry is a live serial vector exclusively owned here.
            if unsafe { ffi::N_VGetLength_Serial(vector) } != index(values.len())? {
                return Err(ProblemError::internal(
                    "incompatible retained IDAS vector extent",
                ));
            }
            self.vector_cursor += 1;
            // SAFETY: the checked vector has exactly the current input extent.
            unsafe {
                write(vector, values);
            }
            return Ok(vector);
        }
        // SAFETY: the session's live context; the vector is freed with the session.
        let v = unsafe { ffi::N_VNew_Serial(index(values.len())?, self.ctx) };
        if v.is_null() {
            return Err(ProblemError::memory("IDAS vector allocation"));
        }
        self.vectors.push(v);
        self.vector_cursor += 1;
        // SAFETY: `v` was just allocated with `values.len()` entries.
        unsafe {
            write(v, values);
        }
        Ok(v)
    }
    /// Common native state/role/sign/analytic-linear preparation, before any IC phase.
    fn prepare(
        oracle: &'a mut dyn Oracle,
        integration: &[f64],
        config: StatePreparation<'_>,
        execution: Execution,
    ) -> Result<(Self, Evaluation), ProblemError> {
        let admission = bbd::admit(config.linear, oracle.contract().states.len(), execution)?;
        Self::prepare_storage(oracle, integration, config, admission, None)
    }
    fn prepare_storage(
        oracle: &'a mut dyn Oracle,
        integration: &[f64],
        config: StatePreparation<'_>,
        admission: bbd::Admission,
        storage: Option<NativeStorage>,
    ) -> Result<(Self, Evaluation), ProblemError> {
        let c = oracle.contract().clone();
        let n = c.states.len();
        let execution = admission.into_execution(config.linear, n)?;
        let allowance = execution.memory.unwrap_or(usize::MAX);
        let layout = match config.layout {
            Some(layout) => layout,
            None => oracle.prepared_layout(allowance)?,
        };
        if layout.states() != n {
            return Err(ProblemError::internal("IDAS prepared layout state extent"));
        }
        let bytes = layout
            .scatter_bytes(DynamicDemand::Base)?
            .checked_add(layout.linear_bytes(config.linear)?)
            .and_then(|bytes| {
                layout
                    .action_bytes(config.linear, c.parameters.len())
                    .ok()
                    .and_then(|actions| bytes.checked_add(actions))
            })
            .ok_or_else(|| ProblemError::memory("IDAS selected workspace extent"))?;
        if bytes > allowance {
            return Err(ProblemError::memory(
                "IDAS selected workspace exceeds foreign allowance",
            ));
        }
        crate::observation::integer(
            "idas.layout.nonzeros",
            "entries",
            i64::try_from(layout.nonzeros()).unwrap_or(i64::MAX),
        );
        crate::observation::integer(
            "idas.layout.bytes",
            "bytes",
            i64::try_from(layout.retained_bytes()).unwrap_or(i64::MAX),
        );
        crate::observation::integer(
            "idas.linear.workspace.bytes",
            "bytes",
            i64::try_from(layout.linear_bytes(config.linear)?).unwrap_or(i64::MAX),
        );
        let opaque_fill = if config.linear == IdasLinear::Klu {
            n.checked_mul(n)
                .and_then(|cells| cells.checked_mul(32))
                .ok_or_else(|| ProblemError::memory("IDAS direct fill allowance extent"))?
        } else {
            0
        };
        crate::observation::integer(
            "idas.linear.opaque_factor_allowance.bytes",
            "bytes",
            i64::try_from(opaque_fill).unwrap_or(i64::MAX),
        );
        crate::observation::integer(
            "idas.linear.known_workspace_allowance.bytes",
            "bytes",
            i64::try_from(layout.linear_bytes(config.linear)? - opaque_fill).unwrap_or(i64::MAX),
        );
        crate::observation::integer(
            "idas.callback.action_workspace.bytes",
            "bytes",
            i64::try_from(layout.action_bytes(config.linear, c.parameters.len())?)
                .unwrap_or(i64::MAX),
        );
        let action = config.linear != IdasLinear::Klu;
        let rhs_direction = if action {
            vec![0.; n + c.parameters.len()]
        } else {
            Vec::new()
        };
        let rhs_product = if action { vec![0.; n] } else { Vec::new() };
        let map = config.map;
        let newton_values = vec![0.0; layout.rows.len()];
        let newton_values_b = vec![0.0; layout.rows_b.len()];
        let mut s = Self {
            reused: false,
            vector_cursor: 0,
            storage_lease: None,
            user_data: None,
            #[cfg(test)]
            force_backward_ic: false,
            replay_steps: None,
            local_error: None,
            observation_generation: crate::observation::generation(),
            observation_counters: std::cell::RefCell::default(),
            ctx: std::ptr::null_mut(),
            mem: std::ptr::null_mut(),
            matrix: std::ptr::null_mut(),
            linear: std::ptr::null_mut(),
            y: std::ptr::null_mut(),
            dy: std::ptr::null_mut(),
            quad: std::ptr::null_mut(),
            vectors: vec![],
            forward: config.forward,
            sens: vec![],
            dsens: vec![],
            totals: vec![0.0; c.quadratures.len()],
            quad_sens: Vec::new(),
            quad_sens_totals: Vec::new(),
            transition: None,
            adjoint: None,
            callback: Box::new(Context {
                oracle,
                contract: c,
                parameters: config.parameters,
                integration: integration.to_vec(),
                axes: Vec::new(),
                gradient_axes: Vec::new(),
                map,
                mode: 0,
                trial_policy: config.trial_failures,
                callback: CallbackState::new(execution),
                initial_ic_residual: None,
                sample_elimination: SampleEliminationPreparation::with_layout(layout.clone()),
                layout,
                newton_values,
                newton_values_b,
                rhs_direction,
                rhs_product,
                columns_bs: Vec::new(),
                rows_bs: Vec::new(),
                partial_slots_bs: Vec::new(),
                diagonal_slots_bs: Vec::new(),
                coupling_slots_bs: Vec::new(),
                colors: Vec::new(),
                inverse_diagonal: vec![1.0; n],
                #[cfg(test)]
                coupled_probe: None,
            }),
            _local: PhantomData,
        };
        if let Some(storage) = storage {
            storage.install(&mut s);
            s.reused = true;
            s.sens.clear();
            s.dsens.clear();
            s.quad_sens.clear();
            s.totals.fill(0.);
            s.quad_sens_totals.fill(0.);
            s.transition = None;
        }
        if s.ctx.is_null() {
            native!(ffi::SUNContext_Create(0, &raw mut s.ctx), "context")?;
            #[cfg(test)]
            workspace_tests::record_context_construction();
        }
        s.local_error = local_error::LocalError::prepare(n);
        let Some(initial) = s.callback.evaluate(
            Function::Initial,
            config.time,
            &vec![0.0; n],
            config.forward,
        ) else {
            return Err(s.callback.failed("initial function"));
        };
        s.y = s.vector(&initial.values)?;
        s.dy = s.vector(&vec![0.0; n])?;
        let ids: Vec<_> = s
            .callback
            .contract
            .differential
            .iter()
            .map(|v| f64::from(*v))
            .collect();
        let id = s.vector(&ids)?;
        let atol = s.vector(config.atol)?;
        if !s.reused {
            // SAFETY: SUNContext_Create succeeded for this session; its Drop owns the new memory.
            s.mem = unsafe { ffi::IDACreate(s.ctx) };
        }
        if s.mem.is_null() {
            return Err(ProblemError::memory("IDAS memory allocation"));
        }
        if s.reused {
            native!(
                ffi::IDAReInit(s.mem, config.time, s.y, s.dy),
                "independent state reset"
            )?;
        } else {
            native!(
                ffi::IDAInit(s.mem, Some(residual), config.time, s.y, s.dy),
                "initialization"
            )?;
        }
        // The boxed context has a stable address for the session's lifetime.
        s.bind_current_context()?;
        native!(ffi::IDASetId(s.mem, id), "differential identities")?;
        native!(
            ffi::IDASVtolerances(s.mem, config.rtol, atol),
            "state tolerances",
        )?;
        native!(
            ffi::IDASetInitStep(s.mem, config.initial_step.unwrap_or(0.)),
            "initial step"
        )?;
        s.linear_solver(config.linear)?;
        // The authored bounds' signs keep the steps in the domain (ADR-0119 Outcome 4).
        if s.callback
            .contract
            .signs
            .iter()
            .any(|v| *v != StateSign::Free)
        {
            let codes: Vec<_> = s
                .callback
                .contract
                .signs
                .iter()
                .map(|v| state_sign_code(*v))
                .collect();
            let constraints = s.vector(&codes)?;
            native!(
                ffi::IDASetConstraints(s.mem, constraints),
                "sign constraints",
            )?;
        } else {
            native!(
                ffi::IDASetConstraints(s.mem, std::ptr::null_mut()),
                "free sign constraints"
            )?;
        }
        Ok((s, initial))
    }
    #[cfg(test)]
    fn new(
        oracle: &'a mut dyn Oracle,
        integration: &[f64],
        p: &Profile,
        directions: &[usize],
        layout: Option<Arc<PreparedLayout>>,
        execution: Execution,
        forward_quadratures: bool,
    ) -> Result<Self, ProblemError> {
        let admission = bbd::admit(p.idas.linear, oracle.contract().states.len(), execution)?;
        Self::new_storage(
            oracle,
            integration,
            p,
            directions,
            layout,
            admission,
            forward_quadratures,
            None,
        )
    }
    #[expect(
        clippy::too_many_arguments,
        reason = "the current scientific request and optional neutral native resources have separate owners"
    )]
    fn new_storage(
        oracle: &'a mut dyn Oracle,
        integration: &[f64],
        p: &Profile,
        directions: &[usize],
        layout: Option<Arc<PreparedLayout>>,
        admission: bbd::Admission,
        forward_quadratures: bool,
        storage: Option<NativeStorage>,
    ) -> Result<Self, ProblemError> {
        let forward = p.forward() || !directions.is_empty();
        let map = p.columns_at(oracle.contract().parameters.len(), p.start);
        let (mut s, initial) = Self::prepare_storage(
            oracle,
            integration,
            StatePreparation {
                layout,
                time: p.start,
                parameters: p.parameters_at(integration, p.start),
                map,
                trial_failures: p.trial_failures,
                rtol: p.rtol,
                atol: &p.atol,
                initial_step: match p.idas.native_initial_step {
                    IdasInitialStep::Configured => Some(p.initial_step),
                    IdasInitialStep::Automatic => None,
                },
                linear: p.idas.linear,
                forward,
            },
            admission,
            storage,
        )?;
        let n = s.callback.contract.states.len();
        s.callback.axes = if p.sensitivity == DynamicSensitivity::Forward {
            p.forward_axes(s.callback.contract.parameters.len())
        } else if !directions.is_empty() {
            (0..integration.len()).collect()
        } else {
            Vec::new()
        };
        s.callback.gradient_axes = if p.sensitivity == DynamicSensitivity::Adjoint {
            p.adjoint_axes(s.callback.contract.parameters.len())
        } else {
            Vec::new()
        };
        if forward {
            let j = initial
                .jacobian
                .ok_or_else(|| ProblemError::internal("IDAS initial sensitivity missing"))?;
            let symbolic = j.symbolic();
            // Each integration parameter starts from its contract parameter's initial
            // partials when the first segment uses it, and from zero otherwise.
            for k in 0..s.callback.axes.len() {
                let mut values = vec![0.0; n];
                if let Some(contract) = s.callback.map.iter().position(|c| *c == s.callback.axes[k])
                {
                    for e in symbolic.col_range(n + contract) {
                        values[symbolic.row_idx()[e]] = j.val()[e];
                    }
                }
                let v = s.vector(&values)?;
                s.sens.push(v);
                let v = s.vector(&vec![0.0; n])?;
                s.dsens.push(v);
            }
            if s.reused {
                native!(
                    ffi::IDASensReInit(
                        s.mem,
                        corrector(p.idas.sensitivity),
                        s.sens.as_mut_ptr(),
                        s.dsens.as_mut_ptr()
                    ),
                    "independent sensitivity reset"
                )?;
            } else {
                native!(
                    ffi::IDASensInit(
                        s.mem,
                        s.callback
                            .axes
                            .len()
                            .try_into()
                            .map_err(|_| ProblemError::unsupported("IDAS parameter extent"))?,
                        corrector(p.idas.sensitivity),
                        Some(sensitivities),
                        s.sens.as_mut_ptr(),
                        s.dsens.as_mut_ptr(),
                    ),
                    "forward sensitivities",
                )?;
            }
            let mut atol_s = Vec::new();
            let scales = p.integration_parameters(&p.parameter_scales);
            for k in 0..s.callback.axes.len() {
                let scale = scales[s.callback.axes[k]];
                let tolerances: Vec<_> = p.atol.iter().map(|v| v / scale).collect();
                atol_s.push(s.vector(&tolerances)?);
            }
            native!(
                ffi::IDASensSVtolerances(s.mem, p.rtol, atol_s.as_mut_ptr()),
                "sensitivity tolerances",
            )?;
            native!(ffi::IDASetSensErrCon(s.mem, 1), "sensitivity error control")?;
        }
        if !s.callback.contract.quadratures.is_empty() {
            s.quad = s.vector(&vec![0.0; s.callback.contract.quadratures.len()])?;
            let atol = s.vector(&p.out_atol)?;
            if s.reused {
                native!(
                    ffi::IDAQuadReInit(s.mem, s.quad),
                    "independent quadrature reset"
                )?;
            } else {
                native!(
                    ffi::IDAQuadInit(s.mem, Some(quadrature), s.quad),
                    "quadrature"
                )?;
            }
            native!(
                ffi::IDAQuadSVtolerances(
                    s.mem,
                    p.out_rtol
                        .ok_or_else(|| contract("quadrature relative tolerance"))?,
                    atol,
                ),
                "quadrature tolerances",
            )?;
            native!(ffi::IDASetQuadErrCon(s.mem, 1), "quadrature error control")?;
        }
        if p.sensitivity == DynamicSensitivity::Adjoint {
            s.adjoint_init(p, directions)?;
        }
        s.initialize_roots()?;
        if forward_quadratures {
            // IDACalcIC runs IDAInitialSetup and marks it complete. Install the
            // quadrature sensitivity module first so that setup initializes its
            // initial derivative history, before the first physical step.
            s.enable_forward_quadratures(p)?;
        }
        s.consistent(p.start, p, initialization_option(p.idas.initialization))?;
        Ok(s)
    }
    /// `IDAAdjInit` with Hermite interpolation and `steps_between_checkpoints` steps
    /// between checkpoints; the forward pass then runs through `IDASolveF`, and stores the
    /// state sensitivities at its checkpoints when the second-order route integrates them.
    fn adjoint_init(&mut self, p: &Profile, directions: &[usize]) -> Result<(), ProblemError> {
        let n = self.callback.contract.states.len();
        let width = self.callback.gradient_axes.len();
        let steps: c_long = p
            .adjoint
            .steps_between_checkpoints
            .into_inner()
            .try_into()
            .map_err(|_| ProblemError::unsupported("IDAS checkpoint interval extent"))?;
        native!(
            ffi::IDAAdjInit(self.mem, steps, ffi::IDA_HERMITE),
            "adjoint initialization",
        )?;
        let checkpoint_limit: i32 = p
            .adjoint
            .max_checkpoints
            .into_inner()
            .try_into()
            .map_err(|_| ProblemError::unsupported("IDAS checkpoint population extent"))?;
        native!(
            ffi::IDASetCheckpointLimit(self.mem, checkpoint_limit.into()),
            "adjoint checkpoint population limit"
        )?;
        let zeros = vec![0.0; n];
        let mut sensitivities = Vec::new();
        let mut rates = Vec::new();
        if !directions.is_empty() {
            if let Some(allowance) = self.callback.callback.execution.memory
                && self
                    .callback
                    .layout
                    .scatter_bytes(DynamicDemand::ExactHessian)?
                    .gt(&allowance)
            {
                return Err(ProblemError::memory(
                    "IDAS second-order scatter exceeds foreign allowance",
                ));
            }
            // The transposed pattern of `∂f/∂yᵀ` on both diagonal blocks of the 2n Newton
            // matrix and on the coupling block `∂(∇²(λᵀf)·d)_x/∂λ` below them: column j of
            // the coupling holds `(∇²f_j·d)_x`, nonzero only where f_j depends on the state.
            let (columns, rows) = (
                &self.callback.layout.columns_b,
                &self.callback.layout.rows_b,
            );
            let mut pairs = Vec::with_capacity(3 * rows.len());
            for (column, row) in [(0, 0), (0, n), (n, n)] {
                for col in 0..n {
                    for &entry in &rows[columns[col] as usize..columns[col + 1] as usize] {
                        pairs.push((column + col, row + entry as usize));
                    }
                }
            }
            pairs.sort_unstable();
            // Greedy colouring of the residual rows by their state support.
            let mut colors: Vec<Vec<usize>> = Vec::with_capacity(n);
            let mut owned: Vec<Vec<i64>> = Vec::with_capacity(n);
            for j in 0..n {
                let support = &rows[columns[j] as usize..columns[j + 1] as usize];
                let color = owned
                    .iter()
                    .position(|states| support.iter().all(|i| states.binary_search(i).is_err()))
                    .unwrap_or_else(|| {
                        owned.push(Vec::with_capacity(support.len()));
                        colors.push(Vec::with_capacity(1));
                        owned.len() - 1
                    });
                owned[color].extend_from_slice(support);
                owned[color].sort_unstable();
                colors[color].push(j);
            }
            self.callback.colors = colors;
            (self.callback.columns_bs, self.callback.rows_bs) = csc(2 * n, &pairs)?;
            let c = &mut self.callback;
            let columns_bs = &c.columns_bs;
            let rows_bs = &c.rows_bs;
            let target = |col, row| {
                slot(columns_bs, rows_bs, col, row)
                    .ok_or_else(|| ProblemError::internal("IDAS second-order scatter entry absent"))
            };
            c.partial_slots_bs = (0..n)
                .flat_map(|col| {
                    (c.layout.columns[col] as usize..c.layout.columns[col + 1] as usize)
                        .map(move |k| (col, k))
                })
                .map(|(col, k)| {
                    let row = c.layout.rows[k] as usize;
                    Ok([target(row, col)?, target(n + row, n + col)?])
                })
                .collect::<Result<_, ProblemError>>()?;
            c.diagonal_slots_bs = (0..n)
                .map(|i| Ok([target(i, i)?, target(n + i, n + i)?]))
                .collect::<Result<_, ProblemError>>()?;
            c.coupling_slots_bs = (0..n)
                .flat_map(|row| {
                    (c.layout.columns_b[row] as usize..c.layout.columns_b[row + 1] as usize)
                        .map(move |k| (row, k))
                })
                .map(|(row, k)| target(row, n + c.layout.rows_b[k] as usize))
                .collect::<Result<_, _>>()?;

            for _ in 0..width {
                sensitivities.push(self.vector(&zeros)?);
                rates.push(self.vector(&zeros)?);
            }
        }
        self.adjoint = Some(Backward {
            directions: directions.to_vec(),
            problems: Vec::new(),
            forward: self.vector(&zeros)?,
            forward_rate: self.vector(&zeros)?,
            forward_sensitivities: sensitivities,
            forward_sensitivity_rates: rates,
            starts: Vec::new(),
            checkpoints: 0,
            limit: p.adjoint.max_checkpoints.into_inner(),
        });
        Ok(())
    }
    /// Direct KLU over the compiled Jacobian, or matrix-free SPGMR/SPFGMR over analytic
    /// products with an optional Jacobi left preconditioner (IDAS supports left only).
    fn linear_solver(&mut self, linear: IdasLinear) -> Result<(), ProblemError> {
        let n = self.callback.contract.states.len();
        match linear {
            IdasLinear::Klu => {
                if self.matrix.is_null() {
                    // SAFETY: this session owns a live context. Checked native dimensions
                    // and admitted CSC nonzeros match the prepared original state layout.
                    self.matrix = unsafe {
                        ffi::SUNSparseMatrix(
                            index(n)?,
                            index(n)?,
                            index(self.callback.layout.rows.len())?,
                            0,
                            self.ctx,
                        )
                    };
                }
                if self.matrix.is_null() {
                    return Err(ProblemError::memory("IDAS matrix allocation"));
                }
                if self.linear.is_null() {
                    // SAFETY: the session owns the live state vector, dimension-matched
                    // matrix and context; its Drop releases the newly created KLU solver.
                    self.linear = unsafe { ffi::SUNLinSol_KLU(self.y, self.matrix, self.ctx) };
                }
                if self.linear.is_null() {
                    return Err(ProblemError::memory("IDAS KLU allocation"));
                }
                native!(
                    ffi::IDASetLinearSolver(self.mem, self.linear, self.matrix),
                    "KLU",
                )?;
                native!(
                    ffi::IDASetJacFn(self.mem, Some(jacobian)),
                    "analytic Jacobian",
                )?;
            }
            IdasLinear::Spgmr { dimension, .. }
            | IdasLinear::Spfgmr { dimension, .. }
            | IdasLinear::SpgmrBbd { dimension, .. } => {
                let dimension = i32::try_from(dimension.into_inner())
                    .map_err(|_| contract("IDAS Krylov dimension"))?;
                let preconditioner = match linear {
                    IdasLinear::Spgmr { preconditioner, .. }
                    | IdasLinear::Spfgmr { preconditioner, .. } => preconditioner,
                    _ => Preconditioner::None,
                };
                let side = if matches!(linear, IdasLinear::SpgmrBbd { .. }) {
                    ffi::SUN_PREC_LEFT as i32
                } else {
                    (match preconditioner {
                        Preconditioner::None => ffi::SUN_PREC_NONE,
                        Preconditioner::Jacobi => ffi::SUN_PREC_LEFT,
                        Preconditioner::BlockFactor => {
                            return Err(ProblemError::Unsupported(
                                "IDAS block factor preconditioning is not supplied".into(),
                            ));
                        }
                    }) as i32
                };
                if self.linear.is_null() {
                    self.linear = if !matches!(linear, IdasLinear::Spfgmr { .. }) {
                        // SAFETY: the session's live state vector and context; `Drop` frees
                        // the solver.
                        unsafe { ffi::SUNLinSol_SPGMR(self.y, side, dimension, self.ctx) }
                    } else {
                        // SAFETY: as above.
                        unsafe { ffi::SUNLinSol_SPFGMR(self.y, side, dimension, self.ctx) }
                    };
                }
                if self.linear.is_null() {
                    return Err(ProblemError::memory("IDAS Krylov allocation"));
                }
                native!(
                    ffi::IDASetLinearSolver(self.mem, self.linear, std::ptr::null_mut()),
                    "Krylov linear solver",
                )?;
                native!(
                    ffi::IDASetJacTimes(self.mem, None, Some(jtimes)),
                    "analytic Jacobian products",
                )?;
                if matches!(linear, IdasLinear::SpgmrBbd { .. }) {
                    bbd::install(self, linear)?;
                } else if preconditioner == Preconditioner::Jacobi {
                    native!(
                        ffi::IDASetPreconditioner(
                            self.mem,
                            Some(precondition_setup),
                            Some(precondition_solve),
                        ),
                        "Jacobi preconditioner",
                    )?;
                } else {
                    native!(
                        ffi::IDASetPreconditioner(self.mem, None, None),
                        "no preconditioner"
                    )?;
                }
            }
        }
        Ok(())
    }
    /// Root functions of the active mode, with their declared crossing directions.
    fn initialize_roots(&mut self) -> Result<(), ProblemError> {
        let events = &self.callback.contract.events[self.callback.mode];
        let mut directions: Vec<i32> = events.iter().map(|e| root_direction(e.direction)).collect();
        let count = events.len();
        // Same-count IDARootInit preserves old direction and active-root state.
        // Remove that population before binding a mode or independent request.
        native!(ffi::IDARootInit(self.mem, 0, None), "clear previous roots")?;
        native!(
            ffi::IDARootInit(
                self.mem,
                count
                    .try_into()
                    .map_err(|_| ProblemError::unsupported("IDAS root extent"))?,
                if count == 0 { None } else { Some(roots) },
            ),
            "roots",
        )?;
        // IDAS copies the `count` directions.
        if count != 0 {
            native!(
                ffi::IDASetRootDirection(self.mem, directions.as_mut_ptr()),
                "root directions",
            )?;
        }
        Ok(())
    }
    fn consistent(&mut self, t: f64, p: &Profile, option: i32) -> Result<(), ProblemError> {
        // `tout1` only orients and scales the initialization step; nothing is integrated.
        let toward = if p.end > t {
            t + (p.end - t).min(p.initial_step)
        } else {
            t + p.initial_step
        };
        let mode = if option == ffi::IDA_YA_YDP_INIT {
            IdasInitialization::AlgebraicAndRates
        } else {
            IdasInitialization::SteadyStates
        };
        let flag = self.calculate_consistent(toward, mode, &p.idas.initial_conditions)?;
        if flag < 0 {
            return Err(self
                .callback
                .native_failure(flag, "consistent initial conditions"));
        }
        native!(
            ffi::IDAGetConsistentIC(self.mem, self.y, self.dy),
            "consistent state retrieval",
        )?;
        if self.forward {
            native!(
                ffi::IDAGetSensConsistentIC(
                    self.mem,
                    self.sens.as_mut_ptr(),
                    self.dsens.as_mut_ptr(),
                ),
                "consistent sensitivity retrieval",
            )?;
        }
        Ok(())
    }
    /// Configure the native IC phase before calling it, preserving the actual return.
    fn calculate_consistent(
        &mut self,
        toward: f64,
        mode: IdasInitialization,
        controls: &IdasInitialConditions,
    ) -> Result<i32, ProblemError> {
        controls.validate_for(
            mode,
            self.callback
                .contract
                .signs
                .iter()
                .any(|sign| *sign != StateSign::Free),
        )?;
        self.callback.callback.execution.check()?;
        native!(
            ffi::IDASetMaxNumJacsIC(self.mem, controls.jacobian_attempts as i32),
            "IC Jacobian attempts"
        )?;
        native!(
            ffi::IDASetMaxNumItersIC(self.mem, controls.newton_iterations as i32),
            "IC Newton iterations"
        )?;
        native!(
            ffi::IDASetNonlinConvCoefIC(self.mem, controls.convergence_coefficient),
            "IC convergence coefficient"
        )?;
        native!(
            ffi::IDASetLineSearchOffIC(self.mem, i32::from(!controls.line_search)),
            "IC line search"
        )?;
        if let Some(value) = controls.step_trials {
            native!(
                ffi::IDASetMaxNumStepsIC(self.mem, value as i32),
                "IC artificial step trials"
            )?;
        }
        if let Some(value) = controls.backtracks {
            native!(
                ffi::IDASetMaxBacksIC(self.mem, value as i32),
                "IC backtracks"
            )?;
        }
        if let Some(value) = controls.step_tolerance {
            native!(
                ffi::IDASetStepToleranceIC(self.mem, value),
                "IC step tolerance"
            )?;
        }
        self.callback.initial_ic_residual = None;
        // SAFETY: live IDAS memory and contained callbacks on this session's thread.
        Ok(crate::observation::scoped(
            "idas.consistent_initialization",
            || unsafe { ffi::IDACalcIC(self.mem, initialization_option(mode), toward) },
        ))
    }
    /// Restart the same native memory at a scheduled change or an event reset: the
    /// differential states (and their sensitivities) carry over, `IDACalcIC` recomputes
    /// the algebraic states and every rate under the new parameters or mode.
    fn restart(
        &mut self,
        t: f64,
        state: &[f64],
        p: &Profile,
        mode_changed: bool,
    ) -> Result<(), ProblemError> {
        if self.forward {
            crate::observation::count("idas.forward_sensitivity.reinitialize", 1);
            // The carried sensitivities and their rates (the rates are only guesses
            // for `IDACalcIC`).
            native!(
                ffi::IDAGetSensDky(self.mem, t, 0, self.sens.as_mut_ptr()),
                "segment sensitivities",
            )?;
            native!(
                ffi::IDAGetSensDky(self.mem, t, 1, self.dsens.as_mut_ptr()),
                "segment sensitivity rates",
            )?;
        }
        if !self.quad_sens.is_empty() {
            self.quad_sens_totals = self.forward_quadrature_product(t)?;
        }
        if !self.quad.is_null() {
            native!(
                ffi::IDAGetQuadDky(self.mem, t, 0, self.quad),
                "segment quadrature",
            )?;
            // SAFETY: the session's quadrature vector holds one value per total.
            let finished = unsafe { read(self.quad, self.totals.len()) };
            for (total, value) in self.totals.iter_mut().zip(finished) {
                *total += value;
            }
        }
        // SAFETY: the session's state vector holds the `n` states `state` carries.
        unsafe { write(self.y, state) };
        native!(
            ffi::IDAReInit(self.mem, t, self.y, self.dy),
            "reinitialization",
        )?;
        bbd::install(self, p.idas.linear)?;
        self.observation_generation = crate::observation::generation();
        self.observation_counters.borrow_mut().clear();
        crate::observation::count("idas.segment.reinitializations", 1);
        // The adjoint route keeps one segment's checkpoints at a time.
        if self.adjoint.is_some() {
            native!(ffi::IDAAdjReInit(self.mem), "adjoint reinitialization")?;
        }
        if self.forward {
            native!(
                ffi::IDASensReInit(
                    self.mem,
                    corrector(p.idas.sensitivity),
                    self.sens.as_mut_ptr(),
                    self.dsens.as_mut_ptr(),
                ),
                "sensitivity reinitialization",
            )?;
        }
        if !self.quad.is_null() {
            // SAFETY: the session's quadrature vector holds one value per total.
            unsafe { write(self.quad, &vec![0.0; self.totals.len()]) };
            native!(
                ffi::IDAQuadReInit(self.mem, self.quad),
                "quadrature reinitialization",
            )?;
        }
        if !self.quad_sens.is_empty() {
            for vector in &self.quad_sens {
                // SAFETY: each owned serial vector has one entry per quadrature.
                unsafe { write(*vector, &vec![0.; self.totals.len()]) };
            }
            native!(
                ffi::IDAQuadSensReInit(self.mem, self.quad_sens.as_mut_ptr()),
                "quadrature sensitivity reinitialization"
            )?;
        }
        if mode_changed {
            self.initialize_roots()?;
        }
        self.consistent(t, p, ffi::IDA_YA_YDP_INIT)
    }
    fn sample(&mut self, t: f64, stepped: bool) -> Result<Sample, ProblemError> {
        let n = self.callback.contract.states.len();
        // SAFETY: the session's state vector holds the `n` states.
        let x = unsafe { read(self.y, n) };
        let Some(e) = self
            .callback
            .evaluate(Function::Output, t, &x, self.forward)
        else {
            return Err(self.callback.failed("output"));
        };
        let mut state_sensitivities = Vec::new();
        let mut output_sensitivities = Vec::new();
        if self.forward {
            if stepped {
                native!(
                    ffi::IDAGetSensDky(self.mem, t, 0, self.sens.as_mut_ptr()),
                    "sensitivity output",
                )?;
            }
            let columns: Vec<_> = self
                .sens
                .iter()
                // SAFETY: each sensitivity vector of the session holds the `n` states.
                .map(|v| unsafe { read(*v, n) })
                .collect();
            state_sensitivities = (0..n)
                .flat_map(|i| columns.iter().map(move |c| c[i]))
                .collect();
            let j = e
                .jacobian
                .as_ref()
                .ok_or_else(|| ProblemError::internal("IDAS output derivative missing"))?;
            let product = chained(j, &columns, n, &self.callback.map, &self.callback.axes);
            output_sensitivities = (0..e.values.len())
                .flat_map(|row| (0..columns.len()).map(move |k| (row, k)))
                .map(|(row, k)| product[(row, k)])
                .collect();
        }
        let integrals = self.integrals(t, stepped)?;
        Ok(Sample {
            mode: self.callback.mode,
            time: t,
            state: x,
            outputs: e.values,
            state_sensitivities,
            output_sensitivities,
            integrals,
        })
    }
    fn capture_endpoint(
        &mut self,
        r: &mut Report,
        time: f64,
        stepped: bool,
        event: Option<SemanticId>,
    ) -> Result<(), ProblemError> {
        let point = self.sample(time, stepped)?;
        r.endpoint = Some(TrajectoryEndpoint {
            point,
            event,
            input_columns: self.callback.map.clone(),
            inputs: self.callback.parameters.clone(),
        });
        Ok(())
    }
    fn enable_forward_quadratures(&mut self, p: &Profile) -> Result<(), ProblemError> {
        let k = self.callback.axes.len();
        let nq = self.callback.contract.quadratures.len();
        self.quad_sens_totals = vec![0.; k * nq];
        if k == 0 || nq == 0 {
            return Ok(());
        }
        let scales = p.integration_parameters(&p.parameter_scales);
        let mut tolerances = Vec::with_capacity(k);
        for axis in self.callback.axes.clone() {
            let vector = self.vector(&vec![0.; nq])?;
            self.quad_sens.push(vector);
            let values = p
                .out_atol
                .iter()
                .map(|v| v / scales[axis])
                .collect::<Vec<_>>();
            tolerances.push(self.vector(&values)?);
        }
        if self.reused {
            native!(
                ffi::IDAQuadSensReInit(self.mem, self.quad_sens.as_mut_ptr()),
                "independent quadrature sensitivity reset"
            )?;
        } else {
            native!(
                ffi::IDAQuadSensInit(
                    self.mem,
                    Some(quadrature_sensitivities),
                    self.quad_sens.as_mut_ptr()
                ),
                "forward quadrature sensitivities"
            )?;
        }
        native!(
            ffi::IDAQuadSensSVtolerances(
                self.mem,
                p.out_rtol
                    .ok_or_else(|| contract("quadrature relative tolerance"))?,
                tolerances.as_mut_ptr()
            ),
            "quadrature sensitivity tolerances"
        )?;
        native!(
            ffi::IDASetQuadSensErrCon(self.mem, 1),
            "quadrature sensitivity error control"
        )
    }
    fn forward_quadrature_product(&mut self, time: f64) -> Result<Vec<f64>, ProblemError> {
        let k = self.quad_sens.len();
        if k == 0 {
            return Ok(Vec::new());
        }
        native!(
            ffi::IDAGetQuadSensDky(self.mem, time, 0, self.quad_sens.as_mut_ptr()),
            "quadrature sensitivity output"
        )?;
        let nq = self.callback.contract.quadratures.len();
        let mut product = self.quad_sens_totals.clone();
        for (axis, vector) in self.quad_sens.iter().enumerate() {
            // SAFETY: this session owns each nq-entry serial vector.
            let values = unsafe { read(*vector, nq) };
            for (q, value) in values.into_iter().enumerate() {
                product[q * k + axis] += value;
            }
        }
        if product.iter().any(|v| !v.is_finite()) {
            return Err(ProblemError::numerical(
                "nonfinite forward quadrature product",
            ));
        }
        Ok(product)
    }
    fn integrals(&mut self, t: f64, stepped: bool) -> Result<Vec<f64>, ProblemError> {
        let integrals = if self.quad.is_null() {
            vec![]
        } else {
            if stepped {
                native!(
                    ffi::IDAGetQuadDky(self.mem, t, 0, self.quad),
                    "quadrature output",
                )?;
            }
            // SAFETY: the session's quadrature vector holds one value per total.
            unsafe { read(self.quad, self.totals.len()) }
                .iter()
                .zip(&self.totals)
                .map(|(v, total)| v + total)
                .collect()
        };
        Ok(integrals)
    }
    fn inventories(&mut self, time: f64, state: &[f64]) -> Result<Vec<f64>, ProblemError> {
        if self.callback.contract.balances.is_empty() {
            return Ok(vec![]);
        }
        self.callback
            .evaluate(Function::Inventory, time, state, false)
            .map(|v| v.values)
            .ok_or_else(|| self.callback.failed("conserved inventory"))
    }
    fn conserve(
        &mut self,
        report: &mut Report,
        time: f64,
        state: &[f64],
        stepped: bool,
    ) -> Result<(), ProblemError> {
        let inventories = self.inventories(time, state)?;
        let integrals = self.integrals(time, stepped)?;
        let transfers = cumulative_transfers(&self.callback.contract, report);
        observe_conservation(
            &self.callback.contract,
            report,
            time,
            self.callback.mode,
            inventories,
            integrals,
            transfers,
        )
    }
    fn transition(
        &mut self,
        report: &Report,
        event: Option<usize>,
        time: f64,
        state: &[f64],
    ) -> Result<(), ProblemError> {
        let mut transfers = vec![0.0; self.callback.contract.balances.len()];
        if let Some(index) = event {
            let id = self.callback.contract.events[self.callback.mode][index].id;
            if self
                .callback
                .contract
                .balances
                .iter()
                .any(|b| b.transfers.contains(&id))
            {
                transfers = self
                    .callback
                    .evaluate(Function::Transfer(index), time, state, false)
                    .map(|v| v.values)
                    .ok_or_else(|| self.callback.failed("event transfer"))?;
                for (value, balance) in transfers.iter_mut().zip(&self.callback.contract.balances) {
                    if !balance.transfers.contains(&id) && *value != 0.0 {
                        return Err(contract(
                            "event transfer supplied for an unauthorized conserved subject",
                        ));
                    }
                }
            }
        }
        self.transition = Some(Transition {
            record: report.events.len() - 1,
            inventories: self.inventories(time, state)?,
            transfers,
        });
        Ok(())
    }
    /// One forward `IDASolve` call, or `IDASolveF` on the adjoint route, which stores
    /// checkpoints: more than `max_checkpoints` at once is a typed memory limit.
    fn solve(&mut self, target: f64, time: &mut f64, phase: &str) -> Result<i32, ProblemError> {
        let Some(b) = self.adjoint.as_mut() else {
            // SAFETY: the session's live IDAS memory and vectors on its owning thread;
            // `time` is caller storage, and the callbacks reach the context only through
            // their user data.
            let flag = crate::observation::scoped("idas.forward", || unsafe {
                ffi::IDASolve(self.mem, target, time, self.y, self.dy, ffi::IDA_NORMAL)
            });
            if let Some(local_error) = &mut self.local_error {
                // SAFETY: actual owner-thread return, with live memory and selected
                // buffers. This adapter never enables IDASetSuppressAlg (default false).
                unsafe {
                    local_error.returned(
                        self.mem,
                        &self.callback.contract,
                        local_error::ReturnCheckpoint {
                            generation: self.observation_generation,
                            flag,
                            returned_time: *time,
                            suppress_alg: false,
                            phase,
                        },
                        &self.callback.callback.execution,
                    );
                }
            }
            return Ok(flag);
        };
        let mut stored = 0;
        // SAFETY: as above; `stored` is a local out-parameter.
        let flag = crate::observation::scoped("idas.forward_checkpointed", || unsafe {
            ffi::IDASolveF(
                self.mem,
                target,
                time,
                self.y,
                self.dy,
                ffi::IDA_NORMAL,
                &raw mut stored,
            )
        });
        if let Some(local_error) = &mut self.local_error {
            // SAFETY: IDASolveF has returned on the owning thread; the session retains
            // all state and scratch. Checkpoint accounting below cannot create validity.
            unsafe {
                local_error.returned(
                    self.mem,
                    &self.callback.contract,
                    local_error::ReturnCheckpoint {
                        generation: self.observation_generation,
                        flag,
                        returned_time: *time,
                        suppress_alg: false,
                        phase,
                    },
                    &self.callback.callback.execution,
                );
            }
        }
        // Native allocation-point enforcement counts the initial checkpoint too,
        // and records the high-water mark even when a NORMAL call fails partway.
        let (mut current, mut peak, mut refused) = (0, 0, 0);
        native!(
            ffi::IDAGetCheckpointUsage(self.mem, &raw mut current, &raw mut peak, &raw mut refused),
            "checkpoint population observation"
        )?;
        let held = usize::try_from(current)
            .map_err(|_| ProblemError::internal("negative IDAS checkpoint population"))?;
        let peak = usize::try_from(peak)
            .map_err(|_| ProblemError::internal("negative IDAS checkpoint peak"))?;
        b.checkpoints = b.checkpoints.max(peak);
        crate::observation::integer("idas.checkpoints.current", "checkpoints", current);
        crate::observation::integer(
            "idas.checkpoints.peak",
            "checkpoints",
            peak.try_into().unwrap_or(i64::MAX),
        );
        if held > b.limit || peak > b.limit {
            return Err(ProblemError::internal(
                "IDAS checkpoint owner exceeded admitted population",
            ));
        }
        if refused != 0 {
            return Err(ProblemError::memory(
                "the adjoint forward pass needs more than max_checkpoints checkpoints",
            ));
        }
        Ok(flag)
    }
    /// Charge actual replay acceptance, including a later failed native return.
    fn charge_replay(&mut self, accepted: c_long) -> Result<(), ProblemError> {
        let budget = self
            .replay_steps
            .as_mut()
            .ok_or_else(|| ProblemError::internal("replay owner"))?;
        let accepted = usize::try_from(accepted)
            .map_err(|_| ProblemError::internal("negative replay steps"))?;
        budget.charge(accepted, "adjoint forward replay")?;
        let execution = &self.callback.callback.execution;
        execution.progress.push(crate::solve::Event {
            source: Some(crate::solve::EventSource::new(
                "dynamic_replay_effort",
                Some("physical_model"),
                None,
            )),
            phase: "idas.adjoint.replay.acceptance".into(),
            elapsed: execution.started.elapsed(),
            values: std::collections::BTreeMap::from([
                (
                    "accepted_steps_this_call".into(),
                    crate::solve::Metric::Integer(i64::try_from(accepted).unwrap_or(i64::MAX)),
                ),
                (
                    "accepted_steps_cumulative".into(),
                    crate::solve::Metric::Integer(i64::try_from(budget.used()).unwrap_or(i64::MAX)),
                ),
                (
                    "accepted_steps_remaining".into(),
                    crate::solve::Metric::Integer(
                        i64::try_from(budget.remaining()).unwrap_or(i64::MAX),
                    ),
                ),
            ]),
            incumbent: None,
        });
        Ok(())
    }
    fn integrator_statistics(&self) -> Option<IntegratorStatistics> {
        let mut value = IntegratorStatistics::default();
        // SAFETY: this owner-thread session holds live IDAS memory. Every output
        // is local storage of the exact C ABI type; the getter retains no pointer.
        let flag = unsafe {
            ffi::IDAGetIntegratorStats(
                self.mem,
                &raw mut value.steps,
                &raw mut value.residuals,
                &raw mut value.linear_setups,
                &raw mut value.error_test_failures,
                &raw mut value.last_order,
                &raw mut value.current_order,
                &raw mut value.initial_step,
                &raw mut value.last_step,
                &raw mut value.current_step,
                &raw mut value.internal_time,
            )
        };
        (flag == ffi::IDA_SUCCESS).then_some(value)
    }
    /// Native counters of the current segment; IDAS resets them at each reinitialization.
    fn statistics(&self, flag: i32, p: &Profile, returned_time: Option<f64>) -> serde_json::Value {
        let mut value = serde_json::json!({
            "backend": "idas",
            "native_version": crate::sundials_version(),
            "native_flag": flag,
            "derivatives": "analytic state and parameter partials",
            "idas": serde_json::to_value(&p.idas).unwrap_or(serde_json::Value::Null),
        });
        macro_rules! observed_count {
            ($value:expr, $name:literal) => {
                if let Some(v) = $value {
                    value[$name] = serde_json::json!(long_counter(v));
                    if let (Some(context), Ok(count)) =
                        (pse_diagnostics::observation::current(), u64::try_from(v))
                    {
                        let name = concat!("idas.work.", $name);
                        if self.observation_counters.borrow_mut().insert($name) {
                            context.count_value(name, "count", 0, self.observation_generation);
                        }
                        context.count_value(name, "count", count, self.observation_generation);
                    }
                }
            };
        }
        macro_rules! count {
            ($($get:ident => $name:literal),* $(,)?) => {$(
                let mut v: c_long = 0;
                // SAFETY: a counter query of this session's live memory into a local.
                let available = unsafe { ffi::$get(self.mem, &raw mut v) } == ffi::IDA_SUCCESS;
                observed_count!(available.then_some(v), $name);
            )*};
        }
        let stats = self.integrator_statistics();
        observed_count!(stats.map(|s| s.steps), "steps");
        observed_count!(stats.map(|s| s.residuals), "residual_evaluations");
        count!(IDAGetNumNonlinSolvIters => "nonlinear_iterations",
            IDAGetNumNonlinSolvConvFails => "nonlinear_convergence_failures");
        observed_count!(stats.map(|s| s.error_test_failures), "error_test_failures");
        count!(IDAGetNumBacktrackOps => "initialization_backtracks",
            IDAGetNumGEvals => "root_evaluations",
            IDAGetNumLinIters => "linear_iterations",
            IDAGetNumPrecEvals => "preconditioner_evaluations",
            IDAGetNumJtimesEvals => "jacobian_products");
        observed_count!(stats.map(|s| s.linear_setups), "linear_setups");
        count!(IDAGetNumJacEvals => "jacobian_evaluations",
            IDAGetNumPrecSolves => "preconditioner_solves",
            IDAGetNumLinConvFails => "linear_convergence_failures",
            IDAGetNumLinResEvals => "linear_residual_evaluations");
        if self.forward && !self.sens.is_empty() {
            count!(
                IDAGetSensNumResEvals => "sensitivity_residual_evaluations",
                IDAGetSensNumErrTestFails => "sensitivity_error_test_failures",
                IDAGetSensNumLinSolvSetups => "sensitivity_linear_setups",
                IDAGetSensNumNonlinSolvIters => "sensitivity_nonlinear_iterations",
                IDAGetSensNumNonlinSolvConvFails => "sensitivity_nonlinear_failures",
            );
        }
        if !self.quad_sens.is_empty() {
            count!(IDAGetQuadSensNumRhsEvals => "quadrature_sensitivity_rhs_evaluations", IDAGetQuadSensNumErrTestFails => "quadrature_sensitivity_error_test_failures");
        }
        if !self.callback.contract.quadratures.is_empty() {
            count!(IDAGetQuadNumRhsEvals => "quadrature_rhs_evaluations", IDAGetQuadNumErrTestFails => "quadrature_error_test_failures");
        }
        // IDAGetLastStep may expose IDACalcIC's artificial step before integration.
        // Only a positive actual step count establishes last accepted-step meaning.
        let advanced = stats.is_some_and(|s| s.steps > 0);
        let initial_step = stats
            .filter(|_| advanced)
            .map(|s| s.initial_step)
            .filter(|step| step.is_finite() && *step != 0.);
        value["actual_initial_step"] =
            initial_step.map_or(serde_json::Value::Null, serde_json::Value::from);
        if let Some(step) = initial_step {
            crate::observation::gauge("idas.initial_step", "physical_time", step);
        } else {
            crate::observation::unavailable(
                "idas.initial_step",
                "no accepted integration step in the current native segment",
            );
        }
        let observation = crate::solve::NativeStepObservation::Idas {
            last_order: stats
                .filter(|_| advanced)
                .and_then(|s| u32::try_from(s.last_order).ok())
                .filter(|order| *order > 0),
            current_order: stats
                .and_then(|s| u32::try_from(s.current_order).ok())
                .filter(|order| *order > 0),
            last_step: stats.filter(|_| advanced).map(|s| s.last_step),
            current_step: stats.map(|s| s.current_step).filter(|step| *step != 0.),
            internal_time: stats.map(|s| s.internal_time),
            returned_time,
        };
        // The derived observation has only scalar fields and lossless number encoding.
        if let Ok(observation) = serde_json::to_value(observation) {
            value["native_step"] = observation;
        }
        let execution = &self.callback.callback.execution;
        crate::observation::structured(
            &execution.progress,
            execution.started,
            None,
            "dynamic_segment_summary",
            "idas.segment",
            &value,
        );
        value
    }
    /// Integrate one segment toward `stop`, sampling every requested time before it.
    /// Returns the found root index, or `None` at `stop`; step and native limits end the
    /// report without an error.
    fn segment(
        &mut self,
        p: &Profile,
        r: &mut Report,
        stop: f64,
        changing: bool,
        budget: c_long,
        output: &mut SampleOutput<'_>,
    ) -> Result<(Option<usize>, c_long), ProblemError> {
        let mut steps: c_long = 0;
        // A physical segment is the sole hard stop. Native NORMAL output service
        // preserves root ordering and sensitivity/quadrature output at each sample.
        native!(ffi::IDASetStopTime(self.mem, stop), "physical stop time")?;
        native!(ffi::IDASetMaxNumSteps(self.mem, -1), "owner-governed steps")?;
        native!(
            ffi::IDASetAcceptedStepBudget(self.mem, budget),
            "trajectory steps"
        )?;
        r.statistics.push(self.statistics(0, p, None));
        loop {
            self.callback.callback.execution.check()?;
            let target = p
                .samples
                .get(r.coverage.produced())
                .copied()
                .filter(|t| *t < stop)
                .unwrap_or(stop);
            let mut time = r.completed_time;
            let result = self.solve(target, &mut time, "idas.forward.local_error");
            native!(ffi::IDAGetNumSteps(self.mem, &raw mut steps), "step count")?;
            if let Some(last) = r.statistics.last_mut() {
                let flag = result.as_ref().copied().unwrap_or(ffi::IDA_MEM_FAIL);
                *last = self.statistics(flag, p, (flag >= 0).then_some(time));
            }
            let flag = result?;
            if flag == ffi::IDA_TOO_MUCH_WORK {
                // This native return exposes its last accepted physical state,
                // separately from the retained requested-sample prefix.
                r.completed_time = time;
                r.termination = Termination::StepLimit;
                return Ok((None, steps));
            }
            if flag < 0 {
                return Err(self.callback.native_failure(flag, "integration step"));
            }
            r.completed_time = time;
            let execution = &self.callback.callback.execution;
            execution.progress.push(crate::solve::Event {
                source: Some(crate::solve::EventSource::new(
                    "dynamic_output",
                    Some("physical_model"),
                    None,
                )),
                phase: "idas.output".into(),
                elapsed: execution.started.elapsed(),
                values: std::collections::BTreeMap::from([
                    ("time".into(), crate::solve::Metric::Real(time)),
                    (
                        "steps".into(),
                        crate::solve::Metric::Integer(long_counter(steps)),
                    ),
                ]),
                incumbent: None,
            });
            if flag == ffi::IDA_ROOT_RETURN {
                let events = self.callback.contract.events[self.callback.mode].len();
                // One entry per root function of the active mode.
                let mut found = vec![0; events];
                native!(
                    ffi::IDAGetRootInfo(self.mem, found.as_mut_ptr()),
                    "root information",
                )?;
                let index = found
                    .iter()
                    .position(|v| *v != 0)
                    .ok_or_else(|| ProblemError::internal("IDAS root without a root index"))?;
                return Ok((Some(index), steps));
            }
            // A sample at a scheduled change observes the post-change state.
            let deferred = target >= stop && changing;
            if !deferred && p.samples.get(r.coverage.produced()) == Some(&time) {
                let point = self.sample(time, true)?;
                let inventories = self.inventories(time, &point.state)?;
                let transfers = cumulative_transfers(&self.callback.contract, r);
                observe_conservation(
                    &self.callback.contract,
                    r,
                    time,
                    point.mode,
                    inventories,
                    point.integrals.clone(),
                    transfers,
                )?;
                output.push(r, p, point)?;
            }
            if target >= stop {
                return Ok((None, steps));
            }
        }
    }
    /// The segment loop: roots, resets and scheduled changes restart the same native
    /// memory; every transition is recorded with its settled post-transition state.
    fn run(
        &mut self,
        p: &Profile,
        r: &mut Report,
        output: &mut SampleOutput<'_>,
    ) -> Result<(), ProblemError> {
        let n = self.callback.contract.states.len();
        let max_steps: c_long = p
            .max_steps
            .try_into()
            .map_err(|_| ProblemError::unsupported("IDAS step allowance extent"))?;
        let mut used: c_long = 0;
        let mut time = p.start;
        // Scheduled-input changes bound the segments; each restart applies the next
        // intervals' columns to the same integration parameters (I6).
        let boundaries = p.boundaries();
        let np = self.callback.contract.parameters.len();
        let mut segment = 0;
        loop {
            // SAFETY: the session's state vector holds the `n` states.
            let state = unsafe { read(self.y, n) };
            let inventories = self.inventories(time, &state)?;
            let integrals = self.integrals(time, false)?;
            if let Some(transition) = self.transition.take() {
                settle_transition(
                    &self.callback.contract,
                    r,
                    transition,
                    ConservationPoint {
                        time,
                        mode: self.callback.mode,
                        inventories,
                        integrals,
                        transfers: vec![],
                        defects: vec![],
                    },
                    &state,
                )?;
            } else {
                let transfers = cumulative_transfers(&self.callback.contract, r);
                observe_conservation(
                    &self.callback.contract,
                    r,
                    time,
                    self.callback.mode,
                    inventories,
                    integrals,
                    transfers,
                )?;
            }
            // A coincident event reset settles first under the terminating segment.
            // Only then does the scheduled segment become active and initialize consistently.
            if boundaries.get(segment) == Some(&time) {
                if r.events.len() >= p.max_events {
                    r.termination = Termination::EventLimit;
                    return Ok(());
                }
                r.events.push(EventRecord {
                    event: None,
                    time,
                    before: state.clone(),
                    after: None,
                });
                self.transition(r, None, time, &state)?;
                self.callback.map = p.columns_at(np, time);
                self.callback.parameters = p.parameters_at(&self.callback.integration, time);
                segment += 1;
                self.restart(time, &state, p, false)?;
                continue;
            }
            if !self.callback.contract.events[self.callback.mode].is_empty() {
                let Some(guards) = self.callback.evaluate(Function::Roots, time, &state, false)
                else {
                    return Err(self.callback.failed("root function"));
                };
                if guards_at_zero(
                    &guards.values,
                    &self.callback.contract.events[self.callback.mode],
                ) > 0
                {
                    return Err(contract("initial or post-reset root is ambiguous"));
                }
            }
            // A sample at a transition time observes the restarted state.
            if p.samples.get(r.coverage.produced()) == Some(&time) {
                output.push(r, p, self.sample(time, false)?)?;
            }
            r.completed_time = time;
            if time >= p.end {
                self.capture_endpoint(r, time, false, None)?;
                r.termination = Termination::Completed;
                return Ok(());
            }
            if self.adjoint.is_some() {
                let (sensitivities, rates) = if self.forward {
                    (
                        // SAFETY: each sensitivity vector of the session holds the `n`
                        // states.
                        self.sens.iter().map(|v| unsafe { read(*v, n) }).collect(),
                        // SAFETY: as above, for their rates.
                        self.dsens.iter().map(|v| unsafe { read(*v, n) }).collect(),
                    )
                } else {
                    (Vec::new(), Vec::new())
                };
                // SAFETY: the session's rate vector holds the `n` states.
                let dy = unsafe { read(self.dy, n) };
                self.backward_mut()?.starts.push(Start {
                    time,
                    y: state,
                    dy,
                    sensitivities,
                    rates,
                });
            }
            let stop = boundaries.get(segment).copied().unwrap_or(p.end);
            let changing = segment < boundaries.len();
            let (root, steps) = self.segment(p, r, stop, changing, max_steps - used, output)?;
            used += steps;
            if r.termination == Termination::StepLimit {
                return Ok(());
            }
            time = r.completed_time;
            // SAFETY: the session's state vector holds the `n` states.
            let state = unsafe { read(self.y, n) };
            let mut seed = state.clone();
            self.conserve(r, time, &state, true)?;
            if let Some(index) = root {
                let mode = self.callback.mode;
                let Some(guards) = self.callback.evaluate(Function::Roots, time, &state, false)
                else {
                    return Err(self.callback.failed("root function"));
                };
                if guards_at_zero(&guards.values, &self.callback.contract.events[mode]) > 1 {
                    return Err(contract("ambiguous simultaneous dynamic events"));
                }
                let event = self.callback.contract.events[mode][index].clone();
                if r.events.len() >= p.max_events {
                    r.termination = Termination::EventLimit;
                    return Ok(());
                }
                r.events.push(EventRecord {
                    event: Some(event.id),
                    time,
                    before: state.clone(),
                    after: None,
                });
                if event.terminal {
                    self.capture_endpoint(r, time, true, Some(event.id))?;
                    if p.samples.get(r.coverage.produced()) == Some(&time) {
                        let point = self.sample(time, true)?;
                        let inventories = self.inventories(time, &point.state)?;
                        let transfers = cumulative_transfers(&self.callback.contract, r);
                        observe_conservation(
                            &self.callback.contract,
                            r,
                            time,
                            point.mode,
                            inventories,
                            point.integrals.clone(),
                            transfers,
                        )?;
                        output.push(r, p, point)?;
                    }
                    r.termination = Termination::Event;
                    return Ok(());
                }
                self.transition(r, Some(index), time, &state)?;
                let Some(reset) =
                    self.callback
                        .evaluate(Function::Reset(index), time, &state, false)
                else {
                    return Err(self.callback.failed("event reset"));
                };
                seed = reset.values;
                self.callback.mode = event.next_mode;
            }
            let changed = boundaries.get(segment) == Some(&time);
            if root.is_none() && changed {
                // The next loop captures the old-segment inventory before restart.
                continue;
            }
            if time >= p.end && root.is_none() && !changed {
                self.capture_endpoint(r, time, true, None)?;
                r.termination = Termination::Completed;
                return Ok(());
            }
            self.restart(time, &seed, p, root.is_some())?;
        }
    }
}
impl Session<'_> {
    fn backward_mut(&mut self) -> Result<&mut Backward, ProblemError> {
        self.adjoint
            .as_mut()
            .ok_or_else(|| ProblemError::internal("IDAS adjoint not initialized"))
    }
    /// The backward pass over every segment from the last (ADR-0110 items 3 and 4). Each
    /// segment before the last reruns its forward pass from its stored start with
    /// `IDAReInit` + `IDAAdjReInit` + `IDASolveF`, because reinitialization frees the
    /// checkpoints; the backward problems restart with `IDAReInitB` at the segment's end
    /// and at every observed sample, after the jump of `eliminate` (and its tangent on
    /// the second-order route), with `IDACalcICB`/`IDACalcICBS` recomputing the algebraic
    /// adjoint. The quadratures accumulate the gradient (and the Hessian columns); the
    /// first segment adds its initial values' partials, and the differential adjoint
    /// carries across scheduled changes. Returns the gradient, and on the second-order
    /// route the symmetric Hessian over the directions with its largest relative
    /// asymmetry before symmetrization.
    fn backward(
        &mut self,
        p: &Profile,
        r: &Report,
        weights: &[f64],
    ) -> Result<(Vec<f64>, Option<SecondOrder>), ProblemError> {
        self.callback.callback.execution.check()?;
        self.replay_steps = Some(StepBudget::new(p.max_steps));
        if weights.iter().all(|weight| *weight == 0.) {
            crate::observation::boolean("idas.adjoint.zero_cotangent", true);
            let directions = self.backward_mut()?.directions.len();
            let hessian = (directions > 0).then(|| (faer::Mat::zeros(directions, directions), 0.));
            return Ok((vec![0.; self.callback.gradient_axes.len()], hessian));
        }
        let n = self.callback.contract.states.len();
        let np = self.callback.contract.parameters.len();
        let width = self.callback.gradient_axes.len();
        let boundaries = p.boundaries();
        let b = self.backward_mut()?;
        let starts = std::mem::take(&mut b.starts);
        let directions = b.directions.clone();
        let (count, size, quadratures) = if b.second_order() {
            (directions.len(), 2 * n, 2 * width)
        } else {
            (1, n, width)
        };
        let mut carried: Option<Vec<Vec<f64>>> = None;
        let mut totals = vec![vec![0.0; quadratures]; count];
        let mut remaining_samples = r.samples.len();
        for (k, start) in starts.iter().enumerate().rev() {
            let stop = boundaries.get(k).copied().unwrap_or(p.end);
            self.callback.map = p.columns_at(np, start.time);
            self.callback.parameters = p.parameters_at(&self.callback.integration, start.time);
            let last = k + 1 == starts.len();
            if !last {
                self.replay(start, stop, p)?;
            }
            // The segment's samples, latest first; a sample at a change time observes the
            // later segment.
            let end = remaining_samples;
            while remaining_samples > 0 && r.samples[remaining_samples - 1].time >= start.time {
                remaining_samples -= 1;
            }
            let outputs = self.callback.contract.outputs.len();
            let mut observed = (remaining_samples..end)
                .rev()
                .filter(|i| {
                    weights[*i * outputs..(*i + 1) * outputs]
                        .iter()
                        .any(|w| *w != 0.)
                })
                .peekable();
            if end > remaining_samples && r.samples[end - 1].time > stop {
                return Err(ProblemError::internal("adjoint sample outside its segment"));
            }
            let mut states = carried
                .take()
                .unwrap_or_else(|| vec![vec![0.0; size]; count]);
            let mut sums = vec![vec![0.0; quadratures]; count];
            if let Some(i) = observed.next_if(|i| r.samples[*i].time >= stop) {
                self.jump(r, i, weights, &mut states, &mut sums)?;
            }
            // The forward pass ended at the segment's stop.
            let forward = self.forward_at_stop(stop)?;
            self.restart_backward(stop, start.time, (&states, &sums), &forward, p)?;
            let mut first = None;
            for i in observed {
                let t = r.samples[i].time;
                if t <= start.time {
                    first = Some(i);
                    break;
                }
                (states, sums) = self.solve_backward(t)?;
                self.jump(r, i, weights, &mut states, &mut sums)?;
                let forward = self.forward_at(t, &r.samples[i])?;
                self.restart_backward(t, start.time, (&states, &sums), &forward, p)?;
            }
            (states, sums) = self.solve_backward(start.time)?;
            if let Some(i) = first {
                self.jump(r, i, weights, &mut states, &mut sums)?;
            }
            if k == 0 {
                self.initial_values(start.time, &states, &mut sums)?;
            }
            for (total, sum) in totals.iter_mut().zip(&sums) {
                for (t, v) in total.iter_mut().zip(sum) {
                    *t += v;
                }
            }
            carried = Some(states);
            let execution = &self.callback.callback.execution;
            execution.progress.push(crate::solve::Event {
                source: Some(crate::solve::EventSource::new(
                    "dynamic_adjoint",
                    Some("physical_model"),
                    None,
                )),
                phase: "idas.adjoint".into(),
                elapsed: execution.started.elapsed(),
                values: std::collections::BTreeMap::from([
                    ("time".into(), crate::solve::Metric::Real(start.time)),
                    ("segment".into(), crate::solve::Metric::Integer(k as i64)),
                ]),
                incumbent: None,
            });
        }
        self.collect_counters()?;
        if totals.iter().flatten().any(|v| !v.is_finite()) {
            return Err(ProblemError::numerical(
                "nonfinite adjoint gradient or Hessian",
            ));
        }
        let gradient = totals[0][..width].to_vec();
        if directions.is_empty() {
            return Ok((gradient, None));
        }
        // Column `b` of the Hessian is problem `b`'s tangent quadrature; the square block
        // over the directions is symmetric up to integration error.
        let d = directions.len();
        let entry = |row: usize, column: usize| totals[column][width + directions[row]];
        let scale = (0..d)
            .flat_map(|a| (0..d).map(move |b| (a, b)))
            .map(|(a, b)| entry(a, b).abs())
            .fold(0.0_f64, f64::max);
        let asymmetry = (0..d)
            .flat_map(|a| (0..d).map(move |b| (a, b)))
            .map(|(a, b)| (entry(a, b) - entry(b, a)).abs())
            .fold(0.0_f64, f64::max)
            / (1.0 + scale);
        let hessian = faer::Mat::from_fn(d, d, |a, b| 0.5 * (entry(a, b) + entry(b, a)));
        Ok((gradient, Some((hessian, asymmetry))))
    }
    /// Rerun one segment's forward pass from its stored consistent start, with its state
    /// sensitivities when they are integrated, storing its checkpoints again.
    fn replay(&mut self, start: &Start, stop: f64, p: &Profile) -> Result<(), ProblemError> {
        let budget = self
            .replay_steps
            .get_or_insert_with(|| StepBudget::new(p.max_steps));
        budget.require_advance("adjoint forward replay")?;
        let steps: c_long = budget
            .remaining()
            .try_into()
            .map_err(|_| ProblemError::unsupported("IDAS replay allowance extent"))?;
        // SAFETY: the session's state vector holds the `n` states the start was read from.
        unsafe { write(self.y, &start.y) };
        // SAFETY: as above, for the rates.
        unsafe { write(self.dy, &start.dy) };
        native!(
            ffi::IDAReInit(self.mem, start.time, self.y, self.dy),
            "adjoint forward reinitialization",
        )?;
        bbd::install(self, p.idas.linear)?;
        if self.forward {
            for (v, values) in self.sens.iter().zip(&start.sensitivities) {
                // SAFETY: as above, for each state sensitivity.
                unsafe { write(*v, values) };
            }
            for (v, values) in self.dsens.iter().zip(&start.rates) {
                // SAFETY: as above, for each sensitivity rate.
                unsafe { write(*v, values) };
            }
            native!(
                ffi::IDASensReInit(
                    self.mem,
                    corrector(p.idas.sensitivity),
                    self.sens.as_mut_ptr(),
                    self.dsens.as_mut_ptr(),
                ),
                "adjoint forward sensitivity reinitialization",
            )?;
        }
        native!(ffi::IDAAdjReInit(self.mem), "adjoint reinitialization")?;
        native!(ffi::IDASetStopTime(self.mem, stop), "stop time")?;
        native!(
            ffi::IDASetAcceptedStepBudget(self.mem, steps),
            "adjoint forward replay steps",
        )?;
        let mut time = start.time;
        self.observation_generation = crate::observation::generation();
        self.observation_counters.borrow_mut().clear();
        let result = self.solve(stop, &mut time, "idas.replay.local_error");
        let mut accepted: c_long = 0;
        native!(
            ffi::IDAGetAcceptedBudgetSteps(self.mem, &raw mut accepted),
            "replay accepted steps"
        )?;
        self.charge_replay(accepted)?;
        let flag = result?;
        self.statistics(flag, p, (flag >= 0).then_some(time));
        if flag == ffi::IDA_TOO_MUCH_WORK {
            return Err(StepBudget::exhausted("adjoint forward replay"));
        }
        if flag < 0 {
            return Err(self.callback.native_failure(flag, "adjoint forward pass"));
        }
        Ok(())
    }
    /// The forward state, rates and state sensitivities where the forward pass stopped.
    fn forward_at_stop(&mut self, time: f64) -> Result<Forward, ProblemError> {
        let n = self.callback.contract.states.len();
        let sensitivities = if self.adjoint.as_ref().is_some_and(Backward::second_order) {
            native!(
                ffi::IDAGetSensDky(self.mem, time, 0, self.sens.as_mut_ptr()),
                "adjoint forward sensitivities",
            )?;
            // SAFETY: each sensitivity vector of the session holds the `n` states.
            self.sens.iter().map(|v| unsafe { read(*v, n) }).collect()
        } else {
            Vec::new()
        };
        Ok((
            // SAFETY: the session's state and rate vectors hold the `n` states.
            unsafe { read(self.y, n) },
            // SAFETY: as above.
            unsafe { read(self.dy, n) },
            sensitivities,
        ))
    }
    /// The forward state and rates at sample time `t`, interpolated from the checkpoints,
    /// and the sample's own state sensitivities on the second-order route.
    fn forward_at(&mut self, t: f64, sample: &Sample) -> Result<Forward, ProblemError> {
        let n = self.callback.contract.states.len();
        let width = self.callback.integration.len();
        let b = self.backward_mut()?;
        let (forward, rate) = (b.forward, b.forward_rate);
        let sensitivities = if b.second_order() {
            if sample.state_sensitivities.len() != n * width {
                return Err(ProblemError::internal("adjoint sample sensitivity extent"));
            }
            (0..width)
                .map(|k| {
                    (0..n)
                        .map(|i| sample.state_sensitivities[i * width + k])
                        .collect()
                })
                .collect()
        } else {
            Vec::new()
        };
        native!(
            ffi::IDAGetAdjY(self.mem, t, forward, rate),
            "adjoint forward interpolation",
        )?;
        // SAFETY: the adjoint's forward state and rate vectors hold the `n` states.
        let forward = unsafe { read(forward, n) };
        // SAFETY: as above.
        let rate = unsafe { read(rate, n) };
        Ok((forward, rate, sensitivities))
    }
    /// Current exact first-order ODE rates after the caller's actual adjoint jump.
    /// The ordinary callback guard validates current providers, support and domain.
    fn ode_backward_rates(
        &mut self,
        time: f64,
        forward: &[f64],
        states: &[Vec<f64>],
    ) -> Result<Option<Vec<Vec<f64>>>, ProblemError> {
        #[cfg(test)]
        if self.force_backward_ic {
            return Ok(None);
        }
        if !self
            .callback
            .contract
            .differential
            .iter()
            .all(|state| *state)
            || self.backward_mut()?.second_order()
        {
            return Ok(None);
        }
        let n = self.callback.contract.states.len();
        if forward.len() != n || states.len() != 1 || states[0].len() != n {
            return Err(ProblemError::internal(
                "ODE adjoint rate initialization extent",
            ));
        }
        self.callback.callback.execution.check()?;
        let jacobian = self
            .callback
            .partials(time, forward)
            .ok_or_else(|| self.callback.failed("ODE adjoint rate initialization"))?;
        let symbolic = jacobian.symbolic();
        let mut rate = Vec::with_capacity(n);
        for column in 0..n {
            let value = -symbolic
                .col_range(column)
                .map(|entry| jacobian.val()[entry] * states[0][symbolic.row_idx()[entry]])
                .sum::<f64>();
            if !value.is_finite() {
                return Err(ProblemError::numerical(
                    "nonfinite ODE adjoint initial rate",
                ));
            }
            rate.push(value);
        }
        self.callback.callback.execution.check()?;
        Ok(Some(vec![rate]))
    }
    /// Start every backward problem at `t` with its adjoint values and quadratures:
    /// `IDAInitB`/`IDAInitBS` and their settings once, `IDAReInitB` afterwards, then
    /// `IDACalcICB`/`IDACalcICBS` for the algebraic adjoint and every rate.
    fn restart_backward(
        &mut self,
        t: f64,
        start: f64,
        (states, sums): (&[Vec<f64>], &[Vec<f64>]),
        forward: &Forward,
        p: &Profile,
    ) -> Result<(), ProblemError> {
        let mem = self.mem;
        if self.backward_mut()?.problems.is_empty() {
            self.create_backward(t, p)?;
        }
        // The all-differential first-order residual has identity mass. Its
        // current transpose action determines consistent rates exactly; DAE
        // and second-order systems retain native consistent initialization.
        let exact_rates = self.ode_backward_rates(t, &forward.0, states)?;
        let b = self.backward_mut()?;
        let (fy, fyp) = (b.forward, b.forward_rate);
        // SAFETY: the adjoint's forward state vector holds the `n` states `forward` was
        // read with.
        unsafe { write(fy, &forward.0) };
        // SAFETY: as above, for the rates.
        unsafe { write(fyp, &forward.1) };
        for (v, values) in b.forward_sensitivities.iter().zip(&forward.2) {
            // SAFETY: as above, for each state sensitivity.
            unsafe { write(*v, values) };
        }
        for (index, problem) in b.problems.iter_mut().enumerate() {
            let values = states
                .get(index)
                .zip(sums.get(index))
                .ok_or_else(|| ProblemError::internal("IDAS backward problem extent"))?;
            // A problem's counters restart with it; keep the finished interval's.
            // SAFETY: `mem` is the session's live IDAS memory, which owns `problem`.
            unsafe { accumulate(mem, problem)? };
            // SAFETY: the problem's vectors hold its adjoint values and quadratures, the
            // extents the backward pass keeps `values` at.
            unsafe { write(problem.y, values.0) };
            if let Some(rates) = &exact_rates {
                // SAFETY: ode_backward_rates supplies one current n-state rate vector per
                // problem, matching this live problem.yp and the adjoint state extent.
                unsafe { write(problem.yp, &rates[index]) };
            } else {
                // SAFETY: this problem's live rate vector has the same extent as its
                // checked adjoint states; the zero candidate is independent owned storage.
                unsafe { write(problem.yp, &vec![0.0; values.0.len()]) };
            }
            // SAFETY: as above.
            if !problem.quadrature.is_null() {
                // SAFETY: the nonempty admitted quadrature vector has the sum extent.
                unsafe { write(problem.quadrature, values.1) };
            }
            native!(
                ffi::IDAReInitB(mem, problem.which, t, problem.y, problem.yp),
                "adjoint reinitialization",
            )?;
            if !problem.quadrature.is_null() {
                native!(
                    ffi::IDAQuadReInitB(mem, problem.which, problem.quadrature),
                    "adjoint quadrature reinitialization",
                )?;
            }
        }
        if exact_rates.is_some() {
            self.callback.callback.execution.check()?;
            crate::observation::count("idas.adjoint.exact_ode_rate_initializations", 1);
            return Ok(());
        }
        // `tout1` only orients and scales the initialization step toward the start.
        let toward = t - (t - start).min(p.initial_step);
        let second = b.second_order();
        let problems = b.problems.iter().map(|q| q.which).collect::<Vec<_>>();
        let (mut sensitivities, mut rates) = (
            b.forward_sensitivities.clone(),
            b.forward_sensitivity_rates.clone(),
        );
        for which in problems {
            crate::observation::count("idas.adjoint.consistent_initializations", 1);
            let flag = if second {
                // SAFETY: the session's live IDAS memory and adjoint vectors on its owning
                // thread; the sensitivity arrays are local copies of the vector handles.
                unsafe {
                    ffi::IDACalcICBS(
                        mem,
                        which,
                        toward,
                        fy,
                        fyp,
                        sensitivities.as_mut_ptr(),
                        rates.as_mut_ptr(),
                    )
                }
            } else {
                // SAFETY: the session's live IDAS memory and adjoint vectors.
                unsafe { ffi::IDACalcICB(mem, which, toward, fy, fyp) }
            };
            if flag < 0 {
                return Err(self
                    .callback
                    .native_failure(flag, "consistent adjoint initial conditions"));
            }
        }
        Ok(())
    }
    /// Every backward problem with its KLU solver and quadrature: one first-order problem
    /// over the transposed pattern, or one 2n problem per second-order direction over the
    /// block lower-triangular pattern, whose callbacks read the forward sensitivities.
    fn create_backward(&mut self, t: f64, p: &Profile) -> Result<(), ProblemError> {
        let n = self.callback.contract.states.len();
        let width = self.callback.gradient_axes.len();
        let context = self.user_data();
        let directions = self.backward_mut()?.directions.clone();
        let second = !directions.is_empty();
        let (size, quadratures, nonzeros) = if second {
            (2 * n, 2 * width, self.callback.rows_bs.len())
        } else {
            (n, width, self.callback.layout.rows_b.len())
        };
        crate::observation::integer(
            "idas.adjoint.parameter_quadrature_width",
            "count",
            width as i64,
        );
        let opaque_fill = size
            .checked_mul(size)
            .and_then(|cells| cells.checked_mul(32))
            .and_then(|bytes| bytes.checked_mul(directions.len().max(1)))
            .ok_or_else(|| ProblemError::memory("IDAS backward direct fill allowance extent"))?;
        crate::observation::integer(
            "idas.adjoint.opaque_factor_allowance.bytes",
            "bytes",
            i64::try_from(opaque_fill).unwrap_or(i64::MAX),
        );
        let ids: Vec<_> = self
            .callback
            .contract
            .differential
            .iter()
            .cycle()
            .take(size)
            .map(|v| f64::from(*v))
            .collect();
        let id = self.vector(&ids)?;
        let atol = self.vector(
            &p.atol
                .iter()
                .copied()
                .cycle()
                .take(size)
                .collect::<Vec<_>>(),
        )?;
        let smallest = p.atol.iter().copied().fold(f64::INFINITY, f64::min);
        let steps: c_long = p
            .max_steps
            .try_into()
            .map_err(|_| ProblemError::unsupported("IDAS step allowance extent"))?;
        let targets: Vec<Option<usize>> = if second {
            directions.iter().copied().map(Some).collect()
        } else {
            vec![None]
        };
        let (mem, ctx) = (self.mem, self.ctx);
        for target in targets {
            let y = self.vector(&vec![0.0; size])?;
            let yp = self.vector(&vec![0.0; size])?;
            let quadrature = if quadratures == 0 {
                std::ptr::null_mut()
            } else {
                self.vector(&vec![0.0; quadratures])?
            };
            let mut problem = Problem {
                which: 0,
                matrix: std::ptr::null_mut(),
                linear: std::ptr::null_mut(),
                y,
                yp,
                quadrature,
                quadrature_width: quadratures,
                direction: target.map(|column| Box::new(Direction { context, column })),
                counters: [0; 4],
                steps: StepBudget::new(p.max_steps),
            };
            let data: *mut c_void = problem
                .direction
                .as_mut()
                .map_or(context, |d| (&raw mut **d).cast());
            // The problem joins the session before any fallible native call, so `Drop`
            // frees whatever it allocated.
            let b = self.backward_mut()?;
            b.problems.push(problem);
            let problem = b
                .problems
                .last_mut()
                .ok_or_else(|| ProblemError::internal("IDAS backward problem"))?;
            native!(
                ffi::IDACreateB(mem, &raw mut problem.which),
                "backward problem",
            )?;
            let which = problem.which;
            if second {
                native!(
                    ffi::IDAInitBS(mem, which, Some(residual_bs), t, problem.y, problem.yp),
                    "second-order backward initialization",
                )?;
            } else {
                native!(
                    ffi::IDAInitB(mem, which, Some(residual_b), t, problem.y, problem.yp),
                    "backward initialization",
                )?;
            }
            // The user data is the session's boxed context, or the problem's boxed
            // direction: both keep their addresses while the session lives.
            native!(ffi::IDASetUserDataB(mem, which, data), "backward user data")?;
            native!(
                ffi::IDASetIdB(mem, which, id),
                "backward differential identities",
            )?;
            native!(
                ffi::IDASVtolerancesB(mem, which, p.rtol, atol),
                "backward tolerances",
            )?;
            native!(ffi::IDASetMaxNumStepsB(mem, which, steps), "backward steps")?;
            // SAFETY: the session's live context; `Drop` frees the matrix.
            problem.matrix = unsafe {
                ffi::SUNSparseMatrix(index(size)?, index(size)?, index(nonzeros)?, 0, ctx)
            };
            if problem.matrix.is_null() {
                return Err(ProblemError::memory("IDAS adjoint matrix allocation"));
            }
            // SAFETY: the problem's live adjoint vector and matrix and the session's
            // context; `Drop` frees the solver.
            problem.linear = unsafe { ffi::SUNLinSol_KLU(problem.y, problem.matrix, ctx) };
            if problem.linear.is_null() {
                return Err(ProblemError::memory("IDAS adjoint KLU allocation"));
            }
            native!(
                ffi::IDASetLinearSolverB(mem, which, problem.linear, problem.matrix),
                "backward KLU",
            )?;
            if second {
                // A restart at a sample takes the forward sensitivities from the sample,
                // while the steps interpolate IDAS's recomputed checkpoint data: the
                // algebraic adjoint tangent is consistent with the former and off from
                // the latter by interpolation error, which no step size removes. The
                // index-1 algebraic components follow the differential ones, so they
                // leave the local error test (`IDASetSuppressAlgB`).
                native!(
                    ffi::IDASetSuppressAlgB(mem, which, 1),
                    "second-order algebraic error test",
                )?;
                native!(
                    ffi::IDASetJacFnBS(mem, which, Some(jacobian_bs)),
                    "second-order backward Jacobian",
                )?;
                native!(
                    ffi::IDAQuadInitBS(mem, which, Some(quadrature_bs), problem.quadrature),
                    "second-order backward quadrature",
                )?;
            } else {
                native!(
                    ffi::IDASetJacFnB(mem, which, Some(jacobian_b)),
                    "backward analytic Jacobian",
                )?;
                if quadratures != 0 {
                    native!(
                        ffi::IDAQuadInitB(mem, which, Some(quadrature_b), problem.quadrature),
                        "backward quadrature",
                    )?;
                }
            }
            if quadratures != 0 {
                native!(
                    ffi::IDAQuadSStolerancesB(mem, which, p.rtol, smallest),
                    "backward quadrature tolerances",
                )?;
                native!(
                    ffi::IDASetQuadErrConB(mem, which, 1),
                    "backward quadrature error control",
                )?;
            }
        }
        Ok(())
    }
    /// Integrate every backward problem to `t`; their adjoint values and quadratures there.
    fn solve_backward(&mut self, t: f64) -> Result<Adjoints, ProblemError> {
        let mem = self.mem;
        // Restored forward checkpoints never restore permission. Hidden replay
        // uses its own cumulative owner; each backward direction owns its cap.
        native!(
            ffi::IDASetAcceptedStepBudget(mem, -1),
            "disable recording guard during replay"
        )?;
        let replay = self
            .replay_steps
            .as_ref()
            .ok_or_else(|| ProblemError::internal("replay owner"))?;
        let remaining: c_long = replay
            .remaining()
            .try_into()
            .map_err(|_| ProblemError::unsupported("IDAS replay allowance extent"))?;
        native!(
            ffi::IDASetReplayStepLimit(mem, remaining),
            "checkpoint replay steps"
        )?;
        for problem in &self.backward_mut()?.problems {
            // SAFETY: this session owns the exact backward memory returned here.
            let backward = unsafe { ffi::IDAGetAdjIDABmem(mem, problem.which) };
            if backward.is_null() {
                return Err(ProblemError::internal("IDAS backward memory"));
            }
            let remaining: c_long = problem
                .steps
                .remaining()
                .try_into()
                .map_err(|_| ProblemError::unsupported("IDAS backward allowance extent"))?;
            native!(
                ffi::IDASetAcceptedStepBudget(backward, remaining),
                "backward trajectory steps"
            )?;
            native!(
                ffi::IDASetMaxNumSteps(backward, -1),
                "owner-governed backward steps"
            )?;
        }
        // SAFETY: the session's live IDAS memory on its owning thread; the backward
        // callbacks reach the context only through their user data.
        let flag = crate::observation::scoped("idas.backward", || unsafe {
            ffi::IDASolveB(mem, t, ffi::IDA_NORMAL)
        });
        let mut replayed: c_long = 0;
        native!(
            ffi::IDAGetReplayNumSteps(mem, &raw mut replayed),
            "checkpoint replay count"
        )?;
        self.charge_replay(replayed)?;
        let mut exhausted = false;
        for problem in &mut self.backward_mut()?.problems {
            // SAFETY: this session owns the exact backward memory.
            let backward = unsafe { ffi::IDAGetAdjIDABmem(mem, problem.which) };
            let mut accepted: c_long = 0;
            native!(
                ffi::IDAGetAcceptedBudgetSteps(backward, &raw mut accepted),
                "backward accepted count"
            )?;
            problem.steps.charge(
                usize::try_from(accepted)
                    .map_err(|_| ProblemError::internal("negative backward steps"))?,
                "adjoint backward trajectory",
            )?;
            exhausted |= problem.steps.remaining() == 0;
        }
        if flag == ffi::IDA_TOO_MUCH_WORK
            && (exhausted
                || self
                    .replay_steps
                    .as_ref()
                    .is_some_and(|b| b.remaining() == 0))
        {
            return Err(StepBudget::exhausted("adjoint trajectory"));
        }
        if flag < 0 {
            return Err(self.callback.native_failure(flag, "adjoint step"));
        }
        let n = self.callback.contract.states.len();
        let width = self.callback.gradient_axes.len();
        let b = self.backward_mut()?;
        let (size, quadratures) = if b.second_order() {
            (2 * n, 2 * width)
        } else {
            (n, width)
        };
        let mut states = Vec::with_capacity(b.problems.len());
        let mut sums = Vec::with_capacity(b.problems.len());
        for problem in &b.problems {
            let mut time = t;
            native!(
                ffi::IDAGetB(mem, problem.which, &raw mut time, problem.y, problem.yp),
                "adjoint state",
            )?;
            if !problem.quadrature.is_null() {
                native!(
                    ffi::IDAGetQuadB(mem, problem.which, &raw mut time, problem.quadrature),
                    "adjoint quadrature",
                )?;
            }
            // SAFETY: the problem's adjoint vector holds `size` values (`create_backward`).
            states.push(unsafe { read(problem.y, size) });
            // SAFETY: its quadrature vector holds `quadratures` values.
            sums.push(if quadratures == 0 {
                vec![]
            } else {
                // SAFETY: the live quadrature vector has the admitted nonzero extent.
                unsafe { read(problem.quadrature, quadratures) }
            });
        }
        Ok((states, sums))
    }
    /// Add the jump at sample `i` to every backward problem: the first-order jump to the
    /// adjoint and the gradient, in the segment's integration columns, and on the
    /// second-order route its tangent along the problem's direction to `μ` and the Hessian
    /// column: with `d = [s_k; π_k]` and `ŵ` the first-order constraint multipliers,
    /// `v = ∇²(cᵀg)·d − ∇²(ŵᵀf)·d` is eliminated through the same algebraic block. The
    /// forward state and sensitivities are the sample's own.
    fn jump(
        &mut self,
        r: &Report,
        i: usize,
        weights: &[f64],
        states: &mut [Vec<f64>],
        sums: &mut [Vec<f64>],
    ) -> Result<(), ProblemError> {
        let sample = &r.samples[i];
        let n = self.callback.contract.states.len();
        let m = self.callback.contract.outputs.len();
        let width = self.callback.gradient_axes.len();
        let cotangent = &weights[i * m..(i + 1) * m];
        let Some(output) = self
            .callback
            .evaluate(Function::Output, sample.time, &sample.state, true)
            .and_then(|e| e.jacobian)
        else {
            return Err(self.callback.failed("adjoint output partials"));
        };
        let rhs = if self.callback.contract.differential.contains(&false) {
            let Some(j) = self.callback.partials(sample.time, &sample.state) else {
                return Err(self.callback.failed("adjoint constraint partials"));
            };
            Some(j)
        } else {
            None
        };
        let differential = self.callback.contract.differential.clone();
        let mut elimination = SampleElimination::prepared(
            &differential,
            rhs.as_ref().map(|j| j.as_ref().as_ref()),
            output.ncols(),
            &mut self.callback.sample_elimination,
        )?;
        let parameter_columns = self
            .callback
            .gradient_axes
            .iter()
            .map(|axis| self.callback.map.iter().position(|column| column == axis))
            .collect::<Vec<_>>();
        let first = elimination.solve_selected(
            selected_transposed_product(
                output.as_ref().as_ref(),
                cotangent,
                n,
                &parameter_columns,
            )?,
            &parameter_columns,
        )?;
        let directions = self.backward_mut()?.directions.clone();
        for (index, (state, sum)) in states.iter_mut().zip(sums.iter_mut()).enumerate() {
            for (a, d) in state.iter_mut().zip(&first.state) {
                *a += d;
            }
            for (slot, value) in sum[..width].iter_mut().zip(&first.parameters) {
                *slot += value;
            }
            let Some(&column) = directions.get(index) else {
                continue;
            };
            if sample.state_sensitivities.len() != n * width {
                return Err(ProblemError::internal("adjoint sample sensitivity extent"));
            }
            let s = (0..n)
                .map(|x| sample.state_sensitivities[x * width + column])
                .collect::<Vec<_>>();
            let d = self.callback.direction(column, &s);
            let Some(mut v) = self.callback.curvature(
                Function::Output,
                sample.time,
                &sample.state,
                cotangent,
                &d,
            ) else {
                return Err(self.callback.failed("adjoint output curvature"));
            };
            if rhs.is_some() {
                let Some(constraint) = self.callback.curvature(
                    Function::Rhs,
                    sample.time,
                    &sample.state,
                    &first.multipliers,
                    &d,
                ) else {
                    return Err(self.callback.failed("adjoint constraint curvature"));
                };
                for (total, c) in v.iter_mut().zip(constraint) {
                    *total -= c;
                }
            }
            let tangent = elimination.solve(v)?;
            for (a, d) in state[n..].iter_mut().zip(&tangent.state) {
                *a += d;
            }
            for (q, d) in tangent.parameters.iter().enumerate() {
                *sum.get_mut(width + self.callback.map[q])
                    .ok_or_else(|| ProblemError::internal("adjoint jump columns"))? += d;
            }
        }
        Ok(())
    }
    /// The first segment's initial differential values depend on the parameters:
    /// `(∂x₀/∂p)ᵀλ_d` joins the gradient, and on the second-order route
    /// `(∂x₀/∂p)ᵀμ_d + ∇²_pp(λ_dᵀx₀)·π_k` joins the Hessian column (the initial values do
    /// not depend on the state). `IDACalcIC` recomputes the algebraic ones.
    fn initial_values(
        &mut self,
        t: f64,
        states: &[Vec<f64>],
        sums: &mut [Vec<f64>],
    ) -> Result<(), ProblemError> {
        let n = self.callback.contract.states.len();
        let width = self.callback.gradient_axes.len();
        if width == 0 {
            return Ok(());
        }
        let zeros = vec![0.0; n];
        let Some(j) = self
            .callback
            .evaluate(Function::Initial, t, &zeros, true)
            .and_then(|e| e.jacobian)
        else {
            return Err(self.callback.failed("initial partials"));
        };
        let directions = self.backward_mut()?.directions.clone();
        let mask = self.callback.contract.differential.clone();
        let differential = |values: &[f64]| -> Vec<f64> {
            values
                .iter()
                .zip(&mask)
                .map(|(l, d)| if *d { *l } else { 0.0 })
                .collect()
        };
        for (index, (state, sum)) in states.iter().zip(sums.iter_mut()).enumerate() {
            let lambda = differential(&state[..n]);
            let symbolic = j.symbolic();
            for (selected, axis) in self.callback.gradient_axes.iter().enumerate() {
                if let Some(q) = self.callback.map.iter().position(|column| column == axis) {
                    sum[selected] += symbolic
                        .col_range(n + q)
                        .map(|e| j.val()[e] * lambda[symbolic.row_idx()[e]])
                        .sum::<f64>();
                }
            }
            let Some(&column) = directions.get(index) else {
                continue;
            };
            let mu = differential(&state[n..]);
            let product = transposed(&j, &mu);
            let d = self.callback.direction(column, &zeros);
            let Some(v) = self
                .callback
                .curvature(Function::Initial, t, &zeros, &lambda, &d)
            else {
                return Err(self.callback.failed("initial curvature"));
            };
            for (q, target) in self.callback.map.iter().enumerate() {
                sum[width + *target] += product[n + q] + v[n + q];
            }
        }
        Ok(())
    }
    /// Add every backward problem's native counters since its last reinitialization.
    fn collect_counters(&mut self) -> Result<(), ProblemError> {
        let mem = self.mem;
        for problem in &mut self.backward_mut()?.problems {
            // SAFETY: `mem` is the session's live IDAS memory, which owns `problem`.
            unsafe {
                accumulate(mem, problem)?;
            }
        }
        Ok(())
    }
    /// The backward problems' summed native counters, for the statistics.
    fn backward_statistics(&self) -> serde_json::Value {
        let problems = self.adjoint.as_ref().map_or_else(Vec::new, |b| {
            b.problems
                .iter()
                .map(|q| {
                    serde_json::json!({
                        "direction": q.direction.as_ref().map(|d| d.column),
                        "parameter_quadrature_width": q.quadrature_width,
                        "quadrature_allocated": !q.quadrature.is_null(),
                        "steps": q.counters[0],
                        "accepted_step_allowance": q.steps.used() + q.steps.remaining(),
                        "accepted_steps_charged": q.steps.used(),
                        "nonlinear_iterations": q.counters[1],
                        "nonlinear_convergence_failures": q.counters[2],
                        "error_test_failures": q.counters[3],
                    })
                })
                .collect()
        });
        serde_json::Value::Array(problems)
    }
}
/// Add a backward problem's native counters since its last reinitialization, which
/// resets them.
///
/// # Safety
/// `mem` is the live IDAS memory that owns `problem`.
unsafe fn accumulate(mem: *mut c_void, problem: &mut Problem) -> Result<(), ProblemError> {
    // SAFETY: `mem` is the live IDAS memory that owns `problem` (the caller's contract).
    let backward = unsafe { ffi::IDAGetAdjIDABmem(mem, problem.which) };
    if backward.is_null() {
        return Err(ProblemError::internal("IDAS backward memory"));
    }
    // The backward memory is owned by `mem`; the counters are written into `values`.
    let mut values: [c_long; 4] = [0; 4];
    native!(
        ffi::IDAGetNumSteps(backward, &raw mut values[0]),
        "backward steps",
    )?;
    native!(
        ffi::IDAGetNumNonlinSolvIters(backward, &raw mut values[1]),
        "backward nonlinear iterations",
    )?;
    native!(
        ffi::IDAGetNumNonlinSolvConvFails(backward, &raw mut values[2]),
        "backward convergence failures",
    )?;
    native!(
        ffi::IDAGetNumErrTestFails(backward, &raw mut values[3]),
        "backward error-test failures",
    )?;
    for ((name, total), value) in [
        "steps",
        "nonlinear_iterations",
        "nonlinear_convergence_failures",
        "error_test_failures",
    ]
    .into_iter()
    .zip(problem.counters.iter_mut())
    .zip(values)
    {
        *total += long_counter(value);
        crate::observation::integer(
            &format!(
                "idas.backward.{}.{}.completed_segments",
                problem.which, name
            ),
            "count",
            *total,
        );
    }
    Ok(())
}
/// The native sensitivity corrector.
fn corrector(method: SensitivityCorrector) -> i32 {
    match method {
        SensitivityCorrector::Simultaneous => ffi::IDA_SIMULTANEOUS,
        SensitivityCorrector::Staggered => ffi::IDA_STAGGERED,
    }
}

/// Execute the IDAS profile on its owning worker. The caller validated the profile.
pub(super) fn integrate_with_samples(
    oracle: &mut dyn Oracle,
    p: &Profile,
    integration: &[f64],
    cancel: Cancellation,
    progress: Arc<Progress>,
    output: &mut SampleOutput<'_>,
) -> Result<Report, ProblemError> {
    let (report, _, _) = attempt(
        NumericalRequest {
            oracle,
            profile: p,
            integration,
            directions: &[],
            forward_quadratures: false,
            admission: None,
            current_execution: None,
            workspace: None,
        },
        cancel,
        progress,
        output,
        |_, _| Ok(()),
    );
    Ok(report)
}
pub(super) fn integrate_with_storage(
    oracle: &mut dyn Oracle,
    p: &Profile,
    integration: &[f64],
    current: Execution,
    workspace: Option<&mut IdasWorkspace>,
    output: &mut SampleOutput<'_>,
) -> Result<Report, ProblemError> {
    let admission = current
        .memory
        .map(|bytes| oracle.prepared_layout(bytes).map(|layout| (layout, bytes)))
        .transpose()?;
    let (report, _, _) = attempt(
        NumericalRequest {
            oracle,
            profile: p,
            integration,
            directions: &[],
            forward_quadratures: false,
            admission,
            current_execution: Some(current.clone()),
            workspace,
        },
        current.cancellation_control().clone(),
        current.progress.clone(),
        output,
        |_, _| Ok(()),
    );
    Ok(report)
}
/// Additional bound from pinned IDAS 7.1.1: 10 history/work vectors per
/// selected direction, one copied tolerance plus session tolerance/output vectors;
/// the shared saved RHS; host offsets/result/chain/product scratch and pointer arrays.
/// This is accounting for selected storage, not a measured RSS claim.
pub(super) fn forward_quadrature_workspace(
    c: &Contract,
    k: usize,
    integration_width: usize,
) -> Result<usize, ProblemError> {
    if k == 0 || c.quadratures.is_empty() {
        return Ok(0);
    }
    let nq = c.quadratures.len();
    let header = size_of::<ffi::_generic_N_Vector>()
        + size_of::<ffi::_generic_N_Vector_Ops>()
        + size_of::<ffi::_N_VectorContent_Serial>();
    let bytes = nq
        .checked_mul(k)
        .and_then(|v| v.checked_mul(16))
        .and_then(|v| {
            v.checked_add(
                c.states
                    .len()
                    .checked_mul(2)?
                    .checked_add(c.parameters.len())?
                    .checked_mul(k)?,
            )
        })
        .and_then(|v| v.checked_add(nq.checked_mul(2)?))
        .and_then(|v| v.checked_add(integration_width.checked_mul(2)?))
        .and_then(|v| v.checked_mul(size_of::<f64>()))
        .and_then(|v| v.checked_add(k.checked_mul(13)?.checked_add(1)?.checked_mul(header)?))
        .and_then(|v| v.checked_add(k.checked_mul(24)?.checked_mul(size_of::<usize>())?))
        .ok_or_else(|| ProblemError::memory("forward quadrature native workspace extent"))?;
    Ok(bytes)
}
#[allow(
    clippy::too_many_arguments,
    reason = "the admitted workspace and scientific execution have separate owners"
)]
pub(super) fn forward_quadratures(
    oracle: &mut dyn Oracle,
    p: &Profile,
    integration: &[f64],
    cancel: Cancellation,
    progress: Arc<Progress>,
    admission: (Arc<PreparedLayout>, usize),
    reserved_bytes: usize,
    storage: Option<(&mut IdasWorkspace, Execution)>,
) -> Result<ForwardQuadratures, ProblemError> {
    let mut observer = IgnoreSamples;
    let mut output = SampleOutput {
        retention: SampleRetention::Full,
        observer: &mut observer,
    };
    let (workspace, current_execution) = storage.map_or((None, None), |(workspace, execution)| {
        (Some(workspace), Some(execution))
    });
    let (report, derivatives, _) = attempt(
        NumericalRequest {
            oracle,
            profile: p,
            integration,
            directions: &[],
            forward_quadratures: true,
            admission: Some(admission),
            current_execution,
            workspace,
        },
        cancel,
        progress,
        &mut output,
        |session, report| session.forward_quadrature_product(report.completed_time),
    );
    Ok(ForwardQuadratures {
        report,
        derivatives,
        reserved_bytes,
    })
}
/// One IDACalcIC call, with no physical step, sensitivity, quadrature or event setup.
pub(super) fn initialize_consistent(
    oracle: &mut dyn Oracle,
    parameters: &[f64],
    request: &ConsistentInitialization,
    execution: Execution,
) -> Result<ConsistentStateReport, ProblemError> {
    oracle.contract().validate()?;
    request.controls.validate_for(
        request.mode,
        oracle
            .contract()
            .signs
            .iter()
            .any(|sign| *sign != StateSign::Free),
    )?;
    let n = oracle.contract().states.len();
    if !request.time.is_finite()
        || !request.toward.is_finite()
        || request.time == request.toward
        || !(request.toward - request.time).is_finite()
        || !positive(request.rtol)
        || request.atol.len() != n
        || request.residual_tolerances.len() != n
        || request
            .atol
            .iter()
            .chain(&request.residual_tolerances)
            .any(|x| !positive(*x))
        || parameters.len() != oracle.contract().parameters.len()
        || parameters.iter().any(|x| !x.is_finite())
    {
        return Err(contract(
            "invalid consistent-state request or original parameter extent",
        ));
    }
    request.linear.validate_for_states(n)?;
    let dimension = request
        .linear
        .krylov_dimension()
        .map(|dimension| dimension.into_inner());
    if dimension.is_some_and(|value| i32::try_from(value).is_err()) {
        return Err(contract("invalid IDAS initialization Krylov dimension"));
    }
    execution.check()?;
    let allowance = execution.memory.ok_or_else(|| {
        contract("consistent initialization requires a finite foreign memory allowance")
    })?;
    let layout = oracle.prepared_layout(allowance)?;
    let bytes = layout
        .scatter_bytes(DynamicDemand::Base)?
        .checked_add(layout.linear_bytes(request.linear)?)
        .and_then(|bytes| {
            layout
                .action_bytes(request.linear, parameters.len())
                .ok()
                .and_then(|actions| bytes.checked_add(actions))
        })
        .ok_or_else(|| ProblemError::memory("consistent initialization workspace extent"))?;
    if bytes > allowance {
        return Err(ProblemError::memory(
            "consistent initialization workspace exceeds foreign allowance",
        ));
    }
    let final_execution = execution.clone();
    let (mut session, initial) = Session::prepare(
        oracle,
        parameters,
        StatePreparation {
            layout: Some(layout),
            time: request.time,
            parameters: parameters.to_vec(),
            map: (0..parameters.len()).collect(),
            trial_failures: request.trial_failures,
            rtol: request.rtol,
            atol: &request.atol,
            initial_step: None,
            linear: request.linear,
            forward: false,
        },
        execution,
    )?;
    let flag = session.calculate_consistent(request.toward, request.mode, &request.controls)?;
    let initial_residual = session.callback.initial_ic_residual;
    if flag == ffi::IDA_FIRST_RES_FAIL
        && initial_residual == Some(false)
        && session.callback.callback.terminal.is_none()
    {
        // Tagged IDAS cannot repair a failed first residual. This is current-start
        // evaluation evidence, not a globally established model contract failure.
        session.callback.callback.trial_rejections =
            session.callback.callback.trial_rejections.saturating_sub(1);
        session.callback.callback.terminal = Some((
            crate::solve::Termination::Evaluation,
            "invalid initial original residual".into(),
        ));
    }
    let mut native = termination(flag);
    let mut error = if flag < 0 {
        Some(
            session
                .callback
                .native_failure(flag, "consistent initial conditions"),
        )
    } else {
        None
    };
    let callback = &mut session.callback.callback;
    if callback.terminal.is_none()
        && let Some(stop) = callback.execution.stopped()
    {
        callback.last_failure = callback.execution.check().err();
        callback.terminal = Some((
            stop,
            "execution checkpoint after consistent initialization".into(),
        ));
    }
    // SAFETY: only a query/copy of live vectors before any physical step. The pinned
    // API allows retrieval after failed IDACalcIC, so these remain diagnostic candidates.
    let retrieval = unsafe { ffi::IDAGetConsistentIC(session.mem, session.y, session.dy) };
    let candidate = if retrieval == ffi::IDA_SUCCESS {
        // SAFETY: both serial vectors have the authored state extent.
        let state = unsafe { read(session.y, n) };
        // SAFETY: the owned rate serial vector has the same authored state extent.
        let rates = unsafe { read(session.dy, n) };
        if state.iter().chain(&rates).all(|x| x.is_finite()) {
            Some(ConsistentState { state, rates })
        } else {
            None
        }
    } else {
        if error.is_none() {
            error = Some(
                session
                    .callback
                    .native_failure(retrieval, "consistent state retrieval"),
            );
        }
        None
    };
    let iterations = ic_counter(session.mem, ffi::IDAGetNumNonlinSolvIters);
    let backtracks = ic_counter(session.mem, ffi::IDAGetNumBacktrackOps);
    let mut assessment = None;
    let mut validation_error = None;
    if session.callback.callback.terminal.is_none()
        && let Some(candidate) = &candidate
    {
        if let Some(rhs) =
            session
                .callback
                .evaluate(Function::Rhs, request.time, &candidate.state, false)
        {
            let residual: Vec<_> = rhs
                .values
                .iter()
                .enumerate()
                .map(|(i, f)| {
                    if session.callback.contract.differential[i] {
                        candidate.rates[i] - f
                    } else {
                        -f
                    }
                })
                .collect();
            let roles_preserved = match request.mode {
                IdasInitialization::AlgebraicAndRates => session
                    .callback
                    .contract
                    .differential
                    .iter()
                    .enumerate()
                    .all(|(i, d)| !*d || candidate.state[i] == initial.values[i]),
                IdasInitialization::SteadyStates => candidate.rates.iter().all(|v| *v == 0.0),
            };
            let signs_satisfied = session
                .callback
                .contract
                .signs
                .iter()
                .zip(&candidate.state)
                .all(|(sign, x)| match sign {
                    StateSign::Free => true,
                    StateSign::NonNegative => *x >= 0.0,
                    StateSign::Positive => *x > 0.0,
                    StateSign::NonPositive => *x <= 0.0,
                    StateSign::Negative => *x < 0.0,
                });
            let residual_satisfied = residual
                .iter()
                .zip(&request.residual_tolerances)
                .all(|(v, t)| v.is_finite() && v.abs() <= *t);
            assessment = Some(ConsistentStateAssessment {
                residual,
                residual_satisfied,
                roles_preserved,
                signs_satisfied,
            });
        } else {
            validation_error = Some(
                session
                    .callback
                    .failed("consistent-state original assessment"),
            );
        }
    }
    let callback = &mut session.callback.callback;
    if let Some((category, _)) = &callback.terminal {
        native.category = *category;
        if error.is_none() {
            error = callback.terminal_error();
        }
    }
    let mut evidence = crate::solve::Evidence {
        work: crate::solve::WorkEvidence {
            evaluations: callback.counts.values().try_fold(0_u64, |sum, count| {
                sum.checked_add(u64::try_from(*count).ok()?)
            }),
            iterations,
            ..Default::default()
        },
        abandoned: callback.execution.abandonment.observation(),
        callback: crate::solve::CallbackEvidence {
            trial_rejections: callback.trial_rejections,
            regime_crossings: callback.regime_crossings,
            terminal_failure: callback.terminal.is_some(),
        },
        start_submitted: true,
        ..Default::default()
    };
    let identity = session.callback.contract.identity;
    drop(session); // Native destruction stays inside the original task deadline.
    if !evidence.callback.terminal_failure
        && let Some(stop) = final_execution.stopped()
    {
        native.category = stop;
        evidence.callback.terminal_failure = true;
        evidence.abandoned = final_execution.abandonment.observation();
        if error.is_none() {
            error = final_execution.check().err();
        }
        assessment = None;
    }
    Ok(ConsistentStateReport {
        identity,
        time: request.time,
        mode: request.mode,
        initial_residual,
        termination: native,
        requested: initial.values,
        candidate,
        assessment,
        evidence,
        backtracks,
        error,
        validation_error,
    })
}
fn ic_counter(
    mem: *mut c_void,
    get: unsafe extern "C" fn(*mut c_void, *mut c_long) -> i32,
) -> Option<u64> {
    let mut count = 0;
    // SAFETY: counter query on the caller's live IDAS memory, into a correctly typed local.
    if unsafe { get(mem, &raw mut count) } == ffi::IDA_SUCCESS {
        u64::try_from(count).ok()
    } else {
        None
    }
}
/// The gradient of the cotangent's functional over the integration parameters, on the
/// IDAS adjoint (ADR-0110 item 3). The caller validated the profile and charged the
/// checkpoint estimate.
#[expect(
    clippy::too_many_arguments,
    reason = "fresh adjoints bind scientific inputs and the optional actual-current execution separately"
)]
pub(super) fn gradient(
    oracle: &mut dyn Oracle,
    p: &Profile,
    integration: &[f64],
    cotangent: Cotangent<'_>,
    cancel: Cancellation,
    progress: Arc<Progress>,
    admission: (Arc<PreparedLayout>, usize),
    current: Option<Execution>,
) -> Result<Gradient, ProblemError> {
    backward_route(
        oracle,
        p,
        integration,
        &[],
        cotangent,
        cancel,
        progress,
        admission,
        current,
    )
}
/// The gradient and the Hessian over `directions` of the cotangent's functional, by
/// forward-over-adjoint second-order sensitivities (ADR-0110 item 4). The caller admitted
/// the second-order route and charged its estimate.
#[allow(
    clippy::too_many_arguments,
    reason = "numerical inputs, cancellation and admitted native layout have distinct owners"
)]
pub(super) fn hessian(
    oracle: &mut dyn Oracle,
    p: &Profile,
    integration: &[f64],
    directions: &[usize],
    cotangent: Cotangent<'_>,
    cancel: Cancellation,
    progress: Arc<Progress>,
    admission: (Arc<PreparedLayout>, usize),
    current: Option<Execution>,
) -> Result<Gradient, ProblemError> {
    backward_route(
        oracle,
        p,
        integration,
        directions,
        cotangent,
        cancel,
        progress,
        admission,
        current,
    )
}
/// One forward pass with checkpoints, the cotangent of its samples, and the backward pass
/// of the first-order route or, with `directions`, the second-order route.
#[allow(
    clippy::too_many_arguments,
    reason = "numerical inputs, cancellation and admitted native layout have distinct owners"
)]
fn backward_route(
    oracle: &mut dyn Oracle,
    p: &Profile,
    integration: &[f64],
    directions: &[usize],
    cotangent: Cotangent<'_>,
    cancel: Cancellation,
    progress: Arc<Progress>,
    admission: (Arc<PreparedLayout>, usize),
    current: Option<Execution>,
) -> Result<Gradient, ProblemError> {
    let mut observer = IgnoreSamples;
    let mut output = SampleOutput {
        retention: SampleRetention::Full,
        observer: &mut observer,
    };
    let (mut report, outcome, checkpoints) = attempt(
        NumericalRequest {
            oracle,
            profile: p,
            integration,
            directions,
            forward_quadratures: false,
            admission: Some(admission),
            current_execution: current,
            workspace: None,
        },
        cancel,
        progress,
        &mut output,
        |s, r| {
            let weights = cotangent(r)?;
            if weights.len()
                != r.samples
                    .len()
                    .saturating_mul(s.callback.contract.outputs.len())
                || weights.iter().any(|w| !w.is_finite())
            {
                return Err(contract("adjoint cotangent extent or value"));
            }
            let result = s.backward(p, r, &weights)?;
            Ok((
                result,
                s.backward_statistics(),
                s.replay_steps.as_ref().map(StepBudget::used),
            ))
        },
    );
    let completed = report.termination == Termination::Completed;
    let (gradient, hessian) = match outcome {
        Some(((gradient, second), backward, replay_steps)) => {
            report.statistics.push(serde_json::json!({
                "adjoint": {
                    "checkpoints": checkpoints,
                    "steps_between_checkpoints": p.adjoint.steps_between_checkpoints.into_inner(),
                    "segments": p.boundaries().len() + 1,
                    "second_order_directions": directions,
                    "hessian_asymmetry": second.as_ref().map(|(_, a)| *a),
                    "backward": backward,
                    "accepted_step_cap_per_trajectory": p.max_steps,
                    "forward_replay_accepted_steps": replay_steps,
                }
            }));
            (Some(gradient), second.map(|(h, _)| h))
        }
        None => (None, None),
    };
    Ok(Gradient {
        parameter_axes: p.adjoint_axes(
            p.contract_width(integration.len())
                .unwrap_or(integration.len()),
        ),
        gradient: gradient.filter(|_| completed),
        hessian: hessian.filter(|_| completed),
        report,
        checkpoints,
        reserved_bytes: 0,
    })
}
/// Borrowed numerical inputs shared by forward-only and adjoint attempts.
struct NumericalRequest<'a> {
    oracle: &'a mut dyn Oracle,
    profile: &'a Profile,
    integration: &'a [f64],
    directions: &'a [usize],
    forward_quadratures: bool,
    admission: Option<(Arc<PreparedLayout>, usize)>,
    current_execution: Option<Execution>,
    workspace: Option<&'a mut IdasWorkspace>,
}
/// One IDAS attempt: the forward pass, then `after` once it completed, with every failure
/// and stop kept in the report.
fn attempt<T>(
    request: NumericalRequest<'_>,
    cancel: Cancellation,
    progress: Arc<Progress>,
    output: &mut SampleOutput<'_>,
    after: impl FnOnce(&mut Session<'_>, &Report) -> Result<T, ProblemError>,
) -> (Report, Option<T>, usize) {
    let NumericalRequest {
        oracle,
        profile: p,
        integration,
        directions,
        forward_quadratures,
        admission,
        current_execution,
        mut workspace,
    } = request;
    let execution = current_execution.unwrap_or_else(|| Execution {
        cancel: cancel.flag(),
        cancellation: cancel,
        started: Instant::now(),
        time_limit: p.time_limit,
        progress,
        memory: admission.as_ref().map(|(_, memory)| *memory),
        work_admission: None,
        callback_work_owner: true,
        callback_role: pse_diagnostics::observation::ProducerRole::NativeCallback,
        enclosing_scope: None,
        abandonment: Arc::default(),
    });
    let mut r = Report::new(p, output.retention, oracle.contract().parameters.len());
    if !directions.is_empty() {
        r.forward_axes = (0..integration.len()).collect();
    }
    let n = oracle.contract().states.len();
    let failure = |mut report: Report, error: ProblemError| {
        report.termination = stopped(&error, &execution);
        report.error = Some(error);
        (report.progress, report.dropped_progress) = execution.progress.snapshot();
        report
    };
    let mut retained = None;
    let mut storage_key = None;
    let mut coverage = None;
    // Even a fresh/unsupported request must not borrow an allowance occupied by
    // another retained native owner. The temporary identity lasts through teardown.
    let fresh_identity = Arc::new(());
    let identity = workspace
        .as_ref()
        .map_or(&fresh_identity, |owner| &owner.identity)
        .clone();
    let previous = workspace.as_deref_mut().and_then(|owner| owner.held.take());
    if let Some((_, bytes)) = &admission {
        coverage = match execution
            .work_admission
            .as_ref()
            .map_or(Ok(None), |owner| owner.prepaid_storage(&identity, *bytes))
        {
            Ok(coverage) => coverage,
            Err(error) => return (failure(r, error), None, 0),
        };
    }
    if workspace.is_some()
        && coverage.is_some()
        && p.sensitivity != DynamicSensitivity::Adjoint
        && directions.is_empty()
        && !matches!(p.idas.linear, IdasLinear::SpgmrBbd { .. })
        && let Some((layout, _)) = &admission
    {
        let profile = match profile_identity(p) {
            Ok(profile) => profile,
            Err(error) => return (failure(r, error), None, 0),
        };
        let key = StorageKey {
            contract: oracle.contract().identity,
            profile,
            layout: layout.clone(),
            forward_quadratures,
        };
        if let Some((old, storage)) = previous
            && old.matches(&key)
        {
            retained = Some(storage);
        }
        storage_key = Some(key);
    } else {
        drop(previous);
    }
    // Bind and charge the complete BBD replacement population once, before even
    // the requested-initial observation can invoke original source effects.
    let native_admission = match bbd::admit(p.idas.linear, n, execution.clone()) {
        Ok(admission) => admission,
        Err(error) => return (failure(r, error), None, 0),
    };
    let initial = crate::quality::contained(|| {
        execution.check()?;
        let initial = oracle.evaluate(
            0,
            Function::Initial,
            p.start,
            &vec![0.0; n],
            &p.parameters_at(integration, p.start),
            false,
        )?;
        execution.check()?;
        Ok(initial)
    });
    let initial = match initial {
        Ok(initial) => initial,
        Err(error) => return (failure(r, error), None, 0),
    };
    r.requested_initial = initial.values;
    let mut s = match Session::new_storage(
        oracle,
        integration,
        p,
        directions,
        admission.map(|(layout, _)| layout),
        native_admission,
        forward_quadratures,
        retained,
    ) {
        Ok(session) => session,
        Err(error) => return (failure(r, error), None, 0),
    };
    if let Some(coverage) = coverage {
        s.storage_lease = Some(coverage);
    }
    crate::observation::count(
        if s.reused {
            "idas.workspace.reuses"
        } else {
            "idas.workspace.constructions"
        },
        1,
    );
    #[cfg(test)]
    if s.reused
        && let Some(workspace) = workspace.as_deref_mut()
    {
        workspace.reuses += 1;
    }
    // SAFETY: the new session's state vector holds the `n` states.
    r.consistent_initial = unsafe { read(s.y, n) };
    let mut outcome = None;
    let result = s.run(p, &mut r, output).and_then(|()| {
        if r.termination == Termination::Completed {
            outcome = Some(after(&mut s, &r)?);
        }
        Ok(())
    });
    if let Err(e) = result {
        if r.termination != Termination::Panic {
            r.termination = stopped(&e, &s.callback.callback.execution);
        }
        r.error = Some(e);
        outcome = None;
    }
    // Terminal-policy and checkpoint stops keep their typed status and cause.
    if let Some((status, _)) = &s.callback.callback.terminal {
        r.termination = match status {
            crate::solve::Termination::Cancelled => Termination::Cancelled,
            crate::solve::Termination::TimeLimit => Termination::TimeLimit,
            crate::solve::Termination::Panic => Termination::Panic,
            _ => Termination::Failed,
        };
        if r.error.is_none() {
            r.error = s.callback.callback.terminal_error();
        }
        outcome = None;
    }
    (r.progress, r.dropped_progress) = s.callback.callback.execution.progress.snapshot();
    let checkpoints = s.adjoint.as_ref().map_or(0, |b| b.checkpoints);
    if let (Some(workspace), Some(key)) = (workspace, storage_key)
        && r.termination == Termination::Completed
        && r.error.is_none()
        && s.callback.callback.execution.check().is_ok()
    {
        // Detach before the current Box/oracle can die. Failure destroys the
        // native owner while that Box remains alive; no uncertain reset escapes.
        if let Some(slot) = &mut s.user_data {
            slot.current = std::ptr::null_mut();
        }
        if native!(
            ffi::IDASetUserData(s.mem, std::ptr::null_mut()),
            "detach independent callback"
        )
        .is_ok()
        {
            let mut storage = NativeStorage::take(&mut s);
            storage.totals.fill(f64::NAN);
            storage.quad_sens_totals.fill(f64::NAN);
            storage.transition = None;
            workspace.held = Some((key, storage));
        }
    }
    (r, outcome, checkpoints)
}

#[allow(
    clippy::useless_conversion,
    reason = "C long width differs between native ABIs"
)]
fn long_counter(value: c_long) -> i64 {
    i64::from(value)
}

#[cfg(test)]
mod initialization_tests {
    use super::*;
    #[test]
    fn compiled_scatter_preserves_subsets_transpose_and_outside_support_refusal() {
        let pattern = [(0, 0), (0, 2), (1, 0), (1, 1), (2, 1), (2, 2)];
        let (columns, rows) = csc(3, &pattern).unwrap();
        let mut transposed: Vec<_> = pattern.iter().map(|&(col, row)| (row, col)).collect();
        transposed.sort_unstable();
        let (columns_b, rows_b) = csc(3, &transposed).unwrap();
        let destinations: Vec<_> = pattern
            .iter()
            .map(|&(col, row)| slot(&columns_b, &rows_b, row, col).unwrap())
            .collect();
        let source = faer::sparse::SparseColMat::try_new_from_triplets(
            3,
            4,
            &[
                faer::sparse::Triplet::new(2, 0, 4.),
                faer::sparse::Triplet::new(0, 1, 7.),
                faer::sparse::Triplet::new(1, 2, -3.),
                // Parameter columns do not enter the state Newton matrix.
                faer::sparse::Triplet::new(1, 3, 99.),
            ],
        )
        .unwrap();
        let slots = PreparedLayout::new(
            3,
            pattern
                .iter()
                .map(|&(col, row)| SupportEntry::new(OriginalRow::new(row), OriginalCol::new(col))),
            usize::MAX,
        )
        .unwrap();
        let occupied = slots.retained_bytes();
        assert!(slots.scatter_bytes(DynamicDemand::Base).unwrap() >= occupied);
        assert!(
            slots.scatter_bytes(DynamicDemand::ExactHessian).unwrap()
                > slots.scatter_bytes(DynamicDemand::Base).unwrap()
        );
        let mut forward = vec![0.; rows.len()];
        let mut backward = vec![0.; rows_b.len()];
        assert!(scatter_partials(
            source.as_ref(),
            &slots,
            None,
            &mut forward
        ));
        assert!(scatter_partials(
            source.as_ref(),
            &slots,
            Some(&destinations),
            &mut backward
        ));
        assert_eq!(slots.retained_bytes(), occupied);
        assert_eq!(forward, [0., -4., -7., 0., 3., 0.]);
        for &(col, row) in &pattern {
            assert_eq!(
                forward[slot(&columns, &rows, col, row).unwrap()],
                -backward[slot(&columns_b, &rows_b, row, col).unwrap()]
            );
        }
        let outside = faer::sparse::SparseColMat::try_new_from_triplets(
            3,
            3,
            &[faer::sparse::Triplet::new(1, 0, 1.)],
        )
        .unwrap();
        assert!(!scatter_partials(
            outside.as_ref(),
            &slots,
            None,
            &mut forward
        ));
    }
    #[derive(Debug)]
    struct IcOracle {
        contract: Contract,
        residual_calls: usize,
        fail: bool,
        nonlinear: bool,
        omit_support: bool,
        state_pointer: Arc<std::sync::atomic::AtomicUsize>,
    }
    impl IcOracle {
        fn new() -> Self {
            let id = |n: u8| SemanticId::from_bytes([n; 16]);
            Self {
                contract: Contract {
                    identity: ContentHash::from_bytes([91; 32]),
                    states: vec![id(1), id(2)],
                    differential: vec![true, false],
                    parameters: vec![],
                    outputs: vec![id(3)],
                    events: vec![vec![]],
                    signs: vec![],
                    derivatives: pse_kernels::DerivativeOrder::First,
                    quadratures: vec![],
                    balances: vec![],
                },
                residual_calls: 0,
                fail: false,
                nonlinear: false,
                omit_support: false,
                state_pointer: Arc::default(),
            }
        }
    }
    impl Oracle for IcOracle {
        fn contract(&self) -> &Contract {
            &self.contract
        }
        fn support(&self, _: usize, function: Function) -> Vec<SupportEntry> {
            if function == Function::Rhs {
                if self.omit_support {
                    return [(0, 0), (1, 1)]
                        .map(|(row, col)| {
                            SupportEntry::new(OriginalRow::new(row), OriginalCol::new(col))
                        })
                        .to_vec();
                }
                [(0, 0), (1, 0), (1, 1)]
                    .map(|(r, c)| SupportEntry::new(OriginalRow::new(r), OriginalCol::new(c)))
                    .to_vec()
            } else {
                vec![]
            }
        }
        fn evaluate(
            &mut self,
            _: usize,
            function: Function,
            _: f64,
            x: &[f64],
            _: &[f64],
            derivatives: bool,
        ) -> Result<Evaluation, ProblemError> {
            if function == Function::Initial {
                return Ok(Evaluation {
                    values: vec![2.0, if self.nonlinear { 1.0 } else { 0.0 }],
                    jacobian: None,
                });
            }
            assert_eq!(
                function,
                Function::Rhs,
                "IC-only call does not evaluate outputs, events or quadratures"
            );
            self.residual_calls += 1;
            self.state_pointer
                .store(x.as_ptr() as usize, std::sync::atomic::Ordering::Relaxed);
            if self.fail {
                return Err(pse_math::MathError::Domain {
                    source_id: SemanticId::from_bytes([92; 16]),
                    requirement: "IC original-domain witness",
                }
                .into());
            }
            let rhs = if self.nonlinear {
                x[0] * x[0] - x[1] * x[1]
            } else {
                2.0 * x[0] - x[1]
            };
            let jacobian = if derivatives {
                Some(
                    faer::sparse::SparseColMat::try_new_from_triplets(
                        2,
                        2 + self.contract.parameters.len(),
                        &[
                            faer::sparse::Triplet::new(0, 0, -1.0),
                            faer::sparse::Triplet::new(
                                1,
                                0,
                                if self.nonlinear { 2.0 * x[0] } else { 2.0 },
                            ),
                            faer::sparse::Triplet::new(
                                1,
                                1,
                                if self.nonlinear { -2.0 * x[1] } else { -1.0 },
                            ),
                        ],
                    )
                    .unwrap(),
                )
            } else {
                None
            };
            Ok(Evaluation {
                values: vec![-x[0], rhs],
                jacobian: jacobian.map(|matrix| Arc::new(matrix).into()),
            })
        }
    }
    fn request() -> ConsistentInitialization {
        ConsistentInitialization {
            time: 0.0,
            toward: 0.1,
            rtol: 1e-9,
            atol: vec![1e-11; 2],
            residual_tolerances: vec![1e-8; 2],
            mode: IdasInitialization::AlgebraicAndRates,
            linear: IdasLinear::Klu,
            controls: Default::default(),
            trial_failures: TrialPolicy::Terminal,
        }
    }
    fn execution() -> Execution {
        let mut e = Execution::new(Cancellation::default(), &crate::solve::Controls::default());
        e.memory = Some(1 << 20);
        e
    }
    #[test]
    fn public_ic_preserves_original_roles_and_has_no_physical_trajectory() {
        use pse_diagnostics::observation::{ObservationHub, Policy, Value};
        let hub = ObservationHub::new(Policy {
            max_metrics: 64,
            ..Policy::default()
        });
        let operation = hub.start("idas-real-ic-only");
        let _entered = operation.context().enter();
        let mut oracle = IcOracle::new();
        let report = initialize_consistent(&mut oracle, &[], &request(), execution()).unwrap();
        assert_eq!(report.termination.name, "IDA_SUCCESS");
        assert!(report.error.is_none());
        let point = report.candidate.unwrap();
        assert_eq!(point.state[0], 2.0);
        assert!((point.state[1] - 4.0).abs() < 1e-8);
        assert!((point.rates[0] + 2.0).abs() < 1e-8);
        let assessment = report.assessment.unwrap();
        assert!(
            assessment.roles_preserved
                && assessment.residual_satisfied
                && assessment.signs_satisfied
        );
        assert!(report.evidence.work.iterations.is_some_and(|n| n > 0));
        assert!(report.evidence.work.evaluations.is_some_and(|n| n > 0));
        assert_eq!(report.evidence.work.factorizations, None);
        assert_eq!(report.evidence.work.proof_steps, None);
        assert!(report.backtracks.is_some());
        let observed = operation.reader().snapshot(Duration::from_secs(60));
        assert_eq!(
            observed
                .metrics
                .iter()
                .find(|m| m.name == "idas.consistent_initialization.calls")
                .unwrap()
                .cumulative,
            Value::Count(1)
        );
        assert!(
            !observed
                .metrics
                .iter()
                .any(|m| m.name == "idas.forward.calls")
        );
        let mut all_states = request();
        all_states.mode = IdasInitialization::SteadyStates;
        let report =
            initialize_consistent(&mut IcOracle::new(), &[], &all_states, execution()).unwrap();
        assert_eq!(report.termination.name, "IDA_SUCCESS");
        let point = report.candidate.unwrap();
        assert!(point.state.iter().all(|v| v.abs() < 1e-8));
        assert!(point.rates.iter().all(|v| *v == 0.0));
        assert!(report.assessment.unwrap().roles_preserved);
    }
    fn input_session(oracle: &mut IcOracle, linear: IdasLinear) -> Session<'_> {
        let parameters = vec![0.0; oracle.contract.parameters.len()];
        let map = (0..parameters.len()).collect();
        Session::prepare(
            oracle,
            &parameters,
            StatePreparation {
                layout: None,
                time: 0.0,
                parameters: parameters.clone(),
                map,
                trial_failures: TrialPolicy::Terminal,
                rtol: 1e-8,
                atol: &[1e-10; 2],
                initial_step: None,
                linear,
                forward: false,
            },
            execution(),
        )
        .unwrap()
        .0
    }
    #[test]
    fn coalesced_integrator_statistics_match_individual_getters_across_native_lifecycle() {
        fn check(session: &Session<'_>) -> IntegratorStatistics {
            let actual = session.integrator_statistics().unwrap();
            let mut expected = IntegratorStatistics::default();
            macro_rules! get {
                ($getter:ident, $field:ident) => {
                    assert_eq!(
                        // SAFETY: the session owns live native memory and this getter's
                        // exact C ABI output is private local storage. No solve intervenes.
                        unsafe { ffi::$getter(session.mem, &raw mut expected.$field) },
                        ffi::IDA_SUCCESS
                    );
                };
            }
            get!(IDAGetNumSteps, steps);
            get!(IDAGetNumResEvals, residuals);
            get!(IDAGetNumLinSolvSetups, linear_setups);
            get!(IDAGetNumErrTestFails, error_test_failures);
            get!(IDAGetLastOrder, last_order);
            get!(IDAGetCurrentOrder, current_order);
            get!(IDAGetActualInitStep, initial_step);
            get!(IDAGetLastStep, last_step);
            get!(IDAGetCurrentStep, current_step);
            get!(IDAGetCurrentTime, internal_time);
            assert_eq!(actual, expected);
            actual
        }
        let mut oracle = IcOracle::new();
        let mut session = input_session(&mut oracle, IdasLinear::Klu);
        let profile = Profile {
            atol: vec![1e-10; 2],
            sensitivity: DynamicSensitivity::None,
            ..Default::default()
        };
        assert_eq!(check(&session).steps, 0);
        session
            .consistent(0., &profile, ffi::IDA_YA_YDP_INIT)
            .unwrap();
        assert_eq!(check(&session).steps, 0);
        assert!(
            session.statistics(0, &profile, None)["native_step"]["last_step"].is_null(),
            "IC artificial steps never become accepted-step observations"
        );
        assert!(session.statistics(0, &profile, None)["actual_initial_step"].is_null());
        let mut time = 0.;
        assert_eq!(
            session.solve(0.1, &mut time, "statistics control").unwrap(),
            ffi::IDA_SUCCESS
        );
        let advanced = check(&session);
        assert!(advanced.steps > 0);
        assert_eq!(
            session.statistics(0, &profile, None)["actual_initial_step"].as_f64(),
            Some(advanced.initial_step)
        );
        // SAFETY: these two-state result vectors remain owned by this live session.
        let state = unsafe { read(session.y, 2) };
        session.restart(time, &state, &profile, false).unwrap();
        assert_eq!(check(&session).steps, 0);
        assert!(session.statistics(0, &profile, None)["native_step"]["last_step"].is_null());
        assert!(
            session.statistics(0, &profile, None)["actual_initial_step"].is_null(),
            "reset does not retain the earlier segment's initial step"
        );
        assert_eq!(
            // SAFETY: the session owns live native memory exclusively throughout
            // this per-call bound change; no solve is concurrent with this setter.
            unsafe { ffi::IDASetMaxNumSteps(session.mem, 1) },
            ffi::IDA_SUCCESS
        );
        assert_eq!(
            session
                .solve(2., &mut time, "limited statistics control")
                .unwrap(),
            ffi::IDA_TOO_MUCH_WORK
        );
        let limited = check(&session);
        assert!(
            limited.steps > 0,
            "failed native returns retain actual completed work"
        );
        assert_eq!(
            session.statistics(ffi::IDA_TOO_MUCH_WORK, &profile, None)["actual_initial_step"]
                .as_f64(),
            Some(limited.initial_step)
        );
        let mut unavailable = IntegratorStatistics {
            steps: -7,
            residuals: -7,
            linear_setups: -7,
            error_test_failures: -7,
            last_order: -7,
            current_order: -7,
            initial_step: -7.,
            last_step: -7.,
            current_step: -7.,
            internal_time: -7.,
        };
        let before = unavailable;
        // SAFETY: the documented null-memory native error boundary writes none
        // of these valid exact-type local output pointers and retains none.
        let flag = unsafe {
            ffi::IDAGetIntegratorStats(
                std::ptr::null_mut(),
                &raw mut unavailable.steps,
                &raw mut unavailable.residuals,
                &raw mut unavailable.linear_setups,
                &raw mut unavailable.error_test_failures,
                &raw mut unavailable.last_order,
                &raw mut unavailable.current_order,
                &raw mut unavailable.initial_step,
                &raw mut unavailable.last_step,
                &raw mut unavailable.current_step,
                &raw mut unavailable.internal_time,
            )
        };
        assert_eq!(flag, ffi::IDA_MEM_NULL);
        assert_eq!(unavailable, before);
    }
    #[test]
    fn callback_residual_borrows_native_state_and_publishes_after_aliased_inputs_end() {
        for fail in [false, true] {
            for alias_rate in [false, true] {
                let mut oracle = IcOracle::new();
                oracle.fail = fail;
                let observed = oracle.state_pointer.clone();
                let mut session = input_session(&mut oracle, IdasLinear::Klu);
                // SAFETY: the session owns this two-entry state vector with no live borrow.
                unsafe { write(session.y, &[2.0, 0.0]) };
                // SAFETY: the independently writable rate vector also has exactly two entries.
                unsafe { write(session.dy, &[5.0, 7.0]) };
                let output = if alias_rate { session.dy } else { session.y };
                // SAFETY: y is live in this session throughout the synchronous callback.
                let expected_pointer = unsafe { ffi::N_VGetArrayPointer(session.y) } as usize;
                // SAFETY: the live context and every native vector belong to this
                // owner. The output intentionally aliases one of the native inputs.
                let status =
                    unsafe { residual(0.0, session.y, session.dy, output, session.user_data()) };
                assert_eq!(
                    observed.load(std::sync::atomic::Ordering::Relaxed),
                    expected_pointer,
                    "the oracle receives the actual borrowed native input"
                );
                // SAFETY: output is still a live two-entry session vector.
                let actual = unsafe { read(output, 2) };
                if fail {
                    assert_ne!(status, 0);
                    assert_eq!(actual, if alias_rate { [5.0, 7.0] } else { [2.0, 0.0] });
                    assert!(matches!(
                        session.callback.callback.last_failure,
                        Some(ProblemError::Math(pse_math::MathError::Domain { .. }))
                    ));
                } else {
                    assert_eq!(status, 0);
                    // f=[-2,4], M=[1,0]: M*y'-f=[7,-4].
                    assert_eq!(actual, [7.0, -4.0]);
                }
            }
        }
    }
    #[test]
    fn callback_jacobi_reuses_private_product_and_refuses_malformed_inputs_before_publish() {
        let mut oracle = IcOracle::new();
        let mut session = input_session(
            &mut oracle,
            IdasLinear::Spgmr {
                dimension: PositiveCount::try_new(4).unwrap(),
                preconditioner: Preconditioner::Jacobi,
            },
        );
        session
            .callback
            .inverse_diagonal
            .copy_from_slice(&[0.5, -2.0]);
        let vector = session.vector(&[6.0, 9.0]).unwrap();
        let pointer = session.callback.rhs_product.as_ptr();
        for _ in 0..2 {
            // SAFETY: vector is a live session-owned two-entry serial vector.
            unsafe {
                write(vector, &[6.0, 9.0]);
            }
            // SAFETY: this direct callback uses the live context and deliberately
            // aliases its n-entry native input and output; unused inputs are null.
            let status = unsafe {
                precondition_solve(
                    0.0,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    vector,
                    vector,
                    0.0,
                    0.0,
                    session.user_data(),
                )
            };
            assert_eq!(status, 0);
            // SAFETY: vector remains the same live two-entry serial vector.
            assert_eq!(unsafe { read(vector, 2) }, [3.0, -18.0]);
            assert_eq!(session.callback.rhs_product.as_ptr(), pointer);
        }
        let short = session.vector(&[42.0]).unwrap();
        for input in [std::ptr::null_mut(), short] {
            // SAFETY: vector is live and its output sentinel is independently owned.
            unsafe {
                write(vector, &[11.0, 13.0]);
            }
            // SAFETY: input is null or a live serial vector; the callback checks
            // its extent before borrowing. All other pointers obey the contract.
            let status = unsafe {
                precondition_solve(
                    0.0,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    input,
                    vector,
                    0.0,
                    0.0,
                    session.user_data(),
                )
            };
            assert_eq!(status, -1);
            // SAFETY: the output is live and refusal must leave its sentinel intact.
            assert_eq!(unsafe { read(vector, 2) }, [11.0, 13.0]);
            assert!(matches!(
                session.callback.callback.last_failure,
                Some(ProblemError::Internal(_))
            ));
        }
    }
    #[derive(Debug)]
    struct SelectedDiagonal {
        inner: IcOracle,
        diagonal: Arc<[std::sync::atomic::AtomicU64; 2]>,
        full_calls: Arc<std::sync::atomic::AtomicUsize>,
    }
    impl Oracle for SelectedDiagonal {
        fn contract(&self) -> &Contract {
            self.inner.contract()
        }
        fn support(&self, mode: usize, function: Function) -> Vec<SupportEntry> {
            self.inner.support(mode, function)
        }
        fn evaluate(
            &mut self,
            mode: usize,
            function: Function,
            time: f64,
            state: &[f64],
            parameters: &[f64],
            derivatives: bool,
        ) -> Result<Evaluation, ProblemError> {
            if derivatives {
                self.full_calls
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }
            self.inner
                .evaluate(mode, function, time, state, parameters, derivatives)
        }
        fn rhs_state_diagonal(
            &mut self,
            _: usize,
            _: f64,
            _: &[f64],
            _: &[f64],
            output: &mut [f64],
        ) -> Result<RhsDiagonalSource, ProblemError> {
            for (out, value) in output.iter_mut().zip(self.diagonal.iter()) {
                *out = f64::from_bits(value.load(std::sync::atomic::Ordering::Relaxed));
            }
            Ok(RhsDiagonalSource::SelectedEntries)
        }
    }
    #[test]
    fn jacobi_selected_diagonal_preserves_shift_mass_fallback_and_no_full_requests() {
        use std::sync::atomic::{AtomicU64, Ordering};
        let diagonal = Arc::new([
            AtomicU64::new((-1.0f64).to_bits()),
            AtomicU64::new((-2.0f64).to_bits()),
        ]);
        let full_calls = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let mut oracle = SelectedDiagonal {
            inner: IcOracle::new(),
            diagonal: diagonal.clone(),
            full_calls: full_calls.clone(),
        };
        let (mut session, _) = Session::prepare(
            &mut oracle,
            &[],
            StatePreparation {
                layout: None,
                time: 0.,
                parameters: vec![],
                map: vec![],
                trial_failures: TrialPolicy::Terminal,
                rtol: 1e-8,
                atol: &[1e-10; 2],
                initial_step: None,
                linear: IdasLinear::Spgmr {
                    dimension: PositiveCount::try_new(4).unwrap(),
                    preconditioner: Preconditioner::Jacobi,
                },
                forward: false,
            },
            execution(),
        )
        .unwrap();
        let pointer = session.callback.rhs_product.as_ptr();
        for (shift, raw, expected) in [
            (3., [-1., -2.], [0.25, 0.5]),
            (7., [-1., -2.], [0.125, 0.5]),
            (0., [0., 1e-320], [1., 1.]),
            (f64::MAX, [-f64::MAX, 0.], [1., 1.]),
        ] {
            for (value, raw) in diagonal.iter().zip(raw) {
                value.store(f64::to_bits(raw), Ordering::Relaxed);
            }
            // SAFETY: all supplied vectors/context belong to this live two-state
            // session; the unused native inputs are null and never dereferenced.
            let status = unsafe {
                precondition_setup(
                    0.,
                    session.y,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    shift,
                    session.user_data(),
                )
            };
            assert_eq!(status, 0);
            assert_eq!(
                session.callback.inverse_diagonal, expected,
                "independent diag(cj*[1,0]-raw) with unchanged finite/small fallback"
            );
            assert_eq!(session.callback.rhs_product.as_ptr(), pointer);
        }
        assert_eq!(full_calls.load(Ordering::Relaxed), 0);
        let before = session.callback.inverse_diagonal.clone();
        diagonal[1].store(f64::NAN.to_bits(), Ordering::Relaxed);
        // SAFETY: the same live session owns all used handles; malformed raw
        // numerical output is refused before replacing any inverse entry.
        let status = unsafe {
            precondition_setup(
                0.,
                session.y,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                2.,
                session.user_data(),
            )
        };
        assert_ne!(status, 0);
        assert_eq!(session.callback.inverse_diagonal, before);
    }
    #[test]
    fn rhs_diagonal_full_fallback_keeps_prepared_support_refusal_without_support_replay() {
        let mut oracle = IcOracle::new();
        oracle.omit_support = true;
        let mut session = input_session(
            &mut oracle,
            IdasLinear::Spgmr {
                dimension: PositiveCount::try_new(4).unwrap(),
                preconditioner: Preconditioner::Jacobi,
            },
        );
        session
            .callback
            .inverse_diagonal
            .copy_from_slice(&[9., 10.]);
        // SAFETY: all used handles/context belong to this live session. The oracle
        // deliberately returns an off-diagonal absent from its prepared support.
        let status = unsafe {
            precondition_setup(
                0.,
                session.y,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                3.,
                session.user_data(),
            )
        };
        assert_ne!(status, 0);
        assert_eq!(session.callback.inverse_diagonal, [9., 10.]);
        assert!(matches!(
            session.callback.callback.last_failure,
            Some(ProblemError::Contract(_))
        ));
    }
    #[test]
    fn rhs_diagonal_full_fallback_is_explicit_and_refuses_before_output_publish() {
        let mut oracle = IcOracle::new();
        let mut output = [42., 43.];
        assert!(matches!(
            oracle
                .rhs_state_diagonal(0, 0., &[2., 4.], &[], &mut output)
                .unwrap(),
            RhsDiagonalSource::FullJacobian(_)
        ));
        assert_eq!(output, [-1., -1.]);
        oracle.fail = true;
        output = [42., 43.];
        assert!(
            oracle
                .rhs_state_diagonal(0, 0., &[2., 4.], &[], &mut output)
                .is_err()
        );
        assert_eq!(output, [42., 43.]);
        assert!(
            oracle
                .rhs_state_diagonal(1, 0., &[2., 4.], &[], &mut output)
                .is_err()
        );
        assert_eq!(output, [42., 43.]);
    }
    #[test]
    fn callback_sensitivities_finish_all_rate_borrows_before_cross_axis_alias_publication() {
        let mut oracle = IcOracle::new();
        oracle.contract.parameters = vec![
            SemanticId::from_bytes([3; 16]),
            SemanticId::from_bytes([4; 16]),
        ];
        let mut session = input_session(&mut oracle, IdasLinear::Klu);
        // This direct callback fixture selects two physically valid parameter axes;
        // it does not ask the native integrator to advance sensitivity history.
        session.callback.axes = vec![0, 1];
        let mut states = [
            session.vector(&[1.0, 2.0]).unwrap(),
            session.vector(&[3.0, 4.0]).unwrap(),
        ];
        let mut rates = [
            session.vector(&[5.0, 6.0]).unwrap(),
            session.vector(&[7.0, 8.0]).unwrap(),
        ];
        // The first output aliases the second input rate, so publishing the first
        // result early would corrupt the still-needed second input.
        let mut outputs = [rates[1], rates[0]];
        // SAFETY: the live session/context owns each two-entry vector; all three
        // arrays contain the stated two handles. Output/input alias is intentional.
        let status = unsafe {
            sensitivities(
                2,
                0.0,
                session.y,
                session.dy,
                session.dy,
                states.as_mut_ptr(),
                rates.as_mut_ptr(),
                outputs.as_mut_ptr(),
                session.user_data(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        };
        assert_eq!(status, 0);
        // SAFETY: the first output still denotes a live two-entry session vector.
        let first = unsafe { read(outputs[0], 2) };
        // SAFETY: the second output also remains live at its two-entry extent.
        let second = unsafe { read(outputs[1], 2) };
        assert_eq!(first, [6.0, 0.0]);
        assert_eq!(second, [10.0, -2.0]);
    }
    #[test]
    fn krylov_action_reuses_admitted_buffers_and_publishes_after_foreign_input_borrows_end() {
        use pse_model::scalars::PositiveCount;
        for fail in [false, true] {
            let mut oracle = IcOracle::new();
            oracle.fail = fail;
            let linear = IdasLinear::Spgmr {
                dimension: PositiveCount::try_new(4).unwrap(),
                preconditioner: Preconditioner::None,
            };
            let (mut session, _) = Session::prepare(
                &mut oracle,
                &[],
                StatePreparation {
                    layout: None,
                    time: 0.,
                    parameters: vec![],
                    map: vec![],
                    trial_failures: TrialPolicy::Terminal,
                    rtol: 1e-8,
                    atol: &[1e-10; 2],
                    initial_step: None,
                    linear,
                    forward: false,
                },
                execution(),
            )
            .unwrap();
            let direction = session.vector(&[1., 3.]).unwrap();
            let identity = (
                session.callback.rhs_direction.as_ptr(),
                session.callback.rhs_product.as_ptr(),
            );
            for alias_state in [false, true] {
                // SAFETY: both vectors belong to this live session, with two entries.
                unsafe { write(direction, &[1., 3.]) };
                // SAFETY: the session's state vector also owns exactly two entries.
                unsafe { write(session.y, &[2., 0.]) };
                let output = if alias_state { session.y } else { direction };
                // SAFETY: the live callback context owns all borrowed data. This
                // deliberate input/output alias exercises read-before-publish.
                let status = unsafe {
                    jtimes(
                        0.,
                        session.y,
                        session.dy,
                        session.dy,
                        direction,
                        output,
                        2.,
                        session.user_data(),
                        std::ptr::null_mut(),
                        std::ptr::null_mut(),
                    )
                };
                // SAFETY: the selected output is still a live two-entry native vector.
                let actual = unsafe { read(output, 2) };
                if fail {
                    assert_ne!(status, 0);
                    assert_eq!(
                        actual,
                        if alias_state {
                            vec![2., 0.]
                        } else {
                            vec![1., 3.]
                        }
                    );
                } else {
                    assert_eq!(status, 0);
                    // Analytic residual Jacobian: diag(2,0) - [[-1,0],[2,-1]].
                    assert_eq!(actual, [3., 1.]);
                }
                assert_eq!(
                    identity,
                    (
                        session.callback.rhs_direction.as_ptr(),
                        session.callback.rhs_product.as_ptr()
                    )
                );
            }
        }
    }
    #[test]
    fn public_ic_admits_selected_klu_and_krylov_workspaces_before_callbacks() {
        use pse_model::scalars::PositiveCount;
        for linear in [
            IdasLinear::Klu,
            IdasLinear::Spgmr {
                dimension: PositiveCount::try_new(4).unwrap(),
                preconditioner: Preconditioner::Jacobi,
            },
            IdasLinear::Spfgmr {
                dimension: PositiveCount::try_new(4).unwrap(),
                preconditioner: Preconditioner::Jacobi,
            },
        ] {
            let mut request = request();
            request.linear = linear;
            let mut oracle = IcOracle::new();
            let layout = oracle.prepared_layout(1 << 20).unwrap();
            let bytes = layout.scatter_bytes(DynamicDemand::Base).unwrap()
                + layout.linear_bytes(linear).unwrap()
                + layout.action_bytes(linear, 0).unwrap();
            let mut refused = execution();
            refused.memory = Some(bytes - 1);
            assert!(initialize_consistent(&mut oracle, &[], &request, refused).is_err());
            assert_eq!(oracle.residual_calls, 0);
            let mut admitted = execution();
            admitted.memory = Some(bytes);
            let report = initialize_consistent(&mut oracle, &[], &request, admitted).unwrap();
            assert_eq!(
                report.termination.name, "IDA_SUCCESS",
                "{linear:?}: {:?}",
                report.error
            );
            let point = report.candidate.unwrap();
            assert_eq!(point.state[0], 2.0);
            assert!((point.state[1] - 4.0).abs() < 1e-8);
            assert!((point.rates[0] + 2.0).abs() < 1e-8);
            assert!(report.assessment.unwrap().roles_preserved);
        }
    }
    #[test]
    fn public_ic_refuses_undeclared_partials_for_direct_and_both_krylov_methods() {
        use pse_model::scalars::PositiveCount;
        for linear in [
            IdasLinear::Klu,
            IdasLinear::Spgmr {
                dimension: PositiveCount::try_new(4).unwrap(),
                preconditioner: Preconditioner::Jacobi,
            },
            IdasLinear::Spfgmr {
                dimension: PositiveCount::try_new(4).unwrap(),
                preconditioner: Preconditioner::Jacobi,
            },
        ] {
            let mut oracle = IcOracle::new();
            oracle.omit_support = true;
            let mut request = request();
            request.linear = linear;
            let error = match initialize_consistent(&mut oracle, &[], &request, execution()) {
                Err(error) => error,
                Ok(report) => report
                    .error
                    .expect("undeclared state partial must refuse before a native result"),
            };
            assert!(
                matches!(error, ProblemError::Contract(ref detail) if detail.contains("outside its declared support")),
                "{linear:?}: {error:?}"
            );
        }
    }
    #[test]
    fn public_ic_native_cap_keeps_failed_status_counters_and_original_assessment() {
        let mut oracle = IcOracle {
            nonlinear: true,
            ..IcOracle::new()
        };
        let mut request = request();
        request.controls.newton_iterations = 1;
        request.controls.jacobian_attempts = 1;
        request.controls.step_trials = Some(1);
        let report = initialize_consistent(&mut oracle, &[], &request, execution()).unwrap();
        assert_eq!(report.termination.name, "IDA_CONV_FAIL");
        assert!(report.error.is_some());
        assert!(!report.evidence.callback.terminal_failure);
        assert!(
            report
                .evidence
                .work
                .iterations
                .is_some_and(|n| n > 0 && n <= 2)
        );
        assert!(report.assessment.is_some());
        assert!(report.backtracks.is_some());
    }
    #[test]
    fn public_ic_terminal_domain_preserves_typed_cause_and_prevents_fresh_assessment() {
        let mut oracle = IcOracle {
            fail: true,
            ..IcOracle::new()
        };
        let report = initialize_consistent(&mut oracle, &[], &request(), execution()).unwrap();
        assert_eq!(report.termination.name, "IDA_RES_FAIL");
        assert_eq!(
            report.termination.category,
            crate::solve::Termination::Evaluation
        );
        assert!(report.evidence.callback.terminal_failure);
        assert_eq!(oracle.residual_calls, 1);
        assert!(report.assessment.is_none());
        assert!(matches!(
            report.error,
            Some(ProblemError::Math(pse_math::MathError::Domain {
                requirement: "IC original-domain witness",
                ..
            }))
        ));
        assert_eq!(report.evidence.work.iterations, Some(0));
        let mut oracle = IcOracle {
            fail: true,
            ..IcOracle::new()
        };
        let mut r = request();
        r.trial_failures = TrialPolicy::Recoverable;
        let report = initialize_consistent(&mut oracle, &[], &r, execution()).unwrap();
        assert_eq!(report.termination.name, "IDA_FIRST_RES_FAIL");
        assert_eq!(report.initial_residual, Some(false));
        assert!(report.evidence.callback.terminal_failure);
        assert_eq!(
            report.evidence.callback.trial_rejections, 0,
            "failed first residual is not a later Newton trial rejection"
        );
        assert_eq!(oracle.residual_calls, 1);
        assert!(report.assessment.is_none());
        assert!(matches!(
            report.error,
            Some(ProblemError::Math(pse_math::MathError::Domain {
                requirement: "IC original-domain witness",
                ..
            }))
        ));
    }
    #[test]
    fn public_ic_rejects_inert_and_nonfinite_controls_before_callbacks() {
        let mut oracle = IcOracle::new();
        let mut r = request();
        r.mode = IdasInitialization::SteadyStates;
        r.controls.step_trials = Some(3);
        assert!(matches!(
            initialize_consistent(&mut oracle, &[], &r, execution()),
            Err(ProblemError::Contract(_))
        ));
        r = request();
        r.controls.line_search = false;
        r.controls.backtracks = Some(3);
        assert!(matches!(
            initialize_consistent(&mut oracle, &[], &r, execution()),
            Err(ProblemError::Contract(_))
        ));
        r = request();
        r.controls.line_search = false;
        r.controls.step_tolerance = Some(1e-12);
        assert!(matches!(
            initialize_consistent(&mut oracle, &[], &r, execution()),
            Err(ProblemError::Contract(_))
        ));
        r = request();
        r.toward = r.time;
        assert!(matches!(
            initialize_consistent(&mut oracle, &[], &r, execution()),
            Err(ProblemError::Contract(_))
        ));
        let mut e = execution();
        e.memory = Some(1);
        assert!(matches!(
            initialize_consistent(&mut oracle, &[], &request(), e),
            Err(ProblemError::Limit {
                kind: crate::LimitKind::Memory,
                ..
            })
        ));
        assert_eq!(oracle.residual_calls, 0);
        let mut constrained = IcOracle::new();
        constrained.contract.signs = vec![StateSign::NonNegative; 2];
        r = request();
        r.controls.line_search = false;
        r.controls.step_tolerance = Some(1e-12);
        let report = initialize_consistent(&mut constrained, &[], &r, execution()).unwrap();
        assert_eq!(
            report.termination.name, "IDA_SUCCESS",
            "an explicit constraint step floor still acts without line search"
        );
        let e = execution();
        e.cancellation_control().cancel();
        assert!(matches!(
            initialize_consistent(&mut oracle, &[], &request(), e),
            Err(ProblemError::Cancelled)
        ));
        assert_eq!(oracle.residual_calls, 0);
    }
}

#[cfg(test)]
mod checkpoint_owner_tests {
    use super::*;

    pub(super) struct NativeForward {
        pub(super) context: ffi::SUNContext,
        pub(super) memory: *mut c_void,
        pub(super) state: ffi::N_Vector,
        pub(super) rate: ffi::N_Vector,
        matrix: ffi::SUNMatrix,
        linear: ffi::SUNLinearSolver,
    }
    /// Read one entry from a live one-entry serial vector.
    unsafe fn scalar(vector: ffi::N_Vector) -> f64 {
        // SAFETY: the caller supplies a live serial vector with one scalar.
        let pointer = unsafe { ffi::N_VGetArrayPointer(vector) };
        // SAFETY: that vector owns this initialized scalar until the call returns.
        unsafe { *pointer }
    }
    /// Write one entry in an exclusively borrowed one-entry serial vector.
    unsafe fn set_scalar(vector: ffi::N_Vector, value: f64) {
        // SAFETY: the caller supplies a live, exclusively owned serial vector.
        let pointer = unsafe { ffi::N_VGetArrayPointer(vector) };
        // SAFETY: its one scalar is writable and no other access overlaps.
        unsafe {
            *pointer = value;
        }
    }
    unsafe extern "C" fn decay(
        _time: f64,
        state: ffi::N_Vector,
        rate: ffi::N_Vector,
        residual: ffi::N_Vector,
        _data: *mut c_void,
    ) -> i32 {
        // SAFETY: IDAS supplies the live one-entry state vector of this fixture.
        let state = unsafe { scalar(state) };
        // SAFETY: IDAS supplies the live one-entry rate vector of this fixture.
        let rate = unsafe { scalar(rate) };
        // SAFETY: IDAS supplies exclusive writable residual storage of the same size.
        unsafe {
            set_scalar(residual, rate + state);
        }
        0
    }
    impl NativeForward {
        fn new(spacing: c_long) -> Self {
            Self::new_with(spacing, &[1.], &[-1.], decay)
        }
        pub(super) fn new_with(
            spacing: c_long,
            values: &[f64],
            rates: &[f64],
            residual: unsafe extern "C" fn(
                f64,
                ffi::N_Vector,
                ffi::N_Vector,
                ffi::N_Vector,
                *mut c_void,
            ) -> i32,
        ) -> Self {
            let n = index(values.len()).unwrap();
            assert_eq!(values.len(), rates.len());
            let mut context = std::ptr::null_mut();
            // SAFETY: the local output is writable and the serial fixture uses no MPI.
            let context_status =
                unsafe { ffi::SUNContext_Create(ffi::comm_no_mpi(), &raw mut context) };
            assert_eq!(context_status, 0);
            // SAFETY: context is live and the requested extent matches values/rates.
            let state = unsafe { ffi::N_VNew_Serial(n, context) };
            // SAFETY: the same live context owns the matching rate vector.
            let rate = unsafe { ffi::N_VNew_Serial(n, context) };
            assert!(!state.is_null() && !rate.is_null());
            // SAFETY: state has values.len() entries and is exclusively owned here.
            unsafe {
                write(state, values);
            }
            // SAFETY: rate has rates.len() entries and is exclusively owned here.
            unsafe {
                write(rate, rates);
            }
            // SAFETY: this fixture exclusively owns the initialized context.
            let memory = unsafe { ffi::IDACreate(context) };
            // SAFETY: an n-by-n matrix matches both owned vectors.
            let matrix = unsafe { ffi::SUNDenseMatrix(n, n, context) };
            assert!(!memory.is_null() && !matrix.is_null());
            // SAFETY: state, matrix and context are live and dimensionally compatible.
            let linear = unsafe { ffi::SUNLinSol_Dense(state, matrix, context) };
            let owner = Self {
                context,
                memory,
                state,
                rate,
                matrix,
                linear,
            };
            assert!(!linear.is_null());
            // SAFETY: the supplied residual uses the matching vectors through final teardown.
            let init_status = unsafe { ffi::IDAInit(memory, Some(residual), 0., state, rate) };
            assert_eq!(init_status, 0);
            // SAFETY: initialized memory receives finite positive scalar tolerances.
            assert_eq!(unsafe { ffi::IDASStolerances(memory, 0.001, 0.001) }, 0);
            // SAFETY: the solver and matrix share the memory's context and dimensions.
            let linear_status = unsafe { ffi::IDASetLinearSolver(memory, linear, matrix) };
            assert_eq!(linear_status, 0);
            // SAFETY: initialized private memory receives the supplied positive spacing.
            let adjoint_status = unsafe { ffi::IDAAdjInit(memory, spacing, ffi::IDA_HERMITE) };
            assert_eq!(adjoint_status, 0);
            owner
        }
        fn limit(&mut self, maximum: c_long) -> i32 {
            // SAFETY: the owner retains initialized adjoint memory on this thread.
            unsafe { ffi::IDASetCheckpointLimit(self.memory, maximum) }
        }
        fn usage(&self) -> (c_long, c_long, i32) {
            let (mut current, mut peak, mut refused) = (0, 0, 0);
            // SAFETY: live owned memory and independent initialized output storage.
            let usage_status = unsafe {
                ffi::IDAGetCheckpointUsage(
                    self.memory,
                    &raw mut current,
                    &raw mut peak,
                    &raw mut refused,
                )
            };
            assert_eq!(usage_status, 0);
            (current, peak, refused)
        }
        pub(super) fn solve(&mut self, target: f64, task: i32) -> (i32, f64, f64) {
            let (mut time, mut stored) = (0., 0);
            // SAFETY: live native memory, matching state/rate vectors and local output.
            let flag = unsafe {
                ffi::IDASolveF(
                    self.memory,
                    target,
                    &raw mut time,
                    self.state,
                    self.rate,
                    task,
                    &raw mut stored,
                )
            };
            // SAFETY: this owner still retains the one-entry state after native return.
            let state = unsafe { scalar(self.state) };
            (flag, time, state)
        }
    }
    impl Drop for NativeForward {
        fn drop(&mut self) {
            // SAFETY: this owner frees its private IDAS consumer before its inputs.
            unsafe {
                ffi::IDAFree(&raw mut self.memory);
            }
            // SAFETY: the private solver has no remaining IDAS consumer.
            unsafe {
                ffi::SUNLinSolFree(self.linear);
            }
            // SAFETY: the private matrix has no remaining native consumer.
            unsafe {
                ffi::SUNMatDestroy(self.matrix);
            }
            // SAFETY: the private state vector is released exactly once.
            unsafe {
                ffi::N_VDestroy(self.state);
            }
            // SAFETY: the private rate vector is released exactly once.
            unsafe {
                ffi::N_VDestroy(self.rate);
            }
            // SAFETY: all objects belonging to this private context are now freed.
            unsafe {
                ffi::SUNContext_Free(&raw mut self.context);
            }
        }
    }
    #[test]
    fn checkpoint_owner_guards_initial_and_next_allocation_and_latches_refusal() {
        let mut native = NativeForward::new(1);
        assert_eq!(native.limit(-1), ffi::IDA_ILL_INPUT);
        assert_eq!(native.limit(0), 0);
        assert_eq!(native.solve(1., ffi::IDA_NORMAL).0, ffi::IDA_MEM_FAIL);
        assert_eq!(native.usage(), (0, 0, 1), "initial allocation was refused");
        assert_eq!(native.limit(2), 0);
        assert_eq!(native.solve(1., ffi::IDA_NORMAL).0, ffi::IDA_MEM_FAIL);
        assert_eq!(
            native.usage(),
            (0, 0, 1),
            "larger limit cannot repair a failed attempt"
        );
        // SAFETY: this owning thread reinitializes its live adjoint module.
        unsafe {
            assert_eq!(ffi::IDAAdjReInit(native.memory), 0);
        }
        assert_eq!(native.usage(), (0, 0, 0));
        assert_eq!(native.solve(1., ffi::IDA_ONE_STEP).0, ffi::IDA_SUCCESS);
        assert_eq!(
            native.usage(),
            (2, 2, 0),
            "initial plus exactly one new checkpoint"
        );
        assert_eq!(native.limit(1), ffi::IDA_ILL_INPUT);
        assert_eq!(native.solve(1., ffi::IDA_NORMAL).0, ffi::IDA_MEM_FAIL);
        assert_eq!(
            native.usage(),
            (2, 2, 1),
            "no excess checkpoint ever became resident"
        );
        // SAFETY: this owner exclusively retains its live one-entry state.
        unsafe {
            set_scalar(native.state, 1.);
        }
        // SAFETY: this owner exclusively retains its live one-entry rate.
        unsafe {
            set_scalar(native.rate, -1.);
        }
        // SAFETY: reset private memory with its matching owned vectors.
        let reset_status = unsafe { ffi::IDAReInit(native.memory, 0., native.state, native.rate) };
        assert_eq!(reset_status, 0);
        // SAFETY: release the checkpoint population on its sole owning thread.
        assert_eq!(unsafe { ffi::IDAAdjReInit(native.memory) }, 0);
        assert_eq!(
            native.usage(),
            (0, 2, 0),
            "reset preserves lifetime peak and installed limit"
        );
        assert_eq!(native.solve(1., ffi::IDA_NORMAL).0, ffi::IDA_MEM_FAIL);
        assert_eq!(native.usage(), (2, 2, 1));
    }
    #[test]
    fn checkpoint_owner_preserves_normal_interpolation_for_sparse_and_dense_outputs() {
        let mut finals = Vec::new();
        for outputs in [vec![0.3], vec![0.05, 0.1, 0.15, 0.2, 0.3]] {
            let mut observations = Vec::new();
            for bounded in [false, true] {
                let mut native = NativeForward::new(100_000);
                if bounded {
                    assert_eq!(native.limit(2), 0);
                }
                // SAFETY: finite physical stop on live owned IDAS memory.
                unsafe {
                    assert_eq!(ffi::IDASetStopTime(native.memory, 0.3), 0);
                }
                let mut values = Vec::new();
                for &target in &outputs {
                    let (flag, time, value) = native.solve(target, ffi::IDA_NORMAL);
                    assert!(flag == ffi::IDA_SUCCESS || flag == ffi::IDA_TSTOP_RETURN);
                    assert_eq!(time, target);
                    assert!((value - (-target).exp()).abs() < 0.001);
                    assert_eq!(native.usage(), (1, 1, 0));
                    values.push((flag, time, value));
                }
                observations.push(values);
            }
            assert_eq!(
                observations[0], observations[1],
                "population permission does not change native output/history semantics"
            );
            finals.push(observations[0].last().unwrap().2);
        }
        // The first requested output influences IDAS's own initial-step estimate;
        // different output schedules must agree physically, not bit-for-bit.
        assert!((finals[0] - finals[1]).abs() < 0.001);
    }
}
