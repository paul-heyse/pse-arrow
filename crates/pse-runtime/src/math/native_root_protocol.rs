// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Source-owned private root data, independent of native execution preparation.
use pse_contract::boundary_projection::{Direction, LocalRule, ProjectionError, ProjectionGraph};
use pse_ids::{ContentHash, SemanticId};
use pse_math::{
    implicit::{Options, Unknown},
    jets::EvaluationLimits,
};
use pse_model::diagnostic::BoundaryDiagnostic;
use std::{collections::BTreeMap, time::Duration};

/// Existing private root request/frame byte bound.
pub const REQUEST_LIMIT: usize = 16 << 20;
/// Existing private root response/frame byte bound.
pub const RESPONSE_LIMIT: usize = 1 << 20;

pse_contract::projection_struct! {
/// Complete reconstructible input. Opaque recipe bytes retain their original replay authority.
#[derive(Clone, Debug)]
pub struct RootInput {
    projection: { name: "NativeRootInput", rust_path: crate::math::native_root_protocol::RootInput,
        version: 3, direction: Direction::Both, reserved_tags: [5], reserved_names: ["registry_digest"] };
    /// Original admitted recipe; decoded envelope data cannot issue its replay permit.
    pub body: Vec<u8> => { tag: 1, direction: Direction::Both, rules: [LocalRule::Length { min: 1, max: REQUEST_LIMIT as u64 }] };
    /// Complete expected native physical/math/provider interpretation.
    pub spec: pse_math::binding::BodySpec => { tag: 2, direction: Direction::Both, rules: [LocalRule::Required] };
    /// Actual receiving diagnostic maps, separate from arithmetic identity.
    pub source: RootBindings => { tag: 3, direction: Direction::Both, rules: [LocalRule::Required] };
    /// Complete unadmitted physical declarations; the receiving native builder admits them.
    pub registry: pse_quantity::QuantityRegistryBuilder => { tag: 4, direction: Direction::Both, rules: [LocalRule::Required] };
    /// Ordered original physical unknowns and intervals.
    pub unknowns: Vec<Unknown> => { tag: 6, direction: Direction::Both, rules: [] };
    /// Ordered original residual identities.
    pub rows: Vec<SemanticId> => { tag: 7, direction: Direction::Both, rules: [] };
    /// Ordinary numerical parameters, preserving exact binary64 bits.
    pub parameters: Vec<f64> => { tag: 8, direction: Direction::Both, rules: [] };
    /// Original initialization, accuracy and native solve controls.
    pub options: Options => { tag: 9, direction: Direction::Both, rules: [LocalRule::Required] };
    /// Original compiled-body resource allowance.
    pub evaluation: EvaluationLimits => { tag: 10, direction: Direction::Both, rules: [LocalRule::Required] };
    /// Existing original sparse construction allowance.
    pub max_entries: usize => { tag: 11, direction: Direction::Both, rules: [] };
    /// Original prerequisite declarations, distinct from grants that they currently hold.
    pub preconditions: Vec<pse_quantity::PhysicalPrecondition> => { tag: 12, direction: Direction::Both, rules: [] };
}
}
pse_contract::projection_struct! {
/// Current diagnostic context is transported separately from arithmetic identity.
/// Decoding grants no authority: the receiving constructor checks the replayed body.
#[derive(Clone, Debug)]
pub struct RootBindings {
    projection: { name: "NativeRootBindings", rust_path: crate::math::native_root_protocol::RootBindings,
        version: 2, direction: Direction::Both, reserved_tags: [], reserved_names: [] };
    /// Actual receiving assembly instance.
    pub instance: SemanticId => { tag: 1, direction: Direction::Both, rules: [] };
    /// Original checked member to actual receiving member map.
    pub checked_members: BTreeMap<SemanticId, SemanticId> => { tag: 2, direction: Direction::Both, rules: [] };
    /// Original checked source to actual receiving source map.
    pub checked_sources: BTreeMap<SemanticId, SemanticId> => { tag: 3, direction: Direction::Both, rules: [] };
    /// Actual authored locations; absence is never invented attribution.
    pub occurrences: BTreeMap<SemanticId, pse_model::diagnostic::SourceLocation> => { tag: 4, direction: Direction::Both, rules: [] };
}
}

pse_contract::projection_struct! {
/// Actual child image/build observations; independently checked by the original parent owner.
#[derive(Clone, Debug)]
pub struct Hello {
    projection: { name: "NativeRootHello", rust_path: crate::math::native_root_protocol::Hello,
        version: 2, direction: Direction::Both, reserved_tags: [], reserved_names: [] };
    /// Original child diagnostic continuation operation.
    pub diagnostic_operation: pse_diagnostics::observation::OperationId => { tag: 1, direction: Direction::Both, rules: [] };
    /// Exact private profile interpretation.
    pub protocol: String => { tag: 2, direction: Direction::Both, rules: [] };
    /// Parent-issued attempt identity.
    pub attempt: SemanticId => { tag: 3, direction: Direction::Both, rules: [] };
    /// Actual original child PID.
    pub pid: u32 => { tag: 4, direction: Direction::Both, rules: [] };
    /// Actual original parent PID.
    pub parent: u32 => { tag: 5, direction: Direction::Both, rules: [] };
    /// Exact independently observed child image/path/mode.
    pub executable: pse_buildinfo::FileObservation => { tag: 6, direction: Direction::Both, rules: [LocalRule::Required] };
    /// Existing captured implementation facts, without reconstructed producer authority.
    pub build: pse_buildinfo::BuildInfo => { tag: 7, direction: Direction::Both, rules: [LocalRule::Required] };
    /// Native solver profile observation.
    pub native_build: ContentHash => { tag: 8, direction: Direction::Both, rules: [] };
}
}
pse_contract::projection_struct! {
/// One private admitted-process request; the receiver still verifies all original bindings.
#[derive(Clone, Debug)]
pub struct Request {
    projection: { name: "NativeRootRequest", rust_path: crate::math::native_root_protocol::Request,
        version: 2, direction: Direction::Both, reserved_tags: [], reserved_names: [] };
    /// Original parent continuation operation, when observed.
    pub diagnostic_parent: Option<pse_diagnostics::observation::OperationId> => { tag: 1, direction: Direction::Both, rules: [] };
    /// Exact admitted attempt.
    pub attempt: SemanticId => { tag: 2, direction: Direction::Both, rules: [] };
    /// Existing execution-context binding; no new authority is derived from decoding.
    pub execution: ContentHash => { tag: 3, direction: Direction::Both, rules: [] };
    /// Existing exact request-payload binding.
    pub request: ContentHash => { tag: 4, direction: Direction::Both, rules: [] };
    /// Remaining parent allowance sent after startup; parent retains its original clock.
    pub remaining: Duration => { tag: 5, direction: Direction::Both, rules: [LocalRule::PositiveDuration] };
    /// One exact generated NativeRootInput frame.
    pub source: Vec<u8> => { tag: 6, direction: Direction::Both, rules: [LocalRule::Length { min: 1, max: REQUEST_LIMIT as u64 }] };
}
}
pse_contract::projection_enum! {
    /// Child proposal or original typed refusal. Only the parent screens scientific usability.
    #[derive(Clone, Debug)]
    pub enum RootOutcome {
        projection: { name: "NativeRootOutcome", rust_path: crate::math::native_root_protocol::RootOutcome,
            version: 2, direction: Direction::Both, reserved_tags: [], reserved_names: [] };
        /// Original-coordinate candidate awaiting the original parent's verification.
        Proposal(Vec<f64>) => 1;
        /// Original complete native/model diagnostic, including scoped physical observations.
        Refused(BoundaryDiagnostic) => 2;
    }
}
pse_contract::projection_struct! {
/// Exact private invocation outcome, without replay or scientific-admission authority.
#[derive(Clone, Debug)]
pub struct Response {
    projection: { name: "NativeRootResponse", rust_path: crate::math::native_root_protocol::Response,
        version: 2, direction: Direction::Both, reserved_tags: [], reserved_names: [] };
    /// Original attempt reference.
    pub attempt: SemanticId => { tag: 1, direction: Direction::Both, rules: [] };
    /// Original execution binding.
    pub execution: ContentHash => { tag: 2, direction: Direction::Both, rules: [] };
    /// Original request binding.
    pub request: ContentHash => { tag: 3, direction: Direction::Both, rules: [] };
    /// Recorded child proposal/refusal; this is not a current acceptance decision.
    pub result: RootOutcome => { tag: 4, direction: Direction::Both, rules: [LocalRule::Required] };
}
}

/// Actual source-owned closed-root records; a schema walk never reconstructs scientific authority.
pub fn projection_graph() -> Result<ProjectionGraph, ProjectionError> {
    let mut graph = ProjectionGraph::of::<RootInput>()?;
    // Numerical correspondence has its own passive root. Register the actual owner
    // rather than duplicating its recursively nested stages in the runtime DTO.
    graph.register::<pse_math::transfer::ExecutionSnapshot>()?;
    graph.register::<Hello>()?;
    graph.register::<Request>()?;
    graph.register::<Response>()?;
    Ok(graph)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn private_native_source_graph_uses_actual_typed_context_and_platform_data() {
        let graph = projection_graph().unwrap();
        for name in [
            "MathExecutionSnapshot",
            "MathStageSnapshot",
            "MathBlockSnapshot",
            "MathApplicabilitySnapshot",
            "NativeRootInput",
            "NativeRootHello",
            "NativeRootRequest",
            "NativeRootResponse",
            "ModelBoundaryDiagnostic",
            "ModelSourceLocation",
            "ApplicabilityObservation",
            "CapturedBuildInfo",
            "FileObservation",
            "MathBodySpec",
            "ImplicitUnknown",
            "ImplicitOptions",
            "NativePathBuf",
            "PlatformPathBytes",
            "PhysicalRegistryDeclarations",
            "PhysicalPrerequisiteDeclaration",
            "PhysicalPrerequisitePredicate",
        ] {
            assert!(
                graph.definitions().contains_key(name),
                "actual native dependency absent: {name}"
            );
        }
        assert_eq!(graph.definitions()["NativeRootInput"].version, 3);
        assert_eq!(graph.definitions()["NativeRootInput"].reserved_tags, [5]);
        assert!(graph.definitions()["NativePathBuf"].checked_record);
    }
}
