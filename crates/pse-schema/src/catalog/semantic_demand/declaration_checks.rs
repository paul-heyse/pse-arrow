// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Complete provision and dependency contracts determine native inventory projections.
use super::RegistryBuilder;
use crate::model::InvariantKind;
pub(super) fn declare(builder: &mut RegistryBuilder) {
    for (field, relation, target, predicate) in [
        (
            "provides",
            "reference.method_provisions",
            "property_kind_id",
            "TRUE",
        ),
        (
            "requires",
            "reference.method_dependencies",
            "target_id",
            "target_kind = 'property'",
        ),
    ] {
        super::super::inv::declare(
            builder,
            "reference.method_specs",
            &format!("{field}_matches_complete_contract"),
            InvariantKind::Check,
            &["method_id"],
            format!(
                "WITH listed AS (SELECT method_id, unnest({field}) AS property_id FROM reference.method_specs),
                actual AS (SELECT method_id, {target} AS property_id FROM {relation} WHERE {predicate})
                SELECT l.method_id FROM listed l WHERE NOT EXISTS (SELECT 1 FROM actual a WHERE a.method_id = l.method_id AND a.property_id = l.property_id)
                UNION SELECT a.method_id FROM actual a WHERE NOT EXISTS (SELECT 1 FROM listed l WHERE a.method_id = l.method_id AND a.property_id = l.property_id)",
            ),
            &["reference.method_specs", relation],
            "Method property inventories are exact set projections of the complete provision/dependency rows; matching IDs alone do not establish that projection.",
        );
    }
}
