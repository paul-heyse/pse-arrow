// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Explicit physical primitive fixtures shared by conformance and golden stores.
use super::invalid;
use crate::CompilerError;
use pse_ids::SemanticId;
use pse_quantity::{BaseDimension, DimensionVector};
use pse_relations::RecordBatch;
use pse_schema::{
    Registry,
    model::{Cell, RelationKey},
};
use std::collections::BTreeMap;
fn sid(value: u8) -> SemanticId {
    SemanticId::from_bytes([value; 16])
}

/// Eight explicit SI base units and their declared unit set, without a neutral quantity.
/// # Errors
/// Missing fixture contract or malformed declared values.
pub fn unit_rows(registry: &Registry) -> Result<BTreeMap<RelationKey, RecordBatch>, CompilerError> {
    let spec = registry
        .relation("reference.units")
        .ok_or_else(|| invalid("fixture relation undeclared"))?;
    let rows = BaseDimension::ALL
        .iter()
        .map(|dimension| {
            let vector = DimensionVector::base(*dimension);
            vec![
                Cell::Id(sid(dimension.ordinal() + 1)),
                Cell::text(
                    ["m", "kg", "s", "K", "mol", "A", "cd", "USD_fixture"]
                        [usize::from(dimension.ordinal())],
                ),
                Cell::text(dimension.as_str()),
                dimension_cell(vector),
                Cell::F64(1.),
                Cell::F64(0.),
                Cell::Bool(false),
                Cell::Null,
                Cell::text("SI"),
                Cell::text("Fixture base"),
            ]
        })
        .collect::<Vec<_>>();
    let sets = registry
        .relation("reference.unit_sets")
        .ok_or_else(|| invalid("fixture relation undeclared"))?;
    let set = vec![
        Cell::Id(sid(9)),
        Cell::text("base"),
        Cell::Id(sid(3)),
        Cell::Id(sid(1)),
        Cell::Id(sid(2)),
        Cell::Id(sid(5)),
        Cell::Id(sid(4)),
        Cell::Id(sid(6)),
        Cell::Id(sid(7)),
        Cell::Id(sid(8)),
    ];
    Ok(BTreeMap::from([
        (
            spec.key,
            pse_relations::cells::batch_from_cells(registry, spec, &rows)?,
        ),
        (
            sets.key,
            pse_relations::cells::batch_from_cells(registry, sets, &[set])?,
        ),
    ]))
}

/// Actual minimal length and neutral quantity definitions, including all primitive ports.
/// # Errors
/// Missing fixture contract or malformed declared values.
pub fn physical_rows(
    registry: &Registry,
) -> Result<BTreeMap<RelationKey, RecordBatch>, CompilerError> {
    let mut rows = unit_rows(registry)?;
    for name in [
        "reference.reference_states",
        "reference.bases",
        "reference.quantity_kinds",
        "reference.quantity_types",
        "reference.conversion_rules",
        "reference.quantity_operations",
    ] {
        let spec = registry
            .relation(name)
            .ok_or_else(|| invalid("fixture relation undeclared"))?;
        rows.insert(
            spec.key,
            pse_relations::cells::batch_from_cells(registry, spec, &[])?,
        );
    }
    let kinds = registry
        .relation("reference.quantity_kinds")
        .ok_or_else(|| invalid("fixture relation undeclared"))?;
    rows.insert(
        kinds.key,
        pse_relations::cells::batch_from_cells(
            registry,
            kinds,
            &[
                vec![
                    Cell::Id(sid(20)),
                    Cell::text("length"),
                    dimension_cell(DimensionVector::base(BaseDimension::Length)),
                    Cell::Bool(false),
                    Cell::Enum("additive"),
                    Cell::text("length kind"),
                ],
                vec![
                    Cell::Id(sid(21)),
                    Cell::text("neutral"),
                    dimension_cell(DimensionVector::DIMENSIONLESS),
                    Cell::Bool(false),
                    Cell::Enum("additive"),
                    Cell::text("explicit neutral"),
                ],
            ],
        )?,
    );
    add_neutral_unit(registry, &mut rows)?;
    let types = registry
        .relation("reference.quantity_types")
        .ok_or_else(|| invalid("fixture relation undeclared"))?;
    rows.insert(
        types.key,
        pse_relations::cells::batch_from_cells(
            registry,
            types,
            &[
                vec![
                    Cell::Id(sid(30)),
                    Cell::Id(sid(20)),
                    Cell::Null,
                    Cell::Null,
                    Cell::Enum("point"),
                    Cell::List(vec![]),
                    Cell::Null,
                    Cell::Id(sid(1)),
                    Cell::Null,
                    Cell::text("length"),
                ],
                vec![
                    Cell::Id(sid(31)),
                    Cell::Id(sid(21)),
                    Cell::Null,
                    Cell::Null,
                    Cell::Enum("point"),
                    Cell::List(vec![]),
                    Cell::Null,
                    Cell::Id(sid(10)),
                    Cell::Null,
                    Cell::text("neutral"),
                ],
            ],
        )?,
    );
    Ok(rows)
}

fn dimension_cell(value: DimensionVector) -> Cell {
    Cell::List(
        value
            .exponents()
            .iter()
            .map(|ratio| {
                Cell::Struct(vec![
                    Cell::I64(i64::from(ratio.num())),
                    Cell::I64(i64::from(ratio.den())),
                ])
            })
            .collect(),
    )
}
fn add_neutral_unit(
    registry: &Registry,
    rows: &mut BTreeMap<RelationKey, RecordBatch>,
) -> Result<(), CompilerError> {
    let units = registry
        .relation("reference.units")
        .ok_or_else(|| invalid("fixture relation undeclared"))?;
    let mut values = pse_relations::cells::cells_from_batch(registry, units, &rows[&units.key])?;
    values.push(vec![
        Cell::Id(sid(10)),
        Cell::text("dimensionless"),
        Cell::text("one"),
        dimension_cell(DimensionVector::DIMENSIONLESS),
        Cell::F64(1.),
        Cell::F64(0.),
        Cell::Bool(false),
        Cell::Null,
        Cell::text("SI"),
        Cell::text("neutral storage"),
    ]);
    rows.insert(
        units.key,
        pse_relations::cells::batch_from_cells(registry, units, &values)?,
    );
    Ok(())
}
