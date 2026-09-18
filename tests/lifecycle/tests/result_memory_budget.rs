// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Exported Arrow buffers retain accounted ownership after session/provider drops.

mod support;
use pse_ids::CancellationToken;
use std::sync::Arc;

#[tokio::test]
async fn detached_result_arrays_hold_their_actual_shared_pool_claim() {
    let directory = tempfile::tempdir().expect("store");
    let runtime = support::runtime(directory.path(), 64 << 20);
    let reg = support::registry();
    let cancel = CancellationToken::default();
    let session = support::session(&runtime, reg, 100);
    let result = session
        .sql("SELECT id, value FROM authored.samples", &cancel)
        .await
        .expect("bounded exported result");
    let array = Arc::clone(result[0].column(1));
    let buffer = array.to_data().buffers()[0].clone();
    drop(array);
    drop(result);
    drop(session);
    let retained = runtime.reserver().reserved();
    assert!(
        retained >= 100 * 8,
        "detached buffer still owns its allocation claim"
    );
    let clone = buffer.clone();
    assert_eq!(
        runtime.reserver().reserved(),
        retained,
        "clones do not double charge"
    );
    drop(buffer);
    assert_eq!(runtime.reserver().reserved(), retained);
    drop(clone);
    assert_eq!(runtime.reserver().reserved(), 0);
}
