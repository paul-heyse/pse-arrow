// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Class-aware admission of conservative original-equation support.
use crate::{OracleContract, ProblemError};
use pse_structural::{
    incidence::{CaseIncidence, Constraint, Incidence, StructuralAnalysis},
    projection::{GraphLimits, Scope},
};

/// Registry-owned structural interpretation, distinct from numerical rank.
pub use pse_model::generated::enums::NativeStructuralMode as Mode;

/// Structural policy is declared once in the registry.
pub use pse_model::generated::enums::NativeStructuralPolicy as Policy;
/// Explicit root intent dominates; initialization uses its represented mathematical class.
pub fn mode(
    policy: Policy,
    facts: &pse_math::facts::ProblemFacts,
    intent: crate::solve::SolveIntent,
) -> Mode {
    if crate::routing::root_intent(facts, intent) {
        return Mode::Roots;
    }
    match policy {
        Policy::Roots => Mode::Roots,
        Policy::Equalities => Mode::Nlp,
        Policy::NativeFeasibility => Mode::NativeFeasibility,
        Policy::Factorable if facts.affine_rows.iter().all(|affine| *affine) => {
            Mode::NativeFeasibility
        }
        Policy::Factorable => Mode::Nlp,
    }
}

/// Retained original bound-view assessment, including a refused matching witness.
#[derive(Clone, Debug)]
pub struct Assessment {
    /// Representation-qualified interpretation; never numerical rank.
    pub mode: Mode,
    /// Complete original free-variable inventory.
    pub variables: Vec<pse_ids::SemanticId>,
    /// Complete original constraints, including inequality roles and isolated rows.
    pub equations: Vec<Constraint>,
    /// The existing compiler-owned library matching; no second matching search.
    pub witness: pse_math::SharedAllocation<StructuralAnalysis>,
    /// Named matching refusal, retained separately from the witness.
    pub refusal: Option<(Vec<pse_ids::SemanticId>, Vec<pse_ids::SemanticId>)>,
}
impl Assessment {
    /// Complete retained payload, including this assessment's shared witness once.
    /// Callers retaining several assessments of one witness charge that shared owner once.
    pub fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            + self.variables.capacity() * size_of::<pse_ids::SemanticId>()
            + self.equations.capacity() * size_of::<Constraint>()
            + self.witness.retained_bytes()
            + self.refusal.as_ref().map_or(0, |(rows, columns)| {
                (rows.capacity() + columns.capacity()) * size_of::<pse_ids::SemanticId>()
            })
    }
    /// Assess the already analyzed complete original view.
    pub fn new(
        mode: Mode,
        variables: Vec<pse_ids::SemanticId>,
        equations: Vec<Constraint>,
        witness: pse_math::SharedAllocation<StructuralAnalysis>,
    ) -> Result<Self, ProblemError> {
        if matches!(witness.scope, Scope::Partial(_)) {
            return Err(ProblemError::Contract(
                "structural admission requires complete scope".into(),
            ));
        }
        let mut rows = if matches!(mode, Mode::NativeFeasibility | Mode::PointEvaluation) {
            vec![]
        } else {
            witness.over.rows.clone()
        };
        let columns = if mode == Mode::Roots {
            witness.under.columns.clone()
        } else {
            vec![]
        };
        if mode == Mode::Roots {
            let matched = witness
                .matching
                .iter()
                .map(|(r, _)| *r)
                .collect::<std::collections::BTreeSet<_>>();
            rows.extend(
                equations
                    .iter()
                    .filter(|r| r.lower.is_none() || r.lower != r.upper || !matched.contains(&r.id))
                    .map(|r| r.id),
            );
            rows.sort_unstable();
            rows.dedup();
        }
        let refusal = (!rows.is_empty() || !columns.is_empty()).then_some((rows, columns));
        Ok(Self {
            mode,
            variables,
            equations,
            witness,
            refusal,
        })
    }
    /// Publish the existing complete witness and original inventories on success or refusal.
    pub fn row(
        &self,
        request_identity: pse_ids::ContentHash,
        step: i64,
    ) -> pse_model::generated::runtime::structural_assessments::Row {
        use pse_model::generated::{
            enums::StructuralScopeKind as Kind, runtime::structural_assessments::*,
        };
        let (scope, scope_model, scope_members, scope_rows, scope_columns, scope_inputs) =
            match &self.witness.scope {
                Scope::Whole(model) => (Kind::Whole, *model, vec![], vec![], vec![], vec![]),
                Scope::Partial(model) => (Kind::Partial, *model, vec![], vec![], vec![], vec![]),
                Scope::Independent { model, members } => (
                    Kind::Independent,
                    *model,
                    members.iter().copied().collect(),
                    vec![],
                    vec![],
                    vec![],
                ),
                Scope::Conditional {
                    model,
                    rows,
                    columns,
                    inputs,
                } => (
                    Kind::Conditional,
                    *model,
                    vec![],
                    rows.iter().copied().collect(),
                    columns.iter().copied().collect(),
                    inputs.iter().copied().collect(),
                ),
            };
        let matched_rows = self
            .witness
            .matching
            .iter()
            .map(|(row, _)| *row)
            .collect::<std::collections::BTreeSet<_>>();
        let matched_columns = self
            .witness
            .matching
            .iter()
            .map(|(_, column)| *column)
            .collect::<std::collections::BTreeSet<_>>();
        RuntimeStructuralAssessmentsRow {
            request_identity,
            step,
            mode: self.mode,
            scope,
            scope_model,
            scope_members,
            scope_rows,
            scope_columns,
            scope_inputs,
            variables: self.variables.clone(),
            equations: self
                .equations
                .iter()
                .map(|row| RuntimeStructuralAssessmentsFieldEquationsItem {
                    id: row.id,
                    lower: row.lower,
                    upper: row.upper,
                })
                .collect(),
            matching: self
                .witness
                .matching
                .iter()
                .map(
                    |(row, column)| RuntimeStructuralAssessmentsFieldMatchingItem {
                        row: *row,
                        column: *column,
                    },
                )
                .collect(),
            unmatched_rows: self
                .equations
                .iter()
                .filter(|row| {
                    row.lower.is_some() && row.lower == row.upper && !matched_rows.contains(&row.id)
                })
                .map(|row| row.id)
                .collect(),
            unmatched_columns: self
                .variables
                .iter()
                .filter(|id| !matched_columns.contains(id))
                .copied()
                .collect(),
            optimization_freedom: self.optimization_freedom().map(|value| value as i64),
            admitted: self.refusal.is_none(),
            provenance: self.witness.provenance.to_owned(),
        }
    }
    /// Admission retains every original identity, including through refusal.
    pub fn admit(&self) -> Result<(), ProblemError> {
        match &self.refusal {
            Some((rows, columns)) => Err(ProblemError::Structural {
                mode: self.mode,
                rows: rows.clone(),
                columns: columns.clone(),
            }),
            None => Ok(()),
        }
    }
    /// Signed original free-variable/equality inventory difference; never an admission verdict.
    pub fn inventory_difference(&self) -> i64 {
        self.variables.len() as i64
            - self
                .equations
                .iter()
                .filter(|r| r.lower.is_some() && r.lower == r.upper)
                .count() as i64
    }
    /// The count meaningful for this structural mode; no numerical rank or zero-DOF claim.
    pub fn optimization_freedom(&self) -> Option<usize> {
        (self.mode == Mode::Nlp).then(|| {
            self.variables
                .len()
                .saturating_sub(self.witness.matching.len())
        })
    }
}

/// Consume the compiler's existing library-owned analysis without repeating matching.
pub fn admit(analysis: &StructuralAnalysis, mode: Mode) -> Result<(), ProblemError> {
    if matches!(analysis.scope, Scope::Partial(_)) {
        return Err(ProblemError::Contract(
            "structural admission requires complete scope".into(),
        ));
    }
    let rows = if matches!(mode, Mode::NativeFeasibility | Mode::PointEvaluation) {
        vec![]
    } else {
        analysis.over.rows.clone()
    };
    let columns = if mode == Mode::Roots {
        analysis.under.columns.clone()
    } else {
        vec![]
    };
    if !rows.is_empty() || !columns.is_empty() {
        return Err(ProblemError::Structural {
            mode,
            rows,
            columns,
        });
    }
    Ok(())
}

/// Reuse a checked matching witness only against the complete current equation
/// inventory and support. This is linear witness validation, not another matching
/// search. Opaque callback oracles retain the full low-level analysis below.
pub fn check(
    contract: &OracleContract,
    pattern: faer::sparse::SymbolicSparseColMatRef<'_, usize>,
    bounds: &[(f64, f64)],
    mode: Mode,
    analysis: Option<&StructuralAnalysis>,
) -> Result<(), ProblemError> {
    let Some(analysis) = analysis else {
        return oracle(contract, pattern, bounds, mode);
    };
    admit(analysis, mode)?;
    let invalid = || {
        ProblemError::Internal("compiler matching does not establish current oracle support".into())
    };
    if bounds.len() != contract.rows.len()
        || pattern.nrows() != contract.rows.len()
        || pattern.ncols() != contract.variables.len()
    {
        return Err(invalid());
    }
    let rows = contract
        .rows
        .iter()
        .enumerate()
        .map(|(i, id)| (*id, i))
        .collect::<std::collections::BTreeMap<_, _>>();
    let columns = contract
        .variables
        .iter()
        .enumerate()
        .map(|(i, v)| (v.id, i))
        .collect::<std::collections::BTreeMap<_, _>>();
    let required = bounds
        .iter()
        .enumerate()
        .filter_map(|(i, (l, u))| (l.is_finite() && l == u).then_some(i))
        .collect::<std::collections::BTreeSet<_>>();
    let mut matched = std::collections::BTreeSet::new();
    let mut used = std::collections::BTreeSet::new();
    for (row, column) in &analysis.matching {
        let r = *rows.get(row).ok_or_else(invalid)?;
        let c = *columns.get(column).ok_or_else(invalid)?;
        if !required.contains(&r)
            || !matched.insert(r)
            || !used.insert(c)
            || pattern.row_idx()[pattern.col_range(c)]
                .binary_search(&r)
                .is_err()
        {
            return Err(invalid());
        }
    }
    if (matches!(mode, Mode::Roots | Mode::Nlp) && matched != required)
        || (mode == Mode::Roots
            && (required.len() != contract.rows.len() || used.len() != contract.variables.len()))
    {
        return Err(invalid());
    }
    Ok(())
}

/// Whether a native coordinate transform preserves the admitted original identity inventory.
pub(crate) fn same_inventory(assessment: &Assessment, contract: &OracleContract) -> bool {
    assessment
        .variables
        .iter()
        .copied()
        .collect::<std::collections::BTreeSet<_>>()
        == contract
            .variables
            .iter()
            .map(|variable| variable.id)
            .collect()
        && assessment
            .equations
            .iter()
            .map(|row| row.id)
            .collect::<std::collections::BTreeSet<_>>()
            == contract.rows.iter().copied().collect()
}
#[derive(Debug)]
struct NlpWitness {
    inner: Box<dyn crate::NlpOracle>,
    witness: pse_math::SharedAllocation<StructuralAnalysis>,
}
pub(crate) fn retain_nlp(
    inner: Box<dyn crate::NlpOracle>,
    witness: pse_math::SharedAllocation<StructuralAnalysis>,
) -> Box<dyn crate::NlpOracle> {
    Box::new(NlpWitness { inner, witness })
}
impl crate::NlpOracle for NlpWitness {
    fn solve_separator(&self) -> Option<&crate::SolveSeparator> {
        self.inner.solve_separator()
    }
    fn structural_analysis(&self) -> Option<&StructuralAnalysis> {
        Some(&self.witness)
    }
    fn normalization(&self) -> Option<&pse_math::normalization::Normalization> {
        self.inner.normalization()
    }
    fn constraint_sources(&self) -> Result<Vec<pse_math::assembly::OutputValue>, ProblemError> {
        self.inner.constraint_sources()
    }
    fn presolve_facts(&self) -> Option<&pse_math::presolve::Facts> {
        self.inner.presolve_facts()
    }
    fn derivative_facts(&self) -> crate::DerivativeFacts {
        self.inner.derivative_facts()
    }
    fn contract(&self) -> &OracleContract {
        self.inner.contract()
    }
    fn jacobian_pattern(&self) -> faer::sparse::SymbolicSparseColMatRef<'_, usize> {
        self.inner.jacobian_pattern()
    }
    fn hessian_pattern(&self) -> Option<faer::sparse::SymbolicSparseColMatRef<'_, usize>> {
        self.inner.hessian_pattern()
    }
    fn constraint_bounds(&self) -> &[(f64, f64)] {
        self.inner.constraint_bounds()
    }
    fn objective(&mut self, x: &[f64]) -> Result<f64, ProblemError> {
        self.inner.objective(x)
    }
    fn constraints(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        self.inner.constraints(x, out)
    }
    fn gradient(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        self.inner.gradient(x, out)
    }
    fn jacobian(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        self.inner.jacobian(x, out)
    }
    fn hessian(
        &mut self,
        x: &[f64],
        weight: f64,
        multipliers: &[f64],
        out: &mut [f64],
    ) -> Result<(), ProblemError> {
        self.inner.hessian(x, weight, multipliers, out)
    }
}
#[derive(Debug)]
struct RootsWitness {
    inner: Box<dyn crate::NleOracle>,
    witness: pse_math::SharedAllocation<StructuralAnalysis>,
}
pub(crate) fn retain_roots(
    inner: Box<dyn crate::NleOracle>,
    witness: pse_math::SharedAllocation<StructuralAnalysis>,
) -> Box<dyn crate::NleOracle> {
    Box::new(RootsWitness { inner, witness })
}
impl crate::NleOracle for RootsWitness {
    fn operations(&self) -> crate::RootOperations {
        self.inner.operations()
    }
    fn structural_analysis(&self) -> Option<&StructuralAnalysis> {
        Some(&self.witness)
    }
    fn guard_signs(
        &self,
    ) -> std::collections::BTreeMap<pse_ids::SemanticId, pse_math::presolve::GuardSign> {
        self.inner.guard_signs()
    }
    fn observe(&self, residual: Vec<f64>) -> Result<crate::quality::Observation, ProblemError> {
        self.inner.observe(residual)
    }
    fn contract(&self) -> &OracleContract {
        self.inner.contract()
    }
    fn residual(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        self.inner.residual(x, out)
    }
    fn jacobian_pattern(&self) -> faer::sparse::SymbolicSparseColMatRef<'_, usize> {
        self.inner.jacobian_pattern()
    }
    fn jacobian(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        self.inner.jacobian(x, out)
    }
    fn jacobian_product(
        &mut self,
        x: &[f64],
        direction: &[f64],
        out: &mut [f64],
    ) -> Result<(), ProblemError> {
        self.inner.jacobian_product(x, direction, out)
    }
}
/// Produce the existing conservative original matching for contextual policy assessment.
/// This extracts the existing CaseIncidence owner, not a second matching implementation.
pub fn oracle_structure(
    contract: &OracleContract,
    pattern: faer::sparse::SymbolicSparseColMatRef<'_, usize>,
    bounds: &[(f64, f64)],
    objective: bool,
) -> Result<crate::routing::Structure, ProblemError> {
    oracle_structure_with_cancel(
        contract,
        pattern,
        bounds,
        objective,
        &std::sync::atomic::AtomicBool::new(false),
    )
}
/// Checked conservative reservation before creating the original matching projection.
pub fn construction_bytes(
    contract: &OracleContract,
    pattern: faer::sparse::SymbolicSparseColMatRef<'_, usize>,
) -> Result<usize, ProblemError> {
    if pattern.nrows() != contract.rows.len() || pattern.ncols() != contract.variables.len() {
        return Err(ProblemError::Contract(
            "original structural dimensions".into(),
        ));
    }
    CaseIncidence::memory_extent(
        contract.rows.len(),
        contract.variables.len(),
        pattern.row_idx().len(),
    )
    .map_err(structural_error)
}
/// Complete retained structure payload, including its shared witness once.
pub fn retained_bytes(structure: &crate::routing::Structure) -> usize {
    size_of::<crate::routing::Structure>()
        + structure.variables.capacity() * size_of::<pse_ids::SemanticId>()
        + structure.equations.capacity() * size_of::<Constraint>()
        + structure.witness.retained_bytes()
}
fn structural_error(error: pse_structural::projection::ProjectionError) -> ProblemError {
    match error {
        pse_structural::projection::ProjectionError::Cancelled => ProblemError::Cancelled,
        pse_structural::projection::ProjectionError::Limit => {
            ProblemError::memory("structural construction extent")
        }
        error => ProblemError::Contract(error.to_string()),
    }
}
/// Build the same complete conservative witness with the caller's cancellation boundary.
pub fn oracle_structure_with_cancel(
    contract: &OracleContract,
    pattern: faer::sparse::SymbolicSparseColMatRef<'_, usize>,
    bounds: &[(f64, f64)],
    objective: bool,
    cancel: &std::sync::atomic::AtomicBool,
) -> Result<crate::routing::Structure, ProblemError> {
    construction_bytes(contract, pattern)?;
    if cancel.load(std::sync::atomic::Ordering::Relaxed) {
        return Err(ProblemError::Cancelled);
    }
    let n = contract.variables.len();
    let m = contract.rows.len();
    if pattern.nrows() != m || pattern.ncols() != n || bounds.len() != m {
        return Err(ProblemError::Contract(
            "original structural dimensions/equality contract".into(),
        ));
    }
    let rows: Vec<Constraint> = contract
        .rows
        .iter()
        .zip(bounds)
        .map(|(&id, &(lower, upper))| Constraint {
            id,
            lower: (lower != f64::NEG_INFINITY).then_some(lower),
            upper: (upper != f64::INFINITY).then_some(upper),
        })
        .collect();
    let mut edges = Vec::with_capacity(pattern.row_idx().len());
    for c in 0..n {
        for r in pattern.row_idx_of_col(c) {
            edges.push(Incidence {
                row: contract.rows[r],
                column: contract.variables[c].id,
                instance: contract.rows[r],
                output: 0,
            });
        }
    }
    from_inventory(contract, rows, edges, objective, cancel)
}
fn from_inventory(
    contract: &OracleContract,
    rows: Vec<Constraint>,
    edges: Vec<Incidence>,
    objective: bool,
    cancel: &std::sync::atomic::AtomicBool,
) -> Result<crate::routing::Structure, ProblemError> {
    let n = contract.variables.len();
    let m = contract.rows.len();
    let limits = GraphLimits {
        nodes: n.saturating_add(m),
        edges: edges.len(),
    };
    let incidence = CaseIncidence::new(
        Scope::Whole(pse_ids::derive_id(
            pse_ids::Frame::NativeStructuralScopeV1,
            &[contract.identity.as_bytes()],
        )),
        rows.clone(),
        contract.variables.iter().map(|v| v.id).collect(),
        edges,
        if objective {
            contract
                .variables
                .iter()
                .map(|variable| variable.id)
                .collect()
        } else {
            Default::default()
        },
        limits,
    )
    .map_err(structural_error)?;
    let analysis = incidence.analyze(cancel).map_err(structural_error)?;
    Ok(crate::routing::Structure {
        variables: contract
            .variables
            .iter()
            .map(|variable| variable.id)
            .collect(),
        equations: rows,
        witness: std::sync::Arc::new(analysis).into(),
    })
}

/// Original scalar row roles of explicit cone membership.
/// Zero blocks are equalities; other blocks retain membership rather than pretending
/// each component is an independent scalar inequality.
pub fn conic_bounds(problem: &crate::ConicProblem) -> Result<Vec<(f64, f64)>, ProblemError> {
    let mut bounds = Vec::with_capacity(problem.contract.rows.len());
    for cone in &problem.cones {
        let extent = bounds
            .len()
            .checked_add(cone.dim())
            .ok_or_else(|| ProblemError::memory("cone structural rows"))?;
        if extent > problem.contract.rows.len() {
            return Err(ProblemError::Contract("cone structural dimensions".into()));
        }
        let role = if matches!(cone, crate::conic::Cone::Zero { .. }) {
            (0.0, 0.0)
        } else {
            (f64::NEG_INFINITY, f64::INFINITY)
        };
        bounds.resize(extent, role);
    }
    if bounds.len() != problem.contract.rows.len() {
        return Err(ProblemError::Contract("cone structural dimensions".into()));
    }
    Ok(bounds)
}
/// Checked reservation for the explicit cone's existing matching owner.
pub fn conic_construction_bytes(problem: &crate::ConicProblem) -> Result<usize, ProblemError> {
    problem.constraints.validate()?;
    if problem.constraints.rows != problem.contract.rows.len()
        || problem.constraints.columns != problem.contract.variables.len()
    {
        return Err(ProblemError::Contract("cone structural dimensions".into()));
    }
    CaseIncidence::memory_extent(
        problem.contract.rows.len(),
        problem.contract.variables.len(),
        problem.constraints.row_indices.len(),
    )
    .map_err(structural_error)
}
/// Original explicit-cone incidence, using the same CaseIncidence matching producer.
pub fn conic_structure_with_cancel(
    problem: &crate::ConicProblem,
    cancel: &std::sync::atomic::AtomicBool,
) -> Result<crate::routing::Structure, ProblemError> {
    conic_construction_bytes(problem)?;
    if cancel.load(std::sync::atomic::Ordering::Relaxed) {
        return Err(ProblemError::Cancelled);
    }
    let bounds = conic_bounds(problem)?;
    let rows = problem
        .contract
        .rows
        .iter()
        .zip(bounds)
        .map(|(&id, (lower, upper))| Constraint {
            id,
            lower: lower.is_finite().then_some(lower),
            upper: upper.is_finite().then_some(upper),
        })
        .collect();
    let mut edges = Vec::with_capacity(problem.constraints.row_indices.len());
    for column in 0..problem.constraints.columns {
        for &row in &problem.constraints.row_indices[problem.constraints.column(column)] {
            edges.push(Incidence {
                row: problem.contract.rows[row],
                column: problem.contract.variables[column].id,
                instance: problem.contract.rows[row],
                output: 0,
            });
        }
    }
    from_inventory(&problem.contract, rows, edges, true, cancel)
}
/// Validate retained original inventory and its already interpreted witness at entry.
/// Numerical coefficient cancellation cannot invalidate conservative symbolic incidence.
pub fn validate_assessment(
    assessment: &Assessment,
    contract: &OracleContract,
    bounds: &[(f64, f64)],
) -> Result<(), ProblemError> {
    if !same_inventory(assessment, contract) || bounds.len() != contract.rows.len() {
        return Err(ProblemError::Internal(
            "retained structural inventory changed".into(),
        ));
    }
    let rows = pse_math::index::CheckedInventory::new(&assessment.equations, |row| row.id)?;
    for (id, &(lower, upper)) in contract.rows.iter().zip(bounds) {
        let row = rows
            .get(id)
            .ok_or_else(|| ProblemError::Internal("retained structural row missing".into()))?;
        let originally_equal = row
            .lower
            .is_some_and(|lower| lower.is_finite() && row.upper == Some(lower));
        if originally_equal != (lower.is_finite() && lower == upper) {
            return Err(ProblemError::Internal(
                "retained structural equality role changed".into(),
            ));
        }
    }
    admit(&assessment.witness, assessment.mode)
}

/// Low-level final preflight interprets the same original witness producer.
pub fn oracle(
    contract: &OracleContract,
    pattern: faer::sparse::SymbolicSparseColMatRef<'_, usize>,
    bounds: &[(f64, f64)],
    mode: Mode,
) -> Result<(), ProblemError> {
    let original = oracle_structure(contract, pattern, bounds, mode == Mode::Nlp)?;
    Assessment::new(
        mode,
        original.variables,
        original.equations,
        original.witness,
    )?
    .admit()
}

#[cfg(test)]
mod tests {
    use super::*;
    fn id(n: u8) -> pse_ids::SemanticId {
        pse_ids::SemanticId::from_bytes([n; 16])
    }
    #[test]
    fn roots_and_nlp_admit_original_matching_by_class() {
        let c = OracleContract {
            identity: pse_ids::ContentHash::from_bytes([1; 32]),
            variables: vec![
                crate::Variable {
                    id: id(1),
                    lower: -f64::INFINITY,
                    upper: f64::INFINITY,
                },
                crate::Variable {
                    id: id(2),
                    lower: -f64::INFINITY,
                    upper: f64::INFINITY,
                },
            ],
            rows: vec![id(3), id(4)],
            derivatives: pse_kernels::DerivativeOrder::First,
            smoothness: pse_kernels::DerivativeOrder::First,
        };
        let pattern = |pairs: &[(usize, usize)]| {
            faer::sparse::SymbolicSparseColMat::try_new_from_indices(
                2,
                2,
                &pairs
                    .iter()
                    .map(|&(r, c)| faer::sparse::Pair::new(r, c))
                    .collect::<Vec<_>>(),
            )
            .unwrap()
            .0
        };
        let good = pattern(&[(0, 0), (1, 1)]);
        oracle(&c, good.as_ref(), &[(0., 0.); 2], Mode::Roots).unwrap();
        let bad = pattern(&[(0, 0), (1, 0)]);
        let Err(ProblemError::Structural { rows, columns, .. }) =
            oracle(&c, bad.as_ref(), &[(0., 0.); 2], Mode::Roots)
        else {
            panic!("missing named deficiency")
        };
        assert!(rows.contains(&id(3)) && rows.contains(&id(4)));
        assert_eq!(columns, vec![id(2)]);
        assert!(oracle(&c, bad.as_ref(), &[(0., 0.); 2], Mode::Nlp).is_err());
        // Inequality rows do not erase valid optimization freedom or inherit square admission.
        oracle(&c, bad.as_ref(), &[(0., 0.), (-1., 1.)], Mode::Nlp).unwrap();
        assert!(oracle(&c, bad.as_ref(), &[(0., 0.), (-1., 1.)], Mode::Roots).is_err());
    }
    #[test]
    fn matching_witness_reuse_checks_edges_membership_and_equality_class() {
        let contract = OracleContract {
            identity: pse_ids::ContentHash::from_bytes([1; 32]),
            variables: vec![crate::Variable {
                id: id(1),
                lower: -1.0,
                upper: 1.0,
            }],
            rows: vec![id(2)],
            derivatives: pse_kernels::DerivativeOrder::First,
            smoothness: pse_kernels::DerivativeOrder::First,
        };
        let graph = CaseIncidence::new(
            Scope::Whole(id(3)),
            vec![Constraint {
                id: id(2),
                lower: Some(0.0),
                upper: Some(0.0),
            }],
            vec![id(1)],
            vec![Incidence {
                row: id(2),
                column: id(1),
                instance: id(4),
                output: 0,
            }],
            Default::default(),
            GraphLimits { nodes: 2, edges: 1 },
        )
        .unwrap();
        let analysis = graph
            .analyze(&std::sync::atomic::AtomicBool::new(false))
            .unwrap();
        use pse_math::index::{Entry, OriginalCol, OriginalRow};
        let entry = Entry::new(OriginalRow::new(0), OriginalCol::new(0));
        let pattern = pse_math::sparse::AssemblyMatrix::new(1, 1, &[entry], 10).unwrap();
        // Repeated validation consumes the same witness, without invoking analyze.
        for _ in 0..1000 {
            check(
                &contract,
                pattern.matrix().symbolic(),
                &[(0.0, 0.0)],
                Mode::Roots,
                Some(&analysis),
            )
            .unwrap();
        }
        let empty =
            pse_math::sparse::AssemblyMatrix::new::<OriginalRow, OriginalCol>(1, 1, &[], 10)
                .unwrap();
        assert!(
            check(
                &contract,
                empty.matrix().symbolic(),
                &[(0.0, 0.0)],
                Mode::Roots,
                Some(&analysis)
            )
            .is_err()
        );
        assert!(
            check(
                &contract,
                pattern.matrix().symbolic(),
                &[(-1.0, 1.0)],
                Mode::Nlp,
                Some(&analysis)
            )
            .is_err()
        );
        let mut stale = contract.clone();
        stale.variables[0].id = id(9);
        assert!(
            check(
                &stale,
                pattern.matrix().symbolic(),
                &[(0.0, 0.0)],
                Mode::Roots,
                Some(&analysis)
            )
            .is_err()
        );
        let mut incomplete = analysis.clone();
        incomplete.matching.clear();
        assert!(
            check(
                &contract,
                pattern.matrix().symbolic(),
                &[(0.0, 0.0)],
                Mode::Roots,
                Some(&incomplete)
            )
            .is_err()
        );
    }
    #[test]
    fn retained_original_assessment_distinguishes_matching_and_native_feasibility() {
        let equations = vec![
            Constraint {
                id: id(3),
                lower: Some(0.),
                upper: Some(0.),
            },
            Constraint {
                id: id(4),
                lower: Some(0.),
                upper: Some(0.),
            },
        ];
        let graph = CaseIncidence::new(
            Scope::Whole(id(5)),
            equations.clone(),
            vec![id(1)],
            vec![
                Incidence {
                    row: id(3),
                    column: id(1),
                    instance: id(3),
                    output: 0,
                },
                Incidence {
                    row: id(4),
                    column: id(1),
                    instance: id(4),
                    output: 0,
                },
            ],
            Default::default(),
            GraphLimits { nodes: 3, edges: 2 },
        )
        .unwrap();
        let witness = std::sync::Arc::new(
            graph
                .analyze(&std::sync::atomic::AtomicBool::new(false))
                .unwrap(),
        );
        let nonlinear = Assessment::new(
            Mode::Nlp,
            vec![id(1)],
            equations.clone(),
            witness.clone().into(),
        )
        .unwrap();
        assert!(nonlinear.admit().is_err());
        assert_eq!(nonlinear.equations.len(), 2);
        let linear = Assessment::new(
            Mode::NativeFeasibility,
            vec![id(1)],
            equations.clone(),
            witness.clone().into(),
        )
        .unwrap();
        linear.admit().unwrap();
        assert!(linear.optimization_freedom().is_none());
        assert!(pse_math::SharedAllocation::ptr_eq(
            &linear.witness,
            &nonlinear.witness
        ));
        let facts = crate::routing::oracle_facts(
            &OracleContract {
                identity: pse_ids::ContentHash::from_bytes([1; 32]),
                variables: vec![crate::Variable {
                    id: id(1),
                    lower: -1.,
                    upper: 1.,
                }],
                rows: vec![id(3), id(4)],
                derivatives: pse_kernels::DerivativeOrder::First,
                smoothness: pse_kernels::DerivativeOrder::First,
            },
            false,
            true,
        );
        for policy in [
            Policy::Equalities,
            Policy::NativeFeasibility,
            Policy::Factorable,
        ] {
            assert_eq!(
                mode(policy, &facts, crate::solve::SolveIntent::Root),
                Mode::Roots
            );
            assert!(
                Assessment::new(
                    mode(policy, &facts, crate::solve::SolveIntent::Root),
                    vec![id(1)],
                    equations.clone(),
                    witness.clone().into()
                )
                .unwrap()
                .admit()
                .is_err()
            );
        }
    }
    #[test]
    fn retained_assessment_projects_reordered_rows_and_refuses_duplicate_sources() {
        let contract = OracleContract {
            identity: pse_ids::ContentHash::from_bytes([3; 32]),
            variables: vec![
                crate::Variable {
                    id: id(1),
                    lower: -1.0,
                    upper: 1.0,
                },
                crate::Variable {
                    id: id(2),
                    lower: -1.0,
                    upper: 1.0,
                },
            ],
            rows: vec![id(3), id(4)],
            derivatives: pse_kernels::DerivativeOrder::First,
            smoothness: pse_kernels::DerivativeOrder::First,
        };
        let pattern = faer::sparse::SymbolicSparseColMat::try_new_from_indices(
            2,
            2,
            &[faer::sparse::Pair::new(0, 0), faer::sparse::Pair::new(1, 1)],
        )
        .unwrap()
        .0;
        let original = oracle_structure_with_cancel(
            &contract,
            pattern.as_ref(),
            &[(0.0, 0.0), (-1.0, 1.0)],
            true,
            &std::sync::atomic::AtomicBool::new(false),
        )
        .unwrap();
        let mut assessment = Assessment::new(
            Mode::NativeFeasibility,
            original.variables,
            original.equations,
            original.witness,
        )
        .unwrap();
        let mut reordered = contract.clone();
        reordered.rows.reverse();
        validate_assessment(&assessment, &reordered, &[(-1.0, 1.0), (3.0, 3.0)]).unwrap();
        assert!(validate_assessment(&assessment, &reordered, &[(3.0, 3.0), (-1.0, 1.0)]).is_err());
        assessment.equations[1].id = assessment.equations[0].id;
        assert!(validate_assessment(&assessment, &reordered, &[(-1.0, 1.0), (3.0, 3.0)]).is_err());
    }

    #[test]
    fn contextual_unit_original_structure_extent_cancellation_and_retained_validation() {
        let contract = OracleContract {
            identity: pse_ids::ContentHash::from_bytes([1; 32]),
            variables: vec![crate::Variable {
                id: id(1),
                lower: -1.0,
                upper: 1.0,
            }],
            rows: vec![id(2)],
            derivatives: pse_kernels::DerivativeOrder::First,
            smoothness: pse_kernels::DerivativeOrder::First,
        };
        let pattern = faer::sparse::SymbolicSparseColMat::try_new_from_indices(
            1,
            1,
            &[faer::sparse::Pair::new(0, 0)],
        )
        .unwrap()
        .0;
        let extent = construction_bytes(&contract, pattern.as_ref()).unwrap();
        let cancelled = std::sync::atomic::AtomicBool::new(true);
        assert!(matches!(
            oracle_structure_with_cancel(
                &contract,
                pattern.as_ref(),
                &[(0.0, 0.0)],
                true,
                &cancelled
            ),
            Err(ProblemError::Cancelled)
        ));
        let original = oracle_structure_with_cancel(
            &contract,
            pattern.as_ref(),
            &[(0.0, 0.0)],
            true,
            &std::sync::atomic::AtomicBool::new(false),
        )
        .unwrap();
        assert!(extent >= retained_bytes(&original));
        assert!(CaseIncidence::memory_extent(usize::MAX, 1, 1).is_err());
        let assessment = Assessment::new(
            Mode::Nlp,
            original.variables,
            original.equations,
            original.witness,
        )
        .unwrap();
        assert!(extent >= assessment.retained_bytes());
        validate_assessment(&assessment, &contract, &[(4.0, 4.0)]).unwrap();
        assert!(validate_assessment(&assessment, &contract, &[(-1.0, 1.0)]).is_err());
        let mut changed = contract.clone();
        changed.rows[0] = id(9);
        assert!(validate_assessment(&assessment, &changed, &[(0.0, 0.0)]).is_err());
        let cone = crate::ConicProblem {
            contract: contract.clone(),
            quadratic: crate::conic::SparseMatrix::zeros(1, 1),
            objective: vec![0.0],
            constraints: crate::conic::SparseMatrix::identity(1),
            rhs: vec![0.0],
            cones: vec![crate::conic::Cone::Zero { dimension: 1 }],
            objective_constant: 0.0,
        };
        assert!(matches!(
            conic_structure_with_cancel(&cone, &cancelled),
            Err(ProblemError::Cancelled)
        ));
        let structure =
            conic_structure_with_cancel(&cone, &std::sync::atomic::AtomicBool::new(false)).unwrap();
        assert!(conic_construction_bytes(&cone).unwrap() >= retained_bytes(&structure));
        assert_eq!(structure.witness.matching, [(id(2), id(1))]);
        let membership = crate::ConicProblem {
            cones: vec![crate::conic::Cone::Nonnegative { dimension: 1 }],
            ..cone
        };
        assert_eq!(
            conic_bounds(&membership).unwrap(),
            [(f64::NEG_INFINITY, f64::INFINITY)]
        );
    }
}
