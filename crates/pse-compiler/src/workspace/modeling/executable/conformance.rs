// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Authored expectation execution uses the existing guarded library pipeline without a runtime.
use super::*;
/// One source-attributed physical comparison.
#[derive(Clone, Debug, PartialEq)]
pub struct ModelingExpectationResult {
    pub id: SemanticId,
    pub declaration: SemanticId,
    pub actual: f64,
    pub expected: f64,
    /// Combined physical tolerance: absolute + relative * abs(expected).
    pub tolerance: f64,
    /// Authored canonical delta tolerance.
    pub absolute_tolerance: f64,
    /// Authored dimensionless relative tolerance.
    pub relative_tolerance: f64,
    pub passed: bool,
}
impl PreparedModeling {
    /// Select the physical and dimensionless observations for every authored test.
    pub fn expectation_rows(&self) -> BTreeSet<SemanticId> {
        self.admitted
            .outputs
            .iter()
            .filter(|o| matches!(o, ModelingOutput::Test { .. }))
            .map(ModelingOutput::row_id)
            .collect()
    }
    /// Compare original canonical physical values with a declared combined delta budget.
    pub fn assess_expectations(
        &self,
        observations: &BTreeMap<SemanticId, f64>,
    ) -> Result<Vec<ModelingExpectationResult>> {
        self.assess_expectations_for(
            observations,
            &self.model.expectations.keys().copied().collect(),
        )
    }
    /// Assess an explicit subset, requiring every component of each selected assertion.
    pub fn assess_expectations_for(
        &self,
        observations: &BTreeMap<SemanticId, f64>,
        selected: &BTreeSet<SemanticId>,
    ) -> Result<Vec<ModelingExpectationResult>> {
        if selected
            .iter()
            .any(|id| !self.model.expectations.contains_key(id))
        {
            return Err(CompileError::Missing("unknown selected expectation".into()));
        }
        self.model
            .expectations
            .values()
            .filter(|test| selected.contains(&test.id))
            .map(|test| {
                let value = |component| {
                    observations
                        .get(
                            &ModelingOutput::Test {
                                id: test.id,
                                component,
                            }
                            .row_id(),
                        )
                        .copied()
                        .filter(|v| v.is_finite())
                        .ok_or_else(|| {
                            CompileError::Missing("test observation absent or nonfinite".into())
                        })
                };
                let actual = value(ModelingTestValue::Actual)?;
                let expected = value(ModelingTestValue::Expected)?;
                let absolute_tolerance = value(ModelingTestValue::Tolerance)?;
                let relative_tolerance = value(ModelingTestValue::RelativeTolerance)?;
                if absolute_tolerance < 0.
                    || relative_tolerance < 0.
                    || absolute_tolerance == 0. && relative_tolerance == 0.
                {
                    return Err(CompileError::Missing(
                        "authored test requires nonnegative tolerances with at least one positive"
                            .into(),
                    ));
                }
                let tolerance = absolute_tolerance + relative_tolerance * expected.abs();
                if !tolerance.is_finite() {
                    return Err(CompileError::Missing(
                        "combined tolerance is nonfinite".into(),
                    ));
                }
                Ok(ModelingExpectationResult {
                    id: test.id,
                    declaration: test.lineage.declaration,
                    actual,
                    expected,
                    tolerance,
                    absolute_tolerance,
                    relative_tolerance,
                    passed: (actual - expected).abs() <= tolerance,
                })
            })
            .collect()
    }
}
impl CompilerWorkspace {
    /// Execute pure expectations without creating a runtime, acquiring solver resources
    /// or constructing native providers. Models needing a solve use the workflow harness.
    pub fn check_modeling_expectations(
        &mut self,
        root: SemanticId,
        instance: SemanticId,
        bindings: Bindings,
        limits: Limits,
        values: &CaseValues,
        profile: Profile,
        cancel: Arc<AtomicBool>,
    ) -> Result<Vec<ModelingExpectationResult>> {
        let model =
            self.prepare_modeling_cancellable(root, instance, bindings, limits, cancel.clone())?;
        if model.model.expectations.is_empty() {
            return Err(CompileError::Missing(
                "test scope contains no expectations".into(),
            ));
        }
        let fixture = model
            .model
            .fixtures
            .get(&instance)
            .map(ModelingCaseBindings::from)
            .unwrap_or_default();
        let mut inputs = model.case_values(&fixture)?;
        if values
            .scalars
            .keys()
            .any(|id| !model.admitted.inputs.contains(id))
        {
            return Err(CompileError::Missing(
                "test fixture names a non-input coordinate".into(),
            ));
        }
        inputs
            .scalars
            .extend(values.scalars.iter().map(|(id, v)| (*id, *v)));
        let prepared = self.prepare_modeling_observations(
            &model,
            &model.expectation_rows(),
            profile,
            &cancel,
        )?;
        if prepared
            .plan
            .bodies()
            .values()
            .any(|b| !b.providers().is_empty())
        {
            return Err(pse_modeling::ModelingError::Unsupported {
                declaration: root,
                capability: "pure tests cannot construct external or implicit runtime capabilities"
                    .into(),
            }
            .into());
        }
        let assembly = Arc::new(prepared.plan.compile(
            profile.optimization,
            profile.evaluation,
            &cancel,
        )?);
        let mut worker = assembly.worker(BTreeMap::new(), cancel);
        let output = worker.constraints(&inputs)?;
        let rows = prepared
            .plan
            .structure()
            .rows()
            .iter()
            .map(|r| r.id)
            .zip(output)
            .collect();
        model.assess_expectations(&rows)
    }
}
