// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Actual backend interruption cannot expose a mixed or partially published snapshot.

#[path = "../src/fault_store.rs"]
mod fault_store;
mod support;

use datafusion::arrow::array::RecordBatch;
use fault_store::{Fault, FaultPlan, FaultStore};
use object_store::local::LocalFileSystem;
use pse_catalog::store::membership::AdmissionContext;
use pse_catalog::{CatalogError, RefName};
use pse_ids::{CancellationToken, FixedBudget};
use std::sync::Arc;

#[derive(Debug)]
struct ScopedValidator {
    invariants: pse_rules::validator::InvariantValidator,
    ceiling: usize,
    calls: std::sync::atomic::AtomicUsize,
}
impl pse_catalog::store::membership::SemanticValidator for ScopedValidator {
    fn validate<'a>(
        &'a self,
        registry: &'a pse_schema::Registry,
        rows: &'a std::collections::BTreeMap<pse_schema::model::RelationKey, RecordBatch>,
        session: &'a pse_catalog::session::SnapshotSession,
        cancel: &'a CancellationToken,
    ) -> pse_catalog::BoxFut<'a, Result<(), CatalogError>> {
        Box::pin(async move {
            let mut reservation = session.reserver().open("validator:scoped-probe");
            assert!(
                reservation.try_grow(self.ceiling + 1).is_err(),
                "validation must not allocate through its parent factory"
            );
            self.calls
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            self.invariants
                .validate(registry, rows, session, cancel)
                .await
        })
    }
}

#[tokio::test]
async fn bundle_validation_uses_the_actual_publication_resource_scope() {
    use pse_schema::model::provider::{ProviderPolicy, ProviderScope};
    let budget = FixedBudget::new(64 << 20);
    let registry = support::registry();
    let mut policy = ProviderPolicy::new(
        pse_ids::SemanticId::from_bytes([71; 16]),
        ProviderScope::Schema("store".into(), "manifests".into()),
    );
    policy.max_bytes = Some(20 << 20);
    let sessions = support::session_factory::factory(budget.clone());
    let sessions = Arc::new(
        Arc::try_unwrap(sessions)
            .expect("fixture owns the factory")
            .with_policies(vec![policy])
            .expect("scoped factory"),
    );
    let validator = Arc::new(ScopedValidator {
        invariants: pse_rules::validator::InvariantValidator::new(Arc::clone(&registry)),
        ceiling: 20 << 20,
        calls: std::sync::atomic::AtomicUsize::new(0),
    });
    let catalog = pse_catalog::Catalog::open(
        Arc::new(object_store::memory::InMemory::new()),
        registry,
        pse_catalog::TrustLevel::Untrusted,
        Arc::new(pse_catalog::FixedClock("2026-09-15T00:00:00Z".into())),
        sessions,
    )
    .with_semantic_validator(validator.clone());
    let cancel = CancellationToken::new();
    let snapshot = catalog
        .publish_bundle(support::draft(&catalog, 3, 0, &cancel), &cancel)
        .await
        .expect("valid scoped publication");
    assert_eq!(
        validator.calls.load(std::sync::atomic::Ordering::Relaxed),
        1
    );
    drop((snapshot, catalog, validator));
    assert_eq!(budget.reserved(), 0);
}

#[tokio::test]
async fn interruption_at_relation_manifest_and_ref_keeps_the_exact_old_head() {
    for (prefix, cancel_after) in [
        ("relations/", false),
        ("manifests/", false),
        ("refs/", false),
        ("relations/", true),
        ("manifests/", true),
    ] {
        let store = FaultStore::new(Arc::new(object_store::memory::InMemory::new()));
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
        let restored = catalog
            .read_snapshot(&name, &AdmissionContext::default(), &clean)
            .await
            .expect("old still admitted")
            .expect("old ref");
        assert_eq!(restored.manifest_ref(), old_reference);
        drop(restored);
        drop(observed);
        // This backend retains published Bytes and their real reservations in memory.
        // Release the backend too before asserting the complete ownership boundary.
        drop((result, catalog, store));
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

#[tokio::test]
async fn native_ref_publication_preserves_committed_success_after_cancellation() {
    use pse_catalog::store::publication::Visibility;
    let store = FaultStore::new(Arc::new(object_store::memory::InMemory::new()));
    let budget = FixedBudget::new(64 << 20);
    let catalog = support::catalog(store.clone(), support::registry(), budget.clone());
    let clean = CancellationToken::new();
    let target = catalog
        .publish_bundle(support::draft(&catalog, 3, 0, &clean), &clean)
        .await
        .expect("target");
    let cancel = CancellationToken::new();
    let name = RefName::parse("main").expect("name");
    store.arm(FaultPlan {
        operation: "put",
        prefix: "refs/".into(),
        call: 1,
        fault: Fault::Cancel(cancel.clone()),
    });
    let prepared = catalog
        .prepare_ref_update(&name, None, &target, None, &cancel)
        .expect("prepared");
    let journal = prepared.publication_journal();
    let duplicate = prepared.computation().clone();
    assert!(journal.outcomes().expect("journal").is_empty());
    assert_eq!(store.fired(), 0, "preparation does not publish");
    let completed = prepared
        .execute(&cancel)
        .await
        .expect("actual commit remains successful");
    assert!(cancel.is_cancelled());
    assert_eq!(
        completed
            .computation()
            .batches()
            .iter()
            .map(|batch| batch.num_rows())
            .sum::<usize>(),
        1
    );
    let outcomes = journal.outcomes().expect("outcomes");
    assert_eq!(outcomes.len(), 1);
    assert_eq!(outcomes[0].visibility, Visibility::Visible);
    assert!(outcomes[0].durability_confirmed);
    assert!(duplicate.execute(&clean).await.is_err());
    assert_eq!(store.put_trace().len(), 1);
    let state = catalog
        .read_ref(&name, &clean)
        .await
        .expect("read")
        .expect("published");
    assert_eq!(state.manifest_ref(), target.manifest_ref());
    drop((completed, state, target, catalog, store, journal));
    assert_eq!(budget.reserved(), 0);
}

#[tokio::test]
async fn lost_ref_response_retains_exact_uncertain_outcome_without_automatic_retry() {
    use pse_catalog::store::publication::Visibility;
    let store = FaultStore::new(Arc::new(object_store::memory::InMemory::new()));
    let budget = FixedBudget::new(64 << 20);
    let catalog = support::catalog(store.clone(), support::registry(), budget.clone());
    let cancel = CancellationToken::new();
    let target = catalog
        .publish_bundle(support::draft(&catalog, 3, 0, &cancel), &cancel)
        .await
        .expect("target");
    let name = RefName::parse("main").expect("name");
    store.arm(FaultPlan {
        operation: "put",
        prefix: "refs/".into(),
        call: 1,
        fault: Fault::LostResponse,
    });
    let prepared = catalog
        .prepare_ref_update(&name, None, &target, None, &cancel)
        .expect("prepared");
    let journal = prepared.publication_journal();
    let error = prepared
        .execute(&cancel)
        .await
        .expect_err("response was lost");
    let CatalogError::Publication { outcomes, .. } = error else {
        panic!("missing publication evidence: {error:?}")
    };
    assert_eq!(outcomes, journal.outcomes().expect("journal"));
    assert_eq!(outcomes[0].visibility, Visibility::Indeterminate);
    assert!(!outcomes[0].durability_confirmed);
    assert_eq!(store.put_trace().len(), 1, "no automatic duplicate write");
    let state = catalog
        .read_ref(&name, &cancel)
        .await
        .expect("reconcile")
        .expect("actual commit");
    assert_eq!(state.manifest_ref(), target.manifest_ref());
    drop((state, target, catalog, store, journal));
    assert_eq!(budget.reserved(), 0);
}
