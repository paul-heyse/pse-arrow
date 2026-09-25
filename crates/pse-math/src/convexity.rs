// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Exact witnesses and opt-in numerical evidence for an unchanged quadratic snapshot.
use crate::{
    MathError,
    coefficients::{Coefficients, GramCertificate},
};
use pse_ids::{ContentHash, FramedHasher};
use std::sync::atomic::{AtomicBool, Ordering};

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
        if self.matrix != crate::coefficients::quadratic_identity(q, sign) {
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
        matrix: crate::coefficients::quadratic_identity(&result, sign),
        assumptions: proof.assumptions(),
    };
    Ok((result, evidence))
}

/// Numerical assessment is a separate, explicit permission from exact certification.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum ConvexityPolicy {
    /// Only a verified represented-rational Gram identity establishes convexity.
    #[default]
    Exact,
    /// Permit a residual-qualified PSD assessment in normalized coordinates.
    Numerical {
        /// Absolute eigenvalue budget in normalized objective coordinates.
        absolute: f64,
        /// Relative budget against the normalized matrix Frobenius norm.
        relative: f64,
    },
}
/// A bounded assessment that did not establish either requested conclusion.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InconclusiveReason {
    /// The caller's allocation or exact-operation allowance is insufficient.
    ResourceLimit,
    /// No exact Gram witness was established and numerical assessment is disabled.
    NoExactWitness,
    /// The eigensolver did not converge or returned invalid values.
    Eigensolver,
    /// Residual or orthogonality qualification failed.
    Residual,
    /// The uncertainty interval crosses the requested PSD threshold.
    Threshold,
}
/// Scientific interpretation; numerical PSD never masquerades as an exact certificate.
#[derive(Clone, Debug)]
pub enum ConvexityAssessment {
    /// Exact identity against every represented matrix entry and objective sense.
    Exact(GramCertificate),
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
/// Nonforgeable evidence tied to coefficients, values, orientation and normalization.
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
        if self.matrix != crate::coefficients::quadratic_identity(q, sign) {
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
    /// Eligibility under the explicitly selected evidence policy.
    pub fn accepted(&self) -> bool {
        matches!(
            self.assessment,
            ConvexityAssessment::Exact(_) | ConvexityAssessment::NumericalPsd { .. }
        )
    }
}
/// Caller-owned resource allowance. Computation is serial within the caller's CPU lease.
#[derive(Clone, Copy, Debug)]
pub struct ConvexityLimits {
    /// Maximum dense work and library scratch allocation.
    pub bytes: usize,
    /// Maximum represented-rational multiply/add witness work.
    pub exact_operations: usize,
}
impl Coefficients {
    /// Assess the admitted snapshot without changing Q or silently repairing curvature.
    pub fn convexity(
        &self,
        sign: f64,
        scales: &[f64],
        objective_scale: f64,
        policy: ConvexityPolicy,
        limits: ConvexityLimits,
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
        let mut h = FramedHasher::new("pse.math.convexity.v1");
        h.hash(&self.assumptions)
            .u64(sign.to_bits())
            .u64(objective_scale.to_bits());
        for v in scales {
            h.u64(v.to_bits());
        }
        let tolerance = match policy {
            ConvexityPolicy::Exact => {
                h.u64(0);
                None
            }
            ConvexityPolicy::Numerical { absolute, relative } => {
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
                h.u64(1).u64(absolute.to_bits()).u64(relative.to_bits());
                Some((absolute, relative))
            }
        };
        h.str("faer-0.24.4;serial-evd;residual-v1")
            .u64(limits.bytes as u64)
            .u64(limits.exact_operations as u64);
        let matrix = crate::coefficients::quadratic_identity(&self.hessian, sign);
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
        if dense_bytes.is_none_or(|v| v > limits.bytes) {
            return Ok(unknown(InconclusiveReason::ResourceLimit));
        }
        let original = self.hessian.to_dense();
        if (0..n).any(|i| (0..n).any(|j| original[(i, j)] != original[(j, i)])) {
            return Err(MathError::Contract("asymmetric quadratic matrix".into()));
        }
        let signed = faer::Mat::from_fn(n, n, |i, j| sign * original[(i, j)]);
        let diagonal = (0..n).all(|i| (0..n).all(|j| i == j || signed[(i, j)] == 0.0));
        let witness = if diagonal {
            Some((
                faer::Mat::identity(n, n),
                (0..n).map(|i| signed[(i, i)]).collect::<Vec<_>>(),
            ))
        } else if n
            .checked_mul(n)
            .and_then(|v| v.checked_mul(n))
            .is_some_and(|v| v <= limits.exact_operations)
        {
            // faer supplies candidate factors. Exact verification below owns the conclusion.
            let mut factors = signed.clone();
            let scratch = faer::linalg::cholesky::ldlt::factor::cholesky_in_place_scratch::<f64>(
                n,
                faer::Par::Seq,
                Default::default(),
            );
            if dense_bytes
                .and_then(|b| b.checked_add(scratch.unaligned_bytes_required()))
                .is_none_or(|b| b > limits.bytes)
            {
                return Ok(unknown(InconclusiveReason::ResourceLimit));
            }
            let mut memory = faer::dyn_stack::MemBuffer::new(scratch);
            faer::linalg::cholesky::ldlt::factor::cholesky_in_place(
                factors.as_mut(),
                Default::default(),
                faer::Par::Seq,
                faer::dyn_stack::MemStack::new(&mut memory),
                Default::default(),
            )
            .ok()
            .map(|_| {
                let weights = (0..n).map(|i| factors[(i, i)]).collect::<Vec<_>>();
                let transposed = faer::Mat::from_fn(n, n, |i, j| {
                    if i == j {
                        1.0
                    } else if i < j {
                        factors[(j, i)]
                    } else {
                        0.0
                    }
                });
                (transposed, weights)
            })
        } else {
            None
        };
        if let Some((factors, weights)) = witness
            && let Ok(proof) = GramCertificate::new(
                &self.hessian,
                sign,
                &factors,
                &weights,
                limits.exact_operations,
            )
        {
            return Ok(result(ConvexityAssessment::Exact(proof)));
        }
        let Some((absolute, relative)) = tolerance else {
            return Ok(unknown(InconclusiveReason::NoExactWitness));
        };
        if cancel.load(Ordering::Relaxed) {
            return Err(MathError::Cancelled);
        }
        let mut a = faer::Mat::zeros(n, n);
        for i in 0..n {
            for j in 0..n {
                a[(i, j)] = crate::normalization::checked_ratio(
                    crate::normalization::checked_product(
                        crate::normalization::checked_product(signed[(i, j)], scales[i])?,
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
            .is_none_or(|b| b > limits.bytes)
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
