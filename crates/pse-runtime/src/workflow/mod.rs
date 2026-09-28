// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! One public native workflow. Model declarations are generated; mathematics stays in Rust libraries.
mod completion;
mod diagnostics;
mod durable;
pub use completion::Completion;
pub use durable::{
    Durability, DurableRecord, LeasePolicy, Operations, Recovery, RunDurability, TerminationCause,
    TerminationDetail,
};
pub use pse_operations::attempts::AttemptFilter;
/// The operational store a process connects to: `PSE_DATABASE_URL`, else the development
/// default (ADR-0114 Outcome 21).
pub use pse_operations::database_url_from_env;
mod worker;
pub use worker::{
    JOB_PAYLOAD_VERSION, JobPayload, JobStart, JobTask, ModelingJob, PointOverlay, Processed,
    SourceManifest, StudyFinalization, StudyPointBinding, WorkerSettings,
};
mod study;
pub use study::{
    MAXIMUM_STUDY_POINTS, PackageSources, PointStatus, StudyDefinition, StudyHandle, StudyPlan,
    StudyPoint, StudyPointDefinition, StudyStatus,
};
pub use pse_operations::jobs::RetryPolicy;
pub use pse_operations::studies::{StudyCancel, StudyFilter, StudyId, StudyPointState, StudyState};
pub(crate) mod numerics;
mod staged;
mod strategies;
mod time;
pub use strategies::{AnalysisPort, ConicRequest, PreparedConic};
#[cfg(feature = "solver-kinsol")]
pub use strategies::{
    CausalUnitRequest, PreparedInitializationStrategy, PreparedRecycle, RecycleRequest,
};
#[cfg_attr(
    not(feature = "solver-diffsol"),
    allow(
        dead_code,
        reason = "Dynamic source and preparation contracts remain available without the optional execution adapter"
    )
)]
mod dynamics;
mod fitting;
pub use dynamics::SimulationProfile;
pub use fitting::{
    FitData, FitDeclaration, FitDiagnostic, FitProfile, FitReport, FitRule, PreparedFit,
};
mod physical;
pub use physical::PhysicalContext;
mod modeling;
pub use modeling::ModelingNativeAnalysis;
pub use modeling::{
    DiagnosticSampleStop, ElasticObservation, ModelingAnalysis, ModelingCheck,
    ModelingConformanceCheck, ModelingConformancePolicy, ModelingConformanceReport,
    ModelingDiagnosticPolicy, ModelingDiagnosticPreparation, ModelingDiagnosticSamples,
    ModelingDiagnostics, ModelingDynamicEvent, ModelingDynamicMode, ModelingElasticAttempt,
    ModelingFixturePolicy, ModelingInfeasibilityCertificate, ModelingInitialization,
    ModelingInitializationAttempt, ModelingInitializationReport, ModelingInitializationStep,
    ModelingNonlinearExplanation, ModelingNonlinearPolicy, ModelingObservations, ModelingPackage,
    ModelingReport, ModelingResult, ModelingSimulation, ModelingSolvePreparation,
    ModelingStudyPoint, ModelingStudyReport, ModelingTrajectory, StartSource,
    conform_pure_documents,
};
#[cfg(feature = "solver-highs")]
pub use modeling::{ModelingJacobianOptimization, ModelingLinearDiagnostics};
#[cfg(test)]
mod durable_tests;
mod modeling_results;
mod publication;
mod reading;
mod results;
mod retention;
mod run;
mod simulation_results;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod study_tests;
#[cfg(test)]
mod worker_tests;
use crate::{SharedRuntime, math::MathRuntimeError};
use pse_engine::{EngineError, session::EngineFactory};
pub use publication::{
    PublicationAttempt, PublicationSettlement, PublicationTicket, Published, Workspace,
    prepare_artifact_publication,
};
pub use reading::{
    ExportReceipt, LeasedPublication, READER_LEASE, ReaderLeaseGuard, open_export,
};
pub use retention::{CollectReport, ReclaimReport, RetireReport};
pub use run::{RunHandle, RunReport, RunRequest, RunResult, StoredStart};
use std::sync::Arc;

/// Errors retain the native/physical/authoring cause; no string matching or fallback.
#[derive(Debug, thiserror::Error)]
pub enum WorkflowError {
    /// Source-attributed selected-model or execution-boundary failure.
    #[error(transparent)]
    Boundary(#[from] Box<pse_model::diagnostic::BoundaryDiagnostic>),
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
    Contract(String),
    /// The operational store refused or failed a durable operation (ADR-0114).
    #[error(transparent)]
    Operations(#[from] pse_operations::OperationsError),
    /// A run of the ephemeral durability class cannot be published: publication needs a
    /// registered, finished attempt (ADR-0114 Outcome 16). This is a policy, never a
    /// fallback.
    #[error(
        "run {run_id} is ephemeral: publication requires a durable run registered in the operational store"
    )]
    EphemeralPublication {
        /// The run that was asked to publish.
        run_id: pse_ids::SemanticId,
    },
    /// The catalog commit's outcome is unknown: its acknowledgement was lost or the
    /// catalog became unreachable after the members were written. Never retried
    /// implicitly: settle the publication ticket (ADR-0114 Outcome 6).
    #[error("publication {publication} is unresolved: {reason}; settle its ticket")]
    PublicationUnresolved {
        /// The publication the attempt intended.
        publication: pse_operations::catalog::PublicationId,
        /// Why.
        reason: String,
    },
    /// A workspace root holds a Delta publication control table, an unsupported
    /// historical format; its publications are regenerated by rerunning them.
    #[error(
        "{root} holds a Delta publication control table (an unsupported historical format); migration required: regenerate its publications by rerunning them into a new root"
    )]
    LegacyWorkspace {
        /// The refused root.
        root: url::Url,
    },
    /// An exported publication's lease expired; its members may have been removed.
    #[error("the export of publication {publication} expired at {expires_at} (microseconds)")]
    ExportLeaseExpired {
        /// The exported publication.
        publication: pse_ids::SemanticId,
        /// Its expiry, microseconds since the Unix epoch.
        expires_at: i64,
    },
    /// A stored job payload this build cannot execute (ADR-0114 Outcome 14).
    #[error(
        "job payload version {version} is not supported by this worker (supported: {supported})"
    )]
    UnknownPayloadVersion {
        /// The stored payload version.
        version: i32,
        /// The version this build executes.
        supported: i32,
    },
}
impl From<pse_model::diagnostic::BoundaryDiagnostic> for WorkflowError {
    fn from(error: pse_model::diagnostic::BoundaryDiagnostic) -> Self {
        Self::Boundary(Box::new(error))
    }
}
pse_diagnostics::impl_diagnostic! {
    WorkflowError,
    code(this) {match this {Self::Contract(_)=>Some(pse_diagnostics::DiagnosticCode::CompileMath),Self::EphemeralPublication{..}|Self::UnknownPayloadVersion{..}=>Some(pse_diagnostics::DiagnosticCode::ConfigInvalid),Self::PublicationUnresolved{..}|Self::ExportLeaseExpired{..}=>Some(pse_diagnostics::DiagnosticCode::RuntimeInfrastructure),Self::LegacyWorkspace{..}=>Some(pse_diagnostics::DiagnosticCode::SchemaInvalidDeclaration),_=>None}},
    forward(this) {match this {Self::Boundary(e)=>Some(e.as_ref()),Self::Math(e)=>Some(e),Self::Engine(e)=>Some(e),Self::Authoring(e)=>Some(e),Self::Shared(e)=>Some(e.as_ref()),Self::Operations(e)=>Some(e),_=>None}},
    help(_this){None},related(_this){None},source(_this){None}
}
fn contract(message: impl Into<String>) -> WorkflowError {
    WorkflowError::Contract(message.into())
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
    /// How this runtime's runs are kept; ephemeral unless chosen explicitly.
    pub(crate) durability: Durability,
}
impl Runtime {
    /// Clear retained executable programs. Existing workers keep their owners and remain valid.
    pub fn clear_program_cache(&self) {
        self.shared.math().clear_program_cache();
    }
    /// Attach to the already configured shared deployment; creates no second executor or budget.
    /// The runtime is [`Durability::Ephemeral`] until [`Runtime::with_durability`].
    pub fn from_shared(
        shared: Arc<SharedRuntime>,
        registry: Arc<pse_schema::Registry>,
        sessions: Arc<EngineFactory>,
    ) -> Self {
        Self {
            shared,
            registry,
            sessions,
            durability: Durability::Ephemeral,
        }
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
