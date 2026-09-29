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
mod products;
pub mod settings;
pub mod solves;
mod staged;
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
    CompileError, CompilerWorkspace, Inputs, PreparedCase, Profile, WorkspaceLimits,
};
use pse_engine::cache_service::CacheComponent;
use pse_ids::SemanticId;
use pse_kernels::{DerivativeOrder, Provider, ProviderKey};
use pse_math::assembly::{CaseAssembly, CaseWorker};
pub(crate) use staged::NativeSession;
use std::{
    collections::BTreeMap,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicUsize},
    },
};
/// Finite math allowances draw from the deployment pool, never a second memory budget.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MathPolicy {
    /// Completed program retention limit (not an up-front reservation).
    pub artifact_bytes: usize,
    /// Conservative foreign-library allowance per native job and program.
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
            worker_bytes: 2 << 30,
            workspace_bytes: 4 << 30,
            stack_bytes: pse_structural::incidence::MATCHING_STACK,
            jobs: 4,
            flights: 128,
        }
    }
}
impl MathPolicy {
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
    /// Finite admission bound.
    #[error("math resource limit: {0}")]
    Limit(&'static str),
    /// Cancellation does not imply native exit.
    #[error("math work cancelled")]
    Cancelled,
    /// A previous shared load is still joining after its last waiter departed.
    #[error("math program load is retiring; retry after completion")]
    Retiring,
    /// Supervisor/thread failure.
    #[error("math infrastructure: {0}")]
    Infrastructure(String),
}
impl MathRuntimeError {
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
            Self::Pool(e) => ProblemError::memory(e.to_string()),
            Self::Shared(e) => Arc::try_unwrap(e).map_or_else(
                |e| ProblemError::internal(e.to_string()),
                Self::into_problem,
            ),
            Self::Compile(CompileError::Cancelled) => ProblemError::Cancelled,
            Self::Compile(CompileError::Limit(detail)) => ProblemError::Limit {
                kind: LimitKind::Work,
                detail: detail.into(),
            },
            Self::Compile(e) => ProblemError::internal(e.to_string()),
            Self::Retiring | Self::Infrastructure(_) => ProblemError::internal(self.to_string()),
        }
    }
}
pse_diagnostics::impl_diagnostic! {
    MathRuntimeError,
    code(this) { match this {Self::Cancelled=>Some(pse_diagnostics::DiagnosticCode::RuntimeCancelled),Self::Retiring|Self::Limit(_)|Self::Pool(_)=>Some(pse_diagnostics::DiagnosticCode::RuntimeResourceLimit),Self::Infrastructure(_)=>Some(pse_diagnostics::DiagnosticCode::RuntimeInfrastructure),_=>None} },
    forward(this) { match this {Self::Solve(e)=>Some(e),Self::Compile(e)=>Some(e),Self::Math(e)=>Some(e),Self::Shared(e)=>Some(e.as_ref()),_=>None} },
    help(_this) { None },related(_this) { None },source(_this) { None }
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
    flights: Flights<Key, Artifact, MathRuntimeError>,
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
    cancel: &Arc<AtomicBool>,
) -> Result<BTreeMap<ProviderKey, Box<dyn Provider>>, pse_kernels::ProviderError> {
    registrations
        .iter()
        .map(|(key, registration)| {
            registration
                .worker_scoped(cancel.clone())
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
            flights: Flights::new(policy.flights),
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
        &self,
        inputs: Inputs,
        mut limits: WorkspaceLimits,
    ) -> Result<Workspace, MathRuntimeError> {
        limits.input_bytes = limits.input_bytes.min(self.policy.workspace_bytes / 2);
        limits.retained_bytes = limits.retained_bytes.min(self.policy.workspace_bytes / 2);
        let lease = self.reserve("math:compiler-workspace", self.policy.workspace_bytes)?;
        let compiler = CompilerWorkspace::new(inputs, limits)?;
        Ok(Workspace {
            compiler: Arc::new(Mutex::new(compiler)),
            lease,
        })
    }
    /// Publish a validated batch under the single application writer.
    pub fn publish(&self, workspace: &Workspace, inputs: Inputs) -> Result<(), MathRuntimeError> {
        workspace
            .compiler
            .lock()
            .map_err(|_| MathRuntimeError::Infrastructure("compiler lock poisoned".into()))?
            .publish(inputs)?;
        Ok(())
    }
    /// Prepare on a bounded owned worker. Driver cancellation requests native cancellation.
    pub async fn prepare(
        self: &Arc<Self>,
        workspace: Workspace,
        id: SemanticId,
        order: DerivativeOrder,
        profile: Profile,
        coefficients: bool,
        driver: &crate::CancelSource,
    ) -> Result<Preparation, MathRuntimeError> {
        let control = FlightCancellation::default();
        let foreign = self.policy.foreign_bytes;
        let operation = self.job_retained(1, WITHIN_WORKSPACE, control.clone(), move |flag| {
            let _workspace_lease = workspace.lease;
            let mut compiler = workspace
                .compiler
                .lock()
                .map_err(|_| MathRuntimeError::Infrastructure("compiler lock poisoned".into()))?;
            let prepared = compiler.prepare_cancellable(id, order, profile, coefficients, flag)?;
            let bytes = prepared
                .retained_bytes()
                .checked_add(foreign)
                .ok_or(MathRuntimeError::Limit("prepared product extent"))?;
            Ok((prepared, bytes))
        });
        tokio::pin!(operation);
        tokio::select! {result=&mut operation=>self.own_preparation(result?),()=driver.cancelled()=>{control.cancel();Err(MathRuntimeError::Cancelled)}}
    }
    /// Atomically select an immutable revision and prepare it on the same compiler
    /// lock. Concurrent revisions cannot interleave publication and query execution.
    pub async fn prepare_revision(
        self: &Arc<Self>,
        workspace: Workspace,
        inputs: Inputs,
        id: SemanticId,
        order: DerivativeOrder,
        profile: Profile,
        driver: &crate::CancelSource,
    ) -> Result<Preparation, MathRuntimeError> {
        let control = FlightCancellation::default();
        let foreign = self.policy.foreign_bytes;
        let operation = self.job_retained(1, WITHIN_WORKSPACE, control.clone(), move |flag| {
            let _lease = workspace.lease;
            let mut compiler = workspace
                .compiler
                .lock()
                .map_err(|_| MathRuntimeError::Infrastructure("compiler lock poisoned".into()))?;
            compiler.publish(inputs)?;
            let prepared = compiler.prepare_cancellable(id, order, profile, false, flag.clone())?;
            let prepared = if prepared.presolve.coefficient_eligible() {
                compiler.prepare_cancellable(id, order, profile, true, flag)?
            } else {
                prepared
            };
            let bytes = prepared
                .retained_bytes()
                .checked_add(foreign)
                .ok_or(MathRuntimeError::Limit("prepared product extent"))?;
            Ok((prepared, bytes))
        });
        tokio::pin!(operation);
        tokio::select! {result=&mut operation=>self.own_preparation(result?),()=driver.cancelled()=>{control.cancel();Err(MathRuntimeError::Cancelled)}}
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
        self.with_owned_worker(case, providers, driver, move |mut worker| work(&mut worker))
            .await
    }
    /// Transfer a worker to a finite native diagnostic on its admitted execution thread.
    /// The compiled owner remains alive through callback teardown and completion.
    pub(crate) async fn with_owned_worker<T: Send + 'static>(
        self: &Arc<Self>,
        case: Arc<ExecutableCase>,
        providers: BTreeMap<ProviderKey, pse_kernels::Registration>,
        driver: &crate::CancelSource,
        work: impl FnOnce(CaseWorker) -> Result<T, MathRuntimeError> + Send + 'static,
    ) -> Result<T, MathRuntimeError> {
        let control = FlightCancellation::default();
        let service = self.clone();
        let bytes = case.assembly.numeric_worker_bytes();
        let budget = WorkerBudget::new(bytes);
        let operation = self.job(1, bytes, control.clone(), move |flag| {
            let ExecutionWorker {
                worker,
                _case,
                _charge,
            } = service.worker(case, &providers, flag, &budget)?;
            let result = work(worker);
            drop(_case);
            drop(_charge);
            result
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
        cancel: Arc<AtomicBool>,
        budget: &Arc<WorkerBudget>,
    ) -> Result<ExecutionWorker, MathRuntimeError> {
        let bytes = case.assembly.numeric_worker_bytes();
        if bytes > self.policy.worker_bytes {
            return Err(MathRuntimeError::Limit("worker storage"));
        }
        let charge = budget.charge(bytes)?;
        let providers = attempt_providers(registrations, &cancel)
            .map_err(pse_backend_native::ProblemError::Provider)?;
        let worker = case.assembly.worker(providers, cancel);
        Ok(ExecutionWorker {
            worker,
            _case: case,
            _charge: charge,
        })
    }
}

#[cfg(test)]
mod tests;
