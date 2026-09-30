// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Bounded LP/MILP analyses of a supplied Jacobian. These say nothing about nonlinear feasibility.
use crate::{
    CoefficientProblem, OracleContract, ProblemError, Variable, highs, quality::Tolerances,
    solve::*,
};
use faer::sparse::{SparseColMat, SparseColMatRef, Triplet};
use pse_ids::{FramedHasher, SemanticId};
use pse_kernels::DerivativeOrder;
use pse_math::binding::ObjectiveSense;
use pse_model::generated::enums::ModelingVariableDomain;
use std::collections::BTreeSet;

/// Finite search over anchored left-null vectors.
#[derive(Clone, Copy, Debug)]
pub struct Policy {
    /// Row ceiling of the analysed Jacobian.
    pub maximum_rows: usize,
    /// Entry ceiling of every diagnostic problem.
    pub maximum_entries: usize,
    /// Native solve budget across all anchors and both problem families.
    pub maximum_attempts: usize,
    /// Bound on every non-anchor multiplier.
    pub multiplier_bound: f64,
    /// Residual and support tolerance.
    pub tolerance: f64,
    /// Relative singular-value cutoff of the rank checks.
    pub rank_relative: f64,
    /// Branch-and-bound node budget of each minimum-support MILP, separate from the
    /// iteration budget (F10); `None` bounds them by the deadline alone.
    pub maximum_nodes: Option<u32>,
}
/// A verified approximate left-null vector; residuals use the supplied matrix coordinates.
#[derive(Clone, Debug)]
pub struct Certificate {
    /// Row multipliers.
    pub weights: Vec<(SemanticId, f64)>,
    /// Largest absolute entry of the weighted row combination.
    pub residual_maximum: f64,
    /// The row whose multiplier is anchored at one.
    pub pivot: SemanticId,
}
/// A minimum-support candidate qualified by removing each member and checking numerical rank.
#[derive(Clone, Debug)]
pub struct DegenerateSet {
    /// Rows in the support.
    pub rows: Vec<SemanticId>,
    /// The left-null vector that found them.
    pub certificate: Certificate,
    /// Dependent, and every proper subset independent, at the rank tolerance.
    pub irreducible_at_tolerance: bool,
}
/// Everything the bounded analysis found and every native attempt it made.
#[derive(Debug)]
pub struct Report {
    /// Anchored conditioning certificates (LP family).
    pub conditioning: Vec<Certificate>,
    /// Minimum-support degenerate sets (MILP family).
    pub degenerate: Vec<DegenerateSet>,
    /// Every native attempt, LP family first.
    pub attempts: Vec<SolveReport>,
    /// Native HiGHS sessions created: one per problem family, reused across anchors.
    pub sessions: usize,
    /// Every anchor of both families ran to a verified verdict.
    pub complete: bool,
    /// Why parts of the analysis are missing.
    pub unavailable: Vec<String>,
}
struct Builder {
    id: SemanticId,
    variables: Vec<Variable>,
    domains: Vec<ModelingVariableDomain>,
    objective: Vec<f64>,
    rows: Vec<SemanticId>,
    bounds: Vec<(f64, f64)>,
    entries: Vec<Triplet<usize, usize, f64>>,
}
impl Builder {
    fn variable(
        &mut self,
        name: &str,
        lower: f64,
        upper: f64,
        domain: ModelingVariableDomain,
        cost: f64,
    ) -> usize {
        let i = self.variables.len();
        self.variables.push(Variable {
            id: pse_ids::named_id(self.id, name),
            lower,
            upper,
        });
        self.domains.push(domain);
        self.objective.push(cost);
        i
    }
    fn row(&mut self, terms: impl IntoIterator<Item = (usize, f64)>, lower: f64, upper: f64) {
        let row = self.rows.len();
        self.rows
            .push(pse_ids::named_id(self.id, &format!("row-{row}")));
        self.bounds.push((lower, upper));
        self.entries.extend(
            terms
                .into_iter()
                .filter(|(_, v)| *v != 0.)
                .map(|(col, value)| Triplet::new(row, col, value)),
        );
    }
    fn finish(self) -> Result<CoefficientProblem, ProblemError> {
        let constraints = SparseColMat::try_new_from_triplets(
            self.rows.len(),
            self.variables.len(),
            &self.entries,
        )
        .map_err(|e| ProblemError::Internal(format!("diagnostic matrix: {e}")))?;
        let mut h = FramedHasher::new(pse_ids::Frame::JacobianDiagnosticProblemV1);
        h.id(&self.id);
        h.u64(self.variables.len() as u64)
            .u64(self.rows.len() as u64);
        for (v, domain) in self.variables.iter().zip(&self.domains) {
            h.id(&v.id)
                .u64(v.lower.to_bits())
                .u64(v.upper.to_bits())
                .u64(*domain as u64);
        }
        for v in &self.objective {
            h.u64(v.to_bits());
        }
        for (lo, hi) in &self.bounds {
            h.u64(lo.to_bits()).u64(hi.to_bits());
        }
        for v in constraints.col_ptr() {
            h.u64(*v as u64);
        }
        for v in constraints.row_idx() {
            h.u64(*v as u64);
        }
        for v in constraints.val() {
            h.u64(v.to_bits());
        }
        let identity = h.finish_hash();
        let problem = CoefficientProblem {
            contract: OracleContract {
                identity,
                variables: self.variables,
                rows: self.rows,
                derivatives: DerivativeOrder::Value,
                smoothness: DerivativeOrder::Second,
            },
            objective: self.objective,
            objective_constant: 0.,
            sense: ObjectiveSense::Minimize,
            domains: self.domains,
            assumptions: identity,
            constraints,
            hessian: None,
            bounds: self.bounds,
            objectives: Vec::new(),
        };
        problem.validate()?;
        Ok(problem)
    }
}
fn problem(
    matrix: SparseColMatRef<'_, usize, f64>,
    rows: &[SemanticId],
    pivot: usize,
    milp: bool,
    policy: Policy,
) -> Result<CoefficientProblem, ProblemError> {
    // One identity per problem family: the anchor changes only multiplier bounds, so every
    // anchor of a family has the same layout and reuses one native session (L-C4).
    let id = pse_ids::named_id(rows[0], if milp { "degeneracy" } else { "conditioning" });
    let mut b = Builder {
        id,
        variables: vec![],
        domains: vec![],
        objective: vec![],
        rows: vec![],
        bounds: vec![],
        entries: vec![],
    };
    let bound = if milp { policy.multiplier_bound } else { 1. };
    for (i, id) in rows.iter().enumerate() {
        b.variable(
            &format!("multiplier-{id}"),
            if i == pivot { 1. } else { -bound },
            if i == pivot { 1. } else { bound },
            ModelingVariableDomain::Continuous,
            0.,
        );
    }
    if milp {
        for id in rows {
            b.variable(
                &format!("selected-{id}"),
                0.,
                1.,
                ModelingVariableDomain::Binary,
                1.,
            );
        }
        for i in 0..rows.len() {
            b.row([(i, 1.), (rows.len() + i, -bound)], f64::NEG_INFINITY, 0.);
            b.row([(i, -1.), (rows.len() + i, -bound)], f64::NEG_INFINITY, 0.);
        }
        for col in 0..matrix.ncols() {
            b.row(
                matrix
                    .row_idx_of_col(col)
                    .zip(matrix.val_of_col(col))
                    .map(|(r, v)| (r, *v)),
                0.,
                0.,
            );
        }
    } else {
        let residual = b.variable(
            "residual-infinity-norm",
            0.,
            f64::INFINITY,
            ModelingVariableDomain::Continuous,
            1.,
        );
        for col in 0..matrix.ncols() {
            for sign in [-1., 1.] {
                b.row(
                    matrix
                        .row_idx_of_col(col)
                        .zip(matrix.val_of_col(col))
                        .map(|(r, v)| (r, sign * v))
                        .chain([(residual, -1.)]),
                    f64::NEG_INFINITY,
                    0.,
                );
            }
        }
    }
    b.finish()
}
fn verify(
    matrix: SparseColMatRef<'_, usize, f64>,
    rows: &[SemanticId],
    pivot: usize,
    point: &[f64],
    policy: Policy,
) -> Option<Certificate> {
    let weights = point.get(..rows.len())?;
    if weights.iter().any(|v| !v.is_finite()) || (weights[pivot] - 1.).abs() > policy.tolerance {
        return None;
    }
    let residual = (0..matrix.ncols())
        .map(|j| {
            matrix
                .row_idx_of_col(j)
                .zip(matrix.val_of_col(j))
                .map(|(i, v)| v * weights[i])
                .sum::<f64>()
                .abs()
        })
        .fold(0., f64::max);
    if !residual.is_finite() {
        return None;
    }
    Some(Certificate {
        weights: rows.iter().copied().zip(weights.iter().copied()).collect(),
        residual_maximum: residual,
        pivot: rows[pivot],
    })
}
fn rank(
    matrix: SparseColMatRef<'_, usize, f64>,
    selected: &[usize],
    policy: Policy,
    execution: &Execution,
) -> Result<usize, ProblemError> {
    if selected.is_empty() {
        return Ok(0);
    }
    let mut entries = vec![];
    for j in 0..matrix.ncols() {
        for (i, v) in matrix.row_idx_of_col(j).zip(matrix.val_of_col(j)) {
            if let Some(k) = selected.iter().position(|s| *s == i) {
                entries.push(Triplet::new(k, j, *v));
            }
        }
    }
    let sub = SparseColMat::try_new_from_triplets(selected.len(), matrix.ncols(), &entries)
        .map_err(|e| ProblemError::Internal(e.to_string()))?;
    // Rows are the selected subset; only the rank is read.
    Ok(pse_math::diagnostics::analyze_matrix::<
        pse_math::index::ReducedRow,
        pse_math::index::OriginalCol,
    >(
        sub.as_ref(),
        &vec![1.; selected.len()],
        &vec![1.; matrix.ncols()],
        pse_math::diagnostics::MatrixPolicy {
            dense_entries: policy.maximum_entries,
            findings: policy.maximum_entries,
            parallel_tolerance: 0.,
            rank_absolute: policy.tolerance,
            rank_relative: policy.rank_relative,
            // Only the rank is read; no mode names members.
            singular_vector: 0.,
        },
        &execution.cancel,
    )?
    .rank)
}
/// Structural identity of a diagnostic problem family: its pattern and domains, not the
/// anchor's bounds.
fn layout(p: &CoefficientProblem) -> pse_ids::ContentHash {
    let mut h = FramedHasher::new(pse_ids::Frame::JacobianDiagnosticLayoutV1);
    h.u64(p.contract.variables.len() as u64)
        .u64(p.contract.rows.len() as u64);
    for v in &p.contract.variables {
        h.id(&v.id);
    }
    for d in &p.domains {
        h.u64(*d as u64);
    }
    for v in p.constraints.col_ptr() {
        h.u64(*v as u64);
    }
    for v in p.constraints.row_idx() {
        h.u64(*v as u64);
    }
    h.finish_hash()
}
/// Run native LP conditioning certificates and anchored minimum-support MILPs under
/// one outer execution deadline. Each problem family owns one native session, updated from
/// anchor to anchor (L-C4). Native limits and rank-budget refusals remain inconclusive.
pub fn analyze(
    matrix: SparseColMatRef<'_, usize, f64>,
    rows: &[SemanticId],
    policy: Policy,
    controls: &Controls,
    execution: Execution,
) -> Result<Report, ProblemError> {
    if rows.len() != matrix.nrows()
        || rows.is_empty()
        || rows.iter().collect::<BTreeSet<_>>().len() != rows.len()
        || rows.len() > policy.maximum_rows
        || matrix
            .val()
            .len()
            .checked_mul(2)
            .and_then(|n| rows.len().checked_mul(4).and_then(|m| n.checked_add(m)))
            .is_none_or(|n| n > policy.maximum_entries)
        || policy.maximum_attempts == 0
        || !policy.multiplier_bound.is_finite()
        || policy.multiplier_bound < 1.
        || !policy.tolerance.is_finite()
        || policy.tolerance <= 0.
        || !policy.rank_relative.is_finite()
        || policy.rank_relative < 0.
        || policy.rank_relative >= 1.
        || matrix.val().iter().any(|v| !v.is_finite())
        || policy.maximum_nodes == Some(0)
    {
        return Err(ProblemError::Contract(
            "invalid bounded Jacobian diagnostic request".into(),
        ));
    }
    let mut report = Report {
        conditioning: vec![],
        degenerate: vec![],
        attempts: vec![],
        sessions: 0,
        complete: true,
        unavailable: vec![],
    };
    // The diagnostic LP/MILPs are this analysis's own problems: their budgets derive from
    // its tolerance, not from any model's numerical policy.
    let accuracy = ResolvedAccuracy::from_policy(&Default::default(), policy.tolerance)?;
    let settings = highs::Settings {
        nodes: policy.maximum_nodes,
        ..highs::Settings::default()
    };
    for milp in [false, true] {
        // One live HiGHS session per thread: the family's session ends before the next.
        let mut current: Option<highs::Session> = None;
        for pivot in 0..rows.len() {
            if report.attempts.len() >= policy.maximum_attempts || execution.stopped().is_some() {
                report.complete = false;
                report
                    .unavailable
                    .push("diagnostic attempt budget or deadline exhausted".into());
                return Ok(report);
            }
            let p = problem(matrix, rows, pivot, milp, policy)?;
            let identity = layout(&p);
            let stamp = Compatibility {
                layout: identity,
                profile: identity,
                data: p.assumptions,
                backend: Backend::Highs,
            };
            let mut session = match current.take() {
                Some(mut s) => {
                    s.update(&p, None, stamp)?;
                    s
                }
                None => {
                    report.sessions += 1;
                    highs::Session::new(&p, None, stamp)?
                }
            };
            let t = Tolerances {
                variables: vec![policy.tolerance; p.contract.variables.len()],
                rows: vec![policy.tolerance; p.contract.rows.len()],
                integrality: policy.tolerance,
            };
            let outcome = session.solve(
                &p,
                &pse_math::normalization::Normalization::identity(
                    p.contract.variables.len(),
                    p.contract.rows.len(),
                ),
                controls,
                &accuracy,
                &settings,
                execution.clone(),
                &t,
                None,
            )?;
            current = Some(session);
            let optimal = outcome.termination.category == Termination::Success
                && outcome
                    .quality
                    .as_ref()
                    .is_some_and(crate::quality::Quality::feasible);
            if optimal && let Some(point) = outcome.candidate.as_ref().map(|c| &c.primal) {
                if let Some(certificate) = verify(matrix, rows, pivot, point, policy) {
                    if !milp {
                        report.conditioning.push(certificate);
                    } else if certificate.residual_maximum <= policy.tolerance {
                        let selected = point[..rows.len()]
                            .iter()
                            .enumerate()
                            .filter_map(|(i, v)| (v.abs() > policy.tolerance).then_some(i))
                            .collect::<Vec<_>>();
                        let sources = selected.iter().map(|i| rows[*i]).collect::<Vec<_>>();
                        if !report.degenerate.iter().any(|s| s.rows == sources) {
                            let irreducible = (|| -> Result<bool, ProblemError> {
                                if rank(matrix, &selected, policy, &execution)? + 1
                                    != selected.len()
                                {
                                    return Ok(false);
                                }
                                for omit in &selected {
                                    let subset = selected
                                        .iter()
                                        .copied()
                                        .filter(|i| i != omit)
                                        .collect::<Vec<_>>();
                                    if rank(matrix, &subset, policy, &execution)? != subset.len() {
                                        return Ok(false);
                                    }
                                }
                                Ok(true)
                            })();
                            let irreducible_at_tolerance = match irreducible {
                                Ok(v) => v,
                                Err(error) => {
                                    report.complete = false;
                                    report.unavailable.push(error.to_string());
                                    false
                                }
                            };
                            report.degenerate.push(DegenerateSet {
                                rows: sources,
                                certificate,
                                irreducible_at_tolerance,
                            });
                        }
                    }
                } else {
                    report.complete = false;
                }
            } else if outcome.termination.category != Termination::Infeasible {
                report.complete = false;
            }
            report.attempts.push(outcome);
        }
    }
    Ok(report)
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, atomic::AtomicBool};
    #[test]
    fn diagnostic_highs_lp_milp_find_and_verify_a_small_degenerate_set() {
        let matrix = SparseColMat::try_new_from_triplets(
            2,
            2,
            &[
                Triplet::new(0, 0, 1.),
                Triplet::new(0, 1, 1.),
                Triplet::new(1, 0, 2.),
                Triplet::new(1, 1, 2.),
            ],
        )
        .unwrap();
        let rows = [
            SemanticId::from_bytes([1; 16]),
            SemanticId::from_bytes([2; 16]),
        ];
        let controls = Controls::default();
        let policy = Policy {
            maximum_rows: 10,
            maximum_entries: 1000,
            maximum_attempts: 4,
            multiplier_bound: 10.,
            tolerance: 1e-7,
            rank_relative: 1e-8,
            maximum_nodes: None,
        };
        let r = analyze(
            matrix.as_ref(),
            &rows,
            policy,
            &controls,
            Execution::new(Arc::new(AtomicBool::new(false)), &controls),
        )
        .unwrap();
        assert!(r.complete);
        assert_eq!(r.attempts.len(), 4);
        assert_eq!(r.degenerate.len(), 1);
        assert_eq!(r.degenerate[0].rows, rows);
        assert!(r.degenerate[0].irreducible_at_tolerance);
        assert_eq!(r.conditioning.len(), 2);
        assert!(
            r.conditioning
                .iter()
                .any(|c| c.residual_maximum <= policy.tolerance)
        );
        assert!(
            r.conditioning
                .iter()
                .any(|c| c.residual_maximum > policy.tolerance)
        );
    }
    #[test]
    fn degeneracy_hunter_reuses_session() {
        // Rows a and b are parallel; c is independent.
        let matrix = SparseColMat::try_new_from_triplets(
            3,
            3,
            &[
                Triplet::new(0, 0, 1.),
                Triplet::new(0, 1, 1.),
                Triplet::new(1, 0, 2.),
                Triplet::new(1, 1, 2.),
                Triplet::new(2, 2, 1.),
            ],
        )
        .unwrap();
        let rows = [
            SemanticId::from_bytes([1; 16]),
            SemanticId::from_bytes([2; 16]),
            SemanticId::from_bytes([3; 16]),
        ];
        let controls = Controls::default();
        let policy = Policy {
            maximum_rows: 10,
            maximum_entries: 1000,
            maximum_attempts: 6,
            multiplier_bound: 10.,
            tolerance: 1e-7,
            rank_relative: 1e-8,
            maximum_nodes: Some(1000),
        };
        let r = analyze(
            matrix.as_ref(),
            &rows,
            policy,
            &controls,
            Execution::new(Arc::new(AtomicBool::new(false)), &controls),
        )
        .unwrap();
        assert!(r.complete, "{:?}", r.unavailable);
        // One native session per problem family, updated from anchor to anchor.
        assert_eq!(r.sessions, 2);
        assert_eq!(r.attempts.len(), 6);
        let reused = r
            .attempts
            .iter()
            .map(|a| a.evidence.reused_native_state)
            .collect::<Vec<_>>();
        assert_eq!(reused, [false, true, true, false, true, true]);
        // The node budget reaches the MILPs and the iteration budget stays separate.
        assert!(
            r.attempts
                .iter()
                .all(|a| a.options["mip_max_nodes"] == OptionValue::Integer(1000)
                    && a.options["simplex_iteration_limit"] == OptionValue::Integer(3000))
        );
        assert_eq!(r.degenerate.len(), 1);
        assert_eq!(r.degenerate[0].rows, rows[..2]);
        assert!(r.degenerate[0].irreducible_at_tolerance);
    }
}
