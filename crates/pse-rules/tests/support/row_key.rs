// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Test fixtures obtain tokens from actual typed native projections.
#![allow(
    dead_code,
    clippy::unwrap_used,
    reason = "shared test-only native projections"
)]

use datafusion::{
    arrow::{
        array::{FixedSizeBinaryArray, RecordBatch},
        datatypes::Schema,
    },
    execution::context::SessionContext,
    logical_expr::col,
};
use pse_schema::{
    Registry,
    model::{Cell, FieldContract, RelationSpec, RuleHead, RuleSpec},
};
use std::sync::Arc;

pub(crate) async fn values(
    registry: &Registry,
    relation: pse_ids::SemanticId,
    columns: &[&FieldContract],
    values: &[Cell],
) -> pse_ids::ContentHash {
    let fields = columns
        .iter()
        .map(|column| pse_schema::arrow::field_for(registry, column).unwrap())
        .collect::<Vec<_>>();
    let arrays = fields
        .iter()
        .zip(values)
        .map(|(field, value)| {
            pse_relations::cells::array_from_cells(registry, field, std::slice::from_ref(value))
                .unwrap()
        })
        .collect();
    let batch = RecordBatch::try_new(Arc::new(Schema::new(fields)), arrays).unwrap();
    let batches = SessionContext::new()
        .read_batch(batch)
        .unwrap()
        .select(vec![pse_catalog::session::scalar::key(
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
pub(crate) async fn relation(
    spec: &RelationSpec,
    registry: &Registry,
    row: &[Cell],
) -> pse_ids::ContentHash {
    let indices = spec
        .primary_key
        .iter()
        .map(|name| {
            spec.columns
                .iter()
                .position(|column| column.name() == *name)
                .unwrap()
        })
        .collect::<Vec<_>>();
    values(
        registry,
        spec.id,
        &indices
            .iter()
            .map(|i| &spec.columns[*i])
            .collect::<Vec<_>>(),
        &indices.iter().map(|i| row[*i].clone()).collect::<Vec<_>>(),
    )
    .await
}
pub(crate) async fn rule(
    spec: &RuleSpec,
    registry: &Registry,
    row: &[Cell],
) -> pse_ids::ContentHash {
    let target = registry.relation(spec.head.relation()).unwrap();
    let names = match &spec.head {
        RuleHead::Relation(_) => &target.primary_key,
        RuleHead::Violations { key_columns, .. } => key_columns,
    };
    values(
        registry,
        target.id,
        &names
            .iter()
            .map(|name| target.column(name).unwrap())
            .collect::<Vec<_>>(),
        row,
    )
    .await
}
