// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Equal hash witnesses never establish generated declaration equivalence.
#![allow(clippy::unwrap_used, reason = "fixed declaration fixture assertions")]

use pse_ids::SemanticId;
use pse_schema::{
    Registry, RegistryBuilder,
    compiled_contract::relation,
    model::{
        Authority, ColumnRole, ColumnSpec, DerivationGranularity, EnumDecl, EnumMember, ForeignKey,
        LogicalType as T, Namespace, QuantityContract, RelationDecl, SnapshotClass, Stability,
    },
};

fn fixture(member: &'static str) -> Registry {
    let mut builder = RegistryBuilder::new();
    builder.declare_enum(EnumDecl::platform(
        "Choice",
        vec![EnumMember::new(member, "choice")],
    ));
    builder.declare_relation(
        RelationDecl::new(
            Namespace::Authored,
            "contract",
            1,
            Authority::Authored,
            SnapshotClass::Model,
            "contract fixture",
        )
        .pk(&["id"])
        .columns(vec![
            ColumnSpec::key("id", T::id(), "identity"),
            ColumnSpec::payload(
                "value",
                T::Struct(vec![(
                    "choices",
                    T::fixed_list(T::enumeration("Choice"), 2),
                    true,
                )]),
                "nested declaration",
            ),
            ColumnSpec::reference("parent", T::id(), "parent")
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
    let expected = relation(&reg, original).unwrap().literal_spec();
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
    altered!(s, s.doc = "different relation meaning");
    altered!(s, s.columns[1].name = "renamed");
    altered!(s, s.columns[1].nullable = true);
    altered!(
        s,
        s.columns[1].quantity = QuantityContract::Column("temperature")
    );
    altered!(s, s.columns[1].role = ColumnRole::Reference);
    altered!(s, s.columns[1].doc = "different column meaning");
    altered!(s, s.columns[2].fk = None);
    altered!(
        s,
        s.columns[2].fk = Some(ForeignKey::new("authored.contract", "parent"))
    );
    altered!(
        s,
        s.columns[1].logical_type = T::Struct(vec![(
            "choices",
            T::fixed_list(T::enumeration("Choice"), 3),
            true
        )])
    );
    altered!(
        s,
        s.columns[1].logical_type = T::Struct(vec![(
            "choices",
            T::fixed_list(T::enumeration("Choice"), 2),
            false
        )])
    );
    altered!(s, s.columns.swap(1, 2));
    for changed in alternatives {
        assert_eq!(changed.fingerprint, original.fingerprint);
        assert_ne!(relation(&reg, &changed).unwrap().literal_spec(), expected);
    }
}

#[test]
fn resolved_enum_members_and_extension_descriptors_are_compiled_values() {
    let reg = fixture("first");
    let other = fixture("second");
    let spec = reg.relation("authored.contract").unwrap();
    // Same actual relation declaration and witness; only the referenced enum changes.
    assert_ne!(
        relation(&reg, spec).unwrap(),
        relation(&other, spec).unwrap()
    );
    let literal = relation(&reg, spec).unwrap().literal_spec();
    assert!(literal.contains("pse.enum"));
    assert!(literal.contains("Dictionary"));
    assert!(literal.contains("enum_id"));
    assert!(literal.contains("first"));
    assert!(!literal.contains(&spec.fingerprint.to_hex()));
    assert_eq!(
        pse_schema::model::Cell::from_literal_spec(&literal, &reg).unwrap(),
        relation(&reg, spec).unwrap()
    );
}
