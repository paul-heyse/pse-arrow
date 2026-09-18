// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Field-preserving collection delegates all aggregation to the pinned native UDAF.
//! The child field follows the actual argument, including metadata; no declaration
//! or caller-supplied target can reinterpret its values.

mod collection;
mod selection;

use datafusion::{
    arrow::{
        array::{Array, ArrayRef},
        datatypes::{DataType, Field, FieldRef},
    },
    common::{DataFusionError, Result, ScalarValue},
    functions_aggregate::array_agg::array_agg_udaf,
    logical_expr::{
        Accumulator, AggregateUDF, AggregateUDFImpl, GroupsAccumulator, ReversedUDAF, Signature,
        function::{AccumulatorArgs, StateFieldsArgs},
        utils::AggregateOrderSensitivity,
    },
};
use std::sync::Arc;

pub(crate) fn adapt(function: &Arc<AggregateUDF>) -> Option<Arc<AggregateUDF>> {
    if function.as_ref() == array_agg_udaf().as_ref() {
        Some(Arc::new(AggregateUDF::from(Collection {
            native: Arc::clone(function),
        })))
    } else {
        selection::adapt(function)
    }
}
pub(crate) fn native(function: &AggregateUDF) -> Option<&Arc<AggregateUDF>> {
    if let Some(collection) = function.inner().downcast_ref::<Collection>() {
        Some(&collection.native)
    } else {
        selection::native(function)
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct Collection {
    native: Arc<AggregateUDF>,
}

impl AggregateUDFImpl for Collection {
    fn name(&self) -> &'static str {
        "array_agg"
    }
    fn signature(&self) -> &Signature {
        self.native.signature()
    }
    fn return_type(&self, args: &[DataType]) -> Result<DataType> {
        self.native.return_type(args)
    }
    fn return_field(&self, args: &[FieldRef]) -> Result<FieldRef> {
        let [source] = args else {
            return Err(invalid("requires exactly one actual input field"));
        };
        // Collection keeps input null members and yields NULL for an empty group.
        Ok(Arc::new(Field::new_list(self.name(), child(source), true)))
    }
    fn state_fields(&self, args: StateFieldsArgs<'_>) -> Result<Vec<FieldRef>> {
        let field = child(
            args.input_fields
                .first()
                .ok_or_else(|| invalid("input field absent"))?,
        );
        let mut fields = self.native.state_fields(args)?;
        let first = fields
            .first_mut()
            .ok_or_else(|| invalid("native value state absent"))?;
        *first = Arc::new(first.as_ref().clone().with_data_type(DataType::List(field)));
        Ok(fields)
    }
    fn accumulator(&self, args: AccumulatorArgs<'_>) -> Result<Box<dyn Accumulator>> {
        let field = child(
            args.expr_fields
                .first()
                .ok_or_else(|| invalid("input field absent"))?,
        );
        Ok(Box::new(collection::Single {
            native: self.native.accumulator(args)?,
            field,
        }))
    }
    fn groups_accumulator_supported(&self, args: AccumulatorArgs<'_>) -> bool {
        self.native.groups_accumulator_supported(args)
    }
    fn create_groups_accumulator(
        &self,
        args: AccumulatorArgs<'_>,
    ) -> Result<Box<dyn GroupsAccumulator>> {
        let field = child(
            args.expr_fields
                .first()
                .ok_or_else(|| invalid("input field absent"))?,
        );
        Ok(Box::new(collection::Groups {
            native: self.native.create_groups_accumulator(args)?,
            field,
        }))
    }
    fn order_sensitivity(&self) -> AggregateOrderSensitivity {
        self.native.order_sensitivity()
    }
    fn with_beneficial_ordering(
        self: Arc<Self>,
        beneficial: bool,
    ) -> Result<Option<Arc<dyn AggregateUDFImpl>>> {
        Ok(self
            .native
            .as_ref()
            .clone()
            .with_beneficial_ordering(beneficial)?
            .map(|native| {
                let function: Arc<dyn AggregateUDFImpl> = Arc::new(Self {
                    native: Arc::new(native),
                });
                function
            }))
    }
    fn reverse_expr(&self) -> ReversedUDAF {
        ReversedUDAF::Reversed(Arc::new(AggregateUDF::from(Collection {
            native: Arc::clone(&self.native),
        })))
    }
    fn supports_null_handling_clause(&self) -> bool {
        self.native.supports_null_handling_clause()
    }
}

fn child(source: &Field) -> FieldRef {
    Arc::new(source.clone().with_name("item"))
}

fn restore(value: &ArrayRef, field: &FieldRef) -> Result<ArrayRef> {
    use datafusion::arrow::array::ListArray;
    let list = value
        .as_any()
        .downcast_ref::<ListArray>()
        .ok_or_else(|| invalid("native collection is not a List"))?;
    let DataType::List(actual) = list.data_type() else {
        return Err(invalid("native collection layout changed"));
    };
    if actual.data_type() != field.data_type()
        || actual.name() != field.name()
        || actual
            .metadata()
            .iter()
            .any(|(key, value)| field.metadata().get(key) != Some(value))
    {
        return Err(invalid("native collection changed its actual input field"));
    }
    Ok(Arc::new(ListArray::try_new(
        Arc::clone(field),
        list.offsets().clone(),
        Arc::clone(list.values()),
        list.nulls().cloned(),
    )?))
}

fn restore_scalar(value: ScalarValue, field: &FieldRef) -> Result<ScalarValue> {
    let ScalarValue::List(list) = value else {
        return Err(invalid("native collection scalar is not a List"));
    };
    let value: ArrayRef = list;
    let result = restore(&value, field)?;
    // DF 55.1 ScalarValue::try_from_array uses SingleRowListArrayBuilder,
    // whose with_field copies only the child name and nullability. Keep the
    // already single-row List directly, including its restored child metadata.
    let result = result
        .as_any()
        .downcast_ref::<datafusion::arrow::array::ListArray>()
        .ok_or_else(|| invalid("restored collection scalar is not a List"))?;
    Ok(ScalarValue::List(Arc::new(result.clone())))
}

fn invalid(reason: &str) -> DataFusionError {
    DataFusionError::Internal(format!("array_agg field transfer: {reason}"))
}
