// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Exact typed field correspondence around the pinned native array-concat kernel.
//! This leaves unrestricted native `array_concat` available. The adapter serves callers
//! that require an unchanged List element domain or ordered `IndexTuple` semantics.

#[cfg(test)]
mod tests;

use datafusion::{
    arrow::{
        array::RecordBatch,
        compute::{CastOptions, cast_with_options},
        datatypes::{DataType, FieldRef, Schema},
    },
    common::{DataFusionError, Result},
    functions_nested::concat::array_concat_udf,
    logical_expr::{
        ColumnarValue, ReturnFieldArgs, ScalarFunctionArgs, ScalarUDF, ScalarUDFImpl, Signature,
        Volatility,
    },
};
use pse_ids::{MemoryReserver, ReservationLease};
use std::{
    hash::{Hash, Hasher},
    sync::Arc,
};

pub(super) fn function(reserver: Arc<dyn MemoryReserver>) -> Arc<ScalarUDF> {
    Arc::new(ScalarUDF::from(TypedConcat {
        signature: Signature::variadic_any(Volatility::Immutable),
        native: array_concat_udf(),
        reserver,
    }))
}
#[derive(Debug)]
struct TypedConcat {
    signature: Signature,
    native: Arc<ScalarUDF>,
    reserver: Arc<dyn MemoryReserver>,
}
impl PartialEq for TypedConcat {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.reserver, &other.reserver) && Arc::ptr_eq(&self.native, &other.native)
    }
}
impl Eq for TypedConcat {}
impl Hash for TypedConcat {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::ptr::hash(Arc::as_ptr(&self.reserver), state);
        std::ptr::hash(Arc::as_ptr(&self.native), state);
    }
}
impl ScalarUDFImpl for TypedConcat {
    fn name(&self) -> &'static str {
        "pse_array_concat"
    }
    fn signature(&self) -> &Signature {
        &self.signature
    }
    fn return_type(&self, args: &[DataType]) -> Result<DataType> {
        self.native.return_type(args)
    }
    fn return_field_from_args(&self, args: ReturnFieldArgs<'_>) -> Result<FieldRef> {
        let first = args
            .arg_fields
            .first()
            .ok_or_else(|| invalid("at least one list is required"))?;
        let DataType::List(first_child) = first.data_type() else {
            return Err(invalid("equal-layout List inputs are required"));
        };
        if first
            .metadata()
            .get(pse_schema::arrow::KEY_EXTENSION_NAME)
            .is_some_and(|extension| extension != "pse.index_tuple")
        {
            return Err(invalid(
                "this adapter cannot preserve a length-sensitive outer extension",
            ));
        }
        let mut child = first_child.as_ref().clone();
        for field in args.arg_fields {
            let DataType::List(actual) = field.data_type() else {
                return Err(invalid("equal-layout List inputs are required"));
            };
            let nullable = child.is_nullable() || actual.is_nullable();
            child = child.with_nullable(nullable);
        }
        let expected = first
            .as_ref()
            .clone()
            .with_nullable(true)
            .with_data_type(DataType::List(Arc::new(child)));
        for field in args.arg_fields {
            super::super::output::check_field_output(field, &expected)?;
        }
        // The pinned kernel ORs validity bitmaps: a row is NULL only when every
        // argument is NULL. It copies source values in their original argument order.
        Ok(Arc::new(expected.with_nullable(
            args.arg_fields.iter().all(|field| field.is_nullable()),
        )))
    }
    fn invoke_with_args(&self, args: ScalarFunctionArgs) -> Result<ColumnarValue> {
        let extent = args.args.iter().try_fold(8192_usize, |sum, value| {
            let size = match value {
                ColumnarValue::Array(array) => Some(array.get_array_memory_size()),
                ColumnarValue::Scalar(value) => value
                    .size()
                    .checked_add(128)
                    .and_then(|size| size.checked_mul(args.number_rows.max(1))),
            }
            .and_then(|size| size.checked_mul(6));
            size.and_then(|size| sum.checked_add(size)).ok_or_else(|| {
                DataFusionError::ResourcesExhausted("typed list concat extent overflow".to_owned())
            })
        })?;
        let mut reservation = self.reserver.open("query:typed-list-concat");
        reservation.try_grow(extent).map_err(external)?;
        let field = Arc::clone(&args.return_field);
        let count = args.number_rows;
        // Delegate to the actual native implementation, then check our adapter's
        // output with Arrow below. The outer ScalarUDF convenience call asserts
        // that the kernel already emitted our stronger child field, although the
        // purpose of this adapter is to realize that field after native concat.
        let values = self
            .native
            .inner()
            .invoke_with_args(args)?
            .into_array(count)?;
        // The native constructor resets the list's child labels; this native cast
        // realizes the already-proven actual argument-to-output field correspondence.
        let values = cast_with_options(
            values.as_ref(),
            field.data_type(),
            &CastOptions {
                safe: false,
                ..CastOptions::default()
            },
        )?;
        let batch = RecordBatch::try_new(Arc::new(Schema::new(vec![field])), vec![values])?;
        let actual = pse_ids::owned_buffer::retained_buffer_bytes(&batch).map_err(external)?;
        reservation.shrink(reservation.size().saturating_sub(actual));
        let owned =
            pse_ids::owned_buffer::attach_reservation(batch, ReservationLease::new(reservation))
                .map_err(external)?;
        Ok(ColumnarValue::Array(Arc::clone(owned.column(0))))
    }
}
fn invalid(reason: &str) -> DataFusionError {
    DataFusionError::Plan(format!("pse_array_concat: {reason}"))
}
fn external(error: impl std::error::Error + Send + Sync + 'static) -> DataFusionError {
    DataFusionError::External(Box::new(error))
}
