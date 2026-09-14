// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use crate::RelationError;
use arrow_schema::{DataType, Field};
use pse_schema::model::{Cell, QuantityContract, RelationSpec};

pub(super) fn validate_cell(
    field: &Field,
    cell: &Cell,
    row: usize,
    path: &str,
    errors: &mut Vec<RelationError>,
) {
    if matches!(cell, Cell::Null) {
        if !field.is_nullable() {
            errors.push(crate::cells::value_error(
                path,
                row,
                "required visible value is null",
            ));
        }
        return;
    }
    match (field.data_type(), cell) {
        (DataType::Struct(children), Cell::Struct(values)) => {
            for (child, value) in children.iter().zip(values) {
                validate_cell(
                    child,
                    value,
                    row,
                    &format!("{path}.{}", child.name()),
                    errors,
                );
            }
        }
        (DataType::List(child) | DataType::FixedSizeList(child, _), Cell::List(values)) => {
            for (index, value) in values.iter().enumerate() {
                validate_cell(child, value, row, &format!("{path}[{index}]"), errors);
            }
        }
        _ => {}
    }
    let Some(extension) = field.metadata().get(pse_schema::arrow::KEY_EXTENSION_NAME) else {
        return;
    };
    let reason = match extension.as_str() {
        "pse.bound" => bound(cell),
        "pse.dimension_vector" => dimension(cell),
        "pse.source_span" => span(cell),
        "pse.quantity_value" => quantity(cell),
        // Text syntax and target resolution belong to P1/P3 and require their declared
        // grammar and identity context; field admission establishes their Utf8 storage.
        _ => None,
    };
    if let Some(reason) = reason {
        errors.push(crate::cells::value_error(path, row, reason));
    }
}

fn bound(cell: &Cell) -> Option<&'static str> {
    let Cell::Struct(values) = cell else {
        return Some("bound requires its declared struct");
    };
    match values.as_slice() {
        [Cell::Enum("finite"), Cell::F64(value)] if value.is_finite() => None,
        [Cell::Enum("unbounded"), Cell::Null] => None,
        _ => Some("finite bound requires a finite value; unbounded requires an absent value"),
    }
}
fn dimension(cell: &Cell) -> Option<&'static str> {
    let Cell::List(values) = cell else {
        return Some("dimension requires eight rational exponents");
    };
    for value in values {
        let Cell::Struct(pair) = value else {
            return Some("dimension exponent requires numerator and denominator");
        };
        if let [Cell::I64(num), Cell::I64(den)] = pair.as_slice() {
            if *den <= 0 || gcd(num.unsigned_abs(), den.unsigned_abs()) != 1 {
                return Some("dimension exponent must be reduced with a positive denominator");
            }
        } else {
            return Some("dimension exponent requires numerator and denominator");
        }
    }
    None
}
fn gcd(mut one: u64, mut two: u64) -> u64 {
    while two != 0 {
        let rest = one % two;
        one = two;
        two = rest;
    }
    one
}
fn span(cell: &Cell) -> Option<&'static str> {
    let Cell::Struct(values) = cell else {
        return Some("source span requires its declared struct");
    };
    match values.as_slice() {
        [Cell::Id(_), Cell::U64(start), Cell::U64(end)] if start <= end => None,
        _ => Some("source span start exceeds end"),
    }
}
fn quantity(cell: &Cell) -> Option<&'static str> {
    let Cell::Struct(values) = cell else {
        return Some("quantity value requires its declared struct");
    };
    match values.as_slice() {
        [Cell::F64(value), Cell::Id(_), Cell::Id(_)] if value.is_finite() => None,
        _ => {
            Some("quantity value requires a finite value and explicit quantity and unit identities")
        }
    }
}

pub(super) fn validate_quantities(
    spec: &RelationSpec,
    values: &[Cell],
    row: usize,
    errors: &mut Vec<RelationError>,
) {
    for (index, column) in spec.columns.iter().enumerate() {
        if column.quantity != QuantityContract::PerRow
            || matches!(values.get(index), None | Some(Cell::Null))
        {
            continue;
        }
        let sibling = column.per_row_quantity_sibling();
        let value = spec
            .columns
            .iter()
            .position(|candidate| candidate.name == sibling)
            .and_then(|index| values.get(index));
        if !matches!(value, Some(Cell::Id(_))) {
            errors.push(crate::cells::value_error(
                column.name,
                row,
                "visible per-row quantity requires a non-null sibling quantity identity",
            ));
        }
    }
}
