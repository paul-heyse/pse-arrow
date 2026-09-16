// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native provider binding and SQL resolution through the common preparation path.

use std::sync::Arc;

use datafusion::{
    catalog::TableProvider,
    common::{TableReference, tree_node::TreeNodeRecursion},
    datasource::source_as_provider,
    logical_expr::LogicalPlan,
};
use pse_ids::CancellationToken;

use super::{PreparedComputation, SnapshotSession, admission, snapshot_session::engine};
use crate::{
    CatalogError,
    provider::binding::{BindingKey, TableBinding},
};

impl SnapshotSession {
    pub(crate) fn with_output_provider(
        &self,
        port: String,
        reference: TableReference,
        provider: Arc<dyn TableProvider>,
    ) -> Result<Self, CatalogError> {
        let mut result = self.clone();
        result
            .bindings
            .insert(
                BindingKey::Output(port),
                TableBinding::new(reference, provider, None, None),
            )
            .map_err(engine)?;
        Ok(result)
    }
    pub(super) async fn capture_namespace(
        &self,
        state: &datafusion::execution::session_state::SessionState,
        created_memory_table: Option<&TableReference>,
        cancel: &CancellationToken,
    ) -> Result<Self, CatalogError> {
        let mut result = self.clone();
        result.bindings = self.bindings.resolutions_only();
        for catalog_name in state.catalog_list().catalog_names() {
            result.bindings.namespace(&catalog_name, None);
            let catalog = state
                .catalog_list()
                .catalog(&catalog_name)
                .ok_or_else(|| invalid("catalog disappeared during namespace capture"))?;
            for schema_name in catalog.schema_names() {
                result.bindings.namespace(&catalog_name, Some(&schema_name));
                let schema = catalog
                    .schema(&schema_name)
                    .ok_or_else(|| invalid("schema disappeared during namespace capture"))?;
                for table_name in schema.table_names() {
                    cancel.checkpoint()?;
                    let provider = schema
                        .table(&table_name)
                        .await
                        .map_err(engine)?
                        .ok_or_else(|| invalid("table disappeared during namespace capture"))?;
                    let reference =
                        TableReference::full(catalog_name.clone(), schema_name.clone(), table_name);
                    let existing: Vec<_> = self
                        .bindings
                        .iter()
                        .filter(|(_, binding)| {
                            binding.reference == reference
                                && Arc::ptr_eq(&binding.provider, &provider)
                        })
                        .collect();
                    if existing.is_empty() && created_memory_table == Some(&reference) {
                        let constraints = provider.constraints().cloned().unwrap_or_default();
                        let captured = result
                            .prepare_capture_constraints(
                                reference.clone(),
                                provider,
                                constraints,
                                false,
                                cancel,
                            )?
                            .execute(cancel)
                            .await?;
                        result = result.with_mutable_capture(
                            &reference,
                            &captured,
                            Arc::new(super::mutation::MemoryTableFactory),
                            cancel,
                        )?;
                    } else if existing.is_empty() {
                        result = result.with_provider(reference, provider, cancel)?;
                    } else {
                        for (key, binding) in existing {
                            result
                                .bindings
                                .insert(key.clone(), binding.clone())
                                .map_err(engine)?;
                        }
                    }
                }
            }
        }
        result.capture_configuration(state);
        Ok(result)
    }
    /// Bind an actual native provider without a concrete implementation roster.
    /// This establishes source membership and schema compatibility, not value/key
    /// validity. Native external sources remain observations until captured/admitted.
    /// # Errors
    /// Ambiguous names, invalid fields, unestablished constraints or foreign view inputs.
    pub fn with_provider(
        &self,
        reference: TableReference,
        provider: Arc<dyn TableProvider>,
        cancel: &CancellationToken,
    ) -> Result<Self, CatalogError> {
        cancel.checkpoint()?;
        if provider
            .constraints()
            .is_some_and(|constraints| !constraints.is_empty())
        {
            return Err(invalid(
                "native source constraints require actual admitted source facts",
            ));
        }
        for field in provider.schema().fields() {
            admission::admit_field(&self.registry, field).map_err(engine)?;
        }
        if let Some(plan) = provider.get_logical_plan() {
            self.derive_plan_fields(plan.into_owned(), cancel)?;
        }
        let mut result = self.clone();
        result
            .bindings
            .insert(
                BindingKey::Native(reference.clone()),
                TableBinding::new(reference, provider, None, None).observed(),
            )
            .map_err(engine)?;
        Ok(result)
    }

    /// Resolve SQL using the exact native session, retaining the providers actually
    /// returned by its registered metadata/table-function factories. The resulting
    /// plan uses the same field/source/effect checks as a directly constructed plan.
    /// # Errors
    /// Native resolution, unsupported effects, unestablished facts or cancellation.
    pub async fn prepare_sql(
        &self,
        sql: &str,
        cancel: &CancellationToken,
    ) -> Result<PreparedComputation, CatalogError> {
        cancel.checkpoint()?;
        let state = self.bound_state()?;
        // Parse once and retain identifiers before native schema DDL flattens them.
        let statement = state
            .sql_to_statement(sql, &state.config_options().sql_parser.dialect)
            .map_err(engine)?;
        let schema_spelling = super::commands::schema_spelling(&statement);
        for reference in state.resolve_table_references(&statement).map_err(engine)? {
            let defaults = &state.config_options().catalog;
            let reference = reference.resolve(&defaults.default_catalog, &defaults.default_schema);
            self.bindings
                .check_resolution(&TableReference::full(
                    reference.catalog,
                    reference.schema,
                    reference.table,
                ))
                .map_err(engine)?;
        }
        let mut plan = state.statement_to_plan(statement).await.map_err(engine)?;
        if let LogicalPlan::Ddl(datafusion::logical_expr::DdlStatement::CreateCatalogSchema(
            command,
        )) = &mut plan
            && let Some(spelling) = schema_spelling
        {
            command.schema_name = spelling;
        }
        let mut resolved = self.clone();
        plan.apply_with_subqueries(|node| {
            if let LogicalPlan::TableScan(scan) = node {
                let provider = source_as_provider(&scan.source)?;
                if !resolved.bindings.contains_provider(&provider) {
                    let defaults = &state.config_options().catalog;
                    let reference = scan
                        .table_name
                        .clone()
                        .resolve(&defaults.default_catalog, &defaults.default_schema);
                    let reference =
                        TableReference::full(reference.catalog, reference.schema, reference.table);
                    resolved = resolved
                        .with_provider(reference.clone(), provider, cancel)
                        .map_err(|error| {
                            datafusion::common::DataFusionError::External(Box::new(error))
                        })?;
                    if state.config_options().catalog.information_schema
                        && reference.schema() == Some("information_schema")
                    {
                        // DataFusion resolves this schema from this private catalog list.
                        // The provider still passed the same field/source admission.
                        resolved.bindings.resolved_metadata(&reference);
                    }
                }
            }
            Ok(TreeNodeRecursion::Continue)
        })
        .map_err(engine)?;
        resolved.prepare(plan, cancel)
    }
}

fn invalid(reason: &str) -> CatalogError {
    CatalogError::Admission {
        path: "provider.native".to_owned(),
        reason: reason.to_owned(),
    }
}
