// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Qualified regular-square response, distinct from optimization KKT analysis.
use crate::{OracleContract, ProblemError};
use faer::Mat;
use pse_ids::SemanticId;

mod actions;
pub use actions::{ActionEvidence, SparseFactor, SparsePredictor, SparseRequest, point_key};

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
}
impl SquareScope {
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
/// Peak declared matrix and library scratch capacity, checked before allocation.
pub fn workspace_bytes(n: usize, np: usize) -> Option<usize> {
    use faer::linalg::temp_mat_scratch;
    let scratch = rank_scratch(n)
        .size_bytes()
        .max(response_scratch(n, np).size_bytes());
    temp_mat_scratch::<f64>(n, n)
        .size_bytes()
        .checked_mul(3)
        .and_then(|b| b.checked_add(temp_mat_scratch::<f64>(n, np).size_bytes().checked_mul(5)?))
        .and_then(|b| b.checked_add(n.checked_mul(4 * size_of::<usize>())?))
        .and_then(|b| b.checked_add(scratch))
        .and_then(|b| b.checked_add(n.checked_add(np)?.checked_mul(size_of::<SemanticId>())?))
        .and_then(|b| b.checked_add(n.checked_mul(2 * size_of::<f64>())?))
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
/// Solve `F_x X_p = -F_p`, then back-map to physical coordinates.
/// # Errors
/// Original feasibility/structure/neighborhood, rank, resource or backward-error failure.
pub fn response(
    request: Request<'_>,
    evaluate: impl FnOnce() -> Result<(Mat<f64>, Mat<f64>), ProblemError>,
) -> Result<Response, Withheld> {
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
    let matrices = workspace_bytes(n, np)
        .filter(|b| *b <= request.bytes)
        .ok_or(Withheld::Memory)?;
    let (fx, fp) = evaluate().map_err(partial_failure)?;
    if fx.nrows() != n
        || fx.ncols() != n
        || fp.nrows() != n
        || fp.ncols() != np
        || fx
            .col_iter()
            .chain(fp.col_iter())
            .any(|c| c.iter().any(|v| !v.is_finite()))
    {
        return Err(Withheld::Numerical(
            "physical Jacobian dimensions or values".into(),
        ));
    }
    let a = Mat::from_fn(n, n, |i, j| {
        fx[(i, j)] * request.normalization.variables[j] / request.normalization.rows[i]
    });
    let b = Mat::from_fn(n, np, |i, j| {
        -fp[(i, j)] * request.parameter_scales[j] / request.normalization.rows[i]
    });
    let spectrum = spectrum(&a, matrices).map_err(|e| Withheld::Numerical(e.to_string()))?;
    let cutoff = spectrum.first().copied().unwrap_or(0.) * request.rank_tolerance;
    let rank = spectrum.iter().filter(|s| **s > cutoff).count();
    if rank != n {
        return Err(Withheld::Rank {
            rank,
            dimension: n,
            cutoff,
        });
    }
    let x =
        solve_regular(&a, b.clone(), matrices).map_err(|e| Withheld::Numerical(e.to_string()))?;
    let backward_error =
        check_response(&a, &x, &b).map_err(|e| Withheld::Numerical(e.to_string()))?;
    let values = Mat::from_fn(n, np, |i, j| {
        x[(i, j)] * request.normalization.variables[i] / request.parameter_scales[j]
    });
    if values
        .col_iter()
        .any(|column| column.iter().any(|v| !v.is_finite()))
    {
        return Err(Withheld::Numerical(
            "nonfinite response in physical coordinates".into(),
        ));
    }
    Ok(Response {
        owner: None,
        states: c.variables.iter().map(|v| v.id).collect(),
        parameters: request.parameters.to_vec(),
        values,
        evidence: Evidence {
            rank,
            cutoff,
            relative_cutoff: request.rank_tolerance,
            backward_error,
            backward_error_limit: 64. * n.max(1) as f64 * f64::EPSILON,
            neighborhood: "compiled-first-partials/guard-selector-interior; inactive-physical-state-bounds",
        },
    })
}
fn partial_failure(cause: ProblemError) -> Withheld {
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
        || matches!(&cause, ProblemError::Math(cause) if memory(cause))
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
/// Library-declared LU/backsolve scratch for response capacity planning.
pub fn response_scratch(n: usize, np: usize) -> faer::dyn_stack::StackReq {
    use faer::{
        Par,
        linalg::lu::partial_pivoting::{factor, solve},
    };
    factor::lu_in_place_scratch::<usize, f64>(n, n, Par::Seq, Default::default())
        .or(solve::solve_in_place_scratch::<usize, f64>(n, np, Par::Seq))
}
fn solve_regular(a: &Mat<f64>, mut rhs: Mat<f64>, limit: usize) -> Result<Mat<f64>, ProblemError> {
    use faer::{
        Par,
        dyn_stack::{MemBuffer, MemStack},
        linalg::lu::partial_pivoting::{factor, solve},
    };
    let n = a.nrows();
    let mut lu = a.clone();
    let mut perm = vec![0usize; n];
    let mut inverse = vec![0usize; n];
    let req = response_scratch(n, rhs.ncols());
    if req.size_bytes() > limit {
        return Err(ProblemError::memory("response solve scratch allowance"));
    }
    let mut memory = MemBuffer::try_new(req).map_err(|e| ProblemError::memory(e.to_string()))?;
    let (_, permutation) = factor::lu_in_place(
        lu.as_mut(),
        &mut perm,
        &mut inverse,
        Par::Seq,
        MemStack::new(&mut memory),
        Default::default(),
    );
    solve::solve_in_place(
        lu.as_ref(),
        lu.as_ref(),
        permutation,
        rhs.as_mut(),
        Par::Seq,
        MemStack::new(&mut memory),
    );
    if rhs
        .as_ref()
        .col_iter()
        .any(|c| c.iter().any(|v| !v.is_finite()))
    {
        return Err(ProblemError::numerical("nonfinite implicit response solve"));
    }
    Ok(rhs)
}
/// Normwise backward error in the same scaled coordinates used for rank admission.
fn check_response(a: &Mat<f64>, x: &Mat<f64>, b: &Mat<f64>) -> Result<f64, ProblemError> {
    let mut residual = -b;
    faer::linalg::matmul::matmul(
        residual.as_mut(),
        faer::Accum::Add,
        a.as_ref(),
        x.as_ref(),
        1.0,
        faer::Par::Seq,
    );
    let numerator = residual.as_ref().norm_l2();
    let denominator = a.as_ref().norm_l2() * x.as_ref().norm_l2() + b.as_ref().norm_l2();
    let error_bound = 64.0 * a.nrows().max(1) as f64 * f64::EPSILON;
    let backward_error = if denominator == 0.0 {
        numerator
    } else {
        numerator / denominator
    };
    if !denominator.is_finite() || !backward_error.is_finite() || backward_error > error_bound {
        return Err(ProblemError::numerical(format!(
            "implicit response backward error {backward_error} exceeds {error_bound}"
        )));
    }
    Ok(backward_error)
}

#[cfg(test)]
mod tests {
    use super::*;
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
        let result = response(req, || {
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
            response(req(), || Ok((Mat::ones(2, 2), Mat::ones(2, 1)))),
            Err(Withheld::Rank { .. })
        ));
        for detail in ["guard boundary", "unstable branch selector"] {
            assert!(matches!(
                response(req(), || Err(ProblemError::unsupported(detail))),
                Err(Withheld::Neighborhood(_))
            ));
        }
        assert!(matches!(
            response(req(), || Err(ProblemError::memory(
                "partial callback budget"
            ))),
            Err(Withheld::Memory)
        ));
        let mut r = req();
        r.point = &[100., 1.];
        assert!(matches!(
            response(r, || panic!("active bound must refuse before derivatives")),
            Err(Withheld::Neighborhood(_))
        ));
        let mut r = req();
        r.residual_values = &[1., 0.];
        assert!(matches!(
            response(r, || panic!("infeasible point")),
            Err(Withheld::Infeasible(_))
        ));
        let mut r = req();
        r.bytes = 1;
        assert!(matches!(
            response(r, || panic!("unreserved allocation")),
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
            response(r, || Ok((
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
    #[test]
    fn square_response_backward_error_rejects_an_incorrect_solve() {
        let a = faer::mat![[2., 1.], [1., 3.]];
        let b = faer::mat![[4.], [7.]];
        let x = solve_regular(&a, b.clone(), 1 << 20).unwrap();
        assert!(check_response(&a, &x, &b).unwrap() < 1e-14);
        assert!(check_response(&a, &Mat::ones(2, 1), &b).is_err());
        assert!(check_response(&a, &faer::mat![[f64::NAN], [1.]], &b).is_err());
    }
}
