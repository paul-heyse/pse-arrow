// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Recognized convex programs in cone form (ADR-0121 Outcome 6). Preparation's curvature
//! pass establishes that a continuous factorable program is convex with a cone
//! representation ([`pse_math::convexity::ConvexityClass::Cone`]); this module lowers the
//! program's epigraph form ([`pse_math::curvature::Epigraph`]) to Clarabel's vocabulary and
//! raises the adapter's report back to the program's columns and rows, re-evaluating the
//! original model at the candidate (PS-10).
use super::{Cone, SparseMatrix};
use crate::{
    ConicProblem, OracleContract, ProblemError, Variable,
    execution::OriginalModel,
    quality::{self, Observation, Tolerances},
    solve::{Assurance, CertificateKind, RayCoordinate, SolveIntent, SolveReport},
};
use pse_ids::{FramedHasher, SemanticId};
use pse_kernels::DerivativeOrder;
use pse_math::{
    convexity::{Convexity, ConvexityClass, EXACT_OPERATIONS},
    curvature::{self, ConeConstraint, Epigraph, Linear, Recognition, Side},
    factorable::FactorableProgram,
    normalization::Normalization,
};
use std::sync::atomic::AtomicBool;

/// A recognized program in cone form, with the maps back to its columns and rows.
#[derive(Debug)]
pub struct Recognized {
    /// `min qᵀz` over the program's columns then the auxiliary columns, in original
    /// coordinates: every epigraph row side as a zero or nonnegative row, then every atom's
    /// cone. Variable boxes stay on the contract; the cone adapter appends them as rows.
    pub problem: ConicProblem,
    /// The program's columns, the first columns of the cone form.
    pub columns: usize,
    /// The program row and side each cone row states; `None` for an atom's cone row.
    pub rows: Vec<Option<(usize, Side)>>,
    /// The program's contract: its columns with their boxes and its rows.
    pub original: OracleContract,
    /// The program's row bounds.
    pub bounds: Vec<(f64, f64)>,
    /// Authored orientation: one to minimize, minus one to maximize.
    pub sign: f64,
    /// Whether the cone form carries the objective.
    pub objective: bool,
}
/// Lower a recognized `program` for `intent`. The curvature pass runs again over the
/// program the route rebuilt, and its class must equal the preparation's fact: the same
/// program and values give the same recognition.
///
/// # Errors
/// A program that is not recognized, a recognition that differs from `fact`, a
/// malformed cone form, or cancellation.
pub fn lower(
    program: &FactorableProgram,
    fact: &Convexity,
    intent: SolveIntent,
    cancel: &AtomicBool,
) -> Result<Recognized, ProblemError> {
    let Recognition::Cone(epigraph) = curvature::recognize(program, EXACT_OPERATIONS, cancel)?
    else {
        return Err(ProblemError::Unsupported(
            "the program is not a recognized convex cone program".into(),
        ));
    };
    if fact.class != ConvexityClass::Cone(epigraph.summary()) {
        return Err(ProblemError::Contract(
            "the rebuilt program's recognition differs from the preparation's convexity fact"
                .into(),
        ));
    }
    let optimizes = matches!(intent, SolveIntent::Optimize | SolveIntent::Certify);
    lower_epigraph(program, &epigraph, optimizes)
}
/// One cone row: `A z + s = b` with the row of `A` and `b`.
struct ConeRow {
    terms: Vec<(usize, f64)>,
    rhs: f64,
    origin: Option<(usize, Side)>,
}
impl ConeRow {
    /// `s = form − offset`, the member itself.
    fn member(form: &Linear, origin: Option<(usize, Side)>) -> Self {
        Self {
            terms: form.terms.iter().map(|(k, v)| (*k, -v)).collect(),
            rhs: form.constant,
            origin,
        }
    }
    /// `s = bound − form`.
    fn below(form: &Linear, bound: f64, origin: Option<(usize, Side)>) -> Self {
        Self {
            terms: form.terms.iter().map(|(k, v)| (*k, *v)).collect(),
            rhs: bound - form.constant,
            origin,
        }
    }
}
fn lower_epigraph(
    program: &FactorableProgram,
    epigraph: &Epigraph,
    optimizes: bool,
) -> Result<Recognized, ProblemError> {
    let n = epigraph.columns;
    let width = n + epigraph.auxiliaries.len();
    let mut zero = Vec::new();
    let mut nonnegative = Vec::new();
    for r in &epigraph.rows {
        let origin = Some((r.row, r.side));
        match r.side {
            Side::Equal => zero.push(ConeRow::below(&r.form, r.bound, origin)),
            Side::Upper => nonnegative.push(ConeRow::below(&r.form, r.bound, origin)),
            Side::Lower => {
                let mut row = ConeRow::member(&r.form, origin);
                row.rhs -= r.bound;
                nonnegative.push(row);
            }
        }
    }
    let mut blocks: Vec<(Cone, Vec<ConeRow>)> = Vec::new();
    for cone in &epigraph.cones {
        match cone {
            ConeConstraint::Nonnegative(f) => nonnegative.push(ConeRow::member(f, None)),
            ConeConstraint::Exponential(members) => blocks.push((
                Cone::Exponential,
                members.iter().map(|m| ConeRow::member(m, None)).collect(),
            )),
            ConeConstraint::Power { alpha, members } => blocks.push((
                Cone::Power { alpha: *alpha },
                members.iter().map(|m| ConeRow::member(m, None)).collect(),
            )),
            ConeConstraint::SecondOrder(members) => blocks.push((
                Cone::SecondOrder {
                    dimension: members.len(),
                },
                members.iter().map(|m| ConeRow::member(m, None)).collect(),
            )),
        }
    }
    let mut cones = Vec::new();
    if !zero.is_empty() {
        cones.push(Cone::Zero {
            dimension: zero.len(),
        });
    }
    if !nonnegative.is_empty() {
        cones.push(Cone::Nonnegative {
            dimension: nonnegative.len(),
        });
    }
    let mut rows = zero;
    rows.extend(nonnegative);
    for (cone, members) in blocks {
        cones.push(cone);
        rows.extend(members);
    }
    // Identities: a program row names its first cone row; its second side and every atom
    // row and auxiliary column are derived from the program.
    let derived = |kind: &str, index: usize| {
        let mut h = FramedHasher::new(pse_ids::Frame::ConeRecognizedV1);
        h.hash(&program.key).str(kind).u64(index as u64);
        h.finish_id()
    };
    let mut ids = Vec::with_capacity(rows.len());
    let mut named = std::collections::BTreeSet::new();
    for (k, row) in rows.iter().enumerate() {
        ids.push(match row.origin {
            Some((r, _)) if named.insert(r) => program.rows[r].id,
            Some((r, _)) => {
                let mut h = FramedHasher::new(pse_ids::Frame::ConeLoweredRowV1);
                h.id(&program.rows[r].id);
                h.finish_id()
            }
            None => derived("row", k),
        });
    }
    let mut variables: Vec<Variable> = program
        .variables
        .iter()
        .map(|v| Variable {
            id: v.id,
            lower: v.lower,
            upper: v.upper,
        })
        .collect();
    variables.extend((0..epigraph.auxiliaries.len()).map(|k| Variable {
        id: derived("column", k),
        lower: f64::NEG_INFINITY,
        upper: f64::INFINITY,
    }));
    let mut columns: Vec<Vec<(usize, f64)>> = vec![Vec::new(); width];
    for (k, row) in rows.iter().enumerate() {
        for (c, v) in &row.terms {
            if *c >= width {
                return Err(ProblemError::Internal("epigraph coordinate".into()));
            }
            if *v != 0.0 {
                columns[*c].push((k, *v));
            }
        }
    }
    let mut column_starts = vec![0];
    let mut row_indices = Vec::new();
    let mut values = Vec::new();
    for column in &mut columns {
        column.sort_by_key(|(k, _)| *k);
        for (k, v) in column.iter() {
            row_indices.push(*k);
            values.push(*v);
        }
        column_starts.push(row_indices.len());
    }
    let mut objective = vec![0.0; width];
    let mut objective_constant = 0.0;
    let carried = optimizes && epigraph.objective.is_some();
    if let (true, Some(form)) = (optimizes, &epigraph.objective) {
        for (c, v) in &form.terms {
            objective[*c] = *v;
        }
        objective_constant = form.constant;
    }
    let mut identity = FramedHasher::new(pse_ids::Frame::ConeRecognizedV1);
    identity
        .hash(&program.key)
        .str("contract")
        .bool(carried)
        .u64(width as u64)
        .u64(rows.len() as u64);
    let contract = OracleContract {
        identity: identity.finish_hash(),
        variables,
        rows: ids,
        derivatives: DerivativeOrder::Value,
        smoothness: DerivativeOrder::Value,
    };
    let problem = ConicProblem {
        contract,
        quadratic: SparseMatrix::zeros(width, width),
        objective,
        constraints: SparseMatrix::new(rows.len(), width, column_starts, row_indices, values),
        rhs: rows.iter().map(|r| r.rhs).collect(),
        cones,
        objective_constant,
    };
    let original = OracleContract {
        identity: program.key,
        variables: problem.contract.variables[..n].to_vec(),
        rows: program.rows.iter().map(|r| r.id).collect(),
        derivatives: DerivativeOrder::Value,
        smoothness: DerivativeOrder::Value,
    };
    Ok(Recognized {
        columns: n,
        rows: rows.iter().map(|r| r.origin).collect(),
        original,
        bounds: program.rows.iter().map(|r| (r.lower, r.upper)).collect(),
        sign: program.objective.as_ref().map_or(1.0, |o| o.sense.sign()),
        objective: carried,
        problem,
    })
}
impl Recognized {
    /// Known retained buffers of the cone form and its maps.
    pub fn bytes(&self) -> usize {
        let p = &self.problem;
        let matrix = |a: &SparseMatrix| {
            (a.column_starts.capacity() + a.row_indices.capacity()) * size_of::<usize>()
                + a.values.capacity() * size_of::<f64>()
        };
        size_of::<Self>()
            + matrix(&p.quadratic)
            + matrix(&p.constraints)
            + (p.objective.capacity() + p.rhs.capacity()) * size_of::<f64>()
            + (p.contract.variables.capacity() + self.original.variables.capacity())
                * size_of::<Variable>()
            + (p.contract.rows.capacity() + self.original.rows.capacity()) * size_of::<SemanticId>()
            + p.cones.capacity() * size_of::<Cone>()
            + self.rows.capacity() * size_of::<Option<(usize, Side)>>()
            + self.bounds.capacity() * size_of::<(f64, f64)>()
    }
    /// Coordinates of the cone form: the case's normalization for the program's columns
    /// and rows, unit scale for auxiliary columns and for every atom's cone rows (a
    /// nonlinear cone block keeps one common factor).
    ///
    /// # Errors
    /// A normalization of other dimensions.
    pub fn normalization(&self, n: &Normalization) -> Result<Normalization, ProblemError> {
        n.validate(self.columns, self.original.rows.len())?;
        let mut variables = n.variables.clone();
        variables.resize(self.problem.contract.variables.len(), 1.0);
        Ok(Normalization {
            variables,
            rows: self
                .rows
                .iter()
                .map(|origin| origin.map_or(1.0, |(r, _)| n.rows[r]))
                .collect(),
            objective: n.objective,
        })
    }
    /// Acceptance budgets of the cone form in original coordinates: each program row's
    /// budget for its cone rows, and the normalized feasibility budget for auxiliary
    /// columns and atom rows, whose values the original re-evaluation supersedes.
    ///
    /// # Errors
    /// Budgets of other dimensions.
    pub fn tolerances(&self, t: &Tolerances, feasibility: f64) -> Result<Tolerances, ProblemError> {
        t.validate(self.columns, self.original.rows.len())?;
        let mut variables = t.variables.clone();
        variables.resize(self.problem.contract.variables.len(), feasibility);
        Ok(Tolerances {
            variables,
            rows: self
                .rows
                .iter()
                .map(|origin| origin.map_or(feasibility, |(r, _)| t.rows[r]))
                .collect(),
            integrality: t.integrality,
        })
    }
    /// Restate a cone adapter's report, in original coordinates, on the program: the
    /// candidate's program columns, one authored-sense multiplier per program row
    /// (combined from its sides' cone multipliers) and reduced costs from the bound
    /// multipliers; certificate rows labelled by program row and side; and quality and
    /// observation from a fresh evaluation of the original model at the candidate, which
    /// supersede the cone form's (PS-10).
    ///
    /// # Errors
    /// The report does not belong to this lowering.
    pub fn raise(
        &self,
        report: &mut SolveReport,
        original: &mut dyn OriginalModel,
        tolerances: &Tolerances,
    ) -> Result<(), ProblemError> {
        let s = self.sign;
        report.variables = self.original.variables.iter().map(|v| v.id).collect();
        report.rows = self.original.rows.clone();
        if let Some(c) = &mut report.candidate {
            if c.primal.len() != self.problem.contract.variables.len() {
                return Err(ProblemError::Internal("recognized cone candidate".into()));
            }
            c.primal.truncate(self.columns);
            if let Some(z) = c.row_dual.take() {
                if z.len() != self.rows.len() {
                    return Err(ProblemError::Internal("recognized cone multipliers".into()));
                }
                let mut y = vec![0.0; self.original.rows.len()];
                for (origin, z) in self.rows.iter().zip(z) {
                    if let Some((r, side)) = origin {
                        y[*r] += match side {
                            Side::Lower => s * z,
                            Side::Equal | Side::Upper => -s * z,
                        };
                    }
                }
                c.row_dual = Some(y);
            }
            if let Some((lower, upper)) = c.bound_dual.take() {
                c.reduced_costs = Some(
                    lower
                        .iter()
                        .zip(&upper)
                        .take(self.columns)
                        .map(|(l, u)| s * (l - u))
                        .collect(),
                );
            }
            c.slacks = None;
            c.objective = None;
            let primal = c.primal.clone();
            match self.observe(original, tolerances, &primal) {
                Ok((q, o)) => {
                    c.objective = o.objective.filter(|_| self.objective);
                    if !q.feasible() {
                        report.termination.assurance = Assurance::None;
                    }
                    report.quality = Some(q);
                    report.observation = Some(o);
                    report.clear_validation_failure();
                }
                Err(e) => {
                    report.quality = None;
                    report.observation = None;
                    report.record_validation_failure(e);
                }
            }
        }
        if let Some(certificate) = &mut report.certificate
            && certificate.kind == CertificateKind::PrimalInfeasible
        {
            for (entry, origin) in certificate.ray.iter_mut().zip(&self.rows) {
                if let Some((r, side)) = origin {
                    entry.coordinate = match side {
                        Side::Equal => RayCoordinate::Row,
                        Side::Upper => RayCoordinate::RowUpper,
                        Side::Lower => RayCoordinate::RowLower,
                    };
                    entry.id = self.original.rows[*r];
                }
            }
        }
        report.provenance.insert(
            "lowering".into(),
            "recognized convex program in epigraph cone form (ADR-0121): equality rows as one zero cone, each finite row side and absolute-value atom as one nonnegative cone, then one exponential, power or second-order cone per atom over auxiliary epigraph columns; a maximization minimizes the negated objective".into(),
        );
        report.provenance.insert(
            "duals".into(),
            "authored-sense row multipliers combined from the cone multipliers of each row's sides, and reduced costs from the column bound multipliers".into(),
        );
        Ok(())
    }
    /// Original-model quality and observation at the program columns `x`.
    fn observe(
        &self,
        original: &mut dyn OriginalModel,
        tolerances: &Tolerances,
        x: &[f64],
    ) -> Result<(quality::Quality, Observation), ProblemError> {
        quality::contained(|| {
            if x.iter().any(|v| !v.is_finite()) {
                return Err(ProblemError::numerical("invalid recognized cone candidate"));
            }
            let fresh = original.evaluate(x)?;
            let q = quality::observed(
                &self.original,
                &self.bounds,
                x,
                &fresh.constraints,
                tolerances,
            )?;
            let mut observation =
                Observation::from_values(fresh.objective, fresh.constraints, self.bounds.clone())?;
            observation.sources = fresh.sources;
            Ok((q, observation))
        })
    }
}
