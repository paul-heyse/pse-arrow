// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Traverse native Arrow containers while reusing the declared PSE value predicates.

use crate::CatalogError;
use datafusion::arrow::{
    array::{
        Array, ArrayRef,
        cast::AsArray,
        types::{Int16Type, Int32Type, Int64Type},
    },
    datatypes::{DataType, Field},
};
use pse_ids::CancellationToken;
use pse_schema::Registry;
use std::sync::Arc;

pub(super) fn validate(
    registry: &Registry,
    field: &Field,
    array: &dyn Array,
    cancel: &CancellationToken,
) -> Result<(), CatalogError> {
    cancel.checkpoint()?;
    if field.data_type() != array.data_type() {
        return Err(invalid("field and array type differ"));
    }
    if !field.is_nullable() && array.logical_null_count() != 0 {
        return Err(invalid("nonnull field contains logical nulls"));
    }
    if field
        .metadata()
        .get(pse_schema::arrow::KEY_EXTENSION_NAME)
        .is_some_and(|name| name.starts_with("pse."))
        || field
            .metadata()
            .contains_key(pse_schema::model::integer_range::KEY_INTEGER_RANGE)
    {
        return pse_relations::validate::validate_column(registry, field, array)
            .map_err(|errors| pse_relations::RelationError::Validation { errors }.into());
    }
    match field.data_type() {
        DataType::Struct(fields) => {
            let structure = array
                .as_struct_opt()
                .ok_or_else(|| invalid("struct storage differs"))?;
            if structure.null_count() == 0 {
                for (child, values) in fields.iter().zip(structure.columns()) {
                    validate(registry, child, values.as_ref(), cancel)?;
                }
            } else {
                for row in 0..array.len() {
                    cancel.checkpoint()?;
                    if structure.is_null(row) {
                        continue;
                    }
                    for (child, values) in fields.iter().zip(structure.columns()) {
                        validate(registry, child, values.slice(row, 1).as_ref(), cancel)?;
                    }
                }
            }
        }
        DataType::List(child)
        | DataType::LargeList(child)
        | DataType::ListView(child)
        | DataType::LargeListView(child)
        | DataType::FixedSizeList(child, _)
        | DataType::Map(child, _) => {
            for row in 0..array.len() {
                cancel.checkpoint()?;
                if array.is_null(row) {
                    continue;
                }
                let values = list_value(array, row)?;
                validate(registry, child, values.as_ref(), cancel)?;
            }
        }
        DataType::Dictionary(_, value_type) => {
            validate_dictionary(registry, field, array, value_type, cancel)?;
        }
        DataType::Union(fields, _) => {
            let union = array
                .as_union_opt()
                .ok_or_else(|| invalid("union storage differs"))?;
            let nulls = array.logical_nulls();
            for row in 0..array.len() {
                cancel.checkpoint()?;
                if nulls.as_ref().is_some_and(|nulls| nulls.is_null(row)) {
                    continue;
                }
                let child = fields
                    .iter()
                    .find_map(|(id, field)| (id == union.type_id(row)).then_some(field))
                    .ok_or_else(|| invalid("union type ID has no field"))?;
                validate(registry, child, union.value(row).as_ref(), cancel)?;
            }
        }
        DataType::RunEndEncoded(_, child) => {
            for row in 0..array.len() {
                cancel.checkpoint()?;
                let values = run_value(array, row)?;
                validate(registry, child, values.as_ref(), cancel)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn validate_dictionary(
    registry: &Registry,
    field: &Field,
    array: &dyn Array,
    value_type: &DataType,
    cancel: &CancellationToken,
) -> Result<(), CatalogError> {
    let dictionary = array
        .as_any_dictionary_opt()
        .ok_or_else(|| invalid("dictionary storage differs"))?;
    for row in 0..array.len() {
        cancel.checkpoint()?;
        if dictionary.keys().is_null(row) {
            continue;
        }
        // Only a visible key reaches normalized_keys: validated Arrow storage
        // guarantees a nonempty values array and an in-bounds key here.
        let one = array.slice(row, 1);
        let one = one
            .as_any_dictionary_opt()
            .ok_or_else(|| invalid("dictionary slice differs"))?;
        let key = one
            .normalized_keys()
            .into_iter()
            .next()
            .ok_or_else(|| invalid("visible dictionary key absent"))?;
        let child = Field::new("dictionary_value", value_type.clone(), field.is_nullable());
        validate(
            registry,
            &child,
            dictionary.values().slice(key, 1).as_ref(),
            cancel,
        )?;
    }
    Ok(())
}

fn list_value(array: &dyn Array, row: usize) -> Result<ArrayRef, CatalogError> {
    let value = match array.data_type() {
        DataType::List(_) => array.as_list_opt::<i32>().map(|a| a.value(row)),
        DataType::LargeList(_) => array.as_list_opt::<i64>().map(|a| a.value(row)),
        DataType::ListView(_) => array.as_list_view_opt::<i32>().map(|a| a.value(row)),
        DataType::LargeListView(_) => array.as_list_view_opt::<i64>().map(|a| a.value(row)),
        DataType::FixedSizeList(_, _) => array.as_fixed_size_list_opt().map(|a| a.value(row)),
        DataType::Map(_, _) => array.as_map_opt().map(|a| {
            let values: ArrayRef = Arc::new(a.value(row));
            values
        }),
        _ => None,
    };
    value.ok_or_else(|| invalid("list/map storage differs from its declared native type"))
}

fn run_value(array: &dyn Array, row: usize) -> Result<ArrayRef, CatalogError> {
    let DataType::RunEndEncoded(runs, _) = array.data_type() else {
        return Err(invalid("run storage differs"));
    };
    let value = match runs.data_type() {
        DataType::Int16 => array
            .as_run_opt::<Int16Type>()
            .map(|a| a.values().slice(a.get_physical_index(row), 1)),
        DataType::Int32 => array
            .as_run_opt::<Int32Type>()
            .map(|a| a.values().slice(a.get_physical_index(row), 1)),
        DataType::Int64 => array
            .as_run_opt::<Int64Type>()
            .map(|a| a.values().slice(a.get_physical_index(row), 1)),
        _ => None,
    };
    value.ok_or_else(|| invalid("run storage differs from the Arrow run-index contract"))
}

fn invalid(reason: &str) -> CatalogError {
    CatalogError::Admission {
        path: "provider.capture.values".into(),
        reason: reason.into(),
    }
}
