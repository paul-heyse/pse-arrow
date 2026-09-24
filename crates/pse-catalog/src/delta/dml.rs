// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native SQL mutation hooks for editable Delta tables. Published semantic views
//! remain immutable. This provider exposes Delta's storage schema; publication
//! admission establishes the cross-relation and domain contract separately.
mod execution;
mod merge;

use datafusion::{
    arrow::datatypes::SchemaRef,
    catalog::{Session, TableProvider},
    common::{DFSchemaRef, DataFusionError, Result, Statistics},
    execution::session_state::SessionState,
    logical_expr::{
        Expr, TableProviderFilterPushDown, TableType,
        dml::{InsertOp, MergeIntoClause},
    },
    physical_plan::ExecutionPlan,
};
use deltalake::{DeltaTable, kernel::transaction::CommitProperties, protocol::SaveMode};
use std::sync::Arc;

/// An exact Delta snapshot with native validating mutation hooks. Rebind explicitly
/// after a command to select a new version; existing readers retain their snapshot.
/// This is an editable attempt table, never an admission proof or a published view.
#[derive(Debug)]
pub struct WritableTable {
    table: DeltaTable,
    scan: Arc<dyn TableProvider>,
    lease: Option<Arc<super::lease::ReadLease>>,
    commit: CommitProperties,
    contract: Option<super::contract::DeclaredCheck>,
}
impl WritableTable {
    /// Bind an already opened Delta version to its native scan. No data is mutated.
    /// Every command uses the actual session supplied by native physical planning.
    /// # Errors
    /// The table has no loaded version or native scan construction fails.
    pub async fn new(
        table: DeltaTable,
        state: Arc<SessionState>,
        commit: CommitProperties,
    ) -> Result<Self> {
        if table.version().is_none() {
            return Err(DataFusionError::Plan(
                "editable Delta table must be loaded".into(),
            ));
        }
        let lease =
            super::lease::read(table.table_url(), &pse_columnar::CancellationToken::new()).await?;
        let scan = super::leased::retain(
            Arc::new(table.table_provider().with_session(state).build().await?),
            lease.clone(),
        );
        Ok(Self {
            table,
            scan,
            lease,
            commit,
            contract: None,
        })
    }
    /// Bind an editable declared relation, enforcing its fingerprint-bound CHECK
    /// through every native Delta mutation builder.
    /// # Errors
    /// Missing/different contract, incompatible table schema or scan construction.
    pub async fn declared(
        table: DeltaTable,
        state: Arc<SessionState>,
        commit: CommitProperties,
        contract: super::contract::DeclaredCheck,
    ) -> Result<Self> {
        contract.verify(&table)?;
        let mut result = Self::new(table, state, commit).await?;
        pse_schema::field_contract::delta_scan_schema(
            &result.schema(),
            contract.layout().storage_schema(),
        )
        .map_err(|e| DataFusionError::External(Box::new(e)))?;
        result.contract = Some(contract);
        Ok(result)
    }
    fn command(
        &self,
        state: &dyn Session,
        command: execution::Command,
        children: Vec<Arc<dyn ExecutionPlan>>,
    ) -> Result<Arc<dyn ExecutionPlan>> {
        let state = state
            .as_any()
            .downcast_ref::<SessionState>()
            .ok_or_else(|| {
                DataFusionError::Plan("Delta mutation requires the actual SessionState".into())
            })?;
        let state = state.clone();
        Ok(super::leased::retain_execution(
            execution::plan(
                self.table.clone(),
                Arc::new(state),
                self.commit.clone(),
                self.contract.clone(),
                command,
                children,
            )?,
            self.lease.clone(),
        ))
    }
}
#[async_trait::async_trait]
impl TableProvider for WritableTable {
    fn schema(&self) -> SchemaRef {
        self.scan.schema()
    }
    fn table_type(&self) -> TableType {
        TableType::Base
    }
    fn statistics(&self) -> Option<Statistics> {
        self.scan.statistics()
    }
    fn constraints(&self) -> Option<&datafusion::common::Constraints> {
        self.scan.constraints()
    }
    fn get_column_default(&self, column: &str) -> Option<&Expr> {
        self.scan.get_column_default(column)
    }
    fn supports_filters_pushdown(
        &self,
        filters: &[&Expr],
    ) -> Result<Vec<TableProviderFilterPushDown>> {
        self.scan.supports_filters_pushdown(filters)
    }
    async fn scan(
        &self,
        state: &dyn Session,
        projection: Option<&Vec<usize>>,
        filters: &[Expr],
        limit: Option<usize>,
    ) -> Result<Arc<dyn ExecutionPlan>> {
        self.scan.scan(state, projection, filters, limit).await
    }
    async fn insert_into(
        &self,
        state: &dyn Session,
        input: Arc<dyn ExecutionPlan>,
        op: InsertOp,
    ) -> Result<Arc<dyn ExecutionPlan>> {
        let mode = match op {
            InsertOp::Append => SaveMode::Append,
            InsertOp::Overwrite => SaveMode::Overwrite,
            InsertOp::Replace => {
                return Err(DataFusionError::Plan(
                    "REPLACE requires an explicit MERGE key and clauses".into(),
                ));
            }
        };
        self.command(state, execution::Command::Insert(mode), vec![input])
    }
    async fn delete_from(
        &self,
        state: &dyn Session,
        filters: Vec<Expr>,
    ) -> Result<Arc<dyn ExecutionPlan>> {
        self.command(state, execution::Command::Delete(filters), vec![])
    }
    async fn truncate(&self, state: &dyn Session) -> Result<Arc<dyn ExecutionPlan>> {
        self.delete_from(state, vec![]).await
    }
    async fn update(
        &self,
        state: &dyn Session,
        assignments: Vec<(String, Expr)>,
        filters: Vec<Expr>,
    ) -> Result<Arc<dyn ExecutionPlan>> {
        self.command(
            state,
            execution::Command::Update {
                assignments,
                filters,
            },
            vec![],
        )
    }
    async fn merge_into(
        &self,
        state: &dyn Session,
        source: Arc<dyn ExecutionPlan>,
        schema: DFSchemaRef,
        on: Expr,
        clauses: Vec<MergeIntoClause>,
    ) -> Result<Arc<dyn ExecutionPlan>> {
        let command = merge::bind(&schema, self.schema().fields().len(), on, clauses)?;
        self.command(state, command, vec![source])
    }
}
