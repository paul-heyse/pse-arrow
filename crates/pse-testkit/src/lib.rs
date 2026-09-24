// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Development fixtures and contract evidence; never a production dependency.
pub mod capture;
pub mod counting_store;
pub mod execution;
pub mod fault_store;
pub mod introspection;
use pse_engine::{
    EngineError, EngineFactory, ExecutionSettings, ThreadBudget, resources::EngineResources,
};
use std::{num::NonZeroUsize, sync::Arc};

/// Construct a native factory around a caller-owned pool for resource-boundary tests.
/// # Errors
/// Runtime construction or execution settings validation fails.
pub fn factory(
    pool: Arc<dyn pse_columnar::MemoryPool>,
    execution: ExecutionSettings,
    threads: ThreadBudget,
) -> Result<EngineFactory, EngineError> {
    let runtime = datafusion::execution::runtime_env::RuntimeEnvBuilder::new()
        .with_memory_pool(pool.clone())
        .build_arc()
        .map_err(|source| EngineError::Internal {
            message: source.to_string(),
        })?;
    Ok(EngineFactory::new(
        runtime,
        pool,
        execution,
        threads,
        pse_engine::session::native_engine_profile(),
    )?
    .with_extension(Arc::new(pse_engine::resources::CpuAdmission {
        permits: Arc::new(tokio::sync::Semaphore::new(threads.pool_threads.get())),
        workers: threads
            .pool_threads
            .try_into()
            .map_err(|_| EngineError::Internal {
                message: "worker count exceeds CPU admission width".into(),
            })?,
    })))
}

/// Isolated finite resources assembled through the production engine constructor.
#[derive(Debug)]
pub struct NativeFixture {
    /// The one runtime, pool and cache owner.
    pub resources: EngineResources,
    /// Reusable actual engine assembly.
    pub factory: EngineFactory,
    spill: tempfile::TempDir,
}
impl NativeFixture {
    /// Transfer the fixture into a factory, retaining its spill directory lifetime.
    pub fn into_factory(self) -> EngineFactory {
        self.factory.with_extension(Arc::new(self.spill))
    }
    /// Allocate a development fixture with no Delta root or solver.
    /// # Errors
    /// Temporary spill directory, native configuration or resource admission failure.
    pub fn new(bytes: NonZeroUsize) -> Result<Self, EngineError> {
        Self::with_settings(
            bytes,
            ThreadBudget {
                pool_threads: NonZeroUsize::MIN,
                target_partitions: NonZeroUsize::MIN,
            },
            ExecutionSettings::default(),
            pse_engine::cache_service::CacheBudget::for_memory(bytes.get()),
        )
    }
    /// Use native policy types for an isolated fixture, including its actual pool.
    /// # Errors
    /// Resource construction or native settings validation fails.
    pub fn with_settings(
        bytes: NonZeroUsize,
        threads: ThreadBudget,
        execution: ExecutionSettings,
        cache: pse_engine::cache_service::CacheBudget,
    ) -> Result<Self, EngineError> {
        let spill = tempfile::tempdir().map_err(|error| EngineError::Internal {
            message: error.to_string(),
        })?;
        let resources = EngineResources::build(
            bytes,
            NonZeroUsize::new(64).unwrap_or(NonZeroUsize::MIN),
            spill.path().to_owned(),
            64 << 20,
            cache,
        )?;
        let factory = EngineFactory::new(
            resources.runtime.clone(),
            resources.pool.clone(),
            execution,
            threads,
            pse_engine::session::native_engine_profile(),
        )?
        .with_cache_service(resources.caches.clone())
        .with_extension(Arc::new(pse_engine::resources::CpuAdmission {
            permits: Arc::new(tokio::sync::Semaphore::new(threads.pool_threads.get())),
            workers: threads
                .pool_threads
                .try_into()
                .map_err(|_| EngineError::Internal {
                    message: "worker count exceeds CPU admission width".into(),
                })?,
        }));
        Ok(Self {
            resources,
            factory,
            spill,
        })
    }
    /// Open an empty declared session for in-memory native operations.
    /// # Errors
    /// Registry construction or native admission failure.
    pub fn session(
        &self,
        registry: Arc<pse_schema::Registry>,
    ) -> Result<pse_engine::EngineSession, EngineError> {
        self.factory.candidate(
            std::collections::BTreeMap::default(),
            registry,
            &pse_columnar::CancellationToken::new(),
        )
    }
}

#[cfg(test)]
mod tests;
