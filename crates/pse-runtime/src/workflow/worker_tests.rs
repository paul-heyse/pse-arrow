// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Exact bounded canonical source ingress used by native study workers.
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
    let reopened = operations.sources(&receipt).await.unwrap();
    assert_eq!(&**reopened, &documents);
    assert!(runtime.shared.pool().reserved() > before);
    drop(reopened);
    assert_eq!(runtime.shared.pool().reserved(), before);
    let mut wrong = receipt;
    wrong.identity = pse_ids::ContentHash::from_bytes([0; 32]);
    assert!(operations.sources(&wrong).await.is_err());
}
