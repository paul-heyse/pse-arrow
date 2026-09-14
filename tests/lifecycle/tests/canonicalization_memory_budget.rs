// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Canonical preflight and publication refuse insufficient budgets before visibility.

mod support;
use pse_ids::{CancellationToken, CanonicalizeOptions, FixedBudget};
use std::sync::Arc;

#[tokio::test]
async fn canonicalization_and_publication_fail_cleanly_under_finite_or_cancelled_budgets() {
    let directory = tempfile::tempdir().expect("local store");
    let store = Arc::new(
        object_store::local::LocalFileSystem::new_with_prefix(directory.path()).expect("backend"),
    );
    let reg = support::registry();
    let source_budget = FixedBudget::new(64 << 20);
    let producer = support::catalog(store.clone(), Arc::clone(&reg), source_budget.clone());
    let cancel = CancellationToken::default();
    let draft = support::draft(&producer, 2000, 0, &cancel);
    let relation = draft.relations.values().next().expect("complete relation");
    let small = FixedBudget::new(64 << 10);
    assert!(
        pse_ids::canonicalize(
            &relation.contract.canonical,
            &relation.batches,
            small.as_ref(),
            CanonicalizeOptions::default()
        )
        .is_err()
    );
    assert_eq!(small.reserved(), 0);
    let catalog = support::catalog(store, reg, small.clone());
    assert!(
        catalog
            .publish_bundle(draft.clone(), &cancel)
            .await
            .is_err()
    );
    assert!(
        catalog
            .list_refs(&cancel)
            .await
            .expect("no refs")
            .is_empty()
    );
    assert_eq!(small.reserved(), 0);
    let cancelled = CancellationToken::default();
    cancelled.cancel();
    assert!(catalog.publish_bundle(draft, &cancelled).await.is_err());
    assert_eq!(small.reserved(), 0);
    assert_eq!(source_budget.reserved(), 0);
}
