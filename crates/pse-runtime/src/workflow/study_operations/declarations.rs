// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
// Actual source declarations; original native methods and admission remain beside the owner.

pse_contract::projection_struct! {
/// One authored scalar output and its requested physical representation.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SelectedObservationOutput {
    projection: { name: "SelectedObservationOutput", rust_path: crate::workflow::SelectedObservationOutput, version: 1,
        direction: pse_contract::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };
    /// Contextually resolved member in the original source; never a formula string.
    pub member: crate::workflow::BindingTarget => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Required output quantity, checked against actual inferred source support.
    pub quantity: pse_quantity::QuantityTypeId => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Registered output representation unit.
    pub unit: pse_quantity::UnitId => { tag: 3, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
}
}
pse_contract::projection_struct! {
/// Original physically admitted scalar observations, distinct from equation residuals and solves.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SelectedObservationOperation {
    projection: { name: "SelectedObservationOperation", rust_path: crate::workflow::SelectedObservationOperation, version: 1,
        direction: pse_contract::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };
    /// Original selected source definition.
    pub root: DeclarationId => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Original specialized occurrence.
    pub instance: pse_model::generated::identities::InstanceId => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Structural inputs; numerical point values use the ordinary binding owner.
    pub bindings: pse_modeling::Bindings => { tag: 3, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Ordered actual physical scalar outputs.
    pub outputs: Vec<SelectedObservationOutput> => { tag: 4, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
}
}

pse_contract::projection_struct! {
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
/// One existing authored solve and its native settings document.
pub struct CaseOperation {
    projection: { name: "CaseOperation", rust_path: crate::workflow::CaseOperation, version: 1,
        direction: pse_contract::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };
    /// Authored case identity.
    pub case: DeclarationId => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Route captured from the declaration's admission.
    pub route: ModelingAnalysisRoute => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Complete existing solve settings.
    pub settings: SolveSettings => { tag: 3, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
}
}

pse_contract::projection_struct! {
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
/// One authored integration with optional pending physical controls.
pub struct SimulationOperation {
    projection: { name: "SimulationOperation", rust_path: crate::workflow::SimulationOperation, version: 1,
        direction: pse_contract::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };
    /// Authored integration case.
    pub case: DeclarationId => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Pending physical integration controls; absence requests authored controls.
    pub settings: Option<super::PhysicalSimulationRequest> => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
}
}

pse_contract::projection_struct! {
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
/// One pinned authored fit and its pending physical experiment controls.
pub struct FitOperation {
    projection: { name: "FitOperation", rust_path: crate::workflow::FitOperation, version: 1,
        direction: pse_contract::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };
    /// Existing authored fit identity.
    pub fit: FitId => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Pending physical experiment controls, admitted against each actual experiment.
    pub settings: super::PhysicalFitRequest => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
}
}

pse_contract::projection_struct! {
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
/// Original controller signal, move and advanced-step mapping intents.
pub struct ControllerOperation {
    projection: { name: "ControllerOperation", rust_path: crate::workflow::ControllerOperation, version: 1,
        direction: pse_contract::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };
    /// Existing simultaneous controller case.
    pub case: CaseOperation => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Existing signal binding intent.
    pub bindings: Vec<HorizonBinding> => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Existing move path/input-index mapping.
    pub moves: Vec<ControllerMove> => { tag: 3, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Existing advanced-step prediction mapping; absence means full solves.
    pub predictions: Option<Vec<ControllerPrediction>> => { tag: 4, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
}
}

pse_contract::projection_struct! {
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
/// Original estimator window, measurement, input and arrival mapping intents.
pub struct EstimatorOperation {
    projection: { name: "EstimatorOperation", rust_path: crate::workflow::EstimatorOperation, version: 1,
        direction: pse_contract::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };
    /// Existing simultaneous estimator case.
    pub case: CaseOperation => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Existing estimation window.
    pub window: usize => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Existing measurement-to-case-path mapping.
    pub measurements: Vec<EstimatorMeasurement> => { tag: 3, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Existing driven-input mapping.
    pub inputs: Vec<EstimatorInput> => { tag: 4, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Existing arrival prior mapping.
    pub arrival: Vec<ArrivalDocument> => { tag: 5, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
}
}

pse_contract::projection_struct! {
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
/// Reconstructable closed-loop operation in one pinned source context.
pub struct HorizonOperation {
    projection: { name: "HorizonOperation", rust_path: crate::workflow::HorizonOperation, version: 1,
        direction: pse_contract::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };
    /// Existing authored plant simulation.
    pub plant: SimulationOperation => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Existing positive physical sample period in seconds.
    pub period: pse_model::scalars::Tolerance => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Existing closed-loop sample count.
    pub steps: PositiveCount => { tag: 3, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Driven plant inputs.
    pub inputs: Vec<HorizonInputDocument> => { tag: 4, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Optional existing estimator.
    pub estimator: Option<EstimatorOperation> => { tag: 5, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Optional existing controller.
    pub controller: Option<ControllerOperation> => { tag: 6, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
}
}

pse_contract::projection_struct! {
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
/// Canonical assignments admitted against the original physical context.
pub struct AdmittedHorizonValues {
    projection: { name: "AdmittedHorizonValues", rust_path: crate::workflow::AdmittedHorizonValues, version: 1,
        direction: pse_contract::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };
    /// Canonical plant parameter assignments in declared input order.
    pub inputs: Vec<super::AdmittedBindingEntry> => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Canonical estimator priors in declared arrival order.
    pub arrival: Vec<super::AdmittedBindingEntry> => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Canonical controller trajectory values by declared binding index.
    pub trajectories: BTreeMap<usize, Vec<super::AdmittedBindingEntry>> => { tag: 3, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
}
}

pse_contract::projection_struct! {
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
/// Immutable Version8 operation descriptor recorded by an original study occurrence.
pub struct StudyOperation {
    projection: { name: "StudyOperation", rust_path: crate::workflow::StudyOperation, version: 1,
        direction: pse_contract::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };
    /// Interpretation of this operation descriptor.
    pub version: Version<8> => { tag: 1, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Explicit occurrence detail retained independently of original completion.
    pub output: super::StudyOutputDemand => { tag: 2, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Exact source/context reconstruction precondition.
    pub source: OperationSource => { tag: 3, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Complete preparation controls using existing owners' remote projections.
    pub preparation: PreparationSettings => { tag: 4, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Existing operation-owned input document.
    pub operation: OperationRequest => { tag: 5, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
    /// Canonical physical assignments; required for every horizon descriptor.
    pub admitted_horizon: Option<AdmittedHorizonValues> => { tag: 6, direction: pse_contract::boundary_projection::Direction::Both, rules: [] };
}
}

pse_contract::projection_enum! {
    /// Supported operation owner selection; serialized host Shooting is not a native family.
    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
    #[serde(tag = "kind", content = "request", rename_all = "snake_case", deny_unknown_fields)]
    pub enum OperationRequest {
        projection: { name: "OperationRequest", rust_path: crate::workflow::OperationRequest, version: 1,
            direction: pse_contract::boundary_projection::Direction::Both, reserved_tags: [], reserved_names: [] };
        /// Existing authored algebraic solve owner.
        DeclaredCase(CaseOperation) => 1;
        /// Existing authored integration owner.
        Simulation(SimulationOperation) => 2;
        /// Existing shared-parameter fitting owner.
        Fit(FitOperation) => 3;
        /// Existing closed-loop horizon owner.
        Horizon(Box<HorizonOperation>) => 4;
        /// Original physically typed selected scalar observations.
        SelectedObservation(SelectedObservationOperation) => 5;
    }
}
