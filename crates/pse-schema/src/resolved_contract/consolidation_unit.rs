// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Exact contract admission and owner-scoped reuse.
#![allow(clippy::unwrap_used, reason = "fixed declaration fixture assertions")]

use crate::{
    Registry, RegistryBuilder,
    model::{
        Authority, ColumnRole, DerivationGranularity, EnumDecl, EnumMember, FieldContract,
        FieldContract as T, Namespace, QuantityContract, RelationDecl, SnapshotClass, Stability,
    },
};
use pse_ids::SemanticId;

fn fixture(member: &'static str) -> Registry {
    fixture_docs(member, "original prose")
}

fn fixture_docs(member: &'static str, doc: &'static str) -> Registry {
    let mut builder = RegistryBuilder::new();
    builder.declare_enum(EnumDecl::platform(
        "Choice",
        vec![EnumMember::new(member, doc)],
    ));
    builder.declare_relation(
        RelationDecl::new(
            Namespace::Authored,
            "contract",
            1,
            Authority::Authored,
            SnapshotClass::Model,
            doc,
        )
        .pk(&["id"])
        .columns(vec![
            FieldContract::key("id", T::id(), "identity"),
            FieldContract::payload(
                "value",
                T::structure(vec![
                    T::fixed_list(T::enumeration("Choice"), 2)
                        .with_name("choices")
                        .with_nullable(true),
                ]),
                "nested declaration",
            ),
            FieldContract::reference("parent", T::id(), "parent")
                .optional()
                .with_fk("authored.contract", "id"),
        ]),
    );
    builder.build().unwrap()
}

#[test]
fn every_relation_and_column_fact_survives_an_unchanged_fingerprint() {
    let reg = fixture("first");
    let original = reg.relation("authored.contract").unwrap();
    reg.contract(original).unwrap();
    let mut alternatives = vec![];
    macro_rules! altered {
        ($value:ident, $change:expr) => {{
            let mut $value = original.clone();
            $change;
            alternatives.push($value);
        }};
    }
    altered!(s, s.id = SemanticId::NIL);
    altered!(s, s.key.name = "another");
    altered!(s, s.key.namespace = Namespace::Normalized);
    altered!(s, s.key.version = 2);
    altered!(s, s.authority = Authority::Derived);
    altered!(s, s.snapshot_class = SnapshotClass::Case);
    altered!(
        s,
        s.derivation_granularity = Some(DerivationGranularity::Row)
    );
    altered!(s, s.stability = Stability::Stable);
    altered!(s, s.primary_key = vec!["id", "parent"]);
    altered!(s, s.columns[1] = s.columns[1].clone().with_name("renamed"));
    altered!(s, s.columns[1] = s.columns[1].clone().with_nullable(true));
    altered!(
        s,
        s.columns[1] = s.columns[1]
            .clone()
            .with_quantity_contract(QuantityContract::Column("temperature"))
    );
    altered!(
        s,
        s.columns[1] = s.columns[1].clone().with_role(ColumnRole::Reference)
    );
    altered!(s, s.columns[2] = s.columns[2].clone().without_fk());
    altered!(
        s,
        s.columns[2] = s.columns[2].clone().with_fk("authored.contract", "parent")
    );
    altered!(
        s,
        s.columns[1] = FieldContract::payload(
            "value",
            T::structure(vec![
                T::fixed_list(T::enumeration("Choice"), 3)
                    .with_name("choices")
                    .with_nullable(true)
            ]),
            "nested declaration"
        )
    );
    altered!(
        s,
        s.columns[1] = FieldContract::payload(
            "value",
            T::structure(vec![
                T::fixed_list(T::enumeration("Choice"), 2)
                    .with_name("choices")
                    .with_nullable(false)
            ]),
            "nested declaration"
        )
    );
    altered!(s, s.columns.swap(1, 2));
    for changed in alternatives {
        assert_eq!(changed.fingerprint, original.fingerprint);
        assert!(reg.contract(&changed).is_err());
    }
}

#[test]
fn equivalent_owners_and_transitive_enum_changes() {
    let first = fixture("first");
    let same = fixture("first");
    let different = fixture("second");
    let handle = first
        .contract(first.relation("authored.contract").unwrap())
        .unwrap();
    assert!(same.admit_contract(&handle).is_ok());
    assert!(different.admit_contract(&handle).is_err());
}

#[test]
fn documentation_is_projected_but_unknown_metadata_is_semantic() {
    let reg = fixture("first");
    let original = reg.relation("authored.contract").unwrap();
    let mut prose = original.clone();
    prose.doc = "new prose";
    prose.columns[1] = prose.columns[1].clone().with_doc("new column prose");
    assert!(reg.contract(&prose).is_ok());
    let field = prose.columns[1].field().clone();
    let mut metadata = field.metadata().clone();
    metadata.insert("new.unknown.meaning".into(), "changed".into());
    prose.columns[1] = FieldContract::from_field(field.with_metadata(metadata));
    assert!(reg.contract(&prose).is_err());
}

#[test]
fn generated_proof_is_cached_and_failed_comparison_publishes_nothing() {
    let reg = fixture("first");
    let arena = reg.contract_arena().unwrap();
    let expected = Box::leak(Box::new(super::GeneratedContracts {
        relations: arena.graph.relations.clone(),
        enums: arena.graph.enums.clone(),
        extensions: arena.graph.extensions.clone(),
    }));
    let handle = reg
        .contract(reg.relation("authored.contract").unwrap())
        .unwrap();
    for _ in 0..100 {
        handle.require_generated(expected).unwrap();
    }
    assert_eq!(
        arena.comparisons.load(std::sync::atomic::Ordering::Relaxed),
        1
    );
    assert_eq!(arena.proofs.lock().unwrap().len(), 1);
    assert_eq!(arena.graph.relations.len(), reg.relations().len());
    // A self-reference terminates. Changing a resolved enum under the same ID fails.
    let mut bad = super::GeneratedContracts {
        relations: expected.relations.clone(),
        enums: expected.enums.clone(),
        extensions: expected.extensions.clone(),
    };
    bad.enums.values_mut().next().unwrap().members[0].name = "forged";
    assert!(handle.require_generated(Box::leak(Box::new(bad))).is_err());
    assert_eq!(arena.proofs.lock().unwrap().len(), 1);
}

#[test]
fn graph_checks_extensions_checks_policies_and_missing_edges() {
    let reg = fixture("first");
    let arena = reg.contract_arena().unwrap();
    let handle = reg
        .contract(reg.relation("authored.contract").unwrap())
        .unwrap();
    for mutation in 0..4 {
        let mut expected = super::GeneratedContracts {
            relations: arena.graph.relations.clone(),
            enums: arena.graph.enums.clone(),
            extensions: arena.graph.extensions.clone(),
        };
        match mutation {
            0 => expected.extensions.values_mut().next().unwrap().version += 1,
            1 => {
                expected
                    .relations
                    .get_mut(&handle.id)
                    .unwrap()
                    .declaration
                    .checks
                    .insert("changed".into(), "false".into());
            }
            2 => {
                expected
                    .relations
                    .get_mut(&handle.id)
                    .unwrap()
                    .declaration
                    .delta_properties
                    .insert("delta.appendOnly".into(), "true".into());
            }
            _ => {
                expected
                    .relations
                    .get_mut(&handle.id)
                    .unwrap()
                    .references
                    .push(SemanticId::NIL);
            }
        }
        assert!(
            handle
                .require_generated(Box::leak(Box::new(expected)))
                .is_err()
        );
    }
    let mut relations = arena.graph.relations.values().cloned().collect::<Vec<_>>();
    relations[0].references.push(SemanticId::NIL);
    assert!(
        super::GeneratedContracts::new(
            relations,
            arena.graph.enums.values().cloned().collect(),
            arena.graph.extensions.values().cloned().collect()
        )
        .is_err()
    );
    assert!(arena.proofs.lock().unwrap().is_empty());
}

#[test]
fn handles_retain_and_release_their_actual_arena() {
    let reg = fixture("first");
    let weak = std::sync::Arc::downgrade(reg.contract_arena().unwrap());
    let handle = reg
        .contract(reg.relation("authored.contract").unwrap())
        .unwrap();
    drop(reg);
    assert!(weak.upgrade().is_some());
    assert_eq!(handle.resolved().declaration.key.name, "contract");
    drop(handle);
    assert!(weak.upgrade().is_none());
}

fn graph_fixture(changed_target: bool, changed_unrelated: bool) -> Registry {
    let mut builder = RegistryBuilder::new();
    for (name, target) in [("left", "authored.right"), ("right", "authored.left")] {
        let payload = if name == "right" && changed_target {
            T::native(arrow_schema::DataType::Int32)
        } else {
            T::native(arrow_schema::DataType::Int64)
        };
        builder.declare_relation(
            RelationDecl::new(
                Namespace::Authored,
                name,
                1,
                Authority::Authored,
                SnapshotClass::Model,
                "graph",
            )
            .pk(&["id"])
            .columns(vec![
                FieldContract::key("id", T::id(), "key"),
                FieldContract::reference("parent", T::id(), "edge")
                    .optional()
                    .with_fk(target, "id"),
                FieldContract::payload("value", payload, "value"),
            ]),
        );
    }
    builder.declare_relation(
        RelationDecl::new(
            Namespace::Authored,
            "isolated",
            1,
            Authority::Authored,
            SnapshotClass::Model,
            "isolated",
        )
        .pk(&["id"])
        .columns(vec![
            FieldContract::key("id", T::id(), "key"),
            FieldContract::payload(
                "value",
                T::native(if changed_unrelated {
                    arrow_schema::DataType::Int32
                } else {
                    arrow_schema::DataType::Int64
                }),
                "value",
            ),
        ]),
    );
    builder.build().unwrap()
}

#[test]
fn mutual_cycles_compare_transitive_targets_without_binding_unrelated_nodes() {
    let reg = graph_fixture(false, false);
    let equivalent = graph_fixture(false, true);
    let changed = graph_fixture(true, false);
    let handle = reg
        .contract(reg.relation("authored.left").unwrap())
        .unwrap();
    assert!(equivalent.admit_contract(&handle).is_ok());
    assert!(changed.admit_contract(&handle).is_err());
    let arena = reg.contract_arena().unwrap();
    let expected = Box::leak(Box::new(super::GeneratedContracts {
        relations: arena.graph.relations.clone(),
        enums: arena.graph.enums.clone(),
        extensions: arena.graph.extensions.clone(),
    }));
    handle.require_generated(expected).unwrap();
    reg.contract(reg.relation("authored.right").unwrap())
        .unwrap()
        .require_generated(expected)
        .unwrap();
    assert_eq!(
        arena.comparisons.load(std::sync::atomic::Ordering::Relaxed),
        1
    );
    assert_eq!(arena.proofs.lock().unwrap()[0].relations.len(), 2);
}

#[test]
fn nested_dictionary_ordering_survives_native_equality_projection() {
    use arrow_schema::{DataType, Field};
    let make = |ordered| {
        Field::new(
            "outer",
            DataType::Struct(
                vec![
                    Field::new(
                        "dict",
                        DataType::Dictionary(Box::new(DataType::Int8), Box::new(DataType::Utf8)),
                        true,
                    )
                    .with_dict_is_ordered(ordered),
                ]
                .into(),
            ),
            true,
        )
    };
    let left = make(false);
    let right = make(true);
    assert_eq!(left, right); // Arrow omits this flag; the domain contract must not.
    assert_ne!(
        FieldContract::from_field(super::semantic_field(&left).unwrap()),
        FieldContract::from_field(super::semantic_field(&right).unwrap())
    );
}

#[test]
fn docs_only_owner_changes_bind_even_when_transport_fingerprints_change() {
    let reg = fixture_docs("first", "old prose");
    let changed = fixture_docs("first", "new prose");
    assert_ne!(reg.fingerprint(), changed.fingerprint());
    let handle = reg
        .contract(reg.relation("authored.contract").unwrap())
        .unwrap();
    assert!(changed.admit_contract(&handle).is_ok());
    let arena = reg.contract_arena().unwrap();
    let expected = Box::leak(Box::new(super::GeneratedContracts {
        relations: arena.graph.relations.clone(),
        enums: arena.graph.enums.clone(),
        extensions: arena.graph.extensions.clone(),
    }));
    changed
        .admit_contract(&handle)
        .unwrap()
        .require_generated(expected)
        .unwrap();
}
