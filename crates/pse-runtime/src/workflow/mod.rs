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
pub(crate) mod result_export;
mod result_projection;
pub use connected_results::CanonicalResultReader;
pub use result_export::{ResultCursor, ResultOrder, StrategyEventExport};
mod analyses;
pub mod artifacts;
pub use analyses::{AnalysisControls, AnalysisDirection, AnalysisHandle};
mod retention;
pub use retention::{
    HistoryObligation, HistoryOwner, HistoryPurpose, HistoryReclamationPage, ResultReclamationPage,
};
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
    AdmittedSeed, Durability, DurableRecord, LeasePolicy, Operations, Recovery, RunDurability,
    SeedReadError, TerminationCause, TerminationDetail,
};
mod progress_documents;
pub use progress_documents::{
    IncumbentDocument, ProgressCoverage, ProgressEventDocument, ProgressMetricDocument,
    ProgressWriteStage, ProgressWriterFailure, ProgressWriterOutcome,
};
mod progress;
pub use progress::ProgressStream;
mod bindings;
mod native_deployment;
mod physical_cache;
mod worker;
pub use bindings::{
    AdmittedBinding, AdmittedBindingEntry, BindingAssignment, BindingQuantity, BindingTarget,
    PointOverlay,
};
pub use native_deployment::{
    ManagedPrimaryReceiver, NativeAllocation, NativeExecutionAllocation, NativeExecutionSelection,
};
pub use worker::{PhysicalSource, Processed, WorkerSettings};
mod study_demand;
pub use study_demand::{
    CompactStudyResult, StudyDetailDemand, StudyMemberDemand, StudyOutputDemand,
};
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
    FitPreparationDocument, FitProfile, FitProfileDocument, FitReport, FitRule, FitUncertainty,
    FitWithheld, Interval, IntervalBound, PreparedFit, ProfileChain, ProfileControls,
    ProfileFailure, ProfilePoint, ProfileWorkerFailure,
};
#[cfg(feature = "solver-diffsol")]
pub use shooting::{
    PathBound, ShootingControl, ShootingEvaluationPolicy, ShootingMethod, ShootingObjective,
    ShootingProblem, ShootingProfile, ShootingReport,
};
mod physical;
pub use physical::PhysicalContext;
pub(crate) mod modeling;
pub(crate) use modeling::PackageAdmission;
pub use modeling::documents::{
    ConformanceControls, DeclarationEdit, DeclarationInventory, DiagnosticSamplesControls,
    InspectionConnection, InspectionExecution, InspectionInstance, InspectionLineage,
    InspectionMember, InspectionPort, JacobianDiagnosticControls, KnowledgeControls,
    LinearDiagnosticControls, ModelingInspection, PureConformanceControls, SourceDelta,
    SourceDocumentDelta, SourceFitDelta,
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
mod worker_tests;
use crate::{SharedRuntime, math::MathRuntimeError};
use pse_engine::{
    EngineError,
    session::{BoundModel, EngineFactory},
};
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
    code(this) {match this {Self::Input(_)|Self::RecordingUnavailable|Self::CallerClosed=>Some(pse_diagnostics::DiagnosticCode::WorkflowInput),Self::Internal(_)=>Some(pse_diagnostics::DiagnosticCode::WorkflowInternal),Self::Typed(_)=>None,_=>None}},
    forward(this) {match this {Self::SeedRead(e)=>Some(e),Self::ResultBlock(e)=>Some(e),Self::Canonical(e)=>Some(e),Self::Boundary(e)=>Some(e.as_ref()),Self::Typed(e)=>Some(e.as_ref()),Self::ConditionalAdmission{diagnostic,..}|Self::ModelingAdmission{diagnostic,..}=>Some(diagnostic.as_ref()),Self::Math(e)=>Some(e),Self::Engine(e)=>Some(e),Self::Authoring(e)=>Some(e),Self::Shared(e)=>Some(e.as_ref()),Self::StudyCreationUnsettled{source,..}|Self::ArtifactTransferUnsettled{source,..}=>Some(source.as_ref()),_=>None}},
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
    pub(crate) model: BoundModel,
    pub(crate) canonical: Option<CanonicalDeployment>,
    caller: Arc<RuntimeCaller>,
    /// Application runs retain scientific outcomes; ephemerality is an explicit choice.
    pub(crate) durability: Durability,
    physical_cache: Arc<physical_cache::PhysicalCache>,
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
impl DurabilitySelection {
    fn validate(&self) -> Result<(), WorkflowError> {
        if let Self::Durable { policy, .. } = self
            && (policy.heartbeat.is_zero() || policy.lease.is_zero())
        {
            return Err(contract(
                "durable heartbeat and lease intervals must be positive",
            ));
        }
        Ok(())
    }
}
impl Runtime {
    /// Release this logical caller and all of its aliases. Already admitted work
    /// retains its continuation authority; physical drivers belong to the deployment.
    pub fn close(&self) {
        self.caller.close();
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
        let physical_cache = Arc::new(physical_cache::PhysicalCache::default());
        let component: Arc<dyn pse_engine::cache_service::CacheComponent> = physical_cache.clone();
        shared.caches().register_component(&component);
        Ok(Self {
            shared,
            model,
            caller: Arc::new(RuntimeCaller::new(Some(canonical.store().clone()))),
            canonical: Some(canonical),
            durability,
            physical_cache,
        })
    }
    /// Construct an explicitly ephemeral caller over the existing deployment. No
    /// canonical client, producer observation, executor or independent budget is made.
    pub fn from_shared_local(
        shared: Arc<SharedRuntime>,
        model: BoundModel,
    ) -> Result<Self, WorkflowError> {
        Self::check_deployment(&shared, model.factory())?;
        let physical_cache = Arc::new(physical_cache::PhysicalCache::default());
        let component: Arc<dyn pse_engine::cache_service::CacheComponent> = physical_cache.clone();
        shared.caches().register_component(&component);
        Ok(Self {
            shared,
            model,
            canonical: None,
            caller: Arc::new(RuntimeCaller::new(None)),
            durability: Durability::Ephemeral,
            physical_cache,
        })
    }
    /// Attach an explicitly opened import/recording boundary to this same caller.
    /// This creates no executor, model binding, budget or independent close authority.
    pub fn with_canonical(&self, canonical: CanonicalDeployment) -> Result<Self, WorkflowError> {
        let canonical=canonical.independent_caller()?;
        let mut state=self.caller.state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        if state.closed {canonical.store().close();return Err(WorkflowError::CallerClosed);}
        state.stores.push(canonical.store().clone());
        Ok(Self {canonical:Some(canonical),..self.clone()})
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
        self.canonical.as_ref().ok_or(WorkflowError::RecordingUnavailable)
    }
    /// The same deployment under an explicit durability class (ADR-0114 Outcome 16).
    /// Packages and preparations made from the returned runtime run under it.
    /// # Errors
    /// Durable lease or heartbeat intervals are zero.
    pub fn with_durability(
        mut self,
        selection: DurabilitySelection,
    ) -> Result<Self, WorkflowError> {
        selection.validate()?;
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
}
#[derive(Debug, Default)]
struct RuntimeCallerState {
    closed: bool,
    active: usize,
    stores: Vec<pse_operations::canonical::CanonicalStore>,
}
impl RuntimeCaller {
    fn new(store: Option<pse_operations::canonical::CanonicalStore>) -> Self {
        Self { state: std::sync::Mutex::new(RuntimeCallerState { stores: store.into_iter().collect(), ..Default::default() }) }
    }
    fn admit(self: &Arc<Self>) -> Result<RuntimeActivity, WorkflowError> {
        if let Ok(activity) = RUNTIME_CONTINUATION.try_with(Clone::clone)
            && Arc::ptr_eq(&activity.0.caller, self) {
            return Ok(activity);
        }
        let mut state = self.state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        if state.closed { return Err(WorkflowError::CallerClosed); }
        state.active = state.active.checked_add(1).ok_or_else(|| contract("runtime activity extent"))?;
        Ok(RuntimeActivity(Arc::new(RuntimeActivityOwner { caller: self.clone() })))
    }
    fn close(&self) {
        let mut state = self.state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        state.closed = true;
        if state.active == 0 { for store in &state.stores { store.close(); } }
    }
}
/// An admitted local continuation. Required children inherit it explicitly;
/// idle Runtime aliases cannot use it to bypass close.
#[derive(Clone, Debug)]
pub(crate) struct RuntimeActivity(Arc<RuntimeActivityOwner>);
#[derive(Debug)]
struct RuntimeActivityOwner { caller: Arc<RuntimeCaller> }
impl Drop for RuntimeActivityOwner {
    fn drop(&mut self) {
        let mut state = self.caller.state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        state.active -= 1;
        if state.closed && state.active == 0 { for store in &state.stores { store.close(); } }
    }
}
tokio::task_local! { static RUNTIME_CONTINUATION: RuntimeActivity; }
impl RuntimeActivity {
    pub(crate) fn scope<F: Future>(&self, future: F) -> impl Future<Output = F::Output> + use<F> {
        RUNTIME_CONTINUATION.scope(self.clone(), future)
    }
}
pub(crate) fn inherit_runtime_activity<F: Future>(future: F) -> impl Future<Output = F::Output> {
    let activity = RUNTIME_CONTINUATION.try_with(Clone::clone).ok();
    async move { match activity { Some(activity) => activity.scope(future).await, None => future.await } }
}
