// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Source-owned physical law, estimator controls and retained interpretation.
use super::super::{
    BindingQuantity, BindingTarget, CaseOperation, OperationSource, PreparationSettings,
    SelectedObservationOperation,
};
use pse_contract::boundary_projection::Direction;
use pse_ids::SemanticId;
use pse_quantity::{QuantityTypeId, UnitId};

pse_contract::projection_enum! {
/// Original scientific member, with an explicit pure observation choice.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(tag = "kind", content = "request", rename_all = "snake_case", deny_unknown_fields)]
pub enum ExpectationMember {
    projection: { name: "ExpectationMember", rust_path: crate::workflow::expectation::ExpectationMember, version: 1, direction: Direction::Both, reserved_tags: [], reserved_names: [] };
    /// Existing ordinary authored solve owner.
    Case(CaseOperation) => 1;
    /// Selected actual physical outputs under the pure observation owner.
    Observation(SelectedObservationOperation) => 2;
}
}

pse_contract::projection_struct! {
/// One original category; equal values do not collapse identity.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ExpectationCategory {
    projection: { name: "ExpectationCategory", rust_path: crate::workflow::expectation::ExpectationCategory, version: 1, direction: Direction::Both, reserved_tags: [], reserved_names: [] };
    /// Original authored category identity.
    pub category: SemanticId => { tag: 1, direction: Direction::Both, rules: [] };
    /// Submitted finite physical value.
    pub value: BindingQuantity => { tag: 2, direction: Direction::Both, rules: [] };
    /// Explicit target probability, including retained zero mass.
    pub probability: f64 => { tag: 3, direction: Direction::Both, rules: [] };
}
}
pse_contract::projection_enum! {
/// Initial bounded product-law coordinates, separate from DOE generators.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ExpectationCoordinate {
    projection: { name: "ExpectationCoordinate", rust_path: crate::workflow::expectation::ExpectationCoordinate, version: 1, direction: Direction::Both, reserved_tags: [], reserved_names: [] };
    /// Uniform probability over a finite physical interval.
    Uniform {
        /// Original contextual input reference.
        target: BindingTarget => { tag: 1, direction: Direction::Both, rules: [] };
        /// Lower endpoint with its physical representation.
        lower: BindingQuantity => { tag: 2, direction: Direction::Both, rules: [] };
        /// Upper endpoint with its physical representation.
        upper: BindingQuantity => { tag: 3, direction: Direction::Both, rules: [] };
    } => 1;
    /// Finite ordered target population with explicit masses.
    Categorical {
        /// Original contextual input reference.
        target: BindingTarget => { tag: 1, direction: Direction::Both, rules: [] };
        /// Complete original alternatives, retaining zero masses.
        categories: Vec<ExpectationCategory> => { tag: 2, direction: Direction::Both, rules: [] };
    } => 2;
}
}
pse_contract::projection_struct! {
/// One physical result of an authored joint pushforward.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ExpectationTransformOutput {
    projection: { name: "ExpectationTransformOutput", rust_path: crate::workflow::expectation::ExpectationTransformOutput, version: 1, direction: Direction::Both, reserved_tags: [], reserved_names: [] };
    /// Selected authored physical output, never an equation residual relabeling.
    pub output: SemanticId => { tag: 1, direction: Direction::Both, rules: [] };
    /// Original member target receiving that physical value.
    pub target: BindingTarget => { tag: 2, direction: Direction::Both, rules: [] };
}
}
pse_contract::projection_struct! {
/// A selected authored mathematical operation mapping base variables jointly.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ExpectationTransform {
    projection: { name: "ExpectationTransform", rust_path: crate::workflow::expectation::ExpectationTransform, version: 1, direction: Direction::Both, reserved_tags: [], reserved_names: [] };
    /// Source-bound selected physical observation operation.
    pub operation: SelectedObservationOperation => { tag: 1, direction: Direction::Both, rules: [] };
    /// Ordered admitted output-to-member assignments.
    pub outputs: Vec<ExpectationTransformOutput> => { tag: 2, direction: Direction::Both, rules: [] };
}
}
pse_contract::projection_struct! {
/// One requested physical scalar estimand and its interval-valued controls.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ExpectationOutput {
    projection: { name: "ExpectationOutput", rust_path: crate::workflow::expectation::ExpectationOutput, version: 1, direction: Direction::Both, reserved_tags: [], reserved_names: [] };
    /// Exact selected physical member output identity.
    pub output: SemanticId => { tag: 1, direction: Direction::Both, rules: [] };
    /// Full physical quantity requested from the original member.
    pub quantity: QuantityTypeId => { tag: 2, direction: Direction::Both, rules: [] };
    /// Result representation unit.
    pub unit: UnitId => { tag: 3, direction: Direction::Both, rules: [] };
    /// Admitted reference point; mandatory for origin-sensitive point outputs.
    pub reference: Option<BindingQuantity> => { tag: 4, direction: Direction::Both, rules: [] };
    /// Positive characteristic interval used by dimensionless training.
    pub scale: BindingQuantity => { tag: 5, direction: Direction::Both, rules: [] };
    /// Optional absolute nominal standard-error target, as a physical interval.
    pub absolute_error: Option<BindingQuantity> => { tag: 6, direction: Direction::Both, rules: [] };
    /// Optional relative nominal standard-error target against the reference-centered mean.
    pub relative_error: Option<f64> => { tag: 7, direction: Direction::Both, rules: [] };
    /// Whether this output participates in the max-norm adaptation target.
    pub train: bool => { tag: 8, direction: Direction::Both, rules: [] };
}
}
pse_contract::projection_struct! {
/// Explicit positive-support adaptive-grid controls, validated before construction.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ExpectationAdaptiveSettings {
    projection: { name: "ExpectationAdaptiveSettings", rust_path: crate::workflow::expectation::ExpectationAdaptiveSettings, version: 1, direction: Direction::Both, reserved_tags: [], reserved_names: [] };
    /// Positive continuous bin count.
    pub bins: usize => { tag: 1, direction: Direction::Both, rules: [] };
    /// Positive training population threshold.
    pub minimum_training_samples: usize => { tag: 2, direction: Direction::Both, rules: [] };
    /// Optional nonempty positive bin evolution.
    pub bin_evolution: Option<Vec<usize>> => { tag: 3, direction: Direction::Both, rules: [] };
    /// Joint uniform mixture probability in (0,1].
    pub uniform_mixture: f64 => { tag: 4, direction: Direction::Both, rules: [] };
    /// Finite categorical probability ratio at least one.
    pub maximum_probability_ratio: f64 => { tag: 5, direction: Direction::Both, rules: [] };
    /// Finite categorical learning rate in [0,1].
    pub discrete_learning_rate: f64 => { tag: 6, direction: Direction::Both, rules: [] };
    /// Finite continuous learning rate in [0,1].
    pub continuous_learning_rate: f64 => { tag: 7, direction: Direction::Both, rules: [] };
}
}
pse_contract::projection_enum! {
/// Distinct selected probability estimators with no implicit fallback.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(tag = "kind", content = "settings", rename_all = "snake_case", deny_unknown_fields)]
pub enum ExpectationEstimator {
    projection: { name: "ExpectationEstimator", rust_path: crate::workflow::expectation::ExpectationEstimator, version: 1, direction: Direction::Both, reserved_tags: [], reserved_names: [] };
    /// Draw directly from the target law and average each output once.
    FixedMonteCarlo => 1;
    /// Adapt a common frozen Numerica proposal at declared iteration barriers.
    Adaptive(ExpectationAdaptiveSettings) => 2;
}
}
pse_contract::projection_struct! {
/// Finite scientific population and replay controls; native allocation is separate.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ExpectationControls {
    projection: { name: "ExpectationControls", rust_path: crate::workflow::expectation::ExpectationControls, version: 1, direction: Direction::Both, reserved_tags: [], reserved_names: [] };
    /// Root RNG seed, combined with original stream and consumed state.
    pub seed: u64 => { tag: 1, direction: Direction::Both, rules: [] };
    /// Positive finite number of independently owned sampling lanes.
    pub lanes: usize => { tag: 2, direction: Direction::Both, rules: [] };
    /// Finite original sample population per child study, at least two.
    pub samples_per_iteration: usize => { tag: 3, direction: Direction::Both, rules: [] };
    /// Positive finite iteration limit.
    pub maximum_iterations: usize => { tag: 4, direction: Direction::Both, rules: [] };
    /// Total finite population limit; fewer than two remaining draws issue no statistical child.
    pub maximum_samples: usize => { tag: 5, direction: Direction::Both, rules: [] };
    /// Explicit probability-sum admission tolerance; no hidden renormalization policy.
    pub normalization_tolerance: f64 => { tag: 6, direction: Direction::Both, rules: [] };
}
}
pse_contract::projection_struct! {
/// Scientific expectation request, independent of operating allocation settings.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ExpectationRequest {
    projection: { name: "ExpectationRequest", rust_path: crate::workflow::expectation::ExpectationRequest, version: 1, direction: Direction::Both, reserved_tags: [], reserved_names: [] };
    /// Original immutable model, context and research ancestry.
    pub source: OperationSource => { tag: 1, direction: Direction::Both, rules: [] };
    /// Existing numerical preparation settings.
    pub preparation: PreparationSettings => { tag: 2, direction: Direction::Both, rules: [] };
    /// Ordinary authored case or typed selected-observation member.
    pub member: ExpectationMember => { tag: 3, direction: Direction::Both, rules: [] };
    /// Ordered bounded unit-cube and categorical law coordinates.
    pub coordinates: Vec<ExpectationCoordinate> => { tag: 4, direction: Direction::Both, rules: [] };
    /// Optional explicit joint pushforward; no density Jacobian is inferred.
    pub transform: Option<ExpectationTransform> => { tag: 5, direction: Direction::Both, rules: [] };
    /// Ordered physical estimands, retaining per-output availability.
    pub outputs: Vec<ExpectationOutput> => { tag: 6, direction: Direction::Both, rules: [] };
    /// Selected fixed or adaptive numerical realization.
    pub estimator: ExpectationEstimator => { tag: 7, direction: Direction::Both, rules: [] };
    /// Finite scientific population and replay controls.
    pub controls: ExpectationControls => { tag: 8, direction: Direction::Both, rules: [] };
    /// Explicitly request a scientific-failure probability alongside physical outputs.
    pub failure_probability: bool => { tag: 9, direction: Direction::Both, rules: [] };
    /// Explicit population/boundary replay policy; no cross-platform bitwise sum promise.
    pub replay: ExpectationReplay => { tag: 10, direction: Direction::Both, rules: [] };
}
}

pse_contract::projection_enum! {
/// The initially supported replay meaning, declared independently of a seed.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ExpectationReplay {
    projection: { name: "ExpectationReplay", rust_path: crate::workflow::expectation::ExpectationReplay, version: 1, direction: Direction::Both, reserved_tags: [], reserved_names: [] };
    /// Reproduce original points, categories, proposal weights and sealed iteration state.
    PopulationAndBoundary => 1;
}
}

pse_contract::projection_struct! {
/// One output's original estimate, with physical representation and incomplete denominator.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct OutputEstimate {
    projection: { name: "ExpectationOutputEstimate", rust_path: crate::workflow::expectation::OutputEstimate, version: 1, direction: Direction::Output, reserved_tags: [], reserved_names: [] };
    /// Exact original output identity.
    pub output: SemanticId => { tag: 1, direction: Direction::Output, rules: [] };
    /// Full output physical quantity.
    pub quantity: QuantityTypeId => { tag: 2, direction: Direction::Output, rules: [] };
    /// Representation of the estimate and interval standard error.
    pub unit: UnitId => { tag: 3, direction: Direction::Output, rules: [] };
    /// Declared reference in this representation.
    pub reference: f64 => { tag: 4, direction: Direction::Output, rules: [] };
    /// Original issued positive-weight samples.
    pub issued: usize => { tag: 5, direction: Direction::Output, rules: [] };
    /// Samples with this scalar scientifically available.
    pub available: usize => { tag: 6, direction: Direction::Output, rules: [] };
    /// Original global ordinals lacking this output.
    pub unavailable: Vec<u64> => { tag: 7, direction: Direction::Output, rules: [] };
    /// Centered partial estimate; incomplete output never claims completed expectation.
    pub estimate: Option<f64> => { tag: 8, direction: Direction::Output, rules: [] };
    /// Nominal sampling standard error, not a physical enclosure.
    pub standard_error: Option<f64> => { tag: 9, direction: Direction::Output, rules: [] };
    /// All issued samples scientifically supplied this output with sufficient statistics.
    pub complete: bool => { tag: 10, direction: Direction::Output, rules: [] };
    /// Every selected absolute and relative standard-error target is met.
    pub precision_met: bool => { tag: 11, direction: Direction::Output, rules: [] };
}
}

pse_contract::projection_struct! {
/// A separately selected scientific-failure probability, excluding operational loss.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FailureProbability {
    projection: { name: "ExpectationFailureProbability", rust_path: crate::workflow::expectation::FailureProbability, version: 1, direction: Direction::Output, reserved_tags: [], reserved_names: [] };
    /// Indicator estimate under the admitted target law.
    pub estimate: f64 => { tag: 1, direction: Direction::Output, rules: [] };
    /// Nominal sampling standard error.
    pub standard_error: f64 => { tag: 2, direction: Direction::Output, rules: [] };
}
}

pse_contract::projection_struct! {
/// One admitted normalized category mass alongside the submitted original law.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ExpectationCategoryMass {
    projection: { name: "ExpectationCategoryMass", rust_path: crate::workflow::expectation::ExpectationCategoryMass, version: 1, direction: Direction::Output, reserved_tags: [], reserved_names: [] };
    /// Resolved original input coordinate.
    pub member: SemanticId => { tag: 1, direction: Direction::Output, rules: [] };
    /// Original authored category, including zero mass.
    pub category: SemanticId => { tag: 2, direction: Direction::Output, rules: [] };
    /// Admitted normalized mass under the declared tolerance interpretation.
    pub probability: f64 => { tag: 3, direction: Direction::Output, rules: [] };
}
}

pse_contract::projection_struct! {
/// Original parent interpretation and output estimates, including unmet stopping targets.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ExpectationReport {
    projection: { name: "ExpectationReport", rust_path: crate::workflow::expectation::ExpectationReport, version: 1, direction: Direction::Output, reserved_tags: [], reserved_names: [] };
    /// Scientific request defining measure, transform, outputs and estimator.
    pub request: ExpectationRequest => { tag: 1, direction: Direction::Output, rules: [] };
    /// Fully committed iteration count.
    pub completed_iterations: usize => { tag: 2, direction: Direction::Output, rules: [] };
    /// Original issued samples, including unavailable members.
    pub issued: usize => { tag: 3, direction: Direction::Output, rules: [] };
    /// Ordered per-output estimates.
    pub outputs: Vec<OutputEstimate> => { tag: 4, direction: Direction::Output, rules: [] };
    /// Optional separately requested scientific-failure indicator estimate/error.
    pub failure_probability: Option<FailureProbability> => { tag: 5, direction: Direction::Output, rules: [] };
    /// Operationally incomplete indicator evaluations keep failure probability unavailable.
    pub failure_probability_complete: bool => { tag: 6, direction: Direction::Output, rules: [] };
    /// Original parent was stopped before a completed boundary.
    pub interrupted: bool => { tag: 7, direction: Direction::Output, rules: [] };
    /// A finite limit ended execution while at least one selected precision target was unmet.
    pub limit_reached: bool => { tag: 8, direction: Direction::Output, rules: [] };
    /// Original identities with the actual admitted normalized probabilities.
    pub admitted_categories: Vec<ExpectationCategoryMass> => { tag: 9, direction: Direction::Output, rules: [] };
}
}
