// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Native Arrow list casts with explicit field and child-nullability obligations.
//! DataFusion's CAST eligibility rejects these refinements before Arrow can check
//! the actual child validity. This scalar delegates the cast to that pinned kernel.

use datafusion::{
    arrow::{
        array::RecordBatch,
        compute::{CastOptions, cast_with_options},
        datatypes::{DataType, Field, FieldRef, Schema},
    },
    common::{DataFusionError, Result},
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
    Arc::new(ScalarUDF::from(ListField {
        signature: Signature::any(2, Volatility::Immutable),
        reserver,
    }))
}

#[derive(Debug)]
struct ListField {
    signature: Signature,
    reserver: Arc<dyn MemoryReserver>,
}
impl PartialEq for ListField {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.reserver, &other.reserver)
    }
}
impl Eq for ListField {}
impl Hash for ListField {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::ptr::hash(Arc::as_ptr(&self.reserver), state);
    }
}
impl ScalarUDFImpl for ListField {
    fn name(&self) -> &'static str {
        "pse_list_field"
    }
    fn signature(&self) -> &Signature {
        &self.signature
    }
    fn return_type(&self, args: &[DataType]) -> Result<DataType> {
        args.get(1)
            .cloned()
            .ok_or_else(|| invalid("expected input and list template"))
    }
    fn return_field_from_args(&self, args: ReturnFieldArgs<'_>) -> Result<FieldRef> {
        let [source, target] = args.arg_fields else {
            return Err(invalid("expected two fields"));
        };
        if !matches!(
            (source.data_type(), target.data_type()),
            (DataType::List(_), DataType::List(_))
        ) {
            return Err(invalid("only equal-layout List fields are supported"));
        }
        // Only nullability is deferred to the native checked constructor. Every
        // semantic child label is proved from the actual source before execution.
        super::super::output::check_field_output(&nullable(source), &nullable(target))?;
        Ok(Arc::new(
            target.as_ref().clone().with_nullable(source.is_nullable()),
        ))
    }
    fn invoke_with_args(&self, args: ScalarFunctionArgs) -> Result<ColumnarValue> {
        if args.args.len() != 2 {
            return Err(invalid("expected two values"));
        }
        cast_field(args, self.reserver.as_ref(), "query:list-field")
    }
}

/// Realize an already established output field with Arrow's checked constructor.
pub(super) fn cast_field(
    args: ScalarFunctionArgs,
    reserver: &dyn MemoryReserver,
    owner: &str,
) -> Result<ColumnarValue> {
    let value = args
        .args
        .first()
        .ok_or_else(|| invalid("expected one input value"))?;
    let extent = match value {
        ColumnarValue::Array(array) => Some(array.get_array_memory_size()),
        ColumnarValue::Scalar(value) => value
            .size()
            .checked_add(128)
            .and_then(|size| size.checked_mul(args.number_rows.max(1))),
    }
    .and_then(|size| size.checked_mul(4))
    .and_then(|size| size.checked_add(8192))
    .ok_or_else(|| {
        DataFusionError::ResourcesExhausted("list field cast extent overflow".to_owned())
    })?;
    let mut reservation = reserver.open(owner);
    reservation.try_grow(extent).map_err(external)?;
    let array = value.clone().into_array(args.number_rows)?;
    let array = cast_with_options(
        array.as_ref(),
        args.return_field.data_type(),
        &CastOptions {
            safe: false,
            ..CastOptions::default()
        },
    )?;
    let batch = RecordBatch::try_new(Arc::new(Schema::new(vec![args.return_field])), vec![array])?;
    let actual = pse_ids::owned_buffer::retained_buffer_bytes(&batch).map_err(external)?;
    reservation.shrink(reservation.size().saturating_sub(actual));
    let retained =
        pse_ids::owned_buffer::attach_reservation(batch, ReservationLease::new(reservation))
            .map_err(external)?;
    Ok(ColumnarValue::Array(Arc::clone(retained.column(0))))
}

fn nullable(field: &Field) -> Field {
    let kind = match field.data_type() {
        DataType::List(child) => DataType::List(Arc::new(nullable(child))),
        DataType::LargeList(child) => DataType::LargeList(Arc::new(nullable(child))),
        DataType::FixedSizeList(child, size) => {
            DataType::FixedSizeList(Arc::new(nullable(child)), *size)
        }
        DataType::Struct(fields) => DataType::Struct(
            fields
                .iter()
                .map(|field| Arc::new(nullable(field)))
                .collect(),
        ),
        other => other.clone(),
    };
    field.clone().with_nullable(true).with_data_type(kind)
}
fn invalid(reason: &str) -> DataFusionError {
    DataFusionError::Plan(format!("pse_list_field: {reason}"))
}
fn external(error: impl std::error::Error + Send + Sync + 'static) -> DataFusionError {
    DataFusionError::External(Box::new(error))
}
