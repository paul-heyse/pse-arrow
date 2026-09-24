// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Shared resources and policy; native builders retain each operation's semantics.
use datafusion::{
    common::Result,
    execution::session_state::SessionState,
    logical_expr::{Expr, LogicalPlan},
    prelude::DataFrame,
};
use deltalake::{
    DeltaTable,
    delta_datafusion::SessionFallbackPolicy,
    kernel::transaction::CommitProperties,
    operations::{
        delete::DeleteBuilder, merge::MergeBuilder, optimize::OptimizeBuilder,
        update::UpdateBuilder, vacuum::VacuumBuilder, write::WriteBuilder,
    },
    protocol::SaveMode,
};
use pse_engine::session::execution::NativeExecutionContext;
use std::sync::Arc;

#[derive(Clone, Copy, Debug)]
pub(super) enum CommitKind {
    Data,
    Publication,
    Maintenance,
}

pub(super) fn commit_policy(commit: CommitProperties, kind: CommitKind) -> CommitProperties {
    let commit = commit
        .with_max_retries(0)
        .with_cleanup_expired_logs(Some(false));
    match kind {
        CommitKind::Data => commit,
        CommitKind::Publication => commit.with_create_checkpoint(true),
        CommitKind::Maintenance => commit.with_create_checkpoint(false),
    }
}

/// One invocation's actual planner, functions, scoped pool and settlement owner.
#[derive(Debug)]
pub(super) struct DeltaOperationContext {
    pub state: Arc<SessionState>,
    pub services: Arc<NativeExecutionContext>,
    commit: CommitProperties,
    contract: Option<super::contract::DeclaredCheck>,
}
impl DeltaOperationContext {
    pub(super) fn new(
        state: Arc<SessionState>,
        contract: Option<super::contract::DeclaredCheck>,
        commit: CommitProperties,
        kind: CommitKind,
    ) -> Result<Self> {
        let state = match &contract {
            Some(contract) => Arc::new(contract.bind(&state)?),
            None => state,
        };
        let services = NativeExecutionContext::from_session(state.as_ref())?;
        Ok(Self {
            state,
            services,
            contract,
            commit: commit_policy(commit, kind),
        })
    }
    /// Reuse the bound session/contract when one attempt advances its receipt phase.
    pub(super) fn with_commit(&self, commit: CommitProperties, kind: CommitKind) -> Self {
        Self {
            state: self.state.clone(),
            services: self.services.clone(),
            contract: self.contract.clone(),
            commit: commit_policy(commit, kind),
        }
    }
    pub(super) async fn begin(
        &self,
        table: &DeltaTable,
    ) -> Result<Option<Arc<super::lease::ReadLease>>> {
        if let Some(contract) = &self.contract
            && table.version().is_some()
        {
            contract.verify(table)?;
        }
        let lease = super::lease::write(table.table_url(), self.services.cancellation()).await?;
        self.services.require_settlement();
        Ok(lease)
    }
    pub(super) fn write(
        &self,
        table: DeltaTable,
        input: LogicalPlan,
        mode: SaveMode,
    ) -> WriteBuilder {
        table
            .write(Vec::<datafusion::arrow::array::RecordBatch>::new())
            .with_input_plan(input)
            .with_save_mode(mode)
            .with_session_state(self.state.clone())
            .with_session_fallback_policy(SessionFallbackPolicy::RequireSessionState)
            .with_cast_safety(false)
            .with_commit_properties(self.commit.clone())
    }
    pub(super) fn update(&self, table: DeltaTable) -> UpdateBuilder {
        table
            .update()
            .with_session_state(self.state.clone())
            .with_session_fallback_policy(SessionFallbackPolicy::RequireSessionState)
            .with_safe_cast(false)
            .with_commit_properties(self.commit.clone())
    }
    pub(super) fn delete(&self, table: DeltaTable) -> DeleteBuilder {
        table
            .delete()
            .with_session_state(self.state.clone())
            .with_session_fallback_policy(SessionFallbackPolicy::RequireSessionState)
            .with_commit_properties(self.commit.clone())
    }
    pub(super) fn merge(&self, table: DeltaTable, source: DataFrame, on: Expr) -> MergeBuilder {
        table.merge(source, on).with_source_alias("source").with_target_alias("target")
            .with_session_state(self.state.clone())
            .with_session_fallback_policy(SessionFallbackPolicy::RequireSessionState)
            // Invocation streams cannot be consumed again for a statistics pass.
            .with_streaming(true).with_merge_schema(false).with_safe_cast(false)
            .with_commit_properties(self.commit.clone())
    }
    pub(super) fn optimize(&self, table: DeltaTable) -> OptimizeBuilder<'static> {
        table
            .optimize()
            .with_session_state(self.state.clone())
            .with_session_fallback_policy(SessionFallbackPolicy::RequireSessionState)
            .with_commit_properties(self.commit.clone())
    }
    pub(super) fn vacuum(&self, table: DeltaTable) -> VacuumBuilder {
        table
            .vacuum()
            .with_dry_run(false)
            .with_enforce_retention_duration(true)
            .with_commit_properties(self.commit.clone())
    }
}
