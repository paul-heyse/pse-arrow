// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Sparse conditioning and certified inertia through FERAL (Plan 22 N4, L-N6).
//!
//! A square Jacobian's 1-norm condition number is estimated from a sparse LU; a KKT matrix
//! `[H Aᵀ; A 0]` is factored as LDLᵀ, whose pivot signs give its inertia (Sylvester's law)
//! and whose factor gives the 1-norm condition estimate. The inertia decides second-order
//! sufficiency at an NLP candidate: `In(K) = In(Zᵀ H Z) + (m, m, 0)` when the `m` active
//! gradients are independent, with `Z` a basis of their null space.
use crate::{
    NlpOracle, ProblemError,
    quality::{Observation, Tolerances},
    solve::{Candidate, Curvature, SecondOrder},
};
use faer::sparse::SparseColMatRef;
use pse_math::{
    index::{Entry, OriginalCol, OriginalRow, ReducedCol, ReducedRow, TiVec, Triplet},
    normalization::Normalization,
};

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
    // Serial and FMA-free: the diagnostic owns no admitted thread team.
    let mut solver = feral::Solver::new().with_parallel(false);
    match solver.factor(&matrix, None) {
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
    if inertia.total() != order {
        return Err(ProblemError::internal(
            "FERAL inertia does not cover the KKT order",
        ));
    }
    Ok(Kkt {
        inertia: (inertia.positive, inertia.negative, inertia.zero),
        condition_1norm: solver
            .estimate_condition_1norm(&matrix)
            .ok()
            .filter(|v| v.is_finite()),
    })
}

/// Every stored entry of a faer pattern, in storage order, in the spaces `R` and `C`.
fn entries<R: From<usize>, C: From<usize>>(
    pattern: faer::sparse::SymbolicSparseColMatRef<'_, usize>,
) -> Vec<Entry<R, C>> {
    (0..pattern.ncols())
        .flat_map(|j| {
            pattern
                .row_idx_of_col(j)
                .map(move |i| Entry::new(R::from(i), C::from(j)))
        })
        .collect()
}

fn native(error: feral::FeralError) -> ProblemError {
    ProblemError::numerical(format!("FERAL: {error}"))
}

/// Budgets of the post-solve second-order check.
#[derive(Clone, Copy, Debug)]
pub struct Check {
    /// Normalized multiplier magnitude at or below which an active constraint is weakly
    /// active: the dual stationarity budget, below which a multiplier is not
    /// distinguishable from zero.
    pub dual_budget: f64,
    /// Entry ceiling of the KKT matrix, the NLP run's dimension ceiling.
    pub limit: usize,
}

/// Record the second-order check of a qualified optimizing candidate on the report. A
/// candidate without valid multipliers, an infeasible one, or a failed check leaves the
/// evidence empty and the reason in the metrics.
pub(crate) fn attach_second_order(
    report: &mut crate::solve::SolveReport,
    oracle: &mut dyn NlpOracle,
    normalization: &Normalization,
    tolerances: &Tolerances,
    check: Check,
) {
    let (Some(candidate), Some(observation)) = (&report.candidate, &report.observation) else {
        return;
    };
    let outcome = if let Some(error) = &observation.dual_error {
        Err(ProblemError::unsupported(format!("multipliers: {error}")))
    } else if !report
        .quality
        .as_ref()
        .is_some_and(crate::quality::Quality::feasible)
    {
        Err(ProblemError::unsupported("the candidate is not feasible"))
    } else {
        crate::quality::contained(|| {
            second_order(
                oracle,
                candidate,
                observation,
                normalization,
                tolerances,
                check.dual_budget,
                check.limit,
            )
        })
    };
    match outcome {
        Ok(evidence) => report.evidence.second_order = Some(evidence),
        Err(error) => {
            report.metrics.insert(
                "second_order.unavailable".into(),
                crate::solve::Metric::Text(error.to_string()),
            );
        }
    }
}

/// A constraint of the local model: an active row, or an active bound on one variable.
enum Active {
    Row(OriginalRow),
    Bound(OriginalCol),
}

/// The second-order check at an NLP candidate, in normalized coordinates (PS-12). A
/// constraint is active within its physical tolerance and strongly active when its
/// normalized multiplier exceeds `dual_budget`. The reduced Hessian is tested first on the
/// null space of the strongly active constraints, which contains the critical cone
/// (sufficiency), then, when weakly active constraints exist, on the null space of all
/// active constraints, which the critical cone contains (necessity).
///
/// # Errors
/// Missing multipliers or Hessian, a failed evaluation, a KKT larger than `limit` entries,
/// or a failed factorization.
pub(crate) fn second_order(
    oracle: &mut dyn NlpOracle,
    candidate: &Candidate,
    observation: &Observation,
    normalization: &Normalization,
    tolerances: &Tolerances,
    dual_budget: f64,
    limit: usize,
) -> Result<SecondOrder, ProblemError> {
    let n = oracle.contract().variables.len();
    let m = oracle.contract().rows.len();
    let (Some(lambda), Some((zl, zu))) = (&candidate.row_dual, &candidate.bound_dual) else {
        return Err(ProblemError::unsupported("multipliers unavailable"));
    };
    if normalization.variables.len() != n
        || normalization.rows.len() != m
        || tolerances.variables.len() != n
        || tolerances.rows.len() != m
        || observation.values.len() != m
        || lambda.len() != m
        || zl.len() != n
        || zu.len() != n
    {
        return Err(ProblemError::internal("second-order check dimensions"));
    }
    let x = &candidate.primal;
    let (sx, sr, so) = (
        &normalization.variables,
        &normalization.rows,
        normalization.objective,
    );
    // Normalized Lagrangian Hessian (lower triangle) and constraint Jacobian, in the
    // oracle's original coordinates.
    let hessian: Vec<Triplet<OriginalCol, OriginalCol>> = {
        let pattern = oracle
            .hessian_pattern()
            .ok_or_else(|| ProblemError::unsupported("the profile has no exact Hessian"))?;
        let entries = entries::<OriginalCol, OriginalCol>(pattern);
        let mut values = vec![0.0; entries.len()];
        oracle.hessian(x, 1.0, lambda, &mut values)?;
        entries
            .into_iter()
            .zip(values)
            .map(|(e, v)| Triplet::new(e.row, e.col, sx[e.row.get()] * v * sx[e.col.get()] / so))
            .collect()
    };
    let jacobian: Vec<Triplet<OriginalRow, OriginalCol>> = {
        let pattern = oracle.jacobian_pattern();
        let entries = entries::<OriginalRow, OriginalCol>(pattern);
        let mut values = vec![0.0; entries.len()];
        oracle.jacobian(x, &mut values)?;
        entries
            .into_iter()
            .zip(values)
            .map(|(e, v)| Triplet::new(e.row, e.col, v * sx[e.col.get()] / sr[e.row.get()]))
            .collect()
    };
    // Activity and strength of every row and bound.
    let mut active = vec![];
    for (r, (&(l, u), &value)) in observation
        .bounds
        .iter()
        .zip(&observation.values)
        .enumerate()
    {
        let strong = l == u || (lambda[r] * sr[r] / so).abs() > dual_budget;
        if l == u || value - l <= tolerances.rows[r] || u - value <= tolerances.rows[r] {
            active.push((Active::Row(OriginalRow::new(r)), strong));
        }
    }
    for (j, v) in oracle.contract().variables.iter().enumerate() {
        let at_lower = v.lower.is_finite() && x[j] - v.lower <= tolerances.variables[j];
        let at_upper = v.upper.is_finite() && v.upper - x[j] <= tolerances.variables[j];
        if v.lower == v.upper || at_lower || at_upper {
            let multiplier = if at_upper { zu[j] } else { zl[j] };
            let strong = v.lower == v.upper || multiplier * sx[j] / so > dual_budget;
            active.push((Active::Bound(OriginalCol::new(j)), strong));
        }
    }
    let weakly_active = active.iter().filter(|(_, strong)| !strong).count();
    let test = |strong_only: bool| -> Result<SecondOrder, ProblemError> {
        let tested = active
            .iter()
            .filter(|(_, strong)| *strong || !strong_only)
            .map(|(a, _)| a)
            .collect::<Vec<_>>();
        // The KKT block is over the free columns and the tested active rows.
        let mut fixed: TiVec<OriginalCol, bool> = vec![false; n].into();
        let mut rows: TiVec<OriginalRow, Option<ReducedRow>> = vec![None; m].into();
        let mut count = 0;
        for a in &tested {
            match a {
                Active::Bound(j) => fixed[*j] = true,
                Active::Row(r) => {
                    rows[*r] = Some(ReducedRow::new(count));
                    count += 1;
                }
            }
        }
        let mut position: TiVec<OriginalCol, Option<ReducedCol>> = vec![None; n].into();
        let mut free = 0;
        for (j, fixed) in fixed.iter_enumerated() {
            if !fixed {
                position[j] = Some(ReducedCol::new(free));
                free += 1;
            }
        }
        let h = hessian
            .iter()
            .filter_map(|t| Some(Triplet::new(position[t.row]?, position[t.col]?, t.value)))
            .collect::<Vec<_>>();
        let a = jacobian
            .iter()
            .filter_map(|t| Some(Triplet::new(rows[t.row]?, position[t.col]?, t.value)))
            .collect::<Vec<_>>();
        if free + count + h.len() + a.len() > limit {
            return Err(ProblemError::unsupported(
                "the KKT matrix exceeds the diagnostic entry limit",
            ));
        }
        let factored = kkt(free, count, &h, &a)?;
        let (_, negative, zero) = factored.inertia;
        let curvature = if zero > 0 {
            Curvature::Singular
        } else if negative == count {
            if strong_only {
                Curvature::Sufficient
            } else {
                Curvature::Undecided
            }
        } else if !strong_only || weakly_active == 0 {
            Curvature::Negative
        } else {
            Curvature::Undecided
        };
        Ok(SecondOrder {
            free,
            active: count + (n - free),
            weakly_active,
            inertia: factored.inertia,
            condition_1norm: factored.condition_1norm,
            curvature,
        })
    };
    let sufficient = test(true)?;
    if sufficient.curvature != Curvature::Undecided || weakly_active == 0 {
        return Ok(sufficient);
    }
    // Weakly active constraints: negative curvature on the smaller subspace still refutes
    // a local minimizer; otherwise the verdict stays open.
    let necessary = test(false)?;
    Ok(if necessary.curvature == Curvature::Negative {
        necessary
    } else {
        sufficient
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use faer::{
        linalg::solvers::DenseSolveCore,
        sparse::{SparseColMat, Triplet},
    };

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

    /// `f = ½ xᵀ diag(q) x` over two variables with bounds, and one row `x₀ + x₁`.
    #[derive(Debug)]
    struct Quadratic {
        contract: crate::OracleContract,
        q: [f64; 2],
        rows: Vec<(f64, f64)>,
        jacobian: SparseColMat<usize, f64>,
        hessian: SparseColMat<usize, f64>,
    }
    impl Quadratic {
        fn new(q: [f64; 2], bounds: [(f64, f64); 2]) -> Self {
            Self {
                contract: crate::OracleContract {
                    identity: pse_ids::ContentHash::from_bytes([8; 32]),
                    variables: bounds
                        .iter()
                        .enumerate()
                        .map(|(i, &(lower, upper))| crate::Variable {
                            id: pse_ids::SemanticId::from_bytes([40 + i as u8; 16]),
                            lower,
                            upper,
                        })
                        .collect(),
                    rows: vec![pse_ids::SemanticId::from_bytes([50; 16])],
                    derivatives: pse_kernels::DerivativeOrder::Second,
                    smoothness: pse_kernels::DerivativeOrder::Second,
                },
                q,
                rows: vec![(-10.0, 10.0)],
                jacobian: SparseColMat::try_new_from_triplets(
                    1,
                    2,
                    &[Triplet::new(0, 0, 1.0), Triplet::new(0, 1, 1.0)],
                )
                .unwrap(),
                hessian: SparseColMat::try_new_from_triplets(
                    2,
                    2,
                    &[Triplet::new(0, 0, 1.0), Triplet::new(1, 1, 1.0)],
                )
                .unwrap(),
            }
        }
    }
    impl NlpOracle for Quadratic {
        fn contract(&self) -> &crate::OracleContract {
            &self.contract
        }
        fn jacobian_pattern(&self) -> faer::sparse::SymbolicSparseColMatRef<'_, usize> {
            self.jacobian.symbolic()
        }
        fn hessian_pattern(&self) -> Option<faer::sparse::SymbolicSparseColMatRef<'_, usize>> {
            Some(self.hessian.symbolic())
        }
        fn constraint_bounds(&self) -> &[(f64, f64)] {
            &self.rows
        }
        fn objective(&mut self, x: &[f64]) -> Result<f64, ProblemError> {
            Ok(0.5 * (self.q[0] * x[0] * x[0] + self.q[1] * x[1] * x[1]))
        }
        fn constraints(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
            out[0] = x[0] + x[1];
            Ok(())
        }
        fn gradient(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
            out[0] = self.q[0] * x[0];
            out[1] = self.q[1] * x[1];
            Ok(())
        }
        fn jacobian(&mut self, _: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
            out.fill(1.0);
            Ok(())
        }
        fn hessian(
            &mut self,
            _: &[f64],
            weight: f64,
            _: &[f64],
            out: &mut [f64],
        ) -> Result<(), ProblemError> {
            out[0] = weight * self.q[0];
            out[1] = weight * self.q[1];
            Ok(())
        }
    }
    /// The check at the stationary point `x = 0` with zero multipliers.
    fn at_origin(q: [f64; 2], bounds: [(f64, f64); 2]) -> SecondOrder {
        let mut oracle = Quadratic::new(q, bounds);
        let candidate = Candidate {
            kind: crate::solve::CandidateKind::FinalIterate,
            primal: vec![0.0; 2],
            objective: Some(0.0),
            row_dual: Some(vec![0.0]),
            bound_dual: Some((vec![0.0; 2], vec![0.0; 2])),
            reduced_costs: None,
            slacks: None,
        };
        let observation =
            Observation::from_values(Some(0.0), vec![0.0], vec![(-10.0, 10.0)]).unwrap();
        second_order(
            &mut oracle,
            &candidate,
            &observation,
            &Normalization::identity(2, 1),
            &Tolerances {
                variables: vec![1e-8; 2],
                rows: vec![1e-8],
                integrality: 1e-8,
            },
            1e-9,
            1000,
        )
        .unwrap()
    }

    #[test]
    fn second_order_verdicts_follow_the_inertia() {
        let free = [(-1.0, 1.0); 2];
        // A saddle: stationary, not a minimizer.
        let saddle = at_origin([2.0, -2.0], free);
        assert_eq!(saddle.curvature, Curvature::Negative);
        assert_eq!(
            (saddle.free, saddle.active, saddle.inertia),
            (2, 0, (1, 1, 0))
        );
        // A flat direction: the reduced Hessian is singular.
        assert_eq!(at_origin([2.0, 0.0], free).curvature, Curvature::Singular);
        // A weakly active bound (zero multiplier) under positive curvature: the test on the
        // larger subspace still certifies sufficiency.
        let weak = at_origin([2.0, 2.0], [(-1.0, 1.0), (0.0, 1.0)]);
        assert_eq!(weak.curvature, Curvature::Sufficient);
        assert_eq!(weak.weakly_active, 1);
        // A weakly active bound with negative curvature along it: neither test decides.
        let open = at_origin([2.0, -2.0], [(-1.0, 1.0), (0.0, 1.0)]);
        assert_eq!(open.curvature, Curvature::Undecided);
        // A fixed variable is active and removed from the reduced space.
        let fixed = at_origin([2.0, -2.0], [(-1.0, 1.0), (0.0, 0.0)]);
        assert_eq!(fixed.curvature, Curvature::Sufficient);
        assert_eq!((fixed.free, fixed.active), (1, 1));
    }

    #[cfg(all(feature = "ipopt", feature = "pounce"))]
    #[test]
    fn kkt_inertia_certifies_second_order() {
        use crate::{
            execution::BackendSettings,
            presolve::Policy,
            restart_tests::{Simplex, TARGET, run},
            solve::Backend,
        };
        for (backend, settings) in [
            (Backend::Ipopt, BackendSettings::Ipopt(Default::default())),
            (Backend::Pounce, BackendSettings::Pounce(Default::default())),
        ] {
            // The projection onto the simplex keeps three coordinates positive: the simplex
            // row and three lower bounds are active with positive multipliers, and the
            // reduced Hessian 2·I is positive definite, so In(K) = (3, 1, 0).
            let report = run(
                backend,
                &settings,
                Simplex::new(&TARGET, 1.0, 4.0),
                None,
                &Policy::Auto,
            );
            let evidence = report.evidence.second_order.unwrap_or_else(|| {
                panic!(
                    "{backend:?}: {:?}",
                    report.metrics.get("second_order.unavailable")
                )
            });
            assert_eq!(evidence.curvature, Curvature::Sufficient, "{backend:?}");
            assert_eq!(
                (evidence.free, evidence.active, evidence.weakly_active),
                (3, 4, 0),
                "{backend:?}"
            );
            assert_eq!(evidence.inertia, (3, 1, 0), "{backend:?}");
            assert!(evidence.condition_1norm.is_some_and(|c| c >= 1.0));
        }
    }
}
