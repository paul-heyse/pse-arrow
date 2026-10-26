// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Project organization and exact research lineage; canonical source remains authoritative.
use super::declarations::{canonical_relation as relation, column};
use super::substrate::NativeIndex;
use crate::{
    RegistryBuilder,
    model::{FieldContract as T, Namespace as N, SnapshotClass as S},
};
use arrow_schema::DataType as D;
/// Access paths for actual bounded project operations.
pub const NATIVE_INDEXES: &[NativeIndex] = &[
    NativeIndex {
        name: "project_branch_name",
        table: "canonical_project_branches",
        fields: &["project", "name"],
        unique: true,
    },
    NativeIndex {
        name: "project_bookmark_name",
        table: "canonical_project_bookmarks",
        fields: &["project", "name"],
        unique: true,
    },
    NativeIndex {
        name: "project_bookmark_order",
        table: "canonical_project_bookmarks",
        fields: &["project", "key"],
        unique: false,
    },
    NativeIndex {
        name: "project_savepoint_name",
        table: "canonical_project_savepoints",
        fields: &["draft", "name"],
        unique: true,
    },
    NativeIndex {
        name: "project_savepoint_order",
        table: "canonical_project_savepoints",
        fields: &["draft", "key"],
        unique: false,
    },
    NativeIndex {
        name: "project_ancestry",
        table: "canonical_project_ancestry",
        fields: &["revision", "parent"],
        unique: true,
    },
    NativeIndex {
        name: "project_version_order",
        table: "canonical_project_draft_versions",
        fields: &["draft", "key"],
        unique: false,
    },
    NativeIndex {
        name: "draft_diagnostic_order",
        table: "canonical_draft_diagnostics",
        fields: &["version", "key"],
        unique: false,
    },
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
    super::declarations::enumeration(
        b,
        "ProjectMetadataKind",
        [
            "projects",
            "drafts",
            "bookmarks",
            "savepoints",
            "diagnostics",
        ],
    );
    super::declarations::enumeration(
        b,
        "ProjectChangeImpact",
        ["source", "value", "structural", "unknown"],
    );
    super::declarations::enumeration(
        b,
        "ProjectMergeChoice",
        ["target", "proposal", "base", "delete"],
    );
    super::declarations::enumeration(
        b,
        "ProjectMergeConflictKind",
        ["delete_edit", "conflicting_addition", "different_edits"],
    );
    super::declarations::enumeration(b, "ProjectMergeMode", ["merge", "undo"]);
    super::declarations::enumeration(
        b,
        "PreparationProductKind",
        [
            "selected_admission",
            "body",
            "encoded_body",
            "frontier",
            "basis",
        ],
    );
    super::declarations::enumeration(
        b,
        "PreparationEligibility",
        ["compatible", "incompatible", "unchecked"],
    );

    let text = || T::native(D::Utf8);
    let uint = || T::native(D::UInt64);
    let bytes = || T::native(D::Binary);
    for (name, fields, doc) in [
        (
            "canonical_projects",
            vec![
                column("key", text()).with_canonical_key(crate::model::CanonicalKeyKind::Project),
                column("title", text()),
                column("deleted", T::native(D::Boolean)),
                column("generation", uint()),
            ],
            "Project organization only; source heads remain in canonical_problems.",
        ),
        (
            "canonical_project_branches",
            vec![
                column("key", text()).with_canonical_key(crate::model::CanonicalKeyKind::ProjectBranch),
                column("project", text()).with_canonical_key(crate::model::CanonicalKeyKind::Project).canonical_readonly(),
                column("problem", text()).with_canonical_key(crate::model::CanonicalKeyKind::Problem).canonical_readonly(),
                column("name", text()),
                column("origin_revision", text()).with_canonical_key(crate::model::CanonicalKeyKind::Revision).canonical_readonly().optional(),
                column("generation", uint()),
            ],
            "One independent canonical source problem per writable project branch. Origin is ancestry, never a second head.",
        ),
        (
            "canonical_project_drafts",
            vec![
                column("key", text()).with_canonical_key(crate::model::CanonicalKeyKind::ProjectDraft),
                column("project", text()).with_canonical_key(crate::model::CanonicalKeyKind::Project),
                column("branch", text()).with_canonical_key(crate::model::CanonicalKeyKind::ProjectBranch),
                column("generation", uint()),
                column("version", text()).with_canonical_key(crate::model::CanonicalKeyKind::DraftVersion),
                column("deleted", T::native(D::Boolean)),
            ],
            "Generation-fenced draft head selecting one immutable original raw version; deletion changes visibility only.",
        ),
        (
            "canonical_project_draft_versions",
            vec![
                column("key", text()).with_canonical_key(crate::model::CanonicalKeyKind::DraftVersion),
                column("draft", text()).with_canonical_key(crate::model::CanonicalKeyKind::ProjectDraft).canonical_readonly(),
                column("project", text()).with_canonical_key(crate::model::CanonicalKeyKind::Project).canonical_readonly(),
                column("branch", text()).with_canonical_key(crate::model::CanonicalKeyKind::ProjectBranch).canonical_readonly(),
                column("base_revision", text()).with_canonical_key(crate::model::CanonicalKeyKind::Revision).canonical_readonly(),
                column("generation", uint()).canonical_readonly(),
                column("content", bytes()).with_canonical_assertion("$before=NONE OR $before=$value OR ($this.retired=true AND bytes::len($value)=0)"),
                column("artifact", text()).with_canonical_key(crate::model::CanonicalKeyKind::Artifact).canonical_readonly().optional(),
                column("retired", T::native(D::Boolean)).with_canonical_default("false").with_canonical_assertion("($before=NONE AND $value=false) OR ($before=$value) OR ($before=false AND $value=true AND bytes::len($this.content)=0)"),
            ],
            "Immutable original raw draft version. Opaque artifacts retain larger malformed bodies; saving grants no scientific admission.",
        ),
        (
            "canonical_project_savepoints",
            vec![
                column("key", text()).with_canonical_key(crate::model::CanonicalKeyKind::ProjectSavepoint),
                column("project", text()).with_canonical_key(crate::model::CanonicalKeyKind::Project),
                column("draft", text()).with_canonical_key(crate::model::CanonicalKeyKind::ProjectDraft),
                column("version", text()).with_canonical_key(crate::model::CanonicalKeyKind::DraftVersion),
                column("name", text()),
                column("deleted", T::native(D::Boolean)),
            ],
            "Named exact draft version; soft deletion does not release scientific closure.",
        ),
        (
            "canonical_project_bookmarks",
            vec![
                column("key", text()).with_canonical_key(crate::model::CanonicalKeyKind::ProjectBookmark),
                column("project", text()).with_canonical_key(crate::model::CanonicalKeyKind::Project),
                column("revision", text()).with_canonical_key(crate::model::CanonicalKeyKind::Revision),
                column("name", text()),
            ],
            "Presentation-only exact source bookmark, retained through the canonical source owner.",
        ),
        (
            "canonical_project_ancestry",
            vec![
                column("key", text()).with_canonical_key(crate::model::CanonicalKeyKind::ProjectAncestry),
                column("revision", text()).with_canonical_key(crate::model::CanonicalKeyKind::Revision).canonical_readonly(),
                column("parent", text()).with_canonical_key(crate::model::CanonicalKeyKind::Revision).canonical_readonly(),
            ],
            "Immutable additional accepted source ancestry, independent of scientific object identity.",
        ),
        (
            "canonical_draft_diagnostics",
            vec![
                column("key", text()).with_canonical_key(crate::model::CanonicalKeyKind::DraftDiagnostic),
                column("version", text()).with_canonical_key(crate::model::CanonicalKeyKind::DraftVersion).canonical_readonly(),
                column("message", text()).canonical_readonly(),
                column("admitted", T::native(D::Boolean)).canonical_readonly(),
            ],
            "Version-linked local admission diagnostics; no executable authority is granted by this record.",
        ),
        (
            "canonical_project_operations",
            vec![
                column("key", text()).with_canonical_key(crate::model::CanonicalKeyKind::Operation),
                column("kind", text()).canonical_readonly(),
                column("project", text()).with_canonical_key(crate::model::CanonicalKeyKind::Project).canonical_readonly(),
                column("branch", text()).with_canonical_key(crate::model::CanonicalKeyKind::ProjectBranch).canonical_readonly().optional(),
                column("request", bytes()).canonical_readonly(),
                column("revision", text()).with_canonical_key(crate::model::CanonicalKeyKind::Revision).canonical_readonly().optional(),
                column("generation", uint()).canonical_readonly(),
            ],
            "Immutable exact operation acknowledgement, including original input digest/coordinates; settlement never infers success from the current head.",
        ),
        (
            "canonical_research_references",
            vec![
                column("key", text()).with_canonical_key(crate::model::CanonicalKeyKind::ResearchReference),
                column("project", text()).with_canonical_key(crate::model::CanonicalKeyKind::Project).canonical_readonly(),
                column("revision", text()).with_canonical_key(crate::model::CanonicalKeyKind::Revision).canonical_readonly(),
                column("artifact", text()).with_canonical_key(crate::model::CanonicalKeyKind::Artifact).canonical_readonly().optional(),
                column("run", text()).with_canonical_key(crate::model::CanonicalKeyKind::Run).canonical_readonly().optional(),
                column("attempt", text()).with_canonical_key(crate::model::CanonicalKeyKind::Attempt).canonical_readonly().optional(),
                column("request", bytes()).canonical_readonly(),
            ],
            "Explicit source/result lineage reference. Source roots and selected artifact obligations preserve the exact original closure; no numerical value is copied as a new scientific authority.",
        ),
    ] {
        relation(b, N::Runtime, name, S::Derived, &["key"], fields, doc);
    }
}
