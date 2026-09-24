// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Durable semantic row tokens over explicit fields and direct native values.
use crate::{CanonError, FramedHasher, SemanticId};
use arrow_array::{Array, ArrayRef, FixedSizeBinaryArray, builder::FixedSizeBinaryBuilder};
use arrow_schema::FieldRef;
/// Versioned value framing, independent of Arrow's private row sorting representation.
pub const ENCODING: &str = "pse:row-key:native-values:v2";
/// Frame ordered declared key names/domains once; hash actual native values per row.
/// Empty tuples still have the caller's explicit cardinality and relation scope.
/// # Errors
/// Invalid native arrays, mismatched lengths/fields or unsupported value representation.
pub fn tokens(
    scope: SemanticId,
    columns: &[(&str, FieldRef, ArrayRef)],
    rows: usize,
) -> Result<FixedSizeBinaryArray, CanonError> {
    let mut prefix = FramedHasher::new(ENCODING);
    prefix
        .id(&scope)
        .u64(u64::try_from(columns.len()).map_err(|_| invalid("column count overflow"))?);
    for (name, field, values) in columns {
        if values.len() != rows || values.data_type() != field.data_type() {
            return Err(invalid("row token input shape differs"));
        }
        values.to_data().validate_full()?;
        crate::native_value::admit_type(field.data_type())?;
        let meaning = crate::native_field::project(
            field,
            crate::native_field::MetadataPurpose::ValueIdentity,
        )?;
        prefix.str(name).str(
            &crate::native_field::canonical_json(&meaning)
                .map_err(|error| invalid(error.to_string()))?,
        );
    }
    let mut output = FixedSizeBinaryBuilder::with_capacity(rows, 32);
    for row in 0..rows {
        let mut hash = prefix.clone();
        for (_, field, values) in columns {
            hash.hash(&crate::native_value::semantic_payload(
                values.as_ref(),
                field,
                row,
            )?);
        }
        output.append_value(hash.finish_hash().as_bytes())?;
    }
    Ok(output.finish())
}
fn invalid(reason: impl Into<String>) -> CanonError {
    CanonError::Internal(reason.into())
}

#[cfg(test)]
mod consolidation_unit;
