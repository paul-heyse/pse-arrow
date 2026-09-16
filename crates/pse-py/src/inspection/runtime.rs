// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use std::sync::{Arc, Mutex};

use pse_catalog::{CatalogError, session::native_engine_profile};
use pse_ids::MemoryReserver;
use pse_runtime::SharedRuntime;

use super::{errors, settings::EngineSettings};

#[derive(Debug)]
pub(super) struct Runtime {
    pub(super) shared: Arc<SharedRuntime>,
    pub(super) executor: tokio::runtime::Runtime,
    pub(super) registry: Arc<pse_schema::Registry>,
    pub(super) validator: Arc<dyn pse_catalog::store::membership::SemanticValidator>,
    pub(super) sessions: Arc<pse_catalog::session::SessionFactory>,
    settings: EngineSettings,
}

static PROCESS_RUNTIME: Mutex<Option<Arc<Runtime>>> = Mutex::new(None);

pub(super) fn acquire(settings: &EngineSettings) -> Result<Arc<Runtime>, CatalogError> {
    let mut active = PROCESS_RUNTIME
        .lock()
        .map_err(|_| errors::invalid("process runtime lock poisoned"))?;
    if let Some(runtime) = active.as_ref() {
        if runtime.settings.budget != settings.budget
            || runtime.settings.limits.max_object_bytes != settings.limits.max_object_bytes
            || runtime.settings.limits.max_control_bytes != settings.limits.max_control_bytes
            || runtime.settings.limits.envelope != settings.limits.envelope
        {
            return Err(errors::invalid(
                "all inspection stores share the already configured process resource budget",
            ));
        }
        return Ok(Arc::clone(runtime));
    }
    let shared = SharedRuntime::build(settings.budget.clone())
        .map_err(|error| CatalogError::Semantic(Arc::new(error)))?;
    let executor = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(settings.budget.threads.pool_threads.get())
        .max_blocking_threads(1)
        .enable_all()
        .build()
        .map_err(|source| CatalogError::Infrastructure {
            op: "build inspection executor".to_owned(),
            source: Box::new(source),
        })?;
    let registry = Arc::new(
        pse_schema::catalog::assemble().map_err(|error| CatalogError::Semantic(Arc::new(error)))?,
    );
    let reserver: Arc<dyn MemoryReserver> = shared.reserver();
    let sessions = Arc::new(
        pse_catalog::session::SessionFactory::new(
            shared.runtime_env(),
            reserver,
            settings.budget.execution.clone(),
            settings.budget.threads,
            native_engine_profile(),
        )?
        .with_requirement_planner(Arc::new(pse_rules::invariants::RegistryRequirementPlanner)),
    );
    let invariants = pse_rules::validator::InvariantValidator::new(Arc::clone(&registry));
    let validator = Arc::new(
        pse_compiler::validator::CompilerValidator::new(Arc::new(invariants), &registry)
            .map_err(|error| CatalogError::Semantic(Arc::new(error)))?
            .with_sessions(Arc::clone(&sessions)),
    );
    let runtime = Arc::new(Runtime {
        shared,
        executor,
        registry,
        validator,
        sessions,
        settings: settings.clone(),
    });
    *active = Some(Arc::clone(&runtime));
    Ok(runtime)
}
