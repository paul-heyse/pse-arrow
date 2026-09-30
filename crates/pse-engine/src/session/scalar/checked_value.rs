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

pub(crate) fn function(
    registry: Arc<Registry>,
    state: datafusion::execution::session_state::SessionState,
) -> Arc<ScalarUDF> {
    let validation = Arc::new(pse_relations::validate::ValidationContext::new(
        &registry,
        crate::validation::NativeValidation(state),
    ));
    Arc::new(ScalarUDF::from(CheckedValue {
        validation,
        registry,
        signature: Signature::any(3, Volatility::Immutable),
    }))
}

#[derive(Debug)]
struct CheckedValue {
    validation: Arc<pse_relations::validate::ValidationContext>,
    registry: Arc<Registry>,
    signature: Signature,
}
impl PartialEq for CheckedValue {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.validation, &other.validation)
    }
}
impl Eq for CheckedValue {}
impl Hash for CheckedValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Exact registry and captured session ownership identifies this implementation.
        // A digest is not a substitute for comparing declarations.
        Arc::as_ptr(&self.validation).hash(state);
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
        let schema = pse_schema::arrow::relation_schema_ref(&self.registry, spec)
            .map_err(pse_columnar::external)?;
        let index = schema.index_of(&column).map_err(DataFusionError::from)?;
        Ok(Arc::clone(&schema.fields()[index]))
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
        self.validation
            .column(&self.registry, &target)
            .map_err(pse_columnar::external)?;
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
        let prepared = self
            .validation
            .column(&self.registry, &field)
            .map_err(pse_columnar::external)?;
        let batch = datafusion::arrow::array::RecordBatch::try_new(
            Arc::clone(prepared.schema()),
            vec![Arc::clone(&array)],
        )?;
        prepared
            .evaluate(&batch, 256, &pse_columnar::CancellationToken::new())
            .and_then(pse_relations::validate::ValidationReport::require_valid)
            .map_err(pse_columnar::external)?;
        Ok(ColumnarValue::Array(array))
    }
}

fn compatible(source: &Field, target: &Field) -> Result<()> {
    if source.data_type() == &DataType::Null {
        return Ok(());
    }
    if !pse_columnar::native_field::admits_metadata(
        source.metadata(), target.metadata(),
        pse_columnar::native_field::MetadataAdmission::CheckedTarget,
    ) {
        return Err(invalid(
            "value admission cannot invent physical or unclassified meaning, or reinterpret established semantic meaning",
        ));
    }
    if source.dict_is_ordered() != target.dict_is_ordered() {
        return Err(invalid("value admission dictionary ordering differs"));
    }
    compatible_type(source.data_type(), target.data_type())
}

fn compatible_type(source: &DataType, target: &DataType) -> Result<()> {
    use DataType::{Dictionary, FixedSizeList, LargeList, LargeListView, List, ListView, Map, RunEndEncoded, Struct, Union};
    match (source, target) {
        (Struct(left), Struct(right)) if left.len() == right.len() => {
            for (left, right) in left.iter().zip(right) {
                if left.name() != right.name() {
                    return Err(invalid("value admission struct names differ"));
                }
                compatible(left, right)?;
            }
            Ok(())
        }
        (List(left), List(right)) | (LargeList(left), LargeList(right))
        | (ListView(left), ListView(right)) | (LargeListView(left), LargeListView(right)) => compatible(left, right),
        (FixedSizeList(left, n), FixedSizeList(right, m)) if n == m => compatible(left, right),
        (Map(left, n), Map(right, m)) if n == m => compatible(left, right),
        (Dictionary(left_key, left), Dictionary(right_key, right)) if left_key == right_key => compatible_type(left, right),
        (Union(left, left_mode), Union(right, right_mode)) if left_mode == right_mode && left.len() == right.len() => {
            for ((left_id, left), (right_id, right)) in left.iter().zip(right.iter()) {
                if left_id != right_id || left.name() != right.name() {
                    return Err(invalid("value admission union arms differ"));
                }
                compatible(left, right)?;
            }
            Ok(())
        }
        (RunEndEncoded(left_runs, left), RunEndEncoded(right_runs, right)) => {
            compatible(left_runs, right_runs)?;
            compatible(left, right)
        }
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
