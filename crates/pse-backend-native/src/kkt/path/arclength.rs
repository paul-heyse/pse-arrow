// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Scoped sparse bordering and oriented tangents. Admitted native libraries own
//! nonlinear correction; numeric turning indicators do not certify folds.
use crate::{ProblemError, quality, solve::Execution};
use faer::{
    Conj, Mat, Par,
    dyn_stack::{MemBuffer, MemStack},
    sparse::{
        SparseColMat,
        linalg::lu::{self, LuRef, NumericLu},
    },
};
use pse_ids::ContentHash;
use pse_math::{
    continuation::{ArclengthFamily, BoundArclength, ParameterizedOracle},
    sparse::AssemblyMatrix,
};
use std::sync::Arc;

/// Explicit allowances for actual action/factor work and opaque library storage.
#[derive(Clone, Copy, Debug)]
pub struct Limits {
    /// Maximum original state and parameter derivative calls for this operation.
    pub actions: usize,
    /// Finite storage reservation, not measured private library allocation.
    pub bytes: usize,
}
/// Actual operations attempted, including work before a terminal stop.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Work {
    /// Original value callback calls.
    pub values: u64,
    /// Original state-partial calls.
    pub state_actions: u64,
    /// Original scalar parameter-partial calls.
    pub parameter_actions: u64,
    /// Optional original state-curvature calls.
    pub curvature_actions: u64,
    /// Library numeric factorization calls.
    pub factorizations: u64,
    /// Library bordered backsolves.
    pub backsolves: u64,
    /// Library dense SVD calls, only on explicit event probe requests.
    pub rank_probes: u64,
}
/// Original typed cause plus actual work; no failure discards already performed work.
#[derive(Clone, Debug)]
pub struct Failure {
    /// Unmodified original scope/provider/math/library boundary failure.
    pub cause: Arc<ProblemError>,
    /// Actual calls made before stopping.
    pub work: Work,
}
/// Qualified original point and explicit normalized orientation history.
#[derive(Debug)]
pub struct Request<'a> {
    /// Immutable admitted original/parameter family.
    pub family: &'a Arc<ArclengthFamily>,
    /// Physical original states followed by the scalar parameter.
    pub point: &'a [f64],
    /// Unit normalized prior tangent (or explicit starting parameter direction).
    pub previous: &'a [f64],
    /// Complete accepted path/sheet/transport and point source identity.
    pub source: ContentHash,
    /// Required original physical residual budgets, one per original row.
    pub residual_limits: &'a [f64],
    /// Required scaled bordered residual backward-error limit.
    pub backward_limit: f64,
    /// Explicit finite operation/storage allowance.
    pub limits: Limits,
}
/// An estimated oriented tangent. This proposes a connected predictor; it does not
/// establish an original qualified result or a numerical-rank certificate.
#[derive(Clone, Debug)]
pub struct Tangent {
    /// Physical source point in augmented family coordinates.
    pub point: Vec<f64>,
    /// Unit tangent in normalized augmented coordinates.
    pub normalized: Vec<f64>,
    /// Physical-coordinate tangent for the same unit arclength displacement.
    pub physical: Vec<f64>,
    /// Complete family identity actually consumed.
    pub family: ContentHash,
    /// Consumed accepted history/path/transport source.
    pub source: ContentHash,
    /// Positive normalized overlap with the supplied orientation.
    pub orientation: f64,
    /// Estimated normwise bordered solve backward error.
    pub backward_error: f64,
    /// Required error limit actually checked.
    pub backward_limit: f64,
    /// Actual source/linear calls.
    pub work: Work,
}
fn validate<O: ParameterizedOracle<Error = ProblemError>>(
    request: &Request<'_>,
    oracle: &O,
) -> Result<usize, ProblemError> {
    request.family.validate_point(request.point)?;
    let d = request.point.len();
    if !request.family.action_available()
        || oracle.contract() != request.family.original()
        || oracle.parameter() != request.family.parameter()
    {
        return Err(ProblemError::Unsupported(
            "arclength original/parameter First supplier mismatch".into(),
        ));
    }
    if request.previous.len() != d
        || request.previous.iter().any(|v| !v.is_finite())
        || (request.previous.iter().map(|v| v * v).sum::<f64>() - 1.).abs()
            > 256. * d as f64 * f64::EPSILON
        || request.residual_limits.len() != d - 1
        || request
            .residual_limits
            .iter()
            .any(|v| !v.is_finite() || *v <= 0.)
        || !request.backward_limit.is_finite()
        || request.backward_limit <= 0.
        || request.limits.bytes == 0
    {
        return Err(ProblemError::Contract("arclength requires explicit unit orientation, physical accuracy and finite action/storage limits".into()));
    }
    if request.limits.actions < d {
        return Err(ProblemError::Limit {
            kind: crate::LimitKind::Work,
            detail: "arclength derivative action allowance".into(),
        });
    }
    Ok(d)
}
fn sparse_error(error: faer::sparse::FaerError) -> ProblemError {
    match error {
        faer::sparse::FaerError::OutOfMemory => {
            ProblemError::memory("arclength sparse library allocation failed")
        }
        faer::sparse::FaerError::IndexOverflow => {
            ProblemError::Contract("arclength sparse library index overflow".into())
        }
        other => ProblemError::internal(format!("arclength sparse library failure: {other:?}")),
    }
}
fn lu_error(error: faer::sparse::linalg::LuError) -> ProblemError {
    match error {
        faer::sparse::linalg::LuError::SymbolicSingular { index } => {
            ProblemError::numerical(format!("arclength symbolic rank loss at pivot {index}"))
        }
        faer::sparse::linalg::LuError::Generic(error) => sparse_error(error),
    }
}
fn assemble<O: ParameterizedOracle<Error = ProblemError>>(
    request: &Request<'_>,
    oracle: &mut O,
    execution: &Execution,
    work: &mut Work,
) -> Result<SparseColMat<usize, f64>, ProblemError> {
    execution.check()?;
    let d = validate(request, oracle)?;
    let n = d - 1;
    let reservation = d
        .checked_mul(d)
        .and_then(|v| v.checked_mul(64))
        .and_then(|v| d.checked_mul(512).and_then(|a| v.checked_add(a)))
        .and_then(|v| v.checked_add(4096))
        .ok_or_else(|| ProblemError::memory("arclength factor allowance overflow"))?;
    if reservation > request.limits.bytes {
        return Err(ProblemError::memory(
            "arclength finite library factor allowance",
        ));
    }
    let pattern = AssemblyMatrix::new(d, d, request.family.incidence(), i32::MAX as usize)?;
    let symbolic = pattern
        .matrix()
        .symbolic()
        .to_owned()
        .map_err(|e| ProblemError::memory(format!("arclength sparse pattern: {e:?}")))?;
    let mut values = vec![0.; symbolic.row_idx().len()];
    let mut observed = vec![0.; n];
    work.values += 1;
    oracle.values(&request.point[..n], request.point[n], &mut observed)?;
    execution.check()?;
    if observed
        .iter()
        .zip(request.residual_limits)
        .any(|(v, t)| !v.is_finite() || v.abs() > *t)
    {
        return Err(ProblemError::numerical(
            "arclength source fails its required original residual budget",
        ));
    }
    let scales = request.family.coordinate_scales();
    let rows = request.family.row_scales();
    let mut direction = vec![0.; n];
    let mut action = vec![0.; n];
    for column in 0..d {
        execution.check()?;
        action.fill(0.);
        if column < n {
            direction[column] = 1.;
            work.state_actions += 1;
            oracle.state_action(
                &request.point[..n],
                request.point[n],
                &direction,
                &mut action,
            )?;
            direction[column] = 0.;
        } else {
            work.parameter_actions += 1;
            oracle.parameter_action(&request.point[..n], request.point[n], 1., &mut action)?;
        }
        execution.check()?;
        if action.iter().any(|v| !v.is_finite()) {
            return Err(ProblemError::numerical(
                "nonfinite arclength derivative action",
            ));
        }
        for (row, value) in action.iter().enumerate() {
            if *value != 0. && !symbolic.row_idx_of_col(column).any(|r| r == row) {
                return Err(ProblemError::Contract(
                    "arclength derivative escapes declared structural support".into(),
                ));
            }
        }
        for index in symbolic.col_range(column) {
            let row = symbolic.row_idx()[index];
            values[index] = if row == n {
                request.previous[column]
            } else {
                action[row] * scales[column] / rows[row]
            };
        }
    }
    if values.iter().any(|v| !v.is_finite()) {
        return Err(ProblemError::numerical(
            "nonfinite normalized arclength derivative transport",
        ));
    }
    Ok(SparseColMat::new(symbolic, values))
}
/// Solve the actual scaled sparse border and orient its unit tangent against the
/// supplied history. Checkpoints preserve the original absolute scope.
/// # Errors
/// Invalid/unqualified point, missing actual actions, library failure or terminal scope.
pub fn tangent<O: ParameterizedOracle<Error = ProblemError>>(
    request: Request<'_>,
    oracle: &mut O,
    execution: &Execution,
) -> Result<Tangent, Failure> {
    let mut work = Work::default();
    let result = quality::contained(|| {
        let matrix = assemble(&request, oracle, execution, &mut work)?;
        let d = matrix.ncols();
        let symbolic = lu::factorize_symbolic_lu(matrix.symbolic(), Default::default())
            .map_err(sparse_error)?;
        let factor_req = symbolic.factorize_numeric_lu_scratch::<f64>(Par::Seq, Default::default());
        let solve_req = symbolic.solve_in_place_scratch::<f64>(1, Par::Seq);
        let req = factor_req.or(solve_req);
        if req.size_bytes() > request.limits.bytes / 2 {
            return Err(ProblemError::memory("arclength LU scratch allowance"));
        }
        let mut memory =
            MemBuffer::try_new(req).map_err(|e| ProblemError::memory(e.to_string()))?;
        let mut numeric = NumericLu::new();
        execution.check()?;
        work.factorizations += 1;
        symbolic
            .factorize_numeric_lu(
                &mut numeric,
                matrix.as_ref(),
                Par::Seq,
                MemStack::new(&mut memory),
                Default::default(),
            )
            .map_err(lu_error)?;
        execution.check()?;
        let mut x = Mat::zeros(d, 1);
        x[(d - 1, 0)] = 1.;
        work.backsolves += 1;
        LuRef::new_unchecked(&symbolic, &numeric).solve_in_place_with_conj(
            Conj::No,
            x.as_mut(),
            Par::Seq,
            MemStack::new(&mut memory),
        );
        execution.check()?;
        if x.col(0).iter().any(|v| !v.is_finite()) {
            return Err(ProblemError::numerical(
                "singular or nonfinite arclength bordered tangent",
            ));
        }
        let mut residual = vec![0.; d];
        residual[d - 1] = -1.;
        let mut row_norms = vec![0.; d];
        for column in 0..d {
            for index in matrix.symbolic().col_range(column) {
                let row = matrix.symbolic().row_idx()[index];
                residual[row] += matrix.val()[index] * x[(column, 0)];
                row_norms[row] += matrix.val()[index].abs();
            }
        }
        let x_norm = x.col(0).iter().map(|v| v.abs()).fold(0., f64::max);
        let a_norm = row_norms.into_iter().fold(0., f64::max);
        let denominator = a_norm * x_norm + 1.;
        if !denominator.is_finite() {
            return Err(ProblemError::numerical(
                "nonfinite arclength backward-error scaling",
            ));
        }
        let error = residual.into_iter().map(f64::abs).fold(0., f64::max) / denominator;
        if !error.is_finite() || error > request.backward_limit {
            return Err(ProblemError::numerical(
                "arclength bordered backward-error requirement",
            ));
        }
        let norm = x.col(0).norm_l2();
        if !norm.is_finite() || norm == 0. {
            return Err(ProblemError::numerical(
                "arclength zero or nonfinite tangent norm",
            ));
        }
        let mut normalized = (0..d).map(|i| x[(i, 0)] / norm).collect::<Vec<_>>();
        let mut orientation = normalized
            .iter()
            .zip(request.previous)
            .map(|(a, b)| a * b)
            .sum::<f64>();
        if orientation < 0. {
            for v in &mut normalized {
                *v = -*v;
            }
            orientation = -orientation;
        }
        if !orientation.is_finite() || orientation <= 0. {
            return Err(ProblemError::numerical("arclength orientation unresolved"));
        }
        let physical = normalized
            .iter()
            .zip(request.family.coordinate_scales())
            .map(|(v, s)| v * s)
            .collect::<Vec<_>>();
        if physical.iter().any(|v| !v.is_finite()) {
            return Err(ProblemError::numerical(
                "nonfinite physical arclength tangent",
            ));
        }
        Ok(Tangent {
            point: request.point.to_vec(),
            normalized,
            physical,
            family: request.family.key(),
            source: request.source,
            orientation,
            backward_error: error,
            backward_limit: request.backward_limit,
            work: Work::default(),
        })
    });
    match result {
        Ok(mut result) => {
            result.work = work;
            Ok(result)
        }
        Err(cause) => Err(Failure {
            cause: Arc::new(cause),
            work,
        }),
    }
}
/// Actual native root view for a fixed augmented hyperplane. It preserves the same
/// supplier rather than implementing an independent Newton/corrector algorithm.
#[derive(Debug)]
pub struct Corrector<O> {
    binding: BoundArclength<O>,
    contract: crate::OracleContract,
    pattern: AssemblyMatrix,
    bounds: Vec<(f64, f64)>,
}
impl<O: ParameterizedOracle<Error = ProblemError>> Corrector<O> {
    /// Freeze the real family coordinates and sparse support for an admitted root backend.
    pub fn new(
        binding: BoundArclength<O>,
        smoothness: pse_kernels::DerivativeOrder,
    ) -> Result<Self, ProblemError> {
        if smoothness < pse_kernels::DerivativeOrder::First {
            return Err(ProblemError::Unsupported(
                "arclength corrector requires admitted original smoothness".into(),
            ));
        }
        let family = binding.family();
        let d = family.coordinates().len();
        let pattern = AssemblyMatrix::new(d, d, family.incidence(), i32::MAX as usize)?;
        let contract = crate::OracleContract {
            identity: binding.key(),
            variables: family
                .coordinates()
                .into_iter()
                .map(|v| crate::Variable {
                    id: v.id,
                    lower: v.lower,
                    upper: v.upper,
                })
                .collect(),
            rows: family.constraints().into_iter().map(|r| r.id).collect(),
            derivatives: pse_kernels::DerivativeOrder::First,
            smoothness,
        };
        contract.square()?;
        Ok(Self {
            binding,
            contract,
            pattern,
            bounds: vec![(0., 0.); d],
        })
    }
    /// Recover the original actual supplier and correspondence for assessment/rebinding.
    pub fn into_binding(self) -> BoundArclength<O> {
        self.binding
    }
}
impl<O: ParameterizedOracle<Error = ProblemError>> crate::NleOracle for Corrector<O> {
    fn operations(&self) -> crate::RootOperations {
        crate::RootOperations {
            jacobian: true,
            jacobian_product: true,
        }
    }
    fn contract(&self) -> &crate::OracleContract {
        &self.contract
    }
    fn residual(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        self.binding.residual(x, out)
    }
    fn jacobian_pattern(&self) -> faer::sparse::SymbolicSparseColMatRef<'_, usize> {
        self.pattern.matrix().symbolic()
    }
    fn jacobian(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        let pattern = self
            .pattern
            .matrix()
            .symbolic()
            .to_owned()
            .map_err(|e| ProblemError::memory(format!("arclength corrector pattern: {e:?}")))?;
        if x.len() != self.contract.variables.len() || out.len() != pattern.row_idx().len() {
            return Err(ProblemError::Contract(
                "arclength corrector point/sparse value shape".into(),
            ));
        }
        let mut result = vec![0.; out.len()];
        let mut direction = vec![0.; x.len()];
        let mut action = vec![0.; x.len()];
        for column in 0..x.len() {
            direction[column] = 1.;
            self.binding.action(x, &direction, &mut action)?;
            direction[column] = 0.;
            for index in pattern.col_range(column) {
                result[index] = action[pattern.row_idx()[index]];
            }
        }
        out.copy_from_slice(&result);
        Ok(())
    }
    fn jacobian_product(
        &mut self,
        x: &[f64],
        direction: &[f64],
        out: &mut [f64],
    ) -> Result<(), ProblemError> {
        self.binding.action(x, direction, out)
    }
}

/// Actual optional state curvature supplier, separate from First action/full jet claims.
pub trait StateCurvature: std::fmt::Debug {
    /// Exact original inventory consumed by this second-action supplier.
    fn contract(&self) -> &pse_math::derived::OriginalContract;
    /// Exact scalar parameter declaration consumed by this supplier.
    fn parameter(&self) -> &pse_math::continuation::Parameter;
    /// Exact admitted second-action program/provider identity.
    fn source(&self) -> ContentHash;
    /// Original physical `F_xx[direction,direction]`, at fixed scalar parameter.
    fn action(
        &mut self,
        state: &[f64],
        parameter: f64,
        direction: &[f64],
        out: &mut [f64],
    ) -> Result<(), ProblemError>;
}
/// Explicit event localization supplied by the path owner, not inferred from endpoint feasibility.
#[derive(Clone, Copy, Debug)]
pub struct Localization {
    /// Accepted bracketing path observations.
    pub left: ContentHash,
    /// Accepted opposite endpoint observation.
    pub right: ContentHash,
    /// Closed oriented arclength interval, in declared normalized path units.
    pub interval: (f64, f64),
    /// Event coordinate actually localized inside that interval.
    pub at: f64,
}
/// Additional explicit allowance for optional dense numerical rank probes.
#[derive(Clone, Copy, Debug)]
pub struct ProbeLimits {
    /// Finite dense SVD/storage reservation.
    pub bytes: usize,
    /// Maximum library SVD calls for this event probe.
    pub svds: usize,
}
/// Explicit source localization, numerical interpretation thresholds and finite probe allowance.
#[derive(Clone, Copy, Debug)]
pub struct ProbePolicy {
    /// Accepted event localization supplied by the path owner.
    pub localization: Option<Localization>,
    /// Absolute numerical rank threshold.
    pub rank_threshold: f64,
    /// Absolute transversality and curvature threshold.
    pub nondegeneracy_threshold: f64,
    /// Finite library rank-probe allowance.
    pub limits: ProbeLimits,
}
/// Numerical event interpretation; all rank evidence is estimated at explicit thresholds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventKind {
    /// Full state rank at this point; any turning observation remains an indicator.
    Regular,
    /// Localized rank-one loss, regular augmentation and observed transversality/curvature.
    SimpleFold,
    /// Missing localization/support or failed nondegeneracy; not a certified fold/branch point.
    Unresolved,
}
/// Copied actual numerical event observations, including unavailable support.
#[derive(Clone, Debug)]
pub struct Event {
    /// Conditional interpretation of the observations, never a formal certificate.
    pub kind: EventKind,
    /// Actual family/point/history consumption.
    pub source: ContentHash,
    /// Complete family identity actually consumed by the numerical probe.
    pub family: ContentHash,
    /// Actual physical event point, in the family's augmented coordinates.
    pub point: Vec<f64>,
    /// Localized bracket, when supplied by the path owner.
    pub localization: Option<Localization>,
    /// Estimated numerical rank of F_x in declared normalized coordinates.
    pub state_rank: usize,
    /// Estimated numerical rank of `[F_x,F_p]`.
    pub augmented_rank: usize,
    /// Actual descending singular values of F_x.
    pub state_singular_values: Vec<f64>,
    /// Actual descending singular values of `[F_x,F_p]`.
    pub augmented_singular_values: Vec<f64>,
    /// Explicit absolute normalized singular-value threshold used for both ranks.
    pub rank_threshold: f64,
    /// Observed left-null F_p projection, at a rank-one loss.
    pub transversality: Option<f64>,
    /// Observed left-null `F_xx[v,v]` projection, when actual curvature is available.
    pub curvature: Option<f64>,
    /// Actual optional second-action producer identity.
    pub curvature_source: Option<ContentHash>,
    /// Explicit absolute threshold applied to both nondegeneracy observations.
    pub nondegeneracy_threshold: f64,
    /// Actual attempted source/linear calls, including two independent rank probes.
    pub work: Work,
}
type SingularVectors = (Vec<f64>, Mat<f64>, Mat<f64>);
fn svd(
    matrix: &Mat<f64>,
    bytes: usize,
    execution: &Execution,
    work: &mut Work,
) -> Result<SingularVectors, ProblemError> {
    use faer::linalg::svd::{self, ComputeSvdVectors};
    execution.check()?;
    let m = matrix.nrows();
    let n = matrix.ncols();
    let req = svd::svd_scratch::<f64>(
        m,
        n,
        ComputeSvdVectors::Full,
        ComputeSvdVectors::Full,
        Par::Seq,
        Default::default(),
    );
    let owned = m
        .checked_mul(m)
        .and_then(|v| n.checked_mul(n).and_then(|a| v.checked_add(a)))
        .and_then(|v| v.checked_add(m.min(n)))
        .and_then(|v| v.checked_mul(8))
        .ok_or_else(|| ProblemError::memory("arclength rank storage overflow"))?;
    if owned
        .checked_add(req.size_bytes())
        .is_none_or(|v| v > bytes)
    {
        return Err(ProblemError::memory(
            "arclength rank probe scratch allowance",
        ));
    }
    let mut memory = MemBuffer::try_new(req).map_err(|e| ProblemError::memory(e.to_string()))?;
    let mut u = Mat::zeros(m, m);
    let mut v = Mat::zeros(n, n);
    let mut s = faer::diag::Diag::zeros(m.min(n));
    work.rank_probes += 1;
    svd::svd(
        matrix.as_ref(),
        s.as_mut(),
        Some(u.as_mut()),
        Some(v.as_mut()),
        Par::Seq,
        MemStack::new(&mut memory),
        Default::default(),
    )
    .map_err(|e| ProblemError::numerical(format!("arclength event SVD: {e:?}")))?;
    execution.check()?;
    let values = s.column_vector().iter().copied().collect::<Vec<_>>();
    if values.iter().any(|v| !v.is_finite()) {
        return Err(ProblemError::numerical(
            "nonfinite arclength rank observations",
        ));
    }
    Ok((values, u, v))
}
/// Inspect a localized event using actual library rank observations and optional original
/// curvature. Missing support remains unresolved. A branch singularity is never a fold.
/// # Errors
/// Invalid event/thresholds, original source failure, finite allowance or original scope stop.
pub fn probe<O: ParameterizedOracle<Error = ProblemError>>(
    request: Request<'_>,
    oracle: &mut O,
    mut curvature: Option<&mut dyn StateCurvature>,
    policy: ProbePolicy,
    execution: &Execution,
) -> Result<Event, Failure> {
    let ProbePolicy {
        localization,
        rank_threshold,
        nondegeneracy_threshold,
        limits,
    } = policy;
    let mut work = Work::default();
    let result = quality::contained(|| {
        execution.check()?;
        if !rank_threshold.is_finite()
            || rank_threshold <= 0.
            || !nondegeneracy_threshold.is_finite()
            || nondegeneracy_threshold <= 0.
            || localization.is_some_and(|l| {
                !l.interval.0.is_finite()
                    || !l.interval.1.is_finite()
                    || !l.at.is_finite()
                    || l.interval.0 > l.at
                    || l.at > l.interval.1
                    || l.interval.0 >= l.interval.1
            })
        {
            return Err(ProblemError::Contract(
                "arclength localized probe/threshold contract".into(),
            ));
        }
        if limits.svds < 2 {
            return Err(ProblemError::Limit {
                kind: crate::LimitKind::Work,
                detail: "arclength event rank-probe allowance".into(),
            });
        }
        let matrix = assemble(&request, oracle, execution, &mut work)?;
        let n = matrix.nrows() - 1;
        // Reserve both input/output matrices concurrently. This explicit event-only
        // allocation never turns the regular sparse tangent path into a dense one.
        let storage = (n + 1)
            .checked_mul(n + 1)
            .and_then(|v| v.checked_mul(128))
            .ok_or_else(|| ProblemError::memory("arclength event dense storage overflow"))?;
        if storage > limits.bytes {
            return Err(ProblemError::memory("arclength event dense rank allowance"));
        }
        let mut augmented = Mat::zeros(n, n + 1);
        for column in 0..=n {
            for index in matrix.symbolic().col_range(column) {
                let row = matrix.symbolic().row_idx()[index];
                if row < n {
                    augmented[(row, column)] = matrix.val()[index];
                }
            }
        }
        let states = Mat::from_fn(n, n, |r, c| augmented[(r, c)]);
        let (state_values, u, v) = svd(&states, limits.bytes - storage, execution, &mut work)?;
        let (augmented_values, _, _) =
            svd(&augmented, limits.bytes - storage, execution, &mut work)?;
        let state_rank = state_values.iter().filter(|v| **v > rank_threshold).count();
        let augmented_rank = augmented_values
            .iter()
            .filter(|v| **v > rank_threshold)
            .count();
        let mut transversality = None;
        let mut observed_curvature = None;
        let mut curvature_source = None;
        if state_rank + 1 == n {
            transversality = Some(
                (0..n)
                    .map(|r| u[(r, n - 1)] * augmented[(r, n)])
                    .sum::<f64>(),
            );
            if let Some(supplier) = curvature.as_mut() {
                execution.check()?;
                if supplier.contract() != request.family.original()
                    || supplier.parameter() != request.family.parameter()
                {
                    return Err(ProblemError::Contract(
                        "arclength curvature supplier differs from its original/parameter family"
                            .into(),
                    ));
                }
                let direction = (0..n)
                    .map(|c| v[(c, n - 1)] * request.family.scales().variables[c])
                    .collect::<Vec<_>>();
                let mut output = vec![0.; n];
                work.curvature_actions += 1;
                supplier.action(
                    &request.point[..n],
                    request.point[n],
                    &direction,
                    &mut output,
                )?;
                execution.check()?;
                if output.iter().any(|v| !v.is_finite()) {
                    return Err(ProblemError::numerical(
                        "nonfinite arclength curvature action",
                    ));
                }
                observed_curvature = Some(
                    (0..n)
                        .map(|r| u[(r, n - 1)] * output[r] / request.family.scales().rows[r])
                        .sum::<f64>(),
                );
                curvature_source = Some(supplier.source());
            }
        }
        let kind = if state_rank == n {
            EventKind::Regular
        } else if localization.is_some()
            && state_rank + 1 == n
            && augmented_rank == n
            && transversality.is_some_and(|v| v.is_finite() && v.abs() > nondegeneracy_threshold)
            && observed_curvature
                .is_some_and(|v| v.is_finite() && v.abs() > nondegeneracy_threshold)
        {
            EventKind::SimpleFold
        } else {
            EventKind::Unresolved
        };
        Ok(Event {
            kind,
            source: request.source,
            family: request.family.key(),
            point: request.point.to_vec(),
            localization,
            state_rank,
            augmented_rank,
            state_singular_values: state_values,
            augmented_singular_values: augmented_values,
            rank_threshold,
            transversality,
            curvature: observed_curvature,
            curvature_source,
            nondegeneracy_threshold,
            work: Work::default(),
        })
    });
    match result {
        Ok(mut event) => {
            event.work = work;
            Ok(event)
        }
        Err(cause) => Err(Failure {
            cause: Arc::new(cause),
            work,
        }),
    }
}

impl<O: ParameterizedOracle<Error = ProblemError>> crate::NlpOracle for Corrector<O> {
    fn contract(&self) -> &crate::OracleContract {
        &self.contract
    }
    fn jacobian_pattern(&self) -> faer::sparse::SymbolicSparseColMatRef<'_, usize> {
        self.pattern.matrix().symbolic()
    }
    fn hessian_pattern(&self) -> Option<faer::sparse::SymbolicSparseColMatRef<'_, usize>> {
        None
    }
    fn constraint_bounds(&self) -> &[(f64, f64)] {
        &self.bounds
    }
    fn objective(&mut self, _: &[f64]) -> Result<f64, ProblemError> {
        Ok(0.)
    }
    fn gradient(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        if x.len() != self.contract.variables.len() || out.len() != x.len() {
            return Err(ProblemError::Contract(
                "arclength feasibility gradient shape".into(),
            ));
        }
        out.fill(0.);
        Ok(())
    }
    fn constraints(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        self.binding.residual(x, out)
    }
    fn jacobian(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        crate::NleOracle::jacobian(self, x, out)
    }
    fn hessian(&mut self, _: &[f64], _: f64, _: &[f64], _: &mut [f64]) -> Result<(), ProblemError> {
        Err(ProblemError::Unsupported("arclength First feasibility requires a library limited-memory profile; no exact Hessian is fabricated".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pse_ids::SemanticId;
    use pse_kernels::DerivativeOrder;
    use pse_math::{
        continuation::Parameter,
        derived::{
            Constraint, Coordinate, DerivativeSupport, OriginalContract, OriginalObligations,
        },
        index::{Entry, GlobalCol, GlobalRow},
        normalization::Normalization,
    };
    fn id(n: u8) -> SemanticId {
        SemanticId::from_bytes([n; 16])
    }
    fn hash(n: u8) -> ContentHash {
        ContentHash::from_bytes([n; 32])
    }
    #[derive(Debug)]
    struct Curve {
        family: Arc<ArclengthFamily>,
        branch: bool,
        fail_parameter: bool,
    }
    impl ParameterizedOracle for Curve {
        type Error = ProblemError;
        fn contract(&self) -> &OriginalContract {
            self.family.original()
        }
        fn parameter(&self) -> &Parameter {
            self.family.parameter()
        }
        fn values(&mut self, x: &[f64], p: f64, out: &mut [f64]) -> Result<(), ProblemError> {
            out[0] = if self.branch {
                x[0].powi(3) - p * x[0]
            } else {
                x[0] * x[0] - p
            };
            Ok(())
        }
        fn state_action(
            &mut self,
            x: &[f64],
            p: f64,
            v: &[f64],
            out: &mut [f64],
        ) -> Result<(), ProblemError> {
            out[0] = if self.branch {
                (3. * x[0] * x[0] - p) * v[0]
            } else {
                2. * x[0] * v[0]
            };
            Ok(())
        }
        fn parameter_action(
            &mut self,
            x: &[f64],
            _: f64,
            v: f64,
            out: &mut [f64],
        ) -> Result<(), ProblemError> {
            if self.fail_parameter {
                return Err(ProblemError::Math(pse_math::MathError::Domain {
                    source_id: id(9),
                    requirement: "original scripted parameter domain",
                }));
            }
            out[0] = if self.branch { -x[0] * v } else { -v };
            Ok(())
        }
    }
    #[derive(Debug)]
    struct Curvature {
        family: Arc<ArclengthFamily>,
        branch: bool,
    }
    impl StateCurvature for Curvature {
        fn contract(&self) -> &OriginalContract {
            self.family.original()
        }
        fn parameter(&self) -> &Parameter {
            self.family.parameter()
        }
        fn source(&self) -> ContentHash {
            hash(21)
        }
        fn action(
            &mut self,
            x: &[f64],
            _: f64,
            v: &[f64],
            out: &mut [f64],
        ) -> Result<(), ProblemError> {
            out[0] = if self.branch {
                6. * x[0] * v[0] * v[0]
            } else {
                2. * v[0] * v[0]
            };
            Ok(())
        }
    }
    fn family() -> Arc<ArclengthFamily> {
        let scales = Normalization {
            variables: vec![2.],
            rows: vec![5.],
            objective: 1.,
        };
        let original = Arc::new(
            OriginalContract::new(
                hash(1),
                scales.key(),
                vec![Coordinate {
                    id: id(1),
                    lower: -10.,
                    upper: 10.,
                }],
                vec![Constraint {
                    id: id(2),
                    lower: 0.,
                    upper: 0.,
                }],
                vec![Entry::new(GlobalRow::new(0), GlobalCol::new(0))],
                DerivativeSupport {
                    order: DerivativeOrder::First,
                    jacobian_product: true,
                    source: hash(3),
                },
                OriginalObligations {
                    guards: hash(4),
                    selection: hash(5),
                    objective: None,
                },
            )
            .unwrap(),
        );
        Arc::new(
            ArclengthFamily::new(
                original,
                Parameter {
                    id: id(6),
                    lower: -10.,
                    upper: 10.,
                    scale: 3.,
                    source: hash(7),
                    action: true,
                    rows: vec![GlobalRow::new(0)],
                },
                id(8),
                scales,
            )
            .unwrap(),
        )
    }
    fn request<'a>(
        family: &'a Arc<ArclengthFamily>,
        point: &'a [f64],
        previous: &'a [f64],
    ) -> Request<'a> {
        Request {
            family,
            point,
            previous,
            source: hash(10),
            residual_limits: &[1e-10],
            backward_limit: 1e-12,
            limits: Limits {
                actions: 2,
                bytes: 1 << 20,
            },
        }
    }
    fn execution() -> Execution {
        Execution::new(
            Arc::default(),
            &crate::solve::Controls {
                time_limit: std::time::Duration::from_secs(5),
                ..crate::solve::Controls::default()
            },
        )
    }
    #[test]
    fn sparse_border_keeps_orientation_through_parameter_turning_and_reports_actual_work() {
        let family = family();
        let mut curve = Curve {
            family: family.clone(),
            branch: false,
            fail_parameter: false,
        };
        let execution = execution();
        let left = tangent(
            request(&family, &[0.2, 0.04], &[0., -1.]),
            &mut curve,
            &execution,
        )
        .unwrap();
        assert!(left.physical[0] < 0. && left.physical[1] < 0.);
        let fold = tangent(
            request(&family, &[0., 0.], &left.normalized),
            &mut curve,
            &execution,
        )
        .unwrap();
        assert!(fold.physical[0] < 0.);
        assert_eq!(fold.physical[1], 0.);
        let right = tangent(
            request(&family, &[-0.2, 0.04], &fold.normalized),
            &mut curve,
            &execution,
        )
        .unwrap();
        assert!(right.physical[0] < 0. && right.physical[1] > 0.);
        assert!(right.orientation > 0.);
        assert_eq!(
            right.work,
            Work {
                values: 1,
                state_actions: 1,
                parameter_actions: 1,
                factorizations: 1,
                backsolves: 1,
                ..Work::default()
            }
        );
        assert!(right.backward_error <= right.backward_limit);
    }
    #[test]
    fn localized_actual_rank_transversality_and_curvature_distinguish_fold_from_branch_singularity()
    {
        let family = family();
        let execution = execution();
        let mut curve = Curve {
            family: family.clone(),
            branch: false,
            fail_parameter: false,
        };
        let location = Localization {
            left: hash(11),
            right: hash(12),
            interval: (-0.1, 0.1),
            at: 0.,
        };
        let limits = ProbeLimits {
            bytes: 1 << 20,
            svds: 2,
        };
        let mut curvature = Curvature {
            family: family.clone(),
            branch: false,
        };
        let event = probe(
            request(&family, &[0., 0.], &[-1., 0.]),
            &mut curve,
            Some(&mut curvature),
            ProbePolicy {
                localization: Some(location),
                rank_threshold: 1e-10,
                nondegeneracy_threshold: 1e-8,
                limits,
            },
            &execution,
        )
        .unwrap();
        assert_eq!(event.kind, EventKind::SimpleFold);
        assert_eq!(event.state_rank, 0);
        assert_eq!(event.augmented_rank, 1);
        assert_eq!(event.work.rank_probes, 2);
        assert_eq!(event.work.curvature_actions, 1);
        assert!(event.transversality.unwrap().abs() > 1e-8);
        assert!(event.curvature.unwrap().abs() > 1e-8);
        let event = probe(
            request(&family, &[0., 0.], &[-1., 0.]),
            &mut curve,
            None,
            ProbePolicy {
                localization: Some(location),
                rank_threshold: 1e-10,
                nondegeneracy_threshold: 1e-8,
                limits,
            },
            &execution,
        )
        .unwrap();
        assert_eq!(event.kind, EventKind::Unresolved);
        assert!(event.curvature.is_none());
        curve.branch = true;
        curvature.branch = true;
        let event = probe(
            request(&family, &[0., 0.], &[-1., 0.]),
            &mut curve,
            Some(&mut curvature),
            ProbePolicy {
                localization: Some(location),
                rank_threshold: 1e-10,
                nondegeneracy_threshold: 1e-8,
                limits,
            },
            &execution,
        )
        .unwrap();
        assert_eq!(event.kind, EventKind::Unresolved);
        assert_eq!(event.augmented_rank, 0);
    }
    #[test]
    fn typed_stop_and_original_trial_failure_preserve_actual_pre_failure_work() {
        let family = family();
        let mut curve = Curve {
            family: family.clone(),
            branch: false,
            fail_parameter: true,
        };
        let execution = execution();
        let failure = tangent(
            request(&family, &[0., 0.], &[-1., 0.]),
            &mut curve,
            &execution,
        )
        .unwrap_err();
        assert!(
            matches!(failure.cause.as_ref(),ProblemError::Math(pse_math::MathError::Domain {source_id,..}) if *source_id==id(9))
        );
        assert_eq!(failure.work.parameter_actions, 1);
        assert_eq!(failure.work.factorizations, 0);
        execution
            .cancel
            .store(true, std::sync::atomic::Ordering::Relaxed);
        let failure = tangent(
            request(&family, &[0., 0.], &[-1., 0.]),
            &mut curve,
            &execution,
        )
        .unwrap_err();
        assert!(matches!(failure.cause.as_ref(), ProblemError::Cancelled));
        assert_eq!(failure.work, Work::default());
        let execution = self::execution();
        let mut r = request(&family, &[0., 0.], &[-1., 0.]);
        r.limits.bytes = 1;
        let failure = tangent(r, &mut curve, &execution).unwrap_err();
        assert!(matches!(
            failure.cause.as_ref(),
            ProblemError::Limit {
                kind: crate::LimitKind::Memory,
                ..
            }
        ));
        assert_eq!(failure.work, Work::default());
    }
    #[cfg(feature = "ipopt")]
    #[test]
    fn admitted_library_corrector_crosses_fold_without_a_project_newton_loop() {
        use crate::solve::*;
        let family = family();
        let mut curve = Curve {
            family: family.clone(),
            branch: false,
            fail_parameter: false,
        };
        let execution = execution();
        let t = tangent(
            request(&family, &[0., 0.], &[-1., 0.]),
            &mut curve,
            &execution,
        )
        .unwrap();
        let prediction = vec![-0.1, 0.];
        let binding = family
            .bind(curve, prediction.clone(), t.normalized, hash(15))
            .unwrap();
        let mut oracle = Corrector::new(binding, DerivativeOrder::Second).unwrap();
        let controls = Controls {
            hessian: HessianMode::LimitedMemory,
            reuse: ReusePolicy::Fresh,
            time_limit: std::time::Duration::from_secs(5),
            iterations: 50,
            ..Controls::default()
        };
        let tolerances = quality::Tolerances {
            variables: vec![1e-7; 2],
            rows: vec![1e-7; 2],
            integrality: 1e-7,
        };
        let mut sequence = crate::ipopt::Session::new();
        let report = sequence
            .solve(
                &mut oracle,
                &prediction,
                pse_math::binding::ObjectiveSense::Minimize,
                &controls,
                &ResolvedAccuracy::nominal(),
                &crate::ipopt::Settings::default(),
                Execution::new(Arc::default(), &controls),
                &tolerances,
                None,
                crate::solver_tests::stamp(Backend::Ipopt),
            )
            .unwrap();
        assert_eq!(
            report.termination.category,
            Termination::Success,
            "{report:?}"
        );
        let candidate = report.candidate.unwrap();
        assert!((candidate.primal[0] + 0.1).abs() < 1e-7);
        assert!((candidate.primal[1] - 0.01).abs() < 1e-7);
        let binding = oracle.into_binding();
        let (state, parameter) = binding.reconstruct(&candidate.primal).unwrap();
        assert!((state[0] * state[0] - parameter).abs() < 1e-7);
        assert!(!report.evidence.callback.terminal_failure);
    }
}
