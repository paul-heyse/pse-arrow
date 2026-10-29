// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Local public scientific construction shared by independent reference journeys.
use pse_runtime::{SharedRuntime, workflow::Runtime};
use std::{num::NonZeroUsize, path::{Path, PathBuf}, sync::Arc};

pub(crate) type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
pub(crate) fn root() -> PathBuf { Path::new(env!("CARGO_MANIFEST_DIR")).join("../..") }
pub(crate) struct NativeContext {
    pub(crate) shared: Arc<SharedRuntime>,
    pub(crate) runtime: Runtime,
    _spill: tempfile::TempDir,
}

impl NativeContext {
    pub(crate) fn new() -> Result<Self> {
        // Existing downstream local-operation construction, without a canonical service.
        let spill = tempfile::tempdir()?;
        let n = |v| NonZeroUsize::new(v).unwrap();
        let shared = SharedRuntime::build(pse_runtime::ResourceBudget {
            memory_limit_bytes: n(4 << 30), spill_dir: spill.path().to_owned(),
            max_temp_dir_bytes: 1 << 30, top_consumers: n(16),
            threads: pse_engine::ThreadBudget { pool_threads: n(16), target_partitions: n(16) },
            execution: Default::default(), cache: pse_runtime::CacheBudget::disabled(1024),
            math: pse_runtime::math::MathPolicy {
                worker_bytes: 1 << 30, workspace_bytes: 1 << 30, foreign_bytes: 16 << 20,
                jobs: std::thread::available_parallelism()?.get(), ..Default::default()
            },
        })?;
        let factory = Arc::new(shared.session_factory(pse_engine::session::native_engine_profile())?);
        let runtime = Runtime::from_shared_local(shared.clone(), factory.bind_model(pse_schema::shared_registry()?)?)?;
        Ok(Self { shared, runtime, _spill: spill })
    }

    pub(crate) async fn close(&self) -> Result<()> {
        self.runtime.close(); self.runtime.clear_program_cache();
        Ok(self.shared.math().shutdown_native_lanes().await?)
    }
}

