// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Physical framing checks and strict aligned decode before semantic admission.

mod compact;
mod parquet;

use std::sync::Arc;

use datafusion::arrow::array::RecordBatch;
use datafusion::arrow::buffer::Buffer;
use datafusion::arrow::datatypes::{Field, Fields};
use datafusion::arrow::ipc::{self, reader::FileDecoder};
use pse_ids::{CancellationToken, Envelope, MemoryReserver};
use pse_schema::Registry;
use pse_schema::model::RelationSpec;

use crate::{CatalogError, EncodingRecord};

/// Decode explicit role maps without silently replacing an earlier binding.
pub(super) fn unique_map<'de, D, V>(
    deserializer: D,
) -> Result<std::collections::BTreeMap<String, V>, D::Error>
where
    D: serde::Deserializer<'de>,
    V: serde::Deserialize<'de>,
{
    struct Unique<V>(std::marker::PhantomData<V>);
    impl<'de, V: serde::Deserialize<'de>> serde::de::Visitor<'de> for Unique<V> {
        type Value = std::collections::BTreeMap<String, V>;
        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("unique explicit role bindings")
        }
        fn visit_map<M: serde::de::MapAccess<'de>>(
            self,
            mut map: M,
        ) -> Result<Self::Value, M::Error> {
            let mut values = std::collections::BTreeMap::new();
            while let Some((key, value)) = map.next_entry::<String, V>()? {
                if values.insert(key, value).is_some() {
                    return Err(serde::de::Error::custom("duplicate role binding"));
                }
            }
            Ok(values)
        }
    }
    deserializer.deserialize_map(Unique(std::marker::PhantomData))
}

/// Check exact bytes against manifest encoding claims. This establishes transport
/// integrity only; the caller must still decode and admit the actual relation.
///
/// # Errors
/// An inconsistent byte count or checksum.
pub fn encoding(bytes: &[u8], record: &EncodingRecord) -> Result<(), CatalogError> {
    if u64::try_from(bytes.len()).ok() != Some(record.bytes) {
        return Err(admission(
            &record.path,
            "actual encoded length differs from manifest",
        ));
    }
    let actual = pse_ids::encoding_checksum(bytes);
    if actual != record.encoding_checksum {
        return Err(CatalogError::CorruptObject {
            path: record.path.clone(),
            expected: record.encoding_checksum.to_string(),
            actual: actual.to_string(),
        });
    }
    Ok(())
}

/// Decode the supported finished IPC file baseline using one explicit aligned copy.
/// The library is required to preserve alignment and cannot silently allocate copies.
/// Schema/value admission is direct and runs regardless of the encoding checksum.
///
/// # Errors
/// Invalid framing, unsupported compression/version/layout, missing metadata,
/// declaration/value violations, allocation limits or cancellation.
pub fn ipc_file(
    bytes: &[u8],
    reg: &Registry,
    spec: &RelationSpec,
    reserver: &dyn MemoryReserver,
    cancel: &CancellationToken,
    envelope: Envelope,
) -> Result<RecordBatch, CatalogError> {
    cancel.checkpoint()?;
    Envelope::lowered(envelope.max_rows, envelope.max_normalized_bytes)?;
    let end = bytes
        .len()
        .checked_sub(10)
        .ok_or_else(|| admission("IPC", "missing footer"))?;
    if !bytes.starts_with(b"ARROW1") || bytes.get(end + 4..) != Some(b"ARROW1".as_slice()) {
        return Err(admission("IPC", "finished Arrow file magic is absent"));
    }
    let length_bytes: [u8; 4] = bytes[end..end + 4]
        .try_into()
        .map_err(|_| admission("IPC", "invalid footer length"))?;
    let footer_len = usize::try_from(i32::from_le_bytes(length_bytes))
        .map_err(|_| admission("IPC", "negative footer length"))?;
    let start = end
        .checked_sub(footer_len)
        .filter(|offset| *offset >= 8)
        .ok_or_else(|| admission("IPC", "footer exceeds actual file extent"))?;
    let mut control = reserver.open("store:ipc-footer");
    control.try_grow(
        footer_len
            .checked_mul(64)
            .ok_or_else(super::encode::overflow)?,
    )?;
    let footer = ipc::root_as_footer(&bytes[start..end])
        .map_err(|error| admission("IPC footer", &error.to_string()))?;
    if footer.version() != ipc::MetadataVersion::V5 {
        return Err(admission("IPC", "only metadata V5 is supported"));
    }
    let declared = pse_schema::arrow::relation_schema(reg, spec)
        .map_err(|error| admission("IPC schema", &error.to_string()))?;
    let encoded_schema = footer
        .schema()
        .ok_or_else(|| admission("IPC", "footer schema is absent"))?;
    let fields = encoded_schema
        .fields()
        .ok_or_else(|| admission("IPC schema", "fields are absent"))?;
    if fields.len() != declared.fields().len() {
        return Err(admission("IPC schema", "field count differs from registry"));
    }
    for (field, expected) in fields.iter().zip(declared.fields()) {
        field_shape(field, expected)?;
    }
    // The pinned conversion has panic branches for unsupported malformed FB types.
    // Shape was bounded against the declared registry tree before entering it.
    let schema = std::panic::catch_unwind(|| ipc::convert::fb_to_schema(encoded_schema))
        .map_err(|_| admission("IPC schema", "invalid or unsupported FlatBuffer schema"))?;
    pse_relations::validate::validate_schema(reg, spec, &schema)
        .map_err(|errors| admission("IPC schema", &messages(&errors)))?;
    let mut decoder =
        FileDecoder::new(Arc::new(schema), footer.version()).with_require_alignment(true);
    let backing = pse_ids::owned_buffer::copy_aligned_buffer(
        bytes,
        "store:ipc-aligned-backing",
        reserver,
        cancel,
    )?;
    let mut ranges = Vec::new();
    if let Some(dictionaries) = footer.dictionaries() {
        for block in dictionaries {
            let data = checked_block(&backing, block, start, &mut ranges, envelope)?;
            decoder
                .read_dictionary(block, &data)
                .map_err(|error| admission("IPC dictionary", &error.to_string()))?;
        }
    }
    let records = footer
        .recordBatches()
        .ok_or_else(|| admission("IPC", "record batch list is absent"))?;
    if records.len() != 1 {
        return Err(admission(
            "IPC",
            "one complete record batch is required, including for empty relations",
        ));
    }
    let block = records.get(0);
    let data = checked_block(&backing, block, start, &mut ranges, envelope)?;
    let batch = decoder
        .read_record_batch(block, &data)
        .map_err(|error| admission("IPC data", &error.to_string()))?
        .ok_or_else(|| admission("IPC", "record batch is absent"))?;
    cancel.checkpoint()?;
    let mut validation = reserver.open("store:ipc-value-admission");
    validation.try_grow(super::membership::validation_extent(&batch)?)?;
    pse_relations::validate::validate_batch(reg, spec, &batch)
        .map_err(|errors| admission("IPC relation", &messages(&errors)))?;
    Ok(batch)
}

fn field_shape(encoded: ipc::Field<'_>, expected: &Field) -> Result<(), CatalogError> {
    use datafusion::arrow::datatypes::DataType;
    let children: Fields = match expected.data_type() {
        DataType::List(child) | DataType::FixedSizeList(child, _) => vec![Arc::clone(child)].into(),
        DataType::Struct(children) => children.clone(),
        _ => Fields::empty(),
    };
    let actual = encoded.children();
    if actual.map_or(0, |fields| fields.len()) != children.len() {
        return Err(admission(
            "IPC schema",
            "nested field count differs from registry",
        ));
    }
    if let Some(actual) = actual {
        for (field, expected) in actual.iter().zip(&children) {
            field_shape(field, expected)?;
        }
    }
    Ok(())
}

fn checked_block(
    backing: &Buffer,
    block: &ipc::Block,
    footer_start: usize,
    ranges: &mut Vec<(usize, usize)>,
    envelope: Envelope,
) -> Result<Buffer, CatalogError> {
    let offset =
        usize::try_from(block.offset()).map_err(|_| admission("IPC", "negative block offset"))?;
    let metadata = usize::try_from(block.metaDataLength())
        .map_err(|_| admission("IPC", "negative metadata length"))?;
    let body = usize::try_from(block.bodyLength())
        .map_err(|_| admission("IPC", "negative body length"))?;
    let len = metadata
        .checked_add(body)
        .ok_or_else(super::encode::overflow)?;
    let end = offset
        .checked_add(len)
        .filter(|end| *end <= footer_start)
        .ok_or_else(|| admission("IPC", "block exceeds actual data extent"))?;
    if offset < 8
        || ranges
            .iter()
            .any(|(start, stop)| offset < *stop && *start < end)
    {
        return Err(admission("IPC", "overlapping or header-aliasing block"));
    }
    ranges.push((offset, end));
    let bytes = &backing.as_slice()[offset..end];
    let prefix = if bytes.starts_with(&[255; 4]) { 8 } else { 4 };
    let length: [u8; 4] = bytes
        .get(prefix - 4..prefix)
        .ok_or_else(|| admission("IPC", "missing metadata length prefix"))?
        .try_into()
        .map_err(|_| admission("IPC", "invalid metadata prefix"))?;
    let encoded_len = usize::try_from(i32::from_le_bytes(length))
        .map_err(|_| admission("IPC", "negative message length"))?;
    let message_end = prefix
        .checked_add(encoded_len)
        .filter(|end| *end <= metadata)
        .ok_or_else(|| admission("IPC", "message exceeds metadata block"))?;
    let message = ipc::root_as_message(&bytes[prefix..message_end])
        .map_err(|error| admission("IPC message", &error.to_string()))?;
    if message.version() != ipc::MetadataVersion::V5 || message.bodyLength() != block.bodyLength() {
        return Err(admission(
            "IPC",
            "message version/body extent differs from block",
        ));
    }
    let batch = if let Some(dictionary) = message.header_as_dictionary_batch() {
        dictionary.data()
    } else {
        message.header_as_record_batch()
    }
    .ok_or_else(|| admission("IPC", "block does not contain a record or dictionary batch"))?;
    if batch.compression().is_some() {
        return Err(admission(
            "IPC",
            "compressed body is outside the supported import baseline",
        ));
    }
    let rows =
        u64::try_from(batch.length()).map_err(|_| admission("IPC", "negative batch length"))?;
    if message.header_as_record_batch().is_some() && rows > envelope.max_rows {
        return Err(admission("IPC", "row count exceeds supported envelope"));
    }
    if let Some(buffers) = batch.buffers() {
        for buffer in buffers {
            let start = usize::try_from(buffer.offset())
                .map_err(|_| admission("IPC", "negative buffer offset"))?;
            let len = usize::try_from(buffer.length())
                .map_err(|_| admission("IPC", "negative buffer length"))?;
            if start.checked_add(len).is_none_or(|end| end > body) {
                return Err(admission("IPC", "buffer exceeds actual body extent"));
            }
        }
    }
    Ok(backing.slice_with_length(offset, len))
}

pub(crate) fn admission(path: &str, reason: &str) -> CatalogError {
    CatalogError::Admission {
        path: path.to_owned(),
        reason: reason.to_owned(),
    }
}
fn messages(errors: &[pse_relations::RelationError]) -> String {
    errors
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("; ")
}

/// Decode the bounded uncompressed, plain-value Parquet baseline, preserving exact
/// registry semantic metadata and retaining every output buffer reservation.
///
/// # Errors
/// Unsupported encoding, malformed framing, actual schema/value violations or limits.
pub fn parquet_file(
    bytes: &bytes::Bytes,
    reg: &Registry,
    spec: &RelationSpec,
    reserver: &dyn MemoryReserver,
    cancel: &CancellationToken,
    envelope: Envelope,
) -> Result<RecordBatch, CatalogError> {
    parquet::decode(bytes, reg, spec, reserver, cancel, envelope)
}

pub(crate) fn decode_file(
    format: crate::EncodingFormat,
    bytes: &bytes::Bytes,
    reg: &Registry,
    spec: &RelationSpec,
    reserver: &dyn MemoryReserver,
    cancel: &CancellationToken,
    envelope: Envelope,
) -> Result<RecordBatch, CatalogError> {
    Envelope::lowered(envelope.max_rows, envelope.max_normalized_bytes)?;
    match format {
        crate::EncodingFormat::ArrowIpcFile => {
            ipc_file(bytes, reg, spec, reserver, cancel, envelope)
        }
        crate::EncodingFormat::Parquet => {
            parquet_file(bytes, reg, spec, reserver, cancel, envelope)
        }
    }
}

#[cfg(test)]
mod role_map_tests {
    #[derive(serde::Deserialize)]
    struct Roles {
        #[serde(deserialize_with = "super::unique_map")]
        inputs: std::collections::BTreeMap<String, Option<u8>>,
    }
    #[test]
    fn repeated_roles_cannot_overwrite_an_explicit_absence() {
        assert!(serde_json::from_str::<Roles>(r#"{"inputs":{"model":null,"model":1}}"#).is_err());
        let roles =
            serde_json::from_str::<Roles>(r#"{"inputs":{"model":null}}"#).expect("unique role");
        assert_eq!(roles.inputs["model"], None);
    }
}
