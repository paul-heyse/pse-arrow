// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Service-scoped immutable product retention; allocation owners live in products.
use super::*;
use datafusion::{
    common::TableReference,
    execution::cache::{Cache, CacheKey, CacheValue},
};
use pse_compiler::workspace::ModelingBodyRetention;
use pse_ids::roles::{AdmittedClosureHash, PreparedViewHash, SemanticBodyHash};
use std::sync::atomic::Ordering;
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum Key {
    Package(AdmittedClosureHash),
    Selected(Arc<crate::workflow::modeling::SelectedRequest>),
    Body(SemanticBodyHash),
    EncodedBody(SemanticBodyHash),
    Basis(Arc<modeling::BasisKey>),
    Frontier(Arc<modeling::BasisKey>),
    Solver(PreparedViewHash),
    Observation(PreparedViewHash),
    Parametric(PreparedViewHash),
}
impl CacheKey for Key {
    fn size(&self) -> usize {
        size_of::<Self>()
            + 128
            + match self {
                Self::Selected(request) => request.retained_bytes(),
                Self::Basis(key) | Self::Frontier(key) => key.retained_bytes(),
                _ => 0,
            }
    }
    fn table_ref(&self) -> Option<&TableReference> {
        None
    }
}
#[derive(Clone)]
enum Product {
    Package(crate::workflow::PackageAdmission),
    Selected(Arc<Vec<Arc<crate::workflow::modeling::SelectedAdmission>>>),
    Body(Arc<pse_compiler::typed_math::AdmittedBody>),
    EncodedBody(Arc<portable::EncodedBody>),
    Basis(Arc<modeling::PreparedBasis>),
    Frontier(Arc<modeling::OwnedModelingFrontier>),
    Solver(Preparation),
    Program(Arc<ExecutableCase>),
}
impl CacheValue for Product {
    fn size(&self) -> usize {
        // Retention counts reachable payload, and positive entry overhead, independently
        // from unique allocation leases already held by the shared product owners.
        size_of::<Self>()
            + 128
            + match self {
                Self::Package(p) => p.retained_bytes(),
                Self::Selected(values) => {
                    values.capacity()
                        * size_of::<Arc<crate::workflow::modeling::SelectedAdmission>>()
                        + values.iter().map(|p| p.retained_bytes()).sum::<usize>()
                }
                Self::Body(body) => body.math().retained_bytes() + body.descriptor_bytes(),
                Self::EncodedBody(body) => body.retained_bytes(),
                Self::Basis(basis) => basis.retained_bytes(),
                Self::Frontier(frontier) => frontier.retained_bytes(),
                Self::Solver(p) => p.compiled().retained_bytes(),
                Self::Program(p) => p.assembly.retained_bytes(),
            }
    }
}
/// One service-local barrier coordinates pressure, cache publication and native LRU trim.
/// Lock order is this state, then the cache's clear fence, then the native cache lock.
/// Neither pool admission nor an asynchronous wait runs under this barrier.
pub(super) struct Pressure {
    demands: Mutex<usize>,
    artifacts: Arc<DefaultCache<artifacts::Key, Value>>,
    modeling: Arc<DefaultCache<Key, Product>>,
    artifact_fence: Arc<pse_columnar::retention::RetentionFence>,
    modeling_fence: Arc<pse_columnar::retention::RetentionFence>,
    limit: usize,
}
impl std::fmt::Debug for Pressure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Pressure")
            .field("limit", &self.limit)
            .finish_non_exhaustive()
    }
}
pub(super) struct PressureDemand(Arc<Pressure>);
impl Drop for PressureDemand {
    fn drop(&mut self) {
        let mut demands = self
            .0
            .demands
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *demands -= 1;
        if *demands == 0 {
            self.0
                .artifact_fence
                .admit(self.0.artifact_fence.generation(), || {
                    self.0.artifacts.update_cache_limit(self.0.limit)
                });
            self.0
                .modeling_fence
                .admit(self.0.modeling_fence.generation(), || {
                    self.0.modeling.update_cache_limit(self.0.limit)
                });
        }
    }
}
impl Pressure {
    pub(super) fn new(limit: usize) -> Arc<Self> {
        Arc::new(Self {
            demands: Mutex::new(0),
            artifacts: Arc::new(DefaultCache::new(limit).with_name("pse.cache.math_artifacts")),
            modeling: Arc::new(DefaultCache::new(limit).with_name("pse.cache.modeling_products")),
            artifact_fence: Arc::default(),
            modeling_fence: Arc::default(),
            limit,
        })
    }
    pub(super) fn artifacts(&self) -> Arc<DefaultCache<artifacts::Key, Value>> {
        self.artifacts.clone()
    }
    pub(super) fn artifact_fence(&self) -> Arc<pse_columnar::retention::RetentionFence> {
        self.artifact_fence.clone()
    }
    pub(super) fn demand(self: &Arc<Self>) -> PressureDemand {
        let mut demands = self
            .demands
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *demands += 1;
        PressureDemand(self.clone())
    }
    fn active(&self) -> bool {
        *self
            .demands
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            != 0
    }
    /// Reduce reachable retention by the failed growth, then let actual pool admission
    /// determine whether enough unique capacity was released. A final pass uses zero.
    fn trim(&self, growth: usize, all: bool) {
        let _demands = self
            .demands
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let artifacts = self
            .artifact_fence
            .admit(self.artifact_fence.generation(), || {
                let retained = self.artifacts.memory_used();
                let limit = if all {
                    0
                } else {
                    retained.saturating_sub(growth)
                };
                self.artifacts
                    .update_cache_limit(limit.min(self.artifacts.cache_limit()));
                retained
            })
            .unwrap_or(0);
        let remaining = growth.saturating_sub(artifacts);
        self.modeling_fence
            .admit(self.modeling_fence.generation(), || {
                let retained = self.modeling.memory_used();
                let limit = if all {
                    0
                } else {
                    retained.saturating_sub(remaining)
                };
                self.modeling
                    .update_cache_limit(limit.min(self.modeling.cache_limit()));
            });
    }
    pub(super) fn publish_artifact(&self, generation: u64, key: &artifacts::Key, value: Value) {
        let demands = self
            .demands
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if *demands == 0 {
            self.artifact_fence
                .admit(generation, || self.artifacts.put(key, value));
        }
    }
    pub(super) fn clear(&self) {
        let _demands = self
            .demands
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        self.artifact_fence.clear(|| self.artifacts.clear());
        self.modeling_fence.clear(|| self.modeling.clear());
    }
    /// Retry the same reservation after bounded retention reduction. The caller owns
    /// the demand across retries/waits, and gets an unchanged typed pool failure.
    pub(super) fn retry_growth(
        &self,
        reservation: &datafusion::execution::memory_pool::MemoryReservation,
        bytes: usize,
    ) -> datafusion::common::Result<()> {
        self.trim(bytes, false);
        match reservation.try_grow(bytes) {
            Ok(()) => Ok(()),
            Err(datafusion::common::DataFusionError::ResourcesExhausted(_)) => {
                self.trim(bytes, true);
                reservation.try_grow(bytes)
            }
            Err(error) => Err(error),
        }
    }
    pub(super) fn try_grow(
        self: &Arc<Self>,
        reservation: &datafusion::execution::memory_pool::MemoryReservation,
        bytes: usize,
    ) -> datafusion::common::Result<()> {
        match reservation.try_grow(bytes) {
            Ok(()) => Ok(()),
            Err(datafusion::common::DataFusionError::ResourcesExhausted(_)) => {
                let _demand = self.demand();
                self.retry_growth(reservation, bytes)
            }
            Err(error) => Err(error),
        }
    }
}
pub(crate) struct ModelingCache {
    entries: Arc<DefaultCache<Key, Product>>,
    fence: Arc<pse_columnar::retention::RetentionFence>,
    pressure: Arc<Pressure>,
    selected_updates: Mutex<()>,
    hits: AtomicUsize,
    misses: AtomicUsize,
    bypasses: AtomicUsize,
}
impl ModelingCache {
    pub(super) fn new(pressure: Arc<Pressure>) -> Self {
        Self {
            entries: pressure.modeling.clone(),
            fence: pressure.modeling_fence.clone(),
            pressure,
            selected_updates: Mutex::default(),
            hits: Default::default(),
            misses: Default::default(),
            bypasses: Default::default(),
        }
    }
    fn get(&self, key: &Key) -> Option<Product> {
        let value = self.entries.get(key);
        if value.is_some() {
            self.hits.fetch_add(1, Ordering::Relaxed);
        } else {
            self.misses.fetch_add(1, Ordering::Relaxed);
        }
        value
    }
    fn put(&self, generation: u64, key: Key, value: Product) {
        let demands = self
            .pressure
            .demands
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if *demands != 0 {
            self.bypasses.fetch_add(1, Ordering::Relaxed);
            return;
        }
        if self
            .fence
            .admit(generation, || {
                if key.size().saturating_add(value.size()) > self.entries.cache_limit() {
                    self.bypasses.fetch_add(1, Ordering::Relaxed);
                }
                self.entries.put(&key, value)
            })
            .is_none()
        {
            self.bypasses.fetch_add(1, Ordering::Relaxed);
        }
    }
    pub(crate) fn selected(
        &self,
        request: &Arc<crate::workflow::modeling::SelectedRequest>,
    ) -> Option<Arc<Vec<Arc<crate::workflow::modeling::SelectedAdmission>>>> {
        match self.get(&Key::Selected(request.clone()))? {
            Product::Selected(values) => Some(values),
            _ => None,
        }
    }
    pub(crate) fn retain_selected(
        &self,
        generation: u64,
        admission: Arc<crate::workflow::modeling::SelectedAdmission>,
    ) {
        // Avoid allocating the optional candidate vector while a requester needs capacity.
        // Actual publication rechecks the barrier after this early optimization.
        if self.pressure.active() {
            self.bypasses.fetch_add(1, Ordering::Relaxed);
            return;
        }
        // A/B/A candidates share the existing byte-bounded owner. Serialize replacement
        // so concurrent eligible products do not discard each other's candidates.
        let _update = self
            .selected_updates
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let key = Key::Selected(admission.request.clone());
        let mut values = match self.entries.get(&key) {
            Some(Product::Selected(values)) => values.as_ref().clone(),
            _ => Vec::new(),
        };
        if values
            .iter()
            .any(|old| old.dependencies == admission.dependencies)
        {
            return;
        }
        values.push(admission);
        loop {
            let product = Product::Selected(Arc::new(values));
            if key.size().saturating_add(product.size()) <= self.entries.cache_limit() {
                self.put(generation, key, product);
                break;
            }
            let owned = match product {
                Product::Selected(owned) => owned,
                _ => return,
            };
            values = Arc::unwrap_or_clone(owned);
            if values.len() == 1 {
                self.bypasses.fetch_add(1, Ordering::Relaxed);
                break;
            }
            values.remove(0);
        }
    }
    pub(crate) fn generation(&self) -> u64 {
        self.fence.generation()
    }
    /// Pure sealed-body lookup; receiving-host eligibility is checked by the consumer.
    pub(crate) fn body(
        &self,
        key: SemanticBodyHash,
    ) -> Option<Arc<pse_compiler::typed_math::AdmittedBody>> {
        match self.get(&Key::Body(key))? {
            Product::Body(body) => Some(body),
            _ => None,
        }
    }
    /// Exact immutable portable encoding under the compiler-sealed semantic body identity.
    /// Current producer qualification and canonical settlement are separate consumer effects.
    pub(super) fn encoded_body(&self, key: SemanticBodyHash) -> Option<Arc<portable::EncodedBody>> {
        match self.get(&Key::EncodedBody(key))? {
            Product::EncodedBody(body) => Some(body),
            _ => None,
        }
    }
    pub(super) fn retain_encoded_body(
        &self,
        generation: u64,
        key: SemanticBodyHash,
        body: Arc<portable::EncodedBody>,
    ) {
        self.put(
            generation,
            Key::EncodedBody(key),
            Product::EncodedBody(body),
        );
    }
    /// A completed immutable preparation basis shares the same finite cache owner.
    pub(crate) fn frontier(
        &self,
        key: &Arc<modeling::BasisKey>,
    ) -> Option<Arc<modeling::OwnedModelingFrontier>> {
        match self.get(&Key::Frontier(key.clone()))? {
            Product::Frontier(frontier) => Some(frontier),
            _ => None,
        }
    }
    pub(crate) fn retain_frontier(
        &self,
        generation: u64,
        key: Arc<modeling::BasisKey>,
        frontier: Arc<modeling::OwnedModelingFrontier>,
    ) {
        self.put(generation, Key::Frontier(key), Product::Frontier(frontier));
    }
    pub(crate) fn basis(
        &self,
        key: &Arc<modeling::BasisKey>,
    ) -> Option<Arc<modeling::PreparedBasis>> {
        match self.get(&Key::Basis(key.clone()))? {
            Product::Basis(basis) => Some(basis),
            _ => None,
        }
    }
    pub(crate) fn retain_basis(
        &self,
        generation: u64,
        key: Arc<modeling::BasisKey>,
        basis: Arc<modeling::PreparedBasis>,
    ) {
        self.put(generation, Key::Basis(key), Product::Basis(basis));
    }
    pub(crate) fn solver(&self, key: PreparedViewHash) -> Option<Preparation> {
        match self.get(&Key::Solver(key))? {
            Product::Solver(p) => Some(p),
            _ => None,
        }
    }
    pub(crate) fn retain_solver(&self, generation: u64, key: PreparedViewHash, p: Preparation) {
        self.put(generation, Key::Solver(key), Product::Solver(p));
    }
    pub(crate) fn program(
        &self,
        key: PreparedViewHash,
        parametric: bool,
    ) -> Option<Arc<ExecutableCase>> {
        match self.get(&if parametric {
            Key::Parametric(key)
        } else {
            Key::Observation(key)
        })? {
            Product::Program(p) => Some(p),
            _ => None,
        }
    }
    pub(crate) fn retain_program(
        &self,
        generation: u64,
        key: PreparedViewHash,
        p: Arc<ExecutableCase>,
        parametric: bool,
    ) {
        self.put(
            generation,
            if parametric {
                Key::Parametric(key)
            } else {
                Key::Observation(key)
            },
            Product::Program(p),
        );
    }
    pub(crate) fn package(
        &self,
        key: AdmittedClosureHash,
    ) -> Option<crate::workflow::PackageAdmission> {
        match self.get(&Key::Package(key))? {
            Product::Package(p) => Some(p),
            _ => None,
        }
    }
    pub(crate) fn retain_package(
        &self,
        generation: u64,
        key: AdmittedClosureHash,
        p: crate::workflow::PackageAdmission,
    ) {
        self.put(generation, Key::Package(key), Product::Package(p));
    }
    pub(super) fn report(&self, active_loads: usize) -> pse_engine::cache_service::CacheReport {
        pse_engine::cache_service::CacheReport {
            name: self.entries.name(),
            capacity_bytes: 0,
            retained_bytes: self.entries.memory_used(),
            policy_limit_bytes: self.entries.cache_limit(),
            live_bytes: None,
            pinned_bytes: None,
            inflight_bytes: None,
            active_loads: Some(active_loads),
            evictions: None,
            entries: self.entries.len(),
            hits: self.hits.load(Ordering::Relaxed),
            misses: self.misses.load(Ordering::Relaxed),
            bypasses: self.bypasses.load(Ordering::Relaxed),
        }
    }
}

/// A workspace's weak attachment keeps the service cache out of reference cycles.
#[derive(Debug)]
pub(super) struct BodyRetention(pub std::sync::Weak<MathService>);
// Cancellation can unwind admission. Published entries are immutable; lease transfer,
// cache insertion and ownership tables use their own guarded operations and RAII.
impl std::panic::RefUnwindSafe for BodyRetention {}
impl ModelingBodyRetention for BodyRetention {
    fn generation(&self) -> u64 {
        self.0
            .upgrade()
            .map_or(u64::MAX, |service| service.modeling_cache.generation())
    }
    fn get(
        &self,
        key: SemanticBodyHash,
    ) -> Result<Option<Arc<pse_compiler::typed_math::AdmittedBody>>, pse_math::MathError> {
        let Some(service) = self.0.upgrade() else {
            return Ok(None);
        };
        Ok(service.modeling_cache.body(key))
    }
    fn retain(
        &self,
        generation: u64,
        key: SemanticBodyHash,
        body: Arc<pse_compiler::typed_math::AdmittedBody>,
    ) -> Result<Arc<pse_compiler::typed_math::AdmittedBody>, pse_math::MathError> {
        // Only compiler-sealed immutable mathematics enters this pure owner. Producer
        // identities, receiving qualification and publication never partition its meaning.
        if body.semantic_identity() != Some(key) {
            return Err(pse_math::MathError::Contract(
                "body retention requires its exact compiler-sealed semantic identity".into(),
            ));
        }
        let Some(service) = self.0.upgrade() else {
            return Ok(body);
        };
        let owned =
            service
                .own_semantic_body(body)
                .map_err(|error| pse_math::MathError::Typed {
                    retained: size_of::<MathRuntimeError>(),
                    cause: pse_model::diagnostic::DiagnosticCause::new(error),
                })?;
        service
            .modeling_cache
            .put(generation, Key::Body(key), Product::Body(owned.clone()));
        Ok(owned)
    }
}

impl MathService {
    /// Transfer an explicitly received sealed body through the same pure retention
    /// and allocation owner used by compiler preparation. A clear fences late fills.
    pub(crate) fn retain_received_body(
        self: &Arc<Self>,
        generation: u64,
        key: SemanticBodyHash,
        body: Arc<pse_compiler::typed_math::AdmittedBody>,
    ) -> Result<Arc<pse_compiler::typed_math::AdmittedBody>, MathRuntimeError> {
        BodyRetention(Arc::downgrade(self))
            .retain(generation, key, body)
            .map_err(MathRuntimeError::from)
    }
}

#[cfg(test)]
mod pressure_tests {
    use super::*;
    use crate::math::{artifacts::Key as ArtifactKey, tests};
    use datafusion::execution::memory_pool::MemoryConsumer;
    fn key(value: u8) -> ArtifactKey {
        ArtifactKey(pse_ids::ContentHash::from_bytes([value; 32]))
    }
    fn retained_artifact(service: &Arc<MathService>, value: u8) -> Arc<Artifact> {
        let (preparation, _, _) = tests::component_fixture(service);
        let program = preparation.compiled().artifacts[0]
            .build(&Arc::default())
            .unwrap();
        let lease = service
            .reserve(
                "test:optional-program",
                program.retained_bytes() + (8 << 20),
            )
            .unwrap();
        service.live.fetch_add(lease.size(), Ordering::AcqRel);
        let owner = Arc::new(artifacts::ProgramOwner {
            lease,
            live: service.live.clone(),
        });
        let artifact = Arc::new(Artifact {
            program: Arc::new(program.with_owner(owner.clone())),
            lease: owner,
        });
        service.pressure.publish_artifact(
            service.retention.generation(),
            &key(value),
            Value(artifact.clone()),
        );
        artifact
    }
    #[test]
    fn synchronous_pressure_trims_both_families_without_invalidating_live_aliases() {
        let (service, _) = tests::service_with_policy(
            64 << 20,
            MathPolicy {
                artifact_bytes: 32 << 20,
                ..Default::default()
            },
        );
        let artifact = retained_artifact(&service, 1);
        let (model, _, _) = tests::component_fixture(&service);
        let model_key = PreparedViewHash::from_id(pse_ids::ContentHash::from_bytes([2; 32]));
        service
            .modeling_cache
            .retain_solver(service.modeling_cache.generation(), model_key, model);
        let artifact_epoch = service.retention.generation();
        let model_epoch = service.modeling_cache.generation();
        let baseline = service.pool.reserved();
        let pressure = MemoryConsumer::new("test:genuine-live").register(&service.pool);
        pressure.try_grow((64 << 20) - baseline).unwrap();
        // The cache references can yield; the escaped artifact still pins its actual lease.
        assert!(
            service
                .reserve("test:live-alias-refusal", 16 << 20)
                .is_err()
        );
        assert_eq!(service.entries.len(), 0);
        assert_eq!(service.modeling_cache.entries.len(), 0);
        assert_eq!(service.retention.generation(), artifact_epoch);
        assert_eq!(service.modeling_cache.generation(), model_epoch);
        assert!(artifact.lease.size() > 8 << 20);
        assert_eq!(service.entries.cache_limit(), service.policy.artifact_bytes);
        drop(artifact);
        let admitted = service.reserve("test:after-alias", 8 << 20).unwrap();
        drop(admitted);
        drop(pressure);
        assert_eq!(service.pool.reserved(), 0);
    }
    #[tokio::test(flavor = "current_thread")]
    async fn pressure_demands_share_refill_barrier_until_last_waiter_finishes() {
        let (service, _) = tests::service_with_policy(
            64 << 20,
            MathPolicy {
                artifact_bytes: 32 << 20,
                ..Default::default()
            },
        );
        let artifact = retained_artifact(&service, 3);
        let (model, _, _) = tests::component_fixture(&service);
        let model_key = PreparedViewHash::from_id(pse_ids::ContentHash::from_bytes([9; 32]));
        let retained = service.pool.reserved();
        let live = MemoryConsumer::new("test:live-saturation").register(&service.pool);
        live.try_grow((64 << 20) - retained).unwrap();
        let deadline = Some(std::time::Instant::now() + std::time::Duration::from_secs(30));
        let first_cancel = FlightCancellation::default();
        let second_cancel = FlightCancellation::default();
        let first = service.reserve_entry("test:first-waiter", 16 << 20, &first_cancel, deadline);
        let second =
            service.reserve_entry("test:second-waiter", 16 << 20, &second_cancel, deadline);
        tokio::pin!(first, second);
        assert!(futures_util::poll!(&mut first).is_pending());
        assert!(futures_util::poll!(&mut second).is_pending());
        assert_eq!(*service.pressure.demands.lock().unwrap(), 2);
        assert_eq!(service.cpu.available_permits(), 2);
        let artifact_epoch = service.retention.generation();
        service
            .pressure
            .publish_artifact(artifact_epoch, &key(4), Value(artifact.clone()));
        assert_eq!(service.entries.len(), 0);
        service.modeling_cache.retain_solver(
            service.modeling_cache.generation(),
            model_key,
            model.clone(),
        );
        assert_eq!(service.modeling_cache.entries.len(), 0);
        first_cancel.cancel();
        assert!(matches!(first.await, Err(MathRuntimeError::Cancelled)));
        assert_eq!(*service.pressure.demands.lock().unwrap(), 1);
        assert_eq!(service.entries.cache_limit(), 0);
        service
            .pressure
            .publish_artifact(artifact_epoch, &key(5), Value(artifact.clone()));
        assert_eq!(service.entries.len(), 0);
        // Broadcasting without freeing anything must not count as capacity.
        service.released.notify_waiters();
        assert!(futures_util::poll!(&mut second).is_pending());
        // Free before the next await: the enabled notification must preserve the wakeup.
        drop(live);
        drop(artifact);
        drop(model);
        let admitted = second.await.unwrap();
        assert_eq!(*service.pressure.demands.lock().unwrap(), 0);
        assert_eq!(service.entries.cache_limit(), service.policy.artifact_bytes);
        assert_eq!(
            service.modeling_cache.entries.cache_limit(),
            service.policy.artifact_bytes
        );
        drop(admitted);
        assert_eq!(service.pool.reserved(), 0);
    }
    #[tokio::test]
    async fn disposable_retention_yields_to_same_job_and_worker_reservations() {
        let (service, _) = tests::service_with_policy(
            64 << 20,
            MathPolicy {
                artifact_bytes: 32 << 20,
                foreign_bytes: 1 << 20,
                stack_bytes: 1 << 20,
                inner_session_bytes: 1 << 20,
                ..Default::default()
            },
        );
        drop(retained_artifact(&service, 6));
        let baseline = service.pool.reserved();
        let live = MemoryConsumer::new("test:remaining-live").register(&service.pool);
        live.try_grow((64 << 20) - baseline).unwrap();
        let (_, result) = service
            .job_retained(1, 0, FlightCancellation::default(), |_| Ok(((), 0)))
            .await
            .unwrap();
        assert_eq!(service.entries.len(), 0);
        assert_eq!(service.jobs.available_permits(), service.policy.jobs);
        assert_eq!(service.cpu.available_permits(), 2);
        drop(result);
        drop(live);
        drop(retained_artifact(&service, 7));
        let baseline = service.pool.reserved();
        let live = MemoryConsumer::new("test:worker-live").register(&service.pool);
        live.try_grow((64 << 20) - baseline).unwrap();
        assert!(WorkerBudget::drawing(0, &service.pool).charge(1).is_err());
        let budget = WorkerBudget::drawing_for(16 << 20, &service);
        let worker = budget.charge(8 << 20).unwrap();
        assert_eq!(budget.used(), 8 << 20);
        assert!(budget.charge(16 << 20).is_err());
        assert_eq!(budget.used(), 8 << 20);
        drop(worker);
        assert_eq!(budget.used(), 0);
        drop(budget);
        drop(live);
        assert_eq!(service.pool.reserved(), 0);
    }
}
