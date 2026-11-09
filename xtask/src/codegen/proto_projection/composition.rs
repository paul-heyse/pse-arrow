// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Typed split suppliers over one admitted source-aware projection corpus.
use anyhow::{Context, Result, ensure};
use buffa_codegen::{
    GeneratedFile, GeneratedFileKind,
    generated::compiler::{CodeGeneratorRequest, code_generator_response::File},
};
#[cfg(any(
    feature = "product-schemas",
    feature = "host-contracts",
    feature = "recorded-result-contracts",
    feature = "scientific-worker-contracts",
    feature = "reference-contracts"
))]
use pse_codegen::codegen::GeneratedTree;
use pse_codegen::codegen::boundary_projection::ProjectionGeneration;
use std::path::Path;

pub(in crate::codegen) struct Suppliers {
    services: Vec<GeneratedFile>,
    validators: Vec<File>,
}

pub(in crate::codegen) fn generate(
    generation: &ProjectionGeneration,
    rust_module: &str,
) -> Result<Suppliers> {
    let request = CodeGeneratorRequest {
        file_to_generate: generation.targets().to_vec(),
        proto_file: generation.descriptors().file.clone(),
        parameter: Some(format!("proto_module={rust_module}")),
        ..Default::default()
    };
    // One admitted scan and CEL schema; rendering consumes the same preparation.
    let prepared = protoc_gen_protovalidate_buffa::prepare(&request).map_err(|error| {
        let details = format!("validator preparation: {:?}", error.diagnostics());
        anyhow::Error::new(error).context(details)
    })?;
    let mut options = connectrpc_codegen::codegen::Options::default();
    options.buffa = generation.config().clone();
    // Keep the current standalone service stitchers, rather than copying the
    // message supplier's collapsed per-package layout into Connect.
    options.buffa.file_per_package = false;
    if let Some((_, existing)) = options
        .buffa
        .extern_paths
        .iter()
        .find(|(proto, _)| proto == ".")
    {
        ensure!(
            existing == rust_module,
            "Connect catch-all mapping differs from the admitted message module: {existing}"
        );
    } else {
        options
            .buffa
            .extern_paths
            .push((".".into(), rust_module.into()));
    }
    let has_services = generation.descriptors().file.iter().any(|file| {
        file.name
            .as_ref()
            .is_some_and(|name| generation.targets().contains(name))
            && !file.service.is_empty()
    });
    ensure!(
        !has_services || generation.config().generate_views,
        "Connect services require the admitted message generation to include views"
    );
    let services = connectrpc_codegen::codegen::generate_services(
        &generation.descriptors().file,
        generation.targets(),
        &options,
    )
    .context("Connect service generation")?;
    let validators = prepared.render().context("admitted validator rendering")?;
    Ok(Suppliers {
        services,
        validators,
    })
}

impl Suppliers {
    pub(in crate::codegen) fn admit<'a>(
        &self,
        sources: impl IntoIterator<Item = &'a str>,
        destination: &Path,
        reserved: &[&str],
    ) -> Result<()> {
        pse_codegen::codegen::boundary_projection::check_package_mounts(
            self.services
                .iter()
                .filter(|file| file.kind == GeneratedFileKind::PackageMod)
                .map(|file| file.package.as_str()),
        )?;
        let mut paths = sources
            .into_iter()
            .map(|name| destination.join(name))
            .collect::<Vec<_>>();
        paths.extend(reserved.iter().map(|name| destination.join(name)));
        paths.extend(
            self.services
                .iter()
                .map(|file| destination.join("services").join(&file.name)),
        );
        for file in &self.validators {
            ensure!(
                file.insertion_point.is_none(),
                "validator returned unsupported insertion point"
            );
            let name = file
                .name
                .as_deref()
                .context("validator output name absent")?;
            ensure!(
                file.content.is_some(),
                "validator output content absent: {name}"
            );
            paths.push(destination.join("validation").join(name));
        }
        super::admit_paths(paths)
    }

    #[cfg(any(
        feature = "product-schemas",
        feature = "host-contracts",
        feature = "recorded-result-contracts",
        feature = "scientific-worker-contracts",
        feature = "reference-contracts"
    ))]
    pub(in crate::codegen) fn insert(
        self,
        tree: &mut GeneratedTree,
        destination: &Path,
    ) -> Result<()> {
        for file in self.services {
            super::insert(
                tree,
                &destination.join("services"),
                &file.name,
                file.content.as_bytes(),
            )?;
        }
        for file in self.validators {
            super::insert(
                tree,
                &destination.join("validation"),
                file.name
                    .as_deref()
                    .context("validator output name absent")?,
                file.content
                    .as_deref()
                    .context("validator output content absent")?
                    .as_bytes(),
            )?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use buffa::ExtensionSet;
    use buffa_codegen::{
        CodeGenConfig,
        generated::descriptor::{
            DescriptorProto, FieldDescriptorProto, FileDescriptorProto, FileDescriptorSet,
            MessageOptions, MethodDescriptorProto, ServiceDescriptorProto,
            field_descriptor_proto::{Label, Type},
        },
    };
    use protovalidate_buffa_protos::buf::validate::{__buffa::ext::MESSAGE, MessageRules, Rule};

    fn source(expression: &str) -> FileDescriptorSet {
        let mut options = MessageOptions::default();
        options.set_extension(
            &MESSAGE,
            MessageRules {
                cel: vec![Rule {
                    id: Some("positive".into()),
                    expression: Some(expression.into()),
                    message: Some("must be positive".into()),
                    ..Default::default()
                }],
                ..Default::default()
            },
        );
        FileDescriptorSet {
            file: vec![FileDescriptorProto {
                name: Some("selected.proto".into()),
                package: Some("sample.v1".into()),
                syntax: Some("proto3".into()),
                message_type: vec![DescriptorProto {
                    name: Some("Request".into()),
                    options: options.into(),
                    field: vec![FieldDescriptorProto {
                        name: Some("x".into()),
                        number: Some(1),
                        r#type: Some(Type::TYPE_INT32),
                        label: Some(Label::LABEL_OPTIONAL),
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                service: vec![ServiceDescriptorProto {
                    name: Some("Sample".into()),
                    method: vec![MethodDescriptorProto {
                        name: Some("Call".into()),
                        input_type: Some(".sample.v1.Request".into()),
                        output_type: Some(".sample.v1.Request".into()),
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }],
            ..Default::default()
        }
    }

    fn generation(source: &FileDescriptorSet) -> ProjectionGeneration {
        let mut config = CodeGenConfig::default();
        config.file_per_package = true;
        config.generate_json = true;
        ProjectionGeneration::new(source, &["selected.proto".into()], &config).unwrap()
    }

    #[test]
    fn typed_split_composition_preserves_supplier_layout_and_owned_view_validators() {
        let source = source("this.x > 0");
        let generation = generation(&source);
        let actual = generate(&generation, "crate::wire").unwrap();
        assert_eq!(
            actual
                .services
                .iter()
                .map(|file| file.name.as_str())
                .collect::<Vec<_>>(),
            ["selected.__connect.rs", "sample.v1.mod.rs"]
        );
        assert_eq!(actual.services[0].kind, GeneratedFileKind::Companion);
        assert_eq!(actual.services[0].package, "sample.v1");
        assert!(
            actual.services[0]
                .content
                .contains("crate::wire::sample::v1")
        );
        assert_eq!(
            actual
                .validators
                .iter()
                .map(|file| file.name.as_deref().unwrap())
                .collect::<Vec<_>>(),
            ["selected.rs", "sample.v1.mod.rs", "mod.rs"]
        );
        let bodies = actual
            .validators
            .iter()
            .filter_map(|file| file.content.as_deref())
            .collect::<String>();
        assert!(bodies.contains("Validate for Request"));
        assert!(bodies.contains("RequestView<'_>"));
        // Characterize the complete old supplier outputs, with equivalent original
        // request options, rather than matching just one generated symbol.
        let request = CodeGeneratorRequest {
            proto_file: source.file,
            file_to_generate: vec!["selected.proto".into()],
            parameter: Some("extern_path=.=crate::wire".into()),
            ..Default::default()
        };
        let previous = connectrpc_codegen::codegen::generate(&request).unwrap();
        assert_eq!(
            previous
                .file
                .iter()
                .map(|file| (file.name.as_deref(), file.content.as_deref()))
                .collect::<Vec<_>>(),
            actual
                .services
                .iter()
                .map(|file| (Some(file.name.as_str()), Some(file.content.as_str())))
                .collect::<Vec<_>>()
        );
        let mut request = request;
        request.parameter = Some("proto_module=crate::wire".into());
        let previous = protoc_gen_protovalidate_buffa::emit::render_with_options(
            &protoc_gen_protovalidate_buffa::scan::gather(&request).unwrap(),
            &protoc_gen_protovalidate_buffa::emit::Options {
                proto_module: "crate::wire".into(),
            },
        )
        .unwrap();
        assert_eq!(previous, actual.validators);
    }

    #[test]
    fn composed_text_rules_reach_owned_and_view_suppliers_in_both_orders() {
        use pse_codegen::codegen::boundary_projection::descriptors_with_imports;
        use pse_contract::boundary_projection::{
            Direction, Field, LocalRule, ProjectionGraph, Shape,
        };
        struct ComposedText;
        let length = LocalRule::Length { min: 2, max: 3 };
        let bytes = LocalRule::Utf8Bytes { min: 3, max: 5 };
        let mut graph = ProjectionGraph::default();
        graph
            .declare::<ComposedText>(
                "ComposedText",
                file!(),
                1,
                Direction::Both,
                &[],
                &[],
                |graph| {
                    Ok(Shape::Record(vec![
                        Field::of::<String>(
                            graph,
                            "forward",
                            1,
                            Direction::Both,
                            vec![length, bytes],
                        )?,
                        Field::of::<String>(
                            graph,
                            "reverse",
                            2,
                            Direction::Both,
                            vec![bytes, length],
                        )?,
                        Field::of::<Option<String>>(
                            graph,
                            "optional_forward",
                            3,
                            Direction::Both,
                            vec![length, bytes],
                        )?,
                        Field::of::<Option<String>>(
                            graph,
                            "optional_reverse",
                            4,
                            Direction::Both,
                            vec![bytes, length],
                        )?,
                    ]))
                },
            )
            .unwrap();
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let imports = crate::codegen::contract_metadata::imports(root).unwrap();
        let source =
            descriptors_with_imports(&graph, "sample.v1", "selected.proto", &imports.file).unwrap();
        let suppliers = generate(&generation(&source), "crate::wire").unwrap();
        let body = suppliers
            .validators
            .iter()
            .find(|file| file.name.as_deref() == Some("selected.rs"))
            .unwrap()
            .content
            .as_deref()
            .unwrap();
        let parsed = syn::parse_file(body).unwrap();
        let mut owners = Vec::new();
        for item in parsed.items {
            if let syn::Item::Impl(implementation) = item {
                let owner = &implementation.self_ty;
                let owner = quote::quote!(#owner).to_string();
                owners.push(owner);
                let body = quote::quote!(#implementation).to_string();
                for rule in [
                    "string.min_len",
                    "string.max_len",
                    "string.min_bytes",
                    "string.max_bytes",
                ] {
                    assert_eq!(
                        body.matches(&format!("\"{rule}\"")).count(),
                        4,
                        "{rule}: {body}"
                    );
                }
                let compact = body.split_whitespace().collect::<String>();
                assert!(compact.contains(".chars().count()"));
                assert!(compact.contains(".len()"));
            }
        }
        assert_eq!(
            owners.len(),
            2,
            "owned and borrowed view validation: {owners:?}"
        );
        assert!(owners.iter().any(|owner| owner == "ComposedText"));
        assert!(
            owners
                .iter()
                .any(|owner| owner.contains("ComposedTextView"))
        );
    }

    #[test]
    fn typed_composition_retains_structured_invalid_and_unsupported_cel() {
        for expression in ["this.x +", "[this.x, 1.5].size() > 0"] {
            let generation = generation(&source(expression));
            let error = generate(&generation, "crate::wire").err().unwrap();
            let original = error
                .downcast_ref::<protoc_gen_protovalidate_buffa::PreparationError>()
                .unwrap();
            assert!(!original.diagnostics().is_empty());
            assert!(
                original
                    .diagnostics()
                    .iter()
                    .any(|diagnostic| diagnostic.source_file == "selected.proto"
                        && diagnostic.rule == "positive")
            );
        }
    }

    #[test]
    fn typed_composition_uses_longest_prefix_wkt_mapping() {
        let mut source = source("this.x > 0");
        source.file[0]
            .dependency
            .push("google/protobuf/empty.proto".into());
        source.file[0].service[0].method[0].input_type = Some(".google.protobuf.Empty".into());
        source.file.push(FileDescriptorProto {
            name: Some("google/protobuf/empty.proto".into()),
            package: Some("google.protobuf".into()),
            syntax: Some("proto3".into()),
            message_type: vec![DescriptorProto {
                name: Some("Empty".into()),
                ..Default::default()
            }],
            ..Default::default()
        });
        let suppliers = generate(&generation(&source), "crate::wire").unwrap();
        assert!(
            suppliers.services[0]
                .content
                .contains("::buffa_types::google::protobuf::Empty")
        );
        assert!(
            !suppliers.services[0]
                .content
                .contains("crate::wire::google::protobuf::Empty")
        );
    }

    #[test]
    fn supplier_filename_collision_refuses_before_flattening() {
        let mut source = source("this.x > 0");
        source.file[0].name = Some("mod.proto".into());
        let mut config = CodeGenConfig::default();
        config.file_per_package = true;
        let generation =
            ProjectionGeneration::new(&source, &["mod.proto".into()], &config).unwrap();
        let suppliers = generate(&generation, "crate::wire").unwrap();
        // The validator's per-source mod.rs and root mod.rs are both present
        // in its original vector. A map collection would silently lose one.
        assert_eq!(
            suppliers
                .validators
                .iter()
                .filter(|file| file.name.as_deref() == Some("mod.rs"))
                .count(),
            2
        );
        assert!(
            suppliers
                .admit(
                    generation.files.iter().map(|file| file.name.as_str()),
                    Path::new("world"),
                    &[]
                )
                .unwrap_err()
                .to_string()
                .contains("duplicate generated output")
        );
    }

    #[test]
    fn authored_supplier_admission_checks_support_and_reserved_paths_before_insertion() {
        let mut config =
            pse_codegen::codegen::boundary_projection::default_config("crate::wire::__buffa_fds");
        config.file_per_package = false;
        let generation =
            ProjectionGeneration::new(&source("this.x > 0"), &["selected.proto".into()], &config)
                .unwrap();
        let support = generation.support("descriptor-runtime.bin").unwrap();
        let suppliers = generate(&generation, "crate::wire").unwrap();
        let names = || {
            generation
                .files
                .iter()
                .map(|file| file.name.as_str())
                .chain(support.sources.keys().map(String::as_str))
        };
        suppliers
            .admit(
                names(),
                Path::new("world"),
                &["descriptor.bin", "descriptor-runtime.bin", "dispatch.rs"],
            )
            .unwrap();
        for reserved in [
            "selected.rs",
            "mod.rs",
            "services/selected.__connect.rs",
            "validation/selected.rs",
            "descriptor.bin/nested.rs",
        ] {
            let mut paths = vec!["descriptor.bin", reserved];
            if reserved == "descriptor.bin/nested.rs" {
                paths.push("descriptor-runtime.bin");
            }
            assert!(
                suppliers
                    .admit(names(), Path::new("world"), &paths)
                    .is_err(),
                "{reserved}"
            );
        }
        assert!(
            generation
                .files
                .iter()
                .any(|file| file.name == "selected.rs")
        );
        assert!(
            suppliers
                .services
                .iter()
                .any(|file| file.name == "selected.__connect.rs")
        );
    }
}
