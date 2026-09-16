// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Method summaries must agree with their complete typed provision and dependency contracts.
use super::{E, P, RegistryBuilder, anti, eq, filter, literal, project, scan};
use crate::model::{EmptyListPolicy, InvariantKind, NullListPolicy};

pub(super) fn declare(builder: &mut RegistryBuilder) {
    for (field, relation, target, kind) in [
        (
            "provides",
            "reference.method_provisions",
            "property_kind_id",
            None,
        ),
        (
            "requires",
            "reference.method_dependencies",
            "target_id",
            Some("property"),
        ),
    ] {
        let listed = P::Unnest {
            input: Box::new(scan("reference.method_specs", "methods")),
            column: (field).into(),
            value_name: ("listed_property").into(),
            null_list: NullListPolicy::Reject,
            empty_list: EmptyListPolicy::NoMembers,
        };
        let actual = scan(relation, "actual");
        let actual = if let Some(kind) = kind {
            filter(actual, eq(E::col("target_kind"), literal(kind)))
        } else {
            actual
        };
        let actual = project(
            actual,
            vec![
                ("actual_method", E::col("method_id")),
                ("actual_property", E::col(target)),
            ],
        );
        let absent = anti(
            listed.clone(),
            actual.clone(),
            vec![
                ("methods.method_id", "actual_method"),
                ("listed_property", "actual_property"),
            ],
        );
        let extra = anti(
            actual,
            listed,
            vec![
                ("actual_method", "methods.method_id"),
                ("actual_property", "listed_property"),
            ],
        );
        let mismatch = P::Union(vec![
            project(absent, vec![("method_id", E::col("methods.method_id"))]),
            project(extra, vec![("method_id", E::col("actual_method"))]),
        ]);
        super::super::inv::declare(
            builder,
            "reference.method_specs",
            &format!("{field}_matches_complete_contract"),
            InvariantKind::Check,
            &["method_id"],
            mismatch,
            "Method property inventories are exact set projections of the complete provision/dependency rows; matching IDs alone do not establish that projection.",
        );
    }
}
