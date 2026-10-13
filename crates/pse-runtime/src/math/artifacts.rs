// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! DataFusion retention with entry-owned reservations and invalidation epochs.
use super::{MathRuntimeError, MathService};
use datafusion::{
    common::TableReference,
    execution::cache::{Cache, CacheKey, CacheValue},
};
use pse_columnar::flight::FlightError;
use pse_compiler::workspace::ArtifactRequest;
use pse_engine::cache_service::{CacheComponent, CacheEntryReport, CacheReport};
use pse_ids::ContentHash;
use pse_math::guarded::CompiledBody;
use std::sync::{Arc, atomic::Ordering};
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) struct Key(pub ContentHash, pub bool);
impl CacheKey for Key {
    fn size(&self) -> usize {
        128
    }
    fn table_ref(&self) -> Option<&TableReference> {
        None
    }
}
/// Immutable program and the reservation of what it retains. Clones retain accounting.
#[derive(Debug)]
pub struct Artifact {
    pub(super) program: Arc<CompiledBody>,
    pub(super) lease: Arc<ProgramOwner>,
}
#[derive(Debug)]
pub(super) struct ProgramOwner {
    pub(super) lease: Arc<pse_columnar::AllocationLease>,
    pub(super) live: Arc<std::sync::atomic::AtomicUsize>,
}
impl ProgramOwner {
    pub(super) fn size(&self) -> usize {
        self.lease.size()
    }
}
impl Drop for ProgramOwner {
    fn drop(&mut self) {
        self.live.fetch_sub(self.lease.size(), Ordering::AcqRel);
    }
}
#[derive(Clone)]
pub(super) enum Value {
    Artifact(Arc<Artifact>),
    Scalars(Arc<pse_math::library::ScalarProducts>),
}
impl CacheValue for Value {
    fn size(&self) -> usize {
        match self { Self::Artifact(artifact) => artifact.lease.size().saturating_add(artifact.program.scalar_products().retained_bytes()), Self::Scalars(products) => products.retained_bytes() }
    }
}
impl MathService {
    /// Clear retained programs without cancelling live owners or permitting late reinsertion.
    pub fn clear_program_cache(&self) {
        self.trim_compiler_lanes();
        self.pressure.clear();
    }
    async fn scalar_products(self: &Arc<Self>, request: ArtifactRequest) -> Result<Arc<pse_math::library::ScalarProducts>, MathRuntimeError> {
        let key = Key(request.scalar_key(), true);
        if let Some(Value::Scalars(products)) = self.entries.get(&key) { return Ok(products); }
        let service = self.clone();
        let epoch = self.retention.generation();
        let population = super::PopulationScope::capture();
        self.scalar_flights.load_owned(key.clone(), move |cancel| population.run(async move {
            if let Some(Value::Scalars(products)) = service.entries.get(&key) { return Ok(products); }
            let construction = super::jobs::ConstructionEstimate::from_bound(request.construction_allocation_bound()?, service.policy.worker_bytes);
            let check = cancel.flag();
            let (products, lease) = service.job_retained_construction(request.cores(), construction, cancel, None, move |flag| {
                let products = request.build_scalars(&flag).map_err(MathRuntimeError::Math)?;
                let retained = products.retained_bytes();
                Ok((products, retained))
            }).await?;
            if check.load(Ordering::Acquire) { return Err(MathRuntimeError::Cancelled); }
            service.live.fetch_add(lease.size(), Ordering::AcqRel);
            let owner = Arc::new(ProgramOwner { lease, live: service.live.clone() });
            let products = Arc::new(products.with_owner(owner));
            service.pressure.publish_artifact(epoch, &key, Value::Scalars(products.clone()));
            Ok(products)
        })).await.map_err(|error| match error {
            FlightError::Load(error) => MathRuntimeError::Shared(error),
            FlightError::Capacity => MathRuntimeError::Limit("scalar flights"),
            FlightError::Retiring => MathRuntimeError::Retiring,
            FlightError::Panicked => MathRuntimeError::Panic("scalar task panic".into()),
        })
    }
    /// Obtain a compiler-issued artifact; callers cannot supply an independent cache key.
    pub async fn artifact(
        self: &Arc<Self>,
        request: ArtifactRequest,
    ) -> Result<Arc<Artifact>, MathRuntimeError> {
        // Admission applies even to a cache hit under this runtime's current policy.
        if request.cores() == 0
            || request.cores() > self.cores
            || request.scratch_limit() > self.policy.worker_bytes
        {
            return Err(MathRuntimeError::Limit("artifact profile"));
        }
        let key = Key(request.key(), false);
        if let Some(Value::Artifact(v)) = self.entries.get(&key) {
            self.hits.fetch_add(1, Ordering::Relaxed);
            return Ok(v);
        }
        self.misses.fetch_add(1, Ordering::Relaxed);
        let service = self.clone();
        let flight_key = key.clone();
        let epoch = self.retention.generation();
        let population = super::PopulationScope::capture();
        self.flights
            .load_owned(flight_key, move |cancel| {
                population.run(async move {
                    if let Some(Value::Artifact(v)) = service.entries.get(&key) {
                        return Ok(v);
                    }
                    let scalars = service.scalar_products(request.clone()).await?;
                    let cores = request.cores();
                    let construction = super::jobs::ConstructionEstimate::from_bound(
                        request.construction_allocation_bound()?,
                        service.policy.worker_bytes,
                    );
                    let demand = construction.bytes();
                    if demand > service.policy.worker_bytes {
                        return Err(MathRuntimeError::Limit("artifact construction capacity"));
                    }
                    let check = cancel.flag();
                    let (program, lease) = service
                        .job_retained_construction(cores, construction, cancel, None, move |flag| {
                            let program = request.build_reusing(&flag, &scalars).map_err(MathRuntimeError::Math)?;
                            let retained = program.retained_bytes();
                            Ok((program, retained))
                        })
                        .await?;
                    if check.load(Ordering::Acquire) {
                        return Err(MathRuntimeError::Cancelled);
                    }
                    service.live.fetch_add(lease.size(), Ordering::AcqRel);
                    let lease = Arc::new(ProgramOwner {
                        lease,
                        live: service.live.clone(),
                    });
                    let artifact = Arc::new(Artifact {
                        program: Arc::new(program.with_owner(lease.clone())),
                        lease,
                    });
                    service
                        .pressure
                        .publish_artifact(epoch, &key, Value::Artifact(artifact.clone()));
                    Ok(artifact)
                })
            })
            .await
            .map_err(|e| match e {
                FlightError::Load(e) => MathRuntimeError::Shared(e),
                FlightError::Capacity => MathRuntimeError::Limit("artifact flights"),
                FlightError::Retiring => MathRuntimeError::Retiring,
                FlightError::Panicked => MathRuntimeError::Panic("artifact task panic".into()),
            })
    }
}
impl CacheComponent for MathService {
    fn storage_bound(&self) -> bool {
        false
    }
    fn invalidate(&self) {
        self.clear_program_cache();
    }
    fn report(&self) -> Vec<CacheReport> {
        vec![
            self.modeling_cache.report(
                self.selected_flights.active()
                    + self.frontier_flights.active()
                    + self.basis_flights.active(),
            ),
            CacheReport {
                name: self.entries.name(),
                capacity_bytes: 0,
                retained_bytes: self.entries.memory_used(),
                policy_limit_bytes: self.entries.cache_limit(),
                live_bytes: Some(self.live.load(Ordering::Acquire)),
                pinned_bytes: None,
                inflight_bytes: None,
                active_loads: Some(self.flights.active() + self.scalar_flights.active()),
                evictions: None,
                entries: self.entries.len(),
                hits: self.hits.load(Ordering::Relaxed),
                misses: self.misses.load(Ordering::Relaxed),
                bypasses: 0,
            },
        ]
    }
    fn details(
        &self,
        rows: &mut Vec<CacheEntryReport>,
        limit: usize,
        owner: &datafusion::execution::memory_pool::MemoryReservation,
        budget: usize,
    ) -> datafusion::common::Result<()> {
        pse_engine::cache_service::details::reserve_inventory(
            owner,
            self.entries.memory_used(),
            budget,
        )?;
        for (key, e) in self
            .entries
            .list_entries()
            .into_iter()
            .take(limit.saturating_sub(rows.len()))
        {
            rows.push(CacheEntryReport {
                cache: self.entries.name(),
                key: key.0.to_string(),
                bytes: e.size_bytes,
                hits: e.hits,
            });
        }
        Ok(())
    }
    fn execution_report(&self) -> Vec<(&'static str, Option<usize>)> {
        vec![
            (
                "math_native_active",
                Some(self.policy.jobs - self.active_jobs.available_permits()),
            ),
            (
                "math_native_population",
                Some(self.policy.jobs - self.jobs.available_permits()),
            ),
            (
                "math_foreign_allowance_per_job",
                Some(self.policy.foreign_bytes),
            ),
            ("math_stack_per_job", Some(self.policy.stack_bytes)),
            (
                "math_inner_sessions_per_job",
                Some(self.policy.inner_session_bytes),
            ),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test(flavor = "current_thread")]
    async fn scalar_family_flights_share_preparation_and_eviction_keeps_borrowed_leases() {
        let (service, _cache) = super::super::tests::service_with_policy(256 << 20, super::super::MathPolicy::default());
        let (preparation, _, _) = super::super::tests::component_fixture(&service);
        let request = preparation.compiled().artifacts[0].clone();
        let (first, second) = tokio::join!(service.scalar_products(request.clone()), service.scalar_products(request.clone()));
        let first = first.unwrap();
        let second = second.unwrap();
        assert!(Arc::ptr_eq(&first, &second));
        assert!(!first.is_empty());
        let scalar_key = Key(request.scalar_key(), true);
        assert!(matches!(service.entries.get(&scalar_key), Some(Value::Scalars(_))));
        let artifact = service.artifact(request).await.unwrap();
        let worker = artifact.program.worker();
        let charged = service.live.load(Ordering::Acquire);
        assert!(charged >= first.retained_bytes());
        service.clear_program_cache();
        assert_eq!(service.entries.len(), 0);
        drop(first);
        drop(second);
        drop(artifact);
        assert!(service.live.load(Ordering::Acquire) > 0, "worker borrows both scalar and program charges");
        drop(worker);
        assert_eq!(service.live.load(Ordering::Acquire), 0);
    }
}
