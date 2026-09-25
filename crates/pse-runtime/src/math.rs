// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! One Salsa compiler and one shared runtime effect boundary for process mathematics.
mod artifacts;
pub mod flows;
mod functions;
#[cfg(feature = "solver-kinsol")]
pub mod initialization;
mod jobs;
pub mod solves;
pub use artifacts::Artifact;
use artifacts::{Key, Value};
use datafusion::execution::{
    cache::default_cache::DefaultCache,
    memory_pool::{MemoryConsumer, MemoryPool},
};
use pse_compiler::workspace::{
    CompileError, CompilerWorkspace, Inputs, PreparedCase, Profile, WorkspaceLimits,
};
use pse_engine::cache_service::{
    CacheComponent,
    flight::{FlightCancellation, Flights},
};
use pse_ids::SemanticId;
use pse_kernels::{DerivativeOrder, Provider, ProviderKey};
use pse_math::assembly::{CaseAssembly, CaseWorker};
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
    /// Supervisor/thread failure.
    #[error("math infrastructure: {0}")]
    Infrastructure(String),
}
pse_diagnostics::impl_diagnostic! {
    MathRuntimeError,
    code(this) { match this {Self::Cancelled=>Some(pse_diagnostics::DiagnosticCode::RuntimeCancelled),Self::Limit(_)|Self::Pool(_)=>Some(pse_diagnostics::DiagnosticCode::RuntimeResourceLimit),Self::Infrastructure(_)=>Some(pse_diagnostics::DiagnosticCode::RuntimeInfrastructure),_=>None} },
    forward(this) { match this {Self::Solve(e)=>Some(e),Self::Compile(e)=>Some(e),Self::Math(e)=>Some(e),Self::Shared(e)=>Some(e.as_ref()),_=>None} },
    help(_this) { None },related(_this) { None },source(_this) { None }
}
/// Deployment-owned mathematics service registered with native cache reporting/invalidation.
pub struct MathService {
    pool: Arc<dyn MemoryPool>,
    cpu: Arc<tokio::sync::Semaphore>,
    cores: usize,
    policy: MathPolicy,
    jobs: Arc<tokio::sync::Semaphore>,
    entries: DefaultCache<Key, Value>,
    flights: Flights<Key, Artifact, MathRuntimeError>,
    publication: Mutex<()>,
    epoch: AtomicUsize,
    live: Arc<AtomicUsize>,
    hits: AtomicUsize,
    misses: AtomicUsize,
}
impl MathService {
    /// Admitted worker stack, also used by nested native pools.
    #[cfg(feature = "solver-pounce")]
    pub(crate) fn stack_bytes(&self) -> usize {
        self.policy.stack_bytes
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
/// Prepared semantic outputs retain the workspace allowance even after a generation rotates.
#[derive(Clone, Debug)]
pub struct Preparation {
    prepared: Arc<PreparedCase>,
    owner: Arc<pse_columnar::AllocationLease>,
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
    _owner: Arc<pse_columnar::AllocationLease>,
}
/// Attempt-local mutable state and its reservation; never retained in Salsa or a cache.
#[derive(Debug)]
pub struct ExecutionWorker {
    worker: CaseWorker,
    _case: Arc<ExecutableCase>,
    _lease: Arc<pse_columnar::AllocationLease>,
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
            publication: Mutex::new(()),
            epoch: AtomicUsize::new(0),
            live: Arc::default(),
            hits: AtomicUsize::new(0),
            misses: AtomicUsize::new(0),
        });
        let component: Arc<dyn CacheComponent> = service.clone();
        native.register_component(&component);
        service
    }
    fn reserve(
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
        let owner = self.reserve("math:prepared-products", self.policy.workspace_bytes)?;
        let operation = self.job(
            1,
            pse_structural::incidence::MATCHING_STACK,
            control.clone(),
            move |flag| {
                let _workspace_lease = workspace.lease;
                let mut compiler = workspace.compiler.lock().map_err(|_| {
                    MathRuntimeError::Infrastructure("compiler lock poisoned".into())
                })?;
                compiler
                    .prepare_cancellable(id, order, profile, coefficients, flag)
                    .map_err(MathRuntimeError::Compile)
            },
        );
        tokio::pin!(operation);
        tokio::select! {result=&mut operation=>Ok(Preparation{prepared:Arc::new(result?),owner}),()=driver.cancelled()=>{control.cancel();Err(MathRuntimeError::Cancelled)}}
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
        let owner = self.reserve("math:prepared-products", self.policy.workspace_bytes)?;
        let operation = self.job(
            1,
            pse_structural::incidence::MATCHING_STACK,
            control.clone(),
            move |flag| {
                let _lease = workspace.lease;
                let mut compiler = workspace.compiler.lock().map_err(|_| {
                    MathRuntimeError::Infrastructure("compiler lock poisoned".into())
                })?;
                compiler.publish(inputs)?;
                let prepared =
                    compiler.prepare_cancellable(id, order, profile, false, flag.clone())?;
                if prepared.facts.affine_rows.iter().all(|v| *v)
                    && prepared.facts.objective_degree.is_some_and(|d| d <= 2)
                    && prepared
                        .presolve
                        .obligations
                        .values()
                        .all(|s| *s == pse_math::presolve::ObligationStatus::Discharged)
                {
                    Ok(compiler.prepare_cancellable(id, order, profile, true, flag)?)
                } else {
                    Ok(prepared)
                }
            },
        );
        tokio::pin!(operation);
        tokio::select! {result=&mut operation=>Ok(Preparation{prepared:Arc::new(result?),owner}),()=driver.cancelled()=>{control.cancel();Err(MathRuntimeError::Cancelled)}}
    }
    /// Resolve the exact compiler requests and bind immutable programs.
    pub async fn assemble(
        self: &Arc<Self>,
        prepared: Preparation,
    ) -> Result<Arc<ExecutableCase>, MathRuntimeError> {
        let mut artifacts = vec![];
        for request in prepared.prepared.artifacts.iter() {
            artifacts.push(self.artifact(request.clone()).await?);
        }
        let plan = Arc::new(
            prepared
                .prepared
                .plan
                .as_ref()
                .clone()
                .with_owner(prepared.owner.clone()),
        );
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
        let control = FlightCancellation::default();
        let service = self.clone();
        let operation = self.job(1, self.policy.worker_bytes, control.clone(), move |flag| {
            let providers = providers
                .into_iter()
                .map(|(key, factory)| {
                    factory.worker().map(|v| (key, v)).map_err(|e| {
                        MathRuntimeError::Math(pse_math::MathError::Contract(e.to_string()))
                    })
                })
                .collect::<Result<_, _>>()?;
            let mut worker = service.worker(case, providers, flag)?;
            work(worker.worker())
        });
        tokio::pin!(operation);
        tokio::select! {result=&mut operation=>result,()=driver.cancelled()=>{control.cancel();Err(MathRuntimeError::Cancelled)}}
    }
    /// Construct mutable workers after admission on their owning execution thread.
    fn worker(
        &self,
        case: Arc<ExecutableCase>,
        providers: BTreeMap<ProviderKey, Box<dyn Provider>>,
        cancel: Arc<AtomicBool>,
    ) -> Result<ExecutionWorker, MathRuntimeError> {
        let bytes = case.assembly.numeric_worker_bytes();
        if bytes > self.policy.worker_bytes {
            return Err(MathRuntimeError::Limit("worker storage"));
        }
        let lease = self.reserve(
            "math:case-worker",
            self.policy
                .worker_bytes
                .checked_add(self.policy.foreign_bytes)
                .ok_or(MathRuntimeError::Limit("worker allowance"))?,
        )?;
        let worker = case.assembly.worker(providers, cancel);
        Ok(ExecutionWorker {
            worker,
            _case: case,
            _lease: lease,
        })
    }
}

#[cfg(test)]
mod tests;
