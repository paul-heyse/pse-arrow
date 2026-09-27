// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! One public native workflow. Model declarations are generated; mathematics stays in Rust libraries.
mod completion;
mod diagnostics;
pub use completion::Completion;
pub(crate) mod numerics;
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
    conform_pure_documents, ModelingConformanceCheck, ModelingConformancePolicy, ModelingFixturePolicy, ModelingConformanceReport,
    ModelingDiagnosticPolicy, ModelingDiagnosticPreparation, ModelingDiagnosticSamples,
    ModelingDiagnostics, ModelingDynamicEvent, ModelingDynamicMode, ModelingElasticAttempt,
    ModelingInitialization, ModelingInitializationAttempt, ModelingInitializationReport,
    ModelingInitializationStep, ModelingNonlinearExplanation, ModelingNonlinearPolicy,
    ModelingObservations, ModelingPackage, ModelingReport, ModelingResult, ModelingSimulation,
    ModelingSolvePreparation, ModelingStudyPoint, ModelingStudyReport, ModelingTrajectory,
    StartSource,
};
#[cfg(feature = "solver-highs")]
pub use modeling::{ModelingJacobianOptimization, ModelingLinearDiagnostics};
mod publication;
mod results;
mod run;
mod simulation_results;
mod modeling_results;
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
}
impl From<pse_model::diagnostic::BoundaryDiagnostic> for WorkflowError {
    fn from(error: pse_model::diagnostic::BoundaryDiagnostic) -> Self {
        Self::Boundary(Box::new(error))
    }
}
pse_diagnostics::impl_diagnostic! {
    WorkflowError,
    code(this) {match this {Self::Contract(_)=>Some(pse_diagnostics::DiagnosticCode::CompileMath),_=>None}},
    forward(this) {match this {Self::Boundary(e)=>Some(e.as_ref()),Self::Math(e)=>Some(e),Self::Engine(e)=>Some(e),Self::Authoring(e)=>Some(e),Self::Shared(e)=>Some(e.as_ref()),_=>None}},
    help(_this){None},related(_this){None},source(_this){None}
}
fn contract(message: impl Into<String>) -> WorkflowError {
    WorkflowError::Contract(message.into())
}
fn math(error: impl Into<pse_math::MathError>) -> WorkflowError {
    WorkflowError::Math(MathRuntimeError::Math(error.into()))
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
}
impl Runtime {
    /// Clear retained executable programs. Existing workers keep their owners and remain valid.
    pub fn clear_program_cache(&self) {
        self.shared.math().clear_program_cache();
    }
    /// Attach to the already configured shared deployment; creates no second executor or budget.
    pub fn from_shared(
        shared: Arc<SharedRuntime>,
        registry: Arc<pse_schema::Registry>,
        sessions: Arc<EngineFactory>,
    ) -> Self {
        Self {
            shared,
            registry,
            sessions,
        }
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
