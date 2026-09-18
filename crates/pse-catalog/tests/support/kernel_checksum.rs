// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! A real native-kernel commit supplies initial CRC state unavailable from a
//! plain delta-rs open. No fixture serializes or fabricates checksum contents.

#![allow(
    clippy::panic,
    reason = "qualification fixture refuses a noncommitted kernel transaction"
)]

use datafusion::arrow::datatypes::Schema;
use deltalake::kernel::native;
use native::engine::arrow_conversion::TryFromArrow;
use std::sync::Arc;

pub(crate) async fn create_with_checksum(
    root: url::Url,
    schema: &Schema,
    engine: Arc<dyn native::Engine>,
) {
    let schema = Arc::new(deltalake::kernel::StructType::try_from_arrow(schema).unwrap());
    tokio::task::spawn_blocking(move || {
        let transaction = native::transaction::create_table::create_table(
            root.as_str(),
            schema,
            "pse-cache-qualification",
        )
        .build(
            engine.as_ref(),
            Box::new(native::committer::FileSystemCommitter::new()),
        )
        .unwrap();
        let native::transaction::CommitResult::CommittedTransaction(committed) =
            transaction.commit(engine.as_ref()).unwrap()
        else {
            panic!("isolated native checksum fixture did not commit")
        };
        let snapshot = committed.post_commit_snapshot().unwrap();
        snapshot.write_checksum(engine.as_ref()).unwrap();
    })
    .await
    .unwrap();
}
