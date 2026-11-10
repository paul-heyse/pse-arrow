// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit-measure parents use native original-operation custody and protected report reads.
use pse_contract::boundary_projection::{
    Direction, LocalRule, ProjectionError, ProjectionGraph, Service,
};
use pse_model::engineering::OPERATION_ID_RULE;

pse_contract::projection_struct! {
/// Execute or reconcile one immutable original expectation parent.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EngineeringExecuteExpectation {
    projection: { name: "EngineeringExecuteExpectation", rust_path: crate::host_operations::engineering_contract::EngineeringExecuteExpectation, version: 1, direction: Direction::Input, reserved_tags: [], reserved_names: [] };
    /// Original native operation; repeated requests retain exactly the same intent.
    pub operation: String => { tag: 1, direction: Direction::Input, rules: [OPERATION_ID_RULE] };
    /// Original native immutable source admission.
    pub source_operation: String => { tag: 2, direction: Direction::Input, rules: [OPERATION_ID_RULE] };
    /// Parent identity fixed before the first sample or child execution.
    pub parent: pse_model::generated::identities::StudyId => { tag: 3, direction: Direction::Input, rules: [LocalRule::Required] };
    /// Actual source-owned law, transform, member, outputs and finite estimator controls.
    pub request: crate::workflow::expectation::ExpectationRequest => { tag: 4, direction: Direction::Input, rules: [LocalRule::Required] };
}
}
pse_contract::projection_struct! {
/// Select an original completed estimator boundary without invoking any producer.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EngineeringReadExpectation {
    projection: { name: "EngineeringReadExpectation", rust_path: crate::host_operations::engineering_contract::EngineeringReadExpectation, version: 1, direction: Direction::Input, reserved_tags: [], reserved_names: [] };
    /// Exact retained original parent, never a latest name.
    pub parent: pse_model::generated::identities::StudyId => { tag: 1, direction: Direction::Input, rules: [LocalRule::Required] };
}
}
pse_contract::projection_struct! {
/// Read a bounded original sample page from one exact frozen iteration.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EngineeringReadExpectationSamples {
    projection: { name: "EngineeringReadExpectationSamples", rust_path: crate::host_operations::engineering_contract::EngineeringReadExpectationSamples, version: 1, direction: Direction::Input, reserved_tags: [], reserved_names: [] };
    /// Exact original parent.
    pub parent: pse_model::generated::identities::StudyId => { tag: 1, direction: Direction::Input, rules: [LocalRule::Required] };
    /// Original frozen iteration ordinal.
    pub iteration: u64 => { tag: 2, direction: Direction::Input, rules: [] };
    /// Original within-child starting ordinal, without renumbering selected rows.
    pub offset: u64 => { tag: 3, direction: Direction::Input, rules: [] };
    /// Finite selected sample count.
    pub limit: u32 => { tag: 4, direction: Direction::Input, rules: [LocalRule::UnsignedRange { min: 1, max: 1024 }] };
}
}
pse_contract::projection_struct! {
/// Registered original samples copied without invoking a mathematical producer.
#[derive(Clone, Debug)]
pub struct EngineeringExpectationSamples {
    projection: { name: "EngineeringExpectationSamples", rust_path: crate::host_operations::engineering_contract::EngineeringExpectationSamples, version: 1, direction: Direction::Output, reserved_tags: [], reserved_names: [] };
    /// Exact original parent.
    pub parent: pse_model::generated::identities::StudyId => { tag: 1, direction: Direction::Output, rules: [] };
    /// Original iteration.
    pub iteration: u64 => { tag: 2, direction: Direction::Output, rules: [] };
    /// Original selected start.
    pub offset: u64 => { tag: 3, direction: Direction::Output, rules: [] };
    /// Actual registered typed rows, preserving missing outputs and original child attribution.
    pub rows: Vec<pse_model::generated::runtime::expectation_samples::Row> => { tag: 4, direction: Direction::Output, rules: [] };
    /// Next candidate original offset; a final empty page is permitted.
    pub next_offset: Option<u64> => { tag: 5, direction: Direction::Output, rules: [] };
}
}
pub(super) fn service(
    service: Service,
    graph: &mut ProjectionGraph,
) -> Result<Service, ProjectionError> {
    service
        .unary::<EngineeringExecuteExpectation, crate::workflow::expectation::ExpectationReport>(
            graph,
            "ExecuteExpectation",
        )?
        .effects(pse_contract::operation_effects::EffectFacts::NATIVE_EXECUTION)?
        .unary::<EngineeringReadExpectation, crate::workflow::expectation::ExpectationReport>(
            graph,
            "ReadExpectation",
        )?
        .effects(pse_contract::operation_effects::EffectFacts::OBSERVATION)?
        .unary::<EngineeringReadExpectationSamples, EngineeringExpectationSamples>(
            graph,
            "ReadExpectationSamples",
        )?
        .effects(pse_contract::operation_effects::EffectFacts::OBSERVATION)
}
