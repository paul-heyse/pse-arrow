// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Actual contextual model and passive retained-provider inspection crossings.

use pse_contract::boundary_projection::{Direction, LocalRule};
use pse_model::engineering::OPERATION_ID_RULE;

pse_contract::projection_struct! {
    /// Select the original complete contextual model inspection without dispatching its procedure.
    #[derive(Clone, Debug)]
    pub struct EngineeringInspectModeling {
        projection: { name: "EngineeringInspectModeling", rust_path: crate::host_operations::engineering_contract::EngineeringInspectModeling,
            version: 1, direction: Direction::Input, reserved_tags: [], reserved_names: [] };
        /// Exact original operation retaining the admitted immutable package.
        pub source_operation: String => { tag: 1, direction: Direction::Input, rules: [OPERATION_ID_RULE] };
        /// Actual native request including full preparation/numerical controls and finite page.
        pub request: crate::workflow::ModelingInspectionRequest => { tag: 2, direction: Direction::Input, rules: [LocalRule::Required] };
    }
}

pse_contract::projection_struct! {
    /// Select actual retained factories independently of numerical worker construction.
    #[derive(Clone, Debug)]
    pub struct EngineeringInspectProviders {
        projection: { name: "EngineeringInspectProviders", rust_path: crate::host_operations::engineering_contract::EngineeringInspectProviders,
            version: 1, direction: Direction::Input, reserved_tags: [], reserved_names: [] };
        /// Exact original source operation; source descriptor text alone grants no registration.
        pub source_operation: String => { tag: 1, direction: Direction::Input, rules: [OPERATION_ID_RULE] };
        /// Actual bounded native retained-registration selection.
        pub request: crate::workflow::ProviderInspectionRequest => { tag: 2, direction: Direction::Input, rules: [LocalRule::Required] };
    }
}

pub(super) fn service(
    service: pse_contract::boundary_projection::Service,
    graph: &mut pse_contract::boundary_projection::ProjectionGraph,
) -> Result<
    pse_contract::boundary_projection::Service,
    pse_contract::boundary_projection::ProjectionError,
> {
    service
        .unary::<EngineeringInspectModeling, crate::workflow::ModelingInspection>(
            graph,
            "InspectModeling",
        )?
        .effects(pse_contract::operation_effects::EffectFacts::OBSERVATION)?
        .unary::<EngineeringInspectProviders, crate::workflow::ProviderInspectionPage>(
            graph,
            "InspectProviders",
        )?
        .effects(pse_contract::operation_effects::EffectFacts::OBSERVATION)
}
