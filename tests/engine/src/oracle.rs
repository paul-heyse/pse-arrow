// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Complete multiset comparison catches lost matches as well as invalid survivors.
#![allow(
    clippy::expect_used,
    reason = "test oracle reports precise contract failures"
)]
use datafusion::arrow::{
    array::RecordBatch,
    row::{RowConverter, SortField},
};
use datafusion::catalog::TableProvider;
use datafusion::execution::context::{SessionConfig, SessionContext};
use std::sync::Arc;

pub(crate) async fn query(table: Arc<dyn TableProvider>, sql: &str) -> Vec<RecordBatch> {
    let context = SessionContext::new_with_config(SessionConfig::new().with_target_partitions(1));
    context
        .register_table("items", table)
        .expect("register isolated oracle source");
    context
        .sql(sql)
        .await
        .expect("plan oracle query")
        .collect()
        .await
        .expect("execute oracle query")
}
pub(crate) fn multiset(batches: &[RecordBatch]) -> Vec<Vec<u8>> {
    let Some(first) = batches.first() else {
        return Vec::new();
    };
    let converter = RowConverter::new(
        first
            .schema()
            .fields()
            .iter()
            .map(|field| SortField::new(field.data_type().clone()))
            .collect(),
    )
    .expect("exact Arrow row encoding");
    let mut rows = Vec::new();
    for batch in batches {
        assert_eq!(batch.schema(), first.schema());
        let encoded = converter
            .convert_columns(batch.columns())
            .expect("actual row values");
        rows.extend((0..batch.num_rows()).map(|row| encoded.row(row).as_ref().to_vec()));
    }
    rows.sort();
    rows
}
