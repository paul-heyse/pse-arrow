// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Physical-member observation uses the original compiler rows and the common binder.
use super::*;
use crate::workflow::{AdmittedBinding, BindingTarget, PreparationSettings, SelectedObservationOperation};
use pse_compiler::workspace::ModelingOutput;
use pse_math::binding::CaseValues;
use std::collections::BTreeSet;

/// One admitted pure observation operation. No residual row is exposed as an output.
#[derive(Clone, Debug)]
pub struct PreparedSelectedObservation {
    pub(crate) source: ModelingPackage,
    pub(crate) operation: SelectedObservationOperation,
    pub(crate) settings: PreparationSettings,
    pub(crate) model: ModelingPreparation,
    pub(crate) members: Vec<SemanticId>,
    pub(crate) values: CaseValues,
}

/// Original selected physical magnitudes in caller order.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SelectedObservationValues {
    /// Actual member identity, separate from the compiler row identity.
    pub members: Vec<SemanticId>,
    /// Canonical physical magnitudes; representation conversion belongs to admission.
    pub canonical: Vec<f64>,
    /// Original source applicability at this exact point.
    pub applicability: Vec<pse_model::applicability::Observation>,
}

/// Ordered pure-member population retaining the C batch's original point outcomes.
#[derive(Debug)]
pub struct SelectedObservationPopulation {
    /// Actual physical members, in requested output order.
    pub members: Vec<SemanticId>,
    /// Original shared batch; each physical member maps through `ModelingOutput::Member`.
    pub original: Arc<pse_columnar::Leased<pse_math::assembly::CaseBatchOutcome>>,
}
/// A batch-owned original occurrence, consumable without running its mathematics twice.
#[derive(Debug)]
pub struct CompletedSelectedObservation {
    pub(crate) request: Box<PreparedSelectedObservation>,
    pub(crate) result: Result<Box<SelectedObservationValues>, Arc<WorkflowError>>,
    pub(crate) _population: Arc<pse_columnar::Leased<pse_math::assembly::CaseBatchOutcome>>,
}
impl SelectedObservationPopulation {
    /// Canonical physical values only when this original point succeeded.
    pub fn canonical(&self, point: usize) -> Option<Vec<f64>> {
        let values = self.original.row(point)?;
        self.members.iter().map(|member| {
            let row = ModelingOutput::Member(*member).row_id();
            self.original.rows().iter().position(|id| *id == row).and_then(|index| values.get(index).copied())
        }).collect()
    }
}

impl ModelingPackage {
    /// Check requested scalar physical meaning against actual authored member support.
    pub async fn prepare_selected_observation(
        &self,
        services: &crate::workflow::PreparationServices,
        operation: &SelectedObservationOperation,
        settings: &PreparationSettings,
        binding: Option<&AdmittedBinding>,
        cancel: &crate::CancelSource,
    ) -> Result<PreparedSelectedObservation, WorkflowError> {
        self.check_service_bindings(services)?;
        if operation.outputs.is_empty() || operation.outputs.len() > 1024 {
            return Err(contract("selected observation requires bounded nonempty outputs"));
        }
        let mut bindings = operation.bindings.clone();
        bindings.demand.extend(operation.outputs.iter().filter_map(|output| match &output.member {
            BindingTarget::Path(path) => Some(path.clone()),
            BindingTarget::Member(_) => None,
        }));
        let model = self.prepare(services, operation.root, operation.instance, bindings, settings.limits, cancel).await?;
        let mut members = Vec::with_capacity(operation.outputs.len());
        let mut unique = BTreeSet::new();
        for output in &operation.outputs {
            let member = match &output.member {
                BindingTarget::Member(member) => *member,
                BindingTarget::Path(path) => *model.compiled().model.paths.get(path).ok_or_else(|| contract(format!("selected output {path} absent")))?,
            };
            let symbol = model.compiled().model.symbols.get(&member).ok_or_else(|| contract("selected output is not an authored physical member"))?;
            let expected = symbol.ty.quantity_scheme().ok_or_else(|| contract("selected output has no physical type"))?
                .resolve_contract_with_evidence(&self.quantities, &BTreeMap::new(), self.physical.preconditions.as_ref()).map_err(|error| WorkflowError::from(pse_model::diagnostic::project_typed(&error, pse_diagnostics::DiagnosticStage::StudyBinding)))?;
            let supplied = pse_quantity::ResolvedPhysicalContract::named(output.quantity, pse_quantity::IndexSet::new(), &self.quantities)
                .map_err(|error| crate::workflow::math(pse_math::MathError::Quantity(error)))?;
            if !expected.axes().is_empty() || !expected.same_meaning(&supplied) || !unique.insert(member) {
                return Err(contract("selected output scalar, quantity or unique membership differs"));
            }
            pse_quantity::CanonicalConversionPlan::registered(&self.quantities, output.quantity, output.unit)
                .map_err(|error| crate::workflow::math(pse_math::MathError::Quantity(error)))?;
            let row = ModelingOutput::Member(member).row_id();
            if !model.compiled().admitted.outputs.iter().any(|output| output.row_id() == row) {
                return Err(contract("selected physical member was not demanded from original source"));
            }
            members.push(member);
        }
        let mut values = model.compiled().case_values(&Default::default()).map_err(crate::math::MathRuntimeError::from)?;
        if let Some(binding) = binding {
            self.validate_binding(model.compiled(), binding)?;
            values.scalars.extend(binding.values());
        }
        Ok(PreparedSelectedObservation { source: self.clone(), operation: operation.clone(), settings: settings.clone(), model, members, values })
    }
    /// Evaluate a previously admitted physical point through the shared observation owner.
    pub async fn observe_selected(
        &self,
        services: &crate::workflow::PreparationServices,
        operation: &SelectedObservationOperation,
        binding: Option<&AdmittedBinding>,
        settings: &PreparationSettings,
        cancel: &crate::CancelSource,
    ) -> Result<SelectedObservationValues, WorkflowError> {
        self.prepare_selected_observation(services, operation, settings, binding, cancel).await?.evaluate(services, cancel).await
    }
}
impl PreparedSelectedObservation {
    /// Shared immutable mathematics and physical meaning, excluding fresh point values.
    pub fn population_key(&self) -> Result<pse_ids::ContentHash, WorkflowError> {
        pse_ids::document::of(pse_ids::Frame::SelectedObservationV1, &("population", crate::workflow::OperationSource::of(&self.source), &self.operation, &self.settings))
            .map_err(|error| contract(error.to_string()))
    }
    /// Execute one eligible population once, issuing original per-occurrence completions.
    pub async fn evaluate_population(prepared: &[Self], services: &crate::workflow::PreparationServices, cancel: &crate::CancelSource) -> Result<Vec<CompletedSelectedObservation>, WorkflowError> {
        let first = prepared.first().ok_or_else(|| contract("selected observation population is empty"))?;
        if prepared.len() > crate::workflow::MAXIMUM_STUDY_POINTS { return Err(contract("selected observation population extent")); }
        let key = first.population_key()?;
        if prepared.iter().any(|point| point.population_key().ok() != Some(key)) { return Err(contract("selected observation population has different original mathematics")); }
        let rows = first.members.iter().map(|member| ModelingOutput::Member(*member).row_id()).collect();
        let points = prepared.iter().enumerate().map(|(index, point)| {
            let mut bytes = [0;16]; bytes[..8].copy_from_slice(&(index as u64).to_le_bytes());
            (SemanticId::from_bytes(bytes), point.values.clone())
        }).collect();
        let original = first.source.observe_batch(services, first.model.clone(), rows, points, first.settings.compiler, cancel).await?;
        let population = SelectedObservationPopulation { members: first.members.clone(), original };
        let mut completed = Vec::with_capacity(prepared.len());
        for (index, request) in prepared.iter().enumerate() {
            let point = &population.original.points()[index];
            let result = match &point.outcome {
                pse_math::batch::PointOutcome::Succeeded => population.canonical(index).map(|canonical| Box::new(SelectedObservationValues { members: request.members.clone(), canonical, applicability: point.applicability.clone() })).ok_or_else(|| Arc::new(contract("successful selected population lacks original values"))),
                pse_math::batch::PointOutcome::Failed(error) => Err(Arc::new(WorkflowError::from(pse_model::diagnostic::project_typed(error, pse_diagnostics::DiagnosticStage::Evaluation)))),
                pse_math::batch::PointOutcome::Canceled | pse_math::batch::PointOutcome::NotExecuted => Err(Arc::new(crate::math::MathRuntimeError::Cancelled.into())),
            };
            completed.push(CompletedSelectedObservation { request: Box::new(request.clone()), result, _population: population.original.clone() });
        }
        Ok(completed)
    }
    /// Content identity includes exact source, physical context, settings and actual values.
    pub fn identity(&self) -> Result<pse_ids::ContentHash, WorkflowError> {
        pse_ids::document::of(pse_ids::Frame::SelectedObservationV1, &(crate::workflow::OperationSource::of(&self.source), &self.operation, &self.settings, &self.values.scalars))
            .map_err(|error| contract(error.to_string()))
    }
    /// Evaluate original rows, retaining guards and per-point applicability.
    pub async fn evaluate(&self, services: &crate::workflow::PreparationServices, cancel: &crate::CancelSource) -> Result<SelectedObservationValues, WorkflowError> {
        let rows = self.members.iter().map(|member| ModelingOutput::Member(*member).row_id()).collect();
        let observed = self.source.observe(services, self.model.clone(), rows, self.values.clone(), self.settings.compiler, cancel).await?;
        let canonical = self.members.iter().map(|member| observed.get(&ModelingOutput::Member(*member).row_id()).copied().filter(|value| value.is_finite()).ok_or_else(|| contract("selected physical output unavailable"))).collect::<Result<Vec<_>, _>>()?;
        Ok(SelectedObservationValues { members: self.members.clone(), canonical, applicability: observed.applicability.clone() })
    }
    /// Execute independent admitted points through the shared pure C batch owner.
    pub async fn evaluate_batch(&self, services: &crate::workflow::PreparationServices, points: Vec<(SemanticId, AdmittedBinding)>, cancel: &crate::CancelSource) -> Result<SelectedObservationPopulation, WorkflowError> {
        let mut values = Vec::with_capacity(points.len());
        for (point, binding) in points {
            self.source.validate_binding(self.model.compiled(), &binding)?;
            let mut selected = self.values.clone();
            selected.scalars.extend(binding.values());
            values.push((point, selected));
        }
        let rows = self.members.iter().map(|member| ModelingOutput::Member(*member).row_id()).collect();
        let original = self.source.observe_batch(services, self.model.clone(), rows, values, self.settings.compiler, cancel).await?;
        Ok(SelectedObservationPopulation { members: self.members.clone(), original })
    }
}
