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
pub(super) enum Key {
    Artifact(ContentHash),
    Scalars(ContentHash),
}
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
        match self {
            Self::Artifact(artifact) => artifact
                .lease
                .size()
                .saturating_add(artifact.program.scalar_products().retained_bytes()),
            Self::Scalars(products) => products.retained_bytes(),
        }
    }
}
impl MathService {
    /// Clear retained programs without cancelling live owners or permitting late reinsertion.
    pub fn clear_program_cache(&self) {
        self.retire_idle_native_lanes();
        self.trim_compiler_lanes();
        self.pressure.clear();
    }
    async fn scalar_products(
        self: &Arc<Self>,
        request: ArtifactRequest,
        caller: &pse_diagnostics::Cancellation,
    ) -> Result<Arc<pse_math::library::ScalarProducts>, MathRuntimeError> {
        let key = Key::Scalars(request.scalar_key());
        if let Some(Value::Scalars(products)) = self.entries.get(&key) {
            return Ok(products);
        }
        let descriptor_owner =
            self.reserve("math:scalar-flight-descriptor", request.descriptor_bytes())?;
        let service = self.clone();
        let epoch = self.retention.generation();
        let population = super::PopulationScope::capture();
        let attribution = crate::ExecutionContext::capture();
        self.scalar_flights
            .load_owned_until(
                key.clone(),
                move |cancel| {
                    attribution
                        .shared_producer_at(
                            "math.scalar.producer",
                            pse_diagnostics::observation::current(),
                        )
                        .instrument(population.run(async move {
                            let _descriptor_owner = descriptor_owner;
                            if let Some(Value::Scalars(products)) = service.entries.get(&key) {
                                return Ok(products);
                            }
                            let construction = super::jobs::ConstructionEstimate::from_bound(
                                request.construction_allocation_bound()?,
                                service.policy.worker_bytes,
                            );
                            let known_limit = construction.bytes();
                            let foreign = service.policy.foreign_bytes;
                            let check = cancel.flag();
                            let (products, lease) = service
                                .job_retained_construction(
                                    request.cores(),
                                    construction,
                                    cancel,
                                    None,
                                    move |flag| {
                                        let products = request
                                            .build_scalars(&pse_kernels::ExecutionScope::new(
                                                flag, None,
                                            ))
                                            .map_err(MathRuntimeError::Math)?;
                                        let known = products.retained_bytes();
                                        if known > known_limit {
                                            return Err(MathRuntimeError::Limit(
                                                "scalar retained construction extent",
                                            ));
                                        }
                                        let retained = known
                                            .checked_add(if products.is_empty() {
                                                0
                                            } else {
                                                foreign
                                            })
                                            .ok_or(MathRuntimeError::Limit(
                                                "scalar retained foreign extent",
                                            ))?;
                                        Ok((products, retained))
                                    },
                                )
                                .await?;
                            if check.load(Ordering::Acquire) {
                                return Err(MathRuntimeError::Cancelled);
                            }
                            service.live.fetch_add(lease.size(), Ordering::AcqRel);
                            let owner = Arc::new(ProgramOwner {
                                lease,
                                live: service.live.clone(),
                            });
                            let products =
                                Arc::new(products.with_owner_and_foreign_allowance(owner, foreign));
                            service.pressure.publish_artifact(
                                epoch,
                                &key,
                                Value::Scalars(products.clone()),
                            );
                            Ok(products)
                        }))
                },
                caller.cancelled(),
            )
            .await
            .map_err(|error| match error {
                FlightError::Load(error) => MathRuntimeError::Shared(error),
                FlightError::Capacity => MathRuntimeError::Limit("scalar flights"),
                FlightError::Retiring => MathRuntimeError::Retiring,
                FlightError::Panicked => MathRuntimeError::Panic("scalar task panic".into()),
            })?
            .ok_or(MathRuntimeError::Cancelled)
    }
    /// Obtain a compiler-issued artifact; callers cannot supply an independent cache key.
    pub async fn artifact(
        self: &Arc<Self>,
        request: ArtifactRequest,
    ) -> Result<Arc<Artifact>, MathRuntimeError> {
        self.artifact_until(request, std::future::pending())
            .await?
            .ok_or_else(|| MathRuntimeError::Infrastructure("unrequested artifact stop".into()))
    }
    pub(super) async fn artifact_for(
        self: &Arc<Self>,
        request: ArtifactRequest,
        driver: &crate::CancelSource,
    ) -> Result<Arc<Artifact>, MathRuntimeError> {
        let driver = self.preparation_driver(driver)?;
        let deadline = driver.deadline().ok_or_else(|| {
            MathRuntimeError::Infrastructure("artifact wait without deadline".into())
        })?;
        let stop = async {
            tokio::select! { biased; () = driver.cancelled() => {}, () = tokio::time::sleep_until(deadline.into()) => {} }
        };
        match self.artifact_until(request, stop).await? {
            Some(artifact) => {
                let _ = self.preparation_driver(&driver)?;
                Ok(artifact)
            }
            None => {
                let _ = self.preparation_driver(&driver)?;
                Err(MathRuntimeError::Infrastructure(
                    "artifact stopped without caller stop".into(),
                ))
            }
        }
    }
    /// Independent immutable requests use existing shared jobs, finite flights and
    /// memory/CPU admission. Ordered reduction selects the first source error; a
    /// private child stop drains issued last-waiter work without cancelling siblings.
    pub(super) async fn prepare_artifacts<'a>(
        self: &Arc<Self>,
        requests: impl ExactSizeIterator<Item = &'a ArtifactRequest> + Clone,
        driver: &crate::CancelSource,
    ) -> Result<Vec<Arc<Artifact>>, MathRuntimeError> {
        let driver = self.preparation_driver(driver)?;
        let width = self.artifact_width();
        let cloned_bytes = requests
            .clone()
            .try_fold(0usize, |bytes, request| {
                bytes.checked_add(request.descriptor_bytes())
            })
            .ok_or(MathRuntimeError::Limit(
                "artifact request descriptor extent",
            ))?;
        collect_artifacts(
            requests,
            width,
            &driver,
            cloned_bytes,
            |bytes| self.reserve("math:artifact-request-batch", bytes),
            |request, child| async move { self.artifact_for(request.clone(), &child).await },
        )
        .await
    }
    pub(super) fn artifact_width(&self) -> usize {
        // A resident session lends one exclusive population owner to its
        // dependent preparation. Issuing concurrent cold producers under that
        // same loan would refuse before admission; top-level independent
        // preparation also respects the declared pending native population.
        // Ready cache hits return before admission and do not wait for a ticket.
        if super::PopulationScope::capture().belongs_to(self) {
            1
        } else {
            self.cores
                .min(self.policy.flights)
                .min(self.policy.jobs)
                .max(1)
        }
    }
    async fn artifact_until(
        self: &Arc<Self>,
        request: ArtifactRequest,
        stop: impl Future<Output = ()>,
    ) -> Result<Option<Arc<Artifact>>, MathRuntimeError> {
        // Admission applies even to a cache hit under this runtime's current policy.
        if request.cores() == 0
            || request.cores() > self.cores
            || request.scratch_limit() > self.policy.worker_bytes
        {
            return Err(MathRuntimeError::Limit("artifact profile"));
        }
        let key = Key::Artifact(request.key());
        if let Some(Value::Artifact(v)) = self.entries.get(&key) {
            self.hits.fetch_add(1, Ordering::Relaxed);
            return Ok(Some(v));
        }
        self.misses.fetch_add(1, Ordering::Relaxed);
        // The completion-owned producer can outlive this caller when another
        // waiter joins. Its descriptor must not borrow the caller's batch grant.
        let descriptor_owner = self.reserve(
            "math:artifact-flight-descriptor",
            request.descriptor_bytes(),
        )?;
        let service = self.clone();
        let flight_key = key.clone();
        let epoch = self.retention.generation();
        let population = super::PopulationScope::capture();
        let attribution = crate::ExecutionContext::capture();
        self.flights
            .load_owned_until(
                flight_key,
                move |cancel| {
                    attribution
                        .shared_producer_at(
                            "math.artifact.producer",
                            pse_diagnostics::observation::current(),
                        )
                        .instrument_with(
                            population.run(async move {
                                let _descriptor_owner = descriptor_owner;
                                if let Some(Value::Artifact(v)) = service.entries.get(&key) {
                                    return Ok(v);
                                }
                                let scalars =
                                    service.scalar_products(request.clone(), &cancel).await?;
                                let scalar_key = Key::Scalars(request.scalar_key());
                                let cores = request.cores();
                                let construction = super::jobs::ConstructionEstimate::from_bound(
                                    request.construction_allocation_bound()?,
                                    service.policy.worker_bytes,
                                );
                                let demand = construction.bytes();
                                if demand > service.policy.worker_bytes {
                                    return Err(MathRuntimeError::Limit(
                                        "artifact construction capacity",
                                    ));
                                }
                                let foreign = service.policy.foreign_bytes;
                                let known_limit = construction.bytes();
                                let check = cancel.flag();
                                let (program, lease) = service
                                    .job_retained_construction(
                                        cores,
                                        construction,
                                        cancel,
                                        None,
                                        move |flag| {
                                            let program = request
                                                .build_reusing(
                                                    &pse_kernels::ExecutionScope::new(flag, None),
                                                    &scalars,
                                                )
                                                .map_err(MathRuntimeError::Math)?;
                                            let known = program.retained_bytes();
                                            if known > known_limit {
                                                return Err(MathRuntimeError::Limit(
                                                    "program retained construction extent",
                                                ));
                                            }
                                            let retained = known
                                                .checked_add(
                                                    if program.scalar_products().has_unowned() {
                                                        foreign
                                                    } else {
                                                        0
                                                    },
                                                )
                                                .ok_or(MathRuntimeError::Limit(
                                                    "program retained foreign extent",
                                                ))?;
                                            Ok((program, retained))
                                        },
                                    )
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
                                    program: Arc::new(
                                        program.with_owner_and_scalar_foreign_allowance(
                                            lease.clone(),
                                            foreign,
                                        ),
                                    ),
                                    lease,
                                });
                                // Keep order-specific provider-lift scalars as seeds for later
                                // upgrades too; every borrowed entry retains its first lease.
                                service.pressure.publish_artifact(
                                    epoch,
                                    &scalar_key,
                                    Value::Scalars(Arc::new(
                                        artifact.program.scalar_products().clone(),
                                    )),
                                );
                                service.pressure.publish_artifact(
                                    epoch,
                                    &key,
                                    Value::Artifact(artifact.clone()),
                                );
                                Ok(artifact)
                            }),
                            |result| {
                                Some(match result {
                                    Ok(_) => pse_diagnostics::observation::Outcome::Completed,
                                    Err(MathRuntimeError::Cancelled) => {
                                        pse_diagnostics::observation::Outcome::Cancelled
                                    }
                                    Err(_) => pse_diagnostics::observation::Outcome::Failed,
                                })
                            },
                        )
                },
                stop,
            )
            .await
            .map_err(|e| match e {
                FlightError::Load(e) => MathRuntimeError::Shared(e),
                FlightError::Capacity => MathRuntimeError::Limit("artifact flights"),
                FlightError::Retiring => MathRuntimeError::Retiring,
                FlightError::Panicked => MathRuntimeError::Panic("artifact task panic".into()),
            })
    }
}
pub(super) async fn collect_artifacts<T, I, F, Fut, O>(
    requests: I,
    width: usize,
    driver: &crate::CancelSource,
    cloned_bytes: usize,
    reserve: impl FnOnce(usize) -> Result<O, MathRuntimeError>,
    prepare: F,
) -> Result<Vec<T>, MathRuntimeError>
where
    I: IntoIterator,
    I::IntoIter: ExactSizeIterator,
    F: Fn(I::Item, crate::CancelSource) -> Fut,
    Fut: Future<Output = Result<T, MathRuntimeError>>,
{
    use futures_util::StreamExt;
    let requests = requests.into_iter();
    let count = requests.len();
    let width = width.max(1).min(count.max(1));
    // futures-util0.3.34 uses one Task<OrderWrapper<Fut>> per pending
    // future plus a bounded queued-output heap. The fixed allowance covers
    // its links, Arc/waker/queue nodes and our wrapper/child-token metadata;
    // the actual user-future and result sizes and cloned descriptors are separate.
    let pending_bytes = size_of::<Fut>()
        .checked_add(size_of::<I::Item>())
        .and_then(|bytes| bytes.checked_add(1024))
        .and_then(|bytes| bytes.checked_mul(width));
    let bytes = count
        .checked_mul(2)
        .and_then(|n| n.checked_add(4))
        .and_then(|n| n.checked_mul(size_of::<T>()))
        .and_then(|bytes| {
            width
                .checked_mul(2)
                .and_then(|n| n.checked_add(4))
                .and_then(|n| {
                    n.checked_mul(size_of::<Result<T, MathRuntimeError>>() + size_of::<i64>())
                })
                .and_then(|heap| bytes.checked_add(heap))
        })
        .and_then(|bytes| bytes.checked_add(pending_bytes?))
        .and_then(|bytes| bytes.checked_add(cloned_bytes))
        .and_then(|bytes| bytes.checked_add(4096))
        .ok_or(MathRuntimeError::Limit(
            "artifact batch coordination extent",
        ))?;
    let _owner = reserve(bytes)?;
    let child = driver.child_with_deadline(None);
    let prepare = &prepare;
    let child_ref = &child;
    let mut pending = futures_util::stream::iter(requests)
        .map(|request| async move {
            if child_ref.token().is_cancelled() {
                return Err(MathRuntimeError::Cancelled);
            }
            prepare(request, child_ref.clone()).await
        })
        .buffered(width.max(1));
    let mut products = Vec::with_capacity(count);
    let mut failure = None;
    while let Some(result) = pending.next().await {
        match result {
            Ok(product) if failure.is_none() => products.push(product),
            Err(error) if failure.is_none() => {
                failure = Some(error);
                child.cancel();
                products.clear();
            }
            _ => {}
        }
    }
    match failure {
        Some(error) => Err(error),
        None => Ok(products),
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
                key: match key {
                    Key::Artifact(hash) => hash.to_string(),
                    Key::Scalars(hash) => format!("scalar:{hash}"),
                },
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
    async fn cold_artifact_batch_respects_population_and_hot_hits_need_no_ticket() {
        use futures_util::FutureExt;
        let pool: Arc<dyn datafusion::execution::memory_pool::MemoryPool> = Arc::new(
            datafusion::execution::memory_pool::FairSpillPool::new(512 << 20),
        );
        let native = pse_engine::cache_service::NativeCacheService::new(
            pse_engine::cache_service::CacheBudget::disabled(1024),
            &pool,
        )
        .unwrap();
        let policy = super::super::MathPolicy {
            jobs: 2,
            foreign_bytes: 1 << 20,
            worker_bytes: 16 << 20,
            workspace_bytes: 32 << 20,
            artifact_bytes: 64 << 20,
            ..Default::default()
        };
        let service = MathService::new(
            pool,
            Arc::new(pse_engine::resources::ReleaseNotifyingSemaphore::new(8)),
            8,
            policy,
            &native,
        );
        let mut preparations = Vec::new();
        let mut requests = Vec::new();
        for index in 0..6 {
            let source = format!(
                "package p {{def Root {{param p:Scalar=1;var x:Scalar;eq a:x*p=={};annotation start x(1);}}}}",
                index + 1
            );
            let (preparation, _, _) =
                super::super::tests::component_fixture_source(&service, &source);
            requests.push(preparation.compiled().artifacts[0].clone());
            preparations.push(preparation);
        }
        let unique = requests
            .iter()
            .map(|request| request.key())
            .collect::<std::collections::BTreeSet<_>>();
        assert!(
            unique.len() > service.policy.jobs,
            "the batch must contain distinct cold products beyond its population"
        );
        assert_eq!(service.entries.len(), 0);
        let cpu = service.cpu.clone().acquire_many_owned(8).await.unwrap();
        let driver = crate::CancelSource::new();
        let child = driver.clone();
        let worker = service.clone();
        let selected = requests.clone();
        let constructing =
            tokio::spawn(async move { worker.prepare_artifacts(selected.iter(), &child).await });
        let observed = tokio::time::timeout(std::time::Duration::from_secs(5), async {
            while service.jobs.available_permits() != 0 && !constructing.is_finished() {
                tokio::task::yield_now().await;
            }
            // Allow every initially issued producer to reach admission while all
            // CPU permits remain held. An over-wide cold batch fails here.
            for _ in 0..8 {
                tokio::task::yield_now().await;
            }
            !constructing.is_finished()
                && service.jobs.available_permits() == 0
                && service.flights.active() == service.policy.jobs
        })
        .await
        .unwrap_or(false);
        if !observed {
            driver.cancel();
        }
        // Always release the gate and join issued producers before asserting,
        // including when the former over-wide implementation returns an error.
        drop(cpu);
        let products = constructing.await.unwrap();
        assert!(
            observed,
            "actual pending cold producers must fit the native population"
        );
        let products = products.unwrap();
        assert_eq!(products.len(), requests.len());
        assert_eq!(service.jobs.available_permits(), service.policy.jobs);
        let sessions = (0..service.policy.jobs)
            .map(|_| service.open_session().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(service.jobs.available_permits(), 0);
        let cached = service
            .prepare_artifacts(requests.iter(), &crate::CancelSource::new())
            .now_or_never();
        for session in sessions {
            session.close().await.unwrap();
        }
        let cached = cached
            .expect("ready cache hits do not await native population")
            .unwrap();
        assert!(
            products
                .iter()
                .zip(cached.iter())
                .all(|(cold, hot)| Arc::ptr_eq(cold, hot))
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn large_rational_scalar_foreign_capacity_survives_eviction_until_last_worker() {
        let (service, _cache) = super::super::tests::service_with_policy(
            256 << 20,
            super::super::MathPolicy::default(),
        );
        let denominator = "9".repeat(300);
        let source = format!(
            "package p {{def Root {{param p:Scalar=1;var x:Scalar;eq a:x*p==1/{denominator};annotation start x(1);}}}}"
        );
        let (preparation, _, _) = super::super::tests::component_fixture_source(&service, &source);
        let request = preparation.compiled().artifacts[0].clone();
        let caller = pse_diagnostics::Cancellation::default();
        let scalar = service
            .scalar_products(request.clone(), &caller)
            .await
            .unwrap();
        assert_eq!(
            scalar.foreign_allowance_bytes(),
            service.policy.foreign_bytes
        );
        let artifact = service.artifact(request).await.unwrap();
        let worker = artifact.program.worker();
        assert_eq!(
            artifact.program.scalar_products().foreign_allowance_bytes(),
            service.policy.foreign_bytes
        );
        service.clear_program_cache();
        drop(scalar);
        drop(artifact);
        assert!(service.live.load(Ordering::Acquire) >= service.policy.foreign_bytes);
        drop(worker);
        assert_eq!(service.live.load(Ordering::Acquire), 0);
    }
    #[tokio::test(flavor = "current_thread")]
    async fn scalar_family_flights_share_preparation_and_eviction_keeps_borrowed_leases() {
        let (service, _cache) = super::super::tests::service_with_policy(
            256 << 20,
            super::super::MathPolicy::default(),
        );
        let (preparation, _, _) = super::super::tests::component_fixture(&service);
        let request = preparation.compiled().artifacts[0].clone();
        let caller = pse_diagnostics::Cancellation::default();
        let (first, second) = tokio::join!(
            service.scalar_products(request.clone(), &caller),
            service.scalar_products(request.clone(), &caller)
        );
        let first = first.unwrap();
        let second = second.unwrap();
        assert!(Arc::ptr_eq(&first, &second));
        assert!(!first.is_empty());
        let scalar_key = Key::Scalars(request.scalar_key());
        assert!(matches!(
            service.entries.get(&scalar_key),
            Some(Value::Scalars(_))
        ));
        let artifact = service.artifact(request).await.unwrap();
        let worker = artifact.program.worker();
        let charged = service.live.load(Ordering::Acquire);
        assert!(charged >= first.retained_bytes());
        service.clear_program_cache();
        assert_eq!(service.entries.len(), 0);
        drop(first);
        drop(second);
        drop(artifact);
        assert!(
            service.live.load(Ordering::Acquire) > 0,
            "worker borrows both scalar and program charges"
        );
        drop(worker);
        assert_eq!(service.live.load(Ordering::Acquire), 0);
    }
}

#[cfg(test)]
mod parallel_tests {
    use super::*;
    use futures_util::FutureExt;
    use std::sync::atomic::AtomicUsize;

    #[tokio::test]
    async fn artifact_batch_refuses_coordination_capacity_before_any_request_effect() {
        let entered = AtomicUsize::new(0);
        let driver = crate::CancelSource::new();
        let result = collect_artifacts(
            0..4,
            2,
            &driver,
            0,
            |bytes| {
                assert!(bytes > 0);
                Err::<(), _>(MathRuntimeError::Limit("batch denied"))
            },
            |index, _| {
                entered.fetch_add(1, Ordering::SeqCst);
                async move { Ok(index) }
            },
        )
        .await;
        assert!(matches!(
            result,
            Err(MathRuntimeError::Limit("batch denied"))
        ));
        assert_eq!(entered.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn independent_artifacts_overlap_but_publish_in_request_order() {
        let driver = crate::CancelSource::new();
        let entered = Arc::new(AtomicUsize::new(0));
        let gates = [
            Arc::new(tokio::sync::Notify::new()),
            Arc::new(tokio::sync::Notify::new()),
        ];
        let mut operation = Box::pin(collect_artifacts(
            0..2,
            2,
            &driver,
            0,
            |_| Ok(()),
            |index, _| {
                let gate = gates[index].clone();
                let entered = entered.clone();
                async move {
                    entered.fetch_add(1, Ordering::SeqCst);
                    gate.notified().await;
                    Ok(index)
                }
            },
        ));
        assert!(operation.as_mut().now_or_never().is_none());
        assert_eq!(
            entered.load(Ordering::SeqCst),
            2,
            "both independent requests are polled"
        );
        gates[1].notify_one();
        assert!(
            operation.as_mut().now_or_never().is_none(),
            "later completion cannot reorder publication"
        );
        gates[0].notify_one();
        assert_eq!(operation.await.unwrap(), [0, 1]);
        assert!(!driver.token().is_cancelled());
    }

    #[tokio::test]
    async fn first_source_failure_cancels_and_joins_issued_private_work_before_return() {
        let driver = crate::CancelSource::new();
        let flights = pse_columnar::flight::Flights::<usize, usize, &'static str>::new(2);
        let entered = Arc::new(AtomicUsize::new(0));
        let failure = Arc::new(tokio::sync::Notify::new());
        let stopped = Arc::new(tokio::sync::Notify::new());
        let exit = Arc::new(tokio::sync::Notify::new());
        let mut operation = Box::pin(collect_artifacts(
            0..4,
            2,
            &driver,
            0,
            |_| Ok(()),
            |index, child| {
                let entered = entered.clone();
                let failure = failure.clone();
                let stopped = stopped.clone();
                let exit = exit.clone();
                let flights = &flights;
                async move {
                    entered.fetch_add(1, Ordering::SeqCst);
                    let result = flights
                        .load_owned_until(
                            index,
                            move |cancel| async move {
                                if index == 0 {
                                    failure.notified().await;
                                    Err("first source failure")
                                } else {
                                    cancel.cancelled().await;
                                    stopped.notify_one();
                                    exit.notified().await;
                                    Err("stopped later work")
                                }
                            },
                            child.cancelled(),
                        )
                        .await;
                    match result {
                        Ok(Some(value)) => Ok(*value),
                        Ok(None) => Err(MathRuntimeError::Cancelled),
                        Err(_) => Err(MathRuntimeError::Infrastructure(
                            "first source failure".into(),
                        )),
                    }
                }
            },
        ));
        assert!(operation.as_mut().now_or_never().is_none());
        // Start both completion-owned producers before releasing the first failure.
        tokio::task::yield_now().await;
        failure.notify_one();
        tokio::select! {
            () = stopped.notified() => {},
            result = operation.as_mut() => panic!("returned before issued work stopped: {result:?}"),
            () = tokio::time::sleep(std::time::Duration::from_secs(2)) => panic!("issued work did not receive stop"),
        }
        assert_eq!(
            entered.load(Ordering::SeqCst),
            2,
            "queued work is never issued after failure"
        );
        assert_eq!(flights.active(), 1);
        assert!(
            operation.as_mut().now_or_never().is_none(),
            "stopped issued work is still draining"
        );
        exit.notify_one();
        assert!(
            matches!(operation.await, Err(MathRuntimeError::Infrastructure(detail)) if detail == "first source failure")
        );
        assert_eq!(flights.active(), 0);
        assert!(
            !driver.token().is_cancelled(),
            "batch stop cannot cancel its caller"
        );
    }
}
