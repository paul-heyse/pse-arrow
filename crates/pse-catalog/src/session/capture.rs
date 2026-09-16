// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! General provider capture: observe actual rows before exposing optimizer facts.

mod native;
mod tables;
mod values;
use super::materialized::MaterializedTable;
use super::{PreparedComputation, SnapshotSession};
use crate::{
    CatalogError,
    provider::binding::{BindingKey, TableBinding},
};
use datafusion::{
    arrow::datatypes::SchemaRef,
    catalog::TableProvider,
    common::{Constraint, Constraints, TableReference},
    datasource::provider_as_source,
    logical_expr::{Expr, LogicalPlan, LogicalPlanBuilder},
};
use pse_ids::CancellationToken;
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};
use tables::ObservationTable;

/// A source capture prepared without scanning data or trusting advertised constraints.
#[derive(Debug)]
pub struct PreparedProviderCapture {
    native: PreparedComputation,
    schema: SchemaRef,
    defaults: BTreeMap<String, Expr>,
    constraints: Constraints,
}
impl PreparedProviderCapture {
    /// Actual observed source and native capture contract, before row execution.
    pub const fn computation(&self) -> &PreparedComputation {
        &self.native
    }
    /// Materialize one finite source and establish its advertised primary/unique keys.
    /// # Errors
    /// Late source failure, invalid fields/keys, cancellation or resource refusal.
    pub async fn execute(
        self,
        cancel: &CancellationToken,
    ) -> Result<CapturedProvider, CatalogError> {
        let registry = Arc::clone(self.native.bound_session().registry());
        let stream = self.native.execute_stream(cancel).await?;
        if !stream.is_bounded() {
            return Err(invalid("snapshot capture requires a bounded native source"));
        }
        let completion = stream.collect(cancel).await?;
        let table = Arc::new(MaterializedTable {
            schema: self.schema,
            defaults: self.defaults,
            constraints: self.constraints,
            batches: completion.batches().to_vec(),
        });
        Ok(CapturedProvider { table, registry })
    }
}

/// Actual immutable captured buffers and their completed native validation.
/// Construction is private; metadata or caller-created assertions cannot mint this owner.
#[derive(Debug)]
pub struct CapturedProvider {
    table: Arc<MaterializedTable>,
    registry: Arc<pse_schema::Registry>,
}
impl CapturedProvider {
    /// Completed captured buffers. Their leases are independent of producing plans.
    pub fn batches(&self) -> &[pse_ids::owned_buffer::OwnedRecordBatch] {
        &self.table.batches
    }
    /// The read-only provider over actual captured values and established constraints.
    pub fn provider(&self) -> Arc<dyn TableProvider> {
        self.table.clone()
    }
}

impl SnapshotSession {
    /// Prepare a general native provider for explicit snapshot capture and key admission.
    /// Source constraints and inline plans are withheld until actual rows establish them.
    /// # Errors
    /// Invalid field/constraint declarations, conflicting source name or policy refusal.
    pub fn prepare_provider_capture(
        &self,
        reference: TableReference,
        provider: Arc<dyn TableProvider>,
        cancel: &CancellationToken,
    ) -> Result<PreparedProviderCapture, CatalogError> {
        if let Some(plan) = provider.get_logical_plan() {
            self.derive_plan_fields(plan.into_owned(), cancel)?;
        }
        let constraints = provider.constraints().cloned().unwrap_or_default();
        self.prepare_capture_constraints(reference, provider, constraints, true, cancel)
    }

    pub(super) fn prepare_capture_constraints(
        &self,
        reference: TableReference,
        provider: Arc<dyn TableProvider>,
        constraints: Constraints,
        observed: bool,
        cancel: &CancellationToken,
    ) -> Result<PreparedProviderCapture, CatalogError> {
        let schema = provider.schema();
        for constraint in constraints.iter() {
            let indices = match constraint {
                Constraint::PrimaryKey(indices) | Constraint::Unique(indices) => indices,
            };
            if indices.is_empty()
                || indices.iter().any(|index| *index >= schema.fields().len())
                || indices.iter().copied().collect::<BTreeSet<_>>().len() != indices.len()
            {
                return Err(invalid(
                    "constraint indices are empty, repeated or outside the schema",
                ));
            }
        }
        let defaults = schema
            .fields()
            .iter()
            .filter_map(|field| {
                provider
                    .get_column_default(field.name())
                    .map(|value| (field.name().to_owned(), value.clone()))
            })
            .collect();
        let source: Arc<dyn TableProvider> = Arc::new(ObservationTable { provider, schema });
        let mut session = self.with_provider(reference.clone(), Arc::clone(&source), cancel)?;
        if !observed {
            session.bindings.resolved_metadata(&reference);
        }
        let plan = native::plan(
            scan(reference, source)?,
            Arc::clone(session.registry()),
            &constraints,
            cancel.clone(),
        )
        .map_err(super::engine)?;
        let schema = Arc::new(plan.schema().as_arrow().clone());
        let native = session.prepare_rule_plan(plan, cancel)?;
        Ok(PreparedProviderCapture {
            native,
            schema,
            defaults,
            constraints,
        })
    }
    /// Bind actual captured values without resubmitting unverified constraint metadata.
    /// # Errors
    /// Conflicting namespace ownership or fields incompatible with this registry.
    pub fn with_captured_provider(
        &self,
        reference: TableReference,
        captured: &CapturedProvider,
        cancel: &CancellationToken,
    ) -> Result<Self, CatalogError> {
        cancel.checkpoint()?;
        if !Arc::ptr_eq(&self.registry, &captured.registry) {
            return Err(invalid(
                "captured fields belong to a different actual registry",
            ));
        }
        for field in captured.table.schema().fields() {
            super::admission::admit_field(&self.registry, field).map_err(super::engine)?;
        }
        let mut result = self.clone();
        result
            .bindings
            .insert(
                BindingKey::Native(reference.clone()),
                TableBinding::new(reference, captured.provider(), None, None),
            )
            .map_err(super::engine)?;
        Ok(result)
    }
}

fn scan(
    reference: TableReference,
    provider: Arc<dyn TableProvider>,
) -> Result<LogicalPlan, CatalogError> {
    LogicalPlanBuilder::scan(reference, provider_as_source(provider), None)
        .and_then(LogicalPlanBuilder::build)
        .map_err(super::engine)
}
fn invalid(reason: &str) -> CatalogError {
    CatalogError::Admission {
        path: "provider.capture".into(),
        reason: reason.into(),
    }
}
