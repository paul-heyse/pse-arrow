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
//! foreign key, enumeration, per-row quantity contract and ordinal reference lands
//! somewhere — and that the self-describing rows are ordered and reproducible, because
//! they are the preimage of the registry fingerprint.

use pse_schema::model::{
    Cell, EXTENSION_TYPES, ExtensionUse, LogicalType, QuantityContract, RelationSpec,
    render_data_type,
};
use pse_schema::{catalog, registry};

/// The six relations blueprint §4.1 declares.
const SCHEMA_RELATIONS: [&str; 6] = [
    "reference.schema_relations",
    "reference.schema_columns",
    "reference.schema_logical_types",
    "reference.schema_enums",
    "reference.schema_invariants",
    "reference.schema_migrations",
];

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
            let Some(fk) = column.fk else {
                continue;
            };
            let target = reg.relation(fk.relation).unwrap_or_else(|| {
                panic!(
                    "{}.{} references unknown {}",
                    spec.key, column.name, fk.relation
                )
            });
            assert!(
                target.column(fk.column).is_some(),
                "{}.{} references unknown column {fk}",
                spec.key,
                column.name
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
            column.logical_type.walk_for_test(&mut reachable);
            for ty in reachable {
                match ty.extension() {
                    Some(ExtensionUse::Enum(name)) => assert!(
                        reg.enum_spec(name).is_some(),
                        "{}.{} uses undeclared enum:{name}",
                        spec.key,
                        column.name
                    ),
                    Some(ExtensionUse::OrdinalRef { target }) => assert!(
                        reg.relation(target).is_some(),
                        "{}.{} references undeclared relation {target}",
                        spec.key,
                        column.name
                    ),
                    _ => {}
                }
            }
        }
    }
}

#[test]
fn every_per_row_quantity_has_its_sibling() {
    let reg = registry().expect("the shipped catalog assembles");
    for spec in reg.relations() {
        for column in &spec.columns {
            if column.quantity != QuantityContract::PerRow {
                continue;
            }
            let sibling = column.per_row_quantity_sibling();
            assert!(
                spec.column(&sibling).is_some(),
                "{}.{} declares a per-row quantity contract with no {sibling} column",
                spec.key,
                column.name
            );
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
                !column.nullable,
                "{}.{name} is a primary key column and nullable; a null required key is invalid \
                 (blueprint §5.3 step 1)",
                spec.key
            );
        }
    }
}

#[test]
fn there_are_eleven_extension_types_with_their_declared_storage() {
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

    let expected = [
        ("pse.semantic_id", "FixedSizeBinary(16)"),
        ("pse.content_hash", "FixedSizeBinary(32)"),
        (
            "pse.dimension_vector",
            "FixedSizeList<item: Struct<num: Int16, den: Int16>, 8>",
        ),
        (
            "pse.quantity_value",
            "Struct<value: Float64, quantity_type_id: FixedSizeBinary(16), unit_id: FixedSizeBinary(16)>",
        ),
        (
            "pse.bound",
            "Struct<kind: Dictionary(Int8, Utf8), value: Float64?>",
        ),
        ("pse.index_tuple", "List<item: FixedSizeBinary(16)>"),
        ("pse.ordinal_ref", "UInt64"),
        (
            "pse.source_span",
            "Struct<document_id: FixedSizeBinary(16), start: UInt32, end: UInt32>",
        ),
        ("pse.enum", "Dictionary(Int32, Utf8)"),
        ("pse.expr_dsl", "Utf8"),
        ("pse.target_path", "Utf8"),
    ];
    for (name, storage) in expected {
        let spec = EXTENSION_TYPES
            .iter()
            .find(|candidate| candidate.name == name)
            .unwrap_or_else(|| panic!("{name} is declared in blueprint §4.4"));
        assert_eq!(
            render_data_type(&spec.storage()).unwrap(),
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
            column.logical_type.walk_for_test(&mut reachable);
            for ty in reachable {
                assert!(
                    reg.logical_type(&ty.name()).is_some(),
                    "{}.{} uses {} which the catalog does not list",
                    spec.key,
                    column.name,
                    ty.name()
                );
            }
        }
    }
}

#[test]
fn schema_rows_are_primary_key_sorted() {
    let reg = registry().expect("the shipped catalog assembles");
    for (key, rows) in reg.schema_rows() {
        let Some(spec) = reg.relation(&key.qualified_name()) else {
            assert!(
                rows.is_empty(),
                "{key} emits rows before the relation describing them is declared"
            );
            continue;
        };
        let positions = pk_positions(spec);
        let keys: Vec<Vec<String>> = rows
            .iter()
            .map(|row| positions.iter().map(|at| order_key(&row[*at])).collect())
            .collect();
        let mut sorted = keys.clone();
        sorted.sort();
        assert_eq!(keys, sorted, "{key} rows are not primary-key sorted");
        let before = sorted.len();
        sorted.dedup();
        assert_eq!(before, sorted.len(), "{key} has a duplicate primary key");
    }
}

#[test]
fn schema_rows_have_the_declared_width() {
    let reg = registry().expect("the shipped catalog assembles");
    for (key, rows) in reg.schema_rows() {
        let Some(spec) = reg.relation(&key.qualified_name()) else {
            assert!(
                rows.is_empty(),
                "{key} emits rows before the relation describing them is declared"
            );
            continue;
        };
        for row in &rows {
            assert_eq!(
                row.len(),
                spec.columns.len(),
                "{key} emits {} cells for {} declared columns",
                row.len(),
                spec.columns.len()
            );
        }
    }
}

#[test]
fn two_assemblies_produce_the_same_rows() {
    let first = catalog::assemble().expect("the shipped catalog assembles");
    let second = catalog::assemble().expect("the shipped catalog assembles");
    assert_eq!(
        first.schema_rows(),
        second.schema_rows(),
        "assembly is a pure function of the declarations"
    );
}

#[test]
fn the_manifest_envelope_is_declared() {
    let reg = registry().expect("the shipped catalog assembles");
    let manifest = reg.manifest().expect("blueprint §20.2 declares it");
    assert_eq!(manifest.version, "pse.manifest.v2");
    assert_eq!(manifest.membership_profile, "pse.snapshot.v2");
    for required in [
        "manifest_version",
        "snapshot_id",
        "membership_profile",
        "schema_registry_fingerprint",
        "relations",
        "semantic_parents",
    ] {
        assert!(
            manifest.field(required).is_some(),
            "the manifest declares no {required}"
        );
    }
}

/// The positions of a relation's primary key columns.
fn pk_positions(spec: &RelationSpec) -> Vec<usize> {
    spec.primary_key
        .iter()
        .map(|name| {
            spec.columns
                .iter()
                .position(|column| column.name == *name)
                .unwrap_or_else(|| panic!("{} has no column {name}", spec.key))
        })
        .collect()
}

/// An order-preserving rendering of a key cell, written independently of the registry's
/// own sort key so that the test checks the order rather than restating it.
fn order_key(cell: &Cell) -> String {
    match cell {
        Cell::Id(value) => value.to_hex(),
        Cell::Hash(value) => value.to_hex(),
        Cell::U64(value) => format!("{value:020}"),
        Cell::Text(value) => value.clone(),
        Cell::Enum(value) => (*value).to_owned(),
        Cell::Null => String::new(),
        other => panic!("{other:?} is not a key cell shape"),
    }
}

/// A local re-derivation of `LogicalType`'s child walk, so a test failure is about the
/// declarations rather than about the helper under test.
trait WalkForTest {
    /// Appends this type and every type reachable from it to `out`.
    fn walk_for_test(&self, out: &mut Vec<LogicalType>);
}

impl WalkForTest for LogicalType {
    fn walk_for_test(&self, out: &mut Vec<LogicalType>) {
        out.push(self.clone());
        match self {
            LogicalType::List(element) | LogicalType::FixedList(element, _) => {
                element.walk_for_test(out);
            }
            LogicalType::Struct(children) => {
                for (_, ty, _) in children {
                    ty.walk_for_test(out);
                }
            }
            _ => {}
        }
    }
}
