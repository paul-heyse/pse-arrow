// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Exact bounded canonical source ingress used by native study workers.
#![allow(
    clippy::unreachable,
    reason = "canonical document worker fixtures require durable operation ownership"
)]
#[cfg(feature = "canonical-tests")]
use super::*;
use std::{collections::BTreeMap, path::Path};

fn texts(root: &Path) -> BTreeMap<String, Vec<u8>> {
    let mut texts = BTreeMap::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(dir) = pending.pop() {
        for entry in std::fs::read_dir(&dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                pending.push(path);
            } else {
                let key = path
                    .strip_prefix(root)
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .replace('\\', "/");
                texts.insert(key, std::fs::read(&path).unwrap());
            }
        }
    }
    texts
}

/// A manifest dependency on the physical primitives fixture package.
pub(super) const PRIMITIVES: &str = r#"dependencies = [{ package_id = "5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a", version_req = { operator = "exact", major = 1, minor = 0, patch = 0 } }]"#;
/// The authored sources of a one-model package over the physical primitives fixture.
pub(super) fn sources(source: &str) -> (BTreeMap<String, Vec<u8>>, BTreeMap<String, Vec<u8>>) {
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/packages");
    let physical = texts(&fixtures.join("physical-primitives"));
    let manifest = std::fs::read_to_string(fixtures.join("minimal_explicit/package.toml"))
        .unwrap()
        .replace(r#"id_policy = "explicit""#, r#"id_policy = "named""#)
        // The primitives declare `Scalar`; depending on them makes it visible (ADR-0123
        // Outcome 6).
        .replace("dependencies = []", PRIMITIVES);
    let modeling = BTreeMap::from([
        ("package.toml".to_owned(), manifest.into_bytes()),
        ("models/root.pse".to_owned(), source.as_bytes().to_vec()),
    ]);
    (physical, modeling)
}

#[cfg(feature = "canonical-tests")]
#[tokio::test]
async fn canonical_document_sources_chunked_exact_and_kind_checked() {
    let runtime = durable_tests::durable_runtime();
    let Durability::Durable(operations) = runtime.durability() else {
        unreachable!()
    };
    let documents = BTreeMap::from([
        ("large/document.pse".into(), vec![42_u8; 1_100_019]),
        ("empty.pse".into(), Vec::new()),
        ("bits.pse".into(), vec![0, 255, 13, 10]),
    ]);
    let before = runtime.shared.pool().reserved();
    let receipt = operations.put_sources(&documents).await.unwrap();
    let receipt_again = operations.put_sources(&documents).await.unwrap();
    assert_eq!(receipt.revision, receipt_again.revision);
    let revision = operations
        .store()
        .revision(&receipt.revision)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        revision.sequence, 1,
        "all chunks and the final manifest share one coherent source publication"
    );
    assert!(revision.parent.is_none());
    let reopened = operations.sources(&receipt).await.unwrap();
    assert_eq!(&**reopened, &documents);
    assert!(runtime.shared.pool().reserved() > before);
    drop(reopened);
    assert_eq!(runtime.shared.pool().reserved(), before);
    let mut wrong = receipt;
    wrong.identity = pse_ids::ContentHash::from_bytes([0; 32]);
    assert!(operations.sources(&wrong).await.is_err());
}

#[cfg(feature = "canonical-tests")]
#[tokio::test]
async fn canonical_cancelled_dependent_refreshes_rejected_predecessor_premise() {
    use pse_model::study::{
        ActionKind, Dependency, OccurrenceKey, PointPolicy, SeedNeed, StartPolicy,
    };
    use pse_operations::{
        canonical_execution::RunRequest,
        canonical_studies::{NewOccurrence, point_key},
    };
    let runtime = durable_tests::durable_runtime();
    let store = runtime.canonical_store();
    let revision = store
        .edit(
            "cancel-premise",
            None,
            "effect-free cancellation fixture",
            &[],
        )
        .await
        .unwrap();
    let request = |key: &str| RunRequest {
        key: key.into(),
        revision: revision.clone(),
        sources: vec![],
        request: vec![1],
        source_selection: vec![2],
        attestation: vec![3],
    };
    let points = [0, 1].map(|ordinal| NewOccurrence {
        policy: PointPolicy {
            key: OccurrenceKey(ordinal),
            dependencies: if ordinal == 0 {
                vec![]
            } else {
                vec![Dependency::Ordering(OccurrenceKey(0))]
            },
            start: StartPolicy::Fresh,
            seed_need: SeedNeed::NotNeeded,
            attempt_limit: 1,
        },
        descriptor: vec![42],
        run: request(&format!("cancel-premise-point-{ordinal}")),
    });
    store
        .create_study(
            "cancel-premise",
            &request("cancel-premise-summary"),
            &[9],
            &points,
            &|| false,
        )
        .await
        .unwrap();
    store.cancel_study("cancel-premise").await.unwrap();
    // Two independent lanes can snapshot a cancelled dependency before either
    // finishes. Advance the predecessor after capturing the dependent's scope.
    let dependent_key = point_key("cancel-premise", OccurrenceKey(1));
    let stale = store.study_scope(&dependent_key).await.unwrap();
    let stale_action = stale.action(None).unwrap();
    assert!(matches!(stale_action.kind, ActionKind::Cancel));
    let predecessor = store
        .study_scope(&point_key("cancel-premise", OccurrenceKey(0)))
        .await
        .unwrap();
    let settled = runtime
        .settle_current_study_candidate(&predecessor, &predecessor.action(None).unwrap())
        .await
        .unwrap()
        .unwrap();
    assert!(settled.settled && settled.attempt.is_none());
    assert!(
        runtime
            .settle_current_study_candidate(&stale, &stale_action)
            .await
            .unwrap()
            .is_none()
    );
    let fresh = store.study_scope(&dependent_key).await.unwrap();
    assert_eq!(
        fresh.point().revision,
        stale.point().revision,
        "rejection must not mutate the dependent"
    );
    assert_eq!(
        fresh.predecessors()[0].revision,
        stale.predecessors()[0].revision + 1
    );
    let action = fresh.action(None).unwrap();
    assert!(matches!(action.kind, ActionKind::Cancel));
    let settled = runtime
        .settle_current_study_candidate(&fresh, &action)
        .await
        .unwrap()
        .unwrap();
    assert!(settled.settled && settled.attempt.is_none());
    assert_eq!(settled.revision, stale.point().revision + 1);
}
