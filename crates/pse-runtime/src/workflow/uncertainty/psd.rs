// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Dimensionless covariance admission. Statistical origin is a separate premise.
use crate::workflow::{WorkflowError, contract, relation};
use pse_columnar::{CancellationToken, MemoryPool};
use std::sync::Arc;

/// Actual numerical premises of the lower-triangle, sequential faer admission.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CovariancePsdAssessment {
    /// Infinity norm of `diag(1/scales) * covariance * diag(1/scales)`.
    pub normalized_norm_inf: f64,
    /// `64 * EPSILON * n * max(1, normalized_norm_inf)`.
    pub tolerance: f64,
    /// Least computed eigenvalue; zero and singular matrices are permitted.
    pub minimum_eigenvalue: f64,
}

fn checkpoint(cancel: &CancellationToken) -> Result<(), WorkflowError> {
    cancel.checkpoint().map_err(pse_relations::RelationError::from).map_err(relation)
}

/// Check extents before even asking the library for its scratch layout.
fn square_extent(n: usize) -> Result<usize, WorkflowError> {
    n.checked_mul(n)
        .filter(|_| n > 0)
        // faer's internal temporary matrices and block panels must have representable
        // layouts too. This is an address-space check, not another memory allowance.
        .filter(|cells| cells.checked_mul(64).is_some_and(|bytes| bytes <= isize::MAX as usize))
        .ok_or_else(|| contract("covariance matrix/scratch extent is not representable"))
}

pub(super) fn admit(
    n: usize,
    values: &[f64],
    scales: &[f64],
    pool: &Arc<dyn MemoryPool>,
    cancel: &CancellationToken,
) -> Result<CovariancePsdAssessment, WorkflowError> {
    checkpoint(cancel)?;
    let cells = square_extent(n)?;
    if values.len() != cells || scales.len() != n
        || values.iter().any(|value| !value.is_finite())
        || scales.iter().any(|scale| !scale.is_finite() || *scale <= 0.0)
    {
        return Err(contract("covariance admission requires a finite square matrix and positive parameter scales"));
    }
    let scratch = faer::linalg::evd::self_adjoint_evd_scratch::<f64>(
        n, faer::linalg::evd::ComputeEigenvectors::No, faer::Par::Seq, Default::default(),
    );
    let _working = super::super::result_export::working(
        pool, "uncertainty:psd-admission",
        &[(cells, size_of::<f64>()), (n, size_of::<f64>()), (scratch.unaligned_bytes_required(), 1)],
    )?;
    // These slices have no hidden matrix padding. Scratch is the library's actual
    // requested workspace, prepaid before any of the numeric allocations below.
    let mut normalized = Vec::new();
    normalized.try_reserve_exact(cells).map_err(|error| contract(format!("covariance matrix allocation: {error}")))?;
    for (index, value) in values.iter().enumerate() {
        if index % 1024 == 0 { checkpoint(cancel)?; }
        let (row, column) = (index / n, index % n);
        let value = value / scales[row] / scales[column];
        if !value.is_finite() { return Err(contract("covariance normalization overflowed")); }
        normalized.push(value);
    }
    let mut norm = 0.0_f64;
    for row in normalized.chunks_exact(n) {
        let sum = row.iter().try_fold(0.0, |sum, value| {
            let sum = sum + value.abs();
            sum.is_finite().then_some(sum)
                .ok_or_else(|| contract("covariance normalized infinity norm overflowed"))
        })?;
        norm = norm.max(sum);
    }
    let tolerance = 64.0 * f64::EPSILON * n as f64 * norm.max(1.0);
    if !tolerance.is_finite() { return Err(contract("covariance PSD tolerance overflowed")); }
    for row in 0..n {
        for column in 0..row {
            if (normalized[row * n + column] - normalized[column * n + row]).abs() > tolerance {
                return Err(contract("covariance is not symmetric within its dimensionless tolerance"));
            }
        }
    }
    let mut eigenvalues = Vec::new();
    eigenvalues.try_reserve_exact(n).map_err(|error| contract(format!("covariance eigenvalue allocation: {error}")))?;
    eigenvalues.resize(n, 0.0);
    let mut memory = faer::dyn_stack::MemBuffer::try_new(scratch)
        .map_err(|_| contract("covariance eigensolver scratch allocation failed"))?;
    checkpoint(cancel)?;
    // No global parallelism and no eigenvectors. Cancellation cannot interrupt faer's
    // call: this stack retains its entire allowance until that call has returned.
    faer::linalg::evd::self_adjoint_evd(
        faer::MatRef::from_row_major_slice(&normalized, n, n),
        faer::diag::DiagMut::from_slice_mut(&mut eigenvalues), None, faer::Par::Seq,
        faer::dyn_stack::MemStack::new(&mut memory), Default::default(),
    ).map_err(|error| contract(format!("covariance eigensolver did not converge: {error:?}")))?;
    checkpoint(cancel)?;
    if eigenvalues.iter().any(|value| !value.is_finite()) {
        return Err(contract("covariance eigensolver returned nonfinite eigenvalues"));
    }
    let minimum = eigenvalues.iter().copied().fold(f64::INFINITY, f64::min);
    if minimum < -tolerance {
        return Err(contract("covariance has a negative eigenvalue beyond its dimensionless tolerance"));
    }
    Ok(CovariancePsdAssessment { normalized_norm_inf: norm, tolerance, minimum_eigenvalue: minimum })
}

#[cfg(test)]
mod tests {
    use super::*;
    use datafusion::execution::memory_pool::GreedyMemoryPool;
    fn pool(bytes: usize) -> Arc<dyn MemoryPool> { Arc::new(GreedyMemoryPool::new(bytes)) }
    #[test]
    fn indefinite_covariance_is_refused_although_its_diagonal_is_positive() {
        let values = [1.0, 2.0, 2.0, 1.0];
        // Independent sandwich oracle for J=[1,-1].
        assert_eq!(values[0] - values[1] - values[2] + values[3], -2.0);
        assert!(admit(2, &values, &[1.0, 1.0], &pool(1 << 20), &CancellationToken::new()).is_err());
    }
    #[test]
    fn covariance_psd_refuses_asymmetry_before_lower_triangle_eigendecomposition() {
        let pool = pool(1 << 20);
        let cancel = CancellationToken::new();
        // The lower triangle alone is an identity matrix. It cannot establish
        // covariance validity when the independent upper entry disagrees.
        assert!(admit(2, &[1.0, 0.5, 0.0, 1.0], &[1.0; 2], &pool, &cancel).is_err());
        assert!(admit(1, &[-1.0], &[1.0], &pool, &cancel).is_err());
        assert_eq!(pool.reserved(), 0);
    }
    #[test]
    fn zero_singular_and_mixed_unit_psd_covariances_are_admitted() {
        let pool = pool(1 << 20);
        let cancel = CancellationToken::new();
        for (values, scales) in [([0.0; 4], [1.0, 1.0]), ([4.0, 6.0, 6.0, 9.0], [2.0, 3.0]), ([4e12, 6.0, 6.0, 9e-12], [2e6, 3e-6])] {
            let assessment = admit(2, &values, &scales, &pool, &cancel).unwrap();
            assert!(assessment.minimum_eigenvalue >= -assessment.tolerance);
        }
        assert_eq!(pool.reserved(), 0);
    }
    #[test]
    fn covariance_psd_tolerance_has_a_finite_declared_boundary() {
        let pool = pool(1 << 20);
        let cancel = CancellationToken::new();
        let tau = 128.0 * f64::EPSILON;
        assert!(admit(2, &[1.0, 0.0, 0.0, -tau / 2.0], &[1.0; 2], &pool, &cancel).is_ok());
        assert!(admit(2, &[1.0, 0.0, 0.0, -tau * 2.0], &[1.0; 2], &pool, &cancel).is_err());
        assert!(admit(2, &[f64::MAX; 4], &[1.0; 2], &pool, &cancel).is_err());
        assert!(admit(2, &[1.0; 4], &[f64::MIN_POSITIVE; 2], &pool, &cancel).is_err());
        assert_eq!(pool.reserved(), 0);
    }
    #[test]
    fn covariance_psd_refuses_extent_capacity_and_cancellation_before_work() {
        let small = pool(1);
        let cancel = CancellationToken::new();
        assert!(admit(2, &[1.0, 0.0, 0.0, 1.0], &[1.0; 2], &small, &cancel).is_err());
        assert_eq!(small.reserved(), 0);
        assert!(square_extent(usize::MAX).is_err());
        cancel.cancel();
        let pool = pool(1 << 20);
        assert!(admit(2, &[1.0; 4], &[1.0; 2], &pool, &cancel).is_err());
        assert_eq!(pool.reserved(), 0);
    }
}
