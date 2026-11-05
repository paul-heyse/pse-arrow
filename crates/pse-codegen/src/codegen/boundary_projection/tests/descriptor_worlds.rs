// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use super::*;
use buffa_codegen::{EnumTypeOverride, FeatureOverride};

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
    assert!(output.sources["shared-root.rs"].contains("include_bytes!(\"descriptor.bin\")"));
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
        buffa_codegen::CodeGenWarning::FeatureOverrideMatchedNothing { .. }
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
