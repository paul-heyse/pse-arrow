// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Immutable, offline descriptor worlds. Schema identity never supplies Rust identity.
pub mod compare;
pub mod payload;
pub mod search;

use buffa::Message;
use buffa_descriptor::generated::descriptor::{
    DescriptorProto, EnumDescriptorProto, FileDescriptorProto, FileDescriptorSet,
};
use buffa_descriptor::{DescriptorPool, DynamicMessage};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    io::Read as _,
    path::{Path, PathBuf},
    sync::Arc,
};

/// An invalid schema cannot be interpreted as an empty successful query.
#[derive(Debug, thiserror::Error)]
pub enum SchemaError {
    /// The schema operation's caller cancelled descriptor traversal or presentation.
    #[error("schema operation cancelled")]
    Cancelled,
    /// Original filesystem failure.
    #[error("schema input/output: {0}")]
    Io(#[from] std::io::Error),
    /// Inconsistent descriptor closure or invalid selection.
    #[error("invalid schema: {0}")]
    Invalid(String),
    /// Native descriptor decoder or linker failure.
    #[error("schema decoding: {0}")]
    Decode(String),
    /// Original binary decoder failure, retaining resource and malformed categories.
    #[error("schema binary decoding: {0}")]
    BinaryDecode(#[from] buffa::DecodeError),
    /// A complete original or converted input exceeds this request's byte allowance.
    #[error("schema {input} exceeds {maximum} bytes (observed at least {observed_at_least})")]
    InputLimit {
        /// Original descriptor, original payload, or converted textproto bytes.
        input: &'static str,
        /// Caller-selected maximum retained input extent.
        maximum: usize,
        /// Bounded lookahead establishes at least this many original bytes.
        observed_at_least: usize,
    },
    /// Original subprocess-owner failure; returning requires actual owned drain.
    #[error("schema textproto {operation}: {source}")]
    Supervision {
        /// The actual failed subprocess-owner operation.
        operation: &'static str,
        /// Original supervised process failure.
        #[source]
        source: pse_diagnostics::SupervisionError,
    },
    /// Original conversion failure plus independent physical settlement failure.
    #[error("{operation}; schema conversion settlement also failed: {cleanup}")]
    Settlement {
        /// Original failed conversion or read.
        #[source]
        operation: Box<Self>,
        /// Independently attempted process or I/O-owner settlement.
        cleanup: Box<Self>,
    },
    /// The owned output reader panicked; its OS thread has joined.
    #[error("schema textproto output reader panicked: {0}")]
    OutputReaderPanic(String),
    /// Original structured input failure.
    #[error("schema JSON: {0}")]
    Json(#[from] serde_json::Error),
    /// Original developer evidence checker failure, without rendered cause erasure.
    #[error(transparent)]
    DeveloperEvidence(Box<crate::developer_evidence::EvidenceError>),
    /// Original external tool refusal.
    #[error("schema tool: {0}")]
    Tool(String),
}
pse_diagnostics::impl_diagnostic! {SchemaError,
    code(this){if matches!(this, Self::DeveloperEvidence(_)) { None } else { Some(match this {
        Self::Cancelled => pse_diagnostics::DiagnosticCode::RuntimeCancelled,
        Self::InputLimit { .. } => pse_diagnostics::DiagnosticCode::RuntimeResourceLimit,
        Self::BinaryDecode(error) if pse_contract_proto::support::decode_resource_refusal(error) => pse_diagnostics::DiagnosticCode::RuntimeResourceLimit,
        Self::Settlement { .. } | Self::Supervision { .. } => pse_diagnostics::DiagnosticCode::RuntimeInfrastructure,
        Self::OutputReaderPanic(_) => pse_diagnostics::DiagnosticCode::WorkflowPanic,
        _ => pse_diagnostics::DiagnosticCode::HostOwnership,
    }) }},
    forward(this){match this { Self::DeveloperEvidence(error) => Some(error.as_ref()), _ => None }},help(_this){None},related(this){match this {
        Self::Settlement { operation, cleanup } => {
            let operation: &dyn pse_diagnostics::TypedDiagnostic = operation.as_ref();
            let cleanup: &dyn pse_diagnostics::TypedDiagnostic = cleanup.as_ref();
            Some(Box::new([operation, cleanup].into_iter()))
        }, _ => None,
    }},source(this){match this { Self::Supervision { source, .. } => Some(source), _ => None }},
    facts(this){
        let mut facts = pse_diagnostics::DiagnosticFacts::default();
        if let Self::InputLimit { input, maximum, observed_at_least } = this {
            facts.observe("input", pse_diagnostics::DiagnosticObservation::Text((*input).into()));
            facts.observe("maximum_bytes", pse_diagnostics::DiagnosticObservation::Text(maximum.to_string()));
            facts.observe("observed_at_least_bytes", pse_diagnostics::DiagnosticObservation::Text(observed_at_least.to_string()));
        }
        facts
    }
}
impl SchemaError {
    fn settle<T>(operation: Result<T, Self>, cleanup: Result<(), Self>) -> Result<T, Self> {
        match (operation, cleanup) {
            (result, Ok(())) => result,
            (Ok(_), Err(error)) => Err(error),
            (Err(operation), Err(cleanup)) => Err(Self::Settlement {
                operation: Box::new(operation),
                cleanup: Box::new(cleanup),
            }),
        }
    }
}

const DEFAULT_DESCRIPTOR_BYTES: usize = 64 << 20;

fn read_bounded(
    reader: impl std::io::Read,
    maximum: usize,
    input: &'static str,
) -> Result<Vec<u8>, SchemaError> {
    let extent = u64::try_from(maximum)
        .ok()
        .and_then(|value| value.checked_add(1))
        .ok_or_else(|| {
            SchemaError::Invalid("input byte allowance cannot represent bounded lookahead".into())
        })?;
    let mut bytes = Vec::new();
    reader.take(extent).read_to_end(&mut bytes)?;
    if bytes.len() > maximum {
        return Err(SchemaError::InputLimit {
            input,
            maximum,
            observed_at_least: bytes.len(),
        });
    }
    Ok(bytes)
}

/// One acquired complete descriptor artifact, with no mutable process-wide registry.
#[derive(Debug)]
pub struct Bundle {
    /// Absolute original artifact path, preserving native path bytes.
    path: PathBuf,
    /// Digest of original bytes, not their re-encoding.
    digest: String,
    /// Original complete artifact bytes.
    original: Vec<u8>,
    /// Linked request-local immutable schema world.
    pool: Arc<DescriptorPool>,
    /// Complete source-aware descriptors, including raw options.
    set: FileDescriptorSet,
    #[cfg(test)]
    presented: std::sync::atomic::AtomicUsize,
}

/// A complete acquired world selecting one existing generated decoder family.
/// This is neither semantic compatibility nor permission to construct native authority.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompiledWorld {
    /// Native code-intelligence messages and their compiled local validators.
    CodeIntelligence,
    /// Passive developer observations and their compiled local validators.
    DeveloperEvidence,
}

/// Source locations do not change runtime interpretation; all other descriptor content does.
pub(crate) fn same_runtime_world(
    acquired: &FileDescriptorSet,
    compiled: &FileDescriptorSet,
) -> bool {
    if acquired == compiled {
        return true;
    }
    let compact = |source: &FileDescriptorSet| {
        let mut runtime = source.clone();
        for file in &mut runtime.file {
            file.source_code_info = Default::default();
        }
        runtime
    };
    compact(acquired) == compact(compiled)
}

/// A schema address is meaningful only within the exact artifact digest.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct Declaration {
    /// Stable descriptor address; fields use containing declaration and field number.
    pub address: String,
    /// Original schema declaration name.
    pub name: String,
    /// Schema kind, independent of Rust declaration kinds.
    pub kind: String,
    /// Original logical .proto filename.
    pub file: String,
    /// Exact containing schema address.
    pub parent: Option<String>,
    /// Original field number, if a field or extension.
    pub field_number: Option<i32>,
    /// SourceCodeInfo comments when supplied.
    pub comments: String,
    /// Native descriptor data; no inferred application schema.
    pub details: Value,
    /// Explicit schema addresses; these are not native Rust relationships.
    pub relationships: Vec<String>,
    /// Original descriptor coordinates, not claimed UTF-8 byte ranges.
    pub source_info: Option<SourceInfo>,
}

struct NodeIdentity<'a> {
    address: String,
    name: String,
    kind: &'a str,
    parent: Option<String>,
    number: Option<i32>,
}
/// A borrowed descriptor selection runs before optional presentation or option conversion.
type SelectNode<'a> =
    dyn FnMut(&FileDescriptorProto, &[i32], &NodeIdentity<'_>) -> Result<bool, SchemaError> + 'a;
/// Descriptor producer coordinates remain unqualified until original source is verified.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct SourceInfo {
    /// Descriptor-path integers identify the authored descriptor node.
    pub path: Vec<i32>,
    /// Original zero-based producer span.
    pub span: Vec<i32>,
    /// No arbitrary producer coordinate convention is assumed.
    pub coordinates: String,
}
/// Useful schema inspection without a service, generated-source tree or analyzer session.
#[derive(Debug, Serialize, JsonSchema)]
pub struct Report {
    /// Exact acquired native artifact.
    pub descriptor: crate::investigation::NativePath,
    /// Original complete descriptor digest.
    pub descriptor_sha256: String,
    /// Selected declarations and explicit navigable relationships.
    pub declarations: Vec<Declaration>,
    /// Qualified generated-source nominations for the selected declaration only.
    pub associations: Vec<compare::Association>,
    /// Deliberate interpretation boundaries.
    pub limitations: Vec<String>,
}

impl Report {
    /// Attach explicit generated evidence without expanding the whole generation corpus.
    pub fn with_associations(mut self, root: &Path, bundle: &Bundle) -> Result<Self, SchemaError> {
        self.associations = compare::associations(root, bundle)?
            .into_iter()
            .filter(|association| {
                self.declarations.iter().any(|declaration| {
                    declaration.address == association.declaration
                        || declaration
                            .address
                            .split_once('#')
                            .is_some_and(|(parent, _)| parent == association.declaration)
                })
            })
            .collect();
        Ok(self)
    }
}

impl Bundle {
    /// Read and validate a complete supplied descriptor artifact.
    pub fn load(path: &Path) -> Result<Self, SchemaError> {
        Self::load_bounded(
            path,
            DEFAULT_DESCRIPTOR_BYTES,
            &buffa::DecodeOptions::new().with_max_message_size(DEFAULT_DESCRIPTOR_BYTES),
        )
    }
    fn load_bounded(
        path: &Path,
        maximum: usize,
        options: &buffa::DecodeOptions,
    ) -> Result<Self, SchemaError> {
        let bytes = read_bounded(std::fs::File::open(path)?, maximum, "descriptor")?;
        Self::from_bytes_with_options(std::path::absolute(path)?, bytes, options)
    }
    /// Bind original bytes before decoding; useful for source-built fixtures.
    pub fn from_bytes(path: PathBuf, original: Vec<u8>) -> Result<Self, SchemaError> {
        Self::from_bytes_with_options(path, original, &buffa::DecodeOptions::new())
    }
    fn from_bytes_with_options(
        path: PathBuf,
        original: Vec<u8>,
        options: &buffa::DecodeOptions,
    ) -> Result<Self, SchemaError> {
        let digest = Sha256::digest(&original)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();
        let set: FileDescriptorSet = options.decode_from_slice(&original)?;
        let mut files = BTreeMap::new();
        for file in &set.file {
            let name = file
                .name
                .as_deref()
                .filter(|name| !name.is_empty())
                .ok_or_else(|| SchemaError::Invalid("descriptor filename missing".into()))?;
            let encoded = file.encode_to_vec();
            if let Some(previous) = files.insert(name.to_owned(), encoded.clone())
                && previous != encoded
            {
                return Err(SchemaError::Invalid(format!(
                    "conflicting descriptor file: {name}"
                )));
            }
        }
        for file in &set.file {
            for dependency in file.dependency.iter().chain(&file.option_dependency) {
                if !files.contains_key(dependency) {
                    return Err(SchemaError::Invalid(format!(
                        "{} requires missing descriptor {dependency}",
                        file.name.as_deref().unwrap_or("")
                    )));
                }
            }
            for index in file.public_dependency.iter().chain(&file.weak_dependency) {
                if usize::try_from(*index)
                    .ok()
                    .is_none_or(|index| index >= file.dependency.len())
                {
                    return Err(SchemaError::Invalid(format!(
                        "invalid dependency index {index} in {}",
                        file.name.as_deref().unwrap_or("")
                    )));
                }
            }
        }
        let mut seen = BTreeSet::new();
        let mut linked = set.clone();
        linked.file.retain(|file| seen.insert(file.name.clone()));
        let pool = Arc::new(
            DescriptorPool::new(linked).map_err(|error| SchemaError::Decode(error.to_string()))?,
        );
        Ok(Self {
            path,
            digest,
            original,
            pool,
            set,
            #[cfg(test)]
            presented: std::sync::atomic::AtomicUsize::new(0),
        })
    }
    /// Bind complete runtime content to exactly one supported compiled family.
    /// Original artifact bytes and their digest are never normalized by recognition.
    pub fn compiled_world(&self) -> Result<Option<CompiledWorld>, SchemaError> {
        for (world, bytes) in [
            (
                CompiledWorld::CodeIntelligence,
                crate::code_intelligence_contracts::DESCRIPTORS,
            ),
            (
                CompiledWorld::DeveloperEvidence,
                crate::developer_evidence_contracts::DESCRIPTORS,
            ),
        ] {
            let compiled = FileDescriptorSet::decode_from_slice(bytes)?;
            if same_runtime_world(&self.set, &compiled) {
                return Ok(Some(world));
            }
        }
        Ok(None)
    }
    /// Resolve an exact descriptor address or an unambiguous original qualified name.
    pub fn declaration(&self, address: &str) -> Result<Declaration, SchemaError> {
        let address = address.trim_start_matches('.');
        self.index(Some(address))?
            .into_iter()
            .find(|node| node.address == address || node.name == address)
            .ok_or_else(|| {
                SchemaError::Invalid(format!("schema declaration unavailable: {address}"))
            })
    }
    /// Inspect a selected declaration, or list top-level file nodes.
    pub fn inspect(&self, address: Option<&str>) -> Result<Report, SchemaError> {
        let mut declarations = if let Some(address) = address {
            vec![self.declaration(address)?]
        } else {
            self.pool
                .files()
                .iter()
                .map(|file| {
                    self.declaration(&format!("file:{}", file.name.as_deref().unwrap_or("")))
                })
                .collect::<Result<Vec<_>, _>>()?
        };
        declarations.sort_by(|a, b| a.address.cmp(&b.address));
        Ok(Report {descriptor:self.path.clone().into(),descriptor_sha256:self.digest.clone(),declarations,associations:Vec::new(),limitations:vec!["Schema links do not establish native Rust identity or constraint validity.".into(),"Descriptor source spans retain producer coordinates; no source byte action is inferred.".into()]})
    }
    fn options<O: Message + buffa::MessageName + Serialize>(
        &self,
        options: Option<&O>,
    ) -> Result<Value, SchemaError> {
        let Some(options) = options else {
            return Ok(Value::Null);
        };
        if let Some(dynamic) = DynamicMessage::from_options(Arc::clone(&self.pool), options) {
            return Ok(
                json!({"interpretation":"linked_options","value":serde_json::to_value(&dynamic).map_err(|error|SchemaError::Decode(error.to_string()))?,"original_base64":base64::Engine::encode(&base64::engine::general_purpose::STANDARD,options.encode_to_vec())}),
            );
        }
        Ok(
            json!({"interpretation":"typed_options_custom_extensions_unavailable","value":serde_json::to_value(options)?,"original_base64":base64::Engine::encode(&base64::engine::general_purpose::STANDARD,options.encode_to_vec())}),
        )
    }
    fn node(
        &self,
        file: &FileDescriptorProto,
        path: Vec<i32>,
        identity: NodeIdentity<'_>,
        details: Value,
        relationships: Vec<String>,
    ) -> Declaration {
        #[cfg(test)]
        self.presented
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let NodeIdentity {
            address,
            name,
            kind,
            parent,
            number,
        } = identity;
        let location = Self::location(file, &path);
        Declaration {
            address,
            name,
            kind: kind.into(),
            file: file.name.clone().unwrap_or_default(),
            parent,
            field_number: number,
            comments: location
                .map(|location| {
                    [
                        location.leading_comments.as_deref().unwrap_or(""),
                        location.trailing_comments.as_deref().unwrap_or(""),
                    ]
                    .join("\n")
                })
                .unwrap_or_default(),
            details,
            relationships,
            source_info: location.map(|location| SourceInfo {
                path,
                span: location.span.clone(),
                coordinates: "descriptor_producer_unqualified".into(),
            }),
        }
    }
    fn location<'a>(
        file: &'a FileDescriptorProto,
        path: &[i32],
    ) -> Option<&'a buffa_descriptor::generated::descriptor::source_code_info::Location> {
        file.source_code_info
            .as_option()
            .and_then(|info| info.location.iter().find(|location| location.path == path))
    }
    fn index(&self, selected: Option<&str>) -> Result<Vec<Declaration>, SchemaError> {
        self.select(selected, &mut |_, _, _| Ok(true))
    }
    fn select(
        &self,
        selected: Option<&str>,
        include: &mut SelectNode<'_>,
    ) -> Result<Vec<Declaration>, SchemaError> {
        let mut nodes = Vec::new();
        let files = if let Some(address) = selected {
            let file = if let Some(name) = address.strip_prefix("file:") {
                self.pool.file_by_name(name)
            } else {
                let symbol = address.strip_prefix("extension:").unwrap_or(address);
                let symbol = symbol.split(['#', '@']).next().unwrap_or(symbol);
                self.pool.file_containing_symbol(symbol)
                    // Presentation qualifies enum values under their enum; protobuf's
                    // linked symbol index qualifies them under the parent scope.
                    .or_else(|| symbol.rsplit_once('.').and_then(|(parent, _)| self.pool.file_containing_symbol(parent)))
                    .or_else(|| self.pool.file_by_name(address))
            };
            file.into_iter().collect::<Vec<_>>()
        } else {
            self.pool.files().iter().collect()
        };
        let visit = include;
        let mut include = |file: &FileDescriptorProto, path: &[i32], node: &NodeIdentity<'_>| {
            if selected.is_none_or(|selected| selected == node.address || selected == node.name) {
                visit(file, path, node)
            } else {
                Ok(false)
            }
        };
        for file in files {
            let filename = file.name.clone().unwrap_or_default();
            let address = format!("file:{filename}");
            let package = file.package.as_deref().unwrap_or("");
            let mut children = Vec::new();
            for (index, message) in file.message_type.iter().enumerate() {
                let name = qualified(package, message.name.as_deref().unwrap_or(""));
                children.push(name.clone());
                self.message(
                    file,
                    message,
                    name,
                    vec![4, index as i32],
                    &mut include,
                    &mut nodes,
                )?;
            }
            for (index, enumeration) in file.enum_type.iter().enumerate() {
                let name = qualified(package, enumeration.name.as_deref().unwrap_or(""));
                children.push(name.clone());
                self.enumeration(
                    file,
                    enumeration,
                    name,
                    vec![5, index as i32],
                    &mut include,
                    &mut nodes,
                )?;
            }
            for (index, service) in file.service.iter().enumerate() {
                let name = qualified(package, service.name.as_deref().unwrap_or(""));
                children.push(name.clone());
                let mut methods = Vec::new();
                for (method_index, method) in service.method.iter().enumerate() {
                    let method_name = qualified(&name, method.name.as_deref().unwrap_or(""));
                    methods.push(method_name.clone());
                    let path = vec![6, index as i32, 2, method_index as i32];
                    let identity = NodeIdentity {
                        address: method_name.clone(),
                        name: method_name.clone(),
                        kind: "method",
                        parent: Some(name.clone()),
                        number: None,
                    };
                    if include(file, &path, &identity)? {
                        nodes.push(self.node(file, path, identity, json!({"input":method.input_type,"output":method.output_type,"client_streaming":method.client_streaming.unwrap_or(false),"server_streaming":method.server_streaming.unwrap_or(false),"options":self.options(method.options.as_option())?}), [method.input_type.as_deref(),method.output_type.as_deref()].into_iter().flatten().map(|name|name.trim_start_matches('.').into()).collect()));
                    }
                }
                let path = vec![6, index as i32];
                let identity = NodeIdentity {
                    address: name.clone(),
                    name: name.clone(),
                    kind: "service",
                    parent: Some(address.clone()),
                    number: None,
                };
                if include(file, &path, &identity)? {
                    nodes.push(self.node(
                        file,
                        path,
                        identity,
                        json!({"options":self.options(service.options.as_option())?}),
                        methods,
                    ));
                }
            }
            for (index, extension) in file.extension.iter().enumerate() {
                children.push(format!(
                    "extension:{}",
                    qualified(package, extension.name.as_deref().unwrap_or(""))
                ));
                self.field(
                    file,
                    extension,
                    (package, true),
                    vec![7, index as i32],
                    &mut include,
                    &mut nodes,
                )?;
            }
            children.extend(file.dependency.iter().map(|name| format!("file:{name}")));
            let identity = NodeIdentity {
                address: address.clone(),
                name: filename.clone(),
                kind: "file",
                parent: None,
                number: None,
            };
            if include(file, &[], &identity)? {
                nodes.push(self.node(file, vec![], identity, json!({"package":file.package,"syntax":file.syntax,"edition":file.edition.map(|edition|format!("{edition:?}")),"imports":file.dependency,"public_imports":file.public_dependency,"weak_imports":file.weak_dependency,"option_imports":file.option_dependency,"options":self.options(file.options.as_option())?}), children));
            }
        }
        nodes.sort_by(|a, b| a.address.cmp(&b.address));
        Ok(nodes)
    }
    fn message(
        &self,
        file: &FileDescriptorProto,
        message: &DescriptorProto,
        name: String,
        path: Vec<i32>,
        include: &mut SelectNode<'_>,
        nodes: &mut Vec<Declaration>,
    ) -> Result<(), SchemaError> {
        let mut children = Vec::new();
        for (index, field) in message.field.iter().enumerate() {
            children.push(format!("{name}#{}", field.number.unwrap_or_default()));
            self.field(
                file,
                field,
                (&name, false),
                child_path(&path, 2, index),
                include,
                nodes,
            )?;
        }
        for (index, field) in message.extension.iter().enumerate() {
            children.push(format!(
                "extension:{}",
                qualified(&name, field.name.as_deref().unwrap_or(""))
            ));
            self.field(
                file,
                field,
                (&name, true),
                child_path(&path, 6, index),
                include,
                nodes,
            )?;
        }
        for (index, oneof) in message.oneof_decl.iter().enumerate() {
            let child = format!("{name}@oneof:{}", oneof.name.as_deref().unwrap_or(""));
            children.push(child.clone());
            let child_path = child_path(&path, 8, index);
            let identity = NodeIdentity {
                address: child.clone(),
                name: qualified(&name, oneof.name.as_deref().unwrap_or("")),
                kind: "oneof",
                parent: Some(name.clone()),
                number: None,
            };
            if include(file, &child_path, &identity)? {
                nodes.push(
                    self.node(
                        file,
                        child_path,
                        identity,
                        json!({"options":self.options(oneof.options.as_option())?}),
                        message
                            .field
                            .iter()
                            .filter(|field| field.oneof_index == Some(index as i32))
                            .map(|field| format!("{name}#{}", field.number.unwrap_or_default()))
                            .collect(),
                    ),
                );
            }
        }
        for (index, nested) in message.nested_type.iter().enumerate() {
            let child = qualified(&name, nested.name.as_deref().unwrap_or(""));
            children.push(child.clone());
            self.message(
                file,
                nested,
                child,
                child_path(&path, 3, index),
                include,
                nodes,
            )?;
        }
        for (index, enumeration) in message.enum_type.iter().enumerate() {
            let child = qualified(&name, enumeration.name.as_deref().unwrap_or(""));
            children.push(child.clone());
            self.enumeration(
                file,
                enumeration,
                child,
                child_path(&path, 4, index),
                include,
                nodes,
            )?;
        }
        let parent = schema_parent(file, &name);
        let identity = NodeIdentity {
            address: name.clone(),
            name: name.clone(),
            kind: "message",
            parent: Some(parent),
            number: None,
        };
        if include(file, &path, &identity)? {
            nodes.push(self.node(file, path, identity, json!({"options":self.options(message.options.as_option())?,"reserved_names":message.reserved_name,"reserved_ranges":serde_json::to_value(&message.reserved_range)?}), children));
        }
        Ok(())
    }
    fn field(
        &self,
        file: &FileDescriptorProto,
        field: &buffa_descriptor::generated::descriptor::FieldDescriptorProto,
        scope: (&str, bool),
        path: Vec<i32>,
        include: &mut SelectNode<'_>,
        nodes: &mut Vec<Declaration>,
    ) -> Result<(), SchemaError> {
        let (parent, extension) = scope;
        let name = qualified(parent, field.name.as_deref().unwrap_or(""));
        let address = if extension {
            format!("extension:{name}")
        } else {
            format!("{parent}#{}", field.number.unwrap_or_default())
        };
        let container = if extension && path.first() == Some(&7) {
            format!("file:{}", file.name.as_deref().unwrap_or(""))
        } else {
            parent.into()
        };
        let identity = NodeIdentity {
            address,
            name,
            kind: if extension { "extension" } else { "field" },
            parent: Some(container),
            number: field.number,
        };
        if include(file, &path, &identity)? {
            let resolved = self.pool.message_by_name(parent).and_then(|message| {
                message
                    .fields()
                    .iter()
                    .find(|candidate| candidate.number() == field.number.unwrap_or_default() as u32)
            });
            nodes.push(self.node(file, path, identity, json!({"type":field.r#type.map(|kind|format!("{kind:?}")),"type_name":field.type_name,"label":field.label.map(|label|format!("{label:?}")),"presence":resolved.map(|field|format!("{:?}",field.presence())),"proto3_optional":field.proto3_optional,"oneof_index":field.oneof_index,"default":field.default_value,"json_name":field.json_name,"extendee":field.extendee,"options":self.options(field.options.as_option())?}), field.type_name.iter().chain(&field.extendee).map(|name|name.trim_start_matches('.').into()).collect()));
        }
        Ok(())
    }
    fn enumeration(
        &self,
        file: &FileDescriptorProto,
        enumeration: &EnumDescriptorProto,
        name: String,
        path: Vec<i32>,
        include: &mut SelectNode<'_>,
        nodes: &mut Vec<Declaration>,
    ) -> Result<(), SchemaError> {
        let mut children = Vec::new();
        for (index, value) in enumeration.value.iter().enumerate() {
            let child = format!("{name}@value:{}", value.name.as_deref().unwrap_or(""));
            children.push(child.clone());
            let child_path = child_path(&path, 2, index);
            let identity = NodeIdentity {
                address: child.clone(),
                name: qualified(&name, value.name.as_deref().unwrap_or("")),
                kind: "enum_value",
                parent: Some(name.clone()),
                number: None,
            };
            if include(file, &child_path, &identity)? {
                nodes.push(self.node(
                file,
                child_path,
                identity,
                json!({"number":value.number,"options":self.options(value.options.as_option())?}),
                vec![],
            ));
            }
        }
        let parent = schema_parent(file, &name);
        let identity = NodeIdentity {
            address: name.clone(),
            name: name.clone(),
            kind: "enum",
            parent: Some(parent),
            number: None,
        };
        if include(file, &path, &identity)? {
            nodes.push(self.node(
                file,
                path,
                identity,
                json!({"options":self.options(enumeration.options.as_option())?}),
                children,
            ));
        }
        Ok(())
    }
}
fn qualified(parent: &str, name: &str) -> String {
    if parent.is_empty() {
        name.into()
    } else {
        format!("{parent}.{name}")
    }
}
fn schema_parent(file: &FileDescriptorProto, name: &str) -> String {
    let package = file.package.as_deref().unwrap_or("");
    let relative = if package.is_empty() {
        name
    } else {
        name.strip_prefix(package)
            .and_then(|name| name.strip_prefix('.'))
            .unwrap_or(name)
    };
    relative
        .rsplit_once('.')
        .map(|(parent, _)| qualified(package, parent))
        .unwrap_or_else(|| format!("file:{}", file.name.as_deref().unwrap_or("")))
}
fn child_path(parent: &[i32], field: i32, index: usize) -> Vec<i32> {
    let mut path = parent.to_vec();
    path.extend([field, index as i32]);
    path
}

#[cfg(test)]
mod tests {
    use super::*;
    const CORPUS: &[u8] = include_bytes!("../generated/plan61/descriptor.bin");
    #[test]
    fn schema_parent_and_relationship_addresses_resolve() {
        let bundle = Bundle::from_bytes("fixture".into(), CORPUS.to_vec()).unwrap();
        for node in bundle.index(None).unwrap() {
            for address in node.parent.iter().chain(&node.relationships) {
                assert!(
                    bundle.declaration(address).is_ok(),
                    "{} links to missing {address}",
                    node.address
                );
            }
        }
        let extension = bundle
            .declaration("extension:plan61.qualification.v1.qualification_note")
            .unwrap();
        let parent = bundle
            .declaration(extension.parent.as_deref().unwrap())
            .unwrap();
        assert_eq!(parent.kind, "file");
        assert!(parent.relationships.contains(&extension.address));
    }
    #[test]
    fn schema_complete_closure_conflicts_and_unused_imports() {
        let mut set = FileDescriptorSet::decode_from_slice(CORPUS).unwrap();
        let first = set.file[0].clone();
        set.file.push(first.clone());
        assert!(Bundle::from_bytes("fixture".into(), set.encode_to_vec()).is_ok());
        set.file.last_mut().unwrap().package = Some("different".into());
        assert!(
            Bundle::from_bytes("fixture".into(), set.encode_to_vec())
                .unwrap_err()
                .to_string()
                .contains("conflicting")
        );
        let mut set = FileDescriptorSet::decode_from_slice(CORPUS).unwrap();
        set.file[0].dependency.push("missing-unused.proto".into());
        assert!(
            Bundle::from_bytes("fixture".into(), set.encode_to_vec())
                .unwrap_err()
                .to_string()
                .contains("missing-unused")
        );
    }
    #[test]
    fn schema_navigates_method_fields_and_preserves_options() {
        let bundle = Bundle::from_bytes("fixture".into(), CORPUS.to_vec()).unwrap();
        let production = Bundle::from_bytes(
            "production".into(),
            crate::code_intelligence_contracts::DESCRIPTORS.to_vec(),
        )
        .unwrap();
        let service = production
            .index(None)
            .unwrap()
            .into_iter()
            .find(|node| node.kind == "service")
            .unwrap();
        assert!(!service.relationships.is_empty());
        let method = production.declaration(&service.relationships[0]).unwrap();
        assert_eq!(method.kind, "method");
        let message = bundle
            .declaration("plan61.qualification.v1.Payload")
            .unwrap();
        assert_eq!(
            message.details["options"]["interpretation"],
            "linked_options"
        );
        assert_eq!(
            message.details["options"]["value"]["[plan61.qualification.v1.qualification_note]"],
            "original source and custom options survive"
        );
        let original = base64::Engine::decode(
            &base64::engine::general_purpose::STANDARD,
            message.details["options"]["original_base64"]
                .as_str()
                .unwrap(),
        )
        .unwrap();
        let native = bundle
            .pool
            .message_by_name("plan61.qualification.v1.Payload")
            .unwrap();
        assert_eq!(original, native.options().unwrap().encode_to_vec());
        let field = bundle
            .declaration("plan61.qualification.v1.Payload#6")
            .unwrap();
        assert_eq!(field.field_number, Some(6));
        assert!(field.source_info.is_some());
    }
    #[test]
    fn schema_exact_presentation_is_lazy_and_matches_exhaustive_rows() {
        let bundle = Bundle::from_bytes("fixture".into(), CORPUS.to_vec()).unwrap();
        assert_eq!(
            bundle.presented.load(std::sync::atomic::Ordering::Relaxed),
            0
        );
        let exact = bundle
            .declaration("plan61.qualification.v1.Payload#6")
            .unwrap();
        assert_eq!(exact.field_number, Some(6));
        assert_eq!(
            bundle.presented.load(std::sync::atomic::Ordering::Relaxed),
            1
        );
        let files = bundle.inspect(None).unwrap().declarations;
        assert!(!files.is_empty());
        assert_eq!(
            bundle.presented.load(std::sync::atomic::Ordering::Relaxed),
            1 + files.len()
        );
        for expected in &bundle.index(None).unwrap() {
            for selector in [&expected.address, &expected.name] {
                let selected = bundle.declaration(selector).unwrap();
                assert_eq!(
                    serde_json::to_value(selected).unwrap(),
                    serde_json::to_value(expected).unwrap(),
                    "{selector}"
                );
            }
        }
        assert!(bundle.declaration("absent.message").is_err());
    }
    #[test]
    fn schema_file_listing_keeps_sorted_unique_presentation_without_full_index() {
        let mut set = FileDescriptorSet::decode_from_slice(CORPUS).unwrap();
        let expected = Bundle::from_bytes("original".into(), CORPUS.to_vec())
            .unwrap()
            .index(None)
            .unwrap()
            .iter()
            .filter(|node| node.kind == "file")
            .map(|node| node.address.clone())
            .collect::<Vec<_>>();
        set.file.reverse();
        set.file.push(set.file[0].clone());
        let bundle = Bundle::from_bytes("reordered".into(), set.encode_to_vec()).unwrap();
        let actual = bundle
            .inspect(None)
            .unwrap()
            .declarations
            .into_iter()
            .map(|node| node.address)
            .collect::<Vec<_>>();
        assert_eq!(actual, expected);
        assert_eq!(
            bundle.presented.load(std::sync::atomic::Ordering::Relaxed),
            actual.len()
        );
    }
    #[test]
    fn schema_compiled_binding_preserves_source_identity_and_refuses_other_content() {
        use buffa_descriptor::generated::descriptor::{SourceCodeInfo, source_code_info::Location};
        for (world, bytes) in [
            (
                CompiledWorld::CodeIntelligence,
                crate::code_intelligence_contracts::DESCRIPTORS,
            ),
            (
                CompiledWorld::DeveloperEvidence,
                crate::developer_evidence_contracts::DESCRIPTORS,
            ),
        ] {
            let original = Bundle::from_bytes("compiled".into(), bytes.to_vec()).unwrap();
            assert_eq!(original.compiled_world().unwrap(), Some(world));
            let mut source = FileDescriptorSet::decode_from_slice(bytes).unwrap();
            source.file[0].source_code_info = SourceCodeInfo {
                location: vec![Location {
                    path: vec![4, 0],
                    leading_comments: Some("selected source information".into()),
                    ..Default::default()
                }],
                ..Default::default()
            }
            .into();
            let source_bytes = source.encode_to_vec();
            let acquired = Bundle::from_bytes("source".into(), source_bytes.clone()).unwrap();
            assert_eq!(acquired.compiled_world().unwrap(), Some(world));
            assert_ne!(acquired.digest, original.digest);
            assert_eq!(acquired.original, source_bytes);
            assert_eq!(
                acquired
                    .presented
                    .load(std::sync::atomic::Ordering::Relaxed),
                0
            );
            let mut unknown = source_bytes;
            unknown.extend_from_slice(&[0xa0, 0x06, 0x09]);
            assert_eq!(
                Bundle::from_bytes("unknown".into(), unknown)
                    .unwrap()
                    .compiled_world()
                    .unwrap(),
                None
            );
            let mut reordered = source.clone();
            reordered.file.reverse();
            assert_eq!(
                Bundle::from_bytes("reordered".into(), reordered.encode_to_vec())
                    .unwrap()
                    .compiled_world()
                    .unwrap(),
                None
            );
            let mut duplicate = source.clone();
            duplicate.file.push(source.file[0].clone());
            assert_eq!(
                Bundle::from_bytes("duplicate".into(), duplicate.encode_to_vec())
                    .unwrap()
                    .compiled_world()
                    .unwrap(),
                None
            );
            let mut imports = source.clone();
            let file = imports
                .file
                .iter_mut()
                .find(|file| file.dependency.len() >= 2)
                .unwrap();
            file.dependency.swap(0, 1);
            assert_eq!(
                Bundle::from_bytes("imports".into(), imports.encode_to_vec())
                    .unwrap()
                    .compiled_world()
                    .unwrap(),
                None
            );
            let mut features = source.clone();
            let mut file = features.file[0].encode_to_vec();
            // FileOptions.features (field50), FeatureSet.field_presence=EXPLICIT.
            file.extend_from_slice(&[0x42, 0x05, 0x92, 0x03, 0x02, 0x08, 0x01]);
            features.file[0] = FileDescriptorProto::decode_from_slice(&file).unwrap();
            assert_eq!(
                Bundle::from_bytes("features".into(), features.encode_to_vec())
                    .unwrap()
                    .compiled_world()
                    .unwrap(),
                None
            );
            let mut option = source;
            let mut file = option.file[0].encode_to_vec();
            // FileDescriptorProto.options (field8) with an independently specified
            // unknown option (field100) is runtime content, not source information.
            file.extend_from_slice(&[0x42, 0x03, 0xa0, 0x06, 0x09]);
            option.file[0] = FileDescriptorProto::decode_from_slice(&file).unwrap();
            assert_eq!(
                Bundle::from_bytes("option".into(), option.encode_to_vec())
                    .unwrap()
                    .compiled_world()
                    .unwrap(),
                None
            );
        }
    }
}
