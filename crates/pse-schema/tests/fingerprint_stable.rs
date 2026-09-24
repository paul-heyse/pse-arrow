// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a test reports by panicking with the offending declaration; the workspace panic policy governs library code"
)]

//! The registry fingerprint is a function of the declarations and of nothing else.
//!
//! Two properties, and both matter. **Stable**: two assemblies of the same declarations
//! agree, so a fingerprint stored in an artifact can be compared to one computed now.
//! **Sensitive**: a changed declaration changes it, so an artifact written under an older
//! contract cannot be read back under a newer one while looking valid.
//!
//! The second is the one that is easy to lose. A digest that covered only the columns'
//! names and types would keep its value when a doc string, an enumeration member or a
//! nullability changed — and every one of those changes what a generated view will accept.

use pse_ids::ContentHash;
use pse_schema::builder::RegistryBuilder;
use pse_schema::model::{
    Authority, EnumDecl, EnumMember, FieldContract, Namespace, RelationDecl, SnapshotClass,
};
use pse_schema::{Registry, catalog, fingerprint, registry};

#[test]
fn two_assemblies_agree() {
    let first = catalog::assemble().expect("the shipped catalog assembles");
    let second = catalog::assemble().expect("the shipped catalog assembles");
    assert_eq!(first.fingerprint(), second.fingerprint());
    assert_ne!(first.fingerprint(), ContentHash::NIL);
}

#[test]
fn the_memoized_registry_agrees_with_a_fresh_assembly() {
    let fresh = catalog::assemble().expect("the shipped catalog assembles");
    let memoized = registry().expect("the shipped catalog assembles");
    assert_eq!(fresh.fingerprint(), memoized.fingerprint());
    for spec in memoized.relations() {
        let same = fresh
            .relation_by_id(spec.id)
            .unwrap_or_else(|| panic!("{} is missing from a fresh assembly", spec.key));
        assert_eq!(spec.fingerprint, same.fingerprint, "{}", spec.key);
    }
}

#[test]
fn the_frame_version_is_the_frozen_one() {
    assert_eq!(fingerprint::FRAME_VERSION, "pse.schema.fingerprint.v2");
}

#[test]
fn two_relations_have_different_fingerprints() {
    let reg = registry().expect("the shipped catalog assembles");
    let mut seen: Vec<ContentHash> = reg
        .relations()
        .iter()
        .map(|spec| spec.fingerprint)
        .collect();
    let before = seen.len();
    seen.sort_unstable_by_key(|hash| *hash.as_bytes());
    seen.dedup();
    assert_eq!(
        before,
        seen.len(),
        "two relations share a contract fingerprint; a view generated for one would accept the \
         other's batches"
    );
}

#[test]
fn a_changed_column_doc_changes_the_fingerprint() {
    let before = scratch("the original prose").fingerprint();
    let after = scratch("prose that says something else").fingerprint();
    assert_ne!(
        before, after,
        "a doc string is part of the declaration a consumer reads"
    );
}

#[test]
fn a_changed_column_doc_changes_that_relation_and_not_the_other() {
    let before = scratch("the original prose");
    let after = scratch("prose that says something else");

    let changed_before = before.relation("authored.example").expect("declared");
    let changed_after = after.relation("authored.example").expect("declared");
    assert_ne!(changed_before.fingerprint, changed_after.fingerprint);

    let untouched_before = before.relation("authored.other").expect("declared");
    let untouched_after = after.relation("authored.other").expect("declared");
    assert_eq!(
        untouched_before.fingerprint, untouched_after.fingerprint,
        "an unrelated relation keeps its contract when another one changes"
    );
}

#[test]
fn a_changed_enum_member_changes_the_relation_that_uses_it() {
    let one = scratch_with_members(vec![EnumMember::new("a", "a member")]);
    let other = scratch_with_members(vec![
        EnumMember::new("a", "a member"),
        EnumMember::new("b", "a second member"),
    ]);

    let using = one.relation("authored.example").expect("declared");
    let using_other = other.relation("authored.example").expect("declared");
    assert_ne!(
        using.fingerprint, using_other.fingerprint,
        "a `pse.enum` column's contract includes which members are admissible"
    );
}

#[test]
fn a_changed_nullability_changes_the_fingerprint() {
    let strict = scratch_registry(false, "the original prose", default_members());
    let lax = scratch_registry(true, "the original prose", default_members());
    assert_ne!(strict.fingerprint(), lax.fingerprint());
}

/// The scratch enumeration's default members.
fn default_members() -> Vec<EnumMember> {
    vec![EnumMember::new("a", "a member")]
}

/// A two-relation scratch registry whose `label` column carries `doc`.
fn scratch(doc: &'static str) -> Registry {
    scratch_registry(false, doc, default_members())
}

/// A scratch registry whose enumeration has `members`.
fn scratch_with_members(members: Vec<EnumMember>) -> Registry {
    scratch_registry(false, "the original prose", members)
}

/// Two relations, one enumeration, and the three knobs the tests above turn.
fn scratch_registry(nullable: bool, doc: &'static str, members: Vec<EnumMember>) -> Registry {
    let mut label = FieldContract::label(
        "label",
        FieldContract::native(arrow_schema::DataType::Utf8),
        doc,
    );
    label = label.with_nullable(nullable);

    let mut builder = RegistryBuilder::new();
    builder
        .declare_enum(EnumDecl::platform("Scratch", members))
        .declare_relation(
            RelationDecl::new(
                Namespace::Authored,
                "example",
                1,
                Authority::Authored,
                SnapshotClass::Model,
                "the relation under test",
            )
            .pk(&["id"])
            .columns(vec![
                FieldContract::key("id", FieldContract::id(), "the identity"),
                label,
                FieldContract::label(
                    "kind",
                    FieldContract::enumeration("Scratch"),
                    "a dictionary",
                ),
            ]),
        )
        .declare_relation(
            RelationDecl::new(
                Namespace::Authored,
                "other",
                1,
                Authority::Authored,
                SnapshotClass::Model,
                "a relation the tests do not touch",
            )
            .pk(&["id"])
            .columns(vec![FieldContract::key(
                "id",
                FieldContract::id(),
                "the identity",
            )]),
        );
    builder.build().expect("the scratch registry assembles")
}
