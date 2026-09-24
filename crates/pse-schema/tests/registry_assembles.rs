// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a test reports by panicking with the offending declaration; the workspace panic policy governs library code"
)]

//! The shipped registry assembles, and everything it says resolves.
//!
//! Blueprint §4.1: a relation is declared exactly once and the registry is stored as
//! relations. This test is the check that the declarations are internally closed — every
//! foreign key, enumeration, quantity value and ordinal reference lands
//! somewhere — and that the self-describing rows are ordered and reproducible, because
//! they are the preimage of the registry fingerprint.

use pse_schema::model::{
    EXTENSION_TYPES, ExtensionUse, FieldContract, RelationSpec, render_data_type,
};
use pse_schema::{catalog, registry};

/// The schema reflection relations, including declaration/member normalization.
const SCHEMA_RELATIONS: [&str; 7] = [
    "reference.schema_relations",
    "reference.schema_columns",
    "reference.schema_logical_types",
    "reference.schema_enums",
    "reference.schema_enum_types",
    "reference.schema_invariants",
    "reference.schema_migrations",
];

#[test]
fn enumeration_types_supply_unique_targets_for_member_and_law_references() {
    let registry = registry().expect("registry");
    let spec = registry
        .relation("reference.schema_enum_types")
        .expect("enum types");
    let (_, rows) = registry
        .schema_batches()
        .iter()
        .find(|(key, _)| *key == spec.key)
        .expect("type rows");
    let array = rows
        .column(0)
        .as_any()
        .downcast_ref::<arrow_array::FixedSizeBinaryArray>()
        .unwrap();
    let identities = array
        .iter()
        .map(|value| pse_ids::SemanticId::from_bytes(value.unwrap().try_into().unwrap()))
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(rows.num_rows(), registry.enums().len());
    assert_eq!(identities.len(), rows.num_rows());
    for enumeration in registry.enums() {
        assert!(identities.contains(&enumeration.id));
    }
    for (relation, field) in [
        ("reference.schema_enums", "enum_id"),
        ("authored.template_law_contracts", "balance_enum_id"),
        ("reference.law_bindings", "balance_enum_id"),
    ] {
        let relation = registry.relation(relation).expect("declared enum consumer");
        assert_eq!(
            relation
                .column(field)
                .expect("enum reference")
                .fk()
                .expect("foreign key")
                .relation,
            "reference.schema_enum_types"
        );
    }
}

#[test]
fn the_registry_assembles() {
    let reg = registry().expect("the shipped catalog assembles");
    assert!(
        !reg.relations().is_empty(),
        "an empty registry would make every other assertion here vacuous"
    );
    assert_ne!(
        reg.fingerprint(),
        pse_ids::ContentHash::NIL,
        "the fingerprint is filled at assembly"
    );
    assert_eq!(reg.package_id(), *pse_schema::REGISTRY_PACKAGE_ID);
}

#[test]
fn the_six_schema_relations_are_declared() {
    let reg = registry().expect("the shipped catalog assembles");
    for name in SCHEMA_RELATIONS {
        let spec = reg
            .relation(name)
            .unwrap_or_else(|| panic!("{name} is declared in blueprint §4.1"));
        assert_eq!(spec.key.version, 1, "{name} is declared at version 1");
        assert!(
            !spec.primary_key.is_empty(),
            "{name} declares a primary key"
        );
    }
}

#[test]
fn every_foreign_key_resolves() {
    let reg = registry().expect("the shipped catalog assembles");
    for spec in reg.relations() {
        for column in &spec.columns {
            let Some(fk) = column.fk() else {
                continue;
            };
            let target = reg.relation(fk.relation).unwrap_or_else(|| {
                panic!(
                    "{}.{} references unknown {}",
                    spec.key,
                    column.name(),
                    fk.relation
                )
            });
            assert!(
                target.column(fk.column).is_some(),
                "{}.{} references unknown column {fk}",
                spec.key,
                column.name()
            );
        }
    }
}

#[test]
fn every_enum_and_ordinal_reference_resolves() {
    let reg = registry().expect("the shipped catalog assembles");
    for spec in reg.relations() {
        for column in &spec.columns {
            let mut reachable = Vec::new();
            column.value_type().walk_for_test(&mut reachable);
            for ty in reachable {
                match ty.extension() {
                    Some(ExtensionUse::Enum(name)) => assert!(
                        reg.enum_spec(name).is_some(),
                        "{}.{} uses undeclared enum:{name}",
                        spec.key,
                        column.name()
                    ),
                    Some(ExtensionUse::OrdinalRef { target }) => assert!(
                        reg.relation(target).is_some(),
                        "{}.{} references undeclared relation {target}",
                        spec.key,
                        column.name()
                    ),
                    _ => {}
                }
            }
        }
    }
}

#[test]
fn every_primary_key_column_exists_and_is_not_nullable() {
    let reg = registry().expect("the shipped catalog assembles");
    for spec in reg.relations() {
        for name in &spec.primary_key {
            let column = spec
                .column(name)
                .unwrap_or_else(|| panic!("{} has no column {name}", spec.key));
            assert!(
                !column.nullable(),
                "{}.{name} is a primary key column and nullable; a null required key is invalid \
                 (blueprint §5.3 step 1)",
                spec.key
            );
        }
    }
}

#[test]
fn there_are_eleven_extension_types_with_their_declared_storage() {
    use arrow_schema::{DataType as D, Field as F};
    assert_eq!(EXTENSION_TYPES.len(), 11, "blueprint §4.4 declares eleven");

    let mut names: Vec<&str> = EXTENSION_TYPES.iter().map(|spec| spec.name).collect();
    let before = names.len();
    names.sort_unstable();
    names.dedup();
    assert_eq!(before, names.len(), "two extension types share a name");
    for name in &names {
        assert!(
            name.starts_with("pse."),
            "{name} is not in the platform's extension namespace"
        );
    }

    let id = D::FixedSizeBinary(16);
    let expected = [
        ("pse.semantic_id", id.clone()),
        ("pse.content_hash", D::FixedSizeBinary(32)),
        (
            "pse.dimension_vector",
            D::FixedSizeList(
                F::new(
                    "item",
                    D::Struct(
                        vec![
                            F::new("num", D::Int16, false),
                            F::new("den", D::Int16, false),
                        ]
                        .into(),
                    ),
                    false,
                )
                .into(),
                8,
            ),
        ),
        (
            "pse.quantity_value",
            D::Struct(
                vec![
                    F::new("value", D::Float64, false),
                    F::new("quantity_type_id", id.clone(), false),
                    F::new("unit_id", id.clone(), false),
                ]
                .into(),
            ),
        ),
        (
            "pse.bound",
            D::Struct(
                vec![
                    {
                        let mut kind = pse_schema::arrow::field_for(
                            registry().unwrap(),
                            &FieldContract::enumeration("BoundKind").with_name("kind"),
                        )
                        .unwrap();
                        kind.metadata_mut().remove(pse_schema::arrow::KEY_ROLE);
                        kind
                    },
                    F::new("value", D::Float64, true),
                ]
                .into(),
            ),
        ),
        (
            "pse.index_tuple",
            D::List(F::new("item", id.clone(), false).into()),
        ),
        ("pse.ordinal_ref", D::Int64),
        (
            "pse.source_span",
            D::Struct(
                vec![
                    F::new("document_id", id, false),
                    pse_schema::model::IntegerRange::SOURCE_OFFSET.field("start"),
                    pse_schema::model::IntegerRange::SOURCE_OFFSET.field("end"),
                ]
                .into(),
            ),
        ),
        ("pse.enum", D::Utf8),
        ("pse.expr_dsl", D::Utf8),
        ("pse.target_path", D::Utf8),
    ];
    for (name, storage) in expected {
        let spec = EXTENSION_TYPES
            .iter()
            .find(|candidate| candidate.name == name)
            .unwrap_or_else(|| panic!("{name} is declared in blueprint §4.4"));
        assert_eq!(
            serde_json::from_str::<D>(&render_data_type(&spec.storage()).unwrap()).unwrap(),
            storage,
            "{name} storage"
        );
        assert_eq!(spec.metadata_version, 1, "{name} metadata generation");
    }
}

#[test]
fn the_logical_type_catalog_covers_every_declared_column() {
    let reg = registry().expect("the shipped catalog assembles");
    for spec in reg.relations() {
        for column in &spec.columns {
            let mut reachable = Vec::new();
            column.value_type().walk_for_test(&mut reachable);
            for ty in reachable {
                assert!(
                    reg.logical_type(&ty.type_name().unwrap()).is_some(),
                    "{}.{} uses {} which the catalog does not list",
                    spec.key,
                    column.name(),
                    ty.type_name().unwrap()
                );
            }
        }
    }
}

#[test]
fn schema_batches_are_primary_key_sorted() {
    let reg = registry().expect("registry");
    for (key, batch) in reg.schema_batches() {
        let spec = reg.relation(&key.qualified_name()).unwrap();
        let positions = pk_positions(spec);
        let keys = (0..batch.num_rows())
            .map(|row| {
                positions
                    .iter()
                    .map(|at| {
                        let array = batch.column(*at);
                        match array.data_type() {
                            arrow_schema::DataType::FixedSizeBinary(_) => array
                                .as_any()
                                .downcast_ref::<arrow_array::FixedSizeBinaryArray>()
                                .unwrap()
                                .value(row)
                                .to_vec(),
                            arrow_schema::DataType::Utf8 => array
                                .as_any()
                                .downcast_ref::<arrow_array::StringArray>()
                                .unwrap()
                                .value(row)
                                .as_bytes()
                                .to_vec(),
                            arrow_schema::DataType::Int64 => (array
                                .as_any()
                                .downcast_ref::<arrow_array::Int64Array>()
                                .unwrap()
                                .value(row)
                                ^ i64::MIN)
                                .to_be_bytes()
                                .to_vec(),
                            other => panic!("unexpected intrinsic key {other}"),
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        assert!(
            keys.windows(2).all(|pair| pair[0] < pair[1]),
            "{key}: keys must be strictly increasing"
        );
        assert_eq!(batch.num_columns(), spec.columns.len());
        assert_eq!(
            batch.schema().as_ref(),
            &pse_schema::arrow::relation_schema(reg, spec).unwrap()
        );
    }
}

#[test]
fn two_assemblies_produce_the_same_rows() {
    let first = catalog::assemble().expect("the shipped catalog assembles");
    let second = catalog::assemble().expect("the shipped catalog assembles");
    assert_eq!(
        first.schema_batches(),
        second.schema_batches(),
        "assembly is a pure function of the declarations"
    );
}

/// The positions of a relation's primary key columns.
fn pk_positions(spec: &RelationSpec) -> Vec<usize> {
    spec.primary_key
        .iter()
        .map(|name| {
            spec.columns
                .iter()
                .position(|column| column.name() == *name)
                .unwrap_or_else(|| panic!("{} has no column {name}", spec.key))
        })
        .collect()
}

/// A local re-derivation of `FieldContract`'s child walk, so a test failure is about the
/// declarations rather than about the helper under test.
trait WalkForTest {
    /// Appends this type and every type reachable from it to `out`.
    fn walk_for_test(&self, out: &mut Vec<FieldContract>);
}

impl WalkForTest for FieldContract {
    fn walk_for_test(&self, out: &mut Vec<FieldContract>) {
        out.push(self.clone());
        if self.extension().is_some() {
            return;
        }
        match self.data_type() {
            arrow_schema::DataType::List(field)
            | arrow_schema::DataType::FixedSizeList(field, _) => {
                FieldContract::from_field((*field).clone()).walk_for_test(out);
            }
            arrow_schema::DataType::Struct(children) => {
                for field in &children {
                    FieldContract::from_field((**field).clone()).walk_for_test(out);
                }
            }
            _ => {}
        }
    }
}
