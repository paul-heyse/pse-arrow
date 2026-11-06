// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use super::*;
use buffa_codegen::{EnumTypeOverride, FeatureOverride};

#[test]
fn authored_support_preserves_per_file_layout_complete_world_and_effective_features() {
    let mut source = enum_source();
    source.file.insert(
        0,
        FileDescriptorProto {
            name: Some("dependency.proto".into()),
            package: Some("dependency.v1".into()),
            syntax: Some("proto3".into()),
            ..Default::default()
        },
    );
    source.file[1].dependency.push("dependency.proto".into());
    source.__buffa_unknown_fields.push(buffa::UnknownField {
        number: 100,
        data: buffa::UnknownFieldData::Varint(9),
    });
    let original = source.encode_to_vec();
    let mut config = default_config("crate::authored::__buffa_fds");
    config.file_per_package = false;
    let generation = ProjectionGeneration::new(&source, &["world.proto".into()], &config).unwrap();
    let support = generation.support("descriptor-runtime.bin").unwrap();
    let mut expected = source.clone();
    for file in &mut expected.file {
        file.source_code_info = Default::default();
    }
    assert_eq!(support.descriptor_set, expected);
    assert_eq!(
        FileDescriptorSet::decode_from_slice(&support.descriptor_bytes).unwrap(),
        expected
    );
    assert_eq!(support.source_descriptor_bytes, original);
    assert!(
        support.sources["shared-root.rs"]
            .split_whitespace()
            .collect::<String>()
            .contains("include_bytes!(\"descriptor-runtime.bin\")")
    );
    assert!(support.sources["mod.rs"].contains("world.mod.rs"));
    assert!(generation.files.iter().any(|file| file.name == "world.rs"));
    assert_eq!(source.encode_to_vec(), original);

    config.feature_overrides.push((
        ".world.Color".into(),
        FeatureOverride::EnumType(EnumTypeOverride::Open),
    ));
    config.feature_overrides.push((
        ".absent.Enum".into(),
        FeatureOverride::EnumType(EnumTypeOverride::Open),
    ));
    let effective = ProjectionGeneration::new(&source, &["world.proto".into()], &config).unwrap();
    let support = effective.support("descriptor-runtime.bin").unwrap();
    assert_eq!(
        support.descriptor_set.__buffa_unknown_fields,
        source.__buffa_unknown_fields
    );
    assert_eq!(
        support
            .descriptor_set
            .file
            .iter()
            .map(|file| &file.name)
            .collect::<Vec<_>>(),
        source
            .file
            .iter()
            .map(|file| &file.name)
            .collect::<Vec<_>>()
    );
    assert_eq!(
        support.descriptor_set.file[1].dependency,
        source.file[1].dependency
    );
    assert_eq!(
        support.descriptor_set.file[1].enum_type[0]
            .options
            .as_option()
            .unwrap()
            .__buffa_unknown_fields,
        source.file[1].enum_type[0]
            .options
            .as_option()
            .unwrap()
            .__buffa_unknown_fields
    );
    let pool = buffa_descriptor::DescriptorPool::new(support.descriptor_set).unwrap();
    assert_eq!(
        pool.enum_by_name("world.Color").unwrap().enum_type(),
        buffa::editions::EnumType::Open
    );
    assert!(effective.warnings.iter().any(|warning| matches!(warning, CodeGenWarning::FeatureOverrideMatchedNothing { rule, .. } if rule == ".absent.Enum")));
    assert_eq!(
        effective.descriptors().file[1].source_code_info,
        source.file[1].source_code_info
    );
}

#[test]
fn admitted_generation_shares_full_effective_source_and_retains_original_override_warning() {
    let mut imported = imports();
    imported[0].source_code_info = descriptor::SourceCodeInfo {
        location: vec![descriptor::source_code_info::Location {
            path: vec![2],
            span: vec![0, 0, 1],
            leading_comments: Some("original imported source".into()),
            ..Default::default()
        }],
        ..Default::default()
    }
    .into();
    let graph = ProjectionGraph::of::<Request>().unwrap();
    let mut config = default_config("crate::wire::__buffa_fds");
    config.feature_overrides.push((
        ".no.such.enum".into(),
        FeatureOverride::EnumType(EnumTypeOverride::Open),
    ));
    let output = generate_with_config(
        &graph,
        "pse.boundary.v1",
        "owners.proto",
        &imported,
        &config,
    )
    .unwrap();
    assert_eq!(
        output.generation.descriptors(),
        &FileDescriptorSet::decode_from_slice(&output.source_descriptor_bytes).unwrap()
    );
    assert!(
        output
            .generation
            .descriptors()
            .file
            .iter()
            .any(|file| file.source_code_info.is_set())
    );
    assert!(output.generation.warnings.iter().any(|warning| matches!(warning, CodeGenWarning::FeatureOverrideMatchedNothing { rule, .. } if rule == ".no.such.enum")));
    assert!(output.generation.files.iter().any(
        |file| file.kind == GeneratedFileKind::PackageMod && file.package == "pse.boundary.v1"
    ));
    let mut mismatch = output.generation.config().clone();
    mismatch.pointer_fields.push((
        ".pse.boundary.v1.Request.value".into(),
        buffa_codegen::PointerRepr::Box,
    ));
    let original =
        descriptors_with_imports(&graph, "pse.boundary.v1", "owners.proto", &imported).unwrap();
    let error =
        ProjectionGeneration::new(&original, &["owners.proto".into()], &mismatch).unwrap_err();
    assert!(matches!(
        error,
        ProjectionCodegenError::Buffa(buffa_codegen::CodeGenError::SharedCorpusContextMismatch {
            what: "pointer_fields"
        })
    ));
}

#[test]
fn admitted_generation_refuses_missing_duplicate_targets_and_duplicate_source_files() {
    let source = descriptors_with_imports(
        &ProjectionGraph::of::<Request>().unwrap(),
        "pse.boundary.v1",
        "owners.proto",
        &imports(),
    )
    .unwrap();
    let config = default_config("crate::wire::__buffa_fds");
    for (targets, reason) in [
        (vec![], "explicit generation targets"),
        (vec!["missing.proto".into()], "target missing"),
        (
            vec!["owners.proto".into(), "owners.proto".into()],
            "duplicate generation target",
        ),
    ] {
        assert!(
            ProjectionGeneration::new(&source, &targets, &config)
                .unwrap_err()
                .to_string()
                .contains(reason)
        );
    }
    let mut duplicate = source;
    duplicate.file.push(duplicate.file[0].clone());
    assert!(
        ProjectionGeneration::new(&duplicate, &["owners.proto".into()], &config)
            .unwrap_err()
            .to_string()
            .contains("duplicate descriptor filename")
    );
}

#[test]
fn admitted_generation_refuses_selected_target_excluded_by_package_policy() {
    let source = descriptors_with_imports(
        &ProjectionGraph::of::<Request>().unwrap(),
        "sample.v1",
        "selected.proto",
        &imports(),
    )
    .unwrap();
    let mut config = default_config("crate::wire::__buffa_fds");
    for excluded in ["sample.v1", " .sample "] {
        config.exclude_packages = vec![excluded.into()];
        // This invalid generator setting would otherwise fail in Buffa. Selected
        // population admission must refuse first, before any supplier is entered.
        config.generate_reflection = false;
        let error =
            ProjectionGeneration::new(&source, &["selected.proto".into()], &config).unwrap_err();
        assert!(
            matches!(error, ProjectionCodegenError::ExcludedTarget { file, package } if file == "selected.proto" && package == "sample.v1")
        );
    }
}

#[test]
fn admitted_generation_keeps_excluded_imports_and_package_component_boundaries() {
    let source = descriptors_with_imports(
        &ProjectionGraph::of::<Request>().unwrap(),
        "sample.v1",
        "selected.proto",
        &imports(),
    )
    .unwrap();
    let mut config = default_config("crate::wire::__buffa_fds");
    config.exclude_packages = vec![" .buf.validate ".into(), "sample.v".into()];
    let generated =
        ProjectionGeneration::new(&source, &["selected.proto".into()], &config).unwrap();
    assert_eq!(generated.targets(), &["selected.proto"]);
    assert_eq!(
        generated.descriptors(),
        &source,
        "excluded imports and their original options remain in the complete corpus"
    );
    assert!(
        generated
            .files
            .iter()
            .any(|file| file.package == "sample.v1" && file.kind == GeneratedFileKind::PackageMod)
    );
    assert!(
        generated
            .descriptors()
            .file
            .iter()
            .any(|file| file.name.as_deref() == Some("buf/validate/validate.proto"))
    );
}

#[test]
fn admitted_generation_context_matches_actual_effective_features_and_original_comments() {
    let source = enum_source();
    let mut config = CodeGenConfig::default();
    config.feature_overrides.push((
        ".world.Color".into(),
        FeatureOverride::EnumType(EnumTypeOverride::Open),
    ));
    let generated = ProjectionGeneration::new(&source, &["world.proto".into()], &config).unwrap();
    assert_eq!(
        generated.descriptors().file[0].source_code_info,
        source.file[0].source_code_info
    );
    let pool = buffa_descriptor::DescriptorPool::new(generated.descriptors().clone()).unwrap();
    assert_eq!(
        pool.enum_by_name("world.Color").unwrap().enum_type(),
        buffa::editions::EnumType::Open
    );
    assert!(
        generated
            .files
            .iter()
            .any(|file| file.content.replace(' ', "").contains("EnumValue<Color>"))
    );
    assert!(
        generated
            .files
            .iter()
            .any(|file| file.content.contains("original Paint documentation"))
    );
}

#[test]
fn typed_message_output_collision_admission_keeps_files_and_package_mounts_distinct() {
    let file = |name: &str, package: &str| GeneratedFile {
        name: name.into(),
        package: package.into(),
        kind: GeneratedFileKind::PackageMod,
        content: String::new(),
    };
    for files in [
        vec![file("same.rs", "first.v1"), file("same.rs", "second.v1")],
        vec![file("first.rs", "same.v1"), file("second.rs", "same.v1")],
        vec![file("shared-root.rs", "reserved.v1")],
        vec![
            file("first.rs", "a.self.left"),
            file("second.rs", "a.self_.right"),
        ],
    ] {
        assert!(check_generated_files(&files).is_err());
    }
}

#[test]
fn descriptor_world_source_information_and_typed_options_have_distinct_artifacts() {
    let graph = ProjectionGraph::of::<Request>().unwrap();
    let mut imports = imports();
    imports[0].source_code_info = descriptor::SourceCodeInfo {
        location: vec![descriptor::source_code_info::Location {
            path: vec![2],
            span: vec![0, 0, 1],
            leading_comments: Some("original source documentation".into()),
            ..Default::default()
        }],
        ..Default::default()
    }
    .into();
    let output = generate_with_config(
        &graph,
        "pse.boundary.v1",
        "owners.proto",
        &imports,
        &default_config("crate::generated::__buffa_fds"),
    )
    .unwrap();
    let original =
        descriptors_with_imports(&graph, "pse.boundary.v1", "owners.proto", &imports).unwrap();
    let source = FileDescriptorSet::decode_from_slice(&output.source_descriptor_bytes).unwrap();
    let runtime = FileDescriptorSet::decode_from_slice(&output.descriptor_bytes).unwrap();
    assert_ne!(output.source_descriptor_bytes, output.descriptor_bytes);
    assert!(
        source
            .file
            .iter()
            .any(|file| file.source_code_info.is_set())
    );
    assert!(
        runtime
            .file
            .iter()
            .all(|file| file.source_code_info.is_unset())
    );
    let mut expected = original;
    for file in &mut expected.file {
        file.source_code_info = Default::default();
    }
    assert_eq!(
        runtime, expected,
        "imports, native metadata, local-rule options and unknowns survive compaction"
    );
    assert!(
        expected_request_options(&runtime),
        "independently specified native and validation options survive"
    );
    let mut lossy = runtime.clone();
    let request = lossy
        .file
        .iter_mut()
        .find(|file| file.name.as_deref() == Some("owners.proto"))
        .unwrap()
        .message_type
        .iter_mut()
        .find(|message| message.name.as_deref() == Some("Request"))
        .unwrap();
    request
        .field
        .iter_mut()
        .find(|field| field.number == Some(2))
        .unwrap()
        .options = Default::default();
    assert!(
        !expected_request_options(&lossy),
        "a lossy option transform cannot satisfy the control"
    );
    assert_eq!(output.descriptor_set, runtime);
    let pool = buffa_descriptor::DescriptorPool::new(runtime).unwrap();
    assert!(pool.message_by_name("pse.boundary.v1.Request").is_some());
    assert!(
        output
            .source("shared-root.rs")
            .unwrap()
            .contains("include_bytes!(\"descriptor.bin\")")
    );
}

fn expected_request_options(world: &FileDescriptorSet) -> bool {
    use buffa_descriptor::Value;
    let request = world
        .file
        .iter()
        .find(|file| file.name.as_deref() == Some("owners.proto"))
        .unwrap()
        .message_type
        .iter()
        .find(|message| message.name.as_deref() == Some("Request"))
        .unwrap();
    let encoder = OptionEncoder::new(&world.file).unwrap();
    let Some(metadata) = encoder
        .decode_extension(
            request.options.as_option().unwrap(),
            "pse.contract.v1.message",
        )
        .unwrap()
    else {
        return false;
    };
    if metadata.field_by_number(1) != Some(&Value::U32(2))
        || metadata.field_by_number(3) != Some(&Value::U32(2))
        || metadata.field_by_number(4) != Some(&Value::EnumNumber(1))
    {
        return false;
    }
    let Some(options) = request
        .field
        .iter()
        .find(|field| field.number == Some(2))
        .unwrap()
        .options
        .as_option()
    else {
        return false;
    };
    let Some(rules) = options.extension(&validate::FIELD) else {
        return false;
    };
    let Some(validate::__buffa::oneof::field_rules::Type::Uint32(rules)) = rules.r#type else {
        return false;
    };
    matches!(rules.greater_than, Some(u_int32rules::GreaterThan::Gte(1)))
        && matches!(rules.less_than, Some(u_int32rules::LessThan::Lte(128)))
}

fn enum_source() -> FileDescriptorSet {
    FileDescriptorSet {
        file: vec![FileDescriptorProto {
            name: Some("world.proto".into()),
            package: Some("world".into()),
            syntax: Some("proto2".into()),
            source_code_info: descriptor::SourceCodeInfo {
                location: vec![descriptor::source_code_info::Location {
                    path: vec![4, 0],
                    span: vec![0, 0, 1],
                    leading_comments: Some("original Paint documentation".into()),
                    ..Default::default()
                }],
                ..Default::default()
            }
            .into(),
            enum_type: vec![descriptor::EnumDescriptorProto {
                name: Some("Color".into()),
                value: vec![descriptor::EnumValueDescriptorProto {
                    name: Some("RED".into()),
                    number: Some(0),
                    ..Default::default()
                }],
                options: descriptor::EnumOptions::decode_from_slice(&[0xc0, 0x3e, 0x05])
                    .unwrap()
                    .into(),
                ..Default::default()
            }],
            message_type: vec![DescriptorProto {
                name: Some("Paint".into()),
                field: vec![FieldDescriptorProto {
                    name: Some("color".into()),
                    number: Some(1),
                    label: Some(Label::LABEL_OPTIONAL),
                    r#type: Some(Type::TYPE_ENUM),
                    type_name: Some(".world.Color".into()),
                    ..Default::default()
                }],
                ..Default::default()
            }],
            ..Default::default()
        }],
        ..Default::default()
    }
}

#[test]
fn descriptor_world_effective_enum_features_match_generation_and_keep_old_world() {
    use buffa::editions::EnumType;
    let source = enum_source();
    let original_bytes = source.encode_to_vec();
    let old = buffa_descriptor::DescriptorPool::new(source.clone()).unwrap();
    assert_eq!(
        old.enum_by_name("world.Color").unwrap().enum_type(),
        EnumType::Closed
    );
    let mut config = default_config("crate::generated::__buffa_fds");
    config.feature_overrides.push((
        ".world.Color".into(),
        FeatureOverride::EnumType(EnumTypeOverride::Open),
    ));
    let runtime = effective_runtime_descriptors(&source, &config).unwrap();
    let full = source_aware_descriptors(&source, &runtime).unwrap();
    assert_eq!(
        full.file[0].source_code_info,
        source.file[0].source_code_info
    );
    let mut compact = full.clone();
    for file in &mut compact.file {
        file.source_code_info = Default::default();
    }
    assert_eq!(compact, runtime);
    let full_pool = buffa_descriptor::DescriptorPool::new(full).unwrap();
    assert_eq!(
        full_pool.enum_by_name("world.Color").unwrap().enum_type(),
        EnumType::Open
    );
    let pool = buffa_descriptor::DescriptorPool::new(runtime).unwrap();
    assert_eq!(
        pool.enum_by_name("world.Color").unwrap().enum_type(),
        EnumType::Open
    );
    assert_eq!(
        pool.message_by_name("world.Paint")
            .unwrap()
            .field_by_name("color")
            .unwrap()
            .enum_type(),
        Some(EnumType::Open)
    );
    let (files, warnings) =
        buffa_codegen::generate_with_diagnostics(&source.file, &["world.proto".into()], &config)
            .unwrap();
    assert!(warnings.is_empty(), "{warnings:?}");
    assert!(
        files
            .iter()
            .any(|file| file.content.replace(' ', "").contains("EnumValue<Color>")),
        "generated field must preserve unknown enum values"
    );
    assert_eq!(source.encode_to_vec(), original_bytes);
    assert_eq!(
        old.enum_by_name("world.Color").unwrap().enum_type(),
        EnumType::Closed
    );
    config.feature_overrides.push((
        ".absent.Enum".into(),
        FeatureOverride::EnumType(EnumTypeOverride::Open),
    ));
    let (_, warnings) =
        buffa_codegen::generate_with_diagnostics(&source.file, &["world.proto".into()], &config)
            .unwrap();
    assert!(warnings.iter().any(|warning| matches!(
        warning,
        CodeGenWarning::FeatureOverrideMatchedNothing { .. }
    )));
}

#[test]
fn descriptor_world_runtime_derivation_preserves_outer_and_file_unknown_fields() {
    let mut source = enum_source();
    let mut file_bytes = source.file[0].encode_to_vec();
    file_bytes.extend_from_slice(&[0xc0, 0x3e, 0x07]); // Unknown field 1000, varint 7.
    source.file[0] = FileDescriptorProto::decode_from_slice(&file_bytes).unwrap();
    let mut source_bytes = source.encode_to_vec();
    source_bytes.extend_from_slice(&[0xa0, 0x06, 0x09]); // Unknown outer field 100, varint 9.
    source = FileDescriptorSet::decode_from_slice(&source_bytes).unwrap();
    let mut config = default_config("crate::generated::__buffa_fds");
    config.feature_overrides.push((
        ".world.Color".into(),
        FeatureOverride::EnumType(EnumTypeOverride::Open),
    ));
    let runtime = effective_runtime_descriptors(&source, &config).unwrap();
    let full = source_aware_descriptors(&source, &runtime).unwrap();
    assert_eq!(
        full.file[0].source_code_info,
        source.file[0].source_code_info
    );
    let mut compact = full;
    for file in &mut compact.file {
        file.source_code_info = Default::default();
    }
    assert_eq!(compact, runtime);
    assert!(
        runtime.file[0]
            .encode_to_vec()
            .ends_with(&[0xc0, 0x3e, 0x07])
    );
    assert!(
        runtime.file[0].enum_type[0]
            .options
            .as_option()
            .unwrap()
            .unknown_fields()
            .iter()
            .any(|field| field.number == 1000
                && matches!(field.data, buffa::UnknownFieldData::Varint(5)))
    );
    assert!(runtime.encode_to_vec().ends_with(&[0xa0, 0x06, 0x09]));
}
