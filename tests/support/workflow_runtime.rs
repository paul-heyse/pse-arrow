// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! One domain fixture owner for compiler tests and standalone inspection tooling.
#[path = "workflow_budget.rs"]
mod workflow_budget;
use pse_columnar::CancellationToken;
use pse_engine::{
    ExecutionSettings, ThreadBudget,
    session::{EngineFactory, native_engine_profile},
};
use pse_runtime::{ResourceBudget, SharedRuntime};
use std::{num::NonZeroUsize, sync::Arc};

#[allow(
    dead_code,
    reason = "shared domain owner has different consumers in tests and tooling"
)]
pub(crate) struct WorkflowRuntime {
    pub registry: Arc<pse_schema::Registry>,
    pub sessions: Arc<EngineFactory>,
    pub cancel: CancellationToken,
    pub runtime: Arc<SharedRuntime>,
    _spill: tempfile::TempDir,
}
impl WorkflowRuntime {
    pub(crate) fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Self::with_threads(NonZeroUsize::new(2).ok_or("two is positive")?)
    }
    pub(crate) fn with_threads(
        workers: NonZeroUsize,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Self::with_policy(
            workers,
            Default::default(),
            workflow_budget::MEMORY_LIMIT_BYTES,
        )
    }
    /// A runtime whose native jobs run under an explicit math policy and shared memory
    /// ceiling, such as a larger foreign-library allowance a test declares for itself: the
    /// allowance is charged to every native job and retained program, so the ceiling
    /// grows with it.
    #[allow(
        dead_code,
        reason = "shared domain owner has different consumers in tests and tooling"
    )]
    pub(crate) fn with_math(
        math: pse_runtime::math::MathPolicy,
        memory_limit_bytes: usize,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Self::with_policy(
            NonZeroUsize::new(2).ok_or("two is positive")?,
            math,
            memory_limit_bytes,
        )
    }
    fn with_policy(
        workers: NonZeroUsize,
        math: pse_runtime::math::MathPolicy,
        memory_limit_bytes: usize,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let registry = pse_schema::shared_registry()?;
        let spill = tempfile::tempdir()?;
        let threads = ThreadBudget {
            pool_threads: workers,
            target_partitions: workers,
        };
        let mut cache = pse_runtime::DeltaCacheBudget::for_memory(memory_limit_bytes);
        cache.native.concurrent_queries =
            NonZeroUsize::new(workers.get().min(2)).ok_or("positive queries")?;
        cache.native.concurrent_outputs =
            NonZeroUsize::new(workers.get().min(4)).ok_or("positive outputs")?;
        let runtime = SharedRuntime::build(ResourceBudget {
            memory_limit_bytes: NonZeroUsize::new(memory_limit_bytes)
                .ok_or("positive memory limit")?,
            spill_dir: spill.path().to_path_buf(),
            max_temp_dir_bytes: 1 << 30,
            top_consumers: NonZeroUsize::new(16).ok_or("positive consumers")?,
            threads,
            execution: ExecutionSettings::default(),
            cache,
            math,
            hashing_may_use_pool: false,
        })?;
        let sessions = Arc::new(
            runtime
                .session_factory(native_engine_profile())?
                .with_requirement_planner(Arc::new(
                    pse_rules::invariants::RegistryRequirementPlanner,
                )),
        );
        Ok(Self {
            registry,
            sessions,
            runtime,
            cancel: CancellationToken::new(),
            _spill: spill,
        })
    }
}
