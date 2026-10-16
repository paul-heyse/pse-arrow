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

/// Only these wire targets have an installed recipient. Unknown or malformed
/// correlations are never grounds to construct an owned result tree.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Target {
    Discard,
    Reply { session: Uuid, id: i64 },
    Live { session: Uuid, query: Uuid },
}
fn uuid(value: fb::Value<'_>) -> Result<Uuid, Refusal> {
    let bytes = value.value_as_uuid().and_then(|v| v.bytes()).ok_or("response UUID missing")?;
    Uuid::from_slice(bytes.bytes()).map_err(|_| "response UUID extent")
}
pub(super) fn correlation(root: fb::Value<'_>) -> Result<Target, Refusal> {
    let object = root.value_as_object().ok_or("response envelope is not an object")?;
    let items = object.items().ok_or("response object has no fields")?;
    if items.len() > 8 { return Err("response envelope field limit") }
    // Pinned handle_db_response ignores all sessionless/global responses. Keep
    // that behavior before owned conversion, even when their result is large.
    let mut raw_session = None;
    for entry in items {
        if entry.key() == Some("session") {
            if raw_session.is_some() { return Err("duplicate response session") }
            raw_session = entry.value();
        }
    }
    let Some(raw_session) = raw_session else { return Ok(Target::Discard) };
    if matches!(raw_session.value_type(), fb::ValueType::NONE | fb::ValueType::None | fb::ValueType::Null) {
        return Ok(Target::Discard);
    }
    let session = uuid(raw_session)?;
    let mut id = None;
    let mut result = None;
    let mut error = false;
    for entry in items {
        match entry.key() {
            Some("id") => {
                if id.is_some() { return Err("duplicate response id") }
                id = Some(entry.value().and_then(|v| v.value_as_int_64())
                    .ok_or("response id is not a signed integer")?.value());
            }
            Some("session") => {}
            Some("result") => {
                if result.is_some() { return Err("duplicate response result") }
                result = Some(entry.value().ok_or("response result missing")?);
            }
            Some("error") => {
                if error { return Err("duplicate response error") }
                error = true;
            }
            _ => return Err("unknown response envelope field"),
        }
    }
    if let Some(id) = id { return Ok(Target::Reply { session, id }) }
    // Pinned DbResult::Live grammar: no envelope id, and a notification object
    // with query UUID, action, original record and result. Arbitrary scientific
    // Value variants in record/result remain supported by the native decoder.
    let live = result.and_then(|v| v.value_as_object()).and_then(|v| v.items())
        .ok_or("uncorrelated response is not a live notification")?;
    if error || live.len() > 5 { return Err("invalid live notification envelope") }
    let mut query = None;
    let mut action = false;
    let mut record = false;
    let mut payload = false;
    let mut live_session = false;
    for entry in live {
        match entry.key() {
            Some("id") if query.is_none() => query = Some(uuid(entry.value().ok_or("live id missing")?)?),
            Some("action") if !action => {
                let text = entry.value().and_then(|v| v.value_as_string()).and_then(|v| v.value())
                    .ok_or("live action missing")?;
                if !matches!(text, "CREATE" | "UPDATE" | "DELETE") { return Err("live action invalid") }
                action = true;
            }
            Some("record") if !record => record = entry.value().is_some(),
            Some("result") if !payload => payload = entry.value().is_some(),
            Some("session") if !live_session => live_session = true,
            _ => return Err("duplicate or unknown live notification field"),
        }
    }
    if !action || !record || !payload { return Err("live notification fields missing") }
    Ok(Target::Live { session, query: query.ok_or("live query id missing")? })
}

/// Generic SDK LIVE supports every native Value variant. Its installed query
/// owner uses the socket's finite frame limit plus a finite expanded verifier
/// extent; the canonical metadata route never uses this exception.
pub(super) fn validate_live(binary: &[u8], wire_bytes: usize) -> Result<(), Refusal> {
    if binary.len() > wire_bytes { return Err("live response wire extent") }
    let options = flatbuffers::VerifierOptions {
        max_tables: wire_bytes,
        max_depth: 512,
        max_apparent_size: wire_bytes.saturating_mul(16),
        ..Default::default()
    };
    flatbuffers::root_with_opts::<fb::Value>(&options, binary)
        .map(|_| ()).map_err(|_| "live response expanded extent")
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
    fn correlation_refuses_missing_malformed_and_duplicate_ids() {
        use surrealdb_types::{Object, Value};
        let mut envelope = Object::new();
        envelope.insert("session", Value::Uuid(Uuid::new_v4().into()));
        envelope.insert("result", Value::String("unsolicited".into()));
        let bytes = surrealdb_types::encode(&Value::Object(envelope.clone())).unwrap();
        assert!(correlation(root(&bytes).unwrap()).is_err());
        envelope.insert("id", Value::String("not a request integer".into()));
        let bytes = surrealdb_types::encode(&Value::Object(envelope)).unwrap();
        assert!(correlation(root(&bytes).unwrap()).is_err());
        let mut builder = flatbuffers::FlatBufferBuilder::new();
        let id_text = builder.create_string("id");
        let integer = fb::Int64Value::create(&mut builder, &fb::Int64ValueArgs { value: 7 });
        let value = fb::Value::create(&mut builder, &fb::ValueArgs {
            value_type: fb::ValueType::Int64, value: Some(integer.as_union_value()),
        });
        let field = fb::KeyValue::create(&mut builder, &fb::KeyValueArgs { key: Some(id_text), value: Some(value) });
        let fields = builder.create_vector(&[field, field]);
        let object = fb::Object::create(&mut builder, &fb::ObjectArgs { items: Some(fields) });
        let value = fb::Value::create(&mut builder, &fb::ValueArgs {
            value_type: fb::ValueType::Object, value: Some(object.as_union_value()),
        });
        builder.finish_minimal(value);
        assert_eq!(correlation(root(builder.finished_data()).unwrap()), Err("duplicate response id"));
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
