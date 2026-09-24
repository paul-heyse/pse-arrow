// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Field-directed lossless literals. Construction checks representation, not domain validity.
//! Bulk data remains native arrays; scalar adapters are restricted to scalar API boundaries.

mod decode;
use crate::SchemaError;
use arrow_array::{Array, ArrayRef};
use arrow_schema::FieldRef;
use datafusion_common::ScalarValue;
use std::sync::Arc;
type Result<T> = std::result::Result<T, SchemaError>;

/// One Arrow value and its exact declared field, including all metadata.
#[derive(Clone, Debug)]
pub struct NativeLiteral {
    field: FieldRef,
    array: ArrayRef,
    json: Arc<str>,
}
impl NativeLiteral {
    /// Check that this is one structurally safe value with the exact physical datatype.
    /// Semantic/default admission belongs to the prepared relation contract.
    /// # Errors
    /// Wrong length, malformed buffers, unsupported literal storage or type mismatch.
    pub fn new(field: FieldRef, array: ArrayRef) -> Result<Self> {
        if array.len() != 1 || array.data_type() != field.data_type() {
            return Err(invalid(
                "literal requires one value of its declared native datatype",
            ));
        }
        array
            .to_data()
            .validate_full()
            .map_err(|error| invalid(error.to_string()))?;
        pse_columnar::native_value::admit_type(field.data_type())
            .map_err(|error| invalid(error.to_string()))?;
        let json = to_json(array.as_ref(), &field, 0)?.into();
        Ok(Self { field, array, json })
    }
    /// Exact declared field.
    pub fn field(&self) -> &FieldRef {
        &self.field
    }
    /// Shared one-element native array.
    pub fn array(&self) -> &ArrayRef {
        &self.array
    }
    /// Cached lossless rendering derived when this immutable literal is constructed.
    pub fn as_json(&self) -> &str {
        &self.json
    }
    /// Decode exactly one tagged value according to the supplied field.
    /// # Errors
    /// Malformed tags, incompatible shape, integer overflow or invalid buffers.
    pub fn from_json(field: FieldRef, text: &str) -> Result<Self> {
        admit_type(field.data_type())?;
        let value = serde_json::from_str(text).map_err(|error| invalid(error.to_string()))?;
        let array = decode::value(&field, &value)?;
        Self::new(field, array)
    }
    /// Resolve this declaration's domain metadata into the registry's execution field.
    /// This scalar boundary keeps the exact value bits and does not establish validity.
    /// # Errors
    /// A missing domain/reference or incompatible native representation.
    pub fn bind(&self, registry: &crate::Registry) -> Result<Self> {
        let contract = crate::model::FieldContract::from_field(self.field.as_ref().clone());
        let field = Arc::new(crate::arrow::field_for(registry, &contract)?);
        Self::from_json(field, self.as_json())
    }
    /// Adapt a native scalar at a scalar API boundary.
    /// # Errors
    /// Representation mismatch or unsupported scalar storage.
    pub fn from_scalar(field: FieldRef, scalar: &ScalarValue) -> Result<Self> {
        Self::new(
            field,
            scalar
                .to_array()
                .map_err(|error| invalid(error.to_string()))?,
        )
    }
    /// Adapt this one-value literal to a native scalar expression.
    /// # Errors
    /// Unsupported scalar storage.
    pub fn scalar(&self) -> Result<ScalarValue> {
        scalar_from_array(&self.array)
    }
}
/// Convert exactly one Arrow value without rebuilding nested child fields.
/// This is a scalar API adapter, with no JSON encoding or domain validation.
/// # Errors
/// The array does not contain exactly one value or has unsupported scalar storage.
pub fn scalar_from_array(array: &ArrayRef) -> Result<ScalarValue> {
    if array.len() != 1 {
        return Err(invalid("scalar requires exactly one value"));
    }
    macro_rules! native {
        ($ty:ty, $variant:ident) => {
            ScalarValue::$variant(Arc::new(
                array
                    .as_any()
                    .downcast_ref::<$ty>()
                    .ok_or_else(|| invalid("scalar storage mismatch"))?
                    .clone(),
            ))
        };
    }
    Ok(match array.data_type() {
        arrow_schema::DataType::List(_) => native!(arrow_array::ListArray, List),
        arrow_schema::DataType::LargeList(_) => native!(arrow_array::LargeListArray, LargeList),
        arrow_schema::DataType::FixedSizeList(..) => {
            native!(arrow_array::FixedSizeListArray, FixedSizeList)
        }
        arrow_schema::DataType::ListView(_) => native!(arrow_array::ListViewArray, ListView),
        arrow_schema::DataType::LargeListView(_) => {
            native!(arrow_array::LargeListViewArray, LargeListView)
        }
        arrow_schema::DataType::Struct(_) => native!(arrow_array::StructArray, Struct),
        arrow_schema::DataType::Map(..) => native!(arrow_array::MapArray, Map),
        _ => ScalarValue::try_from_array(array, 0).map_err(|error| invalid(error.to_string()))?,
    })
}
impl PartialEq for NativeLiteral {
    fn eq(&self, other: &Self) -> bool {
        crate::model::FieldContract::from_field(self.field.as_ref().clone())
            == crate::model::FieldContract::from_field(other.field.as_ref().clone())
            && self.json == other.json
    }
}

/// Encode a visible native value directly, without constructing a row or `ScalarValue`.
/// # Errors
/// Wrong field datatype, out-of-bounds row or unsupported representation.
pub fn to_json(array: &dyn Array, field: &arrow_schema::Field, row: usize) -> Result<String> {
    if array.data_type() != field.data_type() {
        return Err(invalid("literal field and array datatype differ"));
    }
    pse_columnar::native_value::to_json(array, field, row)
        .map_err(|error| invalid(error.to_string()))
}

/// Check the declared literal representation without evaluating semantic predicates.
/// # Errors
/// The field has no lossless literal representation.
pub fn admit_type(kind: &arrow_schema::DataType) -> Result<()> {
    pse_columnar::native_value::admit_type(kind).map_err(|error| invalid(error.to_string()))
}
fn invalid(reason: impl Into<String>) -> SchemaError {
    SchemaError::InvalidDeclaration {
        context: "native literal".into(),
        reason: reason.into(),
    }
}

#[cfg(test)]
mod consolidation_unit;
