// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Source-owned private root data, independent of native execution preparation.
use pse_contract::boundary_projection::{Direction, LocalRule, ProjectionError, ProjectionGraph};
use pse_ids::{ContentHash, SemanticId};
use pse_math::implicit::{Options, transfer::RootExecutionSnapshot};
use pse_model::diagnostic::BoundaryDiagnostic;
use std::time::Duration;

/// Existing private root request/frame byte bound.
pub const REQUEST_LIMIT: usize = 16 << 20;
/// Existing private root response/frame byte bound.
pub const RESPONSE_LIMIT: usize = 1 << 20;

pse_contract::projection_struct! {
/// Complete parent-captured execution. Decoding cannot issue source or attempt authority.
#[derive(Clone, Debug)]
pub struct RootInput {
    projection: { name: "NativeRootInput", rust_path: crate::math::native_root_protocol::RootInput,
        version: 4, direction: Direction::Both,
        reserved_tags: [1, 3, 4, 5, 6, 7, 10, 11, 12],
        reserved_names: ["body", "source", "registry", "registry_digest", "unknowns", "rows", "evaluation", "max_entries", "preconditions"] };
    /// Original parent-held physical/math/provider interpretation, without replay permission.
    pub spec: pse_math::binding::BodySpec => { tag: 2, direction: Direction::Both, rules: [LocalRule::Required] };
    /// Ordinary numerical parameters, preserving exact binary64 bits.
    pub parameters: Vec<f64> => { tag: 8, direction: Direction::Both, rules: [] };
    /// Original initialization, accuracy and native solve controls.
    pub options: Options => { tag: 9, direction: Direction::Both, rules: [LocalRule::Required] };
    /// Exact cold execution and full metadata captured from the actual original root worker.
    pub execution: RootExecutionSnapshot => { tag: 13, direction: Direction::Both, rules: [LocalRule::Required] };
}
}
pse_contract::projection_enum! {
    /// Passive preparation either derives demand or retains its original typed refusal.
    #[derive(Clone, Debug)]
    pub enum RestorePreparationOutcome {
        projection: { name: "NativeRootRestorePreparationOutcome", rust_path: crate::math::native_root_protocol::RestorePreparationOutcome,
            version: 1, direction: Direction::Both, reserved_tags: [], reserved_names: [] };
        /// Checked positive demand from actual received native capacities; still no grant.
        Demand(usize => { rules: [LocalRule::UnsignedRange { min: 1, max: u64::MAX }] }) => 1;
        /// Original preparation failure; restoration and native solving have not started.
        Refused(BoundaryDiagnostic) => 2;
    }
}
pse_contract::projection_struct! {
/// Trusted child's preparation result, still requiring actual parent admission on success.
#[derive(Clone, Debug)]
pub struct RestorePreparation {
    projection: { name: "NativeRootRestorePreparation", rust_path: crate::math::native_root_protocol::RestorePreparation,
        version: 1, direction: Direction::Both, reserved_tags: [], reserved_names: [] };
    /// Original attempt, never a new operation.
    pub attempt: SemanticId => { tag: 1, direction: Direction::Both, rules: [] };
    /// Original execution binding.
    pub execution: ContentHash => { tag: 2, direction: Direction::Both, rules: [] };
    /// Exact original payload binding.
    pub request: ContentHash => { tag: 3, direction: Direction::Both, rules: [] };
    /// Original refusal or checked demand, never both and never an absent default success.
    pub result: RestorePreparationOutcome => { tag: 4, direction: Direction::Both, rules: [LocalRule::Required] };
}
}
pse_contract::projection_struct! {
/// Recorded parent pool admission on this exact private attempt; not source authority.
#[derive(Clone, Debug)]
pub struct RestoreGrant {
    projection: { name: "NativeRootRestoreGrant", rust_path: crate::math::native_root_protocol::RestoreGrant,
        version: 1, direction: Direction::Both, reserved_tags: [], reserved_names: [] };
    /// Same original attempt whose demand was received.
    pub attempt: SemanticId => { tag: 1, direction: Direction::Both, rules: [] };
    /// Same original execution binding.
    pub execution: ContentHash => { tag: 2, direction: Direction::Both, rules: [] };
    /// Same original payload binding.
    pub request: ContentHash => { tag: 3, direction: Direction::Both, rules: [] };
    /// Bytes actually reserved by the original parent's pool and retained through reap.
    pub admitted_bytes: usize => { tag: 4, direction: Direction::Both, rules: [LocalRule::UnsignedRange { min: 1, max: u64::MAX }] };
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
    graph.register::<RestorePreparation>()?;
    graph.register::<RestoreGrant>()?;
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
            "MathRootExecutionSnapshot",
            "NativeRootRestorePreparation",
            "NativeRootRestoreGrant",
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
        ] {
            assert!(
                graph.definitions().contains_key(name),
                "actual native dependency absent: {name}"
            );
        }
        assert_eq!(graph.definitions()["NativeRootInput"].version, 4);
        assert_eq!(
            graph.definitions()["NativeRootInput"].reserved_tags,
            [1, 3, 4, 5, 6, 7, 10, 11, 12]
        );
        assert!(graph.definitions()["NativePathBuf"].checked_record);
    }
}
