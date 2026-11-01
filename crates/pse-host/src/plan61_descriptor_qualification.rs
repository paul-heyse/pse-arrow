// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Qualification of acquired descriptor/payload interpretation, not a public reader.

use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

use buffa::Message;
use buffa_descriptor::generated::descriptor::FileDescriptorSet;
use buffa_descriptor::{DescriptorPool, DynamicMessage, ReflectMessage as _, Value};

const DESCRIPTORS: &[u8] = include_bytes!("generated/plan61/descriptor.bin");
const PAYLOAD: &str = "plan61.qualification.v1.Payload";

// Pool additions deliberately skip already-known filenames. An acquired bundle must be
// checked before linking; otherwise the first file silently wins a conflict.
fn checked_pool(mut set: FileDescriptorSet) -> Result<DescriptorPool, String> {
    let mut files = BTreeMap::new();
    for file in &set.file {
        let name = file.name.as_deref().ok_or("descriptor filename missing")?;
        let bytes = file.encode_to_vec();
        if let Some(previous) = files.insert(name, bytes.clone()) {
            if previous != bytes {
                return Err(format!("conflicting descriptor file: {name}"));
            }
        }
    }
    let mut seen = BTreeSet::new();
    set.file.retain(|file| seen.insert(file.name.clone()));
    DescriptorPool::new(set).map_err(|error| error.to_string())
}

fn descriptors() -> FileDescriptorSet {
    FileDescriptorSet::decode_from_slice(DESCRIPTORS).unwrap()
}

fn pool() -> Arc<DescriptorPool> {
    Arc::new(checked_pool(descriptors()).unwrap())
}

// This is the observed Buf 1.75 source-coordinate convention, not a universal
// promise for arbitrary descriptor producers. Original source bytes remain required.
fn source_range(source: &str, span: &[i32]) -> Option<std::ops::Range<usize>> {
    let coordinate = |line: i32, column: i32| {
        let line = usize::try_from(line).ok()?;
        let column = usize::try_from(column).ok()?;
        let mut offset = 0;
        for (number, text) in source.split_inclusive('\n').enumerate() {
            if number == line {
                let text = text.trim_end_matches('\n');
                let within = text
                    .char_indices()
                    .map(|(byte, _)| byte)
                    .chain(std::iter::once(text.len()))
                    .nth(column)?;
                return Some(offset + within);
            }
            offset += text.len();
        }
        None
    };
    let (start, end) = match span {
        [line, start, end] => (coordinate(*line, *start)?, coordinate(*line, *end)?),
        [start_line, start, end_line, end] => (
            coordinate(*start_line, *start)?,
            coordinate(*end_line, *end)?,
        ),
        _ => return None,
    };
    (end >= start).then_some(start..end)
}

#[test]
fn plan61_source_code_info_requires_unicode_column_conversion() {
    let source = include_str!("../tests/fixtures/plan61/plan61/qualification/v1/contract.proto");
    let set = descriptors();
    let file = set
        .file
        .iter()
        .find(|file| file.name.as_deref() == Some("plan61/qualification/v1/contract.proto"))
        .unwrap();
    let payload_index = file
        .message_type
        .iter()
        .position(|message| message.name.as_deref() == Some("Payload"))
        .unwrap();
    let label_index = file.message_type[payload_index]
        .field
        .iter()
        .position(|field| field.name.as_deref() == Some("label"))
        .unwrap();
    let info = file.source_code_info.as_option().unwrap();
    let message_path = [4, i32::try_from(payload_index).unwrap()];
    let message_span = &info
        .location
        .iter()
        .find(|location| location.path == message_path)
        .unwrap()
        .span;
    assert_eq!(message_span.len(), 4, "multiline declaration");
    let range = source_range(source, message_span).unwrap();
    assert!(source[range].starts_with("message Payload {"));
    let field_path = [
        4,
        i32::try_from(payload_index).unwrap(),
        2,
        i32::try_from(label_index).unwrap(),
    ];
    let field_span = &info
        .location
        .iter()
        .find(|location| location.path == field_path)
        .unwrap()
        .span;
    let range = source_range(source, field_span).unwrap();
    assert_eq!(&source[range], "string label = 6;");
    let line = source
        .lines()
        .nth(usize::try_from(field_span[0]).unwrap())
        .unwrap();
    assert_ne!(
        line.find("string").unwrap(),
        usize::try_from(field_span[1]).unwrap(),
        "Unicode scalar column is not a UTF-8 byte column"
    );
    assert!(source_range(source, &[-1, 0, 1]).is_none());
    assert!(source_range(source, &[0, 1000, 1001]).is_none());
    assert!(source_range(source, &[0, 5, 2]).is_none());
}

#[test]
fn plan61_descriptor_closure_options_and_service_relationships() {
    let pool = pool();
    let message = pool.message_by_name(PAYLOAD).unwrap();
    assert_eq!(message.field_by_name("count").unwrap().number(), 1);
    assert!(pool.message_by_name("google.protobuf.Any").is_some());
    assert!(pool.message_by_name("google.protobuf.FieldMask").is_some());
    let service = pool
        .service_by_name("plan61.qualification.v1.Investigation")
        .unwrap();
    assert!(!service.methods().is_empty());
    for method in service.methods() {
        assert!(
            pool.message(method.input())
                .full_name()
                .starts_with("plan61.qualification.v1.")
        );
        assert!(
            pool.message(method.output())
                .full_name()
                .starts_with("plan61.qualification.v1.")
        );
    }
    let file = pool.file_containing_symbol(PAYLOAD).unwrap();
    assert!(file.source_code_info.as_option().is_some());
    // Reading options through their descriptor avoids dropping encoded extensions in
    // a conversion through a typed descriptor representation from another runtime.
    let options =
        DynamicMessage::from_options(Arc::clone(&pool), message.options().unwrap()).unwrap();
    let note = pool
        .extension_by_name("plan61.qualification.v1.qualification_note")
        .unwrap();
    assert!(options.has(note.field()));
    let encoded = descriptors().encode_to_vec();
    let roundtrip = DescriptorPool::decode(&encoded).unwrap();
    assert_eq!(
        roundtrip
            .file_by_name(file.name.as_deref().unwrap())
            .unwrap()
            .encode_to_vec(),
        file.encode_to_vec()
    );
}

#[test]
fn plan61_descriptor_conflicts_refuse_before_pool_skips_duplicate_names() {
    let mut set = descriptors();
    let file = set
        .file
        .iter()
        .find(|file| {
            file.message_type
                .iter()
                .any(|m| m.name.as_deref() == Some("Payload"))
        })
        .unwrap()
        .clone();
    set.file.push(file.clone());
    assert!(
        checked_pool(set.clone()).is_ok(),
        "identical re-add is harmless"
    );
    let mut conflicting = file;
    conflicting.package = Some("different.world".into());
    set.file.push(conflicting);
    assert!(
        checked_pool(set)
            .unwrap_err()
            .starts_with("conflicting descriptor file:")
    );
}

#[test]
fn plan61_descriptor_same_names_do_not_identify_schema_worlds() {
    let original = descriptors();
    let mut changed = original.clone();
    let payload = changed
        .file
        .iter_mut()
        .flat_map(|file| &mut file.message_type)
        .find(|message| message.name.as_deref() == Some("Payload"))
        .unwrap();
    let label = payload
        .field
        .iter_mut()
        .find(|field| field.name.as_deref() == Some("label"))
        .unwrap();
    label.name = Some("renamed_label".into());
    label.json_name = Some("renamedLabel".into());
    assert_ne!(original.encode_to_vec(), changed.encode_to_vec());
    let old_pool = checked_pool(original).unwrap();
    let new_pool = checked_pool(changed).unwrap();
    assert_eq!(
        old_pool.message_by_name(PAYLOAD).unwrap().full_name(),
        new_pool.message_by_name(PAYLOAD).unwrap().full_name()
    );
    assert!(
        old_pool
            .message_by_name(PAYLOAD)
            .unwrap()
            .field_by_name("label")
            .is_some()
    );
    assert!(
        new_pool
            .message_by_name(PAYLOAD)
            .unwrap()
            .field_by_name("renamed_label")
            .is_some()
    );
    // Field numbers remain wire-compatible while authored names and generated Rust
    // change. Native source/Cargo correspondence needs separate evidence.
    assert_eq!(
        old_pool
            .message_by_name(PAYLOAD)
            .unwrap()
            .field_by_name("label")
            .unwrap()
            .number(),
        new_pool
            .message_by_name(PAYLOAD)
            .unwrap()
            .field_by_name("renamed_label")
            .unwrap()
            .number()
    );
    let old_pool = Arc::new(old_pool);
    let old = DynamicMessage::from_json(
        Arc::clone(&old_pool),
        old_pool.message_index(PAYLOAD).unwrap(),
        r#"{"label":"original"}"#,
    )
    .unwrap();
    let new_pool = Arc::new(new_pool);
    let new = DynamicMessage::decode(
        Arc::clone(&new_pool),
        new_pool.message_index(PAYLOAD).unwrap(),
        &old.encode_to_vec(),
    )
    .unwrap();
    let new_json: serde_json::Value = serde_json::from_str(&new.to_json().unwrap()).unwrap();
    assert_eq!(new_json["renamedLabel"], "original");
    assert!(new_json.get("label").is_none());
    let mut missing_import = descriptors();
    missing_import
        .file
        .retain(|file| file.name.as_deref() != Some("google/protobuf/any.proto"));
    assert!(checked_pool(missing_import).is_err());
}

#[test]
fn plan61_payload_text_is_readable_but_not_original_binary_identity() {
    use crate::plan61_qualification::pb::plan61::qualification::v1::Payload;
    let value: Payload =
        buffa::text::decode_from_str("count: 0 path: \"\\377\\000\" label: \"π\"").unwrap();
    assert_eq!(value.count, Some(0));
    assert_eq!(value.path.as_slice(), &[255, 0]);
    let printed = buffa::text::encode_to_string(&value);
    let reparsed: Payload = buffa::text::decode_from_str(&printed).unwrap();
    assert_eq!(value.encode_to_vec(), reparsed.encode_to_vec());
    // Published 0.9.2 impl_text emits skip_value for unknown names without a strict
    // codegen option. This is qualified lossy interpretation, not strict ingress.
    let ignored = buffa::text::decode_from_str::<Payload>("unrecognized_field: 5").unwrap();
    assert!(ignored.encode_to_vec().is_empty());
    assert!(buffa::text::decode_from_str::<Payload>("count: [").is_err());
}

#[test]
fn plan61_payload_presence_unknowns_and_wire_failure_are_distinct() {
    let pool = pool();
    let index = pool.message_index(PAYLOAD).unwrap();
    let empty = DynamicMessage::decode(Arc::clone(&pool), index, &[]).unwrap();
    let explicit_default = DynamicMessage::decode(Arc::clone(&pool), index, &[8, 0]).unwrap();
    assert!(empty.field_by_number(1).is_none());
    assert_eq!(explicit_default.field_by_number(1), Some(&Value::I64(0)));
    assert_eq!(empty.to_json().unwrap(), "{}");
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&explicit_default.to_json().unwrap()).unwrap()["count"],
        "0"
    );
    let unknown_enum = DynamicMessage::decode(Arc::clone(&pool), index, &[24, 99]).unwrap();
    assert_eq!(
        unknown_enum.field_by_number(3),
        Some(&Value::EnumNumber(99))
    );
    let unknown = DynamicMessage::decode(Arc::clone(&pool), index, &[0x98, 6, 7]).unwrap();
    assert!(!unknown.unknown_fields().is_empty());
    assert_eq!(unknown.encode_to_vec(), [0x98, 6, 7]);
    for malformed in [&[0x12, 3, 1][..], &[8, 0x80][..], &[0][..]] {
        assert!(DynamicMessage::decode(Arc::clone(&pool), index, malformed).is_err());
    }
}

#[test]
fn plan61_payload_json_int64_bytes_any_and_field_mask() {
    let pool = pool();
    let index = pool.message_index(PAYLOAD).unwrap();
    let input = r#"{"count":"9223372036854775807","path":"/wA=","mask":"someField,nested.childField","detail":{"@type":"type.googleapis.com/plan61.qualification.v1.Payload","label":"nested"}}"#;
    let value = DynamicMessage::from_json(Arc::clone(&pool), index, input).unwrap();
    assert_eq!(value.field_by_number(1), Some(&Value::I64(i64::MAX)));
    assert_eq!(value.field_by_number(2), Some(&Value::Bytes(vec![255, 0])));
    let output: serde_json::Value = serde_json::from_str(&value.to_json().unwrap()).unwrap();
    assert_eq!(output["count"], "9223372036854775807");
    assert_eq!(output["path"], "/wA=");
    assert_eq!(output["mask"], "someField,nested.childField");
    assert_eq!(output["detail"]["label"], "nested");
    let binary = value.encode_to_vec();
    let decoded = DynamicMessage::decode(Arc::clone(&pool), index, &binary).unwrap();
    assert_eq!(value, decoded);
    assert!(DynamicMessage::from_json(Arc::clone(&pool), index, r#"{"unexpected":1}"#).is_err());
    assert!(
        DynamicMessage::from_json(
            Arc::clone(&pool),
            index,
            r#"{"detail":{"@type":"type.googleapis.com/missing.Type"}}"#
        )
        .is_err()
    );
    assert!(DynamicMessage::from_json(pool, index, r#"{"count":1,"count":2}"#).is_err());
}
