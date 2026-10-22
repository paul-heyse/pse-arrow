// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Weighted owner-local physical products; native attempts never enter retention.
use super::{Operations, PhysicalContext, PhysicalSource, Runtime, WorkflowError, contract};
use crate::math::MathRuntimeError;
use crate::math::retention::WeightedCache;
use datafusion::{
    common::TableReference,
    execution::{
        cache::{CacheKey, CacheValue},
        memory_pool::{MemoryConsumer, MemoryReservation},
    },
};
use pse_engine::cache_service::{CacheComponent, CacheEntryReport, CacheReport};
use pse_model::HeapUsage;
use pse_operations::canonical::{ProtectedSelection, Revision};
#[cfg(test)]
use std::sync::Mutex;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

const RETAINED_BYTES: usize = 16 << 20;
const NAME: &str = "pse.cache.physical_admission";

/// Active immutable values retain their own protections beyond cache eviction.
#[derive(Debug)]
pub(super) struct PhysicalOwner {
    pins: Vec<ProtectedSelection>,
    _allocation: Arc<pse_columnar::AllocationLease>,
    retained_metadata: Option<Arc<pse_columnar::AllocationLease>>,
    pin_metadata_bytes: usize,
}
impl PhysicalOwner {
    fn allocation_bytes(&self) -> usize {
        self._allocation
            .size()
            .saturating_add(self.pin_metadata_bytes)
            .saturating_add(
                self.retained_metadata
                    .as_ref()
                    .map_or(0, |owner| owner.size()),
            )
    }
    fn allocation_without(&self, shared: &Arc<pse_columnar::AllocationLease>) -> usize {
        let bytes = self.allocation_bytes();
        if self
            .retained_metadata
            .as_ref()
            .is_some_and(|owner| Arc::ptr_eq(owner, shared))
        {
            bytes.saturating_sub(shared.size())
        } else {
            bytes
        }
    }
}
#[derive(Debug)]
struct Admission {
    source: PhysicalSource,
    operations: Operations,
    model: pse_engine::session::BoundModel,
    physical: Arc<PhysicalContext>,
    bytes: usize,
    _metadata: Arc<pse_columnar::AllocationLease>,
}
#[derive(Debug)]
pub(super) struct Rows {
    worker: Arc<str>,
    pool: Arc<dyn pse_columnar::MemoryPool>,
    model: pse_engine::session::BoundModel,
    physical: PhysicalContext,
    revisions: Vec<Revision>,
    bytes: usize,
    _metadata: Arc<pse_columnar::AllocationLease>,
}
impl Rows {
    pub(super) fn revisions(&self) -> &[Revision] {
        &self.revisions
    }
}
#[derive(Debug)]
pub(super) struct PhysicalRows {
    pub(super) revisions: Vec<Revision>,
    _owner: Arc<PhysicalOwner>,
}
pub(super) struct PhysicalCache {
    retained: WeightedCache<RetainedKey, RetainedProduct>,
    admission_fence: pse_columnar::retention::RetentionFence,
    rows_fence: pse_columnar::retention::RetentionFence,
    /// Only identical live source admissions share parsing and immutable physical loading.
    /// Native attempts and per-caller canonical protections remain outside the flight.
    admission_flights:
        pse_columnar::flight::Flights<PhysicalAdmissionKey, PhysicalContext, WorkflowError>,
    hits: AtomicUsize,
    misses: AtomicUsize,
    #[cfg(test)]
    admission_pause: Mutex<
        Option<(
            tokio::sync::oneshot::Sender<()>,
            tokio::sync::oneshot::Receiver<()>,
        )>,
    >,
    #[cfg(test)]
    admission_start_pause: Mutex<
        Option<(
            tokio::sync::oneshot::Sender<()>,
            tokio::sync::oneshot::Receiver<()>,
        )>,
    >,
    #[cfg(test)]
    admission_flight_pause: Mutex<
        Option<(
            tokio::sync::oneshot::Sender<()>,
            tokio::sync::oneshot::Receiver<()>,
        )>,
    >,
    #[cfg(test)]
    admission_flight_waiters: AtomicUsize,
    #[cfg(test)]
    admission_flight_loads: AtomicUsize,
}
impl std::fmt::Debug for PhysicalCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PhysicalCache")
            .field("entries", &self.retained.len())
            .field("retained_bytes", &self.retained.memory_used())
            .finish_non_exhaustive()
    }
}
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
enum RetainedKey {
    Admission(PhysicalAdmissionKey),
    // Candidate index only. The exact original checked sources and all owners
    // are compared before use; changed arrays cannot inherit this index's hit.
    Rows(PhysicalAdmissionKey, usize, usize),
}
impl CacheKey for RetainedKey {
    fn size(&self) -> usize {
        let owner = match self {
            Self::Admission(owner) | Self::Rows(owner, ..) => owner,
        };
        size_of::<Self>().saturating_add(owner.revision.capacity())
    }
    fn table_ref(&self) -> Option<&TableReference> {
        None
    }
}
#[derive(Clone)]
enum RetainedProduct {
    Admission(Arc<Admission>),
    Rows(Arc<Rows>),
}
impl CacheValue for RetainedProduct {
    fn size(&self) -> usize {
        match self {
            Self::Admission(entry) => entry.bytes.saturating_add(entry._metadata.size()),
            Self::Rows(entry) => entry.bytes.saturating_add(entry._metadata.size()),
        }
    }
}
fn row_key(runtime: &Runtime, operations: &Operations, physical: &PhysicalContext) -> RetainedKey {
    let source = PhysicalSource {
        revision: String::new(),
        identity: physical.identity(),
    };
    RetainedKey::Rows(
        PhysicalAdmissionKey::new(runtime, operations, &source, 0),
        Arc::as_ptr(&physical.quantities) as usize,
        Arc::as_ptr(&physical.preconditions) as usize,
    )
}
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct PhysicalAdmissionKey {
    revision: String,
    identity: pse_ids::ContentHash,
    worker: usize,
    pool: usize,
    registry: usize,
    sessions: usize,
    epoch: u64,
}
impl PhysicalAdmissionKey {
    fn new(
        runtime: &Runtime,
        operations: &Operations,
        source: &PhysicalSource,
        epoch: u64,
    ) -> Self {
        let (worker, pool) = operations.physical_owner_identity();
        Self {
            revision: source.revision.clone(),
            identity: source.identity,
            worker,
            pool,
            registry: Arc::as_ptr(runtime.registry()) as usize,
            sessions: Arc::as_ptr(runtime.sessions()) as usize,
            epoch,
        }
    }
}
impl Default for PhysicalCache {
    fn default() -> Self {
        Self {
            retained: WeightedCache::new(RETAINED_BYTES, NAME),
            admission_fence: pse_columnar::retention::RetentionFence::default(),
            rows_fence: pse_columnar::retention::RetentionFence::default(),
            admission_flights: pse_columnar::flight::Flights::new(32),
            hits: AtomicUsize::new(0),
            misses: AtomicUsize::new(0),
            #[cfg(test)]
            admission_pause: Mutex::default(),
            #[cfg(test)]
            admission_start_pause: Mutex::default(),
            #[cfg(test)]
            admission_flight_pause: Mutex::default(),
            #[cfg(test)]
            admission_flight_waiters: AtomicUsize::new(0),
            #[cfg(test)]
            admission_flight_loads: AtomicUsize::new(0),
        }
    }
}
impl PhysicalCache {
    pub(super) fn clear(&self) {
        self.admission_fence.clear(|| {
            self.rows_fence.clear(|| self.retained.clear());
        });
    }
    fn admission(
        &self,
        runtime: &Runtime,
        operations: &Operations,
        source: &PhysicalSource,
    ) -> Option<Arc<PhysicalContext>> {
        let key = RetainedKey::Admission(PhysicalAdmissionKey::new(runtime, operations, source, 0));
        let physical = self.retained.get(&key).and_then(|entry| match entry {
            RetainedProduct::Admission(entry)
                if entry.source == *source
                    && entry.operations.same_physical_owner(operations)
                    && Arc::ptr_eq(entry.model.registry(), runtime.registry())
                    && Arc::ptr_eq(entry.model.factory(), runtime.sessions()) =>
            {
                Some(entry.physical.clone())
            }
            _ => None,
        });
        if physical.is_some() {
            self.hits.fetch_add(1, Ordering::Relaxed);
        } else {
            self.misses.fetch_add(1, Ordering::Relaxed);
        }
        physical
    }
    pub(super) fn rows(
        &self,
        runtime: &Runtime,
        operations: &Operations,
        physical: &PhysicalContext,
    ) -> Option<Arc<Rows>> {
        let rows = self
            .retained
            .get(&row_key(runtime, operations, physical))
            .and_then(|entry| match entry {
                RetainedProduct::Rows(entry)
                    if (
                        Arc::as_ptr(&entry.worker) as *const () as usize,
                        Arc::as_ptr(&entry.pool) as *const () as usize,
                    ) == operations.physical_owner_identity()
                        && Arc::ptr_eq(entry.model.registry(), runtime.registry())
                        && Arc::ptr_eq(entry.model.factory(), runtime.sessions())
                        && entry.physical.same_sources(physical) =>
                {
                    Some(entry)
                }
                _ => None,
            });
        if rows.is_some() {
            self.hits.fetch_add(1, Ordering::Relaxed);
        } else {
            self.misses.fetch_add(1, Ordering::Relaxed);
        }
        rows
    }
    pub(super) fn begin_rows(&self) -> u64 {
        // Distinct sources may settle concurrently. Only explicit clear advances
        // the fence; a miss must not discard other exact immutable row receipts.
        self.rows_fence.generation()
    }
    pub(super) fn rows_generation(&self) -> u64 {
        self.rows_fence.generation()
    }
    pub(super) fn retain_rows(
        &self,
        generation: u64,
        runtime: &Runtime,
        operations: &Operations,
        physical: &PhysicalContext,
        rows: &PhysicalRows,
    ) {
        let revisions = &rows.revisions;
        let Some(receipts) = revision_copy_bytes(revisions) else {
            return;
        };
        // Retain reconstructible receipt payload and exact checked support only.
        // Per-caller protection/release grants stay with active PhysicalRows;
        // every warm return still establishes fresh native protection below.
        let bytes = physical
            .retained_bytes()
            .saturating_add(operations.worker().len());
        if bytes > RETAINED_BYTES {
            return;
        }
        let Some(extent) = physical
            .metadata_bytes()
            .checked_add(receipts)
            .and_then(|bytes| bytes.checked_add(size_of::<Rows>() + 1024))
        else {
            return;
        };
        let Some(metadata) = metadata(operations, extent) else {
            return;
        };
        let mut payload = physical.clone();
        payload._source_owner = None;
        let entry = Rows {
            worker: operations.physical_worker_owner(),
            pool: operations.pool.clone(),
            model: runtime.model.clone(),
            physical: payload,
            revisions: revisions.to_vec(),
            bytes,
            _metadata: metadata,
        };
        self.rows_fence.admit(generation, || {
            self.retained.put(
                row_key(runtime, operations, physical),
                RetainedProduct::Rows(Arc::new(entry)),
            )
        });
    }
    pub(super) async fn protect_rows(
        &self,
        operations: &Operations,
        revisions: &[Revision],
    ) -> Result<PhysicalRows, WorkflowError> {
        let bytes = revision_copy_bytes(revisions)
            .and_then(|bytes| bytes.checked_add(size_of::<PhysicalRows>()))
            .ok_or_else(|| contract("physical receipt allocation extent"))?;
        let owner = protect(operations, revisions, bytes).await?;
        Ok(PhysicalRows {
            revisions: revisions.to_vec(),
            _owner: owner,
        })
    }
    pub(super) async fn protected_rows(
        &self,
        generation: u64,
        runtime: &Runtime,
        operations: &Operations,
        physical: &PhysicalContext,
        revisions: &[Revision],
    ) -> Result<PhysicalRows, WorkflowError> {
        // Replace the cached pin with fresh native admission even on an exact hit.
        let rows = self.protect_rows(operations, revisions).await?;
        self.retain_rows(generation, runtime, operations, physical, &rows);
        Ok(rows)
    }
}

fn revision_copy_bytes(revisions: &[Revision]) -> Option<usize> {
    revisions.iter().try_fold(
        revisions.len().checked_mul(size_of::<Revision>())?,
        |bytes, revision| bytes.checked_add(revision.heap_bytes()),
    )
}

async fn protect_pin(
    operations: &Operations,
    revision: &Revision,
) -> Result<(ProtectedSelection, Arc<MemoryReservation>), WorkflowError> {
    use super::connected_results::{pooled_response_owner, retain_protected_metadata};
    use pse_operations::canonical::{MetadataResponse, response_ownership_scope};
    let response = pooled_response_owner(
        &operations.pool,
        "canonical:physical-protection-response",
        MetadataResponse::Header,
    )?;
    let pin = response_ownership_scope(
        response.clone(),
        operations
            .store()
            .protect(revision.clone(), std::time::Duration::from_secs(3600)),
    )
    .await?;
    let Some(bytes) = pin.retained_bytes() else {
        pin.retain_metadata_owner(response)?;
        return Err(contract("physical protection allocation extent"));
    };
    let retained = Arc::new(retain_protected_metadata(
        &operations.pool,
        response,
        bytes,
        |owner| pin.retain_metadata_owner(owner),
    )?);
    pin.retain_metadata_owner(retained.clone())?;
    Ok((pin, retained))
}

async fn protect(
    operations: &Operations,
    revisions: &[Revision],
    bytes: usize,
) -> Result<Arc<PhysicalOwner>, WorkflowError> {
    let allocation = MemoryConsumer::new(NAME).register(&operations.pool);
    let bytes = revisions
        .len()
        .checked_mul(size_of::<ProtectedSelection>())
        .and_then(|pins| pins.checked_add(size_of::<PhysicalOwner>() + 2 * size_of::<usize>()))
        .and_then(|pins| pins.checked_add(bytes))
        .ok_or_else(|| contract("physical protection vector extent"))?;
    allocation
        .try_grow(bytes)
        .map_err(pse_engine::EngineError::from)?;
    let mut owner = PhysicalOwner {
        pins: Vec::with_capacity(revisions.len()),
        _allocation: pse_columnar::AllocationLease::new(allocation),
        retained_metadata: None,
        pin_metadata_bytes: 0,
    };
    for revision in revisions {
        let (pin, metadata) = protect_pin(operations, revision).await?;
        owner.pin_metadata_bytes = owner
            .pin_metadata_bytes
            .checked_add(metadata.size())
            .ok_or_else(|| contract("physical retained protection extent"))?;
        if pin.revision() != revision {
            return Err(contract("physical receipt differs from protected revision"));
        }
        owner.pins.push(pin);
    }
    Ok(Arc::new(owner))
}
fn metadata(operations: &Operations, bytes: usize) -> Option<Arc<pse_columnar::AllocationLease>> {
    let allocation =
        MemoryConsumer::new("canonical:physical-cache-metadata").register(&operations.pool);
    allocation.try_grow(bytes).ok()?;
    Some(pse_columnar::AllocationLease::new(allocation))
}
impl Runtime {
    pub(super) async fn physical_source(
        &self,
        source: &PhysicalSource,
        cancel: &crate::CancelSource,
    ) -> Result<PhysicalContext, WorkflowError> {
        let activity = self.admit_activity()?;
        activity.scope(async {
        checkpoint(cancel)?;
        let operations = self.operations()?;
        // The same atomic fence snapshot identifies both the shared flight and
        // retention. Separate snapshots could join a pre-clear flight while
        // accidentally granting it a post-clear insertion generation.
        let generation = self.physical_cache.admission_fence.generation();
        if generation == u64::MAX {
            return Err(WorkflowError::Math(MathRuntimeError::Limit(
                "physical admission epoch exhausted",
            )));
        }
        let flight_key = PhysicalAdmissionKey::new(self, operations, source, generation);
        let cached = self.physical_cache.admission(self, operations, source);
        // Capture before the first asynchronous source/guard lookup. Distinct
        // exact sources coexist within one weighted budget; explicit clear alone
        // fences an old load and cannot become that load's new epoch.
        #[cfg(test)]
        {
            let pause = self
                .physical_cache
                .admission_start_pause
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .take();
            if let Some((ready, resume)) = pause {
                let _ = ready.send(());
                let _ = resume.await;
            }
        }
        let revision = operations
            .store()
            .revision(&source.revision)
            .await?
            .ok_or_else(|| contract("physical source revision absent"))?;
        if revision.problem != format!("physical:documents:{}", source.identity) {
            return Err(contract("physical document receipt interpretation differs"));
        }
        let mut protection = protect(
            operations,
            &[revision],
            1024 + cached
                .as_ref()
                .map_or(0, |physical| physical.sources.len() * 512),
        )
        .await?;
        checkpoint(cancel)?;
        if let Some(physical) = cached {
            if let Some(owner) = Arc::get_mut(&mut protection) {
                owner.retained_metadata = physical
                    ._source_owner
                    .as_ref()
                    .and_then(|owner| owner.retained_metadata.clone());
            }
            let mut physical = physical.as_ref().clone();
            physical._source_owner = Some(protection);
            return Ok(physical);
        }
        let load_runtime = self.clone();
        let load_operations = operations.clone();
        let load_source = source.clone();
        #[cfg(test)]
        let load_cache = self.physical_cache.clone();
        #[cfg(test)]
        self.physical_cache
            .admission_flight_waiters
            .fetch_add(1, Ordering::SeqCst);
        let mut flight = Box::pin(self.physical_cache.admission_flights.load_owned(
            flight_key,
            move |control| async move {
                #[cfg(test)]
                load_cache
                    .admission_flight_loads
                    .fetch_add(1, Ordering::SeqCst);
                #[cfg(test)]
                let pause = load_cache
                    .admission_flight_pause
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .take();
                #[cfg(test)]
                if let Some((ready, resume)) = pause {
                    let _ = ready.send(());
                    let _ = resume.await;
                }
                let shared_cancel = crate::CancelSource::new();
                let sources = tokio::select! {
                    result = load_operations.sources(&load_source) => result?,
                    () = control.cancelled() => {
                        shared_cancel.cancel();
                        return Err(WorkflowError::Math(MathRuntimeError::Cancelled));
                    }
                };
                let mut physical = tokio::select! {
                    result = load_runtime.physical_from_sources(&sources, &shared_cancel) => result?,
                    () = control.cancelled() => {
                        shared_cancel.cancel();
                        return Err(WorkflowError::Math(MathRuntimeError::Cancelled));
                    }
                };
                physical._source_owner = None;
                Ok(Arc::new(physical))
            },
        ));
        let admitted = tokio::select! {
            result = &mut flight => result.map_err(|error| match error {
                pse_columnar::flight::FlightError::Load(error) => WorkflowError::Shared(error),
                pse_columnar::flight::FlightError::Capacity => WorkflowError::Math(MathRuntimeError::Limit("physical source admissions")),
                pse_columnar::flight::FlightError::Retiring => WorkflowError::Math(MathRuntimeError::Retiring),
                pse_columnar::flight::FlightError::Panicked => WorkflowError::Math(MathRuntimeError::Panic("physical source admission task panic".into())),
            })?,
            () = cancel.cancelled() => return Err(WorkflowError::Math(MathRuntimeError::Cancelled)),
        };
        checkpoint(cancel)?;
        let mut physical = admitted.as_ref().clone();
        checkpoint(cancel)?;
        let bytes = physical.retained_bytes();
        let metadata = metadata(
            operations,
            physical.sources.len() * 512 + source.revision.len() + 1024,
        );
        if let Some(owner) = Arc::get_mut(&mut protection) {
            owner.retained_metadata = metadata.clone();
        }
        physical._source_owner = Some(protection);
        let bytes = bytes.saturating_add(physical._source_owner.as_ref().map_or(0,|owner| {
            metadata.as_ref().map_or_else(||owner.allocation_bytes(),|metadata|owner.allocation_without(metadata))
        }));
        #[cfg(test)]
        {
            let pause = self
                .physical_cache
                .admission_pause
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .take();
            if let Some((ready, resume)) = pause {
                let _ = ready.send(());
                let _ = resume.await;
            }
        }
        checkpoint(cancel)?;
        if let Some(metadata) = metadata.filter(|_| bytes <= RETAINED_BYTES) {
            let entry = Admission {
                source: source.clone(),
                operations: operations.clone(),
                model: self.model.clone(),
                physical: Arc::new(physical.clone()),
                bytes,
                _metadata: metadata,
            };
            self.physical_cache.admission_fence.admit(generation, || {
                self.physical_cache.retained.put(
                    RetainedKey::Admission(PhysicalAdmissionKey::new(self, operations, source, 0)),
                    RetainedProduct::Admission(Arc::new(entry)),
                )
            });
        }
        Ok(physical)

        }).await
    }
}

#[cfg(all(test, feature = "canonical-tests"))]
mod physical_admission_cache_unit {
    #![allow(
        clippy::unwrap_used,
        reason = "canonical cache fixtures require successful durable source admission before exercising owner reuse"
    )]
    use super::super::{Durability, durable_tests::durable_runtime, worker_tests::sources};
    use super::*;

    async fn fixture() -> (Runtime, PhysicalSource) {
        let runtime = durable_runtime();
        let (documents, _) = sources(
            "package p { def Root { param x:Scalar=3; annotation report x(\"constant\"); } }",
        );
        let source = runtime
            .operations()
            .unwrap()
            .put_sources(&documents)
            .await
            .unwrap();
        (runtime, source)
    }
    #[tokio::test]
    async fn canonical_physical_pin_extent_and_release_keep_actual_metadata_grant() {
        let (runtime, source) = fixture().await;
        let operations = runtime.operations().unwrap();
        let revision = operations
            .store()
            .revision(&source.revision)
            .await
            .unwrap()
            .unwrap();
        let before = operations.pool.reserved();
        let (pin, grant) = protect_pin(operations, &revision).await.unwrap();
        assert_eq!(
            grant.size(),
            pin.retained_bytes().unwrap()
                + pse_operations::canonical::MetadataResponse::Protection.allocation_bytes()
        );
        let held = Arc::downgrade(&grant);
        drop(grant);
        assert!(
            held.upgrade().is_some(),
            "actual protected owner retains its exact metadata/release grant"
        );
        drop(pin);
        pse_operations::testing::drain_result_releases(operations.store())
            .await
            .unwrap();
        assert!(
            held.upgrade().is_none(),
            "final native release drain relinquishes the attached grant"
        );
        assert_eq!(operations.pool.reserved(), before);
    }
    #[tokio::test]
    async fn canonical_physical_small_rows_warm_borrow_pressure_and_release_keep_full_weight() {
        let (runtime, source) = fixture().await;
        let operations = runtime.operations().unwrap();
        let mut physical = runtime
            .physical_source(&source, &crate::CancelSource::new())
            .await
            .unwrap();
        let selected = *physical
            .sources
            .iter()
            .find(|(_, table)| table.batch().num_rows() > 0)
            .unwrap()
            .0;
        // Recorded transport selection, not a compiled scientific context.
        physical.sources.retain(|key, _| *key == selected);
        let before = operations.pool.reserved();
        let first = operations
            .put_physical_rows(&runtime, &physical)
            .await
            .unwrap();
        let hits = runtime.physical_cache.hits.load(Ordering::Relaxed);
        let second = operations
            .put_physical_rows(&runtime, &physical)
            .await
            .unwrap();
        assert_eq!(first.revisions, second.revisions);
        assert_eq!(
            runtime.physical_cache.hits.load(Ordering::Relaxed),
            hits + 1
        );
        let held = runtime
            .physical_cache
            .rows(&runtime, operations, &physical)
            .unwrap();
        let again = runtime
            .physical_cache
            .rows(&runtime, operations, &physical)
            .unwrap();
        assert!(
            Arc::ptr_eq(&held, &again),
            "warm lookup borrows the charged retained receipt handle"
        );
        let weight = RetainedProduct::Rows(held.clone()).size();
        assert!(weight >= physical.retained_bytes());
        assert!(
            held.physical._source_owner.is_none(),
            "optional payload retention cannot own caller protection grants"
        );
        assert!(
            !Arc::ptr_eq(&first._owner, &second._owner),
            "every warm holder independently protects its exact receipts"
        );
        assert!(weight <= RETAINED_BYTES);
        let escaped = Arc::downgrade(&first._owner);
        let datafusion::execution::memory_pool::MemoryLimit::Finite(limit) =
            operations.pool.memory_limit()
        else {
            panic!("physical receipt pressure requires finite shared capacity");
        };
        let pressure =
            MemoryConsumer::new("test:physical-warm-receipt-pressure").register(&operations.pool);
        pressure
            .try_grow(limit - operations.pool.reserved() - 16)
            .unwrap();
        assert!(super::super::result_projection::projection_memory_refusal(
            &operations
                .put_physical_rows(&runtime, &physical)
                .await
                .unwrap_err()
        ));
        assert_eq!(
            held.revisions(),
            first.revisions.as_slice(),
            "refused fresh copy/protection preserves exact escaped receipts"
        );
        drop(pressure);
        let retried = operations
            .put_physical_rows(&runtime, &physical)
            .await
            .unwrap();
        assert_eq!(retried.revisions, first.revisions);
        drop(first);
        drop(second);
        drop(retried);
        drop(held);
        drop(again);
        runtime.clear_program_cache();
        assert!(escaped.upgrade().is_none());
        drop(physical);
        pse_operations::testing::drain_result_releases(operations.store())
            .await
            .unwrap();
        assert!(
            operations.pool.reserved() <= before,
            "row and source grants release after their final owners drain"
        );
    }
    #[tokio::test]
    async fn canonical_physical_admission_and_rows_reuse_exact_original_owners() {
        let (runtime, source) = fixture().await;
        let cancel = crate::CancelSource::new();
        let first = runtime.physical_source(&source, &cancel).await.unwrap();
        let second = runtime.physical_source(&source, &cancel).await.unwrap();
        assert!(first.same_sources(&second));
        assert!(
            !Arc::ptr_eq(
                first._source_owner.as_ref().unwrap(),
                second._source_owner.as_ref().unwrap()
            ),
            "each hit has a fresh native protection"
        );
        let operations = runtime.operations().unwrap();
        let first_rows = operations
            .put_physical_rows(&runtime, &first)
            .await
            .unwrap();
        let hits = runtime.physical_cache.hits.load(Ordering::Relaxed);
        let second_rows = operations
            .put_physical_rows(&runtime, &second)
            .await
            .unwrap();
        assert_eq!(first_rows.revisions, second_rows.revisions);
        assert_eq!(
            runtime.physical_cache.hits.load(Ordering::Relaxed),
            hits + 1,
            "reusable receipt payload fits independently of active protection/release grants"
        );
        assert!(first_rows._owner.pin_metadata_bytes > RETAINED_BYTES);
        assert!(
            runtime
                .physical_cache
                .rows(&runtime, operations, &first)
                .is_some()
        );
        let retained = runtime
            .physical_cache
            .rows(&runtime, operations, &first)
            .unwrap();
        assert!(retained.physical._source_owner.is_none());
        assert!(!Arc::ptr_eq(&first_rows._owner, &second_rows._owner));
        assert!(RetainedProduct::Rows(retained.clone()).size() < RETAINED_BYTES);
        eprintln!(
            "physical-granularity revisions={} active_grants={} retained_payload={}",
            first_rows.revisions.len(),
            first_rows._owner.allocation_bytes(),
            RetainedProduct::Rows(retained).size()
        );
        let mut changed = second.clone();
        let key = *changed
            .sources
            .iter()
            .find(|(_, table)| table.batch().num_rows() != 0)
            .unwrap()
            .0;
        let table = changed.sources.get(&key).unwrap();
        changed.sources.insert(key, table.slice(0, 0).unwrap());
        assert_eq!(first.identity(), changed.identity());
        assert!(
            !first.same_sources(&changed),
            "equal physical meaning never hides changed support rows"
        );
        assert!(
            runtime
                .physical_cache
                .rows(&runtime, operations, &changed)
                .is_none()
        );
        // Exercise actual cold/replayed publication after changing the exact row
        // support. This is a transport-owner control, not permission to compile
        // the intentionally altered physical context as a scientific model.
        let changed_rows = operations
            .put_physical_rows(&runtime, &changed)
            .await
            .unwrap();
        assert_ne!(first_rows.revisions, changed_rows.revisions);
        let changed_hits = runtime.physical_cache.hits.load(Ordering::Relaxed);
        let changed_again = operations
            .put_physical_rows(&runtime, &changed)
            .await
            .unwrap();
        assert_eq!(changed_rows.revisions, changed_again.revisions);
        assert_eq!(
            runtime.physical_cache.hits.load(Ordering::Relaxed),
            changed_hits + 1,
        );
        assert!(changed_rows._owner.pin_metadata_bytes > RETAINED_BYTES);
        assert!(
            runtime
                .physical_cache
                .rows(&runtime, operations, &changed)
                .is_some()
        );
        // The optional cache may replace an entry at the same coarse key.
        // Escaped receipts retain independent protection of their exact rows.
        assert_eq!(first_rows._owner.pins.len(), first_rows.revisions.len());
        for (pin, original) in first_rows._owner.pins.iter().zip(&first_rows.revisions) {
            assert_eq!(pin.revision(), original);
            pin.renew(std::time::Duration::from_secs(3600))
                .await
                .unwrap();
            assert_eq!(
                operations
                    .store()
                    .revision(&original.key)
                    .await
                    .unwrap()
                    .as_ref(),
                Some(original),
                "changed publication preserves the independently protected original revision",
            );
        }
        let mut package_changed = second.clone();
        package_changed
            .package
            .as_mut()
            .unwrap()
            .name
            .push_str("changed");
        assert!(
            runtime
                .physical_cache
                .rows(&runtime, operations, &package_changed)
                .is_none()
        );
        runtime.clear_program_cache();
        assert_eq!(runtime.physical_cache.report()[0].entries, 0);
        assert_eq!(
            first_rows.revisions, second_rows.revisions,
            "active receipt consumers survive eviction"
        );
        assert!(first.same_sources(&second));
    }
    #[tokio::test]
    async fn canonical_physical_final_owners_register_cleanup_before_fixture_drain() {
        let (runtime, source) = fixture().await;
        let physical = runtime
            .physical_source(&source, &crate::CancelSource::new())
            .await
            .unwrap();
        let operations = runtime.operations().unwrap();
        let rows = operations
            .put_physical_rows(&runtime, &physical)
            .await
            .unwrap();
        let alias = physical.clone();
        drop(physical);
        runtime.clear_program_cache();
        assert!(operations.store().remove_isolated_fixture().await.is_err());
        drop(alias);
        assert!(
            operations.store().remove_isolated_fixture().await.is_err(),
            "escaped row receipts retain their own protection after cache eviction"
        );
        drop(rows);
        operations.store().remove_isolated_fixture().await.unwrap();
    }
    #[tokio::test]
    async fn canonical_physical_admission_coalesces_same_source_and_localizes_cancellation() {
        const WAITERS: usize = 16;
        let (runtime, source) = fixture().await;
        let (ready, waiting) = tokio::sync::oneshot::channel();
        let (resume, paused) = tokio::sync::oneshot::channel();
        *runtime
            .physical_cache
            .admission_flight_pause
            .lock()
            .unwrap() = Some((ready, paused));
        let leader_cancel = crate::CancelSource::new();
        let leader_runtime = runtime.clone();
        let leader_source = source.clone();
        let leader_token = leader_cancel.clone();
        let leader = tokio::spawn(async move {
            leader_runtime
                .physical_source(&leader_source, &leader_token)
                .await
        });
        waiting.await.unwrap();

        let mut followers = Vec::with_capacity(WAITERS - 1);
        for _ in 1..WAITERS {
            let follower_runtime = runtime.clone();
            let follower_source = source.clone();
            followers.push(tokio::spawn(async move {
                follower_runtime
                    .physical_source(&follower_source, &crate::CancelSource::new())
                    .await
            }));
        }
        tokio::time::timeout(std::time::Duration::from_secs(60), async {
            while runtime
                .physical_cache
                .admission_flight_waiters
                .load(Ordering::SeqCst)
                < WAITERS
            {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();

        leader_cancel.cancel();
        resume.send(()).unwrap();
        assert!(matches!(
            leader.await.unwrap(),
            Err(WorkflowError::Math(MathRuntimeError::Cancelled))
        ));
        let mut admissions = Vec::with_capacity(WAITERS - 1);
        for follower in followers {
            admissions.push(follower.await.unwrap().unwrap());
        }
        assert_eq!(
            runtime
                .physical_cache
                .admission_flight_loads
                .load(Ordering::SeqCst),
            1,
            "identical immutable source preparation has one flight owner"
        );
        assert!(
            admissions
                .iter()
                .all(|admission| admission.same_sources(&admissions[0]))
        );
        for (index, admission) in admissions.iter().enumerate() {
            for other in &admissions[index + 1..] {
                assert!(
                    !Arc::ptr_eq(
                        admission._source_owner.as_ref().unwrap(),
                        other._source_owner.as_ref().unwrap()
                    ),
                    "each waiter keeps its own fresh canonical protection"
                );
            }
        }
        assert_eq!(runtime.physical_cache.admission_flights.active(), 0);
    }
    #[tokio::test]
    async fn canonical_physical_admission_clear_fences_actual_inflight_load() {
        let (runtime, source) = fixture().await;
        let (ready, waiting) = tokio::sync::oneshot::channel();
        let (resume, paused) = tokio::sync::oneshot::channel();
        *runtime
            .physical_cache
            .admission_flight_pause
            .lock()
            .unwrap() = Some((ready, paused));
        let worker = runtime.clone();
        let requested = source.clone();
        let pending = tokio::spawn(async move {
            worker
                .physical_source(&requested, &crate::CancelSource::new())
                .await
                .unwrap()
        });
        waiting.await.unwrap();
        runtime.clear_program_cache();
        let fresh = runtime
            .physical_source(&source, &crate::CancelSource::new())
            .await
            .unwrap();
        assert_eq!(runtime.physical_cache.admission_flights.active(), 1);
        resume.send(()).unwrap();
        let active = pending.await.unwrap();
        assert_eq!(runtime.physical_cache.report()[0].entries, 1);
        assert!(
            !Arc::ptr_eq(&active.quantities, &fresh.quantities),
            "post-clear caller loads under a distinct in-flight epoch"
        );
        assert_eq!(active.identity(), fresh.identity());
        assert!(
            runtime
                .physical_cache
                .admission(&runtime, runtime.operations().unwrap(), &source)
                .is_some_and(|cached| Arc::ptr_eq(&cached.quantities, &fresh.quantities)),
            "pre-clear flight cannot overwrite the fresh admitted cache entry"
        );
    }
    #[tokio::test]
    async fn canonical_physical_admission_clear_before_first_source_await_fences_load() {
        let (runtime, source) = fixture().await;
        let (ready, waiting) = tokio::sync::oneshot::channel();
        let (resume, paused) = tokio::sync::oneshot::channel();
        *runtime.physical_cache.admission_start_pause.lock().unwrap() = Some((ready, paused));
        let worker = runtime.clone();
        let pending = tokio::spawn(async move {
            worker
                .physical_source(&source, &crate::CancelSource::new())
                .await
                .unwrap()
        });
        waiting.await.unwrap();
        runtime.clear_program_cache();
        resume.send(()).unwrap();
        let active = pending.await.unwrap();
        assert!(!active.sources.is_empty());
        assert_eq!(
            runtime.physical_cache.report()[0].entries,
            0,
            "clear before native guard awaits must fence the original load"
        );
    }
    #[tokio::test]
    async fn canonical_physical_admission_owner_revision_and_retirement_fences() {
        let (runtime, source) = fixture().await;
        let cancel = crate::CancelSource::new();
        let physical = runtime.physical_source(&source, &cancel).await.unwrap();
        let operations = runtime.operations().unwrap();
        let mut other = runtime.clone();
        other.model = Arc::new(
            runtime
                .shared
                .session_factory(pse_engine::session::native_engine_profile())
                .unwrap(),
        )
        .bind_model(runtime.registry().clone())
        .unwrap();
        assert!(
            runtime
                .physical_cache
                .admission(&other, operations, &source)
                .is_none()
        );
        other.model = runtime
            .sessions()
            .bind_model(Arc::new(pse_schema::catalog::assemble().unwrap()))
            .unwrap();
        assert!(
            runtime
                .physical_cache
                .admission(&other, operations, &source)
                .is_none()
        );
        let new_operations = Operations::from_store(
            operations.store().clone(),
            "other-owner",
            operations.policy(),
            runtime.shared.pool(),
        );
        assert!(
            runtime
                .physical_cache
                .admission(&runtime, &new_operations, &source)
                .is_none()
        );
        other.durability = Durability::Durable(Box::new(new_operations));
        let other_store = pse_operations::testing::canonical_fixture_store().unwrap();
        let foreign = Operations::from_store(
            other_store.clone(),
            operations.worker(),
            operations.policy(),
            runtime.shared.pool(),
        );
        assert!(
            runtime
                .physical_cache
                .admission(&runtime, &foreign, &source)
                .is_none()
        );
        other_store.remove_isolated_fixture().await.unwrap();
        let mut wrong = source.clone();
        wrong.revision.push_str("absent");
        assert!(
            runtime
                .physical_cache
                .admission(&runtime, operations, &wrong)
                .is_none()
        );
        assert!(runtime.physical_source(&wrong, &cancel).await.is_err());
        let restored = runtime.physical_source(&source, &cancel).await.unwrap();
        let stopped = crate::CancelSource::new();
        stopped.cancel();
        assert!(runtime.physical_source(&source, &stopped).await.is_err());
        let revision = operations
            .store()
            .revision(&source.revision)
            .await
            .unwrap()
            .unwrap();
        operations
            .store()
            .edit(
                &revision.problem,
                Some(&revision.key),
                "retire-physical-manifest",
                &[super::super::durable::source_edit(
                    "manifest".into(),
                    "physical:documents".into(),
                    "manifest".into(),
                    "physical:documents:manifest:v1",
                    b"{}".to_vec(),
                )],
            )
            .await
            .unwrap();
        operations.store().forget_history(&revision).await.unwrap();
        // Fixture: end the cached protection while retaining its stale immutable value.
        // This exercises the fresh server check, rather than clearing away the cache hit.
        for context in [&physical, &restored] {
            for pin in &context._source_owner.as_ref().unwrap().pins {
                operations.store().release(pin).await.unwrap();
            }
        }
        let page = operations
            .store()
            .reclaim_source_page(&revision.problem)
            .await
            .unwrap();
        assert!(page.memberships > 0);
        assert!(
            runtime
                .physical_cache
                .admission(&runtime, operations, &source)
                .is_some()
        );
        assert!(
            runtime.physical_source(&source, &cancel).await.is_err(),
            "reclaimed revision cannot be recovered from an in-memory admission"
        );
    }
    #[tokio::test]
    async fn canonical_physical_admission_interleaved_exact_revisions_keep_distinct_entries() {
        let (runtime, source) = fixture().await;
        let cancel = crate::CancelSource::new();
        let first = runtime.physical_source(&source, &cancel).await.unwrap();
        let operations = runtime.operations().unwrap();
        let revision = operations
            .store()
            .revision(&source.revision)
            .await
            .unwrap()
            .unwrap();
        let next = operations
            .store()
            .edit(
                &revision.problem,
                Some(&revision.key),
                "equivalent-physical-revision",
                &[],
            )
            .await
            .unwrap();
        let revised = PhysicalSource {
            revision: next.key,
            identity: source.identity,
        };
        let second = runtime.physical_source(&revised, &cancel).await.unwrap();
        assert_eq!(first.identity(), second.identity());
        assert!(
            !first.same_sources(&second),
            "changed exact revision re-admits original source rows even when physical meaning is equal"
        );
        assert!(
            runtime
                .physical_cache
                .admission(&runtime, operations, &source)
                .is_some()
        );
        let third = runtime.physical_source(&source, &cancel).await.unwrap();
        assert!(
            first.same_sources(&third),
            "alternating exact sources reuse the original immutable product"
        );
        assert!(
            !Arc::ptr_eq(
                first._source_owner.as_ref().unwrap(),
                third._source_owner.as_ref().unwrap()
            ),
            "every cache hit still establishes fresh protection"
        );
        assert_eq!(
            runtime.physical_cache.report()[0].entries,
            2,
            "distinct exact revisions coexist under the weighted allowance"
        );
        let first_rows = operations
            .put_physical_rows(&runtime, &first)
            .await
            .unwrap();
        let second_rows = operations
            .put_physical_rows(&runtime, &second)
            .await
            .unwrap();
        assert_eq!(
            runtime
                .physical_cache
                .rows(&runtime, operations, &first)
                .as_ref()
                .map(|rows| rows.revisions()),
            Some(first_rows.revisions.as_slice()),
            "granular retained receipts coexist without retaining independent caller protection grants"
        );
        assert_eq!(
            runtime
                .physical_cache
                .rows(&runtime, operations, &second)
                .as_ref()
                .map(|rows| rows.revisions()),
            Some(second_rows.revisions.as_slice())
        );
        assert!(first_rows._owner.pin_metadata_bytes > RETAINED_BYTES);
        assert!(second_rows._owner.pin_metadata_bytes > RETAINED_BYTES);
        // A smaller recorded transport selection supplies the positive rows
        // coexistence control; it is not compiled as scientific physical input.
        let selected = *first
            .sources
            .iter()
            .find(|(_, table)| table.batch().num_rows() > 0)
            .unwrap()
            .0;
        let mut small_first = first.clone();
        small_first.sources.retain(|key, _| *key == selected);
        let mut small_second = second.clone();
        small_second.sources.retain(|key, _| *key == selected);
        let small_first_rows = operations
            .put_physical_rows(&runtime, &small_first)
            .await
            .unwrap();
        let small_second_rows = operations
            .put_physical_rows(&runtime, &small_second)
            .await
            .unwrap();
        assert_eq!(
            runtime
                .physical_cache
                .rows(&runtime, operations, &small_first)
                .unwrap()
                .revisions(),
            small_first_rows.revisions.as_slice()
        );
        assert_eq!(
            runtime
                .physical_cache
                .rows(&runtime, operations, &small_second)
                .unwrap()
                .revisions(),
            small_second_rows.revisions.as_slice()
        );
        let retained = runtime.physical_cache.report()[0].retained_bytes;
        runtime
            .physical_cache
            .retained
            .update_cache_limit(retained / 2);
        assert!(runtime.physical_cache.report()[0].retained_bytes <= retained / 2);
        assert!(
            !first_rows.revisions.is_empty(),
            "pressure cannot invalidate escaped protected receipts"
        );
        assert!(
            !second.sources.is_empty(),
            "pressure cannot invalidate active physical products"
        );
        runtime.clear_program_cache();
        assert_eq!(runtime.physical_cache.report()[0].entries, 0);
        assert!(
            !first.sources.is_empty(),
            "clear preserves the old active immutable admission"
        );
    }
}
fn checkpoint(cancel: &crate::CancelSource) -> Result<(), WorkflowError> {
    if cancel.token().is_cancelled() {
        Err(MathRuntimeError::Cancelled.into())
    } else {
        Ok(())
    }
}
impl CacheComponent for PhysicalCache {
    fn invalidate(&self) {
        self.clear();
    }
    fn report(&self) -> Vec<CacheReport> {
        vec![CacheReport {
            name: NAME.into(),
            capacity_bytes: 0,
            policy_limit_bytes: self.retained.cache_limit(),
            retained_bytes: self.retained.memory_used(),
            entries: self.retained.len(),
            hits: self.hits.load(Ordering::Relaxed),
            misses: self.misses.load(Ordering::Relaxed),
            bypasses: 0,
            live_bytes: None,
            pinned_bytes: None,
            inflight_bytes: None,
            active_loads: None,
            evictions: None,
        }]
    }
    fn details(
        &self,
        rows: &mut Vec<CacheEntryReport>,
        limit: usize,
        owner: &MemoryReservation,
        budget: usize,
    ) -> datafusion::common::Result<()> {
        for report in self.report() {
            if report.entries > 0 && rows.len() < limit {
                pse_engine::cache_service::details::reserve_inventory(owner, 256, budget)?;
                rows.push(CacheEntryReport {
                    cache: report.name,
                    key: "exact physical owners".into(),
                    bytes: report.retained_bytes,
                    hits: report.hits,
                });
            }
        }
        Ok(())
    }
    fn execution_report(&self) -> Vec<(&'static str, Option<usize>)> {
        Vec::new()
    }
}
