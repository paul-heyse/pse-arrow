// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Authored expectation execution uses the existing guarded library pipeline without a runtime.
use super::*;
/// One source-attributed physical comparison.
#[derive(Clone, Debug, PartialEq)]
pub struct ModelingExpectationResult {
    /// Source test identity.
    pub id: SemanticId,
    /// Declaration that authored the test.
    pub declaration: DeclarationId,
    /// Observed canonical value.
    pub actual: f64,
    /// Expected canonical value.
    pub expected: f64,
    /// Combined physical tolerance: absolute + relative * abs(expected).
    pub tolerance: f64,
    /// Authored canonical delta tolerance.
    pub absolute_tolerance: f64,
    /// Authored dimensionless relative tolerance.
    pub relative_tolerance: f64,
    /// Whether the absolute difference is within the combined tolerance.
    pub passed: bool,
}
/// An annotated hard range observed at a pure point, retaining its member, declaration,
/// owning layer and canonical value and bounds.
#[derive(Clone, Debug, PartialEq)]
pub struct ModelingValidityResult {
    /// The bounded member.
    pub target: SemanticId,
    /// The annotation declaring the range.
    pub source: DeclarationId,
    /// The range's owning validity layer.
    pub layer: pse_model::generated::enums::ModelingValidityLayer,
    /// Observed canonical value.
    pub value: f64,
    /// Canonical lower bound.
    pub lower: f64,
    /// Canonical upper bound.
    pub upper: f64,
}
impl ModelingValidityResult {
    /// Whether the value lies within its bounds.
    pub fn within(&self) -> bool {
        self.lower <= self.value && self.value <= self.upper
    }
}
/// What one pure point establishes (Plan 23 H5): its authored expectations and the same
/// hard validity obligations a solved point is assessed against. Invalid domains refuse
/// evaluation; scientific applicability observations remain a separate typed product.
#[derive(Clone, Debug, PartialEq)]
pub struct ModelingPointChecks {
    /// Authored comparisons.
    pub expectations: Vec<ModelingExpectationResult>,
    /// Observed annotated hard ranges.
    pub validity: Vec<ModelingValidityResult>,
    /// Demanded scientific evidence, with actual physical inputs and scoped permission lineage.
    pub applicability: Vec<pse_model::applicability::Observation>,
}
impl PreparedModeling {
    /// The member and bound rows of every validity range and data observation.
    pub fn validity_rows(&self) -> BTreeSet<SemanticId> {
        self.model
            .annotations
            .iter()
            .filter(|a| {
                matches!(
                    a.value,
                    pse_modeling::annotation::AnnotationValue::Valid { .. }
                )
            })
            .flat_map(|a| {
                [
                    ModelingOutput::Member(a.target),
                    ModelingOutput::Hint {
                        target: a.target,
                        declaration: a.lineage.declaration,
                        kind: ModelingHint::ValidLower,
                    },
                    ModelingOutput::Hint {
                        target: a.target,
                        declaration: a.lineage.declaration,
                        kind: ModelingHint::ValidUpper,
                    },
                ]
            })
            .map(|output| output.row_id())
            .collect()
    }
    /// Assess every validity range and data observation at the observed point, and every
    /// static observation decided at specialization.
    pub fn assess_validity(
        &self,
        observations: &BTreeMap<SemanticId, f64>,
    ) -> Result<Vec<ModelingValidityResult>> {
        let observed = |output: ModelingOutput| {
            observations
                .get(&output.row_id())
                .copied()
                .filter(|v| v.is_finite())
                .ok_or_else(|| {
                    CompileError::Missing("validity observation absent or nonfinite".into())
                })
        };
        let mut results = Vec::new();
        for a in &self.model.annotations {
            let pse_modeling::annotation::AnnotationValue::Valid { layer, .. } = &a.value else {
                continue;
            };
            let bound = |kind| ModelingOutput::Hint {
                target: a.target,
                declaration: a.lineage.declaration,
                kind,
            };
            results.push(ModelingValidityResult {
                target: a.target,
                source: a.lineage.declaration,
                layer: *layer,
                value: observed(ModelingOutput::Member(a.target))?,
                lower: observed(bound(ModelingHint::ValidLower))?,
                upper: observed(bound(ModelingHint::ValidUpper))?,
            });
        }
        Ok(results)
    }
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
    /// Execute a pure point without creating a runtime, acquiring solver resources or
    /// constructing native providers: its expectations and the validity obligations a solved
    /// point is assessed against (Plan 23 H5). Models needing a solve use the workflow
    /// harness.
    #[expect(
        clippy::too_many_arguments,
        reason = "the specialization request (root, instance, bindings, limits) travels with the case, profiles and cancellation as independent inputs"
    )]
    pub fn check_modeling_point(
        &mut self,
        root: DeclarationId,
        instance: InstanceId,
        bindings: Bindings,
        limits: Limits,
        values: &CaseValues,
        profile: Profile,
        cancel: Arc<AtomicBool>,
    ) -> Result<ModelingPointChecks> {
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
        let mut observed = model.expectation_rows();
        observed.extend(model.validity_rows());
        let prepared = self.prepare_modeling_observations(&model, &observed, profile, &cancel)?;
        if prepared
            .plan
            .bodies()
            .values()
            .any(|b| !b.providers().is_empty())
        {
            return Err(pse_modeling::ModelingError::Unsupported {
                declaration: root.into(),
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
        Ok(ModelingPointChecks {
            expectations: model.assess_expectations(&rows)?,
            validity: model.assess_validity(&rows)?,
            applicability: worker.applicability_observations(),
        })
    }
}
