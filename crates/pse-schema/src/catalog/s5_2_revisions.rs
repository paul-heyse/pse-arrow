// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! §5.2 revision.

use super::declarations::{column, relation};
use crate::builder::RegistryBuilder;
use crate::model::{FieldContract as T, Namespace as N, SnapshotClass as S};

/// Declares the §5.2 revision contracts.
pub fn declare(builder: &mut RegistryBuilder) {
    declare_authored_model_revisions(builder);
    declare_authored_case_revisions(builder);
}

fn declare_authored_model_revisions(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Authored,
        "model_revisions",
        S::Sidecar,
        &["model_revision_id"],
        vec![
            column("model_revision_id", T::id()),
            column("parent_revision_id", T::id()).optional(),
            column("snapshot_id", T::hash()),
            column(
                "created_at",
                T::native(crate::model::extension::timestamp_storage()),
            ),
            column("author", T::native(arrow_schema::DataType::Utf8)),
            column("message", T::native(arrow_schema::DataType::Utf8)),
        ],
        "blueprint §5.2 revision: model_revisions.",
    );
}

fn declare_authored_case_revisions(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Authored,
        "case_revisions",
        S::Sidecar,
        &["case_revision_id"],
        vec![
            column("case_revision_id", T::id()),
            column("model_revision_id", T::id()),
            column("parent_case_id", T::id()).optional(),
            column("snapshot_id", T::hash()),
            column(
                "created_at",
                T::native(crate::model::extension::timestamp_storage()),
            ),
            column("author", T::native(arrow_schema::DataType::Utf8)),
            column("message", T::native(arrow_schema::DataType::Utf8)),
        ],
        "blueprint §5.2 revision: case_revisions.",
    );
}
