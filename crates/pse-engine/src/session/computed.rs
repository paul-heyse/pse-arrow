// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Owned intermediate native results. They declare no persisted relation or keys.

use super::{CompletedComputation, EngineSession};
use crate::{
    EngineError,
    provider::binding::{BindingKey, TableBinding},
};
use datafusion::{
    catalog::TableProvider,
    common::{Constraints, Result, TableReference},
    datasource::provider_as_source,
    logical_expr::{LogicalPlan, LogicalPlanBuilder, TableSource},
};
use pse_columnar::CancellationToken;
use std::{collections::BTreeMap, sync::Arc};

impl EngineSession {
    /// Bind an owned Arrow argument without advertising relation validity or keys.
    /// Native field predicates check actual values under the caller's session. This
    /// accepts co-located algorithm values and support without a positional side table.
    /// # Errors
    /// Empty/repeated role, invalid local fields/values, or cancellation.
    pub async fn with_columnar_argument(
        &self,
        role: impl Into<String>,
        batch: pse_columnar::owned_buffer::OwnedRecordBatch,
        cancel: &CancellationToken,
    ) -> Result<Self, EngineError> {
        cancel.checkpoint()?;
        let role = role.into();
        if role.is_empty() || self.bindings.computation(&role).is_some() {
            return Err(invalid("columnar argument role is empty or already bound"));
        }
        let scratch = pse_columnar::MemoryConsumer::new("session:columnar-argument-admission")
            .register(&self.pool);
        scratch.try_grow(pse_columnar::schema_working_extent(batch.schema_ref())?)?;
        for field in batch.schema().fields() {
            cancel.checkpoint()?;
            super::admission::admit_field(&self.registry, field)
                .map_err(super::engine_session::engine)?;
        }
        drop(scratch);
        let table: Arc<dyn TableProvider> = Arc::new(super::materialized::ImmutableTable {
            schema: batch.schema(),
            data: vec![batch],
            defaults: BTreeMap::new(),
            constraints: Some(Constraints::default()),
        });
        let mut result = self.clone();
        result
            .bindings
            .insert(
                BindingKey::Computation(role.clone()),
                TableBinding::new(
                    TableReference::full("arguments", "columnar", role.clone()),
                    table,
                    None,
                    None,
                ),
            )
            .map_err(super::engine_session::engine)?;
        let violations = super::checks::local_field_violations(
            &self.registry,
            result.scan_computation_role(&role)?,
        )
        .map_err(super::engine_session::engine)?;
        let checked = result
            .prepare_rule_plan(violations, cancel)?
            .execute(cancel)
            .await?;
        if checked.batches().iter().any(|batch| batch.num_rows() != 0) {
            return Err(invalid("columnar argument violates local field contracts"));
        }
        Ok(result)
    }

    /// Bind complete native computations under explicit, non-replacing source roles.
    /// Their actual prepared schemas remain native intermediate schemas. This does
    /// not invent registry relations or claim PK/FK constraints for temporary data.
    /// # Errors
    /// Empty/repeated role, different registry authority, cancellation or native schema.
    pub fn with_computation_roles(
        &self,
        completed: BTreeMap<String, CompletedComputation>,
        cancel: &CancellationToken,
    ) -> Result<Self, EngineError> {
        let mut captures = BTreeMap::new();
        for (role, completed) in completed {
            cancel.checkpoint()?;
            completed.prepared().check_registry(&self.registry)?;
            captures.insert(
                role,
                super::CapturedComputation {
                    schema: completed.prepared().optimized_plan().schema().clone(),
                    batches: completed.batches().to_vec(),
                    registry: self.registry.clone(),
                },
            );
        }
        self.with_captured_computations(captures, cancel)
    }

    /// Bind actual composed child captures without a second execution or relation claim.
    /// # Errors
    /// A foreign registry, duplicate role or cancellation.
    pub fn with_captured_computations(
        &self,
        completed: BTreeMap<String, super::CapturedComputation>,
        cancel: &CancellationToken,
    ) -> Result<Self, EngineError> {
        let mut result = self.clone();
        for (role, captured) in completed {
            cancel.checkpoint()?;
            if role.is_empty()
                || result.bindings.computation(&role).is_some()
                || !Arc::ptr_eq(&self.registry, &captured.registry)
            {
                return Err(invalid("composed query role or registry differs"));
            }
            let ownership = pse_columnar::owned_buffer::AllocationScope::default();
            for batch in &captured.batches {
                ownership.import(batch)?;
            }
            let fields = captured
                .schema
                .fields()
                .iter()
                .enumerate()
                .map(|(index, field)| {
                    field
                        .as_ref()
                        .clone()
                        .with_name(format!("__composed_{index}"))
                })
                .collect::<Vec<_>>();
            let schema = Arc::new(datafusion::arrow::datatypes::Schema::new(fields));
            let batches = captured
                .batches
                .into_iter()
                .map(|batch| batch.with_schema(schema.clone()))
                .collect::<std::result::Result<Vec<_>, _>>()?;
            let table: Arc<dyn TableProvider> = Arc::new(super::materialized::ImmutableTable {
                schema,
                data: batches,
                defaults: BTreeMap::new(),
                constraints: Some(Constraints::default()),
            });
            let scan = LogicalPlanBuilder::scan(
                TableReference::full("captured", "inputs", role.clone()),
                provider_as_source(table.clone()),
                None,
            )
            .and_then(LogicalPlanBuilder::build)
            .map_err(super::engine)?;
            let expressions = captured
                .schema
                .iter()
                .enumerate()
                .map(|(index, (qualifier, field))| {
                    datafusion::logical_expr::col(format!("__composed_{index}"))
                        .alias_qualified(qualifier.cloned(), field.name())
                })
                .collect();
            let plan = LogicalPlan::Projection(
                datafusion::logical_expr::Projection::try_new(expressions, Arc::new(scan))
                    .map_err(super::engine)?,
            );
            let view: Arc<dyn TableProvider> =
                Arc::new(datafusion::datasource::ViewTable::new(plan, None));
            let mut binding = TableBinding::new(
                TableReference::full("computations", "inputs", role.clone()),
                view,
                None,
                None,
            );
            binding.ownership = Some(ownership);
            binding.dependencies.push(table);
            result
                .bindings
                .insert(BindingKey::Computation(role), binding)
                .map_err(super::engine)?;
        }
        Ok(result)
    }

    /// Actual retained source of a complete native intermediate computation.
    /// # Errors
    /// The requested role was not bound.
    pub fn computation_source(&self, role: &str) -> Result<Arc<dyn TableSource>, EngineError> {
        self.bindings
            .computation(role)
            .map(|binding| provider_as_source(Arc::clone(&binding.provider)))
            .ok_or_else(|| invalid("computed role is absent"))
    }
    /// Native scan of an explicit intermediate role.
    /// # Errors
    /// Missing role or native schema binding error.
    pub fn scan_computation_role(&self, role: &str) -> Result<LogicalPlan, EngineError> {
        let binding = self
            .bindings
            .computation(role)
            .ok_or_else(|| invalid("computed role is absent"))?;
        if let Some(plan) = binding.provider.get_logical_plan() {
            return Ok(plan.into_owned());
        }
        LogicalPlanBuilder::scan(
            binding.reference.clone(),
            self.computation_source(role)?,
            None,
        )
        .and_then(LogicalPlanBuilder::build)
        .map_err(super::engine_session::engine)
    }
    /// Complete intermediate input inventory retained before optimization.
    pub fn computation_roles(&self) -> impl Iterator<Item = &str> {
        self.bindings.iter().filter_map(|(slot, _)| match slot {
            BindingKey::Computation(role) => Some(role.as_str()),
            _ => None,
        })
    }
}
fn invalid(reason: &str) -> EngineError {
    EngineError::Admission {
        path: "session.computation_roles".to_owned(),
        reason: reason.to_owned(),
    }
}
