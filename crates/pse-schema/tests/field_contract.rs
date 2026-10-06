// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Negative oracles for exact recursive admission, independent of generated rows.
#![allow(clippy::unwrap_used, clippy::panic, reason = "test assertions")]

use std::collections::HashMap;

use arrow_schema::{DataType, Field, Schema};
use pse_schema::field_contract::{declaration, execution_schema};

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
fn native_type_eligibility_is_independent_of_pse_domain_types() {
    let native = Schema::new(vec![Field::new(
        "decimal",
        DataType::Decimal256(60, 8),
        false,
    )]);
    execution_schema(&native, &native).unwrap();
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
