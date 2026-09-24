// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit durable engine composition.
use std::sync::Arc;
/// Actual Delta and publication extension planners.
pub fn planners() -> Vec<Arc<dyn datafusion::physical_planner::ExtensionPlanner + Send + Sync>> {
    vec![deltalake::delta_datafusion::planner::DeltaExtensionPlanner::new()]
}
/// Admitted reset support for immutable selected source implementations.
pub fn supports_round_reset(plan: &dyn datafusion::physical_plan::ExecutionPlan) -> bool {
    plan.is::<deltalake::delta_datafusion::DeltaScanExec>()
        || crate::delta::leased::supports_round_reset(plan)
        || crate::cache_service::resident::supports_round_reset(plan)
}
