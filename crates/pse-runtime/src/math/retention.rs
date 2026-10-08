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
    Body(SemanticBodyHash, Option<String>),
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
                Self::Body(_, producer) => producer.as_ref().map_or(0, String::capacity),
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
impl BodyRetention {
    // Clone the namespaced candidate before entering the loader scope. Cache locks
    // must never be acquired from inside that scope; the canonical attachment
    // checks current admission before exposing a candidate for mathematical use.
    pub(super) fn get_for_producer(
        &self,
        key: SemanticBodyHash,
        producer: Option<&portable::ReplayAdmission>,
    ) -> Result<Option<Arc<pse_compiler::typed_math::AdmittedBody>>, pse_math::MathError> {
        let Some(service) = self.0.upgrade() else {
            return Ok(None);
        };
        Ok(
            match service.modeling_cache.get(&Key::Body(
                key,
                producer.map(portable::ReplayAdmission::key),
            )) {
                Some(Product::Body(body)) => Some(body),
                _ => None,
            },
        )
    }
    pub(super) fn retain_for_producer(
        &self,
        generation: u64,
        key: SemanticBodyHash,
        body: Arc<pse_compiler::typed_math::AdmittedBody>,
        producer: Option<&portable::ReplayAdmission>,
    ) -> Result<Arc<pse_compiler::typed_math::AdmittedBody>, pse_math::MathError> {
        let Some(service) = self.0.upgrade() else {
            return Ok(body);
        };
        // Tag this fill only while its exact admission is current. Commit no cache
        // side effects until the scope's before/after validation has succeeded.
        // A stale admission can still retain fresh mathematical output, but only
        // in the ordinary unqualified namespace, never under its historical key.
        let (body, namespace) = producer
            .and_then(|producer| producer.with_current(|| (body.clone(), producer.key())))
            .map_or_else(|| (body, None), |(body, key)| (body, Some(key)));
        let owned =
            service
                .own_semantic_body(body)
                .map_err(|error| pse_math::MathError::Typed {
                    retained: size_of::<MathRuntimeError>(),
                    cause: pse_model::diagnostic::DiagnosticCause::new(error),
                })?;
        service.modeling_cache.put(
            generation,
            Key::Body(key, namespace),
            Product::Body(owned.clone()),
        );
        Ok(owned)
    }
}
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
        self.get_for_producer(key, None)
    }
    fn retain(
        &self,
        generation: u64,
        key: SemanticBodyHash,
        body: Arc<pse_compiler::typed_math::AdmittedBody>,
    ) -> Result<Arc<pse_compiler::typed_math::AdmittedBody>, pse_math::MathError> {
        self.retain_for_producer(generation, key, body, None)
    }
}
/// Protected selected-demand attachment; immutable math alone enters memory retention.
#[derive(Debug)]
pub(super) struct CanonicalBodyRetention {
    pub(super) memory: BodyRetention,
    pub(super) store: Arc<pse_operations::canonical::CanonicalStore>,
    pub(super) read: Arc<Mutex<pse_operations::canonical_selection::SelectedRead>>,
    pub(super) producer: Option<portable::ReplayAdmission>,
    pub(super) outer_build: pse_ids::ContentHash,
    pub(super) inputs: CompilerContext,
    pub(super) cancelled: Arc<std::sync::atomic::AtomicBool>,
    pub(super) handle: tokio::runtime::Handle,
}
impl std::panic::RefUnwindSafe for CanonicalBodyRetention {}
fn portable_error(error: portable::PortableError) -> pse_math::MathError {
    match error {
        portable::PortableError::Math(error) => error,
        error => {
            // Keep the storage owner's exact typed identity and facts in an owned,
            // accounted envelope rather than classifying its rendered native message.
            let diagnostic = pse_model::diagnostic::project_typed(
                &error,
                pse_diagnostics::DiagnosticStage::ModelingAdmission,
            );
            let retained = size_of::<pse_model::diagnostic::BoundaryDiagnostic>()
                + pse_model::HeapUsage::heap_bytes(&diagnostic);
            pse_math::MathError::Typed {
                retained,
                cause: pse_model::diagnostic::DiagnosticCause::new(diagnostic),
            }
        }
    }
}
async fn canonical_request<T>(
    cancelled: &std::sync::atomic::AtomicBool,
    timeout: std::time::Duration,
    operation: impl Future<Output = Result<T, portable::PortableError>>,
) -> Result<T, pse_math::MathError> {
    let cancellation = async {
        loop {
            if cancelled.load(Ordering::Relaxed) {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
    };
    tokio::pin!(operation);
    tokio::select! {
        biased;
        ()=cancellation=>Err(pse_math::MathError::Cancelled),
        ()=tokio::time::sleep(timeout)=>Err(portable_error(portable::PortableError::Store(pse_operations::canonical::CanonicalError::Timeout))),
        result=&mut operation=>result.map_err(portable_error),
    }
}
impl ModelingBodyRetention for CanonicalBodyRetention {
    fn generation(&self) -> u64 {
        self.memory.generation()
    }
    fn get(
        &self,
        key: SemanticBodyHash,
    ) -> Result<Option<Arc<pse_compiler::typed_math::AdmittedBody>>, pse_math::MathError> {
        if self.cancelled.load(Ordering::Relaxed) {
            return Err(pse_math::MathError::Cancelled);
        }
        // A cache probe only clones an immutable candidate; no mathematical use
        // escapes until admission is checked. Empty probes need no loader scope.
        if let Some(candidate) = self.memory.get_for_producer(key, self.producer.as_ref())? {
            let body = match &self.producer {
                Some(admission) => {
                    let Some(body) = admission.with_current(|| candidate) else {
                        return Ok(None);
                    };
                    body
                }
                None => candidate,
            };
            // A memory hit still makes this revision's durable reachability concrete.
            self.publish(&body)?;
            return Ok(Some(body));
        }
        let Some(producer) = &self.producer else {
            return Ok(None);
        };
        let service = self.memory.0.upgrade().ok_or_else(|| {
            pse_math::MathError::Contract("canonical replay memory owner is unavailable".into())
        })?;
        let mut read = self.read.lock().map_err(|_| {
            pse_math::MathError::Contract("canonical selection lock poisoned".into())
        })?;
        let body = self.handle.block_on(canonical_request(
            &self.cancelled,
            pse_operations::canonical::REQUEST_TIMEOUT,
            portable::reuse_body(
                &service,
                &self.store,
                &mut read,
                producer,
                key,
                &self.inputs,
                &self.cancelled,
            ),
        ))?;
        drop(read);
        body.map(|body| {
            self.memory.retain_for_producer(
                self.generation(),
                key,
                Arc::new(body),
                self.producer.as_ref(),
            )
        })
        .transpose()
    }
    fn retain(
        &self,
        generation: u64,
        key: SemanticBodyHash,
        body: Arc<pse_compiler::typed_math::AdmittedBody>,
    ) -> Result<Arc<pse_compiler::typed_math::AdmittedBody>, pse_math::MathError> {
        let body =
            self.memory
                .retain_for_producer(generation, key, body, self.producer.as_ref())?;
        self.publish(&body)?;
        Ok(body)
    }
}
impl CanonicalBodyRetention {
    fn publish(
        &self,
        body: &pse_compiler::typed_math::AdmittedBody,
    ) -> Result<(), pse_math::MathError> {
        if self.cancelled.load(Ordering::Relaxed) {
            return Err(pse_math::MathError::Cancelled);
        }
        let service = self.memory.0.upgrade().ok_or_else(|| {
            pse_math::MathError::Contract(
                "canonical publication memory owner is unavailable".into(),
            )
        })?;
        let read = self.read.lock().map_err(|_| {
            pse_math::MathError::Contract("canonical selection lock poisoned".into())
        })?;
        self.handle.block_on(canonical_request(
            &self.cancelled,
            pse_operations::canonical::REQUEST_TIMEOUT,
            async {
                match &self.producer {
                    Some(producer) if producer.is_current() => {
                        portable::publish_body(
                            &service,
                            &self.store,
                            &read,
                            producer,
                            body,
                            &self.inputs,
                        )
                        .await
                    }
                    _ => {
                        portable::publish_unqualified_body(
                            &service,
                            &self.store,
                            &read,
                            self.outer_build,
                            body,
                            &self.inputs,
                        )
                        .await
                    }
                }
            },
        ))?;
        Ok(())
    }
}
#[cfg(test)]
mod canonical_portable_body_callback_tests {
    use super::*;
    use std::sync::atomic::AtomicBool;
    use std::time::Duration;
    struct DropProbe(Arc<AtomicBool>);
    impl Drop for DropProbe {
        fn drop(&mut self) {
            self.0.store(true, Ordering::Relaxed);
        }
    }
    #[tokio::test]
    async fn canonical_portable_body_callback_cancellation_drops_pending_operation() {
        let cancelled = Arc::new(AtomicBool::new(false));
        let dropped = Arc::new(AtomicBool::new(false));
        let probe = DropProbe(dropped.clone());
        let operation = async move {
            let _probe = probe;
            std::future::pending::<Result<(), portable::PortableError>>().await
        };
        let trigger = cancelled.clone();
        let cancel = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(15)).await;
            trigger.store(true, Ordering::Relaxed);
        });
        assert!(matches!(
            canonical_request(&cancelled, Duration::from_secs(1), operation).await,
            Err(pse_math::MathError::Cancelled)
        ));
        cancel.await.unwrap();
        assert!(dropped.load(Ordering::Relaxed));
    }
    #[tokio::test]
    async fn canonical_portable_body_callback_deadline_refuses_and_drops_pending_operation() {
        use pse_diagnostics::{DiagnosticCode, TypedDiagnostic};
        let cancelled = AtomicBool::new(false);
        let dropped = Arc::new(AtomicBool::new(false));
        let probe = DropProbe(dropped.clone());
        let operation = async move {
            let _probe = probe;
            std::future::pending::<Result<(), portable::PortableError>>().await
        };
        let error = canonical_request(&cancelled, Duration::from_millis(15), operation)
            .await
            .unwrap_err();
        assert_eq!(
            error.diagnostic_code(),
            Some(DiagnosticCode::RuntimeInfrastructure)
        );
        assert!(dropped.load(Ordering::Relaxed));
    }
    #[tokio::test]
    async fn canonical_portable_body_callback_preexisting_cancellation_prevents_poll() {
        let cancelled = AtomicBool::new(true);
        let polled = AtomicBool::new(false);
        let operation = async {
            polled.store(true, Ordering::Relaxed);
            Ok::<(), portable::PortableError>(())
        };
        assert!(matches!(
            canonical_request(&cancelled, Duration::from_secs(1), operation).await,
            Err(pse_math::MathError::Cancelled)
        ));
        assert!(!polled.load(Ordering::Relaxed));
    }
}
