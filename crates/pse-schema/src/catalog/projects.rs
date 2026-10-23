// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Project organization and exact research lineage; canonical source remains authoritative.
use super::declarations::{column, relation};
use crate::{RegistryBuilder, model::{FieldContract as T, Namespace as N, SnapshotClass as S}};
use arrow_schema::DataType as D;
use super::substrate::NativeIndex;
/// Access paths for actual bounded project operations.
pub const NATIVE_INDEXES: &[NativeIndex] = &[
    NativeIndex { name: "project_research_run", table: "canonical_research_references", fields: &["run", "key"], unique: false },
    NativeIndex { name: "project_branch_order", table: "canonical_project_branches", fields: &["project", "key"], unique: false },
    NativeIndex { name: "project_branch_problem", table: "canonical_project_branches", fields: &["problem"], unique: true },
    NativeIndex { name: "project_draft_order", table: "canonical_project_drafts", fields: &["branch", "key"], unique: false },
    NativeIndex { name: "project_research_order", table: "canonical_research_references", fields: &["project", "key"], unique: false },
];
pub(super) fn declare(b: &mut RegistryBuilder) {
    let text = || T::native(D::Utf8);
    let uint = || T::native(D::UInt64);
    let bytes = || T::native(D::Binary);
    for (name, fields, doc) in [
        ("canonical_projects", vec![column("key",text()),column("title",text()),column("generation",uint())], "Project organization only; source heads remain in canonical_problems."),
        ("canonical_project_branches", vec![column("key",text()),column("project",text()),column("problem",text()),column("origin_revision",text()).optional(),column("generation",uint())], "One independent canonical source problem per writable project branch. Origin is ancestry, never a second head."),
        ("canonical_project_drafts", vec![column("key",text()),column("project",text()),column("branch",text()),column("base_revision",text()),column("generation",uint()),column("content",bytes()),column("artifact",text()).optional()], "Original raw draft bytes, possibly malformed or empty. Larger bodies have an owned opaque artifact; content then contains no body. Saving grants no scientific admission."),
        ("canonical_project_operations", vec![column("key",text()),column("kind",text()),column("project",text()),column("branch",text()).optional(),column("request",bytes()),column("revision",text()).optional(),column("generation",uint())], "Immutable exact operation acknowledgement, including original input digest/coordinates; settlement never infers success from the current head."),
        ("canonical_research_references", vec![column("key",text()),column("project",text()),column("revision",text()),column("artifact",text()).optional(),column("run",text()).optional(),column("attempt",text()).optional(),column("request",bytes())], "Explicit source/result lineage reference. Source roots and selected artifact obligations preserve the exact original closure; no numerical value is copied as a new scientific authority."),
    ] { relation(b,N::Runtime,name,S::Derived,&["key"],fields,doc); }
}
