// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! One public native workflow. Model declarations are generated; mathematics stays in Rust libraries.
mod completion;
mod diagnostics;
mod durable;
pub use completion::Completion;
pub use durable::{Durability, DurableRecord, LeasePolicy, Operations, Recovery, RunDurability};
pub use pse_operations::attempts::{AttemptFilter, AttemptRecord};
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
    ModelingFixturePolicy, ModelingInitialization, ModelingInitializationAttempt,
    ModelingInitializationReport, ModelingInitializationStep, ModelingNonlinearExplanation,
    ModelingNonlinearPolicy, ModelingObservations, ModelingPackage, ModelingReport, ModelingResult,
    ModelingSimulation, ModelingSolvePreparation, ModelingStudyPoint, ModelingStudyReport,
    ModelingTrajectory, StartSource, conform_pure_documents,
};
#[cfg(feature = "solver-highs")]
pub use modeling::{ModelingJacobianOptimization, ModelingLinearDiagnostics};
#[cfg(test)]
mod durable_tests;
mod modeling_results;
mod publication;
mod results;
mod run;
mod simulation_results;
#[cfg(test)]
mod tests;
use crate::{SharedRuntime, math::MathRuntimeError};
use pse_engine::{EngineError, session::EngineFactory};
pub use publication::{
    PublicationAttempt, PublicationRequest, PublicationSettlement, PublicationTicket,
};
pub use run::{RunHandle, RunReport, RunRequest, RunResult};
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
    /// The operational store refused or failed a durable operation (ADR-0112).
    #[error(transparent)]
    Operations(#[from] pse_operations::OperationsError),
    /// A run of the ephemeral durability class cannot be published: publication needs a
    /// registered, finished attempt (ADR-0112 Outcome 16). This is a policy, never a
    /// fallback.
    #[error(
        "run {run_id} is ephemeral: publication requires a durable run registered in the operational store"
    )]
    EphemeralPublication {
        /// The run that was asked to publish.
        run_id: pse_ids::SemanticId,
    },
    /// A stored job payload this build cannot execute (ADR-0112 Outcome 14).
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
    code(this) {match this {Self::Contract(_)=>Some(pse_diagnostics::DiagnosticCode::CompileMath),Self::EphemeralPublication{..}|Self::UnknownPayloadVersion{..}=>Some(pse_diagnostics::DiagnosticCode::ConfigInvalid),_=>None}},
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
    /// The same deployment under an explicit durability class (ADR-0112 Outcome 16).
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
}
