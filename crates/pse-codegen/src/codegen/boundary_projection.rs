// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Deterministic lowering of actual owner-emitted types, without JSON Schema inference.

use std::collections::BTreeMap;

use buffa::{ExtensionSet, Message};
use buffa_codegen::{
    CodeGenConfig, FdsEmbedding, GeneratedFileKind, IncludeMode,
    generated::descriptor::{
        self, DescriptorProto, FieldDescriptorProto, FieldOptions, FileDescriptorProto,
        FileDescriptorSet, MethodDescriptorProto, OneofDescriptorProto, OneofOptions,
        ServiceDescriptorProto,
        field_descriptor_proto::{Label, Type},
    },
};
use protovalidate_buffa_protos::buf::validate::{
    self as validate,
    __buffa::oneof::{fixed64rules, s_int32rules, s_int64rules, u_int32rules, u_int64rules},
    BytesRules, FieldRules, Fixed64Rules, MapRules, OneofRules, RepeatedRules, Rule, SInt32Rules,
    SInt64Rules, StringRules, UInt32Rules, UInt64Rules,
};
use pse_contract::boundary_projection::{
    Direction, Field, LocalRule, ProjectionGraph, Scalar, Shape, TypeRef,
};
use pse_schema::error::SchemaError;

mod adapters;
mod float_collections;
pub mod metadata;
mod registry_facets;
pub use adapters::{native_adapters, native_adapters_retaining_wire};

/// One complete generation closure. The caller owns publication and generated roots.
#[derive(Debug)]
pub struct ProjectionOutput {
    /// Complete descriptor world, including transitive imports and services.
    pub descriptor_set: FileDescriptorSet,
    /// Exact encoded descriptor set embedded by the shared reflection root.
    pub descriptor_bytes: Vec<u8>,
    /// Buffa-generated filenames and Rust source, plus `shared-root.rs`.
    pub sources: BTreeMap<String, String>,
    /// Original generator warnings; callers must surface them.
    pub warnings: Vec<String>,
    /// Inspectable field/tag/constraint and source-owner reference from this same graph.
    pub documentation: String,
}

fn error(reason: impl Into<String>) -> SchemaError {
    SchemaError::Codegen {
        language: "protobuf",
        reason: reason.into(),
    }
}

/// Lower a graph without external imports. Required typed metadata imports refuse.
pub fn descriptors(
    graph: &ProjectionGraph,
    package: &str,
    file_name: &str,
) -> Result<FileDescriptorSet, SchemaError> {
    descriptors_with_imports(graph, package, file_name, &[])
}

/// Lower a graph and retain its complete provided import world.
/// Imported files are preserved, including custom options, and duplicate filenames refuse.
pub fn descriptors_with_imports(
    graph: &ProjectionGraph,
    package: &str,
    file_name: &str,
    imports: &[FileDescriptorProto],
) -> Result<FileDescriptorSet, SchemaError> {
    descriptors_for_direction(graph, package, file_name, imports, Direction::Both)
}

/// Lower selected input/output fields while retaining stable names, tags and reservations.
pub fn descriptors_for_direction(
    graph: &ProjectionGraph,
    package: &str,
    file_name: &str,
    imports: &[FileDescriptorProto],
    direction: Direction,
) -> Result<FileDescriptorSet, SchemaError> {
    if package.split('.').any(|part| !identifier(part))
        || !package.split('.').next_back().is_some_and(|part| {
            part.strip_prefix('v')
                .is_some_and(|major| major.parse::<u32>().is_ok_and(|major| major > 0))
        })
    {
        return Err(error(
            "externally consumed package must end in an explicit nonzero major (for example pse.engineering.v1)",
        ));
    }
    if !file_name.ends_with(".proto")
        || file_name.starts_with('/')
        || file_name.split('/').any(|p| p == ".." || p.is_empty())
    {
        return Err(error("projection filename must be a relative .proto path"));
    }
    let mut files = BTreeMap::new();
    for import in imports {
        let name = import
            .name
            .as_ref()
            .ok_or_else(|| error("import has no filename"))?;
        if name == file_name || files.insert(name.clone(), import.clone()).is_some() {
            return Err(error(format!("duplicate descriptor filename {name}")));
        }
    }
    for import in imports {
        for dependency in &import.dependency {
            if !files.contains_key(dependency) {
                return Err(error(format!(
                    "import {} lacks transitive dependency {dependency}",
                    import.name.as_deref().unwrap_or_default()
                )));
            }
        }
    }
    let metadata = metadata::OptionEncoder::new(imports)?;
    let mut lower = Lower {
        graph,
        metadata: &metadata,
        package,
        direction,
        uses_validation: false,
        uses_registry: false,
    };
    let mut messages = Vec::new();
    for definition in graph.definitions().values() {
        let mut message = DescriptorProto {
            name: Some(definition.name.clone()),
            options: metadata.message_options(definition)?.into(),
            reserved_range: definition
                .reserved_tags
                .iter()
                .map(|tag| descriptor::descriptor_proto::ReservedRange {
                    start: Some(*tag as i32),
                    end: Some((*tag + 1) as i32),
                    ..Default::default()
                })
                .collect(),
            reserved_name: definition.reserved_names.clone(),
            ..Default::default()
        };
        match &definition.shape {
            Shape::Record(fields) => lower.fields(&mut message, fields).map_err(|e| error(format!("{}:{}: {e}", definition.source, definition.rust_type)))?,
            Shape::Alternatives(alternatives) => {
                message.reserved_name = definition
                    .reserved_names
                    .iter()
                    .map(|name| snake_case(name))
                    .collect();
                let mut options = OneofOptions::default();
                options
                    .try_set_extension(
                        &validate::ONEOF,
                        OneofRules {
                            required: Some(true),
                            ..Default::default()
                        },
                    )
                    .map_err(|e| error(e.to_string()))?;
                lower.uses_validation = true;
                message.oneof_decl.push(OneofDescriptorProto {
                    name: Some("kind".into()),
                    options: options.into(),
                    ..Default::default()
                });
                for alternative in alternatives {
                    let nested_name = format!("Alternative{}", alternative.tag);
                    let mut nested = DescriptorProto {
                        name: Some(nested_name.clone()),
                        ..Default::default()
                    };
                    lower.fields(&mut nested, &alternative.fields).map_err(|e| error(format!("{}:{}::{}: {e}", definition.source, definition.rust_type, alternative.name)))?;
                    message.nested_type.push(nested);
                    let mut field_options = FieldOptions::default();
                    metadata.field_correspondence(&mut field_options, &alternative.name, definition.direction)?;
                    message.field.push(FieldDescriptorProto {
                        name: Some(snake_case(&alternative.name)),
                        number: Some(alternative.tag as i32),
                        label: Some(Label::LABEL_OPTIONAL),
                        r#type: Some(Type::TYPE_MESSAGE),
                        type_name: Some(format!(".{package}.{}.{}", definition.name, nested_name)),
                        oneof_index: Some(0),
                        options: field_options.into(),
                        ..Default::default()
                    });
                }
            }
            Shape::StringEnum(members) => {
                let mut field = FieldDescriptorProto {
                    name: Some("value".into()),
                    number: Some(1),
                    label: Some(Label::LABEL_OPTIONAL),
                    r#type: Some(Type::TYPE_STRING),
                    ..Default::default()
                };
                lower.options(
                    &mut field,
                    FieldRules {
                        r#type: Some(
                            StringRules {
                                r#in: members.clone(),
                                ..Default::default()
                            }
                            .into(),
                        ),
                        ..Default::default()
                    },
                )?;
                let mut options = field.options.as_option().cloned().unwrap_or_default();
                metadata.field_correspondence(&mut options, "value", definition.direction)?;
                field.options = options.into();
                message.field.push(field);
            }
        }
        qualify_nested(&mut message, package);
        validate_message_names(&message)?;
        messages.push(message);
    }
    if lower.uses_validation && !files.contains_key("buf/validate/validate.proto") {
        return Err(error(
            "local rules, fixed identities and required alternatives require the complete buf/validate/validate.proto import closure",
        ));
    }
    let mut services = Vec::new();
    for service in graph.services().values() {
        services.push(ServiceDescriptorProto {
            name: Some(service.name.clone()),
            method: service
                .methods
                .iter()
                .map(|method| {
                    let named = |ty: &TypeRef| match ty {
                        TypeRef::Named(name) => Ok(format!(".{package}.{name}")),
                        _ => Err(error("service root must be a named owner")),
                    };
                    Ok(MethodDescriptorProto {
                        name: Some(method.name.clone()),
                        input_type: Some(named(&method.input)?),
                        output_type: Some(named(&method.output)?),
                        client_streaming: Some(false),
                        server_streaming: Some(method.server_streaming),
                        options: metadata.method_options(method.effects).map_err(|e| error(format!("{}.{}: {e}", service.name, method.name)))?.into(),
                        ..Default::default()
                    })
                })
                .collect::<Result<Vec<_>, SchemaError>>()?,
            ..Default::default()
        });
    }
    let file = FileDescriptorProto {
        name: Some(file_name.into()),
        package: Some(package.into()),
        syntax: Some("proto3".into()),
        dependency: std::iter::once(metadata::CORE_IMPORT.into()).chain(lower.uses_validation.then(|| "buf/validate/validate.proto".into())).chain(lower.uses_registry.then(|| registry_facets::IMPORT.into())).collect(),
        message_type: messages,
        service: services,
        ..Default::default()
    };
    files.insert(file_name.into(), file);
    let descriptor_set = FileDescriptorSet {
        file: dependency_order(files)?,
        ..Default::default()
    };
    let validators: [(&str, fn(&buffa_descriptor::DynamicMessage) -> Result<(), SchemaError>); 1] = [(registry_facets::EXTENSION, registry_facets::check_option)];
    let owner_versions = graph.definitions().values().map(|owner| (format!("{package}.{}", owner.name), owner.version)).collect::<Vec<_>>();
    let interpretations = owner_versions.iter().map(|(name, version)| (name.as_str(), *version)).collect::<Vec<_>>();
    let names = metadata::native_selection(graph, package, direction);
    let owners = names.owners.iter().map(String::as_str).collect::<Vec<_>>();
    let fields = names.fields.iter().map(String::as_str).collect::<Vec<_>>();
    let methods = names.methods.iter().map(String::as_str).collect::<Vec<_>>();
    let selection = metadata::NativeSelection { owners: &owners, fields: &fields, methods: &methods };
    let policy = metadata::TypedPolicy::new(&validators, &interpretations, selection);
    let selected_extensions = if lower.uses_registry { &[registry_facets::EXTENSION][..] } else { &[] };
    let assessment = metadata::check_descriptors(&descriptor_set, file_name, &policy, selected_extensions)?;
    if assessment.disposition != pse_contract::metadata::Disposition::Compatible {
        return Err(error(format!("semantic metadata rejected by {}: {:?}", assessment.policy, assessment.findings)));
    }
    Ok(descriptor_set)
}

/// Generate owned messages, borrowed views and one complete shared reflection pool.
/// The default root is `crate::generated::__buffa_fds`; callers with another module use
/// [`generate_with_config`].
pub fn generate(
    graph: &ProjectionGraph,
    package: &str,
    file_name: &str,
) -> Result<ProjectionOutput, SchemaError> {
    generate_with_config(
        graph,
        package,
        file_name,
        &[],
        &default_config("crate::generated::__buffa_fds"),
    )
}

/// Selected product configuration, with an explicit shared root in the consuming crate.
pub fn default_config(shared_root: &str) -> CodeGenConfig {
    let mut config = CodeGenConfig::default();
    config.generate_json = true;
    config.generate_text = true;
    config.generate_reflection = true;
    config.generate_reflection_vtable = true;
    config.shared_descriptor_pool = true;
    config.shared_descriptor_pool_root = Some(shared_root.into());
    config.file_per_package = true;
    config
}

/// Generate with a complete import closure and the consumer's actual module layout.
pub fn generate_with_config(
    graph: &ProjectionGraph,
    package: &str,
    file_name: &str,
    imports: &[FileDescriptorProto],
    config: &CodeGenConfig,
) -> Result<ProjectionOutput, SchemaError> {
    if !config.generate_reflection
        || !config.generate_reflection_vtable
        || !config.shared_descriptor_pool
    {
        return Err(error(
            "owner projections require vtable reflection and one complete shared descriptor pool",
        ));
    }
    let descriptor_set = descriptors_with_imports(graph, package, file_name, imports)?;
    let descriptor_bytes = descriptor_set.encode_to_vec();
    let (generated, warnings) =
        buffa_codegen::generate_with_diagnostics(&descriptor_set.file, &[file_name.into()], config)
            .map_err(|e| error(e.to_string()))?;
    let entries = generated
        .iter()
        .filter(|file| file.kind == GeneratedFileKind::PackageMod)
        .map(|file| (file.name.clone(), file.package.clone()))
        .collect::<Vec<_>>();
    let mut module_tree =
        buffa_codegen::generate_module_tree(&entries, IncludeMode::Relative(""), false);
    module_tree.push_str("\ninclude!(\"shared-root.rs\");\n");
    let mut sources = generated
        .into_iter()
        .map(|file| (file.name, file.content))
        .collect::<BTreeMap<_, _>>();
    sources.insert(
        "shared-root.rs".into(),
        buffa_codegen::shared_descriptor_root_module(&descriptor_bytes, FdsEmbedding::Inline, None),
    );
    sources.insert("mod.rs".into(), module_tree);
    Ok(ProjectionOutput {
        descriptor_set,
        descriptor_bytes,
        sources,
        warnings: warnings.iter().map(ToString::to_string).collect(),
        documentation: owner_reference(graph, package),
    })
}

/// Render stable tags, constraints, interpretation versions and actual source links.
pub fn owner_reference(graph: &ProjectionGraph, package: &str) -> String {
    use std::fmt::Write;
    let mut out = format!(
        "# `{package}` boundary owners\n\nGenerated from actual typed declarations. Registry and contextual scientific admission retain authority.\n"
    );
    for owner in graph.definitions().values() {
        let _ = writeln!(
            out,
            "\n## `{}`\n\nRust owner: `{}`. Source: `{}`. Interpretation version: {}. Direction: `{:?}`.\n",
            owner.name, owner.rust_type, owner.source, owner.version, owner.direction
        );
        let _ = writeln!(
            out,
            "Reserved numbers: `{:?}`. Reserved names: `{:?}`.\n",
            owner.reserved_tags, owner.reserved_names
        );
        let _ = writeln!(
            out,
            "| Field or alternative | Stable tag | Actual type | Direction | Local rules | Original named facets |\n| --- | --- | --- | --- | --- | --- |"
        );
        let mut row = |prefix: &str, field: &Field| {
            let _ = writeln!(
                out,
                "| `{prefix}{}` | {} | `{:?}` | `{:?}` | `{:?}` | `{}` |",
                field.name,
                field.tag,
                field.ty,
                field.direction,
                field.rules,
                format!("{:?}", field.named_facets)
                    .replace('|', "\\|")
                    .replace('`', "&#96;")
            );
        };
        match &owner.shape {
            Shape::Record(fields) => {
                for field in fields {
                    row("", field);
                }
            }
            Shape::Alternatives(alternatives) => {
                for alternative in alternatives {
                    let prefix = format!(
                        "{} (alternative tag {}).",
                        alternative.name, alternative.tag
                    );
                    if alternative.fields.is_empty() {
                        row(
                            "",
                            &Field {
                                name: alternative.name.clone(),
                                tag: alternative.tag,
                                ty: TypeRef::Named("unit alternative".into()),
                                direction: owner.direction,
                                rules: vec![LocalRule::Required],
                                named_facets: Default::default(),
                            },
                        );
                    } else {
                        for field in &alternative.fields {
                            row(&prefix, field);
                        }
                    }
                }
            }
            Shape::StringEnum(members) => {
                row(
                    "",
                    &Field {
                        name: "value".into(),
                        tag: 1,
                        ty: TypeRef::Scalar(Scalar::String),
                        direction: owner.direction,
                        rules: vec![],
                        named_facets: Default::default(),
                    },
                );
                let _ = writeln!(out, "\nAllowed canonical members: `{members:?}`.");
            }
        }
    }
    for service in graph.services().values() {
        let _ = writeln!(out, "\n## Service `{}`\n", service.name);
        for method in &service.methods {
            let _ = writeln!(
                out,
                "- `{}`: `{:?}` -> `{:?}`{}.",
                method.name,
                method.input,
                method.output,
                if method.server_streaming {
                    " (server stream)"
                } else {
                    ""
                }
            );
            if let Some(effects) = method.effects {
                let _ = writeln!(
                    out,
                    "  Owner effects: `{:?}`; read only: {}; idempotent: {}; destructive: {}.",
                    effects.scope, effects.read_only, effects.idempotent, effects.destructive
                );
            }
        }
    }
    out
}

struct Lower<'a> {
    graph: &'a ProjectionGraph,
    metadata: &'a metadata::OptionEncoder,
    package: &'a str,
    direction: Direction,
    uses_validation: bool,
    uses_registry: bool,
}

impl Lower<'_> {
    fn fields(
        &mut self,
        message: &mut DescriptorProto,
        fields: &[Field],
    ) -> Result<(), SchemaError> {
        let direction = self.direction;
        for field in fields
            .iter()
            .filter(|field| field.direction.includes(direction))
        {
            let mut lowered = self.field(message, &field.name, field.tag, &field.ty)?;
            if !field.rules.is_empty() {
                let mut rules = self.base_rules(&field.ty);
                for rule in &field.rules {
                    apply_rule(&mut rules, &field.ty, rule, self.graph)?;
                }
                self.options(&mut lowered, rules)?;
                if field.rules.contains(&LocalRule::Required)
                    && lowered.label == Some(Label::LABEL_OPTIONAL)
                    && lowered.r#type != Some(Type::TYPE_MESSAGE)
                    && lowered.oneof_index.is_none()
                {
                    lowered.proto3_optional = Some(true);
                    lowered.oneof_index = Some(message.oneof_decl.len() as i32);
                    message.oneof_decl.push(OneofDescriptorProto {
                        name: Some(format!("_{}", field.name)),
                        ..Default::default()
                    });
                }
            }
            let mut options = lowered.options.as_option().cloned().unwrap_or_default();
            self.metadata.field_options(&mut options, field)?;
            if let Some(facets) = registry_facets::field_options(field, self.graph, self.metadata)? {
                self.metadata.attach(&mut options, registry_facets::EXTENSION, facets)?;
                self.uses_registry = true;
            }
            lowered.options = options.into();
            message.field.push(lowered);
        }
        Ok(())
    }

    fn field(
        &mut self,
        parent: &mut DescriptorProto,
        name: &str,
        tag: u32,
        ty: &TypeRef,
    ) -> Result<FieldDescriptorProto, SchemaError> {
        let mut field = FieldDescriptorProto {
            name: Some(name.into()),
            number: Some(tag as i32),
            label: Some(Label::LABEL_OPTIONAL),
            ..Default::default()
        };
        match ty {
            TypeRef::Scalar(scalar) => {
                field.r#type = Some(match scalar {
                    Scalar::Bool => Type::TYPE_BOOL,
                    Scalar::String => Type::TYPE_STRING,
                    Scalar::U8 | Scalar::U16 | Scalar::U32 => Type::TYPE_UINT32,
                    Scalar::U64 => Type::TYPE_UINT64,
                    Scalar::I16 | Scalar::I32 => Type::TYPE_SINT32,
                    Scalar::I64 => Type::TYPE_SINT64,
                    Scalar::F32Bits => Type::TYPE_FIXED32,
                    Scalar::F64Bits => Type::TYPE_FIXED64,
                    Scalar::Bytes | Scalar::SemanticId | Scalar::ContentHash => Type::TYPE_BYTES,
                });
                if matches!(
                    scalar,
                    Scalar::SemanticId
                        | Scalar::ContentHash
                        | Scalar::U8
                        | Scalar::U16
                        | Scalar::I16
                ) {
                    self.options(&mut field, self.base_rules(ty))?;
                }
            }
            TypeRef::Named(owner) => {
                if !self.graph.definitions().contains_key(owner) {
                    return Err(error(format!("unresolved owner {owner}")));
                }
                field.r#type = Some(Type::TYPE_MESSAGE);
                field.type_name = Some(format!(".{}.{owner}", self.package));
            }
            TypeRef::Optional(inner) => match inner.as_ref() {
                TypeRef::Scalar(_) | TypeRef::Named(_) => {
                    field = self.field(parent, name, tag, inner)?;
                    if matches!(inner.as_ref(), TypeRef::Scalar(_)) {
                        field.proto3_optional = Some(true);
                        field.oneof_index = Some(parent.oneof_decl.len() as i32);
                        parent.oneof_decl.push(OneofDescriptorProto {
                            name: Some(format!("_{name}")),
                            ..Default::default()
                        });
                    }
                }
                _ => self.wrapper(parent, &mut field, tag, inner)?,
            },
            TypeRef::Repeated(inner) | TypeRef::Set(inner) => {
                if matches!(inner.as_ref(), TypeRef::Scalar(_) | TypeRef::Named(_)) {
                    field = self.field(parent, name, tag, inner)?;
                } else {
                    self.wrapper(parent, &mut field, tag, inner)?;
                }
                field.label = Some(Label::LABEL_REPEATED);
                let rules = self.base_rules(ty);
                if rules.r#type.is_some() || !rules.cel.is_empty() {
                    self.options(&mut field, rules)?;
                }
            }
            TypeRef::Map(key, value) => {
                let entry_name = format!("Field{tag}Entry");
                let mut entry = DescriptorProto {
                    name: Some(entry_name.clone()),
                    options: descriptor::MessageOptions {
                        map_entry: Some(protobuf_map_key(key)),
                        ..Default::default()
                    }
                    .into(),
                    ..Default::default()
                };
                let mut key_field = self.field(&mut entry, "key", 1, key)?;
                let mut value_field =
                    if matches!(value.as_ref(), TypeRef::Scalar(_) | TypeRef::Named(_)) {
                        self.field(&mut entry, "value", 2, value)?
                    } else {
                        let mut field = FieldDescriptorProto {
                            name: Some("value".into()),
                            number: Some(2),
                            label: Some(Label::LABEL_OPTIONAL),
                            ..Default::default()
                        };
                        self.wrapper(&mut entry, &mut field, 2, value)?;
                        field
                    };
                if !protobuf_map_key(key) {
                    for field in [&mut key_field, &mut value_field] {
                        if field.r#type == Some(Type::TYPE_MESSAGE) {
                            let mut rules = field
                                .options
                                .as_option()
                                .and_then(|options| options.extension(&validate::FIELD))
                                .unwrap_or_default();
                            rules.required = Some(true);
                            self.options(field, rules)?;
                        }
                    }
                }
                entry.field.extend([key_field, value_field]);
                field.label = Some(Label::LABEL_REPEATED);
                field.r#type = Some(Type::TYPE_MESSAGE);
                // Relative nested names work at every nesting depth.
                field.type_name = Some(entry_name);
                parent.nested_type.push(entry);
                let rules = self.base_rules(ty);
                if rules.r#type.is_some() || !rules.cel.is_empty() {
                    self.options(&mut field, rules)?;
                }
            }
        }
        Ok(field)
    }

    fn wrapper(
        &mut self,
        parent: &mut DescriptorProto,
        field: &mut FieldDescriptorProto,
        tag: u32,
        inner: &TypeRef,
    ) -> Result<(), SchemaError> {
        let name = format!("Field{tag}Value");
        let mut wrapper = DescriptorProto {
            name: Some(name.clone()),
            ..Default::default()
        };
        let value = self.field(&mut wrapper, "value", 1, inner)?;
        wrapper.field.push(value);
        field.r#type = Some(Type::TYPE_MESSAGE);
        field.type_name = Some(name);
        parent.nested_type.push(wrapper);
        Ok(())
    }

    fn base_rules(&self, ty: &TypeRef) -> FieldRules {
        let mut rules = FieldRules::default();
        match ty {
            TypeRef::Set(inner)
                if matches!(inner.as_ref(),TypeRef::Named(name)
                if self.graph.definitions().get(name).is_some_and(|owner|matches!(owner.shape,Shape::StringEnum(_)))) =>
            {
                // String vocabularies retain their source-owned wrapper. The
                // standard repeated.unique rule does not support message items;
                // maintained CEL compares their actual declared scalar value.
                rules.cel.push(Rule {
                    id: Some("set.unique_vocabulary".into()),
                    message: Some("set must not repeat a declared vocabulary member".into()),
                    expression: Some(
                        "this.all(x, this.filter(y, y.value == x.value).size() == 1)".into(),
                    ),
                    ..Default::default()
                });
            }
            TypeRef::Scalar(Scalar::I16) => {
                rules.r#type = Some(
                    SInt32Rules {
                        greater_than: Some(s_int32rules::GreaterThan::Gte(i32::from(i16::MIN))),
                        less_than: Some(s_int32rules::LessThan::Lte(i32::from(i16::MAX))),
                        ..Default::default()
                    }
                    .into(),
                );
            }
            TypeRef::Scalar(Scalar::SemanticId | Scalar::ContentHash) => {
                let len = if matches!(ty, TypeRef::Scalar(Scalar::SemanticId)) {
                    16
                } else {
                    32
                };
                rules.r#type = Some(
                    BytesRules {
                        len: Some(len),
                        ..Default::default()
                    }
                    .into(),
                );
            }
            TypeRef::Scalar(Scalar::U8) => {
                rules.r#type = Some(
                    UInt32Rules {
                        less_than: Some(u_int32rules::LessThan::Lte(255)),
                        ..Default::default()
                    }
                    .into(),
                );
            }
            TypeRef::Scalar(Scalar::U16) => {
                rules.r#type = Some(
                    UInt32Rules {
                        less_than: Some(u_int32rules::LessThan::Lte(u32::from(u16::MAX))),
                        ..Default::default()
                    }
                    .into(),
                );
            }
            TypeRef::Repeated(inner) | TypeRef::Set(inner)
                if matches!(inner.as_ref(), TypeRef::Scalar(_)) =>
            {
                let item = self.base_rules(inner);
                let unique = matches!(ty, TypeRef::Set(_));
                if item.r#type.is_some() || unique {
                    rules.r#type = Some(
                        RepeatedRules {
                            items: item.into(),
                            unique: unique.then_some(true),
                            ..Default::default()
                        }
                        .into(),
                    );
                }
            }
            TypeRef::Map(key, value) if protobuf_map_key(key) => {
                let keys = self.base_rules(key);
                let values = self.base_rules(value);
                if keys.r#type.is_some() || values.r#type.is_some() {
                    rules.r#type = Some(
                        MapRules {
                            keys: keys.into(),
                            values: values.into(),
                            ..Default::default()
                        }
                        .into(),
                    );
                }
            }
            _ => {}
        }
        rules
    }

    fn options(
        &mut self,
        field: &mut FieldDescriptorProto,
        rules: FieldRules,
    ) -> Result<(), SchemaError> {
        let mut options = FieldOptions::default();
        options
            .try_set_extension(&validate::FIELD, rules)
            .map_err(|e| error(e.to_string()))?;
        field.options = options.into();
        self.uses_validation = true;
        Ok(())
    }
}

fn apply_rule(
    rules: &mut FieldRules,
    ty: &TypeRef,
    rule: &LocalRule,
    graph: &ProjectionGraph,
) -> Result<(), SchemaError> {
    let source_type = ty;
    let ty = match (rule, ty) {
        (
            LocalRule::UnsignedRange { .. }
            | LocalRule::SignedRange { .. }
            | LocalRule::Utf8Bytes { .. }
            | LocalRule::Float64Domain(_),
            TypeRef::Optional(inner),
        ) => inner.as_ref(),
        _ => ty,
    };
    match rule {
        LocalRule::PositiveDuration => {
            let ty = match ty {
                TypeRef::Optional(inner) => inner.as_ref(),
                ty => ty,
            };
            let TypeRef::Named(name) = ty else {
                return Err(error("positive duration requires its actual named owner"));
            };
            let Some(pse_contract::boundary_projection::NativeContainer::Duration {
                seconds_field,
                nanoseconds_field,
            }) = graph
                .definitions()
                .get(name)
                .and_then(|owner| owner.native_container.as_ref())
            else {
                return Err(error(
                    "positive duration requires the source-proven standard Duration owner",
                ));
            };
            rules.cel.push(Rule {
                id: Some("duration.positive".into()),
                message: Some("a positive duration is required".into()),
                expression: Some(format!(
                    "this.{seconds_field} > 0u || this.{nanoseconds_field} > 0u"
                )),
                ..Default::default()
            });
        }
        LocalRule::Float64Domain(domain) => {
            if !matches!(ty, TypeRef::Scalar(Scalar::F64Bits)) {
                rules.cel.push(Rule {
                    id: Some("float64.collection_domain".into()),
                    message: Some(format!("binary64 leaves must satisfy {domain:?}")),
                    expression: Some(float_collections::collection_expression(
                        graph,
                        source_type,
                        *domain,
                    )?),
                    ..Default::default()
                });
                return Ok(());
            }
            let interval = domain.interval();
            if !interval.is_valid() {
                return Err(error("floating domain has no finite represented values"));
            }
            match interval.bit_intervals() {
                [Some((minimum, maximum)), None] | [None, Some((minimum, maximum))] => {
                    rules.r#type = Some(
                        Fixed64Rules {
                            greater_than: minimum.checked_sub(1).map(fixed64rules::GreaterThan::Gt),
                            less_than: Some(fixed64rules::LessThan::Lte(maximum)),
                            ..Default::default()
                        }
                        .into(),
                    );
                }
                _ => {
                    let (id, message) = match domain {
                        pse_contract::boundary_projection::Float64Domain::Finite => {
                            ("float64.finite", "binary64 value must be finite")
                        }
                        pse_contract::boundary_projection::Float64Domain::NonnegativeFinite => (
                            "float64.nonnegative_finite",
                            "binary64 value must be finite and nonnegative",
                        ),
                        _ => (
                            "float64.interval",
                            "binary64 value must satisfy its source-owned finite interval",
                        ),
                    };
                    rules.cel.push(Rule {
                        id: Some(id.into()),
                        message: Some(message.into()),
                        expression: Some(float_collections::scalar_expression(*domain, "this")?),
                        ..Default::default()
                    });
                }
            }
        }
        LocalRule::Required => rules.required = Some(true),
        LocalRule::SignedRange { min, max } => match ty {
            TypeRef::Scalar(Scalar::I16 | Scalar::I32) => {
                let min =
                    i32::try_from(*min).map_err(|_| error("signed32 lower bound overflow"))?;
                let max =
                    i32::try_from(*max).map_err(|_| error("signed32 upper bound overflow"))?;
                rules.r#type = Some(
                    SInt32Rules {
                        greater_than: Some(s_int32rules::GreaterThan::Gte(min)),
                        less_than: Some(s_int32rules::LessThan::Lte(max)),
                        ..Default::default()
                    }
                    .into(),
                );
            }
            TypeRef::Scalar(Scalar::I64) => {
                rules.r#type = Some(
                    SInt64Rules {
                        greater_than: Some(s_int64rules::GreaterThan::Gte(*min)),
                        less_than: Some(s_int64rules::LessThan::Lte(*max)),
                        ..Default::default()
                    }
                    .into(),
                );
            }
            _ => return Err(error("signed range requires actual signed scalar")),
        },
        LocalRule::Utf8Bytes { min, max } => {
            if !matches!(ty, TypeRef::Scalar(Scalar::String)) {
                return Err(error("UTF-8 byte rule requires actual string type"));
            }
            rules.r#type = Some(
                StringRules {
                    min_bytes: (*min != 0).then_some(*min),
                    max_bytes: Some(*max),
                    ..Default::default()
                }
                .into(),
            );
        }
        LocalRule::UnsignedRange { min, max } => match ty {
            TypeRef::Scalar(Scalar::U8 | Scalar::U16 | Scalar::U32) => {
                let lower =
                    u32::try_from(*min).map_err(|_| error("uint32 rule lower bound overflow"))?;
                let upper =
                    u32::try_from(*max).map_err(|_| error("uint32 rule upper bound overflow"))?;
                let native_maximum = match ty {
                    TypeRef::Scalar(Scalar::U8) => u32::from(u8::MAX),
                    TypeRef::Scalar(Scalar::U16) => u32::from(u16::MAX),
                    _ => u32::MAX,
                };
                if upper > native_maximum {
                    return Err(error("unsigned range exceeds its actual native width"));
                }
                rules.r#type = Some(
                    UInt32Rules {
                        // The unsigned domain already establishes this exact lower bound.
                        greater_than: (lower != 0).then_some(u_int32rules::GreaterThan::Gte(lower)),
                        less_than: (upper != u32::MAX)
                            .then_some(u_int32rules::LessThan::Lte(upper)),
                        ..Default::default()
                    }
                    .into(),
                );
            }
            TypeRef::Scalar(Scalar::U64) => {
                rules.r#type = Some(
                    UInt64Rules {
                        greater_than: (*min != 0).then_some(u_int64rules::GreaterThan::Gte(*min)),
                        less_than: (*max != u64::MAX).then_some(u_int64rules::LessThan::Lte(*max)),
                        ..Default::default()
                    }
                    .into(),
                );
            }
            _ => return Err(error("unsigned range rule has unsupported actual type")),
        },
        LocalRule::Length { min, max } => {
            rules.r#type = Some(match ty {
                TypeRef::Scalar(Scalar::String) => StringRules {
                    min_len: (*min != 0).then_some(*min),
                    max_len: Some(*max),
                    ..Default::default()
                }
                .into(),
                TypeRef::Scalar(Scalar::Bytes | Scalar::SemanticId | Scalar::ContentHash) => {
                    BytesRules {
                        min_len: (*min != 0).then_some(*min),
                        max_len: Some(*max),
                        len: match ty {
                            TypeRef::Scalar(Scalar::SemanticId) => Some(16),
                            TypeRef::Scalar(Scalar::ContentHash) => Some(32),
                            _ => None,
                        },
                        ..Default::default()
                    }
                    .into()
                }
                TypeRef::Repeated(_) | TypeRef::Set(_) => {
                    let previous = match rules.r#type.take() {
                        Some(validate::__buffa::oneof::field_rules::Type::Repeated(previous)) => {
                            *previous
                        }
                        _ => RepeatedRules::default(),
                    };
                    RepeatedRules {
                        min_items: (*min != 0).then_some(*min),
                        max_items: Some(*max),
                        ..previous
                    }
                    .into()
                }
                TypeRef::Map(key, _) if protobuf_map_key(key) => {
                    let previous = match rules.r#type.take() {
                        Some(validate::__buffa::oneof::field_rules::Type::Map(previous)) => {
                            *previous
                        }
                        _ => MapRules::default(),
                    };
                    MapRules {
                        min_pairs: (*min != 0).then_some(*min),
                        max_pairs: Some(*max),
                        ..previous
                    }
                    .into()
                }
                TypeRef::Map(_, _) => RepeatedRules {
                    min_items: (*min != 0).then_some(*min),
                    max_items: Some(*max),
                    ..Default::default()
                }
                .into(),
                _ => return Err(error("length rule has unsupported actual type")),
            });
        }
    }
    Ok(())
}

fn identifier(value: &str) -> bool {
    let mut bytes = value.bytes();
    bytes
        .next()
        .is_some_and(|byte| byte.is_ascii_alphabetic() || byte == b'_')
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

fn protobuf_map_key(ty: &TypeRef) -> bool {
    matches!(
        ty,
        TypeRef::Scalar(
            Scalar::Bool
                | Scalar::String
                | Scalar::U8
                | Scalar::U16
                | Scalar::U32
                | Scalar::U64
                | Scalar::I16
                | Scalar::I32
                | Scalar::I64
        )
    )
}

fn qualify_nested(message: &mut DescriptorProto, prefix: &str) {
    let scope = format!("{prefix}.{}", message.name.as_deref().unwrap_or_default());
    for field in &mut message.field {
        if let Some(name) = &mut field.type_name {
            if !name.starts_with('.') {
                *name = format!(".{scope}.{name}");
            }
        }
    }
    for nested in &mut message.nested_type {
        qualify_nested(nested, &scope);
    }
}

fn validate_message_names(message: &DescriptorProto) -> Result<(), SchemaError> {
    let mut names = std::collections::BTreeSet::new();
    let reserved = message
        .reserved_name
        .iter()
        .collect::<std::collections::BTreeSet<_>>();
    for field in &message.field {
        let name = field
            .name
            .as_ref()
            .ok_or_else(|| error("projected field has no name"))?;
        if !names.insert(name) || reserved.contains(name) {
            return Err(error(format!(
                "{} has duplicate or reserved public field name {name}",
                message.name.as_deref().unwrap_or_default()
            )));
        }
    }
    for oneof in &message.oneof_decl {
        let name = oneof
            .name
            .as_ref()
            .ok_or_else(|| error("projected oneof has no name"))?;
        if !names.insert(name) {
            return Err(error(format!(
                "{} has colliding field/oneof name {name}",
                message.name.as_deref().unwrap_or_default()
            )));
        }
    }
    for nested in &message.nested_type {
        validate_message_names(nested)?;
    }
    Ok(())
}

fn dependency_order(
    mut files: BTreeMap<String, FileDescriptorProto>,
) -> Result<Vec<FileDescriptorProto>, SchemaError> {
    let mut ordered = Vec::new();
    while !files.is_empty() {
        let ready = files
            .iter()
            .find(|(_, file)| file.dependency.iter().all(|name| !files.contains_key(name)))
            .map(|(name, _)| name.clone());
        let Some(name) = ready else {
            return Err(error("descriptor imports contain a dependency cycle"));
        };
        if let Some(file) = files.remove(&name) {
            ordered.push(file);
        }
    }
    Ok(ordered)
}

fn snake_case(value: &str) -> String {
    let mut output = String::new();
    for (index, ch) in value.chars().enumerate() {
        if ch.is_ascii_uppercase() && index != 0 {
            output.push('_');
        }
        output.push(ch.to_ascii_lowercase());
    }
    output
}

#[cfg(test)]
mod tests;
