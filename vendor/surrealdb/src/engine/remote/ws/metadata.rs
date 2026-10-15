//! Borrowed preflight at the existing owned-response conversion seam. This
//! uses pinned generated accessors; it never decodes a second owned Value tree.

use surrealdb_protocol::fb::v1 as fb;
use surrealdb_engine_api::ResponseLimits;
use uuid::Uuid;

type Refusal = &'static str;

pub(super) fn root(binary: &[u8]) -> Result<fb::Value<'_>, Refusal> {
    // Same finite verifier envelope as types3.3, solely to access the borrowed
    // correlation fields safely. The endpoint's tighter occurrence bounds run
    // below before any owned tree is constructed.
    let options = flatbuffers::VerifierOptions {
        max_tables: binary.len(),
        max_depth: 512,
        ..Default::default()
    };
    flatbuffers::root_with_opts::<fb::Value>(&options, binary)
        .map_err(|_| "invalid or excessive response FlatBuffers structure")
}

/// Correlate only fixed-size session UUID and signed numeric request ID. The
/// verified envelope's field scan allocates nothing and ignores result payload.
pub(super) fn correlation(root: fb::Value<'_>) -> Result<Option<(Uuid, i64)>, Refusal> {
    let Some(object) = root.value_as_object() else { return Ok(None) };
    let items = object.items().ok_or("response object has no fields")?;
    if items.len() > 8 { return Err("response envelope field limit") }
    let mut session = None;
    let mut id = None;
    for entry in items {
        match entry.key() {
            Some("id") => {
                if id.is_some() { return Err("duplicate response id") }
                id = entry.value().and_then(|v| v.value_as_int_64()).map(|v| v.value());
            }
            Some("session") => {
                if session.is_some() { return Err("duplicate response session") }
                if let Some(bytes) = entry.value().and_then(|v| v.value_as_uuid())
                    .and_then(|v| v.bytes()) {
                    session = Some(Uuid::from_slice(bytes.bytes()).map_err(|_| "response UUID extent")?);
                }
            }
            _ => {}
        }
    }
    Ok(session.zip(id))
}

pub(super) fn validate(binary: &[u8], limits: ResponseLimits) -> Result<(), Refusal> {
    if binary.len() > limits.wire_bytes { return Err("metadata response wire extent") }
    let options = flatbuffers::VerifierOptions {
        max_tables: limits.nodes.saturating_mul(4),
        max_depth: limits.depth.saturating_mul(4),
        max_apparent_size: limits.string_bytes.saturating_add(limits.blob_bytes)
            .saturating_add(limits.nodes.saturating_mul(128)),
        ..Default::default()
    };
    let root = flatbuffers::root_with_opts::<fb::Value>(&options, binary)
        .map_err(|_| "metadata response verified extent")?;
    Count { limits, nodes: 0, strings: 0, blobs: 0 }.value(root, 0)
}

struct Count {
    limits: ResponseLimits,
    nodes: usize,
    strings: usize,
    blobs: usize,
}
impl Count {
    fn node(&mut self) -> Result<(), Refusal> {
        self.nodes = self.nodes.checked_add(1).ok_or("metadata node overflow")?;
        if self.nodes > self.limits.nodes { return Err("metadata node extent") }
        Ok(())
    }
    fn string(&mut self, value: &str) -> Result<(), Refusal> {
        if value.len() > self.limits.string_length { return Err("metadata string length") }
        self.strings = self.strings.checked_add(value.len()).ok_or("metadata string overflow")?;
        if self.strings > self.limits.string_bytes { return Err("metadata string copy extent") }
        Ok(())
    }
    fn container(&self, len: usize) -> Result<(), Refusal> {
        if len > self.limits.container_items { return Err("metadata container extent") }
        Ok(())
    }
    fn value(&mut self, value: fb::Value<'_>, depth: usize) -> Result<(), Refusal> {
        if depth > self.limits.depth { return Err("metadata nesting extent") }
        self.node()?;
        match value.value_type() {
            fb::ValueType::NONE | fb::ValueType::None | fb::ValueType::Null => {}
            fb::ValueType::Bool if value.value_as_bool().is_some() => {}
            fb::ValueType::Int64 if value.value_as_int_64().is_some() => {}
            fb::ValueType::Float64 if value.value_as_float_64().is_some() => {}
            fb::ValueType::Datetime if value.value_as_datetime().is_some() => {}
            fb::ValueType::Duration if value.value_as_duration().is_some() => {}
            fb::ValueType::Uuid => {
                if value.value_as_uuid().and_then(|v| v.bytes()).is_none_or(|v| v.len() != 16) {
                    return Err("metadata UUID extent");
                }
            }
            fb::ValueType::String => self.string(value.value_as_string()
                .and_then(|v| v.value()).ok_or("metadata string missing")?)?,
            fb::ValueType::Decimal => self.string(value.value_as_decimal()
                .and_then(|v| v.value()).ok_or("metadata decimal missing")?)?,
            fb::ValueType::Bytes => {
                let bytes = value.value_as_bytes().and_then(|v| v.value()).ok_or("metadata bytes missing")?;
                if bytes.len() > self.limits.blob_length { return Err("metadata blob length") }
                self.blobs = self.blobs.checked_add(bytes.len()).ok_or("metadata blob overflow")?;
                if self.blobs > self.limits.blob_bytes { return Err("metadata blob copy extent") }
            }
            fb::ValueType::Array => {
                let values = value.value_as_array().and_then(|v| v.values()).ok_or("metadata array missing")?;
                self.container(values.len())?;
                for value in values { self.value(value, depth + 1)?; }
            }
            fb::ValueType::Object => {
                let entries = value.value_as_object().and_then(|v| v.items()).ok_or("metadata object missing")?;
                self.container(entries.len())?;
                for entry in entries {
                    // Count each occurrence: a shared offset is copied once per
                    // reference by types3.3::from_fb, not once per wire address.
                    self.node()?;
                    self.string(entry.key().ok_or("metadata key missing")?)?;
                    self.value(entry.value().ok_or("metadata value missing")?, depth + 1)?;
                }
            }
            fb::ValueType::RecordId => {
                let record = value.value_as_record_id().ok_or("metadata record missing")?;
                self.string(record.table().ok_or("metadata record table missing")?)?;
                let key = record.id().ok_or("metadata record key missing")?;
                self.node()?;
                match key.id_type() {
                    fb::RecordIdKeyType::String => self.string(key.id_as_string()
                        .and_then(|v| v.value()).ok_or("metadata record string missing")?)?,
                    fb::RecordIdKeyType::Int64 => {}
                    fb::RecordIdKeyType::Uuid => {
                        if key.id_as_uuid().and_then(|v| v.bytes()).is_none_or(|v| v.len() != 16) {
                            return Err("metadata record UUID extent");
                        }
                    }
                    _ => return Err("metadata record key shape"),
                }
            }
            _ => return Err("value type outside canonical metadata layout"),
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn limits() -> ResponseLimits {
        ResponseLimits { wire_bytes: 4096, nodes: 128, depth: 8, container_items: 64,
            string_length: 128, string_bytes: 256, blob_length: 128, blob_bytes: 128 }
    }
    #[test]
    fn bounded_metadata_preflight_refuses_unbounded_fields_and_wrong_shapes() {
        use surrealdb_types::{Array, Value};
        let value = surrealdb_types::encode(&Value::String("a".repeat(129))).unwrap();
        assert!(validate(&value, limits()).is_err());
        let value = surrealdb_types::encode(&Value::Array(Array::from(vec![Value::Null; 65]))).unwrap();
        assert!(validate(&value, limits()).is_err());
        let value = surrealdb_types::encode(&Value::String("valid".into())).unwrap();
        assert!(validate(&value, limits()).is_ok());
        assert!(root(&[0, 1, 2]).is_err());
    }
    #[test]
    fn repeated_offsets_count_each_owned_string_copy() {
        let mut builder = flatbuffers::FlatBufferBuilder::new();
        let text = builder.create_string(&"s".repeat(128));
        let string = fb::StringValue::create(&mut builder, &fb::StringValueArgs { value: Some(text) });
        let value = fb::Value::create(&mut builder, &fb::ValueArgs {
            value_type: fb::ValueType::String, value: Some(string.as_union_value()),
        });
        let values = builder.create_vector(&[value, value, value]);
        let array = fb::Array::create(&mut builder, &fb::ArrayArgs { values: Some(values) });
        let root = fb::Value::create(&mut builder, &fb::ValueArgs {
            value_type: fb::ValueType::Array, value: Some(array.as_union_value()),
        });
        builder.finish_minimal(root);
        let mut bound = limits();
        bound.string_bytes = 384;
        assert!(validate(builder.finished_data(), bound).is_ok());
        bound.string_bytes = 383;
        assert!(validate(builder.finished_data(), bound).is_err());
    }
}
