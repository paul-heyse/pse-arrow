// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Deterministic concrete data construction; expected keys never evaluate `RulePlan`.
#[path = "generate/semantic.rs"]
mod semantic;
use super::Fixture;
use pse_ids::{ContentHash, SemanticId};
use pse_schema::{
    Registry,
    model::{Cell, ExtensionUse, FieldContract, InvariantSpec, RelationSpec},
};
use std::collections::BTreeMap;
pub(super) type Row = BTreeMap<String, Cell>;
pub(super) type Rows = BTreeMap<String, Vec<Row>>;

pub(crate) fn pair(registry: &Registry, invariant: &InvariantSpec) -> (Fixture, Fixture) {
    let spec = registry.relation(&invariant.relation).unwrap();
    let rule = registry.rule(&invariant.rule).unwrap();
    let mut valid: Rows = rule
        .plan
        .dependencies()
        .into_iter()
        .map(|(relation, _, _)| (relation.to_owned(), vec![]))
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
            spec.primary_key
                .iter()
                .map(|column| row[*column].literal_spec())
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
                                .map(|(name, value)| (name, value.literal_spec()))
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
pub(super) fn set(rows: &mut Rows, relation: &str, column: &str, value: Cell) {
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
pub(super) fn id(value: u8) -> Cell {
    Cell::Id(SemanticId::from_bytes([value; 16]))
}
fn default_value(registry: &Registry, ty: &FieldContract, value: u8) -> Cell {
    use datafusion::arrow::datatypes::DataType;
    if let Some(extension) = ty.extension() {
        return match extension {
            ExtensionUse::SemanticId => id(value),
            ExtensionUse::ContentHash => Cell::Hash(ContentHash::from_bytes([value; 32])),
            ExtensionUse::Enum(name) => {
                Cell::Enum(registry.enum_spec(name).unwrap().members[0].name)
            }
            ExtensionUse::OrdinalRef { .. } => Cell::U64(u64::from(value)),
            ExtensionUse::ExprDsl => Cell::text("1"),
            ExtensionUse::TargetPath => Cell::text("/fixture.symbol"),
            ExtensionUse::DimensionVector => {
                Cell::List(vec![Cell::Struct(vec![Cell::I64(0), Cell::I64(1)]); 8])
            }
            ExtensionUse::IndexTuple => Cell::List(vec![id(value)]),
            ExtensionUse::QuantityValue => Cell::Struct(vec![Cell::F64(1.0), id(value), id(value)]),
            ExtensionUse::Bound => Cell::Struct(vec![Cell::Enum("unbounded"), Cell::Null]),
            ExtensionUse::SourceSpan => Cell::Struct(vec![id(value), Cell::U64(0), Cell::U64(1)]),
        };
    }
    match ty.data_type() {
        DataType::Boolean => Cell::Bool(value != 0),
        DataType::Float64 => Cell::F64(f64::from(value)),
        DataType::Int64 | DataType::Int32 | DataType::Timestamp(..) => Cell::I64(i64::from(value)),
        DataType::UInt8 | DataType::UInt16 | DataType::UInt32 | DataType::UInt64 => {
            Cell::U64(u64::from(value))
        }
        DataType::Utf8 => Cell::text(format!("value-{value}")),
        DataType::List(_) => Cell::List(vec![]),
        DataType::FixedSizeList(child, width) => Cell::List(
            (0..width)
                .map(|_| {
                    default_value(
                        registry,
                        &FieldContract::from_field((*child).clone()),
                        value,
                    )
                })
                .collect(),
        ),
        DataType::Struct(children) => Cell::Struct(
            children
                .iter()
                .map(|child| {
                    default_value(
                        registry,
                        &FieldContract::from_field((**child).clone()),
                        value,
                    )
                })
                .collect(),
        ),
        other => panic!("fixture has no default for {other}"),
    }
}
