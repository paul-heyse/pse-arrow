// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native column projections of the sole declarations, including bootstrap schemas.
use super::{SELF_DESCRIBING_RELATIONS, SchemaBatches, quantity_type_id};
use crate::{
    Registry, RegistryBuilder, SchemaError,
    model::{QuantityContract, RelationSpec},
};
use arrow_array::{Array, ArrayRef, RecordBatch};
use arrow_schema::SchemaRef;
use std::sync::{Arc, LazyLock};
mod arrays;
use arrays::{boolean, ids, int, list, structure, structure_nullable, text};

// Resolve declarations only: never call `build`, fingerprint the whole registry, or
// materialize self-description from here. The declarations are the production catalog's.
static BOOTSTRAP: LazyLock<Result<Registry, SchemaError>> = LazyLock::new(|| {
    let mut builder = RegistryBuilder::new();
    crate::catalog::declare(&mut builder);
    let mut registry = builder.resolve_base()?;
    builder.derive_integrity();
    registry.invariants = super::resolve_invariants(builder.invariants, &registry)?;
    registry
        .invariants
        .sort_by_key(crate::model::InvariantSpec::qualified_name);
    let fingerprints = registry
        .relations
        .iter()
        .map(|spec| {
            crate::fingerprint::semantic_product(
                &registry,
                &std::collections::BTreeSet::from([spec.id]),
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    for (spec, fingerprint) in registry.relations.iter_mut().zip(fingerprints) {
        spec.fingerprint = fingerprint;
    }
    Ok(registry)
});

pub(super) fn materialize(reg: &Registry) -> Result<SchemaBatches, SchemaError> {
    let bootstrap = BOOTSTRAP.as_ref().map_err(Clone::clone)?;
    SELF_DESCRIBING_RELATIONS
        .iter()
        .map(|name| {
            let owner = if reg.relation(name).is_some() {
                reg
            } else {
                bootstrap
            };
            let spec = owner
                .relation(name)
                .ok_or_else(|| invalid(format!("missing intrinsic declaration {name}")))?;
            let columns = columns(reg, name)?;
            let schema = Arc::new(crate::arrow::relation_schema(owner, spec)?);
            let batch = project(schema, columns)?;
            Ok((spec.key, sorted(spec, batch)?))
        })
        .collect()
}

fn project(schema: SchemaRef, columns: Vec<ArrayRef>) -> Result<RecordBatch, SchemaError> {
    if columns.len() != schema.fields().len() {
        return Err(invalid("projection width differs from declaration"));
    }
    let columns = columns
        .into_iter()
        .zip(schema.fields())
        .map(|(array, field)| arrays::align(array, field.data_type()))
        .collect::<Result<_, _>>()?;
    let batch = RecordBatch::try_new(schema, columns).map_err(invalid)?;
    for array in batch.columns() {
        array.to_data().validate_full().map_err(invalid)?;
    }
    Ok(batch)
}
fn sorted(spec: &RelationSpec, batch: RecordBatch) -> Result<RecordBatch, SchemaError> {
    if batch.num_rows() == 0 {
        return Ok(batch);
    }
    let keys = spec
        .primary_key
        .iter()
        .map(|name| {
            let array = batch
                .column_by_name(name)
                .ok_or_else(|| invalid(format!("missing key {name}")))?;
            Ok(arrow_ord::sort::SortColumn {
                values: Arc::clone(array),
                options: None,
            })
        })
        .collect::<Result<Vec<_>, SchemaError>>()?;
    if keys.is_empty() {
        return Err(invalid(format!(
            "intrinsic relation {} requires a key",
            spec.key
        )));
    }
    let indices = arrow_ord::sort::lexsort_to_indices(&keys, None).map_err(invalid)?;
    arrow_select::take::take_record_batch(&batch, &indices).map_err(invalid)
}
fn invalid(error: impl std::fmt::Display) -> SchemaError {
    crate::checks::invalid("native registry projection", error.to_string())
}
fn ordinal(value: usize) -> Result<i64, SchemaError> {
    i64::try_from(value).map_err(invalid)
}
fn target(reg: &Registry, name: &str) -> Result<pse_ids::SemanticId, SchemaError> {
    reg.relation(name)
        .map(|spec| spec.id)
        .ok_or_else(|| invalid(format!("missing relation {name}")))
}
#[allow(
    clippy::too_many_lines,
    reason = "one native column projection per intrinsic relation"
)]
fn columns(reg: &Registry, name: &str) -> Result<Vec<ArrayRef>, SchemaError> {
    Ok(match name {
        "reference.schema_relations" => relations(reg)?,
        "reference.schema_columns" => fields(reg)?,
        "reference.schema_logical_types" => vec![
            ids(reg.logical_types.iter().map(|r| Some(r.id)))?,
            text(reg.logical_types.iter().map(|r| Some(r.name.as_str()))),
            text(
                reg.logical_types
                    .iter()
                    .map(|r| Some(r.arrow_storage.as_str())),
            ),
            text(
                reg.logical_types
                    .iter()
                    .map(|r| r.extension_name.as_deref()),
            ),
            text(
                reg.logical_types
                    .iter()
                    .map(|r| r.metadata_schema.as_deref()),
            ),
        ],
        "reference.schema_enums" => {
            let members = reg
                .enums
                .iter()
                .flat_map(|e| e.members.iter().enumerate().map(move |(i, m)| (e, i, m)))
                .collect::<Vec<_>>();
            vec![
                ids(members.iter().map(|(e, _, _)| Some(e.id)))?,
                int(members
                    .iter()
                    .map(|(_, i, _)| ordinal(*i).map(Some))
                    .collect::<Result<Vec<_>, _>>()?),
                text(members.iter().map(|(_, _, m)| Some(m.name))),
                text(members.iter().map(|(_, _, m)| m.idaes_name)),
                boolean(members.iter().map(|(_, _, m)| m.deprecated)),
                text(members.iter().map(|(_, _, m)| Some(m.doc))),
            ]
        }
        "reference.schema_enum_types" => vec![
            ids(reg.enums.iter().map(|r| Some(r.id)))?,
            text(reg.enums.iter().map(|r| Some(r.name))),
            text(reg.enums.iter().map(|r| r.idaes_source)),
        ],
        "reference.schema_invariants" => vec![
            ids(reg.invariants.iter().map(|r| Some(r.id)))?,
            ids(reg
                .invariants
                .iter()
                .map(|r| target(reg, &r.relation).map(Some))
                .collect::<Result<Vec<_>, _>>()?)?,
            text(reg.invariants.iter().map(|r| Some(r.kind.as_str()))),
            text(reg.invariants.iter().map(|r| Some(r.query.as_str()))),
            list(
                reg.invariants.iter().map(|r| r.inputs.len()),
                text(
                    reg.invariants
                        .iter()
                        .flat_map(|r| r.inputs.iter().map(|v| Some(v.as_str()))),
                ),
            )?,
            list(
                reg.invariants.iter().map(|r| r.key_columns.len()),
                text(
                    reg.invariants
                        .iter()
                        .flat_map(|r| r.key_columns.iter().copied().map(Some)),
                ),
            )?,
            text(reg.invariants.iter().map(|r| Some(r.severity.as_str()))),
            text(reg.invariants.iter().map(|r| Some(r.doc))),
        ],
        "reference.schema_migrations" => vec![
            ids(reg
                .migrations
                .iter()
                .map(|r| target(reg, r.relation).map(Some))
                .collect::<Result<Vec<_>, _>>()?)?,
            int(reg
                .migrations
                .iter()
                .map(|r| Some(i64::from(r.from_version)))),
            int(reg.migrations.iter().map(|r| Some(i64::from(r.to_version)))),
            text(reg.migrations.iter().map(|r| Some(r.plan_spec()))),
            text(reg.migrations.iter().map(|r| Some(r.doc))),
        ],
        "reference.algorithm_specs" => algorithms(reg)?,
        "reference.algorithm_arguments" => arguments(reg)?,
        "reference.algorithm_results" => {
            let ports = reg
                .algorithms
                .iter()
                .flat_map(|a| a.outputs.iter().map(move |p| (a, p)))
                .collect::<Vec<_>>();
            vec![
                ids(ports.iter().map(|(a, _)| Some(a.id)))?,
                text(ports.iter().map(|(_, p)| Some(p.port.as_str()))),
                ids(ports
                    .iter()
                    .map(|(_, p)| target(reg, &p.relation).map(Some))
                    .collect::<Result<Vec<_>, _>>()?)?,
            ]
        }
        "reference.schema_documents" => vec![
            text(reg.documents.iter().map(|d| Some(d.name))),
            text(reg.documents.iter().map(|d| Some(d.kind.as_str()))),
            text(reg.documents.iter().map(|d| Some(d.path_glob))),
            text(reg.documents.iter().map(|d| Some(d.doc))),
        ],
        "reference.schema_document_sections" => sections(reg)?,
        "reference.artifact_profiles" => vec![
            text(reg.artifact_profiles.keys().map(Some)),
            list(
                reg.artifact_profiles
                    .values()
                    .map(std::collections::BTreeSet::len),
                ids(reg
                    .artifact_profiles
                    .values()
                    .flat_map(|v| v.iter().copied().map(Some)))?,
            )?,
        ],
        "reference.function_capabilities" => functions(),
        _ => return Err(invalid(format!("unknown self-description {name}"))),
    })
}
fn relations(reg: &Registry) -> Result<Vec<ArrayRef>, SchemaError> {
    let rows = &reg.relations;
    Ok(vec![
        ids(rows.iter().map(|r| Some(r.id)))?,
        text(rows.iter().map(|r| Some(r.key.namespace.as_str()))),
        text(rows.iter().map(|r| Some(r.key.name))),
        int(rows.iter().map(|r| Some(i64::from(r.key.version)))),
        text(rows.iter().map(|r| Some(r.authority.as_str()))),
        text(rows.iter().map(|r| Some(r.snapshot_class.as_str()))),
        list(
            rows.iter().map(|r| r.primary_key.len()),
            text(
                rows.iter()
                    .flat_map(|r| r.primary_key.iter().copied().map(Some)),
            ),
        )?,
        text(rows.iter().map(|r| {
            r.derivation_granularity
                .map(crate::model::DerivationGranularity::as_str)
        })),
        text(rows.iter().map(|r| Some(r.stability.as_str()))),
        text(rows.iter().map(|r| Some(r.doc))),
        list(
            rows.iter().map(|r| r.checks.len()),
            structure(vec![
                (
                    "name",
                    text(rows.iter().flat_map(|r| r.checks.keys().map(Some))),
                ),
                (
                    "sql",
                    text(rows.iter().flat_map(|r| r.checks.values().map(Some))),
                ),
            ])?,
        )?,
        list(
            rows.iter().map(|r| r.delta_properties.len()),
            structure(vec![
                (
                    "name",
                    text(
                        rows.iter()
                            .flat_map(|r| r.delta_properties.keys().map(Some)),
                    ),
                ),
                (
                    "value",
                    text(
                        rows.iter()
                            .flat_map(|r| r.delta_properties.values().map(Some)),
                    ),
                ),
            ])?,
        )?,
    ])
}
fn fields(reg: &Registry) -> Result<Vec<ArrayRef>, SchemaError> {
    let rows = reg
        .relations
        .iter()
        .flat_map(|r| r.columns.iter().enumerate().map(move |(i, c)| (r, i, c)))
        .collect::<Vec<_>>();
    Ok(vec![
        ids(rows.iter().map(|(r, _, _)| Some(r.id)))?,
        int(rows
            .iter()
            .map(|(_, i, _)| ordinal(*i).map(Some))
            .collect::<Result<Vec<_>, _>>()?),
        text(rows.iter().map(|(_, _, c)| Some(c.name()))),
        ids(rows
            .iter()
            .map(|(_, _, c)| Ok(reg.logical_type(&c.value_type().type_name()?).map(|r| r.id)))
            .collect::<Result<Vec<_>, SchemaError>>()?)?,
        boolean(rows.iter().map(|(_, _, c)| c.nullable())),
        ids(rows.iter().map(|(_, _, c)| match c.quantity() {
            QuantityContract::Column(name) => Some(quantity_type_id(name)),
            QuantityContract::None => None,
        }))?,
        ids(rows.iter().map(|(_, _, c)| {
            c.fk()
                .and_then(|fk| reg.relation(fk.relation).map(|r| r.id))
        }))?,
        text(rows.iter().map(|(_, _, c)| c.fk().map(|fk| fk.column))),
        text(rows.iter().map(|(_, _, c)| Some(c.role().as_str()))),
        text(rows.iter().map(|(_, _, c)| Some(c.doc()))),
        text(
            rows.iter()
                .map(|(_, _, c)| c.canonical_json().map(Some))
                .collect::<Result<Vec<_>, _>>()?,
        ),
    ])
}
fn algorithms(reg: &Registry) -> Result<Vec<ArrayRef>, SchemaError> {
    let rows = &reg.algorithms;
    let pre = rows
        .iter()
        .flat_map(|r| &r.preconditions)
        .map(|v| {
            reg.invariant_id(v)
                .map(Some)
                .ok_or_else(|| invalid(format!("missing invariant {v}")))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let post = rows
        .iter()
        .flat_map(|r| &r.postconditions)
        .map(|v| {
            reg.invariant_id(v)
                .map(Some)
                .ok_or_else(|| invalid(format!("missing invariant {v}")))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(vec![
        ids(rows.iter().map(|r| Some(r.id)))?,
        text(rows.iter().map(|r| Some(r.name))),
        text(rows.iter().map(|r| Some(r.version))),
        list(rows.iter().map(|r| r.preconditions.len()), ids(pre)?)?,
        list(rows.iter().map(|r| r.postconditions.len()), ids(post)?)?,
        text(rows.iter().map(|r| Some(r.determinism.as_str()))),
        list(
            rows.iter().map(|r| r.diagnostics.len()),
            text(
                rows.iter()
                    .flat_map(|r| r.diagnostics.iter().copied().map(Some)),
            ),
        )?,
        list(
            rows.iter().map(|r| r.effects.len()),
            text(
                rows.iter()
                    .flat_map(|r| r.effects.iter().map(|v| Some(v.as_str()))),
            ),
        )?,
    ])
}
fn arguments(reg: &Registry) -> Result<Vec<ArrayRef>, SchemaError> {
    use crate::model::algorithm::InputConsumption;
    let rows = reg
        .algorithms
        .iter()
        .flat_map(|r| r.inputs.iter().map(move |p| (r, p)))
        .collect::<Vec<_>>();
    let names = rows
        .iter()
        .map(|(_, p)| match &p.consumption {
            InputConsumption::Whole => Vec::new(),
            InputConsumption::Columns(names) => names.iter().collect::<Vec<_>>(),
        })
        .collect::<Vec<_>>();
    let fields = structure_nullable(
        vec![(
            "names",
            list(
                names.iter().map(Vec::len),
                text(names.iter().flat_map(|v| v.iter().map(Some))),
            )?,
        )],
        Some(
            rows.iter()
                .map(|(_, p)| matches!(p.consumption, InputConsumption::Columns(_)))
                .collect(),
        ),
    )?;
    let consumption = structure(vec![
        (
            "kind",
            text(rows.iter().map(|(_, p)| {
                Some(match p.consumption {
                    InputConsumption::Whole => "whole",
                    InputConsumption::Columns(_) => "columns",
                })
            })),
        ),
        ("columns", fields),
    ])?;
    Ok(vec![
        ids(rows.iter().map(|(r, _)| Some(r.id)))?,
        text(rows.iter().map(|(_, p)| Some(p.port.as_str()))),
        ids(rows
            .iter()
            .map(|(_, p)| target(reg, &p.relation).map(Some))
            .collect::<Result<Vec<_>, _>>()?)?,
        boolean(rows.iter().map(|(_, p)| p.required)),
        consumption,
    ])
}

fn sections(reg: &Registry) -> Result<Vec<ArrayRef>, SchemaError> {
    let rows = reg
        .documents
        .iter()
        .flat_map(|d| d.sections.iter().enumerate().map(move |(i, s)| (d, i, s)))
        .collect::<Vec<_>>();
    Ok(vec![
        text(rows.iter().map(|(d, _, _)| Some(d.name))),
        int(rows
            .iter()
            .map(|(_, i, _)| ordinal(*i).map(Some))
            .collect::<Result<Vec<_>, _>>()?),
        text(rows.iter().map(|(_, _, s)| Some(s.key))),
        ids(rows
            .iter()
            .map(|(_, _, s)| target(reg, s.relation).map(Some))
            .collect::<Result<Vec<_>, _>>()?)?,
        boolean(rows.iter().map(|(_, _, s)| s.repeated)),
        text(rows.iter().map(|(_, _, s)| s.identity_column)),
        text(rows.iter().map(|(_, _, s)| s.entity_kind)),
        text(rows.iter().map(|(_, _, s)| s.name_column)),
        text(rows.iter().map(|(_, _, s)| s.naming_scope_column)),
        text(rows.iter().map(|(_, _, s)| s.expression_owner_column)),
        text(rows.iter().map(|(_, _, s)| {
            s.expression_owner_kind
                .map(super::super::model::document::ExpressionOwnerKind::as_str)
        })),
        text(rows.iter().map(|(_, _, s)| Some(s.doc))),
        list(
            rows.iter().map(|(_, _, s)| s.expression_fields.len()),
            structure(vec![
                (
                    "path",
                    text(
                        rows.iter().flat_map(|(_, _, s)| {
                            s.expression_fields.iter().map(|(p, _)| Some(*p))
                        }),
                    ),
                ),
                (
                    "syntax",
                    text(rows.iter().flat_map(|(_, _, s)| {
                        s.expression_fields.iter().map(|(_, v)| Some(v.as_str()))
                    })),
                ),
            ])?,
        )?,
    ])
}

fn functions() -> Vec<ArrayRef> {
    let rows = pse_quantity::functions::Function::all();
    vec![
        text(rows.iter().map(|f| Some(f.as_str()))),
        text(rows.iter().map(|f| Some(f.implementation().as_str()))),
    ]
}

#[cfg(test)]
mod consolidation_unit {
    use super::*;
    #[test]
    fn empty_and_custom_registries_have_native_intrinsic_contracts() -> Result<(), SchemaError> {
        let empty = RegistryBuilder::new().build()?;
        let again = RegistryBuilder::new().build()?;
        assert_eq!(empty.fingerprint(), again.fingerprint());
        assert_eq!(empty.schema_batches(), again.schema_batches());
        assert_eq!(
            empty.schema_batches().len(),
            SELF_DESCRIBING_RELATIONS.len()
        );
        for (_, batch) in empty.schema_batches() {
            for array in batch.columns() {
                array.to_data().validate_full().map_err(invalid)?;
            }
        }
        let schema_columns = &empty
            .schema_batches()
            .iter()
            .find(|(k, _)| k.name == "schema_columns")
            .ok_or_else(|| invalid("missing columns"))?
            .1;
        assert_eq!(schema_columns.num_rows(), 0);
        assert!(schema_columns.num_columns() > 0);
        let platform = crate::catalog::assemble()?;
        let columns = &platform
            .schema_batches()
            .iter()
            .find(|(k, _)| k.name == "schema_columns")
            .ok_or_else(|| invalid("missing platform columns"))?
            .1;
        assert!(columns.num_rows() > 0);
        assert_ne!(empty.fingerprint(), platform.fingerprint());
        let clone = columns.clone();
        assert!(Arc::ptr_eq(clone.column(0), columns.column(0)));
        Ok(())
    }
}
