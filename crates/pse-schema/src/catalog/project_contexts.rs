// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Immutable engineering context selections; UTC instants are signed microseconds.
use super::declarations::{column, canonical_relation as relation};
use super::substrate::NativeIndex;
use crate::{
    RegistryBuilder,
    model::{FieldContract as T, Namespace as N, SnapshotClass as S},
};
use arrow_schema::DataType as D;
/// Indexed context selection and exact correction ancestry.
pub const NATIVE_INDEXES: &[NativeIndex] = &[
    NativeIndex {
        name: "project_context_scope",
        table: "canonical_project_contexts",
        fields: &["project", "scope", "recorded_sequence"],
        unique: false,
    },
    NativeIndex {
        name: "project_context_ancestor",
        table: "canonical_context_ancestry",
        fields: &["descendant", "ancestor"],
        unique: true,
    },
];
pub(super) fn declare(b: &mut RegistryBuilder) {
    let text = || T::native(D::Utf8);
    let uint = || T::native(D::UInt64);
    let instant = || T::native(D::Int64);
    for (name, fields, doc) in [
        (
            "canonical_context_journals",
            vec![column("key", text()), column("sequence", uint())],
            "Project-scoped recorded cut; independent of source and membership generations.",
        ),
        (
            "canonical_project_contexts",
            vec![
                column("key", text()).with_canonical_key(crate::model::CanonicalKeyKind::ProjectContext),
                column("project", text()).with_canonical_key(crate::model::CanonicalKeyKind::Project).canonical_readonly(),
                column("scope", text()).canonical_readonly(),
                column("revision", text()).with_canonical_key(crate::model::CanonicalKeyKind::Revision).canonical_readonly(),
                column("source_identity", text()).canonical_readonly(),
                column("physical_context", text()).canonical_readonly(),
                column("physical_revision", text()).canonical_readonly(),
                column("physical_source_identity", text()).canonical_readonly(),
                column("effective_from_us", instant()).canonical_readonly(),
                column("effective_to_us", instant()).canonical_readonly().optional(),
                column("recorded_sequence", uint()).canonical_readonly(),
                column("corrects", text()).with_canonical_key(crate::model::CanonicalKeyKind::ProjectContext).canonical_readonly().optional(),
                column("deleted", T::native(D::Boolean)).canonical_readonly(),
                column("request", T::native(D::Binary)).canonical_readonly(),
            ],
            "Immutable engineering context with UTC half-open effective interval and exact admitted source reference; no copied scientific values.",
        ),
        (
            "canonical_context_ancestry",
            vec![
                column("key", text()).with_canonical_key(crate::model::CanonicalKeyKind::ContextAncestry),
                column("descendant", text()).canonical_readonly(),
                column("ancestor", text()).canonical_readonly(),
            ],
            "Immutable transitive correction ancestry. A correction dominates ancestors only inside its own effective interval.",
        ),
    ] {
        relation(b, N::Runtime, name, S::Derived, &["key"], fields, doc);
    }
}
