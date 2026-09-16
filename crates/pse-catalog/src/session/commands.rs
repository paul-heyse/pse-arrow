// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Typed command destinations participate before any eager native handler runs.

use super::{SnapshotSession, snapshot_session::engine};
use crate::{
    CatalogError,
    provider::{catalog::SnapshotCatalog, list::SnapshotCatalogList, schema::SnapshotSchema},
};
use datafusion::{
    catalog::CatalogProviderList,
    common::{
        DataFusionError, Result, SchemaReference, TableReference, tree_node::TreeNodeRecursion,
    },
    dataframe::DataFrame,
    execution::context::SessionContext,
    logical_expr::{DdlStatement, LogicalPlan, LogicalPlanBuilder},
    sql::{
        parser::Statement,
        sqlparser::ast::{SchemaName, Statement as SqlStatement},
    },
};
use pse_schema::model::provider::ProviderScope;
use std::{collections::BTreeMap, sync::Arc};

/// Preserve the identifier structure that DataFusion 55.1 flattens in schema DDL.
pub(super) fn schema_spelling(statement: &Statement) -> Option<String> {
    let Statement::Statement(statement) = statement else {
        return None;
    };
    let SqlStatement::CreateSchema { schema_name, .. } = statement.as_ref() else {
        return None;
    };
    Some(match schema_name {
        SchemaName::Simple(name) => name.to_string(),
        SchemaName::UnnamedAuthorization(name) => name.to_string(),
        SchemaName::NamedAuthorization(name, owner) => format!("{name}.{owner}"),
    })
}

fn schema_target(name: &str, default_catalog: &str) -> Result<ProviderScope> {
    match TableReference::parse_str(name) {
        TableReference::Bare { table } => Ok(ProviderScope::Schema(
            default_catalog.to_owned(),
            table.to_string(),
        )),
        TableReference::Partial { schema, table } => {
            Ok(ProviderScope::Schema(schema.to_string(), table.to_string()))
        }
        TableReference::Full { .. } => Err(DataFusionError::Plan(
            "a schema destination has at most catalog and schema components".into(),
        )),
    }
}

impl SnapshotSession {
    pub(super) fn bind_targets(&self, plan: &LogicalPlan) -> Result<Self, CatalogError> {
        let mut result = self.clone();
        let state = self.bound_state()?;
        let defaults = &state.config_options().catalog;
        let table_scope = |name: &TableReference| {
            let name = name
                .clone()
                .resolve(&defaults.default_catalog, &defaults.default_schema);
            ProviderScope::Table(
                name.catalog.to_string(),
                name.schema.to_string(),
                name.table.to_string(),
            )
        };
        plan.apply_with_subqueries(|node| {
            let target = match node {
                LogicalPlan::Dml(command) => Some(table_scope(&command.table_name)),
                LogicalPlan::Ddl(command) => match command {
                    DdlStatement::CreateMemoryTable(c) => Some(table_scope(&c.name)),
                    DdlStatement::CreateExternalTable(c) => Some(table_scope(&c.name)),
                    DdlStatement::CreateView(c) => Some(table_scope(&c.name)),
                    DdlStatement::DropTable(c) => Some(table_scope(&c.name)),
                    DdlStatement::DropView(c) => Some(table_scope(&c.name)),
                    DdlStatement::CreateIndex(c) => Some(table_scope(&c.table)),
                    DdlStatement::CreateCatalog(c) => {
                        Some(ProviderScope::Catalog(c.catalog_name.clone()))
                    }
                    DdlStatement::CreateCatalogSchema(c) => {
                        Some(schema_target(&c.schema_name, &defaults.default_catalog)?)
                    }
                    DdlStatement::DropCatalogSchema(c) => Some(match &c.name {
                        SchemaReference::Bare { schema } => ProviderScope::Schema(
                            defaults.default_catalog.clone(),
                            schema.to_string(),
                        ),
                        SchemaReference::Full { catalog, schema } => {
                            ProviderScope::Schema(catalog.to_string(), schema.to_string())
                        }
                    }),
                    DdlStatement::CreateFunction(_) | DdlStatement::DropFunction(_) => None,
                },
                _ => None,
            };
            if let Some(target) = target {
                result.bindings.target(target);
            }
            Ok(TreeNodeRecursion::Continue)
        })
        .map_err(engine)?;
        Ok(result)
    }
}

/// Native namespace registration against the private generation. New descendants
/// use the same atomic registration and mutation witness as existing descendants.
pub(super) fn create_namespace(
    context: &SessionContext,
    command: &DdlStatement,
) -> Result<Option<DataFrame>> {
    let state = context.state();
    let list = state
        .catalog_list()
        .downcast_ref::<SnapshotCatalogList>()
        .ok_or_else(|| DataFusionError::Internal("missing private catalog generation".into()))?;
    match command {
        DdlStatement::CreateCatalog(c) => {
            if list.catalog(&c.catalog_name).is_some() {
                if !c.if_not_exists {
                    return Err(DataFusionError::Execution(format!(
                        "Catalog '{}' already exists",
                        c.catalog_name
                    )));
                }
            } else {
                list.register_catalog(
                    c.catalog_name.clone(),
                    Arc::new(SnapshotCatalog::from_tables(
                        &BTreeMap::new(),
                        None,
                        std::iter::empty(),
                        list.generation_owner(),
                    )),
                );
            }
        }
        DdlStatement::CreateCatalogSchema(c) => {
            let ProviderScope::Schema(catalog, schema) = schema_target(
                &c.schema_name,
                &state.config_options().catalog.default_catalog,
            )?
            else {
                return Err(DataFusionError::Internal(
                    "missing schema destination".into(),
                ));
            };
            let catalog = context.catalog(&catalog).ok_or_else(|| {
                DataFusionError::Execution(format!("Missing catalog '{catalog}'"))
            })?;
            if catalog.schema(&schema).is_some() {
                if !c.if_not_exists {
                    return Err(DataFusionError::Execution(format!(
                        "Schema '{schema}' already exists"
                    )));
                }
            } else {
                catalog.register_schema(
                    &schema,
                    Arc::new(SnapshotSchema::new(
                        BTreeMap::new(),
                        list.generation_owner(),
                    )),
                )?;
            }
        }
        _ => return Ok(None),
    }
    Ok(Some(DataFrame::new(
        context.state(),
        LogicalPlanBuilder::empty(false).build()?,
    )))
}
