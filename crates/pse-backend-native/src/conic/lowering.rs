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
// A B-tree can own a complete node for each entry. The node's eleven key/value
// positions plus sixteen words cover its child links and header, including sparse
// partially occupied trees. These are typed payloads, not an allocator/RSS claim.
fn contract_validation_bytes(columns: usize, rows: usize) -> Option<usize> {
    columns
        .checked_add(rows)?
        .checked_mul(11 * size_of::<pse_ids::SemanticId>() + 16 * size_of::<usize>())
}
// faer 0.24.4 triplet construction retains input triplets, one argsort index,
// one output row index/value and n+1 column offsets. Sorting is in place.
fn symmetric_sparse_bytes(columns: usize, source_entries: usize) -> Option<usize> {
    source_entries
        .checked_mul(2)?
        .checked_mul(
            size_of::<faer::sparse::Triplet<usize, usize, f64>>()
                + 2 * size_of::<usize>()
                + size_of::<f64>(),
        )?
        .checked_add(columns.checked_add(1)?.checked_mul(size_of::<usize>())?)
}
fn construction_overflow() -> ProblemError {
    ProblemError::Limit {
        kind: crate::LimitKind::Memory,
        detail: "conic construction extent".into(),
    }
}
impl ConicProblem {
    /// Actual source populations for coefficient-to-cone construction. Every bound
    /// contributes at most two cone rows and every A entry at most two entries.
    /// Includes quadratic symmetry maps, faer triplet/CSC conversion, temporary
    /// column sorting, Vec growth and the retained lowered contract. Opaque native
    /// solver/library workspace remains the caller's separate foreign allowance.
    /// # Errors
    /// Overflow before constructing any population.
    pub fn coefficient_allocation_bound(p: &CoefficientProblem) -> Result<usize, ProblemError> {
        let extent = || -> Option<usize> {
            let n = p.contract.variables.len();
            let m = p.bounds.len();
            let q = p.hessian.as_ref().map_or(0, |q| q.val().len());
            let a = p.constraints.val().len();
            let cone_rows = m.checked_mul(2)?;
            let sparse = symmetric_sparse_bytes(n, q)?;
            let symmetry =
                q.checked_mul(11 * size_of::<((usize, usize), f64)>() + 16 * size_of::<usize>())?;
            // Upper()'s per-column pairs coexist with their stable-sort scratch
            // and the growing output CSC arrays; the zero-Q source also owns offsets.
            let quadratic_arrays = q
                .checked_mul(2)?
                .checked_add(4)?
                .checked_mul(size_of::<usize>() + size_of::<f64>())?
                .checked_add(q.checked_mul(3)?.checked_add(4)?.checked_mul(size_of::<(
                    usize,
                    f64,
                )>(
                ))?)?
                .checked_add(
                    n.checked_add(1)?
                        .checked_mul(3)?
                        .checked_add(4)?
                        .checked_mul(size_of::<usize>())?,
                )?;
            let a_entries = a.checked_mul(2)?;
            let constraint_arrays = a_entries
                .checked_mul(2)?
                .checked_add(4)?
                .checked_mul(size_of::<usize>() + size_of::<f64>())?
                .checked_add(
                    a_entries
                        .checked_mul(3)?
                        .checked_add(4)?
                        .checked_mul(size_of::<(usize, f64)>())?,
                )?
                .checked_add(
                    n.checked_add(1)?
                        .checked_mul(2)?
                        .checked_add(4)?
                        .checked_mul(size_of::<usize>())?,
                )?;
            let row_maps = m
                .checked_mul(size_of::<Vec<(usize, f64)>>() + 4 * size_of::<(usize, f64)>())?
                .checked_add(
                    cone_rows
                        .checked_mul(2)?
                        .checked_add(4)?
                        .checked_mul(size_of::<LoweredRow>())?,
                )?;
            let contract = n
                .checked_mul(size_of::<crate::Variable>() + size_of::<f64>())?
                .checked_add(
                    p.contract
                        .rows
                        .len()
                        .checked_add(cone_rows)?
                        .checked_mul(size_of::<pse_ids::SemanticId>())?,
                )?
                .checked_add(cone_rows.checked_mul(size_of::<f64>())?)?
                .checked_add(4 * size_of::<Cone>())?;
            contract_validation_bytes(n, p.contract.rows.len())?
                .checked_mul(3)?
                .checked_add(sparse)?
                .checked_add(symmetry)?
                .checked_add(quadratic_arrays)?
                .checked_add(constraint_arrays)?
                .checked_add(row_maps)?
                .checked_add(contract)?
                .checked_add(size_of::<Lowered>())
        };
        extent().ok_or_else(construction_overflow)
    }
    /// Actual validation populations: identity uniqueness sets and the mirrored
    /// quadratic's triplet/argsort/CSC arrays. Sealed evidence validates by scanning
    /// the quadratic rather than allocating a new Gram or numerical assessment.
    /// # Errors
    /// Overflow before validation constructs its metadata or symmetric matrix.
    pub fn validation_allocation_bound(&self) -> Result<usize, ProblemError> {
        let n = self.contract.variables.len();
        contract_validation_bytes(n, self.contract.rows.len())
            .and_then(|v| v.checked_add(symmetric_sparse_bytes(n, self.quadratic.values.len())?))
            .ok_or_else(construction_overflow)
    }
    /// Lower a continuous linear or convex quadratic coefficient program: each equality
    /// row to a zero-cone row and each finite side of the other rows to a nonnegative row
    /// (`layout`), a maximization to the minimization of the negated objective, and the
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
/// The cone form of `p` over `layout`, with `quadratic` as its upper-triangle objective.
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
    /// Transport an admitted original lowering without repeating coefficient-to-cone lowering.
    /// The same row/side map and authored orientation govern report raising.
    pub fn transport(
        &self,
        normalization: &pse_math::normalization::Normalization,
    ) -> Result<Self, ProblemError> {
        let rows = self
            .rows
            .iter()
            .map(|row| {
                normalization.rows.get(row.row).copied().ok_or_else(|| {
                    ProblemError::Contract("lowered row normalization inventory".into())
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let cone_normalization = pse_math::normalization::Normalization {
            variables: normalization.variables.clone(),
            rows,
            objective: normalization.objective,
        };
        let (problem, evidence) =
            crate::transport::conic(&self.problem, &cone_normalization, &self.evidence)?;
        Ok(Self {
            problem,
            evidence,
            rows: self.rows.clone(),
            sign: self.sign,
        })
    }
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

#[cfg(test)]
mod contextual_tests {
    use super::*;
    use pse_math::{binding::ObjectiveSense, normalization::Normalization};
    fn id(n: u8) -> pse_ids::SemanticId {
        pse_ids::SemanticId::from_bytes([n; 16])
    }
    #[test]
    fn contextual_unit_admitted_lowering_transport_retains_maps_and_original_orientation() {
        let p = CoefficientProblem {
            contract: OracleContract {
                identity: pse_ids::ContentHash::from_bytes([1; 32]),
                variables: vec![crate::Variable {
                    id: id(1),
                    lower: -10.0,
                    upper: 10.0,
                }],
                rows: vec![id(2), id(3)],
                derivatives: pse_kernels::DerivativeOrder::Value,
                smoothness: pse_kernels::DerivativeOrder::Second,
            },
            objective: vec![3.0],
            objective_constant: 7.0,
            sense: ObjectiveSense::Maximize,
            domains: vec![ModelingVariableDomain::Continuous],
            assumptions: pse_ids::ContentHash::from_bytes([2; 32]),
            constraints: faer::sparse::SparseColMat::try_new_from_triplets(
                2,
                1,
                &[
                    faer::sparse::Triplet::new(0, 0, 1.0),
                    faer::sparse::Triplet::new(1, 0, 3.0),
                ],
            )
            .unwrap(),
            hessian: None,
            bounds: vec![(4.0, 4.0), (-2.0, 5.0)],
            objectives: vec![],
        };
        let demand = ConicProblem::coefficient_allocation_bound(&p).unwrap();
        assert!(demand < 1 << 20);
        let admitted = ConicProblem::from_coefficients(&p, None).unwrap();
        assert_eq!(admitted.rows.len(), 3);
        assert!(admitted.problem.constraints.values.len() <= 2 * p.constraints.val().len());
        assert!(admitted.problem.validation_allocation_bound().unwrap() < 1 << 20);
        assert!(symmetric_sparse_bytes(usize::MAX, 1).is_none());
        assert!(symmetric_sparse_bytes(1, usize::MAX).is_none());
        let normalization = Normalization {
            variables: vec![2.0],
            rows: vec![4.0, 8.0],
            objective: 16.0,
        };
        let transported = admitted.transport(&normalization).unwrap();
        let (normalized, evidence) =
            crate::transport::coefficients(&p, &normalization, None).unwrap();
        let reference = ConicProblem::from_coefficients(
            &normalized,
            evidence.as_ref().map(|e| -> &dyn QuadraticEvidence { e }),
        )
        .unwrap();
        assert_eq!(transported.sign, -1.0);
        assert_eq!(
            transported
                .rows
                .iter()
                .map(|r| (r.row, r.side))
                .collect::<Vec<_>>(),
            [
                (0, RowSide::Equal),
                (1, RowSide::Upper),
                (1, RowSide::Lower)
            ]
        );
        assert_eq!(
            transported.problem.constraints,
            reference.problem.constraints
        );
        assert_eq!(transported.problem.rhs, reference.problem.rhs);
        assert_eq!(transported.problem.objective, reference.problem.objective);
        assert_eq!(
            transported.problem.objective_constant,
            reference.problem.objective_constant
        );
        assert_eq!(
            transported.problem.contract.rows,
            admitted.problem.contract.rows
        );
        transported.problem.validate(&transported.evidence).unwrap();
        admitted.problem.validate(&admitted.evidence).unwrap();
        assert_eq!(admitted.problem.rhs, [4.0, 5.0, 2.0]);
        let facts = crate::routing::conic_facts(&admitted.problem, &admitted.evidence).unwrap();
        assert!(facts.convexity.cone());
        assert!(crate::routing::class_evidence_required(
            &facts,
            crate::solve::SolveIntent::Optimize,
            crate::solve::SolverSelection::Auto
        ));
        assert_eq!(
            crate::routing::problem_classes(&facts, crate::solve::SolveIntent::Optimize, false)[0],
            crate::solve::ProblemClass::ContinuousCone
        );
    }
    fn assert_cone_ready(
        problem: &ConicProblem,
        certificate: &dyn QuadraticEvidence,
        coefficient_certificate: Option<&dyn QuadraticEvidence>,
    ) {
        use crate::{
            execution::{BackendSettings, Representation},
            routing::{ArtifactDemand, AssessmentState, ConeEvidence, Requirements, Route},
            solve::{Backend, Controls, SolveIntent, SolverSelection},
        };
        let facts = crate::routing::conic_facts(problem, certificate).unwrap();
        let mut context = crate::routing::test_context(&crate::execution::LINKED);
        context.structure = Some(
            crate::structural::conic_structure_with_cancel(
                problem,
                &std::sync::atomic::AtomicBool::new(false),
            )
            .unwrap(),
        );
        context.cone = Some(ConeEvidence {
            problem,
            certificate,
        });
        context.certificate = coefficient_certificate;
        context.prepared = &[ArtifactDemand::Representation(Representation::Cone)];
        let controls = Controls::default();
        let decision = Requirements {
            table: &crate::execution::LINKED,
            facts: &facts,
            intent: SolveIntent::Optimize,
            numerical_psd: false,
            least_squares: false,
            controls: &controls,
            settings: &BackendSettings::Default,
            sensitivity: false,
            context,
        }
        .decision(SolverSelection::Explicit(Backend::Clarabel));
        assert_eq!(decision.state, AssessmentState::Ready, "{decision:?}");
        assert_eq!(decision.route().unwrap(), Route::Native(Backend::Clarabel));
    }
    #[test]
    fn contextual_unit_cone_space_evidence_is_ready_for_linear_max_and_recognized_forms() {
        let cancel = std::sync::atomic::AtomicBool::new(false);
        let linear = ConicProblem {
            contract: OracleContract {
                identity: pse_ids::ContentHash::from_bytes([3; 32]),
                variables: vec![crate::Variable {
                    id: id(1),
                    lower: -1.0,
                    upper: 1.0,
                }],
                rows: vec![id(2)],
                derivatives: pse_kernels::DerivativeOrder::Value,
                smoothness: pse_kernels::DerivativeOrder::Second,
            },
            quadratic: SparseMatrix::zeros(1, 1),
            objective: vec![1.0],
            constraints: SparseMatrix::identity(1),
            rhs: vec![0.0],
            cones: vec![Cone::Nonnegative { dimension: 1 }],
            objective_constant: 0.0,
        };
        let zero = crate::conic::zero_certificate(1, &cancel).unwrap();
        assert_cone_ready(&linear, &zero, None);
        let hessian = faer::sparse::SparseColMat::try_new_from_triplets(
            1,
            1,
            &[faer::sparse::Triplet::new(0, 0, -2.0)],
        )
        .unwrap();
        let pse_math::convexity::Definiteness::Psd(original_proof) =
            pse_math::convexity::GramCertificate::certify(&hessian, -1.0, 10, &cancel).unwrap()
        else {
            panic!("negative Hessian must be concave in maximization sense")
        };
        let p = CoefficientProblem {
            contract: linear.contract.clone(),
            objective: vec![1.0],
            objective_constant: 0.0,
            sense: ObjectiveSense::Maximize,
            domains: vec![ModelingVariableDomain::Continuous],
            assumptions: linear.contract.identity,
            constraints: faer::sparse::SparseColMat::try_new_from_triplets(
                1,
                1,
                &[faer::sparse::Triplet::new(0, 0, 1.0)],
            )
            .unwrap(),
            hessian: Some(hessian),
            bounds: vec![(f64::NEG_INFINITY, 0.0)],
            objectives: vec![],
        };
        let maximum = ConicProblem::from_coefficients(&p, Some(&original_proof)).unwrap();
        assert_cone_ready(&maximum.problem, &maximum.evidence, Some(&original_proof));
        // The epigraph form produced by recognition has a zero quadratic, separate
        // auxiliary columns, and an exponential block; its retained proof is mandatory.
        let recognized = ConicProblem {
            contract: OracleContract {
                variables: vec![
                    crate::Variable {
                        id: id(1),
                        lower: -1.0,
                        upper: 1.0,
                    },
                    crate::Variable {
                        id: id(4),
                        lower: f64::NEG_INFINITY,
                        upper: f64::INFINITY,
                    },
                ],
                rows: vec![id(5), id(6), id(7)],
                ..linear.contract.clone()
            },
            quadratic: SparseMatrix::zeros(2, 2),
            objective: vec![0.0, 1.0],
            constraints: SparseMatrix::new(3, 2, vec![0, 1, 2], vec![0, 2], vec![-1.0, -1.0]),
            rhs: vec![0.0, 1.0, 0.0],
            cones: vec![Cone::Exponential],
            objective_constant: 0.0,
        };
        let recognized_proof = crate::conic::zero_certificate(2, &cancel).unwrap();
        assert_cone_ready(&recognized, &recognized_proof, None);
        assert!(matches!(
            crate::conic::zero_certificate(2, &std::sync::atomic::AtomicBool::new(true)),
            Err(ProblemError::Cancelled)
        ));
    }
}
