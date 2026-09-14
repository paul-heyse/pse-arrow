// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Shared engine resources without a dependency on the higher runtime/controller crate.
use super::{EngineProfile, ExecutionSettings, SnapshotSession, ThreadBudget};
use crate::CatalogError;
use datafusion::{arrow::array::RecordBatch, execution::runtime_env::RuntimeEnv};
use pse_ids::{CancellationToken, MemoryReserver};
use pse_schema::{Registry, model::RelationKey};
use std::{collections::BTreeMap, sync::Arc};

/// Reusable session construction settings with one shared deployment memory pool.
#[derive(Debug)]
pub struct SessionFactory {
    runtime: Arc<RuntimeEnv>,
    reserver: Arc<dyn MemoryReserver>,
    settings: ExecutionSettings,
    budget: ThreadBudget,
    profile: EngineProfile,
}
impl SessionFactory {
    /// Open a read-only session over exact admitted snapshots using this factory's
    /// shared runtime, allocator and immutable engine profile.
    /// # Errors
    /// Invalid or ambiguous snapshot bindings, cancellation or configuration failure.
    pub fn open_session(
        &self,
        snapshots: Vec<Arc<crate::Snapshot>>,
        registry: Arc<Registry>,
        cancel: &CancellationToken,
    ) -> Result<SnapshotSession, CatalogError> {
        cancel.checkpoint()?;
        super::build_session(
            snapshots,
            registry,
            Arc::clone(&self.runtime),
            Arc::clone(&self.reserver),
            self.settings.clone(),
            self.budget,
            self.profile.clone(),
        )
    }
    /// Bind the already constructed shared runtime and explicit execution profile.
    /// # Errors
    /// Invalid thread/configuration settings or an unknown explicit rule implementation.
    pub fn new(
        runtime: Arc<RuntimeEnv>,
        reserver: Arc<dyn MemoryReserver>,
        settings: ExecutionSettings,
        budget: ThreadBudget,
        profile: EngineProfile,
    ) -> Result<Self, CatalogError> {
        super::config::build(&settings, budget)?;
        for name in &profile.analyzer_rules {
            super::RuleCatalog::analyzer(name)?;
        }
        for name in &profile.optimizer_rules {
            super::RuleCatalog::optimizer(name)?;
        }
        for name in &profile.physical_optimizer_rules {
            super::RuleCatalog::physical(name)?;
        }
        Ok(Self {
            runtime,
            reserver,
            settings,
            budget,
            profile,
        })
    }
    /// Construct a constraint-free session over the complete actual candidate rows.
    /// # Errors
    /// Schema/value admission, cancellation, reservation or explicit engine configuration.
    pub fn candidate(
        &self,
        rows: BTreeMap<RelationKey, RecordBatch>,
        registry: Arc<Registry>,
        cancel: &CancellationToken,
    ) -> Result<SnapshotSession, CatalogError> {
        cancel.checkpoint()?;
        super::snapshot_session::build_candidate_session_with_cancel(
            rows,
            registry,
            Arc::clone(&self.runtime),
            Arc::clone(&self.reserver),
            self.settings.clone(),
            self.budget,
            self.profile.clone(),
            cancel,
        )
    }
    /// The same allocator used by every session and compiler output.
    pub fn reserver(&self) -> &Arc<dyn MemoryReserver> {
        &self.reserver
    }
}
