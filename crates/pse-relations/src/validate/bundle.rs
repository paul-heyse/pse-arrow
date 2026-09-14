// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use crate::RelationError;
use arrow_array::RecordBatch;
use pse_schema::Registry;
use pse_schema::model::{Cell, ExtensionUse, LogicalType, RelationKey, RelationSpec};
use std::collections::{BTreeMap, BTreeSet};

/// Admits each batch and then validates primary keys, declared foreign keys and nested
/// ordinal targets by comparing actual values. The map is an explicitly complete
/// reference context for its included relations; absent referenced targets are errors.
/// Stage membership and domain invariants remain the publication/P2 caller's duty.
///
/// # Errors
/// All independently found batch, duplicate-key, missing-reference and ordinal errors.
pub fn validate_bundle(
    reg: &Registry,
    batches: &BTreeMap<RelationKey, RecordBatch>,
) -> Result<(), Vec<RelationError>> {
    let mut errors = Vec::new();
    let mut decoded = BTreeMap::new();
    for (key, batch) in batches {
        let Some(spec) = reg.relations().iter().find(|spec| spec.key == *key) else {
            errors.push(RelationError::UnknownRegistry {
                relation: key.to_string(),
            });
            continue;
        };
        match crate::cells::cells_from_batch(reg, spec, batch) {
            Ok(rows) => {
                decoded.insert(*key, rows);
            }
            Err(RelationError::Validation { errors: found }) => errors.extend(found),
            Err(error) => errors.push(error),
        }
    }
    if !errors.is_empty() {
        return Err(errors);
    }
    for (key, rows) in &decoded {
        let Some(spec) = reg.relations().iter().find(|spec| spec.key == *key) else {
            continue;
        };
        keys(spec, rows, &mut errors);
        references(reg, spec, rows, &decoded, &mut errors);
        for (row, values) in rows.iter().enumerate() {
            for (column, value) in spec.columns.iter().zip(values) {
                ordinal(
                    reg,
                    &column.logical_type,
                    value,
                    column.name,
                    row,
                    &decoded,
                    &mut errors,
                );
            }
        }
    }
    super::finish(errors)
}

fn keys(spec: &RelationSpec, rows: &[Vec<Cell>], errors: &mut Vec<RelationError>) {
    let indices = spec
        .primary_key
        .iter()
        .filter_map(|name| spec.columns.iter().position(|column| column.name == *name))
        .collect::<Vec<_>>();
    let mut seen = BTreeSet::new();
    for (row, values) in rows.iter().enumerate() {
        // Tagged, lossless literal encodings compare actual values without a digest.
        let key = indices
            .iter()
            .map(|index| values[*index].literal_spec())
            .collect::<Vec<_>>();
        if !seen.insert(key) {
            errors.push(crate::cells::value_error(
                &spec.key.to_string(),
                row,
                "duplicate declared primary key",
            ));
        }
    }
}
fn references(
    reg: &Registry,
    spec: &RelationSpec,
    rows: &[Vec<Cell>],
    all: &BTreeMap<RelationKey, Vec<Vec<Cell>>>,
    errors: &mut Vec<RelationError>,
) {
    for (index, column) in spec.columns.iter().enumerate() {
        let Some(fk) = column.fk else {
            continue;
        };
        let target = reg.relation(fk.relation);
        let context = target.and_then(|target| {
            all.get(&target.key).zip(
                target
                    .columns
                    .iter()
                    .position(|candidate| candidate.name == fk.column),
            )
        });
        let Some((target_rows, target_index)) = context else {
            errors.push(super::mismatch(
                &spec.key.to_string(),
                format!("foreign key target {} is absent", fk.relation),
            ));
            continue;
        };
        let members = target_rows
            .iter()
            .map(|row| row[target_index].literal_spec())
            .collect::<BTreeSet<_>>();
        for (row, values) in rows.iter().enumerate() {
            if !matches!(values[index], Cell::Null)
                && !members.contains(&values[index].literal_spec())
            {
                errors.push(crate::cells::value_error(
                    column.name,
                    row,
                    &format!("foreign key value is absent from {fk}"),
                ));
            }
        }
    }
}
fn ordinal(
    reg: &Registry,
    ty: &LogicalType,
    value: &Cell,
    path: &str,
    row: usize,
    all: &BTreeMap<RelationKey, Vec<Vec<Cell>>>,
    errors: &mut Vec<RelationError>,
) {
    match (ty, value) {
        (LogicalType::Ext(ExtensionUse::OrdinalRef { target }), Cell::U64(ordinal)) => {
            let count = reg
                .relation(target)
                .and_then(|spec| all.get(&spec.key))
                .and_then(|rows| u64::try_from(rows.len()).ok());
            match count {
                Some(rows) if *ordinal < rows => {}
                Some(rows) => errors.push(RelationError::OrdinalRange {
                    field: format!("{path} row {row}"),
                    ordinal: *ordinal,
                    target: (*target).to_owned(),
                    rows,
                }),
                None => errors.push(super::mismatch(
                    path,
                    format!("ordinal target {target} is absent"),
                )),
            }
        }
        (LogicalType::List(child) | LogicalType::FixedList(child, _), Cell::List(values)) => {
            for (index, value) in values.iter().enumerate() {
                ordinal(
                    reg,
                    child,
                    value,
                    &format!("{path}[{index}]"),
                    row,
                    all,
                    errors,
                );
            }
        }
        (LogicalType::Struct(children), Cell::Struct(values)) => {
            for ((name, child, _), value) in children.iter().zip(values) {
                ordinal(
                    reg,
                    child,
                    value,
                    &format!("{path}.{name}"),
                    row,
                    all,
                    errors,
                );
            }
        }
        _ => {}
    }
}
