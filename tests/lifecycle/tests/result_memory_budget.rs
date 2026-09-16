// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Exported Arrow buffers retain accounted ownership after session/provider drops.

mod support;
use pse_catalog::session::native_engine_profile;
use pse_ids::CancellationToken;
use std::sync::Arc;

#[tokio::test]
async fn detached_result_arrays_hold_their_actual_shared_pool_claim() {
    let directory = tempfile::tempdir().expect("store");
    let runtime = support::runtime(directory.path(), 64 << 20);
    let reg = support::registry();
    let catalog = support::catalog(
        Arc::new(
            object_store::local::LocalFileSystem::new_with_prefix(directory.path())
                .expect("backend"),
        ),
        Arc::clone(&reg),
        runtime.reserver(),
    );
    let cancel = CancellationToken::default();
    let snapshot = catalog
        .publish_bundle(support::draft(&catalog, 100, 0, &cancel), &cancel)
        .await
        .expect("snapshot");
    let factory = runtime
        .session_factory(native_engine_profile())
        .expect("factory");
    let session = factory
        .open_session(vec![snapshot], reg, &cancel)
        .expect("session");
    let result = session
        .sql("SELECT id, value FROM authored.samples", &cancel)
        .await
        .expect("bounded exported result");
    let array = Arc::clone(result[0].column(1));
    let buffer = array.to_data().buffers()[0].clone();
    drop(array);
    drop(result);
    drop(session);
    drop(factory);
    drop(catalog);
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
