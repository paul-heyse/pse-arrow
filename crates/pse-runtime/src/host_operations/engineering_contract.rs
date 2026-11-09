// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Source-owned engineering service composition over actual native preparation records.

use pse_contract::boundary_projection::{Direction, LocalRule, ProjectionError, ProjectionGraph};
use pse_model::engineering::OPERATION_ID_RULE;

mod inspection;
mod study;
pub use inspection::{EngineeringInspectModeling, EngineeringInspectProviders};
pub use study::{
    EXPANDED_STUDY_SERVICE, EXPLICIT_STUDY_SERVICE, STUDY_CONTROL_SERVICE,
    EngineeringCohortRelease, EngineeringFreezeStudyCohort, EngineeringObserveStudy,
    EngineeringPrepareExpandedStudy, EngineeringPrepareExplicitStudy, EngineeringPreparedStudy,
    EngineeringReadStudyResult, EngineeringRestoreExpandedStudy, EngineeringRestoreExplicitStudy,
    EngineeringStudyCohortPage, EngineeringStudyHeaderPage,
    EngineeringStudyKey, EngineeringStudyResultPage, EngineeringStudyStatus,
    EngineeringSubmitStudy, ExpandedStudyDefinition, ExplicitStudyDefinition, StudyDefinitionContext,
};

pse_contract::projection_struct! {
    #[derive(Clone, Debug)]
    /// Select one retained source for the original typed native discovery request.
    pub struct EngineeringDiscoverPreparation {
        projection: { name: "EngineeringDiscoverPreparation", rust_path: crate::host_operations::engineering_contract::EngineeringDiscoverPreparation,
            version: 1, direction: Direction::Input, reserved_tags: [], reserved_names: [] };
        /// Exact original operation retaining the admitted package.
        pub source_operation: String => { tag: 1, direction: Direction::Input, rules: [OPERATION_ID_RULE] };
        /// The actual native owner supplies all request meaning, fields and validation.
        pub request: crate::workflow::PreparationDiscoveryRequest => { tag: 2, direction: Direction::Input, rules: [] };
    }
}

pse_contract::projection_struct! {
    #[derive(Clone, Debug)]
    /// Select an exact retained package for the original one-file source reader.
    pub struct EngineeringReadSourceFile {
        projection: { name: "EngineeringReadSourceFile", rust_path: crate::host_operations::engineering_contract::EngineeringReadSourceFile,
            version: 1, direction: Direction::Input, reserved_tags: [], reserved_names: [] };
        /// Exact original service operation retaining the admitted package.
        pub source_operation: String => { tag: 1, direction: Direction::Input, rules: [OPERATION_ID_RULE] };
        /// Actual native request selects its exact parent, package and relative path.
        pub request: crate::workflow::OriginalSourceFileRequest => { tag: 2, direction: Direction::Input, rules: [LocalRule::Required] };
    }
}

pse_contract::projection_struct! {
    #[derive(Clone, Debug)]
    /// Select original physical meaning from the exact admitted source context.
    pub struct EngineeringInspectPhysical {
        projection: { name: "EngineeringInspectPhysical", rust_path: crate::host_operations::engineering_contract::EngineeringInspectPhysical,
            version: 1, direction: Direction::Input, reserved_tags: [], reserved_names: [] };
        /// Exact original service operation retaining the selected admitted package.
        pub source_operation: String => { tag: 1, direction: Direction::Input, rules: [OPERATION_ID_RULE] };
        /// Actual native physical query, including quantity/unit identity and finite paging.
        pub request: crate::workflow::PhysicalInspectionRequest => { tag: 2, direction: Direction::Input, rules: [LocalRule::Required] };
    }
}

/// Compose lightweight service methods and the actual runtime-owned discovery crossing.
/// # Errors
/// Contradictory source owner fields, stable tags, effects or local rules refuse generation.
pub fn projection() -> Result<ProjectionGraph, ProjectionError> {
    let mut graph = ProjectionGraph::default();
    // DescribeContract publishes these actual resolved registry owners alongside
    // the admitted physical-context shapes. Their source facets grant no readiness.
    graph.register::<pse_model::generated::reference::quantity_types::Row>()?;
    graph.register::<pse_model::generated::reference::bases::Row>()?;
    graph.register::<crate::workflow::ScientificPermission>()?;
    graph.register::<crate::workflow::ProductPermission>()?;
    graph.register::<crate::workflow::MemberScientificPermission>()?;
    let service = pse_model::engineering::service(&mut graph)?
        .unary::<EngineeringDiscoverPreparation, crate::workflow::PreparationDiscoveryReport>(
            &mut graph,
            "DiscoverPreparation",
        )?
        .effects(pse_contract::operation_effects::EffectFacts::OBSERVATION)?
        .unary::<EngineeringReadSourceFile, crate::workflow::OriginalSourceFile>(
            &mut graph,
            "ReadSourceFile",
        )?
        .effects(pse_contract::operation_effects::EffectFacts::OBSERVATION)?
        .unary::<EngineeringInspectPhysical, crate::workflow::PhysicalInspectionDocument>(
            &mut graph,
            "InspectPhysical",
        )?
        .effects(pse_contract::operation_effects::EffectFacts::OBSERVATION)?;
    let service = inspection::service(service, &mut graph)?;
    graph.service(service)?;
    study::services(&mut graph)?;
    Ok(graph)
}

/// Describe actual method roots, directions and effects from this same source graph.
/// # Errors
/// Missing named owners or effects refuse instead of publishing invented metadata.
pub fn methods() -> Result<Vec<pse_model::engineering::EngineeringMethodContract>, ProjectionError>
{
    method_contracts(&projection()?)
}

#[cfg(feature = "engineering-service")]
pub(super) fn explicit_study_methods() -> Result<Vec<pse_model::engineering::EngineeringMethodContract>, ProjectionError> {
    let mut graph = ProjectionGraph::default();
    study::explicit_services(&mut graph)?;
    method_contracts(&graph)
}

fn method_contracts(graph: &ProjectionGraph) -> Result<Vec<pse_model::engineering::EngineeringMethodContract>, ProjectionError> {
    use pse_contract::boundary_projection::TypeRef;
    let direction = |direction| match direction {
        Direction::Input => "input",
        Direction::Output => "output",
        Direction::Both => "both",
    };
    graph
        .services()
        .values()
        .flat_map(|service| &service.methods)
        .map(|method| {
            let (TypeRef::Named(input), TypeRef::Named(output)) = (&method.input, &method.output)
            else {
                return Err(ProjectionError::invalid(
                    &method.name,
                    "actual method has an unnamed root",
                ));
            };
            let input_owner = graph.definitions().get(input).ok_or_else(|| {
                ProjectionError::invalid(&method.name, "actual input owner is missing")
            })?;
            let output_owner = graph.definitions().get(output).ok_or_else(|| {
                ProjectionError::invalid(&method.name, "actual output owner is missing")
            })?;
            let effects = method.effects.ok_or_else(|| {
                ProjectionError::invalid(&method.name, "actual effects are missing")
            })?;
            Ok(pse_model::engineering::EngineeringMethodContract {
                name: method.name.clone(),
                input: input.clone(),
                output: output.clone(),
                input_direction: direction(input_owner.direction).into(),
                output_direction: direction(output_owner.direction).into(),
                server_streaming: method.server_streaming,
                effects,
            })
        })
        .collect()
}
