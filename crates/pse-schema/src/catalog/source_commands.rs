// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Typed request relations consumed by finite native source operations.
use super::declarations::{column, relation};
use crate::{
    RegistryBuilder,
    model::{FieldContract as T, Namespace as N, SnapshotClass as S},
};

pub(super) fn declare(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Authored,
        "document_edits",
        S::Sidecar,
        &["document_id"],
        vec![
            column("document_id", T::id()),
            column("path", T::native(arrow_schema::DataType::Utf8)),
            column("before", T::native(arrow_schema::DataType::Utf8)),
            column("after", T::native(arrow_schema::DataType::Utf8)),
        ],
        "Exact document before-images and replacements for a native source edit.",
    );
    relation(
        builder,
        N::Authored,
        "rename_requests",
        S::Sidecar,
        &["entity_id"],
        vec![
            column("entity_id", T::id()),
            column("expected_name", T::native(arrow_schema::DataType::Utf8)),
            column("new_name", T::native(arrow_schema::DataType::Utf8)),
        ],
        "Identity-bound rename of one actual source entity with an exact expected name.",
    );
}
