// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! A native assertion preserves failure of every evaluated numerical intermediate.
use datafusion::{
    arrow::{
        array::{Array, Float64Array},
        datatypes::{DataType, FieldRef},
    },
    common::{DataFusionError, Result},
    logical_expr::{
        ColumnarValue, Expr, ReturnFieldArgs, ScalarFunctionArgs, ScalarUDF, ScalarUDFImpl,
        Signature, Volatility,
    },
};
use pse_ids::{CancellationToken, ReservationLease};
use std::{
    hash::{Hash, Hasher},
    sync::Arc,
};

#[derive(Debug)]
struct Binding {
    cancel: CancellationToken,
    signature: Signature,
    _allocation: Arc<ReservationLease>,
}
#[derive(Clone, Debug)]
struct Finite(Arc<Binding>);
impl PartialEq for Finite {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for Finite {}
impl Hash for Finite {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.0).hash(state);
    }
}
impl ScalarUDFImpl for Finite {
    fn name(&self) -> &'static str {
        "pse_finite_numerical_value"
    }
    fn signature(&self) -> &Signature {
        &self.0.signature
    }
    fn return_type(&self, _: &[DataType]) -> Result<DataType> {
        Ok(DataType::Float64)
    }
    fn return_field_from_args(&self, args: ReturnFieldArgs<'_>) -> Result<FieldRef> {
        let [field] = args.arg_fields else {
            return Err(DataFusionError::Internal(
                "finite assertion requires one field".into(),
            ));
        };
        Ok(Arc::new(field.as_ref().clone().with_nullable(false)))
    }
    fn invoke_with_args(&self, args: ScalarFunctionArgs) -> Result<ColumnarValue> {
        self.0
            .cancel
            .checkpoint()
            .map_err(|error| DataFusionError::External(Box::new(error)))?;
        let [value] = args.args.as_slice() else {
            return Err(DataFusionError::Internal(
                "finite assertion requires one argument".into(),
            ));
        };
        let array = value.to_array(args.number_rows)?;
        let values = array
            .as_any()
            .downcast_ref::<Float64Array>()
            .ok_or_else(|| DataFusionError::Internal("finite assertion requires Float64".into()))?;
        if values.null_count() != 0 || values.values().iter().any(|value| !value.is_finite()) {
            return Err(DataFusionError::Execution(
                "non-finite or null numerical intermediate".into(),
            ));
        }
        Ok(value.clone())
    }
}

pub(crate) fn function(
    cancel: CancellationToken,
    allocation: Arc<ReservationLease>,
) -> Arc<ScalarUDF> {
    Arc::new(ScalarUDF::from(Finite(Arc::new(Binding {
        cancel,
        // Cancellation is invocation state. Folding or eliding this check would
        // change observable execution, even when its numeric input is constant.
        signature: Signature::exact(vec![DataType::Float64], Volatility::Volatile),
        _allocation: allocation,
    }))))
}
pub(crate) fn checked(function: &ScalarUDF, value: Expr) -> Expr {
    function.call(vec![value])
}
