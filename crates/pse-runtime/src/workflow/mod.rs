// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! One public native workflow. Model declarations are generated; mathematics stays in Rust libraries.
mod route_documents;
pub use route_documents::{EligibilityDocument, IneligibleDocument, RouteDocument};
mod controls;
pub use controls::{
    InventoryControls, ProgressControls, RunControls, StudyRunControls, StudyWaitControls,
};
mod completion;
mod connected_results;
pub mod result_blocks;
mod result_export;
mod result_projection;
pub use connected_results::CanonicalResultReader;
pub use result_export::{ResultCursor, ResultOrder, StrategyEventExport};
mod analyses;
pub use analyses::{AnalysisControls, AnalysisDirection, AnalysisHandle};
mod retention;
pub use retention::ResultReclamationPage;
mod diagnostic_documents;
pub use diagnostic_documents::{
    DiagnosticAnnotationDocument, DiagnosticCauseDocument, DiagnosticContextDocument,
    DiagnosticNoteDocument, DiagnosticSpanDocument,
};
pub(crate) mod diagnostic_rows;
mod diagnostics;
mod durable;
pub use completion::Completion;
pub use durable::{
    Durability, DurableRecord, LeasePolicy, Operations, Recovery, RunDurability, SeedReadError,
    TerminationCause, TerminationDetail,
};
mod progress_documents;
pub use progress_documents::{IncumbentDocument, ProgressEventDocument, ProgressMetricDocument};
mod progress;
pub use progress::ProgressStream;
mod bindings;
mod physical_cache;
mod worker;
pub use bindings::{
    AdmittedBinding, AdmittedBindingEntry, BindingAssignment, BindingQuantity, BindingTarget,
    PointOverlay,
};
pub use worker::{PhysicalSource, Processed, WorkerSettings};
mod study;
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
    MAXIMUM_STUDY_POINTS, PackageSources, PointAttemptOutcome, PointOutcome, PointStatus,
    StudyCancel, StudyDefinition, StudyHandle, StudyPlan, StudyPoint, StudyPointDefinition,
    StudyPointPolicy, StudyRequest, StudyResults, StudyStatus,
};
pub use study_operations::{
    AdmittedHorizonValues, ArrivalDocument, CaseOperation, ControllerOperation, EstimatorInput,
    EstimatorOperation, FitOperation, FitOperationSettings, HorizonBinding, HorizonInputDocument,
    HorizonOperation, HorizonSignalDocument, OperationRequest, OperationSource,
    PreparationSettings, PreparedStudyOperation, SimulationOperation, StudyOperation,
    StudyResultRole,
};
pub(crate) mod numerics;
mod objectives;
pub use objectives::{ModelingLevelsReport, ModelingObjectiveLevel};
mod staged;
mod strategies;
mod time;
pub use strategies::{
    AnalysisPort, CausalUnitRealization, CausalUnitRequest, ConicRequest, InitializationDocument,
    InitializationStageDocument, PreparedConic, RecycleRequest,
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
mod fitting;
mod horizon;
mod integrated;
pub use horizon::{
    AdvancedStep, Arrival, Horizon, HorizonController, HorizonDecision, HorizonEstimator,
    HorizonInput, HorizonReport, HorizonSignal, HorizonStep, WindowInput,
};
#[cfg(feature = "solver-diffsol")]
mod shooting;
pub use dynamics::SimulationProfile;
pub use fitting::{
    Covariance, FitDeclaration, FitDeclarations, FitDerivatives, FitDiagnostic,
    FitPreparationDocument, FitProfile, FitProfileDocument, FitReport, FitRule, FitUncertainty,
    FitWithheld, Interval, IntervalBound, PreparedFit, ProfileChain, ProfileControls,
    ProfileFailure, ProfilePoint, ProfileWorkerFailure,
};
#[cfg(feature = "solver-diffsol")]
pub use shooting::{
    PathBound, ShootingControl, ShootingMethod, ShootingObjective, ShootingProblem,
    ShootingProfile, ShootingReport,
};
mod physical;
pub use physical::PhysicalContext;
pub(crate) mod modeling;
pub(crate) use modeling::PackageAdmission;
pub use modeling::documents::{
    ConformanceControls, DeclarationEdit, DeclarationInventory, DiagnosticSamplesControls,
    InspectionConnection, InspectionExecution, InspectionInstance, InspectionLineage,
    InspectionMember, InspectionPort, JacobianDiagnosticControls, KnowledgeControls,
    LinearDiagnosticControls, ModelingInspection, PureConformanceControls,
};
pub mod uncertainty;
#[cfg(feature = "solver-idas")]
pub use modeling::ConsistentInitializationResult;
pub use modeling::ModelingNativeAnalysis;
pub use modeling::{
    DeclaredExecution, DeclaredProcedure, DiagnosticSampleStop, DiscreteInitialization,
    ElasticObservation, InitializationOverrides, ModelingAnalysis, ModelingCheck,
    ModelingConformanceCheck, ModelingConformancePolicy, ModelingConformanceReport,
    ModelingDiagnosticPolicy, ModelingDiagnosticPreparation, ModelingDiagnosticSamples,
    ModelingDiagnostics, ModelingElasticAttempt, ModelingFixtureSelection,
    ModelingInfeasibilityCertificate, ModelingInitialization, ModelingInitializationAttempt,
    ModelingInitializationReport, ModelingInitializationStep, ModelingKnowledge,
    ModelingNonlinearExplanation, ModelingNonlinearPolicy, ModelingObservations, ModelingPackage,
    ModelingReport, ModelingResult, ModelingSimulation, ModelingSolvePreparation,
    ModelingTrajectory, OwnedDeclarations, StartSource, conform_pure_documents,
};
#[cfg(feature = "solver-highs")]
pub use modeling::{ModelingJacobianOptimization, ModelingLinearDiagnostics};
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
mod worker_tests;
use crate::{SharedRuntime, math::MathRuntimeError};
use pse_engine::{EngineError, session::EngineFactory};
pub use run::{RunHandle, RunReport, RunRequest, RunResult, StoredStart};
use std::sync::Arc;
mod canonical;
pub use canonical::{CanonicalDeployment, OuterAttestation};

/// Errors retain the native/physical/authoring cause; no string matching or fallback.
#[derive(Debug, thiserror::Error)]
pub enum WorkflowError {
    /// Typed absence of an explicitly requested immutable scientific seed.
    #[error(transparent)]
    SeedRead(#[from] SeedReadError),
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
    Authoring(#[from] crate::authoring_driver::DriverError),
    /// Shared immutable encoding failure with its full diagnostic cause.
    #[error(transparent)]
    Shared(Arc<WorkflowError>),
    /// A complete diagnostic for invalid API input.
    #[error("native workflow contract: {0}")]
    Input(String),
    /// An internal operation postcondition failed, distinct from authored input refusal.
    #[error("native workflow invariant: {0}")]
    Internal(String),
    /// Original typed source failure, preserving diagnostic facts and causal structure.
    #[error(transparent)]
    Typed(pse_model::diagnostic::DiagnosticCause),
}
impl From<BoundaryDiagnostic> for WorkflowError {
    fn from(error: BoundaryDiagnostic) -> Self {
        Self::Boundary(Box::new(error))
    }
}
pse_diagnostics::impl_diagnostic! {
    WorkflowError,
    code(this) {match this {Self::Input(_)=>Some(pse_diagnostics::DiagnosticCode::WorkflowInput),Self::Internal(_)=>Some(pse_diagnostics::DiagnosticCode::WorkflowInternal),Self::Typed(_)=>None,_=>None}},
    forward(this) {match this {Self::SeedRead(e)=>Some(e),Self::ResultBlock(e)=>Some(e),Self::Canonical(e)=>Some(e),Self::Boundary(e)=>Some(e.as_ref()),Self::Typed(e)=>Some(e.as_ref()),Self::ConditionalAdmission{diagnostic,..}|Self::ModelingAdmission{diagnostic,..}=>Some(diagnostic.as_ref()),Self::Math(e)=>Some(e),Self::Engine(e)=>Some(e),Self::Authoring(e)=>Some(e),Self::Shared(e)=>Some(e.as_ref()),_=>None}},
    help(_this){None},related(_this){None},source(_this){None}
}
fn contract(message: impl Into<String>) -> WorkflowError {
    WorkflowError::Input(message.into())
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
    pub(crate) registry: Arc<pse_schema::Registry>,
    pub(crate) sessions: Arc<EngineFactory>,
    pub(crate) canonical: CanonicalDeployment,
    /// Application runs retain scientific outcomes; ephemerality is an explicit choice.
    pub(crate) durability: Durability,
    physical_cache: Arc<physical_cache::PhysicalCache>,
}
impl Runtime {
    pub(crate) fn validation_context(
        &self,
    ) -> Result<Arc<pse_relations::validate::ValidationContext>, WorkflowError> {
        Ok(self.sessions.validation_context(&self.registry)?)
    }

    /// Clear retained executable programs. Existing workers keep their owners and remain valid.
    pub fn clear_program_cache(&self) {
        self.shared.math().clear_program_cache();
        self.physical_cache.clear();
    }
    /// Attach to the already configured shared deployment; creates no second executor or budget.
    /// Scientific source revisions and compilation products use the supplied canonical deployment.
    pub fn from_shared(
        shared: Arc<SharedRuntime>,
        registry: Arc<pse_schema::Registry>,
        sessions: Arc<EngineFactory>,
        canonical: CanonicalDeployment,
    ) -> Self {
        let durability = Durability::Durable(Box::new(Operations::from_store(
            canonical.store().clone(),
            Operations::process_worker("runtime"),
            LeasePolicy::default(),
            shared.pool(),
        )));
        let physical_cache = Arc::new(physical_cache::PhysicalCache::default());
        let component: Arc<dyn pse_engine::cache_service::CacheComponent> = physical_cache.clone();
        shared.caches().register_component(&component);
        Self {
            shared,
            registry,
            sessions,
            canonical,
            durability,
            physical_cache,
        }
    }
    /// Configured canonical scientific deployment.
    pub fn canonical(&self) -> &CanonicalDeployment {
        &self.canonical
    }
    /// The same deployment under an explicit durability class (ADR-0114 Outcome 16).
    /// Packages and preparations made from the returned runtime run under it.
    #[must_use]
    pub fn with_durability(mut self, durability: Durability) -> Self {
        self.durability = durability;
        self
    }
    /// The durability class of this runtime's runs.
    pub const fn durability(&self) -> &Durability {
        &self.durability
    }
    /// Whether the native BDF semi-explicit adapter is linked in this deployment.
    pub fn simulation_available(&self) -> bool {
        cfg!(feature = "solver-diffsol")
    }
    /// Actual linked implementations; model eligibility is separately reported by preparation.
    /// The published rows are the capability records routing reads (F21).
    pub fn capabilities(&self) -> Vec<pse_model::generated::runtime::solver_capabilities::Row> {
        pse_backend_native::execution::LINKED.published()
    }
    /// Existing typed native services, including cone preparation, initialization,
    /// graph analysis, MILP tears and declared recycle maps, use this same owner.
    pub fn native(&self) -> &Arc<crate::math::MathService> {
        self.shared.math()
    }
    /// The complete schema authority for generated declaration/result rows.
    pub fn registry(&self) -> &Arc<pse_schema::Registry> {
        &self.registry
    }
    /// The engine factory sessions, artifacts and publications of this runtime use.
    pub fn sessions(&self) -> &Arc<EngineFactory> {
        &self.sessions
    }
}
