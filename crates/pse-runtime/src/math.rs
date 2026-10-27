// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! One Salsa compiler and one shared runtime effect boundary for process mathematics.
mod artifacts;
pub mod flows;
mod functions;
#[cfg(feature = "solver-kinsol")]
pub mod initialization;
mod jobs;
pub mod modeling;
#[cfg(all(unix, feature = "solver-kinsol"))]
pub mod native_child;
pub(crate) mod numerics;
pub mod portable;
pub mod prediction;
mod preparation;
mod products;
pub(crate) mod retention;
pub mod settings;
pub mod solves;
mod staged;
pub(crate) mod strategy;
pub mod surrogate;
pub use artifacts::Artifact;
use artifacts::{Key, Value};
use datafusion::execution::{
    cache::default_cache::DefaultCache,
    memory_pool::{MemoryConsumer, MemoryPool},
};
pub(crate) use jobs::{ConstructionEstimate, PopulationScope, Submission};
pub use jobs::{WorkerBudget, WorkerCharge};
use pse_columnar::flight::Flights;
use pse_compiler::workspace::{
    CompileError, CompilerContext, CompilerWorkspace, PreparedCase, WorkspaceLimits,
};
use pse_diagnostics::Cancellation;
use pse_engine::cache_service::CacheComponent;
use pse_kernels::ProviderKey;
use pse_math::assembly::{CaseAssembly, CaseWorker};
pub(crate) use staged::NativeSession;
#[cfg(all(test, feature = "solver-ipopt", not(feature = "canonical-tests")))]
pub(crate) use staged::observed_native_entry;
#[cfg(all(test, feature = "solver-ipopt"))]
pub(crate) use staged::observed_native_task_entry;
pub(crate) use staged::{SessionDisposition, StepRetention};
#[cfg(feature = "canonical-tests")]
#[doc(hidden)]
pub use staged::{install_qualified_native_entry, observed_native_entry};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex, atomic::AtomicUsize},
};
/// Await the actual enclosing stop without periodically polling its read flag.
#[cfg(feature = "solver-petsc")]
pub(crate) async fn scope_stopped(
    scope: &pse_kernels::ExecutionScope,
) -> pse_kernels::ProviderError {
    if let Err(error) = scope.check() {
        return error;
    }
    tokio::select! {
        biased;
        () = scope.cancellation_control().cancelled() => pse_kernels::ProviderError::Cancelled,
        () = async { match scope.deadline() { Some(at) => tokio::time::sleep_until(at.into()).await, None => std::future::pending().await } } => pse_kernels::ProviderError::Deadline,
    }
}
/// Finite math allowances draw from the deployment pool, never a second memory budget.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MathPolicy {
    /// Completed program retention limit (not an up-front reservation).
    pub artifact_bytes: usize,
    /// Conservative foreign-library allowance of one native job or open native session,
    /// charged while its native work runs. A compiled program, prepared product or prepared
    /// solve that outlives its job is charged its own extent instead.
    pub foreign_bytes: usize,
    /// Retained KINSOL inner-solve sessions per native job or session thread, in bytes;
    /// reserved in its lease, and zero retains none (Plan 22 I14).
    pub inner_session_bytes: usize,
    /// Maximum aggregate worker numeric capacity.
    pub worker_bytes: usize,
    /// Reserved workspace generation capacity, including metadata allowance.
    pub workspace_bytes: usize,
    /// Explicit native stack, qualified for structural recursion.
    pub stack_bytes: usize,
    /// Maximum admitted population, including waiters and resident sessions.
    /// Active native stages have the same finite upper bound, with separate ownership.
    pub jobs: usize,
    /// Finite admission-only wait when the caller supplies no task deadline.
    /// It does not limit execution after dispatch or renew an enclosing task clock.
    pub admission_wait: std::time::Duration,
    /// Finite execution allowance of each service-owned shared preparation producer.
    pub preparation_limit: std::time::Duration,
    /// Maximum distinct live artifact keys and waiters per key.
    pub flights: usize,
}
impl Default for MathPolicy {
    fn default() -> Self {
        Self {
            artifact_bytes: 8 << 30,
            foreign_bytes: 64 << 20,
            inner_session_bytes: 16 << 20,
            worker_bytes: 8 << 30,
            workspace_bytes: 4 << 30,
            stack_bytes: pse_structural::incidence::MATCHING_STACK,
            jobs: 4,
            admission_wait: std::time::Duration::from_secs(30),
            flights: 128,
            preparation_limit: std::time::Duration::from_secs(600),
        }
    }
}
impl MathPolicy {
    /// The foreign-library allowance of one solve: the allowance its controls declare, else
    /// this deployment's. It is reserved only while the solve runs, the deployment's by its
    /// native session and a declared one by its step, and a library that enforces its own
    /// memory limit receives it (`Execution::memory`).
    pub fn foreign_allowance(&self, controls: &pse_backend_native::solve::Controls) -> usize {
        controls.foreign_bytes.unwrap_or(self.foreign_bytes)
    }
    pub(crate) fn validate(&self) -> Result<(), crate::RuntimeError> {
        if [
            self.artifact_bytes,
            self.foreign_bytes,
            self.worker_bytes,
            self.workspace_bytes,
            self.jobs,
            self.flights,
        ]
        .contains(&0)
            || self.preparation_limit.is_zero()
            || std::time::Instant::now()
                .checked_add(self.preparation_limit)
                .is_none()
            || self.admission_wait.is_zero()
            || std::time::Instant::now()
                .checked_add(self.admission_wait)
                .is_none()
            || self.stack_bytes < pse_structural::incidence::MATCHING_STACK
        {
            return Err(crate::RuntimeError::ConfigInvalid {
                key: "math.policy".into(),
                reason: "math allowances must be finite and positive with a qualified stack".into(),
            });
        }
        Ok(())
    }
}
/// Runtime math errors retain typed compiler, math and shared-load causes.
#[derive(Debug, thiserror::Error)]
pub enum MathRuntimeError {
    /// Native solver admission or original mathematical evaluation failure.
    #[error(transparent)]
    Solve(#[from] pse_backend_native::ProblemError),
    /// Pure compiler failure.
    #[error(transparent)]
    Compile(#[from] CompileError),
    /// Native library failure.
    #[error(transparent)]
    Math(#[from] pse_math::MathError),
    /// Original pool refusal.
    #[error(transparent)]
    Pool(#[from] datafusion::common::DataFusionError),
    /// Shared typed failure.
    #[error(transparent)]
    Shared(Arc<MathRuntimeError>),
    /// Failed numerical task retains its actual driver events and original typed cause.
    #[error("{cause}")]
    Strategy {
        /// Original failure, used by classification and diagnostic projection.
        #[source]
        cause: Arc<MathRuntimeError>,
        /// Accounted immutable events, including failures before a native report exists.
        trace: Arc<strategy::Trace>,
    },
    /// Original failure survives when a foreign payload cannot supply retained extent.
    #[error("{cause}")]
    StrategyTraceUnavailable {
        /// Actual original failure, which remains the classification authority.
        #[source]
        cause: Arc<MathRuntimeError>,
        /// Typed reason the actual event allocation could not be retained.
        refusal: Arc<pse_backend_native::ProblemError>,
    },
    /// Allocation refusal retains the allocator cause and actual producer operation.
    #[error("math allocation refused during {operation}: {source}")]
    Allocation {
        /// Actual allocation owner.
        operation: &'static str,
        /// Original capacity or allocation refusal.
        #[source]
        source: std::collections::TryReserveError,
    },
    /// A joined compiler task failed independently of authored model input.
    #[error("math worker failed during {operation}")]
    WorkerJoin {
        /// Actual worker operation.
        operation: &'static str,
        /// Original join failure, including its panic or cancellation identity.
        #[source]
        source: tokio::task::JoinError,
    },
    /// Finite admission bound.
    #[error("math resource limit: {0}")]
    Limit(&'static str),
    /// The service-owned producer exhausted its finite allowance, independently of waiters.
    #[error("shared preparation producer deadline expired")]
    PreparationExpired,
    /// Cancellation does not imply native exit.
    #[error("math work cancelled")]
    Cancelled,
    /// A previous shared load is still joining after its last waiter departed.
    #[error("math program load is retiring; retry after completion")]
    Retiring,
    /// A caught worker panic, distinct from resource or infrastructure refusal.
    #[error("math worker panic: {0}")]
    Panic(String),
    /// An invariant of the mathematical runtime failed independently of supplied input.
    #[error("math runtime invariant: {0}")]
    Internal(String),
    /// Supervisor/thread failure.
    #[error("math infrastructure: {0}")]
    Infrastructure(String),
}
impl MathRuntimeError {
    pub(crate) fn preparation_producer_failure(self) -> Self {
        preparation::producer_failure(self)
    }

    /// Producer-owned retained failure extent. Separately leased strategy traces are
    /// excluded; opaque foreign errors without an allocation contract are refused.
    pub(crate) fn retained_bytes(&self) -> Result<usize, pse_backend_native::ProblemError> {
        use pse_backend_native::{LimitKind, ProblemError};
        let add = |left: usize, right: usize| {
            left.checked_add(right).ok_or_else(|| ProblemError::Limit {
                kind: LimitKind::Memory,
                detail: "retained runtime failure extent".into(),
            })
        };
        fn pool(error: &datafusion::common::DataFusionError) -> Result<usize, ProblemError> {
            use datafusion::common::DataFusionError as E;
            let payload = match error {
                E::ResourcesExhausted(text)
                | E::NotImplemented(text)
                | E::Internal(text)
                | E::Plan(text)
                | E::Configuration(text)
                | E::Execution(text)
                | E::Substrait(text)
                | E::Ffi(text) => Some(text.capacity()),
                E::Context(text, cause) => text.capacity().checked_add(pool(cause)?),
                E::Shared(cause) => pool(cause)?.checked_add(2 * size_of::<usize>()),
                E::Collection(causes) => {
                    let mut bytes = causes.capacity().checked_mul(size_of::<E>());
                    for cause in causes {
                        let extent = pool(cause)?;
                        bytes = bytes.and_then(|bytes| bytes.checked_add(extent));
                    }
                    bytes
                }
                _ => {
                    return Err(ProblemError::Unsupported(
                        "foreign pool failure has no retained allocation contract".into(),
                    ));
                }
            };
            let payload = payload.ok_or_else(|| ProblemError::Limit {
                kind: LimitKind::Memory,
                detail: "retained pool failure extent".into(),
            })?;
            size_of::<E>()
                .checked_add(payload)
                .ok_or_else(|| ProblemError::Limit {
                    kind: LimitKind::Memory,
                    detail: "retained pool failure extent".into(),
                })
        }
        let payload = match self {
            Self::Solve(cause) => cause.retained_bytes(),
            Self::Compile(cause) => cause.retained_bytes(),
            Self::Math(cause) => cause.retained_bytes(),
            Self::Pool(cause) => pool(cause)?,
            Self::Shared(cause) | Self::Strategy { cause, .. } => {
                add(2 * size_of::<usize>(), cause.retained_bytes()?)?
            }
            Self::StrategyTraceUnavailable { cause, refusal } => add(
                add(4 * size_of::<usize>(), cause.retained_bytes()?)?,
                refusal.retained_bytes(),
            )?,
            Self::Panic(text) | Self::Internal(text) | Self::Infrastructure(text) => {
                text.capacity()
            }
            Self::Allocation { .. } => 0,
            Self::WorkerJoin { source, .. } if source.is_cancelled() => 0,
            Self::WorkerJoin { .. } => {
                return Err(ProblemError::Unsupported(
                    "worker panic payload has no retained allocation contract".into(),
                ));
            }
            Self::Limit(_) | Self::PreparationExpired | Self::Cancelled | Self::Retiring => 0,
        };
        add(size_of::<Self>(), payload)
    }
    /// Actual execution trace survives typed failure and shared-load wrappers.
    pub fn strategy_trace(&self) -> Option<&Arc<strategy::Trace>> {
        match self {
            Self::Strategy { trace, .. } => Some(trace),
            Self::Shared(cause) => cause.strategy_trace(),
            _ => None,
        }
    }
    /// Actual unavailable-accounting reason, kept separate from numerical failure class.
    pub fn strategy_trace_refusal(&self) -> Option<&Arc<pse_backend_native::ProblemError>> {
        match self {
            Self::StrategyTraceUnavailable { refusal, .. } => Some(refusal),
            Self::Shared(cause) => cause.strategy_trace_refusal(),
            _ => None,
        }
    }
    /// The native error ABI requires an infallible extent. Unknown foreign extents
    /// use an unreservable admission bound, never a purported observed allocation.
    fn into_retained_problem(self) -> pse_backend_native::ProblemError {
        let retained = self.retained_bytes().unwrap_or(usize::MAX);
        pse_backend_native::ProblemError::Math(pse_math::MathError::Typed {
            retained,
            cause: pse_model::diagnostic::DiagnosticCause::new(self),
        })
    }
    /// Carry an exact runtime failure through a mathematical callback without changing its class.
    pub(crate) fn into_math(self) -> pse_math::MathError {
        let retained = self.retained_bytes().unwrap_or(usize::MAX);
        pse_math::MathError::Typed {
            retained,
            cause: pse_model::diagnostic::DiagnosticCause::new(self),
        }
    }
    /// Carry a runtime failure into the native report's single typed validation record,
    /// keeping its class: cancellation, limits and native causes stay typed.
    pub fn into_problem(self) -> pse_backend_native::ProblemError {
        use pse_backend_native::{LimitKind, ProblemError};
        match self {
            Self::Solve(e) => e,
            Self::Math(e) => ProblemError::Math(e),
            Self::Cancelled => ProblemError::Cancelled,
            Self::PreparationExpired => ProblemError::Limit {
                kind: LimitKind::Time,
                detail: "shared preparation producer deadline".into(),
            },
            Self::Limit(detail) => ProblemError::Limit {
                kind: LimitKind::Memory,
                detail: detail.into(),
            },
            Self::Shared(error) => {
                fn producer_expired(error: &MathRuntimeError) -> bool {
                    match error {
                        MathRuntimeError::PreparationExpired => true,
                        MathRuntimeError::Shared(cause) => producer_expired(cause),
                        _ => false,
                    }
                }
                if producer_expired(&error) {
                    return Self::PreparationExpired.into_problem();
                }
                match Arc::try_unwrap(error) {
                    Ok(error) => error.into_problem(),
                    Err(error) => Self::Shared(error).into_retained_problem(),
                }
            }
            Self::Pool(_) | Self::Strategy { .. } | Self::StrategyTraceUnavailable { .. } => {
                self.into_retained_problem()
            }
            Self::Compile(CompileError::Cancelled) => ProblemError::Cancelled,
            Self::Compile(CompileError::Limit(detail)) => ProblemError::Limit {
                kind: LimitKind::Work,
                detail: detail.into(),
            },
            Self::Compile(e) => ProblemError::Math(pse_math::MathError::Typed {
                retained: e.retained_bytes(),
                cause: pse_model::diagnostic::DiagnosticCause::new(e),
            }),
            Self::WorkerJoin { ref source, .. } if source.is_cancelled() => ProblemError::Cancelled,
            Self::Allocation { .. }
            | Self::WorkerJoin { .. }
            | Self::Retiring
            | Self::Infrastructure(_)
            | Self::Internal(_)
            | Self::Panic(_) => self.into_retained_problem(),
        }
    }
}
fn pool_diagnostic_code(
    error: &datafusion::common::DataFusionError,
) -> pse_diagnostics::DiagnosticCode {
    use datafusion::common::DataFusionError as E;
    use pse_diagnostics::DiagnosticCode as C;
    match error {
        E::ResourcesExhausted(_) => C::RuntimeResourceLimit,
        E::Internal(_) => C::InternalInvariant,
        E::Context(_, cause) => pool_diagnostic_code(cause),
        E::Shared(cause) => pool_diagnostic_code(cause),
        _ => C::RuntimeInfrastructure,
    }
}
pse_diagnostics::impl_diagnostic! {
    MathRuntimeError,
    code(this) { match this {Self::WorkerJoin {source,..}=>Some(if source.is_cancelled() {pse_diagnostics::DiagnosticCode::RuntimeCancelled} else {pse_diagnostics::DiagnosticCode::WorkflowPanic}),Self::Allocation {..}=>Some(pse_diagnostics::DiagnosticCode::RuntimeResourceLimit),Self::Cancelled=>Some(pse_diagnostics::DiagnosticCode::RuntimeCancelled),Self::Retiring|Self::PreparationExpired|Self::Limit(_)=>Some(pse_diagnostics::DiagnosticCode::RuntimeResourceLimit),Self::Pool(error)=>Some(pool_diagnostic_code(error)),Self::Infrastructure(_)=>Some(pse_diagnostics::DiagnosticCode::RuntimeInfrastructure),Self::Internal(_)=>Some(pse_diagnostics::DiagnosticCode::InternalInvariant),Self::Panic(_)=>Some(pse_diagnostics::DiagnosticCode::WorkflowPanic),_=>None} },
    forward(this) { match this {Self::Solve(e)=>Some(e),Self::Compile(e)=>Some(e),Self::Math(e)=>Some(e),Self::Shared(e)=>Some(e.as_ref()),Self::Strategy {cause,..}|Self::StrategyTraceUnavailable {cause,..}=>Some(cause.as_ref()),_=>None} },
    help(_this) { None },related(_this) { None },source(_this) { None },
    facts(this) {
        use pse_diagnostics::{DiagnosticRule as R, DiagnosticFacts, DiagnosticObservation as O};
        let mut facts=DiagnosticFacts { rule: match this { Self::WorkerJoin {source,..}=>Some(if source.is_cancelled() {R::MathCancelled} else {R::WorkflowPanic}),Self::Allocation {..}=>Some(R::MathLimit),Self::Cancelled=>Some(R::MathCancelled), Self::Retiring|Self::PreparationExpired|Self::Limit(_)=>Some(R::MathLimit),Self::Pool(error)=>Some(match pool_diagnostic_code(error) { pse_diagnostics::DiagnosticCode::RuntimeResourceLimit=>R::MathLimit,pse_diagnostics::DiagnosticCode::InternalInvariant=>R::WorkflowInternal,_=>R::MathLibrary }), Self::Panic(_)=>Some(R::WorkflowPanic), Self::Infrastructure(_)=>Some(R::MathLibrary), Self::Internal(_)=>Some(R::WorkflowInternal), Self::Solve(_) | Self::Compile(_) | Self::Math(_) | Self::Shared(_) | Self::Strategy {..}|Self::StrategyTraceUnavailable {..}=>None },..Default::default() };
        if !matches!(this,Self::Solve(_)|Self::Compile(_)|Self::Math(_)|Self::Shared(_)|Self::Strategy {..}|Self::StrategyTraceUnavailable {..}) {facts.observe("detail",O::Text(this.to_string()));}
        facts
    }
}
/// Working allowance of a job that runs inside a held workspace lease. The workspace lease
/// already covers the compiler's working set, so the job charges only its native thread
/// and then its retained product at the product's extent.
const WITHIN_WORKSPACE: usize = 0;
/// Deployment-owned mathematics service registered with native cache reporting/invalidation.
pub struct MathService {
    pool: Arc<dyn MemoryPool>,
    released: Arc<tokio::sync::Notify>,
    cpu: Arc<pse_engine::resources::ReleaseNotifyingSemaphore>,
    cores: usize,
    policy: MathPolicy,
    jobs: Arc<pse_engine::resources::ReleaseNotifyingSemaphore>,
    active_jobs: Arc<pse_engine::resources::ReleaseNotifyingSemaphore>,
    entries: Arc<DefaultCache<Key, Value>>,
    pub(crate) modeling_cache: retention::ModelingCache,
    compiler_lanes: Mutex<Vec<modeling::CompilerLane>>,
    native_lanes: Mutex<Vec<staged::ParkedSession>>,
    flights: Flights<Key, Artifact, MathRuntimeError>,
    scalar_flights: Flights<Key, pse_math::library::ScalarProducts, MathRuntimeError>,
    pub(crate) selected_flights: Flights<
        (
            Arc<crate::workflow::modeling::SelectedRequest>,
            Arc<crate::workflow::modeling::LocalSourcePremises>,
        ),
        modeling::ModelingRevision,
        MathRuntimeError,
    >,
    pub(crate) frontier_flights:
        Flights<Arc<modeling::BasisKey>, preparation::OwnedModelingFrontier, MathRuntimeError>,
    pub(crate) basis_flights: Flights<
        (
            Arc<modeling::BasisKey>,
            Arc<crate::workflow::modeling::LocalSourcePremises>,
        ),
        modeling::PreparedBasis,
        MathRuntimeError,
    >,
    retention: Arc<pse_columnar::retention::RetentionFence>,
    pressure: Arc<retention::Pressure>,
    live: Arc<AtomicUsize>,
    hits: AtomicUsize,
    misses: AtomicUsize,
    products: Mutex<BTreeMap<Vec<usize>, std::sync::Weak<products::ProductOwner>>>,
    preparations: Preparations,
    #[cfg(test)]
    preparation_effects: Arc<PreparationEffects>,
}
/// Isolated service-local observations survive worker and flight task boundaries.
/// These controls are deliberately absent from product contracts and release builds.
#[cfg(test)]
#[derive(Default)]
struct PreparationEffects {
    workspaces: AtomicUsize,
    frontiers: AtomicUsize,
    completions: AtomicUsize,
    function_assemblies: AtomicUsize,
    compiler_entry_gate: Mutex<Option<CompilerEntryGate>>,
}
#[cfg(test)]
struct CompilerEntryGate {
    entered: tokio::sync::oneshot::Sender<()>,
    stopped: tokio::sync::oneshot::Sender<()>,
    exit: std::sync::mpsc::Receiver<()>,
}
#[cfg(test)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct PreparationEffectCounts {
    pub(crate) workspaces: usize,
    pub(crate) frontiers: usize,
    pub(crate) completions: usize,
    pub(crate) function_assemblies: usize,
}
#[cfg(test)]
impl PreparationEffectCounts {
    pub(crate) fn since(self, before: Self) -> Self {
        Self {
            workspaces: self.workspaces - before.workspaces,
            frontiers: self.frontiers - before.frontiers,
            completions: self.completions - before.completions,
            function_assemblies: self.function_assemblies - before.function_assemblies,
        }
    }
}
#[cfg(test)]
impl MathService {
    pub(crate) fn preparation_effects(&self) -> PreparationEffectCounts {
        use std::sync::atomic::Ordering::Relaxed;
        let effects = &self.preparation_effects;
        PreparationEffectCounts {
            workspaces: effects.workspaces.load(Relaxed),
            frontiers: effects.frontiers.load(Relaxed),
            completions: effects.completions.load(Relaxed),
            function_assemblies: effects.function_assemblies.load(Relaxed),
        }
    }
}
/// Compiler preparations and value rebinds this service performed (A6). Observation only:
/// nothing decides on them.
#[derive(Debug, Default)]
struct Preparations {
    views: AtomicUsize,
    observations: AtomicUsize,
    rebuilt: AtomicUsize,
    shared: AtomicUsize,
}
/// Finite performing-owner preparation populations; summaries may overlap.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PreparationProduct {
    StructuralView,
    ObservationProgram,
    ValueRebuilt,
    ValueShared,
}
impl PreparationProduct {
    pub(crate) const ALL: [Self; 4] = [
        Self::StructuralView,
        Self::ObservationProgram,
        Self::ValueRebuilt,
        Self::ValueShared,
    ];
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::StructuralView => "structural_view",
            Self::ObservationProgram => "observation_program",
            Self::ValueRebuilt => "value_rebuilt",
            Self::ValueShared => "value_shared",
        }
    }
    fn counter(self, owner: &Preparations) -> &AtomicUsize {
        match self {
            Self::StructuralView => &owner.views,
            Self::ObservationProgram => &owner.observations,
            Self::ValueRebuilt => &owner.rebuilt,
            Self::ValueShared => &owner.shared,
        }
    }
    pub(crate) fn count(self, sample: &PreparationCounts) -> usize {
        match self {
            Self::StructuralView => sample.views,
            Self::ObservationProgram => sample.observations,
            Self::ValueRebuilt => sample.rebuilt,
            Self::ValueShared => sample.shared,
        }
    }
}

#[cfg(test)]
mod preparation_metric_tests {
    use super::*;
    #[test]
    fn preparation_metric_keys_select_actual_owner_counters_after_reordering() {
        let owner = Preparations::default();
        for (product, count) in [
            (PreparationProduct::ValueShared, 11),
            (PreparationProduct::ObservationProgram, 5),
            (PreparationProduct::StructuralView, 3),
            (PreparationProduct::ValueRebuilt, 7),
        ] {
            product
                .counter(&owner)
                .fetch_add(count, std::sync::atomic::Ordering::Relaxed);
        }
        let sample = owner.snapshot();
        assert_eq!(
            (
                sample.views,
                sample.observations,
                sample.rebuilt,
                sample.shared
            ),
            (3, 5, 7, 11)
        );
        let selected = PreparationProduct::ALL
            .into_iter()
            .rev()
            .map(|key| (key.as_str(), key.count(&sample)))
            .collect::<BTreeMap<_, _>>();
        assert_eq!(
            selected,
            [
                ("structural_view", 3),
                ("observation_program", 5),
                ("value_rebuilt", 7),
                ("value_shared", 11)
            ]
            .into()
        );
    }
}
static PROCESS_PREPARATIONS: std::sync::OnceLock<Preparations> = std::sync::OnceLock::new();
/// Performing-owner process totals; operation and service summaries are not additive.
pub fn preparation_metrics() -> PreparationCounts {
    PROCESS_PREPARATIONS
        .get()
        .map_or_else(PreparationCounts::default, Preparations::snapshot)
}
impl Preparations {
    fn snapshot(&self) -> PreparationCounts {
        let read = |product: PreparationProduct| {
            product
                .counter(self)
                .load(std::sync::atomic::Ordering::Relaxed)
        };
        PreparationCounts {
            views: read(PreparationProduct::StructuralView),
            observations: read(PreparationProduct::ObservationProgram),
            rebuilt: read(PreparationProduct::ValueRebuilt),
            shared: read(PreparationProduct::ValueShared),
        }
    }
}
tokio::task_local! {
    /// The counters of every operation enclosing the current task, such as one study, so
    /// concurrent operations on one service each count only their own preparations.
    static SCOPES: Vec<Arc<Preparations>>;
}
/// Run `work` and count the preparations it performs on any math service (A6), whatever
/// else runs concurrently. An enclosing count also includes this one.
pub(crate) async fn counted<T>(work: impl Future<Output = T>) -> (T, PreparationCounts) {
    let counters = Arc::new(Preparations::default());
    let mut scopes = SCOPES.try_with(Clone::clone).unwrap_or_default();
    scopes.push(Arc::clone(&counters));
    let output = SCOPES.scope(scopes, work).await;
    (output, counters.snapshot())
}
/// Synchronous operation scope, including futures polled by a caller-owned runtime.
pub(crate) fn counted_sync<T>(work: impl FnOnce() -> T) -> (T, PreparationCounts) {
    let counters = Arc::new(Preparations::default());
    let mut scopes = SCOPES.try_with(Clone::clone).unwrap_or_default();
    scopes.push(Arc::clone(&counters));
    let output = SCOPES.sync_scope(scopes, work);
    (output, counters.snapshot())
}
/// Snapshot the innermost operation-local preparation scope, never global service deltas.
pub(crate) fn current_preparations() -> Option<PreparationCounts> {
    SCOPES
        .try_with(|scopes| scopes.last().map(|scope| scope.snapshot()))
        .ok()
        .flatten()
}
/// Structural preparations and value rebinds: of a math service, or of one operation such
/// as a study (A6).
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
    schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct PreparationCounts {
    /// Structural solver-view preparations: a case plan, structural analysis and artifact
    /// requests built from a bound structure.
    pub views: usize,
    /// Value-independent observation programs compiled.
    pub observations: usize,
    /// Value rebinds that rebuilt value-dependent products because a consumed value changed.
    pub rebuilt: usize,
    /// Value rebinds that shared every product because no consumed value changed.
    pub shared: usize,
}
impl MathService {
    /// Admitted worker stack, also used by nested native pools.
    pub(crate) fn stack_bytes(&self) -> usize {
        self.policy.stack_bytes
    }
    /// The cores one native job may admit.
    pub(crate) fn cores(&self) -> usize {
        self.cores
    }
    /// Existing resource policy used by explicit mathematical attempt owners.
    pub(crate) fn policy(&self) -> &MathPolicy {
        &self.policy
    }
    /// Existing accounted allocation pool; no additional owner is constructed.
    pub(crate) fn pool(&self) -> &Arc<dyn MemoryPool> {
        &self.pool
    }
    #[cfg(test)]
    pub(crate) fn test_admission_snapshot(&self) -> (usize, usize, usize) {
        (
            self.pool.reserved(),
            self.cpu.available_permits(),
            self.jobs.available_permits(),
        )
    }
}
impl std::fmt::Debug for MathService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MathService")
            .field("policy", &self.policy)
            .finish_non_exhaustive()
    }
}
/// Runtime workspace owner, including generation storage and preparation serialization.
#[derive(Clone, Debug)]
pub struct Workspace {
    desired_context: Option<CompilerContext>,
    compiler: Arc<Mutex<CompilerWorkspace>>,
    _lease: Arc<pse_columnar::AllocationLease>,
    lane_head: Arc<Mutex<Option<modeling::LaneStamp>>>,
    admission: Arc<tokio::sync::Semaphore>,
}
/// Actual compiler exclusion and workspace storage retained by the native join owner.
#[derive(Debug)]
struct CompilerOwner {
    _workspace: Workspace,
    _permit: tokio::sync::OwnedSemaphorePermit,
}
/// Exclusive compiler authority acquired before active worker dispatch.
#[derive(Debug)]
pub(crate) struct CompilerAuthority {
    workspace: Workspace,
    retention: Arc<Mutex<Option<CompilerOwner>>>,
}
impl CompilerAuthority {
    fn new(workspace: Workspace, permit: tokio::sync::OwnedSemaphorePermit) -> Self {
        Self {
            retention: Arc::new(Mutex::new(Some(CompilerOwner {
                _workspace: workspace.clone(),
                _permit: permit,
            }))),
            workspace,
        }
    }
    fn lock(&self) -> Result<std::sync::MutexGuard<'_, CompilerWorkspace>, MathRuntimeError> {
        let retained = self
            .retention
            .lock()
            .map_err(|_| MathRuntimeError::Internal("compiler retention lock poisoned".into()))?;
        if !retained
            .as_ref()
            .is_some_and(|owner| Arc::ptr_eq(&owner._workspace.compiler, &self.workspace.compiler))
        {
            return Err(MathRuntimeError::Infrastructure(
                "compiler authority is not held".into(),
            ));
        }
        drop(retained);
        self.workspace
            .compiler
            .try_lock()
            .map_err(|error| match error {
                std::sync::TryLockError::Poisoned(_) => {
                    MathRuntimeError::Internal("admitted compiler lock poisoned".into())
                }
                error @ std::sync::TryLockError::WouldBlock => MathRuntimeError::Infrastructure(
                    format!("admitted compiler lock unavailable: {error}"),
                ),
            })
    }
    fn release(&self) -> Result<(), MathRuntimeError> {
        drop(
            self.retention
                .lock()
                .map_err(|_| MathRuntimeError::Internal("compiler retention lock poisoned".into()))?
                .take(),
        );
        Ok(())
    }
    /// Transfer the actual join owner's authority to a fresh private workspace.
    fn replace_private(self, workspace: Workspace) -> Result<Self, MathRuntimeError> {
        let permit = workspace
            .admission
            .clone()
            .try_acquire_owned()
            .map_err(|_| {
                MathRuntimeError::Infrastructure(
                    "new private compiler admission unavailable".into(),
                )
            })?;
        self.replace_held(workspace, permit)
    }
    /// Transfer only after the previous authority has been released. A retained
    /// historical accelerator has the same exclusive writer semaphore as a live one.
    fn replace_held(
        mut self,
        workspace: Workspace,
        permit: tokio::sync::OwnedSemaphorePermit,
    ) -> Result<Self, MathRuntimeError> {
        *self
            .retention
            .lock()
            .map_err(|_| MathRuntimeError::Internal("compiler retention lock poisoned".into()))? =
            Some(CompilerOwner {
                _workspace: workspace.clone(),
                _permit: permit,
            });
        self.workspace = workspace;
        Ok(self)
    }
    #[cfg(test)]
    fn private(workspace: Workspace) -> Result<Self, MathRuntimeError> {
        let permit = workspace
            .admission
            .clone()
            .try_acquire_owned()
            .map_err(|_| {
                MathRuntimeError::Infrastructure("test compiler admission unavailable".into())
            })?;
        Ok(Self::new(workspace, permit))
    }
}
/// Prepared semantic outputs retain their own payload after a generation rotates.
#[derive(Clone, Debug)]
pub struct Preparation {
    prepared: Arc<PreparedCase>,
    _owner: Arc<products::ProductOwner>,
    /// The assembled programs of this structure, shared by every value rebind (A6).
    executable: Arc<std::sync::OnceLock<Arc<ExecutableCase>>>,
}
impl Preparation {
    /// Immutable compiler products, including source maps and proof assumptions.
    pub fn compiled(&self) -> &PreparedCase {
        &self.prepared
    }
    /// Share the compiler-owned complete structural witness without copying or rematching.
    pub fn structural_witness(
        &self,
    ) -> pse_math::SharedAllocation<pse_structural::incidence::StructuralAnalysis> {
        self.prepared.structure.clone()
    }
    /// Pure semantic outputs. No evaluator is constructed by preparation.
    pub fn structure(&self) -> &pse_structural::incidence::StructuralAnalysis {
        &self.prepared.structure
    }
}
/// Executable selected case retaining every program and semantic allocation owner.
#[derive(Debug)]
pub struct ExecutableCase {
    pub(crate) assembly: Arc<CaseAssembly>,
    _artifacts: Vec<Arc<Artifact>>,
    _layout: Arc<pse_columnar::AllocationLease>,
    _owner: Arc<dyn pse_math::AllocationOwner>,
}
/// Provider workers for one attempt, each attached to the attempt's cooperative
/// cancellation. Nested native providers (implicit inner solves) poll it while iterating,
/// so cancelling an attempt reaches every evaluator it builds (F31).
pub(crate) fn attempt_providers(
    registrations: &BTreeMap<ProviderKey, pse_kernels::Registration>,
    scope: &pse_kernels::ExecutionScope,
) -> Result<pse_kernels::ProviderBindings, pse_kernels::ProviderError> {
    registrations
        .iter()
        .map(|(key, registration)| {
            registration
                .worker_scoped(scope.nested())
                .map(|worker| (*key, worker))
        })
        .collect()
}
/// Attempt-local mutable state and its reservation. An opted-in native session may
/// reclaim exclusively drained/reset numeric storage; never retained in Salsa.
#[derive(Debug)]
pub struct ExecutionWorker {
    worker: CaseWorker,
    _case: Arc<ExecutableCase>,
    _charge: WorkerCharge,
}
impl ExecutionWorker {
    /// Access attempt-local numeric operations.
    pub fn worker(&mut self) -> &mut CaseWorker {
        &mut self.worker
    }
}
impl MathService {
    pub(crate) fn new(
        pool: Arc<dyn MemoryPool>,
        cpu: Arc<pse_engine::resources::ReleaseNotifyingSemaphore>,
        cores: usize,
        policy: MathPolicy,
        native: &Arc<pse_engine::cache_service::NativeCacheService>,
    ) -> Arc<Self> {
        // Production assembly installs this outer wrapper before queries and caches.
        // Isolated compiler fixtures may supply a bare pool; wrap their math consumers too.
        let (pool, released): (Arc<dyn MemoryPool>, _) =
            match pool.downcast_ref::<pse_engine::resources::ReleaseNotifyingPool>() {
                Some(notifying) => (pool.clone(), notifying.released()),
                None => {
                    let notifying = pse_engine::resources::ReleaseNotifyingPool::new(pool);
                    let released = notifying.released();
                    (Arc::new(notifying), released)
                }
            };
        let pressure = retention::Pressure::new(policy.artifact_bytes);
        let service = Arc::new(Self {
            pool,
            released,
            cpu,
            cores,
            jobs: Arc::new(pse_engine::resources::ReleaseNotifyingSemaphore::new(
                policy.jobs,
            )),
            active_jobs: Arc::new(pse_engine::resources::ReleaseNotifyingSemaphore::new(
                policy.jobs,
            )),
            entries: pressure.artifacts(),
            modeling_cache: retention::ModelingCache::new(pressure.clone()),
            compiler_lanes: Mutex::default(),
            native_lanes: Mutex::default(),
            flights: Flights::new(policy.flights),
            scalar_flights: Flights::new(policy.flights),
            selected_flights: Flights::new(policy.flights),
            frontier_flights: Flights::new(policy.flights),
            basis_flights: Flights::new(policy.flights),
            policy,
            retention: pressure.artifact_fence(),
            pressure,
            live: Arc::default(),
            hits: AtomicUsize::new(0),
            misses: AtomicUsize::new(0),
            products: Mutex::default(),
            preparations: Preparations::default(),
            #[cfg(test)]
            preparation_effects: Arc::default(),
        });
        let component: Arc<dyn CacheComponent> = service.clone();
        native.register_component(&component);
        service
    }
    pub(crate) fn reserve(
        &self,
        name: &str,
        bytes: usize,
    ) -> Result<Arc<pse_columnar::AllocationLease>, MathRuntimeError> {
        let r = MemoryConsumer::new(name).register(&self.pool);
        if self.pressure.try_grow(&r, bytes).is_err() {
            self.trim_compiler_lanes();
            self.retire_idle_native_lanes();
            self.pressure.try_grow(&r, bytes)?;
        }
        Ok(pse_columnar::AllocationLease::new(r))
    }
    pub(crate) fn dynamic_snapshots(self: &Arc<Self>) -> crate::workflow::SnapshotAllocator {
        let pool = self.pool.clone();
        let pressure = self.pressure.clone();
        let service = Arc::downgrade(self);
        crate::workflow::SnapshotAllocator::new(move |bytes| {
            let reservation =
                MemoryConsumer::new("math:dynamic-derivative-snapshot").register(&pool);
            if pressure.try_grow(&reservation, bytes).is_err() {
                if let Some(service) = service.upgrade() {
                    service.trim_compiler_lanes();
                }
            } else {
                return Ok(pse_columnar::AllocationLease::new(reservation));
            }
            pressure
                .try_grow(&reservation, bytes)
                .map_err(|error| pse_backend_native::ProblemError::memory(error.to_string()))?;
            Ok(pse_columnar::AllocationLease::new(reservation))
        })
    }
    /// Reserve a workspace before constructing its Salsa generation.
    pub fn workspace(
        self: &Arc<Self>,
        inputs: CompilerContext,
        mut limits: WorkspaceLimits,
    ) -> Result<Workspace, MathRuntimeError> {
        limits.input_bytes = limits.input_bytes.min(self.policy.workspace_bytes / 2);
        limits.retained_bytes = limits.retained_bytes.min(self.policy.workspace_bytes / 2);
        let lease = self.reserve("math:compiler-workspace", self.policy.workspace_bytes)?;
        let mut compiler = CompilerWorkspace::new(inputs.clone(), limits)?;
        compiler.attach_body_retention(Arc::new(retention::BodyRetention(Arc::downgrade(self))))?;
        #[cfg(test)]
        self.preparation_effects
            .workspaces
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Ok(Workspace {
            desired_context: Some(inputs),
            compiler: Arc::new(Mutex::new(compiler)),
            _lease: lease,
            lane_head: Arc::default(),
            admission: Arc::new(tokio::sync::Semaphore::new(1)),
        })
    }
    /// Structural preparations and value rebinds performed so far (A6).
    pub fn preparations(&self) -> PreparationCounts {
        self.preparations.snapshot()
    }
    /// Count one preparation in this service and in every enclosing [`counted`] operation.
    fn count(&self, product: PreparationProduct) {
        use std::sync::atomic::Ordering::Relaxed;
        crate::tooling::operational_metrics::preparation_activity();
        product
            .counter(PROCESS_PREPARATIONS.get_or_init(Preparations::default))
            .fetch_add(1, Relaxed);
        product.counter(&self.preparations).fetch_add(1, Relaxed);
        let _ = SCOPES.try_with(|scopes| {
            for scope in scopes {
                product.counter(scope).fetch_add(1, Relaxed);
            }
        });
    }
    /// Resolve the exact compiler requests and bind immutable programs once per structure;
    /// every value rebind of the structure shares them.
    pub async fn assemble(
        self: &Arc<Self>,
        prepared: Preparation,
    ) -> Result<Arc<ExecutableCase>, MathRuntimeError> {
        let driver = self.preparation_driver(&crate::CancelSource::new())?;
        self.assemble_for(prepared, &driver).await
    }
    pub(super) async fn assemble_for(
        self: &Arc<Self>,
        prepared: Preparation,
        driver: &crate::CancelSource,
    ) -> Result<Arc<ExecutableCase>, MathRuntimeError> {
        let driver = self.preparation_driver(driver)?;
        if let Some(executable) = prepared.executable.get() {
            return Ok(executable.clone());
        }
        let memo = prepared.executable.clone();
        let executable = self.assemble_programs(prepared, &driver).await?;
        Ok(memo.get_or_init(|| executable).clone())
    }
    async fn assemble_programs(
        self: &Arc<Self>,
        prepared: Preparation,
        driver: &crate::CancelSource,
    ) -> Result<Arc<ExecutableCase>, MathRuntimeError> {
        let plan = prepared.prepared.plan.clone();
        let layout = self.reserve(
            "case_operation_layout",
            plan.operation_layout_allocation_bound()?,
        )?;
        let artifacts = self
            .prepare_artifacts(prepared.prepared.artifacts.iter(), driver)
            .await?;
        let _span =
            tracing::info_span!("pse.case.program_assembly", pse.owner.stage = true).entered();
        let assembly =
            Arc::new(plan.assemble(artifacts.iter().map(|a| a.program.clone()).collect())?);
        let _ = self.preparation_driver(driver)?;
        Ok(Arc::new(ExecutableCase {
            assembly,
            _artifacts: artifacts,
            _layout: layout,
            // The executable is shared by every binding of this stable plan.
            // Its programs retain their own leases; binding allocations must retire.
            _owner: plan,
        }))
    }
    /// Construct, use and destroy native workers on the same admitted thread.
    /// Only owned Send inputs/results cross the boundary; worker types may remain !Send.
    pub async fn with_worker<T: Send + 'static>(
        self: &Arc<Self>,
        case: Arc<ExecutableCase>,
        providers: BTreeMap<ProviderKey, pse_kernels::Registration>,
        driver: &crate::CancelSource,
        work: impl FnOnce(&mut CaseWorker) -> Result<T, MathRuntimeError> + Send + 'static,
    ) -> Result<T, MathRuntimeError> {
        self.with_owned_worker(case, providers, driver, None, move |mut worker| {
            work(&mut worker)
        })
        .await
    }
    /// Transfer a worker to a finite native diagnostic on its admitted execution thread.
    /// The compiled owner remains alive through callback teardown and completion.
    /// A scoped task supplies the original flight cancellation and absolute deadline;
    /// `None` deliberately retains direct observation's unbounded task semantics.
    pub(crate) async fn with_owned_worker<T: Send + 'static>(
        self: &Arc<Self>,
        case: Arc<ExecutableCase>,
        providers: BTreeMap<ProviderKey, pse_kernels::Registration>,
        driver: &crate::CancelSource,
        task: Option<(pse_kernels::ExecutionScope, Cancellation)>,
        work: impl FnOnce(CaseWorker) -> Result<T, MathRuntimeError> + Send + 'static,
    ) -> Result<T, MathRuntimeError> {
        if driver.token().is_cancelled() {
            return Err(MathRuntimeError::Cancelled);
        }
        let (scope, control) = match task {
            Some((scope, control)) => {
                if !Arc::ptr_eq(scope.cancellation(), &control.flag()) {
                    return Err(pse_backend_native::ProblemError::Contract(
                        "worker task cancellation owner differs from flight".into(),
                    )
                    .into());
                }
                scope
                    .check()
                    .map_err(pse_backend_native::ProblemError::Provider)?;
                (scope, control)
            }
            None => {
                let control = Cancellation::default();
                (
                    pse_kernels::ExecutionScope::new(control.clone(), None),
                    control,
                )
            }
        };
        let service = self.clone();
        let bytes = case.assembly.numeric_worker_bytes();
        let budget = WorkerBudget::new(bytes);
        let operation = self.job_scoped(1, bytes, control.clone(), scope.deadline(), move |flag| {
            if !Arc::ptr_eq(&flag, scope.cancellation()) {
                return Err(pse_backend_native::ProblemError::Contract(
                    "worker admission changed task cancellation owner".into(),
                )
                .into());
            }
            let ExecutionWorker {
                worker,
                _case,
                _charge,
            } = service.worker(case, &providers, scope.clone(), &budget)?;
            let result = work(worker);
            drop(_case);
            drop(_charge);
            let value = result?;
            scope
                .check()
                .map_err(pse_backend_native::ProblemError::Provider)?;
            Ok(value)
        });
        tokio::pin!(operation);
        tokio::select! {result=&mut operation=>result,()=driver.cancelled()=>{control.cancel();let _=operation.await;Err(MathRuntimeError::Cancelled)}}
    }
    /// Opt into exclusive reuse only on the actual native session owner thread.
    /// Ordinary workers (including independent original assessments) remain cold.
    pub(crate) fn reusable_worker(
        &self,
        case: Arc<ExecutableCase>,
        registrations: &BTreeMap<ProviderKey, pse_kernels::Registration>,
        scope: pse_kernels::ExecutionScope,
        budget: &Arc<WorkerBudget>,
    ) -> Result<ExecutionWorker, MathRuntimeError> {
        let Some(origin) = staged::scratch_origin() else {
            return self.worker(case, registrations, scope, budget);
        };
        // Covers the callback box, two passive allocation handles, lane slot metadata
        // and exact immutable-owner references before any opt-in allocation.
        let Some(bytes) = case
            .assembly
            .numeric_worker_bytes()
            .checked_add(1024)
            .filter(|bytes| *bytes <= self.policy.worker_bytes)
        else {
            return self.worker(case, registrations, scope, budget);
        };
        scope
            .check()
            .map_err(pse_backend_native::ProblemError::Provider)?;
        let reusable = staged::take_scratch(&origin, &case, budget, bytes);
        let (scratch, charge) = match reusable {
            Some((scratch, charge)) => (Some(scratch), charge),
            None => match budget.charge(bytes) {
                Ok(charge) => (None, charge),
                // Reuse is optional. Its metadata must not reduce the existing
                // scientific worker's policy or actual budget eligibility.
                Err(_) => return self.worker(case, registrations, scope, budget),
            },
        };
        let providers = attempt_providers(registrations, &scope)
            .map_err(pse_backend_native::ProblemError::Provider)?;
        let mut worker = match scratch {
            Some(scratch) => case
                .assembly
                .worker_from_scratch(scratch, providers, scope)?,
            None => case.assembly.worker_scoped(providers, scope),
        };
        let returned = charge.share();
        let owner = case.clone();
        worker.recycle_on_drop(move |scratch| {
            staged::return_scratch(origin, owner, scratch, returned)
        });
        Ok(ExecutionWorker {
            worker,
            _case: case,
            _charge: charge,
        })
    }
    /// Construct a mutable attempt evaluator after admission on its owning execution
    /// thread. It is the one construction path: its providers observe the attempt's
    /// cancellation, and its numeric storage is charged to the job's budget for as long as
    /// it lives (F31).
    pub(crate) fn worker(
        &self,
        case: Arc<ExecutableCase>,
        registrations: &BTreeMap<ProviderKey, pse_kernels::Registration>,
        scope: pse_kernels::ExecutionScope,
        budget: &Arc<WorkerBudget>,
    ) -> Result<ExecutionWorker, MathRuntimeError> {
        let bytes = case.assembly.numeric_worker_bytes();
        if bytes > self.policy.worker_bytes {
            return Err(MathRuntimeError::Limit("worker storage"));
        }
        let charge = match budget.charge(bytes) {
            Ok(charge) => charge,
            Err(_) if staged::yield_idle_scratch(budget) => {
                scope
                    .check()
                    .map_err(pse_backend_native::ProblemError::Provider)?;
                budget.charge(bytes)?
            }
            Err(error) => return Err(error),
        };
        scope
            .check()
            .map_err(pse_backend_native::ProblemError::Provider)?;
        let providers = attempt_providers(registrations, &scope)
            .map_err(pse_backend_native::ProblemError::Provider)?;
        let worker = case.assembly.worker_scoped(providers, scope);
        Ok(ExecutionWorker {
            worker,
            _case: case,
            _charge: charge,
        })
    }
}

#[cfg(test)]
mod tests;

impl pse_model::diagnostic::DiagnosticProjection for MathRuntimeError {
    fn boundary_diagnostic(
        &self,
        stage: pse_diagnostics::DiagnosticStage,
    ) -> pse_model::diagnostic::BoundaryDiagnostic {
        use pse_diagnostics::TypedDiagnostic;
        use pse_model::diagnostic::project_facts;
        match self {
            Self::Solve(e) => e.boundary_diagnostic(stage),
            Self::Compile(e) => e.boundary_diagnostic(stage),
            Self::Math(e) => e.boundary_diagnostic(stage),
            Self::Shared(e) => e.boundary_diagnostic(stage),
            Self::Strategy { cause, .. } => cause.boundary_diagnostic(stage),
            Self::StrategyTraceUnavailable { cause, refusal } => {
                let mut diagnostic = cause.boundary_diagnostic(stage);
                diagnostic.causes.push(refusal.boundary_diagnostic(stage));
                diagnostic
            }
            Self::Allocation { .. }
            | Self::WorkerJoin { .. }
            | Self::Pool(_)
            | Self::Limit(_)
            | Self::PreparationExpired
            | Self::Cancelled
            | Self::Retiring
            | Self::Infrastructure(_)
            | Self::Internal(_)
            | Self::Panic(_) => {
                project_facts(self.diagnostic_code(), self.diagnostic_facts(), stage)
            }
        }
    }
}

#[cfg(test)]
mod compiler_authority_tests {
    #![allow(
        clippy::unwrap_used,
        clippy::panic,
        reason = "controls poison actual compiler ownership locks and inspect their retained typed failures"
    )]
    use super::*;
    use pse_model::diagnostic::{BoundaryClass, DiagnosticProjection};

    fn workspace(service: &Arc<MathService>) -> Workspace {
        let physical = crate::workflow::tests::physical();
        service
            .workspace(
                CompilerContext {
                    quantities: physical.quantities.clone(),
                    preconditions: physical.preconditions.clone(),
                    providers: BTreeMap::new(),
                },
                WorkspaceLimits::default(),
            )
            .unwrap()
    }

    fn assert_internal(error: MathRuntimeError) {
        assert!(matches!(&error, MathRuntimeError::Internal(_)));
        let stage = pse_diagnostics::DiagnosticStage::Evaluation;
        assert_eq!(
            error.boundary_diagnostic(stage).class,
            BoundaryClass::Internal
        );
        let nested = MathRuntimeError::Math(error.into_math());
        assert_eq!(
            nested.boundary_diagnostic(stage).class,
            BoundaryClass::Internal
        );
        assert_eq!(
            strategy::runtime_failure(&nested),
            pse_model::generated::enums::NumericalAttemptObservation::OperationalFailure
        );
    }

    #[test]
    fn poisoned_compiler_retention_preserves_internal_class_through_every_authority_operation() {
        let service = tests::service();
        let authority = CompilerAuthority::private(workspace(&service)).unwrap();
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let _held = authority.retention.lock().unwrap();
                panic!("injected compiler retention poison");
            }))
            .is_err()
        );
        assert_internal(authority.lock().unwrap_err());
        assert_internal(authority.release().unwrap_err());
        let replacement = workspace(&service);
        let permit = replacement.admission.clone().try_acquire_owned().unwrap();
        assert_internal(authority.replace_held(replacement, permit).unwrap_err());
    }

    #[test]
    fn poisoned_admitted_compiler_mutex_is_internal_rather_than_external_infrastructure() {
        let service = tests::service();
        let authority = CompilerAuthority::private(workspace(&service)).unwrap();
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let _held = authority.workspace.compiler.lock().unwrap();
                panic!("injected compiler workspace poison");
            }))
            .is_err()
        );
        assert_internal(authority.lock().unwrap_err());
    }
}
