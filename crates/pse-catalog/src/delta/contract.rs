// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Self-describing Delta contracts: native SQL predicates and table properties.
use super::layout::DurableLayout;
use datafusion::{
    common::{DFSchema, DataFusionError, Result, config::Dialect as ParserDialect},
    execution::session_state::SessionState,
    logical_expr::{Expr, registry::FunctionRegistry},
    sql::{
        sqlparser::ast,
        unparser::{
            Unparser,
            dialect::{Dialect, DuckDBDialect},
            expr_to_sql,
        },
    },
};
use deltalake::{
    DeltaTable,
    kernel::{
        StructType,
        engine::arrow_conversion::{TryIntoArrow, TryIntoKernel},
        transaction::CommitProperties,
    },
};
use pse_schema::Registry;
use std::{collections::BTreeMap, sync::Arc};

/// One exact relation declaration lowered to native Delta CHECK and identity properties.
#[derive(Debug, Clone)]
pub struct DeclaredCheck {
    layout: DurableLayout,
    properties: BTreeMap<String, String>,
    nested: Vec<super::nested_check::NestedCheck>,
}
impl DeclaredCheck {
    /// Derive portable predicates from actual fields, including nested value domains.
    /// Relational keys/references remain publication-wide checks.
    /// # Errors
    /// The declaration, durable mapping or native SQL expression is unsupported.
    pub fn new(registry: &Registry, relation_id: pse_ids::SemanticId) -> Result<Self> {
        let spec = registry
            .relation_by_id(relation_id)
            .ok_or_else(|| invalid("unknown durable relation"))?;
        let schema = pse_schema::arrow::relation_schema(registry, spec).map_err(external)?;
        let layout = DurableLayout::new(Arc::new(schema.clone()))?;
        let mut properties: BTreeMap<_, _> = schema
            .metadata()
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        let mut nested = vec![];
        let mut checks = vec![];
        for (index, field) in schema.fields().iter().enumerate() {
            let column = Expr::Column(datafusion::common::Column::from_name(field.name()));
            let predicate = super::predicates::field_value(registry, field, column.clone(), 0)?;
            if super::nested_check::contains_collection(field) {
                properties.insert(expression_property(field.name()), native_sql(&predicate)?);
                let adapter = super::nested_check::NestedCheck::new(
                    format!("pse_nested_{}_{index}", spec.fingerprint),
                    layout.storage_schema().field(index).clone(),
                    predicate,
                )?;
                checks.push(adapter.function().call(vec![column]));
                nested.push(adapter);
            } else {
                checks.push(predicate);
            }
        }
        if !nested.is_empty() {
            properties.insert("pse.check.nested.dialect".into(), "duckdb".into());
        }
        let sql = expr_to_sql(&super::predicates::combine(spec, checks))?.to_string();
        properties.insert("delta.constraints.pse_contract".into(), sql);
        Ok(Self {
            layout,
            properties,
            nested,
        })
    }
    /// Reconstruct the recorded fields and native predicates without a registry.
    /// This establishes self-consistency, not publication admission or agreement
    /// with an independently supplied registry contract.
    /// # Errors
    /// Missing descriptors/properties, unknown collection dialect or invalid SQL.
    pub fn open(table: &DeltaTable, state: &SessionState) -> Result<Self> {
        let snapshot = table.snapshot().map_err(external)?;
        let stored: datafusion::arrow::datatypes::Schema = snapshot
            .schema()
            .as_ref()
            .try_into_arrow()
            .map_err(external)?;
        let execution = pse_schema::delta::execution_schema(&stored).map_err(external)?;
        let layout = DurableLayout::new(Arc::new(execution))?;
        let configuration = snapshot.metadata().configuration();
        let get = |key: &str| {
            configuration
                .get(key)
                .cloned()
                .ok_or_else(|| invalid(&format!("missing Delta contract property {key}")))
        };
        let mut properties: BTreeMap<_, _> = layout
            .execution_schema()
            .metadata()
            .iter()
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect();
        let fingerprint = properties
            .get(pse_schema::arrow::KEY_CONTRACT_FINGERPRINT)
            .ok_or_else(|| invalid("missing durable contract fingerprint"))?
            .clone();
        let mut parser = state.clone();
        parser.config_mut().options_mut().sql_parser.dialect = ParserDialect::DuckDB;
        let mut nested = vec![];
        for (index, field) in layout.execution_schema().fields().iter().enumerate() {
            if !super::nested_check::contains_collection(field) {
                continue;
            }
            let dialect = get("pse.check.nested.dialect")?;
            if dialect != "duckdb" {
                return Err(invalid("unknown nested CHECK dialect"));
            }
            properties.insert("pse.check.nested.dialect".into(), dialect);
            let key = expression_property(field.name());
            let sql = get(&key)?;
            let stored_field = stored.field(index).clone();
            let schema = DFSchema::try_from(datafusion::arrow::datatypes::Schema::new(vec![
                stored_field.clone(),
            ]))?;
            let predicate = parser.create_logical_expr(&sql, &schema)?;
            nested.push(super::nested_check::NestedCheck::new(
                format!("pse_nested_{fingerprint}_{index}"),
                stored_field,
                predicate,
            )?);
            properties.insert(key, sql);
        }
        properties.insert(
            "delta.constraints.pse_contract".into(),
            get("delta.constraints.pse_contract")?,
        );
        let contract = Self {
            layout,
            properties,
            nested,
        };
        contract.verify(table)?;
        // Fail at open if this caller cannot execute the recorded native contract.
        let bound = contract.bind(state)?;
        let schema = DFSchema::try_from(stored)?;
        let predicate = bound.create_logical_expr(
            &contract.properties["delta.constraints.pse_contract"],
            &schema,
        )?;
        bound.create_physical_expr(predicate, &schema)?;
        Ok(contract)
    }
    /// Bind only the collection expressions the pinned Delta parser cannot express.
    /// Caller configuration, planner, runtime and existing functions remain intact.
    /// # Errors
    /// Native compilation or a conflicting function binding refuses the contract.
    pub fn bind(&self, state: &SessionState) -> Result<SessionState> {
        let mut state = state.clone();
        for nested in &self.nested {
            let function = Arc::new(nested.bind(&state)?);
            if let Ok(existing) = state.udf(function.name())
                && existing != function
            {
                return Err(invalid(
                    "nested CHECK function conflicts with caller binding",
                ));
            }
            state.register_udf(function)?;
        }
        Ok(state)
    }
    /// Exact execution/storage projection, derived from the field declaration.
    pub fn layout(&self) -> &DurableLayout {
        &self.layout
    }

    /// Native SQL checks, identity and explicit collection-expression requirements.
    pub fn properties(&self) -> &BTreeMap<String, String> {
        &self.properties
    }

    /// Create an empty declared table with constraints before the first data write.
    /// # Errors
    /// Native schema conversion or Delta creation refuses the declaration.
    pub(super) async fn create(
        &self,
        table: DeltaTable,
        commit: CommitProperties,
    ) -> Result<DeltaTable> {
        let schema: StructType = self
            .layout
            .storage_schema()
            .as_ref()
            .try_into_kernel()
            .map_err(external)?;
        let properties = self
            .properties
            .iter()
            .map(|(key, value)| (key.clone(), Some(value.clone())))
            .chain([("delta.minWriterVersion".into(), Some("3".into()))]);
        table
            .create()
            .with_columns(schema.fields().cloned())
            .with_configuration(properties)
            .with_raise_if_key_not_exists(false)
            .with_commit_properties(commit)
            .await
            .map_err(external)
    }

    /// Ensure the loaded table binds this exact declaration before offering mutations.
    /// # Errors
    /// Absent/different identity, fingerprint, namespace, version or native CHECK.
    pub fn verify(&self, table: &DeltaTable) -> Result<()> {
        let snapshot = table.snapshot().map_err(external)?;
        let stored: datafusion::arrow::datatypes::Schema = snapshot
            .schema()
            .as_ref()
            .try_into_arrow()
            .map_err(external)?;
        pse_schema::field_contract::delta_scan_schema(&stored, self.layout.storage_schema())
            .map_err(external)?;
        let configuration = snapshot.metadata().configuration();
        for (key, value) in &self.properties {
            if configuration.get(key) != Some(value) {
                return Err(invalid(&format!(
                    "Delta table property {key} differs from the declared contract"
                )));
            }
        }
        Ok(())
    }
}
fn invalid(reason: &str) -> DataFusionError {
    DataFusionError::Plan(reason.to_owned())
}
fn expression_property(field: &str) -> String {
    // Field names occupy their own property namespace, including a field named dialect.
    format!("pse.check.nested.expression.{field}")
}
fn external(error: impl std::error::Error + Send + Sync + 'static) -> DataFusionError {
    DataFusionError::External(Box::new(error))
}

// Dot syntax loses lambda scope in the pinned SQL planner. Its native unparser
// override hook renders explicit get_field calls, preserving the actual Expr tree.
fn native_sql(predicate: &Expr) -> Result<String> {
    let dialect = DuckDBDialect::new().with_custom_scalar_overrides(vec![(
        "get_field",
        Box::new(|unparser, args| {
            let args = args
                .iter()
                .map(|arg| {
                    Ok(ast::FunctionArg::Unnamed(ast::FunctionArgExpr::Expr(
                        unparser.expr_to_sql(arg)?,
                    )))
                })
                .collect::<Result<Vec<_>>>()?;
            Ok(Some(ast::Expr::Function(ast::Function {
                name: ast::ObjectName::from(vec![ast::Ident::new("get_field")]),
                args: ast::FunctionArguments::List(ast::FunctionArgumentList {
                    duplicate_treatment: None,
                    args,
                    clauses: vec![],
                }),
                filter: None,
                null_treatment: None,
                over: None,
                within_group: vec![],
                parameters: ast::FunctionArguments::None,
                uses_odbc_syntax: false,
            })))
        }),
    )]);
    Ok(Unparser::new(&dialect).expr_to_sql(predicate)?.to_string())
}
