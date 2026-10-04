// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Sparse conditioning and certified inertia through FERAL (Plan 22 N4, L-N6).
//!
//! A square Jacobian's 1-norm condition number is estimated from a sparse LU; a symmetric
//! KKT matrix `[H Aᵀ; A 0]` is factored as LDLᵀ, whose pivot signs give its inertia
//! (Sylvester's law) and whose factor gives the 1-norm condition estimate. The KKT-point
//! analysis at an NLP candidate ([`crate::kkt`]) factors through the same entry points.
use crate::ProblemError;
use faer::sparse::SparseColMatRef;
use pse_math::index::Triplet;

/// Hager–Higham estimate of the 1-norm condition number of the square matrix
/// `diag(row_scales) · A · diag(column_scales)`. The estimate is a lower bound on the true
/// value; `None` when the sparse LU finds the matrix singular at FERAL's pivot tolerance.
///
/// # Errors
/// A non-square or non-finite input, or a failed factorization other than singularity.
pub fn jacobian_condition(
    matrix: SparseColMatRef<'_, usize, f64>,
    row_scales: &[f64],
    column_scales: &[f64],
) -> Result<Option<f64>, ProblemError> {
    let m = matrix.nrows();
    if matrix.ncols() != m || row_scales.len() != m || column_scales.len() != m {
        return Err(ProblemError::Contract(
            "a condition estimate needs a square matrix and one scale per row and column".into(),
        ));
    }
    if row_scales
        .iter()
        .chain(column_scales)
        .any(|s| !s.is_finite() || *s <= 0.0)
    {
        return Err(ProblemError::Contract(
            "condition estimate scales must be finite and positive".into(),
        ));
    }
    let columns = (0..m)
        .map(|j| {
            matrix
                .row_idx_of_col(j)
                .zip(matrix.val_of_col(j))
                .map(|(i, v)| (i, row_scales[i] * v * column_scales[j]))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    if columns.iter().flatten().any(|(_, v)| !v.is_finite()) {
        return Err(ProblemError::numerical("nonfinite Jacobian entry"));
    }
    let a = feral::SparseColMatrix::from_sparse_columns(m, &columns).map_err(native)?;
    let mut lu = match feral::SparseLu::factor_markowitz(&a, feral::LuParams::default()) {
        Ok(lu) => lu,
        Err(feral::FeralError::SingularBasis { .. }) => return Ok(None),
        Err(e) => return Err(native(e)),
    };
    match lu.condition_estimate_1(&a) {
        Ok(v) if v.is_finite() => Ok(Some(v)),
        Ok(_) | Err(feral::FeralError::SingularBasis { .. }) => Ok(None),
        Err(e) => Err(native(e)),
    }
}

/// Inertia and conditioning of a symmetric KKT matrix.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Kkt {
    /// Certified inertia: positive, negative and zero pivot counts of the LDLᵀ factor.
    pub inertia: (usize, usize, usize),
    /// Hager–Higham 1-norm condition estimate (a lower bound); `None` when unavailable.
    pub condition_1norm: Option<f64>,
}

/// Factor `[H Aᵀ; A 0]` of order `n + m` with a serial FERAL LDLᵀ. `hessian` holds the
/// lower triangle of the symmetric `n × n` block over the column space `C` (entries above
/// the diagonal are mirrored); `constraints` holds the `m × n` block `A`, rows of `R` by
/// columns of `C`. A Hessian triplet is `(column, column)`, so the constraint block cannot
/// stand in for it:
///
/// ```
/// use pse_backend_native::conditioning::kkt;
/// use pse_math::index::{ReducedCol, ReducedRow, Triplet};
///
/// let hessian = [Triplet::new(ReducedCol::new(0), ReducedCol::new(0), 2.0)];
/// let jacobian = [Triplet::new(ReducedRow::new(0), ReducedCol::new(0), 1.0)];
/// assert_eq!(kkt(1, 1, &hessian, &jacobian).unwrap().inertia, (1, 1, 0));
/// ```
///
/// ```compile_fail,E0308
/// use pse_backend_native::conditioning::kkt;
/// use pse_math::index::{ReducedCol, ReducedRow, Triplet};
///
/// let jacobian = [Triplet::new(ReducedRow::new(0), ReducedCol::new(0), 1.0)];
/// let _ = kkt(1, 1, &jacobian, &jacobian);
/// ```
///
/// # Errors
/// Indices out of range, non-finite values, or a failed factorization.
pub fn kkt<R, C>(
    n: usize,
    m: usize,
    hessian: &[Triplet<C, C>],
    constraints: &[Triplet<R, C>],
) -> Result<Kkt, ProblemError>
where
    R: Copy + Into<usize>,
    C: Copy + Into<usize>,
{
    let order = n + m;
    let mut rows = Vec::with_capacity(order + hessian.len() + constraints.len());
    let mut cols = Vec::with_capacity(rows.capacity());
    let mut values = Vec::with_capacity(rows.capacity());
    // Every diagonal is structurally present, so a zero pivot is counted, never skipped.
    for i in 0..order {
        rows.push(i);
        cols.push(i);
        values.push(0.0);
    }
    // FERAL's interior speaks `usize`; the typed spaces end here.
    for t in hessian {
        let (i, j, v): (usize, usize, f64) = (t.row.into(), t.col.into(), t.value);
        if i >= n || j >= n {
            return Err(ProblemError::internal("KKT Hessian index"));
        }
        rows.push(i.max(j));
        cols.push(i.min(j));
        values.push(v);
    }
    for t in constraints {
        let (r, j, v): (usize, usize, f64) = (t.row.into(), t.col.into(), t.value);
        if r >= m || j >= n {
            return Err(ProblemError::internal("KKT constraint index"));
        }
        rows.push(n + r);
        cols.push(j);
        values.push(v);
    }
    if values.iter().any(|v| !v.is_finite()) {
        return Err(ProblemError::numerical("nonfinite KKT entry"));
    }
    if order == 0 {
        return Ok(Kkt {
            inertia: (0, 0, 0),
            condition_1norm: None,
        });
    }
    let matrix = feral::CscMatrix::from_triplets(order, &rows, &cols, &values).map_err(native)?;
    let (solver, inertia) = factor(&matrix)?;
    Ok(Kkt {
        inertia,
        condition_1norm: solver
            .estimate_condition_1norm(&matrix)
            .ok()
            .filter(|v| v.is_finite()),
    })
}

/// Factor a symmetric matrix (its lower triangle) with a new serial FERAL LDLᵀ solver, and
/// read its certified inertia (positive, negative, zero).
///
/// # Errors
/// A factorization that reports no inertia, or an inertia that does not cover the order.
pub(crate) fn factor(
    matrix: &feral::CscMatrix,
) -> Result<(feral::Solver, (usize, usize, usize)), ProblemError> {
    // Serial and FMA-free: a diagnostic owns no admitted thread team.
    let mut solver = feral::Solver::new().with_parallel(false);
    let inertia = factor_into(&mut solver, matrix)?;
    Ok((solver, inertia))
}
/// [`factor`] on an existing solver, which keeps its symbolic analysis for an unchanged
/// pattern.
///
/// # Errors
/// As [`factor`].
pub(crate) fn factor_into(
    solver: &mut feral::Solver,
    matrix: &feral::CscMatrix,
) -> Result<(usize, usize, usize), ProblemError> {
    match solver.factor(matrix, None) {
        feral::FactorStatus::Success | feral::FactorStatus::WrongInertia { .. } => {}
        feral::FactorStatus::Singular => {
            return Err(ProblemError::numerical(
                "KKT factorization reported a singular matrix without an inertia",
            ));
        }
        feral::FactorStatus::FatalError(e) => return Err(native(e)),
    }
    let inertia = solver
        .inertia()
        .ok_or_else(|| ProblemError::internal("FERAL factor without an inertia"))?;
    if inertia.total() != matrix.n {
        return Err(ProblemError::internal(
            "FERAL inertia does not cover the KKT order",
        ));
    }
    Ok((inertia.positive, inertia.negative, inertia.zero))
}

pub(crate) fn native(error: feral::FeralError) -> ProblemError {
    use crate::LinearFailureKind as K;
    let kind = match &error {
        feral::FeralError::InvalidInput(_) | feral::FeralError::DimensionMismatch { .. } => {
            K::Contract
        }
        feral::FeralError::IoError(_) | feral::FeralError::NoFactor => K::Internal,
        feral::FeralError::DelayBudgetExceeded { .. } => K::Memory,
        feral::FeralError::NumericallyRankDeficient
        | feral::FeralError::SingularBasis { .. }
        | feral::FeralError::SqdContractViolated { .. }
        | feral::FeralError::NeedsRefactor => K::Numerical,
    };
    ProblemError::Linear {
        kind,
        cause: Box::new(error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn feral_failure_mapping_retains_original_typed_cause_and_true_disposition() {
        use crate::LinearFailureKind as K;
        let cases = [
            (
                feral::FeralError::InvalidInput("diagnostic containing numerical words".into()),
                K::Contract,
            ),
            (
                feral::FeralError::DimensionMismatch {
                    expected: 3,
                    got: 2,
                },
                K::Contract,
            ),
            (
                feral::FeralError::IoError("diagnostic containing singular words".into()),
                K::Internal,
            ),
            (feral::FeralError::NoFactor, K::Internal),
            (
                feral::FeralError::DelayBudgetExceeded {
                    supernode: 2,
                    required: 7,
                    capacity: 3,
                },
                K::Memory,
            ),
            (feral::FeralError::NumericallyRankDeficient, K::Numerical),
            (feral::FeralError::SingularBasis { column: 4 }, K::Numerical),
            (
                feral::FeralError::SqdContractViolated {
                    column: 5,
                    pivot: 0.25,
                },
                K::Numerical,
            ),
            (feral::FeralError::NeedsRefactor, K::Numerical),
        ];
        for (cause, expected) in cases {
            let original = std::mem::discriminant(&cause);
            let ProblemError::Linear { kind, cause } = native(cause) else {
                panic!("typed FERAL cause flattened");
            };
            assert_eq!(kind, expected);
            assert_eq!(std::mem::discriminant(cause.as_ref()), original);
            if let feral::FeralError::DelayBudgetExceeded {
                supernode,
                required,
                capacity,
            } = *cause
            {
                assert_eq!((supernode, required, capacity), (2, 7, 3));
            }
        }
    }
    use faer::{
        linalg::solvers::DenseSolveCore,
        sparse::{SparseColMat, Triplet},
    };
    use pse_math::index::{ReducedCol, ReducedRow};

    /// Exact κ₁ from the dense inverse.
    fn dense_condition(a: &[[f64; 4]; 4]) -> f64 {
        let dense = faer::Mat::from_fn(4, 4, |i, j| a[i][j]);
        let inverse = dense.partial_piv_lu().inverse();
        let norm = |m: &faer::Mat<f64>| {
            (0..4)
                .map(|j| (0..4).map(|i| m[(i, j)].abs()).sum::<f64>())
                .fold(0.0, f64::max)
        };
        norm(&dense) * norm(&inverse)
    }

    #[test]
    fn jacobian_condition_estimate_matches_dense_reference() {
        // A sparse nonsymmetric Jacobian with a wide spread of magnitudes.
        let a = [
            [4.0, 0.0, 1e-3, 0.0],
            [0.0, 2e2, 0.0, -3.0],
            [1.0, 0.0, 5e-2, 0.0],
            [0.0, -1.0, 0.0, 7.0],
        ];
        let triplets = (0..4)
            .flat_map(|i| (0..4).map(move |j| (i, j)))
            .filter(|&(i, j)| a[i][j] != 0.0)
            .map(|(i, j)| Triplet::new(i, j, a[i][j]))
            .collect::<Vec<_>>();
        let matrix = SparseColMat::try_new_from_triplets(4, 4, &triplets).unwrap();
        let reference = dense_condition(&a);
        let estimate = jacobian_condition(matrix.as_ref(), &[1.0; 4], &[1.0; 4])
            .unwrap()
            .unwrap();
        // Hager–Higham returns a lower bound, exact on small matrices in practice.
        assert!(
            estimate <= reference * (1.0 + 1e-10),
            "{estimate} {reference}"
        );
        assert!(
            estimate >= reference * (1.0 - 1e-8),
            "{estimate} {reference}"
        );
        // Scaling enters as diag(r)·A·diag(c): the estimate is that of the scaled matrix.
        let (r, c) = ([1.0, 1e-2, 1.0, 0.5], [2.0, 1.0, 10.0, 1.0]);
        let mut scaled = a;
        for (i, row) in scaled.iter_mut().enumerate() {
            for (j, v) in row.iter_mut().enumerate() {
                *v *= r[i] * c[j];
            }
        }
        let estimate = jacobian_condition(matrix.as_ref(), &r, &c)
            .unwrap()
            .unwrap();
        let reference = dense_condition(&scaled);
        assert!(
            (estimate - reference).abs() <= 1e-8 * reference,
            "{estimate} {reference}"
        );
        // A singular matrix has no estimate; a rectangular one is refused.
        let singular = SparseColMat::try_new_from_triplets(
            2,
            2,
            &[
                Triplet::new(0, 0, 1.0),
                Triplet::new(0, 1, 1.0),
                Triplet::new(1, 0, 2.0),
                Triplet::new(1, 1, 2.0),
            ],
        )
        .unwrap();
        assert_eq!(
            jacobian_condition(singular.as_ref(), &[1.0; 2], &[1.0; 2]).unwrap(),
            None
        );
        let wide = SparseColMat::try_new_from_triplets(1, 2, &[Triplet::new(0, 0, 1.0)]).unwrap();
        assert!(jacobian_condition(wide.as_ref(), &[1.0], &[1.0; 2]).is_err());
    }

    type Hessian = pse_math::index::Triplet<ReducedCol, ReducedCol>;
    type Jacobian = pse_math::index::Triplet<ReducedRow, ReducedCol>;
    fn h(i: usize, j: usize, v: f64) -> Hessian {
        Hessian::new(ReducedCol::new(i), ReducedCol::new(j), v)
    }
    fn a(r: usize, j: usize, v: f64) -> Jacobian {
        Jacobian::new(ReducedRow::new(r), ReducedCol::new(j), v)
    }

    #[test]
    fn kkt_inertia_counts_pivot_signs() {
        // H = diag(2, -1), A = [1 1]: Zᵀ H Z = (2 - 1)/2 > 0, so In(K) = (2, 1, 0).
        let k = kkt(
            2,
            1,
            &[h(0, 0, 2.0), h(1, 1, -1.0)],
            &[a(0, 0, 1.0), a(0, 1, 1.0)],
        )
        .unwrap();
        assert_eq!(k.inertia, (2, 1, 0));
        assert!(k.condition_1norm.is_some_and(|c| c >= 1.0));
        // H = diag(1, -2): Zᵀ H Z < 0, so In(K) = (1, 2, 0).
        let k = kkt(
            2,
            1,
            &[h(0, 0, 1.0), h(1, 1, -2.0)],
            &[a(0, 0, 1.0), a(0, 1, 1.0)],
        )
        .unwrap();
        assert_eq!(k.inertia, (1, 2, 0));
        // Dependent constraint rows leave a zero eigenvalue.
        let k = kkt(
            2,
            2,
            &[h(0, 0, 1.0), h(1, 1, 1.0)],
            &[a(0, 0, 1.0), a(0, 1, 1.0), a(1, 0, 2.0), a(1, 1, 2.0)],
        )
        .unwrap();
        assert_eq!(k.inertia.2, 1, "{:?}", k.inertia);
    }
}
