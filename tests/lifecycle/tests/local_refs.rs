// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Independently opened local catalogs coordinate exact conditional publication.

#![cfg(unix)]
#![allow(
    clippy::expect_used,
    reason = "fixed local publication integration fixtures"
)]

mod support;

use pse_catalog::store::membership::AdmissionContext;
use pse_catalog::{Catalog, CatalogError, FixedClock, RefName, TrustLevel};
use pse_ids::{CancellationToken, FixedBudget};
use std::sync::Arc;

fn open(directory: &std::path::Path, budget: &Arc<FixedBudget>) -> Catalog {
    Catalog::open_local(
        directory,
        support::registry(),
        TrustLevel::Untrusted,
        Arc::new(FixedClock("2026-09-14T00:00:00Z".to_owned())),
        budget.clone(),
    )
    .expect("local catalog")
}

#[tokio::test]
async fn independently_opened_local_catalogs_race_and_reject_stale_exact_values() {
    let directory = tempfile::tempdir().expect("directory");
    let budget = FixedBudget::new(64 << 20);
    let left = open(directory.path(), &budget);
    let right = open(directory.path(), &budget);
    let cancel = CancellationToken::default();
    let name = RefName::parse("main").expect("ref");
    let old = left
        .publish_bundle(support::draft(&left, 3, 0, &cancel), &cancel)
        .await
        .expect("old");
    left.compare_and_swap_ref(&name, None, &old, &cancel)
        .await
        .expect("initial ref");
    // A crashed writer can leave a finished, unrenamed sibling. It is never a ref.
    std::fs::write(
        directory.path().join("refs/.pse-control-1-1.pending"),
        b"partial candidate",
    )
    .expect("abandoned temporary");
    assert_eq!(
        left.list_refs(&cancel).await.expect("list actual refs"),
        vec![name.clone()]
    );
    let expected_left = left
        .read_ref(&name, &cancel)
        .await
        .expect("read")
        .expect("left ref");
    let expected_right = right
        .read_ref(&name, &cancel)
        .await
        .expect("read")
        .expect("right ref");
    let next_left = left
        .publish_bundle(support::draft(&left, 3, 10, &cancel), &cancel)
        .await
        .expect("left");
    let next_right = right
        .publish_bundle(support::draft(&right, 3, 20, &cancel), &cancel)
        .await
        .expect("right");
    let (first, second) = tokio::join!(
        left.compare_and_swap_ref(&name, Some(&expected_left), &next_left, &cancel),
        right.compare_and_swap_ref(&name, Some(&expected_right), &next_right, &cancel),
    );
    assert_ne!(
        first.is_ok(),
        second.is_ok(),
        "exactly one conditional update commits"
    );
    let expected_winner = if first.is_ok() {
        next_left.manifest_ref()
    } else {
        next_right.manifest_ref()
    };
    let loser = if first.is_err() { first } else { second };
    assert!(matches!(loser, Err(CatalogError::RefConflict { .. })));
    assert!(matches!(
        left.compare_and_swap_ref(&name, Some(&expected_left), &old, &cancel)
            .await,
        Err(CatalogError::RefConflict { .. })
    ));
    let visible = left
        .read_snapshot(&name, &AdmissionContext::default(), &cancel)
        .await
        .expect("admit")
        .expect("ref");
    assert_eq!(visible.manifest_ref(), expected_winner);
    let current = right
        .read_ref(&name, &cancel)
        .await
        .expect("read")
        .expect("ref");
    right
        .compare_and_swap_ref(&name, Some(&current), &next_right, &cancel)
        .await
        .expect("explicit reread retry");
    drop((
        visible,
        current,
        expected_left,
        expected_right,
        old,
        next_left,
        next_right,
    ));
    drop((left, right));
    assert_eq!(
        budget.reserved(),
        0,
        "all snapshots and observed control owners released"
    );
}

#[tokio::test]
async fn local_lock_wait_is_cancellable_and_never_overwrites_the_old_ref() {
    let directory = tempfile::tempdir().expect("directory");
    let budget = FixedBudget::new(64 << 20);
    let catalog = Arc::new(open(directory.path(), &budget));
    let clean = CancellationToken::default();
    let name = RefName::parse("main").expect("ref");
    let old = catalog
        .publish_bundle(support::draft(&catalog, 3, 0, &clean), &clean)
        .await
        .expect("old");
    catalog
        .compare_and_swap_ref(&name, None, &old, &clean)
        .await
        .expect("initial");
    let observed = catalog
        .read_ref(&name, &clean)
        .await
        .expect("read")
        .expect("ref");
    let next = catalog
        .publish_bundle(support::draft(&catalog, 3, 99, &clean), &clean)
        .await
        .expect("next");
    let lock = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(directory.path().join(".pse-control.lock"))
        .expect("stable lock");
    lock.try_lock().expect("exclusive fixture lock");
    let cancel = CancellationToken::default();
    let (result, ()) = tokio::join!(
        catalog.compare_and_swap_ref(&name, Some(&observed), &next, &cancel),
        async {
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
            cancel.cancel();
        },
    );
    assert!(matches!(
        result,
        Err(CatalogError::Canon(pse_ids::CanonError::Cancelled) | CatalogError::Cancelled)
    ));
    drop(lock);
    let restored = catalog
        .read_snapshot(&name, &AdmissionContext::default(), &clean)
        .await
        .expect("admit")
        .expect("ref");
    assert_eq!(restored.manifest_ref(), old.manifest_ref());
    drop((restored, observed, next, old, catalog));
    assert_eq!(budget.reserved(), 0);
}

#[tokio::test]
async fn equal_local_etag_does_not_replace_actual_control_byte_comparison() {
    use object_store::ObjectStoreExt;
    let directory = tempfile::tempdir().expect("directory");
    let budget = FixedBudget::new(64 << 20);
    let catalog = open(directory.path(), &budget);
    let cancel = CancellationToken::default();
    let name = RefName::parse("main").expect("ref");
    let snapshot = catalog
        .publish_bundle(support::draft(&catalog, 3, 0, &cancel), &cancel)
        .await
        .expect("snapshot");
    catalog
        .compare_and_swap_ref(&name, None, &snapshot, &cancel)
        .await
        .expect("initial ref");
    let observed = catalog
        .read_ref(&name, &cancel)
        .await
        .expect("read")
        .expect("ref");
    let backend =
        object_store::local::LocalFileSystem::new_with_prefix(directory.path()).expect("backend");
    let key = object_store::path::Path::from("refs/main.json");
    let before = backend.head(&key).await.expect("old metadata");
    let path = directory.path().join("refs/main.json");
    let modified = std::fs::metadata(&path)
        .expect("metadata")
        .modified()
        .expect("modified");
    let old = std::fs::read_to_string(&path).expect("actual ref");
    let changed = old.replace("2026-09-14T00:00:00Z", "2026-09-15T00:00:00Z");
    assert_ne!(old, changed);
    assert_eq!(old.len(), changed.len());
    std::fs::write(&path, &changed).expect("same-length actual bytes changed");
    std::fs::File::options()
        .write(true)
        .open(&path)
        .expect("file")
        .set_modified(modified)
        .expect("restore observed modification metadata");
    let after = backend.head(&key).await.expect("new metadata");
    assert_eq!(
        before.e_tag, after.e_tag,
        "pinned local ETag metadata collides"
    );
    assert!(matches!(
        catalog
            .compare_and_swap_ref(&name, Some(&observed), &snapshot, &cancel)
            .await,
        Err(CatalogError::RefConflict { .. })
    ));
    assert_eq!(
        std::fs::read_to_string(&path).expect("still changed"),
        changed
    );
    drop((observed, snapshot, catalog));
    assert_eq!(budget.reserved(), 0);
}
