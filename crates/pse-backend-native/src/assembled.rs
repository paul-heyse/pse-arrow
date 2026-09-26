// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Direct adapters over admitted, library-derived case products.
use crate::{CoefficientProblem, NleOracle, NlpOracle, OracleContract, ProblemError, Variable};
use pse_kernels::DerivativeOrder;
use pse_math::{
    assembly::{CasePlan, CaseWorker},
    binding::{CaseValues, VariableDomain},
    coefficients::Coefficients,
};

/// Attempt-owned algebraic oracle; fixed values and parameters cannot change structure.
#[derive(Debug)]
pub struct AlgebraicOracle {
    worker: CaseWorker,
    values: CaseValues,
    contract: OracleContract,
    bounds: Vec<(f64, f64)>,
    facts: crate::DerivativeFacts,
    normalization: Option<pse_math::normalization::Normalization>,
    presolve: Option<std::sync::Arc<pse_math::presolve::Facts>>,
    structure: Option<std::sync::Arc<pse_structural::incidence::StructuralAnalysis>>,
}
impl AlgebraicOracle {
    /// Admit continuous NLP/NLE variables. All-fixed cases stay on constant evaluation.
    pub fn new(worker: CaseWorker, values: CaseValues) -> Result<Self, ProblemError> {
        let assembly = worker.assembly();
        assembly.structure().validate_values(&values)?;
        if assembly
            .structure()
            .variables()
            .iter()
            .any(|v| !v.fixed && v.domain != VariableDomain::Continuous)
        {
            return Err(ProblemError::Contract(
                "continuous oracle cannot admit integer variables".into(),
            ));
        }
        let contract = contract(assembly);
        contract.validate(assembly.order())?;
        let bounds = assembly
            .structure()
            .rows()
            .iter()
            .map(|r| (r.lower, r.upper))
            .collect();
        Ok(Self {
            worker,
            values,
            contract,
            bounds,
            facts: Default::default(),
            normalization: None,
            presolve: None,
            structure: None,
        })
    }
    /// Attach a compiler analysis. Native admission checks its complete matching
    /// witness against the current IDs, equality classification and sparse edges.
    pub fn with_structural_analysis(
        mut self,
        analysis: std::sync::Arc<pse_structural::incidence::StructuralAnalysis>,
    ) -> Self {
        self.structure = Some(analysis);
        self
    }
    /// Attach the immutable resolved coordinate projection before native preparation.
    pub fn with_normalization(
        mut self,
        normalization: pse_math::normalization::Normalization,
    ) -> Result<Self, ProblemError> {
        normalization.validate(self.contract.variables.len(), self.contract.rows.len())?;
        self.normalization = Some(normalization);
        Ok(self)
    }
    /// Attach library-proved coefficient facts only for the exact current structure
    /// and fixed/parameter values. Replacing values clears these assumptions.
    pub fn with_coefficient_facts(mut self, c: &Coefficients) -> Result<Self, ProblemError> {
        if c.structure() != self.worker.assembly().structure().key()
            || !c.matches_values(&self.values)
        {
            return Err(ProblemError::Contract(
                "stale derivative coefficient proof".into(),
            ));
        }
        self.facts = crate::DerivativeFacts {
            gradient_constant: c.hessian.val().iter().all(|v| *v == 0.0),
            jacobian_constant: true,
            hessian_constant: true,
        };
        Ok(self)
    }
    /// Attach compiler facts only when their complete assumptions still match.
    pub fn with_presolve_facts(
        mut self,
        facts: std::sync::Arc<pse_math::presolve::Facts>,
    ) -> Result<Self, ProblemError> {
        if !facts.matches(self.worker.assembly(), &self.values) {
            return Err(ProblemError::Contract("stale presolve facts".into()));
        }
        self.presolve = Some(facts);
        Ok(self)
    }
    /// Root-system view additionally requires equality rows and no optimization objective.
    pub fn admit_nle(&self) -> Result<(), ProblemError> {
        self.contract.square()?;
        if self.worker.assembly().structure().objective().is_some()
            || self.bounds.iter().any(|(l, u)| !l.is_finite() || l != u)
        {
            return Err(ProblemError::Contract(
                "NLE requires selected finite equalities and no objective".into(),
            ));
        }
        Ok(())
    }
    /// Updating parameters invalidates numerical caches by complete local input identity.
    pub fn update_values(&mut self, values: CaseValues) -> Result<(), ProblemError> {
        self.worker
            .assembly()
            .structure()
            .validate_values(&values)?;
        self.facts = Default::default();
        self.presolve = None;
        self.values = values;
        Ok(())
    }
    fn trial(&mut self, x: &[f64]) -> Result<(), ProblemError> {
        if x.len() != self.contract.variables.len() || x.iter().any(|v| !v.is_finite()) {
            return Err(ProblemError::Contract(
                "native trial dimensions or values".into(),
            ));
        }
        for (v, &x) in self.contract.variables.iter().zip(x) {
            self.values.scalars.insert(v.id, x);
        }
        Ok(())
    }
}
fn copy(source: &[f64], target: &mut [f64]) -> Result<(), ProblemError> {
    if source.len() != target.len() {
        return Err(ProblemError::Contract("native output dimensions".into()));
    }
    target.copy_from_slice(source);
    Ok(())
}
impl NlpOracle for AlgebraicOracle {
    fn structural_analysis(&self) -> Option<&pse_structural::incidence::StructuralAnalysis> {
        self.structure.as_deref()
    }
    fn normalization(&self) -> Option<&pse_math::normalization::Normalization> {
        self.normalization.as_ref()
    }
    fn constraint_sources(&self) -> Result<Vec<pse_math::assembly::OutputValue>, ProblemError> {
        Ok(self.worker.constraint_sources()?)
    }
    fn presolve_facts(&self) -> Option<&pse_math::presolve::Facts> {
        self.presolve.as_deref()
    }
    fn derivative_facts(&self) -> crate::DerivativeFacts {
        self.facts
    }
    fn contract(&self) -> &OracleContract {
        &self.contract
    }
    fn jacobian_pattern(&self) -> faer::sparse::SymbolicSparseColMatRef<'_, usize> {
        self.worker.assembly().jacobian_pattern()
    }
    fn hessian_pattern(&self) -> Option<faer::sparse::SymbolicSparseColMatRef<'_, usize>> {
        (self.contract.derivatives >= DerivativeOrder::Second)
            .then(|| self.worker.assembly().hessian_pattern())
    }
    fn constraint_bounds(&self) -> &[(f64, f64)] {
        &self.bounds
    }
    fn objective(&mut self, x: &[f64]) -> Result<f64, ProblemError> {
        self.trial(x)?;
        Ok(self.worker.objective(&self.values)?)
    }
    fn constraints(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        self.trial(x)?;
        copy(&self.worker.constraints(&self.values)?, out)
    }
    fn gradient(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        self.trial(x)?;
        copy(&self.worker.gradient(&self.values)?, out)
    }
    fn jacobian(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        self.trial(x)?;
        copy(self.worker.jacobian(&self.values)?.val(), out)
    }
    fn hessian(
        &mut self,
        x: &[f64],
        objective_weight: f64,
        multipliers: &[f64],
        out: &mut [f64],
    ) -> Result<(), ProblemError> {
        self.trial(x)?;
        copy(
            self.worker
                .hessian(&self.values, objective_weight, multipliers)?
                .val(),
            out,
        )
    }
}
impl NleOracle for AlgebraicOracle {
    fn structural_analysis(&self) -> Option<&pse_structural::incidence::StructuralAnalysis> {
        self.structure.as_deref()
    }
    fn guard_signs(
        &self,
    ) -> std::collections::BTreeMap<pse_ids::SemanticId, pse_math::presolve::GuardSign> {
        self.presolve
            .as_ref()
            .map_or_else(Default::default, |p| p.signs.clone())
    }
    fn observe(&self, mut residual: Vec<f64>) -> Result<crate::quality::Observation, ProblemError> {
        self.admit_nle()?;
        if residual.len() != self.bounds.len() {
            return Err(ProblemError::Contract("root observation dimensions".into()));
        }
        for (v, (l, _)) in residual.iter_mut().zip(&self.bounds) {
            *v += l;
        }
        let mut observation =
            crate::quality::Observation::from_values(None, residual, self.bounds.clone())?;
        observation.sources = self.worker.constraint_sources()?;
        Ok(observation)
    }
    fn contract(&self) -> &OracleContract {
        &self.contract
    }
    fn jacobian_pattern(&self) -> faer::sparse::SymbolicSparseColMatRef<'_, usize> {
        self.worker.assembly().jacobian_pattern()
    }
    fn jacobian(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        self.admit_nle()?;
        NlpOracle::jacobian(self, x, out)
    }
    fn residual(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        self.admit_nle()?;
        self.trial(x)?;
        let mut values = self.worker.constraints(&self.values)?;
        for (v, (l, _)) in values.iter_mut().zip(&self.bounds) {
            *v -= l;
        }
        copy(&values, out)
    }
    fn jacobian_product(
        &mut self,
        x: &[f64],
        direction: &[f64],
        out: &mut [f64],
    ) -> Result<(), ProblemError> {
        self.admit_nle()?;
        self.trial(x)?;
        let mut result = vec![0.0; out.len()];
        self.worker
            .jacobian_product(&self.values, direction, &mut result)?;
        copy(&result, out)
    }
}
/// Native contract preserves every selected variable/row, including isolated vertices.
pub fn contract(assembly: &CasePlan) -> OracleContract {
    OracleContract {
        identity: assembly.structure().key(),
        variables: assembly
            .structure()
            .variables()
            .iter()
            .filter(|v| !v.fixed)
            .map(|v| Variable {
                id: v.port.id,
                lower: if v.domain == VariableDomain::Binary {
                    v.lower.unwrap_or(0.0).max(0.0)
                } else {
                    v.lower.unwrap_or(f64::NEG_INFINITY)
                },
                upper: if v.domain == VariableDomain::Binary {
                    v.upper.unwrap_or(1.0).min(1.0)
                } else {
                    v.upper.unwrap_or(f64::INFINITY)
                },
            })
            .collect(),
        rows: assembly.structure().rows().iter().map(|r| r.id).collect(),
        derivatives: assembly.order(),
        smoothness: assembly.available_order(),
    }
}
impl CoefficientProblem {
    /// Materialize a class view with parameter-sensitive identity and shifted row bounds.
    pub fn from_plan(
        assembly: &CasePlan,
        coefficients: Coefficients,
    ) -> Result<Self, ProblemError> {
        if coefficients.structure() != assembly.structure().key() {
            return Err(ProblemError::Contract(
                "coefficient snapshot belongs to another structure".into(),
            ));
        }
        let problem = Self {
            contract: contract(assembly),
            objective: coefficients.objective,
            objective_constant: coefficients.objective_constant,
            sense: assembly
                .structure()
                .objective()
                .map_or(pse_math::binding::ObjectiveSense::Minimize, |o| o.sense),
            domains: assembly
                .structure()
                .variables()
                .iter()
                .filter(|v| !v.fixed)
                .map(|v| v.domain)
                .collect(),
            assumptions: coefficients.assumptions,
            constraints: coefficients.constraints,
            hessian: Some(coefficients.hessian),
            bounds: assembly
                .structure()
                .rows()
                .iter()
                .zip(coefficients.row_constants)
                .map(|(r, c)| {
                    let shifted = (r.lower - c, r.upper - c);
                    if r.lower.is_finite() && !shifted.0.is_finite()
                        || r.upper.is_finite() && !shifted.1.is_finite()
                    {
                        return Err(ProblemError::Contract(
                            "affine coefficient bound shift overflow".into(),
                        ));
                    }
                    Ok(shifted)
                })
                .collect::<Result<_, ProblemError>>()?,
        };
        problem.validate()?;
        Ok(problem)
    }
}

/// Explicit feasibility representation: the native objective and all of its
/// derivatives are identically zero, while original constraints are unchanged.
#[derive(Debug)]
pub struct FeasibilityOracle(pub Box<dyn NlpOracle>);
impl NlpOracle for FeasibilityOracle {
    fn structural_analysis(&self) -> Option<&pse_structural::incidence::StructuralAnalysis> {
        self.0.structural_analysis()
    }
    fn normalization(&self) -> Option<&pse_math::normalization::Normalization> {
        self.0.normalization()
    }
    fn constraint_sources(&self) -> Result<Vec<pse_math::assembly::OutputValue>, ProblemError> {
        self.0.constraint_sources()
    }
    fn presolve_facts(&self) -> Option<&pse_math::presolve::Facts> {
        self.0.presolve_facts()
    }
    fn derivative_facts(&self) -> crate::DerivativeFacts {
        let mut f = self.0.derivative_facts();
        f.gradient_constant = true;
        f.hessian_constant = f.jacobian_constant;
        f
    }
    fn contract(&self) -> &OracleContract {
        self.0.contract()
    }
    fn jacobian_pattern(&self) -> faer::sparse::SymbolicSparseColMatRef<'_, usize> {
        self.0.jacobian_pattern()
    }
    fn hessian_pattern(&self) -> Option<faer::sparse::SymbolicSparseColMatRef<'_, usize>> {
        self.0.hessian_pattern()
    }
    fn constraint_bounds(&self) -> &[(f64, f64)] {
        self.0.constraint_bounds()
    }
    fn objective(&mut self, x: &[f64]) -> Result<f64, ProblemError> {
        if x.len() != self.contract().variables.len() || x.iter().any(|v| !v.is_finite()) {
            return Err(ProblemError::Contract(
                "feasibility trial dimensions/values".into(),
            ));
        }
        Ok(0.0)
    }
    fn gradient(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        self.objective(x)?;
        if out.len() != x.len() {
            return Err(ProblemError::Contract(
                "feasibility gradient dimensions".into(),
            ));
        }
        out.fill(0.0);
        Ok(())
    }
    fn constraints(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        self.0.constraints(x, out)
    }
    fn jacobian(&mut self, x: &[f64], out: &mut [f64]) -> Result<(), ProblemError> {
        self.0.jacobian(x, out)
    }
    fn hessian(
        &mut self,
        x: &[f64],
        _weight: f64,
        lambda: &[f64],
        out: &mut [f64],
    ) -> Result<(), ProblemError> {
        self.0.hessian(x, 0.0, lambda, out)
    }
}
