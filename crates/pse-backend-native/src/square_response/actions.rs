// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Fresh sparse response actions. Numerical LU/backward error is estimated evidence,
//! not a numerical-rank certificate or a forward-error bound on a nonlinear root.
use super::{SquareScope, Withheld};
use crate::{
    ProblemError,
    quality::Tolerances,
    solve::{Execution, NativeStorageScope, WorkEvidence},
};
use faer::{
    Accum, Conj, Mat, Par,
    dyn_stack::{MemBuffer, MemStack},
    sparse::{
        SparseColMat, SparseColMatRef,
        linalg::{
            lu::{self, LuRef, NumericLu, SymbolicLu},
            matmul::sparse_dense_matmul,
        },
    },
};
use pse_ids::{ContentHash, SemanticId};
use pse_math::normalization::Normalization;
use pse_model::strategy::SemanticProductKey;
use std::sync::Arc;

/// Inputs actually consumed by a fresh response factor at a physically feasible root.
#[derive(Debug)]
pub struct SparseRequest<'a> {
    /// Complete original square closure and matching.
    pub scope: &'a SquareScope,
    /// Original coordinate values at the fresh derivative evaluation.
    pub point: &'a [f64],
    /// Independently observed original row values at the same point.
    pub values: &'a [f64],
    /// Original physical budgets.
    pub tolerances: &'a Tolerances,
    /// Positive physical state/row scales.
    pub normalization: &'a Normalization,
    /// Producer key, including the actual point, parameter, source/order and normalization.
    pub key: SemanticProductKey,
    /// Finite opaque factor reservation and explicit scratch allowance. This is neither
    /// a complete producer-owned fill bound nor measured allocation bytes.
    pub bytes: usize,
}
/// Evidence of the performed sparse action, in scaled coordinates.
#[derive(Clone, Copy, Debug)]
pub struct ActionEvidence {
    /// Normwise residual backward error; an estimate, never a forward certificate.
    pub backward_error: f64,
    /// Required numerical backward-error limit.
    pub limit: f64,
    /// Actual backsolves used, including bounded iterative refinement.
    pub backsolves: u64,
}
/// Immutable numeric factor at one consumed original point. Clones share the actual
/// factor and its retained allowance; clearing an unrelated session cannot invalidate it.
#[derive(Clone, Debug)]
pub struct SparseFactor {
    held: Arc<Owned>,
    owner: Option<Arc<dyn pse_math::AllocationOwner>>,
}
#[derive(Debug)]
struct Owned {
    symbolic: SymbolicLu<usize>,
    numeric: NumericLu<usize, f64>,
    matrix: SparseColMat<usize, f64>,
    states: Vec<SemanticId>,
    rows: Vec<SemanticId>,
    point: Vec<f64>,
    scales: Normalization,
    key: SemanticProductKey,
    allowance: usize,
}
impl SparseFactor {
    /// Build once from freshly evaluated physical First partials. No stale iteration
    /// setup is accepted here. The caller's evaluator retains guard/selector admission.
    /// # Errors
    /// Infeasible/exterior point, key/shape mismatch, resource or numerical failure.
    pub fn prepare(
        request: SparseRequest<'_>,
        evaluate: impl FnOnce() -> Result<SparseColMat<usize, f64>, ProblemError>,
        execution: &Execution,
    ) -> Result<Self, Withheld> {
        execution.check().map_err(partial)?;
        let allowance = Self::admit(&request, execution)?;
        let n = request.scope.contract.variables.len();
        let physical = evaluate().map_err(partial)?;
        execution.check().map_err(partial)?;
        if physical.nrows() != n
            || physical.ncols() != n
            || physical.val().iter().any(|x| !x.is_finite())
        {
            return Err(Withheld::Numerical(
                "fresh sparse Jacobian shape or nonfinite values".into(),
            ));
        }
        if (0..n).any(|c| {
            !physical.symbolic().row_idx_of_col(c).eq(request
                .scope
                .pattern
                .as_ref()
                .row_idx_of_col(c))
        }) {
            return Err(Withheld::Structural(
                "fresh factor differs from admitted complete original support".into(),
            ));
        }
        // One construction includes the symbolic and numeric LU. The scientific
        // evaluation above is charged by its original worker callback owner.
        execution
            .counted(work_unit(0, 1), || {
                let pattern = physical.symbolic().to_owned().map_err(sparse_failure)?;
                let mut values = physical.val().to_vec();
                for column in 0..n {
                    for entry in physical.symbolic().col_range(column) {
                        values[entry] *= request.normalization.variables[column]
                            / request.normalization.rows[physical.symbolic().row_idx()[entry]];
                    }
                }
                if values.iter().any(|value| !value.is_finite()) {
                    return Err(ProblemError::numerical(
                        "nonfinite normalized fresh sparse Jacobian",
                    ));
                }
                let matrix = SparseColMat::new(pattern, values);
                let symbolic = crate::observation::scoped("square_response.faer.symbolic", || {
                    lu::factorize_symbolic_lu(matrix.symbolic(), Default::default())
                })
                .map_err(sparse_failure)?;
                let scratch =
                    symbolic.factorize_numeric_lu_scratch::<f64>(Par::Seq, Default::default());
                if scratch.size_bytes() > request.bytes {
                    return Err(ProblemError::memory(
                        "fresh sparse factor scratch allowance",
                    ));
                }
                let mut memory = MemBuffer::try_new(scratch)
                    .map_err(|_| ProblemError::memory("fresh sparse factor scratch allocation"))?;
                let mut numeric = NumericLu::new();
                crate::observation::scoped("square_response.faer.factor", || {
                    symbolic.factorize_numeric_lu(
                        &mut numeric,
                        matrix.as_ref(),
                        Par::Seq,
                        MemStack::new(&mut memory),
                        Default::default(),
                    )
                })
                .map_err(|e| match e {
                    faer::sparse::linalg::LuError::Generic(error) => sparse_failure(error),
                    faer::sparse::linalg::LuError::SymbolicSingular { index } => {
                        ProblemError::numerical(format!(
                            "fresh sparse factor rank loss at pivot {index}"
                        ))
                    }
                })?;
                execution.check()?;
                Ok(Self {
                    held: Arc::new(Owned {
                        symbolic,
                        numeric,
                        matrix,
                        states: request
                            .scope
                            .contract
                            .variables
                            .iter()
                            .map(|v| v.id)
                            .collect(),
                        rows: request.scope.contract.rows.clone(),
                        point: request.point.to_vec(),
                        scales: request.normalization.clone(),
                        key: request.key,
                        allowance,
                    }),
                    owner: None,
                })
            })
            .map_err(partial)
    }
    /// Check the complete original point/key and admission before derivatives.
    /// # Errors
    /// Physical feasibility, neighborhood, consumed key or allowance refusal.
    pub fn check(request: &SparseRequest<'_>) -> Result<usize, Withheld> {
        let n = request.scope.contract.variables.len();
        request
            .tolerances
            .validate(n, n)
            .map_err(|e| Withheld::Infeasible(e.to_string()))?;
        request
            .normalization
            .validate(n, n)
            .map_err(|e| Withheld::Numerical(e.to_string()))?;
        if n == 0
            || request.point.len() != n
            || request.values.len() != n
            || request
                .point
                .iter()
                .chain(request.values)
                .any(|x| !x.is_finite())
        {
            return Err(Withheld::Infeasible(
                "fresh sparse response point/row shape or values".into(),
            ));
        }
        if request
            .values
            .iter()
            .zip(&request.scope.bounds)
            .zip(&request.tolerances.rows)
            .any(|((value, (bound, _)), tol)| (value - bound).abs() > *tol)
        {
            return Err(Withheld::Infeasible(
                "fresh sparse response original physical residual".into(),
            ));
        }
        if request
            .scope
            .contract
            .variables
            .iter()
            .zip(request.point)
            .zip(&request.tolerances.variables)
            .any(|((variable, x), tol)| *x - variable.lower <= *tol || variable.upper - *x <= *tol)
        {
            return Err(Withheld::Neighborhood(
                "fresh sparse response requires inactive physical bounds".into(),
            ));
        }
        if request.key.point != Some(point_key(request.point))
            || request.key.normalization != Some(request.normalization.key())
            || request.key.accuracy.is_none()
            || request.key.structure != request.scope.contract.identity
        {
            return Err(Withheld::Numerical("fresh sparse factor key does not bind point, normalization and derivative accuracy".into()));
        }
        // Shape-based deployment reservation for private library fill. Neither this
        // estimate nor the separate MemStack scratch describes all live heap storage.
        Self::allowance(n)
            .filter(|bytes| *bytes <= request.bytes)
            .ok_or(Withheld::Memory)
    }
    pub(super) fn admit(
        request: &SparseRequest<'_>,
        execution: &Execution,
    ) -> Result<usize, Withheld> {
        let allowance = Self::check(request)?;
        admit_storage(execution).map_err(partial)?;
        Ok(allowance)
    }
    /// Shape-based opaque-library reservation, not a complete fill/peak bound.
    /// Strict storage admission refuses this producer before evaluating derivatives.
    pub fn allowance(n: usize) -> Option<usize> {
        n.checked_mul(n)?
            .checked_mul(64)?
            .checked_add(n.checked_mul(512)?)?
            .checked_add(4096)
    }
    /// Bind the actual escaping reservation, without reporting it as measured bytes.
    pub fn with_owner(mut self, owner: Arc<dyn pse_math::AllocationOwner>) -> Self {
        self.owner = Some(pse_math::retain_allocation_owner(self.owner.take(), owner));
        self
    }
    /// Retained original consumed dependencies; target parameter transport is separate.
    pub fn key(&self) -> SemanticProductKey {
        self.held.key
    }
    /// Ordered original state coordinates.
    pub fn states(&self) -> &[SemanticId] {
        &self.held.states
    }
    /// Ordered original equality rows, matching residuals and physical row scales.
    pub fn rows(&self) -> &[SemanticId] {
        &self.held.rows
    }
    /// Fresh physical point at which the factor was constructed.
    pub fn point(&self) -> &[f64] {
        &self.held.point
    }
    /// Reservation held by this product, not exact opaque foreign allocation.
    pub fn reserved_bytes(&self) -> usize {
        self.held.allowance
    }
    /// Solve a physical residual action using the retained factor, with at most two
    /// refinement backsolves. This never grants permission to a predicted nonlinear point.
    /// # Errors
    /// Shape/nonfinite, scope, scratch or backward-error failure.
    pub fn action(
        &self,
        rhs: &[f64],
        execution: &Execution,
    ) -> Result<(Vec<f64>, ActionEvidence), ProblemError> {
        execution.check()?;
        admit_storage(execution)?;
        if rhs.len() != self.held.point.len() || rhs.iter().any(|value| !value.is_finite()) {
            return Err(ProblemError::Contract(
                "sparse response action shape or nonfinite values".into(),
            ));
        }
        let rhs = Mat::from_fn(rhs.len(), 1, |row, _| rhs[row]);
        let (values, evidence) = self.actions(rhs.as_ref(), self.held.allowance, execution)?;
        Ok((values.col(0).iter().copied().collect(), evidence))
    }
    /// Apply all admitted physical RHS columns using this one numeric factor.
    pub(super) fn actions(
        &self,
        rhs: faer::MatRef<'_, f64>,
        bytes: usize,
        execution: &Execution,
    ) -> Result<(Mat<f64>, ActionEvidence), ProblemError> {
        execution.check()?;
        admit_storage(execution)?;
        let n = self.held.point.len();
        if rhs.nrows() != n
            || rhs.ncols() == 0
            || rhs
                .col_iter()
                .any(|column| column.iter().any(|value| !value.is_finite()))
        {
            return Err(ProblemError::Contract(
                "sparse response action shape or nonfinite values".into(),
            ));
        }
        let req = self
            .held
            .symbolic
            .solve_in_place_scratch::<f64>(rhs.ncols(), Par::Seq);
        if req.size_bytes() > bytes {
            return Err(ProblemError::memory("sparse response action scratch"));
        }
        let mut memory =
            MemBuffer::try_new(req).map_err(|e| ProblemError::memory(e.to_string()))?;
        let b = Mat::from_fn(n, rhs.ncols(), |row, column| {
            rhs[(row, column)] / self.held.scales.rows[row]
        });
        if b.col_iter()
            .any(|column| column.iter().any(|value| !value.is_finite()))
        {
            return Err(ProblemError::numerical(
                "nonfinite normalized sparse response RHS",
            ));
        }
        let mut x = b.clone();
        // Numeric/symbolic fields are private, installed together by the constructor.
        let lu = LuRef::new_unchecked(&self.held.symbolic, &self.held.numeric);
        execution.counted(work_unit(0, 0), || {
            crate::observation::integer(
                "square_response.faer.solve.rhs",
                "count",
                i64::try_from(rhs.ncols()).unwrap_or(i64::MAX),
            );
            crate::observation::scoped("square_response.faer.solve", || {
                lu.solve_in_place_with_conj(
                    Conj::No,
                    x.as_mut(),
                    Par::Seq,
                    MemStack::new(&mut memory),
                )
            });
            Ok(())
        })?;
        let limit = 64. * n.max(1) as f64 * f64::EPSILON;
        let mut backsolves = 1;
        loop {
            execution.check()?;
            if x.col_iter()
                .any(|column| column.iter().any(|value| !value.is_finite()))
            {
                return Err(ProblemError::numerical("nonfinite sparse response action"));
            }
            let mut residual = b.clone();
            sparse_dense_matmul(
                residual.as_mut(),
                Accum::Add,
                self.held.matrix.as_ref(),
                x.as_ref(),
                -1.,
                Par::Seq,
            );
            let error = backward(
                self.held.matrix.as_ref(),
                x.as_ref(),
                b.as_ref(),
                residual.as_ref(),
            );
            if error.is_finite() && error <= limit {
                let values = Mat::from_fn(n, rhs.ncols(), |row, column| {
                    x[(row, column)] * self.held.scales.variables[row]
                });
                if values
                    .col_iter()
                    .any(|column| column.iter().any(|value| !value.is_finite()))
                {
                    return Err(ProblemError::numerical(
                        "nonfinite physical sparse response",
                    ));
                }
                return Ok((
                    values,
                    ActionEvidence {
                        backward_error: error,
                        limit,
                        backsolves,
                    },
                ));
            }
            if backsolves == 3 {
                return Err(ProblemError::numerical(
                    "sparse response action backward-error allowance",
                ));
            }
            // Refinement is an actual solver iteration. A reused-factor action is
            // neither another factor construction nor a proof operation.
            execution.counted(work_unit(1, 0), || {
                crate::observation::scoped("square_response.faer.refinement_solve", || {
                    lu.solve_in_place_with_conj(
                        Conj::No,
                        residual.as_mut(),
                        Par::Seq,
                        MemStack::new(&mut memory),
                    )
                });
                for column in 0..rhs.ncols() {
                    for row in 0..n {
                        x[(row, column)] += residual[(row, column)];
                    }
                }
                backsolves += 1;
                Ok(())
            })?;
        }
    }
    /// Apply the physical transpose action `J^-T q_x` using the already retained
    /// normalized sparse factor. This is an estimated adjoint, not a certificate.
    /// # Errors
    /// Shape/nonfinite, scope, scratch or backward-error failure.
    pub fn transpose_action(
        &self,
        rhs: &[f64],
        execution: &Execution,
    ) -> Result<(Vec<f64>, ActionEvidence), ProblemError> {
        execution.check()?;
        admit_storage(execution)?;
        let n = self.held.point.len();
        if rhs.len() != n || rhs.iter().any(|x| !x.is_finite()) {
            return Err(ProblemError::Contract(
                "sparse response transpose action shape or nonfinite values".into(),
            ));
        }
        let req = self
            .held
            .symbolic
            .solve_in_place_scratch::<f64>(1, Par::Seq);
        if req.size_bytes() > self.held.allowance {
            return Err(ProblemError::memory("sparse response transpose scratch"));
        }
        let mut memory =
            MemBuffer::try_new(req).map_err(|e| ProblemError::memory(e.to_string()))?;
        // J = D_r A D_x^-1, hence J^T lambda=q_x iff
        // A^T (D_r lambda)=D_x q_x.
        let b = Mat::from_fn(n, 1, |i, _| rhs[i] * self.held.scales.variables[i]);
        let mut z = b.clone();
        let lu = LuRef::new_unchecked(&self.held.symbolic, &self.held.numeric);
        execution.counted(work_unit(0, 0), || {
            crate::observation::scoped("square_response.faer.transpose_solve", || {
                lu.solve_transpose_in_place_with_conj(
                    Conj::No,
                    z.as_mut(),
                    Par::Seq,
                    MemStack::new(&mut memory),
                )
            });
            Ok(())
        })?;
        let limit = 64. * n.max(1) as f64 * f64::EPSILON;
        let mut backsolves = 1;
        loop {
            execution.check()?;
            if z.col(0).iter().any(|v| !v.is_finite()) {
                return Err(ProblemError::numerical(
                    "nonfinite sparse response transpose action",
                ));
            }
            let mut residual = b.clone();
            for column in 0..n {
                for entry in self.held.matrix.symbolic().col_range(column) {
                    let row = self.held.matrix.symbolic().row_idx()[entry];
                    residual[(column, 0)] -= self.held.matrix.val()[entry] * z[(row, 0)];
                }
            }
            let error = transpose_backward(
                self.held.matrix.as_ref(),
                z.as_ref(),
                b.as_ref(),
                residual.as_ref(),
            );
            if error.is_finite() && error <= limit {
                let values = (0..n)
                    .map(|i| z[(i, 0)] / self.held.scales.rows[i])
                    .collect::<Vec<_>>();
                if values.iter().any(|v| !v.is_finite()) {
                    return Err(ProblemError::numerical(
                        "nonfinite physical sparse response transpose",
                    ));
                }
                return Ok((
                    values,
                    ActionEvidence {
                        backward_error: error,
                        limit,
                        backsolves,
                    },
                ));
            }
            if backsolves == 3 {
                return Err(ProblemError::numerical(
                    "sparse response transpose backward-error allowance",
                ));
            }
            execution.counted(work_unit(1, 0), || {
                crate::observation::scoped(
                    "square_response.faer.transpose_refinement_solve",
                    || {
                        lu.solve_transpose_in_place_with_conj(
                            Conj::No,
                            residual.as_mut(),
                            Par::Seq,
                            MemStack::new(&mut memory),
                        )
                    },
                );
                for i in 0..n {
                    z[(i, 0)] += residual[(i, 0)];
                }
                backsolves += 1;
                Ok(())
            })?;
        }
    }
}

fn work_unit(iterations: u64, factorizations: u64) -> WorkEvidence {
    WorkEvidence {
        evaluations: Some(0),
        iterations: Some(iterations),
        factorizations: Some(factorizations),
        proof_steps: Some(0),
    }
}

fn backward(
    a: SparseColMatRef<'_, usize, f64>,
    x: faer::MatRef<'_, f64>,
    b: faer::MatRef<'_, f64>,
    r: faer::MatRef<'_, f64>,
) -> f64 {
    let norm = |m: faer::MatRef<'_, f64>| {
        (0..m.nrows())
            .map(|row| {
                (0..m.ncols())
                    .map(|column| m[(row, column)].abs())
                    .sum::<f64>()
            })
            .fold(0., f64::max)
    };
    let mut rows = vec![0.; a.nrows()];
    for c in 0..a.ncols() {
        for k in a.symbolic().col_range(c) {
            rows[a.symbolic().row_idx()[k]] += a.val()[k].abs();
        }
    }
    let denominator = rows.into_iter().fold(0., f64::max) * norm(x) + norm(b);
    if !denominator.is_finite() {
        f64::INFINITY
    } else if denominator == 0. {
        norm(r)
    } else {
        norm(r) / denominator
    }
}

fn transpose_backward(
    a: SparseColMatRef<'_, usize, f64>,
    x: faer::MatRef<'_, f64>,
    b: faer::MatRef<'_, f64>,
    r: faer::MatRef<'_, f64>,
) -> f64 {
    let norm = |m: faer::MatRef<'_, f64>| m.col(0).iter().map(|v| v.abs()).fold(0., f64::max);
    // ||A^T||_inf is the largest absolute column sum of A.
    let mut columns = vec![0.; a.ncols()];
    for (column, total) in columns.iter_mut().enumerate() {
        for entry in a.symbolic().col_range(column) {
            *total += a.val()[entry].abs();
        }
    }
    let denominator = columns.into_iter().fold(0., f64::max) * norm(x) + norm(b);
    let residual = norm(r);
    if denominator == 0. {
        residual
    } else {
        residual / denominator
    }
}
/// A fresh sparse state factor paired with the parameter partials evaluated at that
/// same original point. Every prediction reuses this factor and performs only a
/// parameter action and bounded backsolves; it does not materialize a dense response.
#[derive(Clone, Debug)]
pub struct SparsePredictor {
    factor: SparseFactor,
    partials: Arc<ParameterPartials>,
    owner: Option<Arc<dyn pse_math::AllocationOwner>>,
}
#[derive(Debug)]
struct ParameterPartials {
    parameters: Vec<(SemanticId, f64)>,
    matrix: SparseColMat<usize, f64>,
}
impl SparsePredictor {
    /// Bind actual original parameter columns from the same fresh evaluation as the
    /// factor. Parameter order and finite base values are explicit.
    /// # Errors
    /// Duplicate/nonfinite parameters or an inconsistent/nonfinite sparse product.
    pub fn new(
        factor: SparseFactor,
        parameters: Vec<(SemanticId, f64)>,
        matrix: SparseColMat<usize, f64>,
    ) -> Result<Self, ProblemError> {
        if parameters.is_empty()
            || parameters
                .iter()
                .map(|p| p.0)
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                != parameters.len()
            || parameters.iter().any(|p| !p.1.is_finite())
            || matrix.nrows() != factor.states().len()
            || matrix.ncols() != parameters.len()
            || matrix.val().iter().any(|v| !v.is_finite())
        {
            return Err(ProblemError::Contract(
                "fresh root parameter partials differ from the admitted point/coordinates".into(),
            ));
        }
        Ok(Self {
            factor,
            partials: Arc::new(ParameterPartials { parameters, matrix }),
            owner: None,
        })
    }
    /// Attach the reservation for actual retained parameter storage; the factor retains
    /// its own independently admitted allocation anchor.
    pub fn with_owner(mut self, owner: Arc<dyn pse_math::AllocationOwner>) -> Self {
        if self.factor.owner.is_none() {
            self.factor = self.factor.with_owner(owner.clone());
        }
        self.owner = Some(pse_math::retain_allocation_owner(self.owner.take(), owner));
        self
    }
    /// Actual factor at the original point, shared by all related target actions.
    pub fn factor(&self) -> &SparseFactor {
        &self.factor
    }
    /// Parameter identities and physical values at the fresh derivative evaluation.
    pub fn parameters(&self) -> &[(SemanticId, f64)] {
        &self.partials.parameters
    }
    /// Compute the physical `-F_p Δp` action and apply the retained state factor.
    /// This remains an estimated tangent and cannot qualify its nonlinear target.
    /// # Errors
    /// Different/nonfinite parameter inventory, execution stop or numerical action failure.
    pub fn action(
        &self,
        parameters: &[(SemanticId, f64)],
        execution: &Execution,
    ) -> Result<(Vec<f64>, ActionEvidence), ProblemError> {
        execution.check()?;
        admit_storage(execution)?;
        let values = parameters
            .iter()
            .copied()
            .collect::<std::collections::BTreeMap<_, _>>();
        if values.len() != parameters.len()
            || values.len() != self.partials.parameters.len()
            || self
                .partials
                .parameters
                .iter()
                .any(|(id, _)| values.get(id).is_none_or(|value| !value.is_finite()))
        {
            return Err(ProblemError::Contract(
                "root predictor parameter inventory or values".into(),
            ));
        }
        let direction = Mat::from_fn(self.partials.parameters.len(), 1, |i, _| {
            values[&self.partials.parameters[i].0] - self.partials.parameters[i].1
        });
        if direction.col(0).iter().any(|v| !v.is_finite()) {
            return Err(ProblemError::Contract(
                "nonfinite root parameter difference".into(),
            ));
        }
        let mut rhs = Mat::zeros(self.factor.states().len(), 1);
        sparse_dense_matmul(
            rhs.as_mut(),
            Accum::Replace,
            self.partials.matrix.as_ref(),
            direction.as_ref(),
            -1.,
            Par::Seq,
        );
        self.factor
            .action(&rhs.col(0).iter().copied().collect::<Vec<_>>(), execution)
    }
}
/// Point identity shared by the factor producer and semantic start owner.
pub fn point_key(values: &[f64]) -> ContentHash {
    let mut h = pse_ids::FramedHasher::new(pse_ids::Frame::ImplicitNumericalProductV1);
    h.str("fresh-root-point").u64(values.len() as u64);
    for value in values {
        h.f64(*value);
    }
    h.finish_hash()
}
fn partial(e: ProblemError) -> Withheld {
    Withheld::Cause(Arc::new(e))
}
fn admit_storage(execution: &Execution) -> Result<(), ProblemError> {
    if let Some(admission) = &execution.work_admission {
        // faer 0.24.4's scratch API covers MemStack, while symbolic preparation and
        // pivot-dependent NumericLu fill grow private heap Vecs. Do not present the
        // deployment estimate as known bytes or a complete bound, including for retained-factor actions.
        admission.admit_storage(
            NativeStorageScope::Linear,
            "square-response-faer-lu",
            0,
            true,
        )?;
    }
    Ok(())
}
fn sparse_failure(error: faer::sparse::FaerError) -> ProblemError {
    match error {
        faer::sparse::FaerError::OutOfMemory => {
            ProblemError::memory("fresh sparse factor allocation failed")
        }
        faer::sparse::FaerError::IndexOverflow => {
            ProblemError::Contract("fresh sparse factor index overflow".into())
        }
        other => ProblemError::internal(format!("fresh sparse factor library failure: {other:?}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn factor(
        bytes: usize,
        point: &[f64],
        execution: &Execution,
    ) -> Result<SparseFactor, Withheld> {
        factor_values(bytes, point, execution, vec![4., 1., 3., 1.])
    }
    fn factor_values(
        bytes: usize,
        point: &[f64],
        execution: &Execution,
        values: Vec<f64>,
    ) -> Result<SparseFactor, Withheld> {
        factor_evaluations(bytes, point, execution, values, &std::cell::Cell::new(0))
    }
    fn factor_evaluations(
        bytes: usize,
        point: &[f64],
        execution: &Execution,
        values: Vec<f64>,
        evaluated: &std::cell::Cell<usize>,
    ) -> Result<SparseFactor, Withheld> {
        let scope = super::super::tests::scope();
        let scales = Normalization {
            variables: vec![100., 0.01],
            rows: vec![2., 400.],
            objective: 1.,
        };
        let tolerance = Tolerances {
            variables: vec![1e-8; 2],
            rows: vec![1e-8; 2],
            integrality: 1e-8,
        };
        let hash = ContentHash::from_bytes([4; 32]);
        let key = SemanticProductKey {
            structure: scope.contract.identity,
            binding: hash,
            numerical_policy: Some(hash),
            normalization: Some(scales.key()),
            point: Some(point_key(point)),
            parameters: Some(hash),
            derivation: None,
            branch: Some(hash),
            accuracy: Some(hash),
        };
        SparseFactor::prepare(
            SparseRequest {
                scope: &scope,
                point,
                values: &[0., 0.],
                tolerances: &tolerance,
                normalization: &scales,
                key,
                bytes,
            },
            || {
                evaluated.set(evaluated.get() + 1);
                Ok(SparseColMat::new(scope.pattern.clone(), values))
            },
            execution,
        )
    }
    #[derive(Debug, Default)]
    struct Admission {
        seen: std::sync::Mutex<Vec<WorkEvidence>>,
        storage: std::sync::Mutex<Vec<(NativeStorageScope, usize, bool)>>,
        strict_storage: bool,
        factors: Option<u64>,
        iterations: Option<u64>,
    }
    impl crate::solve::WorkAdmission for Admission {
        fn admit_storage(
            &self,
            scope: NativeStorageScope,
            _: &str,
            bytes: usize,
            opaque: bool,
        ) -> Result<(), ProblemError> {
            self.storage.lock().unwrap().push((scope, bytes, opaque));
            if self.strict_storage && scope == NativeStorageScope::Linear && opaque {
                return Err(ProblemError::Unsupported(
                    "strict linear storage requires complete producer-owned fill bound".into(),
                ));
            }
            Ok(())
        }
        fn admit(&self, work: WorkEvidence) -> Result<(), ProblemError> {
            let seen = self.seen.lock().unwrap();
            let factors: u64 = seen.iter().map(|w| w.factorizations.unwrap()).sum();
            let iterations: u64 = seen.iter().map(|w| w.iterations.unwrap()).sum();
            if self
                .factors
                .is_some_and(|cap| factors + work.factorizations.unwrap() > cap)
                || self
                    .iterations
                    .is_some_and(|cap| iterations + work.iterations.unwrap() > cap)
            {
                return Err(ProblemError::Limit {
                    kind: crate::LimitKind::Work,
                    detail: "fresh sparse test work allowance".into(),
                });
            }
            Ok(())
        }
        fn observe(&self, work: WorkEvidence) -> Result<(), ProblemError> {
            self.seen.lock().unwrap().push(work);
            Ok(())
        }
    }
    fn admitted(admission: Arc<Admission>) -> Execution {
        let mut execution = Execution::new(Arc::default(), &crate::solve::Controls::default());
        execution.work_admission = Some(admission);
        execution
    }
    #[test]
    fn opaque_sparse_fill_refuses_strict_before_derivatives_and_retained_actions() {
        let admission = Arc::new(Admission {
            strict_storage: true,
            ..Default::default()
        });
        let execution = admitted(admission.clone());
        let evaluated = std::cell::Cell::new(0);
        let result = factor_evaluations(
            1 << 20,
            &[2., 1.],
            &execution,
            vec![4., 1., 3., 1.],
            &evaluated,
        );
        assert!(
            matches!(result, Err(Withheld::Cause(cause)) if matches!(&*cause, ProblemError::Unsupported(_)))
        );
        assert_eq!(evaluated.get(), 0);
        assert!(admission.seen.lock().unwrap().is_empty());
        assert_eq!(
            *admission.storage.lock().unwrap(),
            [(NativeStorageScope::Linear, 0, true)]
        );

        let ordinary = admitted(Arc::new(Admission::default()));
        let held = factor(1 << 20, &[2., 1.], &ordinary).unwrap();
        assert!(matches!(
            held.action(&[-1., 1.], &execution),
            Err(ProblemError::Unsupported(_))
        ));
        assert!(matches!(
            held.transpose_action(&[1., 2.], &execution),
            Err(ProblemError::Unsupported(_))
        ));
        let rhs = faer::mat![[-1., -2.], [1., -3.]];
        assert!(matches!(
            held.actions(rhs.as_ref(), 1 << 20, &execution),
            Err(ProblemError::Unsupported(_))
        ));
        assert!(
            admission.seen.lock().unwrap().is_empty(),
            "refused actions must not execute a backsolve"
        );
        let (values, evidence) = held.actions(rhs.as_ref(), 1 << 20, &ordinary).unwrap();
        let expected = faer::mat![[-4., 7.], [5., -10.]];
        for column in 0..2 {
            for row in 0..2 {
                assert!((values[(row, column)] - expected[(row, column)]).abs() < 1e-11);
            }
        }
        assert_eq!(evidence.backsolves, 1);
    }
    #[test]
    fn fresh_factor_admission_counts_failed_construction_and_never_recharges_actions() {
        let refused = Arc::new(Admission {
            factors: Some(0),
            ..Default::default()
        });
        assert!(
            matches!(factor(1 << 20, &[2., 1.], &admitted(refused.clone())),
            Err(Withheld::Cause(cause)) if matches!(&*cause, ProblemError::Limit { .. }))
        );
        assert!(refused.seen.lock().unwrap().is_empty());

        let admission = Arc::new(Admission {
            factors: Some(1),
            ..Default::default()
        });
        let execution = admitted(admission.clone());
        let failure = factor_values(1 << 20, &[2., 1.], &execution, vec![f64::MAX; 4]);
        assert!(matches!(failure, Err(Withheld::Cause(cause))
            if matches!(&*cause, ProblemError::Numerical { .. })));
        assert_eq!(*admission.seen.lock().unwrap(), [work_unit(0, 1)]);
        assert!(factor(1 << 20, &[2., 1.], &execution).is_err());
        assert_eq!(admission.seen.lock().unwrap().len(), 1);

        let successful = Arc::new(Admission {
            factors: Some(1),
            ..Default::default()
        });
        let execution = admitted(successful.clone());
        let held = factor(1 << 20, &[2., 1.], &execution).unwrap();
        let (_, first) = held.action(&[-1., 1.], &execution).unwrap();
        let (_, second) = held.action(&[0., 0.], &execution).unwrap();
        assert_eq!((first.backsolves, second.backsolves), (1, 1));
        assert_eq!(
            *successful.seen.lock().unwrap(),
            [work_unit(0, 1), work_unit(0, 0), work_unit(0, 0)]
        );
    }
    #[test]
    fn refinement_iterations_admit_before_work_and_preserve_failed_observations() {
        let base = Execution::new(Arc::default(), &crate::solve::Controls::default());
        let mut held = factor(1 << 20, &[2., 1.], &base).unwrap();
        // A deliberately inconsistent residual operator makes both corrections
        // unavoidable, independently of platform rounding or refinement heuristics.
        let owned = Arc::get_mut(&mut held.held).unwrap();
        for value in owned.matrix.val_mut() {
            *value *= 2.;
        }
        let refusal = Arc::new(Admission {
            iterations: Some(0),
            ..Default::default()
        });
        assert!(matches!(
            held.action(&[-1., 1.], &admitted(refusal.clone())),
            Err(ProblemError::Limit { .. })
        ));
        assert_eq!(*refusal.seen.lock().unwrap(), [work_unit(0, 0)]);

        let one_step = Arc::new(Admission {
            iterations: Some(1),
            ..Default::default()
        });
        assert!(matches!(
            held.action(&[-1., 1.], &admitted(one_step.clone())),
            Err(ProblemError::Limit { .. })
        ));
        assert_eq!(
            *one_step.seen.lock().unwrap(),
            [work_unit(0, 0), work_unit(1, 0)]
        );

        let admitted_steps = Arc::new(Admission {
            iterations: Some(2),
            ..Default::default()
        });
        let failed = held.action(&[-1., 1.], &admitted(admitted_steps.clone()));
        assert!(matches!(failed, Err(ProblemError::Numerical { .. })));
        assert_eq!(
            *admitted_steps.seen.lock().unwrap(),
            [work_unit(0, 0), work_unit(1, 0), work_unit(1, 0)]
        );
    }
    #[test]
    fn fresh_sparse_actions_reuse_factor_and_match_independent_perturbed_roots() {
        let execution = Execution::new(Arc::default(), &crate::solve::Controls::default());
        let held = factor(1 << 20, &[2., 1.], &execution).unwrap();
        let shared = held.clone();
        assert!(Arc::ptr_eq(&held.held, &shared.held));
        let (action, evidence) = held.action(&[-1., 1.], &execution).unwrap();
        let root = |p: f64| {
            let x = (p + (28. - 3. * p * p).sqrt()) / 2.;
            [x, p - x]
        };
        let h = 1e-7;
        let high = root(3. + h);
        let low = root(3. - h);
        for i in 0..2 {
            assert!((action[i] - (high[i] - low[i]) / (2. * h)).abs() < 2e-7);
        }
        assert!(evidence.backward_error <= evidence.limit);
        assert_eq!(evidence.backsolves, 1);
        let (zero, _) = shared.action(&[0., 0.], &execution).unwrap();
        assert_eq!(zero, [0., 0.]);
        execution
            .cancel
            .store(true, std::sync::atomic::Ordering::Release);
        assert!(matches!(
            held.action(&[-1., 1.], &execution),
            Err(ProblemError::Cancelled)
        ));
    }
    #[test]
    fn transpose_action_applies_physical_scaling_and_charges_only_backsolves() {
        let admission = Arc::new(Admission {
            factors: Some(1),
            ..Default::default()
        });
        let execution = admitted(admission.clone());
        let held = factor(1 << 20, &[2., 1.], &execution).unwrap();
        // J = [[4, 3], [1, 1]], so J^T [-1, 5] = [1, 2].
        let (lambda, evidence) = held.transpose_action(&[1., 2.], &execution).unwrap();
        assert!((lambda[0] + 1.).abs() < 1e-12);
        assert!((lambda[1] - 5.).abs() < 1e-12);
        assert!(evidence.backward_error <= evidence.limit);
        assert_eq!(evidence.backsolves, 1);
        assert_eq!(
            *admission.seen.lock().unwrap(),
            [work_unit(0, 1), work_unit(0, 0)]
        );
    }
    #[test]
    fn fresh_sparse_action_refuses_allowance_exterior_and_bad_directions() {
        let execution = Execution::new(Arc::default(), &crate::solve::Controls::default());
        assert!(matches!(
            factor(1, &[2., 1.], &execution),
            Err(Withheld::Memory)
        ));
        assert!(matches!(
            factor(1 << 20, &[100., 1.], &execution),
            Err(Withheld::Neighborhood(_))
        ));
        let held = factor(1 << 20, &[2., 1.], &execution).unwrap();
        assert!(matches!(
            held.action(&[f64::NAN, 1.], &execution),
            Err(ProblemError::Contract(_))
        ));
    }
    #[test]
    fn parameter_actions_share_fresh_factor_without_dense_responses() {
        let execution = Execution::new(Arc::default(), &crate::solve::Controls::default());
        let parameter = SemanticId::from_bytes([8; 16]);
        let matrix = SparseColMat::try_new_from_triplets(
            2,
            1,
            &[
                faer::sparse::Triplet::new(0, 0, 1.),
                faer::sparse::Triplet::new(1, 0, -1.),
            ],
        )
        .unwrap();
        let predictor = SparsePredictor::new(
            factor(1 << 20, &[2., 1.], &execution).unwrap(),
            vec![(parameter, 3.)],
            matrix,
        )
        .unwrap();
        let clone = predictor.clone();
        assert!(Arc::ptr_eq(&predictor.factor.held, &clone.factor.held));
        assert!(Arc::ptr_eq(&predictor.partials, &clone.partials));
        let (delta, evidence) = clone.action(&[(parameter, 3.01)], &execution).unwrap();
        assert!((delta[0] + 0.04).abs() < 1e-12 && (delta[1] - 0.05).abs() < 1e-12);
        assert_eq!(evidence.backsolves, 1);
        assert!(
            clone
                .action(&[(SemanticId::NIL, 3.01)], &execution)
                .is_err()
        );
        assert!(
            clone
                .action(&[(parameter, f64::INFINITY)], &execution)
                .is_err()
        );
    }
    #[test]
    fn escaped_shared_factors_retain_original_and_combined_allocation_owners() {
        let execution = Execution::new(Arc::default(), &crate::solve::Controls::default());
        let matrix = || {
            SparseColMat::try_new_from_triplets(
                2,
                1,
                &[
                    faer::sparse::Triplet::new(0, 0, 1.),
                    faer::sparse::Triplet::new(1, 0, -1.),
                ],
            )
            .unwrap()
        };
        let factor_owner = Arc::new(());
        let parameter_owner = Arc::new(());
        let factor_weak = Arc::downgrade(&factor_owner);
        let parameter_weak = Arc::downgrade(&parameter_owner);
        let predictor = SparsePredictor::new(
            factor(1 << 20, &[2., 1.], &execution)
                .unwrap()
                .with_owner(factor_owner.clone()),
            vec![(SemanticId::NIL, 3.)],
            matrix(),
        )
        .unwrap()
        .with_owner(parameter_owner.clone());
        let escaped = predictor.factor().clone();
        drop(factor_owner);
        drop(parameter_owner);
        drop(predictor);
        assert!(factor_weak.upgrade().is_some());
        assert!(parameter_weak.upgrade().is_none());
        drop(escaped);
        assert!(factor_weak.upgrade().is_none());
        let combined = Arc::new(());
        let weak = Arc::downgrade(&combined);
        let predictor = SparsePredictor::new(
            factor(1 << 20, &[2., 1.], &execution).unwrap(),
            vec![(SemanticId::NIL, 3.)],
            matrix(),
        )
        .unwrap()
        .with_owner(combined.clone());
        let escaped = predictor.factor().clone();
        drop(combined);
        drop(predictor);
        assert!(weak.upgrade().is_some());
        drop(escaped);
        assert!(weak.upgrade().is_none());
    }
}
