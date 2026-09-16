// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use crate::RelationError;
use arrow_schema::{DataType, Field};
use pse_schema::model::Cell;

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
    if let Ok(Some(range)) = pse_schema::model::IntegerRange::from_field(field)
        && !matches!(cell, Cell::I64(value) if range.contains(*value))
    {
        errors.push(crate::cells::value_error(
            path,
            row,
            "value outside declared integer domain",
        ));
    }
    if let Ok(Some(alternative)) = pse_schema::model::TaggedAlternative::from_field(field)
        && !alternative.accepts(field, cell)
    {
        errors.push(crate::cells::value_error(
            path,
            row,
            "tagged value requires exactly its selected arm",
        ));
    }
    if let Ok(Some(contract)) = pse_schema::model::CollectionContract::from_field(field)
        && !matches!(cell, Cell::List(values) if contract.accepts(values))
    {
        errors.push(crate::cells::value_error(
            path,
            row,
            "collection violates cardinality or uniqueness",
        ));
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
        [Cell::Enum(kind), Cell::F64(value)] if super::local_values::bound(kind, Some(*value)) => {
            None
        }
        [Cell::Enum(kind), Cell::Null] if super::local_values::bound(kind, None) => None,
        _ => Some(super::local_values::BOUND_ERROR),
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
            if !super::local_values::dimension_exponent(*num, *den) {
                return Some(super::local_values::DIMENSION_ERROR);
            }
        } else {
            return Some("dimension exponent requires numerator and denominator");
        }
    }
    None
}
fn span(cell: &Cell) -> Option<&'static str> {
    let Cell::Struct(values) = cell else {
        return Some("source span requires its declared struct");
    };
    match values.as_slice() {
        [Cell::Id(_), Cell::I64(start), Cell::I64(end)]
            if super::local_values::source_span(*start, *end) =>
        {
            None
        }
        _ => Some(super::local_values::SPAN_ERROR),
    }
}
fn quantity(cell: &Cell) -> Option<&'static str> {
    let Cell::Struct(values) = cell else {
        return Some("quantity value requires its declared struct");
    };
    match values.as_slice() {
        [Cell::F64(value), Cell::Id(_), Cell::Id(_)] if super::local_values::quantity(*value) => {
            None
        }
        _ => Some(super::local_values::QUANTITY_ERROR),
    }
}
