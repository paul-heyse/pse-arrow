// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Test fixtures obtain tokens from actual typed native projections.
#![allow(clippy::unwrap_used, reason = "shared test-only native projections")]

use datafusion::{
    arrow::{
        array::{FixedSizeBinaryArray, RecordBatch},
        datatypes::Schema,
    },
    execution::context::SessionContext,
    logical_expr::col,
};
use pse_schema::{Registry, model::FieldContract};
use std::sync::Arc;

pub(crate) async fn values(
    registry: &Registry,
    relation: pse_ids::SemanticId,
    columns: &[&FieldContract],
    values: &[serde_json::Value],
) -> pse_ids::ContentHash {
    let fields = columns
        .iter()
        .map(|column| pse_schema::arrow::field_for(registry, column).unwrap())
        .collect::<Vec<_>>();
    let arrays = fields
        .iter()
        .zip(values)
        .map(|(field, value)| {
            pse_relations::testing::array_from_literals(
                registry,
                field,
                std::slice::from_ref(value),
            )
            .unwrap()
        })
        .collect();
    let batch = RecordBatch::try_new(Arc::new(Schema::new(fields)), arrays).unwrap();
    let batches = SessionContext::new()
        .read_batch(batch)
        .unwrap()
        .select(vec![pse_relations::identity::key(
            relation,
            columns
                .iter()
                .map(|column| (column.name(), col(column.name())))
                .collect(),
        )])
        .unwrap()
        .collect()
        .await
        .unwrap();
    pse_ids::ContentHash::try_from_slice(
        batches[0]
            .column(0)
            .as_any()
            .downcast_ref::<FixedSizeBinaryArray>()
            .unwrap()
            .value(0),
    )
    .unwrap()
}
