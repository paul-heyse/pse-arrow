// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Qualified regular-square response, distinct from optimization KKT analysis.
use crate::{OracleContract, ProblemError};
use faer::Mat;
use pse_ids::SemanticId;

mod actions;
pub use actions::{ActionEvidence, SparseFactor, SparsePredictor, SparseRequest, SparseActionExecutor, point_key};

/// Default relative scaled rank cutoff of public Root parameter analysis.
pub const DEFAULT_RELATIVE_RANK_CUTOFF: f64 = 1e-10;

/// Typed cause for withholding a response while retaining its base solution.
#[derive(Clone, Debug, thiserror::Error, miette::Diagnostic)]
#[diagnostic(code(pse::native::square_response))]
pub enum Withheld {
    /// No original candidate exists.
    #[error("no original candidate for square response")]
    NoCandidate,
    /// Original physical equality feasibility failed.
    #[error("square response requires original physical feasibility: {0}")]
    Infeasible(String),
    /// Complete original matching did not establish a square closure.
    #[error("square response structural admission: {0}")]
    Structural(String),
    /// A bound, guard, or selector has no established local interior.
    #[error("square response neighborhood unavailable: {0}")]
    Neighborhood(String),
    /// Numerical rank fails at declared scaling/cutoff.
    #[error("square response rank {rank} of {dimension} at cutoff {cutoff}")]
    Rank {
        /// Observed scaled numerical rank.
        rank: usize,
        /// Required full state dimension.
        dimension: usize,
        /// Stated singular-value threshold.
        cutoff: f64,
    },
    /// Evaluation or factorization failed.
    #[error("square response numerical analysis: {0}")]
    Numerical(String),
    /// The explicit worker allowance cannot hold the operation.
    #[error("square response memory allowance")]
    Memory,
    /// Exact mathematical, operational or checkpoint cause of an optional action.
    #[error("square response action: {0}")]
    Cause(#[source] std::sync::Arc<ProblemError>),
}
/// A complete original equality scope admitted using library-owned matching.
#[derive(Clone, Debug)]
pub struct SquareScope {
    contract: OracleContract,
    bounds: Vec<(f64, f64)>,
    pattern: faer::sparse::SymbolicSparseColMat<usize>,
    symbolic: std::sync::Arc<std::sync::Mutex<Option<actions::SymbolicPlan>>>,
    symbolic_owner: std::sync::Arc<std::sync::Mutex<Option<std::sync::Arc<dyn pse_math::AllocationOwner>>>>,
}
impl SquareScope {
    /// Share a symbolic preparation only for the same original support and order.
    /// Scaling and fixed library settings are checked by the factor producer.
    pub fn reuse_symbolic_from(&mut self, previous: &Self) -> bool {
        let compatible = self.contract.identity == previous.contract.identity
            && self.contract.rows == previous.contract.rows
            && self.contract.variables.iter().map(|v| v.id).eq(previous.contract.variables.iter().map(|v| v.id))
            && self.pattern.nrows() == previous.pattern.nrows()
            && self.pattern.ncols() == previous.pattern.ncols()
            && (0..self.pattern.ncols()).all(|c| self.pattern.as_ref().row_idx_of_col(c).eq(previous.pattern.as_ref().row_idx_of_col(c)));
        if compatible { self.symbolic = previous.symbolic.clone(); self.symbolic_owner = previous.symbolic_owner.clone(); }
        compatible
    }
    /// Anchor the retained symbolic cache independently of any escaping numeric factor.
    pub fn with_owner(self, owner: std::sync::Arc<dyn pse_math::AllocationOwner>) -> Self {
        if let Ok(mut held)=self.symbolic_owner.lock() {if held.is_none() {*held=Some(owner);}}
        self
    }
    /// Validate every original row/free coordinate and its matching witness.
    /// # Errors
    /// Partial, ill-posed, non-equality, or inconsistent original scope.
    pub fn admit(
        contract: &OracleContract,
        pattern: faer::sparse::SymbolicSparseColMatRef<'_, usize>,
        bounds: &[(f64, f64)],
        analysis: Option<&pse_structural::incidence::StructuralAnalysis>,
    ) -> Result<Self, Withheld> {
        contract
            .square()
            .map_err(|e| Withheld::Structural(e.to_string()))?;
        if bounds.iter().any(|(l, u)| !l.is_finite() || l != u) {
            return Err(Withheld::Structural(
                "all original rows must be finite equalities".into(),
            ));
        }
        crate::structural::check(
            contract,
            pattern,
            bounds,
            crate::structural::Mode::Roots,
            analysis,
        )
        .map_err(|e| Withheld::Structural(e.to_string()))?;
        Ok(Self {
            contract: contract.clone(),
            bounds: bounds.to_vec(),
            pattern: pattern.to_owned().map_err(|_| Withheld::Memory)?,
            symbolic: Default::default(),
            symbolic_owner: Default::default(),
        })
    }
}
/// Explicit qualification evidence in the scaled system; no KKT claims.
#[derive(Clone, Debug)]
pub struct Evidence {
    /// Numerical rank of the scaled state Jacobian.
    pub rank: usize,
    /// Absolute singular-value cutoff used for admission.
    pub cutoff: f64,
    /// Relative cutoff stated by the caller's numerical policy.
    pub relative_cutoff: f64,
    /// Normwise backward error of the scaled solve.
    pub backward_error: f64,
    /// Acceptance bound on that error.
    pub backward_error_limit: f64,
    /// The compiled First derivatives evaluated in their admitted guard/selector interior,
    /// and every state bound is inactive at its physical tolerance.
    pub neighborhood: &'static str,
}
/// Physical response with ordered coordinate identities.
#[derive(Clone, Debug)]
pub struct Response {
    /// Physical state row order.
    pub states: Vec<SemanticId>,
    /// Parameter column order.
    pub parameters: Vec<SemanticId>,
    /// State × parameter, in state units per parameter unit.
    pub values: Mat<f64>,
    /// Rank, backward error and local validity evidence.
    pub evidence: Evidence,
    owner: Option<std::sync::Arc<dyn pse_math::AllocationOwner>>,
}
impl Response {
    /// Retain the runtime reservation with every clone of the response.
    pub fn with_owner(mut self, owner: std::sync::Arc<dyn pse_math::AllocationOwner>) -> Self {
        self.owner = Some(owner);
        self
    }
}
/// Known dense workspaces plus an opaque LU reservation, checked before allocation.
/// This is not a complete producer-owned bound on private symbolic/numeric LU fill.
pub fn workspace_bytes(n: usize, np: usize) -> Option<usize> {
    use faer::linalg::temp_mat_scratch;
    n.checked_mul(np)?.checked_mul(5 * size_of::<f64>())?;
    // Dense spectrum is explicitly bounded by the admitted workspace. Opaque
    // sparse LU fill retains a shape-based opaque reservation independently. Strict
    // storage admission refuses that unknown producer extent before derivatives.
    SparseFactor::allowance(n)?
        .checked_add(temp_mat_scratch::<f64>(n, n).size_bytes())?
        .checked_add(temp_mat_scratch::<f64>(n, np).size_bytes().checked_mul(5)?)?
        .checked_add(
            faer::sparse::linalg::lu::supernodal::solve_in_place_scratch::<usize, f64>(
                n,
                np,
                faer::Par::Seq,
            )
            .size_bytes(),
        )?
        .checked_add(rank_scratch(n).size_bytes())?
        .checked_add(n.checked_add(np)?.checked_mul(size_of::<SemanticId>())?)?
        .checked_add(n.checked_mul(2 * size_of::<f64>())?)
}
/// Numerical inputs from a qualified original point. `evaluate` must evaluate admitted
/// First physical partials at this point, retaining compiler guard/selector checks.
#[derive(Debug)]
pub struct Request<'a> {
    /// Complete original closure.
    pub scope: &'a SquareScope,
    /// Physical states.
    pub point: &'a [f64],
    /// Fresh original row values.
    pub residual_values: &'a [f64],
    /// Physical state bound and row tolerances.
    pub tolerances: &'a crate::quality::Tolerances,
    /// Positive state and row coordinate scales; objective factor is unused.
    pub normalization: &'a pse_math::normalization::Normalization,
    /// Parameters, in derivative column order.
    pub parameters: &'a [SemanticId],
    /// Positive parameter coordinate scales.
    pub parameter_scales: &'a [f64],
    /// Relative numerical rank cutoff.
    pub rank_tolerance: f64,
    /// Maximum retained/scratch bytes.
    pub bytes: usize,
}
/// The independently withheld public response and optionally demanded predictor.
#[derive(Debug)]
pub struct Products {
    /// Singular-value-qualified public physical response.
    pub response: Result<Response, Withheld>,
    /// Same-point sparse prediction product, absent unless explicitly demanded.
    pub predictor: Option<Result<SparsePredictor, Withheld>>,
}
impl Products {
    fn withheld(cause: Withheld, retain: bool) -> Self {
        Self {
            response: Err(cause.clone()),
            predictor: retain.then_some(Err(cause)),
        }
    }
}
/// Evaluate original partials once and share one same-point sparse numeric factor.
/// Spectrum admission remains independent of estimated predictor actions.
pub fn products(
    request: Request<'_>,
    key: pse_model::strategy::SemanticProductKey,
    parameter_values: &[f64],
    retain: bool,
    evaluate: impl FnOnce() -> Result<
        (
            faer::sparse::SparseColMat<usize, f64>,
            faer::sparse::SparseColMat<usize, f64>,
        ),
        ProblemError,
    >,
    execution: &crate::solve::Execution,
) -> Products {
    if let Err(cause) = execution.check() {
        return Products::withheld(Withheld::Cause(std::sync::Arc::new(cause)), retain);
    }
    if let Err(cause) = admit_response(&request) {
        return Products::withheld(cause, retain);
    }
    let n = request.scope.contract.variables.len();
    let np = request.parameters.len();
    if parameter_values.len() != np || parameter_values.iter().any(|value| !value.is_finite()) {
        return Products::withheld(Withheld::Numerical("parameter base values".into()), retain);
    }
    if let Err(cause) = SparseFactor::admit(
        &SparseRequest {
            scope: request.scope,
            point: request.point,
            values: request.residual_values,
            tolerances: request.tolerances,
            normalization: request.normalization,
            key,
            bytes: request.bytes,
        },
        execution,
    ) {
        return Products::withheld(cause, retain);
    }
    let (fx, fp) = match evaluate() {
        Ok(partials) => partials,
        Err(cause) => return Products::withheld(evaluation_failure(&cause), retain),
    };
    if fx.nrows() != n
        || fx.ncols() != n
        || fp.nrows() != n
        || fp.ncols() != np
        || fx
            .val()
            .iter()
            .chain(fp.val())
            .any(|value| !value.is_finite())
    {
        return Products::withheld(
            Withheld::Numerical("physical Jacobian dimensions or values".into()),
            retain,
        );
    }
    if (0..n).any(|column| {
        !fx.symbolic().row_idx_of_col(column).eq(request
            .scope
            .pattern
            .as_ref()
            .row_idx_of_col(column))
    }) {
        return Products::withheld(
            Withheld::Structural(
                "fresh partials differ from admitted complete original support".into(),
            ),
            retain,
        );
    }
    let rank = (|| {
        execution
            .check()
            .map_err(|cause| Withheld::Cause(std::sync::Arc::new(cause)))?;
        let mut a = Mat::zeros(n, n);
        for column in 0..n {
            for entry in fx.symbolic().col_range(column) {
                let row = fx.symbolic().row_idx()[entry];
                a[(row, column)] = fx.val()[entry] * request.normalization.variables[column]
                    / request.normalization.rows[row];
            }
        }
        if a.col_iter()
            .any(|column| column.iter().any(|value| !value.is_finite()))
        {
            return Err(Withheld::Numerical(
                "nonfinite normalized response Jacobian".into(),
            ));
        }
        let singular =
            spectrum(&a, request.bytes).map_err(|cause| Withheld::Numerical(cause.to_string()))?;
        let cutoff = singular.first().copied().unwrap_or(0.) * request.rank_tolerance;
        let rank = singular.iter().filter(|value| **value > cutoff).count();
        if rank != n {
            return Err(Withheld::Rank {
                rank,
                dimension: n,
                cutoff,
            });
        }
        Ok((rank, cutoff))
    })();
    if !retain && let Err(cause) = &rank {
        return Products {
            response: Err(cause.clone()),
            predictor: None,
        };
    }
    let factor = match SparseFactor::prepare(
        SparseRequest {
            scope: request.scope,
            point: request.point,
            values: request.residual_values,
            tolerances: request.tolerances,
            normalization: request.normalization,
            key,
            bytes: request.bytes,
        },
        || Ok(fx),
        execution,
    ) {
        Ok(factor) => factor,
        Err(cause) => {
            return Products {
                response: match rank {
                    Err(rank) => Err(rank),
                    Ok(_) => Err(cause.clone()),
                },
                predictor: retain.then_some(Err(cause)),
            };
        }
    };
    let response = rank.and_then(|(rank, cutoff)| {
        let mut rhs = Mat::zeros(n, np);
        for column in 0..np {
            for entry in fp.symbolic().col_range(column) {
                rhs[(fp.symbolic().row_idx()[entry], column)] = -fp.val()[entry] * request.parameter_scales[column];
            }
        }
        let (scaled, evidence) = factor.actions(rhs.as_ref(), request.bytes, execution).map_err(|cause| Withheld::Cause(std::sync::Arc::new(cause)))?;
        let values = Mat::from_fn(n, np, |row, column| scaled[(row, column)] / request.parameter_scales[column]);
        if values.col_iter().any(|column| column.iter().any(|value| !value.is_finite())) { return Err(Withheld::Numerical("nonfinite response in physical coordinates".into())); }
        Ok(Response { owner: None, states: factor.states().to_vec(), parameters: request.parameters.to_vec(), values,
            evidence: Evidence { rank, cutoff, relative_cutoff: request.rank_tolerance,
                backward_error: evidence.backward_error, backward_error_limit: evidence.limit,
                neighborhood: "compiled-first-partials/guard-selector-interior; inactive-physical-state-bounds" } })
    });
    let predictor = retain.then(|| {
        SparsePredictor::new(
            factor,
            request
                .parameters
                .iter()
                .copied()
                .zip(parameter_values.iter().copied())
                .collect(),
            fp,
        )
        .map_err(|cause| Withheld::Cause(std::sync::Arc::new(cause)))
    });
    Products {
        response,
        predictor,
    }
}
fn admit_response(request: &Request<'_>) -> Result<(), Withheld> {
    let c = &request.scope.contract;
    let n = c.variables.len();
    let np = request.parameters.len();
    request
        .tolerances
        .validate(n, n)
        .map_err(|e| Withheld::Infeasible(e.to_string()))?;
    request
        .normalization
        .validate(n, n)
        .map_err(|e| Withheld::Numerical(e.to_string()))?;
    if request.point.len() != n
        || request.residual_values.len() != n
        || request
            .point
            .iter()
            .chain(request.residual_values)
            .any(|v| !v.is_finite())
    {
        return Err(Withheld::Infeasible(
            "point dimensions or nonfinite values".into(),
        ));
    }
    if request
        .residual_values
        .iter()
        .zip(&request.scope.bounds)
        .zip(&request.tolerances.rows)
        .any(|((v, (l, _)), t)| (v - l).abs() > *t)
    {
        return Err(Withheld::Infeasible(
            "original equality residual exceeds its physical budget".into(),
        ));
    }
    if c.variables
        .iter()
        .zip(request.point)
        .zip(&request.tolerances.variables)
        .any(|((v, x), t)| (*x - v.lower) <= *t || (v.upper - *x) <= *t)
    {
        return Err(Withheld::Neighborhood(
            "active or exterior state bound".into(),
        ));
    }
    if np == 0
        || request
            .parameters
            .iter()
            .enumerate()
            .any(|(i, p)| request.parameters[..i].contains(p))
        || c.variables
            .iter()
            .any(|v| request.parameters.contains(&v.id))
        || request.parameter_scales.len() != np
        || request
            .parameter_scales
            .iter()
            .any(|s| !s.is_finite() || *s <= 0.)
        || !request.rank_tolerance.is_finite()
        || request.rank_tolerance <= 0.
        || request.rank_tolerance >= 1.
    {
        return Err(Withheld::Numerical(
            "parameter identities/scales or rank cutoff".into(),
        ));
    }
    workspace_bytes(n, np)
        .filter(|b| *b <= request.bytes)
        .ok_or(Withheld::Memory)?;
    Ok(())
}
/// Classify a failed fresh response evaluation without discarding the producer
/// cause retained independently by a predictor.
pub fn evaluation_failure(cause: &ProblemError) -> Withheld {
    fn memory(cause: &pse_math::MathError) -> bool {
        match cause {
            pse_math::MathError::Instance { cause, .. } => memory(cause),
            pse_math::MathError::Limit(_)
            | pse_math::MathError::ByteLimit { .. }
            | pse_math::MathError::SlotLimit { .. }
            | pse_math::MathError::WorkLimit { .. } => true,
            _ => false,
        }
    }
    if matches!(cause, ProblemError::Limit { .. })
        || matches!(cause, ProblemError::Math(cause) if memory(cause))
    {
        Withheld::Memory
    } else {
        Withheld::Neighborhood(cause.to_string())
    }
}
fn rank_scratch(n: usize) -> faer::dyn_stack::StackReq {
    use faer::linalg::svd::{self, ComputeSvdVectors};
    svd::svd_scratch::<f64>(
        n,
        n,
        ComputeSvdVectors::No,
        ComputeSvdVectors::No,
        faer::Par::Seq,
        Default::default(),
    )
}
fn spectrum(a: &Mat<f64>, bytes: usize) -> Result<Vec<f64>, ProblemError> {
    use faer::{
        Par,
        diag::Diag,
        dyn_stack::{MemBuffer, MemStack},
        linalg::svd,
    };
    let req = rank_scratch(a.nrows());
    if req.size_bytes() > bytes {
        return Err(ProblemError::memory("square response rank scratch"));
    }
    let mut memory = MemBuffer::try_new(req).map_err(|e| ProblemError::memory(e.to_string()))?;
    let mut s = Diag::<f64>::zeros(a.nrows());
    svd::svd(
        a.as_ref(),
        s.as_mut(),
        None,
        None,
        Par::Seq,
        MemStack::new(&mut memory),
        Default::default(),
    )
    .map_err(|e| ProblemError::numerical(format!("{e:?}")))?;
    Ok(s.column_vector().iter().copied().collect())
}
#[cfg(test)]
mod tests {
    use super::*;
    fn sparse(matrix: &Mat<f64>) -> faer::sparse::SparseColMat<usize, f64> {
        let mut entries = Vec::new();
        for column in 0..matrix.ncols() {
            for row in 0..matrix.nrows() {
                entries.push(faer::sparse::Triplet::new(
                    row,
                    column,
                    matrix[(row, column)],
                ));
            }
        }
        faer::sparse::SparseColMat::try_new_from_triplets(matrix.nrows(), matrix.ncols(), &entries)
            .unwrap()
    }
    fn key(request: &Request<'_>) -> pse_model::strategy::SemanticProductKey {
        let hash = pse_ids::ContentHash::from_bytes([4; 32]);
        pse_model::strategy::SemanticProductKey {
            structure: request.scope.contract.identity,
            binding: hash,
            numerical_policy: Some(hash),
            normalization: Some(request.normalization.key()),
            point: Some(point_key(request.point)),
            parameters: Some(hash),
            derivation: None,
            branch: Some(hash),
            accuracy: Some(hash),
        }
    }
    fn public_response(
        request: Request<'_>,
        evaluate: impl FnOnce() -> Result<(Mat<f64>, Mat<f64>), ProblemError>,
    ) -> Result<Response, Withheld> {
        let key = key(&request);
        let base = vec![0.; request.parameters.len()];
        let execution = crate::solve::Execution::new(
            pse_diagnostics::Cancellation::default(),
            &crate::solve::Controls::default(),
        );
        products(
            request,
            key,
            &base,
            false,
            || evaluate().map(|(state, parameters)| (sparse(&state), sparse(&parameters))),
            &execution,
        )
        .response
    }
    fn id(n: u8) -> SemanticId {
        SemanticId::from_bytes([n; 16])
    }
    pub(super) fn scope() -> SquareScope {
        let contract = OracleContract {
            identity: pse_ids::ContentHash::from_bytes([1; 32]),
            variables: vec![
                crate::Variable {
                    id: id(1),
                    lower: -100.,
                    upper: 100.,
                },
                crate::Variable {
                    id: id(2),
                    lower: -100.,
                    upper: 100.,
                },
            ],
            rows: vec![id(3), id(4)],
            derivatives: pse_kernels::DerivativeOrder::First,
            smoothness: pse_kernels::DerivativeOrder::First,
        };
        let pairs = [(0, 0), (1, 0), (0, 1), (1, 1)].map(|(r, c)| faer::sparse::Pair::new(r, c));
        let (pattern, _) =
            faer::sparse::SymbolicSparseColMat::try_new_from_indices(2, 2, &pairs).unwrap();
        SquareScope::admit(&contract, pattern.as_ref(), &[(0., 0.); 2], None).unwrap()
    }
    #[test]
    fn square_response_matches_independent_perturbed_roots_in_physical_coordinates() {
        let scope = scope();
        let tolerance = crate::quality::Tolerances {
            variables: vec![1e-8; 2],
            rows: vec![1e-8; 2],
            integrality: 1e-8,
        };
        let scale = pse_math::normalization::Normalization {
            variables: vec![100., 0.01],
            rows: vec![2., 400.],
            objective: 1.,
        };
        let params = [id(5)];
        let req = Request {
            scope: &scope,
            point: &[2., 1.],
            residual_values: &[0., 0.],
            tolerances: &tolerance,
            normalization: &scale,
            parameters: &params,
            parameter_scales: &[1e6],
            rank_tolerance: 1e-12,
            bytes: 1 << 20,
        };
        let result = public_response(req, || {
            Ok((faer::mat![[4., 3.], [1., 1.]], faer::mat![[1.], [-1.]]))
        })
        .unwrap();
        // Solve these original equations analytically at independently changed p:
        // x0^2+p*x1-7=0, x0+x1-p=0; selected x0=(p+sqrt(28-3p^2))/2.
        let root = |p: f64| {
            let x = (p + (28. - 3. * p * p).sqrt()) / 2.;
            [x, p - x]
        };
        let h = 1e-7;
        let hi = root(3. + h);
        let lo = root(3. - h);
        for i in 0..2 {
            assert!((result.values[(i, 0)] - (hi[i] - lo[i]) / (2. * h)).abs() < 2e-7);
        }
        assert_eq!(result.states, [id(1), id(2)]);
        assert_eq!(result.parameters, params);
        assert_eq!(result.evidence.rank, 2);
        assert!(result.evidence.backward_error <= result.evidence.backward_error_limit);
    }
    #[test]
    fn square_response_withholds_rank_bound_guard_branch_and_resource_failures() {
        let scope = scope();
        let tolerance = crate::quality::Tolerances {
            variables: vec![1e-8; 2],
            rows: vec![1e-8; 2],
            integrality: 1e-8,
        };
        let scale = pse_math::normalization::Normalization::identity(2, 2);
        let params = [id(5)];
        let req = || Request {
            scope: &scope,
            point: &[2., 1.],
            residual_values: &[0., 0.],
            tolerances: &tolerance,
            normalization: &scale,
            parameters: &params,
            parameter_scales: &[1.],
            rank_tolerance: 1e-10,
            bytes: 1 << 20,
        };
        assert!(matches!(
            public_response(req(), || Ok((Mat::ones(2, 2), Mat::ones(2, 1)))),
            Err(Withheld::Rank { .. })
        ));
        for detail in ["guard boundary", "unstable branch selector"] {
            assert!(matches!(
                public_response(req(), || Err(ProblemError::unsupported(detail))),
                Err(Withheld::Neighborhood(_))
            ));
        }
        assert!(matches!(
            public_response(req(), || Err(ProblemError::memory(
                "partial callback budget"
            ))),
            Err(Withheld::Memory)
        ));
        let mut r = req();
        r.point = &[100., 1.];
        assert!(matches!(
            public_response(r, || panic!("active bound must refuse before derivatives")),
            Err(Withheld::Neighborhood(_))
        ));
        let mut r = req();
        r.residual_values = &[1., 0.];
        assert!(matches!(
            public_response(r, || panic!("infeasible point")),
            Err(Withheld::Infeasible(_))
        ));
        let mut r = req();
        r.bytes = 1;
        assert!(matches!(
            public_response(r, || panic!("unreserved allocation")),
            Err(Withheld::Memory)
        ));
        let extreme = pse_math::normalization::Normalization {
            variables: vec![1e308; 2],
            rows: vec![1.; 2],
            objective: 1.,
        };
        let mut r = req();
        r.normalization = &extreme;
        r.parameter_scales = &[1e-308];
        assert!(matches!(
            public_response(r, || Ok((
                faer::mat![[1e-308, 0.], [0., 1e-308]],
                Mat::full(2, 1, 1e308)
            ))),
            Err(Withheld::Numerical(_))
        ));
    }
    #[test]
    fn square_scope_requires_original_matching_even_when_counts_equal() {
        let good = scope();
        let pairs = [faer::sparse::Pair::new(0, 0), faer::sparse::Pair::new(1, 0)];
        let (pattern, _) =
            faer::sparse::SymbolicSparseColMat::try_new_from_indices(2, 2, &pairs).unwrap();
        assert!(matches!(
            SquareScope::admit(&good.contract, pattern.as_ref(), &[(0., 0.); 2], None),
            Err(Withheld::Structural(_))
        ));
    }
    #[derive(Debug, Default)]
    struct ConstructionCount(std::sync::atomic::AtomicU64);
    impl crate::solve::WorkAdmission for ConstructionCount {
        fn admit(&self, _: crate::solve::WorkEvidence) -> Result<(), ProblemError> {
            Ok(())
        }
        fn observe(&self, work: crate::solve::WorkEvidence) -> Result<(), ProblemError> {
            self.0.fetch_add(
                work.factorizations.unwrap_or(0),
                std::sync::atomic::Ordering::Relaxed,
            );
            Ok(())
        }
    }
    #[test]
    fn strict_products_refuse_unknown_sparse_fill_before_shared_oracle() {
        #[derive(Debug, Default)]
        struct StrictStorage {
            storage: std::sync::atomic::AtomicUsize,
            work: std::sync::atomic::AtomicUsize,
        }
        impl crate::solve::WorkAdmission for StrictStorage {
            fn admit(&self, _: crate::solve::WorkEvidence) -> Result<(), ProblemError> {
                self.work.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                Ok(())
            }
            fn observe(&self, _: crate::solve::WorkEvidence) -> Result<(), ProblemError> {
                Ok(())
            }
            fn admit_storage(
                &self,
                scope: crate::solve::NativeStorageScope,
                _: &str,
                _: usize,
                opaque: bool,
            ) -> Result<(), ProblemError> {
                assert_eq!(scope, crate::solve::NativeStorageScope::Linear);
                assert!(opaque);
                self.storage
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                Err(ProblemError::Unsupported(
                    "strict linear storage requires a complete producer-owned bound".into(),
                ))
            }
        }
        let scope = scope();
        let tolerance = crate::quality::Tolerances {
            variables: vec![1e-8; 2],
            rows: vec![1e-8; 2],
            integrality: 1e-8,
        };
        let normalization = pse_math::normalization::Normalization::identity(2, 2);
        let request = Request {
            scope: &scope,
            point: &[2., 1.],
            residual_values: &[0., 0.],
            tolerances: &tolerance,
            normalization: &normalization,
            parameters: &[id(5)],
            parameter_scales: &[1.],
            rank_tolerance: 1e-12,
            bytes: 1 << 20,
        };
        let source = key(&request);
        let evaluated = std::cell::Cell::new(0);
        let admission = std::sync::Arc::new(StrictStorage::default());
        let mut execution = crate::solve::Execution::new(
            pse_diagnostics::Cancellation::default(),
            &crate::solve::Controls::default(),
        );
        execution.work_admission = Some(admission.clone());
        let result = products(
            request,
            source,
            &[3.],
            true,
            || {
                evaluated.set(evaluated.get() + 1);
                Ok((
                    sparse(&faer::mat![[4., 3.], [1., 1.]]),
                    sparse(&faer::mat![[1.], [-1.]]),
                ))
            },
            &execution,
        );
        assert!(
            matches!(result.response, Err(Withheld::Cause(cause)) if matches!(&*cause, ProblemError::Unsupported(_)))
        );
        assert!(
            matches!(result.predictor.unwrap(), Err(Withheld::Cause(cause)) if matches!(&*cause, ProblemError::Unsupported(_)))
        );
        assert_eq!(evaluated.get(), 0);
        assert_eq!(
            admission.storage.load(std::sync::atomic::Ordering::Relaxed),
            1
        );
        assert_eq!(admission.work.load(std::sync::atomic::Ordering::Relaxed), 0);
    }
    #[test]
    fn same_point_products_evaluate_once_and_share_factor_for_multi_rhs_and_prediction() {
        let scope = scope();
        let tolerance = crate::quality::Tolerances {
            variables: vec![1e-8; 2],
            rows: vec![1e-8; 2],
            integrality: 1e-8,
        };
        let normalization = pse_math::normalization::Normalization {
            variables: vec![100., 0.01],
            rows: vec![2., 400.],
            objective: 1.,
        };
        let parameters = [id(5), id(6)];
        let request = Request {
            scope: &scope,
            point: &[2., 1.],
            residual_values: &[0., 0.],
            tolerances: &tolerance,
            normalization: &normalization,
            parameters: &parameters,
            parameter_scales: &[1e6, 0.25],
            rank_tolerance: 1e-12,
            bytes: 1 << 20,
        };
        let source = key(&request);
        let constructed = std::cell::Cell::new(0);
        let counts = std::sync::Arc::new(ConstructionCount::default());
        let mut execution = crate::solve::Execution::new(
            pse_diagnostics::Cancellation::default(),
            &crate::solve::Controls::default(),
        );
        execution.work_admission = Some(counts.clone());
        let product = products(
            request,
            source,
            &[3., 7.],
            true,
            || {
                constructed.set(constructed.get() + 1);
                Ok((
                    sparse(&faer::mat![[4., 3.], [1., 1.]]),
                    sparse(&faer::mat![[1., 2.], [-1., 3.]]),
                ))
            },
            &execution,
        );
        let response = product.response.unwrap();
        let expected = faer::mat![[-4., 7.], [5., -10.]];
        for column in 0..2 {
            for row in 0..2 {
                assert!((response.values[(row, column)] - expected[(row, column)]).abs() < 1e-11);
            }
        }
        let predictor = product.predictor.unwrap().unwrap();
        let (direction, evidence) = predictor
            .action(&[(id(5), 3.01), (id(6), 7.02)], &execution)
            .unwrap();
        assert!((direction[0] - 0.1).abs() < 1e-11);
        assert!((direction[1] + 0.15).abs() < 1e-11);
        assert!(evidence.backward_error <= evidence.limit);
        assert_eq!(constructed.get(), 1);
        assert_eq!(counts.0.load(std::sync::atomic::Ordering::Relaxed), 1);
        assert_eq!(predictor.factor().key(), source);
    }
    #[test]
    fn singular_value_withholding_is_independent_of_predictor_and_retention_demand() {
        let scope = scope();
        let tolerance = crate::quality::Tolerances {
            variables: vec![1e-8; 2],
            rows: vec![1e-8; 2],
            integrality: 1e-8,
        };
        let normalization = pse_math::normalization::Normalization::identity(2, 2);
        let parameters = [id(5)];
        let request = || Request {
            scope: &scope,
            point: &[2., 1.],
            residual_values: &[0., 0.],
            tolerances: &tolerance,
            normalization: &normalization,
            parameters: &parameters,
            parameter_scales: &[1.],
            rank_tolerance: 1e-8,
            bytes: 1 << 20,
        };
        let evaluation = || {
            Ok((
                sparse(&faer::mat![[1., 0.], [0., 2f64.powi(-30)]]),
                sparse(&faer::mat![[1.], [0.]]),
            ))
        };
        let execution = crate::solve::Execution::new(
            pse_diagnostics::Cancellation::default(),
            &crate::solve::Controls::default(),
        );
        let product = products(
            request(),
            key(&request()),
            &[3.],
            true,
            evaluation,
            &execution,
        );
        assert!(matches!(
            product.response,
            Err(Withheld::Rank {
                rank: 1,
                dimension: 2,
                ..
            })
        ));
        let predictor = product.predictor.unwrap().unwrap();
        assert_eq!(
            predictor.action(&[(id(5), 4.)], &execution).unwrap().0,
            [-1., 0.]
        );
        let product = products(
            request(),
            key(&request()),
            &[3.],
            false,
            evaluation,
            &execution,
        );
        assert!(matches!(product.response, Err(Withheld::Rank { .. })));
        assert!(product.predictor.is_none());
        let product = products(
            request(),
            key(&request()),
            &[3.],
            true,
            || Ok((sparse(&Mat::ones(2, 2)), sparse(&Mat::ones(2, 1)))),
            &execution,
        );
        assert!(matches!(
            product.response,
            Err(Withheld::Rank { rank: 1, .. })
        ));
        assert!(match product.predictor.unwrap() {
            Err(_) => true,
            Ok(predictor) => predictor.action(&[(id(5), 4.)], &execution).is_err(),
        });
    }
    #[test]
    fn same_point_products_refuse_changed_keys_scales_support_and_provider_branch() {
        let scope = scope();
        let tolerance = crate::quality::Tolerances {
            variables: vec![1e-8; 2],
            rows: vec![1e-8; 2],
            integrality: 1e-8,
        };
        let normalization = pse_math::normalization::Normalization::identity(2, 2);
        let parameters = [id(5)];
        let request = || Request {
            scope: &scope,
            point: &[2., 1.],
            residual_values: &[0., 0.],
            tolerances: &tolerance,
            normalization: &normalization,
            parameters: &parameters,
            parameter_scales: &[1.],
            rank_tolerance: 1e-10,
            bytes: 1 << 20,
        };
        let source = key(&request());
        for changed in [
            pse_model::strategy::SemanticProductKey {
                point: Some(point_key(&[2., -0.])),
                ..source
            },
            pse_model::strategy::SemanticProductKey {
                normalization: Some(pse_ids::ContentHash::from_bytes([9; 32])),
                ..source
            },
            pse_model::strategy::SemanticProductKey {
                structure: pse_ids::ContentHash::from_bytes([9; 32]),
                ..source
            },
            pse_model::strategy::SemanticProductKey {
                accuracy: None,
                ..source
            },
        ] {
            let execution = crate::solve::Execution::new(
                pse_diagnostics::Cancellation::default(),
                &crate::solve::Controls::default(),
            );
            let product = products(
                request(),
                changed,
                &[3.],
                true,
                || panic!("foreign context must refuse before partial construction"),
                &execution,
            );
            assert!(product.response.is_err());
            assert!(product.predictor.unwrap().is_err());
        }
        let execution = crate::solve::Execution::new(
            pse_diagnostics::Cancellation::default(),
            &crate::solve::Controls::default(),
        );
        let diagonal = faer::sparse::SparseColMat::try_new_from_triplets(
            2,
            2,
            &[
                faer::sparse::Triplet::new(0, 0, 1.),
                faer::sparse::Triplet::new(1, 1, 1.),
            ],
        )
        .unwrap();
        let product = products(
            request(),
            source,
            &[3.],
            true,
            || Ok((diagonal, sparse(&Mat::ones(2, 1)))),
            &execution,
        );
        assert!(matches!(product.response, Err(Withheld::Structural(_))));
        assert!(matches!(
            product.predictor.unwrap(),
            Err(Withheld::Structural(_))
        ));
        let product = products(
            request(),
            source,
            &[3.],
            true,
            || {
                Err(ProblemError::unsupported(
                    "provider branch is outside its qualified neighborhood",
                ))
            },
            &execution,
        );
        assert!(matches!(product.response, Err(Withheld::Neighborhood(_))));
        assert!(matches!(
            product.predictor.unwrap(),
            Err(Withheld::Neighborhood(_))
        ));
    }
}
