// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Independent value cases across the source-derived Proto and native storage projections.

use buffa::Message;
use buffa_descriptor::{DescriptorPool, DynamicMessage, Value};
use pse_model::generated::{
    enums::{CanonicalAttemptPhase, CanonicalTerminalOutcome},
    runtime::{canonical_attempts, canonical_result_blocks, canonical_result_manifests},
};
use pse_operations::{canonical_codec as codec, generated::surreal as storage};
use std::sync::Arc;

use crate::engineering_projection as projection;
use crate::host_operations::engineering::generated::{DESCRIPTORS, pb::pse::engineering::v1 as pb};

// Buffa 0.9.2 links the published closure, then decodes by descriptor without the
// generated message type. This is a separate read path with literal expected tags.
// https://github.com/anthropics/buffa/blob/v0.9.2/buffa-descriptor/src/reflect/dynamic.rs
fn inspect(name: &str, bytes: &[u8]) -> DynamicMessage {
    let pool = Arc::new(DescriptorPool::decode(DESCRIPTORS).unwrap());
    let message = pool.message_index(name).unwrap();
    DynamicMessage::decode(pool, message, bytes).unwrap()
}

#[test]
fn result_manifest_projections_preserve_unsigned_extent_opaque_bytes_and_declared_text() {
    let original = canonical_result_manifests::Row {
        key: "record:declared-text".into(),
        attempt: "attempt:original".into(),
        generation: u64::MAX,
        digest: "a".repeat(64),
        descriptors: vec![0, 255, 1, 128, 0].into(),
    };
    let bytes = projection::encode_canonical_result_manifest(original.clone())
        .unwrap()
        .encode_to_vec();
    let independent = inspect("pse.engineering.v1.CanonicalResultManifest", &bytes);
    assert_eq!(
        independent.field_by_number(1),
        Some(&Value::String("record:declared-text".into()))
    );
    assert_eq!(independent.field_by_number(3), Some(&Value::U64(u64::MAX)));
    assert_eq!(
        independent.field_by_number(5),
        Some(&Value::Bytes(vec![0, 255, 1, 128, 0]))
    );
    let restored = projection::decode_canonical_result_manifest(
        pb::CanonicalResultManifest::decode_from_slice(&bytes).unwrap(),
    )
    .unwrap();
    assert_eq!(restored, original);
    let native = storage::encode_canonical_result_manifests(&restored).unwrap();
    let mut fields = native.clone();
    assert_eq!(
        codec::decode_string(codec::required(&mut fields, "key").unwrap()).unwrap(),
        "record:declared-text"
    );
    assert_eq!(
        codec::decode_uint(codec::required(&mut fields, "generation").unwrap()).unwrap(),
        u64::MAX
    );
    assert_eq!(
        storage::decode_canonical_result_manifests(native).unwrap(),
        original
    );
}

fn block() -> canonical_result_blocks::Row {
    canonical_result_blocks::Row {
        key: "block:original".into(),
        result_set: "set:original".into(),
        batch: "batch:original".into(),
        output: "temperature".into(),
        partition: "step:3".into(),
        ordinal: u64::MAX,
        start: 0,
        end: 1,
        rows: 1,
        columns: 1,
        coordinate_min: Some(-0.0),
        coordinate_max: None,
        payload_bytes: u64::MAX,
        payload_digest: "b".repeat(64),
        statistics: Some(Vec::new().into()),
        interpretation: "source-owned-fixture.v1".into(),
    }
}

#[test]
fn result_block_projections_keep_signed_zero_presence_and_empty_binary_distinct() {
    for statistics in [None, Some(Vec::new().into()), Some(vec![0, 255].into())] {
        let original = canonical_result_blocks::Row {
            statistics,
            ..block()
        };
        let bytes = projection::encode_canonical_result_block_metadata(original.clone())
            .unwrap()
            .encode_to_vec();
        let independent = inspect("pse.engineering.v1.CanonicalResultBlockMetadata", &bytes);
        assert_eq!(independent.field_by_number(6), Some(&Value::U64(u64::MAX)));
        assert_eq!(
            independent.field_by_number(11),
            Some(&Value::U64((-0.0f64).to_bits()))
        );
        assert_eq!(independent.field_by_number(12), None);
        assert_eq!(independent.field_by_number(13), Some(&Value::U64(u64::MAX)));
        assert_eq!(
            independent.field_by_number(15),
            original
                .statistics
                .as_ref()
                .map(|bytes| Value::Bytes(bytes.as_slice().to_vec()))
                .as_ref()
        );
        let restored = projection::decode_canonical_result_block_metadata(
            pb::CanonicalResultBlockMetadata::decode_from_slice(&bytes).unwrap(),
        )
        .unwrap();
        assert_eq!(restored, original);
        let native = storage::encode_canonical_result_blocks(&restored).unwrap();
        assert_eq!(
            storage::decode_canonical_result_blocks(native).unwrap(),
            original
        );
    }
}

#[test]
fn readable_result_bits_and_future_wire_fields_do_not_grant_native_storage_adoption() {
    let bits = 0x7ff8_0000_0000_1234;
    let raw = canonical_result_blocks::Row {
        coordinate_min: Some(f64::from_bits(bits)),
        ..block()
    };
    let mut bytes = projection::encode_canonical_result_block_metadata(raw)
        .unwrap()
        .encode_to_vec();
    let independent = inspect("pse.engineering.v1.CanonicalResultBlockMetadata", &bytes);
    assert_eq!(independent.field_by_number(11), Some(&Value::U64(bits)));
    let wire = pb::CanonicalResultBlockMetadata::decode_from_slice(&bytes).unwrap();
    let restored = projection::decode_canonical_result_block_metadata(wire).unwrap();
    assert_eq!(restored.coordinate_min.unwrap().to_bits(), bits);
    assert!(
        storage::encode_canonical_result_blocks(&restored).is_err(),
        "native finite-coordinate policy remains a separate admission"
    );
    bytes.extend_from_slice(&[0xa0, 0x06, 0x01]); // Future tag100, varint1.
    let future = pb::CanonicalResultBlockMetadata::decode_from_slice(&bytes).unwrap();
    assert!(!buffa_descriptor::ReflectMessage::unknown_fields(&future).is_empty());
    assert!(projection::decode_canonical_result_block_metadata(future).is_err());
}

#[test]
fn original_attempt_metadata_preserves_signed_time_and_recorded_outcome_without_issuing_authority()
{
    let original = canonical_attempts::Row {
        key: "original:attempt".into(),
        run: "original:run".into(),
        generation: u64::MAX,
        claim_operation: "original:claim".into(),
        request: vec![0, 255].into(),
        worker: "original:worker".into(),
        expires_at: -1,
        phase: CanonicalAttemptPhase::Terminal,
        close_generation: Some(u64::MAX),
        outcome: Some(CanonicalTerminalOutcome::Partial),
        closed_manifest: Some("original:manifest".into()),
        completion: Some(Vec::new().into()),
        seed_manifest: None,
        finalization_study: None,
    };
    // The protected snapshot contains this native row. Data conversion cannot
    // create ClosedManifest, ResultRead, a protection owner or scientific permission.
    let wire: pb::CanonicalAttemptMetadata =
        pse_contract_proto::support::encode_root(original.clone(), "attempt").unwrap();
    let bytes = wire.encode_to_vec();
    let independent = inspect("pse.engineering.v1.CanonicalAttemptMetadata", &bytes);
    assert_eq!(independent.field_by_number(3), Some(&Value::U64(u64::MAX)));
    assert_eq!(independent.field_by_number(7), Some(&Value::I64(-1)));
    assert_eq!(
        independent.field_by_number(12),
        Some(&Value::Bytes(Vec::new()))
    );
    let restored: canonical_attempts::Row =
        pse_contract_proto::support::decode_root(wire, "attempt").unwrap();
    assert_eq!(restored, original);
    let native = storage::encode_canonical_attempts(&restored).unwrap();
    assert_eq!(
        storage::decode_canonical_attempts(native).unwrap(),
        original
    );
}
