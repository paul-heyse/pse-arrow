// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Bounded numerical evidence; decomposition and matrix products belong to faer.
use crate::MathError;
use faer::{Mat, sparse::SparseColMatRef};
use std::sync::atomic::{AtomicBool, Ordering};

/// Explicit diagnostic work and numerical classification limits.
#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MatrixPolicy {
    /// Includes dense input and both complete singular-vector factors/Gram products.
    pub dense_entries: usize,
    pub findings: usize,
    pub parallel_tolerance: f64,
    pub rank_absolute: f64,
    pub rank_relative: f64,
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
    pub value: f64,
    pub left: Vec<f64>,
    pub right: Vec<f64>,
}
/// Numerical evidence at exactly one supplied point, never a structural or feasibility proof.
#[derive(Clone, Debug)]
pub struct MatrixReport {
    pub row_norms: Vec<f64>,
    pub column_norms: Vec<f64>,
    pub parallel_rows: Vec<(usize, usize, f64)>,
    pub parallel_columns: Vec<(usize, usize, f64)>,
    pub rank: usize,
    pub cutoff: f64,
    /// Ascending singular values, including rectangular null-space modes.
    pub modes: Vec<SingularMode>,
}
/// Normalize once using frozen positive scales. Use library matrix products and full
/// SVD so the smallest modes, including rectangular null spaces, remain available.
pub fn analyze_matrix(
    matrix: SparseColMatRef<'_, usize, f64>,
    row_scales: &[f64],
    column_scales: &[f64],
    policy: MatrixPolicy,
    cancel: &AtomicBool,
) -> Result<MatrixReport, MathError> {
    if cancel.load(Ordering::Acquire) {
        return Err(MathError::Cancelled);
    }
    let (m, n) = (matrix.nrows(), matrix.ncols());
    policy.validate()?;
    if row_scales.len() != m || column_scales.len() != n
        || row_scales.iter().chain(column_scales).any(|s| !s.is_finite() || *s <= 0.0) {
        return Err(MathError::Contract("diagnostic matrix shape or scales".into()));
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
    let parallel_rows = pairs(&row_gram, &row_norms)?;
    let parallel_columns = pairs(&column_gram, &column_norms)?;
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
        };
        let flag = AtomicBool::new(false);
        let r = analyze_matrix(a.as_ref(), &[1.; 3], &[1.; 2], p, &flag).unwrap();
        assert_eq!(r.rank, 1);
        assert_eq!(r.parallel_rows.len(), 3);
        assert_eq!(r.parallel_columns.len(), 1);
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
            analyze_matrix(
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
            analyze_matrix(a.as_ref(), &[1.; 3], &[1.; 2], p, &flag),
            Err(MathError::Cancelled)
        ));
    }
}

/// Bounded examination of original signed additive terms, before simplification.
#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TermPolicy {
    pub zero: f64,
    pub mismatch: f64,
    pub cancellation: f64,
    pub maximum_terms: usize,
    pub combinations: usize,
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
#[derive(Clone, Debug)]
pub struct TermReport {
    pub mismatched: Vec<usize>,
    pub cancellations: Vec<Vec<usize>>,
    pub complete: bool,
    pub examined: usize,
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
        return Err(MathError::Contract("nonfinite term diagnostic input".into()));
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
