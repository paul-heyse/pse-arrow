// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Concrete local engineering service inputs and selected original-owner outcomes.

pub use crate::engineering_edit::{
    EngineeringEditSource, SourceFileChange, SourceFileDelta, SourceFileEdit,
};
use pse_contract::boundary_projection::{
    Direction, LocalRule, ProjectionError, ProjectionGraph, Service,
};
use pse_contract::operation_effects::EffectFacts;
use pse_contract::{projection_enum, projection_struct};
use pse_ids::{ContentHash, SemanticId};
use std::collections::BTreeMap;

/// Byte extent of one opaque caller operation identity, shared with native issuance.
pub const OPERATION_ID_RULE: LocalRule = LocalRule::Utf8Bytes { min: 1, max: 128 };
/// Byte extent of one exact local receiver token.
pub const LOCAL_REVISION_RULE: LocalRule = LocalRule::Utf8Bytes { min: 1, max: 4096 };
/// Byte extent of one selected authored package-relative path.
pub const SOURCE_FILE_PATH_RULE: LocalRule = LocalRule::Utf8Bytes { min: 1, max: 4096 };
/// Original canonical result relation-name allowance, including its namespace.
pub const RESULT_RELATION_RULE: LocalRule = LocalRule::Utf8Bytes { min: 1, max: 128 };

projection_struct! {
    #[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    /// Request the immutable actual service contract, without scientific construction.
    pub struct EngineeringDescribeRequest {
        projection: { name: "EngineeringDescribeRequest", version: 1, direction: Direction::Input, reserved_tags: [], reserved_names: [] };
    }
}
projection_struct! {
    #[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    /// Immutable compiled descriptors and actual owner-declared service meanings.
    pub struct EngineeringContract {
        projection: { name: "EngineeringContract", version: 1, direction: Direction::Output, reserved_tags: [], reserved_names: [] };
        /// Exact complete descriptor bytes for this service incarnation.
        pub descriptor: Vec<u8> => { tag: 1, direction: Direction::Output, rules: [] };
        /// SHA256 of those exact immutable descriptor bytes.
        pub descriptor_sha256: Vec<u8> => { tag: 2, direction: Direction::Output, rules: [LocalRule::Length { min: 32, max: 32 }] };
        /// Scientific contract owner, separate from interpretation and credentials.
        pub semantics: String => { tag: 3, direction: Direction::Output, rules: [] };
        /// Actual source-selected methods, effects and owner-declared directions.
        pub methods: Vec<EngineeringMethodContract> => { tag: 4, direction: Direction::Output, rules: [] };
    }
}
projection_struct! {
    #[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    /// One actual method's source roots, directions and effects.
    pub struct EngineeringMethodContract {
        projection: { name: "EngineeringMethodContract", version: 1, direction: Direction::Output, reserved_tags: [], reserved_names: [] };
        /// Stable actual generated method name.
        pub name: String => { tag: 1, direction: Direction::Output, rules: [] };
        /// Actual source-owned request message.
        pub input: String => { tag: 2, direction: Direction::Output, rules: [] };
        /// Actual source-owned response message.
        pub output: String => { tag: 3, direction: Direction::Output, rules: [] };
        /// Source-owned request direction, independent of scientific eligibility.
        pub input_direction: String => { tag: 4, direction: Direction::Output, rules: [] };
        /// Source-owned response direction, independent of scientific permission.
        pub output_direction: String => { tag: 5, direction: Direction::Output, rules: [] };
        /// Whether the actual generated method streams its observations.
        pub server_streaming: bool => { tag: 6, direction: Direction::Output, rules: [] };
        /// Declared once on this actual service method; never inferred from its name.
        pub effects: EffectFacts => { tag: 7, direction: Direction::Output, rules: [LocalRule::Required] };
    }
}
projection_struct! {
    #[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    /// Original authored package inventory, interpreted by the existing loader.
    pub struct EngineeringPackage {
        projection: { name: "EngineeringPackage", version: 1, direction: Direction::Both, reserved_tags: [], reserved_names: [] };
        /// Original authored relative file names and bytes; ordinary admission owns interpretation.
        pub files: BTreeMap<String, Vec<u8>> => { tag: 1, direction: Direction::Both, rules: [LocalRule::Length { min: 1, max: 1024 }] };
    }
}
projection_struct! {
    #[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    /// One original request to admit an explicit physical/modeling package closure.
    pub struct EngineeringAdmitSource {
        projection: { name: "EngineeringAdmitSource", version: 1, direction: Direction::Input, reserved_tags: [], reserved_names: [] };
        /// Original caller identity, retained before work and acknowledgement.
        pub operation: String => { tag: 1, direction: Direction::Input, rules: [OPERATION_ID_RULE] };
        /// Complete explicit physical package closure.
        pub physical: Vec<EngineeringPackage> => { tag: 2, direction: Direction::Input, rules: [LocalRule::Length { min: 1, max: 16 }] };
        /// Complete explicit modeling package closure.
        pub modeling: Vec<EngineeringPackage> => { tag: 3, direction: Direction::Input, rules: [LocalRule::Length { min: 1, max: 16 }] };
        /// Exact package-local names selected from the service's authorized registrations.
        pub provider_registrations: Vec<String> => { tag: 4, direction: Direction::Input, rules: [LocalRule::Length { min: 0, max: 16 }] };
    }
}
projection_struct! {
    #[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    /// Exact original service operation, without granting source or execution authority.
    pub struct EngineeringOperationKey {
        projection: { name: "EngineeringOperationKey", version: 1, direction: Direction::Both, reserved_tags: [], reserved_names: [] };
        /// Exact retained original operation in this service incarnation.
        pub operation: String => { tag: 1, direction: Direction::Both, rules: [OPERATION_ID_RULE] };
    }
}
projection_struct! {
    #[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    /// Actual immutable source and physical identities retained by an original admission.
    pub struct EngineeringSourceSelection {
        projection: { name: "EngineeringSourceSelection", version: 1, direction: Direction::Output, reserved_tags: [], reserved_names: [] };
        /// Original operation retaining this exact package.
        pub operation: String => { tag: 1, direction: Direction::Output, rules: [] };
        /// Exact local parent token; semantic content equality cannot replace it.
        pub local_revision: String => { tag: 2, direction: Direction::Output, rules: [] };
        /// Actual admitted source revision identity.
        pub source_revision: ContentHash => { tag: 3, direction: Direction::Output, rules: [] };
        /// Actual admitted complete physical-context identity.
        pub physical_context: ContentHash => { tag: 4, direction: Direction::Output, rules: [] };
    }
}
projection_struct! {
    #[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    /// A selected original owner refusal, independent of transport success.
    pub struct EngineeringFailure {
        projection: { name: "EngineeringFailure", version: 1, direction: Direction::Output, reserved_tags: [], reserved_names: [] };
        /// Original typed diagnostic identity where the owner supplied it.
        pub code: String => { tag: 1, direction: Direction::Output, rules: [] };
        /// Original diagnostic explanation, not a scientific permission.
        pub message: String => { tag: 2, direction: Direction::Output, rules: [] };
        /// Original diagnostic source identities, when supplied by the native owner.
        pub sources: Vec<SemanticId> => { tag: 3, direction: Direction::Output, rules: [] };
        /// Original scientific/contextual rule identity, when supplied.
        pub rule: Option<String> => { tag: 4, direction: Direction::Output, rules: [] };
        /// Original locally decidable owner/field/index path, when a local rule refused.
        pub field_path: Option<String> => { tag: 5, direction: Direction::Output, rules: [] };
        /// Original local validation rule identity, distinct from contextual scientific rules.
        pub field_rule: Option<String> => { tag: 6, direction: Direction::Output, rules: [] };
    }
}
projection_enum! {
    #[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    /// Exclusive original source-admission state; pending never implies scientific readiness.
    pub enum EngineeringSourceOutcome {
        projection: { name: "EngineeringSourceOutcome", version: 1, direction: Direction::Output, reserved_tags: [], reserved_names: [] };
        /// Admitted work still belongs to its original service task.
        Pending => 1;
        /// Actual immutable source/physical result of admission.
        Ready(EngineeringSourceSelection) => 2;
        /// Original retained refusal; this identity cannot start another request.
        Failed(EngineeringFailure) => 3;
    }
}
projection_struct! {
    #[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    /// A finite declaration page selected at the original native source producer.
    pub struct EngineeringInspectSource {
        projection: { name: "EngineeringInspectSource", version: 1, direction: Direction::Input, reserved_tags: [], reserved_names: [] };
        /// Original admission retaining the selected package.
        pub operation: String => { tag: 1, direction: Direction::Input, rules: [OPERATION_ID_RULE] };
        /// Original declaration inventory coordinate.
        pub offset: u64 => { tag: 2, direction: Direction::Input, rules: [] };
        /// Finite selected declaration page, applied at its native producer.
        pub limit: u32 => { tag: 3, direction: Direction::Input, rules: [LocalRule::UnsignedRange { min: 1, max: 128 }] };
    }
}
projection_struct! {
    #[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    /// Original authored declaration identity and name, without a completeness promotion.
    pub struct EngineeringDeclarationHeader {
        projection: { name: "EngineeringDeclarationHeader", version: 1, direction: Direction::Output, reserved_tags: [], reserved_names: [] };
        /// Actual authored semantic declaration identity.
        pub declaration: SemanticId => { tag: 1, direction: Direction::Output, rules: [] };
        /// Actual authored declaration name.
        pub name: String => { tag: 2, direction: Direction::Output, rules: [] };
    }
}
projection_struct! {
    #[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    /// Selected source headers with original population and continuation coordinates.
    pub struct EngineeringDeclarationPage {
        projection: { name: "EngineeringDeclarationPage", version: 1, direction: Direction::Output, reserved_tags: [], reserved_names: [] };
        /// Exact selected source and physical context.
        pub source: EngineeringSourceSelection => { tag: 1, direction: Direction::Output, rules: [LocalRule::Required] };
        /// Actual original inventory population, including omitted pages.
        pub total: u64 => { tag: 2, direction: Direction::Output, rules: [] };
        /// Original selected offset.
        pub offset: u64 => { tag: 3, direction: Direction::Output, rules: [] };
        /// Original continuation, absent only after the actual producer's last page.
        pub next_offset: Option<u64> => { tag: 4, direction: Direction::Output, rules: [] };
        /// Selected declaration headers; this does not claim complete scientific inspection.
        pub declarations: Vec<EngineeringDeclarationHeader> => { tag: 5, direction: Direction::Output, rules: [] };
    }
}
projection_struct! {
    #[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    /// Prepare and run one exact authored case under caller-issued original coordinates.
    pub struct EngineeringSubmitRun {
        projection: { name: "EngineeringSubmitRun", version: 2, direction: Direction::Input, reserved_tags: [2, 5], reserved_names: ["preparation_operation", "expected_preparation"] };
        /// Original request identity, retained before native preparation or start.
        pub operation: String => { tag: 1, direction: Direction::Input, rules: [OPERATION_ID_RULE] };
        /// Caller-issued original run, fixed before a possibly lost acknowledgement.
        pub run: SemanticId => { tag: 3, direction: Direction::Input, rules: [] };
        /// Caller-issued original scientific attempt, distinct from a new execution.
        pub attempt: SemanticId => { tag: 4, direction: Direction::Input, rules: [] };
        /// Original admission retaining the selected immutable package.
        pub source_operation: String => { tag: 6, direction: Direction::Input, rules: [OPERATION_ID_RULE] };
        /// Exact parent token required before native preparation.
        pub expected_local_revision: String => { tag: 7, direction: Direction::Input, rules: [LOCAL_REVISION_RULE] };
        /// Actual authored case declaration, interpreted only by native preparation.
        pub case: SemanticId => { tag: 8, direction: Direction::Input, rules: [] };
        /// Actual registry-owned mathematical purpose; no service default substitutes it.
        pub intent: crate::generated::enums::NativeSolveIntent => { tag: 9, direction: Direction::Input, rules: [LocalRule::Required] };
        /// Optional exact native backend; absence requests existing deterministic routing.
        pub backend: Option<crate::generated::enums::NativeBackend> => { tag: 10, direction: Direction::Input, rules: [] };
    }
}
projection_struct! {
    #[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    /// Original execution coordinates usable independently of a connection or service incarnation.
    pub struct EngineeringRunKey {
        projection: { name: "EngineeringRunKey", version: 1, direction: Direction::Input, reserved_tags: [], reserved_names: [] };
        /// Exact original caller operation, also checked at canonical registration.
        pub operation: String => { tag: 1, direction: Direction::Input, rules: [OPERATION_ID_RULE] };
        /// Exact original native execution identity, known before submission.
        pub run: SemanticId => { tag: 2, direction: Direction::Input, rules: [] };
        /// Exact original scientific attempt, known before submission.
        pub attempt: SemanticId => { tag: 3, direction: Direction::Input, rules: [] };
    }
}
projection_struct! {
    #[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    /// Actual original execution, settlement and completion permission observations.
    pub struct EngineeringRunStatus {
        projection: { name: "EngineeringRunStatus", version: 2, direction: Direction::Output, reserved_tags: [], reserved_names: [] };
        /// Original caller identity, independent of the native run identity.
        pub operation: String => { tag: 1, direction: Direction::Output, rules: [] };
        /// Actual original native identity, available only after start acknowledgement.
        pub run: Option<SemanticId> => { tag: 2, direction: Direction::Output, rules: [] };
        /// True only after the native supervisor published its joined immutable result.
        pub settled: bool => { tag: 3, direction: Direction::Output, rules: [] };
        /// Copied whole-request permission from that original result; absent before settlement.
        pub usable: Option<bool> => { tag: 4, direction: Direction::Output, rules: [] };
        /// Requesting cancellation does not establish settlement.
        pub cancellation_requested: bool => { tag: 5, direction: Direction::Output, rules: [] };
        /// Original start or completion refusal, without replacing usable products.
        pub failure: Option<EngineeringFailure> => { tag: 6, direction: Direction::Output, rules: [] };
        /// Actual native canonical lookup coordinate, absent for explicit local execution.
        pub canonical_run: Option<String> => { tag: 7, direction: Direction::Output, rules: [] };
        /// Actual original attempt lookup coordinate; another attempt is never substituted.
        pub canonical_attempt: Option<String> => { tag: 8, direction: Direction::Output, rules: [] };
        /// Selected original live progress checkpoint; absent after service retirement.
        pub progress: Option<EngineeringRunProgress> => { tag: 9, direction: Direction::Output, rules: [] };
        /// Immutable native preparation actually used by this execution, if started.
        pub preparation: Option<ContentHash> => { tag: 10, direction: Direction::Output, rules: [] };
    }
}
projection_struct! {
    #[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    /// Selected original native checkpoint and its independent source coverage.
    pub struct EngineeringRunProgress {
        projection: { name: "EngineeringRunProgress", version: 1, direction: Direction::Output, reserved_tags: [], reserved_names: [] };
        /// Original attempt-local source generation, independent of transport messages.
        pub generation: u64 => { tag: 1, direction: Direction::Output, rules: [] };
        /// Highest source-issued sequence, including observations lost before retention.
        pub highest_issued_sequence: u64 => { tag: 2, direction: Direction::Output, rules: [] };
        /// Actual source retention/publication loss count.
        pub dropped: u64 => { tag: 3, direction: Direction::Output, rules: [] };
        /// Whether the copied event matched the independently read source head.
        pub current_is_latest: bool => { tag: 4, direction: Direction::Output, rules: [] };
        /// Original event phase; absent when the source retained no current event.
        pub phase: Option<String> => { tag: 5, direction: Direction::Output, rules: [] };
        /// Original elapsed seconds as IEEE754 bits, without decimal conversion.
        pub elapsed_seconds_bits: Option<u64> => { tag: 6, direction: Direction::Output, rules: [] };
        /// Original event sequence, which may precede the independent source head.
        pub sequence: Option<u64> => { tag: 7, direction: Direction::Output, rules: [] };
        /// Original backend-specific metric keys and exactly typed values.
        pub values: BTreeMap<String, EngineeringProgressValue> => { tag: 8, direction: Direction::Output, rules: [] };
        /// Actual passive diagnostic reader availability; absent without that owner.
        pub observations_available: Option<bool> => { tag: 9, direction: Direction::Output, rules: [] };
        /// Original passive diagnostic reader update loss, separate from progress loss.
        pub observation_updates_dropped: Option<u64> => { tag: 10, direction: Direction::Output, rules: [] };
    }
}
projection_enum! {
    #[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    /// Exact selected native metric value; nonfinite reals retain their IEEE754 bits.
    pub enum EngineeringProgressValue {
        projection: { name: "EngineeringProgressValue", version: 1, direction: Direction::Output, reserved_tags: [], reserved_names: [] };
        /// Original exact integral value.
        Integer(i64) => 1;
        /// Original real value represented without precision loss or missing sentinels.
        RealBits(u64) => 2;
        /// Original explanatory text.
        Text(String) => 3;
        /// Original Boolean value.
        Boolean(bool) => 4;
        /// Original native reason that evidence was unavailable.
        Unavailable(crate::generated::enums::EvidenceUnavailableReason) => 5;
    }
}
projection_struct! {
    #[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    /// One exact original product relation/range with an atomic finite response allowance.
    pub struct EngineeringReadResult {
        projection: { name: "EngineeringReadResult", version: 1, direction: Direction::Input, reserved_tags: [], reserved_names: [] };
        /// Original run-start identity, retained before its acknowledgement.
        pub operation: String => { tag: 1, direction: Direction::Input, rules: [OPERATION_ID_RULE] };
        /// Exact native result relation name.
        pub relation: String => { tag: 2, direction: Direction::Input, rules: [RESULT_RELATION_RULE] };
        /// Inclusive original producer row coordinate.
        pub start: u64 => { tag: 3, direction: Direction::Input, rules: [] };
        /// Exclusive original producer row coordinate; equal coordinates select an empty page.
        pub end: u64 => { tag: 4, direction: Direction::Input, rules: [] };
        /// Explicit finite complete response allowance; exhaustion fails atomically.
        pub maximum_bytes: u64 => { tag: 5, direction: Direction::Input, rules: [LocalRule::UnsignedRange { min: 1, max: 16777216 }] };
        /// Original issued run, independently usable after service retirement.
        pub run: SemanticId => { tag: 6, direction: Direction::Input, rules: [] };
        /// Exact original attempt; later attempts never substitute its observations.
        pub attempt: SemanticId => { tag: 7, direction: Direction::Input, rules: [] };
    }
}
projection_struct! {
    #[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    /// Successfully exhausted selected original Arrow product range.
    pub struct EngineeringResultPage {
        projection: { name: "EngineeringResultPage", version: 1, direction: Direction::Output, reserved_tags: [], reserved_names: [] };
        /// Actual original native run, copied from its retained completed result.
        pub run: SemanticId => { tag: 1, direction: Direction::Output, rules: [] };
        /// Exact selected producer relation.
        pub relation: String => { tag: 2, direction: Direction::Output, rules: [] };
        /// Original requested range, not a claim about whole-relation completeness.
        pub start: u64 => { tag: 3, direction: Direction::Output, rules: [] };
        /// Original requested exclusive range end.
        pub end: u64 => { tag: 4, direction: Direction::Output, rules: [] };
        /// Actual rows emitted by the selected producer before successful exhaustion.
        pub rows: u64 => { tag: 5, direction: Direction::Output, rules: [] };
        /// Original checked Arrow stream for that complete selected range.
        pub ipc: Vec<u8> => { tag: 6, direction: Direction::Output, rules: [] };
    }
}

/// One graph over the actual service records, consumed by descriptor/client generation.
/// # Errors
/// Invalid stable tags, unresolved field closure or contradictory rules refuse generation.
pub fn engineering_projection() -> Result<ProjectionGraph, ProjectionError> {
    let mut graph = ProjectionGraph::default();
    let service = service(&mut graph)?;
    graph.service(service)?;
    Ok(graph)
}

/// Actual lightweight method declaration, extendable by the runtime's original native owners.
/// # Errors
/// Invalid source-selected roots, local rules or effect declarations refuse composition.
pub fn service(graph: &mut ProjectionGraph) -> Result<Service, ProjectionError> {
    Service::new("EngineeringService")
        .unary::<EngineeringDescribeRequest, EngineeringContract>(graph, "DescribeContract")?
        .effects(EffectFacts::OBSERVATION)?
        .unary::<EngineeringAdmitSource, EngineeringSourceOutcome>(graph, "AdmitSource")?
        .effects(EffectFacts::SOURCE_ADMISSION)?
        .unary::<EngineeringEditSource, EngineeringSourceOutcome>(graph, "EditSource")?
        .effects(EffectFacts::SOURCE_ADMISSION)?
        .unary::<EngineeringOperationKey, EngineeringSourceOutcome>(graph, "SourceStatus")?
        .effects(EffectFacts::OBSERVATION)?
        .unary::<EngineeringInspectSource, EngineeringDeclarationPage>(graph, "InspectSource")?
        .effects(EffectFacts::OBSERVATION)?
        .unary::<EngineeringSubmitRun, EngineeringRunStatus>(graph, "SubmitRun")?
        .effects(EffectFacts::NATIVE_EXECUTION)?
        .unary::<EngineeringRunKey, EngineeringRunStatus>(graph, "RunStatus")?
        .effects(EffectFacts::OBSERVATION)?
        .unary::<EngineeringRunKey, EngineeringRunStatus>(graph, "CancelRun")?
        .effects(EffectFacts::CANONICAL_COMMAND)?
        .unary::<EngineeringReadResult, EngineeringResultPage>(graph, "ReadResult")?
        .effects(EffectFacts::OBSERVATION)?
        .server_streaming::<EngineeringRunKey, EngineeringRunStatus>(graph, "ObserveRun")?
        .effects(EffectFacts::OBSERVATION)
}
