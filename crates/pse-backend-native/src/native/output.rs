// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Transient solver return tuple; durable runtime facts are selected by normal
//! native projection/UNNEST and published through the common Delta route.
use datafusion::arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use std::sync::Arc;

pub(super) fn schema() -> SchemaRef {
    let list = DataType::LargeList(Arc::new(Field::new("item", DataType::Float64, true)));
    Arc::new(Schema::new(vec![
        Field::new("ipopt_status", DataType::Int32, false),
        Field::new("cancelled", DataType::Boolean, false),
        Field::new("objective", DataType::Float64, true),
        Field::new("values", list.clone(), false),
        Field::new("constraints", list.clone(), false),
        Field::new("constraint_duals", list.clone(), false),
        Field::new("lower_duals", list.clone(), false),
        Field::new("upper_duals", list, false),
        Field::new("diagnostic_code", DataType::Utf8, true),
        Field::new("diagnostic", DataType::Utf8, true),
        Field::new(
            "iterations",
            DataType::LargeList(Arc::new(Field::new(
                "item",
                DataType::Struct(iteration_fields()),
                false,
            ))),
            false,
        ),
    ]))
}
fn iteration_fields() -> datafusion::arrow::datatypes::Fields {
    vec![
        Field::new("iteration", DataType::Int32, false),
        Field::new("restoration", DataType::Boolean, false),
        Field::new("objective", DataType::Float64, true),
        Field::new("primal_infeasibility", DataType::Float64, true),
        Field::new("dual_infeasibility", DataType::Float64, true),
        Field::new("barrier", DataType::Float64, true),
        Field::new("step", DataType::Float64, true),
    ]
    .into()
}

#[cfg(feature = "ipopt")]
pub(super) use linked::pack;
#[cfg(feature = "ipopt")]
mod linked {
    use super::{iteration_fields, schema};
    use datafusion::arrow::datatypes::{DataType, Field};
    use datafusion::arrow::{
        array::{
            ArrayRef, BooleanArray, Float64Array, Int32Array, LargeListArray, RecordBatch,
            StringArray, StructArray,
        },
        buffer::OffsetBuffer,
    };
    use std::sync::Arc;
    fn optional(value: f64) -> Option<f64> {
        value.is_finite().then_some(value)
    }
    fn list(values: Vec<f64>) -> datafusion::common::Result<ArrayRef> {
        let len = i64::try_from(values.len()).map_err(|_| {
            datafusion::common::DataFusionError::Execution("solver result extent".into())
        })?;
        Ok(Arc::new(LargeListArray::try_new(
            Arc::new(Field::new("item", DataType::Float64, true)),
            OffsetBuffer::new(vec![0, len].into()),
            Arc::new(Float64Array::from(
                values.into_iter().map(optional).collect::<Vec<_>>(),
            )),
            None,
        )?))
    }
    fn iterations(history: &[crate::driver::Iteration]) -> datafusion::common::Result<ArrayRef> {
        let iteration_len = i64::try_from(history.len()).map_err(|_| {
            datafusion::common::DataFusionError::Execution("solver history extent".into())
        })?;
        let fields = iteration_fields();
        let rows = StructArray::try_new(
            fields.clone(),
            vec![
                Arc::new(Int32Array::from(
                    history.iter().map(|i| i.index).collect::<Vec<_>>(),
                )),
                Arc::new(BooleanArray::from(
                    history.iter().map(|i| i.restoration).collect::<Vec<_>>(),
                )),
                Arc::new(Float64Array::from(
                    history
                        .iter()
                        .map(|i| optional(i.objective))
                        .collect::<Vec<_>>(),
                )),
                Arc::new(Float64Array::from(
                    history
                        .iter()
                        .map(|i| optional(i.primal_infeasibility))
                        .collect::<Vec<_>>(),
                )),
                Arc::new(Float64Array::from(
                    history
                        .iter()
                        .map(|i| optional(i.dual_infeasibility))
                        .collect::<Vec<_>>(),
                )),
                Arc::new(Float64Array::from(
                    history
                        .iter()
                        .map(|i| optional(i.barrier))
                        .collect::<Vec<_>>(),
                )),
                Arc::new(Float64Array::from(
                    history.iter().map(|i| optional(i.step)).collect::<Vec<_>>(),
                )),
            ],
            None,
        )?;
        Ok(Arc::new(LargeListArray::try_new(
            Arc::new(Field::new("item", DataType::Struct(fields), false)),
            OffsetBuffer::new(vec![0, iteration_len].into()),
            Arc::new(rows),
            None,
        )?))
    }
    pub(crate) fn pack(outcome: crate::driver::Outcome) -> datafusion::common::Result<RecordBatch> {
        let code = outcome
            .failure
            .as_ref()
            .and_then(miette::Diagnostic::code)
            .map(|code| code.to_string());
        let diagnostic = outcome.failure.as_ref().map(ToString::to_string);
        let batch = RecordBatch::try_new(
            schema(),
            vec![
                Arc::new(Int32Array::from(vec![outcome.status])),
                Arc::new(BooleanArray::from(vec![outcome.cancelled])),
                Arc::new(Float64Array::from(vec![optional(outcome.objective)])),
                list(outcome.values)?,
                list(outcome.constraints)?,
                list(outcome.constraint_duals)?,
                list(outcome.lower_duals)?,
                list(outcome.upper_duals)?,
                Arc::new(StringArray::from(vec![code.as_deref()])),
                Arc::new(StringArray::from(vec![diagnostic.as_deref()])),
                iterations(&outcome.iterations)?,
            ],
        )?;
        pse_ids::owned_buffer::attach_reservation(batch, outcome.allocation)
            .map_err(|error| datafusion::common::DataFusionError::External(Box::new(error)))
    }
}
