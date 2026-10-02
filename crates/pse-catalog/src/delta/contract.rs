// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Self-describing Delta contracts: native SQL predicates and table properties.
use super::layout::DurableLayout;
use datafusion::{
    common::{DFSchema, DataFusionError, Result},
    execution::session_state::SessionState,
    logical_expr::{Expr, registry::FunctionRegistry},
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
use pse_schema::compatibility::VerifiedRecordedContract;
use pse_schema::compatibility::{self, CompatibilityError};
use pse_schema::fingerprint::SemanticContract;
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

/// One exact relation declaration lowered to native Delta CHECK and identity properties.
#[derive(Debug, Clone)]
pub struct DeclaredCheck {
    layout: DurableLayout,
    properties: Arc<BTreeMap<String, String>>,
    adapters: Arc<[super::field_check::FieldCheck]>,
    semantic: Arc<VerifiedRecordedContract>,
    encoding: pse_ids::ContentHash,
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
        let semantic = Arc::new(
            VerifiedRecordedContract::from_contract(
                SemanticContract::new(registry, &[relation_id].into()).map_err(external)?,
            )
            .map_err(external)?,
        );
        let encoding =
            pse_schema::fingerprint::encoding_relation(registry, spec).map_err(external)?;
        properties.insert(
            compatibility::KEY_FORMAT.into(),
            compatibility::FORMAT.into(),
        );
        properties.insert(
            compatibility::KEY_CONTRACT.into(),
            pse_columnar::native_field::canonical_json(semantic.contract())
                .map_err(|e| DataFusionError::External(Box::new(e)))?,
        );
        properties.insert(compatibility::KEY_ENCODING.into(), encoding.to_string());
        let (adapters, sql) = compile_predicates(
            &layout,
            &pse_relations::validate::predicates::DomainEnvironment::current(registry),
            &spec.fingerprint.to_string(),
        )?;
        properties.insert("delta.constraints.pse_contract".into(), sql);
        properties.extend(crate::contract::row_checks::properties(&schema)?);
        properties.extend(pse_schema::arrow::delta_properties(&schema).map_err(external)?);
        Ok(Self {
            layout,
            properties: Arc::new(properties),
            adapters: adapters.into(),
            semantic,
            encoding,
        })
    }
    /// Reconstruct the recorded fields and native predicates without a registry.
    /// This establishes self-consistency, not publication admission or agreement
    /// with an independently supplied registry contract.
    /// # Errors
    /// Missing descriptors/properties or contradictory portable meaning and derived checks.
    pub fn open(table: &DeltaTable, state: &SessionState) -> Result<Self> {
        let snapshot = table.snapshot().map_err(external)?;
        let stored: datafusion::arrow::datatypes::Schema = snapshot
            .schema()
            .as_ref()
            .try_into_arrow()
            .map_err(external)?;
        Self::open_fields(stored, snapshot.metadata().configuration(), state)
    }
    fn open_fields(
        stored: datafusion::arrow::datatypes::Schema,
        configuration: &std::collections::HashMap<String, String>,
        state: &SessionState,
    ) -> Result<Self> {
        let semantic = recorded_contract(configuration)?;
        let execution = pse_schema::delta::execution_schema(&stored).map_err(|error| {
            external(CompatibilityError::UnsupportedEncoding(error.to_string()))
        })?;
        let layout = DurableLayout::new(Arc::new(execution)).map_err(|error| {
            external(CompatibilityError::UnsupportedEncoding(error.to_string()))
        })?;
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
        for key in [
            compatibility::KEY_FORMAT,
            compatibility::KEY_CONTRACT,
            compatibility::KEY_ENCODING,
        ] {
            properties.insert(key.into(), get(key)?);
        }
        let encoding = verify_recorded_fields(&semantic, layout.execution_schema(), configuration)?;
        let domains = pse_relations::validate::predicates::DomainEnvironment::recorded(&semantic)?;
        let (adapters, sql) = compile_predicates(&layout, &domains, &fingerprint)?;
        let original_sql = get("delta.constraints.pse_contract")?;
        properties.insert("delta.constraints.pse_contract".into(), original_sql);
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
            semantic: Arc::new(semantic),
            encoding,
        };
        contract.verify_properties(configuration)?;
        // Fail at open if this caller cannot execute the recorded native contract.
        let bound = contract.bind(state)?;
        let schema = DFSchema::try_from(stored)?;
        let predicate = bound.create_logical_expr(
            &contract.properties["delta.constraints.pse_contract"],
            &schema,
        )?;
        let reconstructed = bound.create_logical_expr(&sql, &schema)?;
        if check_normalized(predicate.clone())? != check_normalized(reconstructed)? {
            return Err(external(CompatibilityError::Malformed(
                "derived CHECK contradicts portable recorded meaning".into(),
            )));
        }
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
    /// Independently verified recorded meaning retained by the read interpreter.
    pub fn recorded(&self) -> &VerifiedRecordedContract {
        &self.semantic
    }
    /// Exact observed execution layout identity.
    pub fn encoding_identity(&self) -> pse_ids::ContentHash {
        self.encoding
    }
    /// Read admission cannot authorize writes or retarget a recorded predicate.
    /// # Errors
    /// Consumed meaning is absent or incompatible.
    pub fn consumer_projection(
        &self,
        consumer: &Self,
    ) -> Result<compatibility::ConsumerProjection> {
        self.semantic
            .project(consumer.semantic.contract())
            .map_err(external)
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
        let configuration = snapshot.metadata().configuration();
        let actual = recorded_contract(configuration)?;
        let execution = pse_schema::delta::execution_schema(&stored).map_err(|error| {
            external(CompatibilityError::UnsupportedEncoding(error.to_string()))
        })?;
        let encoding = verify_recorded_fields(&actual, &execution, configuration)?;
        actual
            .admit_exact_write(self.semantic.contract(), encoding, self.encoding)
            .map_err(external)?;
        self.verify_properties(configuration)
    }
    fn verify_properties(
        &self,
        configuration: &std::collections::HashMap<String, String>,
    ) -> Result<()> {
        if let Some(name) = configuration.keys().find(|name| {
            name.starts_with("delta.constraints.") && !self.properties.contains_key(*name)
        }) {
            return Err(invalid(&format!(
                "Delta CHECK {name} is outside the declared contract"
            )));
        }
        for (key, value) in self.properties.iter() {
            let same = if key.starts_with("delta.constraints.") {
                configuration
                    .get(key)
                    .map(|sql| pse_schema::fingerprint::canonical_sql(sql, true))
                    .transpose()
                    .map_err(external)?
                    == Some(pse_schema::fingerprint::canonical_sql(value, true).map_err(external)?)
            } else if key == pse_schema::arrow::KEY_CHECKS {
                true // The recorded witness and native CHECK expressions own the canonical meaning.
            } else {
                configuration.get(key) == Some(value)
            };
            if !same {
                return Err(invalid(&format!(
                    "Delta table property {key} differs from the declared contract"
                )));
            }
        }
        Ok(())
    }
}
fn recorded_contract(
    properties: &std::collections::HashMap<String, String>,
) -> Result<VerifiedRecordedContract> {
    if properties
        .get(compatibility::KEY_FORMAT)
        .map(String::as_str)
        != Some(compatibility::FORMAT)
    {
        return Err(external(CompatibilityError::MigrationRequired(
            "unrecognized durable semantic contract".into(),
        )));
    }
    let text = properties
        .get(pse_schema::arrow::KEY_CONTRACT_FINGERPRINT)
        .ok_or_else(|| {
            external(CompatibilityError::Malformed(
                "missing semantic digest".into(),
            ))
        })?;
    let hash = pse_ids::ContentHash::parse_hex(text)
        .map_err(|e| external(CompatibilityError::Malformed(e.to_string())))?;
    compatibility::verify_recorded(
        properties
            .get(compatibility::KEY_FORMAT)
            .map(String::as_str),
        properties
            .get(compatibility::KEY_CONTRACT)
            .map(String::as_str),
        hash,
    )
    .map_err(external)
}
fn verify_recorded_fields(
    semantic: &VerifiedRecordedContract,
    schema: &datafusion::arrow::datatypes::Schema,
    properties: &std::collections::HashMap<String, String>,
) -> Result<pse_ids::ContentHash> {
    let malformed = |reason: &str| external(CompatibilityError::Malformed(reason.into()));
    let id = pse_ids::SemanticId::parse_hex(
        properties
            .get(pse_schema::arrow::KEY_CONTRACT_ID)
            .ok_or_else(|| malformed("missing relation identity"))?,
    )
    .map_err(|_| malformed("invalid relation identity"))?;
    if semantic.contract().roots != [id].into() {
        return Err(malformed("relation witness has different roots"));
    }
    let description = semantic
        .contract()
        .relations
        .get(&id)
        .ok_or_else(|| malformed("missing root declaration"))?;
    let checks = pse_schema::arrow::native_checks(schema)
        .map_err(external)?
        .into_iter()
        .map(|(name, sql)| {
            Ok((
                name,
                pse_schema::fingerprint::canonical_sql(&sql, true).map_err(external)?,
            ))
        })
        .collect::<Result<BTreeMap<_, _>>>()?;
    if description.get("checks")
        != Some(&serde_json::to_value(checks).map_err(|e| DataFusionError::External(Box::new(e)))?)
    {
        return Err(malformed(
            "recorded native CHECK expressions contradict semantic declaration",
        ));
    }
    let actual = pse_ids::ContentHash::parse_hex(
        properties
            .get(compatibility::KEY_ENCODING)
            .ok_or_else(|| malformed("missing execution encoding"))?,
    )
    .map_err(|error| external(CompatibilityError::Malformed(error.to_string())))?;
    semantic
        .verify_fields(id, schema, actual)
        .map_err(external)?;
    let (qualified, version) = description["relation"]
        .as_str()
        .and_then(|name| name.rsplit_once('@'))
        .ok_or_else(|| malformed("invalid recorded relation name"))?;
    if properties
        .get(pse_schema::arrow::KEY_CONTRACT_VERSION)
        .map(String::as_str)
        != Some(version)
        || properties
            .get(pse_schema::arrow::KEY_NAMESPACE)
            .map(String::as_str)
            != qualified.split_once('.').map(|(namespace, _)| namespace)
        || schema.metadata().get(pse_schema::arrow::KEY_CONTRACT_ID)
            != properties.get(pse_schema::arrow::KEY_CONTRACT_ID)
        || schema
            .metadata()
            .get(pse_schema::arrow::KEY_CONTRACT_VERSION)
            != properties.get(pse_schema::arrow::KEY_CONTRACT_VERSION)
        || schema
            .metadata()
            .get(pse_schema::arrow::KEY_CONTRACT_FINGERPRINT)
            != properties.get(pse_schema::arrow::KEY_CONTRACT_FINGERPRINT)
    {
        return Err(malformed(
            "recorded root identity/version or schema metadata differs",
        ));
    }
    Ok(actual)
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
fn check_normalized(expression: Expr) -> Result<Expr> {
    use datafusion::common::tree_node::{Transformed, TransformedResult, TreeNode};
    expression
        .transform(|expression| {
            if let Expr::InList(mut list) = expression {
                // Domain member order is presentation. Only literal IN sets are commutative;
                // every other predicate structure, reference and UDF identity stays exact.
                if list
                    .list
                    .iter()
                    .all(|value| matches!(value, Expr::Literal(_, _)))
                {
                    list.list.sort_by_key(ToString::to_string);
                }
                Ok(Transformed::yes(Expr::InList(list)))
            } else {
                Ok(Transformed::no(expression))
            }
        })
        .data()
}
fn compile_predicates(
    layout: &DurableLayout,
    domains: &pse_relations::validate::predicates::DomainEnvironment,
    fingerprint: &str,
) -> Result<(Vec<super::field_check::FieldCheck>, String)> {
    let mut adapters = Vec::new();
    let mut checks = Vec::new();
    for (index, field) in layout.execution_schema().fields().iter().enumerate() {
        let column = Expr::Column(datafusion::common::Column::from_name(field.name()));
        let predicate =
            pse_relations::validate::predicates::field_value_in(domains, field, column.clone(), 0)?;
        if needs_adapter(field, layout.storage_schema().field(index)) {
            let adapter = super::field_check::FieldCheck::new(
                format!("pse_field_{fingerprint}_{index}"),
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
    let sql =
        datafusion::sql::unparser::expr_to_sql(&pse_relations::validate::predicates::balanced(
            pse_relations::validate::predicates::combine(checks),
        )?)?
        .to_string();
    Ok((adapters, sql))
}
fn external(error: impl Into<DataFusionError>) -> DataFusionError {
    error.into()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn recorded_fixture(growth: bool) -> DeclaredCheck {
        use pse_schema::{
            RegistryBuilder,
            model::{
                Authority, EnumDecl, EnumMember, FieldContract, Namespace, RelationDecl,
                SnapshotClass,
            },
        };
        let mut builder = RegistryBuilder::new();
        let mut members = vec![
            EnumMember::new("b", "second"),
            EnumMember::new("a", "first"),
        ];
        if growth {
            members.push(EnumMember::new("c", "later"));
        }
        builder.declare_enum(EnumDecl::platform("Choice", members));
        builder.declare_relation(
            RelationDecl::new(
                Namespace::Authored,
                "recorded_predicates",
                1,
                Authority::Authored,
                SnapshotClass::Model,
                "portable witness control",
            )
            .pk(&["id"])
            .columns(vec![
                FieldContract::key("id", FieldContract::nonnegative(100), "identity"),
                FieldContract::payload(
                    "choices",
                    FieldContract::list(FieldContract::enumeration("Choice")),
                    "nested historical domain",
                ),
            ]),
        );
        let registry = builder.build().unwrap();
        DeclaredCheck::new(
            &registry,
            registry
                .relation("authored.recorded_predicates")
                .unwrap()
                .id,
        )
        .unwrap()
    }
    #[test]
    fn recorded_nested_predicates_preserve_old_domains_without_executable_bytes() {
        use datafusion::arrow::{
            array::{
                Array, ArrayRef, BooleanArray, Int64Array, ListArray, RecordBatch, StringArray,
            },
            buffer::OffsetBuffer,
            datatypes::DataType,
        };
        let old = recorded_fixture(false);
        let current = recorded_fixture(true);
        assert!(
            !old.properties()
                .keys()
                .any(|key| key.starts_with("pse.check.field."))
        );
        let mut properties: std::collections::HashMap<_, _> = old
            .properties()
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        // A sufficient portable v2 witness dispenses with the obsolete codec/bytes.
        properties.insert(
            "pse.check.field.encoding".into(),
            "unsupported-old-codec".into(),
        );
        properties.insert(
            "pse.check.field.expression.choices".into(),
            "not executable bytes".into(),
        );
        let before = properties.clone();
        let state = datafusion::execution::session_state::SessionStateBuilder::new()
            .with_default_features()
            .build();
        let opened = DeclaredCheck::open_fields(
            old.layout.storage_schema().as_ref().clone(),
            &properties,
            &state,
        )
        .unwrap();
        assert_eq!(before, properties);
        assert_eq!(opened.recorded().identity(), old.recorded().identity());
        opened.consumer_projection(&current).unwrap();
        assert!(
            opened
                .recorded()
                .admit_exact_write(
                    current.recorded().contract(),
                    opened.encoding,
                    current.encoding
                )
                .is_err()
        );
        let DataType::List(child) = opened.layout.storage_schema().field(1).data_type() else {
            panic!("list fixture");
        };
        let values: ArrayRef = Arc::new(StringArray::from(vec!["a", "c"]));
        let choices: ArrayRef = Arc::new(ListArray::new(
            child.clone(),
            OffsetBuffer::new(vec![0_i32, 1, 2].into()),
            values,
            None,
        ));
        let batch = RecordBatch::try_new(
            opened.layout.storage_schema().clone(),
            vec![Arc::new(Int64Array::from(vec![1, 2])), choices],
        )
        .unwrap();
        let bound = opened.bind(&state).unwrap();
        let schema = DFSchema::try_from(batch.schema().as_ref().clone()).unwrap();
        let expression = bound
            .create_logical_expr(
                &opened.properties["delta.constraints.pse_contract"],
                &schema,
            )
            .unwrap();
        let result = bound
            .create_physical_expr(expression, &schema)
            .unwrap()
            .evaluate(&batch)
            .unwrap()
            .to_array(2)
            .unwrap();
        let result = result.as_any().downcast_ref::<BooleanArray>().unwrap();
        assert_eq!(result.null_count(), 0);
        assert_eq!(result.values().iter().collect::<Vec<_>>(), [true, false]);
    }
    #[test]
    fn recorded_open_refuses_contradictory_checks_and_encoding() {
        let old = recorded_fixture(false);
        let properties: std::collections::HashMap<_, _> = old
            .properties()
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        let state = datafusion::execution::session_state::SessionStateBuilder::new()
            .with_default_features()
            .build();
        let mut contradiction = properties.clone();
        contradiction.insert("delta.constraints.pse_contract".into(), "TRUE".into());
        assert!(
            DeclaredCheck::open_fields(
                old.layout.storage_schema().as_ref().clone(),
                &contradiction,
                &state
            )
            .is_err()
        );
        let mut contradiction = properties;
        contradiction.insert(
            compatibility::KEY_ENCODING.into(),
            pse_ids::ContentHash::NIL.to_hex(),
        );
        assert!(
            DeclaredCheck::open_fields(
                old.layout.storage_schema().as_ref().clone(),
                &contradiction,
                &state
            )
            .is_err()
        );
    }

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
