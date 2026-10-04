// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Fresh sparse response actions. Numerical LU/backward error is estimated evidence,
//! not a numerical-rank certificate or a forward-error bound on a nonlinear root.
use super::{SquareScope, Withheld};
use crate::{ProblemError, quality::Tolerances, solve::Execution};
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
    /// Finite retained/scratch and opaque factor allowance, not measured allocation bytes.
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
        // Conservative shape reservation for private library fill; this is not an
        // observation of opaque factor allocation, nor an allocator-enforced hard cap.
        let allowance = Self::allowance(n)
            .filter(|bytes| *bytes <= request.bytes)
            .ok_or(Withheld::Memory)?;
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
        let pattern = physical
            .symbolic()
            .to_owned()
            .map_err(|e| partial(sparse_failure(e)))?;
        let mut values = physical.val().to_vec();
        for column in 0..n {
            for entry in physical.symbolic().col_range(column) {
                values[entry] *= request.normalization.variables[column]
                    / request.normalization.rows[physical.symbolic().row_idx()[entry]];
            }
        }
        let matrix = SparseColMat::new(pattern, values);
        let symbolic = lu::factorize_symbolic_lu(matrix.symbolic(), Default::default())
            .map_err(|e| partial(sparse_failure(e)))?;
        let scratch = symbolic.factorize_numeric_lu_scratch::<f64>(Par::Seq, Default::default());
        if scratch.size_bytes() > request.bytes.saturating_sub(allowance / 2) {
            return Err(Withheld::Memory);
        }
        let mut memory = MemBuffer::try_new(scratch).map_err(|_| Withheld::Memory)?;
        let mut numeric = NumericLu::new();
        symbolic
            .factorize_numeric_lu(
                &mut numeric,
                matrix.as_ref(),
                Par::Seq,
                MemStack::new(&mut memory),
                Default::default(),
            )
            .map_err(|e| {
                partial(match e {
                    faer::sparse::linalg::LuError::Generic(error) => sparse_failure(error),
                    faer::sparse::linalg::LuError::SymbolicSingular { index } => {
                        ProblemError::numerical(format!(
                            "fresh sparse factor rank loss at pivot {index}"
                        ))
                    }
                })
            })?;
        execution.check().map_err(partial)?;
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
                point: request.point.to_vec(),
                scales: request.normalization.clone(),
                key: request.key,
                allowance,
            }),
            owner: None,
        })
    }
    /// Conservative opaque-library allowance. Dense response matrices are not allocated.
    pub fn allowance(n: usize) -> Option<usize> {
        n.checked_mul(n)?
            .checked_mul(64)?
            .checked_add(n.checked_mul(512)?)?
            .checked_add(4096)
    }
    /// Bind the actual escaping reservation, without reporting it as measured bytes.
    pub fn with_owner(mut self, owner: Arc<dyn pse_math::AllocationOwner>) -> Self {
        self.owner = Some(owner);
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
        let n = self.held.point.len();
        if rhs.len() != n || rhs.iter().any(|x| !x.is_finite()) {
            return Err(ProblemError::Contract(
                "sparse response action shape or nonfinite values".into(),
            ));
        }
        let req = self
            .held
            .symbolic
            .solve_in_place_scratch::<f64>(1, Par::Seq);
        if req.size_bytes() > self.held.allowance {
            return Err(ProblemError::memory("sparse response action scratch"));
        }
        let mut memory =
            MemBuffer::try_new(req).map_err(|e| ProblemError::memory(e.to_string()))?;
        let b = Mat::from_fn(n, 1, |i, _| rhs[i] / self.held.scales.rows[i]);
        let mut x = b.clone();
        // Numeric/symbolic fields are private, installed together by the constructor.
        let lu = LuRef::new_unchecked(&self.held.symbolic, &self.held.numeric);
        lu.solve_in_place_with_conj(Conj::No, x.as_mut(), Par::Seq, MemStack::new(&mut memory));
        let limit = 64. * n.max(1) as f64 * f64::EPSILON;
        let mut backsolves = 1;
        loop {
            execution.check()?;
            if x.col(0).iter().any(|v| !v.is_finite()) {
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
                let values = (0..n)
                    .map(|i| x[(i, 0)] * self.held.scales.variables[i])
                    .collect::<Vec<_>>();
                if values.iter().any(|v| !v.is_finite()) {
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
            lu.solve_in_place_with_conj(
                Conj::No,
                residual.as_mut(),
                Par::Seq,
                MemStack::new(&mut memory),
            );
            for i in 0..n {
                x[(i, 0)] += residual[(i, 0)];
            }
            backsolves += 1;
        }
    }
}

fn backward(
    a: SparseColMatRef<'_, usize, f64>,
    x: faer::MatRef<'_, f64>,
    b: faer::MatRef<'_, f64>,
    r: faer::MatRef<'_, f64>,
) -> f64 {
    let norm = |m: faer::MatRef<'_, f64>| m.col(0).iter().map(|v| v.abs()).fold(0., f64::max);
    let mut rows = vec![0.; a.nrows()];
    for c in 0..a.ncols() {
        for k in a.symbolic().col_range(c) {
            rows[a.symbolic().row_idx()[k]] += a.val()[k].abs();
        }
    }
    let denominator = rows.into_iter().fold(0., f64::max) * norm(x) + norm(b);
    if denominator == 0. {
        norm(r)
    } else {
        norm(r) / denominator
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
        self.owner = Some(owner);
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
                Ok(SparseColMat::new(
                    scope.pattern.clone(),
                    vec![4., 1., 3., 1.],
                ))
            },
            execution,
        )
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
}
