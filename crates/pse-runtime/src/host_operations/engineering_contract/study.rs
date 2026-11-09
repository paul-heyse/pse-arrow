// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Typed crossings to original study admission, creation, observation and protected cohorts.

use pse_contract::boundary_projection::{Direction, LocalRule, ProjectionError, ProjectionGraph, Service};
use pse_contract::operation_effects::EffectFacts;
use pse_ids::ContentHash;
use pse_model::document::Version;
use pse_model::engineering::{EngineeringContract, EngineeringDescribeRequest, EngineeringFailure, EngineeringOperationKey, EngineeringSourceSelection, OPERATION_ID_RULE};
use pse_model::generated::identities::StudyId;
use crate::workflow::{
    LazyStudyDefinition, PhysicalSource, ResearchSource, STUDY_DEFINITION_VERSION, StudyDefinition,
    StudyOutputDemand, StudyPoint, StudyPointDefinition, StudyVariants, WorkflowError,
};

macro_rules! prepare_request {
    ($name:ident, $wire:literal, $member:ident: $inputs:ty, [$($rule:expr),*]) => {
        pse_contract::projection_struct! {
    /// Admit one Study input capability against its exact retained source.
    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct $name {
        projection: { name: $wire, rust_path: crate::host_operations::engineering_contract::$name,
            version: 2, direction: Direction::Input, reserved_tags: [], reserved_names: [] };
        /// Original operation retaining this admission independently of its waiters.
        pub operation: String => { tag: 1, direction: Direction::Input, rules: [OPERATION_ID_RULE] };
        /// Original admitted source operation.
        pub source_operation: String => { tag: 2, direction: Direction::Input, rules: [OPERATION_ID_RULE] };
        /// Exact immutable local parent, never a latest-name selection.
        pub expected_source: String => { tag: 3, direction: Direction::Input, rules: [pse_model::engineering::LOCAL_REVISION_RULE] };
        /// Original summary and occurrence-detail demand.
        pub output: StudyOutputDemand => { tag: 4, direction: Direction::Input, rules: [LocalRule::Required] };
        /// Selected input capability.
        pub $member: $inputs => { tag: 5, direction: Direction::Input, rules: [$($rule),*] };
    }
        }
    };
}

prepare_request!(
    EngineeringPrepareExplicitStudy, "EngineeringPrepareExplicitStudy", points: Vec<StudyPoint>,
    [LocalRule::Length { min: 0, max: crate::workflow::MAXIMUM_STUDY_POINTS as u64 }]
);
prepare_request!(
    EngineeringPrepareExpandedStudy, "EngineeringPrepareExpandedStudy", variants: StudyVariants,
    [LocalRule::Required]
);

pse_contract::projection_struct! {
    /// Immutable native admitted definition selection, without unbounded occurrence materialization.
    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct EngineeringPreparedStudy {
        projection: { name: "EngineeringPreparedStudy", rust_path: crate::host_operations::engineering_contract::EngineeringPreparedStudy,
            version: 1, direction: Direction::Output, reserved_tags: [], reserved_names: [] };
        /// Original admission operation.
        pub operation: String => { tag: 1, direction: Direction::Output, rules: [OPERATION_ID_RULE] };
        /// Exact immutable admitted source.
        pub source: Option<EngineeringSourceSelection> => { tag: 2, direction: Direction::Output, rules: [] };
        /// Full original admitted definition hash, including complete scientific and source intent.
        pub preparation: ContentHash => { tag: 3, direction: Direction::Output, rules: [LocalRule::Required] };
        /// Complete native selected occurrence population, including lazy selected ordinals.
        pub occurrences: u64 => { tag: 4, direction: Direction::Output, rules: [LocalRule::UnsignedRange { min: 1, max: crate::workflow::MAXIMUM_STUDY_POINTS as u64 }] };
    }
}

macro_rules! restore_request {
    ($name:ident, $wire:literal, $definition:ty) => {
        pse_contract::projection_struct! {
    /// Retain an exact original portable definition for interrupted native creation reconciliation.
    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct $name {
        projection: { name: $wire, rust_path: crate::host_operations::engineering_contract::$name,
            version: 2, direction: Direction::Input, reserved_tags: [], reserved_names: [] };
        /// New local retention operation, not a new scientific execution identity.
        pub operation: String => { tag: 1, direction: Direction::Input, rules: [OPERATION_ID_RULE] };
        /// Original complete immutable preparation, not a mutable name.
        pub expected_preparation: ContentHash => { tag: 2, direction: Direction::Input, rules: [LocalRule::Required] };
        /// Actual source-owned portable definition. Original native start revalidates its full context.
        pub definition: $definition => { tag: 3, direction: Direction::Input, rules: [LocalRule::Required] };
    }
        }
    };
}

restore_request!(EngineeringRestoreExplicitStudy, "EngineeringRestoreExplicitStudy", ExplicitStudyDefinition);
restore_request!(EngineeringRestoreExpandedStudy, "EngineeringRestoreExpandedStudy", ExpandedStudyDefinition);

pse_contract::projection_struct! {
    /// Shared immutable source context of the two portable preparation capabilities.
    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct StudyDefinitionContext {
        projection: { name: "StudyDefinitionContext", rust_path: crate::host_operations::engineering_contract::StudyDefinitionContext,
            version: 2, direction: Direction::Both, reserved_tags: [], reserved_names: [] };
        /// Original research ancestry.
        pub research: Option<ResearchSource> => { tag: 1, direction: Direction::Both, rules: [] };
        /// Exact historical source context selected before admission.
        pub context: Option<pse_operations::canonical_contexts::ContextSelection> => { tag: 2, direction: Direction::Both, rules: [] };
        /// Interpretation of the original native durable document.
        pub version: Version<STUDY_DEFINITION_VERSION> => { tag: 3, direction: Direction::Both, rules: [] };
        /// Original physical source bundle.
        pub physical: PhysicalSource => { tag: 4, direction: Direction::Both, rules: [] };
        /// Exact immutable canonical modeling revision.
        pub modeling_revision: String => { tag: 5, direction: Direction::Both, rules: [] };
    }
}

impl StudyDefinitionContext {
    fn split(value: StudyDefinition) -> (Self, Option<LazyStudyDefinition>, Vec<StudyPointDefinition>) {
        let StudyDefinition { research, context, version, physical, modeling_revision, variants, points } = value;
        (Self { research, context, version, physical, modeling_revision }, variants, points)
    }

    fn join(self, variants: Option<LazyStudyDefinition>, points: Vec<StudyPointDefinition>) -> StudyDefinition {
        let Self { research, context, version, physical, modeling_revision } = self;
        StudyDefinition { research, context, version, physical, modeling_revision, variants, points }
    }
}

pse_contract::projection_struct! {
    /// Exact prepared explicit points, without any inline authoring contract.
    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct ExplicitStudyDefinition {
        projection: { name: "ExplicitStudyDefinition", rust_path: crate::host_operations::engineering_contract::ExplicitStudyDefinition,
            version: 2, direction: Direction::Both, reserved_tags: [], reserved_names: [] };
        /// Original context and interpretation.
        pub context: StudyDefinitionContext => { tag: 1, direction: Direction::Both, rules: [LocalRule::Required] };
        /// Exact original admitted points in order.
        pub points: Vec<StudyPointDefinition> => { tag: 2, direction: Direction::Both,
            rules: [LocalRule::Length { min: 0, max: crate::workflow::MAXIMUM_STUDY_POINTS as u64 }] };
    }
}

pse_contract::projection_struct! {
    /// Exact prepared source expansion and selected occurrences.
    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct ExpandedStudyDefinition {
        projection: { name: "ExpandedStudyDefinition", rust_path: crate::host_operations::engineering_contract::ExpandedStudyDefinition,
            version: 2, direction: Direction::Both, reserved_tags: [], reserved_names: [] };
        /// Original context and interpretation.
        pub context: StudyDefinitionContext => { tag: 1, direction: Direction::Both, rules: [LocalRule::Required] };
        /// Exact original parent-qualified expansion.
        pub variants: LazyStudyDefinition => { tag: 2, direction: Direction::Both, rules: [LocalRule::Required] };
    }
}

impl From<ExplicitStudyDefinition> for StudyDefinition {
    fn from(value: ExplicitStudyDefinition) -> Self {
        value.context.join(None, value.points)
    }
}

impl From<ExpandedStudyDefinition> for StudyDefinition {
    fn from(value: ExpandedStudyDefinition) -> Self {
        value.context.join(Some(value.variants), Vec::new())
    }
}

impl TryFrom<StudyDefinition> for ExplicitStudyDefinition {
    type Error = WorkflowError;

    fn try_from(value: StudyDefinition) -> Result<Self, Self::Error> {
        let (context, variants, points) = StudyDefinitionContext::split(value);
        if variants.is_some() {
            return Err(crate::workflow::contract("study was prepared by source expansion"));
        }
        Ok(Self { context, points })
    }
}

impl TryFrom<StudyDefinition> for ExpandedStudyDefinition {
    type Error = WorkflowError;

    fn try_from(value: StudyDefinition) -> Result<Self, Self::Error> {
        let (context, variants, points) = StudyDefinitionContext::split(value);
        let Some(variants) = variants.filter(|_| points.is_empty()) else {
            return Err(crate::workflow::contract("study was not prepared by source expansion"));
        };
        Ok(Self { context, variants })
    }
}

pse_contract::projection_struct! {
    /// Submit or reconcile exactly one original native study creation.
    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct EngineeringSubmitStudy {
        projection: { name: "EngineeringSubmitStudy", rust_path: crate::host_operations::engineering_contract::EngineeringSubmitStudy,
            version: 1, direction: Direction::Input, reserved_tags: [], reserved_names: [] };
        /// Caller operation fixed before submission or acknowledgement.
        pub operation: String => { tag: 1, direction: Direction::Input, rules: [OPERATION_ID_RULE] };
        /// Original retained study admission operation.
        pub preparation_operation: String => { tag: 2, direction: Direction::Input, rules: [OPERATION_ID_RULE] };
        /// Immutable native preparation, preventing same-name rebinding after service retirement.
        pub expected_preparation: ContentHash => { tag: 3, direction: Direction::Input, rules: [LocalRule::Required] };
        /// Exact caller-issued study identity; equal science never merges separate studies.
        pub study: StudyId => { tag: 4, direction: Direction::Input, rules: [LocalRule::Required] };
    }
}

pse_contract::projection_struct! {
    /// Original study lookup coordinates. Decoding alone grants no execution or result authority.
    #[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct EngineeringStudyKey {
        projection: { name: "EngineeringStudyKey", rust_path: crate::host_operations::engineering_contract::EngineeringStudyKey,
            version: 1, direction: Direction::Input, reserved_tags: [], reserved_names: [] };
        /// Exact original caller operation.
        pub operation: String => { tag: 1, direction: Direction::Input, rules: [OPERATION_ID_RULE] };
        /// Exact original caller-issued study.
        pub study: StudyId => { tag: 2, direction: Direction::Input, rules: [LocalRule::Required] };
    }
}

pse_contract::projection_struct! {
    /// Original native creation knowledge; an absent header does not establish completion.
    #[derive(Clone, Debug)]
    pub struct EngineeringStudyStatus {
        projection: { name: "EngineeringStudyStatus", rust_path: crate::host_operations::engineering_contract::EngineeringStudyStatus,
            version: 1, direction: Direction::Output, reserved_tags: [], reserved_names: [] };
        /// Exact original operation.
        pub operation: String => { tag: 1, direction: Direction::Output, rules: [OPERATION_ID_RULE] };
        /// Exact caller-issued study identity.
        pub study: StudyId => { tag: 2, direction: Direction::Output, rules: [LocalRule::Required] };
        /// Actual compact canonical header; absent during an unacknowledged creation prefix.
        pub canonical: Option<pse_operations::canonical_studies::StudySummary> => { tag: 3, direction: Direction::Output, rules: [] };
        /// Original retained native refusal, independent of canonical settlement knowledge.
        pub failure: Option<EngineeringFailure> => { tag: 4, direction: Direction::Output, rules: [] };
    }
}

pse_contract::projection_struct! {
    /// Observe or reconnect the original bounded study observer; never owns its cancellation lease.
    #[derive(Clone, Debug)]
    pub struct EngineeringObserveStudy {
        projection: { name: "EngineeringObserveStudy", rust_path: crate::host_operations::engineering_contract::EngineeringObserveStudy,
            version: 1, direction: Direction::Input, reserved_tags: [], reserved_names: [] };
        /// Original caller identity verified against its persisted native parent request.
        pub original: EngineeringStudyKey => { tag: 1, direction: Direction::Input, rules: [LocalRule::Required] };
        /// Original bounded selected population and telemetry allowance.
        pub request: crate::workflow::StudyObservationRequest => { tag: 2, direction: Direction::Input, rules: [LocalRule::Required] };
        /// Exact native reconnect cursor, including original source/attempt generations.
        pub cursor: Option<crate::workflow::StudyObservationCursor> => { tag: 3, direction: Direction::Input, rules: [] };
    }
}

pse_contract::projection_struct! {
    /// Retain a live original frozen-cohort capability independently of its serialized receipt.
    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct EngineeringFreezeStudyCohort {
        projection: { name: "EngineeringFreezeStudyCohort", rust_path: crate::host_operations::engineering_contract::EngineeringFreezeStudyCohort,
            version: 1, direction: Direction::Input, reserved_tags: [], reserved_names: [] };
        /// Original operation retaining the actual native protection owner.
        pub operation: String => { tag: 1, direction: Direction::Input, rules: [OPERATION_ID_RULE] };
        /// Actual complete finite native selection, including unavailable siblings.
        pub request: crate::workflow::StudyCohortRequest => { tag: 2, direction: Direction::Input, rules: [LocalRule::Required] };
    }
}

pse_contract::projection_struct! {
    /// Read one compact header page from the exact retained live native capture.
    #[derive(Clone, Debug)]
    pub struct EngineeringStudyHeaderPage {
        projection: { name: "EngineeringStudyHeaderPage", rust_path: crate::host_operations::engineering_contract::EngineeringStudyHeaderPage,
            version: 1, direction: Direction::Input, reserved_tags: [], reserved_names: [] };
        /// Original freeze operation retaining the capture's actual local capability.
        pub operation: String => { tag: 1, direction: Direction::Input, rules: [OPERATION_ID_RULE] };
        /// Actual native header allowance, bounded to64 before the canonical read.
        pub request: crate::workflow::StudyCohortHeaderRequest => { tag: 2, direction: Direction::Input, rules: [LocalRule::Required] };
    }
}

pse_contract::projection_struct! {
    /// Read one finite page from the exact retained live native capture.
    #[derive(Clone, Debug)]
    pub struct EngineeringStudyCohortPage {
        projection: { name: "EngineeringStudyCohortPage", rust_path: crate::host_operations::engineering_contract::EngineeringStudyCohortPage,
            version: 1, direction: Direction::Input, reserved_tags: [], reserved_names: [] };
        /// Original freeze operation; a capture ID or receipt does not replace this live owner.
        pub operation: String => { tag: 1, direction: Direction::Input, rules: [OPERATION_ID_RULE] };
        /// Native capture-bound cursor and finite page allowance.
        pub request: crate::workflow::StudyCohortPageRequest => { tag: 2, direction: Direction::Input, rules: [LocalRule::Required] };
    }
}

pse_contract::projection_struct! {
    /// Read a selected original relation through its actual frozen-cohort result protection.
    #[derive(Clone, Debug)]
    pub struct EngineeringReadStudyResult {
        projection: { name: "EngineeringReadStudyResult", rust_path: crate::host_operations::engineering_contract::EngineeringReadStudyResult,
            version: 1, direction: Direction::Input, reserved_tags: [], reserved_names: [] };
        /// Original freeze operation retaining the native capture capability.
        pub operation: String => { tag: 1, direction: Direction::Input, rules: [OPERATION_ID_RULE] };
        /// Exact global native capture member position; unavailable siblings remain explicit.
        pub position: usize => { tag: 2, direction: Direction::Input, rules: [] };
        /// Exact original registered relation.
        pub relation: String => { tag: 3, direction: Direction::Input, rules: [pse_model::engineering::RESULT_RELATION_RULE] };
        /// Inclusive original row coordinate.
        pub start: u64 => { tag: 4, direction: Direction::Input, rules: [] };
        /// Exclusive original row coordinate; the complete range must fit the finite allowance.
        pub end: u64 => { tag: 5, direction: Direction::Input, rules: [] };
        /// Maximum complete Arrow IPC response, bounded before native materialization.
        pub maximum_bytes: usize => { tag: 6, direction: Direction::Input, rules: [LocalRule::UnsignedRange { min: 1, max: 16 << 20 }] };
    }
}

pse_contract::projection_struct! {
    /// Complete selected Arrow IPC response with exact original captured result attribution.
    #[derive(Clone, Debug)]
    pub struct EngineeringStudyResultPage {
        projection: { name: "EngineeringStudyResultPage", rust_path: crate::host_operations::engineering_contract::EngineeringStudyResultPage,
            version: 1, direction: Direction::Output, reserved_tags: [], reserved_names: [] };
        /// Exact original live native capture used for this protected read.
        pub capture: String => { tag: 1, direction: Direction::Output, rules: [] };
        /// Original global member position in that immutable cut.
        pub position: usize => { tag: 2, direction: Direction::Output, rules: [] };
        /// Exact original canonical run selected by the native reader.
        pub run: String => { tag: 3, direction: Direction::Output, rules: [] };
        /// Exact original attempt; a retry cannot substitute another result.
        pub attempt: String => { tag: 4, direction: Direction::Output, rules: [] };
        /// Exact original closed manifest retained by the frozen native cut.
        pub manifest: String => { tag: 5, direction: Direction::Output, rules: [] };
        /// Original registered relation, preserving its physical Arrow schema.
        pub relation: String => { tag: 6, direction: Direction::Output, rules: [pse_model::engineering::RESULT_RELATION_RULE] };
        /// Inclusive original row coordinate.
        pub start: u64 => { tag: 7, direction: Direction::Output, rules: [] };
        /// Exclusive original row coordinate.
        pub end: u64 => { tag: 8, direction: Direction::Output, rules: [] };
        /// Actual completely exhausted selected row count.
        pub rows: u64 => { tag: 9, direction: Direction::Output, rules: [] };
        /// Complete Arrow IPC stream; exceeding the finite allowance refuses publication.
        pub ipc: Vec<u8> => { tag: 10, direction: Direction::Output, rules: [LocalRule::Length { min: 0, max: 16 << 20 }] };
    }
}

pse_contract::projection_struct! {
    /// Local capture admission has closed; escaped native response owners may still retain it.
    #[derive(Clone, Debug)]
    pub struct EngineeringCohortRelease {
        projection: { name: "EngineeringCohortRelease", rust_path: crate::host_operations::engineering_contract::EngineeringCohortRelease,
            version: 1, direction: Direction::Output, reserved_tags: [], reserved_names: [] };
        /// Exact original freeze operation whose local capability has been closed.
        pub operation: String => { tag: 1, direction: Direction::Output, rules: [OPERATION_ID_RULE] };
        /// The frontend will refuse further page/result reads under this operation.
        pub local_admission_closed: bool => { tag: 2, direction: Direction::Output, rules: [] };
    }
}

/// Versioned source-owned explicit preparation procedure group.
pub const EXPLICIT_STUDY_SERVICE: &str = "ExplicitStudyPreparationV2";
/// Versioned source-owned expansion preparation procedure group.
pub const EXPANDED_STUDY_SERVICE: &str = "ExpandedStudyPreparationV2";
/// Versioned source-owned shared Study control procedure group.
pub const STUDY_CONTROL_SERVICE: &str = "StudyControlV2";

pub(super) fn services(graph: &mut ProjectionGraph) -> Result<(), ProjectionError> {
    explicit_services(graph)?;
    let expanded = Service::new(EXPANDED_STUDY_SERVICE)
        .unary::<EngineeringPrepareExpandedStudy, EngineeringPreparedStudy>(graph, "PrepareExpandedStudy")?
        .effects(EffectFacts::SOURCE_ADMISSION)?
        .unary::<EngineeringRestoreExpandedStudy, EngineeringPreparedStudy>(graph, "RestoreExpandedStudy")?
        .effects(EffectFacts::SOURCE_ADMISSION)?
        .unary::<EngineeringOperationKey, ExpandedStudyDefinition>(graph, "ReadPreparedExpandedStudy")?
        .effects(EffectFacts::OBSERVATION)?;
    graph.service(expanded)
}

pub(super) fn explicit_services(graph: &mut ProjectionGraph) -> Result<(), ProjectionError> {
    let explicit = Service::new(EXPLICIT_STUDY_SERVICE)
        .unary::<EngineeringPrepareExplicitStudy, EngineeringPreparedStudy>(graph, "PrepareExplicitStudy")?
        .effects(EffectFacts::SOURCE_ADMISSION)?
        .unary::<EngineeringRestoreExplicitStudy, EngineeringPreparedStudy>(graph, "RestoreExplicitStudy")?
        .effects(EffectFacts::SOURCE_ADMISSION)?
        .unary::<EngineeringOperationKey, ExplicitStudyDefinition>(graph, "ReadPreparedExplicitStudy")?
        .effects(EffectFacts::OBSERVATION)?;
    graph.service(explicit)?;
    let control = Service::new(STUDY_CONTROL_SERVICE)
        .unary::<EngineeringDescribeRequest, EngineeringContract>(graph, "DescribeStudyContract")?
        .effects(EffectFacts::OBSERVATION)?
        .unary::<crate::workflow::StudyWorkbenchRequest, crate::workflow::StudyWorkbenchPage>(
            graph,
            "BrowseStudies",
        )?
        .effects(EffectFacts::OBSERVATION)?
        .unary::<EngineeringSubmitStudy, EngineeringStudyStatus>(graph, "SubmitStudy")?
        .effects(EffectFacts::NATIVE_EXECUTION)?
        .unary::<EngineeringStudyKey, EngineeringStudyStatus>(graph, "StudyStatus")?
        .effects(EffectFacts::OBSERVATION)?
        .unary::<EngineeringStudyKey, EngineeringStudyStatus>(graph, "CancelStudy")?
        .effects(EffectFacts::CANONICAL_COMMAND)?
        .server_streaming::<EngineeringObserveStudy, crate::workflow::StudyObservationPage>(
            graph,
            "ObserveStudy",
        )?
        .effects(EffectFacts::OBSERVATION)?
        .unary::<EngineeringFreezeStudyCohort, crate::workflow::StudyCohortReceipt>(
            graph,
            "FreezeStudyCohort",
        )?
        .effects(EffectFacts::CAPTURE)?
        .unary::<EngineeringStudyHeaderPage, crate::workflow::StudyCohortHeaderPage>(
            graph,
            "ReadStudyHeaders",
        )?
        .effects(EffectFacts::OBSERVATION)?
        .unary::<EngineeringStudyCohortPage, crate::workflow::StudyCohortMemberPage>(
            graph,
            "ReadStudyMembers",
        )?
        .effects(EffectFacts::OBSERVATION)?
        .unary::<EngineeringReadStudyResult, EngineeringStudyResultPage>(graph, "ReadStudyResult")?
        .effects(EffectFacts::OBSERVATION)?
        .unary::<EngineeringOperationKey, EngineeringCohortRelease>(graph, "ReleaseStudyCohort")?
        .effects(EffectFacts {
            idempotent: true,
            ..EffectFacts::RETENTION_RELEASE
        })?;
    graph.service(control)
}
