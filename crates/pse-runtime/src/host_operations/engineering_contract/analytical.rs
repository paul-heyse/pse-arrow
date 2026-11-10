// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Selected mathematics uses ordinary original run/attempt and protected result custody.
use pse_contract::boundary_projection::{Direction, ProjectionError, ProjectionGraph, Service};
use pse_model::engineering::{LOCAL_REVISION_RULE, OPERATION_ID_RULE};
pse_contract::projection_struct! {
/// Submit one exact source-bound mathematical request under original caller coordinates.
#[derive(Clone,Debug,serde::Serialize,serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EngineeringExecuteMathematical {
    projection:{name:"EngineeringExecuteMathematical",rust_path:crate::host_operations::engineering_contract::EngineeringExecuteMathematical,version:1,direction:Direction::Input,reserved_tags:[],reserved_names:[]};
    /// Original operation fixed before submission or a lost acknowledgement.
    pub operation:String => {tag:1,direction:Direction::Input,rules:[OPERATION_ID_RULE]};
    /// Original caller-issued run.
    pub run:pse_ids::SemanticId => {tag:2,direction:Direction::Input,rules:[]};
    /// Exact original scientific attempt; later attempts do not substitute it.
    pub attempt:pse_ids::SemanticId => {tag:3,direction:Direction::Input,rules:[]};
    /// Original immutable source admission.
    pub source_operation:String => {tag:4,direction:Direction::Input,rules:[OPERATION_ID_RULE]};
    /// Required original local source revision.
    pub expected_local_revision:String => {tag:5,direction:Direction::Input,rules:[LOCAL_REVISION_RULE]};
    /// Actual native selected request; contains no mathematical parser input.
    pub request:crate::workflow::MathematicalRequest => {tag:6,direction:Direction::Input,rules:[]};
    /// Original expansion and compiler policy.
    pub preparation:crate::workflow::PreparationSettings => {tag:7,direction:Direction::Input,rules:[]};
}
}
pub(super) fn service(
    service: Service,
    graph: &mut ProjectionGraph,
) -> Result<Service, ProjectionError> {
    service
        .unary::<EngineeringExecuteMathematical, pse_model::engineering::EngineeringRunStatus>(
            graph,
            "ExecuteMathematical",
        )?
        .effects(pse_contract::operation_effects::EffectFacts::NATIVE_EXECUTION)
}
