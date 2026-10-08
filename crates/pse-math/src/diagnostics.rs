// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Bounded numerical evidence; decomposition and matrix products belong to faer.
use crate::MathError;
use faer::{Mat, sparse::SparseColMatRef};
use std::sync::atomic::{AtomicBool, Ordering};

/// Explicit diagnostic work and numerical classification limits.
#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MatrixPolicy {
    /// Includes dense input and both complete singular-vector factors/Gram products.
    pub dense_entries: usize,
    /// Maximum parallel row and column pairs reported before the analysis is refused.
    pub findings: usize,
    /// Pairs with `1 - |cosine|` at or below this are reported as parallel; in [0, 1).
    pub parallel_tolerance: f64,
    /// Absolute singular-value cutoff for numerical rank.
    pub rank_absolute: f64,
    /// Singular-value cutoff relative to the largest singular value; in [0, 1).
    pub rank_relative: f64,
    /// Magnitude above which a row or column's component of a singular vector below the
    /// rank cutoff names it as a member of that near-null mode; in [0, 1).
    pub singular_vector: f64,
}
impl MatrixPolicy {
    /// Validate explicit classification and work bounds before allocating an analysis.
    pub fn validate(&self) -> Result<(), MathError> {
        if self.dense_entries == 0
            || self.findings == 0
            || !self.parallel_tolerance.is_finite()
            || !(0.0..1.0).contains(&self.parallel_tolerance)
            || !self.rank_absolute.is_finite()
            || self.rank_absolute < 0.0
            || !self.rank_relative.is_finite()
            || !(0.0..1.0).contains(&self.rank_relative)
            || !self.singular_vector.is_finite()
            || !(0.0..1.0).contains(&self.singular_vector)
        {
            return Err(MathError::Contract("diagnostic matrix policy".into()));
        }
        Ok(())
    }
}
/// Singular vectors in scaled source-coordinate order. A rectangular null mode has
/// an empty vector on the side for which no corresponding singular vector exists.
#[derive(Clone, Debug)]
pub struct SingularMode {
    /// Singular value.
    pub value: f64,
    /// Left singular vector over the rows.
    pub left: Vec<f64>,
    /// Right singular vector over the columns.
    pub right: Vec<f64>,
}
/// Two numerically parallel rows, or two parallel columns, of one index space.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Parallel<I> {
    /// The earlier of the two.
    pub first: I,
    /// The later of the two.
    pub second: I,
    /// Absolute cosine of the angle between them, at most one.
    pub cosine: f64,
}
/// Numerical evidence at exactly one supplied point, never a structural or feasibility proof.
/// Rows are indices of the row space `R` and columns of the column space `C`.
#[derive(Clone, Debug)]
pub struct MatrixReport<R, C> {
    /// Euclidean norm of each scaled row, in row order.
    pub row_norms: Vec<f64>,
    /// Euclidean norm of each scaled column, in column order.
    pub column_norms: Vec<f64>,
    /// Row pairs within the parallel tolerance.
    pub parallel_rows: Vec<Parallel<R>>,
    /// Column pairs within the parallel tolerance.
    pub parallel_columns: Vec<Parallel<C>>,
    /// Singular values above `cutoff`.
    pub rank: usize,
    /// The rank cutoff: the larger of the absolute and the relative rank tolerance.
    pub cutoff: f64,
    /// Ascending singular values, including rectangular null-space modes.
    pub modes: Vec<SingularMode>,
}
/// Normalize once using frozen positive scales. Use library matrix products and full
/// SVD so the smallest modes, including rectangular null spaces, remain available.
pub fn analyze_matrix<R: From<usize>, C: From<usize>>(
    matrix: SparseColMatRef<'_, usize, f64>,
    row_scales: &[f64],
    column_scales: &[f64],
    policy: MatrixPolicy,
    cancel: &AtomicBool,
) -> Result<MatrixReport<R, C>, MathError> {
    if cancel.load(Ordering::Acquire) {
        return Err(MathError::Cancelled);
    }
    let (m, n) = (matrix.nrows(), matrix.ncols());
    policy.validate()?;
    if row_scales.len() != m
        || column_scales.len() != n
        || row_scales
            .iter()
            .chain(column_scales)
            .any(|s| !s.is_finite() || *s <= 0.0)
    {
        return Err(MathError::Contract(
            "diagnostic matrix shape or scales".into(),
        ));
    }
    let extent = m
        .checked_mul(m)
        .and_then(|v| n.checked_mul(n).and_then(|w| v.checked_add(w)))
        .and_then(|v| m.checked_mul(n).and_then(|w| v.checked_add(w)));
    if extent.is_none_or(|v| v > policy.dense_entries) {
        return Err(MathError::Limit("dense diagnostic matrix extent"));
    }
    if matrix.val().iter().any(|v| !v.is_finite()) {
        return Err(MathError::Contract("nonfinite diagnostic Jacobian".into()));
    }
    let mut scaled = Mat::<f64>::zeros(m, n);
    for j in 0..n {
        for (i, v) in matrix.row_idx_of_col(j).zip(matrix.val_of_col(j)) {
            scaled[(i, j)] = *v * row_scales[i] / column_scales[j];
        }
    }
    if scaled.col_iter().any(|c| c.iter().any(|v| !v.is_finite())) {
        return Err(MathError::Contract(
            "diagnostic normalization overflow".into(),
        ));
    }
    let row_norms = (0..m).map(|i| scaled.row(i).norm_l2()).collect::<Vec<_>>();
    let column_norms = (0..n).map(|j| scaled.col(j).norm_l2()).collect::<Vec<_>>();
    let rows = Mat::from_fn(m, n, |i, j| {
        if row_norms[i] > 0. {
            scaled[(i, j)] / row_norms[i]
        } else {
            0.
        }
    });
    let columns = Mat::from_fn(m, n, |i, j| {
        if column_norms[j] > 0. {
            scaled[(i, j)] / column_norms[j]
        } else {
            0.
        }
    });
    let row_gram = &rows * rows.transpose();
    let column_gram = columns.transpose() * &columns;
    let mut findings = 0usize;
    let mut pairs =
        |gram: &Mat<f64>, norms: &[f64]| -> Result<Vec<(usize, usize, f64)>, MathError> {
            let mut out = vec![];
            for j in 0..norms.len() {
                if cancel.load(Ordering::Acquire) {
                    return Err(MathError::Cancelled);
                }
                for i in 0..j {
                    if norms[i] > 0. && norms[j] > 0. {
                        let cosine = gram[(i, j)].abs().min(1.);
                        if 1. - cosine <= policy.parallel_tolerance {
                            findings = findings
                                .checked_add(1)
                                .ok_or(MathError::Limit("diagnostic finding extent"))?;
                            if findings > policy.findings {
                                return Err(MathError::Limit("diagnostic finding extent"));
                            }
                            out.push((i, j, cosine));
                        }
                    }
                }
            }
            Ok(out)
        };
    // faer's dense interior indexes by `usize`; the report names the spaces.
    let parallel_rows = pairs(&row_gram, &row_norms)?
        .into_iter()
        .map(|(i, j, cosine)| Parallel {
            first: R::from(i),
            second: R::from(j),
            cosine,
        })
        .collect();
    let parallel_columns = pairs(&column_gram, &column_norms)?
        .into_iter()
        .map(|(i, j, cosine)| Parallel {
            first: C::from(i),
            second: C::from(j),
            cosine,
        })
        .collect();
    let decomposition = scaled
        .svd()
        .map_err(|e| MathError::Library(format!("diagnostic SVD: {e:?}")))?;
    if cancel.load(Ordering::Acquire) {
        return Err(MathError::Cancelled);
    }
    let k = m.min(n);
    let maximum = (0..k).map(|i| decomposition.S()[i]).fold(0., f64::max);
    let cutoff = policy.rank_absolute.max(maximum * policy.rank_relative);
    let rank = (0..k).filter(|i| decomposition.S()[*i] > cutoff).count();
    let mut modes = (0..m.max(n))
        .map(|i| SingularMode {
            value: if i < k { decomposition.S()[i] } else { 0. },
            left: if i < m {
                (0..m).map(|j| decomposition.U()[(j, i)]).collect()
            } else {
                vec![]
            },
            right: if i < n {
                (0..n).map(|j| decomposition.V()[(j, i)]).collect()
            } else {
                vec![]
            },
        })
        .collect::<Vec<_>>();
    modes.sort_by(|a, b| a.value.total_cmp(&b.value));
    Ok(MatrixReport {
        row_norms,
        column_norms,
        parallel_rows,
        parallel_columns,
        rank,
        cutoff,
        modes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::index::{GlobalCol, GlobalRow};
    type Report = MatrixReport<GlobalRow, GlobalCol>;
    #[test]
    fn diagnostics_faer_smallest_modes_parallelism_and_bounded_rectangular_nullspaces() {
        let a = faer::sparse::SparseColMat::try_new_from_triplets(
            3,
            2,
            &[
                faer::sparse::Triplet::new(0, 0, 1.),
                faer::sparse::Triplet::new(0, 1, 2.),
                faer::sparse::Triplet::new(1, 0, 2.),
                faer::sparse::Triplet::new(1, 1, 4.),
                faer::sparse::Triplet::new(2, 0, -1.),
                faer::sparse::Triplet::new(2, 1, -2.),
            ],
        )
        .unwrap();
        let p = MatrixPolicy {
            dense_entries: 100,
            findings: 100,
            parallel_tolerance: 1e-8,
            rank_absolute: 1e-12,
            rank_relative: 1e-8,
            singular_vector: 0.1,
        };
        let flag = AtomicBool::new(false);
        let r: Report = analyze_matrix(a.as_ref(), &[1.; 3], &[1.; 2], p, &flag).unwrap();
        assert_eq!(r.rank, 1);
        assert_eq!(r.parallel_rows.len(), 3);
        assert_eq!(
            r.parallel_columns,
            [Parallel {
                first: GlobalCol::new(0),
                second: GlobalCol::new(1),
                cosine: 1.0
            }]
        );
        assert_eq!(r.modes.len(), 3);
        assert!(r.modes[0].value <= 1e-12);
        assert!(
            r.modes
                .iter()
                .any(|s| s.right.is_empty() && s.left.len() == 3)
        );
        for mode in r.modes.iter().filter(|s| s.value <= r.cutoff) {
            for j in 0..2 {
                let dot = (0..3)
                    .map(|i| [1., 2., -1.][i] * [1., 2.][j] * mode.left[i])
                    .sum::<f64>();
                assert!(dot.abs() < 1e-12);
            }
        }
        assert!(matches!(
            analyze_matrix::<GlobalRow, GlobalCol>(
                a.as_ref(),
                &[1.; 3],
                &[1.; 2],
                MatrixPolicy {
                    dense_entries: 2,
                    ..p
                },
                &flag
            ),
            Err(MathError::Limit(_))
        ));
        flag.store(true, Ordering::Release);
        assert!(matches!(
            analyze_matrix::<GlobalRow, GlobalCol>(a.as_ref(), &[1.; 3], &[1.; 2], p, &flag),
            Err(MathError::Cancelled)
        ));
    }
}

/// Bounded examination of original signed additive terms, before simplification.
#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TermPolicy {
    /// Magnitude at or below which a term is treated as zero and not examined.
    pub zero: f64,
    /// Ratio to the largest magnitude beyond which a smaller term is reported as mismatched.
    pub mismatch: f64,
    /// Normalized subset sum at or below which the subset is reported as cancelling.
    pub cancellation: f64,
    /// Largest subset size examined for cancellation, from 2 to 32.
    pub maximum_terms: usize,
    /// Maximum number of subsets examined.
    pub combinations: usize,
    /// Maximum number of cancelling subsets reported.
    pub findings: usize,
}
impl TermPolicy {
    /// Validate finite tolerances and bounded subset enumeration.
    pub fn validate(&self) -> Result<(), MathError> {
        if !self.zero.is_finite()
            || self.zero < 0.0
            || !self.mismatch.is_finite()
            || self.mismatch <= 1.0
            || !self.cancellation.is_finite()
            || !(0.0..1.0).contains(&self.cancellation)
            || !(2..=32).contains(&self.maximum_terms)
            || self.combinations == 0
            || self.findings == 0
        {
            return Err(MathError::Contract("term diagnostic policy".into()));
        }
        Ok(())
    }
}
/// Scale mismatches and cancelling subsets among the original terms.
#[derive(Clone, Debug)]
pub struct TermReport {
    /// Terms much smaller than the largest term, by index.
    pub mismatched: Vec<usize>,
    /// Minimal subsets of term indices whose normalized sum nearly cancels.
    pub cancellations: Vec<Vec<usize>>,
    /// Whether every subset within the policy was examined.
    pub complete: bool,
    /// Subsets examined.
    pub examined: usize,
}
/// Conservative construction peak for `analyze_terms`, including transient index
/// selection and all escaping report vectors. This bounds the enumeration algorithm
/// before construction rather than measuring a completed report.
/// # Errors
/// Invalid policy or an extent too large to represent.
pub fn term_allocation_bound(terms: &[f64], policy: TermPolicy) -> Result<usize, MathError> {
    policy.validate()?;
    let n = terms.len();
    let count = n.min(policy.maximum_terms);
    let cap = policy.findings.min(policy.combinations);
    // At most these subsets can become findings, irrespective of their numeric values.
    // u128 intermediates saturate to the finite finding allowance on enormous input.
    let mut choose = 1u128;
    let mut subsets = 0u128;
    for k in 1..=count {
        choose = choose
            .checked_mul((n - k + 1) as u128)
            .map(|value| value / k as u128)
            .unwrap_or(u128::MAX);
        if k >= 2 {
            subsets = subsets.saturating_add(choose);
        }
        if subsets >= cap as u128 {
            break;
        }
    }
    let findings = usize::try_from(subsets.min(cap as u128))
        .map_err(|_| MathError::Limit("term diagnostic finding extent"))?;
    // Filtered vectors and push-grown output vectors may double capacity. The
    // minimum four-element allocation is included even for tiny inputs/findings.
    let vectors = n
        .checked_mul(4)
        .and_then(|n| n.checked_add(8))
        .and_then(|n| n.checked_mul(size_of::<usize>()));
    let selections = findings
        .checked_add(1)
        .and_then(|n| n.checked_mul(count.max(4)))
        .and_then(|n| n.checked_mul(size_of::<usize>()));
    let output = findings
        .checked_mul(2)
        .and_then(|n| n.checked_add(4))
        .and_then(|n| n.checked_mul(size_of::<Vec<usize>>()));
    vectors
        .and_then(|n| n.checked_add(selections?))
        .and_then(|n| n.checked_add(output?))
        .and_then(|n| n.checked_add(count.checked_mul(size_of::<usize>())?))
        .and_then(|n| n.checked_add(size_of::<TermReport>() + 256))
        .ok_or(MathError::Limit("term diagnostic construction extent"))
}

/// Numerica supplies combination enumeration. All reductions use a common magnitude
/// to avoid overflowing an intermediate sum; an exhausted budget is explicitly incomplete.
pub fn analyze_terms(
    terms: &[f64],
    policy: TermPolicy,
    cancel: &AtomicBool,
) -> Result<TermReport, MathError> {
    policy.validate()?;
    if terms.iter().any(|v| !v.is_finite()) {
        return Err(MathError::Contract(
            "nonfinite term diagnostic input".into(),
        ));
    }
    if cancel.load(Ordering::Acquire) {
        return Err(MathError::Cancelled);
    }
    let indices = terms
        .iter()
        .enumerate()
        .filter_map(|(i, v)| (v.abs() > policy.zero).then_some(i))
        .collect::<Vec<_>>();
    let maximum = terms.iter().map(|v| v.abs()).fold(0., f64::max);
    let mismatched = indices
        .iter()
        .copied()
        .filter(|i| terms[*i].abs() < maximum / policy.mismatch)
        .collect::<Vec<_>>();
    let mut report = TermReport {
        mismatched,
        cancellations: vec![],
        complete: true,
        examined: 0,
    };
    if indices.len() < 2 {
        return Ok(report);
    }
    for count in 2..=indices.len().min(policy.maximum_terms) {
        let mut combinations =
            symbolica::combinatorics::CombinationIterator::new(indices.len(), count);
        while let Some(combination) = combinations.next() {
            if cancel.load(Ordering::Acquire) {
                return Err(MathError::Cancelled);
            }
            if report.examined >= policy.combinations
                || report.cancellations.len() >= policy.findings
            {
                report.complete = false;
                return Ok(report);
            }
            report.examined += 1;
            let selected = combination.iter().map(|j| indices[*j]).collect::<Vec<_>>();
            if report
                .cancellations
                .iter()
                .any(|c| c.iter().all(|i| selected.contains(i)))
            {
                continue;
            }
            let magnitude = selected.iter().map(|i| terms[*i].abs()).fold(0., f64::max);
            let sum = selected.iter().map(|i| terms[*i] / magnitude).sum::<f64>();
            if sum.abs() <= policy.cancellation {
                report.cancellations.push(selected);
            }
        }
    }
    Ok(report)
}
#[cfg(test)]
mod term_tests {
    use super::*;
    #[test]
    fn term_construction_bound_covers_growing_reports_without_using_capacity_ceiling() {
        let policy = TermPolicy {
            zero: 0.0,
            mismatch: 1e6,
            cancellation: 1e-8,
            maximum_terms: 8,
            combinations: 10_000,
            findings: 10_000,
        };
        for n in 0..=12 {
            let terms = (0..n)
                .map(|i| if i % 2 == 0 { 1.0 } else { -1.0 })
                .collect::<Vec<_>>();
            let report = analyze_terms(&terms, policy, &AtomicBool::new(false)).unwrap();
            let owned = report.mismatched.capacity() * size_of::<usize>()
                + report.cancellations.capacity() * size_of::<Vec<usize>>()
                + report
                    .cancellations
                    .iter()
                    .map(|v| v.capacity() * size_of::<usize>())
                    .sum::<usize>()
                + size_of::<TermReport>();
            assert!(term_allocation_bound(&terms, policy).unwrap() >= owned);
        }
        assert!(term_allocation_bound(&[1.0, -1.0], policy).unwrap() < 4096);
    }
    #[test]
    fn diagnostics_original_terms_bounded_combinations_report_incompleteness() {
        let p = TermPolicy {
            zero: 1e-12,
            mismatch: 1e6,
            cancellation: 1e-4,
            maximum_terms: 5,
            combinations: 100,
            findings: 10,
        };
        let flag = AtomicBool::new(false);
        let r = analyze_terms(&[1e4, -1e4, 1e-9], p, &flag).unwrap();
        assert_eq!(r.mismatched, vec![2]);
        assert_eq!(r.cancellations, vec![vec![0, 1]]);
        assert!(r.complete);
        let r = analyze_terms(
            &[1e308, 1e308, -1e308, -1e308],
            TermPolicy {
                combinations: 1,
                ..p
            },
            &flag,
        )
        .unwrap();
        assert!(!r.complete);
        assert_eq!(r.examined, 1);
    }
}
