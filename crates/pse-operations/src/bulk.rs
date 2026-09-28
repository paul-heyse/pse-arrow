// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Binary `COPY` into store tables (ADR-0114 Outcome 26), with the statements and type
//! probes generated from the registry (`generated/copy.rs`).

use tokio_postgres::binary_copy::BinaryCopyInWriter;
use tokio_postgres::types::{ToSql, Type};

use crate::attempts::Tx;
use crate::error::{Classify, OperationsError, Target};
use crate::generated::copy::CopyIn;

/// One row's cells for a binary copy, in the table's column order.
pub(crate) type Cells<'v> = Vec<&'v (dyn ToSql + Sync)>;

/// Copy rows into a store table in the binary format, typed by the table's probe (a
/// domain column arrives as its base type, an ENUM column as its ENUM type). Returns the
/// rows copied.
pub(crate) async fn copy_in(
    tx: &Tx<'_>,
    target: &Target,
    table: &CopyIn,
    rows: &[Cells<'_>],
) -> Result<u64, OperationsError> {
    let probe = tx.prepare_cached(table.probe).await.classify(target)?;
    let types: Vec<Type> = probe
        .columns()
        .iter()
        .map(|column| column.type_().clone())
        .collect();
    let sink = tx.copy_in(table.statement).await.classify(target)?;
    let writer = BinaryCopyInWriter::new(sink, &types);
    tokio::pin!(writer);
    for row in rows {
        // The writer refuses a row of another width by panicking; refuse it first.
        if row.len() != types.len() {
            return Err(OperationsError::InvalidRequest {
                reason: format!(
                    "a {}-cell row copied into {} of {} columns",
                    row.len(),
                    table.table,
                    types.len()
                ),
            });
        }
        writer.as_mut().write(row).await.classify(target)?;
    }
    writer.finish().await.classify(target)
}
