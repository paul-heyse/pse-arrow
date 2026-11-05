// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Passive payload interpretation in one explicitly selected descriptor world.
use super::{Bundle, CompiledWorld, SchemaError};
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
    io::{Seek as _, Write as _},
    path::PathBuf,
    process::{Command, Stdio},
    sync::Arc,
    time::{Duration, Instant},
};

const DEFAULT_INPUT_BYTES: usize = crate::developer_evidence::MAX_INPUT_BYTES;

/// Request-owned byte and binary decode allowances, independent of presentation.
#[derive(Clone, Copy, Debug, ClapArgs, Serialize, JsonSchema)]
pub struct Admission {
    /// Maximum complete original descriptor artifact bytes.
    #[arg(long, default_value_t = super::DEFAULT_DESCRIPTOR_BYTES)]
    pub max_descriptor_bytes: usize,
    /// Maximum complete original payload bytes, in the selected input grammar.
    #[arg(long, default_value_t = DEFAULT_INPUT_BYTES)]
    pub max_input_bytes: usize,
    /// Maximum binary output bytes collected from textproto conversion.
    #[arg(long, default_value_t = DEFAULT_INPUT_BYTES)]
    pub max_converted_bytes: usize,
    /// Maximum materialized element bytes per descriptor or payload binary decode.
    #[arg(long, default_value_t = buffa::DecodeOptions::new().element_memory_limit())]
    pub max_binary_element_bytes: usize,
    /// Maximum nested messages per descriptor or payload binary decode.
    #[arg(long, default_value_t = buffa::DecodeOptions::new().recursion_limit())]
    pub binary_recursion_limit: u32,
}
impl Default for Admission {
    fn default() -> Self {
        Self {
            max_descriptor_bytes: super::DEFAULT_DESCRIPTOR_BYTES,
            max_input_bytes: DEFAULT_INPUT_BYTES,
            max_converted_bytes: DEFAULT_INPUT_BYTES,
            max_binary_element_bytes: buffa::DecodeOptions::new().element_memory_limit(),
            binary_recursion_limit: buffa::DecodeOptions::new().recursion_limit(),
        }
    }
}
impl Admission {
    fn validate(self) -> Result<(), SchemaError> {
        let maximum = buffa::DecodeOptions::new().max_message_size();
        if [
            self.max_descriptor_bytes,
            self.max_input_bytes,
            self.max_converted_bytes,
        ]
        .into_iter()
        .any(|bytes| bytes > maximum)
        {
            return Err(SchemaError::Invalid(format!(
                "input byte allowances must not exceed the pinned binary codec maximum of {maximum}"
            )));
        }
        Ok(())
    }
    fn binary_options(self, maximum: usize) -> buffa::DecodeOptions {
        buffa::DecodeOptions::new()
            .with_max_message_size(maximum)
            .with_element_memory_limit(self.max_binary_element_bytes)
            .with_recursion_limit(self.binary_recursion_limit)
    }
}

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
    /// Explicit byte/decode admission, separate from preview expansion.
    #[command(flatten)]
    pub admission: Admission,
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
    /// Supported original field/rule paths and loss-aware disclosure.
    pub correction: Option<crate::envelope::ValidationCorrection>,
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
    /// Effective request allowances; captured evidence also retains its family byte cap.
    pub admission: Admission,
    /// Binary payload allowances apply to binary and converted textproto, not JSON parsing.
    pub binary_payload_limits_applied: bool,
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
    /// Original-byte association checks for captured native evidence roots only.
    pub developer_evidence: Option<crate::developer_evidence::Readback>,
    /// Native interpretation/integrity refusal; raw fields do not become current native facts.
    pub developer_evidence_refusal: Option<crate::envelope::ErrorRecord>,
    /// Bounded interpreted fields.
    pub preview: Preview,
    /// Original successful text-parser warnings, if any.
    pub parser_diagnostics: String,
    /// Limits on interpretation and preservation.
    pub limitations: Vec<String>,
}

/// Read selected artifacts once, decode passively and preserve their identities.
pub fn inspect(args: &Args) -> Result<Report, SchemaError> {
    args.admission.validate()?;
    if args.limit == 0 || args.depth > 64 {
        return Err(SchemaError::Invalid(
            "payload limit must be positive and display depth at most 64".into(),
        ));
    }
    let bundle = Bundle::load_bounded(
        &args.descriptor,
        args.admission.max_descriptor_bytes,
        &args
            .admission
            .binary_options(args.admission.max_descriptor_bytes),
    )?;
    inspect_bundle(args, &bundle)
}

fn inspect_bundle(args: &Args, bundle: &Bundle) -> Result<Report, SchemaError> {
    args.admission.validate()?;
    let input_path = std::fs::canonicalize(&args.input)?;
    let captured_evidence = matches!(
        args.message.as_str(),
        "pse.developer.evidence.v1.CapturedWorkflowReport"
            | "pse.developer.evidence.v1.CapturedWorkflowEvidence"
    );
    let mut admission = args.admission;
    if captured_evidence {
        admission.max_input_bytes = admission
            .max_input_bytes
            .min(crate::developer_evidence::MAX_INPUT_BYTES);
        admission.max_converted_bytes = admission
            .max_converted_bytes
            .min(crate::developer_evidence::MAX_INPUT_BYTES);
    }
    let input = super::read_bounded(
        std::fs::File::open(&input_path)?,
        admission.max_input_bytes,
        "payload",
    )
    .map_err(|error| match error {
        SchemaError::InputLimit {
            maximum,
            observed_at_least,
            ..
        } if captured_evidence && maximum == crate::developer_evidence::MAX_INPUT_BYTES => {
            SchemaError::DeveloperEvidence(Box::new(
                crate::developer_evidence::EvidenceError::InputLimit {
                    maximum,
                    observed_at_least,
                },
            ))
        }
        error => error,
    })?;
    let index = bundle.pool.message_index(&args.message).ok_or_else(|| {
        SchemaError::Invalid(format!(
            "message absent from selected descriptor: {}",
            args.message
        ))
    })?;
    let compiled_world = bundle.compiled_world()?;
    if matches!(args.encoding, Encoding::Json) {
        strict_json(&input)?;
        if compiled_world.is_some() {
            let json = serde_json::from_slice(&input)?;
            check_keys(&bundle.pool, bundle.pool.message(index), &json)?;
        }
    }
    let (wire, parser_diagnostics) = if matches!(args.encoding, Encoding::Textproto) {
        text_wire(
            &bundle,
            &args.message,
            &input,
            admission.max_converted_bytes,
        )?
    } else {
        (input.clone(), String::new())
    };
    let options = admission.binary_options(if matches!(args.encoding, Encoding::Textproto) {
        admission.max_converted_bytes
    } else {
        admission.max_input_bytes
    });
    let static_value = if let Some(world) = compiled_world {
        compiled(
            world,
            &args.message,
            &wire,
            args.encoding,
            args.depth,
            args.limit,
            &options,
        )?
    } else {
        None
    };
    let (preview, mut validation, representation) = if let Some((preview, validation)) =
        static_value
    {
        (preview, validation, "generated_static")
    } else {
        let value = if matches!(args.encoding, Encoding::Json) {
            let json = std::str::from_utf8(&wire)
                .map_err(|error| SchemaError::Decode(error.to_string()))?;
            DynamicMessage::from_json(Arc::clone(&bundle.pool), index, json)?
        } else {
            DynamicMessage::decode_with_options(Arc::clone(&bundle.pool), index, &wire, &options)?
        };
        (message_preview(&value, args.depth, args.limit), Validation { state: "not_evaluated".into(), diagnostics: vec!["Arbitrary acquired schemas have no compiled validator; constraints are metadata, not an established valid payload.".into()], correction: None }, "acquired_dynamic")
    };
    let (developer_evidence, developer_evidence_refusal) =
        if captured_evidence && !matches!(args.encoding, Encoding::Json) {
            match crate::developer_evidence::inspect(&bundle.set, &args.message, &wire) {
                Ok(readback) => (Some(readback), None),
                Err(
                    error @ (crate::developer_evidence::EvidenceError::Decode(_)
                    | crate::developer_evidence::EvidenceError::InputLimit { .. }),
                ) => return Err(SchemaError::DeveloperEvidence(Box::new(error))),
                Err(error) => (
                    None,
                    Some(crate::envelope::ErrorRecord::from_typed(
                        &error,
                        &crate::envelope::RedactionPolicy::from_environment(),
                    )),
                ),
            }
        } else {
            (None, None)
        };
    if let Some(correction) = &mut validation.correction {
        correction.disclosure.descriptor_sha256 = Some(bundle.digest.clone());
    }
    Ok(Report {
        admission, binary_payload_limits_applied: !matches!(args.encoding, Encoding::Json),
        descriptor: bundle.path.clone().into(), descriptor_sha256: bundle.digest.clone(),
        input: input_path.into(), payload_sha256: Sha256::digest(&input).iter().map(|byte| format!("{byte:02x}")).collect(), payload_bytes: input.len(),
        original_base64: args.original.then(|| STANDARD.encode(&input)), message: args.message.clone(), encoding: args.encoding,
        representation: representation.into(), validation, developer_evidence, developer_evidence_refusal, preview, parser_diagnostics,
        limitations: vec!["Successful decoding does not establish semantic validity. Re-encoding does not preserve original byte identity or wire ordering.".into(), "Presence follows protobuf semantics; implicit scalar default and absence may be indistinguishable. Display limits do not limit decode coverage.".into(), "Any resolves only within the selected descriptor closure; nested payload bytes remain bytes. Original path/digest does not guarantee future availability; --original includes exact input.".into(), "Byte admission bounds complete original and converted inputs. Binary element/recursion limits apply per decode, including descriptors; they do not account complete JSON parsing, descriptor linking, cumulative operation heap or process memory.".into()],
    })
}

fn compiled(
    world: CompiledWorld,
    name: &str,
    input: &[u8],
    encoding: Encoding,
    depth: usize,
    limit: usize,
    options: &buffa::DecodeOptions,
) -> Result<Option<(Preview, Validation)>, SchemaError> {
    if world == CompiledWorld::DeveloperEvidence {
        use crate::developer_evidence_contracts::pb::pse::developer::evidence::v1 as pb;
        return match name {
            "pse.developer.evidence.v1.CapturedWorkflowReport" => {
                Some(static_preview::<pb::CapturedWorkflowReport>(
                    input, encoding, depth, limit, options,
                ))
            }
            "pse.developer.evidence.v1.CapturedWorkflowEvidence" => {
                Some(static_preview::<pb::CapturedWorkflowEvidence>(
                    input, encoding, depth, limit, options,
                ))
            }
            "pse.developer.evidence.v1.NativeWorkflowReport" => {
                Some(static_preview::<pb::NativeWorkflowReport>(
                    input, encoding, depth, limit, options,
                ))
            }
            "pse.developer.evidence.v1.WorkflowReportObservation" => {
                Some(static_preview::<pb::WorkflowReportObservation>(
                    input, encoding, depth, limit, options,
                ))
            }
            "pse.developer.evidence.v1.WorkflowTerminalRecord" => {
                Some(static_preview::<pb::WorkflowTerminalRecord>(
                    input, encoding, depth, limit, options,
                ))
            }
            _ => None,
        }
        .transpose();
    }
    use crate::code_intelligence_contracts::pb::pse::code_intelligence::v1 as pb;
    macro_rules! dispatch { ($($ty:ident),+ $(,)?) => { match name { $(concat!("pse.code_intelligence.v1.", stringify!($ty)) => Some(static_preview::<pb::$ty>(input, encoding, depth, limit, options)),)+ _ => None }.transpose() }; }
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
    options: &buffa::DecodeOptions,
) -> Result<(Preview, Validation), SchemaError> {
    let value: T = if matches!(encoding, Encoding::Json) {
        serde_json::from_slice(input)?
    } else {
        options.decode_from_slice::<T>(input)?
    };
    let validation = match value.validate() {
        Ok(()) => Validation {
            state: "passed".into(),
            diagnostics: vec![],
            correction: None,
        },
        Err(error) => Validation {
            state: if error.compile_error.is_some() {
                "compile_error"
            } else if error.runtime_error.is_some() || error.violations.is_empty() {
                "evaluation_error"
            } else {
                "violations"
            }
            .into(),
            diagnostics: error
                .compile_error
                .iter()
                .chain(error.runtime_error.iter())
                .map(|message| crate::envelope::RedactionPolicy::from_environment().redact(message))
                .collect(),
            correction: Some(crate::envelope::ValidationCorrection::local(
                &error,
                value.message_descriptor().full_name(),
            )),
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
    maximum: usize,
) -> Result<(Vec<u8>, String), SchemaError> {
    // Private snapshots ensure descriptor and payload identities cannot race their
    // original mutable paths. The conversion owner settles its bounded output
    // reader and supervised process group before these snapshots depart.
    let mut descriptor = tempfile::NamedTempFile::new()?;
    descriptor.write_all(&bundle.original)?;
    let mut original = tempfile::tempfile()?;
    original.write_all(input)?;
    original.rewind()?;
    let mut command = Command::new("protoc");
    let mut descriptor_argument = std::ffi::OsString::from("--descriptor_set_in=");
    descriptor_argument.push(descriptor.path());
    command
        .arg(descriptor_argument)
        .arg(format!("--encode={message}"))
        .stdin(Stdio::from(original));
    let deadline = Instant::now()
        .checked_add(Duration::from_secs(30))
        .ok_or_else(|| SchemaError::Invalid("textproto conversion deadline extent".into()))?;
    convert(&mut command, maximum, deadline)
}

struct Conversion {
    child: pse_diagnostics::SupervisedChild,
    output: Option<std::thread::JoinHandle<Result<Vec<u8>, SchemaError>>>,
}
impl Conversion {
    fn cancel(&mut self) -> Result<(), SchemaError> {
        let process = self
            .child
            .cancel()
            .map(|_| ())
            .map_err(|source| SchemaError::Supervision {
                operation: "drain",
                source,
            });
        let output = self
            .output
            .take()
            .map_or(Ok(()), |output| join_output(output).map(|_| ()));
        SchemaError::settle(process, output)
    }
    fn finish(&mut self, deadline: Instant) -> Result<(Vec<u8>, String), SchemaError> {
        while self
            .output
            .as_ref()
            .is_some_and(|output| !output.is_finished())
            && Instant::now() < deadline
        {
            std::thread::sleep(
                Duration::from_millis(2).min(deadline.saturating_duration_since(Instant::now())),
            );
        }
        let mut output = if self
            .output
            .as_ref()
            .is_some_and(std::thread::JoinHandle::is_finished)
        {
            self.output.take().map(join_output)
        } else {
            None
        };
        let wait = match &output {
            Some(Err(_)) => Ok(()),
            Some(Ok(_)) => self
                .child
                .wait_ready(None, Some(deadline))
                .map_err(|source| SchemaError::Supervision {
                    operation: "exit",
                    source,
                }),
            None => Err(SchemaError::Supervision {
                operation: "output deadline",
                source: pse_diagnostics::SupervisionError::Deadline,
            }),
        };
        // Kill/drain/reap independently of the original read/deadline outcome.
        // Park custody retains this operation if group settlement is unknown.
        let settled = self
            .child
            .cancel()
            .map_err(|source| SchemaError::Supervision {
                operation: "drain",
                source,
            });
        if output.is_none() {
            output = self.output.take().map(join_output);
        }
        let output = output
            .unwrap_or_else(|| Err(SchemaError::Tool("textproto output owner absent".into())));
        let (status, cleanup) = match settled {
            Ok(status) => (Some(status), Ok(())),
            Err(error) => (None, Err(error)),
        };
        let bytes = SchemaError::settle(SchemaError::settle(output, wait), cleanup)?;
        let diagnostics =
            String::from_utf8_lossy(&self.child.take_stderr_prefix().unwrap_or_default())
                .into_owned();
        if status.is_none_or(|status| !status.success()) {
            return Err(SchemaError::Tool(format!(
                "protoc {status:?}: {diagnostics}"
            )));
        }
        Ok((bytes, diagnostics))
    }
}
impl Drop for Conversion {
    fn drop(&mut self) {
        if let Err(error) = self.cancel() {
            tracing::error!(error = %error, "textproto abandonment settlement failed");
        }
    }
}
fn join_output(
    output: std::thread::JoinHandle<Result<Vec<u8>, SchemaError>>,
) -> Result<Vec<u8>, SchemaError> {
    output.join().unwrap_or_else(|payload| {
        Err(SchemaError::OutputReaderPanic(
            pse_diagnostics::panic::settle_payload(payload, "non-text output-reader panic payload"),
        ))
    })
}
fn convert(
    command: &mut Command,
    maximum: usize,
    deadline: Instant,
) -> Result<(Vec<u8>, String), SchemaError> {
    use pse_diagnostics::{DrainFailurePolicy, StderrCapture, SupervisedChild};
    command.stdout(Stdio::piped());
    let child = SupervisedChild::spawn(
        command,
        DrainFailurePolicy::Park,
        Some(StderrCapture {
            bytes: 65536,
            stack_bytes: 256 * 1024,
        }),
    )
    .map_err(|source| SchemaError::Supervision {
        operation: "spawn",
        source,
    })?;
    let mut owner = Conversion {
        child,
        output: None,
    };
    let Some(stdout) = owner.child.take_stdout() else {
        return SchemaError::settle(
            Err(SchemaError::Tool("textproto stdout pipe absent".into())),
            owner.cancel(),
        );
    };
    let output = std::thread::Builder::new()
        .name("pse-textproto-output".into())
        .stack_size(256 * 1024)
        .spawn(move || super::read_bounded(stdout, maximum, "converted textproto"));
    match output {
        Ok(output) => owner.output = Some(output),
        Err(error) => return SchemaError::settle(Err(SchemaError::Io(error)), owner.cancel()),
    }
    owner.finish(deadline)
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
                admission: Admission::default(),
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
        for bytes in [br#"{"count":1,"count":2}"#.as_slice(), br#"{"detail":{"@type":"type.googleapis.com/plan61.qualification.v1.Payload","label":"a","label":"b"}}"#, br#"{"unknown":1}"#, b"{} {}"] {
            std::fs::write(&args.input, bytes).unwrap();
            assert!(inspect(&args).is_err());
        }
    }
    #[test]
    fn schema_payload_malformed_wire_refuses() {
        for (bytes, expected) in [
            (&[0x12, 3, 1][..], buffa::DecodeError::UnexpectedEof),
            (&[8, 0x80][..], buffa::DecodeError::UnexpectedEof),
            (&[0][..], buffa::DecodeError::InvalidFieldNumber),
        ] {
            let (_directory, args) = args(bytes, Encoding::Binary);
            let SchemaError::BinaryDecode(error) = inspect(&args).unwrap_err() else {
                panic!()
            };
            assert_eq!(error, expected);
            assert!(!pse_contract_proto::support::decode_resource_refusal(
                &error
            ));
        }
    }
    #[test]
    fn schema_payload_original_byte_admission_precedes_decode_at_depth_zero() {
        let (_directory, mut args) = args(&[0; 9], Encoding::Binary);
        args.depth = 0;
        args.admission.max_input_bytes = 8;
        for encoding in [Encoding::Binary, Encoding::Json, Encoding::Textproto] {
            args.encoding = encoding;
            assert!(matches!(
                inspect(&args),
                Err(SchemaError::InputLimit {
                    input: "payload",
                    maximum: 8,
                    observed_at_least: 9,
                })
            ));
        }
        args.admission.max_descriptor_bytes = 16;
        assert!(matches!(
            inspect(&args),
            Err(SchemaError::InputLimit {
                input: "descriptor",
                maximum: 16,
                observed_at_least: 17,
            })
        ));
        args.admission.max_descriptor_bytes = usize::MAX;
        assert!(matches!(inspect(&args), Err(SchemaError::Invalid(_))));
    }
    #[test]
    fn schema_payload_generated_and_dynamic_binary_share_explicit_refusals() {
        use crate::code_intelligence_contracts::pb::pse::code_intelligence::v1 as pb;
        use buffa::Message as _;
        let bundle = Bundle::from_bytes(
            "compiled".into(),
            crate::code_intelligence_contracts::DESCRIPTORS.to_vec(),
        )
        .unwrap();
        let (_directory, mut args) = args(&[], Encoding::Binary);
        args.depth = 0;
        let repeated = pb::BooleanGroup {
            clauses: vec![pb::Predicate::default()],
            ..Default::default()
        }
        .encode_to_vec();
        for (name, wire, element_bytes, recursion, expected) in [
            (
                "BooleanGroup",
                repeated,
                1,
                100,
                buffa::DecodeError::ElementMemoryLimitExceeded,
            ),
            (
                "NativeControl",
                vec![0x12, 0],
                1024,
                0,
                buffa::DecodeError::RecursionLimitExceeded,
            ),
            (
                "NativeControl",
                vec![0x12, 3, 1],
                1024,
                100,
                buffa::DecodeError::UnexpectedEof,
            ),
        ] {
            args.message = format!("pse.code_intelligence.v1.{name}");
            args.admission.max_binary_element_bytes = element_bytes;
            args.admission.binary_recursion_limit = recursion;
            std::fs::write(&args.input, &wire).unwrap();
            let failure = inspect_bundle(&args, &bundle).unwrap_err();
            let resource = pse_contract_proto::support::decode_resource_refusal(&expected);
            assert_eq!(
                pse_diagnostics::TypedDiagnostic::diagnostic_code(&failure),
                Some(if resource {
                    pse_diagnostics::DiagnosticCode::RuntimeResourceLimit
                } else {
                    pse_diagnostics::DiagnosticCode::HostOwnership
                })
            );
            let SchemaError::BinaryDecode(error) = failure else {
                panic!()
            };
            assert_eq!(error, expected, "generated {name}");
            let options = args
                .admission
                .binary_options(args.admission.max_input_bytes);
            let dynamic = DynamicMessage::decode_with_options(
                Arc::clone(&bundle.pool),
                bundle.pool.message_index(&args.message).unwrap(),
                &wire,
                &options,
            )
            .unwrap_err();
            assert_eq!(dynamic, expected, "dynamic {name}");
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
    fn schema_payload_source_world_uses_same_validator_without_presentation() {
        let (_directory, mut args) = args(&[], Encoding::Binary);
        args.message = "pse.code_intelligence.v1.NativeControl".into();
        args.depth = 0;
        let mut digests = Vec::new();
        for bytes in [
            include_bytes!("../generated/code-intelligence/descriptor-runtime.bin").as_slice(),
            crate::code_intelligence_contracts::DESCRIPTORS,
        ] {
            let bundle = Bundle::from_bytes("compiled".into(), bytes.to_vec()).unwrap();
            let report = inspect_bundle(&args, &bundle).unwrap();
            assert_eq!(report.representation, "generated_static");
            assert_eq!(report.validation.state, "violations");
            assert!(matches!(report.preview, Preview::Deferred));
            assert!(report.binary_payload_limits_applied);
            assert_eq!(
                report.admission.max_binary_element_bytes,
                args.admission.max_binary_element_bytes
            );
            assert!(bundle.declarations.get().is_none());
            digests.push(report.descriptor_sha256);
        }
        assert_ne!(digests[0], digests[1]);
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
        assert!(!report.binary_payload_limits_applied);
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
    #[test]
    fn schema_payload_conversion_keeps_exact_bounded_output_and_warnings() {
        let mut command = Command::new("/bin/sh");
        command.args(["-c", "printf abc; printf warning >&2"]);
        let (bytes, warnings) =
            convert(&mut command, 3, Instant::now() + Duration::from_secs(5)).unwrap();
        assert_eq!(bytes, b"abc");
        assert_eq!(warnings, "warning");
    }
    #[test]
    fn schema_payload_output_reader_settles_panicking_payload_destructor() {
        struct PanickingDrop;
        impl Drop for PanickingDrop {
            fn drop(&mut self) {
                panic!("reader payload destructor");
            }
        }
        let reader = std::thread::spawn(|| -> Result<Vec<u8>, SchemaError> {
            std::panic::panic_any(PanickingDrop);
        });
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| join_output(reader)))
            .expect("reader panic payload destruction must remain contained");
        let error = result.unwrap_err();
        assert_eq!(
            pse_diagnostics::TypedDiagnostic::diagnostic_code(&error),
            Some(pse_diagnostics::DiagnosticCode::WorkflowPanic),
        );
        let SchemaError::OutputReaderPanic(detail) = error else {
            panic!("typed reader panic expected")
        };
        assert_eq!(
            detail,
            "non-text output-reader panic payload; panic payload destruction also panicked: reader payload destructor"
        );
    }
    #[test]
    fn schema_payload_conversion_limit_and_deadline_physically_settle_processes() {
        let directory = tempfile::tempdir().unwrap();
        for (script, maximum, timeout, limit) in [
            (
                "printf '%s' $$ > \"$1\"; while :; do printf 1234567890; printf diagnostic >&2; done",
                5,
                Duration::from_secs(5),
                true,
            ),
            (
                "printf '%s' $$ > \"$1\"; sleep 30",
                5,
                Duration::from_millis(250),
                false,
            ),
        ] {
            let marker = directory
                .path()
                .join(if limit { "limit.pid" } else { "deadline.pid" });
            let mut command = Command::new("/bin/sh");
            command.args(["-c", script, "conversion-test"]).arg(&marker);
            let error = convert(&mut command, maximum, Instant::now() + timeout).unwrap_err();
            if limit {
                assert!(
                    matches!(
                        error,
                        SchemaError::InputLimit {
                            input: "converted textproto",
                            maximum: 5,
                            observed_at_least: 6
                        }
                    ),
                    "{error:?}"
                );
            } else {
                assert!(
                    matches!(
                        error,
                        SchemaError::Supervision {
                            source: pse_diagnostics::SupervisionError::Deadline,
                            ..
                        }
                    ),
                    "{error:?}"
                );
            }
            let pid: u32 = std::fs::read_to_string(marker).unwrap().parse().unwrap();
            assert!(
                !std::path::Path::new("/proc").join(pid.to_string()).exists(),
                "conversion returned before actual leader reap"
            );
        }
    }
}
