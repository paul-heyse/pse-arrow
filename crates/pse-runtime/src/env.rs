// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The one finite query/platform budget shared by all sessions (ADR-0046).

use std::sync::Arc;

use datafusion_execution::memory_pool::{
    FairSpillPool, MemoryLimit, MemoryPool, PeakRecordingPool, TrackConsumersPool,
};
use datafusion_execution::runtime_env::{RuntimeEnv, RuntimeEnvBuilder};

use crate::{PoolReserver, ResourceBudget, ResourceReport, RuntimeError};

/// Shared deployment state. Building session handles never constructs another pool.
#[derive(Debug)]
pub struct SharedRuntime {
    env: Arc<RuntimeEnv>,
    tracked: Arc<TrackConsumersPool<FairSpillPool>>,
    peak: Arc<PeakRecordingPool>,
    reserver: Arc<PoolReserver>,
    budget: ResourceBudget,
    caches: Arc<pse_catalog::cache_service::NativeCacheService>,
}

impl SharedRuntime {
    /// Validates configuration, builds the finite pool and spill manager, then reads
    /// back the limits before making the runtime available.
    ///
    /// # Errors
    /// [`RuntimeError::ConfigInvalid`] for rejected configuration;
    /// [`RuntimeError::Catalog`] for an engine construction failure;
    /// [`RuntimeError::Internal`] for failed configuration read-back.
    pub fn build(budget: ResourceBudget) -> Result<Arc<Self>, RuntimeError> {
        budget.validate()?;
        let tracked = Arc::new(TrackConsumersPool::new(
            FairSpillPool::new(budget.memory_limit_bytes.get()),
            budget.top_consumers,
        ));
        let tracked_pool: Arc<dyn MemoryPool> = tracked.clone();
        let peak = Arc::new(PeakRecordingPool::new(tracked_pool));
        let pool: Arc<dyn MemoryPool> = peak.clone();
        let reserver = Arc::new(PoolReserver::new(Arc::clone(&pool))?);
        let caches =
            pse_catalog::cache_service::NativeCacheService::new(budget.cache.clone(), &pool)
                .map_err(|error| {
                    RuntimeError::Catalog(pse_catalog::failure::collapse_classified(
                        pse_catalog::classify(error, pse_catalog::PlanOrigin::RuleCompiler),
                    ))
                })?;
        let env = RuntimeEnvBuilder::new()
            .with_cache_manager(pse_catalog::cache_service::NativeCacheService::unbound_config())
            .with_memory_pool(pool)
            .with_temp_file_path(budget.spill_dir.clone())
            .with_max_temp_directory_size(budget.max_temp_dir_bytes)
            .build()
            .map_err(|error| {
                RuntimeError::Catalog(pse_catalog::failure::collapse_classified(
                    pse_catalog::classify(error, pse_catalog::PlanOrigin::RuleCompiler),
                ))
            })?;
        if !matches!(env.memory_pool.memory_limit(), MemoryLimit::Finite(limit) if limit == budget.memory_limit_bytes.get())
            || env.disk_manager.max_temp_directory_size() != budget.max_temp_dir_bytes
            || !env.disk_manager.tmp_files_enabled()
        {
            return Err(RuntimeError::Internal {
                message: "runtime memory/spill limits failed read-back".to_owned(),
            });
        }
        Ok(Arc::new(Self {
            env: Arc::new(env),
            tracked,
            peak,
            reserver,
            budget,
            caches,
        }))
    }

    /// The same engine runtime for every session constructed by the runtime layer.
    pub fn runtime_env(&self) -> Arc<RuntimeEnv> {
        Arc::clone(&self.env)
    }

    /// The pool shared by query operators and platform reservations.
    pub fn pool(&self) -> Arc<dyn MemoryPool> {
        Arc::clone(&self.env.memory_pool)
    }

    /// An attributed reserve-before-allocate adapter over [`Self::pool`].
    pub fn reserver(&self) -> Arc<PoolReserver> {
        Arc::clone(&self.reserver)
    }

    /// Validated resource and execution policy.
    pub const fn budget(&self) -> &ResourceBudget {
        &self.budget
    }

    /// The same native cache owner for all factories and provider scopes.
    pub fn caches(&self) -> &Arc<pse_catalog::cache_service::NativeCacheService> {
        &self.caches
    }

    /// Accounted pool usage and independently measured process peak RSS.
    ///
    /// # Errors
    /// [`RuntimeError::Infrastructure`] if a supported process metric cannot be read.
    pub fn report(&self) -> Result<ResourceReport, RuntimeError> {
        let mut consumers: Vec<_> = self
            .tracked
            .metrics()
            .into_iter()
            .map(|m| (m.name, m.reserved))
            .collect();
        consumers.sort_by(|(left_name, left_size), (right_name, right_size)| {
            right_size.cmp(left_size).then(left_name.cmp(right_name))
        });
        consumers.truncate(self.budget.top_consumers.get());
        Ok(ResourceReport {
            limit_bytes: self.budget.memory_limit_bytes.get(),
            pool_peak_bytes: self.peak.max_reserved(),
            pool_reserved_now: self.peak.reserved(),
            top_consumers: consumers,
            process_peak_rss_bytes: crate::peak::process_peak_rss()?,
            caches: self.caches.report(),
        })
    }
}
