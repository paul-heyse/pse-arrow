// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Immutable engineering context selections; UTC instants are signed microseconds.
use super::declarations::{column, relation};
use super::substrate::NativeIndex;
use crate::{RegistryBuilder, model::{FieldContract as T, Namespace as N, SnapshotClass as S}};
use arrow_schema::DataType as D;
/// Indexed context selection and exact correction ancestry.
pub const NATIVE_INDEXES: &[NativeIndex] = &[
    NativeIndex { name: "project_context_scope", table: "canonical_project_contexts", fields: &["project", "scope", "recorded_sequence"], unique: false },
    NativeIndex { name: "project_context_ancestor", table: "canonical_context_ancestry", fields: &["descendant", "ancestor"], unique: true },
];
pub(super) fn declare(b: &mut RegistryBuilder) {
    let text = || T::native(D::Utf8);
    let uint = || T::native(D::UInt64);
    let instant = || T::native(D::Int64);
    for (name, fields, doc) in [
        ("canonical_context_journals", vec![column("key", text()), column("sequence", uint())], "Project-scoped recorded cut; independent of source and membership generations."),
        ("canonical_project_contexts", vec![
            column("key", text()), column("project", text()), column("scope", text()),
            column("revision", text()), column("physical_context", text()),
            column("effective_from_us", instant()), column("effective_to_us", instant()).optional(),
            column("recorded_sequence", uint()), column("corrects", text()).optional(),
            column("deleted", T::native(D::Boolean)), column("request", T::native(D::Binary)),
        ], "Immutable engineering context with UTC half-open effective interval and exact admitted source reference; no copied scientific values."),
        ("canonical_context_ancestry", vec![column("key", text()), column("descendant", text()), column("ancestor", text())], "Immutable transitive correction ancestry. A correction dominates ancestors only inside its own effective interval."),
    ] { relation(b, N::Runtime, name, S::Derived, &["key"], fields, doc); }
}
