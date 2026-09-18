// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Explicit value admission in native expressions. The kernel checks values;
//! attaching a declaration alone never establishes its local predicates.

use datafusion::{
    arrow::{
        compute::{CastOptions, cast_with_options},
        datatypes::{DataType, Field, FieldRef},
    },
    common::{DataFusionError, Result, ScalarValue},
    logical_expr::{
        ColumnarValue, ReturnFieldArgs, ScalarFunctionArgs, ScalarUDF, ScalarUDFImpl, Signature,
        Volatility,
    },
};
use pse_schema::Registry;
use std::{
    hash::{Hash, Hasher},
    sync::Arc,
};

pub(crate) fn function(registry: Arc<Registry>) -> Arc<ScalarUDF> {
    Arc::new(ScalarUDF::from(CheckedValue {
        registry,
        signature: Signature::any(3, Volatility::Immutable),
    }))
}

pub(crate) fn is_bound(function: &ScalarUDF, registry: &Arc<Registry>) -> bool {
    let implementation: &dyn std::any::Any = function.inner().as_ref();
    implementation
        .downcast_ref::<CheckedValue>()
        .is_some_and(|value| Arc::ptr_eq(&value.registry, registry))
}

#[derive(Debug)]
struct CheckedValue {
    registry: Arc<Registry>,
    signature: Signature,
}
impl PartialEq for CheckedValue {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.registry, &other.registry)
    }
}
impl Eq for CheckedValue {}
impl Hash for CheckedValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Exact immutable registry ownership identifies this implementation.
        // A digest is not a substitute for comparing declarations.
        Arc::as_ptr(&self.registry).hash(state);
    }
}
impl CheckedValue {
    fn field(&self, relation: &ScalarValue, column: &ScalarValue) -> Result<FieldRef> {
        let text = |value: &ScalarValue| match value {
            ScalarValue::Utf8(Some(value)) => Ok(value.clone()),
            _ => Err(invalid("declaration names must be constant UTF-8 strings")),
        };
        let relation = text(relation)?;
        let column = text(column)?;
        let spec = self
            .registry
            .relation(&relation)
            .ok_or_else(|| invalid("unknown value declaration relation"))?;
        let field = spec
            .column(&column)
            .ok_or_else(|| invalid("unknown value declaration column"))?;
        pse_schema::arrow::field_for(&self.registry, field)
            .map(Arc::new)
            .map_err(|error| DataFusionError::External(Box::new(error)))
    }
}
impl ScalarUDFImpl for CheckedValue {
    fn name(&self) -> &'static str {
        "pse_checked_value"
    }
    fn signature(&self) -> &Signature {
        &self.signature
    }
    fn return_type(&self, _: &[DataType]) -> Result<DataType> {
        Err(invalid(
            "value admission requires its literal declaration arguments",
        ))
    }
    fn return_field_from_args(&self, args: ReturnFieldArgs<'_>) -> Result<FieldRef> {
        let [source, _, _] = args.arg_fields else {
            return Err(invalid("value admission requires three arguments"));
        };
        let [_, Some(relation), Some(column)] = args.scalar_arguments else {
            return Err(invalid("value admission declaration must be literal"));
        };
        let target = self.field(relation, column)?;
        compatible(source, &target)?;
        Ok(target)
    }
    fn invoke_with_args(&self, args: ScalarFunctionArgs) -> Result<ColumnarValue> {
        let [
            value,
            ColumnarValue::Scalar(relation),
            ColumnarValue::Scalar(column),
        ] = args.args.as_slice()
        else {
            return Err(invalid("value admission declaration must be scalar"));
        };
        let field = self.field(relation, column)?;
        let array = value.to_array(args.number_rows)?;
        let array = if array.data_type() == field.data_type() {
            array
        } else {
            // Storage shape was checked during field inference. Native Arrow
            // performs the structural cast; failed conversions never become NULL.
            cast_with_options(
                &array,
                field.data_type(),
                &CastOptions {
                    safe: false,
                    ..Default::default()
                },
            )?
        };
        pse_relations::validate::validate_column(&self.registry, &field, array.as_ref()).map_err(
            |errors| {
                DataFusionError::External(Box::new(pse_relations::RelationError::Validation {
                    errors,
                }))
            },
        )?;
        Ok(ColumnarValue::Array(array))
    }
}

fn compatible(source: &Field, target: &Field) -> Result<()> {
    use DataType::{FixedSizeList, LargeList, List, Map, Null, Struct};
    if source.data_type() == &Null {
        return Ok(());
    }
    if source.metadata().get(pse_schema::arrow::KEY_QUANTITY_TYPE)
        != target.metadata().get(pse_schema::arrow::KEY_QUANTITY_TYPE)
    {
        return Err(invalid(
            "value admission cannot invent or convert physical quantity meaning",
        ));
    }
    for key in [
        pse_schema::arrow::KEY_ENUM,
        pse_schema::arrow::KEY_EXTENSION_NAME,
        pse_schema::arrow::KEY_EXTENSION_METADATA,
    ] {
        if let Some(meaning) = source.metadata().get(key)
            && target.metadata().get(key) != Some(meaning)
        {
            return Err(invalid(
                "value admission cannot reinterpret an already typed semantic value",
            ));
        }
    }
    match (source.data_type(), target.data_type()) {
        (Struct(left), Struct(right)) if left.len() == right.len() => {
            for (left, right) in left.iter().zip(right) {
                if left.name() != right.name() {
                    return Err(invalid("value admission struct names differ"));
                }
                compatible(left, right)?;
            }
            Ok(())
        }
        (List(left), List(right)) | (LargeList(left), LargeList(right)) => compatible(left, right),
        (FixedSizeList(left, n), FixedSizeList(right, m)) if n == m => compatible(left, right),
        (Map(left, n), Map(right, m)) if n == m => compatible(left, right),
        (left, right) if left == right => Ok(()),
        _ => Err(invalid(
            "value admission requires matching storage; use an explicit native cast for a conversion",
        )),
    }
}
fn invalid(message: &str) -> DataFusionError {
    DataFusionError::Plan(message.into())
}

#[cfg(test)]
mod tests;
