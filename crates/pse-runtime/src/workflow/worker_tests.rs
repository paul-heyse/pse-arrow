// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Exact bounded canonical source ingress used by native study workers.
#![allow(
    clippy::unreachable,
    reason = "canonical document worker fixtures require durable operation ownership"
)]
#[cfg(feature = "canonical-tests")]
use super::*;
#[cfg(feature = "canonical-tests")]
use std::collections::BTreeMap;

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

