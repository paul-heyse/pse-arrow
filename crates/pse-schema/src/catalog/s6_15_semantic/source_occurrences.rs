// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Parser occurrences and native binding results preserve source identity and scope.
use super::{ExtensionUse, N, RegistryBuilder, S, T, column, enumeration, relation};

pub(super) fn declare(builder: &mut RegistryBuilder) {
    enumeration(
        builder,
        "SourceBindingKind",
        [
            "symbol",
            "equation",
            "port",
            "domain",
            "parameter",
            "feature",
            "entity",
            "unit",
        ],
    );
    relation(
        builder,
        N::Normalized,
        "source_occurrences",
        S::Sidecar,
        &["document_id", "field_path", "ordinal"],
        vec![
            column("document_id", T::id()).with_fk("authored.documents", "document_id"),
            column("field_path", T::native(arrow_schema::DataType::Utf8)),
            column("ordinal", T::nonnegative(i64::MAX)),
            column("source_relation_id", T::id())
                .with_fk("reference.schema_relations", "relation_id"),
            column("owner_kind", T::enumeration("ExpressionOwnerKind")),
            column("owner_id", T::id()),
            column("lookup_name", T::native(arrow_schema::DataType::Utf8)).optional(),
            column("source_span", T::extended(ExtensionUse::SourceSpan)),
        ],
        "Parser field roots (lookup_name absent), named path segments and qualified prefixes. Field paths and source spans locate syntax; native source-key projection binds the complete declared key of the retained source row. Lexical interpretation remains in the source AST.",
    );
    relation(
        builder,
        N::Normalized,
        "resolved_source_occurrences",
        S::Sidecar,
        &["document_id", "field_path", "ordinal", "match_ordinal"],
        vec![
            column("document_id", T::id()),
            column("field_path", T::native(arrow_schema::DataType::Utf8)),
            column("ordinal", T::nonnegative(i64::MAX)),
            column("match_ordinal", T::nonnegative(i64::MAX)),
            column("source_key", T::row_key()),
            column("kind", T::enumeration("SourceBindingKind")),
            column("owner_template_id", T::id()).optional(),
            column("semantic_id", T::id()).optional(),
            column("name", T::native(arrow_schema::DataType::Utf8)),
        ],
        "Native occurrence/declaration joins retain every matching declaration, including ambiguous matches; syntax binding must select the declared lexical/global/template scope.",
    );
}
