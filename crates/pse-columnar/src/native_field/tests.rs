// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

use super::*;
use std::collections::HashMap;

fn annotated(key: &str, value: &str) -> Field {
    Field::new("value", DataType::Float64, true)
        .with_metadata(HashMap::from([(key.into(), value.into())]))
}

fn named_child<'a>(field: &'a Field, name: &str) -> Option<&'a Field> {
    if field.name() == name {
        return Some(field);
    }
    children(field.data_type()).into_iter().find_map(|child| named_child(child, name))
}

#[test]
fn field_purposes_distinguish_presentation_usage_quantity_and_unknown_meaning() {
    use MetadataPurpose::{ExecutionIdentity, PhysicalObservation, ValueIdentity};
    for (key, execution, value) in [
        ("pse.domain.doc", false, false),
        ("pse.domain.structure", false, false),
        ("pse.domain.role", true, false),
        ("pse.semantic.role", true, false),
        ("pse.domain.fk.relation", true, false),
        ("pse.domain.fk.column", true, false),
        ("pse.semantic.fk", true, false),
        ("pse.semantic.reference", true, false),
        ("pse.domain.quantity", true, true),
        ("pse.semantic.quantity_type", true, true),
        ("pse.domain.transfer_context", true, true),
        ("pse.semantic.transfer_context", true, true),
        ("pse.semantic.identity", true, true),
        ("custom.meaning", true, true),
    ] {
        let source = annotated(key, "meaning");
        assert_eq!(project(&source, PhysicalObservation).unwrap(), source);
        assert_eq!(project(&source, ExecutionIdentity).unwrap().metadata().contains_key(key), execution, "{key}");
        let projected = project(&source, ValueIdentity).unwrap();
        assert_eq!(projected.metadata().contains_key(key), value, "{key}");
        assert_eq!(projected.name(), "item");
        assert!(!projected.is_nullable());
    }
}

#[test]
fn logical_storage_is_root_only_and_cannot_prove_quantity_compatibility() {
    let child = annotated("pse.domain.quantity", "enthalpy:datum-b").with_name("component");
    let source = annotated("pse.domain.quantity", "enthalpy:datum-a")
        .with_data_type(DataType::List(Arc::new(child.clone())));
    let storage = project(&source, MetadataPurpose::LogicalStorageType).unwrap();
    assert!(storage.metadata().is_empty());
    assert_eq!(children(storage.data_type()), vec![&child]);
    let incompatible = annotated("pse.domain.quantity", "enthalpy:datum-c")
        .with_data_type(source.data_type().clone());
    assert_eq!(storage, project(&incompatible, MetadataPurpose::LogicalStorageType).unwrap());
    assert!(!admits_metadata(source.metadata(), incompatible.metadata(), MetadataAdmission::CheckedTarget));
}

#[test]
fn nested_projection_retains_member_names_nullability_and_dictionary_ordering() {
    let child = Field::new_dictionary("meaningful", DataType::Int32, DataType::Utf8, true)
        .with_dict_is_ordered(true)
        .with_metadata(HashMap::from([
            ("pse.domain.doc".into(), "prose".into()),
            ("custom.meaning".into(), "domain".into()),
        ]));
    let containers = [
        DataType::List(Arc::new(child.clone())),
        DataType::LargeList(Arc::new(child.clone())),
        DataType::ListView(Arc::new(child.clone())),
        DataType::LargeListView(Arc::new(child.clone())),
        DataType::FixedSizeList(Arc::new(child.clone()), 2),
        DataType::Struct(vec![child.clone()].into()),
        DataType::Map(Arc::new(Field::new("entries", DataType::Struct(vec![
            Field::new("key", DataType::Int32, false), child.clone(),
        ].into()), false)), true),
        DataType::Union(UnionFields::try_new([3], [child.clone()]).unwrap(), arrow_schema::UnionMode::Dense),
        DataType::Dictionary(Box::new(DataType::Int32), Box::new(DataType::Struct(vec![child.clone()].into()))),
        DataType::RunEndEncoded(Arc::new(Field::new("runs", DataType::Int32, false)), Arc::new(child.clone())),
    ];
    for container in containers {
        let source = Field::new("alias", container, true);
        let projected = project(&source, MetadataPurpose::ValueIdentity).unwrap();
        let child = named_child(&projected, "meaningful").unwrap();
        assert!(child.is_nullable());
        assert_eq!(child.dict_is_ordered(), Some(true));
        assert!(!child.metadata().contains_key("pse.domain.doc"));
        assert_eq!(child.metadata()["custom.meaning"], "domain");
        assert_eq!(project(&source, MetadataPurpose::PhysicalObservation).unwrap().data_type(), source.data_type());
    }
}

#[test]
fn directional_admission_and_established_restoration_are_different_questions() {
    let bare = HashMap::new();
    let facet = |key: &str, value: &str| HashMap::from([(key.to_owned(), value.to_owned())]);
    let target = facet("pse.semantic.enum", "phase");
    assert!(admits_metadata(&bare, &target, MetadataAdmission::CheckedTarget));
    assert!(!admits_metadata(&target, &bare, MetadataAdmission::CheckedTarget));
    assert!(!admits_metadata(&target, &facet("pse.semantic.enum", "species"), MetadataAdmission::CheckedTarget));
    for key in ["pse.domain.quantity", "pse.semantic.quantity_type", "pse.domain.transfer_context", "pse.semantic.transfer_context", "custom.meaning"] {
        let meaning = facet(key, "established");
        assert!(!admits_metadata(&bare, &meaning, MetadataAdmission::CheckedTarget));
        assert!(!admits_metadata(&meaning, &bare, MetadataAdmission::CheckedTarget));
        assert!(admits_metadata(&meaning, &meaning, MetadataAdmission::CheckedTarget));
        assert!(admits_metadata(&bare, &meaning, MetadataAdmission::RestoreEstablished));
        assert!(!admits_metadata(&meaning, &facet(key, "conflict"), MetadataAdmission::RestoreEstablished));
    }
    let first = facet("pse.semantic.role", "key");
    let second = facet("pse.semantic.role", "payload");
    assert!(admits_metadata(&first, &second, MetadataAdmission::CheckedTarget));
    assert!(!admits_metadata(&first, &second, MetadataAdmission::RestoreEstablished));
}
