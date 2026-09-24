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
use pse_columnar::CancellationToken;

use super::{EngineSession, PreparedComputation, admission, engine_session::engine};
use crate::{
    EngineError,
    provider::binding::{BindingKey, TableBinding},
};

impl EngineSession {
    /// Bind a declared native query to the exact selected semantic relation inputs.
    /// The private native namespace changes names only: each source is the actual
    /// retained provider, with the caller's functions, configuration and resources.
    /// No data operator executes during binding.
    /// # Errors
    /// Undeclared, unused or unavailable inputs, non-query SQL, or native binding failure.
    pub async fn bind_declared_query(
        &self,
        sql: &str,
        inputs: &[pse_schema::model::RelationKey],
        cancel: &CancellationToken,
    ) -> Result<LogicalPlan, EngineError> {
        self.bind_declared_queries(&[(sql, inputs)], cancel)
            .await?
            .pop()
            .ok_or_else(|| invalid("declared query result absent"))
    }

    /// Bind sibling native queries in one immutable source/field admission scope.
    /// Each query still declares its exact inputs; shared producers are checked once.
    /// # Errors
    /// An invalid query/input declaration, foreign source, field or budget refusal.
    pub async fn bind_declared_queries(
        &self,
        queries: &[(&str, &[pse_schema::model::RelationKey])],
        cancel: &CancellationToken,
    ) -> Result<Vec<LogicalPlan>, EngineError> {
        let mut plans = Vec::with_capacity(queries.len());
        for (sql, inputs) in queries {
            plans.push(self.bind_declared_query_plan(sql, inputs, cancel).await?);
        }
        self.derive_plan_fields_many(&plans, cancel)
    }

    async fn bind_declared_query_plan(
        &self,
        sql: &str,
        inputs: &[pse_schema::model::RelationKey],
        cancel: &CancellationToken,
    ) -> Result<LogicalPlan, EngineError> {
        use datafusion::catalog::{CatalogProvider, CatalogProviderList, SchemaProvider};
        use datafusion::catalog::{
            MemoryCatalogProvider, MemoryCatalogProviderList, MemorySchemaProvider,
        };
        use datafusion::sql::{parser::Statement, sqlparser::ast::Statement as SqlStatement};
        use std::collections::{BTreeMap, BTreeSet};

        cancel.checkpoint()?;
        let state = self.bound_state()?;
        crate::cache_service::metrics::record(&state, |metrics| &metrics.sql_bindings);
        let statement = crate::cache_service::syntax::parse(&state, sql, cancel).map_err(engine)?;
        if !matches!(&statement, Statement::Statement(statement) if matches!(statement.as_ref(), SqlStatement::Query(_)))
        {
            return Err(invalid("a declared query must be a native SELECT query"));
        }
        let expected = inputs
            .iter()
            .map(pse_schema::model::RelationKey::qualified_name)
            .collect::<BTreeSet<_>>();
        let actual = state
            .resolve_table_references(&statement)
            .map_err(engine)?
            .into_iter()
            .map(|name| {
                if name.catalog().is_some() {
                    return Err(invalid(
                        "declared queries use semantic schema/table names, not external catalogs",
                    ));
                }
                let schema = name
                    .schema()
                    .ok_or_else(|| invalid("declared query input needs its semantic schema"))?;
                Ok(format!("{schema}.{}", name.table()))
            })
            .collect::<Result<BTreeSet<_>, EngineError>>()?;
        if actual != expected {
            return Err(invalid(&format!(
                "declared query inputs differ: declared {expected:?}, actual {actual:?}"
            )));
        }
        let catalog = Arc::new(MemoryCatalogProvider::new());
        let mut schemas = BTreeMap::<&str, Arc<MemorySchemaProvider>>::new();
        let mut binding_memory =
            pse_columnar::MemoryConsumer::new("session:query-cache-boundaries")
                .register(&self.pool);
        let providers = self.bindings.providers();
        for key in inputs {
            let binding = self
                .bindings
                .relation(*key)
                .ok_or_else(|| invalid("declared query input is not selected"))?;
            let schema = schemas
                .entry(key.namespace.as_str())
                .or_insert_with(|| Arc::new(MemorySchemaProvider::new()));
            let provider: Arc<dyn TableProvider> = if let Some(view) =
                binding
                    .provider
                    .downcast_ref::<datafusion::datasource::ViewTable>()
            {
                let plan = view
                    .get_logical_plan()
                    .ok_or_else(|| invalid("native view plan absent"))?
                    .into_owned();
                let plan = super::cache::logical::for_query_binding(
                    plan,
                    &self.registry,
                    &providers,
                    &mut binding_memory,
                    cancel,
                )
                .map_err(engine)?;
                Arc::new(datafusion::datasource::ViewTable::new(plan, None))
            } else {
                Arc::clone(&binding.provider)
            };
            schema
                .register_table(key.name.to_owned(), provider)
                .map_err(engine)?;
        }
        for (name, schema) in schemas {
            catalog.register_schema(name, schema).map_err(engine)?;
        }
        let catalogs = Arc::new(MemoryCatalogProviderList::new());
        catalogs.register_catalog(
            state.config_options().catalog.default_catalog.clone(),
            catalog,
        );
        let mut state = state;
        state.register_catalog_list(catalogs);
        let plan = state.statement_to_plan(statement).await.map_err(engine)?;
        cancel.checkpoint()?;
        super::cache::logical::restore_query_bindings(plan).map_err(engine)
    }
    pub(super) async fn capture_namespace(
        &self,
        state: &datafusion::execution::session_state::SessionState,
        created_memory_table: Option<&TableReference>,
        cancel: &CancellationToken,
    ) -> Result<Self, EngineError> {
        let mut result = self.clone();
        result.bindings = self.bindings.resolutions_only();
        for catalog_name in state.catalog_list().catalog_names() {
            result
                .bindings
                .namespace(&catalog_name, None)
                .map_err(engine)?;
            let catalog = state
                .catalog_list()
                .catalog(&catalog_name)
                .ok_or_else(|| invalid("catalog disappeared during namespace capture"))?;
            for schema_name in catalog.schema_names() {
                result
                    .bindings
                    .namespace(&catalog_name, Some(&schema_name))
                    .map_err(engine)?;
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
                    let native_binding = schema
                        .downcast_ref::<crate::provider::schema::SnapshotSchema>()
                        .and_then(|schema| schema.binding(reference.table()))
                        .filter(|binding| {
                            binding.reference == reference
                                && Arc::ptr_eq(&binding.provider, &provider)
                        });
                    let existing: Vec<_> = self
                        .bindings
                        .iter()
                        .filter(|(_, binding)| binding.reference == reference)
                        .filter_map(|(key, binding)| {
                            native_binding
                                .clone()
                                .or_else(|| {
                                    Arc::ptr_eq(&binding.provider, &provider).then_some(binding)
                                })
                                .map(|binding| (key, binding))
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
                                .insert(key.clone(), binding.as_ref().clone())
                                .map_err(engine)?;
                        }
                    }
                }
            }
        }
        result.capture_configuration(state)?;
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
    ) -> Result<Self, EngineError> {
        cancel.checkpoint()?;
        let state = self.bound_state()?;
        let defaults = &state.config_options().catalog;
        let reference = reference.resolve(&defaults.default_catalog, &defaults.default_schema);
        let reference = TableReference::full(reference.catalog, reference.schema, reference.table);
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
    ) -> Result<PreparedComputation, EngineError> {
        cancel.checkpoint()?;
        let state = self.bound_state()?;
        crate::cache_service::metrics::record(&state, |metrics| &metrics.sql_bindings);
        // Parse once and retain identifiers before native schema DDL flattens them.
        let statement = crate::cache_service::syntax::parse(&state, sql, cancel).map_err(engine)?;
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
                        .map_err(pse_columnar::external)?;
                    if state.config_options().catalog.information_schema
                        && reference.schema() == Some("information_schema")
                    {
                        // DataFusion resolves this schema from this private catalog list.
                        // The provider still passed the same field/source admission.
                        resolved.bindings.resolved_metadata(&reference)?;
                    }
                }
            }
            Ok(TreeNodeRecursion::Continue)
        })
        .map_err(engine)?;
        resolved.prepare(plan, cancel)
    }
}

fn invalid(reason: &str) -> EngineError {
    EngineError::Admission {
        path: "provider.native".to_owned(),
        reason: reason.to_owned(),
    }
}
