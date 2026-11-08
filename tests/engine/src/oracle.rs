// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Complete multiset comparison catches lost matches as well as invalid survivors.
#![allow(
    clippy::expect_used,
    reason = "test oracle reports precise contract failures"
)]
#[path = "../../support/arrow_rows.rs"]
mod arrow_rows;
pub(crate) use arrow_rows::multiset;

use datafusion::arrow::array::RecordBatch;
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
