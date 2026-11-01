// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Passive payload interpretation in one explicitly selected descriptor world.
use super::{Bundle, SchemaError};
use crate::investigation::NativePath;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use buffa_descriptor::{DynamicMessage, ReflectMessage, ValueRef};
use clap::{Args as ClapArgs, ValueEnum};
use protovalidate_buffa::Validate;
use schemars::JsonSchema;
use serde::{
    Deserialize, Serialize,
    de::{self, DeserializeOwned, Visitor},
};
use sha2::{Digest as _, Sha256};
use std::{
    collections::BTreeSet,
    fmt,
    io::{Read as _, Seek as _, Write as _},
    path::PathBuf,
    process::{Command, Stdio},
    sync::Arc,
    time::{Duration, Instant},
};

/// Interpretation grammar, never inferred from a filename.
#[derive(Clone, Copy, Debug, ValueEnum, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Encoding {
    /// Protocol buffer wire bytes.
    Binary,
    /// Schema-qualified protobuf JSON.
    Json,
    /// Protobuf text format parsed by native protoc.
    Textproto,
}
/// Explicit passive payload inspection.
#[derive(Debug, ClapArgs)]
pub struct Args {
    /// Complete descriptor-set artifact.
    #[arg(long)]
    pub descriptor: PathBuf,
    /// Fully qualified message within that artifact.
    #[arg(long)]
    pub message: String,
    /// Original payload artifact.
    #[arg(long)]
    pub input: PathBuf,
    /// Explicit input grammar.
    #[arg(long, value_enum)]
    pub encoding: Encoding,
    /// Nested message expansion depth.
    #[arg(long, default_value_t = 2)]
    pub depth: usize,
    /// Maximum members shown per message or collection.
    #[arg(long, default_value_t = 20)]
    pub limit: usize,
    /// Include the exact original input bytes as base64.
    #[arg(long)]
    pub original: bool,
}
/// Decode and validation are separate outcomes.
#[derive(Debug, Serialize, JsonSchema)]
pub struct Validation {
    /// passed, violations, compile_error, evaluation_error, or not_evaluated.
    pub state: String,
    /// Native validator diagnostics, when evaluated.
    pub diagnostics: Vec<String>,
}
/// A field read through its message's own reflection descriptors.
#[derive(Debug, Serialize, JsonSchema)]
pub struct Field {
    /// Authored field number.
    pub number: u32,
    /// Authored field name.
    pub name: String,
    /// Descriptor kind in the selected world.
    pub kind: String,
    /// Explicit, implicit or legacy required presence from the selected descriptor.
    pub presence: String,
    /// Fully qualified message or enum type, where applicable.
    pub type_name: Option<String>,
    /// Semantic presence; false is distinct from a selected explicit default.
    pub present: bool,
    /// Default or observed value; containers may be deferred by depth.
    pub value: Preview,
}
/// A bounded view, never a replacement for the original payload.
#[derive(Debug, Serialize, JsonSchema)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum Preview {
    /// Boolean scalar.
    Bool(bool),
    /// Signed integer as exact decimal text.
    Signed(String),
    /// Unsigned integer as exact decimal text.
    Unsigned(String),
    /// Float text, including non-finite values.
    Float(String),
    /// String, shortened only at a character boundary.
    String {
        /// Visible prefix.
        text: String,
        /// Omitted Unicode scalar count.
        omitted: usize,
    },
    /// Bytes retain their base64 representation.
    Bytes {
        /// Visible base64 prefix.
        base64: String,
        /// Original byte length.
        bytes: usize,
        /// Unshown bytes.
        omitted: usize,
    },
    /// Enum number remains meaningful even when unnamed.
    Enum(i32),
    /// Nested message fields and wire unknowns.
    Message {
        /// Visible fields.
        fields: Vec<Field>,
        /// Unshown fields.
        omitted: usize,
        /// Original unknown-wire observations.
        unknown: Vec<Unknown>,
        /// Unshown unknown-wire observations.
        unknown_omitted: usize,
    },
    /// Repeated elements in order.
    List {
        /// Visible elements.
        items: Vec<Preview>,
        /// Unshown elements.
        omitted: usize,
    },
    /// Map entries; native map order has no semantic meaning.
    Map {
        /// Visible entries.
        entries: Vec<MapEntry>,
        /// Unshown entries.
        omitted: usize,
    },
    /// Expansion requested less depth than this value requires.
    Deferred,
}
/// One exact map key and its interpreted value.
#[derive(Debug, Serialize, JsonSchema)]
pub struct MapEntry {
    /// Native reflection key representation.
    pub key: String,
    /// Corresponding value.
    pub value: Preview,
}
/// Wire data unrecognized by this selected schema.
#[derive(Debug, Serialize, JsonSchema)]
pub struct Unknown {
    /// Observed field number, without an invented name.
    pub number: u32,
    /// Native wire kind.
    pub wire_kind: String,
    /// Encoded size of tag and value.
    pub encoded_bytes: usize,
}
/// Payload interpretation with mandatory original identities and coverage.
#[derive(Debug, Serialize, JsonSchema)]
pub struct Report {
    /// Selected descriptor artifact path.
    pub descriptor: NativePath,
    /// Identity of the complete original descriptor bytes.
    pub descriptor_sha256: String,
    /// Original payload path; does not claim continued custody of a mutable file.
    pub input: NativePath,
    /// Identity of bytes actually read.
    pub payload_sha256: String,
    /// Length of bytes actually read.
    pub payload_bytes: usize,
    /// Exact replay bytes, only on explicit expansion.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_base64: Option<String>,
    /// Message identity within the selected descriptor world.
    pub message: String,
    /// Selected input grammar.
    pub encoding: Encoding,
    /// generated_static or acquired_dynamic.
    pub representation: String,
    /// Semantic validation coverage.
    pub validation: Validation,
    /// Bounded interpreted fields.
    pub preview: Preview,
    /// Original successful text-parser warnings, if any.
    pub parser_diagnostics: String,
    /// Limits on interpretation and preservation.
    pub limitations: Vec<String>,
}

/// Read selected artifacts once, decode passively and preserve their identities.
pub fn inspect(args: &Args) -> Result<Report, SchemaError> {
    if args.limit == 0 || args.depth > 64 {
        return Err(SchemaError::Invalid(
            "payload limit must be positive and display depth at most 64".into(),
        ));
    }
    let bundle = Bundle::load(&args.descriptor)?;
    let input_path = std::fs::canonicalize(&args.input)?;
    let input = std::fs::read(&input_path)?;
    let index = bundle.pool.message_index(&args.message).ok_or_else(|| {
        SchemaError::Invalid(format!(
            "message absent from selected descriptor: {}",
            args.message
        ))
    })?;
    if matches!(args.encoding, Encoding::Json) {
        strict_json(&input)?;
        if bundle.compiled_world() {
            let json = serde_json::from_slice(&input)?;
            check_keys(&bundle.pool, bundle.pool.message(index), &json)?;
        }
    }
    let (wire, parser_diagnostics) = if matches!(args.encoding, Encoding::Textproto) {
        text_wire(&bundle, &args.message, &input)?
    } else {
        (input.clone(), String::new())
    };
    let static_value = if bundle.compiled_world() {
        compiled(&args.message, &wire, args.encoding, args.depth, args.limit)?
    } else {
        None
    };
    let (preview, validation, representation) = if let Some((preview, validation)) = static_value {
        (preview, validation, "generated_static")
    } else {
        let value = if matches!(args.encoding, Encoding::Json) {
            let json =
                std::str::from_utf8(&wire).map_err(|e| SchemaError::Decode(e.to_string()))?;
            DynamicMessage::from_json(Arc::clone(&bundle.pool), index, json)
                .map_err(|e| SchemaError::Decode(e.to_string()))?
        } else {
            DynamicMessage::decode(Arc::clone(&bundle.pool), index, &wire)
                .map_err(|e| SchemaError::Decode(e.to_string()))?
        };
        (message_preview(&value, args.depth, args.limit), Validation { state: "not_evaluated".into(), diagnostics: vec!["Arbitrary acquired schemas have no compiled validator; constraints are metadata, not an established valid payload.".into()] }, "acquired_dynamic")
    };
    Ok(Report {
        descriptor: bundle.path.clone().into(), descriptor_sha256: bundle.digest.clone(),
        input: input_path.into(), payload_sha256: Sha256::digest(&input).iter().map(|byte| format!("{byte:02x}")).collect(), payload_bytes: input.len(),
        original_base64: args.original.then(|| STANDARD.encode(&input)), message: args.message.clone(), encoding: args.encoding,
        representation: representation.into(), validation, preview, parser_diagnostics,
        limitations: vec!["Successful decoding does not establish semantic validity. Re-encoding does not preserve original byte identity or wire ordering.".into(), "Presence follows protobuf semantics; implicit scalar default and absence may be indistinguishable. Display limits do not limit decode coverage.".into(), "Any resolves only within the selected descriptor closure; nested payload bytes remain bytes. Original path/digest does not guarantee future availability; --original includes exact input.".into()],
    })
}

fn compiled(
    name: &str,
    input: &[u8],
    encoding: Encoding,
    depth: usize,
    limit: usize,
) -> Result<Option<(Preview, Validation)>, SchemaError> {
    use crate::code_intelligence_contracts::pb::pse::code_intelligence::v1 as pb;
    macro_rules! dispatch { ($($ty:ident),+ $(,)?) => { match name { $(concat!("pse.code_intelligence.v1.", stringify!($ty)) => Some(static_preview::<pb::$ty>(input, encoding, depth, limit)),)+ _ => None }.transpose() }; }
    dispatch!(
        ContextGuard,
        SourceSelector,
        Predicate,
        BooleanGroup,
        Negation,
        TextQuery,
        SyntaxQuery,
        SymbolQuery,
        ApiQuery,
        StructuredQuery,
        HistoryQuery,
        GuardedReference,
        Related,
        Refinement,
        Refine,
        AnswerDemand,
        Request,
        Count,
        Location,
        OriginalEvidence,
        SourceFact,
        ApiFact,
        ApiDefinition,
        PredicateFact,
        Failure,
        NativeCancellationObservation,
        Observation,
        Terminal,
        Control,
        Event,
        NativeQuery,
        NativeImportTarget,
        NativeWorkspaceQuery,
        NativeRequest,
        NativeConfiguration,
        NativeAnalysisContext,
        NativeSessionStatus,
        NativeLibraryProfile,
        NativeSourceHash,
        NativePackageContext,
        NativePackageRequest,
        NativePackageContextAnswer,
        NativePackageReply,
        NativePosition,
        NativeRange,
        NativeByteLocation,
        NativeProtocolLocation,
        NativeLocation,
        NativeSymbolFact,
        NativeRelationshipFact,
        NativePredicateFact,
        NativeImportabilityFact,
        NativeLazyCandidate,
        NativeTestCandidate,
        NativeObservedFacts,
        NativeOmittedRelationship,
        NativeSection,
        NativeCandidateHandle,
        NativeSourceIdentity,
        NativeTypeFact,
        NativeTargetFact,
        NativeBoundFact,
        NativeTypedFacts,
        NativeProducerContext,
        NativeEditCandidate,
        NativeFileOperation,
        NativeConflict,
        NativePlaceholder,
        NativePreview,
        NativeSourceText,
        NativeAnswer,
        NativeWorkspaceAnswer,
        NativeControl,
        NativeEvent
    )
}
fn static_preview<T: buffa::Message + DeserializeOwned + ReflectMessage + Validate>(
    input: &[u8],
    encoding: Encoding,
    depth: usize,
    limit: usize,
) -> Result<(Preview, Validation), SchemaError> {
    let value: T = if matches!(encoding, Encoding::Json) {
        serde_json::from_slice(input)?
    } else {
        T::decode_from_slice(input).map_err(|e| SchemaError::Decode(e.to_string()))?
    };
    let validation = match value.validate() {
        Ok(()) => Validation {
            state: "passed".into(),
            diagnostics: vec![],
        },
        Err(error) => Validation {
            state: if error.compile_error.is_some() {
                "compile_error"
            } else if error.runtime_error.is_some() {
                "evaluation_error"
            } else {
                "violations"
            }
            .into(),
            diagnostics: vec![error.to_string()],
        },
    };
    Ok((message_preview(&value, depth, limit), validation))
}
fn message_preview(message: &dyn ReflectMessage, depth: usize, limit: usize) -> Preview {
    if depth == 0 {
        return Preview::Deferred;
    }
    let descriptor = message.message_descriptor();
    let fields = descriptor
        .fields()
        .iter()
        .take(limit)
        .map(|field| Field {
            number: field.number(),
            name: field.name().into(),
            kind: format!("{:?}", field.kind()),
            presence: format!("{:?}", field.presence()),
            type_name: type_name(message.pool(), field.kind()),
            present: message.has(field),
            value: preview(message.get(field), depth - 1, limit),
        })
        .collect();
    let unknown = message
        .unknown_fields()
        .iter()
        .take(limit)
        .map(|field| Unknown {
            number: field.number,
            wire_kind: match &field.data {
                buffa::UnknownFieldData::Varint(_) => "varint",
                buffa::UnknownFieldData::Fixed64(_) => "fixed64",
                buffa::UnknownFieldData::Fixed32(_) => "fixed32",
                buffa::UnknownFieldData::LengthDelimited(_) => "length_delimited",
                buffa::UnknownFieldData::Group(_) => "group",
            }
            .into(),
            encoded_bytes: field.encoded_len(),
        })
        .collect();
    Preview::Message {
        fields,
        omitted: descriptor.fields().len().saturating_sub(limit),
        unknown,
        unknown_omitted: message.unknown_fields().len().saturating_sub(limit),
    }
}
fn type_name(
    pool: &buffa_descriptor::DescriptorPool,
    kind: buffa_descriptor::FieldKind,
) -> Option<String> {
    use buffa_descriptor::{FieldKind, SingularKind};
    let kind = match kind {
        FieldKind::Singular(kind) | FieldKind::List(kind) => kind,
        FieldKind::Map { value, .. } => value,
    };
    match kind {
        SingularKind::Message(index) => Some(pool.message(index).full_name().into()),
        SingularKind::Enum(index) => Some(pool.enumeration(index).full_name().into()),
        SingularKind::Scalar(_) => None,
    }
}
fn preview(value: ValueRef<'_>, depth: usize, limit: usize) -> Preview {
    match value {
        ValueRef::Bool(v) => Preview::Bool(v),
        ValueRef::I32(v) => Preview::Signed(v.to_string()),
        ValueRef::I64(v) => Preview::Signed(v.to_string()),
        ValueRef::U32(v) => Preview::Unsigned(v.to_string()),
        ValueRef::U64(v) => Preview::Unsigned(v.to_string()),
        ValueRef::F32(v) => Preview::Float(v.to_string()),
        ValueRef::F64(v) => Preview::Float(v.to_string()),
        ValueRef::EnumNumber(v) => Preview::Enum(v),
        ValueRef::String(v) => Preview::String {
            text: v.chars().take(1024).collect(),
            omitted: v.chars().count().saturating_sub(1024),
        },
        ValueRef::Bytes(v) => Preview::Bytes {
            base64: STANDARD.encode(&v[..v.len().min(1024)]),
            bytes: v.len(),
            omitted: v.len().saturating_sub(1024),
        },
        ValueRef::Message(v) => message_preview(&*v, depth, limit),
        ValueRef::List(v) => Preview::List {
            items: (0..v.len().min(limit))
                .filter_map(|i| v.get(i))
                .map(|v| preview(v, depth, limit))
                .collect(),
            omitted: v.len().saturating_sub(limit),
        },
        ValueRef::Map(v) => {
            let mut entries = vec![];
            v.for_each(&mut |key, value| {
                if entries.len() < limit {
                    entries.push(MapEntry {
                        key: format!("{key:?}"),
                        value: preview(value, depth, limit),
                    });
                }
            });
            Preview::Map {
                entries,
                omitted: v.len().saturating_sub(limit),
            }
        }
    }
}

// A serde visitor checks the original token stream before Any's JSON map conversion
// can collapse duplicate members. No second JSON value tree is materialized here.
struct UniqueJson;
impl<'de> Deserialize<'de> for UniqueJson {
    fn deserialize<D: de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(Self)
    }
}
impl<'de> Visitor<'de> for UniqueJson {
    type Value = Self;
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("JSON without duplicate object members")
    }
    fn visit_bool<E: de::Error>(self, _: bool) -> Result<Self, E> {
        Ok(Self)
    }
    fn visit_i64<E: de::Error>(self, _: i64) -> Result<Self, E> {
        Ok(Self)
    }
    fn visit_u64<E: de::Error>(self, _: u64) -> Result<Self, E> {
        Ok(Self)
    }
    fn visit_f64<E: de::Error>(self, _: f64) -> Result<Self, E> {
        Ok(Self)
    }
    fn visit_str<E: de::Error>(self, _: &str) -> Result<Self, E> {
        Ok(Self)
    }
    fn visit_unit<E: de::Error>(self) -> Result<Self, E> {
        Ok(Self)
    }
    fn visit_seq<A: de::SeqAccess<'de>>(self, mut sequence: A) -> Result<Self, A::Error> {
        while sequence.next_element::<Self>()?.is_some() {}
        Ok(Self)
    }
    fn visit_map<A: de::MapAccess<'de>>(self, mut map: A) -> Result<Self, A::Error> {
        let mut keys = BTreeSet::new();
        while let Some(key) = map.next_key::<String>()? {
            if !keys.insert(key.clone()) {
                return Err(de::Error::custom(format!("duplicate JSON member: {key}")));
            }
            map.next_value::<Self>()?;
        }
        Ok(Self)
    }
}
fn strict_json(bytes: &[u8]) -> Result<(), SchemaError> {
    let mut decoder = serde_json::Deserializer::from_slice(bytes);
    UniqueJson::deserialize(&mut decoder)?;
    decoder.end()?;
    Ok(())
}

// The generated Serde reader ignores unknown names. Validate names against the
// exact compiled schema first, while still decoding and projecting generated
// messages directly rather than transcoding them through DynamicMessage.
fn check_keys(
    pool: &buffa_descriptor::DescriptorPool,
    message: &buffa_descriptor::MessageDescriptor,
    json: &serde_json::Value,
) -> Result<(), SchemaError> {
    use buffa_descriptor::{FieldKind, SingularKind};
    if matches!(
        message.full_name(),
        "google.protobuf.Struct" | "google.protobuf.Value" | "google.protobuf.ListValue"
    ) {
        return Ok(());
    }
    let Some(object) = json.as_object() else {
        return Ok(());
    };
    let mut numbers = BTreeSet::new();
    for (key, value) in object {
        let field = message
            .fields()
            .iter()
            .find(|f| f.name() == key || f.json_name() == key)
            .ok_or_else(|| {
                SchemaError::Decode(format!("unknown JSON field {}.{key}", message.full_name()))
            })?;
        if !numbers.insert(field.number()) {
            return Err(SchemaError::Decode(format!(
                "duplicate JSON field aliases: {}.{key}",
                message.full_name()
            )));
        }
        let nested = |kind: SingularKind, value: &serde_json::Value| -> Result<(), SchemaError> {
            if let SingularKind::Message(index) = kind {
                check_keys(pool, pool.message(index), value)?;
            }
            Ok(())
        };
        match field.kind() {
            FieldKind::Singular(kind) => nested(kind, value)?,
            FieldKind::List(kind) => {
                if let Some(items) = value.as_array() {
                    for item in items {
                        nested(kind, item)?;
                    }
                }
            }
            FieldKind::Map { value: kind, .. } => {
                if let Some(items) = value.as_object() {
                    for item in items.values() {
                        nested(kind, item)?;
                    }
                }
            }
        }
    }
    Ok(())
}

fn text_wire(
    bundle: &Bundle,
    message: &str,
    input: &[u8],
) -> Result<(Vec<u8>, String), SchemaError> {
    use pse_diagnostics::{DrainFailurePolicy, StderrCapture, SupervisedChild};
    // Private snapshots ensure descriptor and payload identities cannot race their
    // original mutable paths. Regular-file stdio requires no independent pipe task.
    let mut descriptor = tempfile::NamedTempFile::new()?;
    descriptor.write_all(&bundle.original)?;
    let mut original = tempfile::tempfile()?;
    original.write_all(input)?;
    original.rewind()?;
    let mut binary = tempfile::tempfile()?;
    let mut command = Command::new("protoc");
    let mut descriptor_argument = std::ffi::OsString::from("--descriptor_set_in=");
    descriptor_argument.push(descriptor.path());
    command
        .arg(descriptor_argument)
        .arg(format!("--encode={message}"))
        .stdin(Stdio::from(original))
        .stdout(Stdio::from(binary.try_clone()?));
    let mut child = SupervisedChild::spawn(
        &mut command,
        DrainFailurePolicy::Park,
        Some(StderrCapture {
            bytes: 65536,
            stack_bytes: 256 * 1024,
        }),
    )
    .map_err(|e| SchemaError::Tool(e.to_string()))?;
    let wait = child.wait_ready(None, Some(Instant::now() + Duration::from_secs(30)));
    let settled = child.cancel();
    let diagnostics =
        String::from_utf8_lossy(&child.take_stderr_prefix().unwrap_or_default()).into_owned();
    if let Err(error) = wait {
        return Err(SchemaError::Tool(format!(
            "protoc: {error}; settlement={settled:?}; {diagnostics}"
        )));
    }
    let status =
        settled.map_err(|e| SchemaError::Tool(format!("protoc settlement: {e}; {diagnostics}")))?;
    if !status.success() {
        return Err(SchemaError::Tool(format!("protoc {status}: {diagnostics}")));
    }
    binary.rewind()?;
    let mut bytes = Vec::new();
    binary.read_to_end(&mut bytes)?;
    Ok((bytes, diagnostics))
}

#[cfg(test)]
mod tests {
    use super::*;
    const NAME: &str = "plan61.qualification.v1.Payload";
    fn args(bytes: &[u8], encoding: Encoding) -> (tempfile::TempDir, Args) {
        let directory = tempfile::tempdir().unwrap();
        let descriptor = directory.path().join("schema.bin");
        let input = directory.path().join("payload");
        std::fs::write(
            &descriptor,
            include_bytes!("../generated/plan61/descriptor.bin"),
        )
        .unwrap();
        std::fs::write(&input, bytes).unwrap();
        (
            directory,
            Args {
                descriptor,
                message: NAME.into(),
                input,
                encoding,
                depth: 3,
                limit: 20,
                original: true,
            },
        )
    }
    #[test]
    fn schema_payload_binary_preserves_presence_unknowns_and_original() {
        let (_directory, mut args) = args(&[8, 0, 24, 99, 0x98, 6, 7], Encoding::Binary);
        let report = inspect(&args).unwrap();
        assert_eq!(
            report.original_base64,
            Some(STANDARD.encode([8, 0, 24, 99, 0x98, 6, 7]))
        );
        assert_eq!(report.validation.state, "not_evaluated");
        let Preview::Message {
            fields, unknown, ..
        } = report.preview
        else {
            panic!()
        };
        assert!(fields.iter().find(|f| f.number == 1).unwrap().present);
        assert_eq!(unknown[0].number, 99);
        std::fs::write(&args.input, []).unwrap();
        args.original = false;
        let report = inspect(&args).unwrap();
        assert!(report.original_base64.is_none());
        let Preview::Message { fields, .. } = report.preview else {
            panic!()
        };
        assert!(!fields.iter().find(|f| f.number == 1).unwrap().present);
    }
    #[test]
    fn schema_payload_json_qualifies_any_and_rejects_nested_duplicate() {
        let (_directory, args) = args(br#"{"count":"9223372036854775807","path":"/wA=","mask":"someField","detail":{"@type":"type.googleapis.com/plan61.qualification.v1.Payload","label":"nested"}}"#, Encoding::Json);
        assert!(inspect(&args).is_ok());
        for bytes in [br#"{"count":1,"count":2}"#.as_slice(), br#"{"detail":{"@type":"type.googleapis.com/plan61.qualification.v1.Payload","label":"a","label":"b"}}"#, br#"{"unknown":1}"#] {
            std::fs::write(&args.input, bytes).unwrap();
            assert!(inspect(&args).is_err());
        }
    }
    #[test]
    fn schema_payload_malformed_wire_refuses() {
        for bytes in [&[0x12, 3, 1][..], &[8, 0x80], &[0]] {
            let (_directory, args) = args(bytes, Encoding::Binary);
            assert!(inspect(&args).is_err());
        }
    }
    #[test]
    fn schema_payload_compiled_world_uses_static_validation() {
        let (_directory, mut args) = args(&[], Encoding::Binary);
        std::fs::write(
            &args.descriptor,
            crate::code_intelligence_contracts::DESCRIPTORS,
        )
        .unwrap();
        args.message = "pse.code_intelligence.v1.NativeControl".into();
        let report = inspect(&args).unwrap();
        assert_eq!(report.representation, "generated_static");
        assert_eq!(report.validation.state, "violations");
        assert!(matches!(report.preview, Preview::Message { .. }));
    }
    #[test]
    fn schema_payload_compiled_json_never_discards_unknown_names() {
        let (_directory, mut args) = args(br#"{"cancel":{"unknown":1}}"#, Encoding::Json);
        std::fs::write(
            &args.descriptor,
            crate::code_intelligence_contracts::DESCRIPTORS,
        )
        .unwrap();
        args.message = "pse.code_intelligence.v1.NativeControl".into();
        assert!(inspect(&args).is_err());
        std::fs::write(&args.input, br#"{"cancel":{}}"#).unwrap();
        let report = inspect(&args).unwrap();
        assert_eq!(report.representation, "generated_static");
        assert_eq!(report.validation.state, "passed");
        std::fs::write(&args.input, br#"{"start":{},"cancel":{}}"#).unwrap();
        assert!(
            inspect(&args).is_err(),
            "multiple oneof members must refuse"
        );
        args.message = "pse.code_intelligence.v1.ContextGuard".into();
        std::fs::write(
            &args.input,
            br#"{"source_digest":"AA==","sourceDigest":"AQ=="}"#,
        )
        .unwrap();
        assert!(
            inspect(&args).is_err(),
            "different aliases of one field must refuse"
        );
    }
    #[test]
    fn schema_payload_textproto_rejects_unknown_names_and_preserves_warning_boundary() {
        let (_directory, args) = args(b"count: 0 label: \"text\"", Encoding::Textproto);
        assert!(inspect(&args).is_ok());
        std::fs::write(&args.input, b"unrecognized_field: 5").unwrap();
        assert!(inspect(&args).is_err());
    }
    #[test]
    fn schema_payload_proto2_required_warning_is_not_validation() {
        use buffa::Message as _;
        use buffa_descriptor::generated::descriptor::{
            DescriptorProto, FieldDescriptorProto, FileDescriptorProto, FileDescriptorSet,
            field_descriptor_proto::{Label, Type},
        };
        let (_directory, mut args) = args(b"", Encoding::Textproto);
        let set = FileDescriptorSet {
            file: vec![FileDescriptorProto {
                name: Some("required.proto".into()),
                package: Some("required_test".into()),
                syntax: Some("proto2".into()),
                message_type: vec![DescriptorProto {
                    name: Some("Payload".into()),
                    field: vec![FieldDescriptorProto {
                        name: Some("count".into()),
                        number: Some(1),
                        label: Some(Label::LABEL_REQUIRED),
                        r#type: Some(Type::TYPE_INT64),
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }],
            ..Default::default()
        };
        std::fs::write(&args.descriptor, set.encode_to_vec()).unwrap();
        args.message = "required_test.Payload".into();
        let report = inspect(&args).unwrap();
        assert_eq!(report.validation.state, "not_evaluated");
        assert!(
            !report.parser_diagnostics.is_empty(),
            "native partial-message warning must remain visible"
        );
    }
}
