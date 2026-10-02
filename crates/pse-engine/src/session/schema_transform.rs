// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Explicit, portable schema evolution compiles into the common native plan.

use super::{EngineSession, PreparedComputation, engine_session::engine, output, scalar};
use crate::EngineError;
use datafusion::{
    arrow::datatypes::{Field, Schema},
    common::{DFSchema, ScalarValue, metadata::FieldMetadata},
    logical_expr::{
        Expr, ExprSchemable, LogicalPlan, LogicalPlanBuilder, conditional_expressions::CaseBuilder,
        lit,
    },
};
use pse_columnar::CancellationToken;
use pse_schema::{
    compatibility::{MigrationAdmission, VerifiedRecordedContract},
    fingerprint::SemanticContract,
    model::{
        MappingPolicy, MigrationNulls, MigrationSpec, MigrationStep, MigrationValues, RelationSpec,
    },
};

#[derive(Clone, Copy)]
enum MappingSource {
    Recorded,
    Target,
}

impl EngineSession {
    /// Prepare one explicitly selected registered transformation through the common compiler.
    /// # Errors
    /// Unknown declaration, absent source, incompatible mapping or native execution policy.
    pub fn prepare_schema_transform(
        &self,
        declaration: &str,
        input_role: &str,
        cancel: &CancellationToken,
    ) -> Result<PreparedComputation, EngineError> {
        let registry = self.registry();
        let declaration = registry
            .migrations()
            .iter()
            .find(|candidate| candidate.qualified_name() == declaration)
            .ok_or_else(|| invalid("schema transformation is not registered"))?;
        let endpoint = |version| {
            registry
                .relations()
                .iter()
                .find(|relation| {
                    relation.qualified_name() == declaration.relation
                        && relation.key.version == version
                })
                .ok_or_else(|| invalid("schema transformation endpoint is not declared"))
        };
        let source = endpoint(declaration.from_version)?;
        let target = endpoint(declaration.to_version)?;
        if !self
            .input_roles()
            .any(|(role, key)| role == input_role && key == source.key)
        {
            return Err(invalid(
                "input role does not bind the declared source version",
            ));
        }
        let verified = |spec: &RelationSpec| {
            VerifiedRecordedContract::from_contract(
                SemanticContract::new(registry, &[spec.id].into())
                    .map_err(pse_columnar::external)
                    .map_err(engine)?,
            )
            .map_err(pse_columnar::external)
            .map_err(engine)
        };
        let admission = MigrationAdmission::new(verified(source)?, verified(target)?);
        let plan = self.plan_schema_transform(
            declaration,
            self.scan_role(input_role)?,
            &admission,
            target,
            cancel,
        )?;
        self.prepare_rule_plan(plan, cancel)
    }

    /// Compile an explicit source/target migration without requiring a historical source
    /// declaration in today's registry. The source fields are independently verified
    /// against its recorded witness. Target contracts come from this session's registry.
    /// No rows are filtered or duplicated; unmatched finite maps fail.
    /// Each column has one value policy; repeated/overlapping maps must be composed
    /// into one finite declaration so reference-closure agreement remains checkable.
    /// # Errors
    /// Wrong exact endpoints, malformed maps, missing semantic/reference mappings or
    /// incompatible native expressions. Execution retains target relational obligations.
    pub fn plan_schema_transform(
        &self,
        declaration: &MigrationSpec,
        input: LogicalPlan,
        admission: &MigrationAdmission,
        target: &RelationSpec,
        cancel: &CancellationToken,
    ) -> Result<LogicalPlan, EngineError> {
        cancel.checkpoint()?;
        let expected = SemanticContract::new(self.registry(), &[target.id].into())
            .map_err(pse_columnar::external)
            .map_err(engine)?;
        if admission.target().contract() != &expected
            || declaration.relation != target.qualified_name()
            || declaration.to_version != target.key.version
            || declaration.from_version > declaration.to_version
            || self
                .registry()
                .relation(declaration.relation)
                .map(|spec| spec.id)
                != Some(target.id)
        {
            return Err(invalid(
                "migration target is not the exact current write declaration",
            ));
        }
        if declaration.from_version == declaration.to_version
            && (admission.source_identity() == admission.target_identity()
                || !declaration.steps.iter().any(|step| {
                    matches!(
                        step,
                        MigrationStep::RecodeDomain { .. } | MigrationStep::MapReferenceKey { .. }
                    )
                }))
        {
            return Err(invalid(
                "same-version migration requires changed recorded meaning and an explicit value policy",
            ));
        }
        let source_name = format!("{}@{}", declaration.relation, declaration.from_version);
        let source_id = admission
            .source()
            .contract()
            .roots
            .iter()
            .copied()
            .find(|id| {
                admission.source().contract().relations[id]["relation"].as_str()
                    == Some(source_name.as_str())
            })
            .ok_or_else(|| invalid("migration source does not match the recorded endpoint"))?;
        let source_schema = Schema::new(input.schema().fields().to_vec());
        let encoding = pse_schema::fingerprint::encoding_fields(
            source_schema.fields().iter().map(AsRef::as_ref),
        )
        .map_err(pse_columnar::external)
        .map_err(engine)?;
        admission
            .source()
            .verify_fields(source_id, &source_schema, encoding)
            .map_err(pse_columnar::external)
            .map_err(engine)?;
        let input = output::forget_relation_annotations(input).map_err(engine)?;
        let mut columns = input
            .schema()
            .columns()
            .into_iter()
            .map(|column| (column.name().to_owned(), Expr::Column(column)))
            .collect::<Vec<_>>();
        let mut origins = columns
            .iter()
            .map(|(name, _)| (name.clone(), MappingSource::Recorded))
            .collect::<std::collections::BTreeMap<_, _>>();
        let mut mapped_columns = std::collections::BTreeSet::new();
        for step in &declaration.steps {
            cancel.checkpoint()?;
            match step {
                MigrationStep::AddColumn { name, default } => {
                    if columns.iter().any(|(column, _)| column == name) {
                        return Err(invalid("default column already exists"));
                    }
                    let column = target
                        .column(name)
                        .ok_or_else(|| invalid("default column is undeclared"))?;
                    let field = pse_schema::arrow::field_for(self.registry(), column)
                        .map_err(pse_columnar::external)
                        .map_err(engine)?;
                    let bound = if default.field().as_ref() == &field {
                        default.clone()
                    } else {
                        default
                            .bind(self.registry())
                            .map_err(pse_columnar::external)
                            .map_err(engine)?
                    };
                    if bound.field().as_ref() != &field {
                        return Err(invalid("default does not carry its exact target field"));
                    }
                    columns.push((
                        (*name).to_owned(),
                        output::checked_array_literal(self.registry(), column, bound.array())
                            .map_err(engine)?,
                    ));
                    origins.insert((*name).to_owned(), MappingSource::Target);
                }
                MigrationStep::DropColumn(name) => {
                    column_mut(&mut columns, name)?;
                    columns.retain(|(column, _)| column != name);
                    origins.remove(*name);
                }
                MigrationStep::RenameColumn { from, to } => {
                    if columns.iter().any(|(column, _)| column == to) {
                        return Err(invalid("rename target already exists"));
                    }
                    (*to).clone_into(&mut column_mut(&mut columns, from)?.0);
                    let origin = origins
                        .remove(*from)
                        .ok_or_else(|| invalid("migration column provenance is absent"))?;
                    origins.insert((*to).to_owned(), origin);
                    if mapped_columns.remove(*from) {
                        mapped_columns.insert((*to).to_owned());
                    }
                }
                MigrationStep::ChangeNullable { name, nullable } => {
                    let (_, value) = column_mut(&mut columns, name)?;
                    *value = if *nullable {
                        scalar::nullable(value.clone())
                    } else {
                        scalar::require_nonnull(value.clone())
                    };
                }
                MigrationStep::RecodeDomain { name, mapping } => {
                    if !mapped_columns.insert((*name).to_owned()) {
                        return Err(invalid(
                            "overlapping value mappings must be declared as one composed finite policy",
                        ));
                    }
                    apply_mapping(
                        self,
                        input.schema(),
                        target,
                        &mut columns,
                        &origins,
                        &[*name],
                        mapping,
                        false,
                        admission,
                    )?;
                }
                MigrationStep::MapReferenceKey {
                    columns: names,
                    mapping,
                } => {
                    if names
                        .iter()
                        .any(|name| !mapped_columns.insert((*name).to_owned()))
                    {
                        return Err(invalid(
                            "overlapping value mappings must be declared as one composed finite policy",
                        ));
                    }
                    apply_mapping(
                        self,
                        input.schema(),
                        target,
                        &mut columns,
                        &origins,
                        names,
                        mapping,
                        true,
                        admission,
                    )?;
                }
            }
        }
        if columns.len() != target.columns.len() {
            return Err(invalid("migration output column inventory differs"));
        }
        let expressions = target
            .columns
            .iter()
            .map(|column| {
                columns
                    .iter()
                    .find(|(name, _)| name == column.name())
                    .map(|(_, expression)| expression.clone().alias(column.name()))
                    .ok_or_else(|| invalid("schema transformation output column is absent"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let plan = LogicalPlanBuilder::from(input)
            .project(expressions)
            .and_then(LogicalPlanBuilder::build)
            .map_err(engine)?;
        output::declare_relation_output(plan, self.registry(), target).map_err(engine)
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "one checked native mapping binds current expression provenance and exact source/target admission"
)]
fn apply_mapping(
    session: &EngineSession,
    schema: &DFSchema,
    target: &RelationSpec,
    columns: &mut [(String, Expr)],
    origins: &std::collections::BTreeMap<String, MappingSource>,
    names: &[&str],
    mapping: &MigrationValues,
    injective: bool,
    admission: &MigrationAdmission,
) -> Result<(), EngineError> {
    let mut unique = std::collections::BTreeSet::new();
    let indices = names
        .iter()
        .map(|name| {
            if !unique.insert(*name) {
                return Err(invalid("mapping column is duplicated"));
            }
            columns
                .iter()
                .position(|(column, _)| column == name)
                .ok_or_else(|| invalid("mapping source column is absent"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let values = indices
        .iter()
        .map(|index| columns[*index].1.clone())
        .collect::<Vec<_>>();
    let source_fields = values
        .iter()
        .map(|value| {
            value
                .to_field(schema)
                .map(|(_, field)| field.as_ref().clone())
                .map_err(engine)
        })
        .collect::<Result<Vec<Field>, _>>()?;
    let target_columns = names
        .iter()
        .map(|name| {
            target
                .column(name)
                .ok_or_else(|| invalid("mapping target column is absent"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let target_fields = target_columns
        .iter()
        .map(|column| {
            pse_schema::arrow::field_for(session.registry(), column)
                .map_err(pse_columnar::external)
                .map_err(engine)
        })
        .collect::<Result<Vec<_>, _>>()?;
    // Ordered expressions own their current field, including prior nullability and
    // rename operations. Provenance chooses recorded versus checked target domains.
    mapping
        .validate_fields(&source_fields, &target_fields, injective)
        .map_err(pse_columnar::external)
        .map_err(engine)?;
    let mapped = match &mapping.policy {
        MappingPolicy::Identity => {
            let checked = session.scalar_function("pse_checked_value")?;
            let presence = if mapping.nulls == MigrationNulls::Preserve && values.len() > 1 {
                let present = values
                    .iter()
                    .cloned()
                    .map(Expr::is_not_null)
                    .reduce(Expr::and)
                    .ok_or_else(|| invalid("empty key mapping"))?;
                let absent = values
                    .iter()
                    .cloned()
                    .map(Expr::is_null)
                    .reduce(Expr::and)
                    .ok_or_else(|| invalid("empty key mapping"))?;
                Some(scalar::require_nonnull(
                    CaseBuilder::new(
                        None,
                        vec![present.or(absent)],
                        vec![lit(true)],
                        Some(Box::new(lit(ScalarValue::Boolean(None)))),
                    )
                    .end()
                    .map_err(engine)?,
                ))
            } else {
                None
            };
            values
                .into_iter()
                .zip(source_fields)
                .zip(&target_fields)
                .map(|((value, source), target_field)| {
                    let value = if let Some(presence) = &presence {
                        CaseBuilder::new(
                            None,
                            vec![presence.clone()],
                            vec![value],
                            Some(Box::new(lit(ScalarValue::try_from(source.data_type())
                                .map_err(datafusion::common::DataFusionError::from)
                                .map_err(engine)?))),
                        )
                        .end()
                        .map_err(engine)?
                    } else {
                        value
                    };
                    let value = if mapping.nulls == MigrationNulls::Reject {
                        scalar::require_nonnull(value)
                    } else {
                        value
                    };
                    output::migration_identity_value(value, &source, target_field, target, &checked)
                        .map_err(engine)
                })
                .collect::<Result<Vec<_>, _>>()?
        }
        MappingPolicy::Finite(rows) => {
            let mut conditions = Vec::new();
            let mut ordinals = Vec::new();
            if mapping.nulls == MigrationNulls::Preserve {
                conditions.push(
                    values
                        .iter()
                        .cloned()
                        .map(Expr::is_null)
                        .reduce(Expr::and)
                        .ok_or_else(|| invalid("empty key mapping"))?,
                );
                ordinals.push(lit(-1_i64));
            }
            for (ordinal, row) in rows.iter().enumerate() {
                let source = row
                    .source
                    .iter()
                    .zip(names)
                    .map(|(literal, name)| {
                        let origin = origins
                            .get(*name)
                            .ok_or_else(|| invalid("mapping source provenance is absent"))?;
                        validate_source_literal(session, literal, admission, *origin)?;
                        Ok(Expr::Literal(
                            literal
                                .scalar()
                                .map_err(pse_columnar::external)
                                .map_err(engine)?,
                            Some(FieldMetadata::from(literal.field().as_ref())),
                        ))
                    })
                    .collect::<Result<Vec<_>, EngineError>>()?;
                conditions.push(
                    values
                        .iter()
                        .cloned()
                        .zip(source)
                        .map(|(value, source)| value.eq(source))
                        .reduce(Expr::and)
                        .ok_or_else(|| invalid("empty key mapping"))?,
                );
                ordinals.push(lit(
                    i64::try_from(ordinal).map_err(|_| invalid("mapping extent overflows"))?
                ));
            }
            let selected = scalar::require_nonnull(
                CaseBuilder::new(
                    None,
                    conditions,
                    ordinals,
                    Some(Box::new(lit(ScalarValue::Int64(None)))),
                )
                .end()
                .map_err(engine)?,
            );
            target_columns
                .iter()
                .enumerate()
                .map(|(slot, column)| {
                    let mut conditions = Vec::new();
                    let mut alternatives = Vec::new();
                    for (ordinal, row) in rows.iter().enumerate() {
                        conditions.push(
                            selected.clone().eq(lit(i64::try_from(ordinal)
                                .map_err(|_| invalid("mapping extent overflows"))?)),
                        );
                        alternatives.push(
                            output::checked_array_literal(
                                session.registry(),
                                column,
                                row.target[slot].array(),
                            )
                            .map_err(engine)?,
                        );
                    }
                    let null = ScalarValue::try_from(target_fields[slot].data_type())
                        .map_err(datafusion::common::DataFusionError::from)
                        .map_err(engine)?;
                    let fallback = output::checked_literal(
                        session.registry(),
                        &(*column).clone().optional(),
                        null,
                    )
                    .map_err(engine)?;
                    let expression = output::same_field_cases(
                        schema,
                        conditions.into_iter().zip(alternatives).collect(),
                        fallback,
                    )
                    .map_err(engine)?;
                    Ok(if column.nullable() {
                        expression
                    } else {
                        scalar::require_nonnull(expression)
                    })
                })
                .collect::<Result<Vec<_>, EngineError>>()?
        }
    };
    for (index, expression) in indices.into_iter().zip(mapped) {
        columns[index].1 = expression;
    }
    Ok(())
}

fn validate_source_literal(
    session: &EngineSession,
    literal: &pse_schema::NativeLiteral,
    admission: &MigrationAdmission,
    origin: MappingSource,
) -> Result<(), EngineError> {
    use datafusion::arrow::array::{Array, BooleanArray, RecordBatch};
    use pse_relations::validate::planner::ValidationPlanner;
    let domains = match origin {
        MappingSource::Recorded => {
            pse_relations::validate::predicates::DomainEnvironment::recorded(admission.source())
                .map_err(engine)?
        }
        MappingSource::Target => {
            pse_relations::validate::predicates::DomainEnvironment::current(session.registry())
        }
    };
    let schema = Schema::new(vec![literal.field().as_ref().clone()]);
    let batch = RecordBatch::try_new(
        std::sync::Arc::new(schema.clone()),
        vec![literal.array().clone()],
    )
    .map_err(datafusion::common::DataFusionError::from)
    .map_err(engine)?;
    let expression = pse_relations::validate::predicates::field_value_in(
        &domains,
        literal.field(),
        Expr::Column(datafusion::common::Column::from_name(
            literal.field().name(),
        )),
        0,
    )
    .map_err(engine)?;
    let value = crate::validation::NativeValidation(session.bound_state()?)
        .prepare(expression, &DFSchema::try_from(schema).map_err(engine)?)
        .map_err(engine)?
        .evaluate(&batch)
        .map_err(engine)?
        .to_array(1)
        .map_err(engine)?;
    let valid = value
        .as_any()
        .downcast_ref::<BooleanArray>()
        .ok_or_else(|| invalid("mapping predicate is not boolean"))?;
    if valid.is_null(0) || !valid.value(0) {
        return Err(invalid(
            "mapping literal lies outside its recorded source domain",
        ));
    }

    Ok(())
}

fn column_mut<'a>(
    columns: &'a mut [(String, Expr)],
    name: &str,
) -> Result<&'a mut (String, Expr), EngineError> {
    columns
        .iter_mut()
        .find(|(column, _)| column == name)
        .ok_or_else(|| invalid("schema transformation source column is absent"))
}
fn invalid(reason: &str) -> EngineError {
    EngineError::Admission {
        path: "schema transformation".to_owned(),
        reason: reason.to_owned(),
    }
}

#[cfg(test)]
mod mapping_unit {
    use super::*;
    use datafusion::{
        arrow::{
            array::{ArrayRef, Int64Array, RecordBatch},
            datatypes::DataType,
        },
        common::TableReference,
        datasource::MemTable,
        execution::{runtime_env::RuntimeEnv, session_state::SessionStateBuilder},
    };
    use pse_schema::{
        RegistryBuilder,
        model::{
            Authority, FieldContract, MigrationValuePair, Namespace, RelationDecl, SnapshotClass,
        },
    };
    use std::{collections::BTreeMap, sync::Arc};

    fn fixture(
        values: Vec<Option<i64>>,
        nullable: bool,
    ) -> (
        EngineSession,
        LogicalPlan,
        MigrationAdmission,
        MigrationSpec,
    ) {
        let declaration = |version| {
            RelationDecl::new(
                Namespace::Authored,
                "keys",
                version,
                Authority::Authored,
                SnapshotClass::Model,
                "keys",
            )
            .pk(&[])
            .columns(vec![
                FieldContract::payload("key", FieldContract::native(DataType::Int64), "value")
                    .with_nullable(nullable),
            ])
        };
        let mut old = RegistryBuilder::new();
        old.declare_relation(declaration(1));
        let old = old.build().unwrap();
        let mut current = RegistryBuilder::new();
        current.declare_relation(declaration(2));
        let current = Arc::new(current.build().unwrap());
        let source = old.relation("authored.keys").unwrap();
        let target = current.relation("authored.keys").unwrap();
        let source_schema = Arc::new(pse_schema::arrow::relation_schema(&old, source).unwrap());
        let target_schema = pse_schema::arrow::relation_schema(&current, target).unwrap();
        let source_contract = VerifiedRecordedContract::from_contract(
            SemanticContract::new(&old, &[source.id].into()).unwrap(),
        )
        .unwrap();
        let target_contract = VerifiedRecordedContract::from_contract(
            SemanticContract::new(&current, &[target.id].into()).unwrap(),
        )
        .unwrap();
        let array: ArrayRef = Arc::new(Int64Array::from(values));
        let batch = RecordBatch::try_new(source_schema.clone(), vec![array]).unwrap();
        let provider =
            Arc::new(MemTable::try_new(source_schema.clone(), vec![vec![batch]]).unwrap());
        let factory = super::super::EngineFactory::from_builder(
            Arc::new(RuntimeEnv::default()),
            Arc::new(pse_columnar::GreedyMemoryPool::new(32 << 20)),
            "migration-unit",
            SessionStateBuilder::new_with_default_features(),
        );
        let cancel = CancellationToken::new();
        let mut session = factory
            .candidate(BTreeMap::new(), current, &cancel)
            .unwrap();
        let reference = TableReference::full("recorded", "authored", "keys");
        session
            .bind_source(
                crate::provider::binding::BindingKey::Native(reference.clone()),
                crate::provider::binding::TableBinding::new(
                    reference.clone(),
                    provider.clone(),
                    None,
                    None,
                ),
            )
            .unwrap();
        let input = LogicalPlanBuilder::scan(
            reference,
            datafusion::datasource::provider_as_source(provider),
            None,
        )
        .unwrap()
        .build()
        .unwrap();
        let row = |from, to| MigrationValuePair {
            source: vec![
                pse_schema::NativeLiteral::from_scalar(
                    source_schema.fields()[0].clone(),
                    &ScalarValue::Int64(Some(from)),
                )
                .unwrap(),
            ],
            target: vec![
                pse_schema::NativeLiteral::from_scalar(
                    Arc::new(target_schema.field(0).clone()),
                    &ScalarValue::Int64(Some(to)),
                )
                .unwrap(),
            ],
        };
        let spec = MigrationSpec {
            relation: "authored.keys",
            from_version: 1,
            to_version: 2,
            steps: vec![MigrationStep::RecodeDomain {
                name: "key",
                mapping: MigrationValues {
                    policy: MappingPolicy::Finite(vec![row(1, 11), row(2, 22)]),
                    nulls: if nullable {
                        MigrationNulls::Preserve
                    } else {
                        MigrationNulls::Reject
                    },
                },
            }],
            doc: "finite map",
        };
        (
            session,
            input,
            MigrationAdmission::new(source_contract, target_contract),
            spec,
        )
    }
    fn enum_fixture(
        add_column: bool,
    ) -> (
        EngineSession,
        LogicalPlan,
        MigrationAdmission,
        MigrationSpec,
    ) {
        use pse_schema::model::{EnumDecl, EnumMember};
        let registry = |members: &[&'static str]| {
            let mut builder = RegistryBuilder::new();
            builder.declare_enum(EnumDecl::platform(
                "Choice",
                members
                    .iter()
                    .map(|member| EnumMember::new(member, "choice"))
                    .collect(),
            ));
            let mut columns = vec![FieldContract::payload(
                "key",
                FieldContract::enumeration("Choice"),
                "choice",
            )];
            if add_column && members.len() == 3 {
                columns.push(FieldContract::payload(
                    "added",
                    FieldContract::enumeration("Choice"),
                    "added choice",
                ));
            }
            builder.declare_relation(
                RelationDecl::new(
                    Namespace::Authored,
                    "keys",
                    1,
                    Authority::Authored,
                    SnapshotClass::Model,
                    "keys",
                )
                .pk(&[])
                .columns(columns),
            );
            builder.build().unwrap()
        };
        let old = registry(&["a", "b"]);
        let current = Arc::new(registry(&["a", "b", "c"]));
        let source = old.relation("authored.keys").unwrap();
        let target = current.relation("authored.keys").unwrap();
        let source_schema = Arc::new(pse_schema::arrow::relation_schema(&old, source).unwrap());
        let batch = RecordBatch::try_new(
            source_schema.clone(),
            vec![Arc::new(datafusion::arrow::array::StringArray::from(vec![
                "a", "b",
            ]))],
        )
        .unwrap();
        let provider = Arc::new(MemTable::try_new(source_schema, vec![vec![batch]]).unwrap());
        let admission = MigrationAdmission::new(
            VerifiedRecordedContract::from_contract(
                SemanticContract::new(&old, &[source.id].into()).unwrap(),
            )
            .unwrap(),
            VerifiedRecordedContract::from_contract(
                SemanticContract::new(&current, &[target.id].into()).unwrap(),
            )
            .unwrap(),
        );
        let factory = super::super::EngineFactory::from_builder(
            Arc::new(RuntimeEnv::default()),
            Arc::new(pse_columnar::GreedyMemoryPool::new(32 << 20)),
            "migration-enum-unit",
            SessionStateBuilder::new_with_default_features(),
        );
        let mut session = factory
            .candidate(BTreeMap::new(), current, &CancellationToken::new())
            .unwrap();
        let reference = TableReference::full("recorded", "authored", "keys");
        session
            .bind_source(
                crate::provider::binding::BindingKey::Native(reference.clone()),
                crate::provider::binding::TableBinding::new(
                    reference.clone(),
                    provider.clone(),
                    None,
                    None,
                ),
            )
            .unwrap();
        let input = LogicalPlanBuilder::scan(
            reference,
            datafusion::datasource::provider_as_source(provider),
            None,
        )
        .unwrap()
        .build()
        .unwrap();
        let spec = MigrationSpec {
            relation: "authored.keys",
            from_version: 1,
            to_version: 1,
            steps: vec![MigrationStep::RecodeDomain {
                name: "key",
                mapping: MigrationValues {
                    policy: MappingPolicy::Identity,
                    nulls: MigrationNulls::Reject,
                },
            }],
            doc: "explicit changed enum support",
        };
        (session, input, admission, spec)
    }
    #[tokio::test]
    async fn same_version_support_growth_requires_explicit_policy_and_exact_target_values() {
        let (session, input, admission, mut spec) = enum_fixture(false);
        assert_ne!(admission.source_identity(), admission.target_identity());
        let target = session.registry().relation("authored.keys").unwrap();
        let cancel = CancellationToken::new();
        let plan = session
            .plan_schema_transform(&spec, input.clone(), &admission, target, &cancel)
            .unwrap();
        let completed = session
            .prepare_rule_plan(plan, &cancel)
            .unwrap()
            .execute(&cancel)
            .await
            .unwrap();
        assert_eq!(
            completed
                .batches()
                .iter()
                .map(|batch| batch.batch().num_rows())
                .sum::<usize>(),
            2
        );
        let target_field = Arc::new(
            pse_schema::arrow::relation_schema(session.registry(), target)
                .unwrap()
                .field(0)
                .clone(),
        );
        assert_eq!(
            completed.batches()[0].batch().schema().field(0),
            target_field.as_ref()
        );
        spec.steps.clear();
        assert!(
            session
                .plan_schema_transform(&spec, input.clone(), &admission, target, &cancel)
                .is_err()
        );
        let source_field = input
            .schema()
            .field_with_unqualified_name("key")
            .unwrap()
            .clone();
        let literal = |field, value: &str| {
            pse_schema::NativeLiteral::from_scalar(
                field,
                &ScalarValue::Utf8(Some(value.to_owned())),
            )
            .unwrap()
        };
        spec.steps.push(MigrationStep::RecodeDomain {
            name: "key",
            mapping: MigrationValues {
                policy: MappingPolicy::Finite(vec![MigrationValuePair {
                    source: vec![literal(source_field, "a")],
                    target: vec![literal(target_field, "undeclared")],
                }]),
                nulls: MigrationNulls::Reject,
            },
        });
        assert!(
            session
                .plan_schema_transform(&spec, input, &admission, target, &cancel)
                .is_err()
        );
    }
    #[tokio::test]
    async fn mapping_uses_current_nullability_after_structural_change() {
        let (session, input, admission, mut spec) = fixture(vec![Some(1)], false);
        if let MigrationStep::RecodeDomain {
            mapping:
                MigrationValues {
                    policy: MappingPolicy::Finite(rows),
                    ..
                },
            ..
        } = &mut spec.steps[0]
        {
            for row in rows {
                let literal = &row.source[0];
                row.source[0] = pse_schema::NativeLiteral::from_scalar(
                    Arc::new(literal.field().as_ref().clone().with_nullable(true)),
                    &literal.scalar().unwrap(),
                )
                .unwrap();
            }
        }
        spec.steps.insert(
            0,
            MigrationStep::ChangeNullable {
                name: "key",
                nullable: true,
            },
        );
        let cancel = CancellationToken::new();
        let plan = session
            .plan_schema_transform(
                &spec,
                input,
                &admission,
                session.registry().relation("authored.keys").unwrap(),
                &cancel,
            )
            .unwrap();
        let completed = session
            .prepare_rule_plan(plan, &cancel)
            .unwrap()
            .execute(&cancel)
            .await
            .unwrap();
        assert_eq!(
            completed.batches()[0]
                .batch()
                .column(0)
                .as_any()
                .downcast_ref::<Int64Array>()
                .unwrap()
                .value(0),
            11
        );
    }
    #[tokio::test]
    async fn added_column_mapping_uses_checked_target_domain_provenance() {
        let (session, input, admission, mut spec) = enum_fixture(true);
        let target = session.registry().relation("authored.keys").unwrap();
        let field = Arc::new(
            pse_schema::arrow::relation_schema(session.registry(), target)
                .unwrap()
                .field(1)
                .clone(),
        );
        let value = |text: &str| {
            pse_schema::NativeLiteral::from_scalar(
                field.clone(),
                &ScalarValue::Utf8(Some(text.to_owned())),
            )
            .unwrap()
        };
        spec.steps.push(MigrationStep::AddColumn {
            name: "added",
            default: value("c"),
        });
        spec.steps.push(MigrationStep::RecodeDomain {
            name: "added",
            mapping: MigrationValues {
                policy: MappingPolicy::Finite(vec![MigrationValuePair {
                    source: vec![value("c")],
                    target: vec![value("a")],
                }]),
                nulls: MigrationNulls::Reject,
            },
        });
        let cancel = CancellationToken::new();
        let plan = session
            .plan_schema_transform(&spec, input, &admission, target, &cancel)
            .unwrap();
        let completed = session
            .prepare_rule_plan(plan, &cancel)
            .unwrap()
            .execute(&cancel)
            .await
            .unwrap();
        let values = completed
            .batches()
            .iter()
            .flat_map(|batch| {
                batch
                    .batch()
                    .column(1)
                    .as_any()
                    .downcast_ref::<datafusion::arrow::array::StringArray>()
                    .unwrap()
                    .iter()
            })
            .collect::<Vec<_>>();
        assert_eq!(values, vec![Some("a"), Some("a")]);
    }
    #[tokio::test]
    async fn recorded_source_absent_from_current_registry_maps_without_changing_cardinality() {
        let (session, input, admission, spec) = fixture(vec![Some(2), Some(1), Some(2)], false);
        assert!(
            session
                .registry()
                .relation_by_id(*admission.source().contract().roots.first().unwrap())
                .is_none()
        );
        let target = session.registry().relation("authored.keys").unwrap();
        let cancel = CancellationToken::new();
        let plan = session
            .plan_schema_transform(&spec, input, &admission, target, &cancel)
            .unwrap();
        let completed = session
            .prepare_rule_plan(plan, &cancel)
            .unwrap()
            .execute(&cancel)
            .await
            .unwrap();
        let values = completed
            .batches()
            .iter()
            .flat_map(|batch| {
                batch
                    .batch()
                    .column(0)
                    .as_any()
                    .downcast_ref::<Int64Array>()
                    .unwrap()
                    .iter()
            })
            .collect::<Vec<_>>();
        assert_eq!(values, vec![Some(22), Some(11), Some(22)]);
    }
    #[tokio::test]
    async fn finite_mapping_refuses_unmatched_and_preserves_only_declared_absence() {
        let (session, input, admission, spec) = fixture(vec![Some(99)], true);
        let cancel = CancellationToken::new();
        let target = session.registry().relation("authored.keys").unwrap();
        let plan = session
            .plan_schema_transform(&spec, input, &admission, target, &cancel)
            .unwrap();
        assert!(
            session
                .prepare_rule_plan(plan, &cancel)
                .unwrap()
                .execute(&cancel)
                .await
                .is_err()
        );
        let (session, input, admission, spec) = fixture(vec![None, Some(1)], true);
        let target = session.registry().relation("authored.keys").unwrap();
        let plan = session
            .plan_schema_transform(&spec, input, &admission, target, &cancel)
            .unwrap();
        let completed = session
            .prepare_rule_plan(plan, &cancel)
            .unwrap()
            .execute(&cancel)
            .await
            .unwrap();
        let values = completed
            .batches()
            .iter()
            .flat_map(|batch| {
                batch
                    .batch()
                    .column(0)
                    .as_any()
                    .downcast_ref::<Int64Array>()
                    .unwrap()
                    .iter()
            })
            .collect::<Vec<_>>();
        assert_eq!(values, vec![None, Some(11)]);
    }
    #[test]
    fn duplicate_map_and_wrong_recorded_endpoint_refuse_before_execution() {
        let (session, input, admission, mut spec) = fixture(vec![Some(1)], false);
        let target = session.registry().relation("authored.keys").unwrap();
        let cancel = CancellationToken::new();
        if let MigrationStep::RecodeDomain {
            mapping:
                MigrationValues {
                    policy: MappingPolicy::Finite(rows),
                    ..
                },
            ..
        } = &mut spec.steps[0]
        {
            rows.push(rows[0].clone());
        }
        assert!(
            session
                .plan_schema_transform(&spec, input.clone(), &admission, target, &cancel)
                .is_err()
        );
        spec.from_version = 0;
        assert!(
            session
                .plan_schema_transform(&spec, input, &admission, target, &cancel)
                .is_err()
        );
        let (session, input, admission, mut spec) = fixture(vec![Some(1)], false);
        spec.steps.push(spec.steps[0].clone());
        assert!(
            session
                .plan_schema_transform(
                    &spec,
                    input,
                    &admission,
                    session.registry().relation("authored.keys").unwrap(),
                    &cancel
                )
                .is_err()
        );
    }
}
