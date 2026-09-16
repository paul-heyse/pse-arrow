// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! A query source retains column meaning; persisted relation identity belongs to its owner.

use datafusion::arrow::{
    array::RecordBatch,
    datatypes::{Schema, SchemaRef},
};
use std::sync::Arc;

pub(crate) fn schema(input: &Schema) -> SchemaRef {
    let mut metadata = input.metadata().clone();
    metadata.retain(|key, _| !key.starts_with("pse.contract.") && key != "pse.namespace");
    Arc::new(Schema::new_with_metadata(input.fields().clone(), metadata))
}

pub(crate) fn batch(input: &RecordBatch) -> datafusion::common::Result<RecordBatch> {
    Ok(RecordBatch::try_new_with_options(
        schema(input.schema().as_ref()),
        input.columns().to_vec(),
        &datafusion::arrow::array::RecordBatchOptions::new().with_row_count(Some(input.num_rows())),
    )?)
}
