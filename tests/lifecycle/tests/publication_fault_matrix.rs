// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Actual backend interruption cannot expose a mixed or partially published snapshot.

#[path = "../src/fault_store.rs"]
mod fault_store;
mod support;

use fault_store::{Fault, FaultPlan, FaultStore};
use object_store::local::LocalFileSystem;
use pse_catalog::store::membership::AdmissionContext;
use pse_catalog::{CatalogError, RefName};
use pse_ids::{CancellationToken, FixedBudget};
use std::sync::Arc;

#[tokio::test]
async fn interruption_at_relation_manifest_and_ref_keeps_the_exact_old_head() {
    for (prefix, cancel_after) in [
        ("relations/", false),
        ("manifests/", false),
        ("refs/", false),
        ("relations/", true),
        ("manifests/", true),
    ] {
        let directory = tempfile::tempdir().expect("local store");
        let store = FaultStore::new(Arc::new(
            LocalFileSystem::new_with_prefix(directory.path()).expect("backend"),
        ));
        let budget = FixedBudget::new(64 << 20);
        let catalog = support::catalog(store.clone(), support::registry(), budget.clone());
        let clean = CancellationToken::default();
        let name = RefName::parse("main").expect("name");
        let old = catalog
            .publish_bundle(support::draft(&catalog, 3, 0, &clean), &clean)
            .await
            .expect("old");
        catalog
            .compare_and_swap_ref(&name, None, &old, &clean)
            .await
            .expect("old ref");
        let observed = catalog
            .read_ref(&name, &clean)
            .await
            .expect("read")
            .expect("ref");
        let old_reference = old.manifest_ref();
        drop(old);
        let observed_baseline = budget.reserved();
        assert!(
            observed_baseline > 0,
            "observed ref retains its exact control bytes"
        );
        let cancel = CancellationToken::default();
        let candidate = support::draft(&catalog, 3, 99, &clean);
        store.arm(FaultPlan {
            operation: "put",
            prefix: prefix.to_owned(),
            call: 1,
            fault: if cancel_after {
                Fault::Cancel(cancel.clone())
            } else {
                Fault::FailBefore
            },
        });
        let result = match catalog.publish_bundle(candidate, &cancel).await {
            Ok(snapshot) => {
                catalog
                    .compare_and_swap_ref(&name, Some(&observed), &snapshot, &cancel)
                    .await
            }
            Err(error) => Err(error),
        };
        assert!(result.is_err(), "fault at {prefix}");
        assert_eq!(store.fired(), 1, "actual {prefix} operation occurred");
        assert_eq!(
            budget.reserved(),
            observed_baseline,
            "failure releases prepared and decoded objects"
        );
        let restored = catalog
            .read_snapshot(&name, &AdmissionContext::default(), &clean)
            .await
            .expect("old still admitted")
            .expect("old ref");
        assert_eq!(restored.manifest_ref(), old_reference);
        drop(restored);
        drop(observed);
        assert_eq!(budget.reserved(), 0);
    }
}

#[tokio::test]
async fn one_shot_cas_conflict_requires_an_explicit_ref_reread_and_retry() {
    let store = FaultStore::new(Arc::new(object_store::memory::InMemory::new()));
    let budget = FixedBudget::new(64 << 20);
    let catalog = support::catalog(store.clone(), support::registry(), budget.clone());
    let cancel = CancellationToken::default();
    let name = RefName::parse("main").expect("ref");
    let old = catalog
        .publish_bundle(support::draft(&catalog, 3, 0, &cancel), &cancel)
        .await
        .expect("old");
    catalog
        .compare_and_swap_ref(&name, None, &old, &cancel)
        .await
        .expect("old head");
    let observed = catalog
        .read_ref(&name, &cancel)
        .await
        .expect("read")
        .expect("ref");
    let next = catalog
        .publish_bundle(support::draft(&catalog, 3, 99, &cancel), &cancel)
        .await
        .expect("next");
    store.arm(FaultPlan {
        operation: "put",
        prefix: "refs/".to_owned(),
        call: 1,
        fault: Fault::Precondition,
    });
    assert!(matches!(
        catalog
            .compare_and_swap_ref(&name, Some(&observed), &next, &cancel)
            .await,
        Err(CatalogError::RefConflict { .. })
    ));
    let current = catalog
        .read_ref(&name, &cancel)
        .await
        .expect("read")
        .expect("ref");
    assert_eq!(current.manifest_ref(), old.manifest_ref());
    catalog
        .compare_and_swap_ref(&name, Some(&current), &next, &cancel)
        .await
        .expect("explicit retry");
    assert_eq!(
        catalog
            .read_ref(&name, &cancel)
            .await
            .expect("read")
            .expect("ref")
            .manifest_ref(),
        next.manifest_ref()
    );
    drop(next);
    drop(old);
    drop(current);
    drop(observed);
    drop(catalog);
    drop(store);
    assert_eq!(budget.reserved(), 0);
}

#[tokio::test]
async fn corrupted_existing_bytes_under_correct_names_fail_restore_and_reuse() {
    for fault in [Fault::Truncate, Fault::FlipByte] {
        let directory = tempfile::tempdir().expect("local store");
        let store = FaultStore::new(Arc::new(
            LocalFileSystem::new_with_prefix(directory.path()).expect("backend"),
        ));
        let budget = FixedBudget::new(64 << 20);
        let catalog = support::catalog(store.clone(), support::registry(), budget.clone());
        let cancel = CancellationToken::default();
        let old = catalog
            .publish_bundle(support::draft(&catalog, 3, 0, &cancel), &cancel)
            .await
            .expect("old");
        let reference = old.manifest_ref();
        drop(old);
        store.arm(FaultPlan {
            operation: "get",
            prefix: "relations/".to_owned(),
            call: 1,
            fault,
        });
        assert!(
            catalog
                .read_manifest(reference, &AdmissionContext::default(), &cancel)
                .await
                .is_err()
        );
        assert_eq!(store.fired(), 1);
        assert_eq!(budget.reserved(), 0);
        assert!(
            catalog
                .publish_bundle(support::draft(&catalog, 3, 0, &cancel), &cancel)
                .await
                .is_err(),
            "AlreadyExists rereads exact actual bytes"
        );
        assert_eq!(budget.reserved(), 0);
    }
}
