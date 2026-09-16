// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Composite references are typed, fingerprinted declarations within one occurrence.
#![allow(clippy::unwrap_used, reason = "contract admission assertions")]

use arrow_schema::DataType;
use pse_schema::{RegistryBuilder, model::*};

fn reference() -> ReferenceContract {
    ReferenceContract {
        relation: "authored.target".into(),
        columns: vec![ReferenceColumn {
            source: vec!["literal.dot".into()],
            target: "id".into(),
        }],
        null_policy: ReferenceNullPolicy::Required,
    }
}

fn local() -> FieldContract {
    FieldContract::structure(vec![
        FieldContract::native(DataType::Int64)
            .with_name("literal.dot")
            .optional(),
    ])
    .with_name("reference")
}

fn relation(name: &'static str, columns: Vec<FieldContract>) -> RelationDecl {
    RelationDecl::new(
        Namespace::Authored,
        name,
        1,
        Authority::Authored,
        SnapshotClass::Model,
        "reference fixture",
    )
    .pk(&[])
    .columns(columns)
}

fn build(
    mapping: &ReferenceContract,
    target: FieldContract,
) -> Result<pse_schema::Registry, pse_schema::SchemaError> {
    let mut builder = RegistryBuilder::new();
    builder.declare_relation(relation("source", vec![local().with_reference(mapping)?]));
    builder.declare_relation(relation("target", vec![target.with_name("id")]));
    builder.build()
}

#[test]
fn reference_target_types_and_policies_belong_to_exact_contract_identity() {
    let mapping = reference();
    let required = build(&mapping, FieldContract::native(DataType::Int64)).unwrap();
    let optional = build(
        &ReferenceContract {
            null_policy: ReferenceNullPolicy::AllOrNone,
            ..mapping.clone()
        },
        FieldContract::native(DataType::Int64),
    )
    .unwrap();
    assert_ne!(
        required.relation("authored.source").unwrap().fingerprint,
        optional.relation("authored.source").unwrap().fingerprint
    );
    assert!(build(&mapping, FieldContract::native(DataType::Utf8)).is_err());
    assert!(
        build(
            &ReferenceContract {
                relation: "authored.absent".into(),
                ..mapping.clone()
            },
            FieldContract::native(DataType::Int64)
        )
        .is_err()
    );
    let mut missing = mapping;
    missing.columns[0].target = "absent".into();
    assert!(build(&missing, FieldContract::native(DataType::Int64)).is_err());
}

#[test]
fn reference_paths_cannot_split_a_collection_or_repeat_components() {
    let mut duplicate = reference();
    duplicate.columns.push(duplicate.columns[0].clone());
    assert!(local().with_reference(&duplicate).is_err());
    let mut empty = reference();
    empty.columns.clear();
    assert!(local().with_reference(&empty).is_err());
    let mut parsed_dot = reference();
    parsed_dot.columns[0].source = vec!["literal".into(), "dot".into()];
    assert!(local().with_reference(&parsed_dot).is_err());
    let collection = FieldContract::structure(vec![
        FieldContract::list(FieldContract::structure(vec![
            FieldContract::native(DataType::Int64).with_name("id"),
        ]))
        .with_name("items"),
    ]);
    let mut crossing = reference();
    crossing.columns[0].source = vec!["items".into(), "id".into()];
    assert!(collection.with_reference(&crossing).is_err());
    assert!(
        FieldContract::native(DataType::Float64)
            .with_reference(&ReferenceContract {
                columns: vec![ReferenceColumn {
                    source: vec![],
                    target: "id".into()
                }],
                ..reference()
            })
            .is_err()
    );
}
