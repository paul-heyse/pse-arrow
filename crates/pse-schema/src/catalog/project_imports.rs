// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Immutable logical-project import identity and monotonic completed membership.
use super::declarations::{column, canonical_relation as relation};
use super::substrate::NativeIndex;
use crate::{
    RegistryBuilder,
    model::{FieldContract as T, Namespace as N, SnapshotClass as S},
};
use arrow_schema::DataType as D;
/// Unique receiving project and problem attribution for availability guards.
pub const NATIVE_INDEXES: &[NativeIndex] = &[
    NativeIndex {
        name: "project_import_project",
        table: "canonical_project_imports",
        fields: &["project"],
        unique: true,
    },
    NativeIndex {
        name: "project_import_problem",
        table: "canonical_project_imports",
        fields: &["problem"],
        unique: true,
    },
];
pub(super) fn declare(b: &mut RegistryBuilder) {
    let text = || T::native(D::Utf8);
    let bytes = || T::native(D::Binary);
    relation(
        b,
        N::Runtime,
        "canonical_project_imports",
        S::Derived,
        &["key"],
        vec![
            column("key", text()).with_canonical_key(crate::model::CanonicalKeyKind::ProjectImport),
            column("issuing_store", text()).canonical_readonly(),
            column("bundle_digest", text()).canonical_readonly(),
            column("project", text()).with_canonical_key(crate::model::CanonicalKeyKind::Project).canonical_readonly(),
            column("branch", text()).with_canonical_key(crate::model::CanonicalKeyKind::ProjectBranch).canonical_readonly(),
            column("problem", text()).with_canonical_key(crate::model::CanonicalKeyKind::Problem).canonical_readonly(),
            column("revision", text()).with_canonical_key(crate::model::CanonicalKeyKind::Revision).canonical_readonly(),
            column("origin", bytes()).canonical_readonly(),
            column("request", bytes()).canonical_readonly(),
            column("artifact", bytes()).with_canonical_assertion("$before=NONE OR $before=$value").optional(),
            column("retired_artifacts", bytes()).optional(),
            column("complete", T::native(D::Boolean)).with_canonical_default("false").with_canonical_assertion("($before=NONE AND $value=false) OR $before=$value OR ($before=false AND $value=true AND $this.artifact!=NONE)"),
        ],
        "Original immutable fresh-authority import intent; source admission is pending until exact copied evidence artifact ownership finishes. Historical terminals remain origin evidence only.",
    );
}
