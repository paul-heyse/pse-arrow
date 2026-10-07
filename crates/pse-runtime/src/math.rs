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
pub mod portable;
pub mod prediction;
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
pub(crate) use jobs::Submission;
pub use jobs::{WorkerBudget, WorkerCharge};
use pse_columnar::flight::{FlightCancellation, Flights};
use pse_compiler::workspace::{
    CompileError, CompilerContext, CompilerWorkspace, PreparedCase, WorkspaceLimits,
};
use pse_engine::cache_service::CacheComponent;
use pse_kernels::{Provider, ProviderKey};
use pse_math::assembly::{CaseAssembly, CaseWorker};
pub(crate) use staged::NativeSession;
pub(crate) use staged::{SessionDisposition, StepRetention};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex, atomic::AtomicUsize},
};
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
    /// Maximum admitted native jobs, including admission waiters.
    pub jobs: usize,
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
            flights: 128,
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
    /// Finite admission bound.
    #[error("math resource limit: {0}")]
    Limit(&'static str),
    /// Cancellation does not imply native exit.
    #[error("math work cancelled")]
    Cancelled,
    /// A previous shared load is still joining after its last waiter departed.
    #[error("math program load is retiring; retry after completion")]
    Retiring,
    /// A caught worker panic, distinct from resource or infrastructure refusal.
    #[error("math worker panic: {0}")]
    Panic(String),
    /// Supervisor/thread failure.
    #[error("math infrastructure: {0}")]
    Infrastructure(String),
}
impl MathRuntimeError {
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
            Self::Panic(text) | Self::Infrastructure(text) => text.capacity(),
            Self::Limit(_) | Self::Cancelled | Self::Retiring => 0,
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
    /// Carry a runtime failure into the native report's single typed validation record,
    /// keeping its class: cancellation, limits and native causes stay typed.
    pub fn into_problem(self) -> pse_backend_native::ProblemError {
        use pse_backend_native::{LimitKind, ProblemError};
        match self {
            Self::Solve(e) => e,
            Self::Math(e) => ProblemError::Math(e),
            Self::Cancelled => ProblemError::Cancelled,
            Self::Limit(detail) => ProblemError::Limit {
                kind: LimitKind::Memory,
                detail: detail.into(),
            },
            Self::Shared(error) => match Arc::try_unwrap(error) {
                Ok(error) => error.into_problem(),
                Err(error) => Self::Shared(error).into_retained_problem(),
            },
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
            Self::Retiring | Self::Infrastructure(_) | Self::Panic(_) => {
                self.into_retained_problem()
            }
        }
    }
}
pse_diagnostics::impl_diagnostic! {
    MathRuntimeError,
    code(this) { match this {Self::Cancelled=>Some(pse_diagnostics::DiagnosticCode::RuntimeCancelled),Self::Retiring|Self::Limit(_)|Self::Pool(_)=>Some(pse_diagnostics::DiagnosticCode::RuntimeResourceLimit),Self::Infrastructure(_)=>Some(pse_diagnostics::DiagnosticCode::RuntimeInfrastructure),Self::Panic(_)=>Some(pse_diagnostics::DiagnosticCode::WorkflowPanic),_=>None} },
    forward(this) { match this {Self::Solve(e)=>Some(e),Self::Compile(e)=>Some(e),Self::Math(e)=>Some(e),Self::Shared(e)=>Some(e.as_ref()),Self::Strategy {cause,..}|Self::StrategyTraceUnavailable {cause,..}=>Some(cause.as_ref()),_=>None} },
    help(_this) { None },related(_this) { None },source(_this) { None },
    facts(this) {
        use pse_diagnostics::{DiagnosticRule as R, DiagnosticFacts, DiagnosticObservation as O};
        let mut facts=DiagnosticFacts { rule: match this { Self::Cancelled=>Some(R::MathCancelled), Self::Retiring|Self::Limit(_)|Self::Pool(_)=>Some(R::MathLimit), Self::Panic(_)=>Some(R::WorkflowPanic), Self::Infrastructure(_)=>Some(R::MathLibrary), Self::Solve(_) | Self::Compile(_) | Self::Math(_) | Self::Shared(_) | Self::Strategy {..}|Self::StrategyTraceUnavailable {..}=>None },..Default::default() };
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
    cpu: Arc<tokio::sync::Semaphore>,
    cores: usize,
    policy: MathPolicy,
    jobs: Arc<tokio::sync::Semaphore>,
    entries: DefaultCache<Key, Value>,
    pub(crate) modeling_cache: retention::ModelingCache,
    flights: Flights<Key, Artifact, MathRuntimeError>,
    pub(crate) selected_flights: Flights<
        (
            Arc<crate::workflow::modeling::SelectedRequest>,
            Arc<pse_operations::canonical_selection::SelectedDependencies>,
        ),
        modeling::ModelingRevision,
        MathRuntimeError,
    >,
    retention: pse_columnar::retention::RetentionFence,
    live: Arc<AtomicUsize>,
    hits: AtomicUsize,
    misses: AtomicUsize,
    products: Mutex<BTreeMap<Vec<usize>, std::sync::Weak<products::ProductOwner>>>,
    preparations: Preparations,
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
impl Preparations {
    fn snapshot(&self) -> PreparationCounts {
        let read = |n: &AtomicUsize| n.load(std::sync::atomic::Ordering::Relaxed);
        PreparationCounts {
            views: read(&self.views),
            observations: read(&self.observations),
            rebuilt: read(&self.rebuilt),
            shared: read(&self.shared),
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
    compiler: Arc<Mutex<CompilerWorkspace>>,
    lease: Arc<pse_columnar::AllocationLease>,
}
/// Prepared semantic outputs retain their own payload after a generation rotates.
#[derive(Clone, Debug)]
pub struct Preparation {
    prepared: Arc<PreparedCase>,
    owner: Arc<products::ProductOwner>,
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
    _owner: Arc<dyn pse_math::AllocationOwner>,
}
/// Provider workers for one attempt, each attached to the attempt's cooperative
/// cancellation. Nested native providers (implicit inner solves) poll it while iterating,
/// so cancelling an attempt reaches every evaluator it builds (F31).
pub(crate) fn attempt_providers(
    registrations: &BTreeMap<ProviderKey, pse_kernels::Registration>,
    scope: &pse_kernels::ExecutionScope,
) -> Result<BTreeMap<ProviderKey, Box<dyn Provider>>, pse_kernels::ProviderError> {
    registrations
        .iter()
        .map(|(key, registration)| {
            registration
                .worker_scoped(scope.clone())
                .map(|worker| (*key, worker))
        })
        .collect()
}
/// Attempt-local mutable state and its reservation; never retained in Salsa or a cache.
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
        cpu: Arc<tokio::sync::Semaphore>,
        cores: usize,
        policy: MathPolicy,
        native: &Arc<pse_engine::cache_service::NativeCacheService>,
    ) -> Arc<Self> {
        let service = Arc::new(Self {
            pool,
            cpu,
            cores,
            jobs: Arc::new(tokio::sync::Semaphore::new(policy.jobs)),
            entries: DefaultCache::new(policy.artifact_bytes).with_name("pse.cache.math_artifacts"),
            modeling_cache: retention::ModelingCache::new(policy.artifact_bytes),
            flights: Flights::new(policy.flights),
            selected_flights: Flights::new(policy.flights),
            policy,
            retention: Default::default(),
            live: Arc::default(),
            hits: AtomicUsize::new(0),
            misses: AtomicUsize::new(0),
            products: Mutex::default(),
            preparations: Preparations::default(),
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
        r.try_grow(bytes)?;
        Ok(pse_columnar::AllocationLease::new(r))
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
        let mut compiler = CompilerWorkspace::new(inputs, limits)?;
        compiler.attach_body_retention(Arc::new(retention::BodyRetention(Arc::downgrade(self))))?;
        Ok(Workspace {
            compiler: Arc::new(Mutex::new(compiler)),
            lease,
        })
    }
    /// Fresh selected-demand workspace. Durable scientific meaning is looked up before
    /// admission and published before the caller releases its selection protection.
    #[expect(
        clippy::too_many_arguments,
        reason = "one selected workspace binds compiler inputs and limits to its canonical store, protected read, qualified producer, outer build and cancellation owner"
    )]
    pub fn canonical_workspace(
        self: &Arc<Self>,
        inputs: CompilerContext,
        mut limits: WorkspaceLimits,
        store: Arc<pse_operations::canonical::CanonicalStore>,
        read: Arc<Mutex<pse_operations::canonical_selection::SelectedRead>>,
        producer: Option<portable::QualifiedProducer>,
        outer_build: pse_ids::ContentHash,
        cancelled: Arc<std::sync::atomic::AtomicBool>,
    ) -> Result<Workspace, MathRuntimeError> {
        limits.input_bytes = limits.input_bytes.min(self.policy.workspace_bytes / 2);
        limits.retained_bytes = limits.retained_bytes.min(self.policy.workspace_bytes / 2);
        let lease = self.reserve(
            "math:canonical-compiler-workspace",
            self.policy.workspace_bytes,
        )?;
        let handle = tokio::runtime::Handle::try_current().map_err(|e| {
            MathRuntimeError::Infrastructure(format!("canonical workspace requires runtime: {e}"))
        })?;
        let mut compiler = CompilerWorkspace::new(inputs.clone(), limits)?;
        compiler.attach_body_retention(Arc::new(retention::CanonicalBodyRetention {
            memory: retention::BodyRetention(Arc::downgrade(self)),
            store,
            read,
            producer,
            outer_build,
            inputs,
            cancelled,
            handle,
        }))?;
        Ok(Workspace {
            compiler: Arc::new(Mutex::new(compiler)),
            lease,
        })
    }
    /// Structural preparations and value rebinds performed so far (A6).
    pub fn preparations(&self) -> PreparationCounts {
        self.preparations.snapshot()
    }
    /// Count one preparation in this service and in every enclosing [`counted`] operation.
    fn count(&self, counter: impl Fn(&Preparations) -> &AtomicUsize) {
        use std::sync::atomic::Ordering::Relaxed;
        counter(&self.preparations).fetch_add(1, Relaxed);
        let _ = SCOPES.try_with(|scopes| {
            for scope in scopes {
                counter(scope).fetch_add(1, Relaxed);
            }
        });
    }
    /// Resolve the exact compiler requests and bind immutable programs once per structure;
    /// every value rebind of the structure shares them.
    pub async fn assemble(
        self: &Arc<Self>,
        prepared: Preparation,
    ) -> Result<Arc<ExecutableCase>, MathRuntimeError> {
        if let Some(executable) = prepared.executable.get() {
            return Ok(executable.clone());
        }
        let memo = prepared.executable.clone();
        let executable = self.assemble_programs(prepared).await?;
        Ok(memo.get_or_init(|| executable).clone())
    }
    async fn assemble_programs(
        self: &Arc<Self>,
        prepared: Preparation,
    ) -> Result<Arc<ExecutableCase>, MathRuntimeError> {
        let mut artifacts = vec![];
        for request in prepared.prepared.artifacts.iter() {
            artifacts.push(self.artifact(request.clone()).await?);
        }
        let plan = prepared.prepared.plan.clone();
        let _span = tracing::info_span!("pse.case.program_assembly").entered();
        let assembly =
            Arc::new(plan.assemble(artifacts.iter().map(|a| a.program.clone()).collect())?);
        Ok(Arc::new(ExecutableCase {
            assembly,
            _artifacts: artifacts,
            _owner: prepared.owner,
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
        task: Option<(pse_kernels::ExecutionScope, FlightCancellation)>,
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
                let control = FlightCancellation::default();
                (
                    pse_kernels::ExecutionScope::new(control.flag(), None),
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
        let charge = budget.charge(bytes)?;
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
            Self::Pool(_)
            | Self::Limit(_)
            | Self::Cancelled
            | Self::Retiring
            | Self::Infrastructure(_)
            | Self::Panic(_) => {
                project_facts(self.diagnostic_code(), self.diagnostic_facts(), stage)
            }
        }
    }
}
