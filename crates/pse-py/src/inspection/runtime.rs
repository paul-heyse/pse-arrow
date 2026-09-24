// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use std::sync::{Arc, Mutex, OnceLock};

use pse_engine::{EngineError, session::native_engine_profile};
use pse_runtime::SharedRuntime;

use super::{errors, settings::EngineSettings};

#[derive(Debug)]
pub(crate) struct Runtime {
    pub(crate) shared: Arc<SharedRuntime>,
    pub(crate) executor: &'static tokio::runtime::Runtime,
    pub(crate) registry: Arc<pse_schema::Registry>,
    pub(crate) sessions: Arc<pse_engine::session::EngineFactory>,
    settings: EngineSettings,
}

static EXECUTOR: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

static PROCESS_RUNTIME: Mutex<Option<Arc<Runtime>>> = Mutex::new(None);

pub(crate) fn acquire(settings: &EngineSettings) -> Result<Arc<Runtime>, EngineError> {
    let mut active = PROCESS_RUNTIME
        .lock()
        .map_err(|_| errors::invalid("process runtime lock poisoned"))?;
    if let Some(runtime) = active.as_ref() {
        if runtime.settings.budget != settings.budget {
            return Err(errors::invalid(
                "all publication handles share the already configured process resource budget",
            ));
        }
        return Ok(Arc::clone(runtime));
    }
    let shared = SharedRuntime::build(settings.budget.clone())
        .map_err(|error| EngineError::Semantic(Arc::new(error)))?;
    // Complete fallible application admission before publishing the process executor.
    // A failed registry/session setup can then be retried without creating a spare pool.
    let registry =
        pse_schema::shared_registry().map_err(|error| EngineError::Semantic(Arc::new(error)))?;
    let sessions = Arc::new(
        shared
            .session_factory(native_engine_profile())
            .map_err(|error| EngineError::Semantic(Arc::new(error)))?
            .with_requirement_planner(Arc::new(pse_rules::invariants::RegistryRequirementPlanner)),
    );
    let executor = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(settings.budget.threads.pool_threads.get())
        // Delta's native kernel executor nests block_in_place and spawn_blocking
        // (including delivery of its completion). A one-thread blocking pool
        // deadlocks even one table open. Retain Tokio's native blocking capacity;
        // the declared cache/read semaphores bound admitted work independently.
        .enable_all()
        .build()
        .map_err(|source| EngineError::Infrastructure {
            op: "build inspection executor".to_owned(),
            source: Box::new(source),
        })?;
    let executor = EXECUTOR.get_or_init(|| executor);
    if pyo3_async_runtimes::tokio::init_with_runtime(executor).is_err()
        && !std::ptr::eq(pyo3_async_runtimes::tokio::get_runtime(), executor)
    {
        return Err(errors::invalid(
            "async bridge was initialized with another executor",
        ));
    }
    let runtime = Arc::new(Runtime {
        shared,
        executor,
        registry,
        sessions,
        settings: settings.clone(),
    });
    *active = Some(Arc::clone(&runtime));
    Ok(runtime)
}
