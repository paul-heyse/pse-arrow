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
    relation(
        builder,
        N::Provenance,
        "algorithm_source_occurrences",
        S::Sidecar,
        &[
            "output_relation_id",
            "constructed_row_ordinal",
            "source_port",
            "source_relation_id",
            "source_key",
        ],
        vec![
            column("output_relation_id", T::id())
                .with_fk("reference.schema_relations", "relation_id"),
            column(
                "constructed_row_ordinal",
                T::native(arrow_schema::DataType::UInt64),
            ),
            column("source_port", T::native(arrow_schema::DataType::Utf8)),
            column("source_relation_id", T::id())
                .with_fk("reference.schema_relations", "relation_id"),
            column("source_key", T::native(arrow_schema::DataType::Utf8)),
        ],
        "Transient occurrences in retained generated algorithm output batches. The ordinal is construction bookkeeping, never a semantic key. The port selects one exact immutable source role; a relation identity alone cannot distinguish two roles using that declaration. Native plans calculate output keys from actual output columns and join source keys to exact bound sources before producing support. This declaration does not require publication of the transient batch.",
    );
}
