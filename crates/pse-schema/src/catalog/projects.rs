// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Project organization and exact research lineage; canonical source remains authoritative.
use super::declarations::{column, relation};
use super::substrate::NativeIndex;
use crate::{
    RegistryBuilder,
    model::{FieldContract as T, Namespace as N, SnapshotClass as S},
};
use arrow_schema::DataType as D;
/// Access paths for actual bounded project operations.
pub const NATIVE_INDEXES: &[NativeIndex] = &[
    NativeIndex {name:"project_branch_name",table:"canonical_project_branches",fields:&["project","name"],unique:true},
    NativeIndex {name:"project_bookmark_name",table:"canonical_project_bookmarks",fields:&["project","name"],unique:true},
    NativeIndex {name:"project_bookmark_order",table:"canonical_project_bookmarks",fields:&["project","key"],unique:false},
    NativeIndex {name:"project_savepoint_name",table:"canonical_project_savepoints",fields:&["draft","name"],unique:true},
    NativeIndex {name:"project_savepoint_order",table:"canonical_project_savepoints",fields:&["draft","key"],unique:false},
    NativeIndex {name:"project_ancestry",table:"canonical_project_ancestry",fields:&["revision","parent"],unique:true},
    NativeIndex {name:"project_version_order",table:"canonical_project_draft_versions",fields:&["draft","key"],unique:false},
    NativeIndex {name:"draft_diagnostic_order",table:"canonical_draft_diagnostics",fields:&["version","key"],unique:false},
    NativeIndex {
        name: "project_research_run",
        table: "canonical_research_references",
        fields: &["run", "key"],
        unique: false,
    },
    NativeIndex {
        name: "project_branch_order",
        table: "canonical_project_branches",
        fields: &["project", "key"],
        unique: false,
    },
    NativeIndex {
        name: "project_branch_problem",
        table: "canonical_project_branches",
        fields: &["problem"],
        unique: true,
    },
    NativeIndex {
        name: "project_draft_order",
        table: "canonical_project_drafts",
        fields: &["branch", "key"],
        unique: false,
    },
    NativeIndex {
        name: "project_research_order",
        table: "canonical_research_references",
        fields: &["project", "key"],
        unique: false,
    },
];
pub(super) fn declare(b: &mut RegistryBuilder) {
    let text = || T::native(D::Utf8);
    let uint = || T::native(D::UInt64);
    let bytes = || T::native(D::Binary);
    for (name, fields, doc) in [
        (
            "canonical_projects",
            vec![
                column("key", text()),
                column("title", text()),
                column("deleted", T::native(D::Boolean)),
                column("generation", uint()),
            ],
            "Project organization only; source heads remain in canonical_problems.",
        ),
        (
            "canonical_project_branches",
            vec![
                column("key", text()),
                column("project", text()),
                column("problem", text()),
                column("name", text()),
                column("origin_revision", text()).optional(),
                column("generation", uint()),
            ],
            "One independent canonical source problem per writable project branch. Origin is ancestry, never a second head.",
        ),
        (
            "canonical_project_drafts",
            vec![
                column("key", text()),
                column("project", text()),
                column("branch", text()),
                column("generation", uint()),
                column("version", text()),
                column("deleted", T::native(D::Boolean)),
            ],
            "Generation-fenced draft head selecting one immutable original raw version; deletion changes visibility only.",
        ),
        (
            "canonical_project_draft_versions",
            vec![column("key", text()), column("draft", text()), column("project", text()), column("branch", text()), column("base_revision", text()), column("generation", uint()), column("content", bytes()), column("artifact", text()).optional()],
            "Immutable original raw draft version. Opaque artifacts retain larger malformed bodies; saving grants no scientific admission.",
        ),
        (
            "canonical_project_savepoints",
            vec![column("key", text()), column("project", text()), column("draft", text()), column("version", text()), column("name", text()), column("deleted", T::native(D::Boolean))],
            "Named exact draft version; soft deletion does not release scientific closure.",
        ),
        (
            "canonical_project_bookmarks",
            vec![column("key", text()), column("project", text()), column("revision", text()), column("name", text())],
            "Presentation-only exact source bookmark, retained through the canonical source owner.",
        ),
        (
            "canonical_project_ancestry",
            vec![column("key", text()), column("revision", text()), column("parent", text())],
            "Immutable additional accepted source ancestry, independent of scientific object identity.",
        ),
        (
            "canonical_draft_diagnostics",
            vec![column("key", text()), column("version", text()), column("message", text()), column("admitted", T::native(D::Boolean))],
            "Version-linked local admission diagnostics; no executable authority is granted by this record.",
        ),
        (
            "canonical_project_operations",
            vec![
                column("key", text()),
                column("kind", text()),
                column("project", text()),
                column("branch", text()).optional(),
                column("request", bytes()),
                column("revision", text()).optional(),
                column("generation", uint()),
            ],
            "Immutable exact operation acknowledgement, including original input digest/coordinates; settlement never infers success from the current head.",
        ),
        (
            "canonical_research_references",
            vec![
                column("key", text()),
                column("project", text()),
                column("revision", text()),
                column("artifact", text()).optional(),
                column("run", text()).optional(),
                column("attempt", text()).optional(),
                column("request", bytes()),
            ],
            "Explicit source/result lineage reference. Source roots and selected artifact obligations preserve the exact original closure; no numerical value is copied as a new scientific authority.",
        ),
    ] {
        relation(b, N::Runtime, name, S::Derived, &["key"], fields, doc);
    }
}
