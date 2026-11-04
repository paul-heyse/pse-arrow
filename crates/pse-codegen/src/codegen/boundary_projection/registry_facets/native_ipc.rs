// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Standard single-field IPC carrier. Verify ambiguous metadata and logical union
//! premises before Arrow's infallible conversion can discard or panic on them.
use super::super::{error, ProjectionCodegenError};
use arrow_ipc as ipc;
use arrow_schema::{Field, Schema};
use std::collections::HashSet;

pub(super) fn encode(field: &Field) -> Result<Vec<u8>, ProjectionCodegenError> {
    let schema = Schema::new(vec![field.clone()]);
    let encoded = ipc::writer::IpcDataGenerator::default().schema_to_bytes_with_dictionary_tracker(
        &schema, &mut ipc::writer::DictionaryTracker::new(false), &ipc::writer::IpcWriteOptions::default());
    // The same receiving guard applies to authored source classes, so publication
    // cannot mint a carrier this known reader cannot safely interpret.
    decode(&encoded.ipc_message)?;
    Ok(encoded.ipc_message)
}

pub(super) fn decode(bytes: &[u8]) -> Result<Field, ProjectionCodegenError> {
    let message = ipc::root_as_message(bytes)?;
    if message.bodyLength() != 0 || !matches!(message.version(), ipc::MetadataVersion::V4 | ipc::MetadataVersion::V5) {
        return Err(error("native field carrier requires a supported body-free IPC schema message"));
    }
    if message.custom_metadata().is_some_and(|metadata| !metadata.is_empty()) { return Err(error("native field carrier has unsupported message metadata")); }
    let schema = message.header_as_schema().ok_or_else(|| error("native field carrier is not an IPC schema"))?;
    if schema.endianness() != ipc::Endianness::Little || schema.features().is_some_and(|features| !features.is_empty()) {
        return Err(error("native field carrier has unsupported schema endianness/features"));
    }
    if let Some(metadata) = schema.custom_metadata() { check_metadata(metadata.iter())?; if !metadata.is_empty() { return Err(error("single native field carrier cannot contain schema-level metadata")); } }
    let fields = schema.fields().ok_or_else(|| error("native field carrier has no fields"))?;
    if fields.len() != 1 { return Err(error("native field carrier must contain exactly one field")); }
    check_field(fields.get(0), &mut HashSet::new())?;
    let native = ipc::convert::fb_to_schema(schema);
    native.fields().first().map(|field| field.as_ref().clone()).ok_or_else(|| error("native field carrier lost its required field"))
}

fn check_metadata<'a>(metadata: impl Iterator<Item=ipc::KeyValue<'a>>) -> Result<(), ProjectionCodegenError> {
    let mut keys = HashSet::new();
    for entry in metadata {
        let key = entry.key().ok_or_else(|| error("native IPC metadata key is absent"))?;
        if entry.value().is_none() || !keys.insert(key) { return Err(error("native IPC metadata has an absent value or duplicate key")); }
    }
    Ok(())
}

fn check_field(field: ipc::Field<'_>, dictionaries: &mut HashSet<i64>) -> Result<(), ProjectionCodegenError> {
    if field.name().is_none_or(str::is_empty) { return Err(error("native IPC field name is absent/empty")); }
    if let Some(metadata) = field.custom_metadata() { check_metadata(metadata.iter())?; }
    if let Some(dictionary) = field.dictionary() {
        let index = dictionary.indexType().ok_or_else(|| error("native IPC dictionary has no index type"))?;
        if !matches!(index.bitWidth(), 8 | 16 | 32 | 64) || !dictionaries.insert(dictionary.id())
            || dictionary.dictionaryKind() != ipc::DictionaryKind::DenseArray {
            return Err(error("unsupported or ambiguous native IPC dictionary encoding"));
        }
    }
    let children = field.children();
    let child_count = children.map_or(0, |children| children.len());
    let invalid = || error("unsupported or inconsistent native IPC logical type");
    let leaf = match field.type_type() {
        ipc::Type::Null | ipc::Type::Bool | ipc::Type::Binary | ipc::Type::LargeBinary
        | ipc::Type::BinaryView | ipc::Type::Utf8 | ipc::Type::LargeUtf8 | ipc::Type::Utf8View => true,
        ipc::Type::Int => { if !field.type_as_int().is_some_and(|value| matches!(value.bitWidth(), 8 | 16 | 32 | 64)) { return Err(invalid()); } true },
        ipc::Type::FloatingPoint => { if !field.type_as_floating_point().is_some_and(|value| matches!(value.precision(), ipc::Precision::HALF | ipc::Precision::SINGLE | ipc::Precision::DOUBLE)) { return Err(invalid()); } true },
        ipc::Type::FixedSizeBinary => { if !field.type_as_fixed_size_binary().is_some_and(|value| value.byteWidth() >= 0) { return Err(invalid()); } true },
        ipc::Type::Date => { if !field.type_as_date().is_some_and(|value| matches!(value.unit(), ipc::DateUnit::DAY | ipc::DateUnit::MILLISECOND)) { return Err(invalid()); } true },
        ipc::Type::Time => { if !field.type_as_time().is_some_and(|value| matches!((value.bitWidth(), value.unit()), (32, ipc::TimeUnit::SECOND | ipc::TimeUnit::MILLISECOND) | (64, ipc::TimeUnit::MICROSECOND | ipc::TimeUnit::NANOSECOND))) { return Err(invalid()); } true },
        ipc::Type::Timestamp => { if !field.type_as_timestamp().is_some_and(|value| time_unit(value.unit())) { return Err(invalid()); } true },
        ipc::Type::Duration => { if !field.type_as_duration().is_some_and(|value| time_unit(value.unit())) { return Err(invalid()); } true },
        ipc::Type::Interval => { if !field.type_as_interval().is_some_and(|value| matches!(value.unit(), ipc::IntervalUnit::YEAR_MONTH | ipc::IntervalUnit::DAY_TIME | ipc::IntervalUnit::MONTH_DAY_NANO)) { return Err(invalid()); } true },
        ipc::Type::Decimal => {
            if !field.type_as_decimal().is_some_and(|value| {
                let max_precision = match value.bitWidth() { 32 => 9, 64 => 18, 128 => 38, 256 => 76, _ => return false };
                (1..=max_precision).contains(&value.precision()) && i8::try_from(value.scale()).is_ok() && value.scale() <= value.precision()
            }) { return Err(invalid()); } true
        },
        ipc::Type::List | ipc::Type::LargeList | ipc::Type::ListView | ipc::Type::LargeListView => { if child_count != 1 { return Err(invalid()); } false },
        ipc::Type::FixedSizeList => { if child_count != 1 || !field.type_as_fixed_size_list().is_some_and(|value| value.listSize() >= 0) { return Err(invalid()); } false },
        ipc::Type::Struct_ => false,
        ipc::Type::Map => {
            if child_count != 1 || field.type_as_map().is_none() { return Err(invalid()); }
            let entry = children.ok_or_else(invalid)?.get(0);
            if entry.nullable() || entry.type_type() != ipc::Type::Struct_ || entry.children().is_none_or(|fields| fields.len() != 2 || fields.get(0).nullable()) { return Err(invalid()); }
            false
        },
        ipc::Type::RunEndEncoded => {
            if child_count != 2 { return Err(invalid()); }
            let run = children.ok_or_else(invalid)?.get(0);
            if run.nullable() || run.dictionary().is_some() || !run.type_as_int().is_some_and(|value| value.is_signed() && matches!(value.bitWidth(), 16 | 32 | 64)) { return Err(invalid()); }
            false
        },
        ipc::Type::Union => {
            let union = field.type_as_union().ok_or_else(invalid)?;
            if !matches!(union.mode(), ipc::UnionMode::Dense | ipc::UnionMode::Sparse) { return Err(invalid()); }
            // Require explicit nonnegative IDs: avoids implicit inference and casts
            // that truncate acquired i32 IDs to i8 before native UnionFields checks.
            let ids = union.typeIds().ok_or_else(invalid)?;
            let mut seen = HashSet::new();
            if ids.len() != child_count || ids.iter().any(|id| !(0..=127).contains(&id) || !seen.insert(id)) { return Err(invalid()); }
            false
        },
        _ => return Err(invalid()),
    };
    if leaf && child_count != 0 { return Err(invalid()); }
    if let Some(children) = children {
        let mut names = HashSet::new();
        for child in children { if !names.insert(child.name()) { return Err(error("ambiguous duplicate native IPC child name")); } check_field(child, dictionaries)?; }
    }
    Ok(())
}
fn time_unit(unit: ipc::TimeUnit) -> bool { matches!(unit, ipc::TimeUnit::SECOND | ipc::TimeUnit::MILLISECOND | ipc::TimeUnit::MICROSECOND | ipc::TimeUnit::NANOSECOND) }

#[cfg(test)]
mod tests {
    use super::*;
    use arrow_schema::{DataType, TimeUnit};

    #[test]
    fn native_ipc_preserves_temporal_dictionary_decimal_and_nested_field_meaning() {
        for kind in [DataType::Timestamp(TimeUnit::Microsecond, Some("UTC".into())), DataType::Int64,
            DataType::Decimal128(20, -2), DataType::Dictionary(Box::new(DataType::Int16), Box::new(DataType::Utf8)),
            DataType::Struct(vec![std::sync::Arc::new(Field::new("child", DataType::Utf8, true).with_metadata(std::collections::HashMap::from([("source".into(), "native".into())])))].into())] {
            let field = Field::new("source", kind, true);
            assert_eq!(decode(&encode(&field).unwrap()).unwrap(), field);
        }
    }

    // Literal acquired FlatBuffers are independent of the source writer. Each
    // malformed input used to reach a lossy map conversion or panic branch.
    fn acquired(case: &str) -> Vec<u8> {
        let mut builder = flatbuffers::FlatBufferBuilder::new();
        let key = builder.create_string("pse.domain.role");
        let value = builder.create_string("measure");
        let metadata = ipc::KeyValue::create(&mut builder, &ipc::KeyValueArgs { key: Some(key), value: (case != "missing-value").then_some(value) });
        let metadata = if matches!(case, "duplicate" | "nested-duplicate") { builder.create_vector(&[metadata, metadata]) } else { builder.create_vector(&[metadata]) };
        let name = builder.create_string("source");
        let int = ipc::Int::create(&mut builder, &ipc::IntArgs { bitWidth: if case == "invalid-width" { 7 } else { 64 }, is_signed: true });
        let field = ipc::Field::create(&mut builder, &ipc::FieldArgs { name: Some(name), type_type: ipc::Type::Int, type_: Some(int.as_union_value()), custom_metadata: Some(metadata), ..Default::default() });
        let fields = if case == "nested-duplicate" {
            let children = builder.create_vector(&[field]);
            let structure = ipc::Struct_::create(&mut builder, &ipc::Struct_Args::default());
            let root = ipc::Field::create(&mut builder, &ipc::FieldArgs { name: Some(name), type_type: ipc::Type::Struct_, type_: Some(structure.as_union_value()), children: Some(children), ..Default::default() });
            builder.create_vector(&[root])
        } else if case == "cardinality" { builder.create_vector(&[field, field]) } else { builder.create_vector(&[field]) };
        let schema = ipc::Schema::create(&mut builder, &ipc::SchemaArgs { fields: Some(fields), ..Default::default() });
        let message = ipc::Message::create(&mut builder, &ipc::MessageArgs { version: ipc::MetadataVersion::V5, header_type: ipc::MessageHeader::Schema, header: Some(schema.as_union_value()), ..Default::default() });
        ipc::finish_message_buffer(&mut builder, message);
        builder.finished_data().into()
    }
    #[test]
    fn acquired_ipc_refuses_ambiguity_before_library_map_or_logical_type_conversion() {
        assert!(decode(&acquired("valid")).is_ok());
        for case in ["duplicate", "nested-duplicate", "missing-value", "invalid-width", "cardinality"] {
            assert!(decode(&acquired(case)).is_err(), "{case}");
        }
    }
}
