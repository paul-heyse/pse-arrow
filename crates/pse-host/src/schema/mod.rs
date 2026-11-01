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
    path::{Path, PathBuf},
    sync::Arc,
};

/// An invalid schema cannot be interpreted as an empty successful query.
#[derive(Debug, thiserror::Error)]
pub enum SchemaError {
    /// Original filesystem failure.
    #[error("schema input/output: {0}")]
    Io(#[from] std::io::Error),
    /// Inconsistent descriptor closure or invalid selection.
    #[error("invalid schema: {0}")]
    Invalid(String),
    /// Native descriptor decoder or linker failure.
    #[error("schema decoding: {0}")]
    Decode(String),
    /// Original structured input failure.
    #[error("schema JSON: {0}")]
    Json(#[from] serde_json::Error),
    /// Original external tool refusal.
    #[error("schema tool: {0}")]
    Tool(String),
}
pse_diagnostics::impl_diagnostic! {SchemaError,
    code(_this){Some(pse_diagnostics::DiagnosticCode::HostOwnership)},
    forward(_this){None},help(_this){None},related(_this){None},source(_this){None}
}

/// One acquired complete descriptor artifact, with no mutable process-wide registry.
#[derive(Debug)]
pub struct Bundle {
    /// Absolute original artifact path, preserving native path bytes.
    pub path: PathBuf,
    /// Digest of original bytes, not their re-encoding.
    pub digest: String,
    /// Original complete artifact bytes.
    pub original: Vec<u8>,
    /// Linked request-local immutable schema world.
    pub pool: Arc<DescriptorPool>,
    /// Complete source-aware descriptors, including raw options.
    pub set: FileDescriptorSet,
    declarations: Vec<Declaration>,
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
        Self::from_bytes(std::path::absolute(path)?, std::fs::read(path)?)
    }
    /// Bind original bytes before decoding; useful for source-built fixtures.
    pub fn from_bytes(path: PathBuf, original: Vec<u8>) -> Result<Self, SchemaError> {
        let digest = Sha256::digest(&original)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();
        let mut set = FileDescriptorSet::decode_from_slice(&original)
            .map_err(|error| SchemaError::Decode(error.to_string()))?;
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
        set.file.retain(|file| seen.insert(file.name.clone()));
        let pool = Arc::new(
            DescriptorPool::new(set.clone())
                .map_err(|error| SchemaError::Decode(error.to_string()))?,
        );
        let mut bundle = Self {
            path,
            digest,
            original,
            pool,
            set,
            declarations: Vec::new(),
        };
        bundle.declarations = bundle.index()?;
        Ok(bundle)
    }
    /// Compiled decoders/validators require this exact compiled schema world.
    pub fn compiled_world(&self) -> bool {
        self.original == crate::code_intelligence_contracts::DESCRIPTORS
    }
    /// All declaration observations in deterministic descriptor-address order.
    pub fn declarations(&self) -> &[Declaration] {
        &self.declarations
    }
    /// Resolve an exact descriptor address or an unambiguous original qualified name.
    pub fn declaration(&self, address: &str) -> Result<&Declaration, SchemaError> {
        self.declarations
            .iter()
            .find(|node| node.address == address)
            .or_else(|| {
                self.declarations
                    .iter()
                    .find(|node| node.name == address.trim_start_matches('.'))
            })
            .ok_or_else(|| {
                SchemaError::Invalid(format!("schema declaration unavailable: {address}"))
            })
    }
    /// Inspect a selected declaration, or list top-level file nodes.
    pub fn inspect(&self, address: Option<&str>) -> Result<Report, SchemaError> {
        let declarations = if let Some(address) = address {
            vec![self.declaration(address)?.clone()]
        } else {
            self.declarations
                .iter()
                .filter(|node| node.kind == "file")
                .cloned()
                .collect()
        };
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
                json!({"interpretation":"linked_options","value":serde_json::from_str::<Value>(&dynamic.to_json().map_err(|error|SchemaError::Decode(error.to_string()))?)?,"original_base64":base64::Engine::encode(&base64::engine::general_purpose::STANDARD,options.encode_to_vec())}),
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
        address: String,
        name: String,
        kind: &str,
        parent: Option<String>,
        number: Option<i32>,
        details: Value,
        relationships: Vec<String>,
    ) -> Declaration {
        let location = file
            .source_code_info
            .as_option()
            .and_then(|info| info.location.iter().find(|location| location.path == path));
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
    fn index(&self) -> Result<Vec<Declaration>, SchemaError> {
        let mut nodes = Vec::new();
        for file in &self.set.file {
            let filename = file.name.clone().unwrap_or_default();
            let address = format!("file:{filename}");
            let package = file.package.as_deref().unwrap_or("");
            let mut children = Vec::new();
            for (index, message) in file.message_type.iter().enumerate() {
                let name = qualified(package, message.name.as_deref().unwrap_or(""));
                children.push(name.clone());
                self.message(file, message, name, vec![4, index as i32], &mut nodes)?;
            }
            for (index, enumeration) in file.enum_type.iter().enumerate() {
                let name = qualified(package, enumeration.name.as_deref().unwrap_or(""));
                children.push(name.clone());
                self.enumeration(file, enumeration, name, vec![5, index as i32], &mut nodes)?;
            }
            for (index, service) in file.service.iter().enumerate() {
                let name = qualified(package, service.name.as_deref().unwrap_or(""));
                children.push(name.clone());
                let mut methods = Vec::new();
                for (method_index, method) in service.method.iter().enumerate() {
                    let method_name = qualified(&name, method.name.as_deref().unwrap_or(""));
                    methods.push(method_name.clone());
                    nodes.push(self.node(file,vec![6,index as i32,2,method_index as i32],method_name.clone(),method_name,"method",Some(name.clone()),None,json!({"input":method.input_type,"output":method.output_type,"client_streaming":method.client_streaming.unwrap_or(false),"server_streaming":method.server_streaming.unwrap_or(false),"options":self.options(method.options.as_option())?}),[method.input_type.as_deref(),method.output_type.as_deref()].into_iter().flatten().map(|name|name.trim_start_matches('.').into()).collect()));
                }
                nodes.push(self.node(
                    file,
                    vec![6, index as i32],
                    name.clone(),
                    name,
                    "service",
                    Some(address.clone()),
                    None,
                    json!({"options":self.options(service.options.as_option())?}),
                    methods,
                ));
            }
            for (index, extension) in file.extension.iter().enumerate() {
                children.push(format!(
                    "extension:{}",
                    qualified(package, extension.name.as_deref().unwrap_or(""))
                ));
                self.field(
                    file,
                    extension,
                    package,
                    vec![7, index as i32],
                    true,
                    &mut nodes,
                )?;
            }
            children.extend(file.dependency.iter().map(|name| format!("file:{name}")));
            nodes.push(self.node(file,vec![],address,filename,"file",None,None,json!({"package":file.package,"syntax":file.syntax,"edition":file.edition.map(|edition|format!("{edition:?}")),"imports":file.dependency,"public_imports":file.public_dependency,"weak_imports":file.weak_dependency,"option_imports":file.option_dependency,"options":self.options(file.options.as_option())?}),children));
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
        nodes: &mut Vec<Declaration>,
    ) -> Result<(), SchemaError> {
        let mut children = Vec::new();
        for (index, field) in message.field.iter().enumerate() {
            children.push(format!("{name}#{}", field.number.unwrap_or_default()));
            self.field(
                file,
                field,
                &name,
                child_path(&path, 2, index),
                false,
                nodes,
            )?;
        }
        for (index, field) in message.extension.iter().enumerate() {
            children.push(format!(
                "extension:{}",
                qualified(&name, field.name.as_deref().unwrap_or(""))
            ));
            self.field(file, field, &name, child_path(&path, 6, index), true, nodes)?;
        }
        for (index, oneof) in message.oneof_decl.iter().enumerate() {
            let child = format!("{name}@oneof:{}", oneof.name.as_deref().unwrap_or(""));
            children.push(child.clone());
            nodes.push(
                self.node(
                    file,
                    child_path(&path, 8, index),
                    child,
                    qualified(&name, oneof.name.as_deref().unwrap_or("")),
                    "oneof",
                    Some(name.clone()),
                    None,
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
        for (index, nested) in message.nested_type.iter().enumerate() {
            let child = qualified(&name, nested.name.as_deref().unwrap_or(""));
            children.push(child.clone());
            self.message(file, nested, child, child_path(&path, 3, index), nodes)?;
        }
        for (index, enumeration) in message.enum_type.iter().enumerate() {
            let child = qualified(&name, enumeration.name.as_deref().unwrap_or(""));
            children.push(child.clone());
            self.enumeration(file, enumeration, child, child_path(&path, 4, index), nodes)?;
        }
        let parent = schema_parent(file, &name);
        nodes.push(self.node(file,path,name.clone(),name,"message",Some(parent),None,json!({"options":self.options(message.options.as_option())?,"reserved_names":message.reserved_name,"reserved_ranges":serde_json::to_value(&message.reserved_range)?}),children));
        Ok(())
    }
    fn field(
        &self,
        file: &FileDescriptorProto,
        field: &buffa_descriptor::generated::descriptor::FieldDescriptorProto,
        parent: &str,
        path: Vec<i32>,
        extension: bool,
        nodes: &mut Vec<Declaration>,
    ) -> Result<(), SchemaError> {
        let name = qualified(parent, field.name.as_deref().unwrap_or(""));
        let address = if extension {
            format!("extension:{name}")
        } else {
            format!("{parent}#{}", field.number.unwrap_or_default())
        };
        let resolved = self.pool.message_by_name(parent).and_then(|message| {
            message
                .fields()
                .iter()
                .find(|candidate| candidate.number() == field.number.unwrap_or_default() as u32)
        });
        let container = if extension && path.first() == Some(&7) {
            format!("file:{}", file.name.as_deref().unwrap_or(""))
        } else {
            parent.into()
        };
        nodes.push(self.node(file,path,address,name,if extension{"extension"}else{"field"},Some(container),field.number,json!({"type":field.r#type.map(|kind|format!("{kind:?}")),"type_name":field.type_name,"label":field.label.map(|label|format!("{label:?}")),"presence":resolved.map(|field|format!("{:?}",field.presence())),"proto3_optional":field.proto3_optional,"oneof_index":field.oneof_index,"default":field.default_value,"json_name":field.json_name,"extendee":field.extendee,"options":self.options(field.options.as_option())?}),field.type_name.iter().chain(&field.extendee).map(|name|name.trim_start_matches('.').into()).collect()));
        Ok(())
    }
    fn enumeration(
        &self,
        file: &FileDescriptorProto,
        enumeration: &EnumDescriptorProto,
        name: String,
        path: Vec<i32>,
        nodes: &mut Vec<Declaration>,
    ) -> Result<(), SchemaError> {
        let mut children = Vec::new();
        for (index, value) in enumeration.value.iter().enumerate() {
            let child = format!("{name}@value:{}", value.name.as_deref().unwrap_or(""));
            children.push(child.clone());
            nodes.push(self.node(
                file,
                child_path(&path, 2, index),
                child,
                qualified(&name, value.name.as_deref().unwrap_or("")),
                "enum_value",
                Some(name.clone()),
                None,
                json!({"number":value.number,"options":self.options(value.options.as_option())?}),
                vec![],
            ));
        }
        let parent = schema_parent(file, &name);
        nodes.push(self.node(
            file,
            path,
            name.clone(),
            name,
            "enum",
            Some(parent),
            None,
            json!({"options":self.options(enumeration.options.as_option())?}),
            children,
        ));
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
        for node in bundle.declarations() {
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
            .declarations()
            .iter()
            .find(|node| node.kind == "service")
            .unwrap();
        assert!(!service.relationships.is_empty());
        let method = production.declaration(&service.relationships[0]).unwrap();
        assert_eq!(method.kind, "method");
        let message = bundle
            .declaration("plan61.qualification.v1.Payload")
            .unwrap();
        assert!(message.details.to_string().contains("qualification_note"));
        let field = bundle
            .declaration("plan61.qualification.v1.Payload#6")
            .unwrap();
        assert_eq!(field.field_number, Some(6));
        assert!(field.source_info.is_some());
    }
}
