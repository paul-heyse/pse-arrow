// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Located finite rule support; assertion payload schemas are projected from their heads.
use super::{N, RegistryBuilder, S, T, column, enumeration, relation};

pub(super) fn declare(builder: &mut RegistryBuilder) {
    enumeration(
        builder,
        "RuleOutcomeReason",
        [
            "asserted",
            "predicate_false",
            "predicate_unknown",
            "value_conflict",
        ],
    );
    enumeration(
        builder,
        "RuleSupportKind",
        ["delta", "facts", "workspace", "completed", "absence"],
    );
    relation(
        builder,
        N::Inferred,
        "rule_outcomes",
        S::Derived,
        &["rule_id", "head_relation_id", "head_key"],
        vec![
            column("rule_id", T::id()).with_fk("reference.rule_specs", "rule_id"),
            column("head_relation_id", T::id())
                .with_fk("reference.schema_relations", "relation_id"),
            column("head_key", T::native(arrow_schema::DataType::Utf8)),
            column("truth", T::enumeration("TruthValue")),
            column("reason", T::enumeration("RuleOutcomeReason")),
        ],
        "Four-valued per-rule outcome; head keys use the validated reversible key codec.",
    );
    relation(
        builder,
        N::Provenance,
        "rule_support_edges",
        S::Derived,
        &["edge_id"],
        vec![
            column("edge_id", T::id()),
            column("rule_id", T::id()).with_fk("reference.rule_specs", "rule_id"),
            column("assertion_relation_id", T::id())
                .with_fk("reference.schema_relations", "relation_id"),
            column("assertion_id", T::id()),
            column("head_relation_id", T::id())
                .with_fk("reference.schema_relations", "relation_id"),
            column("head_key", T::native(arrow_schema::DataType::Utf8)),
            column("input_port", T::native(arrow_schema::DataType::Utf8)),
            column("input_relation_id", T::id())
                .with_fk("reference.schema_relations", "relation_id"),
            column("input_selection", super::super::publication::member()).optional(),
            column("support_kind", T::enumeration("RuleSupportKind")),
            column("input_key", T::native(arrow_schema::DataType::Utf8)).optional(),
            column("truth", T::enumeration("TruthValue")),
        ],
        "Exact assertion-to-input support; absence names an entire completed binding.",
    );
    relation(
        builder,
        N::Provenance,
        "constructed_supports",
        S::Sidecar,
        &["mapping_id"],
        vec![
            column("mapping_id", T::id()),
            column("output_relation_id", T::id()),
            column("output_key", T::native(arrow_schema::DataType::Utf8)).optional(),
            column("input_port", T::native(arrow_schema::DataType::Utf8)),
            column("input_relation_id", T::id()),
            column("input_selection", super::super::publication::member()).optional(),
            column("support_kind", T::enumeration("RuleSupportKind")),
            column("input_key", T::native(arrow_schema::DataType::Utf8)).optional(),
        ],
        "Actual constructor witnesses. A null output key names the complete input binding scope; Delta selections name exact table versions and revision slices, while transient facts have no invented durable identity.",
    );
}
