// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Transient typed algorithm-to-native-plan source correspondence.
use super::{N, RegistryBuilder, S, T, column, relation};

pub(super) fn declare(builder: &mut RegistryBuilder) {
    relation(
        builder,
        N::Provenance,
        "property_read_occurrences",
        S::Sidecar,
        &["read_id"],
        vec![
            column("read_id", T::id()),
            column("source_id", T::id()),
            column("symbol_decl_id", T::id()),
            column("read_node_id", T::native(arrow_schema::DataType::UInt64)),
            column(
                "guard_predicate_id",
                T::native(arrow_schema::DataType::UInt64),
            )
            .optional(),
        ],
        "Transient typed syntax reads and their exact branch guards. Native joins select actual property mappings; this is not an assertion that a read denotes a property.",
    );
    relation(
        builder,
        N::Provenance,
        "node_rewrites",
        S::Sidecar,
        &["input_node_id"],
        vec![
            column("input_node_id", T::native(arrow_schema::DataType::UInt64)),
            column("output_node_id", T::native(arrow_schema::DataType::UInt64)),
        ],
        "Transient actual MathIR graph-import mapping, scoped to one exact graph/family argument role. Native joins apply this mapping to carrier columns. It cannot replace root-environment-specific canonicalization correspondence.",
    );
}
