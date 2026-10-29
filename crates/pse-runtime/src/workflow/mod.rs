// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! One public native workflow. Model declarations are generated; mathematics stays in Rust libraries.
//!
//! # Database-free local operations
//!
//! Build one [`crate::SharedRuntime`] from an explicit resource budget, then bind its
//! `session_factory` to the shared schema registry and use [`crate::workflow::Runtime::from_shared_local`].
//! Admit physical documents with [`crate::workflow::Runtime::physical_from_documents`] and model documents
//! with [`crate::workflow::Runtime::modeling_from_documents`]; both use this caller's validation bindings.
//! [`crate::workflow::ModelingPackage::prepare_operation`] consumes the exact [`crate::workflow::OperationSource`] and
//! [`crate::workflow::PreparationSettings`], and [`crate::workflow::PreparedOperation::start`] returns a [`crate::workflow::RunHandle`].
//!
//! A pending `RunHandle::wait` future is only a waiter: dropping it or a passive
//! observation reader does not request stop while a handle remains. Explicit `cancel`
//! requests stop; dropping the last immediate handle also requests stop. The native
//! supervisor retains execution ownership through cleanup and required joins. A repeated
//! wait returns the same immutable result. Closing the logical caller seals new admission,
//! while admitted work retains continuation ownership. Results and exported tables retain
//! their own allocation owners independently of preparation/cache lifetime. Service-owned
//! neutral native lanes have a separate lifetime; their owner explicitly awaits
//! `shared.math().shutdown_native_lanes()` before retiring the shared runtime.
//! None of these local operations requires a canonical deployment, Python or a CLI.
mod conformance_documents;
pub use conformance_documents::{ConformanceManifest, ConformanceRun, RunSettings};
mod diagnostic_request;
mod reduction;
mod reproducer;
pub use diagnostic_request::{
    DiagnosticCommandRequest, DiagnosticExecutionDocument, DiagnosticExecutionRequest,
    DiagnosticRuntimeSettings,
};
pub use reduction::{
    ReductionAnchor, ReductionAnchorRole, ReductionAttempt, ReductionCommandRequest,
    ReductionCorrespondence, ReductionDisposition, ReductionDocument, ReductionEdit,
    ReductionOperator, ReductionRequest,
};
pub use reproducer::{
    DiagnosticReproducer, ReproducerCommandRequest, ReproducerExtractionInput,
    ReproducerExtractionRequest, ReproducerInput, ReproductionDocument, ReproductionPredicate,
    ReproductionStatus,
};
mod route_documents;
pub use route_documents::{EligibilityDocument, IneligibleDocument, RouteDocument};
mod controls;
pub use controls::{
    InventoryControls, ProgressControls, RunControls, StudyRunControls, StudyWaitControls,
};
mod attempt_documents;
mod completion;
mod replay;
mod review_context;
#[cfg(all(test, feature = "solver-kinsol"))]
mod review_live_qualification_tests;
#[cfg(test)]
mod review_qualification_tests;
pub use attempt_documents::{
    AttemptDecision, AttemptFailureSummary, AttemptFinding, AttemptIntervals,
    AttemptOriginalConclusion, AttemptStartSummary, AttemptStepSummary, AttemptSummary,
    AttemptSummaryDocument, AttemptSummaryLimits,
};
pub use replay::{
    ReplayBoundOverride, ReplayCaseConstructor, ReplayCaseInputs, ReplayConstructor,
    ReplayCoverage, ReplayDecisionDocument, ReplayDecisions, ReplayDocument, ReplayFitConstructor,
    ReplayHorizonConstructor, ReplayHorizonController, ReplayHorizonEstimator, ReplayHorizonSignal,
    ReplayNativeAttempt, ReplayNativeRecordCoverage, ReplayPhysicalSource, ReplayProvider,
    ReplayProviderPort, ReplayProviderShape, ReplayScreenedStart, ReplayShootingBound,
    ReplayShootingConstructor, ReplaySimulationConstructor, ReplaySolverSettings,
    ReplaySourceDocument, ReplaySourceObject, ReplayVariableState, ScientificReplay,
};
pub use review_context::{
    ReviewContext, ReviewContextDocument, ReviewContextFact, ReviewContextRequest,
    ReviewContextUnavailable, ReviewProviderDescriptor, ReviewScientificCoordinates,
    ReviewSourceAttribution, ReviewSourceDocument, ReviewSourceOccurrence, ReviewSourceTerm,
};
mod connected_results;
mod review_export;
pub use review_export::{ReviewExportDeployment, ReviewOutputSelection, ReviewScientificSelection};
mod physical_spool;
pub mod result_blocks;
pub mod result_files;
mod result_inspection;
mod result_physical;
pub use result_inspection::{
    ArrowFieldPath, ArrowFieldStep, ResultCellUnavailable, ResultFloatClass, ResultInspectionBasis,
    ResultInspectionCell, ResultInspectionCursor, ResultInspectionDocument, ResultInspectionField,
    ResultInspectionOriginal, ResultInspectionRefusal, ResultInspectionRequest,
    ResultInspectionRow, ResultInspectionStorage, ResultInspectionView, ResultMetadataOmission,
    ResultPhysicalContext,
};
mod result_snapshot;
pub use result_snapshot::{ResultSnapshot, SelectedResultSnapshot};
pub(crate) mod result_export;
mod result_projection;
pub use connected_results::{CanonicalResultReader, SelectedResultReader};
pub use result_export::{ResultCursor, ResultOrder, StrategyEventExport};
mod analyses;
pub mod artifacts;
pub use analyses::{AnalysisControls, AnalysisDirection, AnalysisHandle};
mod retention;
pub use retention::{
    HistoryObligation, HistoryOwner, HistoryPurpose, HistoryReclamationPage, ResultReclamationPage,
    SourceReclamationPage,
};
mod diagnostic_operation;
pub use diagnostic_operation::{DiagnosticOperation, DiagnosticOrigin};
mod diagnostic_documents;
pub use diagnostic_documents::{
    DiagnosticAnnotationDocument, DiagnosticCauseDocument, DiagnosticContextDocument,
    DiagnosticNoteDocument, DiagnosticSpanDocument,
};
pub(crate) mod diagnostic_rows;
mod diagnostics;
mod durable;
pub use completion::{Completion, ProductPermission, ScientificPermission};
pub use durable::{
    AdmittedSeed, Durability, DurableRecord, LeasePolicy, Operations, Recovery, RunDurability,
    SeedReadError, StoredCompletion, TerminationCause, TerminationDetail,
};
mod progress_documents;
pub use progress_documents::{
    IncumbentDocument, ProgressCheckpointDocument, ProgressCoverage, ProgressEventDocument,
    ProgressMetricDocument, ProgressWriteStage, ProgressWriterFailure, ProgressWriterOutcome,
};
mod progress;
pub use progress::ProgressStream;
mod bindings;
mod native_deployment;
mod native_worker;
mod physical_cache;
pub mod sampling;
mod worker;
pub use bindings::{
    AdmittedBinding, AdmittedBindingEntry, BindingAssignment, BindingQuantity, BindingTarget,
    PointOverlay,
};
pub use native_deployment::{
    CapabilityDefaults, DeploymentObservation, DeploymentReadiness, DeploymentRequest,
    DeploymentResponse, ManagedExecutionPolicy, ManagedPrimaryReceiver, NativeAllocation,
    NativeExecutionAllocation, NativeExecutionSelection, PreparedReceiverEnvironment,
    ProductionProfile, ProductionProfileName, ProductionProfiles, RegisteredNativeContext,
};
pub use native_worker::{
    NativeStudyWorker, NativeStudyWorkerInputs, NativeStudyWorkerObserver, NativeStudyWorkerSpec,
    NativeWorkerTerminal,
};
pub use worker::{PhysicalSource, Processed, WorkerSettings};
mod declared_requests;
mod operation_preparation;
pub use declared_requests::conform_pure_request;
pub use operation_preparation::{PreparedOperation, RuntimeOperationRequest};
mod study_demand;
pub use study_demand::{
    CompactStudyResult, StudyDetailDemand, StudyMemberDemand, StudyOutputDemand,
};
mod study_variants;
pub use study_variants::{
    LazyStudyDefinition, SelectedStudyVariant, SourceChoice, StudyInputs, StudyVariants,
};
mod study;
mod study_diagnostics;
pub use study_diagnostics::{
    ManagedDetailOwner, ManagedDetailReference, ManagedDetailReferenceState, ManagedDetailRefusal,
    ManagedDetailSelection, ManagedStudyDiagnosticsDocument, ReviewAttemptSelection,
    ReviewInventoryGap, ReviewInventoryLink, ReviewInventoryMember, ReviewInventoryPage,
    ReviewInventoryRequest, ReviewInventorySelection,
};
mod study_execution;
mod study_tables;
pub use pse_model::diagnostic::BoundaryDiagnostic;
pub use pse_model::study::Conclusion as StudyConclusion;
pub use study_execution::{StudyOccurrenceResult, StudyReport};
mod study_operations;
pub use pse_model::generated::{
    enums::{StudyPointState, StudyState},
    identities::StudyId,
};
pub use study::{
    AdmittedStudy, LocalStudy, MAXIMUM_STUDY_POINTS, PackageSources, PointAttemptOutcome,
    PointOutcome, PointStatus, StudyCancel, StudyDefinition, StudyHandle, StudyPlan, StudyPoint,
    StudyPointDefinition, StudyPointPolicy, StudyRequest, StudyResults, StudyStatus,
};
pub use study_operations::{
    AdmittedHorizonValues, ArrivalDocument, CaseOperation, ControllerOperation, EstimatorInput,
    EstimatorOperation, FitOperation, HorizonBinding, HorizonInputDocument, HorizonOperation,
    HorizonSignalDocument, OperationRequest, OperationSource, PreparationSettings,
    PreparedStudyOperation, SimulationOperation, StudyOperation, StudyResultRole,
};
pub(crate) mod numerics;
mod objectives;
pub use objectives::{ModelingLevelsReport, ModelingObjectiveLevel};
mod staged;
mod strategies;
mod time;
pub use strategies::{
    AnalysisPort, CausalUnitRealization, CausalUnitRequest, ConicRequest, InitializationDocument,
    InitializationStageDocument, PreparedConic, RecycleRequest, RecycleRestartRequest,
};
#[cfg(feature = "solver-kinsol")]
pub use strategies::{PreparedInitializationStrategy, PreparedRecycle};
#[cfg_attr(
    not(feature = "solver-diffsol"),
    allow(
        dead_code,
        reason = "Dynamic source and preparation contracts remain available without the optional execution adapter"
    )
)]
mod dynamics;
pub(crate) use dynamics::SnapshotAllocator;
mod fitting;
mod horizon;
mod integrated;
pub use horizon::{
    AdvancedStep, Arrival, Horizon, HorizonController, HorizonDecision, HorizonEstimator,
    HorizonInput, HorizonOutcome, HorizonReport, HorizonRequest, HorizonSignal, HorizonStep,
    WindowInput,
};
#[cfg(feature = "solver-diffsol")]
mod shooting;
pub use dynamics::SimulationProfile;
pub use fitting::{
    Covariance, FitDeclaration, FitDeclarations, FitDerivatives, FitDiagnostic,
    FitExecutionDocument, FitOptimizer, FitPreparationDocument, FitProfile, FitProfileDocument,
    FitReport, FitResidualReport, FitResidualTermination, FitRoute, FitRule, FitUncertainty,
    FitWithheld, Interval, IntervalBound, PreparedFit, ProfileChain, ProfileControls,
    ProfileFailure, ProfilePoint, ProfileWorkerFailure, ResidualLmCriteria, ResidualLmStopping,
};
#[cfg(feature = "solver-diffsol")]
pub use shooting::{
    PathBound, ShootingControl, ShootingEvaluationPolicy, ShootingMethod, ShootingObjective,
    ShootingProblem, ShootingProfile, ShootingReport,
};
mod physical_inspection;
pub use physical_inspection::{
    PhysicalInspectionDocument, PhysicalInspectionQuery, PhysicalInspectionRequest,
    PhysicalQuantityDescription,
};
mod physical;
pub use physical::{PhysicalAdmissionReceipt, PhysicalContext};
mod physical_values;
pub use physical_values::{
    DiagnosticSamples, InitializationStages, PrimalCoordinates, PrimalValues, StateTolerances,
};
mod simulation_request;
pub use simulation_request::{PhysicalFitRequest, PhysicalSimulationRequest};
mod lifecycle;
pub(crate) mod modeling;
pub(crate) mod portable_payload;
mod project_closure;
mod project_contexts;
mod project_portability;
mod projects;
pub use lifecycle::*;
pub use project_portability::*;
mod research;
mod research_publication;
pub use research::{
    ResearchDataObject, ResearchDatasetRequest, ResearchObservation, ResearchReadRequest,
    ResearchReadResult, ResearchSelectionRequest, ResearchSource,
};
pub use research_publication::{
    ResearchParameterTarget, ResearchPublicationIntent, ResearchPublicationPreview,
    ResearchPublicationRequest,
};
#[cfg(all(test, feature = "canonical-tests"))]
mod projects_consumer_tests;
pub(crate) use modeling::PackageAdmission;
pub use modeling::documents::{
    ConformanceControls, DeclarationEdit, DeclarationInventory, DiagnosticSamplesControls,
    InspectionConnection, InspectionExecution, InspectionInstance, InspectionLineage,
    InspectionMember, InspectionPopulation, InspectionPort, JacobianDiagnosticControls,
    KnowledgeControls, LinearDiagnosticControls, ModelingDeclarationPage,
    ModelingDeclarationRequest, ModelingInspection, ModelingInspectionRequest,
    PureConformanceControls, SourceDelta, SourceDocumentDelta, SourceFitDelta,
};
pub use projects::{
    ProjectDraftDelta, ProjectDraftDocuments, ProjectDraftSaveIntent, ProjectReference,
    ProjectSelectRequest, ProjectSelection,
};
pub mod uncertainty;
#[cfg(feature = "solver-idas")]
pub use modeling::ConsistentInitializationResult;
pub use modeling::ModelingNativeAnalysis;
pub use modeling::{
    DeclaredExecution, DeclaredProcedure, DiagnosticSampleStop, DiscreteInitialization,
    ElasticObservation, InitializationOverrides, ModelingAnalysis, ModelingCheck,
    ModelingComparisonObservation, ModelingComparisonRequest, ModelingConformanceCheck,
    ModelingConformancePolicy, ModelingConformanceReport, ModelingDiagnosticPolicy,
    ModelingDiagnosticPreparation, ModelingDiagnosticSamples, ModelingDiagnostics,
    ModelingElasticAttempt, ModelingFixtureSelection, ModelingInfeasibilityCertificate,
    ModelingInitialization, ModelingInitializationAttempt, ModelingInitializationReport,
    ModelingInitializationStep, ModelingKnowledge, ModelingNonlinearExplanation,
    ModelingNonlinearPolicy, ModelingObservations, ModelingPackage, ModelingReport, ModelingResult,
    ModelingSimulation, ModelingSolvePreparation, ModelingSourceMap, ModelingSourceMapDocument,
    ModelingTrajectory, OwnedDeclarations, PreparationDiscoveryReport, PreparationDiscoveryRequest,
    PreparationEligibility, PreparationProductKind, ProjectChangeImpact, ProjectComparison,
    ProjectComparisonSelection, ProjectDocumentComposition, ProjectDocumentSource,
    ProjectMergeChoice, ProjectMergeConflict, ProjectMergeConflictKind, ProjectMergeIntent,
    ProjectMergeMode, ProjectMergePreview, ProjectMergeProposal, ProjectMergeResolution,
    ProjectObjectChange, ProjectSourceDetail, ProjectSourceObject, ResidentPreparation,
    SimulationSampleAttempt, SimulationSampleObserver, StartSource, conform_pure_documents,
    with_simulation_sample_wait,
};
#[cfg(feature = "solver-highs")]
pub use modeling::{ModelingJacobianOptimization, ModelingLinearDiagnostics};
#[cfg(all(test, feature = "canonical-tests"))]
mod caller_lifetime_tests;
#[cfg(test)]
mod data_documents_tests;
#[cfg(all(test, feature = "canonical-tests"))]
mod durable_tests;
mod local_analysis;
mod modeling_results;
mod results;
mod run;
mod simulation_results;
#[cfg(test)]
mod study_tests;
#[cfg(test)]
pub(crate) mod tests;
#[cfg(test)]
#[cfg(any(feature = "canonical-tests", feature = "solver-ipopt"))]
mod worker_tests;
use crate::{SharedRuntime, math::MathRuntimeError};
use pse_engine::{
    EngineError,
    session::{BoundModel, EngineFactory},
};
pub use run::{RunHandle, RunResult, StoredStart};
mod run_outcome;
pub use run_outcome::{HorizonRunOutcome, RunOutcome, RunReport, RunRequest};
mod live_simulation;
pub use live_simulation::{
    LiveSampleAttempt, LiveSampleBatch, LiveSimulationCursor, LiveSimulationEvent,
    LiveSimulationOptions,
};
use std::sync::Arc;
mod canonical;
mod services;
pub use services::{
    ActivityAdmission, ExecutionServices, PreparationServices, PublicationServices, SourceServices,
};
mod source_settlement;
pub use canonical::{CanonicalDeployment, DeploymentHost, OuterAttestation};
pub use source_settlement::{SourceAcknowledgement, SourceSettlement};

/// Effect knowledge supplied by the selected lazy-admission phase, never inferred
/// from the scientific failure class or an absent receipt.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AdmissionEffect {
    /// The owner established that this admission published no source.
    NoPublication,
    /// The selected source publication returned its original acknowledgement.
    SourceAcknowledged {
        /// Original acknowledged source revision, retained for readback.
        revision: String,
    },
    /// The publication entry point failed without establishing its effect outcome.
    UnresolvedPublication,
}
impl AdmissionEffect {
    pub(super) const fn as_str(&self) -> &'static str {
        match self {
            Self::NoPublication => "no_publication",
            Self::SourceAcknowledged { .. } => "source_acknowledged",
            Self::UnresolvedPublication => "unresolved_publication",
        }
    }
}

/// Errors retain the native/physical/authoring cause; no string matching or fallback.
#[derive(Debug, thiserror::Error)]
pub enum WorkflowError {
    /// Original operation failure and a separately attempted settlement failure.
    #[error("operation failed: {operation}; settlement also failed: {cleanup}")]
    Settlement {
        /// Original operation or earlier settlement cause.
        #[source]
        operation: Box<WorkflowError>,
        /// Independently retained settlement cause.
        cleanup: Box<WorkflowError>,
    },
    /// The admitted primary drained or changed before finalization; records remain recoverable.
    #[error(
        "study {study} execution is unavailable: its admitted managed primary cannot finish this dispatch; durable records require explicit recovery"
    )]
    StudyExecutionUnavailable {
        /// Exact durable study whose unfinished records remain authoritative.
        study: StudyId,
    },
    /// Typed absence of an explicitly requested immutable scientific seed.
    #[error(transparent)]
    SeedRead(#[from] SeedReadError),
    /// Study creation/staging may have committed; retain its exact recovery identity.
    #[error("study {study} creation requires explicit recovery: {source}")]
    StudyCreationUnsettled {
        /// Exact locally issued study occurrence; never reused for a fresh request.
        study: StudyId,
        /// Original staging/creation failure, without inferred terminal science.
        #[source]
        source: Box<WorkflowError>,
        /// Failure to prove pre-header abandonment (an existing header may be live).
        cleanup: Box<pse_operations::canonical::CanonicalError>,
    },
    /// Scientific transfer or graph handoff needs exact attributed cleanup/recovery.
    #[error("artifact {artifact_key} transfer requires recovery: {source}",artifact_key=.artifact.key)]
    ArtifactTransferUnsettled {
        /// Committed or uncertain analysis occurrence, when one was issued.
        analysis: Option<String>,
        /// Original sealed artifact, including inactive or absent staging outcomes.
        artifact: Box<pse_operations::canonical_artifacts::Artifact>,
        /// Exact internal obligation kind; no other owner's lifetime is released.
        owner_kind: String,
        /// Exact internal obligation identity, including random staging owners.
        owner: String,
        /// Original read grant whose explicit release may be uncertain; its finite lease remains authoritative.
        read: Option<String>,
        /// Original typed transfer/cleanup failure.
        #[source]
        source: Box<WorkflowError>,
    },
    /// Original version-first JSON refusal with structured native decode facts.
    #[error(transparent)]
    DocumentDecode(#[from] pse_model::document::JsonDecodeError),
    /// Bounded scientific IPC framing, schema, shape or decoder refusal.
    #[error(transparent)]
    ResultBlock(#[from] result_blocks::ResultBlockError),
    /// Canonical revision, protected selection or portable product failure.
    #[error(transparent)]
    Canonical(#[from] pse_operations::canonical::CanonicalError),
    /// Source-attributed selected-model or execution-boundary failure.
    #[error(transparent)]
    Boundary(#[from] Box<BoundaryDiagnostic>),
    /// A causal unit admission refusal preserves both requested source identities
    /// and the original compiler, policy or selected solver cause.
    #[error("{diagnostic}")]
    ConditionalAdmission {
        /// Structured selected-unit boundary attribution.
        diagnostic: Box<BoundaryDiagnostic>,
        /// Original typed failure before any unit iteration.
        #[source]
        cause: Box<MathRuntimeError>,
    },
    /// Original modeling admission facts with their authored lineage and typed native cause.
    #[error("{diagnostic}")]
    ModelingAdmission {
        /// Original identities and available model source paths.
        diagnostic: Box<BoundaryDiagnostic>,
        /// Retained route decision or structural failure before native execution.
        #[source]
        cause: Box<MathRuntimeError>,
    },
    /// Invalid model, preparation, runtime or solver state.
    #[error(transparent)]
    Math(#[from] MathRuntimeError),
    /// Storage/query/declared relation failure, including settlement effects.
    #[error(transparent)]
    Engine(#[from] EngineError),
    /// Source document syntax or declared boundary failure.
    #[error(transparent)]
    Authoring(#[from] pse_admission::authoring_driver::DriverError),
    /// Shared immutable encoding failure with its full diagnostic cause.
    #[error(transparent)]
    Shared(Arc<WorkflowError>),
    /// Explicitly requested recording/import has no configured sink.
    #[error("runtime has no canonical recording sink")]
    RecordingUnavailable,
    /// This logical runtime and its aliases no longer admit new work.
    #[error("runtime caller is closed")]
    CallerClosed,
    /// A complete diagnostic for invalid API input.
    #[error("native workflow contract: {0}")]
    Input(String),
    /// A producer-owned deterministic refusal of immutable scientific inputs.
    #[error(transparent)]
    ScientificRefusal(Box<BoundaryDiagnostic>),
    /// Lazy admission retains the original failure and the actual publication phase.
    #[error("{cause}")]
    Admission {
        /// Actual effect knowledge at the failure boundary.
        effect: AdmissionEffect,
        /// Original typed cause, including any unsettled canonical occurrence/readback.
        #[source]
        cause: Box<WorkflowError>,
    },
    /// A finite runtime/storage/effort allowance refused this operation.
    #[error("native workflow resource limit: {0}")]
    Resource(String),
    /// Authenticated retained content failed its owning integrity contract.
    #[error("native workflow integrity: {0}")]
    Integrity(String),
    /// An executing service or provider is unavailable or failed.
    #[error("native workflow infrastructure: {0}")]
    Infrastructure(String),
    /// The selected operation has no supported implementation under this configuration.
    #[error("native workflow unsupported: {0}")]
    Unsupported(String),
    /// An internal operation postcondition failed, distinct from authored input refusal.
    #[error("native workflow invariant: {0}")]
    Internal(String),
    /// Original typed source failure, preserving diagnostic facts and causal structure.
    #[error(transparent)]
    Typed(pse_model::diagnostic::DiagnosticCause),
}
impl WorkflowError {
    pub(crate) fn settle<T>(
        operation: Result<T, Self>,
        cleanup: Result<(), Self>,
    ) -> Result<T, Self> {
        match (operation, cleanup) {
            (Ok(value), Ok(())) => Ok(value),
            (Err(error), Ok(())) | (Ok(_), Err(error)) => Err(error),
            (Err(operation), Err(cleanup)) => Err(Self::Settlement {
                operation: Box::new(operation),
                cleanup: Box::new(cleanup),
            }),
        }
    }
}
impl From<BoundaryDiagnostic> for WorkflowError {
    fn from(error: BoundaryDiagnostic) -> Self {
        Self::Boundary(Box::new(error))
    }
}
pse_diagnostics::impl_diagnostic! {
    WorkflowError,
    code(this) {match this {Self::Input(_)|Self::DocumentDecode(_)|Self::RecordingUnavailable|Self::CallerClosed=>Some(pse_diagnostics::DiagnosticCode::WorkflowInput),Self::Internal(_)|Self::StudyExecutionUnavailable{..}=>Some(pse_diagnostics::DiagnosticCode::WorkflowInternal),Self::Resource(_)=>Some(pse_diagnostics::DiagnosticCode::RuntimeResourceLimit),Self::Integrity(_)|Self::Infrastructure(_)=>Some(pse_diagnostics::DiagnosticCode::RuntimeInfrastructure),Self::Unsupported(_)=>Some(pse_diagnostics::DiagnosticCode::NativeUnsupported),Self::Typed(_)=>None,_=>None}},
    forward(this) {match this {Self::SeedRead(e)=>Some(e),Self::DocumentDecode(e)=>Some(e),Self::ResultBlock(e)=>Some(e),Self::Canonical(e)=>Some(e),Self::Boundary(e)|Self::ScientificRefusal(e)=>Some(e.as_ref()),Self::Admission{cause,..}=>Some(cause.as_ref()),Self::Typed(e)=>Some(e.as_ref()),Self::ConditionalAdmission{diagnostic,..}|Self::ModelingAdmission{diagnostic,..}=>Some(diagnostic.as_ref()),Self::Math(e)=>Some(e),Self::Engine(e)=>Some(e),Self::Authoring(e)=>Some(e),Self::Shared(e)=>Some(e.as_ref()),Self::Settlement{operation,..}=>Some(operation.as_ref()),Self::StudyCreationUnsettled{source,..}|Self::ArtifactTransferUnsettled{source,..}=>Some(source.as_ref()),_=>None}},
    help(_this){None},
    related(this) {
        match this {
            Self::Settlement { operation, cleanup } => {
                let causes: [&dyn pse_diagnostics::TypedDiagnostic; 2] = [operation.as_ref(), cleanup.as_ref()];
                Some(Box::new(causes.into_iter()))
            }
            _ => None,
        }
    },source(_this){None}
}
pub(crate) fn scientific_refusal(
    class: pse_model::diagnostic::BoundaryClass,
    stage: pse_diagnostics::DiagnosticStage,
    rule: pse_diagnostics::DiagnosticRule,
    sources: Vec<pse_ids::SemanticId>,
    message: impl Into<String>,
) -> WorkflowError {
    let mut diagnostic = BoundaryDiagnostic::new(class, stage, sources, rule);
    diagnostic.observations.insert(
        "detail".into(),
        pse_model::diagnostic::Observation::Text(message.into()),
    );
    WorkflowError::ScientificRefusal(Box::new(diagnostic))
}
pub(crate) fn contract(message: impl Into<String>) -> WorkflowError {
    WorkflowError::Input(message.into())
}
/// A selected scientific or diagnostic task cannot obtain its finite deadline.
pub(crate) fn deadline_limit(detail: &'static str) -> WorkflowError {
    WorkflowError::Math(
        pse_backend_native::ProblemError::Limit {
            kind: pse_backend_native::LimitKind::Time,
            detail: detail.into(),
        }
        .into(),
    )
}
fn math(error: impl Into<pse_math::MathError>) -> WorkflowError {
    WorkflowError::Math(MathRuntimeError::Math(error.into()))
}
/// A typed modeling refusal keeps its class and source through the compiler boundary.
fn modeling_error(error: pse_modeling::ModelingError) -> WorkflowError {
    WorkflowError::Math(MathRuntimeError::Compile(error.into()))
}
fn relation(error: pse_relations::RelationError) -> WorkflowError {
    WorkflowError::Engine(error.into())
}

/// Public handle borrowing one deployment budget and the existing native services.
#[derive(Clone, Debug)]
pub struct Runtime {
    pub(crate) shared: Arc<SharedRuntime>,
    pub(crate) model: BoundModel,
    pub(crate) canonical: Option<CanonicalDeployment>,
    caller: Arc<RuntimeCaller>,
    outer_attestation: Option<OuterAttestation>,
    /// Application runs retain scientific outcomes; ephemerality is an explicit choice.
    pub(crate) durability: Durability,
    recording_operations: Option<Box<Operations>>,
    physical_cache: Arc<physical_cache::PhysicalCache>,
    managed_worker: Option<NativeStudyWorkerObserver>,
}
/// Select execution durability within this runtime's canonical deployment and pool.
#[derive(Clone, Debug)]
pub enum DurabilitySelection {
    /// Execute without retaining canonical run records.
    Ephemeral,
    /// Retain canonical run records with the selected worker and lease policy.
    Durable {
        /// Worker identity used for authority claims.
        worker: String,
        /// Finite lease and renewal policy.
        policy: LeasePolicy,
    },
}
impl Runtime {
    /// Passive diagnostic registry, available before preparing or starting a run.
    pub fn observations(&self) -> &Arc<crate::observation::Diagnostics> {
        &self.caller.observations
    }

    /// Release this logical caller and all of its aliases. Already admitted work
    /// retains its continuation authority; physical drivers belong to the deployment.
    pub fn close(&self) {
        self.caller.close();
    }

    /// Snapshot every canonical owner of a closed isolated-fixture caller.
    /// An active continuation could still attach a late owner, so it remains
    /// protective rather than being omitted from a physical-drain claim.
    #[doc(hidden)]
    pub fn fixture_connection_owners(
        &self,
    ) -> Result<Vec<pse_operations::canonical::CanonicalStore>, WorkflowError> {
        let state = self
            .caller
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !state.closed {
            return Err(contract(
                "fixture physical drain requires logical caller close",
            ));
        }
        if state.active != 0 {
            return Err(pse_operations::canonical::CanonicalError::ConnectionBusy {
                activities: state.active,
            }
            .into());
        }
        if state
            .stores
            .iter()
            .any(|store| !store.database().starts_with("canonical_test_"))
        {
            return Err(contract(
                "fixture physical drain requires isolated canonical owners",
            ));
        }
        Ok(state.stores.clone())
    }

    /// Actual immutable native admission owner retained by this runtime's bound model.
    pub fn validation_context(&self) -> Arc<pse_relations::validate::ValidationContext> {
        self.model.validation_context()
    }

    /// Clear retained executable programs. Existing workers keep their owners and remain valid.
    pub fn clear_program_cache(&self) {
        self.shared.math().clear_program_cache();
        self.physical_cache.clear();
    }
    /// Attach to the already configured shared deployment; creates no second executor or budget.
    /// Scientific source revisions and compilation products use the supplied canonical deployment.
    /// # Errors
    /// A consumed factory service belongs to another deployment.
    pub fn from_shared(
        shared: Arc<SharedRuntime>,
        model: BoundModel,
        canonical: CanonicalDeployment,
    ) -> Result<Self, WorkflowError> {
        Self::check_deployment(&shared, model.factory())?;
        let canonical = canonical.independent_caller()?;
        let durability = Durability::Durable(Box::new(Operations::from_store(
            canonical.store().clone(),
            Operations::process_worker("runtime"),
            LeasePolicy::default(),
            shared.pool(),
        )));
        let caller = Arc::new(RuntimeCaller::new(
            Some(canonical.store().clone()),
            model.validation_context(),
            Arc::downgrade(&shared.pool()),
        ));
        let physical_cache = Arc::new(physical_cache::PhysicalCache::default());
        let component: Arc<dyn pse_engine::cache_service::CacheComponent> = physical_cache.clone();
        shared.caches().register_component(&component);
        Ok(Self {
            shared,
            model,
            caller,
            outer_attestation: Some(canonical.attestation()),
            canonical: Some(canonical),
            recording_operations: match &durability {
                Durability::Durable(operations) => Some(operations.clone()),
                Durability::Ephemeral => None,
            },
            durability,
            physical_cache,
            managed_worker: None,
        })
    }
    /// Construct an explicitly ephemeral caller over the existing deployment. No
    /// canonical client, producer observation, executor or independent budget is made.
    pub fn from_shared_local(
        shared: Arc<SharedRuntime>,
        model: BoundModel,
    ) -> Result<Self, WorkflowError> {
        Self::check_deployment(&shared, model.factory())?;
        let caller = Arc::new(RuntimeCaller::new(
            None,
            model.validation_context(),
            Arc::downgrade(&shared.pool()),
        ));
        let physical_cache = Arc::new(physical_cache::PhysicalCache::default());
        let component: Arc<dyn pse_engine::cache_service::CacheComponent> = physical_cache.clone();
        shared.caches().register_component(&component);
        Ok(Self {
            shared,
            model,
            canonical: None,
            caller,
            outer_attestation: None,
            durability: Durability::Ephemeral,
            recording_operations: None,
            physical_cache,
            managed_worker: None,
        })
    }
    /// Attach an explicitly opened import/recording boundary to this same caller.
    /// This creates no executor, model binding, budget or independent close authority.
    pub fn with_canonical(&self, canonical: CanonicalDeployment) -> Result<Self, WorkflowError> {
        if self.canonical.is_some() {
            return Err(contract("runtime recording sink is already configured"));
        }
        let canonical = canonical.independent_caller()?;
        let mut state = self
            .caller
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let continuing = RUNTIME_CONTINUATION
            .try_with(|activity| Arc::ptr_eq(&activity.0.caller, &self.caller))
            .unwrap_or(false);
        if state.closed && !continuing {
            canonical.store().close();
            return Err(WorkflowError::CallerClosed);
        }
        let recording_operations = Some(Box::new(Operations::from_store(
            canonical.store().clone(),
            Operations::process_worker("runtime"),
            LeasePolicy::default(),
            self.shared.pool(),
        )));
        state.stores.push(canonical.store().clone());
        Ok(Self {
            outer_attestation: self.outer_attestation.or(Some(canonical.attestation())),
            canonical: Some(canonical),
            recording_operations,
            ..self.clone()
        })
    }
    /// Actual supplied outer provenance is separate from storage configuration and trust.
    pub fn outer_attestation(&self) -> Option<OuterAttestation> {
        self.outer_attestation
    }
    /// Retain a composition root's actual observation, without granting executable reuse.
    pub fn with_attestation(mut self, attestation: OuterAttestation) -> Self {
        self.outer_attestation = Some(attestation);
        self
    }
    /// One admission scope shared by the active operation's children.
    pub fn admission_context(&self) -> Arc<pse_relations::validate::ValidationContext> {
        if let Ok(activity) = RUNTIME_CONTINUATION.try_with(Clone::clone)
            && Arc::ptr_eq(&activity.0.caller, &self.caller)
        {
            return activity.0.validation.clone();
        }
        Arc::new(self.model.admission_context())
    }
    /// Admit an integration before polling its first effect. Required asynchronous
    /// children retain this caller and admission scope until the future drains.
    /// # Errors
    /// The logical caller is closed, or the integration returns its typed failure.
    pub async fn scope_activity<T, F>(&self, operation: F) -> Result<T, WorkflowError>
    where
        F: Future<Output = Result<T, WorkflowError>>,
    {
        let activity = self.admit_activity()?;
        activity.scope(operation).await
    }
    /// Local caller admission is independent of recording-service liveness.
    pub(crate) fn admit_activity(&self) -> Result<RuntimeActivity, WorkflowError> {
        self.caller.admit()
    }
    fn check_deployment(
        shared: &SharedRuntime,
        sessions: &EngineFactory,
    ) -> Result<(), WorkflowError> {
        let pool = shared.pool();
        let expected = shared.runtime_env();
        let state = sessions.native_state();
        let actual = state.runtime_env();
        let same = |matches: bool, owner: &str| {
            if matches {
                Ok(())
            } else {
                Err(contract(format!(
                    "workflow factory {owner} belongs to another deployment"
                )))
            }
        };
        same(Arc::ptr_eq(sessions.pool(), &pool), "allocation pool")?;
        same(
            Arc::ptr_eq(&actual.memory_pool, &pool),
            "native allocation pool",
        )?;
        same(
            Arc::ptr_eq(&actual.disk_manager, &expected.disk_manager),
            "spill manager",
        )?;
        same(
            Arc::ptr_eq(&actual.cache_manager, &expected.cache_manager),
            "native cache manager",
        )?;
        same(
            Arc::ptr_eq(
                &actual.object_store_registry,
                &expected.object_store_registry,
            ),
            "object store registry",
        )?;
        let caches = state
            .config()
            .get_extension::<pse_engine::cache_service::NativeCacheService>()
            .ok_or_else(|| contract("workflow factory cache service is absent"))?;
        same(Arc::ptr_eq(&caches, shared.caches()), "cache service")?;
        same(Arc::ptr_eq(caches.pool(), &pool), "cache allocation pool")?;
        let cpu = state
            .config()
            .get_extension::<pse_engine::resources::CpuAdmission>()
            .ok_or_else(|| contract("workflow factory CPU admission is absent"))?;
        same(
            Arc::ptr_eq(&cpu.permits, &shared.compiler_cpu()),
            "CPU admission",
        )?;
        same(
            cpu.workers.get() as usize == shared.budget().threads.pool_threads.get(),
            "CPU deployment width",
        )?;
        Ok(())
    }
    /// Actual configured recording/import boundary, when explicitly requested.
    pub fn canonical(&self) -> Result<&CanonicalDeployment, WorkflowError> {
        self.canonical
            .as_ref()
            .ok_or(WorkflowError::RecordingUnavailable)
    }
    /// The same deployment under an explicit durability class (ADR-0114 Outcome 16).
    /// Packages and preparations made from the returned runtime run under it.
    /// # Errors
    /// No canonical deployment is attached for a durable selection.
    pub fn with_durability(
        mut self,
        selection: DurabilitySelection,
    ) -> Result<Self, WorkflowError> {
        self.durability = match selection {
            DurabilitySelection::Ephemeral => Durability::Ephemeral,
            DurabilitySelection::Durable { worker, policy } => {
                Durability::Durable(Box::new(Operations::from_store(
                    self.canonical()?.store().clone(),
                    worker,
                    policy,
                    self.shared.pool(),
                )))
            }
        };
        if let Durability::Durable(operations) = &self.durability {
            self.recording_operations = Some(operations.clone());
        }
        Ok(self)
    }
    /// The durability class of this runtime's runs.
    pub const fn durability(&self) -> &Durability {
        &self.durability
    }
    /// Whether the native BDF semi-explicit adapter is linked in this deployment.
    pub fn simulation_available(&self) -> bool {
        cfg!(feature = "solver-diffsol")
    }
    /// Explicitly provision the selected canonical database through its original owner.
    /// Host bindings supply the already admitted shared runtime/environment; no retry is implied.
    /// # Errors
    /// Original selected-state/options and canonical schema installation failures.
    pub async fn initialize_database(
        state: &std::path::Path,
        database: &str,
    ) -> Result<(), WorkflowError> {
        let options = pse_operations::canonical::CanonicalOptions::from_state_for_database(
            state,
            Some(database),
        )?;
        pse_operations::canonical::CanonicalStore::initialize_database(&options).await?;
        Ok(())
    }
    /// Actual linked implementations; model eligibility is separately reported by preparation.
    /// The published rows are the capability records routing reads (F21).
    pub fn capabilities(&self) -> Vec<pse_model::generated::runtime::solver_capabilities::Row> {
        pse_backend_native::execution::LINKED.published()
    }
    /// The linked adapter's actual capability for one selected settings profile.
    /// # Errors
    /// The adapter is unlinked or the settings are foreign or unsupported.
    pub fn capability_for(
        &self,
        backend: pse_backend_native::solve::Backend,
        settings: &pse_backend_native::execution::BackendSettings,
    ) -> Result<pse_model::generated::runtime::solver_capabilities::Row, WorkflowError> {
        pse_backend_native::execution::LINKED
            .capability_for(backend, settings)
            .map_err(MathRuntimeError::from)
            .map_err(WorkflowError::from)
    }
    /// Existing typed native services, including cone preparation, initialization,
    /// graph analysis, MILP tears and declared recycle maps, use this same owner.
    pub fn native(&self) -> &Arc<crate::math::MathService> {
        self.shared.math()
    }
    /// Actual immutable native admission binding retained by this runtime.
    pub fn bound_model(&self) -> &BoundModel {
        &self.model
    }
    /// The complete schema authority for generated declaration/result rows.
    pub fn registry(&self) -> &Arc<pse_schema::Registry> {
        self.model.registry()
    }
    /// The engine factory sessions, artifacts and publications of this runtime use.
    pub fn sessions(&self) -> &Arc<EngineFactory> {
        self.model.factory()
    }
}

#[derive(Debug)]
struct RuntimeCaller {
    state: std::sync::Mutex<RuntimeCallerState>,
    validation: Arc<pse_relations::validate::ValidationContext>,
    observations: Arc<crate::observation::Diagnostics>,
    resources: std::sync::Weak<dyn datafusion::execution::memory_pool::MemoryPool>,
}
#[derive(Debug, Default)]
struct RuntimeCallerState {
    diagnostic_registration: Option<crate::observation::DiagnosticRegistration>,
    closed: bool,
    active: usize,
    stores: Vec<pse_operations::canonical::CanonicalStore>,
}
impl RuntimeCaller {
    fn new(
        store: Option<pse_operations::canonical::CanonicalStore>,
        validation: Arc<pse_relations::validate::ValidationContext>,
        resources: std::sync::Weak<dyn datafusion::execution::memory_pool::MemoryPool>,
    ) -> Self {
        let observations = crate::observation::process_diagnostics().clone();
        Self {
            state: std::sync::Mutex::new(RuntimeCallerState {
                diagnostic_registration: Some(observations.register()),
                stores: store.into_iter().collect(),
                ..Default::default()
            }),
            validation,
            observations,
            resources,
        }
    }
    fn sample_resources(&self, context: &pse_diagnostics::observation::ObservationContext) {
        if let Some(pool) = self.resources.upgrade() {
            match i64::try_from(pool.reserved()) {
                Ok(bytes) => context.integer("resource.shared_pool_reserved", "bytes", bytes),
                Err(_) => context.unavailable(
                    "resource.shared_pool_reserved",
                    "byte sample exceeds signed observation range",
                ),
            }
            context.source_ref(
                "resource.pool.scope",
                b"shared_runtime_current_not_operation_peak",
            );
        }
    }
    fn admit(self: &Arc<Self>) -> Result<RuntimeActivity, WorkflowError> {
        if let Ok(activity) = RUNTIME_CONTINUATION.try_with(Clone::clone)
            && Arc::ptr_eq(&activity.0.caller, self)
        {
            return Ok(activity);
        }
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if state.closed {
            return Err(WorkflowError::CallerClosed);
        }
        state.active = state
            .active
            .checked_add(1)
            .ok_or_else(|| contract("runtime activity extent"))?;
        let observation = match pse_diagnostics::observation::current() {
            Some(context) => context.child(
                "workflow.activity",
                pse_diagnostics::observation::Relationship::Nested,
            ),
            None => self.observations.hub().start("workflow.activity"),
        };
        self.sample_resources(&observation.context());
        Ok(RuntimeActivity(Arc::new(RuntimeActivityOwner {
            caller: self.clone(),
            validation: Arc::new(self.validation.new_scope()),
            observation: Some(observation),
            observation_returned: std::sync::atomic::AtomicBool::new(false),
            observation_interrupted: std::sync::atomic::AtomicBool::new(false),
        })))
    }
    fn close(&self) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.closed = true;
        if state.active == 0 {
            state.diagnostic_registration = None;
            for store in &state.stores {
                store.close();
            }
        }
    }
}
/// An admitted local continuation. Required children inherit it explicitly;
/// idle Runtime aliases cannot use it to bypass close.
#[derive(Clone, Debug)]
pub(crate) struct RuntimeActivity(Arc<RuntimeActivityOwner>);
#[derive(Debug)]
struct RuntimeActivityOwner {
    observation: Option<pse_diagnostics::observation::OperationGuard>,
    observation_returned: std::sync::atomic::AtomicBool,
    observation_interrupted: std::sync::atomic::AtomicBool,
    caller: Arc<RuntimeCaller>,
    validation: Arc<pse_relations::validate::ValidationContext>,
}
impl Drop for RuntimeActivityOwner {
    fn drop(&mut self) {
        // This marks the activity scope drained, not a scientific solution qualified.
        if let Some(observation) = self.observation.take() {
            self.caller.sample_resources(&observation.context());
            observation.context().boolean("activity.drained", true);
            if self
                .observation_returned
                .load(std::sync::atomic::Ordering::Acquire)
                && !self
                    .observation_interrupted
                    .load(std::sync::atomic::Ordering::Acquire)
            {
                observation.finish(pse_diagnostics::observation::Outcome::Completed);
            }
            // Otherwise the guard records an unexplained/interrupted scope drop.
        }
        let mut state = self
            .caller
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.active -= 1;
        if state.closed && state.active == 0 {
            state.diagnostic_registration = None;
            for store in &state.stores {
                store.close();
            }
        }
    }
}
tokio::task_local! { static RUNTIME_CONTINUATION: RuntimeActivity; }
struct ObservationScopeReturn {
    owner: Arc<RuntimeActivityOwner>,
    returned: bool,
}
impl ObservationScopeReturn {
    fn mark_returned(&mut self) {
        self.returned = true;
    }
}
impl Drop for ObservationScopeReturn {
    fn drop(&mut self) {
        if self.returned {
            self.owner
                .observation_returned
                .store(true, std::sync::atomic::Ordering::Release);
        } else {
            self.owner
                .observation_interrupted
                .store(true, std::sync::atomic::Ordering::Release);
        }
    }
}
impl RuntimeActivity {
    fn observation_context(&self) -> Option<pse_diagnostics::observation::ObservationContext> {
        // A nested continuation keeps its actual lane/producer attribution. A different
        // runtime activity enters its own node instead of borrowing another caller's one.
        if RUNTIME_CONTINUATION
            .try_with(|current| Arc::ptr_eq(&current.0, &self.0))
            .unwrap_or(false)
            && let Some(context) = pse_diagnostics::observation::current()
        {
            return Some(context);
        }
        self.0
            .observation
            .as_ref()
            .map(|observation| observation.context())
    }
    pub(crate) fn sync_scope<T>(&self, action: impl FnOnce() -> T) -> T {
        let context = self.observation_context();
        let _entered = context
            .as_ref()
            .map(pse_diagnostics::observation::ObservationContext::enter);
        let mut completion = ObservationScopeReturn {
            owner: self.0.clone(),
            returned: false,
        };
        let output = RUNTIME_CONTINUATION.sync_scope(self.clone(), action);
        completion.returned = true;
        output
    }
    pub(crate) fn scope<F: Future>(&self, future: F) -> impl Future<Output = F::Output> + use<F> {
        let context = self.observation_context();
        let mut completion = ObservationScopeReturn {
            owner: self.0.clone(),
            returned: false,
        };
        // Inspect stores F once and owns the completion guard through cancellation.
        // Another async forwarding layer duplicates large nested preparation futures.
        let future = futures_util::FutureExt::inspect(future, move |_| completion.mark_returned());
        RUNTIME_CONTINUATION.scope(self.clone(), crate::observation::scope(context, future))
    }
}

pub(crate) fn inherit_runtime_activity<F: Future>(future: F) -> impl Future<Output = F::Output> {
    let activity = RUNTIME_CONTINUATION.try_with(Clone::clone).ok();
    let context = crate::ExecutionContext::capture();
    let future = pse_operations::canonical::inherit_activity(future);
    // Authority retains its original owner; copied attribution alone cannot mint continuation.
    match activity {
        Some(activity) => {
            let context = context.with_observation(activity.observation_context());
            futures_util::future::Either::Left(activity.scope(context.instrument(future)))
        }
        None => futures_util::future::Either::Right(context.instrument(future)),
    }
}

// Drop the actual boxed state before its grant, including unpolled or completed work.
pub(crate) struct PrepaidRuntimeActivity<T> {
    future: futures_util::future::BoxFuture<'static, T>,
    _owner: Arc<pse_columnar::AllocationLease>,
    _retained: Option<Arc<pse_columnar::AllocationLease>>,
}
impl<T> std::fmt::Debug for PrepaidRuntimeActivity<T> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PrepaidRuntimeActivity")
            .finish_non_exhaustive()
    }
}
impl<T> Future for PrepaidRuntimeActivity<T> {
    type Output = T;
    fn poll(
        self: std::pin::Pin<&mut Self>,
        context: &mut std::task::Context<'_>,
    ) -> std::task::Poll<T> {
        worker::poll_worker_action(self.get_mut().future.as_mut(), context)
    }
}

/// Prepay state before construction, then inherit runtime activity through a thin handle.
#[inline(never)]
pub(crate) fn prepaid_runtime_activity<F, Fut>(
    pool: &Arc<dyn datafusion::execution::memory_pool::MemoryPool>,
    retained: Option<Arc<pse_columnar::AllocationLease>>,
    factory: F,
) -> Result<PrepaidRuntimeActivity<Fut::Output>, EngineError>
where
    F: FnOnce() -> Fut,
    Fut: Future + Send + 'static,
    Fut::Output: Send + 'static,
{
    let reservation =
        datafusion::execution::memory_pool::MemoryConsumer::new("runtime:inherited-future")
            .register(pool);
    reservation
        .try_grow(size_of::<Fut>())
        .map_err(EngineError::from)?;
    // A large inline future must not travel through each generic scope wrapper.
    let future = inherit_runtime_activity(worker::boxed_worker_action(factory));
    reservation
        .try_grow(size_of_val(&future))
        .map_err(EngineError::from)?;
    Ok(PrepaidRuntimeActivity {
        future: Box::pin(future),
        _owner: pse_columnar::AllocationLease::new(reservation),
        _retained: retained,
    })
}
#[cfg(test)]
mod local_caller_tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn prepaid_inherited_future_refuses_before_factory_construction() {
        let pool: Arc<dyn datafusion::execution::memory_pool::MemoryPool> =
            Arc::new(datafusion::execution::memory_pool::GreedyMemoryPool::new(1));
        let constructed = std::sync::atomic::AtomicBool::new(false);
        let payload = [0u8; 64 * 1024];
        let result = prepaid_runtime_activity(&pool, None, || {
            constructed.store(true, std::sync::atomic::Ordering::Release);
            async move { std::hint::black_box(payload) }
        });
        assert!(result.is_err());
        assert!(!constructed.load(std::sync::atomic::Ordering::Acquire));
        assert_eq!(pool.reserved(), 0);
    }

    #[test]
    fn prepaid_inherited_future_retains_grant_through_ready_and_child_destruction() {
        struct Witness {
            pool: Arc<dyn datafusion::execution::memory_pool::MemoryPool>,
            retained: std::sync::Weak<pse_columnar::AllocationLease>,
            destroyed: Arc<std::sync::atomic::AtomicBool>,
        }
        impl Drop for Witness {
            fn drop(&mut self) {
                assert!(self.pool.reserved() >= 64 * 1024);
                assert!(self.retained.upgrade().is_some());
                self.destroyed
                    .store(true, std::sync::atomic::Ordering::Release);
            }
        }
        for (poll, ready) in [(false, false), (true, false), (true, true)] {
            let pool: Arc<dyn datafusion::execution::memory_pool::MemoryPool> = Arc::new(
                datafusion::execution::memory_pool::GreedyMemoryPool::new(1 << 20),
            );
            let destroyed = Arc::new(std::sync::atomic::AtomicBool::new(false));
            let reservation = datafusion::execution::memory_pool::MemoryConsumer::new(
                "retained producer storage",
            )
            .register(&pool);
            reservation.try_grow(8192).unwrap();
            let retained = pse_columnar::AllocationLease::new(reservation);
            let retained_witness = Arc::downgrade(&retained);
            let witness = Witness {
                pool: pool.clone(),
                retained: retained_witness.clone(),
                destroyed: destroyed.clone(),
            };
            let payload = [0u8; 64 * 1024];
            let mut future = prepaid_runtime_activity(&pool, Some(retained), move || async move {
                if !ready {
                    std::future::pending::<()>().await;
                }
                std::hint::black_box(payload);
                drop(witness);
                7
            })
            .unwrap();
            assert!(pool.reserved() >= 64 * 1024);
            if poll {
                let mut context = std::task::Context::from_waker(std::task::Waker::noop());
                let result = std::pin::Pin::new(&mut future).poll(&mut context);
                assert_eq!(result.is_ready(), ready);
            }
            assert_eq!(destroyed.load(std::sync::atomic::Ordering::Acquire), ready);
            assert!(pool.reserved() >= 64 * 1024);
            drop(future);
            assert!(destroyed.load(std::sync::atomic::Ordering::Acquire));
            assert_eq!(pool.reserved(), 0);
            assert!(retained_witness.upgrade().is_none());
        }
    }

    #[test]
    fn fixture_snapshot_refuses_open_or_active_callers_and_allows_closed_empty() {
        let runtime = tests::runtime();
        assert!(runtime.fixture_connection_owners().is_err());
        let activity = runtime.admit_activity().unwrap();
        runtime.close();
        assert!(runtime.fixture_connection_owners().is_err());
        drop(activity);
        assert!(runtime.fixture_connection_owners().unwrap().is_empty());
    }

    #[test]
    fn observation_runtime_scope_stores_one_inline_future() {
        let runtime = tests::runtime();
        let activity = runtime.admit_activity().unwrap();
        let payload = [0u8; 64 * 1024];
        let future = async move {
            std::future::pending::<()>().await;
            std::hint::black_box(payload)
        };
        let bytes = size_of_val(&future);
        let scoped = activity.scope(future);
        assert!(size_of_val(&scoped) <= bytes + 1024);
        drop(scoped);
        drop(activity);
        runtime.close();
    }
    #[tokio::test]
    async fn observation_inherited_continuation_captures_lane_before_spawn() {
        let runtime = tests::runtime();
        let activity = runtime.admit_activity().unwrap();
        activity
            .scope(async {
                let parent = pse_diagnostics::observation::current().unwrap();
                let lane = parent.child(
                    "study.source.lane",
                    pse_diagnostics::observation::Relationship::Nested,
                );
                let expected = lane.context().id();
                #[expect(
                    clippy::async_yields_async,
                    reason = "construct under the lane context, then poll after leaving it to test captured inheritance"
                )]
                let inherited = crate::observation::scope(Some(lane.context()), async {
                    let pool: Arc<dyn datafusion::execution::memory_pool::MemoryPool> = Arc::new(
                        datafusion::execution::memory_pool::GreedyMemoryPool::new(1 << 20),
                    );
                    prepaid_runtime_activity(&pool, None, || async {
                        pse_diagnostics::observation::current().unwrap().id()
                    }).unwrap()
                })
                .await;
                assert_eq!(
                    pse_diagnostics::observation::current().unwrap().id(),
                    parent.id()
                );
                assert_eq!(tokio::spawn(inherited).await.unwrap(), expected);
                assert_eq!(
                    pse_diagnostics::observation::current().unwrap().id(),
                    parent.id()
                );
                lane.finish(pse_diagnostics::observation::Outcome::Completed);
            })
            .await;
        drop(activity);
        runtime.close();
    }
    #[tokio::test]
    async fn observation_nested_runtime_scope_preserves_actual_producer_attribution() {
        let runtime = tests::runtime();
        let activity = runtime.admit_activity().unwrap();
        activity
            .scope(async {
                let parent = pse_diagnostics::observation::current().unwrap();
                let producer = parent.child(
                    "study.actual.lane",
                    pse_diagnostics::observation::Relationship::Nested,
                );
                let expected = producer.context().id();
                crate::observation::scope(Some(producer.context()), async {
                    let nested = runtime.admit_activity().unwrap();
                    nested
                        .scope(async {
                            assert_eq!(
                                pse_diagnostics::observation::current().unwrap().id(),
                                expected
                            );
                        })
                        .await;
                    nested.sync_scope(|| {
                        assert_eq!(
                            pse_diagnostics::observation::current().unwrap().id(),
                            expected
                        )
                    });
                })
                .await;
                producer.finish(pse_diagnostics::observation::Outcome::Completed);
                assert_eq!(
                    pse_diagnostics::observation::current().unwrap().id(),
                    parent.id()
                );
            })
            .await;
        drop(activity);
        runtime.close();
    }
    #[tokio::test]
    async fn observation_drop_distinguishes_returned_work_from_cancelled_future() {
        let runtime = tests::runtime();
        let directory = tempfile::tempdir().unwrap();
        // This scope control requires admitted diagnostics. The process publisher
        // may legitimately contend with best-effort admission on the global hub.
        let caller = Arc::new(RuntimeCaller {
            state: std::sync::Mutex::new(RuntimeCallerState::default()),
            validation: runtime.validation_context(),
            observations: crate::observation::Diagnostics::new(
                Default::default(),
                directory.path().to_owned(),
            ),
            resources: Arc::downgrade(&runtime.shared.pool()),
        });
        let completed = caller.admit().unwrap();
        let completed_reader = completed.0.observation.as_ref().unwrap().reader();
        completed.scope(async {}).await;
        drop(completed);
        assert_eq!(
            completed_reader.snapshot(Duration::from_secs(60)).outcome,
            Some(pse_diagnostics::observation::Outcome::Completed)
        );
        let interrupted = caller.admit().unwrap();
        let reader = interrupted.0.observation.as_ref().unwrap().reader();
        // A preceding returned continuation must not disguise a later dropped one.
        interrupted.scope(async {}).await;
        let mut pending = Box::pin(interrupted.scope(std::future::pending::<()>()));
        assert!(futures_util::poll!(pending.as_mut()).is_pending());
        drop(pending);
        drop(interrupted);
        caller.close();
        runtime.close();
        let snapshot = reader.snapshot(Duration::from_secs(60));
        assert_eq!(
            snapshot.outcome,
            Some(pse_diagnostics::observation::Outcome::Interrupted)
        );
        assert!(snapshot.coverage.interrupted);
    }
    #[tokio::test]
    async fn local_close_refuses_idle_alias_and_keeps_admitted_children() {
        let runtime = tests::runtime();
        let alias = runtime.clone();
        let activity = runtime.admit_activity().unwrap();
        activity
            .scope(async {
                let child = alias.clone();
                let inherited = inherit_runtime_activity(async move {
                    let nested = child.admit_activity().unwrap();
                    nested
                        .scope(async {
                            assert!(child.canonical().is_err());
                        })
                        .await;
                });
                runtime.close();
                tokio::spawn(inherited).await.unwrap();
            })
            .await;
        drop(activity);
        assert!(matches!(
            alias.admit_activity(),
            Err(WorkflowError::CallerClosed)
        ));
        let called = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let observed = called.clone();
        let refused = runtime
            .scope_activity(async move {
                observed.store(true, std::sync::atomic::Ordering::Release);
                Ok(())
            })
            .await;
        assert!(matches!(refused, Err(WorkflowError::CallerClosed)));
        assert!(!called.load(std::sync::atomic::Ordering::Acquire));
    }
    #[tokio::test]
    async fn admitted_integration_keeps_one_continuation_across_acquisition_and_operation() {
        let runtime = tests::runtime();
        let (acquired, observed) = tokio::sync::oneshot::channel();
        let (resume, released) = tokio::sync::oneshot::channel();
        let executing = runtime.clone();
        let task = tokio::spawn(async move {
            executing
                .scope_activity(async {
                    // This barrier models an acquired integration before its actual operation.
                    let scope = executing.admission_context();
                    acquired
                        .send(())
                        .map_err(|_| contract("acquisition observer dropped"))?;
                    released
                        .await
                        .map_err(|_| contract("integration resume owner dropped"))?;
                    assert!(Arc::ptr_eq(&scope, &executing.admission_context()));
                    let declarations = pse_authoring::language::parse(
                        "package held {param a:Scalar=1;}",
                        pse_ids::SemanticId::NIL,
                        pse_authoring::language::IdentityPolicy::Named,
                        Default::default(),
                    )
                    .unwrap();
                    // Real local source admission starts after close and inherits only
                    // this admitted continuation, never authority from an idle alias.
                    let package = executing
                        .modeling_package(declarations.clone(), tests::physical())
                        .await?;
                    assert_eq!(
                        &*package
                            .declarations(&executing.preparation_services())
                            .await?,
                        declarations.as_slice()
                    );
                    Ok(package.revision_id().to_owned())
                })
                .await
        });
        observed.await.unwrap();
        runtime.close();
        let polled = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let fresh = polled.clone();
        let refused = runtime
            .scope_activity(async move {
                fresh.store(true, std::sync::atomic::Ordering::Release);
                Ok(())
            })
            .await;
        assert!(matches!(refused, Err(WorkflowError::CallerClosed)));
        assert!(!polled.load(std::sync::atomic::Ordering::Acquire));
        resume.send(()).unwrap();
        assert!(!task.await.unwrap().unwrap().is_empty());
        assert!(matches!(
            runtime.admit_activity(),
            Err(WorkflowError::CallerClosed)
        ));
    }
    #[tokio::test]
    async fn local_admission_scope_is_shared_only_by_its_operation() {
        let runtime = tests::runtime();
        let activity = runtime.admit_activity().unwrap();
        activity
            .scope(async {
                let first = runtime.admission_context();
                let second = runtime.admission_context();
                assert!(Arc::ptr_eq(&first, &second));
                let nested = runtime.admit_activity().unwrap();
                nested
                    .scope(async {
                        assert!(Arc::ptr_eq(&first, &runtime.admission_context()));
                    })
                    .await;
            })
            .await;
        let next = runtime.admission_context();
        let other = runtime.admission_context();
        assert!(!Arc::ptr_eq(&next, &other));
    }
}

mod result_selection;
pub use result_selection::{PackResultPredicate, PackResultSelection, PackScientificState};

#[cfg(test)]
mod observation_inquiry_tests;

mod cohort_provider;
mod study_workbench;
pub use study_workbench::{
    CapturedStudyRow, FrozenStudyCohort, StudyCohortAvailability, StudyCohortClassification,
    StudyCohortClassificationPage, StudyCohortCursor, StudyCohortExportRecord,
    StudyCohortHeaderPage, StudyCohortMember, StudyCohortMemberPage, StudyCohortPageRequest,
    StudyCohortReceipt, StudyCohortRequest, StudyCohortScanProgress, StudyCohortScanState,
    StudyKpiAnalysis, StudyKpiExport, StudyKpiRequest, StudyWorkbenchCursor, StudyWorkbenchFilter,
    StudyWorkbenchHeader, StudyWorkbenchPage, StudyWorkbenchRequest,
};

mod study_observation;
pub use study_observation::{
    StudyObservationContext, StudyObservationContinuity, StudyObservationCursor,
    StudyObservationPage, StudyObservationPosition, StudyObservationRequest,
    StudyObservationStatus, StudyObservedSource, StudyObserver,
};
