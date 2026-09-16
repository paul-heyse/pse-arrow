// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Owned intermediate native results. They declare no persisted relation or keys.

use super::{CompletedComputation, SnapshotSession};
use crate::{
    CatalogError,
    provider::binding::{BindingKey, TableBinding},
};
use datafusion::{
    catalog::TableProvider,
    common::{Constraints, Result, TableReference},
    datasource::provider_as_source,
    logical_expr::{LogicalPlan, LogicalPlanBuilder, TableSource},
};
use pse_ids::CancellationToken;
use std::{collections::BTreeMap, sync::Arc};

impl SnapshotSession {
    /// Bind complete native computations under explicit, non-replacing source roles.
    /// Their actual prepared schemas remain native intermediate schemas. This does
    /// not invent registry relations or claim PK/FK constraints for temporary data.
    /// # Errors
    /// Empty/repeated role, different registry authority, cancellation or native schema.
    pub fn with_computation_roles(
        &self,
        completed: BTreeMap<String, CompletedComputation>,
        cancel: &CancellationToken,
    ) -> Result<Self, CatalogError> {
        let mut result = self.clone();
        for (role, completed) in completed {
            cancel.checkpoint()?;
            if role.is_empty() || result.bindings.computation(&role).is_some() {
                return Err(invalid("computed role is empty or already bound"));
            }
            completed.prepared().check_registry(&self.registry)?;
            let schema = super::query_schema::schema(
                completed.prepared().optimized_plan().schema().as_arrow(),
            );
            let batches = completed
                .batches()
                .iter()
                .cloned()
                .map(|batch| batch.with_schema_metadata(schema.metadata().clone()))
                .collect::<Result<Vec<_>, _>>()
                .map_err(CatalogError::from)?;
            // Buffer leases survive independently; a materialized provider does not
            // retain the producer's plans, sessions or captured source graph.
            let table: Arc<dyn TableProvider> = Arc::new(super::materialized::MaterializedTable {
                schema,
                batches,
                defaults: BTreeMap::new(),
                constraints: Constraints::default(),
            });
            result
                .bindings
                .insert(
                    BindingKey::Computation(role.clone()),
                    TableBinding::new(
                        TableReference::full("computations", "inputs", role),
                        table,
                        None,
                        None,
                    ),
                )
                .map_err(super::snapshot_session::engine)?;
        }
        Ok(result)
    }
    /// Actual retained source of a complete native intermediate computation.
    /// # Errors
    /// The requested role was not bound.
    pub fn computation_source(&self, role: &str) -> Result<Arc<dyn TableSource>, CatalogError> {
        self.bindings
            .computation(role)
            .map(|binding| provider_as_source(Arc::clone(&binding.provider)))
            .ok_or_else(|| invalid("computed role is absent"))
    }
    /// Native scan of an explicit intermediate role.
    /// # Errors
    /// Missing role or native schema binding error.
    pub fn scan_computation_role(&self, role: &str) -> Result<LogicalPlan, CatalogError> {
        let binding = self
            .bindings
            .computation(role)
            .ok_or_else(|| invalid("computed role is absent"))?;
        LogicalPlanBuilder::scan(
            binding.reference.clone(),
            self.computation_source(role)?,
            None,
        )
        .and_then(LogicalPlanBuilder::build)
        .map_err(super::snapshot_session::engine)
    }
    /// Complete intermediate input inventory retained before optimization.
    pub fn computation_roles(&self) -> impl Iterator<Item = &str> {
        self.bindings.iter().filter_map(|(slot, _)| match slot {
            BindingKey::Computation(role) => Some(role.as_str()),
            _ => None,
        })
    }
}
fn invalid(reason: &str) -> CatalogError {
    CatalogError::Admission {
        path: "session.computation_roles".to_owned(),
        reason: reason.to_owned(),
    }
}
