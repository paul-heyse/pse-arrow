// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Deterministic concrete data construction; expected keys are independent of the invariant queries.
#[path = "generate/inference.rs"]
mod inference;
#[path = "generate/semantic.rs"]
mod semantic;
use super::Fixture;
use pse_ids::{ContentHash, SemanticId};
use pse_schema::{
    Registry,
    model::{ExtensionUse, FieldContract, InvariantSpec, RelationSpec},
};
use std::collections::BTreeMap;
pub(super) type Row = BTreeMap<String, serde_json::Value>;
pub(super) type Rows = BTreeMap<String, Vec<Row>>;

pub(crate) fn pair(registry: &Registry, invariant: &InvariantSpec) -> (Fixture, Fixture) {
    let spec = registry.relation(&invariant.relation).unwrap();
    let mut valid: Rows = invariant
        .inputs
        .iter()
        .map(|relation| (relation.clone(), vec![]))
        .collect();
    let subject = row(registry, spec, 1);
    valid.insert(invariant.relation.clone(), vec![subject.clone()]);
    let mut invalid = valid.clone();
    if invariant.name == "unique:pk" {
        invalid.get_mut(&invariant.relation).unwrap().push(subject);
    } else if let Some(column) = invariant.name.strip_prefix("foreign_key:") {
        let fk = spec.column(column).unwrap().fk().unwrap();
        let target = registry.relation(fk.relation).unwrap();
        let mut target_row = row(registry, target, 1);
        let value = default_value(registry, &spec.column(column).unwrap().value_type(), 1);
        target_row.insert(fk.column.to_owned(), value.clone());
        valid.get_mut(&invariant.relation).unwrap()[0].insert(column.to_owned(), value);
        if fk.relation == invariant.relation {
            let value = valid[&invariant.relation][0][fk.column].clone();
            valid.get_mut(&invariant.relation).unwrap()[0].insert(column.to_owned(), value);
        } else {
            valid.insert(fk.relation.to_owned(), vec![target_row]);
        }
        invalid = valid.clone();
        invalid.get_mut(&invariant.relation).unwrap()[0].insert(
            column.to_owned(),
            default_value(registry, &spec.column(column).unwrap().value_type(), 2),
        );
    } else {
        semantic::populate(registry, invariant, &mut valid, &mut invalid);
    }
    let keys = invalid[&invariant.relation]
        .iter()
        .take(if invariant.name == "cardinality:member_ordinal" {
            usize::MAX
        } else {
            1
        })
        .map(|row| {
            invariant
                .key_columns
                .iter()
                .map(|column| row[*column].to_string())
                .collect()
        })
        .collect();
    let convert = |rows: Rows, expected_keys| Fixture {
        invariant: invariant.qualified_name(),
        rows: rows
            .into_iter()
            .map(|(name, rows)| {
                (
                    name,
                    rows.into_iter()
                        .map(|row| {
                            row.into_iter()
                                .map(|(name, value)| (name, value.to_string()))
                                .collect()
                        })
                        .collect(),
                )
            })
            .collect(),
        expected_keys,
    };
    (convert(valid, vec![]), convert(invalid, keys))
}
pub(super) fn row(registry: &Registry, spec: &RelationSpec, identity: u8) -> Row {
    spec.columns
        .iter()
        .map(|column| {
            (
                column.name().to_owned(),
                default_value(registry, &column.value_type(), identity),
            )
        })
        .collect()
}
pub(super) fn set(rows: &mut Rows, relation: &str, column: &str, value: serde_json::Value) {
    let row = &mut rows.get_mut(relation).expect("fixture relation")[0];
    assert!(
        row.contains_key(column),
        "declared fixture column: {relation}.{column}"
    );
    row.insert(column.to_owned(), value);
}
pub(super) fn put(registry: &Registry, rows: &mut Rows, relation: &str, identity: u8) {
    rows.insert(
        relation.to_owned(),
        vec![row(
            registry,
            registry.relation(relation).unwrap(),
            identity,
        )],
    );
}
pub(super) fn id(value: u8) -> serde_json::Value {
    serde_json::json!(["id", (SemanticId::from_bytes([value; 16])).to_hex()])
}
#[allow(
    clippy::too_many_lines,
    reason = "one exhaustive native fixture literal dispatch"
)]
fn default_value(registry: &Registry, ty: &FieldContract, value: u8) -> serde_json::Value {
    use datafusion::arrow::datatypes::DataType;
    if let Some(alternative) = pse_schema::model::TaggedAlternative::from_field(ty.field()).unwrap()
    {
        let (tag, selected) = alternative.arms.first_key_value().unwrap();
        let DataType::Struct(fields) = ty.data_type() else {
            panic!("a tagged fixture requires its declared struct storage")
        };
        return serde_json::json!([
            "struct",
            fields
                .iter()
                .map(|field| {
                    if field.name() == &alternative.discriminator {
                        let contract = FieldContract::from_field((**field).clone());
                        match contract.extension() {
                            Some(ExtensionUse::Enum(name)) => {
                                assert!(
                                    registry
                                        .enum_spec(name)
                                        .unwrap()
                                        .members
                                        .iter()
                                        .any(|member| member.name == *tag)
                                );
                                serde_json::json!(["enum", tag])
                            }
                            None if matches!(field.data_type(), DataType::Utf8) => {
                                serde_json::json!(["text", tag])
                            }
                            _ => {
                                panic!("tag discriminator must have a declared enum or text field")
                            }
                        }
                    } else if Some(field.name()) == selected.as_ref() {
                        default_value(
                            registry,
                            &FieldContract::from_field((**field).clone()),
                            value,
                        )
                    } else {
                        serde_json::json!(["null", null])
                    }
                })
                .collect::<Vec<serde_json::Value>>()
        ]);
    }
    if let Some(extension) = ty.extension() {
        return match extension {
            ExtensionUse::SemanticId => id(value),
            ExtensionUse::ContentHash => {
                serde_json::json!(["hash", (ContentHash::from_bytes([value; 32])).to_hex()])
            }
            ExtensionUse::Enum(name) => {
                serde_json::json!(["enum", registry.enum_spec(name).unwrap().members[0].name])
            }
            ExtensionUse::OrdinalRef { .. } => serde_json::json!(["i64", i64::from(value)]),
            ExtensionUse::ExprDsl => serde_json::json!(["text", "1"]),
            ExtensionUse::TargetPath => serde_json::json!(["text", "/fixture.symbol"]),
            ExtensionUse::DimensionVector => {
                serde_json::json!([
                    "list",
                    vec![
                        serde_json::json!([
                            "struct",
                            vec![serde_json::json!(["i64", 0]), serde_json::json!(["i64", 1])]
                        ]);
                        8
                    ]
                ])
            }
            ExtensionUse::IndexTuple => serde_json::json!(["list", vec![id(value)]]),
            ExtensionUse::QuantityValue => serde_json::json!([
                "struct",
                vec![
                    serde_json::json!(["f64", format!("{:016x}", f64::to_bits(1.0))]),
                    id(value),
                    id(value)
                ]
            ]),
            ExtensionUse::Bound => serde_json::json!([
                "struct",
                vec![
                    serde_json::json!(["enum", "unbounded"]),
                    serde_json::json!(["null", null])
                ]
            ]),
            ExtensionUse::SourceSpan => serde_json::json!([
                "struct",
                vec![
                    id(value),
                    serde_json::json!(["i64", 0]),
                    serde_json::json!(["i64", 1])
                ]
            ]),
        };
    }
    match ty.data_type() {
        DataType::Boolean => serde_json::json!(["bool", value != 0]),
        DataType::Float64 => {
            serde_json::json!(["f64", format!("{:016x}", (f64::from(value)).to_bits())])
        }
        DataType::Int64 | DataType::Int32 | DataType::Timestamp(..) => {
            serde_json::json!(["i64", i64::from(value)])
        }
        DataType::UInt8 | DataType::UInt16 | DataType::UInt32 | DataType::UInt64 => {
            serde_json::json!(["u64", u64::from(value)])
        }
        DataType::Utf8 => serde_json::json!(["text", format!("value-{value}")]),
        DataType::List(_) => serde_json::json!(["list", []]),
        DataType::FixedSizeList(child, width) => serde_json::json!([
            "list",
            (0..width)
                .map(|_| {
                    default_value(
                        registry,
                        &FieldContract::from_field((*child).clone()),
                        value,
                    )
                })
                .collect::<Vec<serde_json::Value>>()
        ]),
        DataType::Struct(children) => serde_json::json!([
            "struct",
            children
                .iter()
                .map(|child| {
                    default_value(
                        registry,
                        &FieldContract::from_field((**child).clone()),
                        value,
                    )
                })
                .collect::<Vec<serde_json::Value>>()
        ]),
        other => panic!("fixture has no default for {other}"),
    }
}

#[cfg(test)]
mod unit {
    #[test]
    fn declared_fixture_literals_decode_with_semantic_fields() {
        let registry = pse_schema::catalog::assemble().unwrap();
        for invariant in registry.invariants() {
            let (valid, violating) = super::pair(&registry, invariant);
            // Arrow construction only: no planning, storage, solver or query execution.
            for fixture in [valid, violating] {
                let batches = fixture.batches(&registry);
                assert_eq!(batches.len(), invariant.inputs.len());
            }
        }
    }
}
