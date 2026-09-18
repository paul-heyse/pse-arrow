// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! One native planner composes Delta and application extension planners.
use datafusion::{
    catalog::Session,
    common::Result,
    execution::context::QueryPlanner,
    logical_expr::LogicalPlan,
    physical_plan::ExecutionPlan,
    physical_planner::{DefaultPhysicalPlanner, ExtensionPlanner, PhysicalPlanner},
};
use std::sync::Arc;

/// Native physical planning with Delta's public extensions and domain extensions.
/// Configuration, rules and resource ownership come from the actual caller session.
pub struct UnifiedPlanner {
    extensions: Vec<Arc<dyn ExtensionPlanner + Send + Sync>>,
}
impl std::fmt::Debug for UnifiedPlanner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UnifiedPlanner")
            .field("extensions", &self.extensions.len())
            .finish()
    }
}
impl UnifiedPlanner {
    /// Compose additional domain extensions after the built-in Delta integration.
    #[must_use]
    pub fn new(extensions: Vec<Arc<dyn ExtensionPlanner + Send + Sync>>) -> Self {
        let mut all: Vec<Arc<dyn ExtensionPlanner + Send + Sync>> = vec![
            deltalake::delta_datafusion::planner::DeltaExtensionPlanner::new(),
            Arc::new(crate::delta::write::WritePlanner),
            Arc::new(crate::delta::publish::PublishPlanner),
            Arc::new(crate::delta::maintenance::MaintenancePlanner),
            Arc::new(super::contract::ContractPlanner),
            Arc::new(super::cache::CachePlanner),
            Arc::new(super::commands::deferred::CommandPlanner),
        ];
        all.extend(extensions);
        Self { extensions: all }
    }
}
impl Default for UnifiedPlanner {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}
#[async_trait::async_trait]
impl QueryPlanner for UnifiedPlanner {
    async fn create_physical_plan(
        &self,
        logical_plan: &LogicalPlan,
        session: &dyn Session,
    ) -> Result<Arc<dyn ExecutionPlan>> {
        let plan = super::cache::reuse_completed(logical_plan, session)?;
        let planner = DefaultPhysicalPlanner::with_extension_planners(self.extensions.clone());
        let plan = super::cache::physical::prepare(plan, session, &planner).await?;
        let physical = planner.create_physical_plan(&plan, session).await?;
        super::cache::record_physical(&physical, session)?;
        Ok(physical)
    }
}
