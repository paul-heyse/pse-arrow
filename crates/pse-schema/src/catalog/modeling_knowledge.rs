// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Read-only projections of one admitted modeling revision (ADR-0131).
use super::declarations::{column, enumeration, relation};
use crate::{
    builder::RegistryBuilder,
    model::{FieldContract as T, Namespace as N, SnapshotClass as S},
};
use arrow_schema::DataType as D;

fn arena() -> T {
    T::list(
        T::structure(vec![
            T::enumeration("ModelingKnowledgeValueKind").with_name("kind"),
            T::native(D::Boolean).with_name("boolean").optional(),
            T::native(D::Int64).with_name("integer").optional(),
            T::native(D::Float64).with_name("magnitude").optional(),
            T::id().with_name("quantity_type_id").optional(),
            T::id().with_name("canonical_unit_id").optional(),
            T::id().with_name("reference_id").optional(),
            T::id().with_name("type_id").optional(),
            T::native(D::Utf8).with_name("text").optional(),
            T::list(T::native(D::Utf8)).with_name("labels"),
            T::list(T::native(D::UInt32)).with_name("children"),
        ])
        .named("ModelingKnowledgeValueNode"),
    )
}

pub(super) fn declare(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Runtime,
        "modeling_knowledge_names",
        S::Derived,
        &["source_revision", "name"],
        vec![
            column("source_revision", T::hash()),
            column("name", T::native(D::Utf8)),
            column("declaration_id", T::id()).with_identity("declaration"),
        ],
        "Resolved names of the immutable admitted modeling revision. Aliases name the same declaration; this projection introduces no independent authority.",
    );
    enumeration(
        builder,
        "ModelingKnowledgeValueKind",
        [
            "missing",
            "boolean",
            "integer",
            "quantity",
            "coordinate",
            "text",
            "entity",
            "enumeration",
            "identifier",
            "definition",
            "function",
            "row",
            "set",
            "tuple",
            "quantity_type",
            "reference_state",
        ],
    );
    relation(
        builder,
        N::Runtime,
        "modeling_knowledge",
        S::Derived,
        &["source_revision", "owner_id", "row_index", "slot"],
        vec![
            column("source_revision", T::hash()),
            column("owner_id", T::id()).with_identity("declaration"),
            column("row_index", T::nonnegative(i64::MAX)),
            column("slot", T::native(D::Utf8)),
            column("record_kind_id", T::id())
                .with_identity("declaration")
                .optional(),
            column("origin_id", T::id()).with_identity("declaration"),
            column("keys", arena()),
            column("value", arena()),
            column(
                "uncertainty",
                T::structure(vec![
                    T::enumeration("ModelingUncertaintyKind").with_name("kind"),
                    T::native(D::Float64).with_name("magnitude"),
                ])
                .named("ModelingKnowledgeUncertainty"),
            )
            .optional(),
            column("source_id", T::id())
                .with_identity("declaration")
                .optional(),
            column("role_enumeration_id", T::id())
                .with_identity("declaration")
                .optional(),
            column("role_member_id", T::id()).optional(),
            column(
                "lineage",
                T::list(
                    T::structure(vec![
                        T::enumeration("ModelingLineageKind").with_name("kind"),
                        T::id().with_name("target_id"),
                    ])
                    .named("ModelingKnowledgeLineage"),
                ),
            ),
            column("test_only", T::native(D::Boolean)),
        ],
        "Read-only admitted cells of one immutable source revision. Records use their admitted identity, constants their declaration, and table rows their canonical-order index within that revision. Keys and values are post-order typed arenas: children precede parents, the last node is the root. Canonical quantity types accompany numeric values; references are resolved identities, not source spellings. Declared uncertainty and origin source, role, lineage and transitive test-only status are preserved. This derived relation authorizes no scientific read or mutation.",
    );
}
