// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
use super::*;
use object_store::{ObjectStoreExt, memory::InMemory};

#[tokio::test]
async fn native_ranges_preserve_exact_bytes_and_expose_continuation() {
    let store = InMemory::new();
    let key = Path::from("retained/result.json");
    store
        .put(
            &key,
            bytes::Bytes::from_static(b"{\"n\":9007199254740993,\"n\":2}\r\n").into(),
        )
        .await
        .unwrap();
    let first = read(
        &store,
        &key,
        &ReadRequest {
            length: NonZeroU64::new(7),
            ..ReadRequest::default()
        },
    )
    .await
    .unwrap();
    assert!(!first.whole_object);
    assert_eq!(first.byte_range, 0..7);
    assert_eq!(first.next_offset, Some(7));
    let rest = read(
        &store,
        &key,
        &ReadRequest {
            offset: 7,
            length: None,
            expected_etag: first.etag,
        },
    )
    .await
    .unwrap();
    assert_eq!(rest.byte_range.end, rest.object_bytes);
    assert_eq!(rest.next_offset, None);
    let whole = read(
        &store,
        &key,
        &ReadRequest {
            length: None,
            ..ReadRequest::default()
        },
    )
    .await
    .unwrap();
    let Content::Utf8(text) = whole.content else {
        panic!("UTF-8")
    };
    assert_eq!(text, "{\"n\":9007199254740993,\"n\":2}\r\n");
    assert!(whole.whole_object);
}

#[tokio::test]
async fn changed_object_refuses_a_prior_validator() {
    let store = InMemory::new();
    let key = Path::from("evidence");
    store.put(&key, "before".into()).await.unwrap();
    let before = read(&store, &key, &ReadRequest::default()).await.unwrap();
    store.put(&key, "after".into()).await.unwrap();
    assert!(matches!(
        read(
            &store,
            &key,
            &ReadRequest {
                expected_etag: before.etag,
                ..ReadRequest::default()
            }
        )
        .await,
        Err(ArtifactError::Store(
            object_store::Error::Precondition { .. }
        ))
    ));
}

#[tokio::test]
async fn binary_and_empty_objects_have_exact_typed_content() {
    let store = InMemory::new();
    let key = Path::from("binary");
    store
        .put(&key, bytes::Bytes::from_static(&[0, 255, 7]).into())
        .await
        .unwrap();
    let readback = read(&store, &key, &ReadRequest::default()).await.unwrap();
    let Content::Base64(encoded) = &readback.content else {
        panic!("binary")
    };
    assert_eq!(STANDARD.decode(encoded).unwrap(), [0, 255, 7]);
    let schema = serde_json::to_value(schemars::schema_for!(ArtifactRead)).unwrap();
    jsonschema::validator_for(&schema)
        .unwrap()
        .validate(&serde_json::to_value(&readback).unwrap())
        .unwrap();
    store.put(&key, bytes::Bytes::new().into()).await.unwrap();
    let empty = read(&store, &key, &ReadRequest::default()).await.unwrap();
    assert!(empty.whole_object);
    assert_eq!(empty.byte_range, 0..0);
}

#[tokio::test]
async fn local_file_reads_do_not_create_namespaces_or_mutate_source() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("native-café");
    std::fs::write(&path, b"first\nsecond\n").unwrap();
    let before = std::fs::metadata(&path).unwrap().modified().unwrap();
    let found = read_file(
        &path,
        &ReadRequest {
            offset: 6,
            length: NonZeroU64::new(7),
            expected_etag: None,
        },
    )
    .await
    .unwrap();
    assert_eq!(found.byte_range, 6..13);
    assert_eq!(std::fs::read(&path).unwrap(), b"first\nsecond\n");
    assert_eq!(
        std::fs::metadata(&path).unwrap().modified().unwrap(),
        before
    );
    assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 1);
    assert!(!found.atomic_snapshot);
}

#[tokio::test]
async fn unsupported_native_object_key_is_refused_without_lossy_substitution() {
    use std::os::unix::ffi::OsStringExt as _;
    let directory = tempfile::tempdir().unwrap();
    let path = directory
        .path()
        .join(std::ffi::OsString::from_vec(b"native-\xff".to_vec()));
    std::fs::write(&path, b"exact").unwrap();
    assert!(matches!(
        read_file(&path, &ReadRequest::default()).await,
        Err(ArtifactError::Path(
            object_store::path::Error::NonUnicode { .. }
        ))
    ));
    assert_eq!(std::fs::read(&path).unwrap(), b"exact");
}

#[tokio::test]
async fn invalid_extent_is_a_failure_and_eof_is_explicit() {
    let store = InMemory::new();
    let key = Path::from("one");
    store.put(&key, "x".into()).await.unwrap();
    assert!(matches!(
        read(
            &store,
            &key,
            &ReadRequest {
                offset: 2,
                ..ReadRequest::default()
            }
        )
        .await,
        Err(ArtifactError::Range(_))
    ));
    assert!(matches!(
        read(
            &store,
            &key,
            &ReadRequest {
                offset: u64::MAX,
                length: NonZeroU64::new(1),
                expected_etag: None
            }
        )
        .await,
        Err(ArtifactError::Range(_))
    ));
    let at_eof = read(
        &store,
        &key,
        &ReadRequest {
            offset: 1,
            ..ReadRequest::default()
        },
    )
    .await
    .unwrap();
    assert_eq!(at_eof.byte_range, 1..1);
    assert_eq!(at_eof.next_offset, None);
    assert!(!at_eof.whole_object);
}
