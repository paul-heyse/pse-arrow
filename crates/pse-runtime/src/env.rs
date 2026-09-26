// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The one finite query/platform budget shared by all sessions (ADR-0046).

use std::sync::Arc;

use datafusion_execution::memory_pool::{
    FairSpillPool, MemoryLimit, MemoryPool, PeakRecordingPool, TrackConsumersPool,
};
use datafusion_execution::runtime_env::RuntimeEnv;

use crate::{ResourceBudget, ResourceReport, RuntimeError};

/// Shared deployment state. Building session handles never constructs another pool.
#[derive(Debug)]
pub struct SharedRuntime {
    env: Arc<RuntimeEnv>,
    tracked: Arc<TrackConsumersPool<FairSpillPool>>,
    peak: Arc<PeakRecordingPool>,
    budget: ResourceBudget,
    compiler_cpu: Arc<tokio::sync::Semaphore>,
    caches: Arc<pse_catalog::cache_service::DeltaCacheService>,
    math: Arc<crate::math::MathService>,
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
        let native = pse_engine::resources::EngineResources::build(
            budget.memory_limit_bytes,
            budget.top_consumers,
            budget.spill_dir.clone(),
            budget.max_temp_dir_bytes,
            budget.cache.native.clone(),
        )?;
        let caches = pse_catalog::cache_service::DeltaCacheService::with_native(
            budget.cache.clone(),
            native.caches.clone(),
        )
        .map_err(|error| RuntimeError::Catalog(pse_engine::session::engine(error)))?;
        let env = native.runtime;
        let tracked = native.tracked;
        let peak = native.peak;
        if !matches!(env.memory_pool.memory_limit(), MemoryLimit::Finite(limit) if limit == budget.memory_limit_bytes.get())
            || env.disk_manager.max_temp_directory_size() != budget.max_temp_dir_bytes
            || !env.disk_manager.tmp_files_enabled()
        {
            return Err(RuntimeError::Internal {
                message: "runtime memory/spill limits failed read-back".to_owned(),
            });
        }
        // The standard library accounts for process affinity and Linux CPU quota.
        // Keep the configured pool size as a ceiling; native admission uses the
        // effective capacity observed when this runtime is created.
        let native_cores = std::thread::available_parallelism()
            .map_or(1, std::num::NonZeroUsize::get)
            .min(budget.threads.pool_threads.get());
        let compiler_cpu = Arc::new(tokio::sync::Semaphore::new(native_cores));
        let math = crate::math::MathService::new(
            env.memory_pool.clone(),
            compiler_cpu.clone(),
            native_cores,
            budget.math.clone(),
            caches.native(),
        );
        Ok(Arc::new(Self {
            env,
            tracked,
            peak,
            compiler_cpu,
            math,
            budget,
            caches,
        }))
    }

    /// Shared math compiler/runtime artifact owner.
    pub fn math(&self) -> &Arc<crate::math::MathService> {
        &self.math
    }

    pub(crate) fn compiler_cpu(&self) -> Arc<tokio::sync::Semaphore> {
        self.compiler_cpu.clone()
    }

    /// The same engine runtime for every session constructed by the runtime layer.
    pub fn runtime_env(&self) -> Arc<RuntimeEnv> {
        Arc::clone(&self.env)
    }

    /// The pool shared by query operators and platform reservations.
    pub fn pool(&self) -> Arc<dyn MemoryPool> {
        Arc::clone(&self.env.memory_pool)
    }

    /// Validated resource and execution policy.
    pub const fn budget(&self) -> &ResourceBudget {
        &self.budget
    }

    /// The same native cache owner for all factories and provider scopes.
    pub fn caches(&self) -> &Arc<pse_catalog::cache_service::DeltaCacheService> {
        &self.caches
    }

    /// Start a new pool observation without discarding current live reservations.
    pub fn reset_observation_peak(&self) {
        self.peak.reset_peak();
    }
    /// Maximum reserved bytes since the last observation reset.
    pub fn observation_peak_bytes(&self) -> usize {
        self.peak.peak_reserved()
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
            caches: self.caches.native().report(),
        })
    }
}
