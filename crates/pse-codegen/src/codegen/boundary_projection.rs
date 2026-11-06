// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Deterministic lowering of actual owner-emitted types, without JSON Schema inference.

use std::collections::BTreeMap;

use buffa::{ExtensionSet, Message};
use buffa_codegen::{
    CodeGenConfig, CodeGenWarning, FdsEmbedding, GeneratedFile, GeneratedFileKind, IncludeMode,
    SharedCorpusContext,
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
mod source_error;
use pse_contract_proto::{CORE_IMPORT, NativeSelection, OptionEncoder, check_descriptors};
pub use source_error::ProjectionCodegenError;

// Internal source-generation carrier only. Published options carry standard Arrow IPC.
pub(crate) const REGISTRY_FIELD_SOURCE: &str = "pse.codegen.native_field";

mod adapters;
mod float_collections;
mod lazy_probe;
mod registry_facets;
mod source_metadata;
pub use adapters::{
    AdapterVisibility, NativeViewTarget, native_adapters, native_adapters_retaining_wire,
    native_adapters_scoped, native_view_adapters,
};
pub use lazy_probe::{LazyProbeTarget, lazy_decode_probe};

/// One complete generation closure. The caller owns publication and generated roots.
#[derive(Debug)]
pub struct ProjectionOutput {
    /// Admitted source-aware generation, including typed supplier outputs and warnings.
    pub generation: ProjectionGeneration,
    /// Effective runtime world, including imports, options and configured feature overrides.
    /// Validators, clients and reflection must consume this same interpretation.
    pub descriptor_set: FileDescriptorSet,
    /// Effective runtime bytes with source information removed; publish as `descriptor.bin`.
    /// The shared reflection root embeds this sidecar with `include_bytes!`.
    pub descriptor_bytes: Vec<u8>,
    /// Full effective descriptors with original source information restored.
    /// Publish separately for source inspection; feature meanings match the runtime artifact.
    pub source_descriptor_bytes: Vec<u8>,
    /// Reflection and module-tree support files, separate from typed Buffa output.
    pub support_sources: BTreeMap<String, String>,
    /// Inspectable field/tag/constraint and source-owner reference from this same graph.
    pub documentation: String,
}

impl ProjectionOutput {
    /// All message and support sources without discarding supplier metadata.
    pub fn sources(&self) -> impl Iterator<Item = (&str, &str)> {
        self.generation
            .files
            .iter()
            .map(|file| (file.name.as_str(), file.content.as_str()))
            .chain(
                self.support_sources
                    .iter()
                    .map(|(name, content)| (name.as_str(), content.as_str())),
            )
    }

    /// Inspect one generated source without creating a second filename map.
    pub fn source(&self, name: &str) -> Option<&str> {
        self.sources()
            .find_map(|(actual, content)| (actual == name).then_some(content))
    }
}

/// One effective source-aware corpus and the configuration admitted by Buffa.
/// Downstream suppliers borrow these exact inputs rather than rebuilding a world.
#[derive(Debug)]
pub struct ProjectionGeneration {
    descriptors: FileDescriptorSet,
    targets: Vec<String>,
    config: CodeGenConfig,
    /// Original package and output-kind metadata, before publication flattening.
    pub files: Vec<GeneratedFile>,
    /// Original typed diagnostics, including unmatched feature overrides.
    pub warnings: Vec<CodeGenWarning>,
}

/// Graph-neutral descriptor and module support for one admitted generation.
#[derive(Debug)]
pub struct ProjectionSupport {
    /// Effective shared reflection world, without source locations.
    pub descriptor_set: FileDescriptorSet,
    /// Bytes embedded by the shared reflection root.
    pub descriptor_bytes: Vec<u8>,
    /// Effective source-aware descriptor bytes.
    pub source_descriptor_bytes: Vec<u8>,
    /// Buffa-owned module tree and shared reflection root.
    pub sources: BTreeMap<String, String>,
}

impl ProjectionGeneration {
    /// Apply effective features while preserving original source information, then
    /// perform Buffa's fallible post-override shared-context check and generation.
    pub fn new(
        source: &FileDescriptorSet,
        targets: &[String],
        config: &CodeGenConfig,
    ) -> Result<Self, ProjectionCodegenError> {
        let mut names = std::collections::BTreeSet::new();
        for file in &source.file {
            let name = file
                .name
                .as_deref()
                .ok_or_else(|| error("descriptor has no logical filename"))?;
            if !names.insert(name) {
                return Err(error(format!("duplicate descriptor filename {name}")));
            }
        }
        if targets.is_empty() {
            return Err(error("explicit generation targets are required"));
        }
        let excludes = config
            .exclude_packages
            .iter()
            .map(|entry| {
                buffa_codegen::normalize_exclude_package(entry).map_err(|reason| {
                    buffa_codegen::CodeGenError::Other(format!(
                        "exclude_package {entry:?}: {reason}"
                    ))
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut selected = std::collections::BTreeSet::new();
        for target in targets {
            if !selected.insert(target) {
                return Err(error(format!("duplicate generation target {target}")));
            }
            let file = source
                .file
                .iter()
                .find(|file| file.name.as_deref() == Some(target.as_str()))
                .ok_or_else(|| {
                    error(format!(
                        "generation target missing from descriptor: {target}"
                    ))
                })?;
            let package = file.package.as_deref().unwrap_or("");
            // Buffa alone filters selected excluded packages. Refuse that
            // divergent request before any supplier runs, keeping the complete
            // descriptor corpus (including excluded imports) available below.
            if buffa_codegen::package_is_excluded(package, &excludes) {
                return Err(ProjectionCodegenError::ExcludedTarget {
                    file: target.clone(),
                    package: package.into(),
                });
            }
        }
        let runtime = effective_runtime_descriptors(source, config)?;
        let descriptors = source_aware_descriptors(source, &runtime)?;
        let mut config = config.clone();
        if config.shared_corpus_context.is_none() {
            config.shared_corpus_context =
                Some(SharedCorpusContext::new(&descriptors.file, &config));
        }
        // Generate from the original source so unmatched override diagnostics are
        // preserved. Buffa checks the shared corpus after applying those overrides.
        let (files, warnings) =
            buffa_codegen::generate_with_diagnostics(&source.file, targets, &config)?;
        check_generated_files(&files)?;
        Ok(Self {
            descriptors,
            targets: targets.to_vec(),
            config,
            files,
            warnings,
        })
    }

    /// Full effective descriptors, retaining genuine available source information.
    pub fn descriptors(&self) -> &FileDescriptorSet {
        &self.descriptors
    }
    /// Explicit target population admitted before any supplier executes.
    pub fn targets(&self) -> &[String] {
        &self.targets
    }
    /// Actual admitted configuration, including the shared corpus context.
    pub fn config(&self) -> &CodeGenConfig {
        &self.config
    }

    /// Assemble reflection and module support from this actual admitted corpus.
    /// The caller publishes the runtime bytes under the selected sidecar name.
    pub fn support(
        &self,
        runtime_sidecar: &str,
    ) -> Result<ProjectionSupport, ProjectionCodegenError> {
        if !self.config.generate_reflection
            || !self.config.generate_reflection_vtable
            || !self.config.shared_descriptor_pool
        {
            return Err(error(
                "shared support requires vtable reflection and one complete shared descriptor pool",
            ));
        }
        let source_descriptor_bytes = self.descriptors.encode_to_vec();
        let mut descriptor_set = self.descriptors.clone();
        for file in &mut descriptor_set.file {
            file.source_code_info = Default::default();
        }
        let descriptor_bytes = descriptor_set.encode_to_vec();
        let entries = self
            .files
            .iter()
            .filter(|file| file.kind == GeneratedFileKind::PackageMod)
            .map(|file| (file.name.clone(), file.package.clone()))
            .collect::<Vec<_>>();
        let mut module_tree =
            buffa_codegen::generate_module_tree(&entries, IncludeMode::Relative(""), false);
        module_tree.push_str("\ninclude!(\"shared-root.rs\");\n");
        let sources = BTreeMap::from([
            ("mod.rs".into(), module_tree),
            (
                "shared-root.rs".into(),
                buffa_codegen::shared_descriptor_root_module(
                    &descriptor_bytes,
                    FdsEmbedding::Sidecar {
                        file_name: runtime_sidecar,
                        mode: IncludeMode::Relative(""),
                    },
                    self.config.reflect_feature_gate(),
                ),
            ),
        ]);
        Ok(ProjectionSupport {
            descriptor_set,
            descriptor_bytes,
            source_descriptor_bytes,
            sources,
        })
    }
}

fn check_generated_files(files: &[GeneratedFile]) -> Result<(), ProjectionCodegenError> {
    let mut names = std::collections::BTreeSet::new();
    for file in files {
        if !names.insert(&file.name) || matches!(file.name.as_str(), "mod.rs" | "shared-root.rs") {
            return Err(error(format!("duplicate generated output {}", file.name)));
        }
    }
    check_package_mounts(
        files
            .iter()
            .filter(|file| file.kind == GeneratedFileKind::PackageMod)
            .map(|file| file.package.as_str()),
    )
}

/// Admit actual package mounts using the same identifier escaping as Buffa's
/// module-tree renderer, including collisions at intermediate module segments.
pub fn check_package_mounts<'a>(
    packages: impl IntoIterator<Item = &'a str>,
) -> Result<(), ProjectionCodegenError> {
    let mut selected = std::collections::BTreeSet::new();
    let mut mounts = BTreeMap::new();
    for package in packages {
        if !selected.insert(package) {
            return Err(error(format!(
                "duplicate generated package mount {package}"
            )));
        }
        if package.is_empty() {
            continue;
        }
        let mut path = Vec::new();
        let mut original = Vec::new();
        for segment in package.split('.') {
            path.push(buffa_codegen::idents::escape_mod_ident(segment));
            original.push(segment);
            let prefix = original.join(".");
            if let Some(previous) = mounts.get(&path)
                && previous != &prefix
            {
                return Err(error(format!(
                    "generated package module collision: {previous} and {prefix}"
                )));
            }
            mounts.insert(path.clone(), prefix);
        }
    }
    Ok(())
}

fn error(reason: impl Into<String>) -> ProjectionCodegenError {
    pse_schema::SchemaError::Codegen {
        language: "protobuf",
        reason: reason.into(),
    }
    .into()
}

// Eager inverse adapters and deferred probes must resolve the very same final
// descriptor names, including generated wrapper modules and configured prefixes.
fn local_view_path(
    naming: &buffa_codegen::context::CodeGenContext<'_>,
    package: &str,
    wire_root: &str,
    fqn: &str,
    suffix: &str,
    lifetime: &str,
) -> Result<String, ProjectionCodegenError> {
    let split = naming
        .rust_type_relative_split(fqn, package, 0)
        .ok_or_else(|| error(format!("{fqn}: actual Buffa view naming is unavailable")))?;
    if split.is_extern || !split.to_package.is_empty() {
        return Err(error(format!(
            "{fqn}: view must belong to the explicitly selected generated package"
        )));
    }
    let mut path = syn::parse_str::<syn::Path>(&split.within_package)
        .map_err(|source| error(format!("actual Buffa view path is invalid: {source}")))?;
    let terminal = path
        .segments
        .last_mut()
        .ok_or_else(|| error("actual Buffa message path is empty"))?;
    terminal.ident = quote::format_ident!(
        "{}{suffix}",
        terminal.ident.to_string().trim_start_matches("r#")
    );
    let family = if suffix == "LazyView" {
        "lazy_view"
    } else {
        "view"
    };
    Ok(format!(
        "{wire_root}::__buffa::{family}::{}<{lifetime}>",
        quote::quote!(#path)
    ))
}

fn validate_package(package: &str) -> Result<(), ProjectionCodegenError> {
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
    Ok(())
}

/// Emit immutable native interpretation/coverage constants from the actual source
/// graph, for independent consumers that cannot import the original native owner.
/// Acquired descriptor contents never approve their own missing source coverage.
pub fn native_policy_source(
    graph: &ProjectionGraph,
    package: &str,
    direction: Direction,
) -> Result<String, ProjectionCodegenError> {
    validate_package(package)?;
    let selection = source_metadata::native_selection(graph, package, direction);
    if selection.owners.is_empty() {
        return Err(error(
            "native source policy requires at least one actual owner",
        ));
    }
    Ok(format!(
        "// Source-derived immutable interpretation and coverage; no native authority.\n/// Exact source-owned interpretation versions available to this consumer.\npub const INTERPRETATIONS: &[(&str, u32)] = &{:?};\n/// Required source-selected coverage, independent of acquired option presence.\npub const NATIVE_SELECTION: pse_contract_proto::NativeSelection<'static> = pse_contract_proto::NativeSelection {{ owners: &{:?}, fields: &{:?}, methods: &{:?}, facets: &{:?}, facet_fields: &{:?} }};\n",
        selection.interpretations,
        selection.owners,
        selection.fields,
        selection.methods,
        selection.facets,
        selection.facet_fields,
    ))
}

/// Lower a graph without external imports. Required typed metadata imports refuse.
pub fn descriptors(
    graph: &ProjectionGraph,
    package: &str,
    file_name: &str,
) -> Result<FileDescriptorSet, ProjectionCodegenError> {
    descriptors_with_imports(graph, package, file_name, &[])
}

/// Lower a graph and retain its complete provided import world.
/// Imported files are preserved, including custom options, and duplicate filenames refuse.
pub fn descriptors_with_imports(
    graph: &ProjectionGraph,
    package: &str,
    file_name: &str,
    imports: &[FileDescriptorProto],
) -> Result<FileDescriptorSet, ProjectionCodegenError> {
    descriptors_for_direction(graph, package, file_name, imports, Direction::Both)
}

/// Lower selected input/output fields while retaining stable names, tags and reservations.
pub fn descriptors_for_direction(
    graph: &ProjectionGraph,
    package: &str,
    file_name: &str,
    imports: &[FileDescriptorProto],
    direction: Direction,
) -> Result<FileDescriptorSet, ProjectionCodegenError> {
    validate_package(package)?;
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
    let metadata = OptionEncoder::new(imports)?;
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
            options: metadata.message_options(definition, package)?.into(),
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
            Shape::Record(fields) => lower
                .fields(&mut message, fields)
                .map_err(|e| e.at(definition.source, definition.rust_type))?,
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
                    lower
                        .fields(&mut nested, &alternative.fields)
                        .map_err(|e| {
                            e.at(
                                definition.source,
                                format!("{}::{}", definition.rust_type, alternative.name),
                            )
                        })?;
                    message.nested_type.push(nested);
                    let mut field_options = FieldOptions::default();
                    metadata.field_correspondence(
                        &mut field_options,
                        &alternative.name,
                        definition.direction,
                        &TypeRef::Named(definition.name.clone()),
                        &[],
                        package,
                    )?;
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
                metadata.field_correspondence(
                    &mut options,
                    "value",
                    definition.direction,
                    &TypeRef::Named(definition.name.clone()),
                    &[],
                    package,
                )?;
                field.options = options.into();
                message.field.push(field);
            }
        }
        qualify_nested(&mut message, package)?;
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
                        options: metadata
                            .method_options(method.effects)
                            .map_err(|e| e.context(format!("{}.{}", service.name, method.name)))?
                            .into(),
                        ..Default::default()
                    })
                })
                .collect::<Result<Vec<_>, ProjectionCodegenError>>()?,
            ..Default::default()
        });
    }
    let file = FileDescriptorProto {
        name: Some(file_name.into()),
        package: Some(package.into()),
        syntax: Some("proto3".into()),
        dependency: std::iter::once(CORE_IMPORT.into())
            .chain(
                lower
                    .uses_validation
                    .then(|| "buf/validate/validate.proto".into()),
            )
            .chain(lower.uses_registry.then(|| registry_facets::IMPORT.into()))
            .collect(),
        message_type: messages,
        service: services,
        ..Default::default()
    };
    files.insert(file_name.into(), file);
    let descriptor_set = FileDescriptorSet {
        file: dependency_order(files)?,
        ..Default::default()
    };
    let validators: [(&str, source_metadata::DomainCheck); 1] = [(
        registry_facets::EXTENSION,
        registry_facets::normalize_field_option,
    )];
    let names = source_metadata::native_selection(graph, package, direction);
    let interpretations = names
        .interpretations
        .iter()
        .map(|(name, version)| (name.as_str(), *version))
        .collect::<Vec<_>>();
    let owners = names.owners.iter().map(String::as_str).collect::<Vec<_>>();
    let fields = names.fields.iter().map(String::as_str).collect::<Vec<_>>();
    let methods = names.methods.iter().map(String::as_str).collect::<Vec<_>>();
    let facets = names
        .facets
        .iter()
        .map(|(subject, extension)| (subject.as_str(), extension.as_str()))
        .collect::<Vec<_>>();
    let facet_fields = names
        .facet_fields
        .iter()
        .map(|(subject, extension, field)| (subject.as_str(), extension.as_str(), field.as_str()))
        .collect::<Vec<_>>();
    let selection = NativeSelection {
        owners: &owners,
        fields: &fields,
        methods: &methods,
        facets: &facets,
        facet_fields: &facet_fields,
    };
    let policy = source_metadata::TypedPolicy::new(&validators, &interpretations, selection);
    let selected_extensions = if lower.uses_registry {
        &[registry_facets::EXTENSION][..]
    } else {
        &[]
    };
    let assessment = check_descriptors(&descriptor_set, file_name, &policy, selected_extensions)?;
    if assessment.disposition != pse_contract::metadata::Disposition::Compatible {
        return Err(error(format!(
            "semantic metadata rejected by {}: {:?}",
            assessment.policy, assessment.findings
        )));
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
) -> Result<ProjectionOutput, ProjectionCodegenError> {
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
) -> Result<ProjectionOutput, ProjectionCodegenError> {
    if !config.generate_reflection
        || !config.generate_reflection_vtable
        || !config.shared_descriptor_pool
    {
        return Err(error(
            "owner projections require vtable reflection and one complete shared descriptor pool",
        ));
    }
    let source_descriptors = descriptors_with_imports(graph, package, file_name, imports)?;
    let generation = ProjectionGeneration::new(&source_descriptors, &[file_name.into()], config)?;
    let support = generation.support("descriptor.bin")?;
    Ok(ProjectionOutput {
        generation,
        descriptor_set: support.descriptor_set,
        descriptor_bytes: support.descriptor_bytes,
        source_descriptor_bytes: support.source_descriptor_bytes,
        support_sources: support.sources,
        documentation: owner_reference(graph, package),
    })
}

// The maintained helper performs the same override pass as Rust generation and
// removes only source_code_info. Preserve outer-set unknowns too: its file-slice
// API necessarily constructs a fresh outer FileDescriptorSet.
fn effective_runtime_descriptors(
    source: &FileDescriptorSet,
    config: &CodeGenConfig,
) -> Result<FileDescriptorSet, ProjectionCodegenError> {
    let options = buffa_codegen::tooling_decode_options().map_err(error)?;
    let files: FileDescriptorSet = options
        .decode_from_slice(&buffa_codegen::encode_descriptor_set(
            &source.file,
            &config.feature_overrides,
        ))
        .map_err(|cause| {
            error(buffa_codegen::decode_failure(
                "effective descriptors",
                &cause,
                options.element_memory_limit(),
            ))
        })?;
    let mut runtime = source.clone();
    runtime.file = files.file;
    Ok(runtime)
}

fn source_aware_descriptors(
    source: &FileDescriptorSet,
    runtime: &FileDescriptorSet,
) -> Result<FileDescriptorSet, ProjectionCodegenError> {
    if source.file.len() != runtime.file.len() {
        return Err(error(
            "effective descriptor transformation changed the file population",
        ));
    }
    let mut full = runtime.clone();
    for (file, original) in full.file.iter_mut().zip(&source.file) {
        if file.name != original.name {
            return Err(error(
                "effective descriptor transformation changed a file identity",
            ));
        }
        file.source_code_info = original.source_code_info.clone();
    }
    Ok(full)
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
                format!(
                    "{:?}",
                    field
                        .named_facets
                        .iter()
                        .filter(|(key, _)| key.as_str() != REGISTRY_FIELD_SOURCE)
                        .collect::<BTreeMap<_, _>>()
                )
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
    metadata: &'a OptionEncoder,
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
    ) -> Result<(), ProjectionCodegenError> {
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
            self.metadata
                .field_options(&mut options, field, self.package)?;
            if let Some(facets) = registry_facets::field_options(field, self.graph, self.metadata)?
            {
                self.metadata
                    .attach(&mut options, registry_facets::EXTENSION, facets)?;
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
    ) -> Result<FieldDescriptorProto, ProjectionCodegenError> {
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
                    Scalar::I8 | Scalar::I16 | Scalar::I32 => Type::TYPE_SINT32,
                    Scalar::I64 => Type::TYPE_SINT64,
                    Scalar::F32Bits => Type::TYPE_FIXED32,
                    Scalar::F64Bits => Type::TYPE_FIXED64,
                    Scalar::Bytes | Scalar::SemanticId | Scalar::ContentHash | Scalar::I128Bits => {
                        Type::TYPE_BYTES
                    }
                });
                if matches!(
                    scalar,
                    Scalar::SemanticId
                        | Scalar::ContentHash
                        | Scalar::I128Bits
                        | Scalar::U8
                        | Scalar::U16
                        | Scalar::I8
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
                        if protobuf_map_key(key) {
                            // Synthetic map-entry messages are not emitted as owned
                            // Rust modules. Their collection value wrapper belongs
                            // beside the entry in the actual containing message.
                            self.wrapper(parent, &mut field, tag, value)?;
                        } else {
                            self.wrapper(&mut entry, &mut field, 2, value)?;
                        }
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
    ) -> Result<(), ProjectionCodegenError> {
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
            TypeRef::Scalar(Scalar::I8 | Scalar::I16) => {
                let (minimum, maximum) = if matches!(ty, TypeRef::Scalar(Scalar::I8)) {
                    (i32::from(i8::MIN), i32::from(i8::MAX))
                } else {
                    (i32::from(i16::MIN), i32::from(i16::MAX))
                };
                rules.r#type = Some(
                    SInt32Rules {
                        greater_than: Some(s_int32rules::GreaterThan::Gte(minimum)),
                        less_than: Some(s_int32rules::LessThan::Lte(maximum)),
                        ..Default::default()
                    }
                    .into(),
                );
            }
            TypeRef::Scalar(Scalar::SemanticId | Scalar::ContentHash | Scalar::I128Bits) => {
                let len = if matches!(ty, TypeRef::Scalar(Scalar::SemanticId | Scalar::I128Bits)) {
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
    ) -> Result<(), ProjectionCodegenError> {
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
) -> Result<(), ProjectionCodegenError> {
    let source_type = ty;
    let ty = match (rule, ty) {
        (
            LocalRule::UnsignedRange { .. }
            | LocalRule::SignedRange { .. }
            | LocalRule::Utf8Bytes { .. }
            | LocalRule::Float64Domain(_),
            TypeRef::Optional(inner),
        ) => inner.as_ref(),
        (LocalRule::Length { .. }, TypeRef::Optional(inner))
            if matches!(inner.as_ref(), TypeRef::Scalar(Scalar::String)) =>
        {
            inner.as_ref()
        }
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
            TypeRef::Scalar(Scalar::I8 | Scalar::I16 | Scalar::I32) => {
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
            let previous = match rules.r#type.take() {
                Some(validate::__buffa::oneof::field_rules::Type::String(previous)) => *previous,
                _ => StringRules::default(),
            };
            rules.r#type = Some(
                StringRules {
                    min_bytes: (*min != 0).then_some(*min),
                    max_bytes: Some(*max),
                    ..previous
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
                TypeRef::Scalar(Scalar::String) => {
                    let previous = match rules.r#type.take() {
                        Some(validate::__buffa::oneof::field_rules::Type::String(previous)) => {
                            *previous
                        }
                        _ => StringRules::default(),
                    };
                    StringRules {
                        min_len: (*min != 0).then_some(*min),
                        max_len: Some(*max),
                        ..previous
                    }
                    .into()
                }
                TypeRef::Scalar(
                    Scalar::Bytes | Scalar::SemanticId | Scalar::ContentHash | Scalar::I128Bits,
                ) => BytesRules {
                    min_len: (*min != 0).then_some(*min),
                    max_len: Some(*max),
                    len: match ty {
                        TypeRef::Scalar(Scalar::SemanticId | Scalar::I128Bits) => Some(16),
                        TypeRef::Scalar(Scalar::ContentHash) => Some(32),
                        _ => None,
                    },
                    ..Default::default()
                }
                .into(),
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
                | Scalar::I8
                | Scalar::I16
                | Scalar::I32
                | Scalar::I64
        )
    )
}

fn qualify_nested(
    message: &mut DescriptorProto,
    prefix: &str,
) -> Result<(), ProjectionCodegenError> {
    fn visit(
        message: &mut DescriptorProto,
        prefix: &str,
        ancestors: &mut Vec<(String, std::collections::BTreeSet<String>)>,
    ) -> Result<(), ProjectionCodegenError> {
        let scope = format!("{prefix}.{}", message.name.as_deref().unwrap_or_default());
        ancestors.push((
            scope.clone(),
            message
                .nested_type
                .iter()
                .filter_map(|nested| nested.name.clone())
                .collect(),
        ));
        for field in &mut message.field {
            if let Some(name) = &mut field.type_name
                && !name.starts_with('.')
            {
                let first = name.split('.').next().unwrap_or_default();
                let declaring = ancestors
                    .iter()
                    .rev()
                    .find(|(_, names)| names.contains(first))
                    .ok_or_else(|| {
                        error(format!("{scope}: unresolved relative nested type {name}"))
                    })?;
                *name = format!(".{}.{name}", declaring.0);
            }
        }
        for nested in &mut message.nested_type {
            visit(nested, &scope, ancestors)?;
        }
        ancestors.pop();
        Ok(())
    }
    visit(message, prefix, &mut Vec::new())
}

fn validate_message_names(message: &DescriptorProto) -> Result<(), ProjectionCodegenError> {
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
) -> Result<Vec<FileDescriptorProto>, ProjectionCodegenError> {
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
