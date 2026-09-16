// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The same bounded runtime and semantic validator used by real commits.

#[path = "../../../tests/support/workflow_budget.rs"]
mod workflow_budget;

use anyhow::Result;
use pse_catalog::{
    Catalog, ExecutionSettings, FixedClock, ThreadBudget, TrustLevel,
    session::{SessionFactory, native_engine_profile},
};
use pse_ids::{CancellationToken, MemoryReserver};
use pse_runtime::{ResourceBudget, SharedRuntime};
use std::{num::NonZeroUsize, path::Path, sync::Arc};

pub(crate) struct Environment {
    pub catalog: Arc<Catalog>,
    pub cancel: CancellationToken,
    pub runtime: Arc<SharedRuntime>,
    _spill: tempfile::TempDir,
}
impl Environment {
    pub(crate) fn new(path: &Path) -> Result<Self> {
        Self::with_registry(path, Arc::new(pse_schema::catalog::assemble()?))
    }
    pub(crate) fn with_registry(path: &Path, registry: Arc<pse_schema::Registry>) -> Result<Self> {
        let spill = tempfile::tempdir()?;
        let one = NonZeroUsize::new(1).expect("one is nonzero");
        let threads = ThreadBudget {
            pool_threads: one,
            target_partitions: one,
        };
        let runtime = SharedRuntime::build(ResourceBudget {
            memory_limit_bytes: NonZeroUsize::new(workflow_budget::MEMORY_LIMIT_BYTES)
                .expect("fixed finite budget"),
            spill_dir: spill.path().to_path_buf(),
            max_temp_dir_bytes: 1 << 30,
            top_consumers: NonZeroUsize::new(16).expect("fixed consumer count"),
            threads,
            execution: ExecutionSettings::default(),
            hashing_may_use_pool: false,
        })?;
        let reserver: Arc<dyn MemoryReserver> = runtime.reserver();
        let sessions = Arc::new(
            SessionFactory::new(
                runtime.runtime_env(),
                Arc::clone(&reserver),
                ExecutionSettings::default(),
                threads,
                native_engine_profile(),
            )?
            .with_query_planner(pse_compiler::query_planner()),
        );
        let validator = pse_rules::validator::InvariantValidator::new(Arc::clone(&registry));
        let validator =
            pse_compiler::validator::CompilerValidator::new(Arc::new(validator), &registry)?
                .with_sessions(Arc::clone(&sessions));
        let catalog = Arc::new(
            Catalog::open_local(
                path,
                Arc::clone(&registry),
                TrustLevel::Untrusted,
                Arc::new(FixedClock("2026-09-14T00:00:00Z".to_owned())),
                Arc::clone(&sessions),
            )?
            .with_semantic_validator(Arc::new(validator)),
        );
        Ok(Self {
            catalog,
            cancel: CancellationToken::new(),
            runtime,
            _spill: spill,
        })
    }
}
