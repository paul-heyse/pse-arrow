// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Exact convexity: the compiler fact routing reads (ADR-0121), exact rational LDLᵀ Gram
//! certificates, and opt-in numerical evidence for one request.
//!
//! Convexity is established once, at preparation, and recorded in
//! [`crate::facts::ProblemFacts::convexity`] with the identity of the values it consumed, so
//! it rebinds with them (A6). A coefficient program's objective quadratic is decided by an
//! exact rational LDLᵀ with symmetric pivoting ([`GramCertificate::certify`]); a continuous
//! nonlinear program by the curvature pass ([`crate::curvature`]). Numerical PSD evidence
//! ([`Coefficients::numerical_convexity`]) is an explicit qualification of one request and
//! never a fact.
use crate::{MathError, coefficients::Coefficients};
use pse_ids::{ContentHash, FramedHasher};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
use symbolica::domains::rational::Rational;

mod sealed {
    pub trait Sealed {}
}
/// Only library-verified evidence may admit a convex solver. Implementations are sealed.
pub trait QuadraticEvidence: sealed::Sealed + std::fmt::Debug + Send + Sync {
    /// Value-dependent assumptions, absent for a pure exact matrix identity.
    fn assumptions(&self) -> Option<ContentHash> {
        None
    }
    /// The computed exact, numerical, negative or inconclusive assessment, when present.
    fn assessment(&self) -> Option<&ConvexityAssessment> {
        None
    }

    /// Refuse numerical evidence qualified under another coordinate or tolerance policy.
    fn validate_policy(
        &self,
        _policy: ConvexityPolicy,
        _scales: &[f64],
        _objective: f64,
    ) -> Result<(), MathError> {
        Ok(())
    }
    /// Check current coefficients and objective orientation, and requested evidence eligibility.
    fn validate(
        &self,
        q: &faer::sparse::SparseColMat<usize, f64>,
        sign: f64,
    ) -> Result<(), MathError>;
}
impl sealed::Sealed for GramCertificate {}
impl QuadraticEvidence for GramCertificate {
    fn validate(
        &self,
        q: &faer::sparse::SparseColMat<usize, f64>,
        sign: f64,
    ) -> Result<(), MathError> {
        GramCertificate::validate(self, q, sign)
    }
}
impl sealed::Sealed for ConvexityEvidence {}
impl QuadraticEvidence for ConvexityEvidence {
    fn validate_policy(
        &self,
        policy: ConvexityPolicy,
        scales: &[f64],
        objective: f64,
    ) -> Result<(), MathError> {
        if matches!(self.assessment, ConvexityAssessment::NumericalPsd { .. })
            && (self.policy != policy || self.scales != scales || self.objective_scale != objective)
        {
            return Err(MathError::Contract(
                "numerical PSD evidence has different policy or coordinates".into(),
            ));
        }
        Ok(())
    }
    fn assumptions(&self) -> Option<ContentHash> {
        Some(self.assumptions)
    }
    fn assessment(&self) -> Option<&ConvexityAssessment> {
        Some(&self.assessment)
    }
    fn validate(
        &self,
        q: &faer::sparse::SparseColMat<usize, f64>,
        sign: f64,
    ) -> Result<(), MathError> {
        self.validate_matrix(q, sign)?;
        if !self.accepted() {
            return Err(MathError::Contract(
                "quadratic assessment did not establish the requested convexity evidence".into(),
            ));
        }
        Ok(())
    }
}
/// Positive congruence transports original evidence without claiming a new exact factorization.
#[derive(Clone, Debug)]
pub struct TransportedEvidence {
    assessment: Option<ConvexityAssessment>,
    matrix: ContentHash,
    assumptions: Option<ContentHash>,
}
impl sealed::Sealed for TransportedEvidence {}
impl QuadraticEvidence for TransportedEvidence {
    fn assessment(&self) -> Option<&ConvexityAssessment> {
        self.assessment.as_ref()
    }
    fn validate_policy(
        &self,
        _policy: ConvexityPolicy,
        _scales: &[f64],
        _objective: f64,
    ) -> Result<(), MathError> {
        if matches!(
            self.assessment,
            Some(ConvexityAssessment::NumericalPsd { .. })
        ) {
            return Err(MathError::Contract(
                "transported numerical PSD evidence cannot qualify another preparation".into(),
            ));
        }
        Ok(())
    }
    fn assumptions(&self) -> Option<ContentHash> {
        self.assumptions
    }
    fn validate(
        &self,
        q: &faer::sparse::SparseColMat<usize, f64>,
        sign: f64,
    ) -> Result<(), MathError> {
        if self.matrix != quadratic_identity(q, sign) {
            return Err(MathError::Contract(
                "stale normalized quadratic evidence".into(),
            ));
        }
        Ok(())
    }
}
/// Normalize an unchanged admitted quadratic by a checked positive diagonal congruence.
pub fn normalize_quadratic(
    q: &faer::sparse::SparseColMat<usize, f64>,
    sign: f64,
    scales: &[f64],
    objective: f64,
    proof: &dyn QuadraticEvidence,
) -> Result<(faer::sparse::SparseColMat<usize, f64>, TransportedEvidence), MathError> {
    proof.validate(q, sign)?;
    if q.nrows() != scales.len() || q.ncols() != scales.len() {
        return Err(MathError::Contract(
            "quadratic normalization dimensions".into(),
        ));
    }
    let mut entries = Vec::with_capacity(q.val().len());
    for c in 0..q.ncols() {
        for (r, value) in q.row_idx_of_col(c).zip(q.val_of_col(c)) {
            let value = crate::normalization::checked_ratio(
                crate::normalization::checked_product(
                    crate::normalization::checked_product(*value, scales[r])?,
                    scales[c],
                )?,
                objective,
            )?;
            entries.push(faer::sparse::Triplet::new(r, c, value));
        }
    }
    let result = faer::sparse::SparseColMat::try_new_from_triplets(q.nrows(), q.ncols(), &entries)
        .map_err(|e| MathError::Contract(e.to_string()))?;
    let evidence = TransportedEvidence {
        assessment: proof.assessment().cloned(),
        matrix: quadratic_identity(&result, sign),
        assumptions: proof.assumptions(),
    };
    Ok((result, evidence))
}

/// The minimization form `sign · q` of an admitted objective quadratic as a solver that
/// stores only the upper triangle sees it: the upper triangle mirrored into the lower one.
/// Negation is exact; a mirrored entry replaces its lower partner, which may differ from
/// it only by the rounding of the congruence that produced `q`, so the evidence is the
/// admitted evidence's, transported to exactly the returned matrix under orientation one.
/// A matrix without nonzeros needs no evidence.
///
/// # Errors
/// Missing or stale evidence, a structurally unsymmetric `q`, or partners that differ by
/// more than rounding.
pub fn minimization_form(
    q: &faer::sparse::SparseColMat<usize, f64>,
    sign: f64,
    proof: Option<&dyn QuadraticEvidence>,
) -> Result<(faer::sparse::SparseColMat<usize, f64>, TransportedEvidence), MathError> {
    let n = q.ncols();
    if q.nrows() != n || !(sign == 1.0 || sign == -1.0) {
        return Err(MathError::Contract(
            "objective quadratic dimensions or orientation".into(),
        ));
    }
    let nonzero = q.val().iter().any(|v| *v != 0.0);
    match proof {
        Some(proof) => proof.validate(q, sign)?,
        None if nonzero => {
            return Err(MathError::Contract(
                "a nonzero objective quadratic needs convexity evidence".into(),
            ));
        }
        None => {}
    }
    // Upper-triangle entries, and the strictly lower ones at their mirrored position.
    let mut upper = BTreeMap::new();
    let mut lower = BTreeMap::new();
    for c in 0..n {
        for (r, &v) in q.row_idx_of_col(c).zip(q.val_of_col(c)) {
            if v == 0.0 {
                continue;
            }
            if r <= c {
                upper.insert((r, c), v);
            } else {
                lower.insert((c, r), v);
            }
        }
    }
    // Four units in the last place bound the congruence's rounding of either partner.
    let rounding = 4.0 * f64::EPSILON;
    let unsymmetric = upper.keys().any(|&(r, c)| r != c && !lower.contains_key(&(r, c)))
        || lower.iter().any(|(position, &v)| {
            upper
                .get(position)
                .is_none_or(|&u| (u - v).abs() > rounding * u.abs().max(v.abs()))
        });
    if unsymmetric {
        return Err(MathError::Contract(
            "objective quadratic is not symmetric".into(),
        ));
    }
    let mut entries = Vec::with_capacity(2 * upper.len());
    for (&(r, c), &v) in &upper {
        entries.push(faer::sparse::Triplet::new(r, c, sign * v));
        if r != c {
            entries.push(faer::sparse::Triplet::new(c, r, sign * v));
        }
    }
    let result = faer::sparse::SparseColMat::try_new_from_triplets(n, n, &entries)
        .map_err(|e| MathError::Contract(e.to_string()))?;
    let evidence = TransportedEvidence {
        assessment: proof.and_then(|p| p.assessment().cloned()),
        matrix: quadratic_identity(&result, 1.0),
        assumptions: proof.and_then(|p| p.assumptions()),
    };
    Ok((result, evidence))
}

/// Numerical assessment is a separate, explicit permission from exact certification.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum ConvexityPolicy {
    /// Only the exact convexity fact establishes convexity.
    #[default]
    Exact,
    /// Also permit a residual-qualified PSD assessment in normalized coordinates, for the
    /// one request that states it; it never becomes a fact.
    Numerical {
        /// Absolute eigenvalue budget in normalized objective coordinates.
        absolute: f64,
        /// Relative budget against the normalized matrix Frobenius norm.
        relative: f64,
    },
}
impl ConvexityPolicy {
    /// The policy of optional numerical tolerances: none is exact certification; both,
    /// finite and nonnegative, permit numerical assessment.
    ///
    /// # Errors
    /// Exactly one tolerance, or a nonfinite or negative one.
    pub fn from_tolerances(
        absolute: Option<f64>,
        relative: Option<f64>,
    ) -> Result<Self, MathError> {
        match (absolute, relative) {
            (None, None) => Ok(Self::Exact),
            (Some(absolute), Some(relative))
                if [absolute, relative]
                    .iter()
                    .all(|v| v.is_finite() && *v >= 0.0) =>
            {
                Ok(Self::Numerical { absolute, relative })
            }
            _ => Err(MathError::Contract(
                "numerical convexity requires both finite nonnegative tolerances".into(),
            )),
        }
    }
}
/// A bounded assessment that did not establish either requested conclusion.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InconclusiveReason {
    /// The caller's allocation or exact-operation allowance is insufficient.
    ResourceLimit,
    /// The eigensolver did not converge or returned invalid values.
    Eigensolver,
    /// Residual or orthogonality qualification failed.
    Residual,
    /// The uncertainty interval crosses the requested PSD threshold.
    Threshold,
}
impl InconclusiveReason {
    /// Stable report spelling, independent of Rust debug output.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ResourceLimit => "resource_limit",
            Self::Eigensolver => "eigensolver",
            Self::Residual => "residual",
            Self::Threshold => "threshold",
        }
    }
}

/// A numerical assessment's scientific interpretation; numerical PSD never masquerades
/// as an exact certificate.
#[derive(Clone, Debug)]
pub enum ConvexityAssessment {
    /// PSD only within the explicit tolerance and numerical uncertainty.
    NumericalPsd {
        /// Smallest normalized computed eigenvalue.
        minimum: f64,
        /// Requested absolute plus relative tolerance.
        tolerance: f64,
        /// Residual-derived uncertainty estimate.
        uncertainty: f64,
    },
    /// A negative direction below the permitted PSD tolerance.
    Indefinite {
        /// Smallest normalized computed eigenvalue.
        minimum: f64,
        /// Residual-derived uncertainty estimate.
        uncertainty: f64,
    },
    /// No conclusion, distinct from infeasible or indefinite.
    Inconclusive(InconclusiveReason),
}
/// Nonforgeable numerical evidence tied to coefficients, values, orientation, normalization
/// and the one request's tolerance policy.
#[derive(Clone, Debug)]
pub struct ConvexityEvidence {
    policy: ConvexityPolicy,
    scales: Vec<f64>,
    objective_scale: f64,
    key: ContentHash,
    matrix: ContentHash,
    assumptions: ContentHash,
    assessment: ConvexityAssessment,
}
impl ConvexityEvidence {
    /// Refuse evidence from another represented quadratic matrix or objective sense.
    pub fn validate_matrix(
        &self,
        q: &faer::sparse::SparseColMat<usize, f64>,
        sign: f64,
    ) -> Result<(), MathError> {
        if self.matrix != quadratic_identity(q, sign) {
            return Err(MathError::Contract("stale quadratic evidence".into()));
        }
        Ok(())
    }
    /// Complete evidence identity.
    pub fn key(&self) -> ContentHash {
        self.key
    }
    /// Exact consumed coefficient snapshot identity.
    pub fn assumptions(&self) -> ContentHash {
        self.assumptions
    }
    /// Retained scientific conclusion.
    pub fn assessment(&self) -> &ConvexityAssessment {
        &self.assessment
    }
    /// Whether the assessment established numerical PSD under the request's policy.
    pub fn accepted(&self) -> bool {
        matches!(self.assessment, ConvexityAssessment::NumericalPsd { .. })
    }
}
impl Coefficients {
    /// The residual-qualified numerical PSD assessment of the admitted snapshot under one
    /// request's explicit tolerances (ADR-0121 Outcome 4), in normalized coordinates. It
    /// never changes Q or silently repairs curvature, and it is never a fact: exact
    /// convexity is [`crate::facts::ProblemFacts::convexity`].
    ///
    /// # Errors
    /// Invalid coordinates, matrix or tolerances, or cancellation.
    #[expect(
        clippy::too_many_arguments,
        reason = "an assessment binds its orientation, coordinates, tolerances, allowance and cancellation"
    )]
    pub fn numerical_convexity(
        &self,
        sign: f64,
        scales: &[f64],
        objective_scale: f64,
        absolute: f64,
        relative: f64,
        bytes: usize,
        cancel: &AtomicBool,
    ) -> Result<ConvexityEvidence, MathError> {
        let n = self.hessian.ncols();
        if self.hessian.nrows() != n
            || scales.len() != n
            || !matches!(sign, -1.0 | 1.0)
            || !objective_scale.is_finite()
            || objective_scale <= 0.0
            || scales.iter().any(|v| !v.is_finite() || *v <= 0.0)
            || self.hessian.val().iter().any(|v| !v.is_finite())
        {
            return Err(MathError::Contract(
                "convexity coordinates or matrix".into(),
            ));
        }
        if !absolute.is_finite()
            || !relative.is_finite()
            || absolute < 0.0
            || relative < 0.0
            || absolute + relative <= 0.0
        {
            return Err(MathError::Contract(
                "explicit numerical PSD tolerance".into(),
            ));
        }
        let policy = ConvexityPolicy::Numerical { absolute, relative };
        let mut h = FramedHasher::new(pse_ids::Frame::MathConvexityV1);
        h.hash(&self.assumptions)
            .u64(sign.to_bits())
            .u64(objective_scale.to_bits());
        for v in scales {
            h.u64(v.to_bits());
        }
        h.u64(1).u64(absolute.to_bits()).u64(relative.to_bits());
        h.str("faer-0.24.4;serial-evd;residual-v1")
            .u64(bytes as u64);
        let matrix = quadratic_identity(&self.hessian, sign);
        h.hash(&matrix);
        let key = h.finish_hash();
        let result = |assessment| ConvexityEvidence {
            policy,
            scales: scales.to_vec(),
            objective_scale,
            key,
            matrix,
            assumptions: self.assumptions,
            assessment,
        };
        let unknown = |reason| result(ConvexityAssessment::Inconclusive(reason));
        if cancel.load(Ordering::Relaxed) {
            return Err(MathError::Cancelled);
        }
        let dense_bytes = n.checked_mul(n).and_then(|v| v.checked_mul(8 * 10));
        if dense_bytes.is_none_or(|v| v > bytes) {
            return Ok(unknown(InconclusiveReason::ResourceLimit));
        }
        let original = self.hessian.to_dense();
        if (0..n).any(|i| (0..n).any(|j| original[(i, j)] != original[(j, i)])) {
            return Err(MathError::Contract("asymmetric quadratic matrix".into()));
        }
        let mut a = faer::Mat::zeros(n, n);
        for i in 0..n {
            for j in 0..n {
                a[(i, j)] = crate::normalization::checked_ratio(
                    crate::normalization::checked_product(
                        crate::normalization::checked_product(sign * original[(i, j)], scales[i])?,
                        scales[j],
                    )?,
                    objective_scale,
                )?;
            }
        }
        let norm = a.norm_l2();
        let threshold = absolute + relative * norm;
        if !norm.is_finite() || !threshold.is_finite() {
            return Err(MathError::Contract("nonfinite normalized quadratic".into()));
        }
        let scratch = faer::linalg::evd::self_adjoint_evd_scratch::<f64>(
            n,
            faer::linalg::evd::ComputeEigenvectors::Yes,
            faer::Par::Seq,
            Default::default(),
        );
        if dense_bytes
            .and_then(|b| b.checked_add(scratch.unaligned_bytes_required()))
            .is_none_or(|b| b > bytes)
        {
            return Ok(unknown(InconclusiveReason::ResourceLimit));
        }
        let mut memory = faer::dyn_stack::MemBuffer::new(scratch);
        let mut eigenvalues = faer::diag::Diag::zeros(n);
        let mut vectors = faer::Mat::zeros(n, n);
        if faer::linalg::evd::self_adjoint_evd(
            a.as_ref(),
            eigenvalues.as_mut(),
            Some(vectors.as_mut()),
            faer::Par::Seq,
            faer::dyn_stack::MemStack::new(&mut memory),
            Default::default(),
        )
        .is_err()
        {
            return Ok(unknown(InconclusiveReason::Eigensolver));
        }
        if cancel.load(Ordering::Relaxed) {
            return Err(MathError::Cancelled);
        }
        let weighted = faer::Mat::from_fn(n, n, |i, j| vectors[(i, j)] * eigenvalues[j]);
        let residual = (&a * &vectors - weighted).norm_l2();
        let orthogonality =
            (vectors.transpose() * &vectors - faer::Mat::<f64>::identity(n, n)).norm_l2();
        let allowance = 100.0 * f64::EPSILON * n.max(1) as f64;
        if !residual.is_finite()
            || !orthogonality.is_finite()
            || orthogonality > allowance
            || residual > allowance * norm.max(1.0)
        {
            return Ok(unknown(InconclusiveReason::Residual));
        }
        let minimum = if n == 0 { 0.0 } else { eigenvalues[0] };
        let uncertainty = residual + norm * orthogonality / (1.0 - orthogonality);
        if !minimum.is_finite() || !uncertainty.is_finite() {
            return Ok(unknown(InconclusiveReason::Eigensolver));
        }
        Ok(if minimum + uncertainty < -threshold {
            result(ConvexityAssessment::Indefinite {
                minimum,
                uncertainty,
            })
        } else if minimum - uncertainty >= -threshold {
            result(ConvexityAssessment::NumericalPsd {
                minimum,
                tolerance: threshold,
                uncertainty,
            })
        } else {
            unknown(InconclusiveReason::Threshold)
        })
    }
}

/// The exact-operation allowance of one preparation's certificates: rational
/// multiply-adds of every exact LDLᵀ and quadratic expansion it runs. An exhausted allowance
/// is inconclusive, never convex (ADR-0121 Outcome 2).
pub const EXACT_OPERATIONS: usize = 1 << 22;

/// `Q = sign · Σₖ wₖ rₖ rₖᵀ` with every `wₖ > 0`: a Gram (sum-of-squares) representation of
/// a positive semidefinite quadratic, computed by an exact rational LDLᵀ with symmetric
/// pivoting ([`Self::certify`]) and tied to every numeric matrix entry and the objective
/// orientation. The factors are kept: a cone lowering reads a certified quadratic as a sum
/// of weighted squares.
#[derive(Clone, Debug)]
pub struct GramCertificate {
    identity: ContentHash,
    factors: GramFactors,
}
impl PartialEq for GramCertificate {
    fn eq(&self, other: &Self) -> bool {
        self.identity == other.identity
    }
}
impl Eq for GramCertificate {}
/// The factors of an exact LDLᵀ: rows `rₖ` in original coordinates, each with a unit entry
/// at its pivot, and positive weights `wₖ`.
#[derive(Clone, Debug, Default)]
pub struct GramFactors {
    rows: Vec<Vec<(usize, Rational)>>,
    weights: Vec<Rational>,
}
impl GramFactors {
    /// The squared terms: each positive weight with its row, in pivot order.
    pub fn terms(&self) -> impl Iterator<Item = (&Rational, &[(usize, Rational)])> {
        self.weights
            .iter()
            .zip(self.rows.iter().map(Vec::as_slice))
    }
    /// The number of squares: the rank of the certified matrix.
    pub fn rank(&self) -> usize {
        self.weights.len()
    }
    /// The same factors over the original columns `columns[k]` of local index `k`.
    pub(crate) fn relabel(self, columns: &[usize]) -> Self {
        Self {
            rows: self
                .rows
                .into_iter()
                .map(|row| row.into_iter().map(|(k, v)| (columns[k], v)).collect())
                .collect(),
            weights: self.weights,
        }
    }
}
/// The exact decision of one symmetric matrix.
#[derive(Clone, Debug)]
pub enum Definiteness {
    /// Positive semidefinite, with its certificate.
    Psd(GramCertificate),
    /// A direction of negative curvature exists.
    Indefinite,
    /// The exact-operation allowance was exhausted before a decision.
    Inconclusive,
}
/// The outcome of an exact LDLᵀ over rational rows.
pub(crate) enum Ldlt {
    Psd(GramFactors),
    Indefinite,
    Exhausted,
}
impl GramCertificate {
    /// Decide positive semidefiniteness of `sign · q` exactly: every binary64 entry is a
    /// rational, and a symmetric-pivoted rational LDLᵀ within `limit` multiply-adds either
    /// yields the Gram factors, finds a direction of negative curvature, or is inconclusive.
    ///
    /// # Errors
    /// A non-square, structurally unsymmetric or nonfinite matrix, an orientation other
    /// than ±1, or cancellation.
    pub fn certify(
        q: &faer::sparse::SparseColMat<usize, f64>,
        sign: f64,
        limit: usize,
        cancel: &AtomicBool,
    ) -> Result<Definiteness, MathError> {
        let n = q.ncols();
        if q.nrows() != n || !matches!(sign, -1.0 | 1.0) {
            return Err(MathError::Contract(
                "Gram certificate dimensions or orientation".into(),
            ));
        }
        let mut rows = vec![BTreeMap::new(); n];
        for c in 0..n {
            for (r, v) in q.row_idx_of_col(c).zip(q.val_of_col(c)) {
                if *v == 0.0 {
                    continue;
                }
                let value = Rational::try_from(sign * v)
                    .map_err(|e| MathError::Contract(format!("Gram entry: {e}")))?;
                rows[r].insert(c, value);
            }
        }
        for (r, row) in rows.iter().enumerate() {
            for (c, v) in row {
                if rows[*c].get(&r) != Some(v) {
                    return Err(MathError::Contract("asymmetric quadratic matrix".into()));
                }
            }
        }
        let mut budget = limit;
        Ok(match ldlt(rows, &mut budget, cancel)? {
            Ldlt::Psd(factors) => Definiteness::Psd(Self {
                identity: quadratic_identity(q, sign),
                factors,
            }),
            Ldlt::Indefinite => Definiteness::Indefinite,
            Ldlt::Exhausted => Definiteness::Inconclusive,
        })
    }
    /// Reject a stale certificate after any numeric matrix or sense change.
    ///
    /// # Errors
    /// The certificate belongs to another matrix or orientation.
    pub fn validate(
        &self,
        q: &faer::sparse::SparseColMat<usize, f64>,
        sign: f64,
    ) -> Result<(), MathError> {
        if quadratic_identity(q, sign) != self.identity {
            return Err(MathError::Contract("stale convexity evidence".into()));
        }
        Ok(())
    }
    /// The certified matrix and orientation's identity.
    pub fn identity(&self) -> ContentHash {
        self.identity
    }
    /// The exact factors.
    pub fn factors(&self) -> &GramFactors {
        &self.factors
    }
}
/// Exact LDLᵀ of the symmetric rational matrix `a` (complete rows, zeros omitted) with
/// symmetric pivoting, spending at most `budget` multiply-adds from it. At each step the pivot is the positive diagonal of the current
/// Schur complement whose row has the fewest entries (then the lowest index), which keeps
/// fill low. A negative diagonal, or a zero diagonal with an off-diagonal entry, in any
/// Schur complement is a direction of negative curvature of `a`; when no positive diagonal
/// remains, the complement is zero and the factors are complete.
pub(crate) fn ldlt(
    mut a: Vec<BTreeMap<usize, Rational>>,
    budget: &mut usize,
    cancel: &AtomicBool,
) -> Result<Ldlt, MathError> {
    enum Class {
        Positive,
        Zero,
        Negative,
    }
    let classify = |row: &BTreeMap<usize, Rational>, i: usize| match row.get(&i) {
        Some(d) if d.is_negative() => Class::Negative,
        Some(d) if !d.is_zero() => Class::Positive,
        // A zero diagonal with an off-diagonal entry `b` has the negative direction
        // (b, -t) of the 2×2 minor for large t.
        _ if row.keys().any(|&j| j != i) => Class::Negative,
        _ => Class::Zero,
    };
    let mut positive = BTreeSet::new();
    for (i, row) in a.iter().enumerate() {
        match classify(row, i) {
            Class::Negative => return Ok(Ldlt::Indefinite),
            Class::Positive => {
                positive.insert((row.len(), i));
            }
            Class::Zero => {}
        }
    }
    let mut factors = GramFactors::default();
    while let Some((_, p)) = positive.pop_first() {
        if cancel.load(Ordering::Relaxed) {
            return Err(MathError::Cancelled);
        }
        let row = std::mem::take(&mut a[p]);
        let d = row[&p].clone();
        let neighbors: Vec<(usize, Rational)> = row
            .into_iter()
            .filter(|(j, _)| *j != p)
            .collect();
        let k = neighbors.len();
        let cost = (k.saturating_mul(k + 1) / 2).saturating_add(k).max(1);
        if cost > *budget {
            *budget = 0;
            return Ok(Ldlt::Exhausted);
        }
        *budget -= cost;
        let multipliers: Vec<Rational> = neighbors.iter().map(|(_, v)| v / &d).collect();
        for (i, _) in &neighbors {
            positive.remove(&(a[*i].len(), *i));
            a[*i].remove(&p);
        }
        // The Schur complement: a_ij -= a_ip a_jp / d.
        for (x, (i, _)) in neighbors.iter().enumerate() {
            for (j, a_jp) in neighbors.iter().skip(x) {
                let update = &multipliers[x] * a_jp;
                let entry = a[*i].entry(*j).or_insert_with(Rational::zero);
                *entry -= &update;
                let value = entry.clone();
                if value.is_zero() {
                    a[*i].remove(j);
                    a[*j].remove(i);
                } else if i != j {
                    a[*j].insert(*i, value);
                }
            }
        }
        for (i, _) in &neighbors {
            match classify(&a[*i], *i) {
                Class::Negative => return Ok(Ldlt::Indefinite),
                Class::Positive => {
                    positive.insert((a[*i].len(), *i));
                }
                Class::Zero => {}
            }
        }
        let mut gram = vec![(p, Rational::one())];
        gram.extend(
            neighbors
                .iter()
                .map(|(i, _)| *i)
                .zip(multipliers),
        );
        gram.sort_by_key(|(i, _)| *i);
        factors.rows.push(gram);
        factors.weights.push(d);
    }
    Ok(Ldlt::Psd(factors))
}
/// Identity of a represented quadratic matrix and its objective orientation.
pub(crate) fn quadratic_identity(
    q: &faer::sparse::SparseColMat<usize, f64>,
    sign: f64,
) -> ContentHash {
    let mut h = FramedHasher::new(pse_ids::Frame::MathGramV2);
    h.u64(q.nrows() as u64)
        .u64(q.ncols() as u64)
        .u64(sign.to_bits());
    for c in 0..q.ncols() {
        for (&r, &v) in q
            .symbolic()
            .row_idx_of_col_raw(c)
            .iter()
            .zip(q.val_of_col(c))
        {
            if v != 0.0 {
                h.u64(r as u64).u64(c as u64).u64(v.to_bits());
            }
        }
    }
    h.finish_hash()
}

/// Convexity established at preparation (ADR-0121): the one exact authority routing reads.
/// It is carried by the value-dependent products, so a value rebind re-establishes it, and
/// its key identifies the program or coefficient snapshot and the values it consumed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Convexity {
    /// Identity of what the fact consumed and how it was established.
    pub key: ContentHash,
    /// The established class.
    pub class: ConvexityClass,
}
/// What preparation established about a problem's convexity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConvexityClass {
    /// Nothing was assessed: a coefficient program prepared without its coefficient
    /// snapshot.
    NotAssessed,
    /// A linear coefficient program.
    Affine,
    /// A coefficient program whose objective quadratic, in the minimization sense, has an
    /// exact Gram certificate.
    ConvexQuadratic(Arc<GramCertificate>),
    /// A continuous program whose rows and objective the curvature pass recognizes as
    /// convex in the directions their bounds and sense require, with a cone representation.
    Cone(ConeSummary),
    /// Not established, with the first reason found.
    Unrecognized(Unrecognized),
    /// The exact allowance was exhausted; never convex.
    Inconclusive,
}
/// The cones a recognized program lowers to.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ConeSummary {
    /// Auxiliary epigraph and hypograph columns.
    pub auxiliaries: usize,
    /// Scalar nonnegative constraints of atoms (absolute values).
    pub nonnegative: usize,
    /// Second-order cones (norms and certified quadratics).
    pub second_order: usize,
    /// Exponential cones (exp, log, entropy, relative entropy).
    pub exponential: usize,
    /// Power cones.
    pub power: usize,
}
/// Why convexity was not established.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unrecognized {
    /// A column is discrete; cone classes are continuous.
    Discrete,
    /// A row or the objective is not exported exactly, or the program has auxiliaries,
    /// implicit residuals, native forms or incomplete instances.
    Inexact,
    /// A retained obligation's closure does not follow from the variable box.
    Domain,
    /// A row (`Some(index)`) or the objective (`None`) has no recognized convex form in the
    /// direction its bounds or sense require.
    Curvature {
        /// The row, or `None` for the objective.
        row: Option<usize>,
    },
    /// The objective quadratic has an exact direction of negative curvature.
    Indefinite,
}
impl Convexity {
    /// The certificate of a convex quadratic coefficient objective.
    pub fn convex_quadratic(&self) -> Option<&Arc<GramCertificate>> {
        match &self.class {
            ConvexityClass::ConvexQuadratic(c) => Some(c),
            _ => None,
        }
    }
    /// Whether the program is a recognized continuous cone program.
    pub fn cone(&self) -> bool {
        matches!(self.class, ConvexityClass::Cone(_))
    }
    /// The class of a coefficient program: its objective quadratic decided exactly in the
    /// minimization sense `sign`.
    ///
    /// # Errors
    /// A malformed matrix, or cancellation.
    pub fn of_coefficients(
        coefficients: &Coefficients,
        sign: f64,
        cancel: &AtomicBool,
    ) -> Result<Self, MathError> {
        let q = &coefficients.hessian;
        let class = if q.val().iter().all(|v| *v == 0.0) {
            ConvexityClass::Affine
        } else {
            match GramCertificate::certify(q, sign, EXACT_OPERATIONS, cancel)? {
                Definiteness::Psd(c) => ConvexityClass::ConvexQuadratic(Arc::new(c)),
                Definiteness::Indefinite => ConvexityClass::Unrecognized(Unrecognized::Indefinite),
                Definiteness::Inconclusive => ConvexityClass::Inconclusive,
            }
        };
        let mut h = FramedHasher::new(pse_ids::Frame::MathConvexityFactV1);
        h.hash(&coefficients.assumptions)
            .hash(&quadratic_identity(q, sign))
            .u64(EXACT_OPERATIONS as u64);
        class.hash_into(&mut h);
        Ok(Self {
            key: h.finish_hash(),
            class,
        })
    }
    /// The fact of a problem whose class nothing assessed.
    pub fn not_assessed(structure: ContentHash) -> Self {
        let class = ConvexityClass::NotAssessed;
        let mut h = FramedHasher::new(pse_ids::Frame::MathConvexityFactV1);
        h.hash(&structure);
        class.hash_into(&mut h);
        Self {
            key: h.finish_hash(),
            class,
        }
    }
}
impl ConvexityClass {
    pub(crate) fn hash_into(&self, h: &mut FramedHasher) {
        match self {
            Self::NotAssessed => {
                h.u64(0);
            }
            Self::Affine => {
                h.u64(1);
            }
            Self::ConvexQuadratic(c) => {
                h.u64(2).hash(&c.identity).u64(c.factors.rank() as u64);
            }
            Self::Cone(s) => {
                h.u64(3)
                    .u64(s.auxiliaries as u64)
                    .u64(s.nonnegative as u64)
                    .u64(s.second_order as u64)
                    .u64(s.exponential as u64)
                    .u64(s.power as u64);
            }
            Self::Unrecognized(reason) => {
                h.u64(4);
                match reason {
                    Unrecognized::Discrete => h.u64(0),
                    Unrecognized::Inexact => h.u64(1),
                    Unrecognized::Domain => h.u64(2),
                    Unrecognized::Curvature { row } => {
                        h.u64(3).u64(row.map_or(u64::MAX, |r| r as u64))
                    }
                    Unrecognized::Indefinite => h.u64(4),
                };
            }
            Self::Inconclusive => {
                h.u64(5);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use faer::sparse::{SparseColMat, Triplet};

    fn square(n: usize, entries: &[(usize, usize, f64)]) -> SparseColMat<usize, f64> {
        let triplets: Vec<_> = entries
            .iter()
            .map(|(r, c, v)| Triplet::new(*r, *c, *v))
            .collect();
        SparseColMat::try_new_from_triplets(n, n, &triplets).unwrap()
    }
    fn certify(q: &SparseColMat<usize, f64>, sign: f64, limit: usize) -> Definiteness {
        GramCertificate::certify(q, sign, limit, &AtomicBool::new(false)).unwrap()
    }
    /// `Σ wₖ rₖ rₖᵀ` evaluated exactly, as a dense rational matrix.
    fn reconstruct(n: usize, factors: &GramFactors) -> Vec<Vec<Rational>> {
        let mut m = vec![vec![Rational::zero(); n]; n];
        for (w, row) in factors.terms() {
            for (i, a) in row {
                for (j, b) in row {
                    m[*i][*j] += &(&(w * a) * b);
                }
            }
        }
        m
    }
    /// ADR-0121 Outcome 2: the exact rational LDLᵀ with symmetric pivoting certifies
    /// nondiagonal positive semidefinite matrices that floating-point factors cannot (a
    /// non-dyadic Schur complement, and a leading zero diagonal of a singular matrix that
    /// needs a pivot other than the first), reproduces every entry exactly from its kept
    /// factors, and finds exact negative directions of indefinite matrices.
    #[test]
    fn exact_ldlt_certifies_nondiagonal_psd() {
        let cases = [
            // [[3, 1], [1, 3]]: Schur complement 8/3 is not a binary64 number.
            (2, vec![(0, 0, 3.0), (0, 1, 1.0), (1, 0, 1.0), (1, 1, 3.0)]),
            // [[0.1, 0.3], [0.3, 0.9]] in binary64: the determinant is exactly 2⁻⁵⁶, which a
            // floating-point Schur complement loses.
            (2, vec![(0, 0, 0.1), (0, 1, 0.3), (1, 0, 0.3), (1, 1, 0.9)]),
            // [[0.1, 0.2], [0.2, 0.4]] in binary64: singular exactly.
            (2, vec![(0, 0, 0.1), (0, 1, 0.2), (1, 0, 0.2), (1, 1, 0.4)]),
            // [[0, 0, 0], [0, 2, 1], [0, 1, 1]]: a zero row and a nondiagonal block.
            (3, vec![(1, 1, 2.0), (1, 2, 1.0), (2, 1, 1.0), (2, 2, 1.0)]),
            // [[1, 1, 1], [1, 1, 1], [1, 1, 1]]: rank one.
            (
                3,
                (0..3)
                    .flat_map(|i| (0..3).map(move |j| (i, j, 1.0)))
                    .collect(),
            ),
        ];
        for (n, entries) in cases {
            let q = square(n, &entries);
            let Definiteness::Psd(proof) = certify(&q, 1.0, 1000) else {
                panic!("PSD: {entries:?}")
            };
            proof.validate(&q, 1.0).unwrap();
            assert!(proof.validate(&q, -1.0).is_err());
            assert!(proof.factors().terms().all(|(w, _)| !w.is_negative() && !w.is_zero()));
            let exact = reconstruct(n, proof.factors());
            let dense = q.to_dense();
            for i in 0..n {
                for j in 0..n {
                    assert_eq!(exact[i][j], Rational::try_from(dense[(i, j)]).unwrap());
                }
            }
        }
        // The rank-one matrices have one square each.
        let singular = square(2, &[(0, 0, 0.1), (0, 1, 0.2), (1, 0, 0.2), (1, 1, 0.4)]);
        let Definiteness::Psd(proof) = certify(&singular, 1.0, 1000) else {
            panic!("singular PSD")
        };
        assert_eq!(proof.factors().rank(), 1);
        let ones = square(
            3,
            &(0..3)
                .flat_map(|i| (0..3).map(move |j| (i, j, 1.0)))
                .collect::<Vec<_>>(),
        );
        let Definiteness::Psd(proof) = certify(&ones, 1.0, 1000) else {
            panic!("rank one is PSD")
        };
        assert_eq!(proof.factors().rank(), 1);
        // Indefinite: a negative diagonal, a zero diagonal with an off-diagonal entry, and
        // a negative Schur complement.
        for (n, entries) in [
            (1, vec![(0, 0, -1.0)]),
            (2, vec![(0, 1, 1.0), (1, 0, 1.0), (1, 1, 1.0)]),
            (2, vec![(0, 0, 1.0), (0, 1, 2.0), (1, 0, 2.0), (1, 1, 1.0)]),
        ] {
            assert!(matches!(
                certify(&square(n, &entries), 1.0, 1000),
                Definiteness::Indefinite
            ));
        }
        // The concave quadratic of a maximization is certified in the minimization sense.
        let concave = square(2, &[(0, 0, -3.0), (0, 1, -1.0), (1, 0, -1.0), (1, 1, -3.0)]);
        assert!(matches!(certify(&concave, 1.0, 1000), Definiteness::Indefinite));
        assert!(matches!(certify(&concave, -1.0, 1000), Definiteness::Psd(_)));
        // An exhausted allowance is inconclusive, never convex.
        assert!(matches!(
            certify(&square(2, &[(0, 0, 3.0), (0, 1, 1.0), (1, 0, 1.0), (1, 1, 3.0)]), 1.0, 1),
            Definiteness::Inconclusive
        ));
        // An unsymmetric matrix is a contract error, not a decision.
        assert!(
            GramCertificate::certify(
                &square(2, &[(0, 1, 1.0), (1, 0, 2.0)]),
                1.0,
                1000,
                &AtomicBool::new(false)
            )
            .is_err()
        );
    }
    /// A diagonal matrix of any size is certified in linear work: the symmetric pivoting
    /// takes rows without fill.
    #[test]
    fn exact_ldlt_of_sparse_diagonal_is_linear() {
        let n = 20_000;
        let entries: Vec<_> = (0..n).map(|i| (i, i, 1.0 + i as f64)).collect();
        assert!(matches!(
            certify(&square(n, &entries), 1.0, 2 * n),
            Definiteness::Psd(_)
        ));
    }

    fn matrix(entries: &[(usize, usize, f64)]) -> SparseColMat<usize, f64> {
        let triplets: Vec<_> = entries
            .iter()
            .map(|(r, c, v)| Triplet::new(*r, *c, *v))
            .collect();
        SparseColMat::try_new_from_triplets(2, 2, &triplets).unwrap()
    }
    /// The minimization form negates a maximized concave quadratic exactly, carries its
    /// evidence to exactly the returned matrix under orientation one, and refuses stale
    /// evidence and a nonzero matrix without evidence.
    #[test]
    fn minimization_form_negates_and_retargets_evidence() {
        // Maximize -(2x² + 2xy + 2y²)/2: Q = -[[2, 1], [1, 2]] is certified with sign -1.
        let q = matrix(&[(0, 0, -2.0), (0, 1, -1.0), (1, 0, -1.0), (1, 1, -2.0)]);
        let Definiteness::Psd(proof) = certify(&q, -1.0, 10) else {
            panic!("a concave quadratic is certified in the minimization sense")
        };
        let (p, evidence) = minimization_form(&q, -1.0, Some(&proof)).unwrap();
        assert_eq!(p.to_dense(), faer::mat![[2.0, 1.0], [1.0, 2.0]]);
        evidence.validate(&p, 1.0).unwrap();
        assert!(evidence.validate(&q, -1.0).is_err());
        // Evidence of another orientation, and a nonzero matrix without evidence.
        assert!(minimization_form(&q, 1.0, Some(&proof)).is_err());
        assert!(minimization_form(&q, -1.0, None).is_err());
        // The zero quadratic of a linear program needs none.
        let (zero, evidence) = minimization_form(&matrix(&[]), -1.0, None).unwrap();
        assert!(zero.val().is_empty());
        evidence.validate(&zero, 1.0).unwrap();
    }
}
