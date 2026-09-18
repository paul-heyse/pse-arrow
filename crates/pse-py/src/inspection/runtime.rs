// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use std::sync::{Arc, Mutex};

use pse_catalog::{CatalogError, session::native_engine_profile};
use pse_runtime::SharedRuntime;

use super::{errors, settings::EngineSettings};

#[derive(Debug)]
pub(super) struct Runtime {
    pub(super) shared: Arc<SharedRuntime>,
    pub(super) executor: tokio::runtime::Runtime,
    pub(super) registry: Arc<pse_schema::Registry>,
    pub(super) sessions: Arc<pse_catalog::session::SessionFactory>,
    settings: EngineSettings,
}

static PROCESS_RUNTIME: Mutex<Option<Arc<Runtime>>> = Mutex::new(None);

pub(super) fn acquire(settings: &EngineSettings) -> Result<Arc<Runtime>, CatalogError> {
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
        .map_err(|error| CatalogError::Semantic(Arc::new(error)))?;
    let executor = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(settings.budget.threads.pool_threads.get())
        // Delta's native kernel executor nests block_in_place and spawn_blocking
        // (including delivery of its completion). A one-thread blocking pool
        // deadlocks even one table open. Retain Tokio's native blocking capacity;
        // the declared cache/read semaphores bound admitted work independently.
        .enable_all()
        .build()
        .map_err(|source| CatalogError::Infrastructure {
            op: "build inspection executor".to_owned(),
            source: Box::new(source),
        })?;
    let registry = Arc::new(
        pse_schema::catalog::assemble().map_err(|error| CatalogError::Semantic(Arc::new(error)))?,
    );
    let sessions = Arc::new(
        shared
            .session_factory(native_engine_profile())
            .map_err(|error| CatalogError::Semantic(Arc::new(error)))?
            .with_requirement_planner(Arc::new(pse_rules::invariants::RegistryRequirementPlanner)),
    );
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
