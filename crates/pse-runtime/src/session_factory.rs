// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Session factories retain the deployment's one engine pool and platform pool.

use crate::{RuntimeError, SharedRuntime};
use pse_engine::session::{EngineFactory, EngineProfile};
use std::sync::Arc;

impl SharedRuntime {
    /// Construct session handles without creating a second runtime or allocation budget.
    /// # Errors
    /// The explicit engine profile or bound execution configuration is invalid.
    pub fn session_factory(&self, profile: EngineProfile) -> Result<EngineFactory, RuntimeError> {
        let factory = EngineFactory::new(
            self.runtime_env(),
            self.pool(),
            self.budget().execution.clone(),
            self.budget().threads,
            profile,
        )?
        .with_cache_service(self.caches().native().clone())
        .with_extension(self.caches().clone())
        .with_extension(Arc::new(pse_engine::resources::CpuAdmission {
            permits: self.compiler_cpu(),
            workers: self.budget().threads.pool_threads.try_into().map_err(|_| {
                RuntimeError::Internal {
                    message: "worker count exceeds CPU admission width".into(),
                }
            })?,
        }))
        .with_extension(Arc::new(pse_engine::session::round::RoundResetSupport(
            pse_catalog::assembly::supports_round_reset,
        )))
        .with_query_planner(Arc::new(pse_engine::session::planner::UnifiedPlanner::new(
            pse_catalog::assembly::planners(),
        )));
        Ok(factory)
    }
}
