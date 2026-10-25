// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Exported serde owners shared by schema publication and native document projection.

#[path = "boundary_document_wire.rs"]
mod wire;
pub use wire::{Versioned, VersionedValue, Wire, check_header, check_required_header};

/// Visit exported names, public Rust owners, schema slugs and admitted directions.
/// A round-trip document has independent input and output generator contexts.
#[macro_export]
macro_rules! for_each_boundary_document {
    ($callback:ident) => { $callback! {
            (DecodeLocation, pse_model::document::DecodeLocation, "decode-location", Both, FlatV1),
            (ResultInspectionRequest, pse_runtime::workflow::ResultInspectionRequest, "result-inspection-request", Both, FlatV1),
            (ResultInspectionDocument, pse_runtime::workflow::ResultInspectionDocument, "result-inspection-document", Output, FlatV1),
            (ProjectExportRequest, pse_runtime::workflow::ProjectExportRequest, "project-export-request", Both, Native),
            (ProjectManifest, pse_runtime::workflow::ProjectManifest, "project-manifest", Both, Native),
            (ProjectClosureExplanation, pse_runtime::workflow::ProjectClosureExplanation, "project-closure-explanation", Both, FlatV1),
            (ProjectImportIntent, pse_runtime::workflow::ProjectImportIntent, "project-import-intent", Both, FlatV1),
            (ProjectImportRecoveryReport, pse_runtime::workflow::ProjectImportRecoveryReport, "project-import-recovery-report", Both, FlatV1),
            (ProjectImportReport, pse_runtime::workflow::ProjectImportReport, "project-import-report", Both, FlatV1),
            (LifecycleTarget, pse_operations::canonical_lifecycle::LifecycleTarget, "lifecycle-target", Both, FlatV1),
            (LifecycleSelection, pse_operations::canonical_lifecycle::LifecycleSelection, "lifecycle-selection", Both, FlatV1),
            (LifecycleExplanation, pse_operations::canonical_lifecycle::LifecycleExplanation, "lifecycle-explanation", Both, FlatV1),
            (LifecycleRelease, pse_operations::canonical_lifecycle::LifecycleRelease, "lifecycle-release", Both, FlatV1),
            (ResearchPublicationRequest, pse_runtime::workflow::ResearchPublicationRequest, "research-publication-request", Both, FlatV1),
            (ResearchPublicationIntent, pse_runtime::workflow::ResearchPublicationIntent, "research-publication-intent", Both, FlatV1),
            (ResearchPublicationPreview, pse_runtime::workflow::ResearchPublicationPreview, "research-publication-preview", Both, FlatV1),
            (ResearchPublication, pse_operations::canonical_research::ResearchPublication, "research-publication", Both, FlatV1),
            (ResearchDatasetRequest, pse_runtime::workflow::ResearchDatasetRequest, "research-dataset-request", Both, FlatV1),
            (ResearchSelectionRequest, pse_runtime::workflow::ResearchSelectionRequest, "research-selection-request", Both, FlatV1),
            (ResearchReadRequest, pse_runtime::workflow::ResearchReadRequest, "research-read-request", Both, FlatV1),
            (ResearchReadResult, pse_runtime::workflow::ResearchReadResult, "research-read-result", Both, FlatV1),
            (ResearchDataset, pse_operations::canonical_research::ResearchDataset, "research-dataset", Both, FlatV1),
            (ResearchSelection, pse_operations::canonical_research::ResearchSelection, "research-selection", Both, FlatV1),
            (InitializationOverrides, pse_runtime::workflow::InitializationOverrides, "initialization-overrides", Both, FlatV1),
            (ModelingNonlinearPolicy, pse_runtime::workflow::ModelingNonlinearPolicy, "nonlinear-explanation-policy", Both, FlatV1),
            (ModelingDiagnosticPolicy, pse_runtime::workflow::ModelingDiagnosticPolicy, "modeling-diagnostic-policy", Both, FlatV1),
            (WarmStartSnapshot, pse_backend_native::solve::WarmStartSnapshot, "warm-start-snapshot", Both, FlatV1),
            (SubmittedStartRecord, pse_backend_native::solve::SubmittedStartRecord, "submitted-start-record", Output, FlatV1),
            (DiagnosticCauseDocument, pse_runtime::workflow::DiagnosticCauseDocument, "diagnostic-cause", Both, FlatV1),
            (DiagnosticSpanDocument, pse_runtime::workflow::DiagnosticSpanDocument, "diagnostic-span", Both, FlatV1),
            (DiagnosticNoteDocument, pse_runtime::workflow::DiagnosticNoteDocument, "diagnostic-note", Both, FlatV1),
            (DiagnosticAnnotationDocument, pse_runtime::workflow::DiagnosticAnnotationDocument, "diagnostic-annotation", Both, FlatV1),
            (DiagnosticContextDocument, pse_runtime::workflow::DiagnosticContextDocument, "diagnostic-context", Both, FlatV1),
            (CacheReport, pse_engine::cache_service::CacheReport, "cache-report", Both, FlatV1),
            (ResourceReport, pse_runtime::ResourceReport, "resource-report", Both, FlatV1),
            (OperationId, pse_diagnostics::observation::OperationId, "diagnostic-operation-id", Both, FlatV1),
            (OperationSnapshot, pse_diagnostics::observation::OperationSnapshot, "operation-snapshot", Both, FlatV1),
            (OperationPage, pse_diagnostics::observation::OperationPage, "operation-page", Both, FlatV1),
            (EvidenceSnapshot, pse_operations::evidence::EvidenceSnapshot, "evidence-snapshot", Both, Native),
            (EvidenceComparisonRequest, pse_operations::evidence_query::EvidenceComparisonRequest, "evidence-comparison-request", Both, Native),
            (EvidenceComparison, pse_operations::evidence_query::EvidenceComparison, "evidence-comparison", Both, Native),
            (CheckpointIdentity, pse_runtime::observation::CheckpointIdentity, "checkpoint-identity", Both, FlatV1),
            (ReceiverCheckpoint, pse_runtime::observation::ReceiverCheckpoint, "receiver-checkpoint", Both, FlatV1),
            (ProcessCheckpoint, pse_runtime::observation::ProcessCheckpoint, "process-checkpoint", Both, Native),
            (SourceRecordingLaunchDocument, pse_runtime::tooling::review_launch::SourceRecordingLaunchDocument, "source-recording-launch-document", Both, Native),
            (SourceRecordingLaunchRequest, pse_runtime::tooling::review_launch::SourceRecordingLaunchRequest, "source-recording-launch-request", Both, Native),
            (PackExportRequest, pse_runtime::tooling::review_pack::PackExportRequest, "pack-export-request", Both, Native),
            (ReviewExportDeployment, pse_runtime::workflow::ReviewExportDeployment, "review-export-deployment", Both, Native),
            (ReductionAnchor, pse_runtime::workflow::ReductionAnchor, "reduction-anchor", Both, Native),
            (ReductionAnchorRole, pse_runtime::workflow::ReductionAnchorRole, "reduction-anchor-role", Both, Native),
            (ReductionOperator, pse_runtime::workflow::ReductionOperator, "reduction-operator", Both, Native),
            (ReductionRequest, pse_runtime::workflow::ReductionRequest, "reduction-request", Both, Native),
            (ReductionCommandRequest, pse_runtime::workflow::ReductionCommandRequest, "reduction-command-request", Both, Native),
            (ReductionDisposition, pse_runtime::workflow::ReductionDisposition, "reduction-disposition", Both, Native),
            (ReductionEdit, pse_runtime::workflow::ReductionEdit, "reduction-edit", Both, Native),
            (ReductionAttempt, pse_runtime::workflow::ReductionAttempt, "reduction-attempt", Both, Native),
            (ReductionCorrespondence, pse_runtime::workflow::ReductionCorrespondence, "reduction-correspondence", Both, Native),
            (ReductionDocument, pse_runtime::workflow::ReductionDocument, "reduction-document", Both, Native),
            (DiagnosticReproducer, pse_runtime::workflow::DiagnosticReproducer, "diagnostic-reproducer", Both, Native),
            (ReproducerCommandRequest, pse_runtime::workflow::ReproducerCommandRequest, "reproducer-command-request", Both, Native),
            (ReproducerExtractionRequest, pse_runtime::workflow::ReproducerExtractionRequest, "reproducer-extraction-request", Both, Native),
            (ReproducerExtractionInput, pse_runtime::workflow::ReproducerExtractionInput, "reproducer-extraction-input", Both, FlatV1),
            (ReproducerInput, pse_runtime::workflow::ReproducerInput, "reproducer-input", Both, FlatV1),
            (ReproductionDocument, pse_runtime::workflow::ReproductionDocument, "reproduction-document", Both, Native),
            (ReproductionPredicate, pse_runtime::workflow::ReproductionPredicate, "reproduction-predicate", Both, FlatV1),
            (ReproductionStatus, pse_runtime::workflow::ReproductionStatus, "reproduction-status", Both, FlatV1),
            (DiagnosticCommandRequest, pse_runtime::workflow::DiagnosticCommandRequest, "diagnostic-command-request", Both, Native),
            (DiagnosticExecutionRequest, pse_runtime::workflow::DiagnosticExecutionRequest, "diagnostic-execution-request", Both, Native),
            (DiagnosticExecutionDocument, pse_runtime::workflow::DiagnosticExecutionDocument, "diagnostic-execution-document", Both, Native),
            (CanonicalGraphRequest, pse_operations::canonical_explanations::CanonicalGraphRequest, "canonical-graph-request", Both, Native),
            (CanonicalGraphDocument, pse_operations::canonical_explanations::CanonicalGraphDocument, "canonical-graph-document", Both, Native),
            (TaskDumpRequest, pse_runtime::tooling::taskdump::TaskDumpRequest, "task-dump-request", Both, Native),
            (TaskDumpDocument, pse_runtime::tooling::taskdump::TaskDumpDocument, "task-dump-document", Both, Native),
            (AnalysisRequest, pse_runtime::tooling::review_analysis::AnalysisRequest, "analysis-request", Both, Native),
            (AnalysisDocument, pse_runtime::tooling::review_analysis::AnalysisDocument, "analysis-document", Both, Native),
            (PackSelection, pse_runtime::tooling::review_pack::PackSelection, "pack-selection", Both, Native),
            (PackIndex, pse_runtime::tooling::review_pack::PackIndex, "pack-index", Both, Native),
            (ReviewTableDocument, pse_runtime::tooling::review_query::ReviewTableDocument, "review-table-document", Both, Native),
            (ReviewTableRequest, pse_runtime::tooling::review_query::ReviewTableRequest, "review-table-request", Both, Native),
            (ReviewQueryRequest, pse_runtime::tooling::review_query::ReviewQueryRequest, "review-query-request", Both, Native),
            (ReviewQueryDocument, pse_runtime::tooling::review_query::ReviewQueryDocument, "review-query-document", Both, Native),
            (ReviewQuerySpecification, pse_runtime::tooling::review_query::ReviewQuerySpecification, "review-query-specification", Both, FlatV1),
            (SourceRecordingRequest, pse_runtime::tooling::capture::SourceRecordingRequest, "source-recording-request", Both, Native),
            (ReviewHandoffReceipt, pse_runtime::tooling::capture::ReviewHandoffReceipt, "review-handoff-receipt", Both, Native),
            (SourceRecordingAcknowledgement, pse_runtime::tooling::capture::SourceRecordingAcknowledgement, "source-recording-acknowledgement", Both, Native),
            (FrozenEvidenceSet, pse_runtime::tooling::frozen_inspection::FrozenEvidenceSet, "frozen-evidence-set", Both, Native),
            (FrozenInspectionRequest, pse_runtime::tooling::frozen_inspection::FrozenInspectionRequest, "frozen-inspection-request", Both, Native),
            (FrozenInspectionDocument, pse_runtime::tooling::frozen_inspection::FrozenInspectionDocument, "frozen-inspection-document", Both, Native),
            (ReviewContextRequest, pse_runtime::workflow::ReviewContextRequest, "review-context-request", Both, FlatV1),
            (ReviewContextDocument, pse_runtime::workflow::ReviewContextDocument, "review-context-document", Both, Native),
            (ReviewScientificCoordinates, pse_runtime::workflow::ReviewScientificCoordinates, "review-scientific-coordinates", Both, FlatV1),
            (ReviewInventoryRequest, pse_runtime::workflow::ReviewInventoryRequest, "review-inventory-request", Both, FlatV1),
            (ReviewInventorySelection, pse_runtime::workflow::ReviewInventorySelection, "review-inventory-selection", Both, Native),
            (ReviewInventoryPage, pse_runtime::workflow::ReviewInventoryPage, "review-inventory-page", Both, Native),
            (InspectionRequest, pse_runtime::tooling::inspection::InspectionRequest, "inspection-request", Both, Native),
            (InspectionDocument, pse_runtime::tooling::inspection::InspectionDocument, "inspection-document", Both, Native),
            (DiagnosticListRequest, pse_runtime::tooling::inspection::DiagnosticListRequest, "diagnostic-list-request", Both, FlatV1),
            (DiagnosticProcessPage, pse_runtime::tooling::inspection::DiagnosticProcessPage, "diagnostic-process-page", Both, Native),
            (CaptureRequest, pse_runtime::tooling::capture::CaptureRequest, "capture-request", Both, Native),
            (DiagnosticOrigin, pse_runtime::workflow::DiagnosticOrigin, "diagnostic-origin", Both, FlatV1),
            (LaunchCaptureRequest, pse_runtime::tooling::capture::LaunchCaptureRequest, "launch-capture-request", Both, Native),
            (CaptureDocument, pse_runtime::tooling::capture::CaptureDocument, "capture-document", Both, Native),
            (DetailRequest, pse_runtime::observation_detail::DetailRequest, "detail-request", Both, Native),
            (DetailDocument, pse_runtime::observation_detail::DetailDocument, "detail-document", Both, Native),
            (DetailRecordPage, pse_runtime::observation_detail::DetailRecordPage, "detail-record-page", Both, Native),
            (ModelingSourceMapDocument, pse_runtime::workflow::ModelingSourceMapDocument, "modeling-source-map", Both, Native),
            (CoherentRecord, pse_diagnostics::observation::CoherentRecord, "coherent-record", Both, FlatV1),
            (InvocationEvidenceDocument, pse_runtime::invocation::InvocationEvidenceDocument, "invocation-evidence", Both, Native),
            (InvocationComparisonDocument, pse_runtime::tooling::inspection_comparison::InvocationComparisonDocument, "invocation-comparison", Both, Native),
            (ProgressCheckpointDocument, pse_runtime::workflow::ProgressCheckpointDocument, "progress-checkpoint", Both, Native),
            (ManagedStudyDiagnosticsDocument, pse_runtime::workflow::ManagedStudyDiagnosticsDocument, "managed-study-diagnostics", Both, Native),
            (TableName, pse_runtime::TableName, "table-name", Both, FlatV1),
            (BuildInfo, pse_buildinfo::BuildInfo, "build-info", Both, FlatV1),
            (ProgressEventDocument, pse_runtime::workflow::ProgressEventDocument, "progress-event", Both, Native),
            (ProgressCoverage, pse_runtime::workflow::ProgressCoverage, "progress-coverage", Both, FlatV1),
            (RouteDocument, pse_runtime::workflow::RouteDocument, "route", Both, FlatV1),
            (EligibilityDocument, pse_runtime::workflow::EligibilityDocument, "eligibility", Both, FlatV1),
            (Completion, pse_runtime::workflow::Completion, "completion", Both, Native),
            (AttemptSummary, pse_runtime::workflow::AttemptSummaryDocument, "attempt-summary", Both, FlatV1),
            (ReplayDocument, pse_runtime::workflow::ReplayDocument, "scientific-replay", Output, Native),
            (SolverAdviceInputs, pse_runtime::math::solves::SolverAdviceInputs, "solver-advice-inputs", Both, FlatV1),
            (FrozenAdvice, pse_model::strategy::advice::FrozenAdvice, "frozen-advice", Both, FlatV1),
            (OperationSource, pse_runtime::workflow::OperationSource, "operation-source", Both, FlatV1),
            (PreparationSettings, pse_runtime::workflow::PreparationSettings, "preparation-settings", Both, FlatV1),
            (PhysicalSimulationRequest, pse_runtime::workflow::PhysicalSimulationRequest, "physical-simulation-request", Both, Native),
            (PhysicalFitRequest, pse_runtime::workflow::PhysicalFitRequest, "physical-fit-request", Both, Native),
            (ParameterCovariance, pse_runtime::math::settings::ParameterCovariance, "parameter-covariance", Both, Native),
            (FitExecutionDocument, pse_runtime::workflow::FitExecutionDocument, "fit-execution", Output, FlatV1),
            (FitProfileDocument, pse_runtime::workflow::FitProfileDocument, "fit-profiles", Output, FlatV1),
            (FitDeclarations, pse_runtime::workflow::FitDeclarations, "fit-declarations", Both, FlatV1),
            (RunControls, pse_runtime::workflow::RunControls, "run-controls", Both, FlatV1),
            (StudyRunControls, pse_runtime::workflow::StudyRunControls, "study-run-controls", Both, FlatV1),
            (StudyWaitControls, pse_runtime::workflow::StudyWaitControls, "study-wait-controls", Both, FlatV1),
            (KnowledgeControls, pse_runtime::workflow::KnowledgeControls, "knowledge-controls", Both, FlatV1),
            (DiagnosticSamplesControls, pse_runtime::workflow::DiagnosticSamplesControls, "diagnostic-samples-controls", Both, FlatV1),
            (JacobianDiagnosticControls, pse_runtime::workflow::JacobianDiagnosticControls, "jacobian-diagnostic-controls", Both, FlatV1),
            (LinearDiagnosticControls, pse_runtime::workflow::LinearDiagnosticControls, "linear-diagnostic-controls", Both, FlatV1),
            (PureConformanceControls, pse_runtime::workflow::PureConformanceControls, "pure-conformance-controls", Both, FlatV1),
            (ConformanceControls, pse_runtime::workflow::ConformanceControls, "conformance-controls", Both, FlatV1),
            (DeclarationEdit, pse_runtime::workflow::DeclarationEdit, "declaration-edit", Both, FlatV1),
            (SourceDelta, pse_runtime::workflow::SourceDelta, "source-delta", Both, FlatV1),
            (ProjectNavigationRequest, pse_operations::canonical_projects::ProjectNavigationRequest, "project-navigation-request", Both, Native),
            (ProjectMetadataRequest, pse_operations::canonical_projects::ProjectMetadataRequest, "project-metadata-request", Both, FlatV1),
            (ProjectMetadataCursor, pse_operations::canonical_projects::ProjectMetadataCursor, "project-metadata-cursor", Both, FlatV1),
            (ProjectMetadataPage, pse_operations::canonical_projects::ProjectMetadataPage, "project-metadata-page", Both, FlatV1),
            (ProjectDraftVersion, pse_operations::canonical_projects::DraftVersion, "project-draft-version", Both, FlatV1),
            (ProjectContext, pse_operations::canonical_contexts::ProjectContext, "project-context", Both, FlatV1),
            (ContextCut, pse_operations::canonical_contexts::ContextCut, "context-cut", Both, FlatV1),
            (ContextSelection, pse_operations::canonical_contexts::ContextSelection, "context-selection", Both, FlatV1),
            (ProjectSourceDetail, pse_runtime::workflow::ProjectSourceDetail, "project-source-detail", Both, FlatV1),
            (ProjectComparison, pse_runtime::workflow::ProjectComparison, "project-comparison", Both, FlatV1),
            (ProjectMergeResolution, pse_runtime::workflow::ProjectMergeResolution, "project-merge-resolution", Both, FlatV1),
            (ProjectMergePreview, pse_runtime::workflow::ProjectMergePreview, "project-merge-preview", Both, FlatV1),
            (ProjectMergeIntent, pse_runtime::workflow::ProjectMergeIntent, "project-merge-intent", Both, Native),
            (ProjectDraftDelta, pse_runtime::workflow::ProjectDraftDelta, "project-draft-delta", Both, Native),
            (PreparationDiscoveryRequest, pse_runtime::workflow::PreparationDiscoveryRequest, "preparation-discovery-request", Both, FlatV1),
            (PreparationDiscoveryReport, pse_runtime::workflow::PreparationDiscoveryReport, "preparation-discovery-report", Both, FlatV1),
            (ProjectSelectRequest, pse_runtime::workflow::ProjectSelectRequest, "project-select-request", Both, Native),
            (ProjectSelection, pse_runtime::workflow::ProjectSelection, "project-selection", Both, FlatV1),
            (ProjectPublication, pse_operations::canonical_projects::ProjectPublication, "project-publication", Both, FlatV1),
            (ProjectCursor, pse_operations::canonical_projects::ProjectCursor, "project-cursor", Both, FlatV1),
            (ProjectRecord, pse_operations::canonical_projects::Project, "project-record", Both, FlatV1),
            (ProjectBranchRecord, pse_operations::canonical_projects::Branch, "project-branch-record", Both, FlatV1),
            (ProjectDraftRecord, pse_operations::canonical_projects::Draft, "project-draft-record", Both, FlatV1),
            (ProjectAcknowledgement, pse_operations::canonical_projects::ProjectAcknowledgement, "project-acknowledgement", Both, FlatV1),
            (ResearchReference, pse_operations::canonical_projects::ResearchReference, "research-reference", Both, FlatV1),
            (ProjectBranchPage, pse_operations::canonical_projects::BranchPage, "project-branch-page", Both, FlatV1),
            (ProjectDraftDocuments, pse_runtime::workflow::ProjectDraftDocuments, "project-draft-documents", Both, Native),
            (ProjectDraftSaveIntent, pse_runtime::workflow::ProjectDraftSaveIntent, "project-draft-save-intent", Both, FlatV1),
            (SourceSettlement, pse_runtime::workflow::SourceSettlement, "source-settlement", Both, FlatV1),
            (DeclarationInventory, pse_runtime::workflow::DeclarationInventory, "declaration-inventory", Both, FlatV1),
            (ModelingInspection, pse_runtime::workflow::ModelingInspection, "modeling-inspection", Both, FlatV1),
            (ConicRequest, pse_runtime::workflow::ConicRequest, "conic-request", Both, FlatV1),
            (RecycleRequest, pse_runtime::workflow::RecycleRequest, "recycle-request", Both, FlatV1),
            (RecycleRestartRequest, pse_runtime::workflow::RecycleRestartRequest, "recycle-restart-request", Both, FlatV1),
            (FlowSelectionDocument, pse_runtime::math::flows::FlowSelectionDocument, "flow-selection", Both, FlatV1),
            (FlowGraphDocument, pse_runtime::math::flows::FlowGraphDocument, "flow-graph", Both, FlatV1),
            (TearSelectionDocument, pse_runtime::math::flows::TearSelectionDocument, "tear-selection", Both, FlatV1),
            (InitializationDocument, pse_runtime::workflow::InitializationDocument, "initialization", Both, FlatV1),
            (SolveSettings, pse_runtime::math::settings::SolveSettings, "solve-settings", Both, Native),
            (BlockSolveOverride, pse_runtime::math::settings::BlockSolveOverride, "block-solve-override", Both, FlatV1),
            (EffortPolicy, pse_model::strategy::EffortPolicy, "effort-policy", Both, FlatV1),
            (NumericalStrategyDocument, pse_model::strategy::NumericalStrategyDocument, "numerical-strategy", Both, Native),
            (CompositionRequest, pse_model::strategy::CompositionRequest, "composition-request", Both, FlatV1),
            (BackendSettings, pse_backend_native::execution::BackendSettings, "backend-settings", Both, FlatV1),
            (DiffsolSettings, pse_backend_native::dynamics::DiffsolSettings, "diffsol-settings", Both, Native),
            (IdasSettings, pse_backend_native::dynamics::IdasSettings, "idas-settings", Both, Native),
            (AdjointSettings, pse_backend_native::dynamics::AdjointSettings, "adjoint-settings", Both, Native),
            (StudyDefinition, pse_runtime::workflow::StudyDefinition, "study-definition", Both, Native),
            (StudyRequest, pse_runtime::workflow::StudyRequest, "study-request", Both, Native),
            (StudyStatus, pse_runtime::workflow::StudyStatus, "study-status", Both, FlatV1),
            (StudyWorkbenchCursor, pse_runtime::workflow::StudyWorkbenchCursor, "study-workbench-cursor", Both, FlatV1),
            (StudyWorkbenchRequest, pse_runtime::workflow::StudyWorkbenchRequest, "study-workbench-request", Both, FlatV1),
            (StudyWorkbenchPage, pse_runtime::workflow::StudyWorkbenchPage, "study-workbench-page", Both, FlatV1),
            (StudyCohortRequest, pse_runtime::workflow::StudyCohortRequest, "study-cohort-request", Both, FlatV1),
            (StudyCohortDocument, pse_runtime::workflow::StudyCohortDocument, "study-cohort-document", Both, FlatV1),
            (StudyKpiRequest, pse_runtime::workflow::StudyKpiRequest, "study-kpi-request", Both, FlatV1),
            (StudyObservationRequest, pse_runtime::workflow::StudyObservationRequest, "study-observation-request", Both, FlatV1),
            (StudyObservationCursor, pse_runtime::workflow::StudyObservationCursor, "study-observation-cursor", Both, FlatV1),
            (StudyObservationPage, pse_runtime::workflow::StudyObservationPage, "study-observation-page", Both, FlatV1),
            (StudyResults, pse_runtime::workflow::StudyResults, "study-results", Both, FlatV1),
            (AnalysisControls, pse_runtime::workflow::AnalysisControls, "analysis-controls", Both, FlatV1),
            (ResultReclamationPage, pse_operations::canonical_result_retention::ResultReclamationPage, "result-reclamation-page", Both, FlatV1),
            (SourceReclamationPage, pse_operations::canonical_retention::ReclamationPage, "source-reclamation-page", Output, FlatV1),
            (HistoryOwner, pse_operations::canonical_history::HistoryOwner, "history-owner", Both, FlatV1),
            (HistoryPurpose, pse_operations::canonical_history::HistoryPurpose, "history-purpose", Both, ValueV1),
            (HistoryObligation, pse_operations::canonical_history::HistoryObligation, "history-obligation", Both, FlatV1),
            (HistoryReclamationPage, pse_operations::canonical_history::HistoryReclamationPage, "history-reclamation-page", Both, FlatV1),
            (StudyCancel, pse_runtime::workflow::StudyCancel, "study-cancel", Both, FlatV1),
            (PointOutcome, pse_runtime::workflow::PointOutcome, "study-point-outcome", Both, FlatV1),
            (Conclusion, pse_runtime::workflow::StudyConclusion, "study-conclusion", Both, FlatV1),
            (BoundaryDiagnostic, pse_runtime::workflow::BoundaryDiagnostic, "boundary-diagnostic", Both, FlatV1),
            (TerminationDetail, pse_runtime::workflow::TerminationDetail, "termination-detail", Both, Native),
            (FitUncertainty, pse_runtime::workflow::FitUncertainty, "fit-uncertainty", Both, FlatV1),
            (PreparationCounts, pse_runtime::math::PreparationCounts, "preparation-counts", Both, FlatV1),
            (ProcessLocator, pse_runtime::observation::ProcessLocator, "process-locator", Both, FlatV1),
            (ComparisonPolicy, pse_runtime::tooling::inspection_comparison::ComparisonPolicy, "comparison-policy", Both, FlatV1),
            (SourceLocation, pse_model::diagnostic::SourceLocation, "source-location", Both, FlatV1),
            (IneligibleDocument, pse_runtime::workflow::IneligibleDocument, "ineligible-document", Both, FlatV1),
            (NumericalPolicy, pse_model::numerics::NumericalPolicy, "numerical-policy", Both, Native),
            (RunSettings, pse_runtime::workflow::RunSettings, "conformance-run-settings", Both, Native),
            (ConformanceRun, pse_runtime::workflow::ConformanceRun, "conformance-run", Both, Native),
            (ConformanceManifest, pse_runtime::workflow::ConformanceManifest, "conformance-manifest", Both, Native),
            (LeasePolicy, pse_runtime::workflow::LeasePolicy, "lease-policy", Both, Native),
            (ArtifactIntent, pse_operations::canonical_artifacts::Artifact, "artifact-intent", Both, FlatV1),
            (SolverCapability, pse_model::generated::runtime::solver_capabilities::Row, "solver-capability", Output, FlatV1),
            (StudyPointDefinition, pse_runtime::workflow::StudyPointDefinition, "study-point-definition", Output, FlatV1),
            (LazyStudyDefinition, pse_runtime::workflow::LazyStudyDefinition, "lazy-study-definition", Output, FlatV1),
            (AdmittedBinding, pse_runtime::workflow::AdmittedBinding, "admitted-binding", Output, FlatV1),
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
struct ExportNames<'a>(&'a std::collections::BTreeMap<String, String>);
impl schemars::transform::Transform for ExportNames<'_> {
    fn transform(&mut self, schema: &mut schemars::Schema) {
        if let Some(title) = schema
            .get("title")
            .and_then(serde_json::Value::as_str)
            .and_then(|title| self.0.get(title))
        {
            schema.insert("title".into(), title.clone().into());
        }
        if let Some(reference) = schema
            .get("$ref")
            .and_then(serde_json::Value::as_str)
            .and_then(|reference| reference.strip_prefix("#/$defs/"))
        {
            let name = reference.replace("~1", "/").replace("~0", "~");
            if let Some(export) = self.0.get(&name) {
                schema.insert(
                    "$ref".into(),
                    format!("#/$defs/{}", export.replace('~', "~0").replace('/', "~1")).into(),
                );
            }
        }
        if let Some(definitions) = schema
            .get_mut("$defs")
            .and_then(serde_json::Value::as_object_mut)
        {
            for (original, export) in self.0 {
                if original != export
                    && !definitions.contains_key(export)
                    && let Some(definition) = definitions.remove(original)
                {
                    definitions.insert(export.clone(), definition);
                }
            }
        }
        schemars::transform::transform_subschemas(self, schema);
    }
}
/// The standalone wire contract is selected by the same declaration as its schema.
pub trait BoundaryDocument {
    /// Actual owning wire representation; nested domain serialization stays unchanged.
    const WIRE: Wire;
}
macro_rules! implement_wire {
    ($(($name:ident, $owner:ty, $slug:literal, $direction:ident, $wire:ident)),* $(,)?) => {
        $(impl BoundaryDocument for $owner { const WIRE: Wire = Wire::$wire; })*
    };
}
crate::for_each_boundary_document!(implement_wire);

fn schema<T: schemars::JsonSchema + BoundaryDocument>(
    name: &'static str,
    slug: &'static str,
    rust_type: &'static str,
    direction: Direction,
) -> DocumentSchema {
    let settings = schemars::generate::SchemaSettings::draft2020_12();
    let settings = match direction {
        Direction::Input => settings.for_deserialize(),
        Direction::Output => settings.for_serialize(),
    };
    let generator = settings.into_generator();
    let mut schema = match T::WIRE {
        Wire::Native => generator.into_root_schema_for::<T>(),
        Wire::FlatV1 => generator.into_root_schema_for::<Versioned<T, 1>>(),
        Wire::ValueV1 => generator.into_root_schema_for::<VersionedValue<T, 1>>(),
    };
    schemars::transform::Transform::transform(&mut ExportNames(&export_names()), &mut schema);
    let mut schema = schema.to_value();
    schema.sort_all_objects();
    let field = match T::WIRE {
        Wire::Native => "version",
        Wire::FlatV1 | Wire::ValueV1 => "document_version",
    };
    let version = schema["properties"][field]["const"]
        .as_u64()
        .and_then(|version| u32::try_from(version).ok());
    DocumentSchema {
        name,
        slug,
        rust_type,
        direction,
        version,
        wire: T::WIRE,
        schema,
    }
}
fn export_names() -> std::collections::BTreeMap<String, String> {
    macro_rules! aliases {
        ($(($name:ident, $owner:ty, $slug:literal, $direction:ident, $wire:ident)),* $(,)?) => {{
            let mut names = std::collections::BTreeMap::new();
            $(
                let body = <$owner as schemars::JsonSchema>::schema_name().into_owned();
                let export = stringify!($name);
                match Wire::$wire {
                    Wire::Native => { names.insert(body, export.into()); }
                    Wire::FlatV1 => {
                        names.insert(body, format!("{export}Body"));
                        names.insert(<Versioned<$owner, 1> as schemars::JsonSchema>::schema_name().into_owned(), export.into());
                    }
                    Wire::ValueV1 => {
                        names.insert(body, format!("{export}Body"));
                        names.insert(<VersionedValue<$owner, 1> as schemars::JsonSchema>::schema_name().into_owned(), export.into());
                    }
                }
            )*
            names
        }};
    }
    crate::for_each_boundary_document!(aliases)
}
/// Derive each root with a fresh direction-specific schemars generator.
pub fn schemas() -> Vec<DocumentSchema> {
    fn input<T: schemars::JsonSchema + serde::de::DeserializeOwned + BoundaryDocument>(
        name: &'static str,
        slug: &'static str,
        owner: &'static str,
    ) -> DocumentSchema {
        schema::<T>(name, slug, owner, Direction::Input)
    }
    fn output<T: schemars::JsonSchema + serde::Serialize + BoundaryDocument>(
        name: &'static str,
        slug: &'static str,
        owner: &'static str,
    ) -> DocumentSchema {
        schema::<T>(name, slug, owner, Direction::Output)
    }
    macro_rules! collect {
        ($(($name:ident, $owner:ty, $slug:literal, $direction:ident, $wire:ident)),* $(,)?) => {{
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
    static FINGERPRINT: std::sync::OnceLock<
        Result<pse_ids::ContentHash, pse_ids::document::FrameError>,
    > = std::sync::OnceLock::new();
    FINGERPRINT
        .get_or_init(|| pse_ids::document::of(pse_ids::Frame::BoundarySchemaSetV1, &schemas()))
        .clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exported_names_reach_nested_generics_without_rewriting_document_values() {
        use schemars::transform::Transform;
        let names =
            std::collections::BTreeMap::from([("Internal/Generic~Name".into(), "Public".into())]);
        let mut schema = schemars::json_schema!({
            "title": "Internal/Generic~Name",
            "$defs": {"Internal/Generic~Name": {"type": "object"}},
            "properties": {"nested": {"$ref": "#/$defs/Internal~1Generic~0Name"}},
            "const": {"title": "Internal/Generic~Name", "$ref": "#/$defs/Internal~1Generic~0Name"}
        });
        ExportNames(&names).transform(&mut schema);
        assert_eq!(schema.as_value()["title"], "Public");
        assert!(schema.as_value()["$defs"].get("Public").is_some());
        assert_eq!(
            schema.as_value()["properties"]["nested"]["$ref"],
            "#/$defs/Public"
        );
        assert_eq!(schema.as_value()["const"]["title"], "Internal/Generic~Name");
        assert_eq!(
            schema.as_value()["const"]["$ref"],
            "#/$defs/Internal~1Generic~0Name"
        );
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
            assert!(
                root.version.is_some(),
                "unversioned standalone root {}",
                root.slug
            );
            let field = if root.wire == Wire::Native {
                "version"
            } else {
                "document_version"
            };
            assert!(
                root.schema["required"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|value| value == field),
                "missing required header {}",
                root.slug
            );
        }
        assert_eq!(fingerprint().unwrap(), fingerprint().unwrap());
        let expected = pse_ids::document::of(pse_ids::Frame::BoundarySchemaSetV1, &roots).unwrap();
        let mut changed = roots;
        changed[0].direction = Direction::Output;
        assert_ne!(
            expected,
            pse_ids::document::of(pse_ids::Frame::BoundarySchemaSetV1, &changed).unwrap()
        );
    }
}
