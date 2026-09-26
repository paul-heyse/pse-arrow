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
pub(super) struct Key(pub ContentHash);
impl CacheKey for Key {
    fn size(&self) -> usize {
        128
    }
    fn table_ref(&self) -> Option<&TableReference> {
        None
    }
}
/// Immutable program and its live native allocation allowance. Clones retain accounting.
#[derive(Debug)]
pub struct Artifact {
    pub(super) program: Arc<CompiledBody>,
    pub(super) lease: Arc<ProgramOwner>,
}
#[derive(Debug)]
pub(super) struct ProgramOwner {
    lease: Arc<pse_columnar::AllocationLease>,
    live: Arc<std::sync::atomic::AtomicUsize>,
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
pub(super) struct Value(pub Arc<Artifact>);
impl CacheValue for Value {
    fn size(&self) -> usize {
        self.0.lease.size()
    }
}
impl MathService {
    /// Clear retained programs without cancelling live owners or permitting late reinsertion.
    pub fn clear_program_cache(&self) {
        self.retention.clear(|| self.entries.clear());
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
        let key = Key(request.key());
        if let Some(v) = self.entries.get(&key) {
            self.hits.fetch_add(1, Ordering::Relaxed);
            return Ok(v.0);
        }
        self.misses.fetch_add(1, Ordering::Relaxed);
        let service = self.clone();
        let flight_key = key.clone();
        let epoch = self.retention.generation();
        self.flights
            .load_owned(flight_key, move |cancel| async move {
                if let Some(v) = service.entries.get(&key) {
                    return Ok(v.0);
                }
                let cores = request.cores();
                let check = cancel.flag();
                let foreign = service.policy.foreign_bytes;
                let (program, lease) = service
                    .job_retained(cores, request.scratch_limit(), cancel, move |flag| {
                        let program = request.build(&flag).map_err(MathRuntimeError::Math)?;
                        let retained = program
                            .retained_numeric_bytes()
                            .checked_add(foreign)
                            .ok_or(MathRuntimeError::Limit("retained artifact extent"))?;
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
                    .retention
                    .admit(epoch, || service.entries.put(&key, Value(artifact.clone())));
                Ok(artifact)
            })
            .await
            .map_err(|e| match e {
                FlightError::Load(e) => MathRuntimeError::Shared(e),
                FlightError::Capacity => MathRuntimeError::Limit("artifact flights"),
                FlightError::Retiring => MathRuntimeError::Retiring,
                FlightError::Panicked => {
                    MathRuntimeError::Infrastructure("artifact task panic".into())
                }
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
        vec![CacheReport {
            name: self.entries.name(),
            capacity_bytes: 0,
            retained_bytes: self.entries.memory_used(),
            policy_limit_bytes: self.entries.cache_limit(),
            live_bytes: Some(self.live.load(Ordering::Acquire)),
            pinned_bytes: None,
            inflight_bytes: None,
            active_loads: Some(self.flights.active()),
            evictions: None,
            entries: self.entries.len(),
            hits: self.hits.load(Ordering::Relaxed),
            misses: self.misses.load(Ordering::Relaxed),
            bypasses: 0,
        }]
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
                Some(self.policy.jobs - self.jobs.available_permits()),
            ),
            (
                "math_foreign_allowance_per_job",
                Some(self.policy.foreign_bytes),
            ),
            ("math_stack_per_job", Some(self.policy.stack_bytes)),
        ]
    }
}
