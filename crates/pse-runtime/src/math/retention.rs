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
pub(crate) struct ModelingCache {
    entries: DefaultCache<Key, Product>,
    fence: pse_columnar::retention::RetentionFence,
    selected_updates: Mutex<()>,
    hits: AtomicUsize,
    misses: AtomicUsize,
    bypasses: AtomicUsize,
}
impl ModelingCache {
    pub(super) fn new(bytes: usize) -> Self {
        Self {
            entries: DefaultCache::new(bytes).with_name("pse.cache.modeling_products"),
            fence: Default::default(),
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
    pub(super) fn clear(&self) {
        self.fence.clear(|| self.entries.clear());
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
