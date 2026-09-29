// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Coefficient problems in cone form (Plan 22 I10). The coefficient runner lowers a linear
//! or convex quadratic program here when the selected adapter consumes cones, raises the
//! adapter's report back to the coefficient rows, and verifies certificates against the
//! same layout of the original data.
use super::{Cone, SparseMatrix};
use crate::{
    CoefficientProblem, ConicProblem, OracleContract, ProblemError,
    quality::Tolerances,
    solve::{CertificateKind, RayCoordinate, SolveReport},
};
use pse_math::convexity::{QuadraticEvidence, TransportedEvidence};
use pse_model::generated::enums::ModelingVariableDomain;

/// The side of a coefficient row `L <= a·x <= U` a cone row carries.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RowSide {
    /// `a·x + s = L = U` with `s` in the zero cone.
    Equal,
    /// `a·x + s = U` with `s >= 0`.
    Upper,
    /// `-a·x + s = -L` with `s >= 0`.
    Lower,
}
impl RowSide {
    /// The ray coordinate of a cone row carrying this side.
    pub const fn coordinate(self) -> RayCoordinate {
        match self {
            Self::Equal => RayCoordinate::Row,
            Self::Upper => RayCoordinate::RowUpper,
            Self::Lower => RayCoordinate::RowLower,
        }
    }
    /// The factor of the row's coefficients in the cone row.
    pub(crate) const fn sign(self) -> f64 {
        match self {
            Self::Lower => -1.0,
            Self::Equal | Self::Upper => 1.0,
        }
    }
}
/// One cone row of a lowered coefficient row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LoweredRow {
    /// The coefficient row.
    pub row: usize,
    /// The side the cone row carries.
    pub side: RowSide,
}
/// The cone rows of coefficient rows with these bounds: every equality first, as one zero
/// cone, then each finite side of the other rows (upper before lower) as one nonnegative
/// cone. A free row has no cone row.
pub(crate) fn layout(bounds: &[(f64, f64)]) -> Vec<LoweredRow> {
    let mut rows: Vec<_> = bounds
        .iter()
        .enumerate()
        .filter(|(_, (l, u))| l == u)
        .map(|(row, _)| LoweredRow {
            row,
            side: RowSide::Equal,
        })
        .collect();
    for (row, (l, u)) in bounds.iter().enumerate().filter(|(_, (l, u))| l != u) {
        if u.is_finite() {
            rows.push(LoweredRow {
                row,
                side: RowSide::Upper,
            });
        }
        if l.is_finite() {
            rows.push(LoweredRow {
                row,
                side: RowSide::Lower,
            });
        }
    }
    rows
}
/// A coefficient problem in cone form, with the map back to its rows.
#[derive(Debug)]
pub struct Lowered {
    /// The cone form: `min ½xᵀPx + qᵀx` in the authored sense times `sign`.
    pub problem: ConicProblem,
    /// Evidence for exactly the cone form's quadratic.
    pub evidence: TransportedEvidence,
    /// The coefficient row and side of each cone row.
    pub rows: Vec<LoweredRow>,
    /// Authored orientation: one to minimize, minus one to maximize.
    pub sign: f64,
}
impl ConicProblem {
    /// Lower a continuous linear or convex quadratic coefficient program: each equality
    /// row to a zero-cone row and each finite side of the other rows to a nonnegative row
    /// ([`layout`]), a maximization to the minimization of the negated objective, and the
    /// quadratic to its upper triangle with evidence for exactly that matrix. Variable
    /// bounds stay on the contract; the cone adapter appends them as rows.
    ///
    /// # Errors
    /// A discrete column, a malformed problem, or missing, stale or unsymmetric quadratic
    /// evidence.
    pub fn from_coefficients(
        p: &CoefficientProblem,
        evidence: Option<&dyn QuadraticEvidence>,
    ) -> Result<Lowered, ProblemError> {
        p.validate_convex(evidence)?;
        if !p.objectives.is_empty() {
            return Err(ProblemError::Unsupported(
                "a cone model has one objective; several are optimized lexicographically by HiGHS or a staged sequence".into(),
            ));
        }
        let sign = p.sense.sign();
        let n = p.contract.variables.len();
        let zero;
        let q = match &p.hessian {
            Some(q) => q,
            None => {
                zero = faer::sparse::SparseColMat::try_new_from_triplets(n, n, &[])
                    .map_err(|e| ProblemError::Internal(e.to_string()))?;
                &zero
            }
        };
        let (full, evidence) = pse_math::convexity::minimization_form(q, sign, evidence)?;
        let (problem, rows) = lower(p, upper(&full, 1.0))?;
        Ok(Lowered {
            problem,
            evidence,
            rows,
            sign,
        })
    }
}
/// The upper triangle of `factor · q` in the cone boundary's storage.
fn upper(q: &faer::sparse::SparseColMat<usize, f64>, factor: f64) -> SparseMatrix {
    let n = q.ncols();
    let mut column_starts = vec![0];
    let mut row_indices = Vec::new();
    let mut values = Vec::new();
    for c in 0..n {
        let mut entries: Vec<_> = q
            .row_idx_of_col(c)
            .zip(q.val_of_col(c))
            .filter(|(r, v)| *r <= c && **v != 0.0)
            .map(|(r, v)| (r, factor * v))
            .collect();
        entries.sort_by_key(|(r, _)| *r);
        for (r, v) in entries {
            row_indices.push(r);
            values.push(v);
        }
        column_starts.push(row_indices.len());
    }
    SparseMatrix::new(n, n, column_starts, row_indices, values)
}
/// The cone form of `p` over [`layout`], with `quadratic` as its upper-triangle objective.
fn lower(
    p: &CoefficientProblem,
    quadratic: SparseMatrix,
) -> Result<(ConicProblem, Vec<LoweredRow>), ProblemError> {
    p.validate()?;
    if p.domains
        .iter()
        .any(|d| *d != ModelingVariableDomain::Continuous)
    {
        return Err(ProblemError::Unsupported(
            "the cone form represents continuous variables only".into(),
        ));
    }
    let rows = layout(&p.bounds);
    let mut cone_rows = vec![Vec::new(); p.bounds.len()];
    for (k, r) in rows.iter().enumerate() {
        cone_rows[r.row].push((k, r.side.sign()));
    }
    let a = &p.constraints;
    let n = a.ncols();
    let mut column_starts = vec![0];
    let mut row_indices = Vec::new();
    let mut values = Vec::new();
    for c in 0..n {
        let mut entries = Vec::new();
        for (r, v) in a.row_idx_of_col(c).zip(a.val_of_col(c)) {
            entries.extend(cone_rows[r].iter().map(|(k, sign)| (*k, sign * v)));
        }
        entries.sort_by_key(|(k, _)| *k);
        for (k, v) in entries {
            row_indices.push(k);
            values.push(v);
        }
        column_starts.push(row_indices.len());
    }
    let zeros = rows.iter().filter(|r| r.side == RowSide::Equal).count();
    let mut cones = Vec::new();
    if zeros > 0 {
        cones.push(Cone::Zero { dimension: zeros });
    }
    if rows.len() > zeros {
        cones.push(Cone::Nonnegative {
            dimension: rows.len() - zeros,
        });
    }
    let sign = p.sense.sign();
    let problem = ConicProblem {
        contract: OracleContract {
            rows: rows
                .iter()
                .map(|r| {
                    let id = p.contract.rows[r.row];
                    // A two-sided row's second cone row needs its own identity.
                    if r.side == RowSide::Lower && p.bounds[r.row].1.is_finite() {
                        let mut h = pse_ids::FramedHasher::new(pse_ids::Frame::ConeLoweredRowV1);
                        h.id(&id);
                        h.finish_id()
                    } else {
                        id
                    }
                })
                .collect(),
            ..p.contract.clone()
        },
        quadratic,
        objective: p.objective.iter().map(|c| sign * c).collect(),
        constraints: SparseMatrix::new(rows.len(), n, column_starts, row_indices, values),
        rhs: rows
            .iter()
            .map(|r| {
                let (l, u) = p.bounds[r.row];
                match r.side {
                    RowSide::Equal | RowSide::Upper => u,
                    RowSide::Lower => -l,
                }
            })
            .collect(),
        cones,
        objective_constant: sign * p.objective_constant,
    };
    Ok((problem, rows))
}
/// The cone form of `p` without convexity evidence, for recomputing a certificate against
/// original data: the same layout and data as [`ConicProblem::from_coefficients`].
///
/// # Errors
/// A discrete column or a malformed problem.
pub(crate) fn data(
    p: &CoefficientProblem,
) -> Result<(ConicProblem, Vec<LoweredRow>), ProblemError> {
    let n = p.contract.variables.len();
    let quadratic = match &p.hessian {
        Some(q) => upper(q, p.sense.sign()),
        None => SparseMatrix::zeros(n, n),
    };
    lower(p, quadratic)
}
/// Row budgets of the cone rows: each cone row carries its coefficient row's budget.
pub(crate) fn row_budgets(rows: &[LoweredRow], t: &Tolerances) -> Tolerances {
    Tolerances {
        variables: t.variables.clone(),
        rows: rows.iter().map(|r| t.rows[r.row]).collect(),
        integrality: t.integrality,
    }
}
impl Lowered {
    /// Acceptance budgets of the cone form, from those of the coefficient rows.
    pub fn tolerances(&self, t: &Tolerances) -> Tolerances {
        row_budgets(&self.rows, t)
    }
    /// Restate a cone adapter's report on the coefficient rows it was lowered from, in the
    /// same (normalized) coordinates: the authored objective, one authored-sense multiplier
    /// per row (`c + Qx = Aᵀy + d`) and reduced costs `d` from the bound multipliers, quality
    /// over the coefficient rows, and certificate rows labelled by coefficient row and side.
    ///
    /// # Errors
    /// The report does not belong to this lowering.
    pub fn raise(
        &self,
        report: &mut SolveReport,
        p: &CoefficientProblem,
        tolerances: &Tolerances,
    ) -> Result<(), ProblemError> {
        let s = self.sign;
        report.rows = p.contract.rows.clone();
        if let Some(c) = &mut report.candidate {
            c.objective = c.objective.map(|v| s * v);
            if let Some(z) = c.row_dual.take() {
                if z.len() != self.rows.len() {
                    return Err(ProblemError::Internal("lowered row multipliers".into()));
                }
                let mut y = vec![0.0; p.bounds.len()];
                for (r, z) in self.rows.iter().zip(z) {
                    y[r.row] += match r.side {
                        RowSide::Lower => s * z,
                        RowSide::Equal | RowSide::Upper => -s * z,
                    };
                }
                c.row_dual = Some(y);
            }
            if let Some((lower, upper)) = c.bound_dual.take() {
                c.reduced_costs =
                    Some(lower.iter().zip(&upper).map(|(l, u)| s * (l - u)).collect());
            }
            c.slacks = None;
            let primal = c.primal.clone();
            match crate::quality::contained(|| p.quality(&primal, tolerances)) {
                Ok(q) => {
                    if !q.feasible() {
                        report.termination.assurance = crate::solve::Assurance::None;
                    }
                    report.quality = Some(q);
                }
                Err(e) => {
                    report.quality = None;
                    report.record_validation_failure(e);
                }
            }
        }
        if let Some(certificate) = &mut report.certificate
            && certificate.kind == CertificateKind::PrimalInfeasible
        {
            if certificate.ray.len() < self.rows.len() {
                return Err(ProblemError::Internal("lowered certificate rows".into()));
            }
            for (entry, r) in certificate.ray.iter_mut().zip(&self.rows) {
                entry.coordinate = r.side.coordinate();
                entry.id = p.contract.rows[r.row];
            }
        }
        report.provenance.insert(
            "lowering".into(),
            "coefficient rows in cone form: equalities as one zero cone, then each finite row side (upper a·x + s = U before lower -a·x + s = -L) as one nonnegative cone; a maximization minimizes the negated objective".into(),
        );
        report.provenance.insert(
            "duals".into(),
            "authored-sense row multipliers and reduced costs (c + Qx = Aᵀy + d), combined from the cone multipliers of each row's sides and bounds".into(),
        );
        Ok(())
    }
}
