// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Self-describing Delta contracts: native SQL predicates and table properties.
use super::layout::DurableLayout;
use datafusion::{
    common::{DFSchema, DataFusionError, Result},
    execution::session_state::SessionState,
    logical_expr::{Expr, registry::FunctionRegistry},
};
use datafusion_proto::bytes::Serializeable;
use deltalake::{
    DeltaTable,
    kernel::{
        StructType,
        engine::arrow_conversion::{TryIntoArrow, TryIntoKernel},
        transaction::CommitProperties,
    },
};
use pse_schema::Registry;
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

const EXPRESSION_ENCODING: &str = "datafusion-proto-55.1.0";

/// One exact relation declaration lowered to native Delta CHECK and identity properties.
#[derive(Debug, Clone)]
pub struct DeclaredCheck {
    layout: DurableLayout,
    properties: Arc<BTreeMap<String, String>>,
    adapters: Arc<[super::field_check::FieldCheck]>,
}
#[derive(Default)]
struct Contracts(Mutex<BTreeMap<pse_ids::SemanticId, DeclaredCheck>>);
impl DeclaredCheck {
    pub(super) fn same_declaration(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.properties, &other.properties)
    }

    /// Derive portable predicates from actual fields, including nested value domains.
    /// Relational keys/references remain publication-wide checks.
    /// # Errors
    /// The declaration, durable mapping or native SQL expression is unsupported.
    pub fn new(registry: &Registry, relation_id: pse_ids::SemanticId) -> Result<Self> {
        let declarations = registry
            .derived_implementation(|| Ok(Contracts::default()))
            .map_err(external)?;
        let mut declarations = declarations
            .0
            .lock()
            .map_err(|_| invalid("Delta contract cache poisoned"))?;
        if let Some(contract) = declarations.get(&relation_id) {
            return Ok(contract.clone());
        }
        let contract = Self::derive(registry, relation_id)?;
        declarations.insert(relation_id, contract.clone());
        Ok(contract)
    }
    fn derive(registry: &Registry, relation_id: pse_ids::SemanticId) -> Result<Self> {
        let spec = registry
            .relation_by_id(relation_id)
            .ok_or_else(|| invalid("unknown durable relation"))?;
        registry.contract(spec).map_err(external)?;
        let schema = pse_schema::arrow::relation_schema(registry, spec).map_err(external)?;
        let layout = DurableLayout::new(Arc::new(schema.clone()))?;
        let mut properties: BTreeMap<_, _> = schema
            .metadata()
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        let mut adapters = vec![];
        let mut checks = vec![];
        for (index, field) in schema.fields().iter().enumerate() {
            let column = Expr::Column(datafusion::common::Column::from_name(field.name()));
            let predicate = pse_relations::validate::predicates::field_value(
                registry,
                field,
                column.clone(),
                0,
            )?;
            if needs_adapter(field, layout.storage_schema().field(index)) {
                properties.insert(
                    expression_property(field.name()),
                    serde_json::to_string(predicate.to_bytes()?.as_ref())
                        .map_err(|error| DataFusionError::External(Box::new(error)))?,
                );
                let adapter = super::field_check::FieldCheck::new(
                    format!("pse_field_{}_{index}", spec.fingerprint),
                    layout.storage_schema().field(index).clone(),
                    field.as_ref().clone(),
                    predicate,
                )?;
                checks.push(adapter.function().call(vec![column]));
                adapters.push(adapter);
            } else {
                checks.push(predicate);
            }
        }
        if !adapters.is_empty() {
            properties.insert(
                "pse.check.field.encoding".into(),
                EXPRESSION_ENCODING.into(),
            );
        }
        let sql = datafusion::sql::unparser::expr_to_sql(
            &pse_relations::validate::predicates::combine(checks),
        )?
        .to_string();
        properties.insert("delta.constraints.pse_contract".into(), sql);
        properties.extend(crate::contract::row_checks::properties(&schema)?);
        properties.extend(pse_schema::arrow::delta_properties(&schema).map_err(external)?);
        Ok(Self {
            layout,
            properties: Arc::new(properties),
            adapters: adapters.into(),
        })
    }
    /// Reconstruct the recorded fields and native predicates without a registry.
    /// This establishes self-consistency, not publication admission or agreement
    /// with an independently supplied registry contract.
    /// # Errors
    /// Missing descriptors/properties, unknown expression codec or invalid predicates.
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
        let mut adapters = vec![];
        for (index, field) in layout.execution_schema().fields().iter().enumerate() {
            if !needs_adapter(field, layout.storage_schema().field(index)) {
                continue;
            }
            let encoding = get("pse.check.field.encoding")?;
            if encoding != EXPRESSION_ENCODING {
                return Err(invalid("unknown field CHECK encoding"));
            }
            properties.insert("pse.check.field.encoding".into(), encoding);
            let key = expression_property(field.name());
            let encoded = get(&key)?;
            let stored_field = stored.field(index).clone();
            let bytes: Vec<u8> = serde_json::from_str(&encoded)
                .map_err(|error| DataFusionError::External(Box::new(error)))?;
            let predicate = Expr::from_bytes_with_ctx(&bytes, &state.task_ctx())?;
            adapters.push(super::field_check::FieldCheck::new(
                format!("pse_field_{fingerprint}_{index}"),
                stored_field,
                field.as_ref().clone(),
                predicate,
            )?);
            properties.insert(key, encoded);
        }
        properties.insert(
            "delta.constraints.pse_contract".into(),
            get("delta.constraints.pse_contract")?,
        );
        properties.extend(crate::contract::row_checks::properties(
            layout.execution_schema(),
        )?);
        properties.extend(
            pse_schema::arrow::delta_properties(layout.execution_schema()).map_err(external)?,
        );
        let contract = Self {
            layout,
            properties: Arc::new(properties),
            adapters: adapters.into(),
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
    /// Bind field expressions requiring lambdas or execution/storage restoration.
    /// Caller configuration, planner, runtime and existing functions remain intact.
    /// # Errors
    /// Native compilation or a conflicting function binding refuses the contract.
    pub fn bind(&self, state: &SessionState) -> Result<SessionState> {
        let mut state = state.clone();
        for adapter in self.adapters.iter() {
            let function = Arc::new(
                adapter
                    .bind(&state)
                    .map_err(|error| error.context("bind durable field CHECK"))?,
            );
            if let Ok(existing) = state.udf(function.name())
                && existing != function
            {
                return Err(invalid(
                    "field CHECK function conflicts with caller binding",
                ));
            }
            state.register_udf(function)?;
        }
        let schema = DFSchema::try_from(self.layout.storage_schema().as_ref().clone())?;
        for (name, expression) in pse_relations::validate::row_checks::bind(
            self.layout.storage_schema(),
            &pse_engine::validation::NativeValidation(state.clone()),
        )? {
            state
                .create_physical_expr(expression, &schema)
                .map_err(|error| error.context(format!("bind durable row CHECK {name}")))?;
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
            .map(|(key, value)| (key.clone(), Some(value.clone())));
        table
            .create()
            .with_columns(schema.fields().cloned())
            .with_configuration(properties)
            .with_raise_if_key_not_exists(false)
            .with_commit_properties(super::operation::commit_policy(
                commit,
                super::operation::CommitKind::Data,
            ))
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
        if configuration
            .get(pse_schema::arrow::KEY_CONTRACT_ID)
            .is_some_and(|id| {
                id == &pse_relations::generated::runtime::publications::RELATION_ID.to_string()
            })
            && configuration.contains_key("delta.setTransactionRetentionDuration")
        {
            return Err(invalid(
                "publication transaction identities must survive control log cleanup",
            ));
        }
        if let Some(name) = configuration.keys().find(|name| {
            name.starts_with("delta.constraints.") && !self.properties.contains_key(*name)
        }) {
            return Err(invalid(&format!(
                "Delta CHECK {name} is outside the declared contract"
            )));
        }
        for (key, value) in self.properties.iter() {
            if configuration.get(key) != Some(value) {
                return Err(invalid(&format!(
                    "Delta table property {key} differs from the declared contract"
                )));
            }
        }
        Ok(())
    }
}
fn needs_adapter(
    field: &datafusion::arrow::datatypes::Field,
    storage: &datafusion::arrow::datatypes::Field,
) -> bool {
    super::field_check::contains_collection(field)
        || !field.data_type().equals_datatype(storage.data_type())
}

fn invalid(reason: &str) -> DataFusionError {
    DataFusionError::Plan(reason.to_owned())
}
fn expression_property(field: &str) -> String {
    // Field names occupy their own property namespace, including a field named encoding.
    format!("pse.check.field.expression.{field}")
}
fn external(error: impl Into<DataFusionError>) -> DataFusionError {
    error.into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn all_registry_durable_projections_plan_against_native_physical_inputs() {
        use datafusion::{
            arrow::array::RecordBatch,
            datasource::{MemTable, provider_as_source},
            logical_expr::LogicalPlanBuilder,
        };
        let registry = pse_engine::validation::registry().unwrap();
        let state = datafusion::execution::session_state::SessionStateBuilder::new()
            .with_default_features()
            .build();
        for relation in registry.relations() {
            let contract = DeclaredCheck::new(registry, relation.id).unwrap();
            let schema = Arc::new(pse_schema::arrow::relation_schema(registry, relation).unwrap());
            let provider = Arc::new(
                MemTable::try_new(schema.clone(), vec![vec![RecordBatch::new_empty(schema)]])
                    .unwrap(),
            );
            let input = LogicalPlanBuilder::scan("input", provider_as_source(provider), None)
                .unwrap()
                .build()
                .unwrap();
            let encoded = contract.layout.encode(input).unwrap();
            state
                .create_physical_plan(&encoded)
                .await
                .unwrap_or_else(|error| {
                    panic!("{} durable projection: {error}", relation.qualified_name())
                });
        }
    }

    #[test]
    fn all_registry_durable_checks_bind_to_their_actual_storage_fields() {
        let registry = pse_engine::validation::registry().unwrap();
        let state = datafusion::execution::session_state::SessionStateBuilder::new()
            .with_default_features()
            .build();
        for relation in registry.relations() {
            let contract = DeclaredCheck::new(registry, relation.id).unwrap_or_else(|error| {
                panic!("{} declaration: {error}", relation.qualified_name())
            });
            let bound = contract.bind(&state).unwrap_or_else(|error| {
                panic!(
                    "{} native CHECK binding: {error}",
                    relation.qualified_name()
                )
            });
            let schema =
                DFSchema::try_from(contract.layout.storage_schema().as_ref().clone()).unwrap();
            let expression = deltalake::delta_datafusion::expr::parse_predicate_expression(
                &schema,
                &contract.properties["delta.constraints.pse_contract"],
                &bound,
            )
            .unwrap_or_else(|error| {
                panic!(
                    "{} native SQL CHECK parsing: {error}",
                    relation.qualified_name()
                )
            });
            bound
                .create_physical_expr(expression, &schema)
                .unwrap_or_else(|error| {
                    panic!(
                        "{} native SQL CHECK binding: {error}",
                        relation.qualified_name()
                    )
                });
        }
    }
}
