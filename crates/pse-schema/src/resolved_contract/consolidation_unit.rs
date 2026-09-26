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

fn compiled(registry: &Registry) -> &'static [super::ExpectedContract] {
    Box::leak(
        registry
            .relations()
            .iter()
            .map(|spec| super::ExpectedContract::capture(registry, spec).unwrap())
            .collect::<Vec<_>>()
            .into_boxed_slice(),
    )
}

#[test]
fn independent_compact_expectations_are_cached_and_reject_changed_closures() {
    let generator = fixture("first");
    let expected = compiled(&generator);
    let runtime = fixture("first");
    let arena = runtime.contract_arena().unwrap();
    let handle = runtime
        .contract(runtime.relation("authored.contract").unwrap())
        .unwrap();
    for _ in 0..100 {
        handle.require_generated(expected).unwrap();
    }
    assert_eq!(
        arena.comparisons.load(std::sync::atomic::Ordering::Relaxed),
        1
    );
    assert_eq!(arena.proofs.lock().unwrap().len(), 1);
    let changed = fixture("second");
    assert!(handle.require_generated(compiled(&changed)).is_err());
    for field in 0..4 {
        let mut bad = expected.to_vec();
        match field {
            0 => bad[0].version += 1,
            1 => bad[0].semantic_version += 1,
            2 => bad[0].semantics = pse_ids::ContentHash::NIL,
            _ => bad[0].encoding = pse_ids::ContentHash::NIL,
        }
        assert!(
            handle
                .require_generated(Box::leak(bad.into_boxed_slice()))
                .is_err()
        );
    }
    assert_eq!(arena.proofs.lock().unwrap().len(), 1);
}

#[test]
fn runtime_graph_rejects_missing_edges() {
    let registry = fixture("first");
    let arena = registry.contract_arena().unwrap();
    let mut relations = arena.graph.relations.values().cloned().collect::<Vec<_>>();
    relations[0].references.push(SemanticId::NIL);
    assert!(
        super::ResolvedContracts::new(
            relations,
            arena.graph.enums.values().cloned().collect(),
            arena.graph.extensions.values().cloned().collect()
        )
        .is_err()
    );
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
    let expected = compiled(&equivalent);
    handle.require_generated(expected).unwrap();
    reg.contract(reg.relation("authored.right").unwrap())
        .unwrap()
        .require_generated(expected)
        .unwrap();
    assert!(handle.require_generated(compiled(&changed)).is_err());
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
    let expected = compiled(&reg);
    changed
        .admit_contract(&handle)
        .unwrap()
        .require_generated(expected)
        .unwrap();
}
