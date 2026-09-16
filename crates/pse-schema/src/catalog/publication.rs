// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Target Delta control facts; exact members replace custom manifest membership.
use super::declarations::{column, enumeration, relation};
use crate::{
    RegistryBuilder,
    model::{FieldContract as T, Namespace as N, SnapshotClass as S},
};

pub(super) fn declare(builder: &mut RegistryBuilder) {
    enumeration(
        builder,
        "PublicationKind",
        ["source", "model", "case", "problem", "run"],
    );
    relation(
        builder,
        N::Runtime,
        "publications",
        S::Sidecar,
        &["workspace_id"],
        vec![
            column("workspace_id", T::id()),
            column("publication_id", T::id()),
            column("parent_publication_id", T::id()).optional(),
            column("attempt_id", T::id()),
            column("kind", T::enumeration("PublicationKind")),
            column("inputs", T::list(member())),
            column("members", T::list(member())),
        ],
        "ADR-0068: one Delta control row atomically selects exact relation versions and slices; its Delta version identifies a publication root.",
    );
}
pub(super) fn member() -> T {
    T::structure(vec![
        T::native(arrow_schema::DataType::Utf8)
            .with_name("catalog_name")
            .with_nullable(false),
        T::native(arrow_schema::DataType::Utf8)
            .with_name("schema_name")
            .with_nullable(false),
        T::native(arrow_schema::DataType::Utf8)
            .with_name("table_name")
            .with_nullable(false),
        T::id().with_name("relation_id").with_nullable(false),
        T::native(arrow_schema::DataType::UInt32)
            .with_name("relation_version")
            .with_nullable(false),
        T::hash()
            .with_name("contract_fingerprint")
            .with_nullable(false),
        T::native(arrow_schema::DataType::Utf8)
            .with_name("table_uri")
            .with_nullable(false),
        T::native(arrow_schema::DataType::UInt64)
            .with_name("delta_version")
            .with_nullable(false),
        T::native(arrow_schema::DataType::Utf8)
            .with_name("revision_column")
            .with_nullable(true),
        T::id().with_name("revision_id").with_nullable(true),
    ])
}
