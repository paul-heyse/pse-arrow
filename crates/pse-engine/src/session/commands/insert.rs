// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native SQL owns omitted-column defaults and coercions for every insert source.

use crate::{
    EngineError,
    provider::binding::{BindingKey, TableBinding},
    session::{EngineSession, PreparedComputation, engine},
};
use datafusion::{
    common::TableReference,
    datasource::ViewTable,
    logical_expr::{LogicalPlan, dml::InsertOp},
};
use pse_columnar::CancellationToken;
use std::sync::Arc;

impl EngineSession {
    /// Prepare an insertion from a native plan, matching its named fields to the
    /// target. Arrow sources enter as admitted native providers. Omitted fields,
    /// explicit NULL, coercions and default volatility follow DataFusion SQL.
    /// Derived fields remain native expressions or views, not a second write engine.
    /// No rows execute and no private input name enters the resulting namespace.
    /// The caller's purpose and target policy must already permit the operation.
    /// # Errors
    /// Unknown/ambiguous columns, foreign inputs, native planning or policy refusal.
    pub async fn prepare_insert(
        &self,
        target: TableReference,
        input: LogicalPlan,
        operation: InsertOp,
        cancel: &CancellationToken,
    ) -> Result<PreparedComputation, EngineError> {
        cancel.checkpoint()?;
        let input = self.derive_plan_fields(input, cancel)?;
        let columns = input
            .schema()
            .fields()
            .iter()
            .map(|field| TableReference::bare(field.name().clone()).to_quoted_string())
            .collect::<Vec<_>>()
            .join(", ");
        let mut planning = self.clone();
        let state = self.bound_state()?;
        let defaults = &state.config_options().catalog;
        let target = target.resolve(&defaults.default_catalog, &defaults.default_schema);
        let target = TableReference::full(target.catalog, target.schema, target.table);
        self.bindings.check_resolution(&target).map_err(engine)?;
        let temporary = TableReference::full(
            defaults.default_catalog.clone(),
            defaults.default_schema.clone(),
            format!("pse_insert_{}", uuid::Uuid::new_v4().simple()),
        );
        planning
            .bindings
            .insert(
                BindingKey::Computation(temporary.to_string()),
                TableBinding::new(
                    temporary.clone(),
                    Arc::new(ViewTable::new(input, None)),
                    None,
                    None,
                ),
            )
            .map_err(engine)?;
        let command = match operation {
            InsertOp::Append => "INSERT INTO",
            InsertOp::Overwrite => "INSERT OVERWRITE",
            InsertOp::Replace => "REPLACE INTO",
        };
        let sql = format!(
            "{command} {} ({columns}) SELECT * FROM {}",
            target.to_quoted_string(),
            temporary.to_quoted_string(),
        );
        let state = planning.bound_state()?;
        let statement =
            crate::cache_service::syntax::parse(&state, &sql, cancel).map_err(engine)?;
        let plan = state.statement_to_plan(statement).await.map_err(engine)?;
        // Native scan construction expands ViewTable into its actual child and
        // alias. Admit that graph against the original generation, excluding the
        // temporary planner namespace from execution and dependency authority.
        self.source_bindings(&plan, cancel)?;
        self.prepare(plan, cancel)
    }
}

#[cfg(test)]
mod tests;
