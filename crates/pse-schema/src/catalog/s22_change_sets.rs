// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! The change-set model (blueprint §22.2, revision 6).
//!
//! There is no other write path into `authored` (decision D2, doctrine P20). A change set
//! is validated by P2 on the resulting snapshot, applied atomically, and yields a new model
//! or case revision.
//!
//! `row` points into the change set's per-relation **staged members**, never at JSON text:
//! a staged row is a typed relation row like every other, so D1 holds inside the change
//! log too. `rename` is rejected for a named-policy entity, because under that policy a
//! rename is by definition a new entity (§5.1).
//!
//! Both relations are sidecars: a change log describes how a snapshot came to be and is
//! excluded from the membership it produced (§5.3 step 7).

use crate::builder::RegistryBuilder;
use crate::model::{
    Authority, ColumnSpec, ExtensionUse, LogicalType, Namespace, RelationDecl, SnapshotClass,
    Stability,
};

/// Declares the §22.2 change-set relations.
pub fn declare(builder: &mut RegistryBuilder) {
    declare_change_sets(builder);
    declare_change_ops(builder);
}

/// `authored.change_sets @1` (blueprint §22.2).
fn declare_change_sets(builder: &mut RegistryBuilder) {
    builder.declare_relation(
        RelationDecl::new(
            Namespace::Authored,
            "change_sets",
            1,
            Authority::Authored,
            SnapshotClass::Sidecar,
            "One row per proposed change set: the unit that is validated, applied atomically and \
             accepted or rejected as a whole.",
        )
        .stability(Stability::Stable)
        .pk(&["change_set_id"])
        .columns(vec![
            ColumnSpec::key(
                "change_set_id",
                LogicalType::id(),
                "The change set identity.",
            ),
            ColumnSpec::reference(
                "base_revision_id",
                LogicalType::id(),
                "The revision the change set was written against. Two change sets on one base \
                 conflict explicitly rather than merging.",
            ),
            ColumnSpec::label("author", LogicalType::Text, "Who proposed it."),
            ColumnSpec::label("message", LogicalType::Text, "Why."),
            ColumnSpec::payload(
                "created_at",
                LogicalType::Timestamp,
                "When it was proposed.",
            ),
        ]),
    );
}

/// `authored.change_ops @1` (blueprint §22.2, revision 6).
fn declare_change_ops(builder: &mut RegistryBuilder) {
    builder.declare_relation(
        RelationDecl::new(
            Namespace::Authored,
            "change_ops",
            1,
            Authority::Authored,
            SnapshotClass::Sidecar,
            "The operations of a change set, in application order.",
        )
        .stability(Stability::Stable)
        .pk(&["change_set_id", "ordinal"])
        .columns(vec![
            ColumnSpec::key("change_set_id", LogicalType::id(), "The owning change set.")
                .with_fk("authored.change_sets", "change_set_id"),
            ColumnSpec::key(
                "ordinal",
                LogicalType::U32,
                "The operation's position in the change set.",
            ),
            ColumnSpec::label(
                "op",
                LogicalType::enumeration("ChangeOpKind"),
                "What the operation does.",
            ),
            ColumnSpec::reference(
                "relation_id",
                LogicalType::id(),
                "The relation the operation writes. A derived relation is rejected: an \"expected\" \
                 derived fact goes to `provenance.assertions`.",
            )
            .with_fk("reference.schema_relations", "relation_id"),
            ColumnSpec::payload(
                "row_key",
                LogicalType::Ext(ExtensionUse::IndexTuple),
                "The primary key of the affected row.",
            ),
            ColumnSpec::payload(
                "row",
                LogicalType::Struct(vec![
                    ("staged_port", LogicalType::Text, false),
                    ("staged_ordinal", LogicalType::U64, false),
                ]),
                "A pointer into the change set's per-relation staged members, never JSON text \
                 (decision D1). For a rename the staged row carries the entity identity and the \
                 new name and qualified name.",
            ),
            ColumnSpec::payload(
                "precondition",
                LogicalType::Text,
                "An optional expectation about the base row, checked before the operation applies.",
            )
            .optional(),
        ]),
    );
}
