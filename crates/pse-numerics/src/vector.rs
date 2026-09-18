// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Declared numerical vectors use native array packing without a scalar round trip.
//! At the pinned release, scalar list extraction discards child field metadata.
use datafusion::{
    arrow::{
        array::{Array, Float64Array, ListArray},
        buffer::OffsetBuffer,
        datatypes::{DataType, FieldRef},
    },
    common::{DataFusionError, Result},
    functions_nested::make_array::array_array,
    logical_expr::{
        ColumnarValue, ReturnFieldArgs, ScalarFunctionArgs, ScalarUDF, ScalarUDFImpl, Signature,
        Volatility,
    },
};
use pse_ids::{CancellationToken, ReservationLease};
use std::{
    hash::{Hash, Hasher},
    sync::Arc,
};

#[derive(Debug)]
struct Binding {
    field: FieldRef,
    signature: Signature,
    cancel: CancellationToken,
    _allocation: Arc<ReservationLease>,
}
#[derive(Debug, Clone)]
struct Vector(Arc<Binding>);
impl PartialEq for Vector {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for Vector {}
impl Hash for Vector {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.0).hash(state);
    }
}
impl ScalarUDFImpl for Vector {
    fn name(&self) -> &'static str {
        "pse_numerical_vector"
    }
    fn signature(&self) -> &Signature {
        &self.0.signature
    }
    fn return_type(&self, _: &[DataType]) -> Result<DataType> {
        Ok(self.0.field.data_type().clone())
    }
    fn return_field_from_args(&self, _: ReturnFieldArgs<'_>) -> Result<FieldRef> {
        Ok(Arc::clone(&self.0.field))
    }
    fn invoke_with_args(&self, args: ScalarFunctionArgs) -> Result<ColumnarValue> {
        self.0
            .cancel
            .checkpoint()
            .map_err(|error| DataFusionError::External(Box::new(error)))?;
        let DataType::List(child) = self.0.field.data_type() else {
            return Err(DataFusionError::Internal(
                "declared numerical vector is not a list".into(),
            ));
        };
        let arrays = args
            .args
            .iter()
            .map(|arg| arg.to_array(args.number_rows))
            .collect::<Result<Vec<_>>>()?;
        if arrays.iter().any(|array| array.null_count() != 0) {
            return Err(DataFusionError::Execution(
                "null numerical vector member".into(),
            ));
        }
        let result = if arrays.is_empty() {
            ListArray::try_new(
                Arc::clone(child),
                OffsetBuffer::from_lengths(std::iter::repeat_n(0, args.number_rows)),
                Arc::new(Float64Array::from(Vec::<f64>::new())),
                None,
            )?
        } else {
            let packed = array_array::<i32>(&arrays, DataType::Float64, child.name())?;
            let list = packed.as_any().downcast_ref::<ListArray>().ok_or_else(|| {
                DataFusionError::Internal("native vector packing did not return a list".into())
            })?;
            let (_, offsets, values, nulls) = list.clone().into_parts();
            ListArray::try_new(Arc::clone(child), offsets, values, nulls)?
        };
        Ok(ColumnarValue::Array(Arc::new(result)))
    }
}
pub(crate) fn function(
    field: FieldRef,
    dimension: usize,
    cancel: CancellationToken,
    allocation: Arc<ReservationLease>,
) -> ScalarUDF {
    ScalarUDF::from(Vector(Arc::new(Binding {
        field,
        // Invocation cancellation and the retained preparation allocation are observable,
        // including a zero-dimensional program. Constant folding would erase that check.
        signature: Signature::exact(vec![DataType::Float64; dimension], Volatility::Volatile),
        cancel,
        _allocation: allocation,
    })))
}
