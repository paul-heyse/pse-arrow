// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Service-scoped immutable product retention; allocation owners live in products.
use super::*;
use datafusion::{
    common::TableReference,
    execution::cache::{Cache, CacheKey, CacheValue},
};
use pse_ids::roles::{AdmittedClosureHash, PreparedViewHash, SemanticBodyHash};
use std::sync::atomic::Ordering;
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum Key {
    Package(AdmittedClosureHash),
    Body(SemanticBodyHash),
    Solver(PreparedViewHash),
    Observation(PreparedViewHash),
    Parametric(PreparedViewHash),
}
impl CacheKey for Key {
    fn size(&self) -> usize {
        size_of::<Self>() + 128
    }
    fn table_ref(&self) -> Option<&TableReference> {
        None
    }
}
#[derive(Clone)]
enum Product {
    Package(crate::workflow::PackageAdmission),
    Body(Arc<pse_compiler::typed_math::AdmittedBody>),
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
                Self::Body(body) => body.math().retained_bytes() + body.descriptor_bytes(),
                Self::Solver(p) => p.compiled().retained_bytes(),
                Self::Program(p) => p.assembly.retained_bytes(),
            }
    }
}
pub(crate) struct ModelingCache {
    entries: DefaultCache<Key, Product>,
    fence: pse_columnar::retention::RetentionFence,
    hits: AtomicUsize,
    misses: AtomicUsize,
    bypasses: AtomicUsize,
}
impl ModelingCache {
    pub(super) fn new(bytes: usize) -> Self {
        Self {
            entries: DefaultCache::new(bytes).with_name("pse.cache.modeling_products"),
            fence: Default::default(),
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
    pub(crate) fn generation(&self) -> u64 {
        self.fence.generation()
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
    pub(super) fn report(&self) -> pse_engine::cache_service::CacheReport {
        pse_engine::cache_service::CacheReport {
            name: self.entries.name(),
            capacity_bytes: 0,
            retained_bytes: self.entries.memory_used(),
            policy_limit_bytes: self.entries.cache_limit(),
            live_bytes: None,
            pinned_bytes: None,
            inflight_bytes: None,
            active_loads: None,
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
impl pse_compiler::workspace::ModelingBodyRetention for BodyRetention {
    fn generation(&self) -> u64 {
        self.0
            .upgrade()
            .map_or(u64::MAX, |service| service.modeling_cache.generation())
    }
    fn get(&self, key: SemanticBodyHash) -> Option<Arc<pse_compiler::typed_math::AdmittedBody>> {
        let service = self.0.upgrade()?;
        match service.modeling_cache.get(&Key::Body(key))? {
            Product::Body(body) => Some(body),
            _ => None,
        }
    }
    fn retain(
        &self,
        generation: u64,
        key: SemanticBodyHash,
        body: Arc<pse_compiler::typed_math::AdmittedBody>,
    ) -> Result<Arc<pse_compiler::typed_math::AdmittedBody>, pse_math::MathError> {
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
