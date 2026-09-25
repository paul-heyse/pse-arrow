// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! One public native workflow. Model declarations are generated; mathematics stays in Rust libraries.
use pse_backend_native::solve::BackendCapabilities;
mod balances;
mod composition;
mod numerics;
mod reactions;
mod time;
pub use composition::{AdmissionEntry, CompositionDeclarations, port_id, symbol_id};
mod sources;
mod strategies;
pub use balances::{BalanceCheck, BalanceDeclaration};
pub use strategies::{AnalysisPort, ConicRequest, PreparedConic};
#[cfg(feature = "solver-kinsol")]
pub use strategies::{
    CausalUnitRequest, PreparedInitializationStrategy, PreparedRecycle, RecycleRequest,
};
mod vessel;
pub use vessel::{VesselPorts, VesselQuantities, VesselRecipe};
#[cfg_attr(
    not(feature = "solver-diffsol"),
    allow(
        dead_code,
        reason = "Dynamic source and preparation contracts remain available without the optional execution adapter"
    )
)]
mod dynamics;
mod fitting;
pub use dynamics::{PreparedSimulation, SimulationProfile};
pub use fitting::{FitProfile, FitReport, PreparedFit};
pub use sources::{
    DynamicDeclaration, FitDeclaration, NativeProviderDeclaration, Sources as SourceDeclarations,
};
mod model;
mod publication;
mod results;
mod run;
mod simulation_results;
#[cfg(test)]
mod tests;
use crate::{SharedRuntime, math::MathRuntimeError};
pub use model::{CaseBuilder, ModelBuilder, ModelRevision, PhysicalContext, ProviderBinding};
pub use model::{CaseDeclaration, DefinitionDeclaration, ModelDeclaration};
use pse_engine::{EngineError, session::EngineFactory};
pub use publication::PublicationAttempt;
pub use run::{PreparedCase, RunHandle, RunReport, RunRequest, RunResult};
use std::sync::Arc;

/// Errors retain the native/physical/authoring cause; no string matching or fallback.
#[derive(Debug, thiserror::Error)]
pub enum WorkflowError {
    /// Source-attributed selected-model or execution-boundary failure.
    #[error(transparent)]
    Boundary(#[from] pse_model::diagnostic::BoundaryDiagnostic),
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
pse_diagnostics::impl_diagnostic! {
    WorkflowError,
    code(this) {match this {Self::Contract(_)=>Some(pse_diagnostics::DiagnosticCode::CompileMath),_=>None}},
    forward(this) {match this {Self::Boundary(e)=>Some(e),Self::Math(e)=>Some(e),Self::Engine(e)=>Some(e),Self::Authoring(e)=>Some(e),Self::Shared(e)=>Some(e.as_ref()),_=>None}},
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
    /// Start a new typed draft with an actual admitted physical context.
    pub fn model(
        &self,
        id: pse_ids::SemanticId,
        name: String,
        physical: PhysicalContext,
    ) -> ModelBuilder {
        ModelBuilder::new(self.clone(), id, name, physical)
    }
    /// Whether the native BDF semi-explicit adapter is linked in this deployment.
    pub fn simulation_available(&self) -> bool {
        cfg!(feature = "solver-diffsol")
    }
    /// Actual linked implementations; model eligibility is separately reported by preparation.
    pub fn capabilities(
        &self,
    ) -> Vec<(
        pse_backend_native::solve::Backend,
        pse_backend_native::solve::Capabilities,
    )> {
        use pse_backend_native::solve::Backend::*;
        [Ipopt, Pounce, Kinsol, Highs, Clarabel, Diffsol, Idas]
            .into_iter()
            .filter(|b| b.available())
            .map(|b| (b, b.capabilities()))
            .collect()
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

/// Source-to-scalar mapping produced by selected template admission.
pub use composition::lower::Binding as ScalarBinding;
