// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Payload-only borrowed projection and record budgets, never JSON byte truncation.
use super::{DetailView, Envelope};
use clap::ValueEnum;
use schemars::{JsonSchema, Schema, generate::SchemaSettings};
use serde::Serialize;
use serde_json::{Value, value::RawValue};
use std::{
    collections::BTreeMap,
    io::{self, Write},
};

/// Rendering format selected before argument parsing finishes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    /// Human for a terminal, JSON for pipes.
    #[default]
    Auto,
    /// Concise summary and pretty structured detail.
    Human,
    /// One JSON envelope.
    Json,
    /// Compact normalized records followed by an observation summary.
    Ndjson,
}
impl OutputFormat {
    /// Resolve only terminal presentation; this does not inspect or prepare a service.
    pub fn resolve(self, terminal: bool) -> Self {
        if self == Self::Auto {
            if terminal { Self::Human } else { Self::Json }
        } else {
            self
        }
    }
    /// Read format before clap can report a usage failure, stopping at literal argv tails.
    pub fn early(
        arguments: &[std::ffi::OsString],
        environment: Option<&str>,
        terminal: bool,
    ) -> Self {
        let parse = |value: &str| Self::from_str(value, true).ok();
        let mut selected = environment.and_then(parse).unwrap_or_default();
        let mut args = arguments.iter().peekable();
        while let Some(arg) = args.next() {
            if arg == "--" {
                break;
            }
            if arg == "--format" {
                if let Some(value) = args.next().and_then(|value| value.to_str()).and_then(parse) {
                    selected = value;
                }
            } else if let Some(value) = arg
                .to_str()
                .and_then(|value| value.strip_prefix("--format="))
                .and_then(parse)
            {
                selected = value;
            }
        }
        selected.resolve(terminal)
    }
}
/// Requested detail view. Fields use literal top-level payload names.
#[derive(Clone, Debug, Default)]
pub struct Presentation {
    /// Selected payload fields; mandatory envelope fields are never selectable here.
    pub fields: Option<Vec<String>>,
    /// Maximum payload array records, shared in stable field order.
    pub budget: Option<usize>,
}
fn invalid(detail: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, detail.into())
}
pub(super) fn identity(command: &str, view: &Presentation) -> String {
    let command = command.replace(' ', "/");
    let fields = view.fields.as_ref().map_or("full".into(), |fields| {
        format!("fields:{}", fields.join(","))
    });
    format!(
        "urn:pse:cli:v1:{command}:{fields}:budget:{}",
        view.budget
            .map_or("unbounded".into(), |value| value.to_string())
    )
}
fn dereference(value: &Value, root: &Schema) -> Value {
    value
        .get("$ref")
        .and_then(Value::as_str)
        .filter(|reference| reference.starts_with('#'))
        .and_then(|reference| root.pointer(reference))
        .unwrap_or(value)
        .clone()
}
fn limit_array(schema: &mut Value, budget: usize) {
    let maximum = schema
        .get("maxItems")
        .and_then(Value::as_u64)
        .map_or(budget, |value| {
            budget.min(usize::try_from(value).unwrap_or(usize::MAX))
        });
    schema["maxItems"] = maximum.into();
    // A prefix view is a different contract from a complete fixed-size array.
    // Other fields may already have consumed the shared record allowance.
    if let Some(object) = schema.as_object_mut() {
        object.remove("minItems");
    }
    if let Some(prefix) = schema.get_mut("prefixItems").and_then(Value::as_array_mut) {
        prefix.truncate(maximum);
    }
}
/// Serialization-direction schema for the exact full/projected view.
pub fn schema<T: JsonSchema>(command: &str, view: &Presentation) -> io::Result<Schema> {
    let mut schema = SchemaSettings::draft2020_12()
        .for_serialize()
        .into_generator()
        .into_root_schema_for::<Envelope<T>>();
    let payload = schema
        .pointer("/properties/payload")
        .ok_or_else(|| invalid("envelope payload schema missing"))?;
    let mut payload = dereference(payload, &schema);
    if let Some(fields) = &view.fields {
        if payload.get("properties").is_none() {
            let field_type = payload
                .get("additionalProperties")
                .cloned()
                .unwrap_or(Value::Bool(true));
            payload = serde_json::json!({"type":"object","properties":fields.iter().map(|name|(name.clone(),field_type.clone())).collect::<serde_json::Map<_,_>>(),"required":fields,"additionalProperties":false});
        }
        let properties = payload
            .get_mut("properties")
            .and_then(Value::as_object_mut)
            .ok_or_else(|| invalid("--fields requires an object payload"))?;
        for field in fields {
            if !properties.contains_key(field) {
                return Err(invalid(format!("unknown payload field {field}")));
            }
        }
        properties.retain(|name, _| fields.contains(name));
        if let Some(required) = payload.get_mut("required").and_then(Value::as_array_mut) {
            required.retain(|name| {
                name.as_str()
                    .is_some_and(|name| fields.iter().any(|field| field == name))
            });
        }
        if let Some(object) = payload.as_object_mut() {
            object.insert("additionalProperties".into(), Value::Bool(false));
        }
    }
    if let Some(budget) = view.budget {
        if payload.get("type").and_then(Value::as_str) == Some("array") {
            limit_array(&mut payload, budget);
        }
        if let Some(properties) = payload.get_mut("properties").and_then(Value::as_object_mut) {
            for value in properties.values_mut() {
                let mut resolved = dereference(value, &schema);
                if resolved.get("type").and_then(Value::as_str) == Some("array") {
                    limit_array(&mut resolved, budget);
                    *value = resolved;
                }
            }
        }
    }
    schema.insert("$id".into(), identity(command, view).into());
    if let Some(value) = schema.pointer_mut("/properties/payload") {
        *value = payload;
    }
    Ok(schema)
}
#[derive(Serialize)]
#[serde(untagged)]
enum Presented<'a> {
    Object(BTreeMap<String, Presented<'a>>),
    Array(Vec<&'a RawValue>),
    Raw(&'a RawValue),
}
struct UniqueFields<'a>(BTreeMap<String, &'a RawValue>);
impl<'de> serde::Deserialize<'de> for UniqueFields<'de> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = UniqueFields<'de>;
            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("an object with unambiguous field names for projection")
            }
            fn visit_map<M: serde::de::MapAccess<'de>>(
                self,
                mut access: M,
            ) -> Result<Self::Value, M::Error> {
                let mut fields = BTreeMap::new();
                while let Some((name, value)) = access.next_entry::<String, &RawValue>()? {
                    if fields.insert(name.clone(), value).is_some() {
                        return Err(serde::de::Error::custom(format!(
                            "duplicate payload field {name}; a projected view would be ambiguous"
                        )));
                    }
                }
                Ok(UniqueFields(fields))
            }
        }
        deserializer.deserialize_map(Visitor)
    }
}
fn records<'a>(
    raw: &'a RawValue,
    remaining: &mut Option<usize>,
    omitted: &mut usize,
) -> serde_json::Result<Presented<'a>> {
    if raw.get().starts_with('[')
        && let Some(remaining) = remaining
    {
        let mut values: Vec<&RawValue> = serde_json::from_str(raw.get())?;
        let count = values.len().min(*remaining);
        *omitted += values.len() - count;
        values.truncate(count);
        *remaining -= count;
        Ok(Presented::Array(values))
    } else {
        Ok(Presented::Raw(raw))
    }
}
fn present<'a>(raw: &'a RawValue, view: &Presentation) -> io::Result<(Presented<'a>, DetailView)> {
    let mut detail = DetailView {
        fields: view.fields.clone(),
        budget: view.budget,
        ..DetailView::default()
    };
    let mut remaining = view.budget;
    if view.fields.is_none() && view.budget.is_none() {
        return Ok((Presented::Raw(raw), detail));
    }
    let value = if raw.get().starts_with('{') {
        let UniqueFields(fields) = serde_json::from_str(raw.get())?;
        if let Some(requested) = &view.fields {
            for field in requested {
                if !fields.contains_key(field) {
                    return Err(invalid(format!("unknown payload field {field}")));
                }
            }
        }
        let mut presented = BTreeMap::new();
        for (name, value) in fields {
            if view
                .fields
                .as_ref()
                .is_none_or(|fields| fields.iter().any(|field| field == &name))
            {
                presented.insert(
                    name,
                    records(value, &mut remaining, &mut detail.omitted_records)?,
                );
            } else {
                detail.omitted_fields.push(name);
            }
        }
        Presented::Object(presented)
    } else {
        if view.fields.is_some() {
            return Err(invalid("--fields requires an object payload"));
        }
        records(raw, &mut remaining, &mut detail.omitted_records)?
    };
    detail.truncated = !detail.omitted_fields.is_empty() || detail.omitted_records > 0;
    Ok((value, detail))
}
/// Raw fragments retain their lexical data but whitespace outside strings is removed.
/// RawValue already validates JSON; this formatter neither parses nor reinterprets numbers.
struct LineFormatter;
impl serde_json::ser::Formatter for LineFormatter {
    fn write_raw_fragment<W: ?Sized + Write>(
        &mut self,
        writer: &mut W,
        fragment: &str,
    ) -> io::Result<()> {
        let mut quoted = false;
        let mut escaped = false;
        for byte in fragment.bytes() {
            if quoted {
                writer.write_all(&[byte])?;
                if escaped {
                    escaped = false;
                } else if byte == b'\\' {
                    escaped = true;
                } else if byte == b'"' {
                    quoted = false;
                }
            } else if byte == b'"' {
                quoted = true;
                writer.write_all(&[byte])?;
            } else if !byte.is_ascii_whitespace() {
                writer.write_all(&[byte])?;
            }
        }
        Ok(())
    }
}
/// Write complete JSON records. Failure never publishes a truncated JSON string as a result.
pub fn write<T: Serialize + JsonSchema>(
    writer: impl Write,
    envelope: Envelope<T>,
    format: OutputFormat,
    view: &Presentation,
) -> io::Result<i32> {
    let prepared = prepare::<T>(&envelope.command, view)?;
    write_prepared(writer, envelope, format, prepared)
}
/// A type-checked view prepared before any retained response is published.
pub(super) struct PreparedView<'a, T> {
    view: &'a Presentation,
    payload: std::marker::PhantomData<fn() -> T>,
}
pub(super) fn prepare<'a, T: JsonSchema>(
    command: &str,
    view: &'a Presentation,
) -> io::Result<PreparedView<'a, T>> {
    // Use the same declaration as schema output, once per response.
    let _ = schema::<T>(command, view)?;
    Ok(PreparedView {
        view,
        payload: std::marker::PhantomData,
    })
}
pub(super) fn write_prepared<T: Serialize>(
    mut writer: impl Write,
    envelope: Envelope<T>,
    format: OutputFormat,
    prepared: PreparedView<'_, T>,
) -> io::Result<i32> {
    let view = prepared.view;
    let code = envelope.exit_code();
    let raw = serde_json::value::to_raw_value(&envelope.payload)?;
    let (payload, detail) = present(&raw, view)?;
    let mut envelope = envelope.with_payload(payload);
    envelope.view = detail;
    envelope.schema = identity(&envelope.command, view);
    // Serialize a complete response before publication. I/O can still fail mid-write;
    // no serializer failure can publish a syntactically partial response as evidence.
    let mut frame = Vec::new();
    match format {
        OutputFormat::Human => {
            writeln!(frame, "{}", envelope.summary)?;
            serde_json::to_writer_pretty(&mut frame, &envelope)?;
            writeln!(frame)?;
        }
        OutputFormat::Ndjson => {
            envelope.record = super::RecordKind::ObservationSummary;
            let mut serializer = serde_json::Serializer::with_formatter(&mut frame, LineFormatter);
            envelope.serialize(&mut serializer)?;
            writeln!(frame)?;
        }
        OutputFormat::Auto | OutputFormat::Json => {
            serde_json::to_writer(&mut frame, &envelope)?;
            writeln!(frame)?;
        }
    }
    writer.write_all(&frame)?;
    Ok(code)
}
