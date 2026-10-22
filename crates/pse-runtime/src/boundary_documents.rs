// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Exported serde owners shared by schema publication and native document projection.

#[path = "boundary_document_wire.rs"]
mod wire;
pub use wire::{Versioned, VersionedValue, Wire, check_header};

/// Visit exported names, public Rust owners, schema slugs and admitted directions.
/// A round-trip document has independent input and output generator contexts.
#[macro_export]
macro_rules! for_each_boundary_document {
    ($callback:ident) => { $callback! {
            (InitializationOverrides, pse_runtime::workflow::InitializationOverrides, "initialization-overrides", Both),
            (ModelingNonlinearPolicy, pse_runtime::workflow::ModelingNonlinearPolicy, "nonlinear-explanation-policy", Both),
            (ModelingDiagnosticPolicy, pse_runtime::workflow::ModelingDiagnosticPolicy, "modeling-diagnostic-policy", Both),
            (WarmStartSnapshot, pse_backend_native::solve::WarmStartSnapshot, "warm-start-snapshot", Both),
            (DiagnosticCauseDocument, pse_runtime::workflow::DiagnosticCauseDocument, "diagnostic-cause", Both),
            (DiagnosticSpanDocument, pse_runtime::workflow::DiagnosticSpanDocument, "diagnostic-span", Both),
            (DiagnosticNoteDocument, pse_runtime::workflow::DiagnosticNoteDocument, "diagnostic-note", Both),
            (DiagnosticAnnotationDocument, pse_runtime::workflow::DiagnosticAnnotationDocument, "diagnostic-annotation", Both),
            (DiagnosticContextDocument, pse_runtime::workflow::DiagnosticContextDocument, "diagnostic-context", Both),
            (CacheReport, pse_engine::cache_service::CacheReport, "cache-report", Both),
            (ResourceReport, pse_runtime::ResourceReport, "resource-report", Both),
            (OperationId, pse_diagnostics::observation::OperationId, "diagnostic-operation-id", Both),
            (OperationSnapshot, pse_diagnostics::observation::OperationSnapshot, "operation-snapshot", Both),
            (OperationPage, pse_diagnostics::observation::OperationPage, "operation-page", Both),
            (CheckpointIdentity, pse_runtime::observation::CheckpointIdentity, "checkpoint-identity", Both),
            (ReceiverCheckpoint, pse_runtime::observation::ReceiverCheckpoint, "receiver-checkpoint", Both),
            (ProcessCheckpoint, pse_runtime::observation::ProcessCheckpoint, "process-checkpoint", Both),
            (SourceRecordingLaunchDocument, pse_runtime::tooling::review_launch::SourceRecordingLaunchDocument, "source-recording-launch-document", Both),
            (SourceRecordingLaunchRequest, pse_runtime::tooling::review_launch::SourceRecordingLaunchRequest, "source-recording-launch-request", Both),
            (PackExportRequest, pse_runtime::tooling::review_pack::PackExportRequest, "pack-export-request", Both),
            (ReviewExportDeployment, pse_runtime::workflow::ReviewExportDeployment, "review-export-deployment", Both),
            (DiagnosticCommandRequest, pse_runtime::workflow::DiagnosticCommandRequest, "diagnostic-command-request", Both),
            (DiagnosticExecutionRequest, pse_runtime::workflow::DiagnosticExecutionRequest, "diagnostic-execution-request", Both),
            (DiagnosticExecutionDocument, pse_runtime::workflow::DiagnosticExecutionDocument, "diagnostic-execution-document", Both),
            (AnalysisRequest, pse_runtime::tooling::review_analysis::AnalysisRequest, "analysis-request", Both),
            (AnalysisDocument, pse_runtime::tooling::review_analysis::AnalysisDocument, "analysis-document", Both),
            (PackSelection, pse_runtime::tooling::review_pack::PackSelection, "pack-selection", Both),
            (PackIndex, pse_runtime::tooling::review_pack::PackIndex, "pack-index", Both),
            (ReviewTableDocument, pse_runtime::tooling::review_query::ReviewTableDocument, "review-table-document", Both),
            (ReviewTableRequest, pse_runtime::tooling::review_query::ReviewTableRequest, "review-table-request", Both),
            (ReviewQueryRequest, pse_runtime::tooling::review_query::ReviewQueryRequest, "review-query-request", Both),
            (ReviewQueryDocument, pse_runtime::tooling::review_query::ReviewQueryDocument, "review-query-document", Both),
            (ReviewQuerySpecification, pse_runtime::tooling::review_query::ReviewQuerySpecification, "review-query-specification", Both),
            (SourceRecordingRequest, pse_runtime::tooling::capture::SourceRecordingRequest, "source-recording-request", Both),
            (ReviewHandoffReceipt, pse_runtime::tooling::capture::ReviewHandoffReceipt, "review-handoff-receipt", Both),
            (SourceRecordingAcknowledgement, pse_runtime::tooling::capture::SourceRecordingAcknowledgement, "source-recording-acknowledgement", Both),
            (FrozenEvidenceSet, pse_runtime::tooling::frozen_inspection::FrozenEvidenceSet, "frozen-evidence-set", Both),
            (FrozenInspectionRequest, pse_runtime::tooling::frozen_inspection::FrozenInspectionRequest, "frozen-inspection-request", Both),
            (FrozenInspectionDocument, pse_runtime::tooling::frozen_inspection::FrozenInspectionDocument, "frozen-inspection-document", Both),
            (ReviewContextRequest, pse_runtime::workflow::ReviewContextRequest, "review-context-request", Both),
            (ReviewContextDocument, pse_runtime::workflow::ReviewContextDocument, "review-context-document", Both),
            (ReviewScientificCoordinates, pse_runtime::workflow::ReviewScientificCoordinates, "review-scientific-coordinates", Both),
            (ReviewInventoryRequest, pse_runtime::workflow::ReviewInventoryRequest, "review-inventory-request", Both),
            (ReviewInventorySelection, pse_runtime::workflow::ReviewInventorySelection, "review-inventory-selection", Both),
            (ReviewInventoryPage, pse_runtime::workflow::ReviewInventoryPage, "review-inventory-page", Both),
            (InspectionRequest, pse_runtime::tooling::inspection::InspectionRequest, "inspection-request", Both),
            (InspectionDocument, pse_runtime::tooling::inspection::InspectionDocument, "inspection-document", Both),
            (DiagnosticListRequest, pse_runtime::tooling::inspection::DiagnosticListRequest, "diagnostic-list-request", Both),
            (DiagnosticProcessPage, pse_runtime::tooling::inspection::DiagnosticProcessPage, "diagnostic-process-page", Both),
            (CaptureRequest, pse_runtime::tooling::capture::CaptureRequest, "capture-request", Both),
            (DiagnosticOrigin, pse_runtime::workflow::DiagnosticOrigin, "diagnostic-origin", Both),
            (LaunchCaptureRequest, pse_runtime::tooling::capture::LaunchCaptureRequest, "launch-capture-request", Both),
            (CaptureDocument, pse_runtime::tooling::capture::CaptureDocument, "capture-document", Both),
            (DetailRequest, pse_runtime::observation_detail::DetailRequest, "detail-request", Both),
            (DetailDocument, pse_runtime::observation_detail::DetailDocument, "detail-document", Both),
            (DetailRecordPage, pse_runtime::observation_detail::DetailRecordPage, "detail-record-page", Both),
            (ModelingSourceMapDocument, pse_runtime::workflow::ModelingSourceMapDocument, "modeling-source-map", Both),
            (CoherentRecord, pse_diagnostics::observation::CoherentRecord, "coherent-record", Both),
            (InvocationEvidenceDocument, pse_runtime::invocation::InvocationEvidenceDocument, "invocation-evidence", Both),
            (InvocationComparisonDocument, pse_runtime::tooling::inspection_comparison::InvocationComparisonDocument, "invocation-comparison", Both),
            (ProgressCheckpointDocument, pse_runtime::workflow::ProgressCheckpointDocument, "progress-checkpoint", Both),
            (ManagedStudyDiagnosticsDocument, pse_runtime::workflow::ManagedStudyDiagnosticsDocument, "managed-study-diagnostics", Both),
            (TableName, pse_runtime::TableName, "table-name", Both),
            (BuildInfo, pse_buildinfo::BuildInfo, "build-info", Both),
            (ProgressEventDocument, pse_runtime::workflow::ProgressEventDocument, "progress-event", Both),
            (ProgressCoverage, pse_runtime::workflow::ProgressCoverage, "progress-coverage", Both),
            (RouteDocument, pse_runtime::workflow::RouteDocument, "route", Both),
            (EligibilityDocument, pse_runtime::workflow::EligibilityDocument, "eligibility", Both),
            (Completion, pse_runtime::workflow::Completion, "completion", Both),
            (AttemptSummary, pse_runtime::workflow::AttemptSummaryDocument, "attempt-summary", Both),
            (ReplayDocument, pse_runtime::workflow::ReplayDocument, "scientific-replay", Output),
            (SolverAdviceInputs, pse_runtime::math::solves::SolverAdviceInputs, "solver-advice-inputs", Both),
            (FrozenAdvice, pse_model::strategy::advice::FrozenAdvice, "frozen-advice", Both),
            (OperationSource, pse_runtime::workflow::OperationSource, "operation-source", Both),
            (PreparationSettings, pse_runtime::workflow::PreparationSettings, "preparation-settings", Both),
            (PhysicalSimulationRequest, pse_runtime::workflow::PhysicalSimulationRequest, "physical-simulation-request", Both),
            (PhysicalFitRequest, pse_runtime::workflow::PhysicalFitRequest, "physical-fit-request", Both),
            (ParameterCovariance, pse_runtime::math::settings::ParameterCovariance, "parameter-covariance", Both),
            (FitExecutionDocument, pse_runtime::workflow::FitExecutionDocument, "fit-execution", Output),
            (FitProfileDocument, pse_runtime::workflow::FitProfileDocument, "fit-profiles", Output),
            (FitDeclarations, pse_runtime::workflow::FitDeclarations, "fit-declarations", Both),
            (RunControls, pse_runtime::workflow::RunControls, "run-controls", Both),
            (StudyRunControls, pse_runtime::workflow::StudyRunControls, "study-run-controls", Both),
            (StudyWaitControls, pse_runtime::workflow::StudyWaitControls, "study-wait-controls", Both),
            (KnowledgeControls, pse_runtime::workflow::KnowledgeControls, "knowledge-controls", Both),
            (DiagnosticSamplesControls, pse_runtime::workflow::DiagnosticSamplesControls, "diagnostic-samples-controls", Both),
            (JacobianDiagnosticControls, pse_runtime::workflow::JacobianDiagnosticControls, "jacobian-diagnostic-controls", Both),
            (LinearDiagnosticControls, pse_runtime::workflow::LinearDiagnosticControls, "linear-diagnostic-controls", Both),
            (PureConformanceControls, pse_runtime::workflow::PureConformanceControls, "pure-conformance-controls", Both),
            (ConformanceControls, pse_runtime::workflow::ConformanceControls, "conformance-controls", Both),
            (DeclarationEdit, pse_runtime::workflow::DeclarationEdit, "declaration-edit", Both),
            (SourceDelta, pse_runtime::workflow::SourceDelta, "source-delta", Both),
            (SourceSettlement, pse_runtime::workflow::SourceSettlement, "source-settlement", Both),
            (DeclarationInventory, pse_runtime::workflow::DeclarationInventory, "declaration-inventory", Both),
            (ModelingInspection, pse_runtime::workflow::ModelingInspection, "modeling-inspection", Both),
            (ConicRequest, pse_runtime::workflow::ConicRequest, "conic-request", Both),
            (RecycleRequest, pse_runtime::workflow::RecycleRequest, "recycle-request", Both),
            (RecycleRestartRequest, pse_runtime::workflow::RecycleRestartRequest, "recycle-restart-request", Both),
            (FlowSelectionDocument, pse_runtime::math::flows::FlowSelectionDocument, "flow-selection", Both),
            (FlowGraphDocument, pse_runtime::math::flows::FlowGraphDocument, "flow-graph", Both),
            (TearSelectionDocument, pse_runtime::math::flows::TearSelectionDocument, "tear-selection", Both),
            (InitializationDocument, pse_runtime::workflow::InitializationDocument, "initialization", Both),
            (SolveSettings, pse_runtime::math::settings::SolveSettings, "solve-settings", Both),
            (BlockSolveOverride, pse_runtime::math::settings::BlockSolveOverride, "block-solve-override", Both),
            (EffortPolicy, pse_model::strategy::EffortPolicy, "effort-policy", Both),
            (NumericalStrategyDocument, pse_model::strategy::NumericalStrategyDocument, "numerical-strategy", Both),
            (CompositionRequest, pse_model::strategy::CompositionRequest, "composition-request", Both),
            (BackendSettings, pse_backend_native::execution::BackendSettings, "backend-settings", Both),
            (DiffsolSettings, pse_backend_native::dynamics::DiffsolSettings, "diffsol-settings", Both),
            (IdasSettings, pse_backend_native::dynamics::IdasSettings, "idas-settings", Both),
            (AdjointSettings, pse_backend_native::dynamics::AdjointSettings, "adjoint-settings", Both),
            (StudyDefinition, pse_runtime::workflow::StudyDefinition, "study-definition", Both),
            (StudyRequest, pse_runtime::workflow::StudyRequest, "study-request", Both),
            (StudyStatus, pse_runtime::workflow::StudyStatus, "study-status", Both),
            (StudyResults, pse_runtime::workflow::StudyResults, "study-results", Both),
            (AnalysisControls, pse_runtime::workflow::AnalysisControls, "analysis-controls", Both),
            (ResultReclamationPage, pse_operations::canonical_result_retention::ResultReclamationPage, "result-reclamation-page", Both),
            (HistoryOwner, pse_operations::canonical_history::HistoryOwner, "history-owner", Both),
            (HistoryPurpose, pse_operations::canonical_history::HistoryPurpose, "history-purpose", Both),
            (HistoryObligation, pse_operations::canonical_history::HistoryObligation, "history-obligation", Both),
            (HistoryReclamationPage, pse_operations::canonical_history::HistoryReclamationPage, "history-reclamation-page", Both),
            (StudyCancel, pse_runtime::workflow::StudyCancel, "study-cancel", Both),
            (PointOutcome, pse_runtime::workflow::PointOutcome, "study-point-outcome", Both),
            (Conclusion, pse_runtime::workflow::StudyConclusion, "study-conclusion", Both),
            (BoundaryDiagnostic, pse_runtime::workflow::BoundaryDiagnostic, "boundary-diagnostic", Both),
            (TerminationDetail, pse_runtime::workflow::TerminationDetail, "termination-detail", Both),
            (FitUncertainty, pse_runtime::workflow::FitUncertainty, "fit-uncertainty", Both),
            (PreparationCounts, pse_runtime::math::PreparationCounts, "preparation-counts", Both),
            (ProcessLocator, pse_runtime::observation::ProcessLocator, "process-locator", Both),
            (ComparisonPolicy, pse_runtime::tooling::inspection_comparison::ComparisonPolicy, "comparison-policy", Both),
            (SourceLocation, pse_model::diagnostic::SourceLocation, "source-location", Both),
            (IneligibleDocument, pse_runtime::workflow::IneligibleDocument, "ineligible-document", Both),
            (NumericalPolicy, pse_model::numerics::NumericalPolicy, "numerical-policy", Both),
            (RunSettings, pse_runtime::workflow::RunSettings, "conformance-run-settings", Both),
            (ConformanceRun, pse_runtime::workflow::ConformanceRun, "conformance-run", Both),
            (ConformanceManifest, pse_runtime::workflow::ConformanceManifest, "conformance-manifest", Both),
            (LeasePolicy, pse_runtime::workflow::LeasePolicy, "lease-policy", Both),
            (ArtifactIntent, pse_operations::canonical_artifacts::Artifact, "artifact-intent", Both),
            (SolverCapability, pse_model::generated::runtime::solver_capabilities::Row, "solver-capability", Output),
            (StudyPointDefinition, pse_runtime::workflow::StudyPointDefinition, "study-point-definition", Output),
            (LazyStudyDefinition, pse_runtime::workflow::LazyStudyDefinition, "lazy-study-definition", Output),
            (AdmittedBinding, pse_runtime::workflow::AdmittedBinding, "admitted-binding", Output),
    } };
}

/// Direction of one actual serde schema, independent of its Rust type name.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    /// Actual deserialization contract.
    Input,
    /// Actual serialization contract.
    Output,
}
/// One direction-qualified schema from the actual compiled owner.
#[derive(Clone, Debug, serde::Serialize)]
pub struct DocumentSchema {
    /// Exported Python/schema name, checked against the owner's schema title.
    pub name: &'static str,
    /// Stable published schema slug.
    pub slug: &'static str,
    /// Actual public Rust path used only for generated boundary implementations.
    #[serde(skip)]
    pub rust_type: &'static str,
    /// Separate input/output interpretation.
    pub direction: Direction,
    /// Owning standalone version when the schema declares one.
    pub version: Option<u32>,
    /// Actual standalone wire-owner selection, shared with native projection.
    pub wire: Wire,
    /// Complete schema derived in its direction-specific context.
    pub schema: serde_json::Value,
}
/// Export names come from the catalogue; generic schema names come from their actual
/// instantiations. Apply the same substitution to nested definitions and references.
struct ExportNames<'a>(&'a std::collections::BTreeMap<String, &'static str>);
impl schemars::transform::Transform for ExportNames<'_> {
    fn transform(&mut self, schema: &mut schemars::Schema) {
        if let Some(title) = schema.get("title").and_then(serde_json::Value::as_str).and_then(|title| self.0.get(title)) {
            schema.insert("title".into(), (*title).into());
        }
        if let Some(reference) = schema.get("$ref").and_then(serde_json::Value::as_str).and_then(|reference| reference.strip_prefix("#/$defs/")) {
            let name = reference.replace("~1", "/").replace("~0", "~");
            if let Some(export) = self.0.get(&name) {
                schema.insert("$ref".into(), format!("#/$defs/{}", export.replace('~', "~0").replace('/', "~1")).into());
            }
        }
        if let Some(definitions) = schema.get_mut("$defs").and_then(serde_json::Value::as_object_mut) {
            for (original, export) in self.0 {
                if original != export && !definitions.contains_key(*export) {
                    if let Some(definition) = definitions.remove(original) {
                        definitions.insert((*export).into(), definition);
                    }
                }
            }
        }
        schemars::transform::transform_subschemas(self, schema);
    }
}
fn schema<T: schemars::JsonSchema>(name: &'static str, slug: &'static str, rust_type: &'static str, direction: Direction) -> DocumentSchema {
    let settings = schemars::generate::SchemaSettings::draft2020_12();
    let settings = match direction {
        Direction::Input => settings.for_deserialize(),
        Direction::Output => settings.for_serialize(),
    };
    let mut schema = settings.into_generator().into_root_schema_for::<T>();
    schemars::transform::Transform::transform(&mut ExportNames(&export_names()), &mut schema);
    let mut schema = schema.to_value();
    schema.sort_all_objects();
    let version = schema["properties"]["version"]["const"].as_u64().and_then(|version| u32::try_from(version).ok());
    DocumentSchema { name, slug, rust_type, direction, version, wire: Wire::Native, schema }
}
fn export_names() -> std::collections::BTreeMap<String, &'static str> {
    macro_rules! aliases {
        ($(($name:ident, $owner:ty, $slug:literal, $direction:ident)),* $(,)?) => {
            [$( (<$owner as schemars::JsonSchema>::schema_name().into_owned(), stringify!($name)), )*].into_iter().collect()
        };
    }
    crate::for_each_boundary_document!(aliases)
}
/// Derive each root with a fresh direction-specific schemars generator.
pub fn schemas() -> Vec<DocumentSchema> {
    fn input<T: schemars::JsonSchema + serde::de::DeserializeOwned>(name: &'static str, slug: &'static str, owner: &'static str) -> DocumentSchema {
        schema::<T>(name, slug, owner, Direction::Input)
    }
    fn output<T: schemars::JsonSchema + serde::Serialize>(name: &'static str, slug: &'static str, owner: &'static str) -> DocumentSchema {
        schema::<T>(name, slug, owner, Direction::Output)
    }
    macro_rules! collect {
        ($(($name:ident, $owner:ty, $slug:literal, $direction:ident)),* $(,)?) => {{
            let mut schemas = Vec::new();
            macro_rules! entry {
                ($target:ident, $export:ident, $ty:ty, $file:literal, Both) => {
                    $target.push(input::<$ty>(stringify!($export), $file, stringify!($ty)));
                    $target.push(output::<$ty>(stringify!($export), $file, stringify!($ty)));
                };
                ($target:ident, $export:ident, $ty:ty, $file:literal, Output) => {
                    $target.push(output::<$ty>(stringify!($export), $file, stringify!($ty)));
                };
                ($target:ident, $export:ident, $ty:ty, $file:literal, Input) => {
                    $target.push(input::<$ty>(stringify!($export), $file, stringify!($ty)));
                };
            }
            $(entry!(schemas, $name, $owner, $slug, $direction);)*
            schemas.sort_by_key(|schema| (schema.slug, schema.direction));
            schemas
        }};
    }
    crate::for_each_boundary_document!(collect)
}
/// Exact actual compiled catalogue identity; Rust paths are not identity content.
/// # Errors
/// Canonical framing refused a schema value.
pub fn fingerprint() -> Result<pse_ids::ContentHash, pse_ids::document::FrameError> {
    static FINGERPRINT: std::sync::OnceLock<Result<pse_ids::ContentHash, pse_ids::document::FrameError>> = std::sync::OnceLock::new();
    FINGERPRINT.get_or_init(|| pse_ids::document::of(pse_ids::Frame::BoundarySchemaSetV1, &schemas())).clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exported_names_reach_nested_generics_without_rewriting_document_values() {
        use schemars::transform::Transform;
        let names = std::collections::BTreeMap::from([("Internal/Generic~Name".into(), "Public")]);
        let mut schema = schemars::json_schema!({
            "title": "Internal/Generic~Name",
            "$defs": {"Internal/Generic~Name": {"type": "object"}},
            "properties": {"nested": {"$ref": "#/$defs/Internal~1Generic~0Name"}},
            "const": {"title": "Internal/Generic~Name", "$ref": "#/$defs/Internal~1Generic~0Name"}
        });
        ExportNames(&names).transform(&mut schema);
        assert_eq!(schema.as_value()["title"], "Public");
        assert!(schema.as_value()["$defs"].get("Public").is_some());
        assert_eq!(schema.as_value()["properties"]["nested"]["$ref"], "#/$defs/Public");
        assert_eq!(schema.as_value()["const"]["title"], "Internal/Generic~Name");
        assert_eq!(schema.as_value()["const"]["$ref"], "#/$defs/Internal~1Generic~0Name");
        assert_ne!(
            <crate::workflow::FitPreparationDocument as schemars::JsonSchema>::schema_name(),
            <crate::workflow::FitPreparationDocument<crate::workflow::PhysicalSimulationRequest> as schemars::JsonSchema>::schema_name(),
        );
    }
    #[test]
    fn catalogue_has_distinct_directions_owner_titles_and_completion_version() {
        let roots = schemas();
        let mut names = std::collections::BTreeSet::new();
        for root in &roots {
            assert!(names.insert((root.slug, root.direction)));
            assert_eq!(root.schema["title"], root.name, "{}", root.slug);
            if root.slug == "completion" { assert_eq!(root.version, Some(1)); }
        }
        assert_eq!(fingerprint().unwrap(), fingerprint().unwrap());
        let expected = pse_ids::document::of(pse_ids::Frame::BoundarySchemaSetV1, &roots).unwrap();
        let mut changed = roots;
        changed[0].direction = Direction::Output;
        assert_ne!(expected, pse_ids::document::of(pse_ids::Frame::BoundarySchemaSetV1, &changed).unwrap());
    }
}
