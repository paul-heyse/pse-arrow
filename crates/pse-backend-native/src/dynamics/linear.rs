// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
#![allow(
    unsafe_code,
    reason = "owned SuiteSparse KLU common, symbolic and numeric objects over checked CSC copies"
)]
//! Newton linear solvers of the Diffsol schemes that can fail (Plan 22 I9). Diffsol's own
//! faer and KLU solvers panic when a factorization fails, because
//! `LinearSolver::set_linearisation` is infallible. These solvers keep the failure and
//! return it from `solve_in_place`. Diffsol then counts a nonlinear-solver failure and
//! reduces the step, which is its own recovery, and a final failure reaches the caller as
//! a typed numerical error. A solve that yields a nonfinite value is also refused: a
//! numerically zero pivot is accepted by partial pivoting and shows only in the solution.
use diffsol::{
    FaerContext, FaerSparseMat, FaerVec, LaError, Matrix, MatrixCommon, Vector, VectorHost,
};
use diffsol_la::{LinearOp, LinearSolver, error::LinearSolverError};
use faer::{
    Conj, Par,
    dyn_stack::{MemBuffer, MemStack},
    sparse::linalg::lu::{self, LuRef, NumericLu, SymbolicLu},
};
use std::cell::{RefCell, UnsafeCell};
use suitesparse_sys as klu;

type M = FaerSparseMat<f64>;
type V = FaerVec<f64>;

/// A factorization, or why the last linearization cannot be solved.
type Factor<F> = Result<F, LinearSolverError>;

fn unset<F>() -> Factor<F> {
    Err(LinearSolverError::LinearSolverNotSetup)
}
/// A square matrix with the operator's declared pattern, or why there is none.
fn pattern<C: LinearOp<T = f64, V = V, M = M, C = FaerContext>>(
    op: &C,
) -> Result<M, LinearSolverError> {
    if op.nrows() != op.ncols() {
        return Err(LinearSolverError::LinearSolverMatrixNotSquare);
    }
    let sparsity = op
        .sparsity()
        .ok_or_else(|| LinearSolverError::Other("sparse Newton matrix without a pattern".into()))?;
    if sparsity.nrows() != op.nrows() || sparsity.ncols() != op.ncols() {
        return Err(LinearSolverError::LinearSolverMatrixVectorNotCompatible);
    }
    Ok(M::new_from_sparsity(
        op.nrows(),
        op.ncols(),
        Some(sparsity),
        *op.context(),
    ))
}
/// Refuse a nonfinite solution of a numerically singular factor.
fn finite(x: &V) -> Result<(), LaError> {
    if x.as_slice().iter().all(|v| v.is_finite()) {
        Ok(())
    } else {
        Err(LinearSolverError::LuSolveFailed.into())
    }
}

/// faer sparse LU with partial pivoting.
pub(super) struct FaerLu {
    matrix: Option<M>,
    symbolic: Factor<SymbolicLu<usize>>,
    numeric: NumericLu<usize, f64>,
    scratch: Option<RefCell<MemBuffer>>,
    ready: Factor<()>,
    parallel: Par,
}
impl Default for FaerLu {
    fn default() -> Self {
        Self {
            matrix: None,
            symbolic: unset(),
            numeric: NumericLu::new(),
            scratch: None,
            ready: unset(),
            parallel: Par::Seq,
        }
    }
}
impl LinearSolver<M> for FaerLu {
    fn set_sparsity<C: LinearOp<T = f64, V = V, M = M, C = FaerContext>>(&mut self, op: &C) {
        crate::observation::boolean("diffsol.faer.factor.valid", false);
        self.ready = unset();
        self.numeric = NumericLu::new();
        self.scratch = None;
        // The integrator admits this context's threads; a process-global default may
        // create an unadmitted team alongside independent trajectory workers.
        self.parallel = op.context().par;
        self.symbolic = match pattern(op) {
            Ok(matrix) => {
                let result = crate::observation::scoped("diffsol.faer.symbolic", || {
                    lu::factorize_symbolic_lu(matrix.inner().symbolic(), Default::default())
                })
                .map_err(|e| LinearSolverError::Other(format!("faer symbolic sparse LU: {e:?}")))
                .and_then(|symbolic| {
                    let request = symbolic
                        .factorize_numeric_lu_scratch::<f64>(self.parallel, Default::default())
                        .or(symbolic.solve_in_place_scratch::<f64>(1, self.parallel));
                    let scratch = MemBuffer::try_new(request).map_err(|e| {
                        LinearSolverError::Other(format!("faer sparse LU scratch: {e:?}"))
                    })?;
                    crate::conditioning::publish_faer_storage(
                        "diffsol.faer.storage",
                        matrix.inner().val().len(),
                        request.size_bytes(),
                    );
                    self.scratch = Some(RefCell::new(scratch));
                    Ok(symbolic)
                });
                self.matrix = Some(matrix);
                result
            }
            Err(error) => {
                self.matrix = None;
                Err(error)
            }
        };
    }
    fn set_linearisation<C: LinearOp<T = f64, V = V, M = M, C = FaerContext>>(&mut self, op: &C) {
        crate::observation::boolean("diffsol.faer.factor.valid", false);
        self.ready = match (&self.symbolic, self.matrix.as_mut(), self.scratch.as_mut()) {
            (Ok(symbolic), Some(matrix), Some(scratch)) => {
                let same_pattern = op.sparsity().is_some_and(|pattern| {
                    let current = matrix.inner().symbolic();
                    pattern.nrows() == current.nrows()
                        && pattern.ncols() == current.ncols()
                        && pattern.col_ptr() == current.col_ptr()
                        && pattern.row_idx() == current.row_idx()
                });
                if !same_pattern {
                    return self.ready = Err(LinearSolverError::Other(
                        "Newton matrix pattern changed".into(),
                    ));
                }
                op.matrix_inplace(matrix);
                if matrix.inner().val().iter().any(|value| !value.is_finite()) {
                    Err(LinearSolverError::LuSolveFailed)
                } else {
                    let result = crate::observation::scoped("diffsol.faer.factor", || {
                        symbolic
                            .factorize_numeric_lu(
                                &mut self.numeric,
                                matrix.inner().as_ref(),
                                self.parallel,
                                MemStack::new(scratch.get_mut()),
                                Default::default(),
                            )
                            .map(|_| ())
                            .map_err(|e| LinearSolverError::Other(format!("faer sparse LU: {e:?}")))
                    });
                    crate::observation::count(
                        if result.is_ok() {
                            "diffsol.faer.factor.succeeded"
                        } else {
                            "diffsol.faer.factor.failed"
                        },
                        1,
                    );
                    result
                }
            }
            (Err(error), _, _) => Err(error.clone()),
            _ => unset(),
        };
        crate::observation::boolean("diffsol.faer.factor.valid", self.ready.is_ok());
    }
    fn solve_in_place(&self, x: &mut V) -> Result<(), LaError> {
        self.ready.as_ref().map_err(|e| LaError::from(e.clone()))?;
        let symbolic = self
            .symbolic
            .as_ref()
            .map_err(|e| LaError::from(e.clone()))?;
        let scratch = self
            .scratch
            .as_ref()
            .ok_or(LinearSolverError::LinearSolverNotSetup)?;
        let mut scratch = scratch.borrow_mut();
        let n = x.len();
        crate::observation::scoped("diffsol.faer.solve", || {
            LuRef::new_unchecked(symbolic, &self.numeric).solve_in_place_with_conj(
                Conj::No,
                faer::MatMut::from_column_major_slice_mut(x.as_mut_slice(), n, 1),
                self.parallel,
                MemStack::new(&mut scratch),
            )
        });
        let result = finite(x);
        crate::observation::count(
            if result.is_ok() {
                "diffsol.faer.solve.succeeded"
            } else {
                "diffsol.faer.solve.failed"
            },
            1,
        );
        result
    }
}

/// SuiteSparse KLU over 64-bit index copies of the Newton pattern. The common object is
/// boxed, so the pointer KLU keeps in its factors stays valid when the solver moves.
pub(super) struct Klu {
    common: Box<UnsafeCell<klu::klu_l_common>>,
    matrix: Option<M>,
    columns: Vec<i64>,
    rows: Vec<i64>,
    symbolic: *mut klu::klu_l_symbolic,
    numeric: *mut klu::klu_l_numeric,
    failure: LinearSolverError,
    #[cfg(test)]
    full_factors: usize,
}
impl Default for Klu {
    fn default() -> Self {
        let common = Box::new(UnsafeCell::new(klu::klu_l_common::default()));
        // SAFETY: the boxed common object is writable and outlives every KLU call. KLU's
        // defaults halt a factorization at an exactly zero pivot.
        unsafe { klu::klu_l_defaults(common.get()) };
        Self {
            common,
            matrix: None,
            columns: Vec::new(),
            rows: Vec::new(),
            symbolic: std::ptr::null_mut(),
            numeric: std::ptr::null_mut(),
            failure: LinearSolverError::LinearSolverNotSetup,
            #[cfg(test)]
            full_factors: 0,
        }
    }
}
impl Klu {
    /// Copy only initialized return fields while this thread exclusively owns KLU.
    /// Allocator peaks cover this common object's lifetime, including old factors.
    fn publish_return(&self, phase: &str) {
        // SAFETY: no KLU call is active; defaults initializes these common fields.
        let common = unsafe { &*self.common.get() };
        crate::observation::integer(
            &format!("diffsol.klu.{phase}.status"),
            "native_status",
            i64::from(common.status),
        );
        crate::observation::integer(
            "diffsol.klu.allocator_lifetime.current_bytes",
            "bytes",
            i64::try_from(common.memusage).unwrap_or(i64::MAX),
        );
        crate::observation::integer(
            "diffsol.klu.allocator_lifetime.peak_bytes",
            "bytes",
            i64::try_from(common.mempeak).unwrap_or(i64::MAX),
        );
        for (name, value, unit) in [
            ("structural_rank", common.structural_rank, "rank"),
            (
                "numerical_rank",
                common.numerical_rank,
                "native_pivot_position",
            ),
        ] {
            let current = if name == "structural_rank" {
                !self.symbolic.is_null()
            } else {
                phase != "symbolic"
                    && (common.status == 0 || u32::try_from(common.status) == Ok(klu::KLU_SINGULAR))
            };
            if !current {
                crate::observation::unavailable(
                    &format!("diffsol.klu.{name}"),
                    "this phase did not establish a current rank/pivot result",
                );
            } else if value >= 0 {
                crate::observation::integer(&format!("diffsol.klu.{name}"), unit, value);
            } else {
                crate::observation::unavailable(
                    &format!("diffsol.klu.{name}"),
                    "native -1 means rank was not computed",
                );
            }
        }
        if phase != "symbolic" && common.status == 0 && !self.numeric.is_null() {
            // SAFETY: the successful owner return established this live numeric
            // object's stored L/U/off-diagonal entry counts; no KLU call is active.
            let numeric = unsafe { &*self.numeric };
            for (name, value) in [
                ("l_entries", numeric.lnz),
                ("u_entries", numeric.unz),
                ("off_diagonal_entries", numeric.nzoff),
            ] {
                crate::observation::integer(
                    &format!("diffsol.klu.factor.{name}"),
                    "entries",
                    value,
                );
            }
        } else {
            for name in ["l_entries", "u_entries", "off_diagonal_entries"] {
                crate::observation::unavailable(
                    &format!("diffsol.klu.factor.{name}"),
                    "this return did not establish current numeric storage",
                );
            }
        }
    }
    /// Reuse numeric storage only while KLU's own conditioning diagnostics accept it.
    /// This is the bundled SUNDIALS 7.1.1 KLU setup policy, with failed diagnostics
    /// also routed to a fresh factor rather than leaving a partly updated factor live.
    fn refactor(&mut self, matrix: &mut M) -> bool {
        if self.numeric.is_null() {
            return false;
        }
        let threshold = f64::EPSILON.powf(2.0 / 3.0);
        // SAFETY: full CSC identity was checked before this call. The owned numeric,
        // symbolic and common objects are live and exclusively accessed for the update.
        let refactored = crate::observation::scoped("diffsol.klu.refactor", || unsafe {
            klu::klu_l_refactor(
                self.columns.as_mut_ptr(),
                self.rows.as_mut_ptr(),
                matrix.inner_mut().val_mut().as_mut_ptr(),
                self.symbolic,
                self.numeric,
                self.common.get(),
            )
        });
        if refactored != 1 {
            self.publish_return("refactor");
            crate::observation::count("diffsol.klu.refactor.failed", 1);
            return false;
        }
        crate::observation::count("diffsol.klu.refactor.succeeded", 1);
        // SAFETY: the successful refactor retained the live symbolic and numeric objects;
        // the boxed common object remains exclusively accessed by this solver.
        if unsafe { klu::klu_l_rcond(self.symbolic, self.numeric, self.common.get()) } != 1 {
            crate::observation::count("diffsol.klu.refactor.condition_diagnostic_failed", 1);
            return false;
        }
        // SAFETY: KLU initialized this field in the successful rcond call above.
        let rcond = unsafe { (*self.common.get()).rcond };
        crate::observation::gauge("diffsol.klu.rcond", "ratio", rcond);
        if !rcond.is_finite() || rcond < 0.0 {
            crate::observation::count("diffsol.klu.refactor.conditioning_rejected", 1);
            return false;
        }
        if rcond < threshold {
            // SAFETY: the checked CSC pattern and its owned values still match the live
            // symbolic and numeric factors, and common is exclusively accessed.
            let estimated = unsafe {
                klu::klu_l_condest(
                    self.columns.as_mut_ptr(),
                    matrix.inner_mut().val_mut().as_mut_ptr(),
                    self.symbolic,
                    self.numeric,
                    self.common.get(),
                )
            };
            if estimated != 1 {
                crate::observation::count("diffsol.klu.refactor.condition_diagnostic_failed", 1);
                return false;
            }
            // SAFETY: KLU initialized this field in the successful condest call above.
            let condition = unsafe { (*self.common.get()).condest };
            crate::observation::gauge("diffsol.klu.condest", "ratio", condition);
            let accepted = condition.is_finite() && condition > 0.0 && condition <= 1.0 / threshold;
            if !accepted {
                crate::observation::count("diffsol.klu.refactor.conditioning_rejected", 1);
            }
            self.publish_return("refactor");
            return accepted;
        }
        self.publish_return("refactor");
        true
    }
    fn free_numeric(&mut self) {
        if !self.numeric.is_null() {
            // SAFETY: the numeric object was created with this common object and is freed once.
            unsafe { klu::klu_l_free_numeric(&mut self.numeric, self.common.get()) };
        }
        self.numeric = std::ptr::null_mut();
    }
    fn free_symbolic(&mut self) {
        self.free_numeric();
        if !self.symbolic.is_null() {
            // SAFETY: the symbolic object was created with this common object and is freed once.
            unsafe { klu::klu_l_free_symbolic(&mut self.symbolic, self.common.get()) };
        }
        self.symbolic = std::ptr::null_mut();
    }
    fn analyze(&mut self, matrix: &M) -> Result<(), LinearSolverError> {
        let index = |v: &usize| {
            i64::try_from(*v).map_err(|_| LinearSolverError::Other("KLU index overflow".into()))
        };
        let pattern = matrix.inner().symbolic();
        self.columns = pattern
            .col_ptr()
            .iter()
            .map(index)
            .collect::<Result<_, _>>()?;
        self.rows = pattern
            .row_idx()
            .iter()
            .map(index)
            .collect::<Result<_, _>>()?;
        let n = index(&matrix.nrows())?;
        // SAFETY: the column pointers (n + 1) and row indices describe the n × n pattern
        // and outlive the symbolic object; KLU only reads them.
        self.symbolic = crate::observation::scoped("diffsol.klu.symbolic", || unsafe {
            klu::klu_l_analyze(
                n,
                self.columns.as_ptr(),
                self.rows.as_ptr(),
                self.common.get(),
            )
        });
        self.publish_return("symbolic");
        if self.symbolic.is_null() {
            return Err(LinearSolverError::KluFailedToAnalyze);
        }
        Ok(())
    }
    fn status(&self) -> i32 {
        // SAFETY: no KLU call is in progress; the common object is only read.
        unsafe { (*self.common.get()).status }
    }
}
impl Drop for Klu {
    fn drop(&mut self) {
        self.free_symbolic();
    }
}
impl LinearSolver<M> for Klu {
    fn set_sparsity<C: LinearOp<T = f64, V = V, M = M, C = FaerContext>>(&mut self, op: &C) {
        self.free_symbolic();
        self.matrix = None;
        self.failure = LinearSolverError::LinearSolverNotSetup;
        let analyzed = pattern(op).and_then(|matrix| {
            self.analyze(&matrix)?;
            Ok(matrix)
        });
        match analyzed {
            Ok(matrix) => self.matrix = Some(matrix),
            Err(e) => self.failure = e,
        }
    }
    fn set_linearisation<C: LinearOp<T = f64, V = V, M = M, C = FaerContext>>(&mut self, op: &C) {
        crate::observation::unavailable(
            "diffsol.klu.rcond",
            "not calculated for current numeric attempt",
        );
        crate::observation::unavailable(
            "diffsol.klu.condest",
            "not calculated for current numeric attempt",
        );
        crate::observation::unavailable(
            "diffsol.klu.numerical_rank",
            "current numeric attempt has not established a pivot result",
        );
        crate::observation::boolean("diffsol.klu.factor.valid", false);
        for name in ["l_entries", "u_entries", "off_diagonal_entries"] {
            crate::observation::unavailable(
                &format!("diffsol.klu.factor.{name}"),
                "current numeric attempt has not established storage",
            );
        }
        let Some(mut matrix) = self.matrix.take() else {
            return;
        };
        let same_pattern = op.sparsity().is_some_and(|pattern| {
            let current = matrix.inner().symbolic();
            pattern.nrows() == current.nrows()
                && pattern.ncols() == current.ncols()
                && pattern.col_ptr() == current.col_ptr()
                && pattern.row_idx() == current.row_idx()
        });
        if !same_pattern {
            self.free_numeric();
            self.failure = LinearSolverError::Other("Newton matrix pattern changed".into());
        } else {
            op.matrix_inplace(&mut matrix);
            if matrix.inner().val().iter().any(|value| !value.is_finite()) {
                self.free_numeric();
                self.failure = LinearSolverError::LuSolveFailed;
                self.matrix = Some(matrix);
                return;
            }
            let had_numeric = !self.numeric.is_null();
            if self.refactor(&mut matrix) {
                crate::observation::boolean("diffsol.klu.factor.valid", true);
                self.matrix = Some(matrix);
                return;
            }
            if had_numeric {
                crate::observation::count("diffsol.klu.refactor.rejected_full_fallback", 1);
            }
            self.free_numeric();
            #[cfg(test)]
            {
                self.full_factors += 1;
            }
            // SAFETY: the values follow the analyzed pattern, whose copies outlive the call;
            // the symbolic object and the boxed common object are live.
            self.numeric = crate::observation::scoped("diffsol.klu.factor", || unsafe {
                klu::klu_l_factor(
                    self.columns.as_ptr(),
                    self.rows.as_ptr(),
                    matrix.inner().val().as_ptr(),
                    self.symbolic,
                    self.common.get(),
                )
            });
            crate::observation::count(
                if self.numeric.is_null() {
                    "diffsol.klu.factor.failed"
                } else {
                    "diffsol.klu.factor.succeeded"
                },
                1,
            );
            self.publish_return("factor");
            crate::observation::boolean("diffsol.klu.factor.valid", !self.numeric.is_null());
            if self.numeric.is_null() {
                self.failure = match u32::try_from(self.status()) {
                    Ok(klu::KLU_SINGULAR) => LinearSolverError::Other(
                        "KLU factorization found an exactly zero pivot".into(),
                    ),
                    _ => LinearSolverError::KluFailedToFactorize,
                };
            }
        }
        self.matrix = Some(matrix);
    }
    fn solve_in_place(&self, x: &mut V) -> Result<(), LaError> {
        if self.numeric.is_null() {
            return Err(self.failure.clone().into());
        }
        let n = i64::try_from(x.len()).map_err(|_| LinearSolverError::LuSolveFailed)?;
        if Some(x.len()) != self.matrix.as_ref().map(M::nrows) {
            return Err(LinearSolverError::LinearSolverMatrixVectorNotCompatible.into());
        }
        // SAFETY: `x` holds n contiguous values; the factors and the boxed common object are
        // live, and no other KLU call on them is in progress.
        let solved = crate::observation::scoped("diffsol.klu.solve", || unsafe {
            klu::klu_l_solve(
                self.symbolic,
                self.numeric,
                n,
                1,
                x.as_mut_slice().as_mut_ptr(),
                self.common.get(),
            )
        });
        self.publish_return("solve");
        if solved != 1 {
            return Err(LinearSolverError::LuSolveFailed.into());
        }
        finite(x)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    /// A fixed 2 × 2 Newton matrix with a full pattern, explicit zeros included.
    struct Fixed(faer::sparse::SparseColMat<usize, f64>, FaerContext);
    impl LinearOp for Fixed {
        type T = f64;
        type V = V;
        type M = M;
        type C = FaerContext;
        fn nrows(&self) -> usize {
            2
        }
        fn ncols(&self) -> usize {
            2
        }
        fn context(&self) -> &FaerContext {
            &self.1
        }
        fn matrix_inplace(&self, y: &mut M) {
            y.inner_mut().val_mut().copy_from_slice(self.0.val());
        }
        fn sparsity(&self) -> Option<faer::sparse::SymbolicSparseColMat<usize>> {
            Some(self.0.symbolic().to_owned().unwrap())
        }
    }
    /// The native factor uses the operator's admitted context, not ambient globals.
    #[test]
    fn faer_factor_consumes_serial_integrator_context() {
        use pse_diagnostics::observation::{ObservationHub, Policy, Value};
        let hub = ObservationHub::new(Policy::default());
        let operation = hub.start("faer-scratch-extent");
        let _entered = operation.context().enter();
        let context = FaerContext { par: Par::Seq };
        let matrix = faer::sparse::SparseColMat::try_new_from_triplets(
            2,
            2,
            &[
                faer::sparse::Triplet::new(0, 0, 2.0),
                faer::sparse::Triplet::new(1, 1, 4.0),
            ],
        )
        .unwrap();
        let op = Fixed(matrix, context);
        let mut solver = FaerLu::default();
        solver.set_sparsity(&op);
        assert!(matches!(solver.parallel, Par::Seq));
        solver.set_linearisation(&op);
        let mut x = V::from_vec(vec![2.0, 8.0], context);
        solver.solve_in_place(&mut x).unwrap();
        assert_eq!(x.as_slice(), &[1.0, 2.0]);
        let snapshot = operation
            .reader()
            .snapshot(std::time::Duration::from_secs(60));
        assert_eq!(
            snapshot
                .metrics
                .iter()
                .find(|metric| metric.name == "diffsol.faer.storage.matrix_entries")
                .unwrap()
                .latest,
            Value::Integer(2)
        );
        assert!(
            matches!(snapshot.metrics.iter().find(|metric| metric.name == "diffsol.faer.storage.scratch_requested_bytes").unwrap().latest, Value::Integer(value) if value > 0)
        );
        assert!(matches!(
            snapshot
                .metrics
                .iter()
                .find(|metric| metric.name == "diffsol.faer.storage.numeric_fill")
                .unwrap()
                .latest,
            Value::Unavailable(_)
        ));
    }

    /// Solve with column-major values `[a00, a10, a01, a11]`.
    fn solve<S: LinearSolver<M>>(values: [f64; 4]) -> Result<Vec<f64>, LaError> {
        let triplets = [(0, 0), (1, 0), (0, 1), (1, 1)]
            .iter()
            .zip(values)
            .map(|(&(r, c), v)| faer::sparse::Triplet::new(r, c, v))
            .collect::<Vec<_>>();
        let context = FaerContext::default();
        let op = Fixed(
            faer::sparse::SparseColMat::try_new_from_triplets(2, 2, &triplets).unwrap(),
            context,
        );
        assert_eq!(op.0.val().len(), 4);
        let mut solver = S::default();
        solver.set_sparsity(&op);
        solver.set_linearisation(&op);
        let mut x = V::from_vec(vec![1.0, 2.0], context);
        solver.solve_in_place(&mut x)?;
        Ok(x.as_slice().to_vec())
    }
    #[test]
    fn faer_numeric_refresh_keeps_scratch_and_recovers_from_failed_current_matrix() {
        let context = FaerContext::default();
        let matrix = faer::sparse::SparseColMat::try_new_from_triplets(
            2,
            2,
            &[
                faer::sparse::Triplet::new(0, 0, 2.0),
                faer::sparse::Triplet::new(1, 0, 0.0),
                faer::sparse::Triplet::new(0, 1, 1.0),
                faer::sparse::Triplet::new(1, 1, 4.0),
            ],
        )
        .unwrap();
        let mut op = Fixed(matrix, context);
        let mut solver = FaerLu::default();
        solver.set_sparsity(&op);
        let scratch = solver.scratch.as_ref().unwrap().borrow().as_ptr();
        for (values, expected) in [
            ([2.0, 0.0, 1.0, 4.0], [0.25, 0.5]),
            ([4.0, 0.0, 0.0, 2.0], [0.25, 1.0]),
        ] {
            op.0.val_mut().copy_from_slice(&values);
            solver.set_linearisation(&op);
            let mut x = V::from_vec(vec![1.0, 2.0], context);
            solver.solve_in_place(&mut x).unwrap();
            assert_eq!(solver.scratch.as_ref().unwrap().borrow().as_ptr(), scratch);
            for (actual, expected) in x.as_slice().iter().zip(expected) {
                assert!((actual - expected).abs() < 1e-14);
            }
        }
        for values in [[1.0; 4], [f64::NAN, 0.0, 0.0, 2.0]] {
            op.0.val_mut().copy_from_slice(&values);
            solver.set_linearisation(&op);
            let mut x = V::from_vec(vec![1.0, 2.0], context);
            assert!(solver.solve_in_place(&mut x).is_err());
        }
        op.0.val_mut().copy_from_slice(&[4.0, 0.0, 0.0, 2.0]);
        solver.set_linearisation(&op);
        let mut x = V::from_vec(vec![1.0, 2.0], context);
        solver.solve_in_place(&mut x).unwrap();
        assert_eq!(x.as_slice(), [0.25, 1.0]);
        op.0 = faer::sparse::SparseColMat::try_new_from_triplets(
            2,
            2,
            &[
                faer::sparse::Triplet::new(0, 0, 4.0),
                faer::sparse::Triplet::new(1, 1, 2.0),
            ],
        )
        .unwrap();
        solver.set_linearisation(&op);
        assert!(solver.solve_in_place(&mut x).is_err());
        solver.set_sparsity(&op);
        assert!(solver.solve_in_place(&mut x).is_err());
        solver.set_linearisation(&op);
        let mut x = V::from_vec(vec![1.0, 2.0], context);
        solver.solve_in_place(&mut x).unwrap();
        assert_eq!(x.as_slice(), [0.25, 1.0]);
    }

    /// Both factorizations solve a regular system and refuse singular ones, with a
    /// numerically zero pivot behind nonzero entries and with a zero row alike.
    #[test]
    fn singular_newton_matrix_fails_the_solve() {
        for x in [
            solve::<FaerLu>([2.0, 0.0, 1.0, 4.0]),
            solve::<Klu>([2.0, 0.0, 1.0, 4.0]),
        ] {
            let x = x.unwrap();
            assert!(
                (x[0] - 0.25).abs() < 1e-14 && (x[1] - 0.5).abs() < 1e-14,
                "{x:?}"
            );
        }
        for singular in [[1.0, 1.0, 1.0, 1.0], [1.0, 0.0, 1.0, 0.0]] {
            assert!(solve::<FaerLu>(singular).is_err(), "faer {singular:?}");
            assert!(solve::<Klu>(singular).is_err(), "KLU {singular:?}");
        }
    }

    #[test]
    fn klu_refactor_reuses_storage_and_fresh_factor_recovers_changed_pivots() {
        use pse_diagnostics::observation::{ObservationHub, Policy, Value};
        let hub = ObservationHub::new(Policy::default());
        let operation = hub.start("klu-returned-factor-entries");
        let _entered = operation.context().enter();
        let context = FaerContext { par: Par::Seq };
        let triplets = [(0, 0), (1, 0), (0, 1), (1, 1)]
            .into_iter()
            .map(|(r, c)| faer::sparse::Triplet::new(r, c, 0.0))
            .collect::<Vec<_>>();
        let mut op = Fixed(
            faer::sparse::SparseColMat::try_new_from_triplets(2, 2, &triplets).unwrap(),
            context,
        );
        let mut solver = Klu::default();
        solver.set_sparsity(&op);
        op.0.val_mut().copy_from_slice(&[2., 0., 0., 4.]);
        solver.set_linearisation(&op);
        let original = solver.numeric;
        let initial = operation
            .reader()
            .snapshot(std::time::Duration::from_secs(60));
        for name in [
            "diffsol.klu.factor.l_entries",
            "diffsol.klu.factor.u_entries",
        ] {
            assert!(
                matches!(initial.metrics.iter().find(|metric| metric.name == name).unwrap().latest, Value::Integer(value) if value >= 2)
            );
        }
        op.0.val_mut().copy_from_slice(&[4., 0., 0., 2.]);
        solver.set_linearisation(&op);
        assert_eq!(
            solver.numeric, original,
            "accepted refactor retains the numeric owner"
        );
        assert_eq!(solver.full_factors, 1);
        // A zero diagonal in the old ordering requires fresh pivot selection; the
        // new matrix is perfectly regular and its authored x gives an independent RHS.
        for (index, values) in [[0., 2., 3., 0.], [1e-15, 0., 0., 1.], [2., 1., 3., 4.]]
            .into_iter()
            .enumerate()
        {
            op.0.val_mut().copy_from_slice(&values);
            solver.set_linearisation(&op);
            if index < 2 {
                assert_eq!(
                    solver.full_factors,
                    index + 2,
                    "changed pivot/condest rejection must fresh-factor"
                );
            }
            let rhs = [values[0] * 2. - values[2], values[1] * 2. - values[3]];
            let mut x = V::from_vec(rhs.to_vec(), context);
            solver.solve_in_place(&mut x).unwrap();
            assert!((x.as_slice()[0] - 2.).abs() < 1e-12);
            assert!((x.as_slice()[1] + 1.).abs() < 1e-12);
        }
        for invalid in [[1.; 4], [f64::NAN, 0., 0., 1.], [1., 0., 0., f64::INFINITY]] {
            op.0.val_mut().copy_from_slice(&invalid);
            solver.set_linearisation(&op);
            let mut x = V::from_vec(vec![1., 2.], context);
            assert!(solver.solve_in_place(&mut x).is_err());
        }
        op.0.val_mut().copy_from_slice(&[2., 0., 0., 4.]);
        solver.set_linearisation(&op);
        let mut x = V::from_vec(vec![2., 8.], context);
        solver.solve_in_place(&mut x).unwrap();
        assert_eq!(x.as_slice(), &[1., 2.]);
        // Equal nnz does not establish pattern identity: both off-diagonal entries
        // differ from the analyzed diagonal pattern.
        op.0 = faer::sparse::SparseColMat::try_new_from_triplets(
            2,
            2,
            &[
                faer::sparse::Triplet::new(0, 0, 1.),
                faer::sparse::Triplet::new(1, 1, 1.),
            ],
        )
        .unwrap();
        solver.set_sparsity(&op);
        solver.set_linearisation(&op);
        op.0 = faer::sparse::SparseColMat::try_new_from_triplets(
            2,
            2,
            &[
                faer::sparse::Triplet::new(1, 0, 1.),
                faer::sparse::Triplet::new(0, 1, 1.),
            ],
        )
        .unwrap();
        solver.set_linearisation(&op);
        assert!(solver.solve_in_place(&mut x).is_err());
    }
}
