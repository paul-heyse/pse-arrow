// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use super::*;
use pse_model::boundary_projection::{Direction, LocalRule, ProjectionType, Service};
use pse_model::{projection_enum, projection_struct};

projection_enum! {
    #[derive(Debug)]
    enum Value {
        projection: { name: "Value", version: 1, direction: Direction::Both, reserved_tags: [8], reserved_names: ["retired"] };
        Missing => 1;
        Number(f64) => 3;
        Rows(Vec<Value>) => 7;
    }
}
projection_struct! {
    #[derive(Debug)]
    struct Request {
        projection: { name: "Request", version: 2, direction: Direction::Input, reserved_tags: [5], reserved_names: ["retired"] };
        limit: u32 => { tag: 2, direction: Direction::Both, rules: [LocalRule::UnsignedRange { min: 1, max: 128 }] };
        id: pse_ids::SemanticId => { tag: 4, direction: Direction::Both, rules: [] };
        value: Option<Vec<Option<Value>>> => { tag: 7, direction: Direction::Input, rules: [] };
        output_only: bool => { tag: 9, direction: Direction::Output, rules: [] };
        files: BTreeMap<String, Vec<u8>> => { tag: 11, direction: Direction::Both, rules: [LocalRule::Length { min: 1, max: 8 }] };
    }
}

// Import schema only: no host operation or semantic owner is projected from this corpus.
fn imports() -> Vec<FileDescriptorProto> {
    FileDescriptorSet::decode_from_slice(include_bytes!(
        "../../../../pse-host/src/generated/code-intelligence/descriptor.bin"
    ))
    .unwrap()
    .file
    .into_iter()
    .filter(|file| {
        file.name
            .as_deref()
            .is_some_and(|name| name.starts_with("google/") || name.starts_with("buf/"))
    })
    .collect()
}

#[test]
fn explicit_tags_bits_identity_rules_and_direction_reach_descriptors() {
    let graph = ProjectionGraph::of::<Request>().unwrap();
    let set = descriptors_for_direction(
        &graph,
        "pse.boundary.v1",
        "pse/boundary/v1/owners.proto",
        &imports(),
        Direction::Input,
    )
    .unwrap();
    let file = set
        .file
        .iter()
        .find(|file| file.package.as_deref() == Some("pse.boundary.v1"))
        .unwrap();
    let request = file
        .message_type
        .iter()
        .find(|message| message.name.as_deref() == Some("Request"))
        .unwrap();
    assert_eq!(request.reserved_range[0].start, Some(5));
    assert_eq!(request.reserved_range[0].end, Some(6));
    assert!(
        !request
            .field
            .iter()
            .any(|field| field.name.as_deref() == Some("output_only"))
    );
    let limit = request
        .field
        .iter()
        .find(|field| field.number == Some(2))
        .unwrap();
    let rules = limit
        .options
        .as_option()
        .unwrap()
        .extension(&validate::FIELD)
        .unwrap();
    let Some(validate::__buffa::oneof::field_rules::Type::Uint32(bounds)) = rules.r#type else {
        panic!("uint32 rules")
    };
    assert!(matches!(
        bounds.greater_than,
        Some(u_int32rules::GreaterThan::Gte(1))
    ));
    assert!(matches!(
        bounds.less_than,
        Some(u_int32rules::LessThan::Lte(128))
    ));
    let id = request
        .field
        .iter()
        .find(|field| field.number == Some(4))
        .unwrap();
    let rules = id
        .options
        .as_option()
        .unwrap()
        .extension(&validate::FIELD)
        .unwrap();
    assert!(
        matches!(rules.r#type, Some(validate::__buffa::oneof::field_rules::Type::Bytes(bytes)) if bytes.len == Some(16))
    );
    let value = file
        .message_type
        .iter()
        .find(|message| message.name.as_deref() == Some("Value"))
        .unwrap();
    assert_eq!(
        value
            .field
            .iter()
            .map(|field| field.number)
            .collect::<Vec<_>>(),
        [Some(1), Some(3), Some(7)]
    );
    assert_eq!(
        value.nested_type[1].field[0].r#type,
        Some(Type::TYPE_FIXED64)
    );
    assert!(
        value.oneof_decl[0]
            .options
            .as_option()
            .unwrap()
            .extension(&validate::ONEOF)
            .unwrap()
            .required
            .unwrap()
    );
    // Independently assert a known bit representation. No floating conversion occurs in proto.
    assert_eq!((-0.0_f64).to_bits(), 0x8000_0000_0000_0000);
}

#[test]
fn nested_presence_maps_and_recursive_names_are_fully_resolved() {
    let graph = ProjectionGraph::of::<Request>().unwrap();
    let set =
        descriptors_with_imports(&graph, "pse.boundary.v1", "owners.proto", &imports()).unwrap();
    let file = set
        .file
        .iter()
        .find(|file| file.name.as_deref() == Some("owners.proto"))
        .unwrap();
    let request = file
        .message_type
        .iter()
        .find(|message| message.name.as_deref() == Some("Request"))
        .unwrap();
    let optional_sequence = request
        .field
        .iter()
        .find(|field| field.number == Some(7))
        .unwrap();
    assert_eq!(optional_sequence.label, Some(Label::LABEL_OPTIONAL));
    assert_eq!(
        optional_sequence.type_name.as_deref(),
        Some(".pse.boundary.v1.Request.Field7Value")
    );
    let wrapper = request
        .nested_type
        .iter()
        .find(|message| message.name.as_deref() == Some("Field7Value"))
        .unwrap();
    assert_eq!(wrapper.field[0].label, Some(Label::LABEL_REPEATED));
    assert_eq!(
        wrapper.nested_type[0].field[0].type_name.as_deref(),
        Some(".pse.boundary.v1.Value")
    );
    let entry = request
        .nested_type
        .iter()
        .find(|message| message.name.as_deref() == Some("Field11Entry"))
        .unwrap();
    assert!(entry.options.as_option().unwrap().map_entry.unwrap());
    assert_eq!(entry.field[1].r#type, Some(Type::TYPE_BYTES));
    assert!(!entry.field[1].proto3_optional.unwrap_or_default());
}

#[test]
fn complete_services_generate_one_shared_root_and_reproducible_rust() {
    let mut graph = ProjectionGraph::default();
    let service = Service::new("EngineeringService")
        .unary::<Request, Request>(&mut graph, "Inspect")
        .unwrap();
    graph.service(service).unwrap();
    let config = default_config("crate::generated::__buffa_fds");
    let first = generate_with_config(
        &graph,
        "pse.boundary.v1",
        "owners.proto",
        &imports(),
        &config,
    )
    .unwrap();
    let second = generate_with_config(
        &graph,
        "pse.boundary.v1",
        "owners.proto",
        &imports(),
        &config,
    )
    .unwrap();
    assert_eq!(first.sources, second.sources);
    assert_eq!(first.descriptor_bytes, second.descriptor_bytes);
    assert!(first.warnings.is_empty(), "{:?}", first.warnings);
    let file = first
        .descriptor_set
        .file
        .iter()
        .find(|file| file.name.as_deref() == Some("owners.proto"))
        .unwrap();
    assert_eq!(
        file.service[0].method[0].input_type.as_deref(),
        Some(".pse.boundary.v1.Request")
    );
    assert!(first.sources["mod.rs"].contains("shared-root.rs"));
    assert!(first.sources["shared-root.rs"].contains("pub mod __buffa_fds"));
    for source in first.sources.values() {
        syn::parse_file(source).expect("generated Rust parses");
    }
    let decoded = FileDescriptorSet::decode_from_slice(&first.descriptor_bytes).unwrap();
    assert!(
        decoded
            .file
            .iter()
            .any(|file| file.name.as_deref() == Some("buf/validate/validate.proto"))
    );
}

#[test]
fn missing_imports_unknown_types_and_unversioned_packages_refuse() {
    let graph = ProjectionGraph::of::<Request>().unwrap();
    assert!(descriptors(&graph, "pse.boundary.v1", "owners.proto").is_err());
    assert!(descriptors_with_imports(&graph, "pse.boundary", "owners.proto", &imports()).is_err());
    struct Unsupported;
    impl pse_model::boundary_projection::ValidateProjection for Unsupported {
        fn validate_at(
            &self,
            _: &str,
        ) -> Result<(), pse_model::boundary_projection::ValidationError> {
            Ok(())
        }
    }
    impl ProjectionType for Unsupported {
        fn projection(
            _: &mut ProjectionGraph,
        ) -> Result<TypeRef, pse_model::boundary_projection::ProjectionError> {
            Ok(TypeRef::Named("Absent".into()))
        }
    }
    projection_struct! {
        struct Bad {
            projection: { name: "Bad", version: 1, direction: Direction::Both, reserved_tags: [], reserved_names: [] };
            value: Unsupported => { tag: 1, direction: Direction::Both, rules: [] };
        }
    }
    assert!(
        descriptors(
            &ProjectionGraph::of::<Bad>().unwrap(),
            "pse.boundary.v1",
            "bad.proto"
        )
        .is_err()
    );
}
