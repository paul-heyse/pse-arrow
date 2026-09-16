// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Negative oracles for exact recursive admission, independent of generated rows.
#![allow(clippy::unwrap_used, clippy::panic, reason = "test assertions")]

use std::collections::HashMap;

use arrow_schema::{DataType, Field, Schema};
use pse_schema::field_contract::{declaration, delta_scan_schema, execution_schema};

fn nested(child: Field) -> Schema {
    Schema::new(vec![Field::new(
        "value",
        DataType::List(Field::new("item", DataType::Struct(vec![child].into()), false).into()),
        true,
    )])
}

#[test]
fn exact_admission_refuses_nested_renames_nullability_and_contract_changes() {
    let child = Field::new("source", DataType::Int64, false).with_metadata(HashMap::from([(
        "pse.semantic.role".into(),
        "reference".into(),
    )]));
    let expected = nested(child.clone());
    execution_schema(&expected, &expected).unwrap();
    for changed in [
        child.clone().with_name("destination"),
        child.clone().with_nullable(true),
        child.with_metadata(HashMap::new()),
    ] {
        assert!(execution_schema(&nested(changed), &expected).is_err());
    }
}

#[test]
fn duplicate_paths_and_missing_extra_or_reordered_children_are_rejected() {
    let a = Field::new("a", DataType::Int64, false);
    let b = Field::new("b", DataType::Int64, false);
    let shape = |fields| Schema::new(vec![Field::new("record", DataType::Struct(fields), false)]);
    let expected = shape(vec![a.clone(), b.clone()].into());
    for fields in [
        vec![a.clone(), a.clone()],
        vec![a.clone()],
        vec![
            a.clone(),
            b.clone(),
            Field::new("c", DataType::Int64, false),
        ],
        vec![b, a],
    ] {
        assert!(execution_schema(&shape(fields.into()), &expected).is_err());
    }
    assert!(
        declaration(&Schema::new(vec![
            Field::new("x", DataType::Boolean, true);
            2
        ]))
        .is_err()
    );
}

#[test]
fn scan_adaptation_allows_only_recursive_native_views() {
    let expected = nested(Field::new("label", DataType::Utf8, false));
    let actual = nested(Field::new("label", DataType::Utf8View, false));
    delta_scan_schema(&actual, &expected).unwrap();
    assert!(execution_schema(&actual, &expected).is_err());
    for changed in [
        Field::new("renamed", DataType::Utf8View, false),
        Field::new("label", DataType::Utf8View, true),
        Field::new("label", DataType::LargeUtf8, false),
    ] {
        assert!(delta_scan_schema(&nested(changed), &expected).is_err());
    }
    assert!(
        delta_scan_schema(
            &Schema::new(vec![Field::new("number", DataType::UInt32, false)]),
            &Schema::new(vec![Field::new("number", DataType::Int64, false)]),
        )
        .is_err()
    );
}

#[test]
fn native_type_eligibility_is_independent_of_pse_domain_types() {
    let native = Schema::new(vec![Field::new(
        "decimal",
        DataType::Decimal256(60, 8),
        false,
    )]);
    execution_schema(&native, &native).unwrap();
    assert!(pse_schema::delta::storage_schema(&native).is_err());
}

#[test]
fn exact_dictionary_ordering_is_checked_through_all_native_containers() {
    use arrow_schema::{UnionFields, UnionMode};
    let value = Field::new_dictionary("value", DataType::Int32, DataType::Utf8, false);
    let ordered = value.clone().with_dict_is_ordered(true);
    let wrap = |field: Field| {
        [
            field.clone(),
            Field::new(
                "union",
                DataType::Union(
                    UnionFields::try_new([7], [field.clone()]).unwrap(),
                    UnionMode::Dense,
                ),
                false,
            ),
            Field::new(
                "encoded",
                DataType::RunEndEncoded(
                    Field::new("run_ends", DataType::Int32, false).into(),
                    field.clone().into(),
                ),
                false,
            ),
            Field::new_dictionary(
                "dictionary",
                DataType::Int32,
                DataType::Struct(vec![field].into()),
                false,
            ),
        ]
    };
    for (actual, expected) in wrap(ordered).into_iter().zip(wrap(value)) {
        assert!(
            execution_schema(&Schema::new(vec![actual]), &Schema::new(vec![expected])).is_err()
        );
    }
}

#[test]
fn durable_descriptors_preserve_meaning_without_false_extension_storage() {
    use pse_schema::{
        arrow::{KEY_EXTENSION_METADATA, KEY_EXTENSION_NAME},
        delta::{KEY_EXECUTION_FIELD, KEY_LAYOUT_VERSION, LAYOUT_VERSION},
    };
    let extension = |name: &str, storage| {
        Field::new("value", storage, false).with_metadata(HashMap::from([
            (KEY_EXTENSION_NAME.into(), name.into()),
            (KEY_EXTENSION_METADATA.into(), "{\"v\":1}".into()),
        ]))
    };
    let execution = extension("pse.semantic_id", DataType::FixedSizeBinary(16));
    let storage = pse_schema::delta::storage_schema(&Schema::new(vec![execution.clone()])).unwrap();
    let stored = storage.field(0);
    assert_eq!(stored.data_type(), &DataType::Binary);
    assert!(!stored.metadata().contains_key(KEY_EXTENSION_NAME));
    assert!(!stored.metadata().contains_key(KEY_EXTENSION_METADATA));
    assert_eq!(stored.metadata()[KEY_LAYOUT_VERSION], LAYOUT_VERSION);
    assert_eq!(
        serde_json::from_str::<Field>(&stored.metadata()[KEY_EXECUTION_FIELD]).unwrap(),
        execution
    );
    let same = extension("pse.expr_dsl", DataType::Utf8);
    let storage = pse_schema::delta::storage_schema(&Schema::new(vec![same])).unwrap();
    assert_eq!(
        storage.field(0).metadata()[KEY_EXTENSION_NAME],
        "pse.expr_dsl"
    );
    let invalid = extension("pse.semantic_id", DataType::Binary);
    assert!(pse_schema::delta::storage_schema(&Schema::new(vec![invalid])).is_err());
}

#[test]
fn nested_descriptors_keep_list_child_contracts_and_are_deterministic() {
    use pse_schema::delta::{KEY_EXECUTION_FIELD, storage_schema};
    let field = Field::new("entry", DataType::Utf8, false).with_metadata(HashMap::from([(
        "domain.description".into(),
        "the first child".into(),
    )]));
    let execution = Schema::new(vec![Field::new(
        "children",
        DataType::List(field.into()),
        false,
    )]);
    let stored = storage_schema(&execution).unwrap();
    for _ in 0..20 {
        assert_eq!(storage_schema(&execution).unwrap(), stored);
    }
    let DataType::List(child) = stored.field(0).data_type() else {
        panic!("declared list shape")
    };
    assert_eq!(child.name(), "element");
    assert!(child.metadata().is_empty());
    let restored: Field =
        serde_json::from_str(&stored.field(0).metadata()[KEY_EXECUTION_FIELD]).unwrap();
    assert_eq!(&restored, execution.field(0));
    // A storage descriptor must not be reinterpreted as a new execution declaration.
    assert!(storage_schema(&stored).is_err());
}

#[test]
fn cold_schema_reconstruction_keeps_metadata_scopes_and_rejects_forged_descriptors() {
    use pse_schema::delta::{
        KEY_LAYOUT_VERSION, KEY_SCHEMA_METADATA, execution_schema, storage_schema,
    };
    let field = Field::new("id", DataType::FixedSizeBinary(16), false)
        .with_metadata(HashMap::from([("description".into(), "field".into())]));
    let execution = Schema::new_with_metadata(
        vec![field],
        HashMap::from([("description".into(), "schema".into())]),
    );
    let storage = storage_schema(&execution).unwrap();
    assert_eq!(execution_schema(&storage).unwrap(), execution);
    assert_eq!(storage.field(0).metadata()["description"], "field");
    for key in [KEY_LAYOUT_VERSION, KEY_SCHEMA_METADATA] {
        let mut field = storage.field(0).clone();
        field.metadata_mut().insert(key.into(), "invalid".into());
        assert!(execution_schema(&Schema::new(vec![field])).is_err());
    }
    let forged = Schema::new(vec![storage.field(0).clone().with_name("other")]);
    assert!(execution_schema(&forged).is_err());
    let mut reserved = execution.metadata().clone();
    reserved.insert(KEY_LAYOUT_VERSION.into(), "99".into());
    assert!(
        storage_schema(&Schema::new_with_metadata(
            execution.fields().clone(),
            reserved
        ))
        .is_err()
    );
}
